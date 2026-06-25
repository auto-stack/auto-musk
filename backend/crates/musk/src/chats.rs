//! Persistent multi-turn chat sessions for musk.
//!
//! Mirrors `SpecsStore`'s JSON-file persistence pattern: a single
//! `~/.config/autoos/chats.json` holds all sessions. Each `ChatSession` carries
//! its full message history so any HTTP request can rebuild an agent's memory
//! from it (Plan 008 Stage 3 will feed history into `Agent::with_history`).
//!
//! (Plan 008 — Chats web app, backend.)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Path: `~/.config/autoos/chats.json`.
fn chats_path() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|h| h.join(".config/autoos/chats.json"))
}

/// Seconds since the UNIX epoch (re-used convention from specs.rs).
fn now_sec() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// A random hex id (re-uses auth.rs's rand approach, no new dep).
fn new_id(nbytes: usize) -> String {
    use rand::RngCore;
    let mut buf = vec![0u8; nbytes];
    rand::thread_rng().fill_bytes(&mut buf);
    buf.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Who produced a message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    /// A tool-call observation (rendered inline, not a primary bubble).
    Tool,
}

/// A tool call recorded on an assistant message (tool name + args + result).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool: String,
    pub args: serde_json::Value,
    pub result: String,
}

/// A single chat message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub role: Role,
    pub content: String,
    /// Tool calls made during this (assistant) message, if any.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCall>,
    pub created_at: u64,
}

impl ChatMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            id: new_id(8),
            role: Role::User,
            content: content.into(),
            tool_calls: Vec::new(),
            created_at: now_sec(),
        }
    }
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            id: new_id(8),
            role: Role::Assistant,
            content: content.into(),
            tool_calls: Vec::new(),
            created_at: now_sec(),
        }
    }
}

/// A persisted multi-turn chat session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub name: String,
    /// Mode used to build the agent for this session (e.g. "superpowers").
    pub mode: String,
    pub messages: Vec<ChatMessage>,
    pub created_at: u64,
    pub updated_at: u64,
}

/// A lightweight summary for list views (no message bodies).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSessionSummary {
    pub id: String,
    pub name: String,
    pub mode: String,
    pub message_count: usize,
    /// First ~80 chars of the last user message, for a preview.
    pub preview: String,
    pub updated_at: u64,
}

impl ChatSession {
    pub fn new(mode: impl Into<String>) -> Self {
        let now = now_sec();
        Self {
            id: new_id(12),
            name: "New chat".into(),
            mode: mode.into(),
            messages: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn summary(&self) -> ChatSessionSummary {
        let preview = self
            .messages
            .iter()
            .rev()
            .find(|m| m.role == Role::User)
            .map(|m| {
                let c = m.content.chars().take(80).collect::<String>();
                if m.content.chars().count() > 80 {
                    format!("{c}…")
                } else {
                    c
                }
            })
            .unwrap_or_default();
        ChatSessionSummary {
            id: self.id.clone(),
            name: self.name.clone(),
            mode: self.mode.clone(),
            message_count: self.messages.len(),
            preview,
            updated_at: self.updated_at,
        }
    }

    /// Append a message and bump `updated_at`.
    pub fn append(&mut self, msg: ChatMessage) {
        self.messages.push(msg);
        self.updated_at = now_sec();
        // Auto-name from the first user message if still default.
        if self.name == "New chat" {
            if let Some(first_user) = self.messages.iter().find(|m| m.role == Role::User) {
                self.name = first_user
                    .content
                    .chars()
                    .take(40)
                    .collect::<String>()
                    .trim()
                    .to_string();
            }
        }
    }
}

/// JSON-file-backed store of chat sessions, keyed by session id.
///
/// Fault-tolerant like SpecsStore: a missing file starts empty; a corrupt file
/// logs a warning and starts empty (never panics).
#[derive(Debug, Clone)]
pub struct ChatStore {
    path: std::path::PathBuf,
}

impl ChatStore {
    /// The default store at `~/.config/autoos/chats.json`.
    pub fn default_path() -> Self {
        Self {
            path: chats_path().unwrap_or_else(|| std::path::PathBuf::from("chats.json")),
        }
    }

    /// Open a store at an explicit path (mainly for tests).
    pub fn at(path: impl Into<std::path::PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Load all sessions. Missing/corrupt file → empty map.
    fn load_map(&self) -> HashMap<String, ChatSession> {
        match std::fs::read(&self.path) {
            Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_else(|e| {
                tracing::warn!("chats: failed to parse {}: {e}", self.path.display());
                HashMap::new()
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => HashMap::new(),
            Err(e) => {
                tracing::warn!("chats: failed to read {}: {e}", self.path.display());
                HashMap::new()
            }
        }
    }

    /// Persist the session map to disk.
    fn save_map(&self, map: &HashMap<String, ChatSession>) -> std::io::Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let bytes = serde_json::to_vec_pretty(map)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(&self.path, bytes)
    }

    /// Create + persist a new session; return it.
    pub fn create(&self, mode: &str) -> std::io::Result<ChatSession> {
        let mut map = self.load_map();
        let session = ChatSession::new(mode);
        map.insert(session.id.clone(), session.clone());
        self.save_map(&map)?;
        Ok(session)
    }

    /// List all sessions as summaries, newest first (by updated_at).
    pub fn list(&self) -> Vec<ChatSessionSummary> {
        let mut summaries: Vec<_> = self.load_map().values().map(|s| s.summary()).collect();
        summaries.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        summaries
    }

    /// Get one full session by id.
    pub fn get(&self, id: &str) -> Option<ChatSession> {
        self.load_map().remove(id)
    }

    /// Rename a session.
    pub fn rename(&self, id: &str, name: &str) -> std::io::Result<Option<ChatSession>> {
        let mut map = self.load_map();
        if let Some(session) = map.get_mut(id) {
            session.name = name.to_string();
            session.updated_at = now_sec();
            let updated = session.clone();
            self.save_map(&map)?;
            Ok(Some(updated))
        } else {
            Ok(None)
        }
    }

    /// Delete one session; return whether it existed.
    pub fn delete(&self, id: &str) -> std::io::Result<bool> {
        let mut map = self.load_map();
        let existed = map.remove(id).is_some();
        if existed {
            self.save_map(&map)?;
        }
        Ok(existed)
    }

    /// Delete all sessions.
    pub fn delete_all(&self) -> std::io::Result<()> {
        self.save_map(&HashMap::new())
    }

    /// Append a message to a session and persist. Returns the updated session
    /// or None if the id wasn't found.
    pub fn append_message(
        &self,
        id: &str,
        msg: ChatMessage,
    ) -> std::io::Result<Option<ChatSession>> {
        let mut map = self.load_map();
        if let Some(session) = map.get_mut(id) {
            session.append(msg);
            let updated = session.clone();
            self.save_map(&map)?;
            Ok(Some(updated))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_store() -> (ChatStore, tempfile::NamedTempFile) {
        let f = tempfile::NamedTempFile::new().unwrap();
        let store = ChatStore::at(f.path());
        // Start from an empty file so load_map doesn't warn.
        std::fs::write(f.path(), b"{}").unwrap();
        (store, f)
    }

    #[test]
    fn create_and_get_session() {
        let (store, _f) = temp_store();
        let s = store.create("superpowers").unwrap();
        assert_eq!(s.mode, "superpowers");
        assert!(s.messages.is_empty());
        let loaded = store.get(&s.id).unwrap();
        assert_eq!(loaded.id, s.id);
    }

    #[test]
    fn list_returns_all_summaries() {
        let (store, _f) = temp_store();
        let a = store.create("superpowers").unwrap();
        let b = store.create("coding").unwrap();
        let list = store.list();
        assert_eq!(list.len(), 2);
        // Both created within the same second → equal updated_at; ordering
        // between them is unspecified, so just assert both are present.
        let ids: Vec<_> = list.iter().map(|s| s.id.clone()).collect();
        assert!(ids.contains(&a.id));
        assert!(ids.contains(&b.id));
    }

    #[test]
    fn append_message_persists_and_autonames() {
        let (store, _f) = temp_store();
        let s = store.create("superpowers").unwrap();
        let updated = store
            .append_message(&s.id, ChatMessage::user("List the files in this dir"))
            .unwrap()
            .unwrap();
        assert_eq!(updated.messages.len(), 1);
        assert_eq!(updated.name, "List the files in this dir");
        // Preview reflects the user message.
        assert_eq!(updated.summary().preview, "List the files in this dir");

        // Reload from disk to confirm persistence.
        let reloaded = store.get(&s.id).unwrap();
        assert_eq!(reloaded.messages.len(), 1);
    }

    #[test]
    fn rename_and_delete() {
        let (store, _f) = temp_store();
        let s = store.create("superpowers").unwrap();
        let renamed = store.rename(&s.id, "My coding task").unwrap().unwrap();
        assert_eq!(renamed.name, "My coding task");
        assert!(store.delete(&s.id).unwrap());
        assert!(store.get(&s.id).is_none());
        assert!(!store.delete(&s.id).unwrap()); // already gone
    }

    #[test]
    fn missing_file_starts_empty() {
        let store = ChatStore::at("/nonexistent/path/chats-test.json");
        assert!(store.list().is_empty());
        // create persists (creating the dir).
        let s = store.create("x").unwrap();
        assert_eq!(store.list().len(), 1);
        // cleanup
        let _ = std::fs::remove_file("/nonexistent/path/chats-test.json");
        let _ = s;
    }

    #[test]
    fn corrupt_file_does_not_panic() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(b"not json {{{").unwrap();
        let store = ChatStore::at(f.path());
        assert!(store.list().is_empty()); // warns, returns empty
    }
}
