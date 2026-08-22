---
plan_id: PLAN-036
status: execution_done
feature_name: AutoDown 报告——emit_report 收 .ad 文档 + 现成管线渲染
author: [zhaopuming]
created_at: 2026-08-22T18:30:00+08:00
updated_at: 2026-08-22T19:10:00+08:00

supersedes_spec_components: []
new_spec_components: []
touched_goals: []

current_step: 7
total_steps: 7
---

# [PLAN-036] AutoDown 报告：emit_report 收 `.ad` 文档，AutoDown→ReportBlock 走现成管线

## 变更摘要

`emit_report` 入参从 v2 结构化 JSON 改为 **`.ad` 文档文本**（YAML frontmatter + Markdown 超集正文——`@autodown` 生态格式，LLM 最自然的文档形态）。转化链全部复用现成件：后端拆 frontmatter（标量 + 内联数组子集）→ `ReportMeta.structured`（frontmatter 数据 + 正文）；机械指标照旧自动注入（不信任文档数字）；前端报告卡**正文直接喂现有 StreamingRenderer**（即 `@autodown/vue` 同源副本——`.ad` 的 Markdown 超集它本来就渲染），结构化 blocks（Goal chips/交付物 badges）由 frontmatter 数据驱动（035 已实现部分保留），v2 数据（stages/objective）回退兼容。

## 目标

- Agent 从"填 JSON 表单"变为"写文档"；frontmatter 只约束少数键（宽松）。
- "AutoDown→ReportBlock 转化器"落地为零新组件：数据映射 + 文档渲染两条现成管线的薄封装。
- 机械指标不可伪造不变；HTML/MD 导出产物继续机械生成。

## 架构方案

- `report_tools.rs`：入参 `{ ad: string }`；`parse_ad_frontmatter`（标量 + 内联 `[{k: v}]` 数组的 YAML 子集）；校验（title/正文非空；deliverables 若给出走枚举校验）；`ReportMeta.structured = { title, summary?, goal_links?, deliverables?, body }`。
- 导出产物：`render_report_html_v3` = 标题头 + 指标四格（机械）+ `md_to_html`（最小 Markdown 子集：标题/段落/粗体/行内码/列表/表格/代码栏/hr）机械转换正文；`render_report_markdown_v3` = frontmatter 元信息 + 正文原样 + 指标表。
- `plan_flow.rs`：document 模板第 5 步改为 `.ad` 写作指令（frontmatter 键说明 + 正文建议结构 + "指标勿写，系统注入"）。
- `ReportCard.vue`：`structured.body` 存在 → 正文区喂 StreamingRenderer；goal_links/deliverables 数组存在 → chips/badges（沿用 035 样式）；无 body → v2/旧回退。

## 需求分析与背景调查

- `.ad` 格式（auto-down/docs/02-ad-format.md）：YAML frontmatter + Markdown 超集（callout/双链/表格）；`@autodown/core`→ProseMirror、`@autodown/vue` StreamingRenderer。
- musk 的 `web/src/components/StreamingRenderer.vue` 与 `@autodown/vue` 同源（模板逐字一致）——正文渲染零新代码。
- 后端 frontmatter 解析先例：`plans.rs parse_frontmatter`（扁平标量）；035 已建 `ReportMeta.structured` 通道与前端 blocks。
- Auto（.at）为 JSON 超集（auto-atom，musk 已依赖）；本计划 Phase 1 不动 auto-lang/auto-down 两仓。

## 测试设计

- Rust：frontmatter 子集解析（标量/内联数组/无 frontmatter/畸形容忍）、md_to_html（各块型+转义）、产物含机械指标与正文、v2 结构化字段兼容保留。
- 前端：vue-tsc + vitest（i18n parity）。
- E2E：`/auto-plan:merge` → 报告消息 structured 含 body/frontmatter → 前端走查。

## 验收标准

1. emit_report 只收 `ad` 文本；title/正文必填；deliverables 枚举校验保留。
2. 报告消息 structured = frontmatter 数据 + body；前端正文由 StreamingRenderer 渲染；chips/badges 数据驱动。
3. 机械指标仍自动采集并注入产物与卡片（文档中出现的数字不采信）。
4. 旧数据（v2 stages/objective、v1 summary）回退渲染不报错。
5. `cargo test -p musk` 全绿；vue-tsc 0；vitest 仅存量 2 失败。

## 执行步骤

- [x] **T1** worktree `auto-musk-wt-036`（分支 plan-036）+ 计划入库 executing。 [✅] 提交 f85c7a0
- [x] **T2** `report_tools.rs`：`parse_ad_frontmatter`（标量+内联数组）+ `.ad` 入参与校验 + 单测。 [✅] 6/6（frontmatter 标量+内联数组/无围栏/校验/md_to_html 块型/产物含指标与正文）
- [x] **T3** `report_tools.rs`：`md_to_html` 最小子集 + `render_report_html_v3`/`_markdown_v3` + 单测。 [✅] 6/6（frontmatter 标量+内联数组/无围栏/校验/md_to_html 块型/产物含指标与正文）
- [x] **T4** `plan_flow.rs` 模板 `.ad` 指令 + 守护测试同步。 [✅] 10/10
- [x] **T5** `ReportCard.vue`：body 喂 StreamingRenderer + frontmatter blocks + 回退兼容。 [✅] TSC 0；vitest 22 过（2 存量）
- [x] **T6** 全量回归 + build + 合并部署。 [✅] cargo 全量 0 失败；build 18.2s
- [x] **T7** E2E + 走查计划，结果记录本节。 [✅] E2E（PLAN-030）：Agent 交合规 .ad（frontmatter：title/summary/deliverables×3 带 kind+change；body 为 Markdown 正文）；structured={title,summary,deliverables,body} 入会话报告消息；机械指标（1/1、9、411、50s）同源注入；report.html/md/ad 三产物落盘（html 含指标格与正文表格）。goal_links Agent 省略（可选键，正常）。浏览器走查留用户。

## 复审记录

（待 /auto-plan:review 填写）

## 待澄清事项

1. frontmatter 内联数组子集（无引号值）为 Phase 1 容忍式解析；完整 YAML 依赖待评估 serde_yaml 引入。
2. `.ad` 高级扩展（callout/双链渲染增强、双链路由）后续迭代。
