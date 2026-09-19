---
plan_id: PLAN-076
status: drafting
feature_name: AutoDown 统一引擎三模式接入与差异关闭
author: [agent]
created_at: 2026-09-19T00:00:00Z
updated_at: 2026-09-19T00:00:00Z
plan_revision: 1
current_step: 0
total_steps: 5
supersedes_spec_components:
  - "docs/specs/03-front-component-groups.md"
new_spec_components:
  - "docs/specs/modules/autodown-consumption.md"
touched_goals: [goal-frontend-parity]
depends_on: ["PLAN-074", "PLAN-075"]
---

# PLAN-076 — AutoDown 统一引擎三模式接入与差异关闭

## 0. 变更摘要

本计划为 [总设计012](../designs/012-vue-vm-parity-gallery-roadmap.md) 的 S3。
交付重点：同文档view、逐chunk stream、edit输入保存后再view；覆盖未闭合fence/表格/callout/中文分片、切主题、取消/完成。上游完整引擎语料与Musk消费用例分别出证据。
本文件是 revision 1 草案，不表示已经执行或通过产品验收。

## 1. 目标

- 唯一引擎：所有可达文档渲染/编辑入口映射到autodown-engine双端后端；纯文本或独立解析器不能通过。
- view一致：标题/列表/任务/表格inline code/代码块/引用/扩展块满足引擎合同与视觉预算，072回归保绿。
- stream一致：同chunk检查点内容与顺序一致，无丢失/重复/完成跳变；未闭合语法、取消、中文分片均通过。
- edit一致：两端编辑→变更事件→保存→重开→view保持语义内容；选择/焦点/撤销能力按组件公开合同验收，不能用只读冒充edit。
- 归属和版本闭合：引擎缺陷上游关闭、应用覆盖清退有等价证据、source/dist/CLI版本可追溯；三模式均达标后才能交付。

非目标：实现完整CSS引擎、恢复旧web/产品轨、绕过autodown-engine、另写一套gallery业务模板。
影响仓库：auto-musk；涉及通用能力时auto-lang；引擎差异由auto-down负责，按设计§3定责。
依赖：PLAN-074, PLAN-075。
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
- `src/front/ports/renderer.vm.at`
- `src/front/ports/renderer.web.at`
- `src/front/components/MarkdownRender.vue`
- `src/front/specs_editors.at`
- `src/front/raw_preview.at`
- `src/front/files_view.at`
- `src/front/inject_styles.web-only.ts`
- `../auto-down/autodown/showcase/auto/src/front/app.at`

输出新增路径以§8明确标注；总设计中的gallery/runner/report尚未实现。

## 5. 详细设计

本阶段按§8五个可独立验证任务交付。适用场景与预算遵循总设计§4/§8；
任何改变目标、预算、缺件豁免或仓库范围的决定记录revision变更，不能临场删AC。
纯调查发现上游机制不足时，应输出最小复现、接口决策和受影响case；复杂重构先修订任务，
本阶段未实现的受影响AC维持阻塞，不把“已登记债务”视作通过。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/modules/autodown-consumption.md` | 双轨适配分叉 → 统一引擎模式/属性/事件/版本合同 | 保证消费契约与双端证据同步 | AC-01、AC-05 |
| SD-02 | modify | `docs/specs/03-front-component-groups.md` | 旧Markdown平台描述 → 引擎消费边界 | 保证消费契约与双端证据同步 | AC-01、AC-05 |

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

- **AC-01 唯一引擎**：所有可达文档渲染/编辑入口映射到autodown-engine双端后端；纯文本或独立解析器不能通过。 验证：按§8覆盖本AC的任务结果及报告，交叉复查源/交互/双端证据。
- **AC-02 view一致**：标题/列表/任务/表格inline code/代码块/引用/扩展块满足引擎合同与视觉预算，072回归保绿。 验证：按§8覆盖本AC的任务结果及报告，交叉复查源/交互/双端证据。
- **AC-03 stream一致**：同chunk检查点内容与顺序一致，无丢失/重复/完成跳变；未闭合语法、取消、中文分片均通过。 验证：按§8覆盖本AC的任务结果及报告，交叉复查源/交互/双端证据。
- **AC-04 edit一致**：两端编辑→变更事件→保存→重开→view保持语义内容；选择/焦点/撤销能力按组件公开合同验收，不能用只读冒充edit。 验证：按§8覆盖本AC的任务结果及报告，交叉复查源/交互/双端证据。
- **AC-05 归属和版本闭合**：引擎缺陷上游关闭、应用覆盖清退有等价证据、source/dist/CLI版本可追溯；三模式均达标后才能交付。 验证：按§8覆盖本AC的任务结果及报告，交叉复查源/交互/双端证据。


## 8. 执行步骤

- [ ] **T-01 消费矩阵与接口探针**（依赖：阶段前置；覆盖AC-01、AC-05）：新增076-engine-map报告，枚举全部renderer/编辑调用和生成注册、vendor版本；按上游真实autodown/autodown_editor入口验证view/stream/edit映射，限定调查产出=模式/prop/事件/特性矩阵和缺口owner。 落点：§4源码锚点及新增 `docs/reports/ui-parity/076-evidence.md`；复用074新增gallery/runner/cases，禁止另一份基线工具。验证：对所有调用点静态追踪并运行上游三模式最小探针，预期每个入口有实际注册/props/events映射。

- [ ] **T-02 宿主统一转接**（依赖：T-01；覆盖AC-01）：保留必要兼容端口但统一转接autodown-engine，移除纯文本VM降级；映射content/final/theme/accent/change/save等实际接口，不另造解析器。涵盖消息、文件、Wiki、计划、规范编辑和报告。 落点：§4源码锚点及新增 `docs/reports/ui-parity/076-evidence.md`；复用074新增gallery/runner/cases，禁止另一份基线工具。验证：运行Musk消息、文件与编辑器消费fixture，预期全部到达统一引擎，无纯文本fallback。

- [ ] **T-03 引擎侧三模式修复**（依赖：T-02；覆盖AC-02、AC-03、AC-04）：在auto-down修复块节奏/配色/高亮/流式和编辑差异；auto-lang仅补原生组件桥和通用能力。对Design22引擎章12px/8px等冲突作有证据决策，不在Musk堆深层CSS补丁。 落点：§4源码锚点及新增 `docs/reports/ui-parity/076-evidence.md`；复用074新增gallery/runner/cases，禁止另一份基线工具。验证：上游模式/流式/编辑局部测试加双端截图，预期AC-02/03/04全部通过；命令据当前package scripts冻结记录。

- [ ] **T-04 版本消费与覆盖清退**（依赖：T-03；覆盖AC-02、AC-05）：按PLAN072已交付修复保留表格inline/task列表/h1-h6回归；刷新vendor/依赖产物记录源码与dist hash；仅在上游等价效果验证后移除Musk内部引擎CSS覆盖。 落点：§4源码锚点及新增 `docs/reports/ui-parity/076-evidence.md`；复用074新增gallery/runner/cases，禁止另一份基线工具。验证：应用072表格/任务/标题回归fixture并校验source/dist hash，预期旧修复保留且覆盖移除等价。

- [ ] **T-05 三模式对拍与交接**（依赖：T-04；覆盖AC-01..AC-05）：同文档view、逐chunk stream、edit输入保存后再view；覆盖未闭合fence/表格/callout/中文分片、切主题、取消/完成。上游完整引擎语料与Musk消费用例分别出证据。 落点：§4源码锚点及新增 `docs/reports/ui-parity/076-evidence.md`；复用074新增gallery/runner/cases，禁止另一份基线工具。验证：node scripts/ui-parity.mjs run --plan 076，预期全部宿主case通过并关联上游完整三模式报告。

工作区：代码分支 `plan-076-dev`，组 `D:/autostack/.wt/musk-076/auto-musk`；
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

## 10. 待澄清事项

无阻止写成草案的用户信息缺口。实现期开工责任人处理以下有界事项：
1. T-01核实实际依赖版本、可达调用和上游变化；新增未知项必须归owner与case，不静默缩范围。
2. 074已冻结的导入机制/runner参数/字体与预算需核对；若前置未完成，保持该阶段阻塞。
3. 若原生能力需要大范围机制重构，T-01提供最小复现和修订提案；影响验收标准时需范围决策。
4. 本路线的近期“大体一致”预算是拟议执行合同；长期像素目标仍独立保留，不能自动宣称完成。
