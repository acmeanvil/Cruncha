// SPDX-License-Identifier: GPL-3.0-or-later
// Ported from looking_glass (crates/lg-agent/src/demo.rs), Copyright (c) 2026 acmeanvil, MIT
// License; see LICENSE-LOOKING-GLASS. Changes for Cruncha: the scripted turn reads the document
// through the session's tools instead of exercising files and terminals.

//! Demo mode: a scripted turn without a model or API key. It streams a thought, calls the
//! document's `read_doc` tool (a real tool card), and says how to connect a model.

use std::time::Duration;

use agent_client_protocol::Client;
use agent_client_protocol::v1::ConnectionTo;
use agent_client_protocol::v1::schema::{
    ContentBlock, ContentChunk, PromptRequest, SessionId, SessionNotification, SessionUpdate, StopReason,
};

use crate::{State, tools};

async fn sleep(ms: u64) {
    async_io::Timer::after(Duration::from_millis(ms)).await;
}

/// Stream text in word-sized chunks, like a model would. False when cancelled.
async fn stream(
    cx: &ConnectionTo<Client>,
    state: &State,
    session: &SessionId,
    text: &str,
    wrap: fn(ContentChunk) -> SessionUpdate,
) -> agent_client_protocol::Result<bool> {
    for piece in text.split_inclusive([' ', '\n']) {
        if state.cancelled(session) {
            return Ok(false);
        }
        cx.send_notification(SessionNotification::new(session.clone(), wrap(ContentChunk::new(ContentBlock::from(piece)))))?;
        sleep(8).await;
    }
    Ok(true)
}

pub async fn run_turn(cx: &ConnectionTo<Client>, state: &State, req: PromptRequest) -> agent_client_protocol::Result<StopReason> {
    let session = req.session_id.clone();
    let has_read_doc = state
        .sessions
        .lock()
        .unwrap()
        .get(&session.to_string())
        .is_some_and(|s| s.tools.iter().any(|t| t.tool.name == "read_doc"));
    if !stream(cx, state, &session, "Demo mode: I'll read the document through Cruncha's tools.", SessionUpdate::AgentThoughtChunk).await? {
        return Ok(StopReason::Cancelled);
    }
    let reply = if has_read_doc {
        let ctx = tools::Ctx { cx, state, session: &session, };
        let id = state.next_id("read");
        let text = tools::run(&ctx, "read_doc", "{}", &id).await?;
        if let Some(e) = text.strip_prefix("error: ") {
            format!("I couldn't read the document: {e}\n")
        } else {
            // The mirror's first line: `<!-- cruncha document “…” · version 3 · 12 cells · id … -->`.
            let head = text.lines().next().unwrap_or_default();
            let cells = head.split(" · ").find_map(|p| p.strip_suffix(" cells")).unwrap_or("?");
            format!(
                "I read the document: **{cells} cells**, {} lines in its text mirror.\n\n\
                 This is the built-in agent in *demo mode*. Add a model connection in \
                 **Settings → Agents** to have it answer questions and propose edits.\n",
                text.lines().count()
            )
        }
    } else {
        "This is the built-in agent in *demo mode*, and this session has no document tools. \
         Add a model connection in **Settings → Agents**.\n"
            .to_string()
    };
    if !stream(cx, state, &session, &reply, SessionUpdate::AgentMessageChunk).await? {
        return Ok(StopReason::Cancelled);
    }
    Ok(StopReason::EndTurn)
}
