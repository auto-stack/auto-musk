# 009 — auto-musk vs auto-forge 功能补全计划（Parity Roadmap）

> **状态**：实施计划。基于 2026-06-26 逐模块对比（对比报告见本文件附录 A）。
> **仓库**：auto-musk（`backend/crates/musk/` + `web/`）。
> **当前分支**：`rust-impl`。
> **前提**：auto-musk 基本框架已实现（Rust 后端 ~4700 行 + web 前端 ~520 行，见 003/004/005/008 已完成）。本计划补全相对 auto-forge 的功能差距。

---

## 0. 背景与原则

auto-musk 已有可运行的基础：CLI（run/chat/serve 流式）+ HTTP server（auth/run/chats/specs 端点）+ 9 个代码工具 + Chats web app + Spec 数据模型。

但要达到 auto-forge 的产品完整度，还有大量功能差距。对比报告（附录 A）显示：

| 子系统 | 差距量级 | auto-musk | auto-forge |
|---|---|---|---|
| Forge 聊天循环 | 大 | 纯线性 ReAct | WorkMode 三态 + errand + 审批 + 编排触发 |
| Spec Ledger | 中-大 | 数据模型+单一状态机+JSON | per-section 状态机 + 关系图 + 派生 + .ad 文件 |
| **Relay 编排引擎** | **巨大（最大）** | 占位 30 行 | 12000 行全套 |
| 工具系统 | 大 | 9 个代码工具 | 22 个（含 spec/编排/wiki 类）|
| Profession/Soul | 中 | mode + 下沉 agent | handoff/dispatch 图 + 11 soul + ForgePhase |
| MCP | 大 | 无 | 30 个 forge_* 工具 |
| 前端视图 | 巨大 | 2 视图 | 9 视图 |
| runtime | 不算缺口 | — | 已下沉 auto-ai-agent |

**三成分架构注意**（见 designs/002）：provider/ApiSource 已下沉 daemon，**不算缺口**；Profession 核心已下沉 auto-ai-agent；Relay 引擎必须留在 musk app 层（auto-ai-agent 的 relay.rs 仅 100 行 trait，非引擎）。

**补全原则**：
1. 按 P0→P3 优先级，每阶段可独立交付、可验证。
2. 尊重依赖顺序（Relay 依赖 Spec 派生层；编排工具依赖 Relay）。
3. 每阶段一个 PR/commit，带验收。
4. 前端按后端就绪度滚动补（不单独排前端阶段）。

---

## 1. 阶段总览

```
P0  Spec Ledger 派生层（per-section 状态机 + 关系图 + 派生状态）
 │   无依赖，纯增量，解锁 Spec 自动演进
 │
 ├─→ P1a  Spec 工具集（5 个 spec 工具，让 LLM 读写 spec）
 │
 ├─→ P1b  Chat 循环分支（WorkMode + errand + spec 审批）
 │
 ├─→ P2a  Profession 编排元数据（handoff/dispatch 图 + ForgePhase）
 │        │
 │        └─→ P2b  Relay 编排引擎（pipeline/handoff/driver，最大工程）
 │                 │
 │                 └─→ P2c  编排工具（bring_in/spawn_relay/dispatch 等）+ TaskPlan
 │
 ├─→ P3a  Wiki 模块（独立，可与 P2 并行）
 │
 └─→ P3b  MCP 层（30 个 forge_*，依赖前面全部）
 
 前端：P0→SpecsView，P1→ExplorerView/Roles，P2→RelayView，P3→WikiView
```

---

## 2. P0 — Spec Ledger 派生层（地基，先做）

**依赖**：无（specs.rs 已有数据模型 + JSON 持久化）。
**价值**：解锁 spec 自动演进，是 Relay 协作前置。
**风险**：低（纯增量）。

### Tasks

#### Task 1: per-section SectionConfig 状态机
- 当前 `specs.rs:332` 是单一全局状态机；auto-forge `mod.rs:242-342` 有 7 套。
- 实现 `SectionConfig::for_type(SectionType) -> SectionConfig`，含 `allowed_statuses` + `allowed_transitions: Vec<(Status,Status)>`：
  - Goals: Empty→Proposed→Analysed→Approved→InProgress→Implemented→Done→Archived
  - Architecture|Designs: Empty→Draft→UnderReview→{Approved|Rejected}，Approved→{Superseded|Outdated}
  - Plans: Empty→Draft→Approved→InProgress→Done→Obsolete
  - Tests: Empty→Draft→Implemented→Done→Verified，Implemented↔Blocked
  - Reviews|Reports: Empty→Draft→Published
- `can_transition(st, from, to) -> bool`（参考 `mod.rs:337-341`，**收紧**：去掉"to∈allowed_statuses 即放行"的宽松第三条）
- **绿地修正**：修 auto-forge 的 Reports 状态机 bug（`mod.rs:297` 的 match 臂被 `:324` 遮蔽）。
- [ ] 实现 + 单测（每类 section 的合法/非法转换）
- [ ] commit

#### Task 2: rebuild_relations（关系图）
- 当前 `specs.rs:187` related 是死字段。
- 实现：扫 `depends_on` + 正则扫正文 ID `(?:[A-Za-z]+-)?[GADPSVXTIR]\d+(?:\.\d+)?`（参考 `mod.rs:1825-1865`），建反向 `related`，sort+dedup。
- upsert/delete/load 后自动调用。
- [ ] 实现 + 单测（A depends_on B → B.related 含 A；正文引用也建边）
- [ ] commit

#### Task 3: derive_statuses（派生状态）
- 当前无派生。实现 auto-forge `mod.rs:1875-2040+` 的规则：
  - Goal 全 related Plans Done → Implemented
  - Goal Implemented + 全 related Tests Done/Verified + ≥1 Review Published → Verified
  - section 全 item 满足条件 → section 聚合状态升级
- [ ] 实现 + 单测
- [ ] commit

#### Task 4: overview + drift-check 端点
- `GET /api/specs/overview`（聚合视图，参考 `mod.rs:3515`）
- `POST /api/specs/drift-check`（对比磁盘 vs 内存）
- [ ] 端点 + 手测
- [ ] commit

#### Task 5: SpecsView 前端（跟进）
- 参考 auto-forge SpecsView（7 类 section 卡片 + StatusBadge + 关系面板）
- 用 designs/001 的 SpecSectionWidget 思路（1 个参数化 widget 消灭 7 类重复）
- [ ] SpecsView.vue + useSpecs.ts
- [ ] commit

### 验收
- 每类 section 的状态转换受独立状态机约束（非法转换被拒）。
- 修改一个 spec 的 depends_on，相关项的 related 自动更新。
- Goal 的 Plan 全 Done 后，Goal 自动 Implemented。
- SpecsView 可浏览/编辑 7 类 spec，状态切换受约束。

---

## 3. P1a — Spec 工具集

**依赖**：P0。
**价值**：让 LLM 通过工具读写 spec。
**风险**：低。

### Tasks
- 实现 5 个工具（参考 auto-forge `tools.rs:1929-2580`）：
  - `read_specs(section_id?)` / `list_specs()` / `write_spec(section, content)` / `update_spec(action, section, item_id, ...)` / `write_goals(...)`
- `update_spec` 的 action：upsert/delete/patch/set_status，调 SpecsStore（含 P0 派生）。
- 注册进 `build_agent_from_mode`（按 mode 授权）。
- [ ] 5 个工具 + 单测
- [ ] commit

### 验收
- `musk chat` 中让模型 read_specs/update_spec，能正确读写 spec 并触发派生。

---

## 4. P1b — Chat 循环分支（WorkMode + errand + 审批）

**依赖**：P0（spec 变更）。不强依赖 Relay（WorkMode::Direct 先做）。
**价值**：让 chat 从"线性问答"升级为"带编排入口的对话"。
**风险**：中（需扩 ChatSession）。

### Tasks

#### Task 1: 扩 ChatSession
- 当前 `chats.rs:87` 只有 messages+mode。补字段（参考 `mod.rs:53-78`）：work_mode / pending_spec_changes / active_profession / errand_sessions / active_relay_runs / active_task_plan / status(ForgeStatus)。
- [ ] 字段 + 持久化往返测试
- [ ] commit

#### Task 2: WorkMode 分类 + errand
- chat_stream 加分支：检测工具结果类型 → 设 work_mode（Direct/SingleRelay/MultiRelay）。
- errand：检测 run_errand → 创建子 agent → 跑 → 回写（参考 `mod.rs:2804-2826`）。
- [ ] 实现 + 手测
- [ ] commit

#### Task 3: spec 变更审批
- update_spec 工具产生 pending_spec_changes → approve/reject 端点（`POST /api/chats/session/{id}/approve|reject`）。
- approve 应用到 SpecsStore，reject 清空。
- [ ] 端点 + 手测
- [ ] commit

#### Task 4: 流事件扩展
- 当前 SSE 只有 delta/tool/done/error。补 errand_start/turn_start/tool_result/complete（参考 `mod.rs:609-646`）。
- 前端 ChatsView 处理新事件。
- [ ] 实现 + 前端适配
- [ ] commit

### 验收
- chat 中触发 spec 变更 → 前端显示待审批 → approve 后 spec 更新。
- errand 子任务能派发并回写结果。

---

## 5. P2a — Profession 编排元数据

**依赖**：无（可早做），但为 P2b 前置。
**价值**：为 Relay 提供 handoff/dispatch 图。
**风险**：中（需确认 auto-ai-agent Profession trait 是否需扩展）。

### Tasks
- **先核实** `D:\autostack\auto-ai\crates\auto-ai-agent\src\profession.rs` 的 Profession trait 是否含 handoff_to/dispatchable_to/ForgePhase。当前 `lib.rs:56-91` OwnedProfession 转发未含这三字段。
- 若 trait 不含：在 musk 自建 app 级 Profession 注册表（补 handoff_to/dispatchable_to/owned_sections/ForgePhase），或扩展 auto-ai-agent trait。
- 对齐 auto-forge 9+3 profession 的 handoff/dispatch 关系图（`profession.rs:138-615`）。
- [ ] 核实 + 设计决策
- [ ] 实现 + commit

### 验收
- profession 之间可按 handoff_to/dispatchable_to 图路由。

---

## 6. P2b — Relay 编排引擎（最大工程）

**依赖**：P0（Spec）+ P2a（profession 图）。
**价值**：auto-forge 差异化核心。
**风险**：高（多 agent 状态机，~12000 行）。

### 建议分小阶段（每阶段独立交付）
- **P2b.1**：pipeline.rs（RelayPipelineEngine，串行 step + gate + AdvanceResult）+ store.rs（RunStore 持久化 + 事件流）
- **P2b.2**：turn.rs（AgentTurn 单 agent 一次 turn）+ driver.rs（后台驱动 run）
- **P2b.3**：handoff.rs（HandoffDocument 上下文压缩交接）+ checkpoint.rs（快照回滚）
- **P2b.4**：flow.rs（FlowSpec YAML）+ 内置 flows 模板
- **P2b.5**：budget.rs（token 预算追踪）
- **P2b.6**：api.rs（relay REST 端点：runs list/start/advance/handoff/gate/events）
- **P2b.7**：task_plan_engine.rs（多 relay 编排）
- [ ] 每小阶段一 commit + 单测/手测

### 验收
- 能定义一个 flow（architect→coder→tester），启动 relay run，agent 依次接力（handoff 传递上下文），gate 处审批，产出 work product。
- RelayView 前端可视化 run 进度（P2 跟进）。

---

## 7. P2c — 编排工具

**依赖**：P2b。
**Tasks**：bring_in / spawn_relay / spawn_task_plan / register_task_plan / dispatch（参考 `tools.rs:2590-3100`）。
**验收**：chat 中 spawn_relay 触发 P2b 引擎。

---

## 8. P3a — Wiki 模块（独立，可与 P2 并行）

**依赖**：无。
**Tasks**：wiki.rs（CRUD，参考 auto-forge `forge/wiki.rs:719`）+ 4 个 wiki 工具 + WikiView 前端。
**验收**：LLM 能 create/query wiki 页面。

---

## 9. P3b — MCP 层（最后）

**依赖**：前面所有 app 业务。
**Tasks**：30 个 forge_* 工具（参考 `mcp/mod.rs:234-1264`），暴露 musk 业务给外部 LLM client。
**验收**：外部 LLM 客户端（Claude Desktop/Cursor）能通过 MCP 操作 musk。

---

## 10. 前端补全（滚动）

| 视图 | 跟进阶段 | 依赖 |
|---|---|---|
| SpecsView | P0 | specs 派生 |
| ExplorerView | P1 | 已有基础工具 |
| ProfessionsView/Roles | P2a | profession 图 |
| RelayView | P2b | relay 引擎 |
| WikiView | P3a | wiki |
| ~~ApiSourcesView~~ | 跳过 | 归 daemon |

---

## 附录 A：对比报告关键数据（本计划依据）

代码量级：musk 后端 4684 行 / web 519 行；auto-forge 后端 forge+relay ~16000 行 / frontend ~20000 行。

最大缺口：Relay 编排引擎（musk 30 行占位 vs forge 12000 行全套）。

故意下沉（不算缺口）：provider/ApiSource（daemon）、context 压缩/permission（auto-ai-agent）、SkillRegistry（auto-ai-agent）。

关键风险：auto-ai-agent Profession trait 可能不含 handoff_to/dispatchable_to（P2a 需先核实）；auto-ai-agent relay.rs 仅 100 行 trait（Relay 必须在 musk 重写）。

详细逐子系统差距见 2026-06-26 对比会话记录（本计划浓缩其结论）。
