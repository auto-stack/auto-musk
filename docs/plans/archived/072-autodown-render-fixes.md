---
plan_id: PLAN-072
status: archived
completion_kind: delivered
feature_name: AutoDown 渲染三修（表格 inline code 字面量+竖排 / bullet+checkbox 多换行 / H1-H6 空行节奏）+ 渲染链路定案
author: zhaop / zcode
created_at: 2026-09-18T19:05:00+08:00
updated_at: 2026-09-20T13:42:23+08:00
plan_revision: 1
current_step: 4
total_steps: 4
touched_repos: [auto-down@ed100dd, auto-musk@4f3dd7c]
---

# PLAN-072 — AutoDown 渲染三修 + 渲染链路定案

用户实测（auto-edit 工作区计划 001，gen 应用截图三张）暴露三个渲染缺陷，统一立项修复。
本计划同时定案"musk 的 vue 版到底用哪个渲染库"。

## 0. 渲染链路调查结论（用户问题答案）

- **musk web 轨（`web/`，PlansView/ChatsView）**：用 npm 第三方包 `markstream-vue`
  0.0.14-beta.8（Simon He）——该轨自 PLAN-041 web 轨退役后已非产品主轨。
- **截图对应的应用 = `gen/front/vue`（.at 生成应用，start-musk-web.cmd :3334）**：
  `platform/markdown.vue` → `@autodown/engine` 0.5.0（vendor dist 全量拷贝）。
- 因此答案：**既不是老版 autodown-view，也不是按 view/editor 分包的过渡形态——
  auto-down 已在 plan 017 把 parser/render/editor 合并为二合一 `@autodown/engine`**，
  musk 消费的就是它（`vendor/@autodown/@autodown/vue` 仅为退役别名 re-export）。
  修复落在 `../auto-down` 仓库 `autodown/packages/engine/src/render/`。✔ 与用户判断一致。

## 1. 需求与根因（三缺陷全部实证于引擎 SSR 探针，833 单测全绿）

| ID | 症状（截图） | 根因（engine 源码） |
|:---|:---|:---|
| F-1 | 表格 cell 内 `` `code` `` 显示为字面量 **inline_code**，且各 inline 段竖排 | `tableCellsOfPanel` 把 cell 子节点送 `renderEmbedded`（块级派发）：每个 inline 子节点被 node-slot 包成块（竖排），无 panel 注册的 inline 类型落到 `renderNodeElement` 的 unknown-node fallback 打印 `String(node.type)`。cell 是 parseInline 产物，恒为 inline——正确路径是 `renderInlineChildren` |
| F-2 | bullet+checkbox 任务项 checkbox 与正文之间多一个换行 | view 态 `li.list-item.task-item` = inline `<input.task-checkbox>` + 块级 `div.markdown-renderer`；把两者并排的 flex 规则只写在 `.autodown-editor-content` 域下，纯阅读域不生效（且 view 域保留 list-style 圆点） |
| F-3 | H1-H4 全部太紧凑、无空行 | 二因：① `StreamingRenderer.scrollSync` 默认 **true**——`is-sync` 槽边距清零规则（`!important`）随纯阅读发货，非首槽标题的 margin-top 全被清零（与该 prop 注释的既定语义相悖）；② h4-h6 在 streaming/editor 两域均无排版规则，Tailwind preflight 把 margin 清零 |

## 2. 实施（T-01..T-04，全勾）

- [x] **T-01** engine 三修（auto-down worktree `auto-musk-dev`，提交 **3cdec33**）：
  `block-widget.ts` cell 改走 `renderInlineChildren`；`autodown-editor.css`
  task-item 规则去域前缀（pane-agnostic）+ 补 `.autodown-editor-content` h4-h6；
  `StreamingRenderer.vue` `scrollSync` 默认改 false + 补 h4-h6 scoped 规则
  （h4 1.4/0.7、h5 1.3/0.6、h6 1.2/0.6 rem，承接 h2/h3 的 1.6/0.9 节奏）。
  契约随改：`render.test.ts` 表格用例改断言 inline 直排（无 node-slot/unknown-node）+
  `streaming-table-gold` 快照×2 重生；vitest **833/833 绿**；lint 缺件（eslint 未装，
  与 master 同状，非本计划引入）。dist 重建，**dist-stamp 49a4c59f**。
- [x] **T-02** musk vendor 刷新（worktree `plan-072-dev`，提交 **e097e64**）：
  `vendor/@autodown/engine/dist` 全量换装 + package.json stamp 描述更新
  （3cdec33 / 49a4c59f）。
- [x] **T-03** 应用级验证（gen 应用 :3000 + musk serve --workdir auto-edit :8080，
  打开计划 001 实测 DOM/计算样式/几何三重证据）：
  ① 首行 cell `unknown=0 slots=0`，`<code.inline-code>` 直排于 td 内，与文本同流；
  ② task-item `li` computed display=flex，checkbox 与正文同行（top 差 5.3px=0.35em 预期）；
  ③ 根元素无 `is-sync`，h2 上/下边距 25.6/14.4px 实测生效（=1.6/0.9rem）。
  修复立即对用户运行态生效（gen node_modules 的 pnpm 拷贝已同步新 dist）。
- [x] **T-04** 收尾合并：auto-down `auto-musk-dev` → master（**ed100dd** merge，
  与并行 plan073 的归档提交正常融合）+ worktree/分支清理；musk `plan-072-dev` → main +
  worktree/分支/组目录清理（见 §4 收据）。

## 3. 验收标准（AC-01..03，全过）

- [x] **AC-01**（=F-1）表格 cell：无 unknown-node、无 node-slot；inline_code/strong/link
  等以 inline 流排布，`pac.at` 等代码内容原样呈现（SSR 探针 + 应用 DOM 双证）。
- [x] **AC-02**（=F-2）`- [ ]` 任务项 checkbox 与首行正文同行，无多余换行；
  view 域与 editor 域视觉一致（flex+gap；view 域随之隐藏圆点，与 GitHub 任务列表及
  editor 域既有表现一致）。
- [x] **AC-03**（=F-3）纯阅读（未显式传 scroll-sync）时标题保排印节奏：h1-h6 全级
  有 margin；编辑器/scroll-sync 双workspace 为显式 opt-in 行为不受影响
  （gen WikiView 等显式传 :scroll-sync="true" 的消费面不变）。

## 4. 收据与环境备注

- auto-down：分支提交 3cdec33 → master merge **ed100dd**（蓝图会话并行提交无冲突）；
  worktree `.wt/musk-072/auto-down` 已移除、分支已删。
- auto-musk：worktree 提交 e097e64 → main merge（本文件提交同批）；
  worktree/分支/组目录清理。
- **junction 教训（新会话必读）**：在 worktree 内跑 `pnpm install` 会按 pnpm 布局创建
  全量 junction（本次 **1083 个**，指向组内 `node_modules/.pnpm` 自体 store）；
  wt-guard 正确拦截。处置：PowerShell
  `[System.IO.Directory]::Delete($link)` 逐个摘除（只删链接不穿透目标）后闸门转绿。
  坑位记录：①`Set-Content -Encoding UTF8` 带 BOM 会污染后续脚本读路径；
  ②`MSYS_NO_PATHCONV=1` 会让 git-bash 的 `cmd //c` 停止转换为 `/c`，cmd 静默进入
  交互模式空跑（exit 0 但什么都没做）。
- 已知余项（登记不阻塞）：web/ 老轨的 markstream-vue 渲染链在 web 轨退役背景下
  不另行修复；eslint 在 auto-down 工作区缺装，两计划会话同状，留上游基建任务。

## 9. 复审记录

`stage: review | plan_id: PLAN-072 | plan_revision: 1 | outcome: pass |
reviewed_commit: auto-musk main bb51b42（072 落点 e097e64 → merge 4f3dd7c，均为 main 祖先；当前 vendor 为 076 刷新版 7b81e20，072 修复在其上复验仍存） |
base_commit: 4f3dd7c（072 merge；AC 复验按 main 现态 vendor dist 执行） |
dependency_revisions: auto-down master 现态 d1a83b6；072 提交 3cdec33 → merge ed100dd 均为 master 祖先 |
spec_inputs: docs/specs/modules/autodown-consumption.md（076 建立的 canonical 现态） |
acceptance_results: AC-01 pass / AC-02 pass / AC-03 pass |
findings: 无阻塞项；INFO 见下 |
evidence: 见下 |
next: merge（/auto-plan:merge 沉淀归档）`

复审在独立会话进行（非 072 执行会话），结论全部从仓库工件重建。worktree 已于 T-04
清理，落地经提交祖先链验证（非 worktree 在场复核）。

### 复验方法与证据（对 main 现态 vendor dist，2026-09-20）

- **AC-01 pass（源+运行时双证）**：dist `render-node-*.js` 中 cell 生成函数
  （原 `tableCellsOfPanel`，构建压缩后匿名化）对 cell children 调
  `renderInlineChildren`。SSR 探针（`@vue/server-renderer` `renderToString`
  StreamingRenderer，源含表格+task list+六级标题）：4 个 `<td>` 内
  `<code class="inline-code"><span>pac.at</span></code>` 直排，strong/link 同流，
  `unknown-node`=0、`node-slot`-in-td=0。
- **AC-02 pass（源+运行时双证）**：dist `style.css` 中 `.list-item.task-item`
  规则**无域前缀**（pane-agnostic：`list-style:none; display:flex;
  align-items:flex-start; gap:.4rem` + checkbox `margin-top:.35em; flex-shrink:0`）。
  SSR 探针：`<li class="list-item task-item">` = `input.task-checkbox` 与
  `div.markdown-renderer` 同级兄弟（flex 同行由上述全域 CSS 保证）。
- **AC-03 pass（源+运行时双证）**：dist `StreamingRenderer-*.js`
  `scrollSync: { type: Boolean, default: !1 }`（默认关）；SSR 输出无 `is-sync`
  类；h1-h6 全级元素在场，`style.css` 双域 margin 齐全（streaming 域
  h1-h3 1.6/0.9rem、h4/h5/h6 1.4/0.7、1.3/0.6、1.2/0.6rem；editor 域 h1-h3
  1.25/0.5rem、h4-h6 1.2-1.0/0.45rem——076 刷新后 editor 域数值微调，全级有
  margin 的验收语义不变）。
- 探针环境：gen 应用地 `node_modules` 旁路 scratch（vue 取根解析 + 四依赖
  junction 指向 `.pnpm` 真实包），复验后已完整清除，junction 目标完好。

### Findings（均 INFO，不阻塞）

- **INFO-1 vendor 接力**：当前 main 的 vendor stamp 为 076 刷新
  （auto-down a86cb34 / stamp ae735aaf…，7b81e20），已覆盖 072 原 stamp
  （3cdec33 / 49a4c59f）。三修复在新 dist 复验全部在存（本节证据即对现态采集），
  属正常链路演化非回归；072 时点上游契约证据（vitest 833/833、快照×2）见于 T-01，
  现态 dist 门禁由 076 落地时执行。
- **INFO-2 规范增量说明（空增量的书面依据）**：072 为 legacy 计划、无
  `### 规范增量` 节。三修复属引擎内部（auto-down 仓）契约，由上游测试与快照
  承载；musk 侧经久决策（唯一引擎、vendor 全量 dist + `.dist-stamp`、禁
  symlink、`scrollSync:false` 保自然标题边距语义）已由 PLAN-076 建立的
  `docs/specs/modules/autodown-consumption.md` §3/§5 canonize（含 L68
  scrollSync 语义、L98-102 vendor 链路），无需另立 072 规范条目。
  `supersedes_spec_components`/`new_spec_components` 留空，特此说明。
- **INFO-3 环境脏改盘点（与 072 实现无关）**：musk 主检出
  `M docs/plans/079-app-parity-release-gates.md` + `?? .zcodeignore`（079 轨
  在途）；auto-down 主检出 `?? jade-garden/front/tmp/`（无关未跟踪目录）。
  072 的实现工件（vendor dist、上游提交）均处于已提交状态。

### 2026-09-20 合并收据（merge，PLAN-072:r1）

- stage: merge | plan_id: PLAN-072 | plan_revision: 1 | outcome: **pass（delivered）**
- `prepared`：reviewed 基线 = 前述复审记录（pass @ main bb51b42，独立会话从工件
  重建）；canonical Spec diff = **空**（INFO-2 书面依据：引擎内部契约由 auto-down
  上游测试承载，musk 侧经久决策已由 076 的 autodown-consumption.md §3/§5
  canonize）；delivery_commit `4f3dd7c`（实现 2026-09-18 随 T-04 历史性落
  main，含 e097e64 vendor 刷新）。
- `landed`：main tip bb51b42 含 4f3dd7c（祖先链已验）；无在途分支需 rebase/ff
  （落地于计划当时完成，本次为纯归档收束）；known-good 佐证 = 076 落地门禁
  （7b81e20，同一 vendor 现态）+ 本次复审 SSR 探针三 AC 全过。
- `ledger_refreshed`：**零操作（有据）**——reviewed 增量为空：无 072 所属
  canonical 目标需登记 `docs/specs/index.json`（spec_files 现册核对）；运行时
  账本 `.autoos/specs.json` 不造增量条目（No Spec impact is valid when review
  justifies it）。范围外观察（移交后续，不越权代办）：076 所建
  `modules/autodown-consumption.md` 未注册进 index.json spec_files（076 merge
  漏登记；075/077/078 均有各自 index 提交，唯 076 缺）。
- `archived`：active → `docs/plans/archived/072-autodown-render-fixes.md`
  （git mv）；`status: archived`、`completion_kind: delivered`。
- `cleaned`：T-04 时点已清理，本次复核**零残留**——两仓无 072 相关分支
  （plan-072-dev / auto-musk-dev 均不存在）、`D:/autostack/.wt` 无 musk-072
  组目录（现存 .wt 条目均属他任务）。
