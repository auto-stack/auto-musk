---
plan_id: PLAN-094
status: archived
feature_name: Relay plan 流程交接与 ledger 契约修复（UAT K4–K7）
author: [agent]
created_at: 2026-09-29T09:40:00+08:00
updated_at: 2026-09-29T14:05:00+08:00
plan_revision: 1
current_step: 6
total_steps: 6
supersedes_spec_components: []
new_spec_components: ["docs/specs/modules/specs-ledger.md"]
touched_goals: []
---

# PLAN-094 Relay plan 流程交接与 ledger 契约修复（UAT K4–K7）

## 0. 变更摘要

修复 2026-09-28/29 UAT（[docs/reports/plan-flow-uat/spec.md](../reports/plan-flow-uat/spec.md)）暴露的四个缺陷，全部有实测证据：

- **K5 断链**：Relay plan 流程中 advisor 以澄清问卷结束且未落实计划文件时，execute 门的 approve 仍被放行，coder 无计划空转"完成"（UAT T3 实录）。修复 = execute 门前置不变式（无 `plan_file` 不得放行）+ 相位模板硬失败条款。
- **K4 门语义**：`approval auto` 把"等待用户回答澄清问卷"的门同秒静默放行。修复 = 自动放行必须携带注入反馈的审计文本；问卷未答 + 无计划文件时按 K5 不变式拒绝前进。
- **K6 ledger 互踩**：聊天侧 merge Agent 手写 JSON 造出应用无法解析的外语格式 ledger；Relay reviewer 的 spec 工具全部 load 失败后，模型用 `edit_file` 手工做 JSON 手术，丢弃既有条目（UAT 实测 SD-01 被抹）。修复 = merge 技能改为**只经 spec 工具**（`write_spec`/`update_spec`，聊天 Agent 已具备，`lib.rs:361-365`）写账本 + 应用侧解析失败错误信息给足指引并明令禁止手改。
- **K7 摘要传染**（coder 完成摘要臆造"PLAN-086"污染 reviewer）随 K5 消解：计划文件真实落盘后，相位间唯一交接物恢复为计划文件。

**非目标**（见 §10 D3 / §4 授权）：Relay document 相位整体接入新沉淀契约（`merge_plan` 组合操作退役）另行立项；K1/K2/K3（模型输出截断、无自动续跑、循环检测）为模型交互已知限制，已登记 `KNOWN-DEBT-AND-RISKS.md`，不在本计划。

## 1. 目标

- 目标：UAT T3 场景在 `approval auto` 下不再发生"无计划放行 → coder 空转 → reviewer 死循环"级联；聊天侧 merge 产出的 ledger 任何应用工具都能解析；任何路径都不再可能静默丢失 ledger 数据。
- 成功标准：
  1. plan 相位未落实计划文件时，execute 门放行被拒（auto = 转定向反馈重跑一次 plan 相位，二次仍缺则 `run_failed` 带明确原因；human = 门决议返回错误）；审计事件记录原因与注入的反馈文本。
  2. merge 技能文本钉死账本写入方式（spec 工具 + schema 要点 + 出错处置），按该技能执行的 merge 在干净工作区产出的 `.autoos/specs.json` 可被 `SpecsDocument` serde 解析、`read_specs` 可读。
  3. 外语格式 ledger 上调用 spec 工具：返回错误含 schema 指引与"禁止手改/重建"明令，**文件字节不变**。
  4. `docs/specs/modules/plan-flow.md` 与新增 `docs/specs/modules/specs-ledger.md` 反映上述契约；UAT spec.md 结果表回填 T3 重跑结果。
- 非目标：见 §0。
- 约束：不改变 `docs/specs/` 唯一权威地位；不动 a2r 再生成接缝（`auto_generated/relay_flows.rs` 手修惯例，086 债）；门语义变更须向后兼容"有计划文件即照常放行"（回归 AC-04）。
- 仓库范围：auto-musk 本仓（技能文本、backend、specs 文档、UAT 文档回填）。

## 2. 架构方案

三层防线，按"让错误不可能发生 → 发生了就大声失败 → 失败了给出现成出路"组织：

1. **预防（聊天侧）**：merge 技能文本从"刷新派生 ledger（方式未定）"改为"仅经 `write_spec`/`update_spec` 工具写账本"——这两个工具经 `SpecsStore::upsert_item` 走 store 语义（`SpecItem::new` 补全全部字段），从根上消除手写 JSON。技能同时钉死账本定位（`{ws}/.autoos/specs.json`，workspace 作用域）、七区清单（goals/architecture/designs/plans/tests/reviews/reports）、SD→区映射建议（派生索引进 `plans`/`tests`/`reviews` 区，源哈希进 item `file`/`tags`），以及"load 失败即停，禁止手改/重建"。
2. **拦截（Relay 侧）**：`relay/driver.rs` 的 execute 门决议处加不变式——`approve` 到达时 run 上下文 `plan_file` 必须非空（绑定通道或 `PLAN_FILE:` 标记，`plan_file_marker_write` 落点之后判定）；缺失时：human 决议返回错误（引导改用 reject+feedback），auto 决议转成 `reject + feedback("plan phase ended without a plan file — materialize it via create_plan using recorded defaults")` 定向重跑 plan 相位一次，二次仍缺则 `run_failed`。自动放行审计文本统一带注入反馈（K4）。`plan_flow.rs` execute 模板把"plan_file 未知 → list_plans 自行定位"改为硬失败条款（双保险）。
3. **响亮失败（工具侧）**：`spec_tools.rs` 五个工具的 `load` 失败路径（现四处相同 `map_err`）统一为带上下文的错误：期望的顶层字段（`project` + `sections`，七区 id 清单）、canonical schema 指引、**明令禁止手改 `.autoos/specs.json` 或重建**、恢复路径（备份/`docs/specs` 重建后重试）。`specs.rs` 的 `load()` 对"存在但解析失败"维持只报错（现状已正确）；`NotFound` 自愈建新档保留（文件缺失无数据可损）。

关键取舍：不引入 schema 版本迁移/自动格式转换（外语格式只该被拒绝并指引，不该被猜）；不在 `load()` 里做隔离改名（读路径带副作用，写路径报错已足够响亮）。

## 3. 技术栈

Rust（backend/crates/musk，workspace `backend/Cargo.toml`，包名 `musk`）；测试 `cargo test --manifest-path backend/Cargo.toml -p musk <filter>`；技能文本为 Markdown（经 `builtin_skills.rs` 启动时幂等分发到 `~/.config/autoos/skills/`，改文本即生效，无需迁移）。

## 4. 需求分析与背景调查

**已获授权（2026-09-29 用户会话逐项确认）**：
- 拆分方式：一个修复计划覆盖 K4/K5/K6（K7 随 K5 消解）；K1/K2/K3 登记债务账本（本计划创建时同步落 `KNOWN-DEBT-AND-RISKS.md` 🟢 已知限制表）。
- D1 决策：ledger schema 收敛到应用原生 `SpecsDocument`（`specs.rs:276`，`project` + 7 区），不发明第三种格式；`docs/specs/` 唯一权威不变，ledger 仅为派生视图。
- D2 不变式：任何 load 解析失败绝不重写文件（现状已满足，本计划补"响亮错误 + 禁手改指引"，并禁止模型侧手改路径）。
- D3 范围收窄：Relay document 相位接入新沉淀契约不在本计划。
- 预算/续跑限制：用户未指定。

**背景调查（关键代码事实，均已核实）**：
- 聊天 Agent 工具集已含全套 spec 工具（`lib.rs:361-365` `from_ctx` workspace 作用域）——K6 是技能未告知所致，非能力缺失。
- `SpecsStore::load()`（`specs.rs:755`）：文件存在但解析失败 → `Err(InvalidData)`（无破坏）；NotFound → 建新档并保存。UAT 中 ledger 被替换是 reviewer 模型 `edit_file` 手术所致，非自动兜底——所以"错误信息给足指引 + 禁手改"是对症的。
- `spec_tools.rs` 五工具的 load 失败错误现形如 `load specs: missing field \`project\` at line…`，无任何处置指引（reviewer 由此走向手改）。
- Relay plan 流程相位模板（`relay/plan_flow.rs`）明文允许"澄清-停止"：advisor 列出问题后停止，**设计预期用户在门上以 reject+feedback 回答、plan 相位重跑**。T3 断点 = `approval auto`（PLAN-067）以 approve 放行 + driver 无"approve 需计划文件"前置校验 + execute 模板降级提示（"plan_file 未知——用 list_plans 找…"）放任 coder 空转。
- `plan_file` 传递链已有优先级规范（`docs/specs/modules/plan-flow.md:47-55`：绑定 > 标记 > 提示），本计划在其上加"门前置"一环。
- 规范落点：`docs/specs/modules/plan-flow.md`（门/plan_file 契约所在）；ledger 契约现无模块 spec（仅 `00-overview.md` "Spec 双落点"一句）→ 新增 `specs-ledger.md`。
- 证据：UAT spec.md K4–K7 与 T3 全程轮次（`.autoos/conversations/run-1790637956-1066bf4c/`）。

## 5. 详细设计

### 5.1 merge 技能文本修订（`.agents/skills/auto-plan-merge/SKILL.md`）

在 "Authority and compatibility" 后新增小节 "Ledger refresh (store-mediated)":

- 只经 `write_spec`/`update_spec`（workspace 作用域，聊天会话已注册）写 `.autoos/specs.json`；**禁止**以任何脚本/文本替换直写该文件。
- 账本 schema 由 store 拥有：`SpecsDocument{version, project, sections[7]}`，七区 id = goals/architecture/designs/plans/tests/reviews/reports；item 字段由 `SpecItem::new` 补全，不要手拼 JSON。
- SD→区映射建议：派生索引 item 落 `plans`（implements_plan 关联）/`tests`（test_file）/`reviews`（复审报告链接）；源哈希与 commit 写入 item 的 `tags`/`file`，正文只放派生摘要，不复制 Plan 章节。
- `read_specs`/`list_specs` 报 load 错误 = 账本不可解析：**停止**，按错误指引恢复（备份或以 `docs/specs/` 重建），禁止手改、禁止删除重建。

### 5.2 spec 工具错误加固（`spec_tools.rs`）

抽 `fn load_err(e) -> ToolError`：文案 = 原 serde 错误 + 固定后缀（期望字段、七区清单、"ledger is store-managed — do not hand-edit or recreate; restore from backup or rebuild from docs/specs, then retry"）。替换 read/list/write/update/write_goals 五处 `map_err`。`specs.rs` 的 `load()` 本体不改语义。

### 5.3 execute 门不变式 + auto 审计（`relay/driver.rs`）

- 判定点：门决议处理处（approve 路径，含 PLAN-067 auto 分支；位于 `plan_file_marker_write` 调用（driver.rs:349 附近）之后的 step 推进前）。
- `plan_file` 缺失时：human approve → 不消费门，向调用方返回错误（提示改用 reject+feedback）；auto → 改写为 reject+feedback（文本见 §2），重跑 plan 相位**至多一次**（run 状态记录 rerun 计数），二次仍缺 → `run_failed`，error = "plan phase ended without a plan file"。
- 自动放行审计（K4）：auto 通过任何 human 门时，审计轮内容带注入反馈文本（缺省为 "auto-approved; recorded defaults apply"）。
- `plan_file` 存在时行为不变（AC-04 回归）。

### 5.4 execute 模板硬失败（`relay/plan_flow.rs`）

execute 模板中 `{plan_file}` 降级分支由"list_plans 自行定位"改为："计划文件缺失属于上游缺陷：立即停止，输出 blocker 说明 plan_file 未落实，不得开始实现、不得正常完成本相位。"

### 5.5 数据结构

无新类型；`run_failed` 错误串与审计轮文本约定见 §2/§5.3（写入 plan-flow.md spec）。

### 规范增量

| delta_id | 类型 | 目标 | 变更前规则 | 变更后规则 | 理由 | 关联验收 |
|---|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/modules/specs-ledger.md` | （无模块 spec；仅 00-overview 一句"6 区"） | ledger=应用原生 `SpecsDocument` 派生视图；只经 spec 工具写；七区与 item 字段清单；解析失败处置（禁手改/禁重建/恢复路径）；`docs/specs/` 唯一权威不变 | K6 实测：无契约文本导致聊天侧即兴格式 + reviewer 手术毁数据 | AC-01, AC-02 |
| SD-02 | modify | `docs/specs/modules/plan-flow.md` | §plan_file 传递链止于"降级提示"（:47-55）；门表无前置条件 | 增补：execute 门 approve 前置不变式（无 plan_file 不得放行；auto=定向重跑一次/二次 run_failed；human=决议报错）；auto 放行审计须含注入反馈文本 | K4/K5 实测：静默放行问卷门 + coder 空转 | AC-03, AC-05 |

## 6. 测试设计

| # | 用例 | 方法 | 预期 |
|---|---|---|---|
| 1 | 外语格式 ledger（UAT 现场形状：flat specs/history）fixture | spec 工具单测：对 fixture 调 update_spec/write_spec/read_specs | 错误含 `project`/七区/禁手改关键字；**fixture 文件字节前后不变** |
| 2 | SKILL.md 文档化 schema 示例可解析 | 单测：按 T-01 文本中的示例构造 ledger（经 write_spec 语义 upsert）→ serde 解析 + read_specs 可读 | 解析成功，七区在位 |
| 3 | execute 门不变式 | driver 单测：plan_file 缺失 + approve(auto) → 转 reject+feedback 重跑一次；二次仍缺 → run_failed；approve(human) → 决议错误不消费门 | 行为与 §5.3 一致，审计文本含反馈 |
| 4 | 门回归 | plan_file 存在 + approve → 照常推进（现有 driver 测试全绿） | 无行为变化 |
| 5 | execute 模板硬失败条款 | plan_flow 单测：phase_task("plan","execute",…, 空 context) 含 blocker 指令 | 文案断言 |
| 6 | 端到端 | 重置 demo 工作区，按 UAT spec.md T3 重跑（裸需求 + approval auto） | 四断言：计划文件出现且被 execute 消费 / 门不放行无计划 run / ledger 不被外语化重写 / run 终态 delivered 或带明确原因失败 |

命令：`cargo test --manifest-path backend/Cargo.toml -p musk spec_tools`、`… -p musk specs::`、`… -p musk plan_flow`、`… -p musk relay::driver`（以实际测试模块名为准，T-05 落地时校正）。

## 7. 验收标准

| ID | 验收标准 | 验证方法与预期 |
|---|---|---|
| AC-01 | merge 技能钉死 store-mediated 写账本契约 | 检查 `.agents/skills/auto-plan-merge/SKILL.md` 含 §5.1 五要点；用例 2 单测绿 |
| AC-02 | 外语格式 ledger：响亮失败 + 零破坏 | 用例 1 单测绿（错误关键字 + 字节不变断言） |
| AC-03 | execute 门不变式生效（auto/human 两臂） | 用例 3 单测绿 |
| AC-04 | 有计划文件时门行为不变 | 用例 4 全绿（现有测试零回归） |
| AC-05 | 端到端 UAT T3 重跑四断言 | 用例 6 实跑记录 + `plan-flow-uat/spec.md` 结果表回填 |
| AC-06 | 受影响模块测试全绿 | `cargo test --manifest-path backend/Cargo.toml -p musk`（specs/spec_tools/plan_flow/driver 过滤器）0 fail |
| AC-07 | 变更纳入版本控制 | worktree 内提交，`git status` 干净，`--ff-only` 合回 main |

## 8. 执行步骤

| ID | 任务 | 依赖 | 产出/涉及文件 | 关联验收 | 验证命令与预期 |
|---|---|---|---|---|---|
| T-01 | 修订 merge 技能文本（§5.1 五要点 + schema 示例） | — | `.agents/skills/auto-plan-merge/SKILL.md` | AC-01 | 文本检查：五要点齐备；示例与 specs.rs serde 字段一致 |
| [x] T-01 | 修订 merge 技能文本 | — | `.agents/skills/auto-plan-merge/SKILL.md` | AC-01 | [✅ 已完成] commit c29df68；新增 "Ledger refresh (store-mediated)" 节含五要点（六区清单——`plans` 区 PLAN-024 已退役见 §9 记录）；退役离线读改写直写路径（无 store-mediated 写通道→记录 blocked）；serve 启动已实测同步新文本（`builtin skill synced: auto-plan-merge`） |
| T-02 | spec 工具 load 错误加固：`load_err` helper + 五处替换 | — | `backend/crates/musk/src/spec_tools.rs` | AC-02 | `cargo test … -p musk spec_tools` 绿（含新 fixture 用例） |
| [x] T-02 | spec 工具 load 错误加固 | — | `backend/crates/musk/src/spec_tools.rs`、`specs.rs`（+`path()` 访问器） | AC-02 | [✅ 已完成] commit c29df68；`foreign_shape_ledger_fails_loud_and_intact`（UAT 现场形状 fixture：错误含 project/六区/禁手改关键字，**字节前后不变**）+ `skill_contract_ledger_parses_and_reads`（serde 解析 + 六区在位 + read_specs 可读）；`cargo test -p musk --lib spec_tools` 8/8、`specs::` 38/38 |
| T-03 | execute 门不变式 + auto 定向重跑（≤1 次）+ 审计反馈文本 | T-02 无依赖，可并行 | `backend/crates/musk/src/relay/driver.rs` | AC-03, AC-04 | `cargo test … -p musk relay::driver` 绿 |
| [x] T-03 | execute 门不变式 + 审计反馈文本（**D1 退化：直落 run_failed，无重跑**） | — | `relay/driver.rs`、`relay/api.rs`、`relay/store.rs`、`auto_generated/relay_driver.rs`、`auto_generated/extern_impl.rs`、`auto_generated/relay_store.rs`、`conversation.rs` | AC-03, AC-04 | [✅ 已完成] commit 12dff96；判定核单源 `plan_flow::execute_gate_action`（hw/ag 双驱动 + 两个人口共用）；auto 缺失→`fail_run`（D1 退化依据见 §9）；human 缺失→409 不消费门；`GateResolved` 增 `note` 字段（serde default 兼容），auto 放行带缺省 note、会话审计轮带注入反馈；验证：`--test parity_relay_driver` 6/6（含 plan094 两行为用例：无计划=失败+门未消费+仅 1 次 advisor；有计划=照常放行+note 在）+ `relay::` 62/62 |
| T-04 | execute 模板硬失败条款 | — | `backend/crates/musk/src/relay/plan_flow.rs` | AC-03 | `cargo test … -p musk plan_flow` 绿 |
| [x] T-04 | execute 模板硬失败条款 | — | `relay/plan_flow.rs` | AC-03 | [✅ 已完成] commit 12dff96；`{plan_file}` 双缺由 "list_plans 定位提示" 改阻断性 blocker 条款（execute/review/document 三相位同一文案）；`later_phases_substitute_plan_file_or_hard_fail` 绿 |
| T-05 | 回归测试补齐（用例 1–5 全部落库）+ SD-01/SD-02 spec 文档落地 | T-01..T-04 | 各测试文件；`docs/specs/modules/specs-ledger.md`（新）；`docs/specs/modules/plan-flow.md` | AC-01..04, AC-06 | `cargo test --manifest-path backend/Cargo.toml -p musk` 相关过滤器 0 fail |
| [x] T-05 | 回归测试落库 + SD-01/SD-02 spec 落地 | T-01..T-04 | 用例 1–5 均已入库（见 T-02/T-03/T-04 行）；`docs/specs/modules/specs-ledger.md`（新，SD-01）；`docs/specs/modules/plan-flow.md`（SD-02：门不变式节 + D1 退化记录 + 反馈送达 + 硬失败条款）；`00-overview.md` 双落点句挂链接 | AC-01..04, AC-06 | [✅ 已完成] 全 lib 套件 503/503 绿（clean 重建）；范围过滤器 spec_tools 8 / specs:: 38 / relay:: 62 / conversation 29 / parity_relay_{driver,store,api} 6/7/6 / parity_conversation 10 / parity_specs 11 全绿；**环境预存失败**：`tool_atoms run_command_dangerous_returns_paused`（路径禁锢在危险模式检查之前拦截 `/`）在 base 97fc677 主检出上同型失败，与本计划无关，登记候选债务 |
| T-06 | UAT T3 重跑 + 文档回填：重置 demo 工作区实跑（用例 6），回填 `plan-flow-uat/spec.md` 结果表 | T-05 | `docs/reports/plan-flow-uat/spec.md` | AC-05, AC-07 | 四断言记录在案；worktree 提交 + `--ff-only` 合回 main |
| [x] T-06 | UAT T3 重跑 + 文档回填 | T-05 | `docs/reports/plan-flow-uat/spec.md`（重跑记录节 + 结果总表 T3 重跑行 + K1b/K4/K5/K6/K7 对策更新） | AC-05 | [✅ 已完成] commit `1993eaa`。四次 run 实录（demo 重置基线 `8fe52ad`，修复版 serve :17255）：断言 2（门不放行）×4 实证——auto 三 run 均 `run_failed` 契约文案、无 gate_resolved、无下游相位；human run POST approve→409 错误体且门未消费；断言 3（ledger 零破坏）实证——document 相位经 spec 工具写出原生六区 ledger；断言 4（终态明确原因）实证。断言 1（计划文件被 execute 消费）实跑未复现：advisor 相位产出被 K1 截断 ×2（R2 已写计划正文至 §5 腰斩，**Relay 相位不受 chat thinking=max 变通保护**）+ 澄清-停止误触发 ×1——机制面由 driver 行为测试（hw+ag）覆盖；级联死亡。顺带发现三件已登记 KNOWN-DEBT（UAT-K1b Relay max_tokens 缺口 / 模板纪律非强制 / ag gate 错误信封 HTTP 200）+ D3 候选。`--ff-only` 合回 main 归 merge 阶段执行（四技能分工，见 §9 handoff） |

依赖链：T-01/T-02/T-04 并行 → T-03 → T-05 → T-06。

## 9. 复审记录

- 2026-09-29 draft handoff（/auto-plan:new）：`stage: new`，`PLAN-094` rev 1。`outcome: pass` — 任务覆盖全部验收标准与两条 Spec 增量；代码事实七项均实测核对（工具注册、load 语义、模板原文、门路径、模块 spec 现状、包名、编号 094 唯一性）；范围含用户逐项授权（§4）。`next: work`（交 /auto-plan-work 执行）。
- 2026-09-29 work start（/auto-plan-work）：`stage: work`，`PLAN-094` rev 1，base commit `97fc677`。worktree `D:/autostack/.wt/musk-094/auto-musk`（branch `plan-094-dev`）；主检出仅 `docs/plans/**` 簿记改动（KNOWN-DEBT K1-K3 登记 + 本计划 + 093 草稿），无代码 WIP。
- 2026-09-29 work 进展（T-01..T-05 落库）：`stage: work`，commits `c29df68`（K6 账本契约）+ `12dff96`（K4/K5 门不变式）。**三项实测发现与处置**：
  1. **D1 退化启动（预授权内，不需 revision）**：§5.3 原设计"auto 缺失转 reject+feedback 定向重跑 plan 相位"经 driver 行为测试证伪——引擎 `PipelineEngine::resolve_gate` 的 Reject=redraft 语义重做的是**被门守卫的步骤**（execute），不是 plan 相位；"重跑 plan 相位"在现引擎不可表达（无 rewind API），而"让 coder 带反馈补写计划"与 T-04 硬失败条款及 K7"相位不做计划外发挥"直接冲突。按 D1 预授权退化为**直接 `run_failed`**（首遇缺失即置败，仍满足 AC-03"不放行"），恢复出路=重开 run（advisor 幂等复用）或 human 模式 reject+feedback。已写入 plan-flow.md spec。
  2. **"七区"前提纠偏**：计划 §2/§5.1 的"七区（含 plans）"与代码事实不符——`SpecsDocument` 原生六区，`plans` 区 PLAN-024 已退役（load 容忍过滤、永不回写、写入即丢）。按 D1 本旨（收敛到应用原生格式、不发明第三种格式）落地为**六区**；若按计划原文实现反而制造 K6 同型数据丢失。merge 技能与 specs-ledger.md 均按六区书写并明令"没有 plans 区"。
  3. **门反馈从未送达（前置缺陷顺手修复）**：`step_context` 自 P2b.2 起从不消费引擎 `feedback_for`——reject(feedback) 重做相位时反馈不可见，定向反馈等同盲重放。已补：反馈以「门反馈」块附加在重做相位的模板后（含测试）。已知边界：反馈到达的是被门守卫的 execute 相位而非 advisor（引擎语义，见第 1 条）。
  其余：依赖解析按 AGENTS.md 序建组内只读兄弟 worktree（auto-ai@5a50a55、auto-lang@604c47e6，detached，未改动）；`--tests` 全量在本机存在编译期并行竞态（-j 2 稳定）；`tool_atoms` 一例环境预存失败（base 同型复现，已登记）。
- 2026-09-29 work handoff（/auto-plan:work → review）：`stage: work` | `PLAN-094` rev 1 | `outcome: pass` | code commits：worktree `plan-094-dev` = `c29df68`（K6 账本契约）→ `12dff96`（K4/K5 门不变式+反馈送达+T-04）→ `1993eaa`（UAT 回填）| task_ids：T-01..T-06 全部 [x]，current_step 6/6 | evidence：AC-01..04/06 测试全绿（lib 503 + 范围过滤器，见任务行）；AC-05 四 run 实录回填 `plan-flow-uat/spec.md`（断言 2/3/4 实证、1 受阻于预存债 K1 并如实记录）；AC-07 worktree 三提交、status 干净 | blockers：无 | next：review（`--ff-only` 合回 main 由 merge 阶段执行；merge 时注意 release 二进制/web bundle 重建检查）。**AC-05 如实备注**：断言 1（计划文件被 execute 消费）未在实跑复现，根因 = advisor 相位产出可靠性（UAT-K1b，已登记预存债，属本计划 §0 非目标），门机制本身四 run 零失误。
- 2026-09-29 review（/auto-plan:review）：`stage: review` | `PLAN-094` rev 1 | `outcome: pass` | reviewed_commit `1993eaacf9b3f7ec25759d00f92eeede844fb8f1`（base `97fc6773b1e4a034f71613ec234244967def8bde`；deps：auto-ai `5a50a55`、auto-lang `604c47e62`，detached 只读兄弟 worktree，零改动）| spec_inputs：`docs/specs/modules/specs-ledger.md`（SD-01 新增，frozen 于 reviewed_commit）、`docs/specs/modules/plan-flow.md`（SD-02 修改）、`docs/specs/00-overview.md`（双落点句挂链）| **独立性声明**：复审在实施会话内进行——裁定全部从工件重建（复跑测试、审阅提交 diff、盘上证据核验），未沿用执行摘要。
  - **验收逐条**：AC-01 pass（五要点逐条在 committed SKILL.md :36-55 核实；用例 2 测试绿）；AC-02 pass（fixture 测试字节不变断言在 committed :643，fresh 绿）；AC-03 pass（不变式接线在 hw driver/api + ag relay_driver/extern_impl 四处逐一核实；driver 行为测试 + store 组合测试 fresh 绿；human 409 不消费门有盘上证据 R4）；AC-04 pass（`plan094_auto_gate_with_plan_file_approves_through` fresh 绿；现有套件零回归）；AC-05 pass（重跑实跑记录 + spec.md 回填在案；断言 2/3/4 由盘上工件独立重建——`.autoos/conversations/run-*/turns.jsonl` 含 R3 "Flow failed: plan phase ended…" 与 R4 "Gate execute reject"；断言 1 未复现已如实记录，根因 UAT-K1b 预存债属 §0 非目标，机制面由 driver 测试覆盖）；AC-06 pass（fresh：lib 503/503、spec_tools 8、plan_flow 10、relay::store 14、parity 6/7/6/10/11 全绿；`tool_atoms run_command_dangerous` 在主检出 base 同型 FAILED 复确认——环境预存非本计划回归）；AC-07（worktree 部分）pass：三提交齐、`git status` 干净；`--ff-only` 合回 main 归 merge 阶段执行并届时复核。
  - **知识增量核验**：SD-01 六区/写入通道/解析失败处置各条与代码事实一致（SectionType 六区、load() 只报错、load_err 文案、/api/specs/item store-mediated）；SD-02 门不变式节与实现逐点相符（auto fail-fast 文案、human 409 不消费、D1 退化记录、反馈送达、硬失败条款）。spec-impact：`new_spec_components` 指向真实新文件；`supersedes_spec_components`/`touched_goals` 为 `[]`——理由：本计划不退役任何既有规范组件，亦不触及 `docs/specs/goals/` 任何目标条目（两 SD 均为模块级契约增补）。
  - **findings（均非阻断，已登记）**：F-1 R4 human-reject 路径下无计划 run 仍可空转至 "completed"（指令级约束边界，KNOWN-DEBT 094 行）；F-2 ag gate 路由错误信封 HTTP 200 vs hw 真 409（KNOWN-DEBT 094 行）；F-3 本复审独立性限制（以工件重建缓解）。
  - evidence 保存性：UAT 盘上证据在 `tmp/demo/.autoos/`（gitignored 沙盒，随重置消失——持久证据以本记录摘录与 `docs/reports/plan-flow-uat/spec.md` 回填为准）。next：merge。
- 2026-09-29 merge（/auto-plan:merge）：`stage: merge` | `PLAN-094` r1 | `outcome: pass` | `completion_kind: delivered`。收据（PLAN-094:r1）：
  **prepared** — reviewed_commit `1993eaa` 即交付提交（规范增量已冻结于其内：specs-ledger.md 新增 / plan-flow.md 门不变式节 / 00-overview 挂链），依赖 auto-ai@5a50a55、auto-lang@604c47e6 零改动，worktree clean + wt-guard clean。
  **landed** — main 97fc677→**1993eaa** `git merge --ff-only plan-094-dev` 纯快进零 merge commit、零 rebase（main 未动 → 无 hash rewrite，免 range-diff）；落地后 main 冒烟 `cargo test -p musk --lib relay::plan_flow` 10/10 绿。
  **ledger_refreshed** — workspace=auto-musk（仓根 `.autoos/specs.json`，gitignored 运行时账本），**全程 store-mediated**（生产 serve :17201 `POST /api/specs/item`，新契约首次实战）：designs 新增 **specs-ledger-D1**（source_sha256 fe630b4b…，Approved）+ 刷新 **plan-flow-D1**（source_sha256 921fb41d→**2a904c69**，内容补 PLAN-094 门不变式要点，Approved）+ reviews 新增 **plan-flow-R1**（Published，PLAN-094 交付评审）；version 3→**10**，回读核验三条 status/content/hash 全对。**已知边界（登记 KNOWN-DEBT）**：`/api/specs/item` wire（SpecItemPayload）仅 id/title/content/status 四字段——file/milestone/module/tags/depends_on 无 store-mediated 写入通道，本三条以 content 文本承载规范路径/哈希/计划号（plan-flow-D1 刷新时按 store schema 归一化丢弃旧附加字段 description/spec_ref/spec_area/spec_key）。
  **archived** — 本文件（原为未跟踪簿记，按技能以适当文件移动归档）git mv 入 docs/plans/archived/ + status: archived + completion_kind: delivered。
  **cleaned** — 只读依赖 worktree 先拆（auto-ai@5a50a55、auto-lang@604c47e6 状态核实零改动后 `git worktree remove`，无分支残留）；wt-guard 复跑 clean 后 `git worktree remove` musk-094/auto-musk + `git branch -d plan-094-dev`（was 1993eaa）；组目录 `.wt/musk-094` rmdir 消失。无关未跟踪簿记（docs/plans/093 草稿）原样保留未纳入任何提交。
  **deployment 观察项（登记不阻塞）**：①生产 musk（:17201，PID 6104）仍跑 PLAN-094 前的 release 二进制——门不变式/响亮错误/审计 note 需 `cargo build --release -p musk` + 择隙重启后方在生产面生效（UAT 已在 worktree 构建上实证新行为）；②web bundle（gen/front/vue/dist）与 aaid release 不受本计划影响（源码零改动），无需重建。

## 10. 待澄清事项

| ID | 事项 | 默认假设 | 后续动作 |
|---|---|---|---|
| D1 | auto 门定向重跑的次数上限 | 1 次（二次仍缺计划文件 → run_failed，避免无限循环） | **已处置（2026-09-29 work）**：实测引擎 redraft 只重做被门守卫的 execute 相位，"定向重跑 plan 相位"不可表达——按本行预授权退化为直接 `run_failed`（首遇即置败），依据与出路见 §9 work 进展第 1 条 |
| D2 | 错误文案语言 | 与现有工具错误一致用英文（模型消费），spec 文档用中文 | — |
| D3 | Relay document 相位接入新沉淀契约 | 范围外，另行立项（用户 2026-09-29 已确认） | 登记候选：KNOWN-DEBT 📋 未来增强 |
