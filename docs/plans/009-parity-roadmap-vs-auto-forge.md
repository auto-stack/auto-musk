# 009 — auto-musk vs auto-forge 功能补全计划（Parity Roadmap）

> **状态**：✅ **核心功能 100% 落地**（2026-08-04 核对代码）。P0/P1a/P1b/P2a/P2b(全)/P2c(全)/P3a 均已实施。P2b.3 checkpoint 回滚与 P3b MCP 层经评估**降级为按需 Backlog**(理由见文末「剩余工作」)。
> **架构说明**：Plan 008 已把通用编排原语（PipelineEngine/HandoffDocument/FlowSpec/BudgetTracker）下沉到外部 `auto-ai-agent` crate（`backend/crates/musk/Cargo.toml:17` path 依赖），故本计划 P2b 中"在 musk 找不到"的 `.rs` 文件实为**已下沉实现**，非缺失。
> **状态**：实施计划。基于 2026-06-26 逐模块对比（对比报告见本文件附录 A）。
> **仓库**：auto-musk（`backend/crates/musk/` + `web/`）。
> **当前分支**：`main`（原 `rust-impl` 已合并删除）。
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
- [x] 实现 + 单测（每类 section 的合法/非法转换）— `specs.rs` `SectionConfig::for_type` + `can_transition`（642-755 行），~30 单测覆盖
- [x] commit — `7cfee8e`

#### Task 2: rebuild_relations（关系图）
- 当前 `specs.rs:187` related 是死字段。
- 实现：扫 `depends_on` + 正则扫正文 ID `(?:[A-Za-z]+-)?[GADPSVXTIR]\d+(?:\.\d+)?`（参考 `mod.rs:1825-1865`），建反向 `related`，sort+dedup。
- upsert/delete/load 后自动调用。
- [x] 实现 + 单测（A depends_on B → B.related 含 A；正文引用也建边）— `rebuild_relations`（specs.rs:363）+ `id_regex`/`scan_refs`，upsert/delete 自动调用
- [x] commit — `bd094de`

#### Task 3: derive_statuses（派生状态）
- 当前无派生。实现 auto-forge `mod.rs:1875-2040+` 的规则：
  - Goal 全 related Plans Done → Implemented
  - Goal Implemented + 全 related Tests Done/Verified + ≥1 Review Published → Verified
  - section 全 item 满足条件 → section 聚合状态升级
- [x] 实现 + 单测 — `derive_statuses`（specs.rs:417）3 规则全覆盖
- [x] commit — `deb600a`

#### Task 4: overview + drift-check 端点
- `GET /api/specs/overview`（聚合视图，参考 `mod.rs:3515`）
- `POST /api/specs/drift-check`（对比磁盘 vs 内存）
- [x] 端点 + 手测 — `GET /api/specs/overview` + `POST /api/specs/drift-check` + 额外 `rebuild-relations`/`related`（server.rs:119-122）
- [x] commit — `ef1278f`

#### Task 5: SpecsView 前端（跟进）
- 参考 auto-forge SpecsView（7 类 section 卡片 + StatusBadge + 关系面板）
- 用 designs/001 的 SpecSectionWidget 思路（1 个参数化 widget 消灭 7 类重复）
- [x] SpecsView.vue + useSpecs.ts — `web/src/views/SpecsView.vue`(1396 行) + `useSpecs.ts`(135 行) + 7 类 category 组件
- [x] commit — `9976953`（P0 complete）

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
- [x] 5 个工具 + 单测 — `spec_tools.rs`(559 行) 5 个工具：read_specs/list_specs/write_spec/update_spec/write_goals，注册于 lib.rs:165-169
- [x] commit — `d9cb8e7`

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
- [x] 字段 + 持久化往返测试 — ChatSession 已扩展（work_mode/pending_spec_changes/active_profession 等），对话统一模型见 conversation.rs
- [x] commit — `105e16d`（部分）/ `522f61e`（三 work mode）

#### Task 2: WorkMode 分类 + errand
- chat_stream 加分支：检测工具结果类型 → 设 work_mode（Direct/SingleRelay/MultiRelay）。
- errand：检测 run_errand → 创建子 agent → 跑 → 回写（参考 `mod.rs:2804-2826`）。
- [x] 实现 + 手测 — 三 work mode（superpower/relay/bring_in）+ errand 派发回写已落地
- [x] commit — `522f61e`（三 work mode + relay flows + bring_in）/ `cbc269b`（spawn_relay + dispatch）

#### Task 3: spec 变更审批
- update_spec 工具产生 pending_spec_changes → approve/reject 端点（`POST /api/chats/session/{id}/approve|reject`）。
- approve 应用到 SpecsStore，reject 清空。
- [x] 端点 + 手测 — `POST /api/chats/session/{id}/approve|reject|reject-all`（server.rs:1689-1723）
- [x] commit — `105e16d`

#### Task 4: 流事件扩展
- 当前 SSE 只有 delta/tool/done/error。补 errand_start/turn_start/tool_result/complete（参考 `mod.rs:609-646`）。
- 前端 ChatsView 处理新事件。
- [x] 实现 + 前端适配 — 统一 SSE 事件总线（见 plan 013）；ChatsView 处理 relay_spawned/relay_update 等事件
- [x] commit — 随 conversation 统一（plan 013）落地

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
- [x] 核实 + 设计决策 — Profession（后改名 Role，`505cd95`）编排元数据落地；通用原语下沉 auto-ai-agent
- [x] 实现 + commit — `8f8c752`（P2a）/ `505cd95`（Profession→Role 重命名）

### 验收
- profession 之间可按 handoff_to/dispatchable_to 图路由。

---

## 6. P2b — Relay 编排引擎（最大工程）

**依赖**：P0（Spec）+ P2a（profession 图）。
**价值**：auto-forge 差异化核心。
**风险**：高（多 agent 状态机，~12000 行）。

### 建议分小阶段（每阶段独立交付）
- [x] **P2b.1** ✅：pipeline（PipelineEngine/AdvanceResult 已下沉 auto-ai-agent）+ `relay/store.rs`(1078 行，RunStore 持久化+事件流)
- [x] **P2b.2** ✅：driver.rs（`drive_run`/`drive_loop` 后台驱动）；turn 概念折叠进 `conversation.rs:Turn`（无独立 turn.rs，by design）
- [x] **P2b.3** ✅（核心已落地，checkpoint 降级）：HandoffDocument（auto-ai-agent `handoff.at`）+ `store.rs:591 rerun`（失败 step 重试）已落地；**checkpoint 快照/回滚降级为按需 Backlog**（auto-forge 自身 569 行 checkpoint.rs 从未接线进 driver = 死代码；git 已提供文件级回滚替代；详见文末）
- [x] **P2b.4** ✅：FlowSpec/FlowStep/GateType（auto-ai-agent `flow.at`）+ `relay/flows.rs` + `auto_generated/relay_flows.rs` 内置 default/simple/superpower/relay 4 模板（注：代码定义，非 YAML 加载）
- [x] **P2b.5** ✅：BudgetTracker/BudgetStrategy/TokenBudget（auto-ai-agent `budget.at`）+ profession.rs/server.rs 串联
- [x] **P2b.6** ✅：`relay/api.rs`(393 行) 全端点：runs list/start/get/delete/title/advance/rerun/handoff/gate/events + professions/souls/flows
- [x] **P2b.7** ✅：task_plan_engine.rs（多 relay 编排）— 已移植 auto-forge：数据模型+解析（`task_plan.rs`/`task_plan_parser.rs`，Atom DSL）、`HandoffStore`（跨 run 交接）、`TaskPlanRegistry`（每工作区内置+用户 plan）、`TaskPlanEngine`（拓扑排序 phase + serial/parallel + 失败传播 + input_from 串接）、默认执行器 `drive_task_plan_run`（复用 musk `drive_run`）、6 REST 端点 + SSE。19 单测全通过。
- [x] 每小阶段一 commit + 单测/手测 — P2b.1-6 ✅；P2b.7 ✅（Step 1-6 各一 commit）

### 验收
- 能定义一个 flow（architect→coder→tester），启动 relay run，agent 依次接力（handoff 传递上下文），gate 处审批，产出 work product。
- RelayView 前端可视化 run 进度（P2 跟进）。

---

## 7. P2c — 编排工具

> **状态**：✅ **5/5 全部落地**（2026-08-04）。bring_in / spawn_relay / dispatch ✅（`orch_tools.rs`）；**spawn_task_plan / register_task_plan ✅**（随 P2b.7 落地，`orch_tools.rs` + lib.rs orch_tools 列表）。

**依赖**：P2b。
**Tasks**：bring_in / spawn_relay / spawn_task_plan / register_task_plan / dispatch（参考 `tools.rs:2590-3100`）。
**验收**：chat 中 spawn_relay 触发 P2b 引擎。

---

## 8. P3a — Wiki 模块（独立，可与 P2 并行）

> **状态**：✅ **完成**（`8f8c752`）。`wiki.rs`(847 行，WikiStore + wiki_routes) + `web/src/views/WikiView.vue`(865 行) + `useWiki.ts`(231 行)。

**依赖**：无。
**Tasks**：wiki.rs（CRUD，参考 auto-forge `forge/wiki.rs:719`）+ 4 个 wiki 工具 + WikiView 前端。
**验收**：LLM 能 create/query wiki 页面。

---

## 9. P3b — MCP 层（按需 Backlog）

> **状态**：⏸️ **降级为按需 Backlog**（2026-08-04 评估）。musk 内部工具集（20 个 `impl Tool`）已覆盖 auto-forge 内部 Tool trait 的 90%+；P3b 缺的仅是给**外部** MCP 客户端（Claude Desktop/Cursor）的第三套对外接口,与 musk 现有三条入口(CLI/REPL/Web)正交。**无当前用户需求时不做**；若未来需要,先做 5-8 个高价值工具验证。详见文末「剩余工作」。

**依赖**：前面所有 app 业务。
**Tasks**（若实施）：27 个 forge_* 工具（参考 `mcp/mod.rs:234-1264`），暴露 musk 业务给外部 LLM client。
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

## 11. 降级评估（2026-08-04）

Plan 009 核心功能已 100% 落地。以下 2 项经评估**降级为按需 Backlog**,不阻塞收官:

### P2b.3 — checkpoint 快照/回滚(降级理由)
- **参考实现是死代码**:auto-forge 的 `relay/checkpoint.rs`(569 行)从未接线进 driver——`save_checkpoint`/`from_checkpoint`/`restore_files` 在整个代码库里除了自身定义和 `mod.rs` 的 `pub use` 外无任何调用。唯一相关的是 `forge/tools.rs` 里一个只读 `get_checkpoint_diff` 工具,且它不碰 `Checkpoint` 结构体。
- **有廉价替代**:agent 工作区通常是 git repo,`git reset`/`git checkout` 已提供文件级回滚能力。
- **musk 已覆盖核心场景**:`store.rs:591 rerun()` 提供失败 step 重试,这是 checkpoint 最常见的实际用途。
- **移植属 net-new 高风险**:auto-forge 自己都没接线,意味着没有已验证的快照触发时机和回滚语义设计。

### P3b — MCP 层(降级理由)
- **对 musk 核心场景零增益**:musk 内部工具集(20 个 `impl Tool`:10 文件 + 5 spec + 5 编排)已覆盖 auto-forge 内部 Tool trait 的 90%+。P3b 缺的仅是把工具再封装一层 `#[tool]` 暴露给**外部** MCP 客户端(Claude Desktop/Cursor/Kimi CLI)——这是 auto-forge 的第三套对外接口(REST 给前端、内部 Tool 给自己的 LLM、MCP 给外部 LLM),与 musk 现有三条入口(CLI/REPL/Web app)正交。
- **工具能力零净增**:MCP 工具只是转发到内部 store/server 函数,和 REST 端点高度重复。
- **目标用户错位**:musk 定位是终端应用(自带 LLM 的 agent),不是给别人当后端的工具平台。无当前外部 MCP 客户端需求。
- **若未来需要**:不必做全 27 个——先做 5-8 个高价值的(create_session / send_message / start_run / get_run / read_specs / update_spec)验证是否有用户,再决定补全。

---

## 附录 A：对比报告关键数据（本计划依据）

代码量级：musk 后端 4684 行 / web 519 行；auto-forge 后端 forge+relay ~16000 行 / frontend ~20000 行。

最大缺口：Relay 编排引擎（musk 30 行占位 vs forge 12000 行全套）。

故意下沉（不算缺口）：provider/ApiSource（daemon）、context 压缩/permission（auto-ai-agent）、SkillRegistry（auto-ai-agent）。

关键风险：auto-ai-agent Profession trait 可能不含 handoff_to/dispatchable_to（P2a 需先核实）；auto-ai-agent relay.rs 仅 100 行 trait（Relay 必须在 musk 重写）。

详细逐子系统差距见 2026-06-26 对比会话记录（本计划浓缩其结论）。
