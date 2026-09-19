---
plan_id: PLAN-078
status: drafting
feature_name: 输入、导航、业务页面组件与组合一致性
author: [agent]
created_at: 2026-09-19T00:00:00Z
updated_at: 2026-09-19T00:00:00Z
plan_revision: 1
current_step: 0
total_steps: 5
supersedes_spec_components:
  - "docs/specs/modules/web-input-contracts.md"
  - "docs/specs/modules/files-browser.md"
new_spec_components:
  - "docs/specs/modules/ui-compositions.md"
touched_goals: [goal-frontend-parity]
depends_on: ["PLAN-074", "PLAN-075", "PLAN-076", "PLAN-077"]
---

# PLAN-078 — 输入、导航、业务页面组件与组合一致性

## 0. 变更摘要

本计划为 [总设计012](../designs/012-vue-vm-parity-gallery-roadmap.md) 的 S5。
交付重点：三尺寸与两DPI，键盘顺序和主题切换；完整078报告与App场景映射。平台no-op中生产可达功能要实现或明确阻塞，不按样式降级放过。
本文件是 revision 1 草案，不表示已经执行或通过产品验收。

## 1. 目标

- 清单无遗漏：所有非消息可达单元有状态/场景，产品默认入口与gallery一致。
- 输入一致：两端IME组词可见、不重复发送；Enter/Shift+Enter/mention/聚焦/多行尺寸符合既定合同和预算。
- 壳一致：菜单/弹窗锚定、遮挡、焦点和关闭行为一致，rail收缩/主题/Logo不依赖无VM对应的CSS。
- 页面一致：文件/Wiki/Specs/Plans/白名单加载错误空态和编辑保存路径双端通过，无占位冒充能力。
- 组合适配：三尺寸两DPI无裁切/不可达操作，布局预算通过，报告映射079所有App入口。

非目标：实现完整CSS引擎、恢复旧web/产品轨、绕过autodown-engine、另写一套gallery业务模板。
影响仓库：auto-musk；涉及通用能力时auto-lang；引擎差异由auto-down负责，按设计§3定责。
依赖：PLAN-074, PLAN-075, PLAN-076, PLAN-077。
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
- `src/front/mention_input.at`
- `src/front/mention_dropdown.at`
- `src/front/ports/platform.vm.at`
- `src/front/app.at`
- `src/front/settings_menu.at`
- `src/front/workspace_selector.at`
- `src/front/specs_editors.at`
- `src/front/files_view.at`
- `src/front/wiki_view.at`
- `src/front/plans_view.at`
- `src/front/whitelist_view.at`

输出新增路径以§8明确标注；总设计中的gallery/runner/report尚未实现。

## 5. 详细设计

本阶段按§8五个可独立验证任务交付。适用场景与预算遵循总设计§4/§8；
任何改变目标、预算、缺件豁免或仓库范围的决定记录revision变更，不能临场删AC。
纯调查发现上游机制不足时，应输出最小复现、接口决策和受影响case；复杂重构先修订任务，
本阶段未实现的受影响AC维持阻塞，不把“已登记债务”视作通过。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/modules/web-input-contracts.md` | 保留Web输入规范 → 增双端等价合同 | 保证消费契约与双端证据同步 | AC-01、AC-05 |
| SD-02 | modify | `docs/specs/modules/files-browser.md` | Vue与VM缺口 → 双端文件消费和状态验收 | 保证消费契约与双端证据同步 | AC-01、AC-05 |
| SD-03 | add | `docs/specs/modules/ui-compositions.md` | 无组合级门 → 壳/输入/业务页面场景门 | 保证消费契约与双端证据同步 | AC-01、AC-05 |

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

- **AC-01 清单无遗漏**：所有非消息可达单元有状态/场景，产品默认入口与gallery一致。 验证：按§8覆盖本AC的任务结果及报告，交叉复查源/交互/双端证据。
- **AC-02 输入一致**：两端IME组词可见、不重复发送；Enter/Shift+Enter/mention/聚焦/多行尺寸符合既定合同和预算。 验证：按§8覆盖本AC的任务结果及报告，交叉复查源/交互/双端证据。
- **AC-03 壳一致**：菜单/弹窗锚定、遮挡、焦点和关闭行为一致，rail收缩/主题/Logo不依赖无VM对应的CSS。 验证：按§8覆盖本AC的任务结果及报告，交叉复查源/交互/双端证据。
- **AC-04 页面一致**：文件/Wiki/Specs/Plans/白名单加载错误空态和编辑保存路径双端通过，无占位冒充能力。 验证：按§8覆盖本AC的任务结果及报告，交叉复查源/交互/双端证据。
- **AC-05 组合适配**：三尺寸两DPI无裁切/不可达操作，布局预算通过，报告映射079所有App入口。 验证：按§8覆盖本AC的任务结果及报告，交叉复查源/交互/双端证据。


## 8. 执行步骤

- [ ] **T-01 补齐非消息单元场景**（依赖：阶段前置；覆盖AC-01）：按074表覆盖输入/导航/会话/文件/Wiki/Specs/Plans/白名单可达组件；每个单元空/加载/错误/成功以及适用编辑状态，遗留项需可达性证据。 落点：§4源码锚点及新增 `docs/reports/ui-parity/078-evidence.md`；复用074新增gallery/runner/cases，禁止另一份基线工具。验证：非消息单元清单与场景覆盖检查，预期零未归属。

- [ ] **T-02 输入能力收敛**（依赖：T-01；覆盖AC-02）：原生输入auto-grow/max-height/内滚、focus、IME、mention键盘导航、Enter与Shift+Enter、发送停止disabled；Web DOM与VM native可不同，但observable合同相同。 落点：§4源码锚点及新增 `docs/reports/ui-parity/078-evidence.md`；复用074新增gallery/runner/cases，禁止另一份基线工具。验证：实际中英文键盘/IME与mention测试，预期组词、选区、发送停止和自适应几何符合合同。

- [ ] **T-03 导航浮层和壳收敛**（依赖：T-02；覆盖AC-03）：收缩rail、Logo、主题/语言、工作区/设置菜单、删除确认；优先AutoUI Popover/Overlay承接定位和焦点。验证窗口边界/遮挡/点击外部关闭，不能用内联面板当等价。 落点：§4源码锚点及新增 `docs/reports/ui-parity/078-evidence.md`；复用074新增gallery/runner/cases，禁止另一份基线工具。验证：真实键盘/鼠标打开关闭浮层、主题/导航切换，预期焦点、边界与选中态正确。

- [ ] **T-04 页面组合验证**（依赖：T-03；覆盖AC-04）：长树/文件预览/文档编辑保存、Specs类别/详情/编辑、Plans和Wiki布局、白名单表单；引擎消费复用076，布局修复不新增业务功能。 落点：§4源码锚点及新增 `docs/reports/ui-parity/078-evidence.md`；复用074新增gallery/runner/cases，禁止另一份基线工具。验证：可控数据服务驱动加载/空/错误/编辑保存，预期各页面生产组件无占位缺件。

- [ ] **T-05 多尺寸与交接**（依赖：T-04；覆盖AC-01..AC-05）：三尺寸与两DPI，键盘顺序和主题切换；完整078报告与App场景映射。平台no-op中生产可达功能要实现或明确阻塞，不按样式降级放过。 落点：§4源码锚点及新增 `docs/reports/ui-parity/078-evidence.md`；复用074新增gallery/runner/cases，禁止另一份基线工具。验证：node scripts/ui-parity.mjs run --plan 078，预期多尺寸/DPI矩阵全通过。

工作区：代码分支 `plan-078-dev`，组 `D:/autostack/.wt/musk-078/auto-musk`；
外部依赖同组兄弟目录、分支按AGENTS `auto-musk-dev`，冲突先核查不复用他人分支。
计划进度留主检出。依赖消费通过即及时合回依赖；主仓收尾先commit/clean，
运行 `bash D:/autostack/wt-guard.sh <worktree>` 必须clean，再按AGENTS合回/移除/删分支。
不得丢弃不属于本计划的WIP，包管理器链接风险必须事前验证。

## 9. 复审记录

- stage: new
- plan_revision: 1
- outcome: pass（草案结构与任务/AC/规范增量映射就绪；不是产品验收通过）
- next: work（用户发起执行后，先核实阶段前置和源码漂移）
- changed: T-01..T-05、AC-01..AC-05、SD-01..SD-03（本次新建）
- 本次检查：源路径已核对，新路径明确标注；不修改072/073；未运行产品测试或宣称双端截图达标。

## 10. 待澄清事项

无阻止写成草案的用户信息缺口。实现期开工责任人处理以下有界事项：
1. T-01核实实际依赖版本、可达调用和上游变化；新增未知项必须归owner与case，不静默缩范围。
2. 074已冻结的导入机制/runner参数/字体与预算需核对；若前置未完成，保持该阶段阻塞。
3. 若原生能力需要大范围机制重构，T-01提供最小复现和修订提案；影响验收标准时需范围决策。
4. 本路线的近期“大体一致”预算是拟议执行合同；长期像素目标仍独立保留，不能自动宣称完成。
