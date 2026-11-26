// SPDX-License-Identifier: GPL-3.0-or-later

//! Agents on this computer (ADR-0009, phase 1): the open document through `cruncha-mcp`.
//!
//! The MCP server runs inside the app, which holds the document (the single writer). It
//! starts only when the user allows it, listens on 127.0.0.1 with a fresh token, and stops
//! when the user turns it off or the app quits. Agents can read the document, its results,
//! checks and history; edits (proposals) come in the next phase.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use cruncha_mcp::{ArchiveView, ArchivedSaveInfo, ArchivedSnapshot, Backend, CellChange, DocumentInfo, CellView, DocSnapshot, Edit, Effect, ProposalInfo, ResultView, TemplateFile, TemplateInfo, TemplateInsert, VersionInfo};
use cruncha_store::{CellDiff, CellKind};
use serde::Serialize;
use tokio::sync::Mutex;

use crate::engine::{CellResult, OutputView};
use crate::report::iso_utc;
use crate::proposals::ProposalView;
use crate::session::Session;

/// The session, as the MCP tools of one agent see it: the shared session and the agent's
/// actor (`ai:<agent>`), recorded on its proposals.
#[derive(Clone)]
pub struct AppBackend(pub Arc<Mutex<Session>>, pub String, pub Option<String>);

impl AppBackend {
    /// The session, with this agent's document selected: its own (an agent-panel agent works
    /// on the tab it started in) or, for the external server, the active tab (ADR-0024).
    async fn session(&self) -> Result<tokio::sync::MutexGuard<'_, Session>, String> {
        let mut s = self.0.lock().await;
        s.select(self.2.as_deref())?;
        Ok(s)
    }
}

fn template_info(t: crate::library::TemplateSummary) -> TemplateInfo {
    TemplateInfo {
        id: t.id,
        version: t.version,
        title: t.title,
        summary: t.summary,
        library: t.library.into(),
        category: t.category,
        tags: t.tags,
        inputs: t.inputs,
        outputs: t.outputs,
        table: t.is_table,
    }
}

/// The value part of a result line (`M = w L^2 / 8 = 108 kip·ft` → `108 kip·ft`).
fn value_of(text: &str) -> String {
    text.rsplit_once(" = ").map(|(_, v)| v.trim().to_owned()).unwrap_or_default()
}

fn output_summary(o: &OutputView) -> String {
    match o {
        OutputView::Image { width, height, .. } => format!("figure ({width} × {height} px)"),
        OutputView::Table { columns, total_rows, .. } => {
            format!("table, {total_rows} rows × {} columns ({})", columns.len(), columns.join(", "))
        }
        OutputView::Text { text } => {
            let first = text.lines().next().unwrap_or_default();
            format!("text: {}", first.chars().take(200).collect::<String>())
        }
        OutputView::Figure { mime, summary, .. } => format!("figure ({}): {summary}", figure_library(mime)),
    }
}

/// `application/vnd.plotly.v1+json` → `plotly`.
fn figure_library(mime: &str) -> &str {
    match mime {
        "application/vnd.plotly.v1+json" => "plotly",
        "application/vnd.vegalite.v5+json" => "vega-lite",
        m => m,
    }
}

/// Each trace's data range, for `explain` (ADR-0017 §5): `M (scatter, 41 points): x 0 … 20 ft; y 0 … 100 ft·kip`.
fn figure_ranges(o: &OutputView) -> Vec<String> {
    let OutputView::Figure { ranges, .. } = o else { return vec![] };
    let num = |v: &serde_json::Value| {
        v.as_f64().map(|f| if f.fract() == 0.0 && f.abs() < 1e15 { format!("{f:.0}") } else { format!("{f}") }).unwrap_or_default()
    };
    ranges
        .as_array()
        .into_iter()
        .flatten()
        .map(|r| {
            let mut line = format!(
                "{} ({}, {} points)",
                r["trace"].as_str().unwrap_or("trace"),
                r["type"].as_str().unwrap_or("?"),
                r["points"].as_u64().unwrap_or(0)
            );
            let mut axes = vec![];
            for a in ["x", "y", "z"] {
                if let Some([lo, hi]) = r[a].as_array().map(Vec::as_slice) {
                    let unit = r[format!("{a}_unit")].as_str().map(|u| format!(" {u}")).unwrap_or_default();
                    axes.push(format!("{a} {} … {}{unit}", num(lo), num(hi)));
                }
            }
            if !axes.is_empty() {
                line.push_str(": ");
                line.push_str(&axes.join("; "));
            }
            line
        })
        .collect()
}

#[cfg(test)]
pub(crate) fn result_view_for_tests(r: &CellResult) -> ResultView {
    result_view(r)
}

fn result_view(r: &CellResult) -> ResultView {
    let python_state = r.python.as_ref().map(|p| {
        serde_json::to_value(p.state)
            .ok()
            .and_then(|v| v.as_str().map(str::to_owned))
            .unwrap_or_default()
    });
    let single = r.values.len() == 1;
    ResultView {
        text: r.text.clone(),
        error: r
            .error
            .clone()
            .or_else(|| r.python.as_ref().and_then(|p| p.error.as_ref()).map(|e| format!("{}: {}", e.kind, e.message))),
        warnings: r.warnings.clone(),
        check: r.check,
        pending: r.pending,
        python_state,
        python_note: r.python.as_ref().and_then(|p| p.note.clone()),
        values: r
            .values
            .iter()
            .map(|v| {
                let shown = v.summary.clone().unwrap_or_else(|| if single { value_of(&r.text) } else { v.tex.clone() });
                (v.name.clone(), shown)
            })
            .collect(),
        outputs: r
            .python
            .as_ref()
            .map(|p| {
                let mut o: Vec<String> = p.outputs.iter().map(output_summary).collect();
                if !p.stdout.trim().is_empty() {
                    o.push(format!("printed: {}", p.stdout.trim().chars().take(400).collect::<String>()));
                }
                o
            })
            .unwrap_or_default(),
        figure_data: r
            .python
            .as_ref()
            .map(|p| p.outputs.iter().flat_map(figure_ranges).collect())
            .unwrap_or_default(),
    }
}

fn proposal_info(p: ProposalView) -> ProposalInfo {
    ProposalInfo {
        id: p.id,
        status: p.status.as_str().into(),
        proposed_by: p.proposed_by,
        message: p.message,
        base_version: p.base_version,
        changes: p.changes.iter().map(|c| format!("{} {}", c.op, c.id)).collect(),
        stale: p.stale,
        decision_note: p.decision_note,
        accepted_version: p.accepted_version,
        effects: p
            .effects
            .iter()
            .map(|e| Effect {
                id: e.id.clone(),
                kind: e.kind.clone(),
                before: e.before.as_ref().map(result_view),
                after: e.after.as_ref().map(result_view),
            })
            .collect(),
    }
}

/// A document's cells and results as the MCP server shows them.
fn doc_snapshot(
    name: String,
    id: String,
    version: u64,
    cells: &[cruncha_store::Cell],
    results: &HashMap<String, crate::engine::CellResult>,
) -> DocSnapshot {
    DocSnapshot {
        name,
        id,
        version,
        cells: cells
            .iter()
            .filter(|c| c.parent_id.is_none())
            .map(|c| CellView {
                id: c.id.clone(),
                kind: c.kind.as_str().to_owned(),
                source: c.source.clone(),
                description: c.attrs["description"].as_str().unwrap_or_default().to_owned(),
                defines: c.defines.clone(),
                reads: c.reads.clone(),
                provenance: match &c.provenance {
                    cruncha_store::Provenance::Human => "human".into(),
                    cruncha_store::Provenance::AiProposed { .. } => "ai:proposed".into(),
                    cruncha_store::Provenance::AiAccepted { .. } => "ai:accepted".into(),
                    cruncha_store::Provenance::Imported { .. } => "imported".into(),
                },
                modified_by: c.modified_by.clone(),
            })
            .collect(),
        results: results.iter().map(|(id, r)| (id.clone(), result_view(r))).collect(),
    }
}

impl Backend for AppBackend {
    async fn snapshot(&self, wait: bool) -> Result<DocSnapshot, String> {
        if wait {
            let start = Instant::now();
            while self.session().await?.python_busy().await? && start.elapsed() < Duration::from_secs(120) {
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
        }
        let mut s = self.session().await?;
        if !s.is_open() {
            return Err("no document is open in Cruncha".into());
        }
        let cells = s.cells(None).await?;
        let results = s.results(&Default::default()).await?;
        let info = s.info().await?;
        Ok(doc_snapshot(info.name, info.id, info.version, &cells, &results))
    }

    async fn documents(&self) -> Result<Vec<DocumentInfo>, String> {
        let s = self.0.lock().await;
        let docs = s.documents();
        let own = self.2.clone().or_else(|| docs.iter().find(|d| d.active).map(|d| d.id.clone()));
        Ok(docs
            .into_iter()
            .map(|d| DocumentInfo {
                this: own.as_deref() == Some(d.id.as_str()),
                can_run: s.prefs().trusted.contains(&d.id),
                id: d.id,
                name: d.name,
                version: d.version,
                active: d.active,
            })
            .collect())
    }

    async fn for_document(&self, document: String) -> Result<Self, String> {
        let docs = self.0.lock().await.documents();
        if docs.iter().any(|d| d.id == document) {
            return Ok(AppBackend(self.0.clone(), self.1.clone(), Some(document)));
        }
        let bare = |n: &str| n.strip_suffix(".cruncha").unwrap_or(n).to_lowercase();
        let want = bare(&document);
        let matches: Vec<_> = docs.iter().filter(|d| bare(&d.name) == want).collect();
        match matches.as_slice() {
            [d] => Ok(AppBackend(self.0.clone(), self.1.clone(), Some(d.id.clone()))),
            [] => Err(format!(
                "no open document is called “{document}”; open: {}",
                docs.iter().map(|d| format!("{} ({})", d.name, d.id)).collect::<Vec<_>>().join(", ")
            )),
            many => Err(format!(
                "“{document}” matches several open documents: {}; give its ID",
                many.iter().map(|d| format!("{} ({})", d.path, d.id)).collect::<Vec<_>>().join(", ")
            )),
        }
    }

    async fn propose_copy(
        &self,
        from: String,
        cell_ids: Vec<String>,
        after: Option<String>,
        rename: std::collections::BTreeMap<String, String>,
        message: String,
    ) -> Result<ProposalInfo, String> {
        use crate::session::SourceView;
        let source_doc = self.for_document(from).await?.2.expect("a resolved document");
        let mut s = self.session().await?;
        let target = s.info().await?;
        if source_doc == target.id {
            return Err("that's the same document; use propose_edit to add cells to it".into());
        }
        s.select(Some(&source_doc))?;
        let from_info = s.info().await?;
        let cells = s.cells(None).await?;
        let picked = cell_ids
            .iter()
            .map(|id| {
                cells
                    .iter()
                    .find(|c| &c.id == id && c.parent_id.is_none())
                    .cloned()
                    .ok_or_else(|| format!("no cell {id} in {}", from_info.name))
            })
            .collect::<Result<Vec<_>, String>>()?;
        if picked.is_empty() {
            return Err("give at least one cell to copy".into());
        }
        // Prose travels as Markdown (proposals take prose as Markdown); code as it is.
        let mut views = Vec::with_capacity(picked.len());
        for c in &picked {
            let source = match c.kind {
                CellKind::Prose => cruncha_prose::to_markdown(&cruncha_prose::parse_json(&c.source)),
                CellKind::Calc | CellKind::Python | CellKind::Rust => c.source.clone(),
                other => return Err(format!("a {} cell can't be copied by an agent", other.as_str())),
            };
            views.push(SourceView { kind: c.kind.as_str().into(), source });
        }
        let sources = if rename.is_empty() {
            views.iter().map(|v| v.source.clone()).collect()
        } else {
            s.rename_cells(views.clone(), rename).await?
        };
        s.select(Some(&target.id))?;
        let renamed: Vec<SourceView> =
            views.iter().zip(&sources).map(|(v, src)| SourceView { kind: v.kind.clone(), source: src.clone() }).collect();
        let clashes = s.paste_clashes(renamed).await?;
        if !clashes.is_empty() {
            let example: Vec<String> = clashes.iter().map(|c| format!("\"{}\": \"{}\"", c.name, c.suggestion)).collect();
            return Err(format!(
                "{} already defines {}; copy again with rename, e.g. {{{}}}",
                target.name,
                clashes.iter().map(|c| c.name.as_str()).collect::<Vec<_>>().join(", "),
                example.join(", ")
            ));
        }
        // Listed in reverse with the same `after`, they land in order.
        let edits: Vec<Edit> = picked
            .iter()
            .zip(sources)
            .rev()
            .map(|(c, source)| Edit::Insert {
                after: after.clone(),
                kind: c.kind.as_str().into(),
                source,
                description: c.attrs["description"].as_str().map(str::to_owned),
            })
            .collect();
        let message = format!("{} (copied from {} v{})", message.trim(), from_info.name, from_info.version);
        s.propose(&self.1, None, &message, edits).await.map(proposal_info)
    }

    async fn templates(&self) -> Result<Vec<TemplateInfo>, String> {
        let s = self.0.lock().await;
        Ok(s.templates().into_iter().filter(|t| !t.hidden && t.error.is_none()).map(template_info).collect())
    }

    async fn template(&self, id: String) -> Result<TemplateFile, String> {
        let s = self.0.lock().await;
        let t = s.template_by_id(&id)?;
        let text = s.template_text(&t.summary.key)?;
        let tests = t
            .tests
            .iter()
            .map(|x| match (&x.skipped, x.passed) {
                (Some(why), _) => format!("SKIPPED {}: {why}", x.name),
                (None, true) => format!("PASS {}", x.name),
                (None, false) => format!("FAIL {}: {}", x.name, x.lines.join("; ")),
            })
            .collect();
        Ok(TemplateFile { info: template_info(t.summary.clone()), text, problems: t.problems.clone(), tests })
    }

    async fn propose_template(&self, insert: TemplateInsert) -> Result<ProposalInfo, String> {
        let mut s = self.session().await?;
        if !s.is_open() {
            return Err("no document is open in Cruncha".into());
        }
        let (t, edits) = s
            .template_edits(&insert.id, &insert.bindings, &insert.rename, insert.after, insert.row.as_deref(), insert.columns.as_deref())
            .await?;
        let what = match &t.row {
            Some(row) => format!("{} {row}", t.summary.title),
            None => t.summary.title.clone(),
        };
        let message = format!("Insert template {what} {}: {}", t.summary.version, insert.message.trim());
        s.propose(&self.1, None, &message, edits).await.map(proposal_info)
    }

    async fn snapshot_named(&self, name: String) -> Result<DocSnapshot, String> {
        let mut s = self.session().await?;
        if !s.is_open() {
            return Err("no document is open in Cruncha".into());
        }
        let info = s.info().await?;
        let Some(r) = info.refs.iter().find(|r| r.name == name) else {
            let names: Vec<&str> = info.refs.iter().map(|r| r.name.as_str()).collect();
            return Err(format!("no snapshot “{name}”; the document has: {}", if names.is_empty() { "none".into() } else { names.join(", ") }));
        };
        let version = r.version;
        let cells = s.cells_at_ref(&name).await?;
        let at = crate::session::At { version: None, ref_name: Some(name) };
        let results = s.results(&at).await?;
        Ok(doc_snapshot(info.name, info.id, version, &cells, &results))
    }

    async fn archive(&self, cell: Option<String>) -> Result<ArchiveView, String> {
        let s = self.session().await?;
        if !s.is_open() {
            return Err("no document is open in Cruncha".into());
        }
        let info = s.info().await?;
        let saves = s.archived(cell.as_deref()).await?;
        Ok(ArchiveView {
            snapshots: info
                .refs
                .iter()
                .filter(|r| r.version < info.earliest_version)
                .map(|r| ArchivedSnapshot {
                    name: r.name.clone(),
                    version: r.version,
                    at: iso_utc(r.created_at_ms),
                    message: r.message.clone(),
                })
                .collect(),
            saves: saves
                .into_iter()
                .map(|v| ArchivedSaveInfo {
                    version: v.version,
                    at: iso_utc(v.at_ms),
                    actor: v.actor,
                    kind: v.kind,
                    message: v.message,
                    snapshot: v.snapshot,
                    cells: v.cells,
                })
                .collect(),
        })
    }

    async fn history(&self) -> Result<Vec<VersionInfo>, String> {
        let s = self.session().await?;
        if !s.is_open() {
            return Err("no document is open in Cruncha".into());
        }
        let refs = s.info().await?.refs;
        Ok(s.history()
            .await?
            .into_iter()
            .map(|v| VersionInfo {
                version: v.version,
                at: iso_utc(v.timestamp_ms),
                actor: v.actor,
                message: v.message,
                kind: format!("{:?}", v.kind).to_lowercase(),
                snapshots: refs.iter().filter(|r| r.version == v.version).map(|r| r.name.clone()).collect(),
            })
            .collect())
    }

    async fn diff(&self, from: u64, to: u64) -> Result<Vec<CellChange>, String> {
        let s = self.session().await?;
        let shown = |k: CellKind| k != CellKind::Meta;
        Ok(s.diff(from, to)
            .await?
            .into_iter()
            .filter_map(|d| match d {
                CellDiff::Added(c) if shown(c.kind) => Some(CellChange {
                    id: c.id,
                    kind: c.kind.as_str().into(),
                    change: "added".into(),
                    before: None,
                    after: Some(c.source),
                }),
                CellDiff::Removed(c) if shown(c.kind) => Some(CellChange {
                    id: c.id,
                    kind: c.kind.as_str().into(),
                    change: "removed".into(),
                    before: Some(c.source),
                    after: None,
                }),
                CellDiff::Modified { before, after } if shown(after.kind) => Some(CellChange {
                    id: after.id,
                    kind: after.kind.as_str().into(),
                    change: "modified".into(),
                    before: Some(before.source),
                    after: Some(after.source),
                }),
                _ => None,
            })
            .collect())
    }

    async fn propose(&self, base_version: Option<u64>, message: String, edits: Vec<Edit>) -> Result<ProposalInfo, String> {
        let mut s = self.session().await?;
        if !s.is_open() {
            return Err("no document is open in Cruncha".into());
        }
        s.propose(&self.1, base_version, &message, edits).await.map(proposal_info)
    }

    async fn proposals(&self) -> Result<Vec<ProposalInfo>, String> {
        let mut s = self.session().await?;
        if !s.is_open() {
            return Err("no document is open in Cruncha".into());
        }
        let mut list: Vec<ProposalInfo> = s
            .proposals()
            .await?
            .into_iter()
            .filter(|p| p.status == cruncha_store::ProposalStatus::Open || p.proposed_by == self.1)
            .map(proposal_info)
            .collect();
        list.reverse(); // oldest first, newest last
        Ok(list)
    }

    async fn withdraw(&self, id: String) -> Result<ProposalInfo, String> {
        let mut s = self.session().await?;
        s.withdraw_proposal(&self.1, &id).await.map(proposal_info)
    }
}

/// What the Agents dialog shows while the server runs.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentServerView {
    pub url: String,
    pub token: String,
    /// `claude mcp add …`
    pub claude_command: String,
    /// An `mcpServers` JSON block for other agents.
    pub config_json: String,
}

impl From<&cruncha_mcp::http::Server> for AgentServerView {
    fn from(s: &cruncha_mcp::http::Server) -> Self {
        AgentServerView {
            url: s.url.clone(),
            token: s.token.clone(),
            claude_command: s.claude_command(),
            config_json: s.config_json(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_from_result_lines() {
        assert_eq!(value_of("M = w L^2 / 8 = 108 kip·ft"), "108 kip·ft");
        assert_eq!(value_of("L = 24 ft"), "24 ft");
        assert_eq!(value_of("nothing"), "");
    }

    /// The MCP tools against a real session: the sample document over HTTP.
    #[tokio::test]
    async fn agents_read_the_sample_document() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("beam.cruncha");
        let mut session = Session::new(None);
        session.create_sample(&path).await.unwrap();
        let backend = AppBackend(Arc::new(Mutex::new(session)), "ai:test".into(), None);
        let doc = backend.snapshot(false).await.unwrap();
        let m = doc.cells.iter().find(|c| c.defines.contains(&"M".to_string())).expect("the sample defines M");
        assert_eq!(doc.results[&m.id].values[0].0, "M");
        assert!(doc.results[&m.id].values[0].1.ends_with("kip·ft"), "{:?}", doc.results[&m.id]);
        let mirror = cruncha_mcp::mirror::render(&doc, &Default::default());
        assert!(mirror.contains(&format!("{{#{}", m.id)), "{mirror}");
        let h = backend.history().await.unwrap();
        assert!(h.iter().any(|v| !v.snapshots.is_empty()), "the sample has a snapshot");
        let changes = backend.diff(0, doc.version).await.unwrap();
        assert!(changes.iter().all(|c| c.change == "added"), "everything was added since v0");
    }

    #[tokio::test]
    async fn agents_work_across_open_documents() {
        let dir = tempfile::tempdir().unwrap();
        let mut session = Session::new(None);
        let a = session.create_sample(&dir.path().join("beam.cruncha")).await.unwrap().id;
        let b = session.create(&dir.path().join("column.cruncha")).await.unwrap().id;
        // B gets its own M, so a copy of A's moment clashes.
        session
            .on(Some(&b))
            .unwrap()
            .commit(vec![crate::session::ChangeView::Upsert { cell: {
                let mut c = cruncha_store::Cell::new(CellKind::Calc, "a0", "M = 5 kip·ft");
                c.id = "BM".into();
                c
            } }], "M")
            .await
            .unwrap();
        // An agent outside Cruncha: the active tab (B) by default.
        let backend = AppBackend(Arc::new(Mutex::new(session)), "ai:test".into(), None);
        let docs = backend.documents().await.unwrap();
        assert_eq!(docs.len(), 2);
        assert!(docs.iter().any(|d| d.id == b && d.this && d.active));
        assert_eq!(backend.snapshot(false).await.unwrap().id, b);

        // By name (with or without .cruncha) or by ID; an unknown one lists what's open.
        let on_a = backend.for_document("beam".into()).await.unwrap();
        assert_eq!(on_a.snapshot(false).await.unwrap().id, a);
        assert_eq!(backend.for_document(a.clone()).await.unwrap().2.as_deref(), Some(a.as_str()));
        assert!(backend.for_document("nope".into()).await.err().unwrap().contains("beam.cruncha"));

        // Copy A's moment cell into B: refused while M clashes, then proposed with a rename.
        let doc_a = on_a.snapshot(false).await.unwrap();
        let m = doc_a.cells.iter().find(|c| c.defines.contains(&"M".to_string())).unwrap().id.clone();
        let err = backend
            .propose_copy("beam".into(), vec![m.clone()], None, Default::default(), "reuse the moment".into())
            .await
            .unwrap_err();
        assert!(err.contains("already defines M") && err.contains("\"M\": \"M2\""), "{err}");
        let p = backend
            .propose_copy(
                "beam".into(),
                vec![m.clone()],
                Some("BM".into()),
                std::collections::BTreeMap::from([("M".to_string(), "M2".to_string())]),
                "reuse the moment".into(),
            )
            .await
            .unwrap();
        assert!(p.message.contains("copied from beam.cruncha"), "{}", p.message);
        let views = backend.0.lock().await.on(Some(&b)).unwrap().proposals().await.unwrap();
        let ins = views[0].changes.iter().find(|c| c.op == "insert").expect("an insert");
        assert!(ins.source.as_deref().unwrap_or_default().starts_with("M2 ="), "{ins:?}");
        assert_eq!(ins.after.as_deref(), Some("BM"));
        assert_eq!(ins.description.as_deref(), Some("Maximum bending moment, at midspan"), "the description travels");
        // It's B's proposal; A is untouched.
        assert_eq!(backend.proposals().await.unwrap().len(), 1);
        assert!(on_a.proposals().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn agents_read_archived_snapshots_and_the_save_log() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("beam.cruncha");
        let mut session = Session::new(None);
        session.create_sample(&path).await.unwrap();
        session.close().await;
        let mut store = cruncha_store::Store::open(&path).await.unwrap();
        let mut c = store.cells().await.unwrap().into_iter().find(|c| c.defines.contains(&"L".to_string())).unwrap();
        c.source = "L = 30 ft".into();
        store.commit(&[cruncha_store::Change::Upsert(c.clone())], "user:t", "longer span").await.unwrap();
        // Rev B's own version stays available; this save before it is archived.
        let mut c2 = c.clone();
        c2.source = "L = 32 ft".into();
        store.commit(&[cruncha_store::Change::Upsert(c2)], "user:t", "longer still").await.unwrap();
        let rev_b = store.tag("Rev B", "", "user:t").await.unwrap().doc_version;
        drop(store);
        session.open(&path).await.unwrap();
        session.archive(rev_b).await.unwrap();
        let backend = AppBackend(Arc::new(Mutex::new(session)), "ai:test".into(), None);

        let a = backend.archive(None).await.unwrap();
        assert_eq!(a.snapshots.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(), ["Rev A"]);
        assert!(a.saves.iter().any(|s| s.message == "longer span" && s.cells == [c.id.clone()]));
        assert_eq!(backend.archive(Some(c.id.clone())).await.unwrap().saves.last().unwrap().message, "longer span");

        // The archived snapshot reads with its results; the latest version is unchanged.
        let rev_a = backend.snapshot_named("Rev A".into()).await.unwrap();
        let latest = backend.snapshot(false).await.unwrap();
        let m = |d: &DocSnapshot| {
            let id = &d.cells.iter().find(|c| c.defines.contains(&"M".to_string())).unwrap().id;
            d.results[id].values[0].1.clone()
        };
        assert_ne!(m(&rev_a), m(&latest), "Rev A has the shorter span");
        assert!(backend.snapshot_named("Rev Z".into()).await.unwrap_err().contains("Rev A"));
    }
}
