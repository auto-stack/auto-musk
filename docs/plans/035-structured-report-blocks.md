---
plan_id: PLAN-035
status: execution_done
feature_name: 结构化报告——emit_report 数据化 + 对话流 block 化渲染
author: [zhaopuming]
created_at: 2026-08-22T16:15:00+08:00
updated_at: 2026-08-22T17:05:00+08:00

supersedes_spec_components: []
new_spec_components: []
touched_goals: []

current_step: 7
total_steps: 7
---

# [PLAN-035] 结构化报告：emit_report 数据化 + 对话流 block 化渲染

## 变更摘要

报告从"Agent 手写整页 PPT 风格 HTML"改为"**结构化数据 + 机械渲染**"：`emit_report` 工具入参改为结构化 JSON（目标/关联 Goals/各阶段成果/交付物），HTML 与 markdown 由后端**机械生成**（单页、同数据、零稀疏零冲突），机械指标（步骤/工具调用/令牌/时长）继续自动采集不经过 Agent；报告消息在前端渲染为一组**原生 block**（目标+Goal chips / 流程+成果方框链 / 指标格 / 交付物 badges+详情展开），旧数据（无 structured）回退现有摘要渲染。

## 目标

- 消除报告内容稀疏/与正文重叠冲突：Agent 只交结构化事实，版面由代码渲染。
- 指标绝对可靠：机械采集，Agent 不可触碰。
- 报告与 chat 其它 block 视觉同构、可交互（Goal chip 跳 Specs、交付物可展开详情）。
- 兼容存量：已有 run 的旧式报告照常显示。

## 架构方案

- **后端**：
  - `relay/store.rs`：`ReportMeta` 增 `structured: Option<Value>`（serde default，旧数据兼容）；`RunReportPayload.report` 随之自动携带。
  - `report_tools.rs`：`emit_report` 参数 v2——`{title, objective, goal_links[], stages[], deliverables[], summary?}`（去掉 html/markdown 必填）；校验后由 `render_report_html`/`render_report_markdown` 机械生成双产物（沿用 guard_self_contained）；机械指标经 `run_report()` 读入嵌入 HTML。
  - `relay/plan_flow.rs`：document 相位模板第 5 步改为结构化 emit_report 指令（字段说明 + 禁止手写 HTML）。
- **前端** `ReportCard.vue`：`report.structured` 存在时 body 渲染 blocks——目标行+Goal chips（点击 `setView('specs')`）、流程方框链（stage.title + outcome，箭头连接）、指标格（现有样式）、交付物 badges（kind icon + 名称 + +/-/M 彩色标记，点击展开 detail）；无 structured 走现有 summary 渲染。

## 技术栈

Rust（serde/工具 schema）、Vue3 组件、i18n。无新依赖。

## 需求分析与背景调查

- 现状：`emit_report`（report_tools.rs:63-79）要求 Agent 交整页 HTML+markdown（required），质量取决于 LLM——用户实测"内容稀疏、与文本重叠甚至冲突"。
- 机械指标已在 `build_run_report`（store.rs:132-174）自动装配（goals_met/tool_calls/cost/duration_s/deliverables=变更文件）——这部分数据是可靠的，v2 让它成为唯一指标来源。
- 报告消息链路（PLAN-034 T9）：driver 写回会话的 tool call `report` 携带 `RunReportPayload` 全量 → 前端 `reportFromToolCall` 映射 → ReportCard。`ReportMeta` 扩展后 structured 沿此链路自动到达前端。
- `guard_self_contained`（report_tools.rs:34-47）保留双闸。
- specs 组件分组（docs/specs/03）扩展：ReportCard block 化。

## 详细设计

### D1 emit_report v2 schema

```json
{
  "title": "PLAN-0NN xxx 沉淀报告",
  "objective": "一句话目标",
  "goal_links": [{"id": "G1", "label": "认证体系"}],
  "stages": [{"key": "gate", "title": "门禁校验", "outcome": "reviewed 通过"}],
  "deliverables": [{"kind": "spec", "name": "docs/specs/README.md", "change": "M", "detail": "新增模块条目"}],
  "summary": "可选补充"
}
```
kind ∈ code|spec|doc|file|report；change ∈ +|-|M。校验：title/objective/stages 非空；未知 kind/change 报 Args 错。

### D2 机械渲染

`render_report_html(title, structured, metrics) -> String`：单页自包含（内联 CSS，延续深色基调），区块：头部（标题+日期）/目标+chips/流程方框链/指标四格/交付物表/脚注；`render_report_markdown` 同构。HTML 由代码生成，天然过 guard。

### D3 前端 blocks

ReportCard body（structured 分支）：目标行（objective + chips）→ 流程链（flex 方框+箭头，框下 outcome）→ 指标格（现样式）→ 交付物 badges（icon：code=📄代码/spec=🧩/doc=📝/file=📦/report=📊；change 标记 +绿/M蓝/-红；点击切换 detail 展开行）。summary 作为兜底文本块。

## 测试设计

- Rust 单测：schema 校验（缺 title/objective/stages 拒绝；未知 kind/change 拒绝）、render_html 含各区块标记且过 guard、markdown 同构、ReportMeta structured 序列化兼容（旧 JSON 无 structured 可反序列化）。
- 前端：vue-tsc + vitest 套件（i18n parity）。
- E2E：musk-demo 走 `/auto-plan:merge` → 会话报告消息含 structured → 前端 block 渲染（浏览器走查）。

## 验收标准

1. emit_report 只收结构化参数；HTML/markdown 为机械生成单页；产物过 guard_self_contained。
2. 会话报告消息（tool call `report`）携带 structured；前端渲染目标/流程/指标/交付物 blocks。
3. 机械指标（goals/tool_calls/cost/duration）与 HTML/前端一致（同源）。
4. 旧 run（无 structured）报告回退原渲染，不报错。
5. `cargo test -p musk` 全绿；vue-tsc 0；vitest 仅存量 2 失败。

## 执行步骤

- [x] **T1** worktree `auto-musk-wt-035`（分支 plan-035）+ 计划入库 executing。验证：`git worktree list`。 [✅ 已完成] worktree + 提交 8ef7059
- [x] **T2** `backend/crates/musk/src/relay/store.rs`：ReportMeta.structured + 兼容单测。验证：`cargo test -p musk --lib relay::store`。 [✅ 已完成] 11/11（含 report_meta_structured_field_compat；另修 ag 副本 relay_api.rs 的 ReportMeta 初始化）
- [x] **T3** `backend/crates/musk/src/report_tools.rs`：v2 schema + render_html/render_markdown + 单测。验证：`cargo test -p musk --lib report_tools`。 [✅ 已完成] 4/4（校验枚举/HTML 全区块+guard/MD 同构）
- [x] **T4** `backend/crates/musk/src/relay/plan_flow.rs`：document 模板第 5 步改结构化指令（守护测试同步）。验证：`cargo test -p musk --lib plan_flow`。 [✅ 已完成] 10/10（模板 format! 花括号转义踩坑一次）
- [x] **T5** `web/src/components/ReportCard.vue`：structured blocks 渲染 + 交付物展开 + Goal chip 跳 Specs。验证：`npx vue-tsc --noEmit`。 [✅ 已完成] EXIT 0（chg-+/chg-- 非法类名已映射 add/del/M）
- [x] **T6** 全量回归 + `npm run build` + 合并回 main + 清理 worktree。验证：`cargo test -p musk` EXIT 0 + build 成功。 [✅ 已完成] cargo 全量 0 失败；vitest 22 过（2 存量）；build 28.2s
- [x] **T7** 部署重启 + E2E（/auto-plan:merge 造 run，验证 structured 报告消息与产物）+ 造走查计划，结果记录本节。 [✅ 已完成] 部署重启（8080 曾被 auto-lang 的 ash-gui-auto-back.exe 在重启空档抢占，已清出并说明）；E2E（PLAN-028）：报告消息 structured 齐全（objective/stages/deliverables 带 kind+change），机械指标（1/1、6、290、43s）同源，HTML/MD 机械渲染含全区块；PLAN-029 备好浏览器走查。

## 复审记录

（待 /auto-plan:review 填写）

### 执行期补记

- ag 转译副本（auto_generated/relay_api.rs）的 ReportMeta 初始化需同步 structured——hw/ag 双实现约束再次生效。

## 待澄清事项

1. Goal chip 跳 Specs 目前只到 Specs 视图（v1 不带锚点定位）——锚点级跳转待 Specs 视图支持后补。
2. 交付物详情展开用内联行（非浮层）——若需弹窗式预览后续迭代。
