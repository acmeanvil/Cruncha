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
use crate::{Backend, DocSnapshot, Edit, TemplateInfo, TemplateInsert};

const INSTRUCTIONS: &str = "Tools for the Cruncha calculation document open in the app. \
The engine computes every number: quote results from these tools with their cell IDs and \
never compute values yourself. Start with read_doc. Load the cruncha-core skill for the rules.";

#[derive(Debug, Default, Deserialize, schemars::JsonSchema)]
pub struct ReadDocRequest {
    /// Start at this cell ID (inclusive). Omit to start at the beginning.
    pub from: Option<String>,
    /// At most this many cells. Omit for the whole document.
    pub limit: Option<u32>,
    /// Read this snapshot (e.g. "Rev A"), read-only, instead of the latest version. Archived
    /// snapshots work too. Edits are always proposed against the latest version.
    pub snapshot: Option<String>,
    /// The open document to act on: its ID or name, as `documents` lists them. Omit for this
    /// conversation's document (the app's active tab for agents outside Cruncha).
    pub document: Option<String>,
}

#[derive(Debug, Default, Deserialize, schemars::JsonSchema)]
pub struct RunRequest {
    /// Cell IDs to report. Omit for every cell with a result.
    pub cell_ids: Option<Vec<String>>,
    /// The open document to act on: its ID or name, as `documents` lists them. Omit for this
    /// conversation's document (the app's active tab for agents outside Cruncha).
    pub document: Option<String>,
}

#[derive(Debug, Default, Deserialize, schemars::JsonSchema)]
pub struct ExplainRequest {
    /// The cell to explain.
    pub cell_id: String,
    /// The open document to act on: its ID or name, as `documents` lists them. Omit for this
    /// conversation's document (the app's active tab for agents outside Cruncha).
    pub document: Option<String>,
}

#[derive(Debug, Default, Deserialize, schemars::JsonSchema)]
pub struct HistoryRequest {
    /// Only versions where this cell changed.
    pub cell_id: Option<String>,
    /// Also list what changed from this version to the latest.
    pub since: Option<u64>,
    /// List the archive instead: snapshots kept from archived or trimmed history, and the
    /// save log (who changed which cells, when) of the saves that were removed.
    pub archived: Option<bool>,
    /// The open document to act on: its ID or name, as `documents` lists them. Omit for this
    /// conversation's document (the app's active tab for agents outside Cruncha).
    pub document: Option<String>,
}

#[derive(Debug, Default, Deserialize, schemars::JsonSchema)]
pub struct DocumentRequest {
    /// The open document to act on: its ID or name, as `documents` lists them. Omit for this
    /// conversation's document (the app's active tab for agents outside Cruncha).
    pub document: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CopyFrom {
    /// The open document to copy from: its ID or name.
    pub document: String,
    /// The cells to copy, in the order they should appear.
    pub cell_ids: Vec<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CopyRequest {
    /// Where the cells come from.
    pub from: CopyFrom,
    /// The open document to copy into: its ID or name. Omit for this conversation's document.
    pub document: Option<String>,
    /// The cell in the target to insert after; omit for the start of the document.
    pub after: Option<String>,
    /// Names to rename in the copied cells (old → new), for names the target already
    /// defines. Applied to definitions and uses, and checked; a rename that can't be done
    /// safely is refused.
    #[serde(default)]
    pub rename: std::collections::BTreeMap<String, String>,
    /// What the copy is for, for the engineer who reviews it.
    pub message: String,
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
    /// The open document to act on: its ID or name, as `documents` lists them. Omit for this
    /// conversation's document (the app's active tab for agents outside Cruncha).
    pub document: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WithdrawRequest {
    /// The proposal to withdraw (one of yours that is still open).
    pub proposal_id: String,
    /// The open document to act on: its ID or name, as `documents` lists them. Omit for this
    /// conversation's document (the app's active tab for agents outside Cruncha).
    pub document: Option<String>,
}

#[derive(Debug, Default, Deserialize, schemars::JsonSchema)]
pub struct TemplatesRequest {
    /// Words to search for in titles, summaries, tags, categories, ids and input/output
    /// names (every word must match). Omit to list every template.
    pub query: Option<String>,
    /// Read this template (its id from the list): the whole `.cruncha.md` file, what
    /// validation found and its worked examples.
    pub id: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ProposeTemplateRequest {
    /// The template's id, as `templates` lists it.
    pub id: String,
    /// Inputs to bind to values the document already has (input → the document's name, e.g.
    /// {"L": "L_span"}, or {"L": "L"} for the same name). The input's definition is left out
    /// and the template's cells read the document's name instead.
    #[serde(default)]
    pub bindings: std::collections::BTreeMap<String, String>,
    /// Other names to rename (old → new), for names the document already defines. Applied to
    /// definitions and uses, and checked.
    #[serde(default)]
    pub rename: std::collections::BTreeMap<String, String>,
    /// The cell to insert after; omit for the start of the document.
    pub after: Option<String>,
    /// Why the template goes here, for the engineer who reviews it.
    pub message: String,
    /// The open document to act on: its ID or name, as `documents` lists them. Omit for this
    /// conversation's document (the app's active tab for agents outside Cruncha).
    pub document: Option<String>,
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

    /// The backend for the named document (ADR-0024), or this connection's own.
    async fn on(&self, document: Option<&str>) -> Result<B, String> {
        match document {
            None => Ok(self.backend.clone()),
            Some(d) => self.backend.for_document(d.to_owned()).await,
        }
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
        let b = match self.on(req.document.as_deref()).await {
            Ok(b) => b,
            Err(e) => return fail(e),
        };
        let range = Range { from: req.from, limit: req.limit.map(|l| l as usize) };
        match &req.snapshot {
            None => match b.snapshot(false).await {
                Ok(doc) => ok(mirror::render(&doc, &range)),
                Err(e) => fail(e),
            },
            Some(name) => match b.snapshot_named(name.clone()).await {
                Ok(doc) => ok(format!(
                    "<!-- Snapshot “{name}” (version {}), read-only. Propose edits against the latest version. -->\n{}",
                    doc.version,
                    mirror::render(&doc, &range)
                )),
                Err(e) => fail(e),
            },
        }
    }

    #[tool(description = "The engine's current results. The document recalculates by itself \
        when it changes; this waits (up to two minutes) for Python cells that are running, then \
        returns the result, error or check outcome of each cell (or of the given cells).")]
    async fn run(&self, Parameters(req): Parameters<RunRequest>) -> Result<CallToolResult, ErrorData> {
        let b = match self.on(req.document.as_deref()).await {
            Ok(b) => b,
            Err(e) => return fail(e),
        };
        let doc = match b.snapshot(true).await {
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
        let b = match self.on(req.document.as_deref()).await {
            Ok(b) => b,
            Err(e) => return fail(e),
        };
        let doc = match b.snapshot(false).await {
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
        if !r.figure_data.is_empty() {
            out.push_str("Figure data (each trace's range):\n");
            for line in &r.figure_data {
                out.push_str(&format!("- {line}\n"));
            }
        }
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
    async fn check(&self, Parameters(req): Parameters<DocumentRequest>) -> Result<CallToolResult, ErrorData> {
        let b = match self.on(req.document.as_deref()).await {
            Ok(b) => b,
            Err(e) => return fail(e),
        };
        let doc = match b.snapshot(false).await {
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
        changed from that version to the latest, with their sources before and after. With \
        archived: true, the archive instead: snapshots kept from archived or trimmed history \
        (read them with read_doc's snapshot) and the save log of removed saves (with cell_id, \
        those that changed that cell).")]
    async fn history(&self, Parameters(req): Parameters<HistoryRequest>) -> Result<CallToolResult, ErrorData> {
        let b = match self.on(req.document.as_deref()).await {
            Ok(b) => b,
            Err(e) => return fail(e),
        };
        if req.archived == Some(true) {
            return match b.archive(req.cell_id.clone()).await {
                Ok(a) => ok(archive_text(&a)),
                Err(e) => fail(e),
            };
        }
        let versions = match b.history().await {
            Ok(v) => v,
            Err(e) => return fail(e),
        };
        let latest = versions.iter().map(|v| v.version).max().unwrap_or(0);
        let mut out = String::new();
        let mut listed = 0;
        for (i, v) in versions.iter().enumerate().rev() {
            if let Some(cell) = &req.cell_id {
                let prev = if i == 0 { continue } else { versions[i - 1].version };
                match b.diff(prev, v.version).await {
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
        if let Ok(a) = b.archive(None).await
            && (!a.saves.is_empty() || !a.snapshots.is_empty())
        {
            let names: Vec<&str> = a.snapshots.iter().map(|s| s.name.as_str()).collect();
            out.push_str(&format!(
                "\nArchived: {} removed save(s){}; history with archived: true lists them.\n",
                a.saves.len(),
                if names.is_empty() { String::new() } else { format!(", snapshots {} kept", names.join(", ")) }
            ));
        }
        if let Some(since) = req.since {
            match b.diff(since, latest).await {
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

    #[tool(description = "The documents open in Cruncha (its tabs): ID, name, latest version, which \
        is this conversation's own, which tab is showing, and whether its code may run. Every \
        other tool takes `document` (an ID or a name from this list) to act on another one.")]
    async fn documents(&self) -> Result<CallToolResult, ErrorData> {
        match self.backend.documents().await {
            Ok(docs) if docs.is_empty() => ok("No document is open in Cruncha.".into()),
            Ok(docs) => {
                let mut out = String::from("Open documents:\n");
                for d in docs {
                    let mut tags = Vec::new();
                    if d.this {
                        tags.push("this conversation's");
                    }
                    if d.active {
                        tags.push("showing");
                    }
                    if !d.can_run {
                        tags.push("code doesn't run: not trusted on this computer");
                    }
                    let tags = if tags.is_empty() { String::new() } else { format!(" ({})", tags.join(", ")) };
                    out.push_str(&format!("- {} — {} — version {}{tags}\n", d.name, d.id, d.version));
                }
                ok(out)
            }
            Err(e) => fail(e),
        }
    }

    #[tool(description = "Propose copying cells from one open document into another, as one proposal \
        in the target that the engineer accepts or rejects there. The copies are new cells. If the \
        target already defines a name they define, the proposal is refused and lists the names: \
        call again with `rename` (old → new, e.g. {\"M\": \"M2\"}). The reply has the new cells' IDs \
        and every result the copy would change in the target.")]
    async fn propose_copy(&self, Parameters(req): Parameters<CopyRequest>) -> Result<CallToolResult, ErrorData> {
        let b = match self.on(req.document.as_deref()).await {
            Ok(b) => b,
            Err(e) => return fail(e),
        };
        match b.propose_copy(req.from.document, req.from.cell_ids, req.after, req.rename, req.message).await {
            Ok(p) => ok(format!(
                "Proposal {} recorded in the target document; it waits for the engineer to accept or \
                 reject it in that document's tab. Don't describe it as applied.\n\n{}",
                p.id,
                mirror::proposal_text(&p)
            )),
            Err(e) => fail(e),
        }
    }

    #[tool(description = "The calculation templates and snippets in Cruncha's libraries (the built-in \
        one and the user's own): search them, or read one by id (its .cruncha.md file: TOML front \
        matter with inputs, outputs, references and worked examples, then its cells, with the \
        outcome of its worked examples). Insert one with propose_template. Load the \
        cruncha-templates skill for the format.")]
    async fn templates(&self, Parameters(req): Parameters<TemplatesRequest>) -> Result<CallToolResult, ErrorData> {
        if let Some(id) = req.id {
            return match self.backend.template(id).await {
                Ok(t) => ok(template_text(&t)),
                Err(e) => fail(e),
            };
        }
        let all = match self.backend.templates().await {
            Ok(t) => t,
            Err(e) => return fail(e),
        };
        let found = find_templates(&all, req.query.as_deref().unwrap_or(""));
        if found.is_empty() {
            return ok(if all.is_empty() { "No templates.".into() } else { "No template matches; try fewer words.".into() });
        }
        let mut out = format!("{} template{}:\n", found.len(), if found.len() == 1 { "" } else { "s" });
        for t in found {
            out.push_str(&format!("- {} ({} {}, {})", t.id, t.library, t.version, t.title));
            if !t.category.is_empty() {
                out.push_str(&format!(" [{}]", t.category.join(" / ")));
            }
            out.push('\n');
            if let Some(s) = &t.summary {
                out.push_str(&format!("  {s}\n"));
            }
            if !t.inputs.is_empty() || !t.outputs.is_empty() {
                out.push_str(&format!("  inputs: {}; outputs: {}\n", t.inputs.join(", "), t.outputs.join(", ")));
            }
        }
        ok(out)
    }

    #[tool(description = "Propose inserting a template into the document, as one proposal the engineer \
        accepts or rejects in the app. Bind inputs to values the document already has with \
        `bindings` (the input's definition is left out); other inputs arrive as placeholders (_) \
        for the engineer to fill in. If the document already defines a name the template \
        defines, the proposal is refused and lists the names: call again with `rename` (or \
        `bindings` for inputs). The reply has the new cells' IDs and the results the insert would \
        change.")]
    async fn propose_template(&self, Parameters(req): Parameters<ProposeTemplateRequest>) -> Result<CallToolResult, ErrorData> {
        let b = match self.on(req.document.as_deref()).await {
            Ok(b) => b,
            Err(e) => return fail(e),
        };
        let insert = TemplateInsert { id: req.id, bindings: req.bindings, rename: req.rename, after: req.after, message: req.message };
        match b.propose_template(insert).await {
            Ok(p) => ok(format!(
                "Proposal {} recorded; it waits for the engineer to accept or reject it in Cruncha. \
                 Don't describe it as applied.\n\n{}",
                p.id,
                mirror::proposal_text(&p)
            )),
            Err(e) => fail(e),
        }
    }

    #[tool(description = "Propose a change to the document: insert, modify or delete cells, as one \
        unit a human accepts or rejects in the app. Nothing changes until they accept. The reply \
        has the proposal id, the new cells' ids, and every result the change would alter \
        (\"if accepted\"), computed by the engine. One coherent proposal per logical change.")]
    async fn propose_edit(&self, Parameters(req): Parameters<ProposeRequest>) -> Result<CallToolResult, ErrorData> {
        let b = match self.on(req.document.as_deref()).await {
            Ok(b) => b,
            Err(e) => return fail(e),
        };
        match b.propose(req.base_version, req.message, req.changes).await {
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
    async fn proposals(&self, Parameters(req): Parameters<DocumentRequest>) -> Result<CallToolResult, ErrorData> {
        let b = match self.on(req.document.as_deref()).await {
            Ok(b) => b,
            Err(e) => return fail(e),
        };
        match b.proposals().await {
            Ok(list) if list.is_empty() => ok("No proposals.".into()),
            Ok(list) => ok(list.iter().map(mirror::proposal_text).collect::<Vec<_>>().join("\n")),
            Err(e) => fail(e),
        }
    }

    #[tool(description = "Withdraw one of your open proposals (for example to replace it with a better one).")]
    async fn withdraw(&self, Parameters(req): Parameters<WithdrawRequest>) -> Result<CallToolResult, ErrorData> {
        let b = match self.on(req.document.as_deref()).await {
            Ok(b) => b,
            Err(e) => return fail(e),
        };
        match b.withdraw(req.proposal_id).await {
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

fn words(s: &str) -> Vec<String> {
    s.to_lowercase().split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).map(str::to_owned).collect()
}

/// Templates matching every word of `query` (a prefix of a word in the title, summary, tags,
/// category, id or names), title matches first, then by title; as the Templates tab does.
pub fn find_templates<'a>(all: &'a [TemplateInfo], query: &str) -> Vec<&'a TemplateInfo> {
    let q = words(query);
    let mut scored: Vec<(usize, &TemplateInfo)> = Vec::new();
    'next: for t in all {
        let title = words(&t.title);
        let rest = words(
            &[t.summary.clone().unwrap_or_default(), t.tags.join(" "), t.category.join(" "), t.id.clone(), t.inputs.join(" "), t.outputs.join(" ")]
                .join(" "),
        );
        let mut score = 0;
        for w in &q {
            if title.iter().any(|x| x.starts_with(w.as_str())) {
                score += 2;
            } else if rest.iter().any(|x| x.starts_with(w.as_str())) {
                score += 1;
            } else {
                continue 'next;
            }
        }
        scored.push((score, t));
    }
    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.title.to_lowercase().cmp(&b.1.title.to_lowercase())));
    scored.into_iter().map(|(_, t)| t).collect()
}

/// One template for `templates(id)`: a header, then its file.
fn template_text(t: &crate::TemplateFile) -> String {
    let i = &t.info;
    let mut out = format!("Template {} {} ({} library): {}\n", i.id, i.version, i.library, i.title);
    if !i.inputs.is_empty() || !i.outputs.is_empty() {
        out.push_str(&format!("Inputs: {}; outputs: {}\n", i.inputs.join(", "), i.outputs.join(", ")));
    }
    if t.problems.is_empty() {
        out.push_str("Validation: no problems\n");
    } else {
        out.push_str("Validation problems:\n");
        for p in &t.problems {
            out.push_str(&format!("- {p}\n"));
        }
    }
    if !t.tests.is_empty() {
        out.push_str("Worked examples:\n");
        for line in &t.tests {
            out.push_str(&format!("- {line}\n"));
        }
    }
    out.push_str(&format!("\nThe file:\n\n{}", t.text));
    if !t.text.ends_with('\n') {
        out.push('\n');
    }
    out
}

/// The archive as text: kept snapshots, then removed saves, newest first.
fn archive_text(a: &crate::ArchiveView) -> String {
    let mut out = String::new();
    if a.snapshots.is_empty() && a.saves.is_empty() {
        return "Nothing has been archived or trimmed.\n".into();
    }
    out.push_str("Archived snapshots (read with read_doc snapshot):\n");
    if a.snapshots.is_empty() {
        out.push_str("(none)\n");
    }
    for s in a.snapshots.iter().rev() {
        out.push_str(&format!("- {} (v{}, {}){}\n", s.name, s.version, s.at, if s.message.is_empty() { String::new() } else { format!(": {}", s.message) }));
    }
    out.push_str(&format!("\nRemoved saves ({}), newest first:\n", a.saves.len()));
    for (n, s) in a.saves.iter().rev().enumerate() {
        if n == 200 {
            out.push_str("… older saves omitted\n");
            break;
        }
        out.push_str(&format!(
            "v{} {} {} ({}) in {}: {} — cells {}\n",
            s.version,
            s.at,
            s.actor,
            s.kind,
            s.snapshot.as_deref().unwrap_or("Draft"),
            s.message,
            if s.cells.is_empty() { "none".to_owned() } else { s.cells.join(", ") }
        ));
    }
    out
}
