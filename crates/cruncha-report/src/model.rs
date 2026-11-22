// SPDX-License-Identifier: GPL-3.0-or-later

//! What a report shows: an evaluated document, its title block, revisions and house style.
//!
//! The app builds a [`Report`] from the latest version and its results; the template
//! (`templates/calc-package.typ`) reads it as JSON. Nothing here is evaluated again: a value
//! that wasn't computed is [`Status::NotCalculated`], and the report says so.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Report {
    pub title: TitleBlock,
    /// Newest last: one row per snapshot, plus the current state if it has edits since.
    pub revisions: Vec<Revision>,
    pub style: HouseStyle,
    pub options: Options,
    pub blocks: Vec<Block>,
    pub provenance: Provenance,
}

/// The cover page and page header fields (the document's `report` cell).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TitleBlock {
    /// The calculation's title; the document name when empty.
    pub title: String,
    pub subject: String,
    pub project: String,
    pub project_number: String,
    pub client: String,
    pub prepared_by: String,
    pub checked_by: String,
    pub approved_by: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Revision {
    /// The snapshot name, e.g. `A`, `IFC`, `Rev 2`.
    pub label: String,
    /// ISO date, `2026-11-20`.
    pub date: String,
    pub description: String,
    pub by: String,
}

/// The office's look: set once in Preferences, the same for every document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct HouseStyle {
    pub company: String,
    /// Path of the logo inside the report's files (`/logo.png`), if there is one.
    pub logo: Option<String>,
    /// Accent color for rules and headings, `#RRGGBB`.
    pub accent: String,
    /// Body font names, tried in order. Inter is bundled (the default, the same on every
    /// computer); others come from the system; the bundled Libertinus Serif is the last resort.
    pub fonts: Vec<String>,
    /// Body size in points.
    pub font_size: f64,
    /// `us-letter` or `a4`.
    pub paper: String,
}

impl Default for HouseStyle {
    fn default() -> Self {
        HouseStyle {
            company: String::new(),
            logo: None,
            accent: "#1f4e79".into(),
            fonts: vec!["Inter".into(), "Libertinus Serif".into()],
            font_size: 10.0,
            paper: "us-letter".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Options {
    /// Print the source of code cells (their output is always printed).
    pub show_code: bool,
    /// A summary of every check and problem on the cover page.
    pub check_summary: bool,
    pub table_of_contents: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options { show_code: true, check_summary: true, table_of_contents: true }
    }
}

/// Where the report came from: printed in the appendix and the footer.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Provenance {
    pub document_name: String,
    pub document_id: String,
    pub version: u64,
    /// When that version was committed (ISO 8601, UTC). Also the report's date, so the same
    /// version always gives the same PDF.
    pub committed_at: String,
    /// SHA-256 over every cell's id, kind and source, in order: what a reviewer compares.
    pub content_hash: String,
    pub cruncha_version: String,
    /// The Python environment's lockfile hash, when the document has Python cells.
    pub python_env: Option<String>,
    /// The snapshot this version is, if it is one.
    pub snapshot: Option<String>,
}

/// One cell, ready to print.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum Block {
    Heading {
        level: u8,
        text: Vec<Run>,
    },
    Paragraph {
        text: Vec<Run>,
    },
    /// A bulleted (`ordered: false`) or numbered list. Each item holds paragraphs, lists and
    /// quotes (never headings, calcs or code).
    List {
        ordered: bool,
        start: u64,
        items: Vec<Vec<Block>>,
    },
    Quote {
        blocks: Vec<Block>,
    },
    Rule,
    Calc {
        id: String,
        /// Typst math code (converted from the engine's TeX).
        math: String,
        description: String,
        status: Status,
        /// A plain-text fallback when the math couldn't be converted.
        text: String,
        /// `true` for `@test` lines: they appear in the check summary.
        check: bool,
    },
    Code {
        id: String,
        lang: String,
        source: String,
        status: Status,
        outputs: Vec<Output>,
        /// Names the cell defines, with their values as plain text.
        defines: Vec<(String, String)>,
        /// Where its output came from when it wasn't computed on this computer (a stored
        /// result, ADR-0018), e.g. "Computed on office-mac, 2026-11-01; not run on this
        /// computer". Printed with the cell and counted in the summary.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        provenance: Option<String>,
    },
    /// A cell kind the report can't show yet.
    Other {
        kind: String,
        source: String,
    },
}

/// A run of prose text with its marks.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Run {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub code: bool,
    pub sub: bool,
    pub sup: bool,
    /// An http, https or mailto URL.
    pub link: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum Status {
    Ok,
    /// A check that holds.
    Pass,
    /// A check that fails.
    Fail,
    Error {
        message: String,
    },
    /// Not computed (Python not set up, not trusted, edited and not run, an input missing…).
    /// Printed as NOT CALCULATED, never as a pass.
    NotCalculated {
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum Output {
    /// A figure: path of a PNG or SVG inside the report's files.
    Image { path: String },
    /// An interactive figure (ADR-0017): its static rendering (`path`, SVG or PNG) when the
    /// app could draw one, otherwise FIGURE NOT RENDERED with the `reason`. Never left out.
    Figure { path: Option<String>, summary: String, reason: Option<String> },
    Table { columns: Vec<String>, rows: Vec<Vec<String>>, total_rows: usize },
    Text { text: String, stream: String },
}
