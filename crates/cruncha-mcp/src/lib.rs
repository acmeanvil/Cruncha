// SPDX-License-Identifier: GPL-3.0-or-later

//! Cruncha's MCP server (ADR-0009): the only way agents touch a document.
//!
//! The app (which holds the document's `Store`, the single writer) implements [`Backend`];
//! this crate turns it into MCP tools ([`tools::CrunchaTools`]) and serves them over
//! Streamable HTTP on 127.0.0.1 with a bearer token ([`http::serve`]).
//!
//! The tools follow `skills/cruncha-core`: the read tools `read_doc`, `explain`, `check`,
//! `history` and `run` (the engine's current results), and `propose_edit`, `proposals` and
//! `withdraw` (ADR-0016). Agents never write the document: a proposal waits for a human to
//! accept or reject it in the app.
//!
//! The engine computes every number: tools return the engine's results verbatim, never a
//! value worked out here.

pub mod http;
pub mod mirror;
pub mod tools;

use std::collections::HashMap;
use std::future::Future;

use rmcp::schemars;
use serde::{Deserialize, Serialize};

/// One cell, as agents see it.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct CellView {
    pub id: String,
    /// `prose`, `calc`, `python`, `meta`, …
    pub kind: String,
    pub source: String,
    /// The calc's description (`attrs.description`), if any.
    pub description: String,
    pub defines: Vec<String>,
    pub reads: Vec<String>,
    /// `human`, `ai:proposed`, `ai:accepted`, `imported`.
    pub provenance: String,
    pub modified_by: String,
}

/// A cell's result, as the engine produced it.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct ResultView {
    /// The plain-text result line (`M = w L^2 / 8 = 108 kip·ft`).
    pub text: String,
    pub error: Option<String>,
    pub warnings: Vec<String>,
    /// For `@test` calcs: whether the check holds.
    pub check: Option<bool>,
    /// Waiting for a Python cell upstream.
    pub pending: bool,
    /// For Python cells: `done`, `error`, `edited`, `untrusted`, … and why.
    pub python_state: Option<String>,
    pub python_note: Option<String>,
    /// Each defined name with its value as text (or what the Python object is).
    pub values: Vec<(String, String)>,
    /// Python output, summarized (`figure (plotly): …`, `table 51 × 2`, printed text).
    pub outputs: Vec<String>,
    /// Each figure trace's data range, for `explain` (ADR-0017 §5).
    pub figure_data: Vec<String>,
}

/// The latest version of the open document with its results.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct DocSnapshot {
    pub name: String,
    pub id: String,
    pub version: u64,
    /// In document order.
    pub cells: Vec<CellView>,
    pub results: HashMap<String, ResultView>,
}

/// One version in the history.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct VersionInfo {
    pub version: u64,
    /// ISO 8601, UTC.
    pub at: String,
    pub actor: String,
    pub message: String,
    pub kind: String,
    /// Snapshot names pointing at this version.
    pub snapshots: Vec<String>,
}

/// What archiving and trimming removed (ADR-0023): the snapshots kept from removed history
/// and the save log.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct ArchiveView {
    /// Oldest first.
    pub snapshots: Vec<ArchivedSnapshot>,
    /// Oldest first.
    pub saves: Vec<ArchivedSaveInfo>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct ArchivedSnapshot {
    pub name: String,
    pub version: u64,
    /// ISO 8601, UTC.
    pub at: String,
    pub message: String,
}

/// One removed save.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct ArchivedSaveInfo {
    pub version: u64,
    /// ISO 8601, UTC.
    pub at: String,
    pub actor: String,
    pub kind: String,
    pub message: String,
    /// The snapshot group it was in; `None` before the first snapshot.
    pub snapshot: Option<String>,
    /// IDs of the cells it changed.
    pub cells: Vec<String>,
}

/// A cell that changed between two versions.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CellChange {
    pub id: String,
    pub kind: String,
    /// `added`, `removed` or `modified`.
    pub change: String,
    pub before: Option<String>,
    pub after: Option<String>,
}

/// One change an agent proposes (`propose_edit`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "op", rename_all = "lowercase")]
pub enum Edit {
    /// A new cell after the cell `after` (omit `after` for the start of the document). To add
    /// several cells in a row, give each later one `after` = the id the reply gave the one
    /// before it, or list them in reverse order with the same `after`.
    Insert {
        #[serde(default)]
        after: Option<String>,
        /// `calc`, `python` or `prose`.
        kind: String,
        /// A calc statement, Python code, or one paragraph of prose (`## Title` for a heading).
        source: String,
        /// For calcs: what the defined quantity is (one line).
        #[serde(default)]
        description: Option<String>,
    },
    /// New source and/or description for an existing cell.
    Modify {
        id: String,
        #[serde(default)]
        source: Option<String>,
        #[serde(default)]
        description: Option<String>,
    },
    /// Remove a cell.
    Delete { id: String },
}

/// A cell whose result a proposal changes: before (now) and after (if accepted).
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Effect {
    pub id: String,
    pub kind: String,
    /// `None`: a new cell.
    pub before: Option<ResultView>,
    /// `None`: the cell is deleted.
    pub after: Option<ResultView>,
}

/// A proposal, as the agent sees it.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct ProposalInfo {
    pub id: String,
    /// `open`, `accepted`, `rejected`, `withdrawn`.
    pub status: String,
    pub proposed_by: String,
    pub message: String,
    pub base_version: u64,
    /// The cells it touches, as `op id` (`insert 01J…`, `modify 01J…`).
    pub changes: Vec<String>,
    /// Some cell changed since the proposal was made.
    pub stale: bool,
    /// The reviewer's note on a decision (a rejection's reason).
    pub decision_note: Option<String>,
    pub accepted_version: Option<u64>,
    /// "If accepted" results (open proposals).
    pub effects: Vec<Effect>,
}

/// What the app provides. Every method acts on the document open in the app; an error
/// (such as "no document is open") is returned to the agent as the tool's error.
pub trait Backend: Clone + Send + Sync + 'static {
    /// The latest version and its results. `wait` asks to wait (bounded) for Python cells
    /// that are due to run, so their results are included.
    fn snapshot(&self, wait: bool) -> impl Future<Output = Result<DocSnapshot, String>> + Send;
    fn history(&self) -> impl Future<Output = Result<Vec<VersionInfo>, String>> + Send;
    fn diff(&self, from: u64, to: u64) -> impl Future<Output = Result<Vec<CellChange>, String>> + Send;
    /// Record a proposal by this server's agent against the latest version. `base_version`
    /// is the version the agent read; cells changed after it are refused.
    fn propose(
        &self,
        base_version: Option<u64>,
        message: String,
        edits: Vec<Edit>,
    ) -> impl Future<Output = Result<ProposalInfo, String>> + Send;
    /// The document's open proposals and this agent's decided ones (newest last).
    fn proposals(&self) -> impl Future<Output = Result<Vec<ProposalInfo>, String>> + Send;
    /// Withdraw one of this agent's open proposals.
    fn withdraw(&self, id: String) -> impl Future<Output = Result<ProposalInfo, String>> + Send;
    /// The document at a named snapshot, read-only, with its results (ADR-0023: archived
    /// snapshots too).
    fn snapshot_named(&self, name: String) -> impl Future<Output = Result<DocSnapshot, String>> + Send {
        async move { Err(format!("this server can't read snapshot “{name}”")) }
    }
    /// The archive: snapshots kept from removed history and the save log; with `cell`, only
    /// saves that changed it.
    fn archive(&self, cell: Option<String>) -> impl Future<Output = Result<ArchiveView, String>> + Send {
        let _ = cell;
        async { Ok(ArchiveView::default()) }
    }
}
