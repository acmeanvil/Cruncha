// SPDX-License-Identifier: GPL-3.0-or-later
// Ported from looking_glass (crates/lg-acp/src/model.rs), Copyright (c) 2026 acmeanvil, MIT License;
// see LICENSE-LOOKING-GLASS. Changes: no client terminals.

//! Protocol-neutral types the UI consumes. They serialize to JSON for the web UI.

use agent_client_protocol::v1::schema as acp;
use futures::channel::oneshot;
use serde::{Deserialize, Serialize};

/// Everything the UI needs to know about what the agent is doing.
#[derive(Debug)]
pub enum Event {
    Connected {
        agent_name: String,
        agent_version: Option<String>,
        /// The agent takes images in prompts (ACP `promptCapabilities.image`).
        images: bool,
        /// The agent connects to MCP servers over HTTP (`mcpCapabilities.http`).
        mcp_http: bool,
    },
    /// The session exists. `resumed` is true when an earlier session was
    /// loaded (its history is replayed as events just before this).
    SessionReady {
        session_id: String,
        resumed: bool,
    },
    /// A chunk of the user's message, e.g. when an agent replays history.
    UserText(String),
    /// A chunk of the agent's reply. Chunks are appended to the current reply.
    AgentText(String),
    /// A chunk of the agent's reasoning.
    Thought(String),
    /// Create or update a tool call. `None` fields are unchanged.
    ToolCall(ToolCallPatch),
    Plan(Vec<PlanItem>),
    /// Context use after the latest request (`size` 0 when unknown), plus
    /// session totals when the agent reports them.
    Usage {
        used: u64,
        size: u64,
        totals: Option<UsageTotals>,
    },
    Permission(PermissionPrompt),
    TurnEnded(StopReason),
    Error(String),
    /// The agent's settings for this session (model, mode, …), replacing
    /// any earlier set.
    ConfigOptions(Vec<ConfigOption>),
    /// A line the agent wrote to stderr.
    Log(String),
    Disconnected {
        reason: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub struct ToolCallPatch {
    pub id: String,
    pub title: Option<String>,
    pub kind: Option<ToolKind>,
    pub status: Option<ToolStatus>,
    pub output: Option<Vec<ToolOutput>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolKind {
    Read,
    Edit,
    Delete,
    Move,
    Search,
    Execute,
    Think,
    Fetch,
    Other,
}

impl From<acp::ToolKind> for ToolKind {
    fn from(k: acp::ToolKind) -> Self {
        match k {
            acp::ToolKind::Read => Self::Read,
            acp::ToolKind::Edit => Self::Edit,
            acp::ToolKind::Delete => Self::Delete,
            acp::ToolKind::Move => Self::Move,
            acp::ToolKind::Search => Self::Search,
            acp::ToolKind::Execute => Self::Execute,
            acp::ToolKind::Think => Self::Think,
            acp::ToolKind::Fetch => Self::Fetch,
            _ => Self::Other,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolStatus {
    Pending,
    Running,
    Done,
    Failed,
}

impl From<acp::ToolCallStatus> for ToolStatus {
    fn from(s: acp::ToolCallStatus) -> Self {
        match s {
            acp::ToolCallStatus::Pending => Self::Pending,
            acp::ToolCallStatus::InProgress => Self::Running,
            acp::ToolCallStatus::Completed => Self::Done,
            acp::ToolCallStatus::Failed => Self::Failed,
            #[allow(unreachable_patterns)]
            _ => Self::Pending,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolOutput {
    Text(String),
    Diff {
        path: String,
        old_text: Option<String>,
        new_text: String,
    },
    Terminal(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanItem {
    pub content: String,
    pub status: PlanStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanStatus {
    Pending,
    Active,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopReason {
    EndTurn,
    MaxTokens,
    MaxTurns,
    Refusal,
    Cancelled,
}

impl From<acp::StopReason> for StopReason {
    fn from(r: acp::StopReason) -> Self {
        match r {
            acp::StopReason::EndTurn => Self::EndTurn,
            acp::StopReason::MaxTokens => Self::MaxTokens,
            acp::StopReason::MaxTurnRequests => Self::MaxTurns,
            acp::StopReason::Refusal => Self::Refusal,
            acp::StopReason::Cancelled => Self::Cancelled,
            #[allow(unreachable_patterns)]
            _ => Self::EndTurn,
        }
    }
}

/// The agent is asking the user to approve something.
#[derive(Debug)]
pub struct PermissionPrompt {
    pub id: u64,
    pub tool_call_id: String,
    pub title: String,
    pub options: Vec<PermissionChoice>,
    pub reply: PermissionReply,
}

#[derive(Debug, Clone)]
pub struct PermissionChoice {
    pub id: String,
    pub label: String,
    pub kind: PermissionKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionKind {
    AllowOnce,
    AllowAlways,
    RejectOnce,
    RejectAlways,
}

impl PermissionKind {
    pub fn is_allow(self) -> bool {
        matches!(self, Self::AllowOnce | Self::AllowAlways)
    }
}

impl From<acp::PermissionOptionKind> for PermissionKind {
    fn from(k: acp::PermissionOptionKind) -> Self {
        match k {
            acp::PermissionOptionKind::AllowOnce => Self::AllowOnce,
            acp::PermissionOptionKind::AllowAlways => Self::AllowAlways,
            acp::PermissionOptionKind::RejectOnce => Self::RejectOnce,
            acp::PermissionOptionKind::RejectAlways => Self::RejectAlways,
            #[allow(unreachable_patterns)]
            _ => Self::RejectOnce,
        }
    }
}

/// One-shot reply channel for a permission prompt. Dropping it without
/// answering tells the agent the request was cancelled.
#[derive(Debug)]
pub struct PermissionReply(pub(crate) Option<oneshot::Sender<Option<String>>>);

impl PermissionReply {
    /// Answer with the chosen option id, or `None` to cancel.
    pub fn answer(&mut self, option_id: Option<String>) {
        if let Some(tx) = self.0.take() {
            let _ = tx.send(option_id);
        }
    }

    pub fn is_answered(&self) -> bool {
        self.0.is_none()
    }
}

/// A setting the agent offers for the session, such as which model to use.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigOption {
    pub id: String,
    pub name: String,
    pub category: ConfigCategory,
    pub current: String,
    pub choices: Vec<ConfigChoice>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigChoice {
    pub value: String,
    pub name: String,
    pub description: Option<String>,
    /// Family the agent grouped this under (e.g. "Opus"), if any.
    #[serde(default)]
    pub group: Option<String>,
}

/// Token totals for a session, from the agent.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UsageTotals {
    pub model: Option<String>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
    pub requests: u64,
    /// Cost so far as (amount, currency), when the agent reports it.
    pub cost: Option<(f64, String)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigCategory {
    Model,
    Mode,
    Other,
}

impl ConfigOption {
    pub fn current_name(&self) -> &str {
        self.choices
            .iter()
            .find(|c| c.value == self.current)
            .map_or(self.current.as_str(), |c| c.name.as_str())
    }
}

/// Convert the agent's options; only select-style options are kept.
pub(crate) fn config_options(options: &[acp::SessionConfigOption]) -> Vec<ConfigOption> {
    options
        .iter()
        .filter_map(|o| {
            let acp::SessionConfigKind::Select(select) = &o.kind else {
                return None;
            };
            let choice = |c: &acp::SessionConfigSelectOption, group: Option<&str>| ConfigChoice {
                value: c.value.to_string(),
                name: c.name.clone(),
                description: c.description.clone(),
                group: group.map(str::to_string),
            };
            let choices = match &select.options {
                acp::SessionConfigSelectOptions::Ungrouped(list) => {
                    list.iter().map(|c| choice(c, None)).collect()
                }
                acp::SessionConfigSelectOptions::Grouped(groups) => groups
                    .iter()
                    .flat_map(|g| g.options.iter().map(move |c| (g.name.clone(), c)))
                    .map(|(g, c)| choice(c, Some(&g)))
                    .collect(),
                #[allow(unreachable_patterns)]
                _ => Vec::new(),
            };
            Some(ConfigOption {
                id: o.id.to_string(),
                name: o.name.clone(),
                category: match o.category {
                    Some(acp::SessionConfigOptionCategory::Model) => ConfigCategory::Model,
                    Some(acp::SessionConfigOptionCategory::Mode) => ConfigCategory::Mode,
                    _ => ConfigCategory::Other,
                },
                current: select.current_value.to_string(),
                choices,
            })
        })
        .collect()
}
