# 007 — 统一工作流架构：Chat ↔ Flow 融合

> **状态**：架构设计文档，待审批后进入实施。
> **日期**：2026-07-01
> **仓库**：auto-musk（`backend/crates/musk/` + `web/`）
> **依赖**：Design 006（workspace）已落地；auto-ai-agent Assistant profession 待添加（见附录 B）
> **影响面**：后端 chats + relay 两子系统融合；前端 ChatsView + RelayView 合一

---

## 0. 问题陈述

auto-musk（继承自 auto-forge）把"聊天"和"接力"分成两个独立子系统，各有独立的：

- **数据模型**：ChatSession/ChatMessage vs RunEntry/RunEvent/HandoffDocument
- **ID 空间**：chat_id vs run_id
- **存储**：chats.json vs relay/{run_id}/run.json
- **API**：/api/chats/session/{id}/* vs /api/forge/relay/runs/{id}/*
- **前端**：ChatsView vs RelayView（两个独立标签页）
- **SSE 事件**：StreamEvent（4 种）vs RunEvent（15 种）
- **驱动**：chat_stream handler vs relay driver

但实际上，**chat 和 relay 本质上都是"有结构的对话流"**：

| 维度 | Chat | Relay | 本质 |
|---|---|---|---|
| 驱动者 | 人类用户 | FlowSpec 配置 | 对话参与者之一 |
| 参与者 | user ↔ assistant | advisor→architect→coder→... | 任何 agent（含 human-as-agent） |
| 消息传递 | chat messages | handoff documents | 对话回合 |
| 工具调用 | inline | inline + dispatch/bring_in | inline + 子对话 |
| 审批 | 隐式 | 显式 gate | gate turn |
| 嵌套 | 无 | 无 | 对话子树（errand/sub-flow） |

两套系统重复实现了消息传递、工具执行、事件流、持久化、token 追踪等核心逻辑。

**目标**：统一为一个 **Conversation 模型**，chat 和 flow 成为同一种抽象的两个实例，共享数据模型、存储、API 和 UI，支持嵌套子对话。

---

## 1. 统一数据模型

### 1.1 Conversation（统一抽象）

```rust
/// 一段对话或工作流——chat 和 flow 的共同祖先。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    /// 统一 ID 空间（替代 chat_id + run_id）。
    pub id: String,
    /// 嵌套关系：errand/sub-flow 指向父对话。顶层对话 parent_id = None。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    /// 父对话中触发本对话的那个 Turn 的 id（即 spawn_relay/dispatch 的 tool call turn）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_turn_id: Option<String>,
    /// 对话类型：自由对话 / 流程驱动 / 差遣。
    pub kind: ConversationKind,
    /// 归属 workspace。
    pub workspace_id: String,
    /// 驱动者：人类 / 单 agent / flow 配置。
    pub driver: Driver,
    /// 对话状态。
    pub status: ConversationStatus,
    /// 本对话的消息序列（只读快照 + 可追加）。
    #[serde(default)]
    pub turns: Vec<Turn>,
    /// 对话标题（用于列表展示）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// 原始任务/首条用户消息（用于列表预览）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_prompt: Option<String>,
    /// 使用的 mode（chat）或 flow_id（flow），用于构建 agent。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    /// 对应的 flow 配置（仅 kind=Flow 时有值）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flow: Option<FlowSpec>,
    /// 当前正在执行的 flow step 索引（仅 kind=Flow）。
    #[serde(default)]
    pub current_step: usize,
    /// 累计 token 消耗。
    #[serde(default)]
    pub cumulative_tokens: u64,
    /// Token 预算上限（None = 无限）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub budget_limit: Option<u64>,
    /// 当前等待审批的 gate（仅 status=WaitingGate 时有值）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pending_gate: Option<GateInfo>,
    /// 创建/更新时间戳（秒级，与现有系统一致）。
    pub created_at: u64,
    pub updated_at: u64,
}
```

### 1.2 ConversationKind

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversationKind {
    /// 人类驱动的自由对话（原 Chat）。
    Chat,
    /// Flow 配置驱动的多 agent 接力（原 Relay Run）。
    Flow,
    /// 差遣子任务（原 errand/dispatch，某个 agent 派给 gofer 等的短任务）。
    Errand,
}
```

### 1.3 Driver

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Driver {
    /// 人类用户驱动（等待用户输入推进）。
    Human,
    /// 单个 agent 驱动（errand：某 agent 发起，gofer 执行）。
    Agent { agent_id: String },
    /// Flow 配置驱动（按 FlowSpec 步骤序列自动推进）。
    Flow { flow_id: String },
}
```

### 1.4 ConversationStatus

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversationStatus {
    /// 活跃中（等待用户输入或 agent 正在工作）。
    Active,
    /// 等待人类审批（原 WaitingForHuman gate）。
    WaitingGate,
    /// 已完成。
    Completed,
    /// 失败。
    Failed { error: String },
    /// 暂停（非 gate 的显式暂停，如 loop 达上限）。
    Paused { reason: String },
}
```

### 1.5 Turn（统一消息）

这是统一的核心——chat message 和 relay event 都映射到 Turn：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Turn {
    /// 唯一 id。
    pub id: String,
    /// 在本对话中的序号（0-based）。
    pub seq: usize,
    /// 消息发送者："human" | agent_id（"assistant" / "coder" / "gofer" / ...）。
    pub from: String,
    /// 目标 agent（handoff/dispatch 时有值；user→assistant 消息的 to="assistant"）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    /// 回合类型。
    pub kind: TurnKind,
    /// 主要文本内容（消息正文 / handoff 摘要 / 系统通知 / 错误信息）。
    #[serde(default)]
    pub content: String,
    /// 工具调用记录（kind=ToolCall/ToolResult 时有值）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool: Option<ToolRecord>,
    /// Gate 审批信息（kind=Gate 时有值）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gate: Option<GateRecord>,
    /// 如果这个 turn 触发了子对话（spawn_relay/dispatch），记录子对话 id。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub child_conversation: Option<String>,
    /// 本 turn 的 token 消耗（如有）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens: Option<u64>,
    /// 时间戳（秒级）。
    pub timestamp: u64,
}
```

### 1.6 TurnKind

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TurnKind {
    /// 普通消息（用户发言 / agent 回复 / handoff 摘要）。
    Message,
    /// 工具调用（agent 调了 read_file / write_file 等）。
    ToolCall,
    /// 工具结果（工具执行后的返回值）。
    ToolResult,
    /// Gate 审批事件（系统暂停等待 / 人类做了决定）。
    Gate,
    /// 系统通知（flow 开始 / 步骤切换 / 预算警告 / 完成等）。
    System,
}
```

### 1.7 ToolRecord / GateRecord

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRecord {
    pub name: String,              // "read_file" / "write_file" / "spawn_relay" / ...
    pub args: serde_json::Value,   // 工具参数
    #[serde(default)]
    pub result: String,            // 工具返回值（ToolResult 时填）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_id: Option<String>,   // 工具调用 id（用于配对 call→result）
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateRecord {
    pub step_id: String,           // 触发 gate 的 flow step
    /// "waiting" | "approved" | "rejected" | "edit"
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feedback: Option<String>,  // reject/edit 时的反馈
}

/// Conversation.pending_gate 的载体（比 GateRecord 多 profession_id + since）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateInfo {
    pub step_id: String,
    pub profession_id: String,
    pub since: u64,
}
```

---

## 2. 两种驱动方式的统一映射

### 2.1 原有数据 → Turn 的映射

| 原有类型 | → Turn 字段映射 |
|---|---|
| ChatMessage(role=user) | Turn{from:"human", to:"assistant", kind:Message, content} |
| ChatMessage(role=assistant) | Turn{from:"assistant", to:"human", kind:Message, content} |
| ChatMessage(role=assistant).tool_calls | 每个工具调用拆成一对 Turn: ToolCall + ToolResult |
| ChatMessage(role=tool) | Turn{kind:ToolResult, tool:{result}} |
| RunEvent::StepStarted | Turn{kind:System, content:"step {step_id} started ({profession_id})"} |
| RunEvent::TurnDelta | 合并到当前 agent 的 Message turn 的 content（流式追加） |
| RunEvent::TurnToolCall | Turn{kind:ToolCall, from:profession_id, tool:{name,args}} |
| RunEvent::TurnToolResult | Turn{kind:ToolResult, tool:{result}} |
| RunEvent::TurnComplete | Turn{kind:Message, from:profession_id, content: 累积输出} |
| RunEvent::GateWaiting | Turn{kind:Gate, gate:{step_id,status:"waiting"}} |
| RunEvent::GateResolved | Turn{kind:Gate, from:"human", gate:{status:decision}} |
| RunEvent::StepCompleted | Turn{kind:System, content:"step {step_id} completed"} |
| HandoffDocument | Turn{from, to, kind:Message, content:summary} + 子 Turn（decisions/spec_updates 等作为结构化附件，序列化到 content 或扩展字段） |
| RunEvent::TokenSpend | 合并到 Turn.tokens 字段（最近的 turn） |

### 2.2 原有会话 → Conversation 的映射

| 原有类型 | → Conversation 字段映射 |
|---|---|
| ChatSession | Conversation{kind:Chat, driver:Human, mode:session.mode, turns:messages→Turn[]} |
| RunEntry | Conversation{kind:Flow, driver:Flow{flow_id}, flow:engine.flow, current_step, turns:events→Turn[]} |
| RunMetadata.title | Conversation.title |
| RunMetadata.initial_task | Conversation.initial_prompt |
| RunMetadata.originating_chat_session | Conversation.parent_id（flow 是 chat 的子对话） |
| RunMetadata.workspace_id | Conversation.workspace_id |
| errand（未来） | Conversation{kind:Errand, driver:Agent{coder}, parent_id:父flow的id} |

### 2.3 嵌套关系图

```
Conversation: chat-001 (kind=Chat, driver=Human)
├─ Turn 0: human → assistant "帮我实现认证模块"
├─ Turn 1: assistant → human "好的，我来启动工作流"
├─ Turn 2: assistant ToolCall(spawn_relay) → child_conversation="flow-aaa"
│   │
│   └─ Conversation: flow-aaa (kind=Flow, driver=Flow{default}, parent_id=chat-001)
│      ├─ Turn 0: System "flow started"
│      ├─ Turn 1: advisor → architect "设计 JWT 方案"
│      ├─ Turn 2: architect ToolCall(write_file)
│      ├─ Turn 3: architect ToolResult
│      ├─ Turn 4: Gate {step:design, status:waiting}
│      ├─ Turn 5: human Gate {status:approved}     ← gate 审批传播
│      ├─ Turn 6: architect → planner "移交规划"
│      ├─ Turn 7: planner → coder "开始实现"
│      ├─ Turn 8: coder ToolCall(dispatch) → child_conversation="errand-bbb"
│      │   │
│      │   └─ Conversation: errand-bbb (kind=Errand, driver=Agent{gofer}, parent_id=flow-aaa)
│      │      ├─ Turn 0: coder → gofer "找到所有 TODO 注释"
│      │      ├─ Turn 1: gofer → coder "找到了 5 个文件"
│      │      └─ Turn 2: System "errand completed"
│      │
│      ├─ Turn 9: coder → tester "实现完成，移交测试"
│      └─ Turn 10: System "flow completed"
│
├─ Turn 3: assistant → human "认证模块已完成！"   ← flow 完成后回到顶层
```

---

## 3. 统一存储

### 3.1 存储布局

```
{root}/.autoos/
  └─ conversations/
      ├─ index.json                       # 所有对话的轻量索引
      ├─ {conv-id}/
      │   └─ turns.jsonl                  # 每个对话一个文件，Turn 逐行追加
      └─ {conv-id}/
          └─ turns.jsonl
```

### 3.2 index.json

```jsonc
{
  "conversations": [
    {
      "id": "chat-001",
      "kind": "chat",
      "parent_id": null,
      "workspace_id": "musk-demo",
      "status": "active",
      "title": "帮我实现认证模块",
      "turn_count": 4,
      "child_count": 1,                  // 有 1 个子对话（flow-aaa）
      "cumulative_tokens": 50000,        // 含所有子对话的总 token
      "created_at": 1782800000,
      "updated_at": 1782803600
    }
  ]
}
```

### 3.3 turns.jsonl（每个对话一个文件）

每行一个 Turn JSON，追加写（append-only），天然适合 SSE 流式推送（后端 tail 文件或内存增量推）：

```jsonl
{"id":"t0","seq":0,"from":"human","to":"assistant","kind":"message","content":"帮我实现认证模块","timestamp":1782800000}
{"id":"t1","seq":1,"from":"assistant","to":"human","kind":"message","content":"好的，我来启动工作流","timestamp":1782800010}
{"id":"t2","seq":2,"from":"assistant","kind":"tool_call","tool":{"name":"spawn_relay","args":{"flow_id":"default","task":"实现认证模块"}},"child_conversation":"flow-aaa","timestamp":1782800012}
```

### 3.4 与旧存储的对比

| 旧 | 新 |
|---|---|
| chats.json（整个 workspace 的所有 session） | conversations/{conv-id}/turns.jsonl（每个对话独立文件） |
| relay/{run-id}/run.json | conversations/{conv-id}/turns.jsonl（同构） |
| 两个 ID 空间（chat_id + run_id） | 统一 ID 空间（conv-id） |
| 两套持久化逻辑 | 一套（ConversationStore） |

---

## 4. 统一 API

### 4.1 统一端点设计

所有对话操作走同一组端点，`{id}` 是统一的 conversation id：

| 端点 | 功能 | 替代 |
|---|---|---|
| `GET /api/conversations?workspace=<ws>` | 列出所有对话（可按 kind 过滤） | chat_list + relay list_runs |
| `POST /api/conversations` | 创建新对话（kind=Chat 或 Flow） | chat_create + relay start_run |
| `GET /api/conversations/{id}` | 获取对话详情（含 turns） | chat_get + relay get_run |
| `DELETE /api/conversations/{id}` | 删除对话 | chat_delete + relay delete_run |
| `PATCH /api/conversations/{id}/title` | 重命名 | chat_rename + relay update_title |
| `POST /api/conversations/{id}/message` | 追加一条用户消息（驱动对话推进） | chat_message |
| `POST /api/conversations/{id}/advance` | 推进 flow 对话一步 | relay advance_run |
| `POST /api/conversations/{id}/gate` | 解决 gate | relay resolve_gate |
| `GET /api/conversations/{id}/stream` | SSE 流（实时 turns） | chat_stream + relay run_events |
| `GET /api/conversations/{id}/children` | 列出子对话（errand/sub-flow） | 无（新能力） |
| `GET /api/conversations/{id}/tree` | 获取整棵对话树（含所有后代） | 无（新能力，用于分析） |

### 4.2 统一 SSE 事件

一套事件格式，替代 StreamEvent（4 种）+ RunEvent（15 种）：

```jsonl
{"type":"turn","conversation_id":"chat-001","turn":{...Turn...}}
{"type":"status","conversation_id":"chat-001","status":"active"}
{"type":"child_started","conversation_id":"chat-001","child_id":"flow-aaa","turn_id":"t2"}
{"type":"gate","conversation_id":"flow-aaa","gate":{"step_id":"design","status":"waiting"}}
{"type":"completed","conversation_id":"flow-aaa"}
```

核心事件类型：`turn`（新 turn 追加）、`status`（状态变更）、`child_started`（子对话开始）、`child_completed`（子对话完成）、`gate`（gate 变化）、`completed`、`failed`。

流式增量（原 turn_delta）通过 `turn` 事件携带**相同 turn id 但 content 增长**的 Turn 实现——前端按 turn id 累积拼接。

### 4.3 旧端点兼容期

迁移期间保留旧端点（`/api/chats/*` + `/api/forge/relay/*`），内部适配到新的 ConversationStore。Phase 2 结束后标记废弃，Phase 3 删除。

---

## 5. 统一 UI

### 5.1 标签页合并

当前 4 标签 → 3 标签：

| 当前 | 统一后 |
|---|---|
| 💬 聊天（ChatsView） | 💬 **对话**（ConversationView）— 合并 chat + flow |
| 🌀 流水线（RelayView） | *（合并进对话标签）* |
| 📜 规范（SpecsView） | 📜 规范（不变） |
| 📚 知识库（WikiView） | 📚 知识库（不变） |

### 5.2 对话视图：嵌套 box 模型

```
┌──────────────────────────────────────────────────┐
│  对话列表             │  对话内容                  │
│  ────────────────     │                            │
│  💬 认证模块   ●active │  [user] 帮我实现认证模块   │
│  💬 hello world  done  │                            │
│  💬 fix bug #42  active│  [assistant] 好的，我启动  │
│                        │  工作流。                  │
│                        │                            │
│                        │  ┌─ 🔧 user-auth ────────┐ │
│                        │  │ advisor→architect     │ │
│                        │  │ 设计了 JWT 方案        │ │
│                        │  │                        │ │
│                        │  │ ⏸️ 等待审批 [批准][拒绝]│ │
│                        │  │                        │ │
│                        │  │ [展开详情 ▾]           │ │
│                        │  └────────────────────────┘ │
│                        │                            │
│                        │  [消息输入框____________]   │
└──────────────────────────────────────────────────┘
```

**Flow box**（工作流折叠块）：
- 默认折叠，显示摘要（当前步骤 + 状态徽章）
- 展开后显示 agent 间对话流（advisor→architect→...）+ 工具调用 + gate
- 点开"详情"可进入全屏子对话视图（类似当前 RelayView，但是渲染为对话流而非步骤列表）

**Errand box**（子任务折叠块，嵌套在 Flow box 内）：
- 更小的折叠块，显示 "coder → gofer: 找到 TODO" → "gofer: 找到 5 个"
- 默认折叠，点击展开

**Gate**：
- Inline 审批按钮（批准/拒绝/编辑），不需要切到独立页面
- 审批操作直接在对话流里完成

### 5.3 前端事件处理统一

当前前端有两套 SSE 处理（useForge.ts 的 4 种事件 + useRelay.ts 的 15 种事件）。统一后一套 `useConversation.ts`：

```ts
// 统一 SSE 处理
function handleEvent(event: ConversationSSEEvent) {
  switch (event.type) {
    case 'turn':           // 新 turn（消息/工具/gate/系统）
      appendTurn(event.turn)
      break
    case 'status':         // 对话状态变更
      conversation.status = event.status
      break
    case 'child_started':  // 子对话开始 → 插入 Flow box
      insertChildBox(event.child_id, event.turn_id)
      break
    case 'child_completed':// 子对话完成 → 更新 box 状态
      updateChildBox(event.child_id, 'completed')
      break
    case 'gate':           // gate 变化
      updateGateState(event.gate)
      break
  }
}
```

---

## 6. 统一引擎

### 6.1 统一 ConversationEngine

当前有两个驱动入口：
- `chat_stream`：人类消息 → 构建 agent → `agent.run_stream` → 流式回传
- `relay/driver.rs::drive_run`：flow step → 构建 agent → `agent.run_stream` → submit_handoff → 循环

统一为一个 `ConversationEngine`，根据 `driver` 字段选择策略：

```rust
pub struct ConversationEngine {
    store: Arc<ConversationStore>,
}

impl ConversationEngine {
    /// 驱动一个对话前进。根据 driver 类型自动选择策略：
    /// - Human: 等待用户消息，然后构建 agent 执行一个 turn
    /// - Flow:  按 FlowSpec 步骤序列自动推进，直到 gate/完成/失败
    /// - Agent: errand 模式，单 agent 执行一次任务后完成
    pub async fn drive(&self, state: Arc<AppState>, conv_id: &str) {
        let conv = self.store.get(conv_id);
        match &conv.driver {
            Driver::Human => self.drive_chat(state, conv).await,
            Driver::Flow { flow_id } => self.drive_flow(state, conv).await,
            Driver::Agent { agent_id } => self.drive_errand(state, conv).await,
        }
    }
}
```

- `drive_chat`：从 chat_stream 逻辑提取——等待消息 → 构建 agent（按 mode）→ run_stream → turns 追加
- `drive_flow`：从 relay/driver.rs 提取——advance → 构建 agent（按 step profession）→ run_stream → submit_handoff → 循环
- `drive_errand`：未来实现——某个 agent dispatch 给 gofer → gofer 执行 → 结果回传

**三者的共同内核**：构建 agent + `agent.run_stream` + StreamEvent→Turn 转换 + token 记录。提取为 `run_agent_turn(agent, task, conv_id) -> Vec<Turn>` 共用函数。

### 6.2 tool_safety root 路由不变

`set_current_root` / `clear_current_root` 机制保持不变——engine 在驱动对话前设 root，驱动完清除。

---

## 7. 迁移策略（3 阶段）

### Phase 1：UI 统一（前端为主，后端不动）

**目标**：RelayView 内容搬进 ChatsView，用户不再切标签。后端 API 不变。

**改动**：
- ChatsView 增加"子对话 box"渲染能力：监听 chat 的 SSE，当出现 relay/errand 启动信号时插入 box
- box 内部用现有的 relay SSE（`/api/forge/relay/runs/{id}/events`）驱动内容
- RelayView 标签移除（或保留为"对话历史"入口）
- 4 标签 → 3 标签

**风险**：低（纯前端展示层改动，后端不变）。
**交付物**：用户在对话流里看到 flow 作为可展开 box，gate inline 审批。

### Phase 2：后端数据模型统一（Conversation + ConversationStore）

**目标**：引入 Conversation 模型，ChatSession 和 RunEntry 都映射为 Conversation。统一存储为 conversations/{id}/turns.jsonl。

**改动**：
- 新建 `conversation.rs`：Conversation / Turn / ConversationStore
- 旧数据迁移：chats.json → conversations/、relay/{run_id}/run.json → conversations/
- 新 API：`/api/conversations/*`，内部同时服务 chat + flow
- 旧 API（/api/chats/* + /api/forge/relay/*）适配层 → 内部调 ConversationStore
- 统一 SSE 事件格式
- ConversationEngine（合并 chat_stream + drive_run）

**风险**：中（改动面大，但旧 API 兼容期保证前端可渐进迁移）。
**交付物**：一套数据模型 + 一套 API + 一套存储 + 一套引擎。日志统一，可跨层分析。

### Phase 3：引擎统一 + 子对话（dispatch/errand 作为嵌套对话）

**目标**：dispatch/bring_in/errand 工具调用产生真正的子 Conversation（而非当前的 inline 执行）。

**改动**：
- dispatch 工具 → 创建 kind=Errand 的子 Conversation → drive_errand
- bring_in 工具 → 创建 kind=Chat 的子 Conversation（与人类的子对话）
- spawn_relay → 创建 kind=Flow 的子 Conversation
- 前端渲染嵌套 box 树
- 删除旧 API（/api/chats/* + /api/forge/relay/*）

**风险**：高（涉及编排工具实现，依赖 P2c）。
**前置依赖**：编排工具（bring_in/dispatch/spawn_relay）实现完成。
**交付物**：完整嵌套对话树，所有工作流统一为对话。

---

## 8. 迁移时的旧数据转换

### 8.1 chats.json → conversations/

```
ChatSession → Conversation {
    id: session.id,
    kind: Chat,
    driver: Human,
    mode: session.mode,
    turns: session.messages.iter().enumerate().map(|(i, msg)| msg_to_turn(msg, i)).collect(),
    ...
}
```

`msg_to_turn`：
- role=user → Turn{from:"human", kind:Message}
- role=assistant → Turn{from:"assistant", kind:Message}（tool_calls 拆成额外 ToolCall/ToolResult turns）
- role=tool → Turn{kind:ToolResult}

### 8.2 relay/{run_id}/run.json → conversations/

```
RunEntry → Conversation {
    id: run_id,
    kind: Flow,
    driver: Flow { flow_id: engine.flow.id },
    flow: engine.flow,
    current_step: engine.current_step,
    turns: events.iter().map(events_to_turns).flatten().collect(),
    ...
}
```

`events_to_turns`：见第 2.1 节的映射表。连续的 TurnDelta 合并成一个 Message turn。

### 8.3 迁移触发

首次启动 Phase 2 版本时，ConversationStore 初始化时检测旧格式：
- `{root}/.autoos/chats.json` 存在 → 转换每个 session
- `{root}/.autoos/relay/` 目录存在 → 转换每个 run
- 转换后旧文件重命名为 `.bak`（不立即删除）

---

## 9. 日志分析能力（统一后的价值）

### 9.1 完整工作流回放

```bash
# 获取一棵完整对话树（顶层 + 所有子对话）
GET /api/conversations/chat-001/tree

# 返回嵌套结构：
{
  "conversation": { "id":"chat-001", "kind":"chat", ... },
  "turns": [...],
  "children": [
    {
      "conversation": { "id":"flow-aaa", "kind":"flow", ... },
      "turns": [...],
      "children": [
        { "conversation": {"id":"errand-bbb", "kind":"errand"}, "turns": [...] }
      ]
    }
  ]
}
```

### 9.2 跨层统计

一个用户请求的总成本/时间/agent 数：

```jsonc
{
  "root": "chat-001",
  "total_conversations": 3,        // 1 chat + 1 flow + 1 errand
  "total_turns": 42,
  "total_tokens": 150000,          // 所有子对话加总
  "agents_involved": ["assistant","advisor","architect","planner","coder","tester","gofer"],
  "duration_secs": 1800,
  "gates_encountered": 1,
  "gates_resolved": 1
}
```

### 9.3 调试定位

某 errand 失败 → 直接查 `conversations/errand-bbb/turns.jsonl`，看 gofer 的完整对话，无需关联两个系统。

---

## 10. 范围与不做

### 本设计包含
- ✅ Conversation / Turn 统一数据模型
- ✅ 统一存储（conversations/{id}/turns.jsonl + index.json）
- ✅ 统一 API（/api/conversations/*）
- ✅ 统一 SSE 事件
- ✅ 统一 UI（嵌套 box 模型，3 标签）
- ✅ 统一引擎（ConversationEngine）
- ✅ 3 阶段迁移策略 + 旧数据转换规则
- ✅ 日志分析能力（tree/统计/调试）

### 不做（明确边界）
- ❌ Branch/Condition 路由 + StepValidator + ToolGuard（Plan 009 P2b.3，独立推进）
- ❌ checkpoint 回滚（独立推进）
- ❌ TaskPlan 多 flow 编排（独立推进）
- ❌ 用户作为"Agent"的正式抽象（human-as-agent 是概念模型，不建 Rust trait）
- ❌ Phase 3 的编排工具实现（依赖 P2c）

---

## 11. 风险与权衡

| 风险 | 应对 |
|---|---|
| **改动面巨大**（chats + relay 融合） | 3 阶段渐进式，Phase 1 纯前端零后端风险 |
| **HandoffDocument 丰富结构** vs **Turn 简洁** | HandoffDocument 序列化为 Turn 的结构化附件（content 存摘要，扩展字段存 decisions/spec_updates 等） |
| **长 flow 的 turns.jsonl 过大** | 单个 turn 轻量（~200 bytes），1000 turns ≈ 200KB；超出时考虑分片或 tool_result 截断 |
| **嵌套层级深时 UI 复杂** | 默认折叠，只展开用户关注的层；提供"全屏子对话"入口 |
| **旧 API 兼容期维护成本** | 适配层薄（转发到 ConversationStore），Phase 3 即删 |
| **Phase 2 期间 chat_stream + drive_run 并存** | 两者内部都调 ConversationEngine，过渡期短暂共存 |
| **流式 delta 合并成 Message turn** | 相同 turn_id 的 delta 按序追加 content；前端按 id 聚合 |

---

## 12. 验收标准（按阶段）

### Phase 1
1. 3 个标签（对话/规范/知识库），RelayView 移除
2. 在对话流中，assistant 触发 flow 时出现可展开 box
3. box 内展示 agent 间对话 + gate inline 审批
4. 点 box "详情"可进入全屏子对话视图

### Phase 2
5. 新 API `/api/conversations/*` 可创建/列出/获取/删除对话
6. 旧 chats.json + relay runs 自动迁移到 conversations/
7. 统一 SSE 推送 turn/status/gate/child 事件
8. chat 和 flow 共用同一套 turn 日志（一个文件）
9. 对话树 API 可返回嵌套结构
10. 旧 API 通过适配层继续工作

### Phase 3
11. dispatch 工具产生子 Conversation（嵌套对话树）
12. 前端渲染多层嵌套 box
13. 旧 API 删除，只保留 `/api/conversations/*`

---

## 附录 A：与现有代码的对应关系

| 现有文件 | 统一后 | 处理方式 |
|---|---|---|
| `chats.rs`（ChatSession/ChatMessage/ChatStore） | `conversation.rs`（Conversation/Turn/ConversationStore） | 融合，旧结构映射为新模型子集 |
| `relay/store.rs`（RunStore/RunEvent/RunState） | `conversation.rs`（同上） | RunEvent→Turn 映射，RunStore→ConversationStore |
| `relay/pipeline.rs`（PipelineEngine） | `conversation.rs`（ConversationEngine 的 flow 策略） | 引擎内部策略，PipelineStatus→ConversationStatus |
| `relay/driver.rs`（drive_run） | `conversation.rs`（ConversationEngine::drive_flow） | 提取为引擎方法 |
| `relay/handoff.rs`（HandoffDocument） | `conversation.rs`（Turn 的结构化附件） | handoff 成为 Message turn 的一种内容形式 |
| `relay/api.rs`（relay endpoints） | `server.rs`（/api/conversations/*） | 统一端点 |
| `server.rs` chat_stream handler | `conversation.rs`（ConversationEngine::drive_chat） | 提取为引擎方法 |
| `web/composables/useForge.ts` | `web/composables/useConversation.ts` | 合并 |
| `web/composables/useRelay.ts` | `web/composables/useConversation.ts` | 合并 |
| `web/views/ChatsView.vue` | `web/views/ConversationView.vue` | 扩展（加 flow box 渲染） |
| `web/views/RelayView.vue` | *（移除，能力并入 ConversationView）* | 删除 |

---

## 附录 B：前置依赖——auto-ai-agent Assistant Profession

Phase 1 的 UI 统一不依赖此项，但 Phase 2 的引擎统一需要 `assistant` 作为默认对话角色。

当前 auto-ai-agent 的 `load_builtin` 只有 7 个 profession（coder/architect/tester/reviewer/documenter/translator/runner），缺少 `assistant`。需要：

1. 新建 `crates/auto-ai-agent/resources/souls/assistant.md`（人格 Nicole，对话型助手 soul，去掉编排语义）
2. 新建 `crates/auto-ai-agent/src/professions/assistant.rs`（`impl Profession`，model_tier=Mid，max_turns=12，轻量只读工具集）
3. 在 `professions/mod.rs` 的 `load_builtin` + `builtin_names` 注册

详见前述"给 auto-ai 的 Agent 对话的实施计划"。

musk 侧的改动：`superpowers.at` 的 `profession` 从 `"coder"` 改为 `"assistant"`。
