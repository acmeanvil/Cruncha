// SPDX-License-Identifier: GPL-3.0-or-later
//! A small MCP client over Streamable HTTP (ADR-0009): the built-in agent's tools are the
//! tools of the MCP servers its session was given (Cruncha's `cruncha-mcp`). Blocking, on
//! `ureq`; callers run it off the ACP loop.
//!
//! Requests are JSON-RPC `POST`s. A server may answer with JSON or with a short SSE stream;
//! both are read. The session id the server assigns (`Mcp-Session-Id`) is sent back on every
//! later request.

use serde_json::{Value, json};

const PROTOCOL: &str = "2025-06-18";
/// Cap on what one tool returns to the model.
const MAX_RESULT_CHARS: usize = 40_000;

/// A tool a server offers.
#[derive(Debug, Clone, PartialEq)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub schema: Value,
}

/// A connection to one MCP server.
pub struct Client {
    url: String,
    headers: Vec<(String, String)>,
    session: Option<String>,
    next_id: u64,
    agent: ureq::Agent,
}

impl Client {
    /// Connect (initialize) and list the server's tools.
    pub fn connect(url: &str, headers: Vec<(String, String)>) -> Result<(Client, Vec<Tool>), String> {
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(std::time::Duration::from_secs(300)))
            .http_status_as_error(false)
            .build()
            .new_agent();
        let mut c = Client { url: url.to_owned(), headers, session: None, next_id: 0, agent };
        c.rpc(
            "initialize",
            json!({
                "protocolVersion": PROTOCOL,
                "capabilities": {},
                "clientInfo": {"name": "cruncha-agent", "version": env!("CARGO_PKG_VERSION")},
            }),
        )?;
        c.notify("notifications/initialized")?;
        let mut tools = Vec::new();
        let mut cursor: Option<String> = None;
        loop {
            let params = match &cursor {
                Some(c) => json!({"cursor": c}),
                None => json!({}),
            };
            let r = c.rpc("tools/list", params)?;
            for t in r["tools"].as_array().into_iter().flatten() {
                tools.push(Tool {
                    name: t["name"].as_str().unwrap_or_default().to_owned(),
                    description: t["description"].as_str().unwrap_or_default().to_owned(),
                    schema: t.get("inputSchema").cloned().unwrap_or(json!({"type": "object"})),
                });
            }
            cursor = r["nextCursor"].as_str().map(str::to_owned);
            if cursor.is_none() {
                break;
            }
        }
        Ok((c, tools))
    }

    /// Call a tool. Returns its text and whether the server marked it an error.
    pub fn call(&mut self, name: &str, args: Value) -> Result<(String, bool), String> {
        let r = self.rpc("tools/call", json!({"name": name, "arguments": args}))?;
        let mut text = String::new();
        for c in r["content"].as_array().into_iter().flatten() {
            match c["type"].as_str() {
                Some("text") => text.push_str(c["text"].as_str().unwrap_or_default()),
                Some(other) => text.push_str(&format!("[{other} content]")),
                None => {}
            }
        }
        if text.is_empty()
            && let Some(s) = r.get("structuredContent")
        {
            text = s.to_string();
        }
        if text.chars().count() > MAX_RESULT_CHARS {
            text = text.chars().take(MAX_RESULT_CHARS).collect::<String>() + "\n… (truncated)";
        }
        Ok((text, r["isError"].as_bool().unwrap_or(false)))
    }

    fn request(&self) -> ureq::RequestBuilder<ureq::typestate::WithBody> {
        let mut req = self
            .agent
            .post(&self.url)
            .header("Accept", "application/json, text/event-stream")
            .header("Content-Type", "application/json")
            .header("MCP-Protocol-Version", PROTOCOL);
        if let Some(s) = &self.session {
            req = req.header("Mcp-Session-Id", s);
        }
        for (k, v) in &self.headers {
            req = req.header(k, v);
        }
        req
    }

    fn notify(&mut self, method: &str) -> Result<(), String> {
        let body = json!({"jsonrpc": "2.0", "method": method}).to_string();
        let resp = self.request().send(body).map_err(|e| e.to_string())?;
        if resp.status().as_u16() >= 400 {
            return Err(format!("{method}: HTTP {}", resp.status()));
        }
        Ok(())
    }

    fn rpc(&mut self, method: &str, params: Value) -> Result<Value, String> {
        self.next_id += 1;
        let id = self.next_id;
        let body = json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}).to_string();
        let mut resp = self.request().send(body).map_err(|e| format!("{method}: {e}"))?;
        if let Some(s) = resp.headers().get("mcp-session-id").and_then(|v| v.to_str().ok()) {
            self.session = Some(s.to_owned());
        }
        let status = resp.status().as_u16();
        let sse = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .is_some_and(|t| t.starts_with("text/event-stream"));
        let text = resp
            .body_mut()
            .with_config()
            .limit(64 << 20)
            .read_to_string()
            .map_err(|e| format!("{method}: {e}"))?;
        if status >= 400 {
            return Err(format!("{method}: HTTP {status}: {}", text.chars().take(300).collect::<String>()));
        }
        let message = if sse { from_sse(&text, id) } else { serde_json::from_str(&text).ok() };
        let Some(m) = message else {
            return Err(format!("{method}: no answer from the server"));
        };
        if let Some(e) = m.get("error") {
            return Err(format!("{method}: {}", e["message"].as_str().unwrap_or("error")));
        }
        Ok(m["result"].clone())
    }
}

/// The JSON-RPC response with `id` in an SSE body.
fn from_sse(body: &str, id: u64) -> Option<Value> {
    let mut data = String::new();
    let mut found = None;
    let mut flush = |data: &mut String| {
        if let Ok(v) = serde_json::from_str::<Value>(data)
            && v["id"].as_u64() == Some(id)
        {
            found = Some(v);
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
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_matching_response_from_sse() {
        let body = "event: message\ndata: {\"jsonrpc\":\"2.0\",\"method\":\"notifications/progress\"}\n\n\
                    data: {\"jsonrpc\":\"2.0\",\"id\":3,\ndata: \"result\":{\"ok\":true}}\n\n";
        // Multi-line data joins with newlines (valid JSON whitespace).
        assert_eq!(from_sse(body, 3).unwrap()["result"]["ok"], true);
        assert!(from_sse(body, 4).is_none());
    }
}
