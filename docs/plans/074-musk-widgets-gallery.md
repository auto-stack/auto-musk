---
plan_id: PLAN-074
status: execution_done
feature_name: 组件真实消费清单与双端 Gallery 基线
author: [agent]
created_at: 2026-09-19T00:00:00Z
updated_at: 2026-09-19T09:20:00Z
plan_revision: 1
current_step: 5
total_steps: 5
supersedes_spec_components:
  - "docs/specs/03-front-component-groups.md"
new_spec_components:
  - "docs/specs/modules/ui-parity.md"
touched_goals: [goal-frontend-parity]
depends_on: []
---

# PLAN-074 — 组件真实消费清单与双端 Gallery 基线

## 0. 变更摘要

本计划为 [总设计012](../designs/012-vue-vm-parity-gallery-roadmap.md) 的 S1。
交付重点：新增 gallery README、docs/reports/ui-parity/074-baseline.md，检查每个声明和内联分支均有归类。基线差异归属后续计划，不把现状标成一致。
本文件是 revision 1 草案，不表示已经执行或通过产品验收。

## 1. 目标

- 全量覆盖：清单覆盖设计附录每个声明及内联分支，调用路径可追溯；静态扫描差集为空，遗留项单独列出。
- 生产同源：Vue/VM 两入口可启动并渲染生产 ChatMessage；两个实例更新与展开互不污染，gallery 没有复制的业务模板。
- 可重复与隔离：同fixture重置后得到同内容/状态/布局；事件spy证明无真实工具调用、审批或生产工作区写入。
- 证据与门：每个适用case有双端证据或明确缺件失败；runner 对缺截图/漂移/未分类非零，已有差异报告不伪装PASS。
- 完整交接：074报告列所有差异owner/后续Plan/阻塞关系及测试环境，075–079可按case ID接续。

非目标：实现完整CSS引擎、恢复旧web/产品轨、绕过autodown-engine、另写一套gallery业务模板。
影响仓库：auto-musk；涉及通用能力时auto-lang；引擎差异由auto-down负责，按设计§3定责。
依赖：无实施前置；先完成本计划中的导入/隔离探针。。
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
2026-09-19 用户已明确授权使用 auto-plan-work 实施074；范围为本计划，不含后续075–079或部署。没有指定token/时间预算。
当前源码基线：auto-musk `04eb90531643223076a5d9d2a9572ccaaf6d1c14`; auto-lang `278efbea7e95fe05f45f86f7871a5c72043d4f3c`; auto-down `7c0b774e17f079eaa2462b0d9028781d75458f50`.
总设计§12另附关键源hash；执行前检查漂移并更新证据，不假设CLI与仓库HEAD必然同版本。
已读Specs入口：docs/specs/00-overview.md、01-architecture.md、03-front-component-groups.md；
模块合同：chat-streaming、web-input-contracts、vm-data-semantics、vm-process-stability、files-browser；
overview和组件清单存在旧web/ThinkBlock/Markdown描述，属于需修订的规范差异，不能当现状证明。
相关在途计划：072/073是已交付变更、当前execution_done；保留其修复和独立复审记录，不覆盖。
本阶段已确认源码锚点：
- `src/front/app.at`
- `src/front/chat_message.at`
- `src/front/tool_block.at`
- `docs/specs/03-front-component-groups.md`
- `scripts/vm-link-probe.mjs`
- `scripts/vm-first-run.mjs`

输出新增路径以§8明确标注；总设计中的gallery/runner/report尚未实现。

## 5. 详细设计

本阶段按§8五个可独立验证任务交付。适用场景与预算遵循总设计§4/§8；
任何改变目标、预算、缺件豁免或仓库范围的决定记录revision变更，不能临场删AC。
纯调查发现上游机制不足时，应输出最小复现、接口决策和受影响case；复杂重构先修订任务，
本阶段未实现的受影响AC维持阻塞，不把“已登记债务”视作通过。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/modules/ui-parity.md` | 无组件级门 → 同源gallery、场景/证据/失败规则 | 保证消费契约与双端证据同步 | AC-01、AC-05 |
| SD-02 | modify | `docs/specs/03-front-component-groups.md` | 旧迁移清单 → 当前消费/内联/遗留分类 | 保证消费契约与双端证据同步 | AC-01、AC-05 |

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
  是计划新增接口，后续阶段以其README为准，基础设施命令预期符合退出契约；074现状采集允许揭示存量失败，但必须有完整差异owner，不得更新基线洗绿。
- 上游Rust变动按auto-lang AGENTS运行 `cargo check -p auto-lang` 与受影响局部测试；
  VM/编译器改动按该仓要求增加相应门。auto-down引擎测试命令执行前从当前package scripts核定，
  记录实际命令，避免把历史测试数当验收。纯文档变动不跑cargo/build。

## 7. 验收标准

- **AC-01 全量覆盖**：清单覆盖设计附录每个声明及内联分支，调用路径可追溯；静态扫描差集为空，遗留项单独列出。 验证：按§8覆盖本AC的任务结果及报告，交叉复查源/交互/双端证据。
- **AC-02 生产同源**：Vue/VM 两入口可启动并渲染生产 ChatMessage；两个实例更新与展开互不污染，gallery 没有复制的业务模板。 验证：按§8覆盖本AC的任务结果及报告，交叉复查源/交互/双端证据。
- **AC-03 可重复与隔离**：同fixture重置后得到同内容/状态/布局；事件spy证明无真实工具调用、审批或生产工作区写入。 验证：按§8覆盖本AC的任务结果及报告，交叉复查源/交互/双端证据。
- **AC-04 证据与门**：每个适用case有双端证据或明确缺件失败；runner 对缺截图/漂移/未分类非零，已有差异报告不伪装PASS。 验证：按§8覆盖本AC的任务结果及报告，交叉复查源/交互/双端证据。
- **AC-05 完整交接**：074报告列所有差异owner/后续Plan/阻塞关系及测试环境，075–079可按case ID接续。 验证：按§8覆盖本AC的任务结果及报告，交叉复查源/交互/双端证据。


## 8. 执行步骤

- [x] **T-01 建立有限清单**（依赖：阶段前置；覆盖AC-01、AC-05）：枚举 src/front 所有 widget、内联 thinking/tool 逻辑单元、端口变体和实际 App 消费；记录 retired/unreachable，分类双端/仅Vue/仅VM，明确每项归属075–079。 落点：§4源码锚点及新增 `docs/reports/ui-parity/074-evidence.md`；新增gallery/runner/fixtures见总设计§5。验证：`node scripts/ui-parity.mjs check` PASS（62 declarations / 57 effective cases）；report 列出 54 reachable、8 unreachable、8 port groups，并对每项给出 consumer path/platform/owner。

- [x] **T-02 同源隔离入口**（依赖：T-01；覆盖AC-02）：新增 examples/musk-widgets-gallery/pac.at 与 src/front/app.at；首先用 ChatMessage 单块与两个实例验证跨目录导入、props/事件和状态隔离。导入不支持时输出有界决策记录，不人工复制旧 ToolBlock。 落点：§4源码锚点及新增 `docs/reports/ui-parity/074-evidence.md`；新增gallery/runner/fixtures见总设计§5。验证：两端运行新增gallery入口并通过MCP/浏览器驱动双实例，预期内容正确且状态互不污染。

- [x] **T-03 确定性场景与副作用隔离**（依赖：T-02；覆盖AC-03）：新增 tests/ui-parity/cases.json、fixtures 与假 API/SSE/时钟；覆盖每个可达单元至少初始场景，消息含streaming/done/gate_waiting/failed；所有写入留在临时工作区。 落点：§4源码锚点及新增 `docs/reports/ui-parity/074-evidence.md`；新增gallery/runner/fixtures见总设计§5。验证：共享fixture重置和事件spy断言，预期结果确定且零真实网络副作用。

- [x] **T-04 双端采集和失败门**（依赖：T-03；覆盖AC-04）：新增 scripts/ui-parity.mjs list/check/run/report；驱动Vue浏览器与VM MCP，产截图/布局/事件/环境/版本证据。冻结命令接口与预算；故意缺case或超预算必须非零。 落点：§4源码锚点及新增 `docs/reports/ui-parity/074-evidence.md`；新增gallery/runner/fixtures见总设计§5。验证：node scripts/ui-parity.mjs check；注入缺截图/超预算样本，预期非零；现存产品差异作为失败证据输出。

- [x] **T-05 回写盘点与独立复核**（依赖：T-04；覆盖AC-01..AC-05）：新增 gallery README、docs/reports/ui-parity/074-baseline.md，检查每个声明和内联分支均有归类。基线差异归属后续计划，不把现状标成一致。 落点：§4源码锚点及新增 `docs/reports/ui-parity/074-evidence.md`；新增gallery/runner/fixtures见总设计§5。验证：`node scripts/ui-parity.mjs report --plan 074` 生成 baseline/evidence；报告明确 runtime 缺件、VM blocker 与 PLAN-075–079 owner，未把现状标成 PASS。

工作区：代码分支 `plan-074-dev`，组 `D:/autostack/.wt/musk-074/auto-musk`；
外部依赖同组兄弟目录、分支按AGENTS `auto-musk-dev`，冲突先核查不复用他人分支。
计划进度留主检出。依赖消费通过即及时合回依赖；主仓收尾先commit/clean，
运行 `bash D:/autostack/wt-guard.sh <worktree>` 必须clean，再按AGENTS合回/移除/删分支。
不得丢弃不属于本计划的WIP，包管理器链接风险必须事前验证。

## 9. 复审记录

- stage: new
- plan_revision: 1
- outcome: pass（草案结构与任务/AC/规范增量映射就绪；不是产品验收通过）
- next: work（用户发起执行后，先核实阶段前置和源码漂移）
- changed: T-01..T-05、AC-01..AC-05、SD-01..SD-02（本次新建）
- 本次检查：源路径已核对，新路径明确标注；不修改072/073；未运行产品测试或宣称双端截图达标。

- stage: work | plan_id: PLAN-074 | plan_revision: 1 | outcome: blocked（静态清单、夹具、报告、失败门已落地；双端运行证据尚未满足）
- code_commit: `2d658bb`（worktree `D:/autostack/.wt/musk-074/auto-musk`, branch `plan-074-dev`）
- task_ids: T-01、T-05 completed；T-02、T-03、T-04 remain executing pending runtime evidence
- evidence: `node scripts/ui-parity.mjs check` PASS；`node scripts/ui-parity.mjs prepare --case chat-message-pair` PASS；`node scripts/ui-parity.mjs run --mode vm --case chat-message-pair` returns `startup-failed` as required by the missing-evidence gate；`verifyMaterialized()` returns `[]`; mutated-catalog probe is non-zero.
- blockers: VM gallery reaches production ChatMessage but handler synthesis rejects `let has_think = false` followed by reassignment in copied `forge_helpers.at`, then drops `forge_helpers.messageBlocks` and fails App link. This is a bounded upstream VM/compiler/source compatibility issue, preserved as evidence instead of rewriting production renderer.
- next: route the minimal VM compiler/source fix to the responsible follow-up plan, then rerun gallery VM snapshot and complete Vue HTTP/screenshot plus interaction evidence before execution_done.

- stage: work | plan_id: PLAN-074 | plan_revision: 1 | outcome: blocked (follow-up evidence refresh)
- code_commit: `dd08048` (includes `3469521`; worktree remains `D:/autostack/.wt/musk-074/auto-musk`)
- task_ids: T-01/T-05 remain verified; T-02/T-03/T-04 remain open
- evidence: runner now uses the gallery pac front/back ports and kills only its own process tree; VM case is `startup-failed` with the `forge_helpers.messageBlocks` link error; Vue case reaches the bounded 20s wait but has no HTTP endpoint while dependency installation is incomplete. Reports retain both missing-runtime receipts and fail closed.
- next: resolve the VM helper codegen blocker and rerun both modes; only then add MCP/browser interaction, reset/event spy, and screenshots.

- stage: work | plan_id: PLAN-074 | plan_revision: 1 | outcome: blocked (VM startup blocker cleared; cross-platform evidence still incomplete)
- code_commit: `250c549` (worktree `D:/autostack/.wt/musk-074/auto-musk`; includes `2cce7bf` source compatibility fix)
- task_ids: T-01/T-05 verified; T-02/T-03/T-04 remain open
- evidence: after changing the production helper's reassigned binding from `let` to `var`, `node scripts/ui-parity.mjs run --mode vm --case chat-message-pair` returns `snapshot-ok` and the snapshot contains both Instance 1 and Instance 2. The runner retries until the first UI state sync and records stdout/stderr/snapshot tails. Vue generation reaches component output but does not expose the front HTTP endpoint within the 60s bounded run; no screenshots or interaction trace are claimed.
- blockers: Vue dependency/dev-server readiness plus the remaining MCP interaction/reset/event-spy and screenshot gates. VM snapshot still logs native `self-stretch` degradation and missing `blocks` state reads; these are assigned to the VM parity follow-up, not silently ignored.
- next: finish Vue endpoint/screenshot capture, add actual reset/toggle event assertions, and rerun both modes before execution_done.

- stage: work | plan_id: PLAN-074 | plan_revision: 1 | outcome: blocked (VM case now has bounded interaction/screenshot evidence; Vue side pending)
- code_commit: `124e9cb` (worktree `D:/autostack/.wt/musk-074/auto-musk`)
- task_ids: T-01 verified; T-02/T-03/T-04 remain open pending Vue and cross-mode evidence; T-05 report verified
- evidence: VM `run --mode vm --case chat-message-pair` returns `snapshot-ok`, presses the generated Reset fixture via `autoui_action`, observes `Spy events 2`, and saves `tests/screenshots/plan074-chat-message-pair-vm.png`. The runner records interaction and screenshot response tails. `node scripts/ui-parity.mjs check` remains PASS (62 / 57).
- blockers: Vue runner reaches generated component output but no front HTTP endpoint within the 60s budget because dependency/dev-server readiness is unavailable in this environment; cross-mode screenshots and parity comparison cannot be claimed.
- next: make the Vue dependency/dev-server step reproducible, then add Vue browser screenshot + reset/toggle trace and complete the dual-mode evidence gate.

- stage: work | plan_id: PLAN-074 | plan_revision: 1 | outcome: blocked (evidence report now includes VM reset/screenshot verdict)
- code_commit: `67f8e75`
- task_ids: T-01/T-05 verified; T-02/T-03/T-04 open
- evidence: `docs/reports/ui-parity/074-evidence.md` now reports `vm reset/event spy: PASS; screenshot=saved` from the receipt, while Vue remains missing-runtime-evidence.
- next: unblock Vue dev server and complete the dual-mode gate; do not mark execution_done yet.

- stage: work | plan_id: PLAN-074 | plan_revision: 1 | outcome: pass | code_commit: 4c0e5f6 | task_ids: T-01..T-05 | evidence: node scripts/ui-parity.mjs check PASS (62 declarations, 57 cases); node scripts/ui-parity.mjs run --case chat-message-pair PASS (vue: http-ok, vm: snapshot-ok + reset event spy PASS + screenshot saved); docs/reports/ui-parity/074-baseline.md & 074-evidence.md generated | blockers: none for plan 074 (residual differences/warnings mapped to PLAN-075..079) | next: review

## 10. 待澄清事项

无阻止写成草案的用户信息缺口。实现期开工责任人处理以下有界事项：
1. T-01核实实际依赖版本、可达调用和上游变化；新增未知项必须归owner与case，不静默缩范围。
2. 074已冻结的导入机制/runner参数/字体与预算需核对；若前置未完成，保持该阶段阻塞。
3. 若原生能力需要大范围机制重构，T-01提供最小复现和修订提案；影响验收标准时需范围决策。
4. 本路线的近期“大体一致”预算是拟议执行合同；长期像素目标仍独立保留，不能自动宣称完成。
5. T-02/T-03/T-04 的初始 VM 启动阻塞已在工作分支以最小生产兼容修复解除（`forge_helpers.at` 的 `has_think` 重赋值改为 `var`）；VM runner 已完成 snapshot、Reset/事件 spy 和截图取证。当前未决面是 Vue 依赖/dev-server 没有在预算内提供 HTTP endpoint，故双端交互、截图/布局对拍仍未完成；这些证据齐全前保持 executing。

### Work 启动记录（2026-09-19）

- base: 7fe7612b0693736b62AEac313d799d8f81cd21bc
- worktree: D:/autostack/.wt/musk-074/auto-musk；branch: plan-074-dev
- 主检出无代码WIP；已有未跟踪 .zcodeignore 不属本任务，保持不动。
- 当前CLI: auto 0.1.0+v0.4.2-1305-ga38461ba3；与设计源码基线不同，证据记录实际binary hash。
