// SPDX-License-Identifier: GPL-3.0-or-later
// Ported from looking_glass (crates/lg-agent/tests/anthropic_loop.rs), Copyright (c) 2026
// acmeanvil, MIT License; see LICENSE-LOOKING-GLASS. Changes for Cruncha: the tool is Cruncha's
// `explain`, served by a real cruncha-mcp, instead of a file write.

//! End-to-end: the agent's native Claude loop against a local mock of the Messages API.
//! Checks thinking blocks round-trip unchanged, tool inputs split across deltas, tool results
//! in one user message, required headers, usage reporting, and the model list grouped by family.

mod common;

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use futures::StreamExt;
use cruncha_acp::{AgentSpec, Command, ConfigCategory, Event};
use serde_json::{Value, json};

type Seen = Arc<Mutex<Vec<(HashMap<String, String>, Value)>>>;

/// Answers GET /v1/models with a fixed list and each POST /v1/messages with
/// the next canned SSE event list, recording headers and bodies.
fn mock(responses: Vec<Vec<Value>>) -> (String, Seen) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let seen: Seen = Arc::default();
    let seen2 = seen.clone();
    std::thread::spawn(move || {
        let mut responses = responses.into_iter();
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut request_line = String::new();
            reader.read_line(&mut request_line).unwrap();
            let mut headers = HashMap::new();
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if line == "\r\n" {
                    break;
                }
                if let Some((k, v)) = line.split_once(':') {
                    headers.insert(k.trim().to_ascii_lowercase(), v.trim().to_string());
                }
            }
            if request_line.starts_with("GET /v1/models") {
                let body = json!({"data": [
                    {"id": "claude-opus-5-5", "display_name": "Claude Opus 5.5", "max_input_tokens": 1000000},
                    {"id": "claude-sonnet-5-5", "display_name": "Claude Sonnet 5.5", "max_input_tokens": 1000000},
                    {"id": "claude-haiku-5-5", "display_name": "Claude Haiku 5.5", "max_input_tokens": 1000000}
                ], "has_more": false})
                .to_string();
                let out = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                stream.write_all(out.as_bytes()).unwrap();
                continue;
            }
            let Some(events) = responses.next() else {
                break;
            };
            let len: usize = headers.get("content-length").unwrap().parse().unwrap();
            let mut body = vec![0; len];
            reader.read_exact(&mut body).unwrap();
            seen2
                .lock()
                .unwrap()
                .push((headers, serde_json::from_slice(&body).unwrap()));
            let mut out = String::from(
                "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\n",
            );
            for e in events {
                out.push_str(&format!(
                    "event: {}\ndata: {e}\n\n",
                    e["type"].as_str().unwrap()
                ));
            }
            stream.write_all(out.as_bytes()).unwrap();
        }
    });
    (url, seen)
}

fn start(input: u64) -> Value {
    json!({"type": "message_start", "message": {"id": "msg_1", "type": "message", "role": "assistant",
        "content": [], "usage": {"input_tokens": input, "cache_read_input_tokens": 10,
        "cache_creation_input_tokens": 5, "output_tokens": 1}}})
}

#[test]
fn claude_loop_round_trips_thinking_and_tools() {
    let (url, seen) = mock(vec![
        vec![
            start(100),
            json!({"type": "content_block_start", "index": 0, "content_block": {"type": "thinking", "thinking": "", "signature": ""}}),
            json!({"type": "content_block_delta", "index": 0, "delta": {"type": "thinking_delta", "thinking": "Plan: explain M."}}),
            json!({"type": "content_block_delta", "index": 0, "delta": {"type": "signature_delta", "signature": "sig-abc"}}),
            json!({"type": "content_block_stop", "index": 0}),
            json!({"type": "content_block_start", "index": 1, "content_block": {"type": "tool_use", "id": "toolu_1", "name": "explain", "input": {}}}),
            json!({"type": "content_block_delta", "index": 1, "delta": {"type": "input_json_delta", "partial_json": "{\"cell_"}}),
            json!({"type": "content_block_delta", "index": 1, "delta": {"type": "input_json_delta", "partial_json": "id\": \"M\"}"}}),
            json!({"type": "content_block_stop", "index": 1}),
            json!({"type": "message_delta", "delta": {"stop_reason": "tool_use"}, "usage": {"output_tokens": 40}}),
            json!({"type": "message_stop"}),
        ],
        vec![
            start(200),
            json!({"type": "content_block_start", "index": 0, "content_block": {"type": "text", "text": ""}}),
            json!({"type": "content_block_delta", "index": 0, "delta": {"type": "text_delta", "text": "Done."}}),
            json!({"type": "content_block_stop", "index": 0}),
            json!({"type": "message_delta", "delta": {"stop_reason": "end_turn"}, "usage": {"output_tokens": 3}}),
            json!({"type": "message_stop"}),
        ],
    ]);

    let cwd = common::workdir("claude");
    let (agent, mut events) = cruncha_acp::connect(AgentSpec {
        name: "claude-e2e".into(),
        command: env!("CARGO_BIN_EXE_cruncha-agent").into(),
        args: ["--provider", "anthropic", "--base-url", &url]
            .map(String::from)
            .to_vec(),
        env: [("ANTHROPIC_API_KEY".to_string(), "test-key".to_string())].into(),
        cwd: cwd.clone(),
        resume: None,
        mcp_servers: vec![common::cruncha_server()],
    });

    let (mut text, mut thoughts, mut stop, mut usage_events) =
        (String::new(), String::new(), None, Vec::new());
    let mut groups: Vec<String> = Vec::new();
    futures::executor::block_on(async {
        while let Some(ev) = events.next().await {
            match ev {
                Event::SessionReady { .. } => {
                    agent.send(Command::Prompt("explain M".into()));
                }
                Event::ConfigOptions(opts) => {
                    if let Some(m) = opts.iter().find(|o| o.category == ConfigCategory::Model) {
                        groups = m.choices.iter().filter_map(|c| c.group.clone()).collect();
                    }
                }
                Event::AgentText(t) => text.push_str(&t),
                Event::Thought(t) => thoughts.push_str(&t),
                Event::Usage { used, size, totals } => usage_events.push((used, size, totals)),
                Event::Permission(mut p) => {
                    let allow = p
                        .options
                        .iter()
                        .find(|o| o.label == "Allow")
                        .map(|o| o.id.clone());
                    p.reply.answer(allow);
                }
                Event::TurnEnded(r) => {
                    stop = Some(r);
                    break;
                }
                Event::Disconnected { reason } => panic!("disconnected: {reason:?}"),
                _ => {}
            }
        }
    });

    assert_eq!(stop, Some(cruncha_acp::StopReason::EndTurn));
    assert_eq!(text, "Done.");
    assert_eq!(thoughts, "Plan: explain M.");
    assert!(
        groups.contains(&"Opus".to_string()) && groups.contains(&"Haiku".to_string()),
        "{groups:?}"
    );

    // Usage: one update per request; context = input + cache + output; totals accumulate.
    assert_eq!(usage_events.len(), 2);
    let (used, size, totals) = &usage_events[1];
    assert_eq!(*used, 200 + 10 + 5 + 3);
    assert_eq!(*size, 1_000_000);
    let totals = totals.as_ref().unwrap();
    assert_eq!(totals.input_tokens, 300);
    assert_eq!(totals.output_tokens, 43);
    assert_eq!(totals.requests, 2);

    let seen = seen.lock().unwrap();
    assert_eq!(seen.len(), 2);
    let (headers, first) = &seen[0];
    assert_eq!(headers["x-api-key"], "test-key");
    assert_eq!(headers["anthropic-version"], "2026-10-21");
    assert!(headers["anthropic-beta"].contains("server-side-fallback"));
    assert_eq!(first["model"], "claude-opus-5-5");
    assert_eq!(first["thinking"]["type"], "adaptive");
    assert_eq!(first["fallbacks"], "default");
    assert_eq!(first["tools"][0]["eager_input_streaming"], true);
    assert!(first.get("temperature").is_none());

    // The second request replays the thinking block unchanged, then one user
    // message holding the tool result.
    let msgs = seen[1].1["messages"].as_array().unwrap();
    let assistant = &msgs[1];
    assert_eq!(
        assistant["content"][0],
        json!({"type": "thinking", "thinking": "Plan: explain M.", "signature": "sig-abc"})
    );
    assert_eq!(
        assistant["content"][1]["input"],
        json!({"cell_id": "M"})
    );
    let results = &msgs[2];
    assert_eq!(results["role"], "user");
    assert_eq!(results["content"][0]["type"], "tool_result");
    assert_eq!(results["content"][0]["tool_use_id"], "toolu_1");
    assert_eq!(results["content"][0]["is_error"], false);
    assert!(results["content"][0]["content"].as_str().unwrap().contains("108 kip·ft"));
    std::fs::remove_dir_all(cwd).ok();
}
