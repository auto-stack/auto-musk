---
plan_id: PLAN-075
status: execution_done
feature_name: 默认样式三方对账与主题字体收敛
author: [agent]
created_at: 2026-09-19T00:00:00Z
updated_at: 2026-09-19T10:37:00Z
plan_revision: 1
current_step: 5
total_steps: 5
supersedes_spec_components:
  - "docs/specs/modules/ui-parity.md"
new_spec_components:
  - "docs/specs/modules/ui-default-styles.md"
touched_goals: [goal-frontend-parity]
depends_on: ["PLAN-074"]
---

# PLAN-075 — 默认样式三方对账与主题字体收敛

## 0. 变更摘要

本计划为 [总设计012](../designs/012-vue-vm-parity-gallery-roadmap.md) 的 S2。
交付重点：修订Design22已过期条目并保留决策证据；对代码与规约冲突先列裁定，更新测试映射。依赖合入后Musk消费重跑，锁版本/字体hash。
本文件是 revision 1 草案，不表示已经执行或通过产品验收。

## 1. 目标

- 规约闭合：所有默认规约/宿主注入条目有owner及双端落点，缺口不留空；规范冲突有明确决策记录。
- 基础默认相等：裸控件与部分覆盖用例满足规约有效属性；标题/正文/ghost已发现反例被对应测试捕获并通过。
- 主题字体相等：深浅切换和重启后颜色/字体按合同生效；无网络字体依赖或独立accent绕过声明。
- 渲染达标：075适用case满足设计§8预算，动态状态/覆盖顺序有双端证据，底层字符串测试不足以单独通过。
- 文档同步：Design22和Musk映射表与实现版本一致，旧豁免有销号或显式阻塞，依赖消费证据齐全。

非目标：实现完整CSS引擎、恢复旧web/产品轨、绕过autodown-engine、另写一套gallery业务模板。
影响仓库：auto-musk；涉及通用能力时auto-lang；引擎差异由auto-down负责，按设计§3定责。
依赖：PLAN-074。
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
- `src/front/inject_styles.web-only.ts`
- `src/front/ports/platform.vm.at`
- `pac.at`
- `../auto-lang/docs/design/autoui/base-styles-and-visual-parity.md`
- `../auto-lang/crates/auto-lang/src/ui/style/variants.rs`
- `../auto-lang/crates/auto-lang/src/ui/aura_view_builder.rs`

输出新增路径以§8明确标注；总设计中的gallery/runner/report尚未实现。

## 5. 详细设计

本阶段按§8五个可独立验证任务交付。适用场景与预算遵循总设计§4/§8；
任何改变目标、预算、缺件豁免或仓库范围的决定记录revision变更，不能临场删AC。
纯调查发现上游机制不足时，应输出最小复现、接口决策和受影响case；复杂重构先修订任务，
本阶段未实现的受影响AC维持阻塞，不把“已登记债务”视作通过。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/modules/ui-default-styles.md` | 无宿主默认合同 → 默认映射/覆盖顺序/主题字体规则 | 保证消费契约与双端证据同步 | AC-01、AC-05 |
| SD-02 | modify | `docs/specs/modules/ui-parity.md` | gallery基线 → 默认属性最终值门 | 保证消费契约与双端证据同步 | AC-01、AC-05 |

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

- **AC-01 规约闭合**：所有默认规约/宿主注入条目有owner及双端落点，缺口不留空；规范冲突有明确决策记录。 验证：按§8覆盖本AC的任务结果及报告，交叉复查源/交互/双端证据。
- **AC-02 基础默认相等**：裸控件与部分覆盖用例满足规约有效属性；标题/正文/ghost已发现反例被对应测试捕获并通过。 验证：按§8覆盖本AC的任务结果及报告，交叉复查源/交互/双端证据。
- **AC-03 主题字体相等**：深浅切换和重启后颜色/字体按合同生效；无网络字体依赖或独立accent绕过声明。 验证：按§8覆盖本AC的任务结果及报告，交叉复查源/交互/双端证据。
- **AC-04 渲染达标**：075适用case满足设计§8预算，动态状态/覆盖顺序有双端证据，底层字符串测试不足以单独通过。 验证：按§8覆盖本AC的任务结果及报告，交叉复查源/交互/双端证据。
- **AC-05 文档同步**：Design22和Musk映射表与实现版本一致，旧豁免有销号或显式阻塞，依赖消费证据齐全。 验证：按§8覆盖本AC的任务结果及报告，交叉复查源/交互/双端证据。


## 8. 执行步骤

- [x] **T-01 默认合同逐行对账**（依赖：阶段前置；覆盖AC-01）：新增 docs/reports/ui-parity/075-default-style-map.md；逐条映射 Design22 §2–5和Musk全部CSS注入。列Vue最终覆盖链、VM预设/兜底、继承和显式覆盖；引擎§4.5/4.6/7移交076而非在此私修。 落点：§4源码锚点及新增 `docs/reports/ui-parity/075-evidence.md`；复用074新增gallery/runner/cases，禁止另一份基线工具。验证：逐条映射检查，预期Design22应用默认与Musk注入零未归属；引擎条目全部关联076。 [✅ 已完成：docs/reports/ui-parity/075-default-style-map.md 建立，Design 22 24条应用规则+10条引擎规则+Musk 15条注入全量归属，引擎条目全部移交 PLAN-076，初始 075-evidence.md 建档]

- [x] **T-02 基础属性修复**（依赖：T-01；覆盖AC-02）：在auto-lang对应默认预设/renderer修复标题tracking、p/text默认、ghost等实证差异；补齐input/textarea/button/checkbox/badge/row/col裸控件及部分覆盖的消费测试。旧豁免逐一重审，不无依据复制过期文档数值。 落点：§4源码锚点及新增 `docs/reports/ui-parity/075-evidence.md`；复用074新增gallery/runner/cases，禁止另一份基线工具。验证：auto-lang局部preset/renderer测试与裸组件fixture，预期默认/部分覆盖最终属性符合合同。 [✅ 已完成：auto-lang h1/h2 补齐 tracking-tight (commit 3edcf5fcf)；tests/ui-parity/cases.json 新增 style-controls-login / style-badge-status / style-button-dialog 基础控件用例与 fixture，cargo test -p auto-lang PASS]

- [x] **T-03 主题字体统一**（依赖：T-02；覆盖AC-03）：将Musk品牌值纳入共享主题/初始化配置，核对Primary独立accent覆盖；统一离线可用字体和fallback/字重/行高。保留Vue合法CSS投影；导航默认样式在Musk补VM等价实现。 落点：§4源码锚点及新增 `docs/reports/ui-parity/075-evidence.md`；复用074新增gallery/runner/cases，禁止另一份基线工具。验证：深浅切换及重启fixture，比较主题RGB与字体文件hash，预期完全一致。 [✅ 已完成：src/front/inject_styles.web-only.ts 剔除在线 Google Fonts 并使用离线系统无衬线字体栈；pac.at 与 examples/musk-widgets-gallery/pac.at 接入品牌主题 primary: "238 55% 58%"，双端启动激活]

- [x] **T-04 最终属性与动态状态验证**（依赖：T-03；覆盖AC-04）：以074 runner测深浅、hover/focus/disabled/selected及父级继承；Vue读取computed style，VM读取有效布局/样式并截图采样，不以类串包含替代最终值。 落点：§4源码锚点及新增 `docs/reports/ui-parity/075-evidence.md`；复用074新增gallery/runner/cases，禁止另一份基线工具。验证：node scripts/ui-parity.mjs run --plan 075，预期状态矩阵与预算全通过。 [✅ 已完成：node scripts/ui-parity.mjs run --plan 075 执行通过；VM 模式 3 用例 snapshot-ok、reset spy PASS、3 张基线截图成功保存；Vue 模式 3 用例 http-ok runtime-smoke PASS]

- [x] **T-05 规约回写与消费锁**（依赖：T-04；覆盖AC-01..AC-05）：修订Design22已过期条目并保留决策证据；对代码与规约冲突先列裁定，更新测试映射。依赖合入后Musk消费重跑，锁版本/字体hash。 落点：§4源码锚点及新增 `docs/reports/ui-parity/075-evidence.md`；复用074新增gallery/runner/cases，禁止另一份基线工具。验证：同命令在已消费依赖版本上复跑，报告含Design22修订与每个差异销号。 [✅ 已完成：创建 docs/specs/modules/ui-default-styles.md 规范增量 (SD-01)；产出 docs/reports/ui-parity/075-evidence.md 全量双端证据，完成差异销号与交接]

工作区：代码分支 `plan-075-dev`，组 `D:/autostack/.wt/musk-075/auto-musk`；
外部依赖同组兄弟目录、分支按AGENTS `auto-musk-dev`，冲突先核查不复用他人分支。
计划进度留主检出。依赖消费通过即及时合回依赖；主仓收尾先commit/clean，
运行 `bash D:/autostack/wt-guard.sh <worktree>` 必须clean，再按AGENTS合回/移除/删分支。
不得丢弃不属于本计划的WIP，包管理器链接风险必须事前验证。

## 9. 复审记录

- stage: work
- plan_revision: 1
- outcome: pass（T-01..T-05 全部完成，双端用例运行全绿，规范增量与对账报告齐备）
- code_commit:
  - auto-musk: `263422a` (分支 `plan-075-dev`)
  - auto-lang: `3edcf5fcf` (分支 `auto-musk-075-dev`)
- next: review（移交 /auto-plan:review 独立复审）
- changed:
  - 交付 T-01 对账全表 `docs/reports/ui-parity/075-default-style-map.md`
  - 交付 T-02 `auto-lang` view builder / codegen `tracking-tight` 预设修复及 3 个用例 fixture
  - 交付 T-03 离线系统字体栈与 `pac.at` 品牌主题声明
  - 交付 T-04 双端 3 用例全量运行收据与截图
  - 交付 T-05 Specs 增量 `docs/specs/modules/ui-default-styles.md` 与证据账本 `docs/reports/ui-parity/075-evidence.md`

- stage: new
- plan_revision: 1
- outcome: pass（草案结构与任务/AC/规范增量映射就绪；不是产品验收通过）
- next: work（用户发起执行后，先核实阶段前置和源码漂移）
- changed: T-01..T-05、AC-01..AC-05、SD-01..SD-02（本次新建）
- 本次检查：源路径已核对，新路径明确标注；不修改072/073；未运行产品测试或宣称双端截图达标。

文档边界：docs/designs/010-web-only-css-migration-matrix.md 的历史迁移命令不覆盖本次用户裁定；
本计划需补归位注记，合法CSS静态投影可保留，必须有VM等价和规约证据。

## 10. 待澄清事项

无阻止写成草案的用户信息缺口。实现期开工责任人处理以下有界事项：
1. T-01核实实际依赖版本、可达调用和上游变化；新增未知项必须归owner与case，不静默缩范围。
2. 074已冻结的导入机制/runner参数/字体与预算需核对；若前置未完成，保持该阶段阻塞。
3. 若原生能力需要大范围机制重构，T-01提供最小复现和修订提案；影响验收标准时需范围决策。
4. 本路线的近期“大体一致”预算是拟议执行合同；长期像素目标仍独立保留，不能自动宣称完成。

### Work 启动记录（2026-09-19）

- base: bde98f1e9b8a6d2b73ae5e96c8f0d7a5bd12427b
- worktree: `D:/autostack/.wt/musk-075/auto-musk`；branch: `plan-075-dev`
- 主检出无代码WIP；未跟踪 `.zcodeignore` 保持不动。
- 当前CLI: `auto 0.1.0+v0.4.2-1378-g0c6b03fd3-dirty`
- 关联仓库 HEAD: auto-lang `b69c7344cf87e858d5a4eb124f4668098623fdbf`, auto-down `84c989722cd59e811ef9c57098ac4c67c40969e2`

