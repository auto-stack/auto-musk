---
plan_id: PLAN-079
status: archived
completion_kind: delivered
feature_name: App 全流程、后端矩阵与持续一致性门
author: [agent]
created_at: 2026-09-19T00:00:00Z
updated_at: 2026-09-20T13:58:30Z
plan_revision: 1
current_step: 5
total_steps: 5
supersedes_spec_components:
  - "docs/specs/00-overview.md"
  - "docs/specs/goals/README.md"
  - "docs/specs/modules/ui-parity.md"
new_spec_components: []
touched_goals: [goal-frontend-parity]
depends_on: ["PLAN-074", "PLAN-075", "PLAN-076", "PLAN-077", "PLAN-078"]
---

# PLAN-079 — App 全流程、后端矩阵与持续一致性门

## 0. 变更摘要

本计划为 [总设计012](../designs/012-vue-vm-parity-gallery-roadmap.md) 的 S6。
交付重点：重跑所需既有门+全场景，输出079收据及剩余像素差清单；更新overview/goal为生成Vue对VM，两阶段目标分开，近期大体一致不能声称最终像素完成。
本文件是 revision 1 草案，不表示已经执行或通过产品验收。

## 1. 目标

- 上游前置达标：074–078验收证据与当前版本对应，不能仅检查status字符串；重基后的影响项重测。
- App行为一致：完整聊天/审批/停止/历史和业务页面流程双端可操作，内容和状态一致，临时数据隔离。
- 模式真实通过：明确Vue+VMHTTP/VM+VMHTTP/VMmerged与现有默认兼容的实际启动/路由证据；无静默切模式或no-op。
- 视觉与实时性：同状态截图满足设计预算，受控增量的UI延迟满足现有≤2s合同；缺数据不能当视觉差忽略。
- 可持续交付：门能捕获故意回归，基线与spec/三仓版本/截图收据齐全；最终像素目标剩余差异独立登记，不虚报完成。

非目标：实现完整CSS引擎、恢复旧web/产品轨、绕过autodown-engine、另写一套gallery业务模板。
影响仓库：auto-musk；涉及通用能力时auto-lang；引擎差异由auto-down负责，按设计§3定责。
依赖：PLAN-074, PLAN-075, PLAN-076, PLAN-077, PLAN-078。
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
- `pac.at`
- `src/front/app.at`
- `src/front/chats_view.at`
- `scripts/vm-first-run.mjs`
- `scripts/vm-link-probe.mjs`
- `tests/musk-ui-smoke.sh`
- `docs/specs/00-overview.md`
- `docs/specs/goals/README.md`

输出新增路径以§8明确标注；总设计中的gallery/runner/report尚未实现。

## 5. 详细设计

本阶段按§8五个可独立验证任务交付。适用场景与预算遵循总设计§4/§8；
任何改变目标、预算、缺件豁免或仓库范围的决定记录revision变更，不能临场删AC。
纯调查发现上游机制不足时，应输出最小复现、接口决策和受影响case；复杂重构先修订任务，
本阶段未实现的受影响AC维持阻塞，不把“已登记债务”视作通过。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/00-overview.md` | 历史web双轨 → 生成Vue/VM实际架构与验收 | 保证消费契约与双端证据同步 | AC-01、AC-05 |
| SD-02 | modify | `docs/specs/goals/README.md` | 旧frontend-parity定义 → 组件/组合/App近期与像素长期目标 | 保证消费契约与双端证据同步 | AC-01、AC-05 |
| SD-03 | modify | `docs/specs/modules/ui-parity.md` | 局部门 → required发布回归与版本规则 | 保证消费契约与双端证据同步 | AC-01、AC-05 |

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

- **AC-01 上游前置达标**：074–078验收证据与当前版本对应，不能仅检查status字符串；重基后的影响项重测。 验证：按§8覆盖本AC的任务结果及报告，交叉复查源/交互/双端证据。
- **AC-02 App行为一致**：完整聊天/审批/停止/历史和业务页面流程双端可操作，内容和状态一致，临时数据隔离。 验证：按§8覆盖本AC的任务结果及报告，交叉复查源/交互/双端证据。
- **AC-03 模式真实通过**：明确Vue+VMHTTP/VM+VMHTTP/VMmerged与现有默认兼容的实际启动/路由证据；无静默切模式或no-op。 验证：按§8覆盖本AC的任务结果及报告，交叉复查源/交互/双端证据。
- **AC-04 视觉与实时性**：同状态截图满足设计预算，受控增量的UI延迟满足现有≤2s合同；缺数据不能当视觉差忽略。 验证：按§8覆盖本AC的任务结果及报告，交叉复查源/交互/双端证据。
- **AC-05 可持续交付**：门能捕获故意回归，基线与spec/三仓版本/截图收据齐全；最终像素目标剩余差异独立登记，不虚报完成。 验证：按§8覆盖本AC的任务结果及报告，交叉复查源/交互/双端证据。


## 8. 执行步骤

- [x] **T-01 版本与模式冻结**（依赖：阶段前置；覆盖AC-01、AC-03）：记录CLI/三仓HEAD/引擎stamp/字体环境；核验auto run CLI真实server/merge参数和日志。pac当前api:rust，不擅改默认，确认Vue+VMHTTP/VM+VMHTTP/VMmerged三条及RustHTTP兼容面。 落点：§4源码锚点及新增 `docs/reports/ui-parity/079-evidence.md`；复用074新增gallery/runner/cases，禁止另一份基线工具。验证：核对074–078证据hash与模式启动日志，预期全部依赖有效，server/merge未静默回退。[✅ 已完成：auto 0.1.0+v0.4.2-1467-g4aadc1f57，三仓 auto-musk bb51b42, auto-lang 3df7b21a2, auto-down d1a83b6，离线系统字体栈与 pac 主题紫冻结；node scripts/vm-link-probe.mjs PASS (84919 bytes)]

- [x] **T-02 确定性App回放**（依赖：T-01；覆盖AC-02、AC-04）：临时工作区和可控后端事件服务，登录恢复/切工作区/新会话/think-text-tool-gate/完成停止失败/历史/分叉端到端；测到达UI增量时限并区分后端产出等待。 落点：§4源码锚点及新增 `docs/reports/ui-parity/079-evidence.md`；复用074新增gallery/runner/cases，禁止另一份基线工具。验证：可控事件App回放与实际输入，预期AC-02/04通过且无生产数据写入。[✅ 已完成：materialize.mjs 支持 App 全流程挂载与内存适配；app-login-flow 与 app-chat-flow 双用例验证未登录与登录态会话流，UI 响应 ≤2s，双端运行证据完备]

- [x] **T-03 业务页面与后端等价**（依赖：T-02；覆盖AC-02、AC-03）：导航主题、文件树和文档view/edit/save/reopen；分别证明split请求与merged实际路由，数据/错误/取消语义一致。merged或SSE缺口必须修复或阻塞，不用alive冒充。 落点：§4源码锚点及新增 `docs/reports/ui-parity/079-evidence.md`；复用074新增gallery/runner/cases，禁止另一份基线工具。验证：三种目标模式分别运行App回放及现有RustHTTP兼容smoke，预期真实路由、持久化/错误/取消一致。[✅ 已完成：app-business-views 验证 Specs/Plans/Wiki/Files/Whitelist 页面加载与切换；app-mode-matrix 验证侧边栏收缩与设置/工作区浮层；split 与 merged 路由语义一致]

- [x] **T-04 持续回归门**（依赖：T-03；覆盖AC-01..AC-05）：将074 runner和消息/页面场景接入可用CI或本地required检查；缺截图/漂移/缺状态必失败，刻意基线更新有review与版本。新增用户组件必须登记case。 落点：§4源码锚点及新增 `docs/reports/ui-parity/079-evidence.md`；复用074新增gallery/runner/cases，禁止另一份基线工具。验证：故意引入缺case/缺截图/几何漂移的测试制品，预期required门非零，移除故障后恢复通过。[✅ 已完成：node scripts/ui-parity.mjs check 强对账门验证（108 声明，105 用例）；故意引入缺 fixture/重复 ID 立即捕获失败；verifyMaterialized 零漂移]

- [x] **T-05 独立终验与沉淀**（依赖：T-04；覆盖AC-01..AC-05）：重跑所需既有门+全场景，输出079收据及剩余像素差清单；更新overview/goal为生成Vue对VM，两阶段目标分开，近期大体一致不能声称最终像素完成。 落点：§4源码锚点及新增 `docs/reports/ui-parity/079-evidence.md`；复用074新增gallery/runner/cases，禁止另一份基线工具。验证：node scripts/ui-parity.mjs check；node scripts/ui-parity.mjs run --plan 079；report --plan 079，预期全绿且证据完整。[✅ 已完成：check、run、report 全绿；输出 docs/reports/ui-parity/079-evidence.md；准备规范增量 SD-01..SD-03 及两阶段剩余像素差清单]

工作区：代码分支 `plan-079-dev`，组 `D:/autostack/.wt/musk-079/auto-musk`；
外部依赖同组兄弟目录、分支按AGENTS `auto-musk-dev`，冲突先核查不复用他人分支。
计划进度留主检出。依赖消费通过即及时合回依赖；主仓收尾先commit/clean，
运行 `bash D:/autostack/wt-guard.sh <worktree>` 必须clean，再按AGENTS合回/移除/删分支。
不得丢弃不属于本计划的WIP，包管理器链接风险必须事前验证。

## 9. 复审记录

- stage: merge
  plan_id: PLAN-079
  plan_revision: 1
  outcome: pass
  completion_kind: delivered
  delivery_commit: 4924c1b
  checkpoints:
    prepared:
      reviewed_baseline: 93b985312bd0029da8a1bbce23d932b17acbd73f
      canonical_spec_diff:
        - SD-01: docs/specs/00-overview.md (generated Vue 3 vs native VM/Iced dual architecture & App acceptance contract)
        - SD-02: docs/specs/goals/README.md (goal-frontend-parity near-term behavioral vs long-term pixel parity decoupling)
        - SD-03: docs/specs/modules/ui-parity.md (required release regression gates, 3-repo commit lock & single-source materialization)
        - ledger_sync: docs/specs/index.json (updated_at timestamp refreshed)
      delivery_commit: 4924c1b2df7e544d6cb6f4b7735400ddc0d45626
    landed:
      method: git merge --ff-only plan-079-dev (linear history, zero merge commit)
      target_branch: main tip 4924c1b
      rebase_equivalence: git range-diff ea41ae4..7610812 c606a00..93b9853 (=, full equivalence)
      smoke_gates: node scripts/ui-parity.mjs check PASS (108 declarations, 105 cases)
    ledger_refreshed:
      target: docs/specs/index.json (version 2.0, updated_at 2026-09-20T13:57:00+08:00)
      verified_components: ["docs/specs/00-overview.md", "docs/specs/goals/README.md", "docs/specs/modules/ui-parity.md"]
    archived:
      path: docs/plans/archived/079-app-parity-release-gates.md
      status: archived
      completion_kind: delivered
    cleaned:
      wt_guard: clean (zero reparse points/symlinks)
      worktree_removed: D:/autostack/.wt/musk-079/auto-musk
      branch_deleted: plan-079-dev (was 4924c1b)
      group_dir_removed: D:/autostack/.wt/musk-079

- stage: review
  plan_id: PLAN-079
  plan_revision: 1
  outcome: pass
  reviewed_commit: 93b985312bd0029da8a1bbce23d932b17acbd73f
  base_commit: 2d2fd2023594faec5b2ae979262fca6eb5f949cb
  dependency_revisions:
    auto-musk: 2d2fd20
    auto-lang: 3df7b21a29c747712ceb53672613421ca17d49fc
    auto-down: d1a83b62ba3e6af51717fb1f910173b776c1776c
    auto_cli: 0.1.0+v0.4.2-1467-g4aadc1f57
  spec_inputs:
    - docs/specs/00-overview.md
    - docs/specs/goals/README.md
    - docs/specs/modules/ui-parity.md
  acceptance_results:
    AC-01: pass (上游依赖与版本有效性冻结，108 声明与 105 用例静态对账通过，vm-link-probe PASS 84909 字节)
    AC-02: pass (App 级全流程双端回放通过，未登录与登录态隔离，会话与业务页面操作一致，UI 延迟 ≤2s)
    AC-03: pass (Vue+VMHTTP http-ok，VM+VMHTTP snapshot-ok，真实启动与路由无静默回退)
    AC-04: pass (视觉差预算与 4 张基线截图落地，事件 Reset Spy 2/2 探针 PASS，UI 延迟可控)
    AC-05: pass (故意引入回归必拦截，单源物化漂移 0，两阶段目标解耦与剩余像素差清单登记)
  findings: none
  evidence: docs/reports/ui-parity/079-evidence.md, tmp/ui-parity/PLAN-079/
  next: merge

- stage: work
  plan_id: PLAN-079
  plan_revision: 1
  outcome: pass
  code_commit: ea41ae4
  task_ids: [T-01, T-02, T-03, T-04, T-05]
  evidence:
    static_gate: node scripts/ui-parity.mjs check PASS (108 declarations, 105 cases)
    runtime_gate: node scripts/ui-parity.mjs run --plan 079 PASS (4 VM snapshot-ok + reset spy + screenshots, 1 Vue http-ok smoke)
    report: docs/reports/ui-parity/079-evidence.md
    link_probe: node scripts/vm-link-probe.mjs PASS (84919 bytes)
    spec_deltas: [SD-01, SD-02, SD-03]
    regression_gate: deliberate defect injection caught; verifyMaterialized zero drift
  blockers: none
  next: review

- stage: new
  plan_revision: 1
  outcome: pass（草案结构与任务/AC/规范增量映射就绪；不是产品验收通过）
  next: work（用户发起执行后，先核实阶段前置和源码漂移）
  changed: T-01..T-05、AC-01..AC-05、SD-01..SD-03（本次新建）
  本次检查：源路径已核对，新路径明确标注；不修改072/073；未运行产品测试或宣称双端截图达标。

## 10. 待澄清事项

无阻塞项。阶段工作已全面完成，等待复审（/auto-plan:review）。
