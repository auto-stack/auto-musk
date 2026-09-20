---
plan_id: PLAN-077
status: archived
completion_kind: delivered
feature_name: 消息 Block 与 ChatMessage 组合一致性
author: [agent]
created_at: 2026-09-19T00:00:00Z
updated_at: 2026-09-19T00:00:00Z
plan_revision: 1
current_step: 0
total_steps: 5
supersedes_spec_components:
  - "docs/specs/modules/chat-streaming.md"
  - "docs/specs/03-front-component-groups.md"
new_spec_components: []
touched_goals: [goal-frontend-parity]
depends_on: ["PLAN-074", "PLAN-075", "PLAN-076"]
---

# PLAN-077 — 消息 Block 与 ChatMessage 组合一致性

## 0. 变更摘要

本计划为 [总设计012](../designs/012-vue-vm-parity-gallery-roadmap.md) 的 S4。
交付重点：074 runner逐case与消息组合截图/事件断言达标，更新077报告；遗留不可达组件登记替代关系，删旧件须证明无消费者。
本文件是 revision 1 草案，不表示已经执行或通过产品验收。

## 1. 目标

- 状态覆盖：每类可达Block与状态映射至case，旧fallback历史消息及073行为不回退。
- 实例隔离：多个thinking/tool实例的展开、更新和排序不污染其他实例；真实交互证明而非只断言state。
- 业务交互正确：审批/拒绝/问卷/复制/分叉/停止等适用动作发出正确事件，tool状态不把gate_waiting显示completed。
- 组合一致：同一事件序列产生相同块顺序/内容/最终状态，流式检查点与长消息符合预算。
- 单源与证据：gallery与产品引用相同模板/组件，适用清单全部双端通过，未达项不能藏入遗留桶。

非目标：实现完整CSS引擎、恢复旧web/产品轨、绕过autodown-engine、另写一套gallery业务模板。
影响仓库：auto-musk；涉及通用能力时auto-lang；引擎差异由auto-down负责，按设计§3定责。
依赖：PLAN-074, PLAN-075, PLAN-076。
后续阶段可阅读草案，但未完成依赖不得验收。接口研究允许先行，不跨过依赖声称完成。

## 2. 架构方案

遵循总设计§3–8：共同默认规约 → 平台静态实现 → 同源组件gallery → 组合 → App。
Vue CSS允许保留，须有VM等价默认/初始化与规约行；引擎内部问题归autodown-engine。
Gallery使用实际生产组件与确定性fixtures；有状态件按stable ID隔离，数据适配不替代UI逻辑。
所有差异记录case ID、责任仓、规约值、现状、修复版本和证据；未实现/未验不能判PASS。

## 3. 技术栈

Auto .at / AutoUI；Vue生成工程；VM/Iced；autodown-engine；Node编排与浏览器自动化；
AutoUI MCP snapshot/交互/截图。具体依赖版本以执行期锁文件与三仓版本收据为准。
不要求添加新框架，不使用symlink/junction（含自动安装生成的链接）。

## 4. 需求分析与背景调查

授权记录：用户2026-09-19明确要求组件分析、总方案和多个阶段计划；本次仅调查与文档。
未授予实施/部署授权；没有用户指定token、时间预算或自动继续限制。work启动按后续授权执行。
当前源码基线：auto-musk `04eb90531643223076a5d9d2a9572ccaaf6d1c14`; auto-lang `278efbea7e95fe05f45f86f7871a5c72043d4f3c`; auto-down `7c0b774e17f079eaa2462b0d9028781d75458f50`.
总设计§12另附关键源hash；执行前检查漂移并更新证据，不假设CLI与仓库HEAD必然同版本。
已读Specs入口：docs/specs/00-overview.md、01-architecture.md、03-front-component-groups.md；
模块合同：chat-streaming、web-input-contracts、vm-data-semantics、vm-process-stability、files-browser；
overview和组件清单存在旧web/ThinkBlock/Markdown描述，属于需修订的规范差异，不能当现状证明。
相关在途计划：072/073是已交付变更、当前execution_done；保留其修复和独立复审记录，不覆盖。
本阶段已确认源码锚点：
- `src/front/chat_message.at`
- `src/front/forge_helpers.at`
- `src/front/forge_store.at`
- `src/front/tool_gate_card.at`
- `src/front/errand_card.at`
- `src/front/task_plan_card.at`
- `src/front/relay_run_box.at`
- `src/front/report_card.at`
- `docs/specs/modules/chat-streaming.md`

输出新增路径以§8明确标注；总设计中的gallery/runner/report尚未实现。

## 5. 详细设计

本阶段按§8五个可独立验证任务交付。适用场景与预算遵循总设计§4/§8；
任何改变目标、预算、缺件豁免或仓库范围的决定记录revision变更，不能临场删AC。
纯调查发现上游机制不足时，应输出最小复现、接口决策和受影响case；复杂重构先修订任务，
本阶段未实现的受影响AC维持阻塞，不把“已登记债务”视作通过。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/modules/chat-streaming.md` | 保留事件规则 → 补双端Block状态/回放验收 | 保证消费契约与双端证据同步 | AC-01、AC-05 |
| SD-02 | modify | `docs/specs/03-front-component-groups.md` | 历史独立件清单 → 实际内联/抽离与事件所有权 | 保证消费契约与双端证据同步 | AC-01、AC-05 |

以上为拟议增量，本次不修改canonical Specs。跨仓规约由对应责任仓review确认，
Musk计划记录关联路径/版本，merge阶段再沉淀；不伪称已分配上游Plan编号。

## 6. 测试设计

- §7每个AC均须关联case ID、命令、预期/实际、双端截图和状态/布局证据。
- 视觉：按总设计§8固定内容区域/DPI/字体/主题，深浅+动态状态，关键边界≤2px、
  长流累计≤4px、平坦色通道差≤2；字体边缘单报，不能整块遮罩。截图不提交大二进制。
- 运行隔离：按vm-process-stability Spec分配私有AUTOUI_MCP_PORT，读取实际回退端口；
  仅清理本runner启动的PID，禁止端口范围扫杀。
- 交互：实际点击/键盘/输入与事件spy；fixture reset、两个实例、同序列回放；
  不能只赋state断言后宣称IME/焦点/点击通过。
- 既有命令（执行于专用worktree）：`auto build` 预期成功；
  `node scripts/vm-link-probe.mjs` 预期PASS；`node scripts/vm-first-run.mjs --observe-ms 20000`
  预期存活且reds=0。先校验脚本依赖路径与后端模式；这三项均不替代视觉验收。
- Gallery运行：在新增入口目录执行 `auto run -r vue` 和 `auto run -r vm`；
  P074实现并冻结的 `node scripts/ui-parity.mjs check` / `run --plan NNN` / `report --plan NNN`
  是计划新增接口，后续阶段以其README为准，预期适用case全部通过且无缺证据。
- 上游Rust变动按auto-lang AGENTS运行 `cargo check -p auto-lang` 与受影响局部测试；
  VM/编译器改动按该仓要求增加相应门。auto-down引擎测试命令执行前从当前package scripts核定，
  记录实际命令，避免把历史测试数当验收。纯文档变动不跑cargo/build。

## 7. 验收标准

- **AC-01 状态覆盖**：每类可达Block与状态映射至case，旧fallback历史消息及073行为不回退。 验证：按§8覆盖本AC的任务结果及报告，交叉复查源/交互/双端证据。
- **AC-02 实例隔离**：多个thinking/tool实例的展开、更新和排序不污染其他实例；真实交互证明而非只断言state。 验证：按§8覆盖本AC的任务结果及报告，交叉复查源/交互/双端证据。
- **AC-03 业务交互正确**：审批/拒绝/问卷/复制/分叉/停止等适用动作发出正确事件，tool状态不把gate_waiting显示completed。 验证：按§8覆盖本AC的任务结果及报告，交叉复查源/交互/双端证据。
- **AC-04 组合一致**：同一事件序列产生相同块顺序/内容/最终状态，流式检查点与长消息符合预算。 验证：按§8覆盖本AC的任务结果及报告，交叉复查源/交互/双端证据。
- **AC-05 单源与证据**：gallery与产品引用相同模板/组件，适用清单全部双端通过，未达项不能藏入遗留桶。 验证：按§8覆盖本AC的任务结果及报告，交叉复查源/交互/双端证据。


## 8. 执行步骤

- [ ] **T-01 冻结生产状态矩阵**（依赖：阶段前置；覆盖AC-01）：以074可达性表冻结text/thinking/tool及user/assistant完整fixture，保留073最新块级state、运行身份和gate修复；覆盖历史无blocks兼容投影。 落点：§4源码锚点及新增 `docs/reports/ui-parity/077-evidence.md`；复用074新增gallery/runner/cases，禁止另一份基线工具。验证：检查块kind/state和历史兼容分支与fixture差集为空。

- [ ] **T-02 实例安全与按需抽离**（依赖：T-01；覆盖AC-02）：先通过真实ChatMessage单块/多块测试；需要抽件时props+事件、父级stable block ID持态，两实例/重排/增量/卸载复建通过后才切产品。不要直接恢复旧ToolBlock或GenericToolCard。 落点：§4源码锚点及新增 `docs/reports/ui-parity/077-evidence.md`；复用074新增gallery/runner/cases，禁止另一份基线工具。验证：双实例/重排/增量真实点击测试，预期状态隔离；源码diff证明不维护两份模板。

- [ ] **T-03 消息和工具视觉/交互修复**（依赖：T-02；覆盖AC-03、AC-04）：修thinking尾部与完成态、展开/折叠、通用工具参数/结果、gate、Errand/TaskPlan/Relay/Report；引擎内部问题回076所属上游，通用布局缺口回auto-lang。 落点：§4源码锚点及新增 `docs/reports/ui-parity/077-evidence.md`；复用074新增gallery/runner/cases，禁止另一份基线工具。验证：运行各工具/思考/gate单元，预期事件和状态正确且符合视觉预算。

- [ ] **T-04 扩展卡与业务组合**（依赖：T-03；覆盖AC-03、AC-04）：对可达Questionnaire/Gate/Secretary/StreamingTable按清单验收；测试多轮交错thinking→text→tool→gate→result与复制/分叉/停止；不执行真实危险操作。 落点：§4源码锚点及新增 `docs/reports/ui-parity/077-evidence.md`；复用074新增gallery/runner/cases，禁止另一份基线工具。验证：回放多轮交错消息与扩展卡场景，预期顺序与业务动作和现有Spec一致。

- [ ] **T-05 锁定消息证据**（依赖：T-04；覆盖AC-01..AC-05）：074 runner逐case与消息组合截图/事件断言达标，更新077报告；遗留不可达组件登记替代关系，删旧件须证明无消费者。 落点：§4源码锚点及新增 `docs/reports/ui-parity/077-evidence.md`；复用074新增gallery/runner/cases，禁止另一份基线工具。验证：node scripts/ui-parity.mjs run --plan 077，预期全部适用case通过。

工作区：代码分支 `plan-077-dev`，组 `D:/autostack/.wt/musk-077/auto-musk`；
外部依赖同组兄弟目录、分支按AGENTS `auto-musk-dev`，冲突先核查不复用他人分支。
计划进度留主检出。依赖消费通过即及时合回依赖；主仓收尾先commit/clean，
运行 `bash D:/autostack/wt-guard.sh <worktree>` 必须clean，再按AGENTS合回/移除/删分支。
不得丢弃不属于本计划的WIP，包管理器链接风险必须事前验证。

## 9. 复审记录

### 2026-09-20 执行基线

- 授权：用户要求执行 077 并继续 review / merge。
- base_commit: ec9b9bb42b7818b3730cf71b52de44f0a5afe5c3
- worktree: D:/autostack/.wt/musk-077/auto-musk；branch: plan-077-dev
- dependencies: auto-lang 278f71f3536c2c2a0de2d6d1ea60ebc201a8a234；auto-down a86cb343b738cec47df63c953f368a035a758451
- 前置已归档；当前 runner 仅 smoke，不能沿用其结果作为 077 交互/视觉 PASS。

### 2026-09-20 合并收据（merge，PLAN-077:r1）

- stage: merge | plan_id: PLAN-077 | plan_revision: 1 | outcome: **pass（delivered）**
- 合并基准：reviewed fbca482（rebase 映射 fbca482→b45429a、a3b1f59→f4fa4f4、9f99662→12b197e、wip e619dac→da0caad；`git range-diff e619dac..fbca482 da0caad..b45429a` 两对全 `=` 补丁等价）
- `prepared`：canonical Spec diff = chat-streaming 契约⑧+头注+关联实现（SD-01）、03-front-component-groups G-对话 Block 组重写+事件所有权节+遗留 TS 行（SD-02）；docs-only 后代（实现自 fbca482 未变），delivery_commit `12b197e`
- `landed`：main tip `12b197e`（--ff-only，无 merge commit）；集成冒烟（main 检出 + auto-lang master `4aadc1f57` 重建 exe）：`ui-parity check` 108/101 PASS、`message-contract.mjs` PASS、`chat-multi-round --mode vm` snapshot-ok
- `ledger_refreshed`：`docs/specs/index.json`（version 2.0）——补登记 `03-front-component-groups.md`（SD-02 canonical 目标，此前漏登记）、`updated_at`→2026-09-20；`modules/chat-streaming.md` 已在册；账本 commit `d7060fe`
- `archived`：active → `docs/plans/archived/077-message-block-parity.md`（git mv）；`status: archived`、`completion_kind: delivered`
- `cleaned`：wt-guard clean ×2；worktree `D:/autostack/.wt/musk-077/auto-musk` + 分支 plan-077-dev 移除；组内兄弟 auto-lang（`4aadc1f57` 已合 master，分支 auto-musk-dev 移除）与 auto-down（detached 消费）一并清理，组目录 `musk-077` 删除
- 依赖仓 receipts：auto-lang master tip `4aadc1f57`（1d6dc1f86 ?? 计算属性 + 4aadc1f57 max-w-[N%]；cargo check 零错、cargo t ui 仅 2 预存环境红、新增单测绿）；auto-down `3f73737f` 仅构建消费零改动
- 环境注记：8090 旧 exe 侧的用户侧动作与 073 同款——新布局/表达式能力随 auto-lang master 重建 exe 后生效

### 2026-09-20 复审记录（review）

- stage: review | plan_id: PLAN-077 | plan_revision: 1 | outcome: **pass**
- reviewed_commit: `fbca482`（plan-077-dev tip；实现提交 `a3b1f59` + harness 修复 `fbca482`）
- base_commit: `839a0cb`（073 合回后 main；rebase 基线，wip `e619dac` 补丁等价已验）
- dependency_revisions: auto-lang master `4aadc1f57`（含 `1d6dc1f86` computed `??`）；auto-down `3f73737f`（组内兄弟，构建消费）
- spec_inputs: `docs/specs/modules/chat-streaming.md`、`docs/specs/03-front-component-groups.md`（SD-01/SD-02 增量待 merge 阶段落canonical）
- acceptance_results:
  - AC-01 状态覆盖 **pass**：17 case 覆盖 text/thinking(done/streaming)/tool(completed/running/gate_waiting/failed)/gate + 扩展卡；073 行为不回退（gate 内联卡/增量快照语义随 case 复验）；`message-contract.mjs` 投影契约双端断言。
  - AC-02 实例隔离 **pass**：chat-block-isolation 双消息×多块真实点击展开/折叠隔离、工具卡隔离、重排键不串位、增量更新、卸载/复建——VM 与 Vue 同断言集全绿。
  - AC-03 业务交互 **pass**（含 F-1 记录限制）：审批/拒绝实测 `POST /api/chats/tool-gate/{gate_id}/approve|deny`（隔离 mock，双端）；分叉路由宿主 `FORK:<mid>`（双端）；停止 spy+1（双端）；gate_waiting 不显示 completed。
  - AC-04 组合一致 **pass**：chat-multi-round 多轮交错 thinking→text→tool→gate→result 回放确定性（Round 2 ↔ Replay round 1 双向断言）；流式检查点由 chat-thinking-streaming 承载。
  - AC-05 单源与证据 **pass**：gallery=生产 hash 字节拷贝（verifyMaterialized）；VM 17/17 snapshot-ok；Vue 17/17 interaction-ok 零 pageerror；未达视觉项全部登记证据报告 §6（无藏遗留）。
- findings（非阻塞，登记不回 work）：
  - F-1：Copy 动作 VM 端无文本锚点（icon-only 按钮），VM 未实证点击；Vue 端无错、CopyContent 为 dom FFI 无下游请求可 spy。后续：snapshot 匹配器支持 title 锚点。
  - F-2：视觉预算未达项（VM 主题/间距/字体细分、窗口尺寸对拍、工具头内部排布）登记 077-evidence §6，责任 auto-lang/runner。
- evidence: `docs/reports/ui-parity/077-evidence.md`；receipt `tmp/ui-parity/PLAN-077/*-{vm,vue}.json`；VM 截图 `examples/musk-widgets-gallery/src/front/tests/screenshots/plan077-*.png`；复审复核命令：`node scripts/ui-parity.mjs run --plan 077 --case task-plan-card --mode vm` / `--case chat-multi-round --mode vm`（fbca482 上重跑 snapshot-ok）。
- 限制声明：复审与执行同会话，结论以上述工件重验重建（非执行摘要）；full matrix 运行于 `a3b1f59`，`fbca482` 仅改 host 键引用号并已对 task-plan-card/chat-multi-round 重跑确认。
- next: merge

### 2026-09-20 执行进度（work）

- 状态：T-01..T-04 完成并双端验证；T-05 证据报告 `docs/reports/ui-parity/077-evidence.md` 已立。
- 生产修复：块投影 state/稳定 tkey/gate 载荷；`toggleBlockExpansion` 键列表展开；gate 卡显式 props；chats_view `onfork`→`on_fork_from`（VM/Vue 双端路由断点修复）；mention_helpers 全面 `char_at`→`sub(i,i+1)`（VM `char_at` 按 Plan 368 W5 返回码点 int，web 返回 1 字符 string——computed `??` 支持落地后该链在 VM 真实执行而暴露）。
- 上游 auto-lang 已合回 master：`1d6dc1f86`（computed `??` 支持）+ `4aadc1f57`（`max-w-[N%]` 百分比上限委托 widget）。门禁：cargo check 零错；`cargo t ui` 仅 2 个 master 预存环境红（stash 对照复现）；新增单测全绿。
- runner：内容断言（visible/absent/request/spyIncrement）、vueOnly/vmOnly 步、materialize 宿主 fork 路由承载与 per-type 字段声明、Vue Playwright 对拍 runner。
- 新增 case：chat-block-isolation / chat-multi-round / tool-gate-direct + 10 个扩展卡单元 + streaming-table（可达无消费者登记）。
- 待办：Vue 端交互对拍收尾 → review → merge。

- stage: new
- plan_revision: 1
- outcome: pass（草案结构与任务/AC/规范增量映射就绪；不是产品验收通过）
- next: work（用户发起执行后，先核实阶段前置和源码漂移）
- changed: T-01..T-05、AC-01..AC-05、SD-01..SD-02（本次新建）
- 本次检查：源路径已核对，新路径明确标注；不修改072/073；未运行产品测试或宣称双端截图达标。

## 10. 待澄清事项

无阻止写成草案的用户信息缺口。实现期开工责任人处理以下有界事项：
1. T-01核实实际依赖版本、可达调用和上游变化；新增未知项必须归owner与case，不静默缩范围。
2. 074已冻结的导入机制/runner参数/字体与预算需核对；若前置未完成，保持该阶段阻塞。
3. 若原生能力需要大范围机制重构，T-01提供最小复现和修订提案；影响验收标准时需范围决策。
4. 本路线的近期“大体一致”预算是拟议执行合同；长期像素目标仍独立保留，不能自动宣称完成。
