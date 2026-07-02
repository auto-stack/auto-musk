//! Unified conversation model — the common abstraction for chat + flow.
//! See designs/007-unified-workflow-architecture.md.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

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
}

#[allow(dead_code)]
impl ConversationStore {
    /// Create a store rooted at `dir` (e.g. {root}/.autoos/conversations).
    /// Loads index.json into the cache (summaries only — turns loaded lazily on get).
    pub fn new(dir: PathBuf) -> Self {
        std::fs::create_dir_all(&dir).ok();
        let store = Self {
            dir,
            cache: Mutex::new(HashMap::new()),
        };
        store.warm_cache_from_index();
        store
    }

    /// Alias for `new` — handy in tests.
    pub fn at(dir: PathBuf) -> Self {
        Self::new(dir)
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
        let now = now_secs();
        let conv = Conversation {
            id: new_id(ID_LEN),
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
        };
        self.persist_new(&conv);
        conv
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

        self.mutate(id, |conv| {
            let seq = conv.turns.len();
            turn.seq = seq;
            if let Some(t) = turn.tokens {
                conv.cumulative_tokens = conv.cumulative_tokens.saturating_add(t);
            }
            conv.turns.push(turn);
        })
    }

    /// Update conversation status.
    pub fn set_status(&self, id: &str, status: ConversationStatus) -> Option<Conversation> {
        self.mutate(id, |conv| {
            conv.status = status;
        })
    }

    /// Update pending gate.
    pub fn set_gate(&self, id: &str, gate: Option<GateInfo>) -> Option<Conversation> {
        self.mutate(id, |conv| {
            conv.pending_gate = gate;
        })
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
}
