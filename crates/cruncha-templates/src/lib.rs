// SPDX-License-Identifier: GPL-3.0-or-later

//! Calculation templates and snippets (ADR-0014).
//!
//! A template is a `<id>.cruncha.md` file: TOML front matter between `+++` lines (the
//! [`Manifest`]: id, version, title, inputs, outputs, references, worked-example tests), then
//! the cells in order. Prose is Markdown; each calc, Python, Rust or TeX cell is a fenced block
//! whose info string names the kind and may set attributes (```` ```calc {description="…"} ````).
//! A calc fence holds one calc, or a group of steps to one answer, each step's description on
//! a `>` line before it.
//! Snippets use the same format with only `id`, `version` and `title` required.
//!
//! This crate reads the format ([`parse`]), checks it ([`validate`]) and runs its worked
//! examples with Cruncha's own calc engine ([`run_tests`]). Inserting a template into a
//! document (renames, provenance) is the app's (ADR-0014 §3).

use std::collections::BTreeMap;

use cruncha_eval::{Num, Options, Plain, UnitAware, Vars};
use serde::{Deserialize, Serialize};

/// The format version this crate reads (`cruncha-template = 1`).
pub const FORMAT: u32 = 1;

/// The front matter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    #[serde(rename = "cruncha-template")]
    pub format: u32,
    /// Dotted and stable (`structural.bolts.shearTension`): provenance in documents points to
    /// it, so it never changes once published.
    pub id: String,
    /// `major.minor.patch`.
    pub version: String,
    pub title: String,
    #[serde(default)]
    pub summary: Option<String>,
    /// The category path (`["Structural engineering", "Connections"]`).
    #[serde(default)]
    pub category: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Where it was converted from (`lacewing:structural.bolts`).
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    /// Names the template defines with a placeholder value, for the user to set.
    #[serde(default)]
    pub inputs: Vec<String>,
    /// Names the template promises.
    #[serde(default)]
    pub outputs: Vec<String>,
    #[serde(default)]
    pub references: Vec<Reference>,
    #[serde(default)]
    pub tests: Vec<TestCase>,
}

/// A source: a work, its edition and where in it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Reference {
    pub work: String,
    #[serde(default)]
    pub edition: Option<String>,
    /// Section, clause, equation, table or example.
    #[serde(default)]
    pub locator: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}

/// A worked example: input values, and the outputs they must give.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TestCase {
    pub name: String,
    /// Where the numbers come from.
    #[serde(default)]
    pub source: Option<String>,
    /// Input values as calc literals (`"0.75 in"`), replacing the placeholders.
    #[serde(default)]
    pub inputs: BTreeMap<String, String>,
    /// Expected values as calc literals (`"17.9 kips"`).
    #[serde(default)]
    pub expect: BTreeMap<String, String>,
    /// Relative tolerance: `"0.3%"` or `"0.003"`. Default 1e-9.
    #[serde(default)]
    pub tolerance: Option<String>,
    /// `"pass"`: every `@test` holds; `"fail"`: at least one doesn't.
    #[serde(default)]
    pub checks: Option<String>,
}

/// One cell of a template, without an id or order key (those are made on insert).
#[derive(Debug, Clone, PartialEq)]
pub enum Cell {
    /// ProseMirror JSON, as documents store prose (`cruncha_prose::to_json`).
    Prose { source: String },
    Calc { source: String, description: Option<String> },
    /// `python`, `rust` or `tex`.
    Code { lang: String, source: String, description: Option<String> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Template {
    pub manifest: Manifest,
    pub cells: Vec<Cell>,
}

/// Read a `.cruncha.md` file.
pub fn parse(text: &str) -> Result<Template, String> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let rest = text
        .strip_prefix("+++\n")
        .or_else(|| text.strip_prefix("+++\r\n"))
        .ok_or("a template starts with a +++ line (its TOML front matter)")?;
    let end = rest
        .find("\n+++\n")
        .or_else(|| rest.find("\n+++\r\n"))
        .or_else(|| rest.strip_suffix("\n+++").map(|r| r.len()))
        .ok_or("the front matter ends with a +++ line")?;
    let manifest: Manifest = toml::from_str(&rest[..end]).map_err(|e| format!("front matter: {e}"))?;
    let body = rest[end + 4..].trim_start_matches(['\r', '\n']);
    let cells = cruncha_prose::from_markdown(body)
        .into_iter()
        .map(|p| match p {
            cruncha_prose::Piece::Prose(b) => Cell::Prose { source: cruncha_prose::to_json(&b) },
            cruncha_prose::Piece::Calc { source, mut attrs } => Cell::Calc { source, description: attrs.remove("description") },
            cruncha_prose::Piece::Code { lang, source, mut attrs } => {
                Cell::Code { lang, source, description: attrs.remove("description") }
            }
        })
        .collect();
    Ok(Template { manifest, cells })
}

fn opts() -> Options {
    Options { unit_aware: UnitAware::Always, ..Options::default() }
}

/// What each calc cell defines (empty for one that doesn't prepare).
fn calc_defines(t: &Template) -> Vec<(usize, Vec<String>)> {
    let o = opts();
    t.cells
        .iter()
        .enumerate()
        .filter_map(|(i, c)| match c {
            Cell::Calc { source, .. } => Some((i, cruncha_eval::prepare(source, &o).map(|s| s.defines).unwrap_or_default())),
            _ => None,
        })
        .collect()
}

/// Everything wrong with a template, in words (empty: it's valid). Calcs are evaluated at
/// their placeholder values; code cells aren't run here.
pub fn validate(t: &Template) -> Vec<String> {
    let m = &t.manifest;
    let mut problems = Vec::new();
    if m.format != FORMAT {
        problems.push(format!("cruncha-template is {}; this Cruncha reads {FORMAT}", m.format));
    }
    let segment_ok = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    if !m.id.split('.').all(segment_ok) {
        problems.push(format!("id {:?}: dotted segments of letters, digits, _ and -", m.id));
    }
    let parts: Vec<&str> = m.version.split('.').collect();
    if parts.len() != 3 || parts.iter().any(|p| p.parse::<u64>().is_err()) {
        problems.push(format!("version {:?}: major.minor.patch", m.version));
    }
    if m.title.trim().is_empty() {
        problems.push("the title is empty".into());
    }
    let defines = calc_defines(t);
    let defined_by = |name: &str| defines.iter().find(|(_, d)| d.iter().any(|n| n == name)).map(|(i, _)| *i);
    let has_code = t.cells.iter().any(|c| matches!(c, Cell::Code { .. }));
    for name in &m.inputs {
        match defined_by(name) {
            Some(i) => {
                if let Cell::Calc { source, .. } = &t.cells[i]
                    && !source.trim_end().ends_with('_')
                {
                    problems.push(format!("input {name} is defined without a placeholder (end its value with _)"));
                }
            }
            None => problems.push(format!("input {name} isn't defined by a calc")),
        }
    }
    for name in &m.outputs {
        if defined_by(name).is_none() && !has_code {
            problems.push(format!("output {name} isn't defined"));
        }
    }
    // Every calc evaluates, in order, at the placeholder values.
    let o = opts();
    let mut vars = Vars::new();
    for c in &t.cells {
        if let Cell::Calc { source, .. } = c {
            match cruncha_eval::prepare(source, &o) {
                Err(e) => problems.push(format!("calc {source:?} doesn't parse: {e}")),
                Ok(stmt) => {
                    let out = cruncha_eval::evaluate(&stmt, &vars, &o);
                    // A calc reading a code cell's names can't evaluate here.
                    if let Some(e) = out.error
                        && !has_code
                    {
                        problems.push(format!("calc {source:?}: {e}"));
                    }
                    for (n, v) in out.values {
                        vars.insert(n, v);
                    }
                }
            }
        }
    }
    for (k, case) in m.tests.iter().enumerate() {
        let label = if case.name.is_empty() { format!("test {}", k + 1) } else { format!("test {:?}", case.name) };
        for name in case.inputs.keys() {
            if !m.inputs.contains(name) {
                problems.push(format!("{label}: {name} isn't one of the template's inputs"));
            }
        }
        if case.expect.is_empty() && case.checks.is_none() {
            problems.push(format!("{label} expects nothing"));
        }
        if let Some(tol) = &case.tolerance
            && parse_tolerance(tol).is_none()
        {
            problems.push(format!("{label}: tolerance {tol:?} is not \"0.3%\" or a number"));
        }
        if let Some(c) = &case.checks
            && c != "pass"
            && c != "fail"
        {
            problems.push(format!("{label}: checks is \"pass\" or \"fail\", not {c:?}"));
        }
    }
    problems
}

fn parse_tolerance(s: &str) -> Option<f64> {
    let s = s.trim();
    match s.strip_suffix('%') {
        Some(p) => p.trim().parse::<f64>().ok().map(|x| x / 100.0),
        None => s.parse().ok(),
    }
}

/// A test case's outcome.
#[derive(Debug, Clone, PartialEq)]
pub struct TestReport {
    pub name: String,
    pub passed: bool,
    /// One line per expected value or check, and any error.
    pub lines: Vec<String>,
    /// Why it wasn't run (code cells, which need the app's workers).
    pub skipped: Option<String>,
}

/// A value in SI base units with its dimensions, if it is a real number.
fn si(v: &cruncha_eval::Value) -> Option<(f64, Vec<f64>)> {
    match v.to_plain() {
        Plain::Number { value, unit } => {
            let x = match value {
                Num::Float { value } => value,
                Num::Exact { num, den } => num.parse::<f64>().ok()? / den.parse::<f64>().ok()?,
            };
            Some(match unit {
                Some(u) => ((x + u.gauge) * u.factor, u.dims.to_vec()),
                None => (x, vec![0.0; 8]),
            })
        }
        _ => None,
    }
}

/// Run every worked example: each test's inputs replace the placeholders, the calcs are
/// evaluated in order, and the expected outputs are compared in SI within the tolerance.
/// Templates with code cells are skipped here (the app's tests run them with real workers).
pub fn run_tests(t: &Template) -> Vec<TestReport> {
    let has_code = t.cells.iter().any(|c| matches!(c, Cell::Code { .. }));
    let defines = calc_defines(t);
    let o = opts();
    t.manifest
        .tests
        .iter()
        .map(|case| {
            if has_code {
                return TestReport {
                    name: case.name.clone(),
                    passed: false,
                    lines: Vec::new(),
                    skipped: Some("the template has code cells; the app's tests run it".into()),
                };
            }
            let mut lines = Vec::new();
            let mut passed = true;
            let mut vars = Vars::new();
            let mut checks: Vec<(String, Option<bool>)> = Vec::new();
            for (i, c) in t.cells.iter().enumerate() {
                let Cell::Calc { source, .. } = c else { continue };
                let defs = defines.iter().find(|(j, _)| *j == i).map(|(_, d)| d.as_slice()).unwrap_or(&[]);
                let source = match defs {
                    [name] if case.inputs.contains_key(name) => format!("{name} = {}", case.inputs[name]),
                    _ => source.clone(),
                };
                match cruncha_eval::prepare(&source, &o) {
                    Err(e) => {
                        passed = false;
                        lines.push(format!("calc {source:?} doesn't parse: {e}"));
                    }
                    Ok(stmt) => {
                        let out = cruncha_eval::evaluate(&stmt, &vars, &o);
                        if let Some(e) = &out.error {
                            passed = false;
                            lines.push(format!("calc {source:?}: {e}"));
                        }
                        if source.trim_start().starts_with("@test") {
                            checks.push((source.clone(), out.check));
                        }
                        for (n, v) in out.values {
                            vars.insert(n, v);
                        }
                    }
                }
            }
            let tol = case.tolerance.as_deref().and_then(parse_tolerance).unwrap_or(1e-9);
            for (name, want) in &case.expect {
                let got = vars.get(name).and_then(si);
                let mut scratch = Vars::new();
                cruncha_eval::calculate(&format!("expected = {want}"), &mut scratch, &o);
                let expected = scratch.get("expected").and_then(si);
                match (got, expected) {
                    (Some((g, gd)), Some((e, ed))) => {
                        let rel = if e == 0.0 { g.abs() } else { ((g - e) / e).abs() };
                        let ok = gd == ed && rel <= tol;
                        passed &= ok;
                        lines.push(format!(
                            "{} {name}: expected {want}, got {}{}",
                            if ok { "ok" } else { "FAIL" },
                            g / if e == 0.0 { 1.0 } else { e / want_number(want).unwrap_or(e) },
                            if gd != ed { " (different dimensions)" } else { "" }
                        ));
                    }
                    (None, _) => {
                        passed = false;
                        lines.push(format!("FAIL {name}: not defined, or not a number"));
                    }
                    (_, None) => {
                        passed = false;
                        lines.push(format!("FAIL {name}: the expected value {want:?} doesn't evaluate"));
                    }
                }
            }
            if let Some(want) = &case.checks {
                let all = checks.iter().all(|(_, c)| *c == Some(true));
                let ok = if want == "pass" { all } else { !all };
                passed &= ok;
                lines.push(format!(
                    "{} checks: expected {want}; {}",
                    if ok { "ok" } else { "FAIL" },
                    checks
                        .iter()
                        .map(|(s, c)| format!("{} {}", s.trim_start_matches("@test").trim(), match c {
                            Some(true) => "OK",
                            Some(false) => "NOT OK",
                            None => "not evaluated",
                        }))
                        .collect::<Vec<_>>()
                        .join("; ")
                ));
            }
            TestReport { name: case.name.clone(), passed, lines, skipped: None }
        })
        .collect()
}

/// The number at the start of a literal (`"17.9 kips"` → 17.9), to report results in the
/// expected value's unit.
fn want_number(want: &str) -> Option<f64> {
    let n: String = want.trim().chars().take_while(|c| c.is_ascii_digit() || matches!(c, '.' | '-' | 'e' | 'E' | '+')).collect();
    n.parse().ok().filter(|x: &f64| *x != 0.0)
}
