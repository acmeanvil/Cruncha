// SPDX-License-Identifier: GPL-3.0-or-later

//! The client against a real ACP agent: looking_glass's `lg-agent` in demo mode (no key). Set
//! `CRUNCHA_TEST_ACP_AGENT` to its path; by default the looking_glass checkout next to this
//! one is tried, and the test is skipped when there is none.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use cruncha_acp::{AgentSpec, Command, Event, McpServerSpec, StopReason};
use futures::StreamExt;

fn agent() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("CRUNCHA_TEST_ACP_AGENT") {
        return Some(p.into());
    }
    let home = std::env::var_os("HOME")?;
    ["release", "debug"]
        .iter()
        .map(|p| PathBuf::from(&home).join(format!("Developer/looking_glass/target/{p}/lg-agent")))
        .find(|p| p.is_file())
}

#[test]
fn a_turn_with_the_demo_agent() {
    let Some(program) = agent() else {
        eprintln!("skipped: no lg-agent (set CRUNCHA_TEST_ACP_AGENT)");
        return;
    };
    let dir = std::env::temp_dir().join(format!("cruncha-acp-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (handle, mut events) = cruncha_acp::connect(AgentSpec {
        name: "demo".into(),
        command: program,
        args: vec![],
        env: Default::default(),
        cwd: dir,
        resume: None,
        mcp_servers: vec![McpServerSpec {
            name: "cruncha".into(),
            url: "http://127.0.0.1:9/mcp".into(),
            headers: vec![("Authorization".into(), "Bearer x".into())],
        }],
    });
    let start = Instant::now();
    let (mut connected, mut ready, mut text, mut ended) = (false, false, String::new(), None);
    futures::executor::block_on(async {
        while let Some(ev) = events.next().await {
            match ev {
                Event::Connected { .. } => connected = true,
                Event::SessionReady { .. } => {
                    ready = true;
                    handle.send(Command::Prompt("hello".into()));
                }
                Event::AgentText(t) => text.push_str(&t),
                Event::TurnEnded(r) => {
                    ended = Some(r);
                    break;
                }
                Event::Disconnected { reason } => panic!("disconnected: {reason:?}"),
                _ => {}
            }
            assert!(start.elapsed() < Duration::from_secs(30), "timed out");
        }
    });
    assert!(connected && ready, "handshake");
    assert!(!text.is_empty(), "the agent replied");
    assert_eq!(ended, Some(StopReason::EndTurn));
}
