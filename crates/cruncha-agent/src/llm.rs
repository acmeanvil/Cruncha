//! The agent loop for OpenAI-compatible providers: stream a completion,
//! show text and reasoning as they arrive, run any tool calls, repeat until
//! the model answers without tools.

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use std::time::Duration;

use agent_client_protocol::Client;
use agent_client_protocol::v1::ConnectionTo;
use agent_client_protocol::v1::schema::{
    ContentBlock, ContentChunk, PromptRequest, SessionNotification, SessionUpdate, StopReason,
};
use futures::StreamExt;
use futures::channel::mpsc;
use serde_json::{Value, json};

use crate::State;
use crate::tools;

/// Upper bound on model calls per user turn.
const MAX_STEPS: usize = 40;

enum Delta {
    Text(String),
    Reasoning(String),
    ToolCall {
        index: u64,
        id: Option<String>,
        name: Option<String>,
        args: String,
    },
    Usage(crate::RequestUsage),
    Error(String),
    Done,
}

#[derive(Default)]
struct PartialCall {
    id: String,
    name: String,
    args: String,
}

/// Run the HTTP request and SSE parsing on a thread; deltas come back on a channel.
fn stream_chat(
    cancel: Arc<std::sync::atomic::AtomicBool>,
    url: String,
    key: Option<String>,
    body: Value,
) -> mpsc::UnboundedReceiver<Delta> {
    let (tx, rx) = mpsc::unbounded();
    std::thread::spawn(move || {
        let send = |d: Delta| {
            let _ = tx.unbounded_send(d);
        };
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .timeout_connect(Some(Duration::from_secs(30)))
            .build()
            .into();
        let mut req = agent
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream");
        if let Some(k) = &key {
            req = req.header("Authorization", format!("Bearer {k}"));
        }
        let resp = match req.send_json(&body) {
            Ok(r) => r,
            Err(e) => {
                send(Delta::Error(format!("request to {url} failed: {e}")));
                return;
            }
        };
        let status = resp.status();
        if !status.is_success() {
            let text = resp.into_body().read_to_string().unwrap_or_default();
            send(Delta::Error(format!(
                "HTTP {status}: {}",
                error_message(&text)
            )));
            return;
        }
        let reader = BufReader::new(resp.into_body().into_reader());
        for line in reader.lines() {
            if cancel.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    send(Delta::Error(format!("stream interrupted: {e}")));
                    return;
                }
            };
            let Some(data) = line.strip_prefix("data:") else {
                continue;
            };
            let data = data.trim();
            if data == "[DONE]" {
                break;
            }
            let Ok(v) = serde_json::from_str::<Value>(data) else {
                continue;
            };
            if let Some(err) = v.get("error") {
                send(Delta::Error(error_message(&err.to_string())));
                return;
            }
            // With stream_options.include_usage, the last chunk carries usage.
            if let Some(u) = v.get("usage").filter(|u| u.is_object()) {
                let cached = u["prompt_tokens_details"]["cached_tokens"]
                    .as_u64()
                    .unwrap_or(0);
                send(Delta::Usage(crate::RequestUsage {
                    input: u["prompt_tokens"]
                        .as_u64()
                        .unwrap_or(0)
                        .saturating_sub(cached),
                    output: u["completion_tokens"].as_u64().unwrap_or(0),
                    cache_read: cached,
                    cache_write: 0,
                }));
            }
            let Some(choice) = v["choices"].get(0) else {
                continue;
            };
            let d = &choice["delta"];
            // DeepSeek/Qwen use reasoning_content; some servers use reasoning.
            if let Some(r) = d["reasoning_content"].as_str().or(d["reasoning"].as_str())
                && !r.is_empty()
            {
                send(Delta::Reasoning(r.to_string()));
            }
            if let Some(t) = d["content"].as_str()
                && !t.is_empty()
            {
                send(Delta::Text(t.to_string()));
            }
            for tc in d["tool_calls"].as_array().into_iter().flatten() {
                send(Delta::ToolCall {
                    index: tc["index"].as_u64().unwrap_or(0),
                    id: tc["id"].as_str().map(str::to_string),
                    name: tc["function"]["name"].as_str().map(str::to_string),
                    args: tc["function"]["arguments"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                });
            }
        }
        send(Delta::Done);
    });
    rx
}

/// Pull a readable message out of an error body, which is usually JSON.
pub fn error_message(body: &str) -> String {
    let parsed: Option<Value> = serde_json::from_str(body).ok();
    let msg = parsed.as_ref().and_then(|v| {
        v["error"]["message"]
            .as_str()
            .or(v["message"].as_str())
            .or(v["error"].as_str())
            .or(v["detail"].as_str())
            .map(str::to_string)
    });
    let msg = msg.unwrap_or_else(|| body.trim().to_string());
    msg.chars().take(600).collect()
}

/// Splits `<think>…</think>` out of streamed content, for models that put
/// their reasoning inline instead of in a separate field.
#[derive(Default)]
struct ThinkSplitter {
    in_think: bool,
    pending: String,
}

impl ThinkSplitter {
    /// Returns (is_thought, text) pieces in order.
    fn feed(&mut self, chunk: &str) -> Vec<(bool, String)> {
        self.pending.push_str(chunk);
        let mut out = Vec::new();
        loop {
            let tag = if self.in_think { "</think>" } else { "<think>" };
            if let Some(at) = self.pending.find(tag) {
                let before: String = self.pending[..at].to_string();
                if !before.is_empty() {
                    out.push((self.in_think, before));
                }
                self.pending.drain(..at + tag.len());
                self.in_think = !self.in_think;
                continue;
            }
            // Keep a possible partial tag at the end for the next chunk.
            let keep = (1..tag.len())
                .rev()
                .find(|&n| self.pending.ends_with(&tag[..n]))
                .unwrap_or(0);
            let emit_to = self.pending.len() - keep;
            if emit_to > 0 {
                out.push((self.in_think, self.pending[..emit_to].to_string()));
                self.pending.drain(..emit_to);
            }
            return out;
        }
    }

    fn flush(&mut self) -> Option<(bool, String)> {
        (!self.pending.is_empty()).then(|| (self.in_think, std::mem::take(&mut self.pending)))
    }
}

/// The session's model. Local servers without one take the first model the
/// server lists; hosted providers ask the user to choose. `None` means the
/// turn can't run (the reason has been shown).
pub async fn resolve_model(
    state: &Arc<State>,
    session: &agent_client_protocol::v1::schema::SessionId,
    provider: &crate::provider::Provider,
    say_text: &impl Fn(String) -> agent_client_protocol::Result<()>,
) -> agent_client_protocol::Result<Option<String>> {
    let model = state.model_for(session);
    if !model.is_empty() {
        return Ok(Some(model));
    }
    if !provider.local {
        say_text(format!(
            "**No model selected.** Pick one of {}'s models from the model menu in the session \
             header, or set one in Settings.",
            provider.label
        ))?;
        return Ok(None);
    }
    // The background model list may not have arrived yet: ask now.
    let (tx, rx) = futures::channel::oneshot::channel();
    let p = provider.clone();
    std::thread::spawn(move || {
        let _ = tx.send(p.fetch_models());
    });
    let listed = rx
        .await
        .unwrap_or_else(|_| Err("model list request failed".into()));
    if let Ok(models) = &listed
        && let Some(first) = models.first()
    {
        if let Some(s) = state.sessions.lock().unwrap().get_mut(&session.to_string())
            && s.model.is_empty()
        {
            s.model = first.id.clone();
        }
        return Ok(Some(first.id.clone()));
    }
    let why = match listed {
        Ok(_) => format!("{} has no models installed.", provider.base_url),
        Err(e) => format!("Couldn't list models from {}: {e}", provider.base_url),
    };
    say_text(format!(
        "**No model selected.** {why} Pick a model from the model menu in the session header, \
         or set one in Settings."
    ))?;
    Ok(None)
}

/// The local path in a `file://` URI (`file:///C:/x` is `C:/x` on Windows).
fn uri_path(uri: &str) -> String {
    let path = percent_decode(uri.strip_prefix("file://").unwrap_or(uri));
    let b = path.as_bytes();
    if b.len() >= 3 && b[0] == b'/' && b[1].is_ascii_alphabetic() && b[2] == b':' {
        path[1..].to_string()
    } else {
        path
    }
}

/// Undo `%XX` escapes (file URIs from the client); invalid escapes stay.
fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        let hex = |c: u8| (c as char).to_digit(16);
        if b[i] == b'%'
            && i + 2 < b.len()
            && let (Some(h), Some(l)) = (hex(b[i + 1]), hex(b[i + 2]))
        {
            out.push((h * 16 + l) as u8);
            i += 3;
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// The user's prompt as text; attached files become `[file: path]` lines the
/// model can open with read_file.
pub fn prompt_text(req: &PromptRequest) -> String {
    req.prompt
        .iter()
        .filter_map(|b| match b {
            ContentBlock::Text(t) => Some(t.text.clone()),
            ContentBlock::ResourceLink(l) => Some(format!("[file: {}]", uri_path(&l.uri))),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
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
    let prompt = prompt_text(&req);

    let (_cwd, mut messages) = {
        let sessions = state.sessions.lock().unwrap();
        let s = sessions.get(&session.to_string());
        (
            s.map(|s| s.cwd.clone()).unwrap_or_else(|| "/".into()),
            s.map(|s| s.messages.clone()).unwrap_or_default(),
        )
    };
    if messages.is_empty() {
        messages.push(json!({"role": "system", "content": state.system_prompt(&session)}));
    }
    messages.push(json!({"role": "user", "content": prompt}));
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

    for _ in 0..MAX_STEPS {
        let mut body = json!({
            "model": model,
            "messages": messages,
            "stream": true,
        });
        if provider.tools {
            body["tools"] = tools::definitions(state, &session);
        }
        if provider.stream_usage {
            body["stream_options"] = json!({"include_usage": true});
        }
        let mut rx = stream_chat(state.cancel_flag(&session), provider.chat_url(), key.clone(), body);

        let mut text = String::new();
        let mut reasoning = String::new();
        let mut calls: BTreeMap<u64, PartialCall> = BTreeMap::new();
        let mut splitter = ThinkSplitter::default();
        let mut error = None;
        let mut usage = None;
        let emit = |thought: bool, piece: String, text: &mut String, reasoning: &mut String| {
            if thought {
                reasoning.push_str(&piece);
                say(SessionUpdate::AgentThoughtChunk(ContentChunk::new(
                    piece.into(),
                )))
            } else {
                text.push_str(&piece);
                say(SessionUpdate::AgentMessageChunk(ContentChunk::new(
                    piece.into(),
                )))
            }
        };
        while let Some(delta) = rx.next().await {
            match delta {
                Delta::Text(t) => {
                    for (thought, piece) in splitter.feed(&t) {
                        emit(thought, piece, &mut text, &mut reasoning)?;
                    }
                }
                Delta::Reasoning(r) => emit(true, r, &mut text, &mut reasoning)?,
                Delta::ToolCall {
                    index,
                    id,
                    name,
                    args,
                } => {
                    let call = calls.entry(index).or_default();
                    if let Some(id) = id {
                        call.id = id;
                    }
                    if let Some(name) = name {
                        call.name.push_str(&name);
                    }
                    call.args.push_str(&args);
                }
                Delta::Usage(u) => usage = Some(u),
                Delta::Error(e) => error = Some(e),
                Delta::Done => break,
            }
        }
        if let Some(u) = usage {
            state.report_usage(cx, &session, &model, u)?;
        }
        if let Some((thought, piece)) = splitter.flush() {
            emit(thought, piece, &mut text, &mut reasoning)?;
        }

        if let Some(e) = error {
            say_text(format!("\n\n**{} error:** {e}", provider.label))?;
            save(&messages);
            return Ok(StopReason::EndTurn);
        }

        let mut assistant = json!({"role": "assistant", "content": text});
        if provider.echo_reasoning {
            assistant["reasoning_content"] = json!(reasoning);
        }
        let calls: Vec<PartialCall> = calls
            .into_values()
            .filter(|c| !c.name.is_empty())
            .map(|mut c| {
                if c.id.is_empty() {
                    c.id = state.next_id("call");
                }
                c
            })
            .collect();
        if !calls.is_empty() {
            if text.is_empty() {
                assistant["content"] = Value::Null;
            }
            assistant["tool_calls"] = calls
                .iter()
                .map(|c| {
                    json!({
                        "id": c.id,
                        "type": "function",
                        "function": {"name": c.name, "arguments": c.args},
                    })
                })
                .collect();
        }
        messages.push(assistant);

        if state.cancelled(&session) {
            save(&messages);
            return Ok(StopReason::Cancelled);
        }
        if calls.is_empty() {
            save(&messages);
            return Ok(StopReason::EndTurn);
        }
        for call in &calls {
            let result = tools::run(&ctx, &call.name, &call.args, &call.id).await?;
            messages.push(json!({"role": "tool", "tool_call_id": call.id, "content": result}));
            if state.cancelled(&session) {
                // Every tool call needs a result, or the next request is rejected.
                for rest in calls.iter().skip_while(|c| c.id != call.id).skip(1) {
                    messages.push(json!({
                        "role": "tool", "tool_call_id": rest.id, "content": "Cancelled by the user."
                    }));
                }
                save(&messages);
                return Ok(StopReason::Cancelled);
            }
        }
    }
    say_text(format!("\n\nStopped after {MAX_STEPS} steps."))?;
    save(&messages);
    Ok(StopReason::MaxTurnRequests)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(chunks: &[&str]) -> Vec<(bool, String)> {
        let mut s = ThinkSplitter::default();
        let mut out: Vec<(bool, String)> = Vec::new();
        for c in chunks {
            out.extend(s.feed(c));
        }
        out.extend(s.flush());
        // Merge adjacent pieces of the same kind for easy comparison.
        let mut merged: Vec<(bool, String)> = Vec::new();
        for (t, p) in out {
            match merged.last_mut() {
                Some((lt, lp)) if *lt == t => lp.push_str(&p),
                _ => merged.push((t, p)),
            }
        }
        merged
    }

    #[test]
    fn splits_think_tags_across_chunks() {
        assert_eq!(
            run(&["<th", "ink>plan it</thi", "nk>Answer <b>x</b>"]),
            vec![(true, "plan it".into()), (false, "Answer <b>x</b>".into())]
        );
        assert_eq!(run(&["no tags here"]), vec![(false, "no tags here".into())]);
    }

    #[test]
    fn readable_errors() {
        assert_eq!(
            error_message(r#"{"error":{"message":"Invalid API key","type":"auth"}}"#),
            "Invalid API key"
        );
        assert_eq!(error_message("plain failure"), "plain failure");
    }
}

#[cfg(test)]
mod percent_tests {
    #[test]
    fn decodes_file_uri_escapes() {
        assert_eq!(
            super::percent_decode("/a%20b/%231%20100%25/caf%C3%A9.png"),
            "/a b/#1 100%/café.png"
        );
        assert_eq!(super::percent_decode("50%"), "50%");
        assert_eq!(
            super::uri_path("file:///C:/My%20Docs/a.txt"),
            "C:/My Docs/a.txt"
        );
        assert_eq!(super::uri_path("file:///home/me/a.txt"), "/home/me/a.txt");
        assert_eq!(super::percent_decode("%zz"), "%zz");
    }
}
