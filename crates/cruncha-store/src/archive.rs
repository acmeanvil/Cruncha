// SPDX-License-Identifier: GPL-3.0-or-later
//! The save log (ADR-0023): the `archive/` table, one row per save that a prune or an archive
//! removed. A row says when the save was made, by whom, with what message, in which snapshot
//! group, and which cells it wrote. It is the record that remains once the saves themselves
//! are gone; the snapshots stay in `pins/`.
//!
//! The table is append-only and created by the first prune or archive that removes a save. It
//! is never pruned. Rows are written before the prune's point of no return, and only for
//! versions not already in the table, so a prune that is interrupted and finished later
//! doesn't duplicate them.

use std::sync::{Arc, LazyLock};

use deltalake::arrow::array::{Array, ArrayRef, Int64Array, RecordBatch, StringArray};
use deltalake::arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::schema::conform;

/// One removed save.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchivedSave {
    pub version: u64,
    /// Cruncha's commit time (`cruncha.at`), µs.
    pub at_us: i64,
    pub actor: String,
    pub message: String,
    /// `create`, `edit`, `restore` or `proposal` (an accepted agent proposal).
    pub kind: String,
    /// The snapshot that starts the save's group; `None` before the first snapshot (Draft).
    pub snapshot: Option<String>,
    /// IDs of the cells the save wrote (changed, added or deleted), sorted.
    pub cells: Vec<String>,
    /// The cutoff of the prune or archive that removed it.
    pub removed_before: u64,
}

/// Schema of the `archive` table. `cells` is a JSON array of strings.
pub static ARCHIVE_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
    Arc::new(Schema::new(vec![
        Field::new("version", DataType::Int64, false),
        Field::new("at_us", DataType::Int64, false),
        Field::new("actor", DataType::Utf8, false),
        Field::new("message", DataType::Utf8, false),
        Field::new("kind", DataType::Utf8, false),
        Field::new("snapshot", DataType::Utf8, true),
        Field::new("cells", DataType::Utf8, false),
        Field::new("removed_before", DataType::Int64, false),
    ]))
});

pub(crate) fn rows_to_batch(rows: &[ArchivedSave]) -> Result<RecordBatch> {
    let s = |f: &dyn Fn(&ArchivedSave) -> Option<String>| -> ArrayRef { Arc::new(rows.iter().map(f).collect::<StringArray>()) };
    let i = |f: &dyn Fn(&ArchivedSave) -> i64| -> ArrayRef { Arc::new(rows.iter().map(|r| Some(f(r))).collect::<Int64Array>()) };
    let columns = vec![
        i(&|r| r.version as i64),
        i(&|r| r.at_us),
        s(&|r| Some(r.actor.clone())),
        s(&|r| Some(r.message.clone())),
        s(&|r| Some(r.kind.clone())),
        s(&|r| r.snapshot.clone()),
        s(&|r| Some(serde_json::to_string(&r.cells).expect("strings serialize"))),
        i(&|r| r.removed_before as i64),
    ];
    Ok(RecordBatch::try_new(ARCHIVE_SCHEMA.clone(), columns)?)
}

fn col<'a, T: 'static>(b: &'a RecordBatch, name: &str) -> Result<&'a T> {
    b.column_by_name(name)
        .and_then(|c| c.as_any().downcast_ref::<T>())
        .ok_or_else(|| Error::Corrupt(format!("archive column `{name}` missing or mistyped")))
}

/// The rows, oldest version first, one per version (the first written wins).
pub(crate) fn batches_to_rows(batches: &[RecordBatch]) -> Result<Vec<ArchivedSave>> {
    let mut out: Vec<ArchivedSave> = Vec::new();
    for b in batches {
        let b = conform(b, &ARCHIVE_SCHEMA)?;
        let version = col::<Int64Array>(&b, "version")?;
        let at = col::<Int64Array>(&b, "at_us")?;
        let actor = col::<StringArray>(&b, "actor")?;
        let message = col::<StringArray>(&b, "message")?;
        let kind = col::<StringArray>(&b, "kind")?;
        let snapshot = col::<StringArray>(&b, "snapshot")?;
        let cells = col::<StringArray>(&b, "cells")?;
        let removed = col::<Int64Array>(&b, "removed_before")?;
        for i in 0..b.num_rows() {
            out.push(ArchivedSave {
                version: version.value(i).max(0) as u64,
                at_us: at.value(i),
                actor: actor.value(i).to_owned(),
                message: message.value(i).to_owned(),
                kind: kind.value(i).to_owned(),
                snapshot: snapshot.is_valid(i).then(|| snapshot.value(i).to_owned()),
                cells: serde_json::from_str(cells.value(i))
                    .map_err(|e| Error::Corrupt(format!("archive cells of version {}: {e}", version.value(i))))?,
                removed_before: removed.value(i).max(0) as u64,
            });
        }
    }
    out.sort_by_key(|r| r.version);
    out.dedup_by_key(|r| r.version);
    Ok(out)
}
