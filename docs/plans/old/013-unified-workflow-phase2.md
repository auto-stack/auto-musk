# 013 — 后端数据模型统一 Phase 2: Conversation/Turn/ConversationStore

## Status: COMPLETE

> 🗄️ **已落地**（2026-08-04 核对代码）。`Conversation`/`Turn`/`ConversationKind`/`ConversationStatus`/`GateRecord` 等类型（conversation.rs:15-144）+ `ConversationStore`（:470，create/get/list/delete/rename/append_turn/set_status/set_gate/subscribe/migrate_chats）+ 挂入 `WorkspaceStores.conversations`（workspace.rs:44）+ RunStore 经 `link_conversations`（relay/store.rs:254）双写为 Flow 对话。统一 API `/api/conversations` + `/{id}` + `/{id}/stream` SSE（server.rs:150-156, :1905）；旧 `/api/chats/*` 经适配层（chat_create 双写同 id、chat_message 转 Turn）继续工作。4 子阶段（2a/2b/2c/2d）全部完成。

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax.

**Goal:** 引入统一的 Conversation/Turn 数据模型和 ConversationStore，把 chat 和 relay 的数据统一到一套模型 + 一套存储（conversations/{id}/turns.jsonl）。旧的 /api/chats + /api/forge/relay 端点通过适配层继续工作，前端无感。

**Architecture:** ConversationStore 替代 ChatStore + RunStore 的存储角色（但保留 PipelineEngine 用于 flow 状态机）。Turn 是统一消息格式，ChatMessage 和 RunEvent 都映射到它。存储用 jsonl 追加写（每对话一个文件）。新 API `/api/conversations/*` 同时服务 chat + flow。旧 API 适配层转发到 ConversationStore。

**Spec:** `designs/007-unified-workflow-architecture.md`（Phase 2 部分）

---

## 分阶段策略

Phase 2 改动面大，拆成 4 个子阶段，每个独立可交付：

- **2a**: Conversation/Turn 数据模型 + ConversationStore（纯新增，不碰旧代码）
- **2b**: 旧数据迁移（chats.json + relay runs → conversations/）+ 双写（新旧同步）
- **2c**: 旧 API 适配层（/api/chats + /api/forge/relay 内部改读 ConversationStore）
- **2d**: 统一 SSE + 新 API `/api/conversations/*`

---

## Task 2a-1: Conversation/Turn 数据模型

**Files:**
- Create: `backend/crates/musk/src/conversation.rs`
- Modify: `backend/crates/musk/src/lib.rs` (add `pub mod conversation;`)

### 数据模型（按 Design 007 §1）

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
pub enum ConversationKind { Chat, Flow, Errand }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Driver { Human, Agent { agent_id: String }, Flow { flow_id: String } }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversationStatus {
    Active, WaitingGate, Completed,
    Failed { error: String },
    Paused { reason: String },
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TurnKind { Message, ToolCall, ToolResult, Gate, System }

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
```

- [x] Step 1: 创建 conversation.rs（上述全部类型 + 从 chats.rs/relay store.rs 的转换函数）
- [x] Step 2: 加转换函数 `ChatMessage → Turn` 和 `RunEvent → Vec<Turn>`（按 Design 007 §2.1 映射表）
- [x] Step 3: 单测：转换正确性
- [x] Step 4: lib.rs 注册模块
- [x] Step 5: cargo test + commit

---

## Task 2a-2: ConversationStore

**Files:**
- Modify: `backend/crates/musk/src/conversation.rs`

ConversationStore 管理 conversations/{id}/turns.jsonl + index.json。

```rust
pub struct ConversationStore {
    dir: PathBuf,  // {root}/.autoos/conversations
    cache: std::sync::Mutex<HashMap<String, Conversation>>,
}
```

方法：
- `new(dir)` / `at(dir)`
- `create(kind, workspace_id, driver, ...) -> Conversation`
- `get(id) -> Option<Conversation>`
- `list(workspace_id) -> Vec<ConversationSummary>`
- `delete(id) -> bool`
- `rename(id, title) -> Option<Conversation>`
- `append_turn(id, turn) -> Option<Conversation>` (写 turns.jsonl + 更新 index)
- `set_status(id, status)`
- `set_gate(id, gate: Option<GateInfo>)`

存储：
- index.json: `Vec<ConversationSummary { id, kind, parent_id, workspace_id, status, title, turn_count, child_count, cumulative_tokens, created_at, updated_at }>`
- {id}/turns.jsonl: 每行一个 Turn JSON

- [x] Step 1-5: 实现 + 单测 + commit

---

## Task 2b: 旧数据迁移 + WorkspaceStores 接入

**Files:**
- Modify: `backend/crates/musk/src/workspace.rs`
- Modify: `backend/crates/musk/src/conversation.rs` (migration helper)

把 ConversationStore 加入 WorkspaceStores（第 5 个 store）。首次启动时迁移旧数据。

- `WorkspaceStores::new` 增加 `conversations: Arc<ConversationStore>`
- 迁移函数：检测旧 `chats.json` → 每个 ChatSession 转为 Conversation；检测旧 `relay/` → 每个 run 转为 Conversation
- 迁移后旧文件重命名 `.bak`

- [x] 实现 + 单测 + commit

---

## Task 2c: 旧 API 适配层

**Files:**
- Modify: `backend/crates/musk/src/server.rs` (chat handlers)
- Modify: `backend/crates/musk/src/relay/api.rs` (relay handlers)

旧的 chat/relay 端点内部改为读写 ConversationStore（同时保持旧响应格式不变，前端无感）。

核心改动：每个 chat handler 从 `ws.chats` 改为 `ws.conversations`，但响应仍返回旧格式（ChatSession/ChatSessionSummary）。relay 同理——从 `ws.relay` 改为 `ws.conversations`，响应仍返回 RunState/RunSummary。

**这是最大的改动面**（~40 个 handler 签名），但每个改动是机械的：旧格式 ↔ Conversation 互转。

- [x] 分批改造（先 chats list/get/create/delete，再 chats message/stream/approve/reject，再 relay 全部）
- [x] cargo test + commit

---

## Task 2d: 统一 SSE + 新 API

**Files:**
- Modify: `backend/crates/musk/src/conversation.rs` (event bus)
- Create or modify: `backend/crates/musk/src/conversation_api.rs`
- Modify: `backend/crates/musk/src/server.rs` (register new routes)

统一 SSE 广播（一套事件格式替代 StreamEvent 4 种 + RunEvent 15 种）。新 API `/api/conversations/*`。

- [x] 统一事件总线
- [x] 新端点
- [x] commit

---

**注意：** Phase 2 是大型工程，每个 Task 应拆成更细的 sub-task 用 subagent 执行。上面的 2a-1/2a-2/2b/2c/2d 是子阶段，每个可能需要多个 commit。
