// SPDX-License-Identifier: GPL-3.0-or-later

//! The MCP server end to end: a real MCP client over HTTP against a fake document.

use std::collections::HashMap;

use cruncha_mcp::{Backend, CellChange, CellView, DocSnapshot, Edit, Effect, ProposalInfo, ResultView, TemplateFile, TemplateInfo, TemplateInsert, VersionInfo};
use rmcp::ServiceExt;
use rmcp::model::CallToolRequestParams;
use rmcp::transport::StreamableHttpClientTransport;
use rmcp::transport::streamable_http_client::StreamableHttpClientTransportConfig;

#[derive(Clone)]
struct Fake;

fn cell(id: &str, kind: &str, source: &str, defines: &[&str], reads: &[&str]) -> CellView {
    CellView {
        id: id.into(),
        kind: kind.into(),
        source: source.into(),
        defines: defines.iter().map(|s| s.to_string()).collect(),
        reads: reads.iter().map(|s| s.to_string()).collect(),
        provenance: "human".into(),
        ..Default::default()
    }
}

impl Backend for Fake {
    async fn snapshot(&self, _wait: bool) -> Result<DocSnapshot, String> {
        let mut results = HashMap::new();
        let r = |text: &str, values: &[(&str, &str)]| ResultView {
            text: text.into(),
            values: values.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect(),
            ..Default::default()
        };
        results.insert("L".into(), r("L = 24 ft", &[("L", "24 ft")]));
        results.insert("W".into(), r("w = 1.5 kip/ft", &[("w", "1.5 kip/ft")]));
        results.insert("M".into(), r("M = w L^2 / 8 = 108 kip·ft", &[("M", "108 kip·ft")]));
        results.insert(
            "T".into(),
            ResultView { text: "M < 100 kip·ft, n.g.".into(), check: Some(false), ..Default::default() },
        );
        Ok(DocSnapshot {
            name: "beam.cruncha".into(),
            id: "DOC".into(),
            version: 4,
            cells: vec![
                cell("L", "calc", "L = 24 ft", &["L"], &[]),
                cell("W", "calc", "w = 1.5 kip/ft", &["w"], &[]),
                cell("M", "calc", "M = w L^2 / 8 = ?? kip·ft", &["M"], &["w", "L"]),
                cell("T", "calc", "@test M < 100 'kip·ft'", &[], &["M"]),
                cell("P", "calc", "Fy_ = 50 ksi", &["Fy_"], &["nope"]),
            ],
            results,
        })
    }

    async fn history(&self) -> Result<Vec<VersionInfo>, String> {
        Ok((0..=4)
            .map(|v| VersionInfo {
                version: v,
                at: format!("2026-11-20T10:0{v}:00Z"),
                actor: "user:t".into(),
                message: format!("edit {v}"),
                kind: "edit".into(),
                snapshots: if v == 2 { vec!["Rev A".into()] } else { vec![] },
            })
            .collect())
    }

    async fn diff(&self, from: u64, to: u64) -> Result<Vec<CellChange>, String> {
        Ok(if from < 3 && to >= 3 {
            vec![CellChange {
                id: "L".into(),
                kind: "calc".into(),
                change: "modified".into(),
                before: Some("L = 20 ft".into()),
                after: Some("L = 24 ft".into()),
            }]
        } else {
            vec![]
        })
    }

    async fn propose(&self, base_version: Option<u64>, message: String, edits: Vec<Edit>) -> Result<ProposalInfo, String> {
        if base_version.is_some_and(|v| v < 4) {
            return Err("cell L changed in version 3, after version 2 that you read; call read_doc again".into());
        }
        let Some(Edit::Modify { id, .. }) = edits.first() else { return Err("only modify in the fake".into()) };
        Ok(ProposalInfo {
            id: "P1".into(),
            status: "open".into(),
            proposed_by: "ai:test".into(),
            message,
            base_version: 4,
            changes: vec![format!("modify {id}")],
            effects: vec![Effect {
                id: "M".into(),
                kind: "calc".into(),
                before: Some(ResultView { text: "M = w L^2 / 8 = 108 kip·ft".into(), ..Default::default() }),
                after: Some(ResultView { text: "M = w L^2 / 8 = 75 kip·ft".into(), ..Default::default() }),
            }],
            ..Default::default()
        })
    }

    async fn proposals(&self) -> Result<Vec<ProposalInfo>, String> {
        Ok(vec![])
    }

    async fn withdraw(&self, id: String) -> Result<ProposalInfo, String> {
        Err(format!("no open proposal {id} of yours"))
    }

    async fn templates(&self) -> Result<Vec<TemplateInfo>, String> {
        let t = |id: &str, title: &str, tags: &[&str], inputs: &[&str]| TemplateInfo {
            id: id.into(),
            version: "1.0.0".into(),
            title: title.into(),
            library: "builtin".into(),
            tags: tags.iter().map(|s| s.to_string()).collect(),
            inputs: inputs.iter().map(|s| s.to_string()).collect(),
            ..Default::default()
        };
        Ok(vec![
            t("structural.beam", "Simply supported beam", &["AISC 360"], &["w", "L"]),
            t("structural.bolts", "Bolt shear", &["AISC 360"], &["d_b"]),
            t("fluids.pipe", "Pipe friction", &["Colebrook"], &["D"]),
            TemplateInfo { table: true, ..t("mechanical.tables.nuts", "Hex nuts", &["nut"], &[]) },
        ])
    }

    async fn template(&self, id: String) -> Result<TemplateFile, String> {
        let info = self.templates().await?.into_iter().find(|t| t.id == id).ok_or(format!("no template {id}"))?;
        Ok(TemplateFile { info, text: "+++\nid = \"structural.beam\"\n+++\n".into(), problems: vec![], tests: vec!["PASS midspan".into()] })
    }

    async fn propose_template(&self, insert: TemplateInsert) -> Result<ProposalInfo, String> {
        if insert.id == "mechanical.tables.nuts" {
            assert_eq!(insert.row.as_deref(), Some("M8"));
            assert_eq!(insert.columns, Some(vec!["s_max".to_string()]));
            return Ok(ProposalInfo { id: "P3".into(), status: "open".into(), message: insert.message, ..Default::default() });
        }
        if insert.rename.get("M").map(String::as_str) != Some("M2") {
            return Err("this document already defines M; insert again with rename, e.g. {\"M\": \"M2\"}".into());
        }
        assert_eq!(insert.bindings.get("L").map(String::as_str), Some("L"));
        Ok(ProposalInfo { id: "P2".into(), status: "open".into(), message: insert.message, changes: vec!["insert N1".into()], ..Default::default() })
    }
}

/// reqwest is built without a TLS provider (the server is plain HTTP on 127.0.0.1), but it
/// still wants one installed.
fn tls() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

fn text(r: &rmcp::model::CallToolResult) -> String {
    r.content.iter().filter_map(|c| c.as_text()).map(|t| t.text.clone()).collect()
}

#[tokio::test]
async fn tools_over_http() {
    tls();
    let server = cruncha_mcp::http::serve(Fake).await.unwrap();
    let transport = StreamableHttpClientTransport::from_config(
        StreamableHttpClientTransportConfig::with_uri(server.url.clone()).auth_header(server.token.clone()),
    );
    let client = ().serve(transport).await.expect("MCP handshake");
    let tools = client.list_all_tools().await.unwrap();
    let names: Vec<String> = tools.iter().map(|t| t.name.to_string()).collect();
    for t in ["read_doc", "run", "explain", "check", "history", "propose_edit", "proposals", "withdraw", "templates", "propose_template"] {
        assert!(names.contains(&t.to_string()), "{names:?}");
    }

    let call = |name: &'static str, args: serde_json::Value| {
        let client = &client;
        async move {
            let mut p = CallToolRequestParams::new(name);
            if let serde_json::Value::Object(m) = args {
                p = p.with_arguments(m);
            }
            client.call_tool(p).await.unwrap()
        }
    };
    let doc = text(&call("read_doc", serde_json::json!({})).await);
    assert!(doc.contains("```calc {#M}\nM = w L^2 / 8 = ?? kip·ft\n```\n> ⇒ **M = w L^2 / 8 = 108 kip·ft**"), "{doc}");

    let ex = text(&call("explain", serde_json::json!({"cell_id": "M"})).await);
    assert!(ex.contains("- w: defined in W, = 1.5 kip/ft"), "{ex}");
    assert!(ex.contains("Used by: T"), "{ex}");

    let chk = text(&call("check", serde_json::json!({})).await);
    assert!(chk.contains("0 of 1 checks hold"), "{chk}");
    assert!(chk.contains("FAILING CHECK T"), "{chk}");
    assert!(chk.contains("UNDEFINED P: reads nope"), "{chk}");
    assert!(chk.contains("PLACEHOLDER P: Fy_"), "{chk}");

    let h = text(&call("history", serde_json::json!({"since": 2})).await);
    assert!(h.starts_with("v4 "), "newest first: {h}");
    assert!(h.contains("[snapshot: Rev A]"), "{h}");
    assert!(h.contains("- modified L (calc)\n  before: L = 20 ft\n  after:  L = 24 ft"), "{h}");
    let h = text(&call("history", serde_json::json!({"cell_id": "L"})).await);
    assert_eq!(h.lines().count(), 1, "only v3 changed L: {h}");

    let run = text(&call("run", serde_json::json!({"cell_ids": ["T", "zzz"]})).await);
    assert!(run.contains("CHECK FAILS") && run.contains("zzz: no such cell"), "{run}");

    let p = text(&call("propose_edit", serde_json::json!({
        "message": "shorter span", "base_version": 4,
        "changes": [{"op": "modify", "id": "L", "source": "L = 20 ft"}]
    })).await);
    assert!(p.contains("Proposal P1 recorded") && p.contains("modify L"), "{p}");
    assert!(p.contains("  now:\n  > ⇒ **M = w L^2 / 8 = 108 kip·ft**\n  if accepted:\n  > ⇒ **M = w L^2 / 8 = 75 kip·ft**"), "{p}");
    let stale = call("propose_edit", serde_json::json!({
        "message": "x", "base_version": 2, "changes": [{"op": "delete", "id": "L"}]
    })).await;
    assert_eq!(stale.is_error, Some(true));
    assert!(text(&call("proposals", serde_json::json!({})).await).contains("No proposals"));

    let list = text(&call("templates", serde_json::json!({"query": "aisc"})).await);
    assert!(list.starts_with("2 templates:\n- structural.bolts (builtin 1.0.0, Bolt shear)"), "{list}");
    let list = text(&call("templates", serde_json::json!({"query": "beam aisc"})).await);
    assert!(list.contains("structural.beam") && list.contains("inputs: w, L") && !list.contains("bolts"), "{list}");
    let one = text(&call("templates", serde_json::json!({"id": "structural.beam"})).await);
    assert!(one.contains("Validation: no problems") && one.contains("- PASS midspan") && one.contains("id = \"structural.beam\""), "{one}");
    let clash = call("propose_template", serde_json::json!({"id": "structural.beam", "message": "beam", "bindings": {"L": "L"}})).await;
    assert_eq!(clash.is_error, Some(true));
    assert!(text(&clash).contains("rename"));
    let p = text(&call("propose_template", serde_json::json!({
        "id": "structural.beam", "message": "beam", "bindings": {"L": "L"}, "rename": {"M": "M2"}
    })).await);
    assert!(p.contains("Proposal P2 recorded"), "{p}");
    let nuts = text(&call("templates", serde_json::json!({"query": "nuts"})).await);
    assert!(nuts.contains("table: insert a row with propose_template's `row`"), "{nuts}");
    let p = text(&call("propose_template", serde_json::json!({
        "id": "mechanical.tables.nuts", "message": "the nut", "row": "M8", "columns": ["s_max"]
    })).await);
    assert!(p.contains("Proposal P3 recorded"), "{p}");

    let bad = call("explain", serde_json::json!({"cell_id": "nope"})).await;
    assert_eq!(bad.is_error, Some(true));
    client.cancel().await.unwrap();
    server.stop();
}

#[tokio::test]
async fn the_server_refuses_strangers() {
    tls();
    let server = cruncha_mcp::http::serve(Fake).await.unwrap();
    let http = reqwest::Client::new();
    let body = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"t","version":"1"}}}"#;
    let post = |auth: Option<String>, origin: Option<&str>| {
        let mut r = http
            .post(&server.url)
            .header("content-type", "application/json")
            .header("accept", "application/json, text/event-stream")
            .body(body);
        if let Some(a) = auth {
            r = r.header("authorization", a);
        }
        if let Some(o) = origin {
            r = r.header("origin", o);
        }
        r.send()
    };
    assert_eq!(post(None, None).await.unwrap().status(), 401);
    assert_eq!(post(Some("Bearer wrong".into()), None).await.unwrap().status(), 401);
    assert_eq!(
        post(Some(format!("Bearer {}", server.token)), Some("https://evil.example")).await.unwrap().status(),
        403
    );
    assert!(post(Some(format!("Bearer {}", server.token)), None).await.unwrap().status().is_success());
    assert!(server.claude_command().contains(&server.url));
    server.stop();
}
