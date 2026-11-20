// SPDX-License-Identifier: GPL-3.0-or-later
// Ported from Hurmet (https://hurmet.org), Copyright (c) 2026 Ronald Kok, MIT License; see LICENSE-HURMET.

//! A statement's working attributes (Hurmet's ProseMirror node `attrs`), written by
//! `compile.js` and completed by `evaluate.js` and `result.js`.

use crate::value::{OUnit, V};
use cruncha_units::Dims;

/// The name(s) a statement assigns.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum Names {
    #[default]
    None,
    One(String),
    Many(Vec<String>),
}

impl Names {
    pub fn list(&self) -> Vec<String> {
        match self {
            Names::None => Vec::new(),
            Names::One(n) => vec![n.clone()],
            Names::Many(v) => v.clone(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Attrs {
    pub entry: String,
    /// The TeX of the whole statement, with the selector still in place.
    pub template: String,
    /// The plain-text statement, with the selector still in place.
    pub alt_template: String,
    /// TeX of the result as later echoes show it (after evaluation), or the result template.
    pub resultdisplay: String,
    /// Set by `errorResult` (Hurmet's camel-cased `resultDisplay`).
    pub result_display_err: String,
    pub alt_result_display: String,
    /// The TeX result template (`?? {\text{kip}…}`); `None` for statements without a selector.
    pub resulttemplate: Option<String>,
    /// The plain result template (`?? kip·ft`).
    pub altresulttemplate: String,
    pub name: Names,
    pub rpn: Option<String>,
    pub dependencies: Vec<String>,
    pub value: V,
    pub unit: OUnit,
    pub dtype: i64,
    pub error: bool,
    pub tex: String,
    pub alt: String,
    pub display_selector: String,
    pub expos: Option<Dims>,
    pub md: Option<String>,
    /// Set by compile for a finished statement (no evaluation needed).
    pub is_static: bool,
}
