---
plan_id: PLAN-072
status: execution_done
feature_name: AutoDown 渲染三修（表格 inline code 字面量+竖排 / bullet+checkbox 多换行 / H1-H6 空行节奏）+ 渲染链路定案
author: zhaop / zcode
created_at: 2026-09-18T19:05:00+08:00
updated_at: 2026-09-18T20:05:00+08:00
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
