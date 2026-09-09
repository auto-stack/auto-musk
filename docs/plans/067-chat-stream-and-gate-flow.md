---
plan_id: PLAN-067
status: executing
feature_name: 对话流式实时刷新根修 + approve 门暂停/自动通过流程（含本会话问题沉淀与 VM/Rust 双轨对齐检查）
author: zhaop / zcode
created_at: 2026-09-09T15:30:00+08:00
updated_at: 2026-09-09T16:05:00+08:00
plan_revision: 1
current_step: 0
total_steps: 7
supersedes_spec_components: []
new_spec_components:
  - docs/specs/modules/chat-streaming.md
  - docs/specs/modules/web-input-contracts.md
touched_goals: [goal-frontend-parity, goal-relay]
---

# PLAN-067 — 对话流式实时刷新根修 + approve 门流程改造

## 0. 变更摘要

用户实测（2026-09-09）：对话中 AI 回复不实时刷新，往往整个任务跑完后手动刷新才能看到；其间需要 approve 的门（gate）不显示，全部**到期默认拒绝**；任务不因拒绝而中断，继续执行下一步、再被拒，反复循环；最终回复里大量 Block 内容为"已拒绝"，任务以失败告终——全程无法观察、无法 approve。

本计划三件事，按依赖排序：

1. **P0 流式实时刷新根修**（一切的前提）：AI 回复、工具 Block、gate 卡片必须经 SSE 增量实时呈现。
2. **P1 gate 超时行为改造**：超时后**暂停**等待用户显式决议（而非默认拒绝后继续跑）。
3. **P2 自动 approve 模式**：composer 工具栏（"思考强度"旁）新增审批模式选择（人工/自动），自动模式下 gate 直接放行，用于先验证任务能否跑通。

另含两项收尾：本会话已修复问题沉淀（作为 VM 轨 / Rust web 回退轨对齐检查的种子清单，T-06）；全量回归探针（T-07）。

## 1. 目标

- **G1（P0）**：发送消息后，assistant 回复以增量形式实时渲染（首增量 ≤2s）；工具 Block、`gate_waiting` 等 RunEvent 实时出现在会话流中，无需手动刷新。
- **G2（P1）**：gate 到达超时后任务**暂停**（agent 不再推进下一步），gate 保持可决议状态；用户随后 approve → 从断点继续；reject → 中止该步。不再出现"超时默认拒绝→继续跑→连环拒绝→最后整体失败"。
- **G3（P2）**：composer 提供"审批模式"选择（人工 / 自动 approve），选择持久化，自动模式下 gate 自动放行且在流中有迹可查。
- **G4**：本会话（2026-09-09）五个已修复问题沉淀为规范/清单，并据此对 VM 轨与 Rust web 回退轨完成一轮对齐检查，差异登记。
- **非目标**：不重写 relay 流程引擎；不改动 auto-lang codegen 的 oninput/SSE 契约本身（仅在 musk 侧适配；上游根修另立计划）；不动模型/daemon 层。
- **受影响仓库/模块**：auto-musk（web 前端 `src/front/*`、后端 `backend/crates/musk/src/relay/*`、`chats.rs`）；对齐检查只读 VM 轨与 `web/` 回退轨。
- **成功的样子**：一次带 gate 的 relay 任务，用户全程实时看到推进；离开一段时间回来后任务处于暂停而非连环拒绝；点通过后继续；切自动模式可完整跑通。

## 2. 架构方案

- **流式（P0）**：维持现有 SSE 主通道（`Sse.open("/api/chats/session/<id>/stream", .OnStreamEvent)`，`forge_store.at` Plan 028 T14 形态），根修"事件到达但界面不更新"。两个候选断点由 T-01 定责后二选一或并修：
  - 前端侧：`OnStreamEvent` 分发对增量事件（delta/tool/gate_waiting）未正确写入 `.messages`/树视图，或视图层未响应（本轮实测：UI 每秒 GET session 轮询返回了含新消息的数据但列表不渲染——指向视图应用层而非后端推送）。
  - 后端侧：`chats.rs` stream 端点对 assistant 增量未逐段推送（仅终态回填）。
  - `PollStream` 轮询兜底（deadman 窗，PLAN-536 T12 形态）保留为兜底，语义改为"校正"而非唯一更新通道；KD 059-FU1（"AI 回复了但界面不动"）在 VM 轨的前科与本次 web 轨症状同族，T-01 结论需回写 KD。
- **gate 暂停（P1）**：gate 超时不再产生隐式 Reject 决议，而是把 run 置为 `paused`（`GateState` 保持 waiting），agent loop 停在抛出点；用户经 UI（实时可见的 gate 卡）approve/reject 后恢复/中止。落点在超时决议的产生处（T-03 定位：musk relay driver/flows 或 auto-ai agent loop）。
- **自动 approve（P2）**：composer 工具栏"思考强度"旁新增"审批模式"（dropdownlist：人工 / 自动）。数据流对齐 thinkingLevel 既有模式：MentionInput prop + pick 事件 → ChatsView → ForgeStore 持久化 → relay 启动/run 时携带，gate 产生处按模式自动放行（放行记录仍写入流，可审计）。
- **双轨对齐（G4）**：以第 4 节种子清单为检查表，在 VM 轨（`MUSK_BACKEND=vm`）与 `web/` 回退轨逐项复检，差异登记 `KNOWN-DEBT-AND-RISKS.md` 或立后续计划。

## 3. 技术栈

- 前端：Vue 3（gen/front/vue，Auto `.at` 单源生成）+ Tailwind；SSE（`Sse.open/close` 平台协议）；不新增依赖。
- 后端：Rust/axum（musk）；relay 模块（driver/flows/store/api）。
- 验证：浏览器实测（devtools Network SSE 帧 + 控制台探针）、`curl -N` 直接观察流、`cargo test`、VM 轨 `MUSK_BACKEND=vm`。

## 4. 需求分析与背景调查

### 4.1 授权与范围

- 用户 2026-09-09 明确要求：把流式缺失 + gate 流程问题立案，且"不论选哪个方案（或两个都实现），流式问题首先要解决"。两方案均获提案授权，取舍见 `10. 待澄清事项`①。
- 允许仓库：auto-musk（本计划）；auto-lang 仅在确认 codegen 契约缺陷时**另立计划**，不在本计划内改。
- 预算/自动续跑限制：用户未指定——默认逐任务推进、常规 gate 人工确认。

### 4.2 实测证据（2026-09-09，:3000 实例）

- POST `/api/chats/session/<id>/message` → 200；随后 GET `.../stream` 200 重订阅；UI 以 ~1s 周期 GET session，**响应体已含新消息但列表不渲染**；整页刷新后消息全部出现（含 14:13 的回复）。
- musk 日志：`musk-serve.log` 中 stream 重订阅与 session 轮询序列完整；`client error: daemon unavailable` 一例系 aaid 未启动（已另行处置，非本计划范围）。
- gate 链路现状：`relay/store.rs` 定义 `RunEvent::GateWaiting/GateResolved`、`GateState { gate: "auto"|"human" }`；决议入口 `relay/api.rs:286`（approve|reject|edit）。**超时默认拒绝的实现点尚未定位**（grep expires_at/timeout 未命中 musk relay 与 auto-ai-agent 首层）——T-03 专项定位。
- 前科线索：`forge_store.at` 注释 KD 059-FU1 / KD 047 / 055-4②："AI 回复了但界面不动"在 VM 轨为 `Sse.open` handler-as-value 抛点所致（根修归上游 SSE 专项）；web 轨当时"行为不变"。本次症状为 web 轨同类表现，需重新定责。

### 4.3 本会话已修复问题（沉淀；不作为本计划任务）

| # | 问题 | 根因 | 修复 | 提交 |
|---|---|---|---|---|
| 1 | 深色模式 IME 组合串不可见 | PLAN-493 双层文字技术：textarea `color:transparent`，组词文字不进 v-model/backdrop，浏览器对透明组合串回退系统黑 | 组词期 textarea 实绘 `--foreground` + backdrop 隐藏（`inject_styles.web-only.ts` composition 钩子） | f0c77c9 |
| 2 | Ctrl+A"无法全选" | DOM 选区正常，默认选区高亮在暗色下对比不足不可见 | `textarea.chats-input::selection` 显式主题配色 | 00dee19 |
| 3 | composer 每次输入抛 TypeError；mention 检测与 autoGrow 静默失效 | PLAN-051 oninput 双轨契约（codegen 包 `.value` 传纯文本）vs `mention_detect` 读 `e.target` | 串入参回退 `document.activeElement`，事件对象入参走原路（`mention_helpers.at`） | 07b90a5 |
| 4 | 删除确认弹窗告警、描述行不渲染 | `DeleteConfirmDialog.vue` 导入清单漏 `AlertDialogDescription` | 补导入 | 2ff3983 |
| 5 | 对话无响应（daemon unavailable） | aaid 未运行 | 启动 `auto-ai/target/release/aaid.exe`（:17654）+ 重启 musk（运维项；后续可考虑 start-all/musk 文档强调启动顺序） | 运维处置 |

### 4.4 双轨对齐检查种子清单（T-06 输入）

以上 1–4 项（web 轨已修）逐项在 VM 轨与 `web/` 回退轨复检：IME 组合串可见性、选区可见性、输入链路无 TypeError（VM 轨对应 oninput 契约形态）、alert-dialog 全件渲染；外加本计划新增的流式与 gate 行为。差异登记，不在本计划内实现修复。

## 5. 详细设计

### 5.1 P0 流式根修（T-01 → T-02）

T-01 以浏览器 + 日志双面取证，产出决策记录（写入 `9. 复审记录` 或 `attachments/`）：

- 取证面 A（前端）：devtools Network 观察 SSE 帧是否到达、事件类型与负载；`OnStreamEvent` 分发处埋点确认各事件是否改写 `.messages`/树；对照 PollStream 路径（deadman 窗是否开启、轮询是否在校正）。
- 取证面 B（后端）：`curl -N .../stream` 观察推送粒度——是否仅有终态事件、有无 delta/tool/gate_waiting 事件序列。
- 判定矩阵：①SSE 帧到达且含增量 → 前端应用/渲染缺陷（改 `OnStreamEvent` 分发或视图绑定）；②帧到达但只有终态 → 后端推送粒度缺陷（改 `chats.rs` stream 端点为逐段推送）；③帧不到达 → 连接/代理层（本机无代理变量，可能性低）。

T-02 按 T-01 结论实施，统一验收口径见 AC-01/AC-02。约束：不破坏 VM 轨轮询兜底契约（`forge_store.at` 顶部注记的 500ms 轮询期语义）；web 轨 SSE 增量为唯一主通道，轮询只做校正。

### 5.2 P1 gate 超时暂停（T-03 → T-04）

- T-03 定位超时决议产生点（候选：`relay/driver.rs`、`relay/flows.rs`、`relay/store.rs` 的 GateWaiting 生命周期、auto-ai-agent 的 gate 原语）并确认现行超时值与"拒绝后继续"的决策流。
- T-04 改造：超时触发 `paused`（而非 Reject 决议）——run 状态机新增/复用暂停态；agent 停在当前步；UI 呈现"已暂停，等待决议"；用户 approve → 恢复执行该步并继续；reject → 中止。需要明确：恢复时的上下文续接方式（gate 抛出点重放 vs 状态续跑）以 T-03 结论为准；超时值改为可配置（默认值见 `10.待澄清`②）。

### 5.3 P2 自动 approve 模式（T-05）

- UI：`mention_input.at` 工具栏行（思考档位旁）新增 `审批: 人工/自动` dropdownlist，事件流对齐 thinkingLevel（prop + pick 事件 → `chats_view.at` → ForgeStore 持久化）。
- 生效链：run 启动/运行时把模式带给 relay（`relay/api.rs`/`store.rs`）；gate 产生处按模式直接构造 Approve 决议并写入流（`gate_resolved(auto)` 之类可审计事件），不绕过记录。
- 兼容：模式缺省 = 人工（现行为）；VM 轨同期对齐或登记差异。

### 5.4 双轨对齐与回归（T-06 / T-07）

- T-06 按 4.4 清单在 VM 轨（`MUSK_BACKEND=vm`）与 `web/` 回退轨复检并登记差异（只记录不修复）。
- T-07 固化本会话探针为可重复脚本/清单：IME 组合（深/浅色）、Ctrl+A 选区、输入零报错、弹窗全件、daemon 状态提示、流式增量、gate 暂停/自动放行。

### 规范增量

| delta_id | 变更 | 目标 | before | after | 理由 | 关联 AC |
|---|---|---|---|---|---|---|
| SD-01 | add | docs/specs/modules/chat-streaming.md | （无专门规范；流式行为散见 forge_store.at 注释与 KD） | 对话实时性契约：assistant 回复/工具 Block/gate_waiting 必须经 SSE 增量实时呈现（首增量 ≤2s）；轮询仅可为兜底校正；违反即缺陷 | 流式缺失是本计划最严重缺陷，需成文契约防复发 | AC-01, AC-02 |
| SD-02 | modify | docs/specs/01-architecture.md（relay/gate 行为段） | gate 超时默认拒绝并继续执行 | gate 超时默认暂停等待显式决议；自动 approve 仅为用户显式选择且留审计事件 | 消除"连环拒绝至整体失败"的流程缺陷 | AC-03, AC-04, AC-05 |
| SD-03 | add | docs/specs/modules/web-input-contracts.md | （本会话 5 项修复散在提交与注释） | web 输入面契约沉淀：双层文字技术、PLAN-051 oninput 双轨契约的适配义务、IME/选区可见性规则、alert-dialog 全件导入要求——同时作为 VM/Rust 轨对齐检查表 | 沉淀已知坑为检查表，支撑 G4 与后续双轨巡检 | AC-06 |

## 6. 测试设计

- **流式**：`curl -N ".../stream"` 断言事件序列含增量与 gate 事件；浏览器实测首增量延迟；`pnpm dev` 控制台探针（本会话 `window.__errs` 模式）确认零未捕获异常。
- **gate 暂停**：构造带 human gate 的 relay 任务，静置超时 → 断言 run 进入 paused 且 agent 无后续步；UI approve → 断言从断点继续；reject → 断言中止。后端逻辑配 `cargo test`（relay 模块单测：超时→paused、approve 恢复、reject 中止三态）。
- **自动 approve**：切自动模式跑同任务 → 断言无需人工操作跑通、流中存在自动放行审计事件；切回人工恢复原行为。
- **回归**：T-07 清单逐项。

## 7. 验收标准

- **AC-01**：发送消息后，assistant 回复首增量在 ≤2s 内渲染，全程无需手动刷新。验证：浏览器实测 + Network SSE 帧；`curl -N` 可见增量事件。
- **AC-02**：relay 任务运行中，工具 Block 与 gate 卡片实时出现在会话流（出现延迟 ≤2s）。验证：带工具/gate 的任务实测截图 + 时间戳对照。
- **AC-03**：gate 超时后任务进入暂停态，agent 不执行下一步；UI 明示"已暂停，等待决议"。验证：单测 + 实测（静置超时观察）。
- **AC-04**：暂停后用户 approve → 任务从该 gate 断点继续并完成；reject → 中止且状态一致。验证：实测两分支 + 单测。
- **AC-05**：composer 可切换审批模式（人工/自动），持久化；自动模式下带 gate 任务全程无人干预跑通，流中有自动放行审计事件。验证：实测。
- **AC-06**：VM 轨与 web/ 回退轨按种子清单完成复检，差异登记成文（KNOWN-DEBT 或后续计划）。验证：检查记录文档。
- **AC-07**：回归清单全绿：IME 组合串（深/浅色）、Ctrl+A 可见选区、输入零 TypeError、删除弹窗全件、aaid 状态提示。验证：T-07 清单执行记录。

## 8. 执行步骤

- [ ] **T-01 流式断点定责（调查）**：按 5.1 取证矩阵产出根因决策记录（attachments/或复审记录）。涉及：`src/front/forge_store.at`（OnStreamEvent/PollStream）、`backend/crates/musk/src/chats.rs`（stream 端点）。验证：决策记录成文，含证据截图/日志。→ AC-01。
- [ ] **T-02 流式根修**（依赖 T-01）：按结论实施前端/后端修复；SSE 增量为主通道、轮询校正为兜底。验证：AC-01/AC-02 实测通过；`cargo test`/前端现有测试不回归。
- [ ] **T-03 gate 超时机制定位（调查）**：定位超时默认拒绝产生点与现行超时值，理清 gate 决议数据流（`relay/api.rs` ← UI；`GateWaiting/Resolved` ← store）。产出定位记录。→ AC-03。
- [ ] **T-04 gate 超时改暂停**（依赖 T-03）：暂停态 + 恢复/中止语义 + 超时可配置；补 relay 单测三态。验证：AC-03/AC-04。
- [ ] **T-05 审批模式**（依赖 T-04 定义的决议产生点）：mention_input.at UI + ForgeStore 持久化 + relay 决议链；审计事件。验证：AC-05。
- [ ] **T-06 双轨对齐检查**（依赖 T-02/T-04/T-05 行为定型）：按 4.4 清单复检 VM 轨与 web/ 轨，差异登记。验证：AC-06。
- [ ] **T-07 回归探针固化与执行**：固化清单脚本并全量执行。验证：AC-07。

### 执行记录

- 2026-09-09：用户确认待澄清①——P1 暂停与 P2 自动 approve **都做**，顺序 P0 → P1 → P2 → T-06 → T-07。授权进入 executing。
- worktree：`D:/autostack/.wt/musk-067/auto-musk`（branch `plan-067-dev`，base = main@d7c2a65）。

## 9. 复审记录

- 2026-09-09（new / r1 起草）：基于用户实测与当日本会话证据起草。背景调查记录于 4.2；两处调查型任务（T-01/T-03）以决策记录为产物。handoff：`stage: new`，`outcome: pass`（起草完成，可进入 review）；`next: work` 前置条件 = `10. 待澄清`①（方案取舍）由用户确认，其余可在执行中细化。

## 10. 待澄清事项

1. **方案取舍**：✅ 已决（2026-09-09，用户确认）——P1 暂停与 P2 自动 approve 都做；顺序 P0 → P1 → P2 → T-06 → T-07。
2. **gate 超时值**：暂停模式的默认超时时长（现行为疑似数分钟级自动拒绝）；是否分 gate 类型（human/auto）配置。
3. **自动 approve 作用域**：仅 phase gate，还是含工具调用审批？自动模式下 reject 是否还允许人工介入。
4. **轮询兜底最终形态**：SSE 根修后 PollStream 是否降级为"断线重连期间的临时通道"（涉及 KD 059-FU1 的关闭条件），需 T-01 结论后定。
