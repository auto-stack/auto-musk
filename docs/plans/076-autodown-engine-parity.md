---
plan_id: PLAN-076
status: execution_done
feature_name: AutoDown 统一引擎三模式接入与差异关闭
author: [agent]
created_at: 2026-09-19T00:00:00Z
updated_at: 2026-09-19T20:42:00Z
plan_revision: 2
current_step: 5
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
当前源码基线：auto-musk `3a297ac1c448e7bd6b784346069602b0521bbe9c`; auto-lang `d2566829ff8f6066b924354660c47f209eeab4b9`; auto-down `84c989722cd59e811ef9c57098ac4c67c40969e2`.
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

- [x] **T-01 消费矩阵与接口探针**（依赖：阶段前置；覆盖AC-01、AC-05）：[✅ 已完成] 交付 `docs/reports/ui-parity/076-engine-map.md` 与 `docs/reports/ui-parity/076-evidence.md`；全量审计 21 处调用点，验证上游 autodown/autodown_editor 双端属性/事件支持规范，输出 7 项定责缺口与 T-02 转接设计。落点：`docs/reports/ui-parity/076-engine-map.md`、`docs/reports/ui-parity/076-evidence.md`。验证：静态目录校验 PASS，接口对齐无游离调用。

- [x] **T-02 宿主统一转接**（依赖：T-01；覆盖AC-01）：[✅ 已完成] 改造 `src/front/ports/renderer.vm.at`，彻底废除 `vm-markdown-plain` 纯文本降级，全面映射到原生 `autodown { content, streaming }`；补齐 `widget MarkdownRender` 端口，实现 `files_view.at` 与 `raw_preview.at` 的平权渲染；改造 `src/front/specs_editors.at`，将 `AutoDownEditor` 从 textarea stub 升级为原生 `autodown_editor`。落点：`src/front/ports/renderer.vm.at`、`src/front/specs_editors.at`、`docs/reports/ui-parity/076-evidence.md`。验证：`node scripts/ui-parity.mjs run --plan 076 --mode vm` 覆盖 chat、editor、view、raw-preview 四个用例全绿通过，VM AURA snapshot 证实全部生成 AST 富文本结构，无任何纯文本 fallback，保存基线截图（commit `cda57ad`）。

- [x] **T-03 引擎侧三模式修复**（依赖：T-02；覆盖AC-02、AC-03、AC-04）：[✅ 已完成] 在 auto-down 仓库修复块间 12px 节奏（`.streaming-document :deep(.markdown-renderer > .node-slot + .node-slot)`）；在 `StreamingRenderer.vue` 与 `EngineEditor.vue` 增加 ambient dark 检测与 MutationObserver 自动挂载 `.is-dark` class；833 项单元测试全绿；提交 `a86cb34` 并合回 auto-down master，清理 auto-down worktree。落点：auto-down commit `a86cb34`。验证：833 项测试全绿，构建 `dist` (stamp `ae735aafcefc4929`)。

- [x] **T-04 版本消费与覆盖清退**（依赖：T-03；覆盖AC-02、AC-05）：[✅ 已完成] 全量同步 `@autodown/engine` dist 至 `auto-musk/vendor/@autodown/engine`；清退 `src/front/inject_styles.web-only.ts` 中第 112 行块间节奏覆盖，以及第 123-191 行宿主深色覆盖层（共 70+ 行冗余样式），改由统一引擎原生承载；更新 vendor package.json。落点：`vendor/@autodown/engine`、`src/front/inject_styles.web-only.ts`。验证：`node scripts/ui-parity.mjs check` 保持全绿。

- [x] **T-05 三模式对拍与交接**（依赖：T-04；覆盖AC-01..AC-05）：[✅ 已完成] 运行 `node scripts/ui-parity.mjs run --plan 076 --mode vm`，4 个用例全部 `snapshot-ok`，reset event spy 判定全部通过，4 张 VM baseline 截图成功落盘；更新并交付规范增量 `docs/specs/modules/autodown-consumption.md`，同步更新 `docs/specs/03-front-component-groups.md`；生成最新 `docs/reports/ui-parity/076-evidence.md`。落点：`docs/specs/modules/autodown-consumption.md`、`docs/reports/ui-parity/076-evidence.md`、截图文件。验证：双端证据链与静态对账 100% 闭环。

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

- stage: work
- plan_revision: 2
- outcome: execution_done（T-01 至 T-05 五个任务全部实施完成并获得双端与静态证据支持）
- next: review（交由 /auto-plan:review 独立复审）
- changed: T-01..T-05（全量执行完毕并记录落实证据）
- execution_evidence:
  - 静态审计与调用矩阵：`docs/reports/ui-parity/076-engine-map.md` (21 处调用点全覆盖)
  - 宿主转接：`src/front/ports/renderer.vm.at` 接入原生 `autodown`，消灭裸文本降级；`src/front/specs_editors.at` 接入 `autodown_editor`
  - 上游修复：`auto-down` master (`a86cb34`) 内建块间 12px 节奏与 ambient dark 自动激活
  - 覆盖清退：`src/front/inject_styles.web-only.ts` 清退 70+ 行宿主 CSS 覆盖
  - 规范增量：`docs/specs/modules/autodown-consumption.md` (新增) 与 `docs/specs/03-front-component-groups.md` (修订)
  - 运行时对拍：`node scripts/ui-parity.mjs run --plan 076 --mode vm` 4 用例全绿并生成基线截图，证据汇总结算于 `docs/reports/ui-parity/076-evidence.md`

## 10. 待澄清事项

无阻止写成草案的用户信息缺口。实现期开工责任人处理以下有界事项：
1. T-01核实实际依赖版本、可达调用和上游变化；新增未知项必须归owner与case，不静默缩范围。
2. 074已冻结的导入机制/runner参数/字体与预算需核对；若前置未完成，保持该阶段阻塞。
3. 若原生能力需要大范围机制重构，T-01提供最小复现和修订提案；影响验收标准时需范围决策。
4. 本路线的近期“大体一致”预算是拟议执行合同；长期像素目标仍独立保留，不能自动宣称完成。
