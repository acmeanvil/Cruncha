//! The agent loop for Claude, on Anthropic's Messages API (raw HTTP: Rust
//! has no official SDK).
//!
//! Content blocks are rebuilt exactly as streamed and passed back unchanged
//! on every request (thinking blocks included, even empty ones), and the
//! history is append-only, so preserved thinking and the prompt cache both
//! keep working.

use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use std::time::Duration;

use agent_client_protocol::Client;
use agent_client_protocol::v1::ConnectionTo;
use agent_client_protocol::v1::schema::{
    ContentChunk, PromptRequest, SessionNotification, SessionUpdate, StopReason,
};
use futures::StreamExt;
use futures::channel::mpsc;
use serde_json::{Value, json};

use crate::llm::{error_message, prompt_text, resolve_model};
use crate::{RequestUsage, State, tools};

pub const API_VERSION: &str = "2026-10-21";
const MAX_STEPS: usize = 40;
const MAX_TOKENS: u64 = 64_000;
/// Server-side refusal fallbacks (re-run a declined request on another model).
const FALLBACK_BETA: &str = "server-side-fallback-2026-07-01";

enum Ev {
    BlockStart(usize, Value),
    Text(usize, String),
    Thinking(usize, String),
    Signature(usize, String),
    InputJson(usize, String),
    Usage(RequestUsage),
    Stop(String),
    Error(String),
    Done,
}

/// Models that take `thinking: {type: "adaptive"}` (Claude 4.6 and later).
fn adaptive_thinking(model: &str) -> bool {
    [
        "claude-fable",
        "claude-mythos",
        "claude-opus-5",
        "claude-sonnet-5",
        "claude-haiku-5",
        "claude-opus-4-6",
        "claude-opus-4-7",
        "claude-opus-4-8",
        "claude-sonnet-4-6",
    ]
    .iter()
    .any(|p| model.starts_with(p))
}

/// Models that accept `fallbacks: "default"`.
fn supports_fallbacks(model: &str) -> bool {
    [
        "claude-fable-5-2",
        "claude-fable-5-1",
        "claude-opus-5-5",
        "claude-opus-5",
        "claude-sonnet-5-5",
    ]
    .contains(&model)
}

/// Our tools, in Anthropic's shape. Inputs stream as they're generated, so
/// they're parsed strictly and checked before anything runs.
fn tool_definitions(openai: Value) -> Value {
    openai
        .as_array()
        .into_iter()
        .flatten()
        .map(|t| {
            let f = &t["function"];
            json!({
                "name": f["name"],
                "description": f["description"],
                "input_schema": f["parameters"],
                "eager_input_streaming": true,
            })
        })
        .collect()
}

fn stream_messages(
    cancel: Arc<std::sync::atomic::AtomicBool>,
    url: String,
    key: Option<String>,
    body: Value,
    fallbacks: bool,
) -> mpsc::UnboundedReceiver<Ev> {
    let (tx, rx) = mpsc::unbounded();
    std::thread::spawn(move || {
        let send = |e: Ev| {
            let _ = tx.unbounded_send(e);
        };
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .timeout_connect(Some(Duration::from_secs(30)))
            .build()
            .into();
        let mut req = agent
            .post(&url)
            .header("content-type", "application/json")
            .header("anthropic-version", API_VERSION)
            .header("x-api-key", key.unwrap_or_default());
        if fallbacks {
            req = req.header("anthropic-beta", FALLBACK_BETA);
        }
        let resp = match req.send_json(&body) {
            Ok(r) => r,
            Err(e) => {
                send(Ev::Error(format!("request to {url} failed: {e}")));
                return;
            }
        };
        let status = resp.status();
        if !status.is_success() {
            let text = resp.into_body().read_to_string().unwrap_or_default();
            send(Ev::Error(format!(
                "HTTP {status}: {}",
                error_message(&text)
            )));
            return;
        }
        let mut usage = RequestUsage::default();
        for line in BufReader::new(resp.into_body().into_reader()).lines() {
            if cancel.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    send(Ev::Error(format!("stream interrupted: {e}")));
                    return;
                }
            };
            let Some(data) = line.strip_prefix("data:") else {
                continue;
            };
            let Ok(v) = serde_json::from_str::<Value>(data.trim()) else {
                continue;
            };
            let index = v["index"].as_u64().unwrap_or(0) as usize;
            match v["type"].as_str().unwrap_or("") {
                "message_start" => {
                    let u = &v["message"]["usage"];
                    usage.input = u["input_tokens"].as_u64().unwrap_or(0);
                    usage.cache_read = u["cache_read_input_tokens"].as_u64().unwrap_or(0);
                    usage.cache_write = u["cache_creation_input_tokens"].as_u64().unwrap_or(0);
                    usage.output = u["output_tokens"].as_u64().unwrap_or(0);
                }
                "content_block_start" => send(Ev::BlockStart(index, v["content_block"].clone())),
                "content_block_delta" => {
                    let d = &v["delta"];
                    let s = |k: &str| d[k].as_str().unwrap_or("").to_string();
                    match d["type"].as_str().unwrap_or("") {
                        "text_delta" => send(Ev::Text(index, s("text"))),
                        "thinking_delta" => send(Ev::Thinking(index, s("thinking"))),
                        "signature_delta" => send(Ev::Signature(index, s("signature"))),
                        "input_json_delta" => send(Ev::InputJson(index, s("partial_json"))),
                        _ => {}
                    }
                }
                "message_delta" => {
                    if let Some(out) = v["usage"]["output_tokens"].as_u64() {
                        usage.output = out;
                    }
                    if let Some(reason) = v["delta"]["stop_reason"].as_str() {
                        let detail = v["delta"]["stop_details"]["category"]
                            .as_str()
                            .map(|c| format!(":{c}"))
                            .unwrap_or_default();
                        send(Ev::Stop(format!("{reason}{detail}")));
                    }
                }
                "error" => {
                    send(Ev::Error(error_message(&v.to_string())));
                    return;
                }
                _ => {}
            }
        }
        send(Ev::Usage(usage));
        send(Ev::Done);
    });
    rx
}

/// After a server-side fallback, blocks before the last `fallback` block
/// that the next model can't use (thinking, tool calls) are dropped when the
/// turn is echoed back; text and the fallback block itself stay in place.
fn echo_after_fallback(blocks: Vec<Value>) -> Vec<Value> {
    let Some(boundary) = blocks.iter().rposition(|b| b["type"] == "fallback") else {
        return blocks;
    };
    blocks
        .into_iter()
        .enumerate()
        .filter(|(i, b)| *i >= boundary || matches!(b["type"].as_str(), Some("text")))
        .map(|(_, b)| b)
        .collect()
}

pub async fn run_turn(
    cx: &ConnectionTo<Client>,
    state: &Arc<State>,
    req: PromptRequest,
) -> agent_client_protocol::Result<StopReason> {
    let provider = state.provider.clone().expect("provider mode");
    let session = req.session_id.clone();
    let say = |update: SessionUpdate| {
        cx.send_notification(SessionNotification::new(session.clone(), update))
    };
    let say_text = |text: String| {
        say(SessionUpdate::AgentMessageChunk(ContentChunk::new(
            text.into(),
        )))
    };
    let Some(model) = resolve_model(state, &session, &provider, &say_text).await? else {
        return Ok(StopReason::EndTurn);
    };
    let key = match provider.api_key() {
        Ok(k) => k,
        Err(msg) => {
            say_text(format!("**Can't start:** {msg}"))?;
            return Ok(StopReason::EndTurn);
        }
    };
    let (_cwd, mut messages) = {
        let sessions = state.sessions.lock().unwrap();
        let s = sessions.get(&session.to_string());
        (
            s.map(|s| s.cwd.clone()).unwrap_or_else(|| "/".into()),
            s.map(|s| s.messages.clone()).unwrap_or_default(),
        )
    };
    messages.push(json!({"role": "user", "content": prompt_text(&req)}));
    let save = |messages: &Vec<Value>| {
        if let Some(s) = state.sessions.lock().unwrap().get_mut(&session.to_string()) {
            s.messages = messages.clone();
        }
    };
    let ctx = tools::Ctx {
        cx,
        state,
        session: &session,
    };
    let system = state.system_prompt(&session);
    let fallbacks = supports_fallbacks(&model);

    for _ in 0..MAX_STEPS {
        let mut body = json!({
            "model": model,
            "max_tokens": MAX_TOKENS,
            "stream": true,
            "system": system,
            "messages": messages,
            // Caches the stable prefix (system, tools, history) automatically.
            "cache_control": {"type": "ephemeral"},
        });
        if provider.tools {
            body["tools"] = tool_definitions(tools::definitions(state, &session));
        }
        if adaptive_thinking(&model) {
            body["thinking"] = json!({"type": "adaptive", "display": "summarized"});
        }
        if fallbacks {
            body["fallbacks"] = json!("default");
        }
        let mut rx = stream_messages(
            state.cancel_flag(&session),
            provider.chat_url(),
            key.clone(),
            body,
            fallbacks,
        );

        let mut blocks: Vec<Value> = Vec::new();
        let mut partial_input: HashMap<usize, String> = HashMap::new();
        let mut stop = String::new();
        let mut error = None;
        let mut usage = None;
        while let Some(ev) = rx.next().await {
            match ev {
                Ev::BlockStart(i, block) => {
                    if blocks.len() <= i {
                        blocks.resize(i + 1, Value::Null);
                    }
                    blocks[i] = block;
                }
                Ev::Text(i, t) => {
                    if let Some(b) = blocks.get_mut(i) {
                        let joined = format!("{}{t}", b["text"].as_str().unwrap_or(""));
                        b["text"] = json!(joined);
                    }
                    say(SessionUpdate::AgentMessageChunk(ContentChunk::new(
                        t.into(),
                    )))?;
                }
                Ev::Thinking(i, t) => {
                    if let Some(b) = blocks.get_mut(i) {
                        let joined = format!("{}{t}", b["thinking"].as_str().unwrap_or(""));
                        b["thinking"] = json!(joined);
                    }
                    say(SessionUpdate::AgentThoughtChunk(ContentChunk::new(
                        t.into(),
                    )))?;
                }
                Ev::Signature(i, sig) => {
                    if let Some(b) = blocks.get_mut(i) {
                        let joined = format!("{}{sig}", b["signature"].as_str().unwrap_or(""));
                        b["signature"] = json!(joined);
                    }
                }
                Ev::InputJson(i, part) => partial_input.entry(i).or_default().push_str(&part),
                Ev::Usage(u) => usage = Some(u),
                Ev::Stop(reason) => stop = reason,
                Ev::Error(e) => error = Some(e),
                Ev::Done => break,
            }
        }
        if let Some(u) = usage {
            state.report_usage(cx, &session, &model, u)?;
        }
        if let Some(e) = error {
            say_text(format!("\n\n**Claude error:** {e}"))?;
            save(&messages);
            return Ok(StopReason::EndTurn);
        }

        // Finish tool_use blocks: parse inputs strictly. Invalid or truncated
        // JSON is answered as an error result instead of running the tool.
        let mut invalid: HashMap<String, String> = HashMap::new();
        for (i, b) in blocks.iter_mut().enumerate() {
            if b["type"] != "tool_use" {
                continue;
            }
            let raw = partial_input.remove(&i).unwrap_or_default();
            let raw = if raw.trim().is_empty() {
                "{}".to_string()
            } else {
                raw
            };
            match serde_json::from_str::<Value>(&raw) {
                Ok(v) if v.is_object() => b["input"] = v,
                _ => {
                    invalid.insert(b["id"].as_str().unwrap_or("").to_string(), raw);
                    b["input"] = json!({});
                }
            }
        }
        let blocks: Vec<Value> =
            echo_after_fallback(blocks.into_iter().filter(|b| !b.is_null()).collect());
        let calls: Vec<(String, String, Value)> = blocks
            .iter()
            .filter(|b| b["type"] == "tool_use")
            .map(|b| {
                (
                    b["id"].as_str().unwrap_or("").to_string(),
                    b["name"].as_str().unwrap_or("").to_string(),
                    b["input"].clone(),
                )
            })
            .collect();
        messages.push(json!({"role": "assistant", "content": blocks}));

        let reason = stop.split(':').next().unwrap_or("");
        // Never run a turn's tools after a refusal or a cut-off response; still
        // answer each tool call so the history stays valid.
        let blocked = match reason {
            "refusal" => Some(format!(
                "Claude declined this request{}.",
                stop.split_once(':')
                    .map(|(_, c)| format!(" ({c})"))
                    .unwrap_or_default()
            )),
            "max_tokens" | "model_context_window_exceeded" if !calls.is_empty() => {
                Some("The response was cut off before its tool calls finished.".into())
            }
            _ => None,
        };
        if let Some(why) = &blocked {
            say_text(format!("\n\n**{why}**"))?;
        }
        if state.cancelled(&session) || calls.is_empty() || blocked.is_some() {
            if !calls.is_empty() {
                let note = blocked.unwrap_or_else(|| "Cancelled by the user.".into());
                let results: Vec<Value> = calls
                    .iter()
                    .map(|(id, _, _)| {
                        json!({"type": "tool_result", "tool_use_id": id, "content": note, "is_error": true})
                    })
                    .collect();
                messages.push(json!({"role": "user", "content": results}));
            }
            save(&messages);
            return Ok(if state.cancelled(&session) {
                StopReason::Cancelled
            } else {
                StopReason::EndTurn
            });
        }

        // Run the tools; every result goes back in one user message.
        let mut results = Vec::new();
        for (id, name, input) in &calls {
            if state.cancelled(&session) {
                results.push(json!({"type": "tool_result", "tool_use_id": id,
                    "content": "Cancelled by the user.", "is_error": true}));
                continue;
            }
            if let Some(raw) = invalid.get(id) {
                results.push(
                    json!({"type": "tool_result", "tool_use_id": id, "is_error": true,
                    "content": json!({"INVALID_JSON": raw}).to_string()}),
                );
                continue;
            }
            let output = tools::run(&ctx, name, &input.to_string(), id).await?;
            let is_error = output.starts_with("error:");
            results.push(json!({"type": "tool_result", "tool_use_id": id,
                "content": output, "is_error": is_error}));
        }
        messages.push(json!({"role": "user", "content": results}));
        if state.cancelled(&session) {
            save(&messages);
            return Ok(StopReason::Cancelled);
        }
    }
    say_text(format!("\n\nStopped after {MAX_STEPS} steps."))?;
    save(&messages);
    Ok(StopReason::MaxTurnRequests)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_feature_tables() {
        assert!(adaptive_thinking("claude-opus-5-5"));
        assert!(adaptive_thinking("claude-fable-5-2"));
        assert!(!adaptive_thinking("claude-haiku-4-5"));
        assert!(supports_fallbacks("claude-opus-5-5"));
        assert!(!supports_fallbacks("claude-haiku-5-5"));
    }

    #[test]
    fn fallback_echo_drops_unusable_blocks_before_the_boundary() {
        let blocks = vec![
            json!({"type": "thinking", "thinking": "", "signature": "s"}),
            json!({"type": "text", "text": "partial"}),
            json!({"type": "tool_use", "id": "t1", "name": "x", "input": {}}),
            json!({"type": "fallback", "from": {}, "to": {}}),
            json!({"type": "thinking", "thinking": "", "signature": "s2"}),
            json!({"type": "text", "text": "rest"}),
        ];
        let kept: Vec<String> = echo_after_fallback(blocks)
            .iter()
            .map(|b| b["type"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(kept, ["text", "fallback", "thinking", "text"]);
    }

    #[test]
    fn tools_have_anthropic_shape() {
        let t = tool_definitions(serde_json::json!([{"type": "function", "function": {
            "name": "read_doc", "description": "Read", "parameters": {"type": "object"}}}]));
        let first = &t[0];
        assert!(first["input_schema"].is_object());
        assert_eq!(first["eager_input_streaming"], true);
        assert!(first.get("function").is_none());
    }
}
