// SPDX-License-Identifier: GPL-3.0-or-later
//! A real `cruncha-mcp` server over a small stand-in document, and a working directory with
//! skills, for the agent's end-to-end tests.

#![allow(dead_code)]

use std::collections::HashMap;
use std::path::PathBuf;

use cruncha_acp::McpServerSpec;
use cruncha_mcp::{Backend, CellChange, CellView, DocSnapshot, Edit, ProposalInfo, ResultView, VersionInfo};

#[derive(Clone)]
struct Doc;

impl Backend for Doc {
    async fn snapshot(&self, _wait: bool) -> Result<DocSnapshot, String> {
        let cell = |id: &str, source: &str, defines: &[&str], reads: &[&str]| CellView {
            id: id.into(),
            kind: "calc".into(),
            source: source.into(),
            defines: defines.iter().map(|s| s.to_string()).collect(),
            reads: reads.iter().map(|s| s.to_string()).collect(),
            provenance: "human".into(),
            ..Default::default()
        };
        let r = |text: &str, name: &str, value: &str| ResultView {
            text: text.into(),
            values: vec![(name.into(), value.into())],
            ..Default::default()
        };
        let results = HashMap::from([
            ("L".to_string(), r("L = 24 ft", "L", "24 ft")),
            ("W".to_string(), r("w = 1.5 kip/ft", "w", "1.5 kip/ft")),
            ("M".to_string(), r("M = w L^2 / 8 = 108 kip·ft", "M", "108 kip·ft")),
        ]);
        Ok(DocSnapshot {
            name: "beam.cruncha".into(),
            id: "DOC".into(),
            version: 4,
            cells: vec![
                cell("L", "L = 24 ft", &["L"], &[]),
                cell("W", "w = 1.5 kip/ft", &["w"], &[]),
                cell("M", "M = w L^2 / 8 = ?? kip·ft", &["M"], &["w", "L"]),
            ],
            results,
        })
    }
    async fn history(&self) -> Result<Vec<VersionInfo>, String> {
        Ok(vec![])
    }
    async fn diff(&self, _from: u64, _to: u64) -> Result<Vec<CellChange>, String> {
        Ok(vec![])
    }
    async fn propose(&self, _base: Option<u64>, _message: String, _edits: Vec<Edit>) -> Result<ProposalInfo, String> {
        Err("not in this test".into())
    }
    async fn proposals(&self) -> Result<Vec<ProposalInfo>, String> {
        Ok(vec![])
    }
    async fn withdraw(&self, id: String) -> Result<ProposalInfo, String> {
        Err(format!("no open proposal {id} of yours"))
    }
}

/// Serve the stand-in document on 127.0.0.1 (for the rest of the test process).
pub fn cruncha_server() -> McpServerSpec {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let server = cruncha_mcp::http::serve(Doc).await.unwrap();
            tx.send((server.url.clone(), server.token.clone())).unwrap();
            let _keep = server;
            std::future::pending::<()>().await;
        });
    });
    let (url, token) = rx.recv().unwrap();
    McpServerSpec {
        name: "cruncha".into(),
        url,
        headers: vec![("Authorization".into(), format!("Bearer {token}"))],
        bridge: Some(env!("CARGO_BIN_EXE_cruncha-agent").into()),
    }
}

/// A working directory as the app writes it: AGENTS.md and one skill.
pub fn workdir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("cruncha-agent-{tag}-{}", std::process::id()));
    let skill = dir.join(".claude/skills/cruncha-core");
    std::fs::create_dir_all(&skill).unwrap();
    std::fs::write(skill.join("SKILL.md"), "---\nname: cruncha-core\ndescription: Ground rules for a Cruncha document.\n---\n\n# Rules\nThe engine computes every number.").unwrap();
    std::fs::write(dir.join("AGENTS.md"), "Start with read_doc.").unwrap();
    std::fs::canonicalize(dir).unwrap()
}
