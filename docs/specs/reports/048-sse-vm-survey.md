# 048 SSE 专项勘察报告——流式聊天的 VM 轨替代方案定型

> PLAN-048 T6 产出（2026-08-28）。性质=勘察报告，零流式实现。
> 立项依据方：KNOWN-DEBT 047 行 G1 项（SSE 无 VM 形态）；流式实现归下一计划。

## 1. 背景与现状

- **后端**：musk serve 提供 axum SSE 端点 `GET /api/chats/session/{id}/stream`
  （`src/back/api.at:25` `chat_stream(id) ~Stream<SseEventDto>`，Plan 043 起
  vue 轨 codegen 据此生成 type-driven SSE 消费）。
- **vue 轨**：store composable 经 `resolve_stream_endpoints_for_project`
  （auto-lang `ui_gen/api.rs` Plan 043 stream phase）接 EventSource；乐观插入
  user 消息 + delta 事件回写（forge_stream.ts 语义，已实装）。
- **VM 轨缺口（047 G1）**：VM 无 EventSource/SSE 客户端面。musk 源已预留接线
  点——`chats_view.at:305-306` `.SendInput` 内 `store.Send(.text)` +
  `store.StartStream(.store.session_id, .store.workspace, .store.token, .text)`，
  即 VM 侧消费钩子形状已定，只待上游能力。
- **运行期硬约束（047-R3 降级态未解除）**：流式渲染链依赖的
  `platform:markdown`（StreamingRenderer）VM 侧仍是 Markdown 文本降级组件
  （renderer.vm.at 就绪未激活，卡 047 KD 行 UPSTREAM② ext-link fn-only）。
  即使流式数据面打通，assistant 消息/流式 draft 的富文本渲染仍降级。
- **辅助事实（PLAN-048 T2/T3 勘察新增）**：auto-lang iced renderer 内已有
  **进程内 SSE 泵先例**——shell 桥 HTTP 模式（`renderer.rs:4954` 起，
  `AUTO_BACKEND` 非空时连后端 `/api/stream`，逐帧解析经 mpsc 回流 iced，
  `__sse_*` 预置字段 + 无参 handler 派发模式）；该模式为 VM 轨流式提供了
  零新 native 的上游参照形态。

## 2. 候选方案对比矩阵

| 方案 | 改动面 | 延迟语义 | 上游依赖 | 风险 | 工作量估 |
|:---|:---|:---|:---|:---|:---|
| **a) 轮询**：VM 侧 tick（`extract_tick_interval_from_decl` 既有面）定时 `chats_get_session` 全量拉取/diff 追加 | musk 侧（forge_store 增 tick handler + diff 逻辑），上游零改 | 消息级粒度；延迟=poll 间隔（2-5s 实用）；**无 token 级流式** | 无 | diff 语义漂移（active_leaf/分叉树）；轮询放大后端读放大 | **0.5-1 日** |
| **b) WebSocket 桥**：auto-lang stdlib 新增 `auto.ws.*` native 家族（connect/send/on_message + VM 异步 wait 集成），musk 改订 WS | 上游 stdlib/ffi 新面 + musk 重订 | token 级实时（由后端 WS 化程度定） | **新 native 家族**（async wait 集成面同 Plan 349 形态） | 后端当前是 axum SSE，WS 化需 backend 改动（违本计划不动 backend 约束）；native 面审批重 | **2-4 日上游 + 1 日 musk** |
| **c) 后端为 VM 轨增设增量拉取端点** | backend 源 | 消息级 | 无 | **违「不动 backend」约束**；且 VM 侧仍需轮询器——被方案 a 支配 | 1-2 日（**否决**） |
| **d) iced 渲染器 SSE 桥泛化**（勘察新增）：把 shell 桥 `http_sse_loop` 的「进程内 SSE 泵 + `__sse_*` 预置字段 + 无参 handler 派发」模式泛化为通用 chat 流桥，URL/事件契约由 store 声明驱动 | 上游 renderer.rs（模式已在仓内验证）+ musk 仅声明 | token 级（SSE 原语义保留） | 上游渲染器改造（无新 native；沿 Plan 453/060 事件注入先例） | 事件契约（SseEventDto）与 shell ShellEvent 形态差异需适配层；多流并存语义 | **2-3 日上游 + 1 日 musk** |

## 3. 推荐排序

1. **方案 a（轮询）**——流式计划的首阶段：最小改动、零上游、立即解除
   G1 的「会话推进不可见」；token 级流式留待阶段 2。与 platform:markdown
   降级态叠加后，实际观感损失有限（本就文本降级渲染）。
2. **方案 d（SSE 桥泛化）**——阶段 2 主路线：复用仓内已验证的 shell 桥形态，
   不新增 native 面，保住 SSE 原语义与 token 级流式；与 047 KD 行 UPSTREAM②
   （ext-link 激活 renderer.vm.at Markdown）同批立项可一次性解除流式渲染双降级。
3. **方案 b（WS native）**——远期：仅当后端生态整体 WS 化时再启，当前被 d 支配。
4. **方案 c**——否决（违反不动 backend 约束 + 被方案 a 支配）。

## 4. 证据锚点

- 契约流端点：`src/back/api.at:25`（chat_stream ~Stream<SseEventDto>）。
- VM 侧预留接线点：`src/front/chats_view.at:305-306`（StartStream 四参派发）。
- shell 桥 SSE 泵先例：auto-lang `crates/auto-lang/src/ui/iced/renderer.rs:4954`
  （HTTP 模式逐帧 SSE → mpsc → `__sse_*` 字段 + 无参 handler）。
- vue 轨 type-driven SSE：auto-lang `crates/auto-lang/src/ui_gen/api.rs`
  `resolve_stream_endpoints_for_project`（Plan 043 stream phase）。
- 渲染硬约束：KNOWN-DEBT 047 行 UPSTREAM② / DEGRADED「Markdown 文本降级」。
