// SPDX-License-Identifier: GPL-3.0-or-later

//! The text mirror (design 02 §5.4): the document as Markdown with cell IDs and the engine's
//! results (`⇒` lines). It is what `read_doc` returns, and what agents quote.
//!
//! ````markdown
//! ## Beam check
//!
//! ```calc {#01JC3… description="Midspan moment"}
//! M = w L^2 / 8 = ?? kip·ft
//! ```
//! > ⇒ **M = w L^2 / 8 = 108 kip·ft**
//! ````

use crate::{CellView, DocSnapshot, ProposalInfo, ResultView};

/// Which cells to render.
#[derive(Debug, Clone, Default)]
pub struct Range {
    /// Start at this cell (inclusive).
    pub from: Option<String>,
    /// At most this many cells.
    pub limit: Option<usize>,
}

/// The document (or a range of it) as mirror text.
pub fn render(doc: &DocSnapshot, range: &Range) -> String {
    let mut out = format!(
        "<!-- cruncha document “{}” · version {} · {} cells · id {} -->\n\n",
        doc.name,
        doc.version,
        doc.cells.iter().filter(|c| c.kind != "meta").count(),
        doc.id
    );
    let start = range
        .from
        .as_ref()
        .and_then(|id| doc.cells.iter().position(|c| &c.id == id))
        .unwrap_or(0);
    let empty = ResultView::default();
    for (shown, c) in doc.cells.iter().skip(start).filter(|c| c.kind != "meta").enumerate() {
        if range.limit.is_some_and(|l| shown >= l) {
            out.push_str(&format!("<!-- … more cells after {}; call read_doc with from -->\n", c.id));
            break;
        }
        let r = doc.results.get(&c.id).unwrap_or(&empty);
        match c.kind.as_str() {
            "prose" => {
                let text = prose_markdown(&c.source);
                if !text.trim().is_empty() {
                    out.push_str(&format!("{text} <!-- #{} -->\n\n", c.id));
                }
            }
            "calc" => {
                out.push_str(&fence("calc", c));
                out.push_str(&c.source);
                out.push_str("\n```\n");
                out.push_str(&result_lines(r));
                out.push('\n');
            }
            kind => {
                out.push_str(&fence(kind, c));
                out.push_str(c.source.trim_end());
                out.push_str("\n```\n");
                out.push_str(&result_lines(r));
                out.push('\n');
            }
        }
    }
    out
}

/// ```` ```calc {#ID description="…" provenance=ai:proposed} ````
fn fence(kind: &str, c: &CellView) -> String {
    let mut attrs = format!("#{}", c.id);
    if !c.description.is_empty() {
        attrs.push_str(&format!(" description={}", quoted(&c.description)));
    }
    if kind != "calc" && !c.defines.is_empty() {
        attrs.push_str(&format!(" defines=[{}]", c.defines.join(", ")));
    }
    if !c.provenance.is_empty() && c.provenance != "human" {
        attrs.push_str(&format!(" provenance={}", c.provenance));
    }
    format!("```{kind} {{{attrs}}}\n")
}

fn quoted(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

/// The `⇒` lines: the engine's result, error, check outcome and lints, verbatim.
pub fn result_lines(r: &ResultView) -> String {
    let mut out = String::new();
    if r.pending {
        out.push_str("> ⇒ not calculated yet: waiting for a Python cell\n");
    } else if let Some(state) = &r.python_state {
        let note = r.python_note.as_deref().map(|n| format!(" ({n})")).unwrap_or_default();
        match state.as_str() {
            "done" => out.push_str("> ⇒ ran\n"),
            "error" => out.push_str(&format!(
                "> ⇒ ERROR: {}{note}\n",
                r.error.as_deref().unwrap_or("the cell failed")
            )),
            s => out.push_str(&format!("> ⇒ not calculated: {s}{note}\n")),
        }
        for (name, value) in &r.values {
            out.push_str(&format!("> ⇒ `{name}` = {value}\n"));
        }
        for o in &r.outputs {
            out.push_str(&format!("> ⇒ output: {o}\n"));
        }
    } else if let Some(e) = &r.error {
        out.push_str(&format!("> ⇒ ERROR: {e}\n"));
    } else if !r.text.is_empty() {
        let mark = match r.check {
            Some(true) => "  ✓ (check holds)",
            Some(false) => "  ✗ (CHECK FAILS)",
            None => "",
        };
        out.push_str(&format!("> ⇒ **{}**{mark}\n", r.text));
    }
    for w in &r.warnings {
        out.push_str(&format!("> ⚠ {w}\n"));
    }
    out
}

/// A proposal: status, changes and "if accepted" results, for `propose_edit` and `proposals`.
pub fn proposal_text(p: &ProposalInfo) -> String {
    let mut out = format!(
        "## Proposal {} ({}{}) by {}, against version {}\n{}\n",
        p.id,
        p.status,
        if p.stale { ", STALE: a cell changed since" } else { "" },
        p.proposed_by,
        p.base_version,
        p.message
    );
    if let Some(v) = p.accepted_version {
        out.push_str(&format!("Accepted in version {v}.\n"));
    }
    if let Some(n) = &p.decision_note {
        out.push_str(&format!("Reviewer's note: {n}\n"));
    }
    out.push_str(&format!("Changes: {}\n", p.changes.join("; ")));
    if !p.effects.is_empty() {
        out.push_str("If accepted:\n");
        for e in &p.effects {
            let line = |r: &Option<ResultView>, none: &str| match r {
                None => format!("> {none}\n"),
                Some(r) => {
                    let l = result_lines(r);
                    if l.is_empty() { "> (no result)\n".into() } else { l }
                }
            };
            out.push_str(&format!("- {} ({})\n  now:\n", e.id, e.kind));
            out.push_str(&indent(&line(&e.before, "(new cell)")));
            out.push_str("  if accepted:\n");
            out.push_str(&indent(&line(&e.after, "(deleted)")));
        }
    }
    out
}

fn indent(s: &str) -> String {
    s.lines().map(|l| format!("  {l}\n")).collect()
}

/// A prose cell (a ProseMirror paragraph or heading as JSON) as Markdown.
pub fn prose_markdown(source: &str) -> String {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(source) else {
        return source.to_owned();
    };
    let mut text = String::new();
    for n in v["content"].as_array().into_iter().flatten() {
        match n["type"].as_str() {
            Some("text") => {
                let mut t = n["text"].as_str().unwrap_or_default().to_owned();
                for m in n["marks"].as_array().into_iter().flatten() {
                    t = match m["type"].as_str() {
                        Some("strong") => format!("**{t}**"),
                        Some("em") => format!("*{t}*"),
                        Some("code") => format!("`{t}`"),
                        _ => t,
                    };
                }
                text.push_str(&t);
            }
            Some("hard_break") => text.push_str("  \n"),
            _ => {}
        }
    }
    if v["type"] == "heading" {
        let level = v["attrs"]["level"].as_u64().unwrap_or(1).clamp(1, 6) as usize;
        format!("{} {text}", "#".repeat(level))
    } else {
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn doc() -> DocSnapshot {
        let cell = |id: &str, kind: &str, source: &str| CellView {
            id: id.into(),
            kind: kind.into(),
            source: source.into(),
            provenance: "human".into(),
            ..Default::default()
        };
        let mut m = cell("M", "calc", "M = w L^2 / 8 = ?? kip·ft");
        m.description = "Midspan moment".into();
        let mut results = HashMap::new();
        results.insert(
            "M".into(),
            ResultView { text: "M = w L^2 / 8 = 108 kip·ft".into(), ..Default::default() },
        );
        results.insert(
            "T".into(),
            ResultView { text: "M < 150 kip·ft, ok ✓".into(), check: Some(true), ..Default::default() },
        );
        DocSnapshot {
            name: "beam".into(),
            id: "D1".into(),
            version: 7,
            cells: vec![
                cell("META", "meta", ""),
                cell("H", "prose", r#"{"type":"heading","attrs":{"level":2},"content":[{"type":"text","text":"Beam"}]}"#),
                m,
                cell("T", "calc", "@test M < 150 'kip·ft'"),
            ],
            results,
        }
    }

    #[test]
    fn mirror_has_ids_results_and_descriptions() {
        let text = render(&doc(), &Range::default());
        assert!(text.contains("version 7 · 3 cells"), "{text}");
        assert!(text.contains("## Beam <!-- #H -->"), "{text}");
        assert!(text.contains("```calc {#M description=\"Midspan moment\"}\nM = w L^2 / 8 = ?? kip·ft\n```\n> ⇒ **M = w L^2 / 8 = 108 kip·ft**"), "{text}");
        assert!(text.contains("✓ (check holds)"), "{text}");
        assert!(!text.contains("META"), "the meta cell isn't part of the mirror");
    }

    #[test]
    fn ranges() {
        let text = render(&doc(), &Range { from: Some("M".into()), limit: Some(1) });
        assert!(!text.contains("## Beam"));
        assert!(text.contains("#M"));
        assert!(text.contains("more cells after T"), "{text}");
    }
}
