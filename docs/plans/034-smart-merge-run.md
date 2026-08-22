---
plan_id: PLAN-034
status: executing
feature_name: 智能沉淀闭环——计划页按钮触发 Chats 会话式 AI 沉淀 run
author: [zhaopuming]
created_at: 2026-08-22T13:32:09+08:00
updated_at: 2026-08-22T13:32:09+08:00

supersedes_spec_components: []
new_spec_components: []
touched_goals: []

current_step: 0
total_steps: 7
---

# [PLAN-034] 智能沉淀闭环：`/auto-plan:merge` 会话式 AI 沉淀 run

## 变更摘要

把计划页"沉淀到 Spec"按钮从直连 `/api/plans/{seq}/merge`（纯机械落账）改为触发一次**会话式 AI run**：点击后确认 → 跳转 Chats 并新建会话 → 自动发送 `/auto-plan:merge PLAN-NNN` → ChatsView 斜杠分支启动新的**单相位 relay 流程 `plan-merge`**（document 相位，plan-dev 角色）→ Agent 依次执行 `merge_plan`（机械沉淀+归档，幂等第一步）→ 按 spec-impact 更新 `docs/specs/` 模块树 → `emit_report` 生成类 PPT HTML 报告（relay run 内合法，报告经 report_emitted SSE 在会话中渲染）。

## 目标

- 单一入口：UI 按钮不再有"秒完成"机械语义，智能沉淀是唯一路径（后端 merge API 保留给 Agent/程序调用）。
- 全程可见、可中断、可重跑：沉淀过程在会话里流式展示，会话记录即运行日志；merge_plan 幂等保证中断重跑无害。
- 零新编排：复用 relay FlowSpec、plan-dev 角色、document 相位模板、Chats 报告渲染，全部现成机制。

## 架构方案

- **后端**（2 处小改）：
  - `relay/flows.rs`：新增内置流程 `plan-merge`（单步 `FlowStep::new("document", "plan-dev")`，无 gate），注册进 `builtin_flows()`。
  - `relay/plan_flow.rs`：`phase_task` 放行 `plan-merge`（现仅 `flow_id == "plan"`），为 `document` 步骤复用现有模板并加沉淀专用前言（目标计划取自任务文本中的 PLAN 编号；执行 merge_plan → 更新 specs 模块树 → emit_report 收尾）。
- **前端**（3 处）：
  - `ChatsView.vue` `sendMessage`：新增 `/auto-plan:merge <PLAN-NNN>` 斜杠分支，镜像现有 `/plan` 分支（`startRun({flow_id:'plan-merge', task}) + advanceRun`）。
  - 待发消息机制：`useForge` 增加 `pendingMessage` ref；PlansView 点击按钮 → `createSession()` → 置 `pendingMessage` → `setView('chats')`；ChatsView `onMounted` 消费并调用自身 `sendMessage`（走斜杠分支）。
  - `PlansView.vue`：按钮改走上述流程，确认框文案换为"将开启会话执行智能沉淀"。
- **不动**：`emit_report` 的 relay-run 门禁（在 relay run 中天然满足）；auto-ai/auto-lang（FlowSpec 为通用编排类型，无需改动）。

## 技术栈

Rust（relay FlowSpec/FlowStep）、Vue3 composables（useForge/useRelay/useViewState 单例）、i18n。无新依赖。

## 需求分析与背景调查

Specs 台账（`docs/specs/` 六区 + `00-overview`/`03-front-component-groups` 组件分组）本计划触及 web 组件分组（ChatsView/PlansView/useForge 扩展）与 relay 编排层。已核实的关键事实：

- **导航**：无 vue-router；`useViewState.setView('chats')` 切视图（App.vue L33-39 v-if 分发）。
- **Chats 斜杠先例**：`ChatsView.vue:1095-1133` `/plan` 分支 → `useRelay.startRun({flow_id:'plan', task}) + advanceRun(runId)`；`onReport` 回调已接 report_emitted SSE（L1309）。
- **会话**：`useForge.createSession()`（POST /api/chats/session）；authFetch 全局追加 workspace，EventSource 走 query 参数——工作区上下文天然正确。
- **chat/relay Agent 工具**：`lib.rs build_agent_with_context`（L244-315）已注册 `merge_plan` 等全部 plan 工具 + 工作区文件工具 + spec 工具 + `emit_report`。
- **emit_report 门禁**：仅 relay run 内可用（`report_tools.rs` 以 `ctx.parent_conversation_id` 查 run）——故选 relay 流程而非普通 chat 轮次。
- **流程注册**：`flows.rs builtin_flows()`（L10-18）；`plan_flow()` 四相位（L29-37）为样板；`phase_task`（plan_flow.rs L35-41）目前 `flow_id != "plan"` 直接返回 None。
- **PLAN-033 遗留**：UI 按钮现直连 merge API（PlansView.vue onMerge），本计划替换之。

## 详细设计

### D1 流程 `plan-merge`

```rust
/// 智能沉淀单相位流程（PLAN-034）：直接进入 document 相位。
/// 由计划页"沉淀到 Spec"按钮经 Chats `/auto-plan:merge` 触发。
fn plan_merge_flow() -> FlowSpec {
    let mut flow = FlowSpec::new("plan-merge");
    flow.add_step(FlowStep::new("document", "plan-dev"));
    flow
}
```

### D2 `phase_task` 扩展

```rust
if flow_id != "plan" && flow_id != "plan-merge" { return None }
```
`plan-merge` 仅匹配 `document` 步骤：复用现有 document 模板，前置一段沉淀专用前言：

```
# 任务：智能沉淀（plan-merge 单相位 run）
目标计划：{initial_task 中的 PLAN-NNN}
步骤：read_plan 校验 reviewed → merge_plan 机械沉淀+归档 →
按 frontmatter spec-impact 三字段更新 docs/specs/ 模块树 → emit_report。
```
（其余步骤 id 返回 None，由通用兜底处理——流程只有一个步骤，不会命中。）

### D3 前端斜杠分支（ChatsView.sendMessage）

```ts
const mergeMatch = /^\/auto-plan:merge\s+(PLAN-\d{3}|\d{1,3})\b/.exec(text)
if (mergeMatch) {
  const planId = mergeMatch[1].replace(/^(\d+)$/, (_, d) => `PLAN-${d.padStart(3, '0')}`)
  // 镜像 /plan 分支：startRun({flow_id:'plan-merge', task}) + advanceRun
}
```

### D4 待发消息机制与按钮改造

- `useForge`：`const _pendingMessage = ref<string | null>(null)` + 导出 `pendingMessage`。
- PlansView `onMerge`：确认（新 i18n 键 `plans.mergeRunConfirm`）→ `await createSession()` → `pendingMessage.value = '/auto-plan:merge ' + current.id` → `setView('chats')`；不再调用 `mergePlan`。
- ChatsView `onMounted`：若 `pendingMessage.value` 存在则取出、清空、`await sendMessage(msg)`。

## 测试设计

- 后端单测：`flows.rs` 新增 `plan_merge_flow_is_single_document_step`（步骤/角色/无 gate 断言）；`plan_flow.rs` 新增 plan-merge document 模板测试（含 PLAN 编号嵌入、merge_plan/emit_report 字样）。
- 前端：vue-tsc + 既有 vitest 套件（i18n parity 覆盖新键）。
- 手动冒烟：musk-demo 建一个 reviewed 计划 → 页面点按钮 → 观察 Chats 会话 run 全程 → 计划归档 + specs 树更新 + 报告渲染（需 aaid daemon，本机已运行）。

## 验收标准

1. reviewed 计划点"沉淀到 Spec"：确认框 → 跳 Chats 新会话 → 自动出现 `/auto-plan:merge PLAN-NNN` 并启动 run。
2. run 过程流式可见；完成后计划归档（archived + 移档）、`docs/specs/` 模块树被 Agent 更新、会话内渲染 HTML 报告。
3. 非 reviewed 计划不显示按钮（PLAN-033 行为不变）；后端 merge API 行为不变。
4. `cargo test -p musk` 全绿；`vue-tsc` 0 错误；vitest 仅存量 2 失败。

## 执行步骤

- [ ] **T1** worktree `auto-musk-wt-034`（分支 plan-034，D:/autostack 一级目录）+ 计划入库置 executing。验证：`git worktree list`。
- [ ] **T2** `backend/crates/musk/src/relay/flows.rs`：新增 `plan_merge_flow()` 并注册；测试 `plan_merge_flow_is_single_document_step`。验证：`cargo test -p musk flows`。
- [ ] **T3** `backend/crates/musk/src/relay/plan_flow.rs`：`phase_task` 放行 plan-merge + document 模板前言；测试覆盖。验证：`cargo test -p musk plan_flow`。
- [ ] **T4** `web/src/composables/useForge.ts`：pendingMessage；`web/src/views/ChatsView.vue`：斜杠分支 + onMounted 消费。验证：`npx vue-tsc --noEmit`。
- [ ] **T5** `web/src/views/PlansView.vue` + locales：按钮改触发流程（mergeRunConfirm 确认键，zh/en）。验证：`npx vue-tsc --noEmit && npx vitest run`。
- [ ] **T6** 全量回归 + `npm run build` 重建 dist + 合并回 main + 清理 worktree。验证：`cargo test -p musk` EXIT 0 + build 成功。
- [ ] **T7** 冒烟：重启 musk 服务（新二进制+新 dist），musk-demo 造 reviewed 计划，浏览器走查按钮→会话→run（报告需 daemon）；结果记录本节。

## 复审记录

（待 /auto-plan:review 填写）

## 待澄清事项

1. run 完成后是否需要自动跳回计划页/列表刷新提示——当前设计留在会话内看报告，由用户手动返回。
2. `/auto-plan:merge` 是否需要支持无参数形式（在会话里手动输入时从上下文推断计划）——当前要求显式 PLAN 编号。
