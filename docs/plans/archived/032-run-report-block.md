---
plan_id: PLAN-032
status: archived
feature_name: Run 完成报告（ReportBlock）——document 相位生成 PPT 风格 HTML 汇报
author: [zhaopuming]
created_at: 2026-08-22T09:05:00+08:00
updated_at: 2026-08-24T14:00:00+08:00

# Leave these EMPTY here — /auto-plan:review fills them:
supersedes_spec_components: []
new_spec_components:
  - 报告 deck 预览块（gen platform/deck.vue：16:9 沙箱 iframe，无脚本/无同源/无表单）
touched_goals:
  - Run 报告呈现（deck 预览 + 新窗口打开）

current_step: 9
total_steps: 9
---

# [PLAN-032] Run 完成报告（ReportBlock）——document 相位生成 PPT 风格 HTML 汇报

## 0. 变更摘要

Run 完成后向用户交付一份**演示级汇报报告**：document 相位（知识沉淀）顺带
调用新工具 `emit_report` 生成**自包含、无脚本、PPT 风格的 HTML** 报告，经
`report_emitted` 事件进 run 事件流，前端在现有 ReportCard（数据卡）上叠加
**deck 预览层**（sandbox iframe）展示，支持新窗口打开与下载。报告本体是
**Run 的交付物**（随 workspace 持久化、与 run 1:1），计划侧留索引、spec
ledger 的 reports 区沉淀摘要+指针（全文不进 spec）。本期只做 HTML 格式，
但工具协议、存储布局与前端渲染分派均按 **format 字段**设计，为未来
AutoDown 演示格式（类 Typst 的展示语法，替代 HTML）预留直通位。

## 1. 目标

1. Run 完成时聊天中出现可交互的**精美汇报块**：封面/概览/阶段成果/指标/
   结尾的分节 deck，视觉上"像一份汇报 PPT"而非一篇文档。
2. 报告由 document 相位 agent **顺带生成**（不加新相位、不加额外 LLM 轮次
   之外的调用——与知识沉淀同一轮）。
3. 产物持久化：刷新页面/服务重启后报告块可回放恢复。
4. 双轨（hw 手写 + ag 转译；web + gen 前端）全链路一致。
5. AutoDown 预留：协议带 `format`，存储带 `report.md` 源，前端按 format
   分派渲染分支；未来 AutoDown deck 渲染器就绪后零迁移成本接入。

## 2. 架构方案（三层归属 + 数据流）

```
document 相位 agent
  └─ emit_report({format:"html", title, html, markdown})   ← 新工具
       ├─ 写 .autoos/reports/{run_id}/report.html + report.md
       ├─ run 事件流追加 ReportEmitted{format,title,path}（持久化 + SSE）
       └─ store 记录报告元数据 → run_completed 载荷带 report 字段
前端（web 全局 ReportCard 位 / gen RunBox 内嵌）
  └─ ReportCard 数据卡层（现有）+ deck 预览层（iframe sandbox srcdoc）
       └─ GET /api/forge/relay/runs/{id}/report → HTML 全文
```

**归属裁定（需求之问）**：
- **报告本体 = Run 的交付物**，不是计划文件的一部分、也不是 spec 正文。
  理由：计划是过程性文档（merge 后归档），spec 是长期结构化知识；报告是
  一次性呈现产物，与 run 1:1（同一需求重跑会有多份），挂在 run/ workspace
  上最自然。
- **计划侧**：document 相位汇报（handoff summary）与计划文件 §汇报 带
  report 路径索引（该计划的最终实施 Report）。
- **Spec 侧**：spec ledger **reports 区**（6 区之一，现 5 items/Empty）
  upsert 一条摘要+指针条目——reports 区本就是"运行报告沉淀位"，全文
  不进 spec，避免 spec 膨胀。

## 3. 技术栈

- 后端：Rust（musk crate）——`report_tools.rs` 新工具 + `RunEvent` 枚举扩展
  （hw `relay/store.rs` + ag `auto_generated/relay_store.rs` 双轨 wire parity）
  + `relay/api.rs` 报告读取端点（ag 路由经 extern_impl 委托）。
- 事件：`ReportEmitted { timestamp, format, title, path }`；`run_completed`
  载荷 `report: {format,title,path}` 元数据（内容不进载荷——经端点拉取）。
- 前端：web `ReportCard.vue`（deck 层）+ `useRelay.ts`（报告拉取/缓存）；
  gen `report_card.at` + `relay_store.at`（事件→`run_reports` 扩展元数据）。
- 展示：`<iframe sandbox="" srcdoc="...">`（最严沙箱：无脚本、无同源、
  无表单）——WikiView 的 iframe 先例可循；报告模板**强制自包含内联 CSS、
  禁止 `<script>` 与外链资源**。

## 4. 需求分析与背景调查

（spec overview：goals 5 / architecture 5 stable / designs 5 stable /
tests 5 / reviews 10 / **reports 5（Empty，本计划的首个正式使用者）**）

- **现状缺口**：PLAN-031 T5 的 ReportCard 是**确定性数据卡**（步骤/令牌/
  文件清单 + handoff 摘要 Markdown）——有信息量无"汇报感"；用户要的是
  PPT 形态的精美呈现。两者互补：数据卡作摘要层保留，deck 作呈现层新增。
- **document 相位**（`relay/plan_flow.rs`）现 4 步：检查 review_done →
  merge_plan → 更新 docs/specs 树 → 汇报。加第 5 步"生成汇报报告"与用户
  最初设计原话一致（"Run 最后一步进行 Spec 化时，顺便做一个漂亮的报告"）。
- **工具注册**（`lib.rs build_agent_with_context`）：orch_tools 列表加
  `emit_report` 一处即可让 chat + relay step agents 都具备；ToolContext 的
  `parent_conversation_id` 在 relay step 语境 **= run_id**（MuskAgentFactory
  构造），工具据此定位 run 与事件流——无需新增上下文管道。
- **HTML 预览先例**：`WikiView.vue` 已用 iframe 预览 raw 文件；srcdoc +
  sandbox 是零依赖方案（无 XSS 面，见 §5 安全）。
- **AutoDown 边界**（用户澄清）：AutoDown 现为 Markdown 扩展，尚不支持
  HTML/演示语法；未来扩展为直接生成展示内容（类 Typst）是独立大工程。
  本期 format='html'，协议/存储/渲染分派均按 format 设计。

## 5. 详细设计

### 5.1 `emit_report` 工具（`backend/crates/musk/src/report_tools.rs` 新文件）

- 参数 schema：`{ format: "html"（本期唯一合法值；schema 描述里注明
  'autodown' 为预留）, title: str, html: str（自包含 HTML 文档全文）,
  markdown: str（AutoDown/Markdown 源，未来原生渲染用） }`，required 全量。
- 执行：① `.autoos/reports/{run_id}/` 落盘 `report.html` + `report.md`
  （create_dir_all，覆盖写=幂等）；② `ws.relay.append_report(run_id,
  meta)`（store 新方法：更新 entry 元数据 + push `RunEvent::ReportEmitted`
  + mirror_events 会话镜像 + SSE publish）；③ 返回落盘路径。
- run_id 来源：`ToolContext.parent_conversation_id`；空（chat 语境误用）时
  报错引导。
- 内容防线（v1 粗粒度）：拒绝包含 `<script` / `on\w+=` / `http(s)://`
  外链 `<link|<img src=|@import` 的 html 参数（报错让 agent 自修）；文档
  化于工具 description，模板同款措辞。

### 5.2 document 相位模板第 5 步（`plan_flow.rs`）

> 5. `emit_report` 生成本 Run 汇报报告（format=html）：自包含单文件、
> 内联 CSS、**无任何 `<script>`/外链**；分节：封面（标题+日期+run 概要）
> / 需求与方案 / 各阶段成果（plan/execute/review/document） / 指标
> （步骤/工具调用/令牌/时长）/ 交付物清单 / 结尾。视觉基调：类 PPT 分节
> 卡片、大标题、留白，16:9 心智。同时给 markdown 源（同结构）。

### 5.3 事件与载荷（双轨枚举）

- `RunEvent::ReportEmitted { #[serde(default)] timestamp: u64,
  #[serde(default)] format: String, #[serde(default)] title: String,
  #[serde(default)] path: String }`——hw + ag 同形（parity 测试锚定，
  沿 PLAN-031 T13 手补 ag + auto-src 注释的既定做法）。
- `RunEntry.metadata` 增 `report: Option<ReportMeta>`；`build_run_state`
  带出；`run_completed` 载荷与 `RunReportPayload` 增 `report` 元数据字段
  （内容本体不进事件/载荷）。
- store 增 `append_report(run_id, meta)` 与 `report_html(run_id) ->
  Option<String>`（读盘）。

### 5.4 报告端点

`GET /api/forge/relay/runs/{run_id}/report?workspace=...` →
`text/html; charset=utf-8` 全文（无报告 404）。hw `relay/api.rs` 注册 +
ag 路由（`auto_generated/relay_api.rs` + extern_impl 委托）。鉴权同既有
relay 路由（query token/workspace）。

### 5.5 前端展示（web 先行、gen 镜像）

- `useRelay.ts`：`fetchRunReport(runId)` 拉取 HTML（按元数据存在才拉，
  缓存于模块级 Map）；`run_completed` 载荷的 `report` 元数据并入
  `reportData`。
- `ReportCard.vue` 在指标格与 Deliverables 之间插 **deck 层**：
  `<iframe sandbox="" :srcdoc="html" class="deck-frame">`（aspect-ratio
  16/9、圆角、内滚动），配"▶ 新窗口打开"（Blob URL）与既有下载按钮升级
  （下载 .html 全文 / .md 源二选一，v1 只 .html）。
- gen：`relay_store.at` 事件处理 `report_emitted` → `run_reports[runId]`
  扩展 `{...meta, html: ""}` + `LoadRun` 回扫 events；`report_card.at`
  增 deck 层（iframe srcdoc 绑定 store 缓存；.at 无 fetch 能力——经
  Http.get 平台协议拉取后缓存进 store）。

### 5.6 AutoDown 预留（不变式）

1. 协议：`format` 字段贯通工具→事件→元数据→前端分派。
2. 存储：永远双产物（html 呈现 + md 源）；未来 `format:"autodown"` 时
   前端 deck 层分派到 AutoDown deck 渲染器（届时新增分支，HTML 分支保留
   兼容旧报告）。
3. 结构约定：模板规定的六节结构即未来 AutoDown 演示语法的语义骨架。

## 6. 测试设计

- 单测：`report_tools` 落盘+事件追加+防线拒绝（含 `<script>` 样例）；
  store `append_report` 元数据/幂等；`run_completed` 载荷含 report。
- parity：`parity_relay_store` 增 `ReportEmitted` wire 双轨断言。
- E2E：musk-demo 起真实小 run → document 相位产出报告 → SSE
  `report_emitted` 帧可见 → `GET .../report` 返回 HTML → 刷新页面后
  ReportCard deck 层恢复 → iframe sandbox 属性断言（DOM 检查）。

## 7. 验收标准

1. 真实 run 完成后，聊天 ReportCard 出现可滚动的 PPT 风格 deck 预览，
   "新窗口打开"得到完整可看的汇报页；下载得到 .html 单文件。
2. 报告文件位于 `.autoos/reports/{run_id}/`（html+md 双产物）。
3. 刷新/重启后报告块完整恢复（事件持久化 + 回扫）。
4. 报告 HTML 无 `<script>`/外链；iframe 带 `sandbox=""`。
5. `cargo test -p musk` 全绿（含新 parity）；web `vue-tsc && vite build` 绿；
   gen codegen 绿。
6. spec ledger reports 区出现本能力沉淀条目（摘要+指针）。

## 8. 执行步骤

- [x] 1. `backend/crates/musk/src/relay/store.rs`：`ReportEmitted` 事件
       （hw 枚举 + `metadata.report` + `append_report`/`report_html` +
       `build_run_state`/`RunReportPayload` 带 report 元数据）；同步手补
       `auto_generated/relay_store.rs` + `auto-src/relay_store.at` 注释。
       验证：`cargo check -p musk`。
- [x] 2. `parity_relay_store.rs` 增 `ReportEmitted` 双轨 wire 断言。
       验证：`cargo test -p musk --test parity_relay_store`。
- [x] 3. 新建 `backend/crates/musk/src/report_tools.rs`（`emit_report`
       工具：schema/落盘/防线/返回），`lib.rs` orch_tools 注册 +
       `mod report_tools`。验证：`cargo test -p musk emit_report`。
- [x] 4. `relay/plan_flow.rs` document 模板第 5 步 + 既有模板单测更新。
       验证：`cargo test -p musk phase_task`。
- [x] 5. `relay/api.rs` + `auto_generated/relay_api.rs`（extern_impl 委托）
       增 `GET /runs/{id}/report` 端点。验证：重启后 `curl .../report` 对
       测试 run 返回 HTML/404。
- [x] 6. web：`useRelay.ts`（fetchRunReport+缓存+载荷并入）；
       `ReportCard.vue` deck 层（iframe sandbox srcdoc/Blob 打开/下载）。
       验证：`npm run build`（web/）。
- [x] 7. gen：`relay_store.at`（report_emitted→store 缓存 + Http.get 拉取 +
       LoadRun 回扫）、`report_card.at` deck 层、`relay_run_box.at` 传参。
       验证：`auto build --gen-only` + vite 3001 无错。
- [x] 8. E2E（musk-demo 真实小 run 全流程）+ `cargo test -p musk` 全量 +
       web build；回归 PLAN-031 的 RunBox 展示无破坏。
- [ ] 9. spec reports 区 upsert 本能力条目（摘要+指针）+ 提交。

## 9. 复审记录

（/auto-plan:review 填写）

## 10. 待澄清事项

- deck 层是否需要"全屏放映"模式（v1 仅新窗口打开充当）——待用户试用后定。
- 报告 md 源是否在 UI 暴露"查看源文件"入口（v1 仅落盘不展示）。

### /auto-plan:review 正式复审（2026-08-24）

| 验收项 | 判定 | 证据 |
|---|---|---|
| 任务 1-8 | pass | 全勾；deck.vue 产物现存核验（gen/front/vue/src/platform/deck.vue） |
| 任务 9 spec 沉淀 | 由 merge 承接 | 本复审即其门；沉淀动作归 /auto-plan:merge |
| 待澄清 2 项 | v1 范围决策 | 全屏放映（新窗口充当）/查看源文件（仅落盘）——已记录 |

**结论**：review_done。注：034 T9 已将报告呈现重设计为对话流内联（弹窗形态被取代，deck 预览保留）。
