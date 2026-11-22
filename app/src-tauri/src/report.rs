// SPDX-License-Identifier: GPL-3.0-or-later

//! Reports (spike S6): a version (the latest, a past one or a snapshot) and its results as a
//! `cruncha_report::Report`.
//!
//! The title block and report options live in the document's `meta` cell
//! (`attrs.report`), so they are versioned with the content. The house style is the
//! office's, from Preferences. Revisions are the document's snapshots.
//!
//! Nothing is evaluated here: a cell without a usable result is printed as NOT CALCULATED
//! with the reason, never as a pass.

use std::collections::HashMap;

use base64::Engine as _;
use cruncha_report::{Block, HouseStyle, Options, Output, Provenance, Report, Revision, Run, Status, TitleBlock};
use cruncha_store::{Cell, CellKind};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::engine::{CellResult, OutputView, PyState};
use crate::session::RefView;

/// What the report settings dialog edits: kept in the meta cell's `attrs.report`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ReportSettings {
    pub title: TitleBlock,
    pub options: Options,
}

/// The office's report style (Preferences): the same for every document on this computer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ReportStyle {
    pub company: String,
    /// A PNG, JPEG or SVG file shown in the page header instead of the company name.
    pub logo_path: Option<String>,
    pub accent: String,
    /// `us-letter` or `a4`.
    pub paper: String,
}

impl Default for ReportStyle {
    fn default() -> Self {
        let h = HouseStyle::default();
        ReportStyle { company: String::new(), logo_path: None, accent: h.accent, paper: h.paper }
    }
}

/// The meta cell's report settings, or the defaults.
pub fn settings_of(cells: &[Cell]) -> ReportSettings {
    cells
        .iter()
        .find(|c| c.kind == CellKind::Meta)
        .and_then(|c| c.attrs.get("report"))
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default()
}

/// Everything about the document the report needs besides its cells and results.
pub struct DocFacts {
    pub name: String,
    pub id: String,
    pub version: u64,
    /// When the version was committed, milliseconds since the epoch.
    pub committed_ms: i64,
    pub refs: Vec<RefView>,
    pub python_env: Option<String>,
}

/// `2026-11-20T15:04:05Z` from milliseconds since the epoch (UTC; no time zone database).
pub fn iso_utc(ms: i64) -> String {
    let secs = ms.div_euclid(1000);
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    // Civil from days (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", rem / 3600, rem % 3600 / 60, rem % 60)
}

/// SHA-256 over every cell's id, kind and source, in document order.
pub fn content_hash(cells: &[Cell]) -> String {
    let mut h = Sha256::new();
    for c in cells {
        h.update(c.id.as_bytes());
        h.update([0]);
        h.update(c.kind.as_str().as_bytes());
        h.update([0]);
        h.update(c.source.as_bytes());
        h.update([0xff]);
    }
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// A prose cell (ProseMirror JSON, read by cruncha-prose) as a report block; `None` for an
/// empty paragraph.
fn prose(source: &str) -> Option<Block> {
    prose_block(&cruncha_prose::parse_json(source))
}

fn prose_block(b: &cruncha_prose::Block) -> Option<Block> {
    use cruncha_prose::Block as P;
    let blocks = |bs: &[P]| bs.iter().filter_map(prose_block).collect::<Vec<_>>();
    Some(match b {
        P::Heading { level, content } => Block::Heading { level: *level, text: runs(content) },
        P::Paragraph(content) => {
            let text = runs(content);
            if text.iter().all(|r| r.text.trim().is_empty()) {
                return None;
            }
            Block::Paragraph { text }
        }
        P::BulletList(items) => Block::List { ordered: false, start: 1, items: items.iter().map(|i| blocks(i)).collect() },
        P::OrderedList { start, items } => {
            Block::List { ordered: true, start: *start, items: items.iter().map(|i| blocks(i)).collect() }
        }
        P::Quote(bs) => Block::Quote { blocks: blocks(bs) },
        P::Rule => Block::Rule,
    })
}

fn runs(content: &[cruncha_prose::Inline]) -> Vec<Run> {
    content
        .iter()
        .map(|n| match n {
            cruncha_prose::Inline::Break => Run { text: "\n".into(), ..Default::default() },
            cruncha_prose::Inline::Text { text, marks } => Run {
                text: text.clone(),
                bold: marks.strong,
                italic: marks.em,
                code: marks.code,
                sub: marks.sub,
                sup: marks.sup,
                link: marks.link.clone(),
            },
        })
        .collect()
}

/// The formula of a calc that couldn't be evaluated: the prepared statement's TeX up to the
/// echo or the result placeholder, so the report still shows what the calc is.
fn formula_tex(source: &str) -> Option<String> {
    let opts = cruncha_eval::Options { unit_aware: cruncha_eval::UnitAware::Always, ..Default::default() };
    let stmt = cruncha_eval::prepare(source, &opts).ok()?;
    let t = stmt.template_tex();
    let cut = t.find(" = \\textcolor{#0000ff}").or_else(|| t.rfind(" = ")).unwrap_or(t.len());
    let f = t[..cut].trim();
    (!f.is_empty() && !f.contains('〖')).then(|| f.to_owned())
}

/// The report's note for output read from stored results that weren't reproduced here:
/// computed on another computer (ADR-0018) or with another Python environment (ADR-0019).
fn stored_note(f: &crate::engine::StoredFrom) -> Option<String> {
    let date = &iso_utc(f.at_us / 1000)[..10];
    let host = f.by.rsplit_once('@').map_or("another computer", |(_, h)| h);
    match (f.elsewhere, f.other_env) {
        (false, false) => None,
        (true, false) => Some(format!("Computed on {host}, {date}; not run on this computer (stored result)")),
        (false, true) => Some(format!(
            "Computed {date} with a different Python environment ({}); not run with this one (stored result)",
            f.env
        )),
        (true, true) => Some(format!(
            "Computed on {host}, {date}, with a different Python environment ({}); not run on this computer (stored result)",
            f.env
        )),
    }
}

/// Errors that mean "not computed" rather than "wrong": inputs from Python that aren't there.
fn not_calculated(error: &str) -> bool {
    error.contains("comes from a Python cell")
}

fn calc_status(r: &CellResult) -> Status {
    if r.pending {
        return Status::NotCalculated { reason: "waiting for a Python cell".into() };
    }
    match (&r.error, r.check) {
        (Some(e), _) if not_calculated(e) => Status::NotCalculated { reason: e.clone() },
        (Some(e), _) => Status::Error { message: e.clone() },
        (None, Some(true)) => Status::Pass,
        (None, Some(false)) => Status::Fail,
        _ => Status::Ok,
    }
}

fn python_status(r: &CellResult) -> Status {
    let Some(py) = &r.python else {
        return Status::NotCalculated { reason: "it hasn't run".into() };
    };
    let reason = |s: &str| Status::NotCalculated { reason: py.note.clone().unwrap_or_else(|| s.into()) };
    match py.state {
        PyState::Done => Status::Ok,
        PyState::Error => Status::Error {
            message: py
                .error
                .as_ref()
                .map(|e| format!("{}: {}", e.kind, e.message))
                .or_else(|| py.note.clone())
                .unwrap_or_else(|| "it failed".into()),
        },
        PyState::NoPython => reason("Python isn't set up on this computer"),
        PyState::NoRust => reason("Rust isn't set up on this computer"),
        PyState::Past => reason("no stored result for this version"),
        PyState::Untrusted => reason("this document isn't trusted to run code here"),
        PyState::Edited => reason("edited since it last ran: press Run"),
        PyState::Blocked => reason("an input isn't available"),
        PyState::Queued | PyState::Running => reason("it was still running when the report was made"),
        PyState::Interrupted => reason("it was stopped"),
    }
}

/// `data:<mime>;base64,<data>` → (mime, bytes).
fn decode_data_url(url: &str) -> Option<(String, Vec<u8>)> {
    let (head, data) = url.strip_prefix("data:")?.split_once(";base64,")?;
    Some((head.to_owned(), base64::engine::general_purpose::STANDARD.decode(data).ok()?))
}

/// The report, and the files it refers to (figures, the logo).
///
/// `figure_svg` gives a figure's cached static rendering (ADR-0017 §4) by MIME type and spec.
pub fn build(
    cells: &[Cell],
    results: &HashMap<String, CellResult>,
    facts: &DocFacts,
    style: &ReportStyle,
    figure_svg: &dyn Fn(&str, &str) -> Option<Vec<u8>>,
) -> (Report, Vec<(String, Vec<u8>)>) {
    let settings = settings_of(cells);
    let mut files = Vec::new();
    let mut blocks = Vec::new();
    let empty = CellResult::default();
    for c in cells.iter().filter(|c| c.parent_id.is_none()) {
        let r = results.get(&c.id).unwrap_or(&empty);
        match c.kind {
            CellKind::Meta => {}
            CellKind::Prose => blocks.extend(prose(&c.source)),
            CellKind::Calc => {
                let tex = if r.tex.is_empty() { formula_tex(&c.source).unwrap_or_default() } else { r.tex.clone() };
                let math = if tex.is_empty() {
                    String::new()
                } else {
                    cruncha_report::tex::to_typst(&tex).unwrap_or_else(|e| {
                        eprintln!("cruncha: report: calc {} TeX not converted: {e}", c.id);
                        String::new()
                    })
                };
                blocks.push(Block::Calc {
                    id: c.id.clone(),
                    math,
                    description: c.attrs["description"].as_str().unwrap_or_default().to_owned(),
                    status: calc_status(r),
                    text: if r.text.is_empty() { c.source.clone() } else { r.text.clone() },
                    check: r.check.is_some() || c.source.trim_start().starts_with("@test")
                        || c.source.trim_start().starts_with("@@test"),
                });
            }
            CellKind::Python | CellKind::Rust => {
                let mut outputs = Vec::new();
                if let Some(py) = &r.python {
                    for (i, o) in py.outputs.iter().enumerate() {
                        match o {
                            OutputView::Image { src, .. } => {
                                let data = src.split_once(";base64,").map(|(_, d)| d).unwrap_or_default();
                                if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(data) {
                                    let path = format!("/img/{}-{i}.png", c.id);
                                    files.push((path.clone(), bytes));
                                    outputs.push(Output::Image { path });
                                }
                            }
                            OutputView::Table { columns, rows, total_rows } => outputs.push(Output::Table {
                                columns: columns.clone(),
                                rows: rows
                                    .iter()
                                    .map(|row| row.iter().map(|v| v.clone().unwrap_or_default()).collect())
                                    .collect(),
                                total_rows: *total_rows as usize,
                            }),
                            OutputView::Text { text } => {
                                outputs.push(Output::Text { text: text.clone(), stream: "result".into() })
                            }
                            OutputView::Figure { mime, spec, summary, static_src, .. } => {
                                // The cell's own static image, else the app's rendering; never left out.
                                let own = static_src.as_deref().and_then(decode_data_url);
                                let (ext, bytes) = match own {
                                    Some((m, b)) => (if m == "image/svg+xml" { "svg" } else { "png" }, Some(b)),
                                    None => ("svg", figure_svg(mime, spec)),
                                };
                                match bytes {
                                    Some(bytes) => {
                                        let path = format!("/img/{}-{i}.{ext}", c.id);
                                        files.push((path.clone(), bytes));
                                        outputs.push(Output::Figure { path: Some(path), summary: summary.clone(), reason: None });
                                    }
                                    None => outputs.push(Output::Figure {
                                        path: None,
                                        summary: summary.clone(),
                                        reason: Some("the app hasn't drawn it for a report yet (export the report from the app)".into()),
                                    }),
                                }
                            }
                        }
                    }
                    if !py.stdout.trim().is_empty() {
                        outputs.push(Output::Text { text: py.stdout.trim_end().to_owned(), stream: "stdout".into() });
                    }
                }
                let defines = r
                    .values
                    .iter()
                    .map(|v| {
                        let shown = v
                            .summary
                            .clone()
                            .or_else(|| r.text.lines().find(|l| l.starts_with(&format!("{} =", v.name))).map(str::to_owned))
                            .unwrap_or_default();
                        (v.name.clone(), shown)
                    })
                    .collect();
                blocks.push(Block::Code {
                    id: c.id.clone(),
                    lang: if c.kind == CellKind::Rust { "rust" } else { "python" }.into(),
                    source: c.source.clone(),
                    status: python_status(r),
                    outputs,
                    defines,
                    provenance: r.python.as_ref().and_then(|p| p.stored.as_ref()).and_then(stored_note),
                });
            }
            k => blocks.push(Block::Other { kind: k.as_str().to_owned(), source: c.source.clone() }),
        }
    }

    // Revisions: one per snapshot, oldest first.
    let mut refs = facts.refs.clone();
    refs.sort_by_key(|r| r.version);
    let revisions = refs
        .iter()
        .map(|r| Revision {
            label: r.name.clone(),
            date: iso_utc(r.created_at_ms)[..10].to_owned(),
            description: r.message.clone(),
            by: r.actor.strip_prefix("user:").unwrap_or(&r.actor).to_owned(),
        })
        .collect();

    let mut house = HouseStyle {
        company: style.company.clone(),
        accent: if style.accent.starts_with('#') { style.accent.clone() } else { HouseStyle::default().accent },
        paper: if style.paper == "a4" { "a4".into() } else { "us-letter".into() },
        ..HouseStyle::default()
    };
    if let Some(path) = &style.logo_path {
        match std::fs::read(path) {
            Ok(bytes) => {
                let ext = std::path::Path::new(path)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("png")
                    .to_ascii_lowercase();
                let name = format!("/logo.{ext}");
                files.push((name.clone(), bytes));
                house.logo = Some(name);
            }
            Err(e) => eprintln!("cruncha: report: logo {path} not read: {e}"),
        }
    }

    let report = Report {
        title: settings.title,
        revisions,
        style: house,
        options: settings.options,
        blocks,
        provenance: Provenance {
            document_name: facts.name.clone(),
            document_id: facts.id.clone(),
            version: facts.version,
            committed_at: iso_utc(facts.committed_ms),
            content_hash: content_hash(cells),
            cruncha_version: env!("CARGO_PKG_VERSION").into(),
            python_env: facts.python_env.clone(),
            snapshot: facts.refs.iter().find(|r| r.version == facts.version).map(|r| r.name.clone()),
        },
    };
    (report, files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_and_prose() {
        assert_eq!(iso_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(iso_utc(1_795_183_770_909), "2026-11-20T14:09:30Z");
        let Some(Block::Heading { level, text }) =
            prose(r#"{"type":"heading","attrs":{"level":2},"content":[{"type":"text","text":"Loads"}]}"#)
        else {
            panic!("a heading")
        };
        assert_eq!((level, text[0].text.as_str()), (2, "Loads"));
        let Some(Block::Paragraph { text }) = prose(
            r#"{"type":"paragraph","content":[{"type":"text","text":"a "},{"type":"text","marks":[{"type":"strong"}],"text":"b"}]}"#,
        ) else {
            panic!("a paragraph")
        };
        assert!(text[1].bold && !text[0].bold);
        assert_eq!(prose(r#"{"type":"paragraph"}"#), None);
        let md = "1. [AISC 360](https://www.aisc.org) table J3.2\n2. f<sub>c</sub>\n\n   > quoted";
        let Some(cruncha_prose::Piece::Prose(list)) = cruncha_prose::from_markdown(md).into_iter().next() else {
            panic!("prose")
        };
        let Some(Block::List { ordered: true, start: 1, items }) = prose(&cruncha_prose::to_json(&list)) else {
            panic!("a list")
        };
        let Block::Paragraph { text } = &items[0][0] else { panic!() };
        assert_eq!(text[0].link.as_deref(), Some("https://www.aisc.org"));
        assert!(matches!(&items[1][1], Block::Quote { .. }));
    }
}
