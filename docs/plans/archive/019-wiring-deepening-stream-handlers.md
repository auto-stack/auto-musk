# 019 — 接线运行深化：流式 handler 切换到 Auto 驱动

> **状态**：✅ COMPLETE（2026-08-06 归档）。Phase 0-4 全闭环 + 状态码模型补齐 + §6.1 流式即时错误→400 + §6.2 broadcast 连接泄漏根治。6 个 handler 全部切到 ag，全量 228 lib 测试 + 集成测试全绿。§6.3 登记备忘（与 hw 等价非缺陷）。
> **归档复审**（plan-archiver Step 2.5）：切换后暴露 3 项限制——§6.1（流式 handler 即时错误→400）与 §6.2（conversation_stream broadcast Receiver 连接泄漏）已随本计划闭环并配契约测试；§6.3（mpsc 缓冲 64 丢帧）与 hw 完全等价、非缺陷、仅登记备忘。均见 `KNOWN-DEBT-AND-RISKS.md` 🟢 已知限制表。无 🔴 高风险。
> **前置**：Plan 018（已归档，§11 接线运行 ①-④ 已完成，本计划是 §11 标记的"后续独立计划"）。
> **仓库**：auto-musk（`backend/crates/musk/`）。Phase 1a 重新转译用到 auto-lang worktree（构建 `auto.exe`）。
> **目标**：把 serve() 的 6 个 🔴 流式/daemon handler 从手写 Rust 切换到 `auto_generated::server_stream` 的转译 handler，让整个服务端（除 settings_link + serve 外壳）由 Auto 驱动。

## 实施进度（2026-08-06 更新）

- ✅ **Phase 0a**：`sse_event` 根因修复 —— extern_impl.rs 的 `sse_event` 去掉 `.event(name)`，产出 raw `data:` 帧（与 hw 一致）。**这是计划文档遗漏的第 5 个 wire 形状回归点**：前端只用 `EventSource.onmessage`，按 SSE 协议带 `event:` 行的消息不进 onmessage，ag 给每帧加 event 行会导致前端完全收不到流。根因不在 a2r/Auto 语言，在手写 extern 的 `sse_event` 实现。经 axum 0.8.9 源码验证。
- ✅ **Phase 0b**：契约金标准测试（7 项）—— `stream_event_to_json` wire 形状（name/arguments + tc-{n} id 配对 + turns）、`WorkflowStreamEvent` snake_case tag、`sse_event` 无 event 行、HTTP 层 status/Content-Type。
- ✅ **Phase 1a**：DTO 修正（`RunResponse.turns` + `ToolCallOut.args: Value`）+ **`server_stream.at` 全量 `.view` 借用标记补齐**（根因：移植时遗漏 `.view`，导致 extern 调用点未注入 `&`，之前靠手修产物）。重新转译零 drift（无手修）。
- ✅ **Phase 1b**：extern 真实化 `wf_run`（→ feature_dev::run）+ `agent_run`（→ build_agent_from_mode + agent.run）。
- ✅ **Phase 1c**：serve() 切换 `/api/run` + `/api/workflow/run` 到 ag handler。
- ✅ **Phase 1d**：ag handler 等价性测试（`ag_workflow_run_produces_real_steps_and_outputs` + `ag_run_produces_output_turns_tool_calls`）。
- ✅ **Phase 2**（conversation_stream）：`conversations_subscribe`（broadcast::Receiver 进 side-table）+ `broadcast_recv`（Lagged 跳过续流 / Closed 终止）+ `conv_event_*` 字段提取；`ConvEventDto.turn` → `Option<Value>`（完整 Turn 序列化，与 hw 一致）。
- ✅ **Phase 3**（workflow_run_stream）：mpsc side-table（`mpsc_channel`/`sender`/`receiver`/`try_send`/`recv`，Value 存 i64 id / `{"pair": id}`）；`WorkflowEventDto` 加 `#[serde(tag="type", rename_all="snake_case")]` + Deserialize（`workflow_event_map` 经 `from_value` 无损回读）；`wf_run_with_progress` → `feature_dev::run_stream` + 事件喂 mpsc。
- ✅ **Phase 4**（run_stream_handler + chat_stream）：`agent_run_stream` → build_agent + `agent.run_stream` + 闭包内复刻 **tc_counter/tc_stack id 配对** + `SseEventDto`（变体改名 `ToolCall`/`ToolResult` 得 wire 的 `tool_call`/`tool_result`，字段直接命名 `name`/`arguments`，补 `Thinking`/`status`/`Cancelled` 字段）；`chat_run_stream` → session/history/build_agent_with_context + run_stream + 完成后 `append_message` + 双写 conversation turns。
- ✅ **状态码模型**（KNOWN-DEBT 019 项）：`run`/`workflow_run` 从 `~Json<T>` 改 `~Response` + 错误包络（`wf_run`/`agent_run` 返回 `{"error":{"code","message"}}`，handler 经 `resp_is_err`/`resp_err_message`/`resp_err_code` → `err_response`）。坏 mode / 坏 workflow → 400，build/run 失败 → 500，与 hw 等价。
- ✅ **架构调整**：mpsc channel 由 extern side-table 持有，handler 把 **tx 句柄直传 extern**（去掉 named sink struct 中转）。extern 在 run 结束后 `close_channel`（移除 pair → 唯一 Sender 析构 → channel 关闭），SSE 流在 `mpsc_recv` 得 None 时 `break` 终止——与 hw 的 `while let Some(v) = rx.recv().await` 终止语义一致，避免流永不结束。
- ✅ **验收**：6 个 ag handler 契约/等价性测试（`ag_run_stream_produces_sse_events` / `ag_workflow_run_stream_emits_step_events` / `ag_chat_stream_persists_and_streams` / `ag_conversation_stream_filters_events` / `ag_run_unknown_mode_returns_400` / `ag_workflow_run_invalid_workflow_returns_400`）+ 全量 222 lib 测试 + 集成测试全绿。
- ✅ **§6.1 流式即时错误→400**（2026-08-06）：`run_stream_handler`/`workflow_run_stream` 在 `mpsc_channel()` 前前置 `mode_exists`/`workflow_exists` 校验（与 hw fail-fast 一致），坏 spec → 400 JSON（不再 200 SSE + error 帧）。契约测试 `ag_run_stream_bad_mode_returns_400` + `ag_workflow_run_stream_invalid_workflow_returns_400`。全量 226 lib 测试全绿。
- ✅ **§6.2 broadcast 连接泄漏根治**（2026-08-06）：`conversations_subscribe` 返回 `BroadcastSub`（rx 包 `Arc<Mutex<Option<Receiver>>>`），被 `conv_event_stream` owned → stream drop（客户端断开）→ Arc 归零 → rx 析构。不再用 registry 存 receiver。`conversation_id` 存进 sub 避免 `&str` 流参数（E0700）。测试 `broadcast_sub_drop_reclaims_receiver` + `conversation_stream_drop_reclaims_receiver` 断言 receiver_count 回落。

---

## 0. 背景

Plan 018 §11 完成了接线运行 ①-④（auth/specs/chats/workspace/config/conversations/app_config/harness/relay 等端点已切到 ag handler）。但 7 个 🔴 流式/daemon handler 因 async 泛型闭包 + SSE 管道被标为"硬墙"，留手写。

后来 `server_stream.at` 经 Plan 380 P1-dyn（Arc<dyn T>）+ Plan 321（async_stream ~Stream+yield）**重新评估为可移植**，已移植 6 个 handler（产物 `server_stream.rs` 能编译，已 mod 声明，但 serve() 没用——全是 dead code）。extern_impl 有 13 个 fake stub。

本计划把这 6 个 handler 真正接线（extern 真实化 + DTO 修正 + serve() 切换）。

**不切换**：`settings_link`（reqwest::blocking 无法转译，.at 未移植），保持手写。

---

## 1. 现状（2026-08-06 调研确认）

### server_stream.at 已移植的 handler（6 个，全部已接线）

| handler | .at 行 | SSE? | 接线难度 |
|---|---|---|---|
| workflow_run | 95 | 否 | 🟢 |
| run | 236 | 否 | 🟡 |
| conversation_stream | 203 | 是（axum::Sse） | 🟡 |
| workflow_run_stream | 107 | 是（mpsc+Sse） | 🟡 |
| run_stream_handler | 155 | 是（mpsc+Sse） | 🔴 |
| chat_stream | 250 | 是（mpsc+Sse） | 🔴 |

### extern_impl 的 fake stub（13 个，Phase 2-4 已全部真实化 ✅）

| extern | 真实化状态 |
|---|---|
| mpsc_channel/sender/receiver/try_send/recv | ✅ tokio mpsc side-table（Value 存 i64 id / `{"pair": id}`，不 clone Sender） |
| broadcast_recv | ✅ broadcast::Receiver side-table（Lagged 跳过续流，Closed → None 终止） |
| conversations_subscribe | ✅ ws.conversations.subscribe() → side-table |
| conv_event_matches/id/turn/status | ✅ ConversationEvent 序列化 Value 字段提取 |
| workflow_event_map | ✅ `from_value::<WorkflowEventDto>`（无损回读） |
| stream_event_map | ✅ `from_value::<SseEventDto>`（无损回读） |
| wf_run | ✅ feature_dev::run + 错误包络（400/500） |
| wf_run_with_progress | ✅ feature_dev::run_stream + 事件喂 mpsc |
| agent_run | ✅ build_agent_from_mode + agent.run + 错误包络（400/500） |
| agent_run_stream | ✅ build_agent + agent.run_stream + tc id 配对 + DTO |
| chat_run_stream | ✅ session/history/build + run_stream + 持久化 |

### 4 个 wire 形状回归点（Phase 2-4 已全部修复 ✅）

1. `SseEventDto` 字段名 `tool/args` vs hw `name/arguments`（run_stream/chat_stream）——变体改名为 `ToolCall`/`ToolResult`（snake_case 得 `tool_call`/`tool_result`），字段直接命名 `name`/`arguments`；补 `Thinking` 变体 + `ToolResult.status` + `Cancelled` 载荷
2. `WorkflowEventDto` 无 `#[serde(tag="type")]` vs hw `WorkflowStreamEvent` 有（workflow_run_stream）——补 `#[serde(tag="type", rename_all="snake_case")]` + Deserialize
3. `ConvEventDto.turn: Option<String>` vs hw 完整 Turn 结构（conversation_stream）——改 `Option<Value>` 序列化完整 Turn
4. `RunResponse` 缺 `turns` 字段（run）——Phase 1a 已修

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

1. **side-table 方案**（类型擦除墙）：extern_impl 维护 `static HANDLES: LazyLock<Mutex<HashMap<i64, Box<dyn Any + Send>>>>`，mpsc 的 channel pair / receiver 与 broadcast::Receiver 存这里，Value 只存 i64 id（tx 句柄是 `{"pair": id}` 指针）。不改 .at 产物类型。

2. **DTO 修正**：改 server_stream.at 的 DTO 定义对齐 hw wire 格式，重新转译。4 个回归点逐一修。**转译器不支持 tag 变体字段级 serde 属性**——`SseEventDto` 通过改变体/字段命名（`ToolCall`/`ToolResult` + `name`/`arguments`）而非 rename 属性达成 wire 形状。

3. **tx 直传替代 sink 桥接**（实施时的工程修正）：handler 把 mpsc `tx` 句柄直接传给 extern（去掉 named sink struct 中转）。好处：① extern 在 run 结束后 `close_channel`（移除 pair → 唯一 Sender 析构 → channel 关闭）→ 流侧 `mpsc_recv` 得 None → `.at` 的 break 终止流，与 hw 的 `while let Some` 终止语义一致（避免流永不结束挂死前端）；② 无需把私有 sink 类型 `pub` 化 + `Arc<dyn StreamSink>` 强制转换。原计划"extern 把事件喂给 sink.on_event"的语义不变（事件仍经 DTO 无损回读喂进 mpsc）。

4. **chat_stream 持久化**：在 chat_run_stream extern 内直接做（它知道 session_id），不依赖 sink 回调。

5. **状态码模型**：`run`/`workflow_run` 改 `~Response` + 错误包络（extern 返回 `{"error":{"code","message"}}`，handler 经 `resp_is_err`/`resp_err_message`/`resp_err_code` 转 `err_response`），坏 mode/workflow → 400、build/run 失败 → 500，与 hw 等价。

6. **settings_link 不切换**（reqwest::blocking 无法转译）。

---

## 4. 验收标准

每个 Phase 闭环需：
1. 契约测试在切换前后都绿（行为等价金标准）。
2. 现有测试（server.rs 集成测试）不退化。
3. serve() 路由切换到 ag handler。
4. KNOWN-DEBT-AND-RISKS.md 更新。

最终目标：6 个 handler 全部切换，serve() 里只剩 settings_link + 静态文件/CORS/TcpListener 外壳是手写。

---

## 5. 风险（已消解）

- 🔴 wire 形状回归（前端断）→ 契约测试金标准兜底 ✅（4 个回归点全部修复 + ag 流式测试断言 wire 形状）
- 🔴 类型擦除墙 → side-table 方案 ✅
- 🔴 流终止/挂死 → 实施时引入 tx 直传 + close_channel + .at break（测试断言流可终止）
- 🟡 a2r 转译限制（DTO 修正后）→ 发现"tag 变体字段级 serde 属性"不支持，改命名绕行；`break`/`~Response` 均验证可转译
- 🟡 SSE 传输格式差异（axum::Sse 多 event: 行）→ Phase 0a 根因修复（sse_event 去 `.event(name)`）✅

---

## 6. 后续修复方向（§6.1 + §6.2 已闭环；§6.3 登记备忘）

以下 3 条为切换后暴露的限制，均已在 `KNOWN-DEBT-AND-RISKS.md` 🟢 已知限制表登记。

### 6.1 ✅ 已闭环：ag 流式 handler 即时错误 → 400（2026-08-06）

- **现状（已修复）**：`run_stream_handler`/`workflow_run_stream` 现在在 `mpsc_channel()` **之前**前置 `mode_exists` / `workflow_exists` 校验（与 hw `run_stream_handler` 的 "Resolve the mode up front so we can fail fast"、`workflow_run_stream` 的 `require_builtin` 前置一致）。坏 mode / 坏 workflow → `err_response(msg, 400u)`（400 JSON），不再提交 SSE 后才发 error 帧。
- **实现**：
  - `extern_sigs.at` + `extern_impl.rs` 加 `mode_exists(name @str)` / `workflow_exists(name @str)`（成功返回 `Value::Null`，失败返回 `{"error":{"code":400,"message":...}}`，复用 `resp_is_err`/`resp_err_*` helper）。
  - `server_stream.at` 的两个流式 handler 在 `mpsc_channel()` 前调校验，失败 `return err_response(...)`。重新转译零 drift。
- **验收**：契约测试 `ag_run_stream_bad_mode_returns_400` + `ag_workflow_run_stream_invalid_workflow_returns_400`（断言 400 JSON + error 字段，与 hw 等价）。全量 226 lib 测试全绿。

### 6.2 ✅ 已闭环：conversation_stream 的 broadcast Receiver 连接泄漏（2026-08-06）

- **现状（已修复）**：此前 `conversations_subscribe` 把 `broadcast::Receiver` 存 side-table registry（Value 存 i64 id），客户端在事件间隙断开时 `conv_event_stream` 被 drop、不再调 `broadcast_recv` → registry 条目永不回收（每连接一条，累积）。
- **根治方案**：引入 `BroadcastSub` struct（`use.rust`，a2r 透传），rx 包进 `Arc<Mutex<Option<Receiver>>>`，`BroadcastSub` 持有 Arc。`conv_event_stream(sub: BroadcastSub)` 把 sub（clone）move 进 `async_stream::stream!` 块 → stream drop（正常关闭或客户端断开）→ sub clone drop → **Arc 引用归零 → rx 析构**。**不再用 registry 存 broadcast receiver**，rx 所有权完全由 Arc 引用计数管理（与 hw `BroadcastStream::new(rx)` 把 rx owned 在 stream 里同语义）。
- **工程细节**：
  - `conversation_id`（请求 path）存进 `BroadcastSub`（owned），`conv_event_stream(sub)` 单参数——避免 `&str` 流参数触发 `impl Stream` 生命周期捕获（E0700）。`sub_matches_conv(sub, ev)` 用 sub 内的 id 过滤。
  - `BroadcastSub` 实现 `Clone`（a2r 对非 Copy 类型传参自动 `.clone()`，clone 只增 Arc 计数）。
  - `conversations_subscribe` 接收 conversation_id 参数。
- **验收**：`broadcast_sub_drop_reclaims_receiver`（单元）+ `conversation_stream_drop_reclaims_receiver`（HTTP 层，模拟客户端断开），断言 `ConversationStore::receiver_count` 在 stream drop 后回落到基线。全量 228 lib 测试全绿。

### 6.3 登记备忘：mpsc channel 缓冲 64 条丢帧（与 hw 等价，非缺陷）

- **现状**：`mpsc_channel` 缓冲 64 条（`extern_impl.rs:1537` `tokio::sync::mpsc::channel::<Value>(64)`），流建立前爆发超过 64 事件时 `try_send` 静默丢帧。**与 hw 完全等价**（hw 三个 handler 同样 `mpsc::channel::<Value>(64)` + `try_send`，`server.rs:367/618/904`），非缺陷、无需修复，仅登记备忘。
- **如未来增强**：增大缓冲或 `try_send` 失败降级 `send`（阻塞）——但会偏离 hw 语义，不建议。
