// SPDX-License-Identifier: GPL-3.0-or-later
//! Agent proposals (ADR-0016): the `proposals/` table and its records.
//!
//! The table is append-only and holds one row per state change of a proposal: `seq` 0
//! creates it (with its message and changes), and each decision appends a row with the next
//! `seq`. Reads fold the rows by `proposal_id`. Nothing is ever deleted except by
//! [`crate::Store::prune`], which drops decided proposals older than its cutoff.

use std::collections::BTreeMap;
use std::sync::{Arc, LazyLock};

use deltalake::arrow::array::{Array, ArrayRef, Int64Array, RecordBatch, StringArray};
use deltalake::arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::cell::CellKind;
use crate::error::{Error, Result};
use crate::schema::conform;

/// Where a proposal stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProposalStatus {
    Open,
    /// Applied by a human's commit (`accepted_version`).
    Accepted,
    Rejected,
    /// Taken back by the agent that made it.
    Withdrawn,
}

impl ProposalStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            ProposalStatus::Open => "open",
            ProposalStatus::Accepted => "accepted",
            ProposalStatus::Rejected => "rejected",
            ProposalStatus::Withdrawn => "withdrawn",
        }
    }

    pub fn parse(s: &str) -> Option<ProposalStatus> {
        [
            ProposalStatus::Open,
            ProposalStatus::Accepted,
            ProposalStatus::Rejected,
            ProposalStatus::Withdrawn,
        ]
        .into_iter()
        .find(|k| k.as_str() == s)
    }
}

/// A new cell in a proposal. Its id is assigned when the proposal is made, so the cell keeps
/// it once accepted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProposedCell {
    pub id: String,
    pub kind: CellKind,
    pub source: String,
    #[serde(default)]
    pub attrs: Value,
}

/// One change of a proposal. `base_rev` is the revision of the cell the agent looked at; if
/// the cell has changed since, the proposal is stale (never merged silently).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "lowercase")]
pub enum ProposedChange {
    /// A new cell after `after` (a cell id; `None`: at the start of the document).
    Insert { after: Option<String>, cell: ProposedCell },
    /// New source and/or attributes for a cell.
    Modify {
        id: String,
        base_rev: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        source: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        attrs: Option<Value>,
    },
    Delete { id: String, base_rev: u64 },
}

impl ProposedChange {
    /// The id of the cell the change makes or touches.
    pub fn cell_id(&self) -> &str {
        match self {
            ProposedChange::Insert { cell, .. } => &cell.id,
            ProposedChange::Modify { id, .. } | ProposedChange::Delete { id, .. } => id,
        }
    }
}

/// A proposal, folded from its rows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Proposal {
    pub id: String,
    pub status: ProposalStatus,
    /// `ai:<session>`.
    pub proposed_by: String,
    /// The agent's description, which a reviewer reads.
    pub message: String,
    /// The document version the agent read.
    pub base_version: u64,
    pub changes: Vec<ProposedChange>,
    pub created_at_us: i64,
    /// Who decided (`user:<name>`, or the agent for a withdrawal) and when.
    pub decided_by: Option<String>,
    pub decided_at_us: Option<i64>,
    /// The note given with the decision (a rejection's reason).
    pub decision_note: Option<String>,
    /// The commit that applied it.
    pub accepted_version: Option<u64>,
    /// The highest `seq` (the number of status rows after the first).
    pub seq: i64,
}

/// Schema of the `proposals` table.
pub static PROPOSALS_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
    Arc::new(Schema::new(vec![
        Field::new("proposal_id", DataType::Utf8, false),
        Field::new("seq", DataType::Int64, false),
        Field::new("status", DataType::Utf8, false),
        Field::new("actor", DataType::Utf8, false),
        Field::new("message", DataType::Utf8, true),
        Field::new("base_version", DataType::Int64, true),
        Field::new("changes", DataType::Utf8, true),
        Field::new("at_us", DataType::Int64, false),
        Field::new("accepted_version", DataType::Int64, true),
    ]))
});

/// One row of the table.
#[derive(Debug, Clone)]
pub(crate) struct Row {
    pub proposal_id: String,
    pub seq: i64,
    pub status: ProposalStatus,
    pub actor: String,
    pub message: Option<String>,
    pub base_version: Option<u64>,
    pub changes: Option<String>,
    pub at_us: i64,
    pub accepted_version: Option<u64>,
}

pub(crate) fn rows_to_batch(rows: &[Row]) -> Result<RecordBatch> {
    let s = |f: fn(&Row) -> Option<&str>| -> ArrayRef {
        Arc::new(rows.iter().map(f).collect::<StringArray>())
    };
    let i = |f: fn(&Row) -> Option<i64>| -> ArrayRef {
        Arc::new(rows.iter().map(f).collect::<Int64Array>())
    };
    let columns = vec![
        s(|r| Some(r.proposal_id.as_str())),
        i(|r| Some(r.seq)),
        s(|r| Some(r.status.as_str())),
        s(|r| Some(r.actor.as_str())),
        s(|r| r.message.as_deref()),
        i(|r| r.base_version.map(|v| v as i64)),
        s(|r| r.changes.as_deref()),
        i(|r| Some(r.at_us)),
        i(|r| r.accepted_version.map(|v| v as i64)),
    ];
    Ok(RecordBatch::try_new(PROPOSALS_SCHEMA.clone(), columns)?)
}

fn col<'a, T: 'static>(b: &'a RecordBatch, name: &str) -> Result<&'a T> {
    b.column_by_name(name)
        .and_then(|c| c.as_any().downcast_ref::<T>())
        .ok_or_else(|| Error::Corrupt(format!("proposals column `{name}` missing or mistyped")))
}

pub(crate) fn batches_to_rows(batches: &[RecordBatch]) -> Result<Vec<Row>> {
    let mut out = Vec::new();
    for b in batches {
        let b = conform(b, &PROPOSALS_SCHEMA)?;
        let id = col::<StringArray>(&b, "proposal_id")?;
        let seq = col::<Int64Array>(&b, "seq")?;
        let status = col::<StringArray>(&b, "status")?;
        let actor = col::<StringArray>(&b, "actor")?;
        let message = col::<StringArray>(&b, "message")?;
        let base = col::<Int64Array>(&b, "base_version")?;
        let changes = col::<StringArray>(&b, "changes")?;
        let at = col::<Int64Array>(&b, "at_us")?;
        let acc = col::<Int64Array>(&b, "accepted_version")?;
        let s = |a: &StringArray, i: usize| a.is_valid(i).then(|| a.value(i).to_owned());
        let n = |a: &Int64Array, i: usize| a.is_valid(i).then(|| a.value(i) as u64);
        for i in 0..b.num_rows() {
            out.push(Row {
                proposal_id: id.value(i).to_owned(),
                seq: seq.value(i),
                status: ProposalStatus::parse(status.value(i)).ok_or_else(|| {
                    Error::Corrupt(format!("unknown proposal status {}", status.value(i)))
                })?,
                actor: actor.value(i).to_owned(),
                message: s(message, i),
                base_version: n(base, i),
                changes: s(changes, i),
                at_us: at.value(i),
                accepted_version: n(acc, i),
            });
        }
    }
    Ok(out)
}

/// Fold rows into proposals (oldest first).
pub(crate) fn fold(rows: Vec<Row>) -> Result<Vec<Proposal>> {
    let mut by_id: BTreeMap<String, Vec<Row>> = BTreeMap::new();
    for r in rows {
        by_id.entry(r.proposal_id.clone()).or_default().push(r);
    }
    let mut out = Vec::with_capacity(by_id.len());
    for (id, mut rows) in by_id {
        rows.sort_by_key(|r| r.seq);
        for w in rows.windows(2) {
            if w[0].seq == w[1].seq {
                return Err(Error::Corrupt(format!("two rows for proposal {id} at seq {}", w[0].seq)));
            }
        }
        let first = &rows[0];
        if first.seq != 0 {
            // Its creation row was pruned: only possible for decided proposals, which prune
            // removes whole, so a partial one is corrupt.
            return Err(Error::Corrupt(format!("proposal {id} has no creation row")));
        }
        let changes: Vec<ProposedChange> =
            serde_json::from_str(first.changes.as_deref().unwrap_or("[]"))?;
        let last = rows.last().expect("at least one row");
        let decided = rows.len() > 1;
        out.push(Proposal {
            id,
            status: last.status,
            proposed_by: first.actor.clone(),
            message: first.message.clone().unwrap_or_default(),
            base_version: first.base_version.unwrap_or_default(),
            changes,
            created_at_us: first.at_us,
            decided_by: decided.then(|| last.actor.clone()),
            decided_at_us: decided.then_some(last.at_us),
            decision_note: if decided { last.message.clone() } else { None },
            accepted_version: last.accepted_version,
            seq: last.seq,
        });
    }
    out.sort_by(|a, b| a.created_at_us.cmp(&b.created_at_us).then_with(|| a.id.cmp(&b.id)));
    Ok(out)
}
