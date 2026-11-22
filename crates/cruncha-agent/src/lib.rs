// SPDX-License-Identifier: GPL-3.0-or-later
// Ported from looking_glass (crates/lg-agent), Copyright (c) 2026 acmeanvil, MIT License; see
// LICENSE-LOOKING-GLASS. Changes for Cruncha (ADR-0009): its tools are the session's MCP servers'
// (Cruncha's document tools) plus `load_skill` instead of files and commands; the system prompt
// is per document with a skills loader; cancellation is per session; the demo mode reads the
// document instead of exercising files and terminals.

//! `cruncha-agent`: Cruncha's built-in ACP agent.
//!
//! Two modes:
//! - demo (no flags): a scripted turn that reads the document through its tools, no key needed
//! - provider: an agent on a model API (`--provider anthropic|openai|google|huggingface|
//!   deepseek|qwen|nvidia|tinker|ollama|lmstudio|openai-compatible`); Claude uses Anthropic's
//!   Messages API natively, the rest OpenAI Chat Completions
//!
//! Either way it speaks ACP on stdin/stdout, so Cruncha's agent panel treats it like any other
//! agent, and it works on the document only through the MCP servers the session is given.

mod anthropic;
pub mod bridge;
mod demo;
mod llm;
mod mcp;
mod provider;
mod skills;
mod tools;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use agent_client_protocol::v1::ConnectionTo;
use agent_client_protocol::v1::schema::{
    AgentCapabilities, CancelNotification, ClientCapabilities, ConfigOptionUpdate, Implementation,
    InitializeRequest, InitializeResponse, McpCapabilities, McpServer, NewSessionRequest, NewSessionResponse, PromptRequest,
    PromptResponse, SessionConfigOption, SessionConfigOptionCategory, SessionConfigSelectGroup,
    SessionConfigSelectOption, SessionConfigSelectOptions, SessionId, SessionNotification,
    SessionUpdate, SetSessionConfigOptionRequest, SetSessionConfigOptionResponse, UsageUpdate,
};
use agent_client_protocol::{Agent, Client, Stdio};

use crate::provider::{ModelInfo, Provider};

/// An MCP server over HTTP: name, URL, headers.
type HttpServer = (String, String, Vec<(String, String)>);

/// Per-session state for provider mode.
#[derive(Default)]
pub struct SessionState {
    pub cwd: PathBuf,
    /// The conversation so far, as OpenAI-style messages.
    pub messages: Vec<serde_json::Value>,
    /// The tools of the session's MCP servers.
    pub tools: Vec<tools::SessionTool>,
    /// The skills in the working directory (`.claude/skills`).
    pub skills: Vec<skills::Skill>,
    /// Set by `session/cancel`; cleared when a turn starts.
    pub cancelled: Arc<AtomicBool>,
    /// The model this session uses; empty until one is chosen (local servers).
    pub model: String,
    /// Token totals for the session (for the usage display).
    pub usage: UsageTotals,
}

#[derive(Default, Clone, Copy)]
pub struct UsageTotals {
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_write: u64,
    pub requests: u64,
}

/// Token counts for one model request.
#[derive(Default, Clone, Copy)]
pub struct RequestUsage {
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_write: u64,
}

#[derive(Default)]
pub struct State {
    pub client: Mutex<ClientCapabilities>,
    pub sessions: Mutex<HashMap<String, SessionState>>,
    pub next_id: AtomicU64,
    pub provider: Option<Provider>,
    /// Models the provider's server listed, once fetched.
    pub listed_models: Mutex<Vec<ModelInfo>>,
}

const DEMO_MODELS: &[&str] = &["demo-fast", "demo-thorough"];

impl State {
    pub fn cwd_for(&self, session: &SessionId) -> PathBuf {
        self.sessions
            .lock()
            .unwrap()
            .get(&session.to_string())
            .map(|s| s.cwd.clone())
            .unwrap_or_else(|| PathBuf::from("/"))
    }

    /// Whether the session's current turn was cancelled.
    pub fn cancelled(&self, session: &SessionId) -> bool {
        self.sessions
            .lock()
            .unwrap()
            .get(&session.to_string())
            .is_some_and(|s| s.cancelled.load(Ordering::SeqCst))
    }

    /// The session's cancel flag (for threads that stream a model's reply).
    pub fn cancel_flag(&self, session: &SessionId) -> Arc<AtomicBool> {
        self.sessions
            .lock()
            .unwrap()
            .get(&session.to_string())
            .map(|s| s.cancelled.clone())
            .unwrap_or_default()
    }

    /// The system prompt for a session: Cruncha's rules, the document's AGENTS.md, the tools
    /// and the skills index.
    pub fn system_prompt(&self, session: &SessionId) -> String {
        let sessions = self.sessions.lock().unwrap();
        match sessions.get(&session.to_string()) {
            Some(s) => skills::system_prompt(&s.cwd, &s.skills, &tools::names(&s.tools, !s.skills.is_empty())),
            None => skills::system_prompt(std::path::Path::new("/"), &[], &[]),
        }
    }

    pub fn model_for(&self, session: &SessionId) -> String {
        self.sessions
            .lock()
            .unwrap()
            .get(&session.to_string())
            .map(|s| s.model.clone())
            .unwrap_or_default()
    }

    /// The "model" setting offered to the client for a session.
    fn model_option(&self, session: &str) -> SessionConfigOption {
        let current = self
            .sessions
            .lock()
            .unwrap()
            .get(session)
            .map(|s| s.model.clone())
            .unwrap_or_default();
        let mut models: Vec<ModelInfo> = match &self.provider {
            None => DEMO_MODELS
                .iter()
                .map(|s| ModelInfo {
                    id: s.to_string(),
                    name: s.to_string(),
                    group: Some("Demo".into()),
                    context: None,
                })
                .collect(),
            Some(p) => p.suggested.clone(),
        };
        // The server's own list replaces suggestions with the same id (it has
        // better names and families).
        for m in self.listed_models.lock().unwrap().iter() {
            match models.iter_mut().find(|x| x.id == m.id) {
                Some(existing) => *existing = m.clone(),
                None => models.push(m.clone()),
            }
        }
        if !current.is_empty() && !models.iter().any(|m| m.id == current) {
            models.insert(
                0,
                ModelInfo {
                    id: current.clone(),
                    name: current.clone(),
                    group: None,
                    context: None,
                },
            );
        }
        let option = |m: &ModelInfo| {
            let desc = (m.name != m.id).then(|| m.id.clone());
            SessionConfigSelectOption::new(m.id.clone(), m.name.clone()).description(desc)
        };
        // Grouped by family (Opus / Sonnet / Haiku, Gemini 3, …) when every
        // model has one.
        let options: SessionConfigSelectOptions =
            if !models.is_empty() && models.iter().all(|m| m.group.is_some()) {
                let mut groups: Vec<(String, Vec<SessionConfigSelectOption>)> = Vec::new();
                for m in &models {
                    let g = m.group.clone().unwrap_or_default();
                    match groups.iter_mut().find(|(name, _)| *name == g) {
                        Some((_, opts)) => opts.push(option(m)),
                        None => groups.push((g, vec![option(m)])),
                    }
                }
                groups
                    .into_iter()
                    .map(|(name, opts)| SessionConfigSelectGroup::new(name.clone(), name, opts))
                    .collect::<Vec<_>>()
                    .into()
            } else {
                models.iter().map(option).collect::<Vec<_>>().into()
            };
        SessionConfigOption::select("model", "Model", current, options)
            .category(SessionConfigOptionCategory::Model)
    }

    /// The context window of a model, when the provider reported it.
    pub fn context_window(&self, model: &str) -> u64 {
        self.listed_models
            .lock()
            .unwrap()
            .iter()
            .find(|m| m.id == model)
            .and_then(|m| m.context)
            .unwrap_or(0)
    }

    /// Add one request's usage to the session and tell the client: `used` is
    /// what the last request put in the context window; totals ride in `_meta`.
    pub fn report_usage(
        &self,
        cx: &ConnectionTo<Client>,
        session: &SessionId,
        model: &str,
        u: RequestUsage,
    ) -> agent_client_protocol::Result<()> {
        let totals = {
            let mut sessions = self.sessions.lock().unwrap();
            let Some(s) = sessions.get_mut(&session.to_string()) else {
                return Ok(());
            };
            s.usage.input += u.input;
            s.usage.output += u.output;
            s.usage.cache_read += u.cache_read;
            s.usage.cache_write += u.cache_write;
            s.usage.requests += 1;
            s.usage
        };
        let used = u.input + u.cache_read + u.cache_write + u.output;
        let mut meta = serde_json::Map::new();
        meta.insert(
            "lookingGlass".into(),
            serde_json::json!({
                "model": model,
                "inputTokens": totals.input,
                "outputTokens": totals.output,
                "cacheReadTokens": totals.cache_read,
                "cacheWriteTokens": totals.cache_write,
                "requests": totals.requests,
            }),
        );
        let mut update = UsageUpdate::new(used, self.context_window(model));
        update.meta = Some(meta);
        cx.send_notification(SessionNotification::new(
            session.clone(),
            SessionUpdate::UsageUpdate(update),
        ))
    }

    pub fn next_id(&self, prefix: &str) -> String {
        format!(
            "{prefix}-{}",
            self.next_id.fetch_add(1, Ordering::Relaxed) + 1
        )
    }
}

fn usage() -> ! {
    eprintln!(
        "usage: cruncha-agent [--provider <name>] [--model <id>] [--base-url <url>] \
         [--api-key-env <VAR>] [--no-tools]\n\
         providers: {}",
        provider::NAMES.join(", ")
    );
    std::process::exit(2);
}

fn parse_args(args: Vec<String>) -> Option<Provider> {
    let mut name = None;
    let mut overrides = provider::Overrides::default();
    let mut it = args.into_iter();
    while let Some(arg) = it.next() {
        let mut value = || it.next().unwrap_or_else(|| usage());
        match arg.as_str() {
            "--provider" => name = Some(value()),
            "--model" => overrides.model = Some(value()),
            "--base-url" => overrides.base_url = Some(value()),
            "--api-key-env" => overrides.api_key_env = Some(value()),
            "--no-tools" => overrides.no_tools = true,
            _ => usage(),
        }
    }
    let name = name?;
    match Provider::new(&name, overrides) {
        Ok(p) => Some(p),
        Err(e) => {
            eprintln!("cruncha-agent: {e}");
            std::process::exit(2);
        }
    }
}

/// Run the agent on stdin/stdout. `args` are the flags after the program name.
pub fn run(args: Vec<String>) -> agent_client_protocol::Result<()> {
    let provider = parse_args(args);
    let title = match &provider {
        Some(p) if p.model.is_empty() => p.label.clone(),
        Some(p) => format!("{} · {}", p.label, p.model),
        None => "Cruncha agent (demo)".to_string(),
    };
    let state = Arc::new(State {
        provider,
        ..Default::default()
    });
    let (s0, s1, s2, s3, s4) = (
        state.clone(),
        state.clone(),
        state.clone(),
        state.clone(),
        state.clone(),
    );

    futures::executor::block_on(
        Agent
            .v1()
            .name("cruncha-agent")
            .on_receive_request(
                async move |req: InitializeRequest, responder, _cx| {
                    *s0.client.lock().unwrap() = req.client_capabilities.clone();
                    responder.respond(
                        InitializeResponse::new(req.protocol_version)
                            .agent_capabilities(
                                AgentCapabilities::new().mcp_capabilities(McpCapabilities::new().http(true)),
                            )
                            .agent_info(
                                Implementation::new("cruncha-agent", env!("CARGO_PKG_VERSION"))
                                    .title(title.clone()),
                            ),
                    )
                },
                agent_client_protocol::on_receive_request!(),
            )
            .on_receive_request(
                async move |req: NewSessionRequest, responder, cx: ConnectionTo<Client>| {
                    let id = s1.next_id("session");
                    let model = match &s1.provider {
                        Some(p) => p.model.clone(),
                        None => DEMO_MODELS[0].to_string(),
                    };
                    // Connect to the session's MCP servers (HTTP), off the ACP loop.
                    let servers: Vec<HttpServer> = req
                        .mcp_servers
                        .iter()
                        .filter_map(|m| match m {
                            McpServer::Http(h) => Some((
                                h.name.clone(),
                                h.url.clone(),
                                h.headers.iter().map(|x| (x.name.clone(), x.value.clone())).collect(),
                            )),
                            _ => None,
                        })
                        .collect();
                    let (tx, rx) = futures::channel::oneshot::channel();
                    std::thread::spawn(move || {
                        let mut found = Vec::new();
                        for (name, url, headers) in servers {
                            match mcp::Client::connect(&url, headers) {
                                Ok((client, list)) => {
                                    let client = Arc::new(Mutex::new(client));
                                    found.extend(list.into_iter().map(|tool| tools::SessionTool {
                                        tool,
                                        server: client.clone(),
                                        server_name: name.clone(),
                                    }));
                                }
                                Err(e) => eprintln!("cruncha-agent: MCP server {name}: {e}"),
                            }
                        }
                        let _ = tx.send(found);
                    });
                    let session_tools = rx.await.unwrap_or_default();
                    let session_skills = skills::load(&req.cwd);
                    s1.sessions.lock().unwrap().insert(
                        id.clone(),
                        SessionState {
                            cwd: req.cwd,
                            model,
                            tools: session_tools,
                            skills: session_skills,
                            ..Default::default()
                        },
                    );
                    responder.respond(
                        NewSessionResponse::new(id.clone())
                            .config_options(vec![s1.model_option(&id)]),
                    )?;
                    // Ask the server which models it has, then offer them.
                    let Some(provider) = s1.provider.clone() else {
                        return Ok(());
                    };
                    let (tx, rx) = futures::channel::oneshot::channel();
                    std::thread::spawn(move || {
                        let _ = tx.send(provider.fetch_models());
                    });
                    let state = s1.clone();
                    cx.clone().spawn(async move {
                        let Ok(Ok(models)) = rx.await else {
                            return Ok(());
                        };
                        if models.is_empty() {
                            return Ok(());
                        }
                        *state.listed_models.lock().unwrap() = models.clone();
                        if let Some(s) = state.sessions.lock().unwrap().get_mut(&id)
                            && s.model.is_empty()
                        {
                            s.model = models[0].id.clone();
                        }
                        cx.send_notification(SessionNotification::new(
                            id.clone(),
                            SessionUpdate::ConfigOptionUpdate(ConfigOptionUpdate::new(vec![
                                state.model_option(&id),
                            ])),
                        ))
                    })
                },
                agent_client_protocol::on_receive_request!(),
            )
            .on_receive_request(
                async move |req: SetSessionConfigOptionRequest, responder, _cx| {
                    let id = req.session_id.to_string();
                    let value = match serde_json::to_value(&req.value) {
                        Ok(v) => v["value"].as_str().map(str::to_string),
                        Err(_) => None,
                    };
                    match (req.config_id.to_string().as_str(), value) {
                        ("model", Some(model)) if !model.trim().is_empty() => {
                            if let Some(s) = s4.sessions.lock().unwrap().get_mut(&id) {
                                s.model = model.trim().to_string();
                            }
                            responder.respond(SetSessionConfigOptionResponse::new(vec![
                                s4.model_option(&id),
                            ]))
                        }
                        _ => responder.respond_with_error(
                            agent_client_protocol::Error::invalid_params()
                                .data(serde_json::json!("unknown setting or value")),
                        ),
                    }
                },
                agent_client_protocol::on_receive_request!(),
            )
            .on_receive_notification(
                async move |n: CancelNotification, _cx| {
                    if let Some(s) = s2.sessions.lock().unwrap().get(&n.session_id.to_string()) {
                        s.cancelled.store(true, Ordering::SeqCst);
                    }
                    Ok(())
                },
                agent_client_protocol::on_receive_notification!(),
            )
            .on_receive_request(
                async move |req: PromptRequest, responder, cx: ConnectionTo<Client>| {
                    let state = s3.clone();
                    // A turn requests permission from the client, so it must
                    // run outside the dispatch loop.
                    cx.clone().spawn(async move {
                        if let Some(s) = state.sessions.lock().unwrap().get(&req.session_id.to_string()) {
                            s.cancelled.store(false, Ordering::SeqCst);
                        }
                        let stop = match state.provider.as_ref().map(|p| p.api) {
                            Some(provider::Api::Anthropic) => {
                                anthropic::run_turn(&cx, &state, req).await?
                            }
                            Some(provider::Api::OpenAiChat) => {
                                llm::run_turn(&cx, &state, req).await?
                            }
                            None => demo::run_turn(&cx, &state, req).await?,
                        };
                        responder.respond(PromptResponse::new(stop))
                    })
                },
                agent_client_protocol::on_receive_request!(),
            )
            .connect_to(Stdio::new()),
    )
}
