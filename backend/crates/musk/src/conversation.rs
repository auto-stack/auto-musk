//! Unified conversation model — the common abstraction for chat + flow.
//! See designs/007-unified-workflow-architecture.md.

use serde::{Deserialize, Serialize};

use crate::chats::{ChatMessage, Role};

/// A conversation or workflow — the common ancestor of chat and flow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_turn_id: Option<String>,
    pub kind: ConversationKind,
    pub workspace_id: String,
    pub driver: Driver,
    pub status: ConversationStatus,
    #[serde(default)]
    pub turns: Vec<Turn>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_prompt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flow_id: Option<String>,
    #[serde(default)]
    pub current_step: usize,
    #[serde(default)]
    pub cumulative_tokens: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub budget_limit: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pending_gate: Option<GateInfo>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversationKind {
    Chat,
    Flow,
    Errand,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Driver {
    Human,
    Agent { agent_id: String },
    Flow { flow_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversationStatus {
    Active,
    WaitingGate,
    Completed,
    Failed { error: String },
    Paused { reason: String },
}

impl ConversationStatus {
    pub fn to_status_str(&self) -> String {
        match self {
            Self::Active => "active",
            Self::WaitingGate => "waiting_gate",
            Self::Completed => "completed",
            Self::Failed { .. } => "failed",
            Self::Paused { .. } => "paused",
        }
        .into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Turn {
    pub id: String,
    pub seq: usize,
    pub from: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    pub kind: TurnKind,
    #[serde(default)]
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool: Option<ToolRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gate: Option<GateRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub child_conversation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens: Option<u64>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TurnKind {
    Message,
    ToolCall,
    ToolResult,
    Gate,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRecord {
    pub name: String,
    pub args: serde_json::Value,
    #[serde(default)]
    pub result: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateRecord {
    pub step_id: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feedback: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateInfo {
    pub step_id: String,
    pub profession_id: String,
    pub since: u64,
}

/// Lightweight summary for list views.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSummary {
    pub id: String,
    pub kind: ConversationKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    pub workspace_id: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub turn_count: usize,
    pub cumulative_tokens: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Convert a ChatMessage to one or more Turns (tool_calls expand to extra turns).
pub fn chat_message_to_turns(msg: &ChatMessage, seq_base: usize) -> Vec<Turn> {
    let from = match msg.role {
        Role::User => "human",
        Role::Assistant => "assistant",
        Role::Tool => "system",
    };
    let mut turns = Vec::new();

    // Main message turn
    if !msg.content.is_empty() || msg.tool_calls.is_empty() {
        turns.push(Turn {
            id: msg.id.clone(),
            seq: seq_base,
            from: from.to_string(),
            to: if matches!(msg.role, Role::User) { Some("assistant".into()) } else { None },
            kind: TurnKind::Message,
            content: msg.content.clone(),
            tool: None,
            gate: None,
            child_conversation: None,
            tokens: None,
            timestamp: msg.created_at,
        });
    }

    // Each tool call becomes a ToolCall + ToolResult pair
    for (i, tc) in msg.tool_calls.iter().enumerate() {
        turns.push(Turn {
            id: format!("{}-tc{}", msg.id, i),
            seq: seq_base + turns.len(),
            from: from.to_string(),
            to: None,
            kind: TurnKind::ToolCall,
            content: String::new(),
            tool: Some(ToolRecord {
                name: tc.tool.clone(),
                args: tc.args.clone(),
                result: String::new(),
                tool_id: None,
            }),
            gate: None,
            child_conversation: None,
            tokens: None,
            timestamp: msg.created_at,
        });
        turns.push(Turn {
            id: format!("{}-tr{}", msg.id, i),
            seq: seq_base + turns.len(),
            from: "system".to_string(),
            to: None,
            kind: TurnKind::ToolResult,
            content: String::new(),
            tool: Some(ToolRecord {
                name: tc.tool.clone(),
                args: serde_json::Value::Null,
                result: tc.result.clone(),
                tool_id: None,
            }),
            gate: None,
            child_conversation: None,
            tokens: None,
            timestamp: msg.created_at,
        });
    }

    turns
}

#[allow(dead_code)]
fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[allow(dead_code)]
fn new_id(len: usize) -> String {
    use rand::Rng;
    let chars: &[u8] = b"0123456789abcdef";
    let mut rng = rand::thread_rng();
    (0..len).map(|_| chars[rng.gen_range(0..chars.len())] as char).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chats::{ChatMessage, Role, ToolCall};

    #[test]
    fn user_message_to_turn() {
        let msg = ChatMessage::user("hello");
        let turns = chat_message_to_turns(&msg, 0);
        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0].from, "human");
        assert_eq!(turns[0].to.as_deref(), Some("assistant"));
        assert_eq!(turns[0].kind, TurnKind::Message);
        assert_eq!(turns[0].content, "hello");
    }

    #[test]
    fn assistant_with_tool_calls_expands() {
        let mut msg = ChatMessage::assistant("let me check");
        msg.tool_calls.push(ToolCall {
            tool: "read_file".into(),
            args: serde_json::json!({"path": "test.rs"}),
            result: "file contents".into(),
        });
        let turns = chat_message_to_turns(&msg, 0);
        assert_eq!(turns.len(), 3); // message + tool_call + tool_result
        assert_eq!(turns[1].kind, TurnKind::ToolCall);
        assert_eq!(turns[1].tool.as_ref().unwrap().name, "read_file");
        assert_eq!(turns[2].kind, TurnKind::ToolResult);
        assert_eq!(turns[2].tool.as_ref().unwrap().result, "file contents");
    }

    #[test]
    fn status_str_roundtrip() {
        assert_eq!(ConversationStatus::Active.to_status_str(), "active");
        assert_eq!(ConversationStatus::WaitingGate.to_status_str(), "waiting_gate");
        assert_eq!(ConversationStatus::Completed.to_status_str(), "completed");
    }

    // Silence unused-import warning for Role in tests when not referenced directly.
    #[test]
    fn _role_import_used() {
        let _ = Role::User;
    }
}
