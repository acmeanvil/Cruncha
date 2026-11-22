// SPDX-License-Identifier: GPL-3.0-or-later
//! A stdio ⇄ Streamable-HTTP bridge for MCP (ADR-0009): agents that only start MCP servers as
//! subprocesses get Cruncha's server through it. Each line on stdin is a JSON-RPC message; it is
//! POSTed to the server, and every message in the answer (JSON or SSE) is written to stdout as
//! one line. The server's session id is carried between requests.
//!
//! The authorization header comes from the environment (`CRUNCHA_MCP_AUTH`), so it never
//! appears in a process listing.

use std::io::{BufRead, Write};

use serde_json::Value;

pub const AUTH_ENV: &str = "CRUNCHA_MCP_AUTH";

/// The messages in an SSE body.
fn sse_messages(body: &str) -> Vec<Value> {
    let mut out = Vec::new();
    let mut data = String::new();
    let mut flush = |data: &mut String| {
        if let Ok(v) = serde_json::from_str::<Value>(data) {
            out.push(v);
        }
        data.clear();
    };
    for line in body.lines() {
        if let Some(d) = line.strip_prefix("data:") {
            if !data.is_empty() {
                data.push('\n');
            }
            data.push_str(d.strip_prefix(' ').unwrap_or(d));
        } else if line.is_empty() {
            flush(&mut data);
        }
    }
    flush(&mut data);
    out
}

/// Forward stdin to `url` until stdin closes.
pub fn run(url: &str) -> std::io::Result<()> {
    let auth = std::env::var(AUTH_ENV).ok();
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(std::time::Duration::from_secs(600)))
        .http_status_as_error(false)
        .build()
        .new_agent();
    let mut session: Option<String> = None;
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let id = serde_json::from_str::<Value>(&line).ok().and_then(|v| v.get("id").cloned());
        let mut req = agent
            .post(url)
            .header("Accept", "application/json, text/event-stream")
            .header("Content-Type", "application/json");
        if let Some(a) = &auth {
            req = req.header("Authorization", a);
        }
        if let Some(s) = &session {
            req = req.header("Mcp-Session-Id", s);
        }
        let messages = match req.send(line.as_str()) {
            Ok(mut resp) => {
                if let Some(s) = resp.headers().get("mcp-session-id").and_then(|v| v.to_str().ok()) {
                    session = Some(s.to_owned());
                }
                let status = resp.status().as_u16();
                let sse = resp
                    .headers()
                    .get("content-type")
                    .and_then(|v| v.to_str().ok())
                    .is_some_and(|t| t.starts_with("text/event-stream"));
                let body = resp.body_mut().with_config().limit(64 << 20).read_to_string().unwrap_or_default();
                if status >= 400 {
                    error(&id, &format!("the Cruncha server answered HTTP {status}"))
                } else if sse {
                    sse_messages(&body)
                } else {
                    serde_json::from_str::<Value>(&body).map(|v| vec![v]).unwrap_or_default()
                }
            }
            Err(e) => error(&id, &format!("couldn't reach the Cruncha server: {e}")),
        };
        for m in messages {
            writeln!(stdout, "{m}")?;
        }
        stdout.flush()?;
    }
    Ok(())
}

/// A JSON-RPC error answer for a request (nothing for a notification).
fn error(id: &Option<Value>, message: &str) -> Vec<Value> {
    match id {
        Some(id) => vec![serde_json::json!({"jsonrpc": "2.0", "id": id, "error": {"code": -32603, "message": message}})],
        None => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_sse_message_is_forwarded() {
        let m = sse_messages("data: {\"a\":1}\n\nevent: message\ndata: {\"b\":2}\n\n");
        assert_eq!(m, vec![serde_json::json!({"a": 1}), serde_json::json!({"b": 2})]);
        assert!(error(&None, "x").is_empty());
    }
}
