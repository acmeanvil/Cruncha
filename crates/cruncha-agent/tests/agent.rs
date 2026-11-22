// SPDX-License-Identifier: GPL-3.0-or-later
// The mock model server is ported from looking_glass (crates/lg-agent/tests/provider_loop.rs),
// Copyright (c) 2026 acmeanvil, MIT License; see LICENSE-LOOKING-GLASS.

//! End to end: Cruncha's ACP client drives `cruncha-agent` with a real `cruncha-mcp` server.
//! The demo reads the document without a model; provider mode (against a mock OpenAI-style
//! API) loads a skill and calls Cruncha's tools, and the results reach the model.

mod common;

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use cruncha_acp::{AgentSpec, Command, Event, StopReason};
use futures::StreamExt;
use serde_json::{Value, json};

/// Serve one canned SSE response per chat request (recording each one), and a fixed model
/// list for `GET /models`. Each chat request as (Authorization header, JSON body).
type Seen = Arc<Mutex<Vec<(String, Value)>>>;

fn mock_server(responses: Vec<Vec<Value>>) -> (String, Seen) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let seen = Arc::new(Mutex::new(Vec::new()));
    let seen2 = seen.clone();
    std::thread::spawn(move || {
        let mut responses = responses.into_iter();
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let (mut len, mut auth) = (0, String::new());
            let mut request_line = String::new();
            reader.read_line(&mut request_line).unwrap();
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let lower = line.to_ascii_lowercase();
                if let Some(v) = lower.strip_prefix("content-length:") {
                    len = v.trim().parse().unwrap();
                }
                if lower.starts_with("authorization:") {
                    auth = line["authorization:".len()..].trim().to_string();
                }
                if line == "\r\n" {
                    break;
                }
            }
            if request_line.starts_with("GET /models") {
                let body = r#"{"object":"list","data":[{"id":"m-a"},{"id":"m-b"}]}"#;
                let out = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                stream.write_all(out.as_bytes()).unwrap();
                continue;
            }
            let Some(chunks) = responses.next() else {
                break;
            };
            let mut body = vec![0; len];
            reader.read_exact(&mut body).unwrap();
            seen2
                .lock()
                .unwrap()
                .push((auth, serde_json::from_slice(&body).unwrap()));
            let mut out = String::from(
                "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\n",
            );
            for c in chunks {
                out.push_str(&format!("data: {c}\n\n"));
            }
            out.push_str("data: [DONE]\n\n");
            stream.write_all(out.as_bytes()).unwrap();
        }
    });
    (url, seen)
}

fn delta(d: Value) -> Value {
    json!({"choices": [{"index": 0, "delta": d}]})
}


struct Turn {
    text: String,
    tools: Vec<(String, Option<String>)>,
    stop: Option<StopReason>,
    mcp_http: bool,
}

fn turn(spec: AgentSpec, prompt: &str) -> Turn {
    let (agent, mut events) = cruncha_acp::connect(spec);
    let mut t = Turn { text: String::new(), tools: vec![], stop: None, mcp_http: false };
    futures::executor::block_on(async {
        while let Some(ev) = events.next().await {
            match ev {
                Event::Connected { mcp_http, .. } => t.mcp_http = mcp_http,
                Event::SessionReady { .. } => {
                    agent.send(Command::Prompt(prompt.into()));
                }
                Event::AgentText(s) => t.text.push_str(&s),
                Event::ToolCall(p) => {
                    if let Some(title) = p.title {
                        t.tools.push((p.id.clone(), Some(title)));
                    }
                }
                Event::Permission(_) => panic!("Cruncha's tools never ask"),
                Event::TurnEnded(r) => {
                    t.stop = Some(r);
                    break;
                }
                Event::Disconnected { reason } => panic!("agent disconnected: {reason:?}"),
                _ => {}
            }
        }
    });
    t
}

#[test]
fn the_demo_reads_the_document_through_cruncha_mcp() {
    let server = common::cruncha_server();
    let t = turn(
        AgentSpec {
            name: "demo".into(),
            command: env!("CARGO_BIN_EXE_cruncha-agent").into(),
            args: vec![],
            env: Default::default(),
            cwd: common::workdir("demo"),
            resume: None,
            mcp_servers: vec![server],
        },
        "hello",
    );
    assert!(t.mcp_http, "advertises HTTP MCP");
    assert_eq!(t.stop, Some(StopReason::EndTurn));
    assert!(t.text.contains("**3 cells**"), "{}", t.text);
    assert_eq!(t.tools.first().and_then(|x| x.1.clone()).as_deref(), Some("read doc"));
}

#[test]
fn provider_mode_loads_skills_and_calls_cruncha_tools() {
    let (url, seen) = mock_server(vec![
        // 1: load the skill and explain M, in one step (arguments split across chunks).
        vec![
            delta(json!({"tool_calls": [{"index": 0, "id": "call_1", "type": "function",
                "function": {"name": "load_skill", "arguments": "{\"name\":"}}]})),
            delta(json!({"tool_calls": [{"index": 0, "function": {"arguments": "\"cruncha-core\"}"}}]})),
            delta(json!({"tool_calls": [{"index": 1, "id": "call_2", "type": "function",
                "function": {"name": "explain", "arguments": "{\"cell_id\":\"M\"}"}}]})),
            json!({"choices": [{"index": 0, "delta": {}, "finish_reason": "tool_calls"}]}),
        ],
        // 2: an unknown tool is an error the model sees, not a crash.
        vec![delta(json!({"tool_calls": [{"index": 0, "id": "call_3", "type": "function",
            "function": {"name": "run_command", "arguments": "{\"command\":\"ls\"}"}}]}))],
        // 3: the answer.
        vec![delta(json!({"content": "M = 108 kip·ft (cell M)."}))],
    ]);
    let server = common::cruncha_server();
    let t = turn(
        AgentSpec {
            name: "provider".into(),
            command: env!("CARGO_BIN_EXE_cruncha-agent").into(),
            args: ["--provider", "openai-compatible", "--base-url", &url, "--model", "m-a", "--api-key-env", "CRUNCHA_TEST_KEY"]
                .map(String::from)
                .to_vec(),
            env: [("CRUNCHA_TEST_KEY".to_string(), "test-key".to_string())].into(),
            cwd: common::workdir("provider"),
            resume: None,
            mcp_servers: vec![server],
        },
        "what is M?",
    );
    assert_eq!(t.stop, Some(StopReason::EndTurn));
    assert_eq!(t.text, "M = 108 kip·ft (cell M).");
    let titles: Vec<String> = t.tools.iter().filter_map(|x| x.1.clone()).collect();
    assert_eq!(titles, ["Load skill cruncha-core", "Explain M", "run command"]);

    let seen = seen.lock().unwrap();
    assert_eq!(seen.len(), 3);
    let (auth, first) = &seen[0];
    assert_eq!(auth, "Bearer test-key");
    let tools: Vec<&str> = first["tools"].as_array().unwrap().iter().map(|t| t["function"]["name"].as_str().unwrap()).collect();
    for want in ["read_doc", "explain", "check", "propose_edit", "load_skill"] {
        assert!(tools.contains(&want), "{tools:?}");
    }
    for gone in ["read_file", "write_file", "run_command", "list_directory"] {
        assert!(!tools.contains(&gone), "no coding tools: {tools:?}");
    }
    let system = first["messages"][0]["content"].as_str().unwrap();
    assert!(system.contains("Start with read_doc."), "AGENTS.md");
    assert!(system.contains("- cruncha-core: Ground rules for a Cruncha document."), "skills index");
    assert!(!system.contains("The engine computes every number."), "skill bodies load on demand");

    let results: Vec<&Value> = seen[1].1["messages"].as_array().unwrap().iter().filter(|m| m["role"] == "tool").collect();
    assert_eq!(results.len(), 2);
    assert!(results[0]["content"].as_str().unwrap().contains("The engine computes every number."));
    let explain = results[1]["content"].as_str().unwrap();
    assert!(explain.contains("108 kip·ft") && explain.contains("Reads:"), "{explain}");
    let last: Vec<&Value> = seen[2].1["messages"].as_array().unwrap().iter().filter(|m| m["role"] == "tool").collect();
    assert!(last[2]["content"].as_str().unwrap().starts_with("error: there is no tool named run_command"));
}

#[test]
fn the_stdio_bridge_reaches_cruncha_mcp() {
    use std::process::{Command as Process, Stdio};
    let server = common::cruncha_server();
    let mut child = Process::new(env!("CARGO_BIN_EXE_cruncha-agent"))
        .args(["--mcp-stdio", &server.url])
        .env("CRUNCHA_MCP_AUTH", &server.headers[0].1)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut out = BufReader::new(child.stdout.take().unwrap());
    let mut ask = |m: Value, answer: bool| -> Option<Value> {
        writeln!(stdin, "{m}").unwrap();
        stdin.flush().unwrap();
        answer.then(|| {
            let mut line = String::new();
            out.read_line(&mut line).unwrap();
            serde_json::from_str(&line).unwrap()
        })
    };
    let init = ask(json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
        "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "t", "version": "0"}}}), true).unwrap();
    assert!(init["result"]["serverInfo"].is_object(), "{init}");
    ask(json!({"jsonrpc": "2.0", "method": "notifications/initialized"}), false);
    let list = ask(json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}), true).unwrap();
    assert!(list["result"]["tools"].as_array().unwrap().iter().any(|t| t["name"] == "read_doc"));
    let doc = ask(json!({"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "read_doc", "arguments": {}}}), true).unwrap();
    assert!(doc["result"]["content"][0]["text"].as_str().unwrap().contains("108 kip·ft"));
    drop(stdin);
    assert!(child.wait().unwrap().success());

    // Without the token, the server refuses, and the bridge says so as a JSON-RPC error.
    let mut child = Process::new(env!("CARGO_BIN_EXE_cruncha-agent"))
        .args(["--mcp-stdio", &server.url])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    writeln!(stdin, "{}", json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}})).unwrap();
    drop(stdin);
    let mut line = String::new();
    BufReader::new(child.stdout.take().unwrap()).read_line(&mut line).unwrap();
    let v: Value = serde_json::from_str(&line).unwrap();
    assert!(v["error"]["message"].as_str().unwrap().contains("HTTP 401"), "{v}");
    child.wait().unwrap();
}
