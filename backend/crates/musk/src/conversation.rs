//! Unified conversation model — the common abstraction for chat + flow.
//! See designs/007-unified-workflow-architecture.md.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    /// PLAN-042:工具结构化载荷（edit diff / 截断信息），刷新回放仍可渲染。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
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
    pub role_id: String,
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
                details: None,
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
                details: None,
            }),
            gate: None,
            child_conversation: None,
            tokens: None,
            timestamp: msg.created_at,
        });
    }

    turns
}

/// Convert a relay `RunEvent` to zero or more `Turn`s. This is the dual-write
/// bridge that mirrors flow runs into the unified conversation model. The caller
/// passes in the starting `seq_base` (typically the conversation's current turn
/// count) so turns are numbered monotonically.
pub fn run_event_to_turns(event: &crate::relay::store::RunEvent, seq_base: usize) -> Vec<Turn> {
    use crate::relay::store::RunEvent;
    let ts = event.timestamp();
    let mut turns = Vec::new();
    let mut seq = seq_base;

    macro_rules! push_system {
        ($from:expr, $content:expr) => {
            turns.push(Turn {
                id: new_id(8),
                seq,
                from: $from,
                to: None,
                kind: TurnKind::System,
                content: $content,
                tool: None,
                gate: None,
                child_conversation: None,
                tokens: None,
                timestamp: ts,
            });
            seq += 1;
        };
    }

    match event {
        RunEvent::StepStarted {
            step_id,
            role_id,
            ..
        } => {
            push_system!(
                role_id.clone(),
                format!("Step '{}' started ({})", step_id, role_id)
            );
        }
        RunEvent::StepCompleted {
            step_id,
            handoff_summary,
            ..
        } => {
            push_system!(
                "system".into(),
                format!("Step '{}' completed: {}", step_id, handoff_summary)
            );
        }
        RunEvent::TurnDelta {
            role_id,
            text,
            ..
        } => {
            turns.push(Turn {
                id: new_id(8),
                seq,
                from: role_id.clone(),
                to: None,
                kind: TurnKind::Message,
                content: text.clone(),
                tool: None,
                gate: None,
                child_conversation: None,
                tokens: None,
                timestamp: ts,
            });
            seq += 1;
        }
        RunEvent::TurnToolCall {
            role_id,
            tool_name,
            arguments,
            ..
        } => {
            turns.push(Turn {
                id: new_id(8),
                seq,
                from: role_id.clone(),
                to: None,
                kind: TurnKind::ToolCall,
                content: String::new(),
                tool: Some(ToolRecord {
                    name: tool_name.clone(),
                    args: arguments.clone(),
                    result: String::new(),
                    tool_id: None,
                    details: None,
                }),
                gate: None,
                child_conversation: None,
                tokens: None,
                timestamp: ts,
            });
            seq += 1;
        }
        RunEvent::TurnToolResult {
            role_id,
            result,
            details,
            ..
        } => {
            turns.push(Turn {
                id: new_id(8),
                seq,
                from: role_id.clone(),
                to: None,
                kind: TurnKind::ToolResult,
                content: String::new(),
                tool: Some(ToolRecord {
                    name: String::new(),
                    args: serde_json::Value::Null,
                    result: result.clone(),
                    tool_id: None,
                    details: details.clone(),
                }),
                gate: None,
                child_conversation: None,
                tokens: None,
                timestamp: ts,
            });
            seq += 1;
        }
        RunEvent::TurnComplete { .. } => {
            // No standalone turn — content already captured by TurnDelta turns.
        }
        RunEvent::GateWaiting { step_id, .. } => {
            turns.push(Turn {
                id: new_id(8),
                seq,
                from: "system".into(),
                to: None,
                kind: TurnKind::Gate,
                content: format!("Waiting for gate approval: {}", step_id),
                tool: None,
                gate: Some(GateRecord {
                    step_id: step_id.clone(),
                    status: "waiting".into(),
                    feedback: None,
                }),
                child_conversation: None,
                tokens: None,
                timestamp: ts,
            });
            seq += 1;
        }
        RunEvent::GateResolved {
            step_id, decision, ..
        } => {
            turns.push(Turn {
                id: new_id(8),
                seq,
                from: "human".into(),
                to: None,
                kind: TurnKind::Gate,
                content: format!("Gate {} {}", step_id, decision),
                tool: None,
                gate: Some(GateRecord {
                    step_id: step_id.clone(),
                    status: decision.clone(),
                    feedback: None,
                }),
                child_conversation: None,
                tokens: None,
                timestamp: ts,
            });
            seq += 1;
        }
        RunEvent::ReportEmitted { title, format, path, .. } => {
            push_system!("system".into(), format!("汇报报告已生成：{title}（{format}，path：{path}）"));
        }
        RunEvent::RunCompleted { .. } => {
            push_system!("system".into(), "Flow completed".into());
        }
        RunEvent::RunFailed { error, .. } => {
            push_system!("system".into(), format!("Flow failed: {}", error));
        }
        RunEvent::TokenSpend { .. } => {
            // Metadata only — not surfaced as a turn.
        }
        RunEvent::RelayUpdate {
            step_id,
            role_id,
            status,
            ..
        } => {
            push_system!(
                role_id.clone(),
                format!("Step '{}' {}", step_id, status)
            );
        }
        RunEvent::TurnError {
            role_id,
            message,
            ..
        } => {
            push_system!(
                role_id.clone(),
                format!("Error: {}", message)
            );
        }
        RunEvent::TurnBudgetWarning {
            role_id,
            remaining,
            ..
        } => {
            push_system!(
                role_id.clone(),
                format!("Budget warning: {} tokens remaining", remaining)
            );
        }
        RunEvent::TurnBudgetExceeded {
            role_id, ..
        } => {
            push_system!(role_id.clone(), "Budget exceeded".into());
        }
        // PLAN-040: 流式 partial 是易态,只走 SSE 实时进度,不落会话历史。
        RunEvent::ToolUpdate { .. } => {}
    }
    turns
}

#[allow(dead_code)]
pub fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[allow(dead_code)]
pub fn new_id(len: usize) -> String {
    use rand::Rng;
    let chars: &[u8] = b"0123456789abcdef";
    let mut rng = rand::thread_rng();
    (0..len).map(|_| chars[rng.gen_range(0..chars.len())] as char).collect()
}

// ---------------------------------------------------------------------------
// ConversationStore — CRUD + jsonl storage.
// ---------------------------------------------------------------------------

/// File backing for a conversation: `{dir}/{conv-id}/turns.jsonl` and `meta.json`.
const TURNS_FILE: &str = "turns.jsonl";
const META_FILE: &str = "meta.json";
const INDEX_FILE: &str = "index.json";
const ID_LEN: usize = 16;

#[allow(dead_code)]
pub struct ConversationStore {
    dir: PathBuf,
    cache: Mutex<HashMap<String, Conversation>>,
    event_tx: broadcast::Sender<ConversationEvent>,
}

/// A real-time event broadcast by [`ConversationStore`], consumed via SSE.
#[derive(Clone, Debug)]
pub struct ConversationEvent {
    pub conversation_id: String,
    pub turn: Option<Turn>,
    pub status: Option<String>,
}

#[allow(dead_code)]
impl ConversationStore {
    /// Create a store rooted at `dir` (e.g. {root}/.autoos/conversations).
    /// Loads index.json into the cache (summaries only — turns loaded lazily on get).
    pub fn new(dir: PathBuf) -> Self {
        std::fs::create_dir_all(&dir).ok();
        let (event_tx, _) = broadcast::channel(256);
        let store = Self {
            dir,
            cache: Mutex::new(HashMap::new()),
            event_tx,
        };
        store.warm_cache_from_index();
        store
    }

    /// Alias for `new` — handy in tests.
    pub fn at(dir: PathBuf) -> Self {
        Self::new(dir)
    }

    /// Subscribe to real-time conversation events (for SSE).
    pub fn subscribe(&self) -> broadcast::Receiver<ConversationEvent> {
        self.event_tx.subscribe()
    }

    /// Number of active broadcast receivers (Plan 019 §6.2 测试用:验证
    /// conversation_stream 的 SSE stream drop 后订阅确实回收,无累积泄漏)。
    pub fn receiver_count(&self) -> usize {
        self.event_tx.receiver_count()
    }

    /// Create a new conversation. Returns the created Conversation.
    pub fn create(
        &self,
        kind: ConversationKind,
        workspace_id: String,
        driver: Driver,
        mode: Option<String>,
        title: Option<String>,
    ) -> Conversation {
        let conv = self.build_conversation(
            new_id(ID_LEN),
            kind,
            workspace_id,
            driver,
            mode,
            title,
        );
        self.persist_new(&conv);
        conv
    }

    /// Like `create` but with a caller-supplied id. Used for dual-write so a
    /// chat session and its conversation share the same id, keeping them
    /// linked without an extra mapping.
    pub fn create_with_id(
        &self,
        id: String,
        kind: ConversationKind,
        workspace_id: String,
        driver: Driver,
        mode: Option<String>,
        title: Option<String>,
    ) -> Conversation {
        let conv = self.build_conversation(id, kind, workspace_id, driver, mode, title);
        self.persist_new(&conv);
        conv
    }

    /// Shared constructor used by both `create` and `create_with_id`.
    fn build_conversation(
        &self,
        id: String,
        kind: ConversationKind,
        workspace_id: String,
        driver: Driver,
        mode: Option<String>,
        title: Option<String>,
    ) -> Conversation {
        let now = now_secs();
        Conversation {
            id,
            parent_id: None,
            parent_turn_id: None,
            kind,
            workspace_id,
            driver,
            status: ConversationStatus::Active,
            turns: Vec::new(),
            title,
            initial_prompt: None,
            mode,
            flow_id: None,
            current_step: 0,
            cumulative_tokens: 0,
            budget_limit: None,
            pending_gate: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Get a conversation by id (loads turns from jsonl if not cached).
    pub fn get(&self, id: &str) -> Option<Conversation> {
        {
            let cache = self.cache.lock().unwrap();
            if let Some(c) = cache.get(id) {
                return Some(c.clone());
            }
        }
        // Cache miss — try disk.
        let conv = self.load_from_disk(id)?;
        let mut cache = self.cache.lock().unwrap();
        cache.insert(id.to_string(), conv.clone());
        Some(conv)
    }

    /// List all conversations (from index.json), newest first.
    pub fn list(&self) -> Vec<ConversationSummary> {
        let mut summaries = self.load_index();
        summaries.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        summaries
    }

    /// Delete a conversation (removes dir + index entry).
    pub fn delete(&self, id: &str) -> bool {
        let conv_dir = self.conv_dir(id);
        let existed = conv_dir.exists();
        if existed {
            std::fs::remove_dir_all(&conv_dir).ok();
        }
        self.remove_from_index(id);
        self.cache.lock().unwrap().remove(id);
        existed
    }

    /// Delete all conversations. Returns the number removed.
    pub fn delete_all(&self) -> usize {
        let summaries = self.list();
        let count = summaries.len();
        for s in &summaries {
            self.delete(&s.id);
        }
        count
    }

    /// Rename (set title).
    pub fn rename(&self, id: &str, title: &str) -> Option<Conversation> {
        self.mutate(id, |conv| {
            conv.title = Some(title.to_string());
        })
    }

    /// Append a turn to a conversation. Writes to turns.jsonl + updates index.
    pub fn append_turn(&self, id: &str, mut turn: Turn) -> Option<Conversation> {
        let conv_dir = self.conv_dir(id);
        if !conv_dir.exists() {
            return None;
        }
        // Ensure seq is set relative to existing turn count.
        // (Caller may have set seq already; we trust caller if seq > 0, else assign.)
        let _ = turn.seq;
        let turn_clone = turn.clone();

        // Serialize + append one line to turns.jsonl.
        let turns_path = conv_dir.join(TURNS_FILE);
        if let Ok(line) = serde_json::to_string(&turn) {
            use std::io::Write;
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&turns_path)
            {
                let _ = writeln!(f, "{line}");
            }
        }

        let updated = self.mutate(id, |conv| {
            let seq = conv.turns.len();
            turn.seq = seq;
            if let Some(t) = turn.tokens {
                conv.cumulative_tokens = conv.cumulative_tokens.saturating_add(t);
            }
            conv.turns.push(turn);
        });

        // Broadcast the new turn to SSE subscribers (best-effort).
        if updated.is_some() {
            let _ = self.event_tx.send(ConversationEvent {
                conversation_id: id.to_string(),
                turn: Some(turn_clone),
                status: None,
            });
        }
        updated
    }

    /// Update conversation status.
    pub fn set_status(&self, id: &str, status: ConversationStatus) -> Option<Conversation> {
        let status_str = status.to_status_str();
        let updated = self.mutate(id, |conv| {
            conv.status = status;
        });
        if updated.is_some() {
            let _ = self.event_tx.send(ConversationEvent {
                conversation_id: id.to_string(),
                turn: None,
                status: Some(status_str),
            });
        }
        updated
    }

    /// Update pending gate.
    pub fn set_gate(&self, id: &str, gate: Option<GateInfo>) -> Option<Conversation> {
        self.mutate(id, |conv| {
            conv.pending_gate = gate;
        })
    }

    /// Migrate old chat sessions from chats.json into the unified conversation
    /// model. Each ChatSession becomes a kind=Chat Conversation with its
    /// messages converted to Turns via `chat_message_to_turns`. Idempotent:
    /// skipped if any conversations already exist.
    pub fn migrate_chats(&self, chats: &crate::chats::ChatStore) -> usize {
        // If we already have conversations, don't migrate.
        if !self.list().is_empty() {
            return 0;
        }
        let summaries = chats.list();
        let mut count = 0;
        for summary in &summaries {
            let Some(session) = chats.get(&summary.id) else {
                continue;
            };
            let conv = self.create(
                ConversationKind::Chat,
                session.workspace_id.clone().unwrap_or_default(),
                Driver::Human,
                Some(session.mode.clone()),
                Some(session.name.clone()),
            );
            let mut seq = 0;
            for msg in &session.messages {
                let turns = chat_message_to_turns(msg, seq);
                for turn in turns {
                    let _ = self.append_turn(&conv.id, turn);
                    seq += 1;
                }
            }
            count += 1;
        }
        count
    }

    // -----------------------------------------------------------------
    // Internals
    // -----------------------------------------------------------------

    fn conv_dir(&self, id: &str) -> PathBuf {
        self.dir.join(id)
    }

    fn index_path(&self) -> PathBuf {
        self.dir.join(INDEX_FILE)
    }

    /// On construction, populate the cache from index.json (summaries only).
    /// We don't read turns here — `get()` will lazily load them.
    fn warm_cache_from_index(&self) {
        let summaries = self.load_index();
        if summaries.is_empty() {
            return;
        }
        let mut cache = self.cache.lock().unwrap();
        for s in summaries {
            // Build a stub Conversation from the summary; full turns loaded on get().
            // But we should NOT shadow a real on-disk conversation. If meta.json exists,
            // prefer loading it (cheap, no turns file read necessary in many cases).
            if cache.contains_key(&s.id) {
                continue;
            }
            // We only store a placeholder; mark turns empty. get() will refresh from disk
            // if a caller actually wants turns. To keep things correct, we leave the cache
            // empty for warm — index serves `list()`, and `get()` falls back to disk.
            let _ = s;
        }
    }

    /// Write a freshly created conversation to disk + cache + index.
    fn persist_new(&self, conv: &Conversation) {
        let conv_dir = self.conv_dir(&conv.id);
        std::fs::create_dir_all(&conv_dir).ok();
        self.save_meta(conv);
        self.upsert_index(conv);
        self.cache
            .lock()
            .unwrap()
            .insert(conv.id.clone(), conv.clone());
    }

    /// Save the full conversation (incl. turns) to meta.json.
    fn save_meta(&self, conv: &Conversation) {
        let path = self.conv_dir(&conv.id).join(META_FILE);
        if let Ok(json) = serde_json::to_string_pretty(conv) {
            let _ = std::fs::write(&path, json);
        }
    }

    /// Load a conversation from disk (meta.json + turns.jsonl). Returns None if missing.
    fn load_from_disk(&self, id: &str) -> Option<Conversation> {
        let conv_dir = self.conv_dir(id);
        let meta_path = conv_dir.join(META_FILE);
        if !meta_path.exists() {
            return None;
        }
        let meta_bytes = std::fs::read(&meta_path).ok()?;
        let mut conv: Conversation = serde_json::from_slice(&meta_bytes).ok()?;

        // turns.jsonl is authoritative — reload from it so we never lose appended turns.
        let turns_path = conv_dir.join(TURNS_FILE);
        if turns_path.exists() {
            conv.turns = self.read_turns(&turns_path);
        }
        Some(conv)
    }

    /// Read all turns from a jsonl file.
    fn read_turns(&self, path: &Path) -> Vec<Turn> {
        let mut turns = Vec::new();
        if let Ok(text) = std::fs::read_to_string(path) {
            for line in text.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if let Ok(t) = serde_json::from_str::<Turn>(line) {
                    turns.push(t);
                }
            }
        }
        turns
    }

    /// Apply a mutation to a conversation (from cache or disk), then persist.
    fn mutate<F: FnOnce(&mut Conversation)>(&self, id: &str, f: F) -> Option<Conversation> {
        let mut conv = {
            let cache = self.cache.lock().unwrap();
            cache.get(id).cloned()
        };
        let mut conv = match conv {
            Some(c) => c,
            None => self.load_from_disk(id)?,
        };
        f(&mut conv);
        conv.updated_at = now_secs();
        let conv_dir = self.conv_dir(id);
        std::fs::create_dir_all(&conv_dir).ok();
        self.save_meta(&conv);
        self.upsert_index(&conv);
        let mut cache = self.cache.lock().unwrap();
        cache.insert(id.to_string(), conv.clone());
        Some(conv)
    }

    // --- index.json helpers ---

    fn load_index(&self) -> Vec<ConversationSummary> {
        let path = self.index_path();
        if !path.exists() {
            return Vec::new();
        }
        match std::fs::read(&path) {
            Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
            Err(_) => Vec::new(),
        }
    }

    fn save_index(&self, summaries: &[ConversationSummary]) {
        if let Ok(json) = serde_json::to_string_pretty(summaries) {
            let _ = std::fs::write(self.index_path(), json);
        }
    }

    fn upsert_index(&self, conv: &Conversation) {
        let mut summaries = self.load_index();
        let summary = ConversationSummary {
            id: conv.id.clone(),
            kind: conv.kind.clone(),
            parent_id: conv.parent_id.clone(),
            workspace_id: conv.workspace_id.clone(),
            status: conv.status.to_status_str(),
            title: conv.title.clone(),
            turn_count: conv.turns.len(),
            cumulative_tokens: conv.cumulative_tokens,
            created_at: conv.created_at,
            updated_at: conv.updated_at,
        };
        if let Some(existing) = summaries.iter_mut().find(|s| s.id == conv.id) {
            *existing = summary;
        } else {
            summaries.push(summary);
        }
        self.save_index(&summaries);
    }

    fn remove_from_index(&self, id: &str) {
        let mut summaries = self.load_index();
        let before = summaries.len();
        summaries.retain(|s| s.id != id);
        if summaries.len() != before {
            self.save_index(&summaries);
        }
    }
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
            status: "success".into(),
            id: "tc-1".into(),
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

    // --- ConversationStore tests ---

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "musk-conv-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn create_and_get_conversation() {
        let dir = temp_dir();
        let store = ConversationStore::at(dir.clone());
        let conv = store.create(
            ConversationKind::Chat,
            "ws1".into(),
            Driver::Human,
            None,
            Some("Test".into()),
        );
        assert_eq!(conv.kind, ConversationKind::Chat);
        assert_eq!(conv.workspace_id, "ws1");
        let got = store.get(&conv.id).expect("must exist");
        assert_eq!(got.title.as_deref(), Some("Test"));
        assert!(got.turns.is_empty());
    }

    #[test]
    fn append_turn_and_list() {
        let dir = temp_dir();
        let store = ConversationStore::at(dir);
        let conv = store.create(
            ConversationKind::Chat,
            "ws1".into(),
            Driver::Human,
            None,
            None,
        );
        store.append_turn(
            &conv.id,
            Turn {
                id: "t0".into(),
                seq: 0,
                from: "human".into(),
                to: Some("assistant".into()),
                kind: TurnKind::Message,
                content: "hello".into(),
                tool: None,
                gate: None,
                child_conversation: None,
                tokens: None,
                timestamp: now_secs(),
            },
        );
        let got = store.get(&conv.id).unwrap();
        assert_eq!(got.turns.len(), 1);
        assert_eq!(got.turns[0].content, "hello");
        let list = store.list();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].turn_count, 1);
    }

    #[test]
    fn delete_conversation() {
        let dir = temp_dir();
        let store = ConversationStore::at(dir);
        let conv = store.create(
            ConversationKind::Chat,
            "ws1".into(),
            Driver::Human,
            None,
            None,
        );
        assert!(store.delete(&conv.id));
        assert!(store.get(&conv.id).is_none());
    }

    #[test]
    fn broadcast_on_append_turn() {
        let dir = temp_dir();
        let store = ConversationStore::at(dir);
        let conv = store.create(
            ConversationKind::Chat,
            "ws1".into(),
            Driver::Human,
            None,
            None,
        );

        // Subscribe before appending.
        let mut rx = store.subscribe();

        store.append_turn(
            &conv.id,
            Turn {
                id: "t0".into(),
                seq: 0,
                from: "human".into(),
                to: None,
                kind: TurnKind::Message,
                content: "test".into(),
                tool: None,
                gate: None,
                child_conversation: None,
                tokens: None,
                timestamp: now_secs(),
            },
        );

        // broadcast::Receiver::try_recv is sync — works in a regular #[test].
        let ev = rx.try_recv();
        assert!(ev.is_ok(), "should receive broadcast event");
        assert_eq!(ev.unwrap().conversation_id, conv.id);
    }

    #[test]
    fn broadcast_on_set_status() {
        let dir = temp_dir();
        let store = ConversationStore::at(dir);
        let conv = store.create(
            ConversationKind::Chat,
            "ws1".into(),
            Driver::Human,
            None,
            None,
        );
        let mut rx = store.subscribe();
        store.set_status(&conv.id, ConversationStatus::Completed);
        let ev = rx.try_recv().expect("should receive status event");
        assert_eq!(ev.conversation_id, conv.id);
        assert_eq!(ev.status.as_deref(), Some("completed"));
        assert!(ev.turn.is_none());
    }

    #[test]
    fn persists_across_reload() {
        let dir = temp_dir();
        let id = {
            let store = ConversationStore::at(dir.clone());
            let conv = store.create(
                ConversationKind::Flow,
                "ws1".into(),
                Driver::Flow {
                    flow_id: "default".into(),
                },
                None,
                Some("My Flow".into()),
            );
            store.append_turn(
                &conv.id,
                Turn {
                    id: "t0".into(),
                    seq: 0,
                    from: "advisor".into(),
                    to: None,
                    kind: TurnKind::Message,
                    content: "designed".into(),
                    tool: None,
                    gate: None,
                    child_conversation: None,
                    tokens: Some(100),
                    timestamp: now_secs(),
                },
            );
            conv.id
        };
        // Fresh store reloads from disk
        let store2 = ConversationStore::at(dir);
        let got = store2.get(&id).expect("should persist");
        assert_eq!(got.kind, ConversationKind::Flow);
        assert_eq!(got.title.as_deref(), Some("My Flow"));
        assert_eq!(got.turns.len(), 1);
        assert_eq!(got.turns[0].tokens, Some(100));
    }

    #[test]
    fn rename_and_status_updates() {
        let dir = temp_dir();
        let store = ConversationStore::at(dir);
        let conv = store.create(
            ConversationKind::Chat,
            "ws1".into(),
            Driver::Human,
            None,
            None,
        );
        let renamed = store.rename(&conv.id, "Renamed").expect("renamed");
        assert_eq!(renamed.title.as_deref(), Some("Renamed"));
        let updated = store
            .set_status(&conv.id, ConversationStatus::Completed)
            .expect("status set");
        assert_eq!(updated.status, ConversationStatus::Completed);
        // Reload from disk to verify persistence.
        let store2 = ConversationStore::at(store.dir.clone());
        let got = store2.get(&conv.id).unwrap();
        assert_eq!(got.title.as_deref(), Some("Renamed"));
        assert_eq!(got.status, ConversationStatus::Completed);
    }

    #[test]
    fn list_is_newest_first() {
        let dir = temp_dir();
        let store = ConversationStore::at(dir.clone());
        let a = store.create(
            ConversationKind::Chat,
            "ws1".into(),
            Driver::Human,
            None,
            None,
        );
        // tiny sleep to ensure updated_at differs
        std::thread::sleep(std::time::Duration::from_secs(1));
        let b = store.create(
            ConversationKind::Chat,
            "ws1".into(),
            Driver::Human,
            None,
            None,
        );
        let list = store.list();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].id, b.id, "newest first");
        assert_eq!(list[1].id, a.id);
    }

    #[test]
    fn migrate_chats_creates_conversations() {
        use crate::chats::ChatStore;
        let dir = temp_dir();
        let chat_store = ChatStore::at(dir.join("chats.json"));
        // Create a session with a message.
        let session = chat_store.create("superpowers", None).unwrap();
        chat_store
            .append_message(&session.id, crate::chats::ChatMessage::user("hello world"))
            .unwrap();

        let conv_store = ConversationStore::at(dir.join("conversations"));
        let count = conv_store.migrate_chats(&chat_store);
        assert_eq!(count, 1);
        let list = conv_store.list();
        assert_eq!(list.len(), 1);
        let conv = conv_store.get(&list[0].id).unwrap();
        assert_eq!(conv.kind, ConversationKind::Chat);
        // The user message should be one of the turns.
        assert!(
            conv.turns
                .iter()
                .any(|t| t.content == "hello world" && t.from == "human"),
            "user message should migrate to a human turn"
        );
    }

    #[test]
    fn migrate_chats_idempotent() {
        use crate::chats::ChatStore;
        let dir = temp_dir();
        let chat_store = ChatStore::at(dir.join("chats.json"));
        chat_store.create("superpowers", None).unwrap();

        let conv_store = ConversationStore::at(dir.join("conversations"));
        let count1 = conv_store.migrate_chats(&chat_store);
        assert_eq!(count1, 1);
        // Second call should be a no-op (conversations already exist).
        let count2 = conv_store.migrate_chats(&chat_store);
        assert_eq!(count2, 0);
    }

    #[test]
    fn migrate_chats_empty_is_noop() {
        use crate::chats::ChatStore;
        let dir = temp_dir();
        let chat_store = ChatStore::at(dir.join("chats.json"));
        // No sessions at all.
        let conv_store = ConversationStore::at(dir.join("conversations"));
        let count = conv_store.migrate_chats(&chat_store);
        assert_eq!(count, 0);
        assert!(conv_store.list().is_empty());
    }

    // --- run_event_to_turns tests ---

    #[test]
    fn run_event_step_started_to_system_turn() {
        let event = crate::relay::store::RunEvent::StepStarted {
            timestamp: 42,
            step_id: "s1".into(),
            role_id: "advisor".into(),
        };
        let turns = run_event_to_turns(&event, 0);
        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0].seq, 0);
        assert_eq!(turns[0].kind, TurnKind::System);
        assert_eq!(turns[0].from, "advisor");
        assert_eq!(turns[0].timestamp, 42);
        assert!(turns[0].content.contains("s1"));
    }

    #[test]
    fn run_event_turn_delta_to_message_turn() {
        let event = crate::relay::store::RunEvent::TurnDelta {
            timestamp: 1,
            role_id: "coder".into(),
            text: "writing code".into(),
        };
        let turns = run_event_to_turns(&event, 5);
        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0].kind, TurnKind::Message);
        assert_eq!(turns[0].from, "coder");
        assert_eq!(turns[0].content, "writing code");
        assert_eq!(turns[0].seq, 5);
    }

    #[test]
    fn run_event_tool_call_and_result() {
        let call = crate::relay::store::RunEvent::TurnToolCall {
            timestamp: 0,
            role_id: "coder".into(),
            tool_id: "t1".into(),
            tool_name: "read_file".into(),
            arguments: serde_json::json!({"path": "x"}),
        };
        let res = crate::relay::store::RunEvent::TurnToolResult {
            timestamp: 0,
            role_id: "coder".into(),
            tool_id: "t1".into(),
            result: "contents".into(),
            details: None,
        };
        let call_turns = run_event_to_turns(&call, 0);
        assert_eq!(call_turns.len(), 1);
        assert_eq!(call_turns[0].kind, TurnKind::ToolCall);
        assert_eq!(call_turns[0].tool.as_ref().unwrap().name, "read_file");

        let res_turns = run_event_to_turns(&res, 1);
        assert_eq!(res_turns.len(), 1);
        assert_eq!(res_turns[0].kind, TurnKind::ToolResult);
        assert_eq!(res_turns[0].tool.as_ref().unwrap().result, "contents");
    }

    /// PLAN-042:TurnToolResult.details 映射进 ToolRecord 并随 Turn 序列化
    /// （会话刷新回放后 diff 徽标仍渲染的持久化锚点）。
    #[test]
    fn run_event_tool_result_details_persisted_into_turn() {
        let res = crate::relay::store::RunEvent::TurnToolResult {
            timestamp: 0,
            role_id: "coder".into(),
            tool_id: "t1".into(),
            result: "done".into(),
            details: Some(serde_json::json!({
                "diff": "-2 old\n+2 new",
                "first_changed_line": 2,
            })),
        };
        let turns = run_event_to_turns(&res, 0);
        let tool = turns[0].tool.as_ref().unwrap();
        assert_eq!(
            tool.details.as_ref().unwrap()["diff"],
            "-2 old\n+2 new"
        );
        // 序列化往返(会话落盘/回放链路)details 不丢。
        let json = serde_json::to_value(&turns[0]).unwrap();
        assert_eq!(json["tool"]["details"]["first_changed_line"], 2);
        let back: Turn = serde_json::from_value(json).unwrap();
        assert!(back.tool.unwrap().details.is_some());
    }

    #[test]
    fn run_event_gate_waiting_and_resolved() {
        let waiting = crate::relay::store::RunEvent::GateWaiting {
            timestamp: 0,
            step_id: "gate1".into(),
            gate: "human".into(),
        };
        let resolved = crate::relay::store::RunEvent::GateResolved {
            timestamp: 0,
            step_id: "gate1".into(),
            decision: "approve".into(),
        };
        let w = run_event_to_turns(&waiting, 0);
        assert_eq!(w.len(), 1);
        assert_eq!(w[0].kind, TurnKind::Gate);
        assert_eq!(w[0].gate.as_ref().unwrap().status, "waiting");

        let r = run_event_to_turns(&resolved, 1);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].from, "human");
        assert_eq!(r[0].gate.as_ref().unwrap().status, "approve");
    }

    #[test]
    fn run_event_token_spend_produces_no_turn() {
        let event = crate::relay::store::RunEvent::TokenSpend {
            timestamp: 0,
            cumulative: 100,
            step_tokens: 10,
        };
        let turns = run_event_to_turns(&event, 0);
        assert!(turns.is_empty(), "TokenSpend is metadata, not a turn");
    }

    #[test]
    fn run_event_run_failed_and_completed() {
        let failed = crate::relay::store::RunEvent::RunFailed {
            timestamp: 0,
            error: "boom".into(),
        };
        let completed = crate::relay::store::RunEvent::RunCompleted { timestamp: 0, report: Default::default() };
        let f = run_event_to_turns(&failed, 0);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].kind, TurnKind::System);
        assert!(f[0].content.contains("boom"));
        let c = run_event_to_turns(&completed, 1);
        assert_eq!(c[0].content, "Flow completed");
    }
}
