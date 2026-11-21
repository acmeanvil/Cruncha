// SPDX-License-Identifier: GPL-3.0-or-later

//! Cruncha's MCP server (ADR-0009): the only way agents touch a document.
//!
//! The app (which holds the document's `Store`, the single writer) implements [`Backend`];
//! this crate turns it into MCP tools ([`tools::CrunchaTools`]) and serves them over
//! Streamable HTTP on 127.0.0.1 with a bearer token ([`http::serve`]).
//!
//! The tools follow `skills/cruncha-core`. This first phase has the read tools:
//! `read_doc`, `explain`, `check`, `history`, and `run` (which returns the engine's current
//! results). `propose_edit` comes with proposals (a later phase).
//!
//! The engine computes every number: tools return the engine's results verbatim, never a
//! value worked out here.

pub mod http;
pub mod mirror;
pub mod tools;

use std::collections::HashMap;
use std::future::Future;

use serde::Serialize;

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
    /// Python output, summarized (`figure 1`, `table 51 × 2`, printed text).
    pub outputs: Vec<String>,
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

/// What the app provides. Every method acts on the document open in the app; an error
/// (such as "no document is open") is returned to the agent as the tool's error.
pub trait Backend: Clone + Send + Sync + 'static {
    /// The latest version and its results. `wait` asks to wait (bounded) for Python cells
    /// that are due to run, so their results are included.
    fn snapshot(&self, wait: bool) -> impl Future<Output = Result<DocSnapshot, String>> + Send;
    fn history(&self) -> impl Future<Output = Result<Vec<VersionInfo>, String>> + Send;
    fn diff(&self, from: u64, to: u64) -> impl Future<Output = Result<Vec<CellChange>, String>> + Send;
}
