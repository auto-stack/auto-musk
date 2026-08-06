# 019 — 接线运行深化：流式 handler 切换到 Auto 驱动

> **状态**：实施计划。Phase 0-4 待启动。
> **前置**：Plan 018（已归档，§11 接线运行 ①-④ 已完成，本计划是 §11 标记的"后续独立计划"）。
> **仓库**：auto-musk（`backend/crates/musk/`），纯 auto-musk 侧工作，不涉及 auto-lang。
> **目标**：把 serve() 的 6 个 🔴 流式/daemon handler 从手写 Rust 切换到 `auto_generated::server_stream` 的转译 handler，让整个服务端（除 settings_link + serve 外壳）由 Auto 驱动。

---

## 0. 背景

Plan 018 §11 完成了接线运行 ①-④（auth/specs/chats/workspace/config/conversations/app_config/harness/relay 等端点已切到 ag handler）。但 7 个 🔴 流式/daemon handler 因 async 泛型闭包 + SSE 管道被标为"硬墙"，留手写。

后来 `server_stream.at` 经 Plan 380 P1-dyn（Arc<dyn T>）+ Plan 321（async_stream ~Stream+yield）**重新评估为可移植**，已移植 6 个 handler（产物 `server_stream.rs` 能编译，已 mod 声明，但 serve() 没用——全是 dead code）。extern_impl 有 13 个 fake stub。

本计划把这 6 个 handler 真正接线（extern 真实化 + DTO 修正 + serve() 切换）。

**不切换**：`settings_link`（reqwest::blocking 无法转译，.at 未移植），保持手写。

---

## 1. 现状（2026-08-06 调研确认）

### server_stream.at 已移植的 handler（6 个）

| handler | .at 行 | SSE? | 接线难度 |
|---|---|---|---|
| workflow_run | 95 | 否 | 🟢 |
| run | 236 | 否 | 🟡 |
| conversation_stream | 203 | 是（axum::Sse） | 🟡 |
| workflow_run_stream | 107 | 是（mpsc+Sse） | 🟡 |
| run_stream_handler | 155 | 是（mpsc+Sse） | 🔴 |
| chat_stream | 250 | 是（mpsc+Sse） | 🔴 |

### extern_impl 的 fake stub（13 个需真实化）

| extern | 行 | fake 返回 | 真实化目标 |
|---|---|---|---|
| mpsc_channel/sender/receiver/try_send/recv | 998-1002 | Null/空 | tokio mpsc（side-table 类型擦除） |
| broadcast_recv | 1005 | None | broadcast::Receiver（side-table） |
| conversations_subscribe | 797 | Null | ws.conversations.subscribe() |
| conv_event_matches/id/turn/status | 798-801 | false/空 | ConversationEvent 字段提取 |
| workflow_event_map | 982 | StepSkipped | WorkflowStreamEvent→WorkflowEventDto |
| stream_event_map | 981 | Cancelled | StreamEvent→SseEventDto（含 id 配对） |
| wf_run | 868 | 空 HashMap | relay::feature_dev::run |
| wf_run_with_progress | 869 | 空 | relay::feature_dev::run_stream |
| agent_run | 970 | 空 RunResponse | build_agent_from_mode + agent.run |
| agent_run_stream | 972 | 空 | build_agent + agent.run_stream + sink 适配 |
| chat_run_stream | 971 | 空 | session/history/build + run_stream + 持久化 |

### 4 个 wire 形状回归点

1. `SseEventDto` 字段名 `tool/args` vs hw `name/arguments`（run_stream/chat_stream）
2. `WorkflowEventDto` 无 `#[serde(tag="type")]` vs hw `WorkflowStreamEvent` 有（workflow_run_stream）
3. `ConvEventDto.turn: Option<String>` vs hw 完整 Turn 结构（conversation_stream）
4. `RunResponse` 缺 `turns` 字段（run）

---

## 2. 实施策略：5 阶段，每阶段独立验收

### Phase 0：契约测试（金标准，切换前先写）

用 `tower::ServiceExt::oneshot`（先例 server.rs:1713）打 6 个 handler 的 HTTP 层。断言 status + Content-Type + SSE body 的事件类型/字段名。**切换前后都跑**，作为行为等价金标准。

4 个 wire 形状契约：
- SSE 事件字段名 `name`/`arguments` + `tc-{n}` id 配对
- WorkflowEvent `{"type":"step_start",...}` 格式
- ConversationEvent.turn 完整序列化
- RunResponse.turns 字段

**验收**：契约测试在 hw handler 上全绿。

### Phase 1：简单非流式 handler（workflow_run + run）

- DTO 修正：RunResponse 加 turns；WorkflowRunResponse 对齐 hw；handler 返回 `~Response`（4xx 支持）
- extern 真实化：wf_run → relay::feature_dev::run；agent_run → build_agent + agent.run
- serve() 切换：/api/workflow/run + /api/run
- **验收**：契约测试 + run_endpoint_returns_result 绿

### Phase 2：conversation_stream（SSE，axum::Sse wire 一致）

- side-table 基础设施：extern 建 `static HANDLE_REGISTRY: Mutex<HashMap<i64, Box<dyn Any + Send>>>`，broadcast::Receiver 存这里，Value 存 i64 id
- DTO 修正：ConvEventDto.turn 改完整 Turn 序列化
- extern 真实化：conversations_subscribe + conv_event_*
- serve() 切换：/api/conversations/{id}/stream
- **验收**：契约测试绿

### Phase 3：workflow_run_stream（SSE + mpsc + relay 事件流）

- mpsc side-table：mpsc_channel/sender/receiver/try_send/recv 走 side-table
- DTO 修正：WorkflowEventDto 加 `#[serde(tag="type", rename_all="snake_case")]`
- sink 桥接：wf_run_with_progress 建适配闭包（hw Arc<dyn Fn(WorkflowStreamEvent)> → Value → sink.on_event）
- serve() 切换：/api/workflow/run/stream
- **验收**：契约测试绿

### Phase 4：run_stream_handler + chat_stream（最复杂）

- extern 真实化 agent_run_stream：建 agent + sink 适配成 Arc<dyn Fn(StreamEvent)>，**闭包内复刻 id 配对**（tc_counter/tc_stack）+ stream_event_to_json（name/arguments 格式）
- DTO 修正：SseEventDto 字段名 + id/status/turns 字段
- chat_stream 持久化：chat_run_stream extern 内直接调 hw 持久化 API（session/history/build + run_stream 完成后 append_message + append_turn）
- serve() 切换：/api/run/stream + /api/chats/session/{id}/stream
- **验收**：契约测试绿（id 配对 + name/arguments + 持久化）

---

## 3. 关键架构决策

1. **side-table 方案**（类型擦除墙）：extern_impl 维护 `static HANDLE_REGISTRY: Mutex<HashMap<i64, Box<dyn Any + Send>>>`，mpsc/broadcast 的 tx/rx/Receiver 存这里，Value 只存 i64 id。不改 .at 产物类型。

2. **DTO 修正**：改 server_stream.at 的 DTO 定义对齐 hw wire 格式，重新转译。4 个回归点逐一修。

3. **sink 桥接**：extern 内建适配闭包，把 hw 强类型事件（StreamEvent/WorkflowStreamEvent）转成 Value 喂给 ag sink.on_event。id 配对 + stream_event_to_json 逻辑搬进 extern。

4. **chat_stream 持久化**：在 chat_run_stream extern 内直接做（它知道 session_id），不依赖 sink 回调。

5. **settings_link 不切换**（reqwest::blocking 无法转译）。

---

## 4. 验收标准

每个 Phase 闭环需：
1. 契约测试在切换前后都绿（行为等价金标准）。
2. 现有测试（server.rs 集成测试）不退化。
3. serve() 路由切换到 ag handler。
4. KNOWN-DEBT-AND-RISKS.md 更新。

最终目标：6 个 handler 全部切换，serve() 里只剩 settings_link + 静态文件/CORS/TcpListener 外壳是手写。

---

## 5. 风险

- 🔴 wire 形状回归（前端断）→ 契约测试金标准兜底
- 🔴 类型擦除墙 → side-table 方案
- 🟡 a2r 可能有新转译限制（DTO 修正后）→ 逐个处理，必要时开 auto-lang follow-up
- 🟡 SSE 传输格式差异（axum::Sse 多 event: 行）→ 前端验证
