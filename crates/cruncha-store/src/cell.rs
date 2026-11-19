// SPDX-License-Identifier: GPL-3.0-or-later
//! The cell record: one row of the `doc` table.

use serde::{Deserialize, Serialize};

/// What kind of block a cell is. Stored as its lowercase name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CellKind {
    /// Document-level settings (title, page size, number format, …). At most one.
    Meta,
    Prose,
    Calc,
    Python,
    Rust,
    Data,
    Plot,
    Drawing,
    Test,
    Tex,
    Page,
}

impl CellKind {
    pub const ALL: [CellKind; 11] = [
        CellKind::Meta,
        CellKind::Prose,
        CellKind::Calc,
        CellKind::Python,
        CellKind::Rust,
        CellKind::Data,
        CellKind::Plot,
        CellKind::Drawing,
        CellKind::Test,
        CellKind::Tex,
        CellKind::Page,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            CellKind::Meta => "meta",
            CellKind::Prose => "prose",
            CellKind::Calc => "calc",
            CellKind::Python => "python",
            CellKind::Rust => "rust",
            CellKind::Data => "data",
            CellKind::Plot => "plot",
            CellKind::Drawing => "drawing",
            CellKind::Test => "test",
            CellKind::Tex => "tex",
            CellKind::Page => "page",
        }
    }

    pub fn parse(s: &str) -> Option<CellKind> {
        CellKind::ALL.into_iter().find(|k| k.as_str() == s)
    }
}

/// Who produced or last approved a cell's content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Provenance {
    Human,
    AiProposed { session: String },
    AiAccepted { session: String, by: String },
    Imported { source: String },
}

impl Provenance {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("provenance serializes")
    }

    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

/// One block or inline cell of a document, as persisted.
///
/// Results are deliberately *not* part of a cell; they live in the `results` table keyed by
/// cell id, so editing a document never rewrites cached values and vice versa.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cell {
    /// Stable identifier (a ULID string). Never reused.
    pub id: String,
    pub kind: CellKind,
    /// Fractional-index sort key: blocks are ordered by this, so inserts never renumber.
    pub order_key: String,
    /// For inline cells: the prose block containing them.
    pub parent_id: Option<String>,
    /// The cell's source text (calc statement, Python/Rust code, ProseMirror JSON, …).
    pub source: String,
    /// Kind-specific settings (display selector, format, unit, …) as JSON.
    pub attrs: serde_json::Value,
    /// Names this cell defines in the document namespace.
    pub defines: Vec<String>,
    /// Names this cell reads.
    pub reads: Vec<String>,
    pub provenance: Provenance,
    /// Actor of the last change (`user:<name>`, `ai:<session>`, `cli`, …).
    pub modified_by: String,
    /// Unix epoch microseconds of the last change.
    pub modified_at_us: i64,
}

impl Cell {
    /// A new cell with a fresh id, authored by a human.
    pub fn new(kind: CellKind, order_key: impl Into<String>, source: impl Into<String>) -> Self {
        Cell {
            id: ulid::Ulid::generate().to_string(),
            kind,
            order_key: order_key.into(),
            parent_id: None,
            source: source.into(),
            attrs: serde_json::Value::Object(Default::default()),
            defines: Vec::new(),
            reads: Vec::new(),
            provenance: Provenance::Human,
            modified_by: String::new(),
            modified_at_us: 0,
        }
    }

    /// Equality ignoring the bookkeeping fields (`modified_by`, `modified_at_us`).
    pub fn same_content(&self, other: &Cell) -> bool {
        self.id == other.id
            && self.kind == other.kind
            && self.order_key == other.order_key
            && self.parent_id == other.parent_id
            && self.source == other.source
            && self.attrs == other.attrs
            && self.defines == other.defines
            && self.reads == other.reads
            && self.provenance == other.provenance
    }
}

/// One change in a commit.
#[derive(Debug, Clone, PartialEq)]
#[allow(
    clippy::large_enum_variant,
    reason = "short-lived values; boxing would only hurt ergonomics"
)]
pub enum Change {
    /// Insert the cell, or replace the cell with the same id.
    Upsert(Cell),
    /// Delete the cell with this id (no-op if absent).
    Delete(String),
}

impl Change {
    pub fn id(&self) -> &str {
        match self {
            Change::Upsert(c) => &c.id,
            Change::Delete(id) => id,
        }
    }
}

/// Difference of one cell between two versions.
#[derive(Debug, Clone, PartialEq)]
#[allow(
    clippy::large_enum_variant,
    reason = "short-lived values; boxing would only hurt ergonomics"
)]
pub enum CellDiff {
    Added(Cell),
    Removed(Cell),
    Modified { before: Cell, after: Cell },
}
