# 023 — view fn → 独立组件 codegen（auto-lang 转译器改造）

> **状态**：📋 计划（未实施）——独立项目（auto-lang 仓库），auto-musk 侧仅登记等待。
> **前置**：Plan 022（前端 Auto 化已完成，逃生舱组件架构稳定）；Plan 018-021（后端全 Auto 化方法论）。
> **仓库**：**auto-lang**（a2r 转译器 + a2vue codegen，构建 `auto.exe`）；auto-musk 为验证方。
> **目标**：让 AutoUI 的 `view fn`（`use { fn }` 逃生舱函数中定义、返回 AuraWidget 的函数）被 a2r/a2vue 转译器**原生合成 AuraWidget 组件**，使 `ChatMessage` 等当前靠逃生舱 `.vue` 文件实现的组件脱离逃生舱、纯 `.at` 表达。

---

## 0. 背景与动机

Plan 022 实现中，ChatsView 的消息渲染（ChatMessage）及其子卡片（ErrandCard/RelayCard/TaskPlanCard/GenericToolCard、MentionInput 等）因以下 codegen 缺口走逃生舱：

- **逃生舱现状**：`src/front/components/ChatMessage.vue` 等 20 个 `.vue` 组件 + `forge_helpers.ts`/`relay_commands.ts`/`questionnaire.ts` 等逃生舱 TS。它们经 `use { component/fn }` 声明触发 ext 复制进生成工程，**当前工作正常**（构建通过、行为对齐）。
- **代价**：
  1. 逃生舱 `.vue` 是手写 Vue，不受 `.at` 单一真源约束——`auto build` 零 drift 目标无法覆盖它们；
  2. `view fn`（返回 AuraWidget 的纯函数，如 `renderMentions`、消息分支渲染）目前要么被 `.at` 丢弃成 null（组件 props fn 调用），要么被 codegen 复制成普通 TS 函数（不参与 Vue 响应式合成）；
  3. codegen 有"复制只看 use 块、不递归逃生舱相对 import"的限制，逃生舱内部依赖需逐个在 use 块声明，维护脆弱。

**核心目标**：auto-lang 的转译器（`api.rs` 合成 AuraWidget 的路径）支持 `view fn`——在 `.at` 中声明、返回 AuraWidget 树、可被 widget/组件调用并渲染的函数，直接生成 Vue 组件（或响应式渲染片段），替代逃生舱。

---

## 1. 现状调研要点（2026-08-10 待办）

> 实施前需在 auto-lang 完成下述探针（本项目仅为登记，不实施）。

| 探针 | 对象 | 问题 |
|---|---|---|
| `view fn` 现状 | `trans/rust.rs` / `ui_gen/vue.rs` | a2r 是否支持返回 widget 树的函数？`view` 保留字对函数名的冲突（Plan 5a 发现 `view` 不能做字段名） |
| `api.rs` 合成 AuraWidget 路径 | `crates/ui_gen/api.rs` | 转译器如何把 `.at` 的 widget 树合成 AuraWidget？`fn` 返回 AuraWidget 是否可走同一路径 |
| 组件 props fn 调用 | `expr_to_vue_bound_value`（Plan 022 已修 `Expr::Call`） | 修复后能否覆盖 `view fn` 作为组件 prop |
| `use { fn }` 的返回消费 | `vue.rs` store/widget 生成 | 逃生舱 fn 返回值被 codegen 如何对待（现为普通 import） |

## 2. 目标与验收标准

1. **原生表达**：`ChatMessage`（含消息 role 分支 + 卡片编排 + mention 高亮）用 `.at` 的 `view fn` 表达，不再依赖逃生舱 `.vue`。
2. **零 drift**：移除逃生舱后重新 `auto build`，生成工程 `vue-tsc && vite build` 全绿；逃生舱清理后 diff 产物稳定。
3. **行为一致**：生成工程 ChatView 的交互（流式渲染、卡片展开、mention、命令路由）与原生 `web/` 对齐（沿用 Plan 022 §7.6 parity 口径）。
4. **无回归**：auto-lang `cargo test -p auto-lang` 全绿（含新增 `view fn` a2vue golden）；auto-musk 全量构建通过。
5. **方法论复用**：`view fn` 能力同时惠及其他逃生舱点（useStreamingDocument 增量 JSON、questionnaire 解析、relay_commands 命令路由）。

## 3. 实施阶段（草案，实施时细化）

| 阶段 | 内容 | 验收 |
|---|---|---|
| **P1** | auto-lang 调研：`view fn` 语法设计（与 `use { fn }` 的关系）、`api.rs` 合成 AuraWidget 的最小改动、a2vue golden 基建扩展 | 探针结论 + 设计文档 |
| **P2** | 转译器：`view fn` 解析 + AuraWidget 合成 + Vue 组件生成（或响应式片段）；单测 + golden | `cargo test -p auto-lang` 绿 |
| **P3** | auto-musk 试点：以最简组件（如 UserMessage/mention 高亮）替换逃生舱验证能力 | 生成工程构建 + 行为 parity |
| **P4** | 全量迁移：ChatMessage 及其子组件、forge_helpers/relay_commands/questionnaire 逃生舱 TS | 逃生舱清零（或登记残留）+ 零 drift |
| **P5** | 文档归档：KNOWN-DEBT 更新 + 022 §8 后续项闭环 | 归档 |

## 4. 风险与降级

| 风险 | 级别 | 降级 |
|---|---|---|
| `view fn` 语法与现有 `use { fn }` 冲突 | 🟡 | 新关键字或后缀，向后兼容 |
| AuraWidget 合成路径复杂度超预期 | 🔴 | 分阶段：先支持"纯渲染 fn"（无状态），再支持"带 store 依赖 fn" |
| 流式/增量 JSON（useStreamingDocument）难以纯原生 | 🟡 | 保留逃生舱 + KNOWN-DEBT（Plan 022 已登记） |
| 迁移引发回归 | 🟡 | 每阶段 golden + 全量构建；P3 先单点验证 |

## 5. 与 auto-musk 的关系

- **不阻塞**：Plan 022 已用逃生舱达成功能 parity，本计划是"逃生舱渐进原生化"的大工程。
- **执行方**：auto-lang 仓库（独立 repo，`D:\autostack\auto-lang`）；auto-musk 仅作为验证与迁移对象。
- **状态追踪**：本文件驻留 auto-musk `docs/plans/`，实施在 auto-lang 侧开分支，完成后再回填本文件日志。
