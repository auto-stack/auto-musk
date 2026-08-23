---
plan_id: PLAN-033
status: review_done
feature_name: 计划模块 UI/UX 改进（过滤/状态/徽标/归档语义/MetaBlock）
author: [zhaopuming]
created_at: 2026-08-22T10:40:25+08:00
updated_at: 2026-08-23T15:12:00+08:00

supersedes_spec_components:
  - "backend/crates/musk/src/plans.rs: PlanStatus 状态机与 archive 语义重定义（reviewed/archived 单一终态）"
  - "web/src/views/PlansView.vue: 计划页过滤/头部/操作区重构（下拉过滤+转移按钮组+归档沉淀互斥）"
  - "web/src/types/plans.ts: 前端状态模型与状态机对齐（ALLOWED_TRANSITIONS/planStatusKey）"
  - "docs/designs/008-auto-plan.md: §7.2 状态机修订（reviewed 经 merge 进 archived 单一终态）"
  - ".agents/skills/auto-plan-*: 状态名与终态流程文案同步"
new_spec_components:
  - "web/src/components/plan/PlanMetaBlock.vue: 计划 frontmatter 元数据折叠概要/展开表格组件"
  - "web/src/utils/frontmatter.ts: 轻量 frontmatter 解析器（含 7 项单测）"
touched_goals: []

current_step: 12
total_steps: 12
---

# [PLAN-033] 计划模块 UI/UX 改进：过滤开关、状态重命名、徽标 i18n、归档语义统一、MetaBlock

## 变更摘要

针对 Auto/vue（`web/`，musk-web）"计划"页的五项 UI/UX 与状态模型改进：

1. 左侧列表过滤从"包含归档"checkbox 改为 **Active / All 下拉**，默认 Active，选择持久化。
2. 状态枚举重命名：`merged` → `archived`、`review_done` → `reviewed`（全栈 + 旧文件兼容映射）。
3. 状态徽标接入 i18n（中文模式显示中文）；删除头部冗余的全量状态下拉，状态展示以徽标为唯一来源，状态转移改为"仅列出合法目标"的按钮组。
4. 统一"归档 / 沉淀到 Spec"语义为**单一终态模型**：终态只有 `archived`；reviewed 的唯一出口是"沉淀到 Spec"（沉淀即归档），非 reviewed 计划走"归档"（搁置不沉淀）；两按钮互斥展示，后端同步门禁。
5. 计划正文不再渲染 frontmatter 黑体块；新增 **PlanMetaBlock** 组件：折叠时概要一行，展开后表格显示全部 meta 属性。

## 目标

- 消除状态信息的三处冗余/混排：英文徽标 vs 中文下拉 vs archived 灰标签。
- 让按钮与状态机语义一致：任一时刻"终态动作"按钮至多一个，且名称与结果状态对应。
- 旧数据零迁移成本：磁盘上 `status: merged` / `review_done` 的历史文件读取时自动映射，写入时自愈为新枚举。
- frontmatter 作为元数据被结构化呈现，正文渲染区只渲染正文。

## 架构方案

不改 API 形状（路由、请求/响应 DTO 均不变），只改语义：

- **后端** `backend/crates/musk/src/plans.rs`：`PlanStatus` 枚举变体重命名 + `from_str_lossy` 兼容旧值；`can_transition` 状态机去掉 `reviewed → archived` 手动转移；`archive()` 升级为"置状态 + 移档"并拒绝 reviewed 计划；`merge_plan_stores()` 沉淀后经同一助手进入终态。
- **前端** `web/src/types/plans.ts` 保持为前端单一事实源（枚举、转移表、色调、i18n key 映射），`PlansView.vue` 只做视图编排；新增 `web/src/utils/frontmatter.ts`（纯函数解析）与 `web/src/components/plan/PlanMetaBlock.vue`（展示组件）。
- 跨栈一致性继续靠 `backend/crates/musk/tests/parity_plans.rs` 与约定（无共享常量文件，本计划不引入）。

## 技术栈

Rust（axum hand-written routes）、Vue 3 `<script setup>` + TS、vue-i18n 9（legacy:false）、原生 `<select>`/按钮（无新增 UI 依赖）、Vitest + vue-tsc。**不引入 yaml 依赖**——frontmatter 为扁平键值 + 简单数组，自写 ~40 行解析器并配单测。

## 需求分析与背景调查

 Specs 台账（`docs/specs/`：`00-overview.md`、`01-architecture.md`、`03-front-component-groups.md`、`goals/`、`modules/`、`reviews/`、`index.json`）中与本计划相关的是 `03-front-component-groups.md` 所辖的 web 组件分组——新增 `PlanMetaBlock` 组件、改 `PlansView`/`PlanStatusBadge` 属于该分组的扩展，merge 时需回填。

现状事实（已逐一核对源码）：

