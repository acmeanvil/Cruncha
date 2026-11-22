// SPDX-License-Identifier: GPL-3.0-or-later
// Ported from looking_glass (crates/lg-acp/src/lib.rs), Copyright (c) 2026 acmeanvil, MIT License;
// see LICENSE-LOOKING-GLASS. Changes for Cruncha: sessions get MCP servers (`mcpServers`), and the
// client offers agents no file system or terminal (no remote agents either): an agent touches a
// Cruncha document only through cruncha-mcp (ADR-0009).

//! ACP client for Cruncha's agent panel.
//!
//! Spawns an Agent Client Protocol agent (Claude Code, Codex, Gemini CLI, … through their ACP
//! adapters) as a subprocess, drives the connection on a dedicated thread, and exposes a small
//! interface: a [`Command`] sender in, an [`Event`] stream out. The UI never sees ACP schema
//! types; everything is translated into the neutral types in [`model`].

pub mod model;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use agent_client_protocol::schema::ProtocolVersion;
use agent_client_protocol::v1::ConnectionTo;
use agent_client_protocol::v1::schema::{
    self as acp, CancelNotification, ClientCapabilities, ContentBlock, Implementation, InitializeRequest,
    LoadSessionRequest, NewSessionRequest, PromptRequest, RequestPermissionOutcome, RequestPermissionRequest,
    RequestPermissionResponse, SelectedPermissionOutcome, SessionNotification, SessionUpdate,
    SetSessionConfigOptionRequest, TextContent,
};
use agent_client_protocol::{AcpAgent, AcpAgentConfig, Agent, Client, LineDirection};
use futures::channel::{mpsc, oneshot};
use futures::{FutureExt, StreamExt};

use model::config_options;
pub use model::*;

/// An MCP server the agent should use in this session.
#[derive(Clone, Debug)]
pub struct McpServerSpec {
    pub name: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    /// For agents without HTTP MCP: a program that serves this server over stdio, run as
    /// `<bridge> --mcp-stdio <url>` with the `Authorization` header in `CRUNCHA_MCP_AUTH`.
    pub bridge: Option<PathBuf>,
}

/// How to launch an agent.
#[derive(Clone, Debug)]
pub struct AgentSpec {
    /// Human-readable name shown in the UI.
    pub name: String,
    pub command: PathBuf,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
    /// Working directory for the session.
    pub cwd: PathBuf,
    /// An ACP session id to resume with `session/load`, when the agent supports it.
    pub resume: Option<String>,
    /// MCP servers for the session (Cruncha's own, over HTTP).
    pub mcp_servers: Vec<McpServerSpec>,
}

/// Commands from the UI to the agent connection.
#[derive(Debug)]
pub enum Command {
    Prompt(String),
    Cancel,
    /// Change one of the agent's session settings (see `Event::ConfigOptions`).
    SetConfigOption { id: String, value: String },
}

/// A live connection to an agent. Dropping it shuts the agent down.
pub struct AgentHandle {
    commands: mpsc::UnboundedSender<Command>,
}

impl AgentHandle {
    pub fn send(&self, command: Command) -> bool {
        self.commands.unbounded_send(command).is_ok()
    }
}

/// Launch the agent and connect to it. Events arrive on the returned receiver; the stream ends
/// after [`Event::Disconnected`].
pub fn connect(spec: AgentSpec) -> (AgentHandle, mpsc::UnboundedReceiver<Event>) {
    let (cmd_tx, cmd_rx) = mpsc::unbounded();
    let (ev_tx, ev_rx) = mpsc::unbounded();
    std::thread::Builder::new()
        .name(format!("acp:{}", spec.name))
        .spawn(move || {
            let result = futures::executor::block_on(run(spec, cmd_rx, ev_tx.clone()));
            let reason = result.err().map(|e| e.to_string());
            let _ = ev_tx.unbounded_send(Event::Disconnected { reason });
        })
        .expect("spawn ACP thread");
    (AgentHandle { commands: cmd_tx }, ev_rx)
}

/// The servers through their stdio bridges, for agents without HTTP MCP.
fn mcp_bridged(spec: &AgentSpec) -> Vec<acp::McpServer> {
    spec.mcp_servers
        .iter()
        .filter_map(|m| {
            let bridge = m.bridge.as_ref()?;
            let auth = m.headers.iter().find(|(k, _)| k.eq_ignore_ascii_case("authorization")).map(|(_, v)| v.clone());
            Some(acp::McpServer::Stdio(
                acp::McpServerStdio::new(m.name.clone(), bridge.clone())
                    .args(vec!["--mcp-stdio".into(), m.url.clone()])
                    .env(auth.map(|a| vec![acp::EnvVariable::new("CRUNCHA_MCP_AUTH", a)]).unwrap_or_default()),
            ))
        })
        .collect()
}

fn mcp_servers(spec: &AgentSpec) -> Vec<acp::McpServer> {
    spec.mcp_servers
        .iter()
        .map(|m| {
            acp::McpServer::Http(
                acp::McpServerHttp::new(m.name.clone(), m.url.clone())
                    .headers(m.headers.iter().map(|(k, v)| acp::HttpHeader::new(k.clone(), v.clone())).collect()),
            )
        })
        .collect()
}

async fn run(
    spec: AgentSpec,
    mut commands: mpsc::UnboundedReceiver<Command>,
    events: mpsc::UnboundedSender<Event>,
) -> Result<(), agent_client_protocol::Error> {
    let mut config = AcpAgentConfig::new(resolve_program(&spec.command)).args(spec.args.clone());
    for (k, v) in &spec.env {
        config = config.env(k, v);
    }
    let stderr_events = events.clone();
    let agent = AcpAgent::new(config).with_debug(move |line, direction| {
        if matches!(direction, LineDirection::Stderr) {
            let _ = stderr_events.unbounded_send(Event::Log(line.to_string()));
        }
    });

    let notif_events = events.clone();
    let perm_events = events.clone();
    let next_permission = Arc::new(AtomicU64::new(1));

    Client
        .v1()
        .name("cruncha")
        .on_receive_notification(
            async move |n: SessionNotification, _cx| {
                for ev in translate_update(n.update) {
                    let _ = notif_events.unbounded_send(ev);
                }
                Ok(())
            },
            agent_client_protocol::on_receive_notification!(),
        )
        .on_receive_request(
            async move |req: RequestPermissionRequest, responder, cx: ConnectionTo<Agent>| {
                // Waiting for the user must not hold the dispatch loop, so the wait happens in
                // a spawned task.
                let (reply_tx, reply_rx) = oneshot::channel();
                let prompt = PermissionPrompt {
                    id: next_permission.fetch_add(1, Ordering::Relaxed),
                    tool_call_id: req.tool_call.tool_call_id.to_string(),
                    title: req.tool_call.fields.title.clone().unwrap_or_else(|| "Permission requested".into()),
                    options: req
                        .options
                        .iter()
                        .map(|o| PermissionChoice { id: o.option_id.to_string(), label: o.name.clone(), kind: o.kind.into() })
                        .collect(),
                    reply: PermissionReply(Some(reply_tx)),
                };
                let _ = perm_events.unbounded_send(Event::Permission(prompt));
                cx.spawn(async move {
                    let outcome = match reply_rx.await {
                        Ok(Some(id)) => RequestPermissionOutcome::Selected(SelectedPermissionOutcome::new(id)),
                        // Dismissed, cancelled, or the UI went away.
                        _ => RequestPermissionOutcome::Cancelled,
                    };
                    responder.respond(RequestPermissionResponse::new(outcome))
                })
            },
            agent_client_protocol::on_receive_request!(),
        )
        .connect_with(agent, async move |cx: ConnectionTo<Agent>| {
            let init = cx
                .send_request(
                    InitializeRequest::new(ProtocolVersion::V1)
                        // No file system and no terminal: the document is reached through MCP.
                        .client_capabilities(ClientCapabilities::new())
                        .client_info(Implementation::new("cruncha", env!("CARGO_PKG_VERSION"))),
                )
                .block_task()
                .await?;
            let can_load = init.agent_capabilities.load_session;
            let images = init.agent_capabilities.prompt_capabilities.image;
            let mcp_http = init.agent_capabilities.mcp_capabilities.http;
            let (agent_name, agent_version) = match init.agent_info {
                Some(info) => (info.title.unwrap_or(info.name), Some(info.version)),
                None => (spec.name.clone(), None),
            };
            let _ = events.unbounded_send(Event::Connected { agent_name, agent_version, images, mcp_http });
            // Without HTTP MCP the agent can't reach the document; say so instead of starting a
            // session that can only guess.
            let servers = if mcp_http { mcp_servers(&spec) } else { mcp_bridged(&spec) };
            if !mcp_http && !spec.mcp_servers.is_empty() && servers.is_empty() {
                let _ = events.unbounded_send(Event::Error(
                    "This agent doesn't support MCP servers over HTTP, so it can't read the document.".into(),
                ));
            }

            let (session_id, resumed) = match spec.resume.clone().filter(|_| can_load) {
                // The agent replays the conversation as session/update notifications first.
                Some(id) => {
                    let loaded = cx
                        .send_request(LoadSessionRequest::new(id.clone(), spec.cwd.clone()).mcp_servers(servers))
                        .block_task()
                        .await?;
                    if let Some(options) = loaded.config_options {
                        let _ = events.unbounded_send(Event::ConfigOptions(config_options(&options)));
                    }
                    (acp::SessionId::new(id), true)
                }
                None => {
                    let session = cx
                        .send_request(NewSessionRequest::new(spec.cwd.clone()).mcp_servers(servers))
                        .block_task()
                        .await?;
                    if let Some(options) = session.config_options {
                        let _ = events.unbounded_send(Event::ConfigOptions(config_options(&options)));
                    }
                    (session.session_id, false)
                }
            };
            let _ = events.unbounded_send(Event::SessionReady { session_id: session_id.to_string(), resumed });

            let mut incoming_closed = Box::pin(cx.incoming_closed().fuse());
            loop {
                let command = futures::select! {
                    c = commands.next() => c,
                    _ = incoming_closed => break,
                };
                match command {
                    Some(Command::Prompt(text)) => {
                        let blocks = vec![ContentBlock::Text(TextContent::new(text))];
                        let cx2 = cx.clone();
                        let events = events.clone();
                        let session_id = session_id.clone();
                        cx.spawn(async move {
                            let result = cx2.send_request(PromptRequest::new(session_id, blocks)).block_task().await;
                            let _ = events.unbounded_send(match result {
                                Ok(r) => Event::TurnEnded(r.stop_reason.into()),
                                Err(e) => Event::Error(e.to_string()),
                            });
                            Ok(())
                        })?;
                    }
                    Some(Command::Cancel) => {
                        cx.send_notification(CancelNotification::new(session_id.clone()))?;
                    }
                    Some(Command::SetConfigOption { id, value }) => {
                        let cx2 = cx.clone();
                        let events = events.clone();
                        let session_id = session_id.clone();
                        cx.spawn(async move {
                            let result = cx2
                                .send_request(SetSessionConfigOptionRequest::new(session_id, id, value.as_str()))
                                .block_task()
                                .await;
                            let _ = events.unbounded_send(match result {
                                Ok(r) => Event::ConfigOptions(config_options(&r.config_options)),
                                Err(e) => Event::Error(format!("couldn't change setting: {e}")),
                            });
                            Ok(())
                        })?;
                    }
                    // The UI dropped the handle: shut down.
                    None => break,
                }
            }
            Ok(())
        })
        .await
}

fn translate_update(update: SessionUpdate) -> Vec<Event> {
    match update {
        SessionUpdate::AgentMessageChunk(c) => text_of(&c.content).map(Event::AgentText).into_iter().collect(),
        SessionUpdate::AgentThoughtChunk(c) => text_of(&c.content).map(Event::Thought).into_iter().collect(),
        SessionUpdate::UserMessageChunk(c) => text_of(&c.content).map(Event::UserText).into_iter().collect(),
        SessionUpdate::ToolCall(t) => vec![Event::ToolCall(ToolCallPatch {
            id: t.tool_call_id.to_string(),
            title: Some(t.title),
            kind: Some(t.kind.into()),
            status: Some(t.status.into()),
            output: Some(t.content.iter().map(translate_tool_content).collect()),
        })],
        SessionUpdate::ToolCallUpdate(u) => vec![Event::ToolCall(ToolCallPatch {
            id: u.tool_call_id.to_string(),
            title: u.fields.title,
            kind: u.fields.kind.map(Into::into),
            status: u.fields.status.map(Into::into),
            output: u.fields.content.map(|c| c.iter().map(translate_tool_content).collect()),
        })],
        SessionUpdate::Plan(p) => vec![Event::Plan(
            p.entries
                .into_iter()
                .map(|e| PlanItem {
                    content: e.content,
                    status: match e.status {
                        acp::PlanEntryStatus::Completed => PlanStatus::Done,
                        acp::PlanEntryStatus::InProgress => PlanStatus::Active,
                        _ => PlanStatus::Pending,
                    },
                })
                .collect(),
        )],
        SessionUpdate::ConfigOptionUpdate(u) => vec![Event::ConfigOptions(config_options(&u.config_options))],
        SessionUpdate::UsageUpdate(u) => vec![Event::Usage { used: u.used, size: u.size, totals: usage_totals(&u) }],
        _ => Vec::new(),
    }
}

/// Session totals: looking_glass's agent sends them in `_meta`; any agent may send a cost.
fn usage_totals(u: &acp::UsageUpdate) -> Option<UsageTotals> {
    let lg = u.meta.as_ref().and_then(|m| m.get("lookingGlass"));
    let cost = u.cost.as_ref().map(|c| (c.amount, c.currency.clone()));
    if lg.is_none() && cost.is_none() {
        return None;
    }
    let n = |k: &str| lg.and_then(|v| v[k].as_u64()).unwrap_or(0);
    Some(UsageTotals {
        model: lg.and_then(|v| v["model"].as_str()).map(str::to_string),
        input_tokens: n("inputTokens"),
        output_tokens: n("outputTokens"),
        cache_read_tokens: n("cacheReadTokens"),
        cache_write_tokens: n("cacheWriteTokens"),
        requests: n("requests"),
        cost,
    })
}

fn text_of(block: &ContentBlock) -> Option<String> {
    match block {
        ContentBlock::Text(t) => Some(t.text.clone()),
        ContentBlock::ResourceLink(l) => Some(format!("[{}]({})", l.name, l.uri)),
        _ => None,
    }
}

fn translate_tool_content(c: &acp::ToolCallContent) -> ToolOutput {
    match c {
        acp::ToolCallContent::Content(c) => {
            ToolOutput::Text(text_of(&c.content).unwrap_or_else(|| "[non-text content]".into()))
        }
        acp::ToolCallContent::Diff(d) => ToolOutput::Diff {
            path: d.path.display().to_string(),
            old_text: d.old_text.clone(),
            new_text: d.new_text.clone(),
        },
        acp::ToolCallContent::Terminal(t) => ToolOutput::Terminal(t.terminal_id.to_string()),
        #[allow(unreachable_patterns)]
        _ => ToolOutput::Text("[unsupported tool content]".into()),
    }
}

/// The program to start for `command`. On Windows a bare name like `npx` is `npx.cmd` on disk,
/// which `CreateProcess` won't find without the extension: look it up on `PATH` with each
/// `PATHEXT` extension. Elsewhere (and for paths, or names that already have one) unchanged.
pub fn resolve_program(command: &std::path::Path) -> std::path::PathBuf {
    if !cfg!(windows) || command.extension().is_some() || command.components().count() > 1 {
        return command.to_path_buf();
    }
    let exts = std::env::var("PATHEXT").unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".into());
    let Some(path) = std::env::var_os("PATH") else {
        return command.to_path_buf();
    };
    for dir in std::env::split_paths(&path) {
        for ext in exts.split(';').filter(|e| !e.is_empty()) {
            let candidate = dir.join(format!("{}{}", command.display(), ext.to_lowercase()));
            if candidate.is_file() {
                return candidate;
            }
        }
    }
    command.to_path_buf()
}
