// SPDX-License-Identifier: GPL-3.0-or-later

//! The MCP tools (`skills/cruncha-core`). Each returns text an agent can quote: cell IDs,
//! sources and the engine's results, verbatim.

use std::collections::{BTreeSet, HashMap};

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock, ServerCapabilities, ServerConfig};
use rmcp::{ErrorData, ServerHandler, schemars, tool, tool_handler, tool_router};
use serde::Deserialize;

use crate::mirror::{self, Range};
use crate::{Backend, DocSnapshot, Edit};

const INSTRUCTIONS: &str = "Tools for the Cruncha calculation document open in the app. \
The engine computes every number: quote results from these tools with their cell IDs and \
never compute values yourself. Start with read_doc. Load the cruncha-core skill for the rules.";

#[derive(Debug, Default, Deserialize, schemars::JsonSchema)]
pub struct ReadDocRequest {
    /// Start at this cell ID (inclusive). Omit to start at the beginning.
    pub from: Option<String>,
    /// At most this many cells. Omit for the whole document.
    pub limit: Option<u32>,
}

#[derive(Debug, Default, Deserialize, schemars::JsonSchema)]
pub struct RunRequest {
    /// Cell IDs to report. Omit for every cell with a result.
    pub cell_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ExplainRequest {
    /// The cell to explain.
    pub cell_id: String,
}

#[derive(Debug, Default, Deserialize, schemars::JsonSchema)]
pub struct HistoryRequest {
    /// Only versions where this cell changed.
    pub cell_id: Option<String>,
    /// Also list what changed from this version to the latest.
    pub since: Option<u64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ProposeRequest {
    /// What the change does and why, for the engineer who reviews it (one or two sentences).
    pub message: String,
    /// The changes, applied in order as one unit: the reviewer accepts or rejects all of them.
    pub changes: Vec<Edit>,
    /// The document version you read (from read_doc's header). A cell changed after it is
    /// refused, so you re-read instead of overwriting someone's edit.
    pub base_version: Option<u64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WithdrawRequest {
    /// The proposal to withdraw (one of yours that is still open).
    pub proposal_id: String,
}

/// The tool server for one app; cheap to clone (one per MCP session).
#[derive(Clone)]
pub struct CrunchaTools<B: Backend> {
    backend: B,
    tool_router: ToolRouter<Self>,
}

impl<B: Backend> CrunchaTools<B> {
    pub fn new(backend: B) -> Self {
        CrunchaTools { backend, tool_router: Self::tool_router() }
    }
}

fn ok(text: String) -> Result<CallToolResult, ErrorData> {
    Ok(CallToolResult::success(vec![ContentBlock::text(text)]))
}

fn fail(text: String) -> Result<CallToolResult, ErrorData> {
    Ok(CallToolResult::error(vec![ContentBlock::text(text)]))
}

/// Which cell defines each name.
fn definers(doc: &DocSnapshot) -> HashMap<&str, &str> {
    let mut m = HashMap::new();
    for c in &doc.cells {
        for d in &c.defines {
            m.entry(d.as_str()).or_insert(c.id.as_str());
        }
    }
    m
}

#[tool_router]
impl<B: Backend> CrunchaTools<B> {
    #[tool(description = "The document as Markdown (the text mirror): headings and prose, every cell \
        with its ID in the fence ({#ID}), and the engine's result on a `⇒` line. Start here.")]
    async fn read_doc(&self, Parameters(req): Parameters<ReadDocRequest>) -> Result<CallToolResult, ErrorData> {
        match self.backend.snapshot(false).await {
            Ok(doc) => ok(mirror::render(
                &doc,
                &Range { from: req.from, limit: req.limit.map(|l| l as usize) },
            )),
            Err(e) => fail(e),
        }
    }

    #[tool(description = "The engine's current results. The document recalculates by itself \
        when it changes; this waits (up to two minutes) for Python cells that are running, then \
        returns the result, error or check outcome of each cell (or of the given cells).")]
    async fn run(&self, Parameters(req): Parameters<RunRequest>) -> Result<CallToolResult, ErrorData> {
        let doc = match self.backend.snapshot(true).await {
            Ok(d) => d,
            Err(e) => return fail(e),
        };
        let wanted: Option<BTreeSet<&str>> = req.cell_ids.as_ref().map(|v| v.iter().map(String::as_str).collect());
        let mut out = format!("Results of version {}:\n\n", doc.version);
        let mut found = 0;
        for c in &doc.cells {
            if wanted.as_ref().is_some_and(|w| !w.contains(c.id.as_str())) {
                continue;
            }
            let Some(r) = doc.results.get(&c.id) else { continue };
            let lines = mirror::result_lines(r);
            if lines.is_empty() {
                continue;
            }
            found += 1;
            out.push_str(&format!("{} ({}):\n{lines}\n", c.id, c.kind));
        }
        if let Some(w) = &wanted {
            for id in w {
                if !doc.cells.iter().any(|c| c.id == *id) {
                    out.push_str(&format!("{id}: no such cell\n"));
                }
            }
        }
        if found == 0 {
            out.push_str("(no results)\n");
        }
        ok(out)
    }

    #[tool(description = "One cell in depth: its source, what it defines and reads, where each \
        input is defined and its current value, which cells use it, and its result.")]
    async fn explain(&self, Parameters(req): Parameters<ExplainRequest>) -> Result<CallToolResult, ErrorData> {
        let doc = match self.backend.snapshot(false).await {
            Ok(d) => d,
            Err(e) => return fail(e),
        };
        let Some(c) = doc.cells.iter().find(|c| c.id == req.cell_id) else {
            return fail(format!("no cell {} in this document", req.cell_id));
        };
        let defs = definers(&doc);
        let empty = Default::default();
        let r = doc.results.get(&c.id).unwrap_or(&empty);
        let mut out = format!("Cell {} ({}, provenance {})\n", c.id, c.kind, c.provenance);
        if !c.description.is_empty() {
            out.push_str(&format!("Description: {}\n", c.description));
        }
        out.push_str(&format!("Source:\n```\n{}\n```\n", c.source.trim_end()));
        out.push_str(&format!("Result:\n{}", mirror::result_lines(r)));
        if !c.defines.is_empty() {
            out.push_str(&format!("Defines: {}\n", c.defines.join(", ")));
        }
        if !c.reads.is_empty() {
            out.push_str("Reads:\n");
            for name in &c.reads {
                match defs.get(name.as_str()) {
                    Some(id) => {
                        let value = doc
                            .results
                            .get(*id)
                            .and_then(|r| r.values.iter().find(|(n, _)| n == name).map(|(_, v)| v.clone()))
                            .unwrap_or_else(|| "(no value)".into());
                        out.push_str(&format!("- {name}: defined in {id}, = {value}\n"));
                    }
                    None => out.push_str(&format!("- {name}: NOT DEFINED in this document\n")),
                }
            }
        }
        let users: Vec<&str> = doc
            .cells
            .iter()
            .filter(|u| c.defines.iter().any(|d| u.reads.contains(d)))
            .map(|u| u.id.as_str())
            .collect();
        if !users.is_empty() {
            out.push_str(&format!("Used by: {}\n", users.join(", ")));
        }
        ok(out)
    }

    #[tool(description = "The document's problems: errors, failing @test checks, cells not \
        calculated (and why), lints such as names used above their definition, names read but \
        never defined, and unfilled placeholders (names ending in _).")]
    async fn check(&self) -> Result<CallToolResult, ErrorData> {
        let doc = match self.backend.snapshot(false).await {
            Ok(d) => d,
            Err(e) => return fail(e),
        };
        let defs = definers(&doc);
        let mut findings: Vec<String> = Vec::new();
        let (mut passed, mut tests) = (0, 0);
        let empty = Default::default();
        for c in &doc.cells {
            let r = doc.results.get(&c.id).unwrap_or(&empty);
            if let Some(holds) = r.check {
                tests += 1;
                if holds {
                    passed += 1;
                } else {
                    findings.push(format!("FAILING CHECK {}: {}", c.id, r.text));
                }
            }
            if let Some(e) = &r.error {
                findings.push(format!("ERROR {}: {e}", c.id));
            } else if r.pending {
                findings.push(format!("NOT CALCULATED {}: waiting for a Python cell", c.id));
            } else if let Some(s) = r.python_state.as_deref().filter(|s| *s != "done" && *s != "error") {
                findings.push(format!(
                    "NOT CALCULATED {} (python): {s}{}",
                    c.id,
                    r.python_note.as_deref().map(|n| format!(", {n}")).unwrap_or_default()
                ));
            }
            for w in &r.warnings {
                findings.push(format!("LINT {}: {w}", c.id));
            }
            for name in &c.reads {
                if !defs.contains_key(name.as_str()) && r.error.is_none() {
                    findings.push(format!("UNDEFINED {}: reads {name}, which no cell defines", c.id));
                }
            }
            for d in &c.defines {
                if d.ends_with('_') {
                    findings.push(format!("PLACEHOLDER {}: {d} is a placeholder value to fill in", c.id));
                }
            }
            if c.provenance == "ai:proposed" {
                findings.push(format!("PROPOSED {}: not yet accepted by a human", c.id));
            }
        }
        let mut out = format!(
            "Version {}: {passed} of {tests} checks hold; {} finding{}.\n",
            doc.version,
            findings.len(),
            if findings.len() == 1 { "" } else { "s" }
        );
        for f in findings {
            out.push_str("- ");
            out.push_str(&f);
            out.push('\n');
        }
        ok(out)
    }

    #[tool(description = "Versions (newest first) with actor, time, message and snapshots. \
        With cell_id, only versions where that cell changed; with since, also the cells that \
        changed from that version to the latest, with their sources before and after.")]
    async fn history(&self, Parameters(req): Parameters<HistoryRequest>) -> Result<CallToolResult, ErrorData> {
        let versions = match self.backend.history().await {
            Ok(v) => v,
            Err(e) => return fail(e),
        };
        let latest = versions.iter().map(|v| v.version).max().unwrap_or(0);
        let mut out = String::new();
        let mut listed = 0;
        for (i, v) in versions.iter().enumerate().rev() {
            if let Some(cell) = &req.cell_id {
                let prev = if i == 0 { continue } else { versions[i - 1].version };
                match self.backend.diff(prev, v.version).await {
                    Ok(d) if d.iter().any(|c| &c.id == cell) => {}
                    _ => continue,
                }
            }
            listed += 1;
            if listed > 200 {
                out.push_str("… older versions omitted\n");
                break;
            }
            let snaps = if v.snapshots.is_empty() { String::new() } else { format!(" [snapshot: {}]", v.snapshots.join(", ")) };
            out.push_str(&format!("v{} {} {} ({}): {}{snaps}\n", v.version, v.at, v.actor, v.kind, v.message));
        }
        if listed == 0 {
            out.push_str("(no matching versions)\n");
        }
        if let Some(since) = req.since {
            match self.backend.diff(since, latest).await {
                Ok(changes) => {
                    out.push_str(&format!("\nChanges from v{since} to v{latest}: {}\n", changes.len()));
                    for c in changes {
                        out.push_str(&format!("- {} {} ({})\n", c.change, c.id, c.kind));
                        if let Some(b) = c.before {
                            out.push_str(&format!("  before: {}\n", b.trim_end().replace('\n', "\n          ")));
                        }
                        if let Some(a) = c.after {
                            out.push_str(&format!("  after:  {}\n", a.trim_end().replace('\n', "\n          ")));
                        }
                    }
                }
                Err(e) => out.push_str(&format!("\nChanges since v{since}: {e}\n")),
            }
        }
        ok(out)
    }

    #[tool(description = "Propose a change to the document: insert, modify or delete cells, as one \
        unit a human accepts or rejects in the app. Nothing changes until they accept. The reply \
        has the proposal id, the new cells' ids, and every result the change would alter \
        (\"if accepted\"), computed by the engine. One coherent proposal per logical change.")]
    async fn propose_edit(&self, Parameters(req): Parameters<ProposeRequest>) -> Result<CallToolResult, ErrorData> {
        match self.backend.propose(req.base_version, req.message, req.changes).await {
            Ok(p) => ok(format!(
                "Proposal {} recorded; it waits for the engineer to accept or reject it in Cruncha. \
                 Don't describe it as applied.\n\n{}",
                p.id,
                mirror::proposal_text(&p)
            )),
            Err(e) => fail(e),
        }
    }

    #[tool(description = "The document's open proposals (from any agent) and your decided ones, \
        with status, the reviewer's note on rejections, and \"if accepted\" results.")]
    async fn proposals(&self) -> Result<CallToolResult, ErrorData> {
        match self.backend.proposals().await {
            Ok(list) if list.is_empty() => ok("No proposals.".into()),
            Ok(list) => ok(list.iter().map(mirror::proposal_text).collect::<Vec<_>>().join("\n")),
            Err(e) => fail(e),
        }
    }

    #[tool(description = "Withdraw one of your open proposals (for example to replace it with a better one).")]
    async fn withdraw(&self, Parameters(req): Parameters<WithdrawRequest>) -> Result<CallToolResult, ErrorData> {
        match self.backend.withdraw(req.proposal_id).await {
            Ok(p) => ok(format!("Proposal {} withdrawn.", p.id)),
            Err(e) => fail(e),
        }
    }
}

#[tool_handler(router = self.tool_router)]
impl<B: Backend> ServerHandler for CrunchaTools<B> {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build()).with_instructions(INSTRUCTIONS)
    }
}
