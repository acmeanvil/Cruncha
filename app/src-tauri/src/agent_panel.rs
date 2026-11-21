// SPDX-License-Identifier: GPL-3.0-or-later

//! The agent panel (ADR-0009): an ACP agent (Claude Code, Codex, Gemini CLI, …) working on
//! the open document, in the app. The UX follows looking_glass: streaming replies, thoughts,
//! tool cards, a plan, permission prompts, the agent's model and mode menus, token usage.
//!
//! - The agent reaches the document only through `cruncha-mcp`; the session gets it in
//!   `mcpServers`, and the client offers no file system or terminal (`cruncha-acp`).
//! - Each document gets a working folder in the app's data (`agents/<doc id>/`) with Cruncha's
//!   skills in it (`.claude/skills/…`, `AGENTS.md`), so agents that read project skills find
//!   them. Nothing in the document's own folder is touched.
//! - Events go to the window as `agent-event`; the panel's transcript is kept here too, so it
//!   survives a reload and is saved per conversation (`agents/<doc id>/conversations/`).

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use cruncha_acp::{
    AgentHandle, AgentSpec, Command, ConfigOption, Event, McpServerSpec, PermissionKind, PermissionReply, PlanItem,
    StopReason, ToolKind, ToolOutput, ToolStatus, UsageTotals,
};
use futures::StreamExt;
use serde::{Deserialize, Serialize};

/// An agent the panel can start.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentDef {
    pub id: String,
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    /// Shown under the name in the picker.
    #[serde(default)]
    pub note: String,
}

/// Agents that speak ACP through their adapters (the same table as looking_glass's
/// `config.example.toml`), plus looking_glass's own agent when it is installed.
pub fn catalog() -> Vec<AgentDef> {
    let npx = |id: &str, name: &str, args: &[&str], note: &str| AgentDef {
        id: id.into(),
        name: name.into(),
        command: "npx".into(),
        args: args.iter().map(|s| s.to_string()).collect(),
        env: BTreeMap::new(),
        note: note.into(),
    };
    let mut list = vec![
        npx("claude", "Claude Code", &["-y", "@agentclientprotocol/claude-agent-acp"], "your Claude login or ANTHROPIC_API_KEY"),
        npx("codex", "Codex", &["-y", "@agentclientprotocol/codex-acp"], "your OpenAI login"),
        npx("gemini", "Gemini CLI", &["-y", "@google/gemini-cli", "--acp"], "your Google login"),
        npx("qwen-code", "Qwen Code", &["-y", "@qwen-code/qwen-code", "--acp"], "your Qwen login"),
    ];
    if let Some(lg) = looking_glass_agent() {
        list.push(AgentDef {
            id: "looking-glass".into(),
            name: "looking_glass agent".into(),
            command: lg.display().to_string(),
            args: vec![],
            env: BTreeMap::new(),
            note: "demo mode; providers are set up in looking_glass".into(),
        });
    }
    list
}

/// `lg-agent` on `PATH`, or in a looking_glass checkout next to this one.
fn looking_glass_agent() -> Option<PathBuf> {
    let on_path = std::env::var_os("PATH").and_then(|p| {
        std::env::split_paths(&p).map(|d| d.join("lg-agent")).find(|c| c.is_file())
    });
    on_path.or_else(|| {
        let home = std::env::var_os("HOME")?;
        ["release", "debug"]
            .iter()
            .map(|p| PathBuf::from(&home).join(format!("Developer/looking_glass/target/{p}/lg-agent")))
            .find(|c| c.is_file())
    })
}

/// What the panel shows, entry by entry.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum Entry {
    User { text: String },
    Agent { text: String },
    Thought { text: String },
    Tool { id: String, title: String, kind: String, status: String, output: Vec<ToolOutput> },
    Notice { text: String, error: bool },
}

/// A permission prompt waiting for the user.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionView {
    pub id: u64,
    pub tool_call_id: String,
    pub title: String,
    pub options: Vec<PermissionOptionView>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionOptionView {
    pub id: String,
    pub label: String,
    /// `allowOnce`, `allowAlways`, `rejectOnce`, `rejectAlways`.
    pub kind: String,
}

/// The panel's state, sent whole when it opens and after every event.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PanelView {
    pub agent_id: Option<String>,
    pub agent_name: Option<String>,
    pub agent_version: Option<String>,
    /// An agent process is running (starting, ready or busy).
    pub running: bool,
    /// Connected and the session is ready.
    pub ready: bool,
    /// A turn is running.
    pub busy: bool,
    pub entries: Vec<Entry>,
    pub plan: Vec<PlanItem>,
    pub permissions: Vec<PermissionView>,
    pub options: Vec<ConfigOption>,
    /// Context in use after the last request, and the window size (0 when unknown).
    pub used: u64,
    pub size: u64,
    pub totals: Option<UsageTotals>,
    pub conversation_id: Option<String>,
}

/// A saved conversation (one per agent session).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationSummary {
    pub id: String,
    pub agent: String,
    pub started_ms: i64,
    pub updated_ms: i64,
    /// The first thing the user asked.
    pub title: String,
    pub turns: usize,
    /// The document version when it started.
    pub version: u64,
}

struct Running {
    handle: AgentHandle,
    /// This agent's own MCP server (its own token), stopped with it.
    server: cruncha_mcp::http::Server,
    replies: HashMap<u64, PermissionReply>,
    started_ms: i64,
    version: u64,
}

/// The panel for the open document.
#[derive(Default)]
pub struct Panel {
    view: PanelView,
    running: Option<Running>,
    /// `agents/<doc id>/` in the app's data.
    dir: Option<PathBuf>,
    /// Bumped per start, so events from a stopped agent are ignored.
    generation: u64,
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default()
}

const AGENTS_MD: &str = "\
# Working on a Cruncha document

You are helping with an engineering calculation document open in the Cruncha app.

- The document is reached **only** through the `cruncha` MCP tools: `read_doc`, `explain`, `check`,
  `history`, `run`, `propose_edit`, `proposals`, `withdraw`. Start with `read_doc`. There are no
  document files to read or edit here.
- The engine computes every number. Quote results from the tools with their cell IDs; never compute
  values yourself or present an estimate as a result.
- To change the document, use `propose_edit`: the user accepts or rejects it in the app, and
  nothing changes until they accept. Never write the change into your reply for the user to paste,
  and never say a proposal was applied.
- Plots are matplotlib figures in a Python cell; Cruncha shows them as images in the cell's output
  and in the report. Python cells only get the packages in `cruncha-code-cells` (no Plotly or other
  HTML/JavaScript output). Don't install packages.
- Don't create files or run commands in this folder; it only holds these instructions.

Load the `cruncha-core` skill before anything else; `cruncha-calc-authoring`, `cruncha-calc-review`,
`cruncha-code-cells` and `cruncha-history` cover the rest.
";

/// Copy the skills into `dir/.claude/skills/<name>/SKILL.md` and write `AGENTS.md`/`CLAUDE.md`.
fn prepare_dir(dir: &Path, skills: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir.join("conversations"))?;
    std::fs::write(dir.join("AGENTS.md"), AGENTS_MD)?;
    std::fs::write(dir.join("CLAUDE.md"), AGENTS_MD)?;
    if let Ok(entries) = std::fs::read_dir(skills) {
        for e in entries.flatten() {
            let src = e.path().join("SKILL.md");
            if src.is_file() {
                let dest = dir.join(".claude/skills").join(e.file_name());
                std::fs::create_dir_all(&dest)?;
                std::fs::copy(&src, dest.join("SKILL.md"))?;
            }
        }
    }
    Ok(())
}

fn status_name(s: ToolStatus) -> &'static str {
    match s {
        ToolStatus::Pending => "pending",
        ToolStatus::Running => "running",
        ToolStatus::Done => "done",
        ToolStatus::Failed => "failed",
    }
}

fn kind_name(k: ToolKind) -> &'static str {
    match k {
        ToolKind::Read => "read",
        ToolKind::Edit => "edit",
        ToolKind::Delete => "delete",
        ToolKind::Move => "move",
        ToolKind::Search => "search",
        ToolKind::Execute => "execute",
        ToolKind::Think => "think",
        ToolKind::Fetch => "fetch",
        ToolKind::Other => "other",
    }
}

fn permission_kind(k: PermissionKind) -> &'static str {
    match k {
        PermissionKind::AllowOnce => "allowOnce",
        PermissionKind::AllowAlways => "allowAlways",
        PermissionKind::RejectOnce => "rejectOnce",
        PermissionKind::RejectAlways => "rejectAlways",
    }
}

impl Panel {
    pub fn view(&self) -> PanelView {
        self.view.clone()
    }

    /// Start `agent` on the open document (stopping any agent already running).
    #[allow(clippy::too_many_arguments)]
    pub fn start(
        &mut self,
        agent: &AgentDef,
        doc_id: &str,
        version: u64,
        data_dir: &Path,
        skills: &Path,
        server: cruncha_mcp::http::Server,
    ) -> Result<(u64, futures::channel::mpsc::UnboundedReceiver<Event>), String> {
        self.stop();
        let mcp = McpServerSpec {
            name: "cruncha".into(),
            url: server.url.clone(),
            headers: vec![("Authorization".into(), format!("Bearer {}", server.token))],
        };
        let dir = data_dir.join("agents").join(doc_id);
        prepare_dir(&dir, skills).map_err(|e| format!("couldn't prepare the agent's folder: {e}"))?;
        let (handle, events) = cruncha_acp::connect(AgentSpec {
            name: agent.name.clone(),
            command: agent.command.clone().into(),
            args: agent.args.clone(),
            env: agent.env.clone(),
            cwd: dir.clone(),
            resume: None,
            mcp_servers: vec![mcp],
        });
        self.generation += 1;
        self.dir = Some(dir);
        self.view = PanelView {
            agent_id: Some(agent.id.clone()),
            agent_name: Some(agent.name.clone()),
            running: true,
            conversation_id: Some(format!("{}-{}", now_ms(), self.generation)),
            entries: vec![Entry::Notice { text: format!("Starting {}…", agent.name), error: false }],
            ..PanelView::default()
        };
        self.running = Some(Running { handle, server, replies: HashMap::new(), started_ms: now_ms(), version });
        Ok((self.generation, events))
    }

    pub fn stop(&mut self) {
        if let Some(r) = self.running.take() {
            r.server.stop();
            self.save();
            self.view.running = false;
            self.view.ready = false;
            self.view.busy = false;
            self.view.permissions.clear();
            self.view.entries.push(Entry::Notice { text: "Agent stopped.".into(), error: false });
        }
    }

    pub fn prompt(&mut self, text: String) -> Result<(), String> {
        let r = self.running.as_ref().ok_or("no agent is running")?;
        if !self.view.ready {
            return Err("the agent isn't ready yet".into());
        }
        if !r.handle.send(Command::Prompt(text.clone())) {
            return Err("the agent has stopped".into());
        }
        self.view.entries.push(Entry::User { text });
        self.view.busy = true;
        Ok(())
    }

    pub fn cancel(&mut self) {
        if let Some(r) = &self.running {
            r.handle.send(Command::Cancel);
        }
        // Pending permission prompts are answered "cancelled" by dropping them.
        if let Some(r) = &mut self.running {
            r.replies.clear();
        }
        self.view.permissions.clear();
    }

    pub fn set_option(&mut self, id: String, value: String) {
        if let Some(r) = &self.running {
            r.handle.send(Command::SetConfigOption { id, value });
        }
    }

    pub fn answer(&mut self, id: u64, option: Option<String>) {
        if let Some(r) = &mut self.running
            && let Some(mut reply) = r.replies.remove(&id)
        {
            reply.answer(option);
        }
        self.view.permissions.retain(|p| p.id != id);
    }

    /// Apply an agent event (from the start whose number is `generation`).
    pub fn apply(&mut self, generation: u64, ev: Event) -> bool {
        if generation != self.generation {
            return false;
        }
        let v = &mut self.view;
        let append = |entries: &mut Vec<Entry>, text: String, thought: bool| match (entries.last_mut(), thought) {
            (Some(Entry::Agent { text: t }), false) | (Some(Entry::Thought { text: t }), true) => t.push_str(&text),
            _ => entries.push(if thought { Entry::Thought { text } } else { Entry::Agent { text } }),
        };
        match ev {
            Event::Connected { agent_name, agent_version, .. } => {
                v.agent_name = Some(agent_name);
                v.agent_version = agent_version;
            }
            Event::SessionReady { .. } => {
                v.ready = true;
                if let Some(Entry::Notice { text, .. }) = v.entries.last_mut()
                    && text.starts_with("Starting")
                {
                    *text = format!("{} is ready. It can read this document, its results, checks and history.", v.agent_name.clone().unwrap_or_default());
                }
            }
            Event::UserText(t) => match v.entries.last_mut() {
                Some(Entry::User { text }) => text.push_str(&t),
                _ => v.entries.push(Entry::User { text: t }),
            },
            Event::AgentText(t) => append(&mut v.entries, t, false),
            Event::Thought(t) => append(&mut v.entries, t, true),
            Event::ToolCall(p) => {
                let found = v.entries.iter_mut().rev().find_map(|e| match e {
                    Entry::Tool { id, title, kind, status, output } if *id == p.id => Some((title, kind, status, output)),
                    _ => None,
                });
                match found {
                    Some((title, kind, status, output)) => {
                        if let Some(t) = p.title {
                            *title = t;
                        }
                        if let Some(k) = p.kind {
                            *kind = kind_name(k).into();
                        }
                        if let Some(s) = p.status {
                            *status = status_name(s).into();
                        }
                        if let Some(o) = p.output {
                            *output = o;
                        }
                    }
                    None => v.entries.push(Entry::Tool {
                        id: p.id,
                        title: p.title.unwrap_or_default(),
                        kind: p.kind.map(kind_name).unwrap_or("other").into(),
                        status: p.status.map(status_name).unwrap_or("pending").into(),
                        output: p.output.unwrap_or_default(),
                    }),
                }
            }
            Event::Plan(items) => v.plan = items,
            Event::Usage { used, size, totals } => {
                v.used = used;
                v.size = size;
                if totals.is_some() {
                    v.totals = totals;
                }
            }
            Event::Permission(p) => {
                v.permissions.push(PermissionView {
                    id: p.id,
                    tool_call_id: p.tool_call_id,
                    title: p.title,
                    options: p
                        .options
                        .iter()
                        .map(|o| PermissionOptionView { id: o.id.clone(), label: o.label.clone(), kind: permission_kind(o.kind).into() })
                        .collect(),
                });
                if let Some(r) = &mut self.running {
                    r.replies.insert(p.id, p.reply);
                }
            }
            Event::TurnEnded(reason) => {
                v.busy = false;
                v.permissions.clear();
                let note = match reason {
                    StopReason::EndTurn => None,
                    StopReason::Cancelled => Some("Stopped."),
                    StopReason::MaxTokens => Some("The reply hit the token limit."),
                    StopReason::MaxTurns => Some("The agent hit its limit of steps for one turn."),
                    StopReason::Refusal => Some("The agent declined."),
                };
                if let Some(n) = note {
                    v.entries.push(Entry::Notice { text: n.into(), error: false });
                }
                self.save();
            }
            Event::Error(e) => {
                v.busy = false;
                v.entries.push(Entry::Notice { text: e, error: true });
            }
            Event::ConfigOptions(o) => v.options = o,
            Event::Log(line) => eprintln!("agent: {line}"),
            Event::Disconnected { reason } => {
                let was = v.ready;
                v.running = false;
                v.ready = false;
                v.busy = false;
                v.permissions.clear();
                let failed = reason.is_some() || !was;
                v.entries.push(Entry::Notice {
                    text: match reason {
                        Some(r) => format!("The agent stopped: {r}"),
                        None if was => "The agent disconnected.".into(),
                        None => "The agent couldn't start. Is it installed (Node.js and npx for the adapters)?".into(),
                    },
                    error: failed,
                });
                if let Some(r) = self.running.take() {
                    r.server.stop();
                }
                self.save();
            }
        }
        true
    }

    /// Save the conversation (after each turn and when it ends).
    fn save(&self) {
        let (Some(dir), Some(id)) = (&self.dir, &self.view.conversation_id) else { return };
        let turns = self.view.entries.iter().filter(|e| matches!(e, Entry::User { .. })).count();
        if turns == 0 {
            return;
        }
        let title = self
            .view
            .entries
            .iter()
            .find_map(|e| match e {
                Entry::User { text } => Some(text.chars().take(120).collect::<String>()),
                _ => None,
            })
            .unwrap_or_default();
        let (started_ms, version) = self.running.as_ref().map(|r| (r.started_ms, r.version)).unwrap_or((now_ms(), 0));
        let summary = ConversationSummary {
            id: id.clone(),
            agent: self.view.agent_name.clone().unwrap_or_default(),
            started_ms,
            updated_ms: now_ms(),
            title,
            turns,
            version,
        };
        let body = serde_json::json!({ "summary": summary, "entries": self.view.entries });
        let path = dir.join("conversations").join(format!("{id}.json"));
        if let Err(e) = std::fs::write(&path, serde_json::to_vec_pretty(&body).unwrap_or_default()) {
            eprintln!("cruncha: couldn't save the conversation {}: {e}", path.display());
        }
    }
}

/// Saved conversations for a document, newest first.
pub fn conversations(data_dir: &Path, doc_id: &str) -> Vec<ConversationSummary> {
    let dir = data_dir.join("agents").join(doc_id).join("conversations");
    let mut list: Vec<ConversationSummary> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let v: serde_json::Value = serde_json::from_slice(&std::fs::read(e.path()).ok()?).ok()?;
            serde_json::from_value(v["summary"].clone()).ok()
        })
        .collect();
    list.sort_by_key(|c| std::cmp::Reverse(c.updated_ms));
    list
}

/// One saved conversation's entries, as JSON for the panel.
pub fn conversation(data_dir: &Path, doc_id: &str, id: &str) -> Result<serde_json::Value, String> {
    if id.contains(['/', '\\', '.']) {
        return Err("bad conversation id".into());
    }
    let path = data_dir.join("agents").join(doc_id).join("conversations").join(format!("{id}.json"));
    let bytes = std::fs::read(&path).map_err(|e| format!("couldn't read the conversation: {e}"))?;
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}

/// Drive the events of one start into the panel, notifying the window after each.
pub async fn pump<F>(
    panel: std::sync::Arc<tokio::sync::Mutex<Panel>>,
    generation: u64,
    mut events: futures::channel::mpsc::UnboundedReceiver<Event>,
    notify: F,
) where
    F: Fn(PanelView) + Send + 'static,
{
    while let Some(ev) = events.next().await {
        let mut p = panel.lock().await;
        if p.apply(generation, ev) {
            notify(p.view());
        } else {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cruncha_acp::ToolCallPatch;

    #[test]
    fn events_build_the_transcript() {
        let mut p = Panel { generation: 1, ..Default::default() };
        let ev = |p: &mut Panel, e| assert!(p.apply(1, e));
        ev(&mut p, Event::Connected { agent_name: "Claude Code".into(), agent_version: None, images: true, mcp_http: true });
        ev(&mut p, Event::SessionReady { session_id: "s".into(), resumed: false });
        assert!(p.view.ready);
        p.view.entries.push(Entry::User { text: "check it".into() });
        ev(&mut p, Event::Thought("Let me ".into()));
        ev(&mut p, Event::Thought("read.".into()));
        ev(&mut p, Event::ToolCall(ToolCallPatch {
            id: "t1".into(),
            title: Some("read_doc".into()),
            kind: Some(ToolKind::Read),
            status: Some(ToolStatus::Running),
            output: None,
        }));
        ev(&mut p, Event::ToolCall(ToolCallPatch { id: "t1".into(), title: None, kind: None, status: Some(ToolStatus::Done), output: None }));
        ev(&mut p, Event::AgentText("All ".into()));
        ev(&mut p, Event::AgentText("checks hold.".into()));
        ev(&mut p, Event::TurnEnded(StopReason::EndTurn));
        let kinds: Vec<&str> = p
            .view
            .entries
            .iter()
            .map(|e| match e {
                Entry::User { .. } => "user",
                Entry::Agent { .. } => "agent",
                Entry::Thought { .. } => "thought",
                Entry::Tool { .. } => "tool",
                Entry::Notice { .. } => "notice",
            })
            .collect();
        assert_eq!(kinds, ["user", "thought", "tool", "agent"], "{:?}", p.view.entries);
        assert!(matches!(&p.view.entries[1], Entry::Thought { text } if text == "Let me read."));
        assert!(matches!(&p.view.entries[2], Entry::Tool { status, .. } if status == "done"));
        assert!(matches!(&p.view.entries[3], Entry::Agent { text } if text == "All checks hold."));
        assert!(!p.view.busy);
        // Events from an earlier start are ignored.
        assert!(!p.apply(0, Event::AgentText("stale".into())));
    }

    #[test]
    fn the_agent_folder_has_the_skills() {
        let dir = tempfile::tempdir().unwrap();
        let skills = concat!(env!("CARGO_MANIFEST_DIR"), "/../../skills");
        prepare_dir(dir.path(), Path::new(skills)).unwrap();
        assert!(dir.path().join(".claude/skills/cruncha-core/SKILL.md").is_file());
        assert!(std::fs::read_to_string(dir.path().join("AGENTS.md")).unwrap().contains("read_doc"));
        assert!(catalog().iter().any(|a| a.id == "claude"));
    }
}
