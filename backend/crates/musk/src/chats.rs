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

/// Seconds since the UNIX epoch (re-used convention from specs.rs).
fn now_sec() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// PLAN-084 T-03: 会话列表时间展示三元组（本地时区日界分组 + 时刻/日期文本）。
/// VM 轨无可用时钟（`Date.now()` 返负垃圾既有缺陷，KNOWN-DEBT 083 行），
/// 日历运算收敛在后端——后端与桌面同机同时区。返回 `(day_group, time_text,
/// date_text)`：group 0=今天 / 1=昨天 / 2=更早；今天与昨天给 `"HH:mm"`，
/// 更早年给 `"MM-DD"`（跨年 `"YYYY-MM-DD"`）。时钟异常（epoch 前）回落
/// "更早" + 空文本，不 panic。
fn summary_time_fields(updated: u64) -> (u8, String, String) {
    use chrono::{Datelike, TimeZone};
    let now = chrono::Local::now();
    let dt = match chrono::Local.timestamp_opt(updated as i64, 0).single() {
        Some(dt) => dt,
        None => return (2, String::new(), String::new()),
    };
    let today = now.date_naive();
    let day = dt.date_naive();
    let group = if day == today {
        0
    } else if day == today - chrono::Duration::days(1) {
        1
    } else {
        2
    };
    let time_text = dt.format("%H:%M").to_string();
    let date_text = if day.year() == today.year() {
        dt.format("%m-%d").to_string()
    } else {
        dt.format("%Y-%m-%d").to_string()
    };
    (group, time_text, date_text)
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
///
/// Field wire names (`name` / `arguments`) match what the Vue frontend reads
/// (`ToolCallInfo.name` / `.arguments`). The `alias` keeps older persisted
/// `chats.json` rows (which used `tool` / `args`) deserializable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    #[serde(rename = "name", alias = "tool")]
    pub tool: String,
    #[serde(rename = "arguments", alias = "args")]
    pub args: serde_json::Value,
    pub result: String,
    /// `"success"` / `"error"` — surfaced to the frontend for card styling.
    #[serde(default = "default_status", skip_serializing_if = "is_default_status")]
    pub status: String,
    /// Stable id matching the SSE `tool_call`/`tool_result` pair (e.g. `tc-1`).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
}

fn default_status() -> String { "success".into() }
fn is_default_status(s: &String) -> bool { s == "success" }

/// A single chat message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub role: Role,
    pub content: String,
    /// Reasoning trace streamed before/alongside the text (assistant only).
    /// Persisted so the frontend ThinkBlock survives a page reload.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub thinking: String,
    /// Tool calls made during this (assistant) message, if any.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCall>,
    pub created_at: u64,
    /// PLAN-043: 会话树父指针（None = 线性接在上一条之后/旧数据）。树语义
    /// 单源于 ChatStore（ConversationStore 镜像保持线性 journal）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    /// PLAN-069 W2：活动时间线块（按**执行序**追加）——叙述文本与工具调用
    /// 穿插，替代"content 整段 + tool_calls 尾挂"的塌缩投影。空 = 旧数据
    /// （前端回退 content+tool_calls 渲染）。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocks: Vec<ChatBlock>,
    /// PLAN-071 r3（需求②）：回答方 Agent 职业身份（professions.json 目录
    /// id，如 "assistant"/"advisor"）。仅 assistant 落盘点填充；None = 旧
    /// 数据/无身份（前端回退 "🤖 AI" 徽章）。词汇沿 auto-forge。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profession_id: Option<String>,
    /// PLAN-073 P2-T3：run 增量落盘标记——true = 运行中 turn 快照（收束
    /// 终版以同 id 原位替换并清除）；None = 终态/旧数据（回退安全）。
    /// 前端轮询据此区分"快照已换入但终版未落"与"收束完成"。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pending: Option<bool>,
    /// PLAN-093 T-08：元素附件快照（design_context，可选增量）——用户
    /// 拾取元素的定位数据（vnode/kind/label/source/归属代次）。旧会话
    /// 文件缺字段反序列化为 None（默认值读取，回放安全）；落盘前经
    /// chats_message 归属校验并盖 ownership 章。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub design_context: Option<serde_json::Value>,
}

/// 活动时间线块：kind = "text"（叙述/回答文本）| "tool"（一次工具调用，
/// 执行后 result/status 回填同块）。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatBlock {
    pub kind: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool: Option<ToolCall>,
}

impl ChatMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            id: new_id(8),
            role: Role::User,
            content: content.into(),
            thinking: String::new(),
            tool_calls: Vec::new(),
            created_at: now_sec(),
            parent_id: None,
            blocks: Vec::new(),
            profession_id: None,
            pending: None,
            design_context: None,
        }
    }
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            id: new_id(8),
            role: Role::Assistant,
            content: content.into(),
            thinking: String::new(),
            tool_calls: Vec::new(),
            created_at: now_sec(),
            parent_id: None,
            blocks: Vec::new(),
            profession_id: None,
            pending: None,
            design_context: None,
        }
    }
}

/// PLAN-073 P2-T3：run 消息 id 生成口（chat run 起跑定 id，turn 快照与
/// 收束终版共用，保证 upsert 原位替换）。
pub fn new_message_id() -> String {
    new_id(8)
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
    /// Spec changes proposed by the agent, awaiting user approval (Plan 009 P1b).
    /// When the agent calls `update_spec`, the change is queued here instead of
    /// applied directly; the user approves/rejects via the HTTP endpoints.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pending_spec_changes: Vec<crate::specs::SpecChange>,
    /// Which workspace this session belongs to (for agent root routing).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    /// PLAN-043: 活跃分支叶（消息 id）。None = 线性（旧数据或未分叉）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_leaf: Option<String>,
    /// PLAN-064: 本会话的思考档位（"off"|"low"|"high"|"max"）。None = 跟随
    /// role 默认（不注入 thinking 参数）。随会话持久化，UI 档位选择器写这里。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thinking_level: Option<String>,
    /// PLAN-067 T-05: 本会话审批模式（"human"|"auto"）。auto = relay run 的
    /// human gate 到达即自动放行（driver 侧，留审计事件），用于无人值守跑通
    /// 全流程。缺省/旧数据 = "human"（现行为，gate 等待人工决议）。
    #[serde(default = "default_approval_mode")]
    pub approval_mode: String,
    /// PLAN-084 T-05: 归档态。归档只影响列表可见性（chat_list 滤除），
    /// 不删消息、不 bump updated_at。false 跳过序列化（旧档 JSON 零 diff）。
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub archived: bool,
}

fn default_approval_mode() -> String {
    "human".to_string()
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
    /// PLAN-084 T-05: 归档态（列表级隐藏，不改消息数据）。false 时跳过
    /// 序列化——旧档 JSON 零 diff、旧前端零感知（ag 镜像不承载本字段，
    /// 沿「镜像仅承载 parity 所需字段」哲学）。
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub archived: bool,
    /// PLAN-084 T-03: 展示元数据——后端本地时区现算（见 summary_time_fields）。
    #[serde(default)]
    pub day_group: u8,
    /// 今天/昨天行的时刻文本（"HH:mm"）。
    #[serde(default)]
    pub time_text: String,
    /// 更早行的日期文本（本年 "MM-DD"，跨年 "YYYY-MM-DD"）。
    #[serde(default)]
    pub date_text: String,
}

impl ChatSession {
    pub fn new(mode: impl Into<String>, workspace_id: Option<String>) -> Self {
        let now = now_sec();
        Self {
            id: new_id(12),
            name: "New chat".into(),
            mode: mode.into(),
            messages: Vec::new(),
            created_at: now,
            updated_at: now,
            pending_spec_changes: Vec::new(),
            workspace_id,
            active_leaf: None,
            thinking_level: None,
            approval_mode: default_approval_mode(),
            archived: false,
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
        let (day_group, time_text, date_text) = summary_time_fields(self.updated_at);
        ChatSessionSummary {
            id: self.id.clone(),
            name: self.name.clone(),
            mode: self.mode.clone(),
            message_count: self.messages.len(),
            preview,
            updated_at: self.updated_at,
            archived: self.archived,
            day_group,
            time_text,
            date_text,
        }
    }

    /// Append a message and bump `updated_at`.
    /// PLAN-043: 新消息挂到当前活跃叶下（线性会话 parent=None 直到首次分叉），
    /// 追加后活跃叶前移到新消息。
    pub fn append(&mut self, msg: ChatMessage) {
        let mut msg = msg;
        msg.parent_id = self.active_leaf.clone();
        self.active_leaf = Some(msg.id.clone());
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

    // ── PLAN-043: 会话树投影 ─────────────────────────────────────

    /// 活跃路径（leaf → root 链 + 线性前缀）。旧数据（无 parent 链）自然
    /// 退化为全部消息的线性读取。
    pub fn active_path(&self) -> Vec<&ChatMessage> {
        let leaf_id = match &self.active_leaf {
            Some(l) => l,
            None => return self.messages.iter().collect(),
        };
        let idx: std::collections::HashMap<&str, usize> = self
            .messages
            .iter()
            .enumerate()
            .map(|(i, m)| (m.id.as_str(), i))
            .collect();
        let Some(&leaf) = idx.get(leaf_id.as_str()) else {
            return self.messages.iter().collect();
        };
        // leaf → root 回溯（父索引必须在前——无前向引用/环）。
        let mut chain: Vec<usize> = vec![leaf];
        let mut cur = leaf;
        while let Some(pid) = &self.messages[cur].parent_id {
            match idx.get(pid.as_str()) {
                Some(&pi) if pi < cur => {
                    chain.push(pi);
                    cur = pi;
                }
                _ => break,
            }
        }
        let anchor = chain.pop().expect("chain non-empty"); // 线性前缀终点
        let mut result: Vec<&ChatMessage> = self.messages[..=anchor].iter().collect();
        for &ci in chain.iter().rev() {
            result.push(&self.messages[ci]);
        }
        result
    }

    /// 活跃路径上的 (role, content) 历史对（排除路径末尾的 user 消息——那是
    /// 即将运行的一条；tool 观测不进 plain 历史）。与改造前的线性构建逻辑
    /// 逐字一致，仅消息序列换成活跃路径。
    pub fn history_pairs(&self) -> Vec<(String, String)> {
        let mut pairs: Vec<(String, String)> = Vec::new();
        let mut seen_last_user = false;
        for m in self.active_path().into_iter().rev() {
            if !seen_last_user && m.role == Role::User {
                seen_last_user = true;
                continue;
            }
            let role = match m.role {
                Role::User => "user",
                Role::Assistant => "assistant",
                Role::Tool => continue,
            };
            pairs.push((role.to_string(), m.content.clone()));
        }
        pairs.reverse();
        pairs
    }

    /// 把活跃叶切到指定消息（fork-from 与 navigate 共用同一机制：都不复制
    /// 数据，只改指针——新消息将挂到该点之下形成/延续分支）。返回 false =
    /// 消息不存在。
    pub fn set_active_leaf(&mut self, message_id: &str) -> bool {
        if self.messages.iter().any(|m| m.id == message_id) {
            self.active_leaf = Some(message_id.to_string());
            self.updated_at = now_sec();
            true
        } else {
            false
        }
    }

    /// 树节点投影（GET tree 端点用）：每条消息的 id/role/预览行/子分支数，
    /// 供前端渲染分叉标记与分支切换器。
    pub fn tree_nodes(&self) -> Vec<serde_json::Value> {
        use std::collections::HashMap;
        let mut children: HashMap<&str, usize> = HashMap::new();
        for m in &self.messages {
            if let Some(p) = &m.parent_id {
                *children.entry(p.as_str()).or_insert(0) += 1;
            }
        }
        let active: std::collections::HashSet<&str> =
            self.active_path().iter().map(|m| m.id.as_str()).collect();
        self.messages
            .iter()
            .map(|m| {
                serde_json::json!({
                    "id": m.id,
                    "role": match m.role { Role::User => "user", Role::Assistant => "assistant", Role::Tool => "tool" },
                    "parent_id": m.parent_id,
                    "preview": m.content.chars().take(60).collect::<String>(),
                    "children": children.get(m.id.as_str()).copied().unwrap_or(0),
                    "on_active_path": active.contains(m.id.as_str()),
                })
            })
            .collect()
    }
}

/// JSON-file-backed store of chat sessions, keyed by session id.
///
/// Fault-tolerant like SpecsStore: a missing file starts empty; a corrupt file
/// logs a warning and starts empty (never panics).
#[derive(Debug)]
pub struct ChatStore {
    path: std::path::PathBuf,
    /// PLAN-071 需求⑮：写路径互斥——所有变更走 load_map→改→save_map 的
    /// 非原子读改写，运行主体落盘回复与并发消息 POST（排队自动重发）交错
    /// 时后写覆盖前写（实证：排队消息追平落盘，首个回答被整条覆盖丢失）。
    /// Arc<ChatStore> 共享，方法内取锁串行化全部变更。
    write_lock: std::sync::Mutex<()>,
}

impl ChatStore {
    /// Open a store at an explicit path (mainly for tests).
    pub fn at(path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            path: path.into(),
            write_lock: std::sync::Mutex::new(()),
        }
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
        // PLAN-044 T-05：原子替换（会话全量单文件，撕裂代价最高）
        auto_lang::state_file::atomic_write(&self.path, &bytes)
    }

    /// Create + persist a new session; return it.
    pub fn create(
        &self,
        mode: &str,
        workspace_id: Option<String>,
    ) -> std::io::Result<ChatSession> {
        let _write_guard = self.write_lock.lock().unwrap();
        let mut map = self.load_map();
        let session = ChatSession::new(mode, workspace_id);
        map.insert(session.id.clone(), session.clone());
        self.save_map(&map)?;
        Ok(session)
    }

    /// List all sessions as summaries, newest first (by updated_at).
    /// PLAN-084 T-05: 默认滤除已归档（wire 兼容——旧前端零感知归档态）；
    /// 已归档面走 [`Self::list_archived`]。
    pub fn list(&self) -> Vec<ChatSessionSummary> {
        let mut summaries: Vec<_> = self
            .load_map()
            .values()
            .map(|s| s.summary())
            .filter(|s| !s.archived)
            .collect();
        summaries.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        summaries
    }

    /// PLAN-084 T-05: 已归档会话列表（"已归档"过滤开关的数据面），同序。
    pub fn list_archived(&self) -> Vec<ChatSessionSummary> {
        let mut summaries: Vec<_> = self
            .load_map()
            .values()
            .map(|s| s.summary())
            .filter(|s| s.archived)
            .collect();
        summaries.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        summaries
    }

    /// Get one full session by id.
    pub fn get(&self, id: &str) -> Option<ChatSession> {
        self.load_map().remove(id)
    }

    /// Rename a session.
    pub fn rename(&self, id: &str, name: &str) -> std::io::Result<Option<ChatSession>> {
        let _write_guard = self.write_lock.lock().unwrap();
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

    /// PLAN-084 T-05: 归档/取消归档。列表级隐藏（`list` 滤除 /
    /// `list_archived` 收录），不删消息、不 bump updated_at（归档不是聊天
    /// 活动，避免列表时间跳变）。返回更新后的会话，id 未知返 None。
    pub fn archive(&self, id: &str, archived: bool) -> std::io::Result<Option<ChatSession>> {
        let _write_guard = self.write_lock.lock().unwrap();
        let mut map = self.load_map();
        if let Some(session) = map.get_mut(id) {
            session.archived = archived;
            let updated = session.clone();
            self.save_map(&map)?;
            Ok(Some(updated))
        } else {
            Ok(None)
        }
    }

    /// PLAN-064: set (or clear, with `None`) a session's thinking level.
    /// Stored leniently — unknown names are rejected (warn + skip) at the
    /// daemon's injection point, not here. Empty string normalizes to None
    /// (ag 轨 api 绑定无可选参数，空串即"跟随 role 默认"). Returns the updated
    /// session, or `None` when the id is unknown.
    pub fn set_thinking_level(
        &self,
        id: &str,
        level: Option<String>,
    ) -> std::io::Result<Option<ChatSession>> {
        let _write_guard = self.write_lock.lock().unwrap();
        let level = level.filter(|s| !s.trim().is_empty());
        let mut map = self.load_map();
        if let Some(session) = map.get_mut(id) {
            session.thinking_level = level;
            session.updated_at = now_sec();
            let updated = session.clone();
            self.save_map(&map)?;
            Ok(Some(updated))
        } else {
            Ok(None)
        }
    }

    /// PLAN-067 T-05: 设置会话审批模式（"human"|"auto"）。非法值回落
    /// "human"（gate 等待人工决议的现行为）。
    pub fn set_approval_mode(
        &self,
        id: &str,
        mode: &str,
    ) -> std::io::Result<Option<ChatSession>> {
        let _write_guard = self.write_lock.lock().unwrap();
        let mode = if mode == "auto" { "auto" } else { "human" }.to_string();
        let mut map = self.load_map();
        if let Some(session) = map.get_mut(id) {
            session.approval_mode = mode;
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
        let _write_guard = self.write_lock.lock().unwrap();
        let mut map = self.load_map();
        let existed = map.remove(id).is_some();
        if existed {
            self.save_map(&map)?;
        }
        Ok(existed)
    }

    /// Delete all sessions.
    pub fn delete_all(&self) -> std::io::Result<()> {
        let _write_guard = self.write_lock.lock().unwrap();
        self.save_map(&HashMap::new())
    }

    /// Append a message to a session and persist. Returns the updated session
    /// or None if the id wasn't found.
    pub fn append_message(
        &self,
        id: &str,
        msg: ChatMessage,
    ) -> std::io::Result<Option<ChatSession>> {
        let _write_guard = self.write_lock.lock().unwrap();
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

    /// PLAN-073 P2-T3：按消息 id 原位替换（run 的 turn 增量快照 → 收束终版
    /// 同 id 换入）；session 内无该消息 id 时退化为 append（parent/leaf 语义
    /// 同 append_message）。替换不动 parent_id/active_leaf——消息位置与分支
    /// 关系保持，只换载荷。返回值语义同 append_message。
    pub fn upsert_message(
        &self,
        id: &str,
        msg: ChatMessage,
    ) -> std::io::Result<Option<ChatSession>> {
        let _write_guard = self.write_lock.lock().unwrap();
        let mut map = self.load_map();
        if let Some(session) = map.get_mut(id) {
            match session.messages.iter().position(|m| m.id == msg.id) {
                Some(idx) => {
                    // 原位替换保留树链与创建时点：替换消息由收束组装独立
                    // 构造（不重导 parent），整体换入会把消息踢出所在分支
                    // （测试 upsert_message_replaces_in_place 实证）。
                    let mut msg = msg;
                    msg.parent_id = session.messages[idx].parent_id.clone();
                    msg.created_at = session.messages[idx].created_at;
                    session.messages[idx] = msg;
                    session.updated_at = now_sec();
                }
                None => session.append(msg),
            }
            let updated = session.clone();
            self.save_map(&map)?;
            Ok(Some(updated))
        } else {
            Ok(None)
        }
    }

    /// PLAN-043: 切换会话活跃叶（fork/navigate 共用），持久化后返回更新会话。
    pub fn set_active_leaf(
        &self,
        id: &str,
        message_id: &str,
    ) -> std::io::Result<Option<ChatSession>> {
        let _write_guard = self.write_lock.lock().unwrap();
        let mut map = self.load_map();
        if let Some(session) = map.get_mut(id) {
            if !session.set_active_leaf(message_id) {
                return Ok(None);
            }
            let updated = session.clone();
            self.save_map(&map)?;
            Ok(Some(updated))
        } else {
            Ok(None)
        }
    }

    // ── Spec-change approval (Plan 009 P1b) ──────────────────

    /// Queue a spec change proposed by the agent (not yet applied). Returns
    /// the updated session, or None if the id wasn't found.
    pub fn queue_spec_change(
        &self,
        id: &str,
        change: crate::specs::SpecChange,
    ) -> std::io::Result<Option<ChatSession>> {
        let _write_guard = self.write_lock.lock().unwrap();
        let mut map = self.load_map();
        if let Some(session) = map.get_mut(id) {
            session.pending_spec_changes.push(change);
            session.updated_at = now_sec();
            let updated = session.clone();
            self.save_map(&map)?;
            Ok(Some(updated))
        } else {
            Ok(None)
        }
    }

    /// Approve the spec change at `index`: apply it to `specs` (upsert or
    /// set_status), then remove it from the pending queue. Returns the applied
    /// change + updated session, or None if session/index not found.
    pub fn approve_spec_change(
        &self,
        id: &str,
        index: usize,
        specs: &crate::specs::SpecsStore,
    ) -> Result<Option<(crate::specs::SpecChange, ChatSession)>, String> {
        let _write_guard = self.write_lock.lock().unwrap();
        let mut map = self.load_map();
        let session = map
            .get_mut(id)
            .ok_or_else(|| format!("session '{id}' not found"))?;
        if index >= session.pending_spec_changes.len() {
            return Err(format!("pending change index {index} out of range"));
        }
        let change = session.pending_spec_changes.remove(index);
        // Apply the change to the spec document.
        let mut doc = specs
            .load()
            .map_err(|e| format!("load specs: {e}"))?;
        apply_spec_change(&change, specs, &mut doc)?;
        specs
            .save(&doc)
            .map_err(|e| format!("save specs: {e}"))?;
        session.updated_at = now_sec();
        let updated = session.clone();
        self.save_map(&map)
            .map_err(|e| format!("save chats: {e}"))?;
        Ok(Some((change, updated)))
    }

    /// Reject (discard) the spec change at `index` without applying it.
    pub fn reject_spec_change(
        &self,
        id: &str,
        index: usize,
    ) -> Result<Option<ChatSession>, String> {
        let _write_guard = self.write_lock.lock().unwrap();
        let mut map = self.load_map();
        let session = map
            .get_mut(id)
            .ok_or_else(|| format!("session '{id}' not found"))?;
        if index >= session.pending_spec_changes.len() {
            return Err(format!("pending change index {index} out of range"));
        }
        session.pending_spec_changes.remove(index);
        session.updated_at = now_sec();
        let updated = session.clone();
        self.save_map(&map)
            .map_err(|e| format!("save chats: {e}"))?;
        Ok(Some(updated))
    }

    /// Reject all pending spec changes for a session.
    pub fn reject_all_spec_changes(&self, id: &str) -> Result<Option<ChatSession>, String> {
        let _write_guard = self.write_lock.lock().unwrap();
        let mut map = self.load_map();
        let session = map
            .get_mut(id)
            .ok_or_else(|| format!("session '{id}' not found"))?;
        session.pending_spec_changes.clear();
        session.updated_at = now_sec();
        let updated = session.clone();
        self.save_map(&map)
            .map_err(|e| format!("save chats: {e}"))?;
        Ok(Some(updated))
    }
}

/// Apply one SpecChange to a document via the store (upsert or set_status).
fn apply_spec_change(
    change: &crate::specs::SpecChange,
    store: &crate::specs::SpecsStore,
    doc: &mut crate::specs::SpecsDocument,
) -> Result<(), String> {
    use crate::specs::{SpecItem, SpecStatus};
    // If a status is given, treat as a status transition; otherwise upsert the
    // item (title/content) into the section.
    if let Some(new_status) = change.status {
        store.transition_item(doc, &change.section_id, &change.item_id, new_status)?;
        return Ok(());
    }
    // Build the item from the change (title/content); keep status Empty if new.
    let existing = doc
        .sections
        .iter()
        .find(|s| s.id == change.section_id)
        .and_then(|s| s.items.iter().find(|i| i.id == change.item_id))
        .cloned();
    let mut item = existing.unwrap_or_else(|| SpecItem::new(change.item_id.clone(), ""));
    if let Some(t) = &change.title {
        item.title = t.clone();
    }
    if let Some(c) = &change.content {
        item.content = c.clone();
    }
    if item.title.is_empty() && item.content.is_empty() {
        item.title = "(empty)".into();
    }
    // If brand new, status stays Empty; if existing, keep its status.
    let _ = SpecStatus::Empty; // suppress unused import warning if no status path
    store.upsert_item(doc, &change.section_id, item)?;
    Ok(())
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
        let s = store.create("superpowers", None).unwrap();
        assert_eq!(s.mode, "superpowers");
        assert!(s.messages.is_empty());
        let loaded = store.get(&s.id).unwrap();
        assert_eq!(loaded.id, s.id);
    }

    #[test]
    fn list_returns_all_summaries() {
        let (store, _f) = temp_store();
        let a = store.create("superpowers", None).unwrap();
        let b = store.create("coding", None).unwrap();
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
        let s = store.create("superpowers", None).unwrap();
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

    // ── PLAN-073 P2-T3：upsert_message（run 增量快照 → 收束终版同 id 换入）──

    #[test]
    fn upsert_message_appends_when_id_absent() {
        let (store, _f) = temp_store();
        let s = store.create("superpowers", None).unwrap();
        store
            .append_message(&s.id, ChatMessage::user("run this"))
            .unwrap()
            .unwrap();
        // 快照首落：session 内无该消息 id → 退化为 append（parent 挂当前叶）。
        let mut snap = ChatMessage::assistant("partial");
        snap.pending = Some(true);
        let updated = store.upsert_message(&s.id, snap).unwrap().unwrap();
        assert_eq!(updated.messages.len(), 2);
        assert_eq!(updated.messages[1].role, Role::Assistant);
        assert_eq!(updated.messages[1].pending, Some(true));
        assert!(updated.messages[1].parent_id.is_some());
        assert_eq!(updated.active_leaf.as_deref(), Some(updated.messages[1].id.as_str()));
    }

    #[test]
    fn upsert_message_replaces_in_place_keeping_position() {
        let (store, _f) = temp_store();
        let s = store.create("superpowers", None).unwrap();
        store
            .append_message(&s.id, ChatMessage::user("run this"))
            .unwrap()
            .unwrap();
        // 快照（首落）→ 终版（同 id 换入）。
        let mut snap = ChatMessage::assistant("partial");
        snap.pending = Some(true);
        let updated = store.upsert_message(&s.id, snap.clone()).unwrap().unwrap();
        let msg_id = updated.messages[1].id.clone();
        let parent = updated.messages[1].parent_id.clone();
        let mut final_msg = ChatMessage::assistant("final answer");
        final_msg.id = msg_id.clone();
        final_msg.thinking = "thought".into();
        final_msg.pending = None;
        let updated2 = store.upsert_message(&s.id, final_msg).unwrap().unwrap();
        // 原位替换：不增行、位置不变、parent/leaf 不动、pending 已清。
        assert_eq!(updated2.messages.len(), 2);
        assert_eq!(updated2.messages[1].id, msg_id);
        assert_eq!(updated2.messages[1].content, "final answer");
        assert_eq!(updated2.messages[1].thinking, "thought");
        assert_eq!(updated2.messages[1].pending, None);
        assert_eq!(updated2.messages[1].parent_id, parent);
        assert_eq!(updated2.active_leaf.as_deref(), Some(msg_id.as_str()));
        // 持久化确认。
        let reloaded = store.get(&s.id).unwrap();
        assert_eq!(reloaded.messages[1].content, "final answer");
        assert_eq!(reloaded.messages[1].pending, None);
    }

    #[test]
    fn upsert_message_unknown_session_returns_none() {
        let (store, _f) = temp_store();
        let msg = ChatMessage::assistant("x");
        assert!(store.upsert_message("no-such-session", msg).unwrap().is_none());
    }

    #[test]
    fn pending_field_backcompat_old_json_parses() {
        // 旧数据（无 pending 字段）反序列化安全回退 None；新数据往返不丢。
        let old = br#"{"id":"m1","role":"assistant","content":"hi","created_at":1}"#;
        let m: ChatMessage = serde_json::from_slice(old).unwrap();
        assert_eq!(m.pending, None);
        let mut n = ChatMessage::assistant("hi");
        n.pending = Some(true);
        let round: ChatMessage = serde_json::from_slice(&serde_json::to_vec(&n).unwrap()).unwrap();
        assert_eq!(round.pending, Some(true));
    }

    #[test]
    fn rename_and_delete() {
        let (store, _f) = temp_store();
        let s = store.create("superpowers", None).unwrap();
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
        let s = store.create("x", None).unwrap();
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

    // ── Spec-change approval (Plan 009 P1b) ──────────────────

    use crate::specs::{SpecChange, SpecsStore};

    fn tmp_specs() -> SpecsStore {
        use std::sync::atomic::{AtomicU64, Ordering};
        static N: AtomicU64 = AtomicU64::new(0);
        let n = N.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!("musk_chats_approve_{}_{}.json", std::process::id(), n));
        let _ = std::fs::remove_file(&path);
        SpecsStore::new(path)
    }

    #[test]
    fn queue_then_reject_spec_change() {
        let (store, _f) = temp_store();
        let s = store.create("superpowers", None).unwrap();
        let change = SpecChange {
            section_id: "goals".into(),
            item_id: "G1".into(),
            title: Some("new goal".into()),
            content: None,
            status: None,
            reason: "proposed by agent".into(),
        };
        let updated = store.queue_spec_change(&s.id, change).unwrap().unwrap();
        assert_eq!(updated.pending_spec_changes.len(), 1);

        // reject it
        let after = store.reject_spec_change(&s.id, 0).unwrap().unwrap();
        assert!(after.pending_spec_changes.is_empty());
    }

    #[test]
    fn approve_applies_upsert_to_specs() {
        let (store, _f) = temp_store();
        let specs = tmp_specs();
        let s = store.create("superpowers", None).unwrap();
        let change = SpecChange {
            section_id: "goals".into(),
            item_id: "G1".into(),
            title: Some("approved goal".into()),
            content: Some("body".into()),
            status: None,
            reason: "agent proposal".into(),
        };
        store.queue_spec_change(&s.id, change).unwrap();

        // approve → applies upsert to specs
        let (applied, session) = store.approve_spec_change(&s.id, 0, &specs).unwrap().unwrap();
        assert_eq!(applied.item_id, "G1");
        assert!(session.pending_spec_changes.is_empty());

        // verify it landed in the spec doc
        let doc = specs.load().unwrap();
        let goals = doc.sections.iter().find(|x| x.id == "goals").unwrap();
        assert_eq!(goals.items.len(), 1);
        assert_eq!(goals.items[0].id, "G1");
        assert_eq!(goals.items[0].title, "approved goal");
    }

    #[test]
    fn approve_applies_status_transition() {
        let (store, _f) = temp_store();
        let specs = tmp_specs();
        let s = store.create("superpowers", None).unwrap();
        // seed a goal at Empty first
        let mut doc = specs.load().unwrap();
        specs.upsert_item(&mut doc, "goals", crate::specs::SpecItem::new("G1", "g")).unwrap();
        specs.save(&doc).unwrap();

        // queue a status change Empty -> Proposed (legal for Goals)
        let change = SpecChange {
            section_id: "goals".into(),
            item_id: "G1".into(),
            title: None,
            content: None,
            status: Some(crate::specs::SpecStatus::Proposed),
            reason: "advance".into(),
        };
        store.queue_spec_change(&s.id, change).unwrap();
        store.approve_spec_change(&s.id, 0, &specs).unwrap();

        let doc = specs.load().unwrap();
        let g = doc.sections.iter().find(|x| x.id == "goals").unwrap();
        assert_eq!(g.items[0].status, crate::specs::SpecStatus::Proposed);
    }

    #[test]
    fn reject_all_clears_queue() {
        let (store, _f) = temp_store();
        let s = store.create("superpowers", None).unwrap();
        for i in 0..3 {
            store.queue_spec_change(&s.id, SpecChange {
                section_id: "goals".into(),
                item_id: format!("G{i}"),
                title: None, content: None, status: None, reason: "x".into(),
            }).unwrap();
        }
        let after = store.reject_all_spec_changes(&s.id).unwrap().unwrap();
        assert!(after.pending_spec_changes.is_empty());
    }

    // ── PLAN-043: 会话树投影 ─────────────────────────────────────

    #[test]
    fn append_chains_parent_and_advances_leaf() {
        let mut s = ChatSession::new("superpowers", None);
        s.append(ChatMessage::user("q1"));
        s.append(ChatMessage::assistant("a1"));
        let m1 = &s.messages[1];
        assert_eq!(m1.parent_id.as_deref(), Some(s.messages[0].id.as_str()));
        assert_eq!(s.active_leaf.as_deref(), Some(m1.id.as_str()));
    }

    #[test]
    fn active_path_legacy_linear_fallback() {
        // 旧数据:无 parent/active_leaf 字段(jsonl 缺字段 → default None)。
        let mut s = ChatSession::new("superpowers", None);
        for c in ["q1", "a1", "q2"] {
            s.messages.push(ChatMessage::user(c)); // 直接 push,不经 append(模拟旧数据)
        }
        s.active_leaf = None;
        assert_eq!(s.active_path().len(), 3, "旧数据退化为线性全量");
    }

    #[test]
    fn fork_two_branches_independent_paths() {
        let mut s = ChatSession::new("superpowers", None);
        s.append(ChatMessage::user("q1"));
        s.append(ChatMessage::assistant("a1"));
        let branch_point = s.messages[0].id.clone(); // fork 自 q1 之后
        // 分支 A:从 branch_point 续聊
        assert!(s.set_active_leaf(&branch_point));
        s.append(ChatMessage::assistant("branch-A"));
        let path_a: Vec<String> = s.active_path().iter().map(|m| m.content.clone()).collect();
        assert_eq!(path_a, vec!["q1", "branch-A"]);

        // 分支 B:切回同一分叉点再续
        assert!(s.set_active_leaf(&branch_point));
        s.append(ChatMessage::assistant("branch-B"));
        let path_b: Vec<String> = s.active_path().iter().map(|m| m.content.clone()).collect();
        assert_eq!(path_b, vec!["q1", "branch-B"], "两分支互不污染");
        assert_eq!(s.messages.len(), 4, "append-only:全历史保留");

        // 公共前缀只出现一次
        assert_eq!(path_b.iter().filter(|c| **c == "q1").count(), 1);
        // set_active_leaf 对不存在消息返回 false
        assert!(!s.set_active_leaf("no-such"));
    }

    #[test]
    fn history_pairs_excludes_trailing_user_on_path() {
        let mut s = ChatSession::new("superpowers", None);
        s.append(ChatMessage::user("q1"));
        s.append(ChatMessage::assistant("a1"));
        s.append(ChatMessage::user("q2"));
        let pairs = s.history_pairs();
        assert_eq!(
            pairs,
            vec![("user".to_string(), "q1".to_string()), ("assistant".to_string(), "a1".to_string())],
            "路径末尾的 user(即将运行)被排除"
        );
        // 分叉后:另一分支的消息不进历史
        let bp = s.messages[1].id.clone();
        s.set_active_leaf(&bp);
        s.append(ChatMessage::user("q2-prime"));
        s.append(ChatMessage::assistant("a2-prime"));
        // 逆序首个 user(q2-prime)被跳过(与原线性构建同款语义);
        // q2(旧分支的消息)不在路径上,不进历史。
        let pairs = s.history_pairs();
        let contents: Vec<&str> = pairs.iter().map(|(_, c)| c.as_str()).collect();
        assert_eq!(contents, vec!["q1", "a1", "a2-prime"]);
        assert!(!contents.contains(&"q2"), "旧分支消息不进历史");
    }

    #[test]
    fn tree_nodes_mark_children_and_active_path() {
        let mut s = ChatSession::new("superpowers", None);
        s.append(ChatMessage::user("q1"));
        s.append(ChatMessage::assistant("a1"));
        let bp = s.messages[0].id.clone();
        s.set_active_leaf(&bp);
        s.append(ChatMessage::assistant("a1-B"));
        let nodes = s.tree_nodes();
        let by_content = |c: &str| nodes.iter().find(|n| n["preview"] == c).unwrap().clone();
        assert_eq!(by_content("q1")["children"], 2, "分叉点两个子分支");
        assert_eq!(by_content("a1")["on_active_path"], false);
        assert_eq!(by_content("a1-B")["on_active_path"], true);
    }

    #[test]
    fn store_set_active_leaf_persists() {
        let (store, _f) = temp_store();
        let s = store.create("superpowers", None).unwrap();
        store.append_message(&s.id, ChatMessage::user("q1")).unwrap().unwrap();
        let sess = store.get(&s.id).unwrap();
        let mid = sess.messages[0].id.clone();
        let updated = store.set_active_leaf(&s.id, &mid).unwrap().unwrap();
        assert_eq!(updated.active_leaf.as_deref(), Some(mid.as_str()));
        // 未知消息 → None
        assert!(store.set_active_leaf(&s.id, "nope").unwrap().is_none());
        // 重载持久化
        let reloaded = store.get(&s.id).unwrap();
        assert_eq!(reloaded.active_leaf.as_deref(), Some(mid.as_str()));
    }
}