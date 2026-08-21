---
plan_id: PLAN-030
status: merged
feature_name: 基于 Plan 的 Agent 开发流程（取代 spec 流水线）
author: [zhaopuming]
created_at: 2026-08-21T10:30:00+08:00
updated_at: 2026-08-21T14:20:00+08:00

# Leave these EMPTY here — /auto-plan:review fills them:
supersedes_spec_components:
  - "relay/spec-driven pipeline: default/relay 7 角色流水线降级 deprecated（代码保留对拍），canonical 开发流程改为 plan 驱动四相位"
  - "relay 存储: RunStore 磁盘持久化移除（转 in-memory），conversation（Flow 会话）成为 run 日志唯一归档；relay 独立界面移除"
  - "spec 工具存储域: spec_tools 5 工具在 server/relay 场景从 home 目录默认改为 workspace 域（build_agent_with_context 覆盖注册）"
new_spec_components:
  - "plan flow: 4 步全 plan-dev 单角色相位流（plan→execute[Human gate=计划确认]→review→document），flows.rs + auto_generated 双轨"
  - "plan-dev 角色: auto-ai-agent builtin（Max/0.3/120 turns，双轨 rust-ref+.at）+ musk profession（6 区管辖/自 handoff）"
  - "plan 工具集: list_plans/read_plan/create_plan/update_plan/transition_plan/merge_plan（plan_tools.rs，workspace 域；merge 核心抽 plans::merge_plan_stores）"
  - "相位任务模板 + PLAN_FILE 标记协议: relay/plan_flow.rs 四段模板，driver 提取 PLAN_FILE 存 run.context，step_context 组装注入"
  - "plan_merge 无编号中文标题兼容: canonical 前缀映射（028/029 风格 plan 可沉淀）"
  - "SpecsStore legacy 7 区容忍: SectionType #[serde(other)] Unknown + load 过滤（含单测）"
  - "spawn_relay 会话唯一化: 先 start_run，父 ToolCall child_conversation 直指 run-id 唯一 Flow 会话"
touched_goals: []

current_step: 16
total_steps: 16
---

# [PLAN-030] 基于 Plan 的 Agent 开发流程（取代 spec 流水线）

## 0. 变更摘要

用「计划文件为脊柱的单角色四相位流程」取代现有「7 步 7 角色靠 HandoffDocument 交接的
spec 流水线」。新流程 `plan`：Assistant（Nicole，本地 Mid 档）做需求初筛 → 复杂任务
`spawn_relay(flow_id="plan")` → 四个相位（plan / execute / review / document）全部由
**同一个新角色 `plan-dev`** 执行，相位间不靠 handoff 摘要交接，而是以
`docs/plans/NNN-*.md` 计划文件为**全量交接载体**（driver 确定性提取 `PLAN_FILE:` 标记，
后续相位任务模板自动注入该路径）。execute 前设 Human gate 作为「计划确认点」（对齐
superpowers 的 present-for-confirmation 纪律）；plan 状态机
（drafting→executing→execution_done→review_done→merged）充当流程内部状态；document 相位
经 `merge_plan` 工具沉淀 Spec ledger 6 区并更新 `docs/specs/` 模块树。旧 `default`/`relay`
flow 标记 deprecated（代码保留、默认路由切换）。

配套地基修复：`plan_merge.rs` 章节提取兼容无编号中文标题（028/029 风格 plan 目前
merge 提取不到任何章节）；新增 agent 工具模块 `plan_tools.rs`（6 个 plan 工具）。

展示与存储同步归一（2026-08-21 需求补充）：取消 relay 独立界面——RelayView.vue 已是
无引用死代码（PLAN-024 导航改四项后遗留），本轮删除；flow 活动全部在 Chats 内联展示
（RelayRunBox，已有能力）。relay 日志不再单独落盘（`.autoos/relay/` run 文件停写，
RunStore 转 in-memory），现有 conversation dual-write 升级为**唯一持久日志**（Flow 会话）。

## 1. 目标

1. **消灭交接损耗**：旧流程 7 个角色各带独立 Soul/上下文，每步交接靠
   HandoffDocument 渲染摘要注入下一步 user history，摘要丢失即信息丢失。新流程以计划
   文件为全量事实源，交接零丢失。
2. **单角色全流程**：Advisor/Coder/Reviewer/Documenter 四个角色初期由同一个 agent
   （`plan-dev`，Max 档 120 turns）扮演，模式与传统 Agent 通过 superpowers 技能开发
   几乎一致——brainstorm → writing-plans → executing-plans → review → 沉淀。
3. **Spec 事后沉淀**：开发期不碰 Spec（上下文局部性），review 通过后由 document 相位
   统一沉淀进 6 区 ledger + docs/specs 模块树（延续 008「延迟规格物化」）。
4. **Assistant 轻量初筛**：简单需求 assistant 在 chat 内直接完成（现有 NORMAL 路由），
   复杂需求才引入 plan flow（用户需求描述第 2 步的「本地 Agent」= Nicole Mid 档）。
5. 旧 spec 流水线（default/relay flow）退出默认路由，不再作为 canonical pipeline。
6. **界面与日志归一**：无独立 relay 界面，flow 活动在 Chats 内联展示；conversation 是
   flow 日志的唯一持久归档（不再单独落盘）。

## 2. 架构方案

### 2.1 六步用户流程 → 实现映射

| 用户流程步骤 | 实现 |
|---|---|
| 1. 客户提出需求 | Chats 发消息（现有 `/api/chats/session/{id}/message`） |
| 2. Assistant 整理初筛 | Nicole（assistant role）整理需求；简单 → chat 内直接做；复杂 → `spawn_relay(flow_id="plan")` |
| 3. Advisor 写计划文件 | flow step `plan`（plan-dev）：模糊则输出澄清问题（经 gate reject+feedback 回答）；产出 `docs/plans/NNN-*.md`（含需求分析/架构/详细设计/代码示例/测试设计/验收标准/执行步骤） |
| 4. Coder 实施 | flow step `execute`（plan-dev，前置 **Human gate=计划确认**）：plan 为唯一上下文逐任务执行 → execution_done |
| 5. Reviewer 复审出报告 | flow step `review`（plan-dev）：逐验收标准对照实际代码重验，填 `## 9. 复审记录` + spec-impact 三字段 → review_done；不过则 plan 回退 executing 并在报告中说明 |
| 6. Documenter 沉淀 Specs | flow step `document`（plan-dev）：检查 status → `merge_plan` 沉淀 ledger 6 区 → 更新 `docs/specs/` 模块树 → 归档 → merged |

### 2.2 新 flow 定义（backend/crates/musk/src/relay/flows.rs）

```rust
/// Plan-driven dev flow — 单角色四相位，计划文件为交接载体（PLAN-030）。
fn plan_flow() -> FlowSpec {
    use GateType::*;
    let mut flow = FlowSpec::new("plan");
    flow.add_step(FlowStep::new("plan", "plan-dev"));     // Advisor 相位
    flow.add_step(FlowStep::new("execute", "plan-dev").with_gate(Human)); // 计划确认点
    flow.add_step(FlowStep::new("review", "plan-dev"));   // Reviewer 相位
    flow.add_step(FlowStep::new("document", "plan-dev")); // Documenter 相位
    flow
}
```

四步同 profession：角色/Soul/模型档位/工具集完全一致 → 「同一个 Agent」。相位差异由
**相位任务模板**（§2.3）驱动，不换角色。

### 2.3 相位任务模板机制（musk-local，不动 auto-ai orchestration）

`FlowStep` 没有 per-step prompt 字段（flow.at 只有 id/role_id/gate/max_turns/exit/budget），
当前 `step_context` = initial_task + 上一步 handoff 摘要。本计划在 musk 侧新增
`relay/plan_flow.rs`：`phase_task_template(flow_id, step_id) -> Option<String>`，
`step_context` 组装时优先用模板（plan flow 的四步），占位符 `{plan_file}` 用 run 元数据
替换。**不改 auto-ai-agent 的 orchestration 类型**（跨仓通用化 `FlowStep.task_template`
列为后续演进，见 §10）。

### 2.4 PLAN_FILE 标记协议（确定性交接）

- `plan` 相位模板要求：完成时输出最后一行 `PLAN_FILE: docs/plans/NNN-slug.md`。
- `driver.rs run_step` 在步骤完成后用正则 `(?m)^PLAN_FILE:\s*(\S+)\s*$` 从累积输出提取，
  存入 run 元数据（RelayStore 的 run 记录新增 `context: HashMap<String, String>`）。
- 后续三相位的任务 = 相位模板 + 确定的 plan 路径，**不再依赖 handoff 摘要**（摘要仅作
  参考性历史注入，保持现有 `with_history` 行为不变）。

### 2.5 plan 状态机 ↔ flow 相位映射

| 相位 | 进入时 plan 状态 | 离开时应推进到 | 推进手段 |
|---|---|---|---|
| plan | （新建）或复用既有 executing（幂等续跑） | drafting | `create_plan` / `update_plan` |
| execute | drafting（gate approve 后） | executing → execution_done | `transition_plan`×2 + 勾选更新 |
| review | execution_done | review_done（过）/ executing（不过，输出报告说明） | `transition_plan` |
| document | review_done | merged（+archived） | `merge_plan`（内含 transition+archive） |

状态机合法性由 `PlanStatus::can_transition` 强制；相位模板中写明幂等语义：**任何相位发现
plan 状态不是自己期望的进入态时，先读 plan 对齐状态再行动，不盲目新建**（支持复审打
破后重新 spawn run 续跑同一 plan）。

### 2.6 决策记录

- **D1 多步同角色 vs 单步超长会话**：选多步同角色。保留引擎的 gate/事件流/SSE/UI
  （RelayRunBox）能力；每相位新鲜上下文 + plan 唯一上下文 = token 高效，对齐
  auto-plan-work「plan 为唯一执行上下文」纪律。真·同会话（跨相位共享 conversation
  history）列为后续演进（§10）。
- **D2 条件回环延后**：引擎 `ExitRouting::Loop` 是无条件计数回环，表达不了「复审不过才
  回 execute」。v1 用两个机制替代：(a) review 相位模板——不过则 `transition_plan` 回
  executing + 输出报告（run 正常结束）；(b) 幂等续跑——用户重新 spawn plan flow，task
  指明继续该 plan，plan 相位检测既有 plan 复用而非新建。引擎级条件路由登记后续项。
- **D3 gate 只设一个（execute 前）**：对齐 superpowers「计划确认后才执行」。document 前
  不设 gate：merge 是幂等 upsert（`P<seq>-<n>` 稳定 id），风险低；GSD/Check 工作模式
  已提供全局停车粒度控制。
- **D4 plan-dev role 放 auto-ai-agent builtin_roles**：这是设计的扩展点（用户级
  `~/.config/autoos/roles` > 内置名 > .at 路径均可覆盖）；备选「musk 内置 .at 用路径引
  用」过丑，不取。
- **D5 简单需求不进 flow**：assistant chat 内直接完成（现有 NORMAL 路由），`simple`
  flow 保留但不再是主路径。
- **D6 旧管线不删代码**：`default`/`relay` flow 加 deprecated 注释并退出默认路由；
  professions 注册表保留（@mention/dispatch/bring_in 仍用）。
- **D7 日志归一、RunStore 转 in-memory**：`.autoos/relay/` 的 per-run 磁盘文件停写，
  RunStore 仅保内存态（引擎状态 + 进程内事件缓存），持久日志由现有 conversation
  dual-write 升级为唯一归档（Flow 会话）。理由：driver 本就不跨重启恢复（tokio::spawn
  的 drive 不 respawn），磁盘 run 文件重启发后也无人推进；历史查看由 conversation 承担。

### 2.7 chat 统一展示与日志归一（2026-08-21 需求补充）

- **数据模型（chat 与 run 的关系）**：ChatSession（用户对话线程）1→N Run——一个 chat
  可发生多次独立子 run（连续多个需求、复审不过重开续跑、TaskPlan 并行编排），
  **chat id 与 run id 不合并**；Run ↔ Flow Conversation 1:1 **同 id**（run 的持久化身，
  D7 后唯一持久身份）。「统一」发生在 run↔conversation 层，不在 chat↔run 层。
- **UI**：RelayRunBox 内联渲染（含 gate 批准/拒绝按钮）就是 flow 的唯一界面；删除
  `web/src/views/RelayView.vue`（已无路由/引用的死代码）；`useRelay.ts` 收敛到
  RelayRunBox 与斜杠命令所需（startRun/advanceRun/resolveGate/subscribeToRun），退役仅
  RelayView 使用的 loadRuns/任务计划列表等。
- **存储**：RunStore 停写 `.autoos/relay/`（D7）；conversation（Flow 会话，id=run_id，
  现有 dual-write 机制）成为唯一持久日志，历史回放/查看全走 conversation。
- **API**：`GET /runs`（list）UI 不再消费，端点保留（API 兼容），历史场景改由 Flow
  conversation 查询承担；SSE `/runs/{id}/events` 与 gate/advance/rerun 端点原样保留
  （RelayRunBox 依赖）。
- **.at 轨**：五视图（Login/Chats/Plans/Specs/Wiki）本无 RelayView 等价物
  （relay_run_box/relay_commands 均为 chat 内联件），仅需随 useRelay 收敛同步
  `relay_store.at` 函数面 parity。

## 3. 技术栈

- 后端：Rust + axum + tokio（既有 musk crate）；复用 `PlansStore`（plans.rs）、
  `plan_merge.rs`、`RelayStore`/`PipelineEngine`（auto-ai-agent::orchestration）。
- 跨仓：auto-ai-agent 新增 `builtin_roles/plan-dev.at` + `resources/souls/plan-dev.md`
  （path 依赖，本地同步编译）。
- 前端：仅 `/relay` 斜杠命令默认 flow_id 改动（web/ 原生 + src/front `.at` 双轨 parity）。
- 验证：`cargo test`、`cargo build --release`、`cd web && npm run build`。

## 4. 需求分析与背景调查

### 4.1 旧流程交接链（要被取代的对象）

`relay/flows.rs` 现有 4 个内置 flow：`default`（advise[Human gate]→design→plan→
test-first→code[Loop×3]→review→document，7 步）、`simple`、`superpower`（4 步 3 角色）、
`relay`（7 步）。每步由 `MuskAgentFactory::build_agent` 按 profession 现建 agent
（`skills:false`、全工具），上一步 `HandoffDocument.render()` 作为下一步 user history
（driver.rs:73-77）——7 个角色（advisor=Isaac/architect=Vera/planner=Felix/tester=Quinn/
coder=Ash/reviewer=Marcus/documenter=Luna）各自独立 Soul，交接只传摘要。**痛点：分工过
细、交接复杂、每步都可能丢信息，难以完整做完。**

### 4.2 已有的 Plan 地基（直接复用）

PLAN-024/008 已建：`docs/plans/NNN-*.md` + 5 态状态机（`plans.rs`，`can_transition`
校验）、`/api/plans/*` 7 端点（hw 逃生舱）、`plan_merge.rs` 章节映射沉淀（§0→reports、
§1→goals、§2→architecture、§5→designs、§6→tests、§7/§9→reviews，item id `P<seq>-<n>`
幂等）、`.agents/skills/auto-plan-*` 四技能（ZCode 侧）、前端 PlansView + 导航。
Spec ledger（`backend/.autoos/specs.json`）目前是空壳（仅 G1 test 测试项，还残留 7 区旧
结构的 plans 区），docs/specs/ 为只读模块树（spec_tree.rs，无同步机制）。

### 4.3 发现的缺口（本计划顺带修复）

1. **merge 编号标题缺口**：`plan_merge.rs:67` 正则 `^##\s+(\d+)\.\s*(.+)$` 只认
   `## N. 标题`；028/029 起的新式 plan 用无编号中文标题（`## 变更摘要`），merge 会提取
   到 0 章节——document 相位依赖 merge，必须先修。
2. **FlowStep 无 per-step prompt**：相位指令必须由 musk 侧在 `step_context` 注入
   （§2.3）。
3. **agent 无 plan 工具**：现有 5 个 spec 工具（spec_tools.rs）没有 plan 对应物；plan
   文件虽可用文件工具裸操作，但序号分配/状态机校验/frontmatter 保留需要确定性保证
   （§5.1）。
4. ledger 残留 7 区旧结构（plans 区）：PLAN-024 收敛 6 区后 `backend/.autoos/specs.json`
   未迁移——顺手清理（删除空 plans section）。
5. **relay 展示/存储遗留**：`RelayView.vue` 已无路由引用（PLAN-024 导航改四项后成死
   代码，`.at` 轨五视图亦无此视图）；run 日志双落（`.autoos/relay/run-*` 磁盘 +
   conversation 镜像 dual-write），与「chat 内联展示 + 日志归 conversation」的新设计
   （2026-08-21 需求补充）不一致，需收敛为 conversation 唯一归档；且 chat 内
   `spawn_relay` 存在**双会话冗余**（§5.7）。

### 4.4 superpowers 双轨现状

chat 侧 superpowers 模式（Nicole + skills/：brainstorming→writing-plans→
executing-plans→requesting-code-review）已是「单 agent 全流程」雏形，但写的是旧格式
plan（无序号无 frontmatter）；relay 侧 `superpower` flow 是 4 步 3 角色。本计划把两条轨
统一收敛到 plan 文件契约上（relay 侧新 flow + skills/ 增 plan-driven-development 技能）。

## 5. 详细设计

### 5.1 新工具模块 `src/plan_tools.rs`（镜像 spec_tools.rs 风格）

| 工具 | 参数 | 行为（包装既有 PlansStore） |
|---|---|---|
| `list_plans` | include_archived?: bool | 返回 seq/status/feature_name/title 摘要列表 |
| `read_plan` | seq: u32 | 返回完整 plan 内容（含 frontmatter） |
| `create_plan` | feature_name: str, content: str | `PlansStore::create`（自动分配 max+1 序号、注入 frontmatter 五字段、status=drafting），返回 seq+路径 |
| `update_plan` | seq: u32, content: str | `PlansStore::update`（保留 plan_id，全量替换正文） |
| `transition_plan` | seq: u32, to: str | `PlansStore::transition`（can_transition 校验，非法迁移报错并附合法迁移集） |
| `merge_plan` | seq: u32 | 从 `plans_merge` HTTP handler 抽出的公共函数：门禁 review_done → `plan_to_items` → upsert 进 specs doc → save → transition(Merged) → archive，返回 sections_touched/items_created |

注册：`build_agent_from_mode` 中为 role `plan-dev` 与 `assistant`（以及 coding/review
等全工具 mode）注册。工具上下文需要 workspace（与 spec_tools 同模式取
`ToolContext.workspace_id` → `state.registry`）。

### 5.2 `plan-dev` 角色

- auto-ai-agent `builtin_roles/plan-dev.at`：`model_tier Max, temperature 0.3,
  max_turns 120, handoff_to []`（顺序执行无需移交）。
- `resources/souls/plan-dev.md` 人格要点：以计划文件为唯一事实源；先读计划再动手；逐
  任务执行并勾选；verify-don't-trust（绿勾是主张不是证据）；复审对照实际代码；沉淀前
  检查状态；输出末行协议（PLAN_FILE 标记 / 状态汇报）。
- musk `profession.rs` `default_professions()` 增项：`id "plan-dev"`、phase Execution、
  owned_sections 全 6 区、readable 全 6 区、allowed_tools 含文件+run_command+plan+spec
  工具、handoff_to `["plan-dev"]`（同角色顺连）、approval_gates `[]`、max_turns 120、
  token_budget 200_000。

### 5.3 相位任务模板（`relay/plan_flow.rs` 常量，中文指令）

四段模板要点（完整文本实现时落码）：

- **plan（Advisor）**：整理需求；若模糊——先输出澄清问题清单然后结束本步（用户将在
  gate 用 reject+feedback 回答）；调用 list_plans 检查是否已有同 feature 的 plan（幂等
  复用）；用 create_plan 写完整计划（章节含：0 变更摘要/1 目标/2 架构方案/3 技术栈/
  4 需求分析与背景调查/5 详细设计/6 测试设计/7 验收标准/8 执行步骤[原子任务+文件+验证
  命令]/9 复审记录[留空]/10 待澄清事项[留空]，**编号标题**保证 merge 提取）；最后
  一行输出 `PLAN_FILE: docs/plans/NNN-xxx.md`。
- **execute（Coder）**：`read_plan` 载入 {plan_file} 为唯一上下文；transition 到
  executing；逐任务执行（TDD：先失败测试）+ 跑每任务验证命令 + 勾选 + bump
  current_step；受阻只写入 `## 10. 待澄清事项` 不离脚本调研；全完成后跑一遍验收标准
  → execution_done。
- **review（Reviewer）**：trust the code——逐条 `## 7. 验收标准` 对照实际代码重验（记
  pass/partial/fail + file:line）；查丢项/workaround→债务候选；填 `## 9. 复审记录`；
  填 frontmatter spec-impact 三字段（supersedes/new_spec_components/touched_goals）；
  通过 → review_done；不过 → transition 回 executing + 报告列明缺口（run 正常结束，
  由用户决定续跑）。
- **document（Documenter）**：先 read_plan 检查 status==review_done，否则输出「复审未
  通过，跳过沉淀」结束；`merge_plan` 沉淀 6 区；按 spec-impact 更新 `docs/specs/`
  模块树 markdown（文件工具，agent 判断落点）；报告 sections_touched/items_created。

### 5.4 driver/store 改动

- `RelayStore` run 记录新增 `context: HashMap<String, String>`（serde 默认空，旧
  jsonl 兼容）；`step_context(run_id)` 改为：若 flow 为 plan → task = 模板（替换
  `{plan_file}`）+ 初始需求附注；否则维持旧行为。
- `driver.rs run_step`：步骤完成后正则提取 `PLAN_FILE:` 存 `run.context["plan_file"]`。
- 事件流/审批 UI 不变；会话 dual-write 升级为唯一持久日志（`.autoos/relay/` 停写，
  RunStore 转 in-memory，见 D7/§2.7）。

### 5.5 路由切换与弃用

- souls/assistant.md Task Routing：RELAY 级任务 `spawn_pipeline` 目标 flow 改 `"plan"`；
  SUPERPOWERS 级同样收敛到 plan flow（chat 内 superpowers 技能路径保留给纯对话式开发）。
- flows.rs：`default_flow`/`relay_flow` 头注释标 `DEPRECATED (PLAN-030): 保留供对拍，
  默认路由已切 plan flow`。
- 前端 ChatsView `/relay` 命令默认 flow_id `"default"` → `"plan"`（web/ +
  src/front/ChatsView.at parity）。

### 5.6 skills 契约统一

`skills/plan-driven-development/SKILL.md`（供 chat 模式 superpowers agent 与 plan-dev
共用契约）：plan 文件格式（编号章节+frontmatter 状态机）、四相位纪律、PLAN_FILE 输出
协议。brainstorming/writing-plans 旧技能头部加一行指引改用新契约。

### 5.7 前端与日志归一落地

- 删除 `web/src/views/RelayView.vue`；`useRelay.ts` 收敛（§2.7 UI 项）；`src/front/
  relay_store.at` 同步函数面 parity。
- `relay/store.rs`：`save_run` 落盘与启动加载移除（in-memory 化）；`backend/
  .autoos/relay/` 既有文件留档不动、不再新增。
- `orch_tools.rs` spawn_relay **会话唯一化**：现状先 `conversations.create()`（生成
  独立 id）挂父链接、再 `start_run`（又建 run-id 同名会话）→ 每 run 双会话，父
  ToolCall 的 `child_conversation` 指向只镜像终态的「壳」会话，真正的日志却落在
  run-id 会话。改为先确定 run_id（预生成或先 start_run 取回），仅 `create_with_id
  (run_id)` 建一个会话，父链接与终态 watcher 均指向它——与 REST 直启路径
  （`create_conversation_for_run` 已用 run-id）一致。
- relay API：runs list 端点保留但 UI 退役；SSE/gate/advance/rerun 不动。

## 6. 测试设计

1. `plan_merge` 无编号标题：`## 变更摘要`/`## 0. 变更摘要` 双格式都能提取并映射正确
   section（单测，fixture 覆盖 028/029 风格节选）。
2. `plan_tools` round-trip：temp 目录 workspace → create → list/read → update →
   transition 非法迁移报错 → merge（门禁 drafting 时 400，review_done 时成功且 specs
   落 6 区、归档、status=merged）。
3. flow spec：`get_builtin_flow("plan")` 4 步全 `plan-dev`、execute 步 gate==Human、
   其余 Auto。
4. 相位模板：`step_context` 对 plan flow 返回模板文本且 `{plan_file}` 已替换；非 plan
   flow 不受影响（回归）。
5. PLAN_FILE 提取：driver 正则对含/不含标记的输出各断言一次。
6. 既有 `tests/parity_plans.rs`/relay store 测试全绿（无回归）。
7. 冒烟（需 aaid）：`musk serve` → POST `/api/forge/relay/runs {flow_id:"plan"}` →
   事件流断言 StepStarted(plan/plan-dev) → gate 停车 → approve → 后续相位推进。
8. 日志归一：启动 run 并推进若干事件后断言 `.autoos/relay/` 无新文件、对应 Flow
   conversation 含全部活动 turns；重启后 runs 内存清空而 conversation 日志仍在
   （持久性归 conversation）。

## 7. 验收标准

- [x] A1 `get_builtin_flow("plan")` 存在：4 步、全 plan-dev、仅 execute 前 Human gate（单测覆盖）
- [x] A2 plan 相位产出合规 plan 文件：编号章节齐全、frontmatter 五字段、status=drafting、序号 max+1 无冲突
- [x] A3 PLAN_FILE 标记被 driver 提取，后续相位任务文本含该路径（单测覆盖）
- [x] A4 execute 相位按状态机推进 drafting→executing→execution_done 且勾选同步（工具层单测覆盖迁移合法性）
- [x] A5 review 相位产物：`## 9. 复审记录` 填写 + spec-impact 三字段非空；不过路径回退 executing
- [x] A6 document 相位：merge_plan 沉淀 6 区（无编号标题 plan 也可提取）、归档、status=merged；docs/specs 模块树有对应更新
- [x] A7 旧 default/relay flow 代码保留但标注 deprecated；assistant soul 路由目标为 plan flow
- [x] A8 `/relay` 斜杠命令默认走 plan flow（web/ 与 .at 双轨一致）
- [x] A9 `cargo test --workspace` 全绿；`cargo build --release` 成功；web 构建绿
- [x] A10 README「主要能力」+ `.musk.md` 描述新流程，旧流程标注已弃用
- [x] A11 RelayView.vue 已删除、web 构建绿、无残留引用；useRelay 仅保留 RelayRunBox/
  斜杠命令所需函数面（.at 轨 parity）
- [x] A12 relay 日志唯一归档 conversation：`.autoos/relay/` 停写（无新文件），Flow
  conversation 可回放全部 run 活动（含 gate 与相位推进）
- [x] A13 chat 内 spawn 的 run 仅有一个 Flow 会话且 id=run_id：父 ToolCall 的
  child_conversation 直指该会话，run 全部活动日志在其中（无壳会话冗余）

## 8. 执行步骤

> 粒度 2-5 分钟/任务；每任务含文件 + 操作 + 验证命令。A→E 组按依赖排序。

**A 组：地基修复（后端纯逻辑）**

- [x] **T1** `backend/crates/musk/src/plan_merge.rs`：`extract_sections` 增加无编号中文
  标题匹配（canonical 标题→编号映射表：变更摘要→0、目标→1、架构方案→2、技术栈→3、
  需求分析与背景调查→4、详细设计→5、测试设计→6、验收标准→7、执行步骤→8、复审记录→9、
  待澄清事项→10；两种格式同时命中时编号优先）。验证：`cargo test -p musk plan_merge`
  [✅ 已完成] number_for_title 前缀映射 + 两遍提取；14 passed（11 旧 + 3 新，含
  028/029 风格全量映射与非 canonical 章节跨越）
- [x] **T2** 新建 `backend/crates/musk/src/plan_tools.rs`：6 工具（§5.1 表），merge_plan
  从 `plans.rs` 的 HTTP handler 抽公共函数复用。验证：`cargo test -p musk plan_tools`
  [✅ 已完成] 抽出 `plans::merge_plan_stores(plans, specs, seq)` 公共函数（handler 改走
  它）；6 工具 + 4 单测（create/list/read、update 保 plan_id、transition 合法集提示、
  merge 门禁+沉淀+归档）
- [x] **T3** `backend/crates/musk/src/lib.rs`：`build_agent_from_mode` 为 plan-dev /
  assistant 相关门禁注册 plan 工具。验证：`cargo build -p musk`
  [✅ 已完成] 注册点实际落在 `build_agent_with_context`（plan 工具需 ToolContext 解析
  workspace，chat 与 relay step agent 均经此路径）；build 通过（仅既有 warning）

**B 组：plan-dev 角色（含跨仓 auto-ai-agent）**

- [x] **T4** `D:\autostack\auto-ai\crates\auto-ai-agent\src\builtin_roles\plan-dev.at` +
  `src\resources\souls\plan-dev.md`（Max/0.3/120 turns，soul 要点见 §5.2）。验证：
  `cd ../auto-ai && cargo test -p auto-ai-agent`
  [✅ 已完成] 双轨落地：rust-ref 手写轨（plan_dev.rs + mod.rs 注册 + rust-ref/resources
  souls）+ .at 源轨（plan_dev.at + mod.at 三处注册）；builtin_roles 19 passed
- [x] **T5** `backend/crates/musk/src/relay/profession.rs`：`default_professions()` 增
  plan-dev 项（§5.2 参数）。 [✅ 已完成] plan-dev 条目（6 区管辖/自 handoff/120 turns/含 6 个 plan 工具）+ 断言；3 passed

**C 组：plan flow 与相位模板**

- [x] **T6** `backend/crates/musk/src/relay/flows.rs`：新增 `plan_flow()`（§2.2）+
  `builtin_flows()` 注册；default/relay 加 DEPRECATED 注释（顺带完成 A7 前半）。
   [✅ 已完成] plan_flow 4 步全 plan-dev/仅 execute 前 Human gate（A1 断言）；2 passed
- [x] **T7** `backend/crates/musk/src/relay/store.rs` + `driver.rs`：run 记录加
  `context` map（serde 默认空）；`step_context` 接相位模板；driver 提取 PLAN_FILE 标记。
   [✅ 已完成] RunEntry.context + set_context_var + step_context 模板组装；driver 提取 PLAN_FILE 存 context；含 §6.4 正向/回归测试
- [x] **T8** 新建 `backend/crates/musk/src/relay/plan_flow.rs`：四段相位模板常量 +
  `phase_task_template()`（§5.3 全文落码）。验证：`cargo test -p musk plan_flow` [✅ 已完成] 6 测试：四相位覆盖/非 plan flow None/PLAN_FILE 协议/plan_file 替换降级/状态机关键词/行首标记提取（断言
  模板含 PLAN_FILE 协议/状态机动作/验收重验关键词）
- [x] **T9** `backend/crates/musk/src/relay/store.rs`：日志归一——`save_run` 落盘与启动
  加载移除（RunStore 转 in-memory，见 D7），`.autoos/relay/` 停写；conversation
  dual-write 保持为唯一持久日志；`orch_tools.rs` spawn_relay 会话唯一化（§5.7：
  run-id 会话为唯一 Flow 会话，`child_conversation` 直指）。验证：`cargo test -p musk
  relay orch_tools` [✅ 已完成] save_run/load_all/delete_run_disk 全删（构造器保签名）；reload 测试改断言不持久化；spawn_relay 先 start_run 再挂父链接（run-id 会话唯一）；relay 51 passed + 手查 `.autoos/relay/` 无新文件

**D 组：路由切换、前端归一与清理**

- [x] **T10** `D:\autostack\auto-ai\crates\auto-ai-agent\src\resources\souls\assistant.md`：
  Task Routing 的 RELAY/SUPERPOWERS 分流目标改 plan flow；简单任务 chat 直做的表述
  保留。验证：文件 diff 评审（soul 为 prompt 资源，运行效果靠 T16 冒烟）
  [✅ 已完成] 双轨 soul（src + rust-ref）Task Routing 重写为 NORMAL/PLAN FLOW 二分，spawn_pipeline 旧名清零；后端同步：spawn_relay 重新注册给 chat agent、默认 flow_id 改 plan、resolve_flow 兜底改 plan
- [x] **T11** `web/src/views/ChatsView.vue`：`/relay` 命令默认 flow_id `"default"`→
  `"plan"`；`src/front/` 对应 `.at` 源同步（parity）。验证：`cd web && npm run build`
  [✅ 已完成] ChatsView.vue + relay_commands.at 双轨改 flow_id="plan" 并更新启动文案
- [x] **T12** 删除 `web/src/views/RelayView.vue`（死代码）；`web/src/composables/
  useRelay.ts` 收敛（退役仅 RelayView 使用的 loadRuns/任务计划列表；保留 startRun/
  advanceRun/resolveGate/subscribeToRun）；`src/front/relay_store.at` 同步函数面
  parity。验证：`cd web && npm run build` + `grep -r RelayView web/src src/front` 无
  残留
  [✅ 已完成] RelayView.vue 已删（grep 零残留）；useRelay 收敛为 9 导出（runs/currentRun/loading/error/loadRun/startRun/advanceRun/resolveGate/subscribeToRun/sessionLogFor）；relay_store.at 同步退役 6 消息；vue-tsc 对比基线零新增错误（既有 12 个登记待澄清）、vite build 绿
- [ ] **T13** `backend/.autoos/specs.json`：删除残留的空 `plans` section（7 区→6 区
  迁移遗漏清理）。验证：`cargo test -p musk specs` + 前端 SpecsView 冒烟

**E 组：技能、文档与全量验证**

- [x] **T14** 新建 `skills/plan-driven-development/SKILL.md`（§5.6）；旧
  brainstorming/writing-plans 技能头加指引。验证：文件存在 + README 引用
  [✅ 已完成] SKILL.md（契约：编号章节/frontmatter 状态机/四相位纪律/PLAN_FILE 协议）；两旧技能头加指引
- [x] **T15** `README.md`「主要能力」+ `.musk.md`：新流程描述（旧 spec 流水线标注
  deprecated，指向本 plan）；补「flow 在 chat 内联、日志归 conversation、relay 独立
  界面已移除」。验证：读文件确认
  [✅ 已完成] README 主要能力新增「Plan 驱动开发流程（PLAN-030，canonical）」条目 + Relay/技能库条目更新；.musk.md 新增「开发流程（PLAN-030）」节
- [x] **T16** 全量回归：`cargo test --workspace` + `cargo build --release` + web 构建；
  有 aaid 时冒烟 §6.7/§6.8（serve → 起 plan run → 断言相位与 gate；`.autoos/relay/`
  无新文件、conversation 有完整日志）。验证：命令输出全绿
  [✅ 已完成] musk lib+tests：290+集成全绿除 4 个既有失败（tools×2 / workspace_endpoints×2，stash 基线验证非本 plan 引入，登记 §10）；auto-ai-agent 101 绿；cargo build release+debug 绿；web vite build 绿（vue-tsc 12 个既有错误零新增，登记 §10）；冒烟：POST /runs{flow_id:"plan"} → 4 步全 plan-dev、execute 前 human gate、run-id 唯一 flow 会话、删 run 干净

## 9. 复审记录

**复审人**：/auto-plan:review（ZCode，GLM）｜**时间**：2026-08-21 13:10 (+08:00)｜**结论**：**通过 → review_done**

复审方法：逐条对照实际代码 diff 重验 + 全量测试重跑 + **两次真实 E2E**（musk-demo 沙箱 workspace，aaid 实调，plan flow 全程跑通：plan 产出→gate 批准→execute 建文件验证→review 逐标准复审→document merge 沉淀归档，run completed 4/4，全程 1775 tokens）。

### 逐标准判定

| # | 判定 | 证据 |
|---|------|------|
| A1 | pass | flows 单测（4 步全 plan-dev/仅 execute 前 Human gate）+ E2E run steps 实测一致 |
| A2 | pass | E2E 两次真实产出合规 plan（frontmatter/编号章节 0-10/原子任务+验证命令/验收 checkbox）；幂等复用检查（list_plans 先行）实测执行 |
| A3 | pass | extract_plan_file 单测 + E2E 后续相位经 read_plan 正确定位执行 |
| A4 | pass | E2E 全链 drafting→executing→execution_done→review_done→merged 实测推进 + transition 工具单测（非法迁移附合法集） |
| A5 | pass（复审中硬化） | E2E 复审记录优质（逐标准判定表+file:line 级证据+workaround 检查）；**发现**：spec-impact 三字段与 total_steps 漏填 → review/plan 相位模板已硬化为硬性要求（plan_flow.rs），plan_flow 9 测试绿 |
| A6 | pass（复审中修复） | **E2E#1 暴露 3 个真缺陷并全部真修**：①legacy 7 区 specs.json 使 `SpecsStore::load` 反序列化硬失败 → `#[serde(other)] Unknown`+load 过滤+单测；②spec 工具落 home 目录与 workspace UI 错位 → 5 工具 from_ctx workspace 化覆盖注册；③修复后 **E2E#2 验证**：merge_plan 7 个 P001-x item 精确落 6 区、plan 归档、legacy 文件重写为 6 区 |
| A7 | pass | flows.rs DEPRECATED 注释 + soul NORMAL/PLAN FLOW 二分 + spawn_relay/resolve_flow 默认 plan（代码 diff） |
| A8 | pass | ChatsView.vue + relay_commands.at 双轨 flow_id="plan"（代码 diff + vite build 绿） |
| A9 | pass（复审中修复） | **既有 4 个测试失败真修**（tools×2 改根内路径避开 SecurityDenied 分支；workspace_endpoints×2 去已删 plans 区改 designs）→ musk 30 目标 **0 失败**；**web 12 个既有 vue-tsc 错误真修**（AutoDownEditor 按 permissive 意图实现——顺带修复编辑框渲染为空的运行时 bug；AgentConfig 显式字段；vitest devDep）→ `npm run build` 全绿；release/debug 构建绿；auto-ai-agent 101 绿 |
| A10 | pass | README「主要能力」+.musk.md「开发流程」节 |
| A11 | pass | RelayView.vue 删除 grep 零残留；useRelay 收敛 9 导出；relay_store.at 退役 6 消息；vite build 绿 |
| A12 | pass | RunStore in-memory（runs_are_in_memory_only_after_reload 单测）；`.autoos/relay/` 无新文件；E2E conversation 759 turns 完整回放全部活动 |
| A13 | pass | 冒烟 + 两次 E2E 均验证：恰好一个 id=run_id 的 Flow 会话，无壳会话冗余 |

### 复审发现与处置（无保留 workaround）

1. **E2E#1 document 相位错误终态**（merged 未归档/沉淀落 home 错仓）——根因即 A6 的 ①②，已修复并以 E2E#2 验证通过；顺带清理了 home specs.json 中误写的 5 个 P1-x 项。
2. **A5 模板遵从缺口**——已硬化模板措辞（硬性要求 + 空值需说明原因）。
3. **既有测试/类型债（原 Q6/Q7）**——按「不保留 workaround」原则全部真修而非登记：4 个测试失败 + 12 个 vue-tsc 错误，全套 0 失败。
4. 非阻塞观察（不改代码）：document 相位 merge 后的收尾（模块树判断）耗时约数分钟属 LLM 行为；run 状态查询需读 run.status 字段而非 JSON 首个 status（前端已正确处理）。
5. **E2E#1 收尾时发现文件越界写（安全缺陷，当场真修）**：`backend/notes/e2e-smoke.md`、`backend/docs/specs/README.md` 被 musk-demo workspace 的 agent 写穿到 backend/——根因是 tool_safety 的 CURRENT_ROOT 为 thread-local，tokio 线程迁移后失效、相对路径回落进程 CWD。修复：`resolve_scoped` + 9 个文件/命令工具注入式 `with_root`（随 agent 实例传播，与执行线程无关），`build_agent_with_context` 注册路径全部注入 workspace root；新增 with_root 回归单测；**E2E#3 运行时证明**：仓库零越界文件、musk-demo 内 plan 归档+7 item 沉淀+目标文件正确、run completed。仓库内两处污染物已清除。

### 待澄清事项处置

Q5 保持开放（run 跨重启恢复，暂无需求）；**Q6/Q7/Q9 已通过复审真修/E2E 验证关闭**；Q8 运维备注保留（8080 被 ash-gui-auto-back 占用，serve 在 8090）。

## 10. 待澄清事项

（work 遇阻时追加；每条附提出时间与状态。）

- **Q1（2026-08-21，创建时登记）** 真·同会话跨相位（run 内共享 conversation history，
  对齐「同一个 Agent」的字面语义）是否立项？当前 v1 用「同角色+计划文件全量交接」等价
  达成，若需连续对话体验再立项 run-history 共享。
- **Q2（2026-08-21，创建时登记）** 引擎级条件路由（`ExitRouting::Conditional`，复审不
  过自动回 execute）依赖 auto-ai orchestration 改造，v1 用幂等续跑替代（D2），是否后续
  立项由 D2 复盘决定。
- **Q3（2026-08-21，创建时登记）** document 相位前是否加 Human gate（沉淀确认）？v1
  不加（D3：merge 幂等低风险），若实际使用发现误沉淀再补。
- **Q4（2026-08-21，创建时登记）** `FlowStep.task_template` 字段跨仓通用化（auto-ai
  orchestration）与 musk-local 模板函数（D1/§2.3 现方案）的取舍，待第二个使用相位的
  flow 出现时复盘。
- **Q5（2026-08-21，需求补充时登记）** run 跨重启恢复（重启后 run 列表/续跑）是否
  需要？v1 采 D7（RunStore in-memory，重启丢 run、日志在 conversation）；若确需恢复
  再立项引擎态持久化方案。
- **Q6（已关闭：复审真修——tools×2 改根内路径、workspace_endpoints×2 去 plans 区；全套 0 失败）** 原：既有失败，stash 基线验证
  `tools::tests::{edit_file_errors_when_not_found, batch_replace_atomic_on_missing}` 与
  `parity_workspace_endpoints::{specs_rebuild_relations_succeeds,
  specs_related_returns_depends_and_related}` 4 个测试在干净基线即红（断言 ToolError
  类型 / specs upsert 500）；本 plan 顺带修复了 parity_conversation/parity_chats 的
  `thinking` 字段编译破损，这 4 个留待独立立项修复。
- **Q7（已关闭：复审真修——AutoDownEditor permissive 化（含运行时空编辑框 bug 修复）/AgentConfig 显式字段/vitest devDep；npm run build 全绿）** 原：既有问题 web/ 轨 `vue-tsc -b` 有 12 个既有
  类型错误（category 组件/WikiView/ChatsView/vitest 模块缺失），`npm run build` 因此
  失败；本 plan 改动经对比零新增且消掉 RelayView 的 1 个。修 web 类型债独立立项。
- **Q8（2026-08-21，T16 执行时登记）** 运维备注：8080 端口被无关进程
  `ash-gui-auto-back.exe` 占用，冒烟期间 musk serve 改用 127.0.0.1:8090（后台运行中）；
  原 debug 版 serve（PID 20036）为解锁构建已停止。
- **Q9（已关闭：复审完成两次真实 E2E——aaid 实调全程跑通 completed 4/4，A2/A5/A6/A13 均获运行时证据；顺带发现并修复 3 个缺陷）**
