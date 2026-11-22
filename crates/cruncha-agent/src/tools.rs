// SPDX-License-Identifier: GPL-3.0-or-later
// Ported from looking_glass (crates/lg-agent/src/tools.rs), Copyright (c) 2026 acmeanvil, MIT
// License; see LICENSE-LOOKING-GLASS. Changes for Cruncha (ADR-0009): the coding tools (files,
// commands) are gone; the tools are the session's MCP servers' tools plus `load_skill`.

//! Tools the model can call: each MCP tool of the session's servers (Cruncha's document tools),
//! and `load_skill`. Each call is shown to the client as a tool card. None needs permission:
//! the document tools only read, or make proposals a human accepts.

use std::sync::{Arc, Mutex};

use agent_client_protocol::Client;
use agent_client_protocol::v1::ConnectionTo;
use agent_client_protocol::v1::schema::{
    SessionId, SessionNotification, SessionUpdate, ToolCall, ToolCallStatus, ToolCallUpdate, ToolCallUpdateFields,
    ToolKind,
};
use serde_json::{Value, json};

use crate::State;
use crate::mcp;

/// Cap on what a tool card shows (the model gets up to `mcp::MAX_RESULT_CHARS`).
const MAX_SHOWN_CHARS: usize = 4_000;

/// One tool of a session: which server serves it.
#[derive(Clone)]
pub struct SessionTool {
    pub tool: mcp::Tool,
    pub server: Arc<Mutex<mcp::Client>>,
    pub server_name: String,
}

pub struct Ctx<'a> {
    pub cx: &'a ConnectionTo<Client>,
    pub state: &'a State,
    pub session: &'a SessionId,
}

const LOAD_SKILL: &str = "load_skill";

/// The session's tools, in OpenAI's function shape.
pub fn definitions(state: &State, session: &SessionId) -> Value {
    let sessions = state.sessions.lock().unwrap();
    let Some(s) = sessions.get(&session.to_string()) else { return json!([]) };
    let mut out: Vec<Value> = s
        .tools
        .iter()
        .map(|t| {
            json!({"type": "function", "function": {
                "name": t.tool.name, "description": t.tool.description, "parameters": t.tool.schema,
            }})
        })
        .collect();
    if !s.skills.is_empty() {
        let names: Vec<&str> = s.skills.iter().map(|k| k.name.as_str()).collect();
        out.push(json!({"type": "function", "function": {
            "name": LOAD_SKILL,
            "description": "Load a skill's full instructions. Load a skill before doing what its description covers.",
            "parameters": {"type": "object", "properties": {
                "name": {"type": "string", "enum": names, "description": "The skill's name"}
            }, "required": ["name"]}
        }}));
    }
    json!(out)
}

/// The tool names a session has, for its system prompt.
pub fn names(tools: &[SessionTool], has_skills: bool) -> Vec<String> {
    let mut n: Vec<String> = tools.iter().map(|t| t.tool.name.clone()).collect();
    if has_skills {
        n.push(LOAD_SKILL.into());
    }
    n
}

impl Ctx<'_> {
    fn send(&self, update: SessionUpdate) -> agent_client_protocol::Result<()> {
        self.cx.send_notification(SessionNotification::new(self.session.clone(), update))
    }

    fn finish(&self, id: &str, ok: bool, text: &str) -> agent_client_protocol::Result<()> {
        let shown: String = if text.chars().count() > MAX_SHOWN_CHARS {
            text.chars().take(MAX_SHOWN_CHARS).collect::<String>() + "\n…"
        } else {
            text.to_owned()
        };
        self.send(SessionUpdate::ToolCallUpdate(ToolCallUpdate::new(
            id.to_string(),
            ToolCallUpdateFields::new()
                .status(if ok { ToolCallStatus::Completed } else { ToolCallStatus::Failed })
                .content(vec![shown.into()]),
        )))
    }
}

/// How a tool reads in the client: proposals are edits, the rest reads.
fn kind_of(name: &str) -> ToolKind {
    match name {
        "propose_edit" | "withdraw" => ToolKind::Edit,
        "run" | "check" => ToolKind::Execute,
        _ => ToolKind::Read,
    }
}

fn title_of(name: &str, args: &Value) -> String {
    let arg = |k: &str| args[k].as_str().map(str::to_owned);
    match name {
        "explain" => format!("Explain {}", arg("cell_id").unwrap_or_default()),
        "propose_edit" => format!("Propose: {}", arg("message").unwrap_or_else(|| "an edit".into())),
        LOAD_SKILL => format!("Load skill {}", arg("name").unwrap_or_default()),
        other => other.replace('_', " "),
    }
}

/// Run one tool call and return the text the model sees (`error: …` when it failed).
pub async fn run(ctx: &Ctx<'_>, name: &str, args: &str, id: &str) -> agent_client_protocol::Result<String> {
    let args: Value = match serde_json::from_str(if args.trim().is_empty() { "{}" } else { args }) {
        Ok(v) => v,
        Err(e) => {
            ctx.send(SessionUpdate::ToolCall(
                ToolCall::new(id.to_string(), format!("{name} (invalid arguments)")).status(ToolCallStatus::Failed),
            ))?;
            return Ok(format!("error: arguments were not valid JSON: {e}"));
        }
    };
    ctx.send(SessionUpdate::ToolCall(
        ToolCall::new(id.to_string(), title_of(name, &args))
            .kind(kind_of(name))
            .status(ToolCallStatus::InProgress)
            .raw_input(args.clone()),
    ))?;

    let (tool, skill) = {
        let sessions = ctx.state.sessions.lock().unwrap();
        let s = sessions.get(&ctx.session.to_string());
        let tool = s.and_then(|s| s.tools.iter().find(|t| t.tool.name == name).cloned());
        let skill = (name == LOAD_SKILL)
            .then(|| s.and_then(|s| s.skills.iter().find(|k| Some(k.name.as_str()) == args["name"].as_str()).cloned()))
            .flatten();
        (tool, skill)
    };
    if name == LOAD_SKILL {
        return Ok(match skill {
            Some(k) => {
                ctx.finish(id, true, &format!("Loaded {}", k.name))?;
                k.body
            }
            None => {
                ctx.finish(id, false, "No such skill")?;
                format!("error: there is no skill named {}", args["name"])
            }
        });
    }
    let Some(tool) = tool else {
        ctx.finish(id, false, &format!("Unknown tool {name}"))?;
        return Ok(format!("error: there is no tool named {name}"));
    };
    // The call blocks (HTTP), so it runs off the ACP loop.
    let (tx, rx) = futures::channel::oneshot::channel();
    let call_name = tool.tool.name.clone();
    std::thread::spawn(move || {
        let r = tool.server.lock().unwrap().call(&call_name, args);
        let _ = tx.send(r);
    });
    let result = rx.await.unwrap_or_else(|_| Err("the tool call was dropped".into()));
    Ok(match result {
        Ok((text, false)) => {
            ctx.finish(id, true, &text)?;
            text
        }
        Ok((text, true)) => {
            ctx.finish(id, false, &text)?;
            format!("error: {text}")
        }
        Err(e) => {
            ctx.finish(id, false, &e)?;
            format!("error: {e}")
        }
    })
}
