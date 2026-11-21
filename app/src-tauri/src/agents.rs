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

use cruncha_mcp::{Backend, CellChange, CellView, DocSnapshot, Edit, Effect, ProposalInfo, ResultView, VersionInfo};
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
pub struct AppBackend(pub Arc<Mutex<Session>>, pub String);

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
    }
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

impl Backend for AppBackend {
    async fn snapshot(&self, wait: bool) -> Result<DocSnapshot, String> {
        if wait {
            let start = Instant::now();
            while self.0.lock().await.python_busy().await? && start.elapsed() < Duration::from_secs(120) {
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
        }
        let mut s = self.0.lock().await;
        if !s.is_open() {
            return Err("no document is open in Cruncha".into());
        }
        let cells = s.cells(None).await?;
        let results = s.results(&Default::default()).await?;
        let info = s.info().await?;
        Ok(DocSnapshot {
            name: info.name,
            id: info.id,
            version: info.version,
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
            results: results.iter().map(|(id, r)| (id.clone(), result_view(r))).collect::<HashMap<_, _>>(),
        })
    }

    async fn history(&self) -> Result<Vec<VersionInfo>, String> {
        let s = self.0.lock().await;
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
        let s = self.0.lock().await;
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
        let mut s = self.0.lock().await;
        if !s.is_open() {
            return Err("no document is open in Cruncha".into());
        }
        s.propose(&self.1, base_version, &message, edits).await.map(proposal_info)
    }

    async fn proposals(&self) -> Result<Vec<ProposalInfo>, String> {
        let mut s = self.0.lock().await;
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
        let mut s = self.0.lock().await;
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
        let backend = AppBackend(Arc::new(Mutex::new(session)), "ai:test".into());
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
}
