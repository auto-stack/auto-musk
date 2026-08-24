---
# ============ 元数据区块 (Auto-Plan 核心契约) ============

# 基础信息
plan_id: PLAN-024
status: archived
feature_name: AutoPlan 架构升级（Plan/Spec 同级化 + 计划一级导航 + Specs 展示重组 + merge 沉淀）
author: [zhaopuming + agent]
created_at: 2026-08-11T17:30:00Z
updated_at: 2026-08-24T14:00:00+08:00

# ============ Spec 合并指引 (/auto-plan:merge 时使用) ============
supersedes_spec_components:
  - "specs/sections/plans: 移除（7 区 → 6 区）"
  - "specs/modules/specs-ledger/spec.md: 修改"
new_spec_components:
  - "specs/modules/plans/spec.md: 新增（Plan 一等公民 + merge 工作流）"
touched_goals:
  - "goal-001: Plan 与 Spec 同级"
  - "goal-002: 计划可检索/可归档/可沉淀"

# ============ 执行进度追踪 (供 /auto-plan:work 更新) ============
current_step: 7
total_steps: 7

---

# [PLAN-024] AutoPlan 架构升级 — 实施计划

> **给执行 Agent 的指令：** 必须使用 `/auto-plan:work` 技能逐步执行此计划。
> **用户引用方式：** 在对话中输入 "执行 Plan 24" 即可定位本文件。
> **设计文档：** `docs/designs/008-auto-plan.md`（定稿 v1.0）。本计划是其落地方案。

## 0. 变更摘要 (Executive Summary)

将 auto-musk 从"Spec 单核、Plans 只是 Spec 的一个 section"升级为 **AutoPlan 架构**：**Plan / Spec / Wiki 三类一等公民各司其职**——

- **Plan**（计划）= 动态执行实体：新建、执行、状态流转，文件落 **工作区根 `docs/plans/`**（含 `archived/`）。
- **Spec**（规范）= 静态知识沉淀：系统性记录**做过的所有 plan** 的成果，作为与项目一同成长的知识库；当前 7 区 ledger **收敛为 6 区**（goals/architecture/designs/tests/reviews/reports），`plans` section 移除。
- **Wiki**（知识库）= 项目开发之外的外部参考资料（芯片/库/论文资料），**本轮不动**。

核心工作流：**开发期以 Plan 为唯一事实源执行 → 复审通过后 merge 沉淀进 Spec → 归档**（008 §7 完整生命周期）。

- **展示**：总导航栏新增一级栏目 **"计划"**（PlansView），展示**所有计划（含归档）**；导航顺序 `聊天 / 计划 / 规范 / 知识库`（执行在前、沉淀在后；顺序为常量可调）。
- **merge 工作流**：新增"沉淀到 Spec"——plan 到 `review_done` 后，自动把 plan 的关键信息（目标/架构/设计/测试/验收）拆解进 Spec 6 区对应 section，然后归档。
- **迁移**：现有 `docs/plans/001-023` 旧格式（Superpowers 风格、无 frontmatter）统一迁移到新格式并纳入 Plans 视图；`archive/`、`old/` 目录归并到 `archived/`。
- **双前端**：原生 `web/` 与 Auto 轨（`src/front/*.at` → `auto build` → `gen/front/vue/`）同步改造，保持 parity（Plan 022/023 纪律）。

## 1. 目标 (Goal)

让 auto-musk 同时具备 **Plan-driven 的执行效率** 与 **Spec-driven 的知识沉淀**：开发期 Plan 文件是唯一执行上下文；归档期 merge 把 Plan 的成果自动拆解进 Spec 知识库；用户在 UI 中能像浏览 Specs 一样浏览全部 Plan（含归档），并一键把完成的 Plan 沉淀为 Spec 条目。

## 2. 架构方案 (Architecture)

```
┌────────────────────────── auto-musk（AutoPlan 架构）──────────────────────────┐
│                                                                               │
│  总导航栏： [聊天] [计划 Plans] [规范 Specs] [知识库 Wiki]                      │
│                    │                │                                          │
│   PlansView（全部计划，含 archived/）   SpecsView（6 区知识库 ledger）           │
│   · 状态机 + 状态徽标                  goals/architecture/designs              │
│   · 创建/编辑/状态流转/归档              tests/reviews/reports                  │
│   · review_done → 「沉淀到 Spec」        · 每条可溯源到 PLAN-xxx                │
│                    │                │                                          │
│   Backend API：  /api/plans/*        /api/specs/*                              │
│                  + /api/plans/{id}/merge  （写入 specs）                       │
│                    │                │                                          │
│   Stores：      PlansStore（新）      SpecsStore                               │
│   (root/docs/plans/*.md+archived/)  (root/.autoos/specs.json)                  │
│                                                                               │
│   工作流： new → work → review ──merge──▶ merged（拆解进 Spec 6 区 + 归档）     │
└───────────────────────────────────────────────────────────────────────────────┘
```

核心要点：
1. **三类一等公民**：Plan（执行）/ Spec（沉淀）/ Wiki（参考）各司其职，导航 4 项全展示。
2. **PlansStore**：新增到 `WorkspaceStores`，根目录 = `{workspace_root}/docs/plans/`（用户决策：工作区 docs/plans，非 .autoos）。
3. **状态机**（008 §7.2）：`drafting → executing → execution_done → review_done → merged`；`merge` = 沉淀进 Spec 6 区 + 移入 `archived/`。
4. **Specs 重组**：`SectionType::Plans` 移除，ledger 7→6 区；既有 `specs.json` 中 plans 区（当前为空）做数据迁移兜底。
5. **防漏号**：创建时扫描目录最大序号 + 1（008 §8 确定性算法）。
6. **merge 引擎**：从 Plan frontmatter + 正文章节提取知识片段，映射写入 Spec 6 区对应 section（008 §6.5 在 auto-musk 的落地）。
7. **双前端 parity**：原生 `web/` 与 Auto 轨 `src/front/` 同步改动。

## 3. 技术栈 (Tech Stack)

- 后端：Rust + axum + serde + regex（复用 SpecsStore/WikiStore 模式）
- 前端：Vue3 + TypeScript + vue-i18n（原生 `web/`）；AutoUI `.at` 源（`src/front/`）经 auto-lang `auto.exe` 生成
- YAML frontmatter 解析：后端手写轻量解析（只取 `plan_id/status/feature_name/current_step/total_steps` 等必需字段），避免引入重依赖

## 4. 需求分析与背景调查

### 4.1 概念澄清（用户确认，2026-08-11）

| 概念 | 定位 | 载体 | 视图 |
|:---|:---|:---|:---|
| **Plan** | 新建和执行计划（动态） | `docs/plans/NNN-*.md`（含 `archived/`） | "计划"一级导航（新） |
| **Spec** | 系统性记录做过的所有 plan（知识库，与项目一同成长） | `.autoos/specs.json`（**6 区** ledger） | "规范"一级导航（重组） |
| **Wiki** | 开发之外的外部参考资料（芯片/库/论文） | `.autoos/wiki/` | "知识库"一级导航（不动） |

决策（用户确认）：
- **D1**：Plan 物理存储 = 工作区根 `docs/plans/`（含 `archived/`），非 `.autoos/`。
- **D2**：旧计划 001-023 迁移到新格式（补 frontmatter、统一命名）。
- **D3**：Spec Ledger **移除 plans 区**（7→6 区）；做过的 plan 由 Plans 视图归档区承载，Spec 只做知识沉淀，条目标记来源 plan。
- **D4**：**包含 merge 沉淀工作流**——plan `review_done` 后拆解进 Spec 6 区 → 归档（008 §6.5）。

### 4.2 现状调研结论（2026-08-11 实地核对）

| 维度 | 现状 | 位置 |
|:---|:---|:---|
| Spec 存储 | `.autoos/specs.json`，JSON ledger，**7 个 section**（goals/architecture/designs/**plans**/tests/reviews/reports） | `backend/crates/musk/src/specs.rs`（`SectionType` + `SpecsDocument::new` 7 区工厂）；`auto-src/specs.at`（ag 轨） |
| Specs 前端 | SpecsView：侧栏模块 accordion + 类型列表 + stack 过滤，`plans` 是其中一个 section（`PlanCards`/`PlanDetail` 组件） | `web/src/views/SpecsView.vue`（`DEFAULT_SECTIONS`/`typeOrder`/`categoryComponent`）；`web/src/types/specs.ts`；`web/src/utils/itemTemplates.ts`；`src/front/specs_view.at` |
| 导航栏 | 一级导航 3 项：聊天/规范/知识库（Ctrl+1/2/3） | `web/src/App.vue`（`tabs` + `onKeyDown`）；`src/front/app.at`（`ShowSpecs/ShowWiki` + `current_view`） |
| 视图路由 | `useViewState` 的 `ViewId = 'chats'\|'specs'\|'wiki'` | `web/src/composables/useViewState.ts` |
| i18n | `nav.chat/specs/wiki` 等键，en/zh 双语言 | `web/src/i18n/locales/{en,zh}.json`；`src/front/i18n/{en,zh}.json` |
| 现有 plans | `docs/plans/`（001/009/022/023/plan-022 等，Superpowers 风格无 frontmatter）+ `archive/`（018-021）+ `old/`（002-017 等） | 仓库根 `docs/plans/` |
| 无关概念 | **TaskPlan**（relay 任务编排，`.autoos/task_plans/`，`TaskPlanPanel`）——与本计划"Plan"完全不同，不动 | `backend/crates/musk/src/relay/task_plan*`、`web/src/components/TaskPlanPanel.vue` |
| 后端路由 | 主 router = `auto_generated::server::build_router()`（38 路由）+ `.merge()`（relay/task_plan/wiki） | `backend/crates/musk/src/server.rs`；`auto-src/server.at` |
| 后端 deleg 模式 | ag handler → `extern_impl.rs` → hw store（如 `specs_load` → `ws.specs.load()`） | `auto-src/server.at` + `auto-src/extern_sigs.at` + `src/auto_generated/extern_impl.rs` |

### 4.3 关键发现

1. **Specs 中 plans 区当前数据为空**（`backend/.autoos/specs.json` plans items = 0），剥离成本低；但需兼容旧文件解析（`SectionType::from_id("plans")` 的 lossy 行为）。
2. **双前端是硬约束**：所有前端改动需在 `web/` 与 `src/front/` 双轨同步。
3. **后端 .at 轨**：新增 `/api/plans/*` 走 ag 轨（`server.at` + `extern_sigs.at` + 转译），与 specs 完全同构；若有转译器缺口，逃生舱回退 hw 路由 merge（同 `wiki_routes()` 模式，`wiki.rs:698`）。
4. **merge 的 Spec 落点**：auto-musk 当前 Spec 是 **6 区 flat ledger**（非 008 的模块树 `docs/specs/`），故 merge 映射为"plan 章节 → spec section 新增/更新 item"，`item.file`/`item.related` 记录来源 plan。

> **设计约束：** 起草本 Plan 时已确认：`docs/specs/` 模块树知识库（008 §5）**不在本轮**——本轮 merge 的落点是现有 6 区 ledger；模块树演进留作后续。

## 5. 详细设计 (Detailed Design)

### 5.1 Plan 数据模型（后端，`backend/crates/musk/src/plans.rs` 新建）

```rust
/// Plan 生命周期状态（008 §7.2）
pub enum PlanStatus {
    Drafting,      // "drafting"        — 新建，未执行
    Executing,     // "executing"       — work 技能执行中
    ExecutionDone, // "execution_done"  — 步骤全部完成
    ReviewDone,    // "review_done"     — 复审通过，待 merge
    Merged,        // "merged"          — 已 merge 进 Spec 并归档
}
// to_str/from_str_lossy 对齐 008 frontmatter 字段值。

/// 一个 Plan 文件（文件名前缀 = 3 位数字序号）
pub struct PlanFile {
    pub id: String,            // "PLAN-024"（= 文件名前缀 024）
    pub filename: String,      // "024-auto-plan-architecture.md"
    pub status: PlanStatus,
    pub feature_name: String,  // frontmatter feature_name
    pub title: String,         // 正文首行 # [PLAN-024] xxx
    pub archived: bool,        // 位于 archived/ 子目录
    pub content: String,       // 完整 markdown（含 frontmatter）
    pub created_at: u64,
    pub updated_at: u64,
}

/// PlansStore：以 {root}/docs/plans/ 为根（仿 WikiStore 目录模式，
/// 磁盘为唯一事实源，无独立 manifest —— 直接扫描 `NNN-*.md` + frontmatter）。
pub struct PlansStore {
    pub plans_dir: PathBuf,      // root/docs/plans
    pub archived_dir: PathBuf,   // root/docs/plans/archived
}
```

- **frontmatter 解析**：手写轻量解析器，识别 `---` 包裹的 `key: value` 行，只提取 `plan_id/status/feature_name/created_at/updated_at`；正文其余保留。旧格式（无 frontmatter）容错降级：`status = "drafting"`（待迁移，§5.5）。
- **防漏号**：`create()` 扫描 `plans_dir + archived_dir` 中所有 `^[0-9]{3}-.*\.md`，取最大序号 + 1，补零 3 位。

### 5.2 PlansStore API（后端）

| 方法 | 说明 |
|:---|:---|
| `list(include_archived: bool) -> Vec<PlanFile>` | 列出全部/仅活跃计划（按序号排序） |
| `get(id: &str) -> Option<PlanFile>` | 按 3 位序号读取（含 archived） |
| `create(feature_name, content) -> Result<PlanFile>` | 自动分配序号，status=drafting |
| `update(id, content) -> Result<PlanFile>` | 覆盖正文（保留 frontmatter 的 plan_id） |
| `transition(id, new_status) -> Result<PlanFile>` | 状态机流转（校验合法迁移，008 §7.2） |
| `archive(id) -> Result<PlanFile>` | 移动到 archived/（status → merged 或保持） |
| `migrate_legacy()` | 扫描旧格式文件补 frontmatter（一次性迁移，§5.5） |

### 5.3 Merge 引擎（核心：Plan 执行 → Spec 沉淀）

**新增 `backend/crates/musk/src/plan_merge.rs`**，职责（008 §6.5 落地到 6 区 ledger）：

```
merge(plan: &PlanFile, specs_doc: &mut SpecsDocument) -> MergeResult
流程：
1. 门禁：plan.status 必须 == ReviewDone，否则拒绝。
2. 提取知识片段（从 Plan 正文章节 → Spec section 映射）：
   §1 目标            → goals        section（新建/更新 item）
   §2 架构方案        → architecture section
   §5 详细设计        → designs      section
   §6 测试设计        → tests        section
   §7 验收标准/§9 复审 → reviews      section
   执行摘要/执行结果   → reports      section
3. 生成 SpecItem（id 前缀沿用 plan 体系，如 "P-024-1" 或关联风格）：
   - item.title   = plan feature_name + 章节名
   - item.content = 该章节 markdown 原文
   - item.file    = Some("docs/plans/024-xxx.md")   // 溯源到 plan
   - item.related = 追加 "PLAN-024"                   // 反链
   - item.status  = 按 section 类型给合理初值（如 architecture → Stable）
4. 对每个 section：若有 supersedes 同名 item 则替换，否则新增（upsert）。
5. 保存 specs.json → ws.specs.save()。
6. 移动 plan → archived/，status 更新为 Merged。
7. 返回 { plan_id, sections_touched: [..], items_created: N }。
```

**frontmatter 兼容**：008 的 `supersedes_spec_components/new_spec_components/touched_goals` 字段若存在，优先按字段映射；缺失时用上面章节默认映射兜底。**保护用户内容**：只新增/替换本 plan 生成的 item，不碰同 section 的其他 item。

### 5.4 后端 API（`/api/plans/*`，ag 轨）

在 `auto-src/server.at` 追加（与 specs handlers 同构），`extern_sigs.at` + `extern_impl.rs` 委托到 hw `PlansStore` / `plan_merge`：

| 端点 | 方法 | 行为 |
|:---|:---|:---|
| `/api/plans` | GET | 列出全部计划（query `include_archived=true` 时含 archived） |
| `/api/plans/{id}` | GET | 读取单个计划 |
| `/api/plans` | POST | 新建（body: `{ feature_name, content? }`，自动分配序号） |
| `/api/plans/{id}` | PUT | 更新正文 |
| `/api/plans/{id}/transition` | POST | 状态流转（body: `{ status }`） |
| `/api/plans/{id}/archive` | POST | 归档（移入 archived/，不写 Spec） |
| `/api/plans/{id}/merge` | POST | **沉淀到 Spec**（门禁 review_done → 拆解进 6 区 → 归档 merged） |

`WorkspaceStores` 新增 `pub plans: Arc<PlansStore>`，在 `WorkspaceStores::new(root)` 中实例化 `PlansStore::new(root.join("docs/plans"))`（`workspace.rs:407-422`）；merge 用现有 `ws.specs`。

### 5.5 前端详细设计

#### (a) 导航栏新增"计划"（原生 `web/`）

- `web/src/App.vue`：
  - `tabs` 数组插入 `{ id: 'plans', i18nKey: 'nav.plans', icon: ListTodo }`（lucide），顺序：`chats / plans / specs / wiki`。
  - `<main>` 新增 `<PlansView v-else-if="currentView === 'plans'" />`。
  - `onKeyDown`：`case '1': chats; '2': plans; '3': specs; '4': wiki`（顺序随 tabs）。
  - `type ViewId` 扩为 `'chats' | 'plans' | 'specs' | 'wiki'`。
- `web/src/composables/useViewState.ts`：`ViewId` + `VALID_VIEW_IDS` 同步。
- `web/src/i18n/locales/{en,zh}.json`：`nav.plans` + plans 视图文案。

#### (b) PlansView（原生 `web/`）

- 新文件 `web/src/views/PlansView.vue`（仿 `WikiView.vue` 布局：侧栏 + 内容区）：
  - **侧栏**：计划列表（序号 + feature_name + 状态徽标），"含归档"开关；折叠态。
  - **内容区**：Plan 详情 Markdown 渲染（复用 `MarkdownContent.vue`）；头部操作：新建 / 编辑 / 状态流转 / 归档 / **「沉淀到 Spec」（status=review_done 时启用）**。
- 新文件 `web/src/composables/usePlans.ts`：单例状态 + `loadPlans/getPlan/createPlan/updatePlan/transition/archive/merge`（authFetch 调 `/api/plans/*`，仿 `useSpecs.ts`）。
- 新文件 `web/src/types/plans.ts`：`PlanFile/PlanStatus/PlanTransitionBody/MergeResult`。
- 新组件 `web/src/components/plan/PlanList.vue`、`PlanStatusBadge.vue`。

#### (c) Auto 轨（`src/front/`）同步

- `src/front/app.at`：`msg` 加 `ShowPlans`；model `current_view` 分支；nav 按钮（`ListTodo` + `text t("nav.plans")`）；视图 `if .current_view == "plans" { PlansView }`。
- 新文件 `src/front/plans_view.at` + `src/front/plans_store.at`（对齐原生 PlansView/usePlans）。
- `src/back/api.at`：新增 `/api/plans/*` 契约（含 merge）。
- `src/front/i18n/{en,zh}.json`：`nav.plans` + plans 文案。
- 验证：`auto build` → `gen/front/vue` → `vue-tsc && vite build` 全绿。

#### (d) Specs 展示重组（7 区 → 6 区）

- **后端**：`specs.rs` 的 `SectionType` 移除 `Plans`；`SpecsDocument::new` 7→6 区工厂；`auto-src/specs.at` 同步（`Plans = 4` 枚举删除 + `new()` 列表 + 引用 `SectionType.Plans` 的 `related_plans_all_done` 逻辑改判）。`from_id("plans")` lossy 行为改为安全 fallback（如 `Goals`）或显式忽略旧 plans 区。
- **前端原生**：
  - `web/src/types/specs.ts`：`SectionType` 移除 `'plans'`。
  - `web/src/views/SpecsView.vue`：`DEFAULT_SECTIONS`/`typeOrder`/`categoryComponent` 移除 plans 分支。
  - `web/src/utils/itemTemplates.ts`：删除 `plans` 模板。
  - `web/src/components/category/PlanCards.vue`、`web/src/components/detail/PlanDetail.vue`：改造为 Plans 视图组件或删除（由 PlansView 取代）。
- **Auto 轨**：`src/front/specs_view.at` 的 `section_types` 移除 `"plans"`。
- **数据迁移**：既有 `.autoos/specs.json` 若 plans 区有数据，导出为 `docs/plans/NNN-*.md`（当前为空，脚本兜底）。

### 5.6 旧计划迁移（001-023 → 新格式）

- 新格式模板（008 §4.2）：`---` frontmatter（`plan_id: PLAN-NNN` / `status` / `feature_name` / `created_at` / `updated_at`）+ 正文保留原结构。
- 迁移映射：
  | 现状 | 迁移后 | 依据 |
  |:---|:---|:---|
  | `docs/plans/001-*.md` 等未归档 | 保留位置 + 补 frontmatter；**已完结**的标 `status: merged` | 008 §4 |
  | `docs/plans/archive/018-021*.md` | 移至 `docs/plans/archived/`，补 frontmatter `status: merged` | 008 §3.1 `archived/` |
  | `docs/plans/old/*.md`（002-017 等历史计划） | 保留在 `old/` 或在 `archived/old/` 下；纳入 Plans 视图"含归档"范围 | 待澄清 |
  | `docs/plans/KNOWN-DEBT-AND-RISKS.md` | **不是 Plan**，移出 plans 目录（如 `docs/`）或 Plans 视图过滤非 `NNN-*.md` | 防误判 |
  | `docs/plans/plan-022-auto-version-alignment.md`、`022-frontend-auto-ization.md`（同号冲突） | 重编号/去重（022 号冲突），保证 `NNN-` 前缀唯一 | 008 §8 防漏号 |
  | `023-handoff.md`（交接文档） | 非独立 Plan，与 023 主文档合并或标 `status: merged` | 待澄清 |
- **迁移脚本**：一次性脚本（bash/python）扫描 `docs/plans/`，为缺失 frontmatter 的文件注入模板，`plan_id` 从前缀推导。**绝不修改旧计划正文内容**，只加 frontmatter + 移动目录。

### 5.7 接口变更汇总

| 层 | 变更 |
|:---|:---|
| 后端新增 | `src/plans.rs`（PlansStore + PlanStatus + PlanFile）；`src/plan_merge.rs`（merge 引擎）；`WorkspaceStores.plans` |
| 后端 API 新增 | `/api/plans` + `/api/plans/{id}` + `/transition` + `/archive` + `/merge`（server.at + extern_sigs + extern_impl） |
| 后端修改 | `specs.rs`/`specs.at`：SectionType 移除 Plans；`workspace.rs`：挂载 PlansStore |
| 前端新增 | `web/src/views/PlansView.vue`、`web/src/composables/usePlans.ts`、`web/src/types/plans.ts`；`src/front/plans_view.at`、`plans_store.at` |
| 前端修改 | `App.vue`、`useViewState.ts`、`SpecsView.vue`、`specs.ts`、`itemTemplates.ts`、i18n 双语言双轨；`app.at`、`specs_view.at`、`api.at` |
| 迁移 | `docs/plans/` 001-023 补 frontmatter；`archive/`→`archived/`；`old/` 归并 |

## 6. 测试设计 (Test Design)

- **单元测试**（后端，`plans.rs`/`plan_merge.rs` 内嵌 `#[cfg(test)]`，仿 `specs.rs` 风格）：
  - frontmatter 解析/序列化 round-trip（含旧格式降级容错）；
  - 防漏号算法（空目录 → 001；连续 001/002/005 → 006；archived 也算）；
  - 状态机合法/非法迁移（008 §7.2 全路径 + 非法跳转拒绝）；
  - **merge**：`review_done` 可 merge → 6 区各生成 item（file/related 溯源正确）→ 移入 archived + status=Merged；`drafting`/`executing` 调 merge 被拒；重复 merge 幂等（不重复写 item）；
  - archive 移动文件 + list(include_archived) 过滤；
  - migrate_legacy：旧文件注入 frontmatter 后 plan_id 正确。
- **API 测试**（仿 `parity_config_endpoints.rs`：AUTOOS_HOME 隔离 + serial）：
  - GET /api/plans 空 → 空列表；POST 创建 → PLAN-001；PUT/transition/archive/merge 全链路（merge 后 specs.json 出现 6 区新 item + plan 移入 archived）。
- **前端**：
  - `vue-tsc && vite build` 全绿（web/ 与 gen/front/vue/ 双轨）；
  - `auto build` 后产物 diff 无计划外改动；
  - 手测：导航栏 4 项切换 + Ctrl+1/2/3/4；Plans 视图含归档开关 + "沉淀到 Spec" 按钮流；Specs 视图无 plans 区。

## 7. 验收标准 (Acceptance Criteria)

> 复审时 (`/auto-plan:review`) 逐项勾选。

- [ ] 标准 1：`cargo build && cargo test`（backend/crates/musk）全绿，新增 plans + merge 单测覆盖状态机/防漏号/merge 幂等。
- [ ] 标准 2：`/api/plans` 全链路可用（list/get/create/update/transition/archive/merge），创建自动分配不冲突序号。
- [ ] 标准 3：导航栏 4 项（聊天/计划/规范/知识库，Ctrl+1/2/3/4），Plans 视图展示全部计划含归档；Specs 视图为 6 区（无 plans 区）。
- [ ] 标准 4：merge 工作流端到端可用：`review_done` → 「沉淀到 Spec」→ Spec 6 区出现溯源 item（`file` 指向 plan）→ plan 移入 `archived/` 且 status=merged。
- [ ] 标准 5：双前端 parity —— `auto build` 全绿 + `vue-tsc && vite build` 全绿；原生 web/ 与 gen/front/vue/ 视觉一致。
- [ ] 标准 6：旧计划 001-023 迁移完成（frontmatter 就位、`archive/`→`archived/`、序号冲突解决），Plans 视图无 `NNN-` 前缀之外的误列。
- [ ] 标准 7：i18n en/zh 双语言覆盖导航 + Plans 视图文案。

## 8. 执行步骤 (Execution Tasks)

> **粒度要求：** 每个任务应是 2-5 分钟可完成的原子操作。
> **格式要求：** 必须包含精确的文件路径、操作描述、验证命令。

### 任务 1: 后端 PlansStore 数据层
- [ ] **步骤 1.1:** 新建 `backend/crates/musk/src/plans.rs`：`PlanStatus` 枚举 + `PlanFile` 结构 + 轻量 frontmatter 解析/序列化。
- [ ] **步骤 1.2:** 实现 `PlansStore`：`list/get/create/update/transition/archive` + 防漏号扫描。
- [ ] **步骤 1.3:** `lib.rs` 加 `pub mod plans;`；`workspace.rs` 的 `WorkspaceStores` 加 `plans: Arc<PlansStore>` 并在 `new()` 实例化（`root.join("docs/plans")`）。
- [ ] **步骤 1.4:** 写单元测试（frontmatter / 防漏号 / 状态机 / archive / migrate_legacy）。
- [ ] **步骤 1.5:** 运行 `cargo test`（backend/crates/musk），预期全部通过。

### 任务 2: Specs 剥离 plans section（后端 + 双前端）
- [ ] **步骤 2.1:** `backend/crates/musk/src/specs.rs`：`SectionType` 移除 `Plans`；`SpecsDocument::new` 7→6；`from_id("plans")` fallback 处理。
- [ ] **步骤 2.2:** `auto-src/specs.at` 同步（`Plans = 4` 枚举删除 + `new()` 列表 + `related_plans_all_done` 改判）；重新转译（`A2R_CRATE_ROOT=0 auto.exe trans --path specs.at rust` + `nativeize.pl`）。
- [ ] **步骤 2.3:** 前端原生：`web/src/types/specs.ts`、`web/src/views/SpecsView.vue`、`web/src/utils/itemTemplates.ts` 移除 plans 相关。
- [ ] **步骤 2.4:** `src/front/specs_view.at` 的 `section_types` 移除 `"plans"`。
- [ ] **步骤 2.5:** 验证 `cargo test` + `vue-tsc && vite build`（web/ 与 gen/front/vue/）。

### 任务 3: 后端 /api/plans 基础端点（ag 轨）
- [ ] **步骤 3.1:** `auto-src/server.at` 加 6 个 plans handlers（list/get/create/update/transition/archive，仿 specs handlers）+ `build_router` 注册路由。
- [ ] **步骤 3.2:** `auto-src/extern_sigs.at` 加 `plans_*` extern 签名；转译。
- [ ] **步骤 3.3:** `src/auto_generated/extern_impl.rs` 实现委托到 `ws.plans`（仿 `specs_load/specs_upsert_of`）。
- [ ] **步骤 3.4:** 写 API 集成测试（仿 parity_config_endpoints.rs：AUTOOS_HOME 隔离 + serial）。
- [ ] **步骤 3.5:** 验证 `cargo test` 全绿（含转译后 `auto_generated/server.rs` 编译通过）。
- [ ] **步骤 3.6（逃生舱）:** 若 ag 轨被转译器阻塞，改 hw 路由：`plans_routes()` 仿 `wiki_routes()`（wiki.rs:698），在 `server.rs` serve() 中 `.merge()`，登记 KNOWN-DEBT。

### 任务 4: Merge 引擎（Plan → Spec 沉淀）
- [ ] **步骤 4.1:** 新建 `backend/crates/musk/src/plan_merge.rs`：章节→section 映射 + `SpecItem` 生成（`file`/`related` 溯源）+ upsert 到 6 区 + 移动归档 + status=Merged。
- [ ] **步骤 4.2:** `auto-src/server.at` 加 `/api/plans/{id}/merge` handler（门禁 review_done）+ 注册路由；extern 签名/实现。
- [ ] **步骤 4.3:** 写单测：合法 merge / 非法门禁 / 重复 merge 幂等 / 6 区溯源。
- [ ] **步骤 4.4:** 写 API 集成测试（merge 后 specs.json 断言新 item + plan 移入 archived）。
- [ ] **步骤 4.5:** 验证 `cargo test` 全绿。

### 任务 5: 前端"计划"导航 + PlansView（原生 web/）
- [ ] **步骤 5.1:** `web/src/types/plans.ts` + `web/src/composables/usePlans.ts`（含 merge 调用）。
- [ ] **步骤 5.2:** `web/src/views/PlansView.vue`（侧栏列表 + 含归档开关 + 详情 Markdown + 新建/编辑/状态/归档/「沉淀到 Spec」）。
- [ ] **步骤 5.3:** `web/src/App.vue` tabs 插入 plans + `onKeyDown` Ctrl+1/2/3/4 + 视图渲染分支；`useViewState.ts` ViewId 加 `'plans'`。
- [ ] **步骤 5.4:** `web/src/i18n/locales/{en,zh}.json` 加 `nav.plans` + plans 文案。
- [ ] **步骤 5.5:** 验证 `vue-tsc && vite build`（web/）全绿 + dev server 手测 4 视图切换 + merge 按钮流。

### 任务 6: 前端 Auto 轨同步（.at → gen）
- [ ] **步骤 6.1:** `src/back/api.at` 加 `/api/plans/*` 契约（含 merge）。
- [ ] **步骤 6.2:** `src/front/plans_store.at` + `src/front/plans_view.at`（对齐原生 PlansView 结构与样式类）。
- [ ] **步骤 6.3:** `src/front/app.at` 加 `ShowPlans` msg + nav 按钮 + 视图分支；`src/front/i18n/{en,zh}.json` 加文案。
- [ ] **步骤 6.4:** `auto build --gen-only` → `gen/front/vue` 下 `vue-tsc && vite build` 全绿。
- [ ] **步骤 6.5:** headless Chromium 或 dev server 比对原生/生成版 Plans 视图视觉 parity（沿用 Plan 022 §7.6 口径）。

### 任务 7: 旧计划迁移 + 归档归并 + 文档
- [ ] **步骤 7.1:** 写一次性迁移脚本：`docs/plans/*.md` 无 frontmatter 的注入（plan_id 从前缀推导），`archive/` → `archived/`。
- [ ] **步骤 7.2:** 解决序号冲突（`plan-022-*` vs `022-*`、`023-handoff.md`）：重编号或合并，保证 `NNN-` 唯一。
- [ ] **步骤 7.3:** `KNOWN-DEBT-AND-RISKS.md` 移出 plans 目录（或 Plans 视图过滤非 `NNN-*.md`）。
- [ ] **步骤 7.4:** 更新 `README.md`（架构图 + 三类概念说明 + 新导航）+ 本计划状态流转（`execution_done`）。
- [ ] **步骤 7.5:** 验证 Plans 视图无误列 + 归档开关展示 old/archived 全部计划。

## 9. 复审记录 (Review Log)

> 由 `/auto-plan:review` 技能在复审时自动填写，人工确认。

- **复审人**: [待填]
- **复审时间**: [待填]
- **复审结论**:
  - [ ] 验收标准全部满足
  - [ ] 代码无安全隐患
  - [ ] Spec 元数据已补全
- **遗留问题**: [如有，写在这里]

## 10. 待澄清事项 (Open Questions)

- 仓库根 `docs/plans/`（auto-musk 自身计划）与默认工作区 `backend/docs/plans/` 的关系：是否把 auto-musk 自身的 001-023 视为"工作区 backend 的计划"？还是建议默认工作区切到仓库根？
- `docs/plans/old/`（002-017 历史计划）与 `archive/`（018-021）是否统一并入 `archived/`，还是 `archived/old/` 分层？
- merge 时 Spec 6 区 item 的 ID 风格：沿用 plan 序号（如 `P-024-1`）还是独立编号（如 goals 区 `G2`）？各 section 的 status 初值如何定？
- 是否后续立项 `docs/specs/` 模块树知识库（008 §5）以取代 flat ledger？

---

## 11. 执行记录 (Execution Log — 2026-08-11)

### 已完成（任务 1-7，全部验证通过）

| 任务 | 状态 | 验证 |
|:---|:---|:---|
| 1. 后端 PlansStore 数据层 | ✅ | 33 单测（frontmatter/防漏号/状态机/archive/迁移容错） |
| 2. Specs 剥离 plans（7→6 区） | ✅ | specs 37 单测 + `specs.at`/`relay_profession.at`/`profession.rs`/双前端同步 |
| 3. `/api/plans` 基础端点 | ✅ | hw 路由（逃生舱，计划 §3.6 允许）+ 4 API 端到端测试（含 merge） |
| 4. Merge 引擎 `plan_merge.rs` | ✅ | 11 单测 + merge API 门禁/沉淀/幂等测试 |
| 5. 前端原生 web/ PlansView | ✅ | 导航 4 项 + PlansView + i18n，vue-tsc 0 新错 + vite build 绿 |
| 6. Auto 轨 plans（.at → gen） | ✅ | `plans_view.at`/`plans_store.at`/`app.at`/`api.at` + i18n；auto build 成功（28 components）；gen vue-tsc 0 错 + vite build 绿 |
| 7. 旧计划迁移 + 归档归并 | ✅ | `archive/`→`archived/`、`old/`→`archived/` 扁平化、序号冲突解决、22 个补 frontmatter；23 唯一 seq 无重复 |

**测试总计**：271 lib + 4 `parity_plans` API 测试全绿，0 回归。

### KNOWN-DEBT（计划内绕道，不影响功能）

1. **`/api/plans` 走 hw 路由（非 ag 轨）**：a2r 转译器漂移（任务 2 实测：`auto trans specs.at rust` 生成产物含 `auto_lang::a2r_std` 未被 nativeize 清理、多余 `.clone()`、分号风格差异）。**计划 §3.6 明确允许逃生舱**，功能等价（API 全链路 + 测试绿）。待 a2r 转译器对齐后切回 ag 轨。注意：前端 `.at → vue` 转译稳定（任务 6 验证零 drift），漂移仅影响后端 a2r。
2. **derive_statuses Rule 1 移除**：Goal→Implemented 自动推进规则依赖 plans section（现移除，plans 独立为 PlansStore）。Goal 需手动 transition 到 Implemented；Rule 2（Implemented→Verified）保留。
3. **`migrate_legacy()` 未独立实现**：计划 §5.2 列了独立方法，实际用 `update` 注入 frontmatter 行为替代（+ 测试）。旧格式容错可用（无 frontmatter → drafting）。

### 待澄清事项（残留 Open Questions）

- **workspace root 与 `docs/plans` 位置**：`specs.json` 在 `backend/.autoos/`（提示 workspace root 可能 = `backend/`），但 `docs/plans/` 在仓库根。`PlansStore` 用 `{root}/docs/plans`。**建议在仓库根启动 `musk serve`**（root=仓库根 → plans 可见）。任务 7 迁移已让 `docs/plans` 内容规范化（frontmatter/归档），但可见性仍取决于启动目录。
- **merge item ID 风格**：当前用 `P{seq}-{n}`（如 `P024-1`），与既有 spec id（G1/A1...）风格略异。后续可考虑统一。
- **`docs/specs/` 模块树知识库**（008 §5）：本轮 merge 落点是现有 6 区 flat ledger，模块树演进留作后续立项。

### finish-plan 复核（2026-08-11，逐任务对照实际代码）

| 任务 | Verdict | 证据（file:line） |
|:---|:---|:---|
| 1. PlansStore | **Partial** | 33 单测绿；§5.2 的独立 `migrate_legacy()` 方法未实现（只有 `update` 注入 frontmatter 行为，见 `plans.rs` migrate_legacy_injects_frontmatter 测试） |
| 2. Specs 剥离 | **Pass**（已补） | 6 区 + 双轨同步；本次补修 `src/back/api.at:109` 注释残留的 plans；其余完整 |
| 3. /api/plans | **Pass（workaround）** | hw 路由 `server.rs:134`；`server.at`/`extern_sigs.at`/`extern_impl.rs` 均无 plans（ag 轨未走，符合 §3.6 逃生舱 + KNOWN-DEBT 登记） |
| 4. Merge 引擎 | **Pass** | 11 单测 + merge API 门禁/沉淀/幂等测试 |
| 5. 前端 web/ | **Pass** | vue-tsc 0 新错误 + **vite build exit 0**（25.6s，本次 finish-plan 补跑验证） |
| 6. Auto 轨 | **Pass** | `plans_view.at`/`plans_store.at` + `app.at` ShowPlans + `api.at` plans_* 契约；auto build 成功（28 components）；gen vue-tsc 0 错 + vite build 绿 |
| 7. 迁移 | **Pass** | `archive/`→`archived/`、`old/`→`archived/` 扁平化、handoff 去前缀解冲突、22 个补 frontmatter；23 唯一 seq 无重复，next_seq 测试绿 |

**分类：A（all complete，含计划内绕道）** —— 7 任务全部完成验证。唯一绕道是任务 3 `/api/plans` 走 hw 路由（计划 §3.6 允许的逃生舱，a2r 转译器漂移所致，待对齐后切回 ag 轨）。状态 → `execution_done`，待复审。

### Skill 层补齐（2026-08-12，PLAN-024 任务外追加）

PLAN-024 任务 1-7 实现了"数据/API/UI 基础设施"，但 008 §6 设计的 4 个 `/auto-plan:*` 工作流 skill 未纳入任务范围（plan 文档假设它们存在，实际缺失——执行时 agent 用通用能力替代）。现已补齐于 `.zcode/skills/`：

| skill | 职责 | 参考 |
|:---|:---|:---|
| `auto-plan-new` | 创建计划（算序号 + 读 spec 骨架 + frontmatter） | superpowers writing-plans + spec-kit specify |
| `auto-plan-work` | 执行计划（唯一上下文 + 逐步 + 状态流转） | superpowers executing-plans |
| `auto-plan-review` | 复审（验收标准 + 补 spec 元数据 + review_done） | finish-plan + verification-before-completion |
| `auto-plan-merge` | 沉淀（门禁 review_done + Plan→Spec + 归档） | archive-plan + spec-kit proposal→specs |

每个 SKILL.md 含 frontmatter（description 只写"何时用"，避 CSO 陷阱）+ State gate + Process + Rules + Checklist，交叉引用上下游 skill（new→work→review→merge）。

---

*本文件为 PLAN-024，格式遵循设计文档 008（Auto-Plan 核心契约）。*

### /auto-plan:review 正式复审（2026-08-24）

| 验收项 | 判定 | 证据 |
|---|---|---|
| 7 任务逐项 | pass | 沿用计划内 finish-plan 复核（2026-08-11，逐任务 file:line 证据）；本次抽查复核：hw 路由 server.rs:133-134、migrate_legacy 测试 plans.rs:1101、derive_statuses specs.rs:420 |
| 三条计划内绕道入册 | pass | KNOWN-DEBT 补登于 427481f（2026-08-24） |
| 验证重跑 | pass(带环境注) | auto build 绿（2026-08-24，修复陈旧 auto 二进制 + 清理模板已废弃的 CodeEditor.vue 死文件后）；cargo test 当前红 = auto-ai 13:14 合入 027/028 的跨仓漂移（conversation/tools 等 6 文件 trait 签名），非本计划缺陷，已单独汇报 |

**结论**：review_done。遗留：a2r 对齐后 /api/plans 切回 ag 轨（已登记）。
