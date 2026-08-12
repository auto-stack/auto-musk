---
# ============ 元数据区块 (Auto-Plan 核心契约) ============

# 基础信息
plan_id: PLAN-025
status: execution_done                        # drafting → executing → execution_done → review_done → merged
feature_name: Spec 文件树浏览器（docs/specs/ 模块树）
author: [zhaopuming + agent]
created_at: 2026-08-12T03:30:00Z
updated_at: 2026-08-12T07:00:00Z

# ============ Spec 合并指引 (/auto-plan:merge 时使用) ============
supersedes_spec_components: []
new_spec_components:
  - "docs/specs/: 新增（008 §5 模块树骨架）"
touched_goals:
  - "goal-spec-knowledge: Spec 知识沉淀层（文件树）"

# ============ 执行进度追踪 (供 /auto-plan:work 更新) ============
current_step: 3
total_steps: 4

---

# [PLAN-025] Spec 文件树浏览器 — 实施计划

> **给执行 Agent 的指令：** 用 `/auto-plan:work` 逐步执行此计划。
> **用户引用方式：** 在对话中输入 "执行 Plan 25" 即可定位本文件。
> **设计文档：** `docs/designs/008-auto-plan.md` §5（Spec 知识库格式设计）。
> **来源：** PLAN-024 §4.3 明确把 `docs/specs/` 模块树排除在本轮（"模块树演进留作后续立项"），本计划是该后续。

## 0. 变更摘要 (Executive Summary)

把 Spec 从"单一扁平 JSON ledger（`.autoos/specs.json` 6 区）"扩展出**第二个落点**：`docs/specs/` 文件模块树（008 §5）。前端 SpecsView 增加"文件树"视图模式（文件夹浏览器式，复用现成 `TreeView.vue`），**与现有 6 区结构化 item 编辑器并存**（顶部 toggle 切换）。

核心洞察：**后端几乎零成本**（wiki 的 `build_tree` / `TreeNode` 现成可复用），**原生前端也便宜**（`TreeView.vue` 现成递归组件），唯一不确定点是 Auto 轨的递归树渲染（`.at` 无递归组件，兜底扁平渲染）。

## 1. 目标 (Goal)

让用户能在 SpecsView 里像浏览文件夹一样浏览 `docs/specs/` 的模块化知识树（展开/折叠目录、点 `.md` 文件看内容），为 008 §5 的"Spec 模块树知识库"铺好前端展示层 + 后端文件树 API；同时**不破坏**现有 6 区 item 编辑功能（gate 审批 / relations / 状态机）。

## 2. 架构方案 (Architecture)

```
┌────────────────────────── auto-musk（Spec 双落点）──────────────────────────┐
│                                                                              │
│  SpecsView（顶部 toggle 切换）                                                │
│  ┌──────────────────────────┐  ┌──────────────────────────────────────────┐ │
│  │ 🗂 结构化编辑（现有，不动）│  │ 📄 文件树（新增）                         │ │
│  │  6 区 item + gate + 状态机 │  │  TreeView（docs/specs/ 树）              │ │
│  │  数据：.autoos/specs.json  │  │  点文件 → MarkdownContent                │ │
│  └──────────────────────────┘  │  数据：docs/specs/ 文件树                 │ │
│                                 └──────────────────────────────────────────┘ │
│                                                                              │
│  Backend:                                                                    │
│    GET /api/specs/tree   → build_tree(docs/specs/) → Vec<TreeNode>（复用 wiki）│
│    GET /api/specs/file/{*path} → 读 docs/specs/{path} 正文                    │
│                                                                              │
│  两个落点职责分离：                                                           │
│    specs.json = 结构化工作台（item 级 CRUD + 状态机 + gate）                  │
│    docs/specs/ = 知识沉淀层（markdown 长文档，/auto-plan:merge 的落点）        │
└──────────────────────────────────────────────────────────────────────────────┘
```

核心要点：
1. **并存不替换**：现有 SpecsView 6 区 item 编辑保留；新增"文件树"模式 toggle。
2. **复用 wiki 的树基建**：`build_tree` / `TreeNode` / `strip_md_extensions`（wiki.rs）改 `pub(crate)`，spec_tree handler 直接用。
3. **docs/specs/ 静态文件树**：不建独立 store，handler 从 workspace root 推导路径（仿 wiki_tree）。
4. **空骨架起步**：specs.json 实质为空（1 个占位 item），docs/specs/ 照 008 §5 模板建空骨架，无需迁移。

## 3. 技术栈 (Tech Stack)

- 后端：Rust + axum（hw 路由，仿 wiki_routes）+ 复用 `wiki::build_tree`
- 前端原生：Vue3 + TypeScript + 现成 `TreeView.vue`（递归组件）+ `MarkdownContent.vue`
- Auto 轨：`.at` 源 → `auto build`（递归风险见 §10）

## 4. 需求分析与背景调查

### 4.1 PLAN-024 §4.3 的排除（本计划的设计依据）

PLAN-024 §4.3 原文（`024-auto-plan-architecture.md:127`）：
> **设计约束：** 起草本 Plan 时已确认：`docs/specs/` 模块树知识库（008 §5）**不在本轮** —— 本轮 merge 的落点是现有 6 区 ledger；模块树演进留作后续。

PLAN-024 §10 Open Questions 也呼应（L390）：「是否后续立项 `docs/specs/` 模块树知识库（008 §5）以取代 flat ledger？」本计划就是这个"后续立项"。

### 4.2 调研结论（2026-08-12 实地核对）

| 维度 | 现状 | 对方案的影响 |
|:---|:---|:---|
| `specs.json` 数据 | **实质为空**（1 个 "test" 占位 item，6/7 区全空） | 迁移工作量 ≈ 0，无需批量脚本 |
| `docs/specs/` | **不存在** | 全新建（照 008 §5 模板） |
| 后端 `build_tree`/`TreeNode`/`wiki_tree` | **现成**（wiki.rs:86-134 build_tree + 72-84 TreeNode + 469-479 handler） | 后端工作量极小：加 2 个端点 |
| 前端 `TreeView.vue` | **现成递归组件**（143 行，吃 `TreeNode[]`，零业务耦合） | 原生 web/ 直接复用 |
| Auto 轨递归树组件 | **不存在**（wiki_nav.at 把树渲染成扁平列表，不展开 children） | Auto 轨改动大（见 §10） |
| 现有 SpecsView | module→type 两级 accordion（虚拟树，从 item 元数据派生）+ category 组件 + gate | 并存：加 toggle，不动现有 |

### 4.3 关键决策（用户默认确认）

- **D1：并存**（不替换现有 SpecsView）。两者数据模型独立：`docs/specs/` = 知识沉淀层；`specs.json` = 结构化工作台。
- **D2：照模板建空骨架**（specs.json 空，无需迁移）。
- **D3：后端复用 wiki build_tree**（pub(crate)，不重复代码）。

> **设计约束：** `/auto-plan:merge` 目前写 `specs.json` 6 区；写 `docs/specs/` 模块文件的深度集成**不在本计划**（留作再后续）。

## 5. 详细设计 (Detailed Design)

### 5.1 后端：docs/specs/ 骨架 + 文件树 API

**新建 `docs/specs/` 骨架**（008 §5）：
```
docs/specs/
├── 00-overview.md        # 项目概览（占位）
├── 01-architecture.md    # 全局架构（占位）
├── goals/
│   └── README.md         # 目标索引（占位）
├── modules/
│   └── .gitkeep          # 空，待填
├── reviews/
│   └── .gitkeep          # 空
└── index.json            # {version:"1.0", updated_at, modules:[], goals:[]}
```

**`wiki.rs`**：`build_tree` / `TreeNode` / `strip_md_extensions` 从私有改 `pub(crate)`（供 spec_tree 复用，不重复代码）。

**新建 `backend/crates/musk/src/spec_tree.rs`**（hw 路由，仿 wiki_routes）：

| 端点 | 方法 | 行为 |
|:---|:---|:---|
| `/api/specs/tree` | GET | `build_tree({root}/docs/specs/, "")` → `Json<Vec<TreeNode>>` |
| `/api/specs/file/{*path}` | GET | 读 `{root}/docs/specs/{path}` 正文（校验路径不越界，仿 `validate_path`） |

`spec_tree_routes() -> Router<AppState>`；在 `server.rs` serve() 的 `.merge()` 链注册（仿 plans_routes）。
**workspace.rs 不加 store**（静态文件树，handler 从 workspace root 推导路径）。

### 5.2 前端原生 web/（复用 TreeView.vue）

**`web/src/composables/useSpecs.ts`** 加：
- `loadSpecTree() -> TreeNode[]`（GET /api/specs/tree）
- `loadSpecFile(path) -> string`（GET /api/specs/file/{path}）

**`web/src/views/SpecsView.vue`**：
- 顶部加 toggle："📄 文件树" / "🗂 结构化编辑"（localStorage 持久化 `autoforge-specs-view-mode`）
- **文件树模式**：侧栏 `<TreeView v-for="node in specTree" :node @select>`（复用 `TreeView.vue` 零改动）；点文件 → `loadSpecFile(path)` → `<MarkdownContent :content>`
- **结构化编辑模式**：现有 6 区 item 编辑（不动）
- 复用 `web/src/components/TreeView.vue` + `MarkdownContent.vue`；`TreeNode` 从 `types/wiki.ts` import（已通用）。

### 5.3 Auto 轨（双前端 parity）

- `src/back/api.at`：加 `specs_tree` / `specs_get_file` 的 `#[api]` 契约
- `src/front/i18n/{en,zh}.json`：加 spec 文件树文案
- `src/front/specs_view.at`：加文件树模式

**⚠️ Auto 轨递归风险（见 §10）**：`.at` 无递归组件。兜底 = 扁平两段式渲染（顶层节点列表 + 点文件夹展开直接子项），对 docs/specs/ 浅层深度够用。

### 5.4 接口变更汇总

| 层 | 变更 |
|:---|:---|
| 后端新增 | `src/spec_tree.rs`（spec_tree_routes + 2 handlers）；`wiki.rs` 3 个 fn 改 pub(crate) |
| 后端 API 新增 | `GET /api/specs/tree` + `GET /api/specs/file/{*path}` |
| 数据新建 | `docs/specs/` 骨架（6 文件/目录 + index.json） |
| 前端新增 | useSpecs 加 loadSpecTree/loadSpecFile；SpecsView 加 toggle + 文件树模式 |
| Auto 轨 | api.at + specs_view.at + i18n |

## 6. 测试设计 (Test Design)

- **后端单测**（`spec_tree.rs` 内嵌 `#[cfg(test)]`）：
  - 树构建：扫描 fixture 目录，返回正确的嵌套 TreeNode（文件夹在前 + 字母序）
  - 路径校验：越界路径（`../`、绝对路径）被拒
  - 文件读取：合法 .md 返回正文；不存在 → 404
- **API 集成测试**（`tests/parity_spec_tree.rs`，仿 parity_plans）：
  - GET /api/specs/tree → 200 + 非空树（含 goals/ 文件夹 + 00-overview.md 文件）
  - GET /api/specs/file/00-overview.md → 200 + 正文
  - GET /api/specs/file/../etc/passwd → 400（越界拒绝）
- **前端**：`vue-tsc && vite build` 全绿（web/）；`auto build` + gen `vue-tsc && vite build` 全绿
- **手测**：SpecsView 切"文件树"，展开 `goals/`，点 `README.md` 看 markdown

## 7. 验收标准 (Acceptance Criteria)

> 复审时 (`/auto-plan:review`) 逐项勾选。

- [x] 标准 1：`cargo test -p musk` 全绿（spec_tree 单测 + parity_spec_tree API + 全 lib 无回归）。[⚠️ 部分] plan-025 相关测试全绿（3 单测 + 5 集成 + 451 lib 无回归）；但 `cargo test -p musk` 因 pre-existing plan-018 遗留债务（`parity_workspace_endpoints` 2 失败，见 §10）未全量绿 —— 非本计划引入。
- [x] 标准 2：`GET /api/specs/tree` 返回 docs/specs/ 的嵌套树；`GET /api/specs/file/{path}` 读正文 + 拒绝越界路径。[✅] `parity_spec_tree.rs` 5 集成测试全绿（含 `%2e%2e` 越界 → 400 + 不泄露 secret）。
- [x] 标准 3：SpecsView 顶部有"文件树/结构化编辑"toggle；文件树模式能展开 `goals/`、点 `README.md` 看 markdown；结构化编辑模式功能不变（6 区 item + gate 正常）。[✅ 代码就位] 原生 web 用 `TreeView.vue` 递归（可展开 goals/）；Auto 轨扁平兜底（顶层节点，文件夹展开留 KNOWN-DEBT）；手测待用户运行时验证。
- [x] 标准 4：原生 `vue-tsc && vite build` 全绿（0 新错误）。[✅] vite build 全绿（23.64s）；vue-tsc 0 新错误（13 pre-existing，`git stash` 对比确认，全在未触碰文件）。
- [x] 标准 5：Auto 轨 `auto build --gen-only` 成功 + gen `vue-tsc && vite build` 全绿（或扁平渲染兜底 + 登记 KNOWN-DEBT）。[✅] auto build 28 components；gen build 全绿（20.56s）；扁平兜底 + KNOWN-DEBT（见 §10）。
- [x] 标准 6：`docs/specs/` 骨架就位（00-overview.md / 01-architecture.md / goals/README.md / modules/ / reviews/ / index.json）。[✅] 6 文件/目录就位。

## 8. 执行步骤 (Execution Tasks)

> **粒度要求：** 每个任务应是 2-5 分钟可完成的原子操作。

### 任务 1: 后端文件树 API（docs/specs/ 骨架 + spec_tree.rs）
- [x] **步骤 1.1:** 新建 `docs/specs/` 骨架（00-overview.md / 01-architecture.md / goals/README.md / modules/.gitkeep / reviews/.gitkeep / index.json）。[✅ 已完成] 6 文件就位；build_tree 自动跳过 `.gitkeep`（dotfile）。
- [x] **步骤 1.2:** `wiki.rs` 的 `build_tree` / `TreeNode` / `strip_md_extensions` 改 `pub(crate)`。[✅ 已完成] `build_tree` + `strip_md_extensions` 改 `pub(crate)`；`TreeNode` 本就 `pub`；`validate_path_pub` / `guess_mime` 复用现成 `pub(crate)`（无需改）。
- [x] **步骤 1.3:** 新建 `backend/crates/musk/src/spec_tree.rs`：`spec_tree_routes()` + 2 handlers（tree/file）+ 路径校验；`lib.rs` 加 `pub mod spec_tree;`。[✅ 已完成] `spec_tree.rs`（`spec_tree_routes` + `spec_tree`/`spec_file` handlers + `SpecTreeQuery` + 3 内嵌单测）；`lib.rs` 注册 `pub mod spec_tree;`（specs 之后）。
- [x] **步骤 1.4:** `server.rs` serve() 的 `.merge()` 链加 `.merge(crate::spec_tree::spec_tree_routes())`。[✅ 已完成] 接在 `plans_routes` 之后、`fallback_service` 之前。
- [x] **步骤 1.5:** 写单测（树构建 + 路径校验）+ `tests/parity_spec_tree.rs` API 测；`cargo test -p musk` 全绿。[✅ 已完成] 3 内嵌单测 + 5 集成测全绿（独立验证）；`cargo test -p musk` 因 pre-existing plan-018 遗留债务（见 §10）未全量绿，plan-025 相关测试无回归。

### 任务 2: 前端原生 web/ SpecsView 文件树模式
- [x] **步骤 2.1:** `web/src/composables/useSpecs.ts` 加 `loadSpecTree` + `loadSpecFile`。[✅ 已完成] 加 `_specTree`/`_activeSpecFile` singleton state + 两个 loader（authFetch；`encodeURIComponent` 编码 path 给 `{*path}`）+ return 导出。
- [x] **步骤 2.2:** `web/src/views/SpecsView.vue` 顶部加 toggle（localStorage 持久化）+ 文件树模式（TreeView + MarkdownContent，复用现成组件）。[✅ 已完成] `view-mode-toggle`（🗂 结构化 / 📄 文件树，localStorage `autoforge-specs-view-mode`）+ `v-if` 包裹现有 body + `v-else` 文件树面板（TreeView 侧栏 + MarkdownContent 内容）；i18n key 加到 `web/src/i18n/locales/{en,zh}.json`（structuredMode/fileTreeMode/emptyTree/selectFile）；CSS 复用 `.specs-body` flex 布局（`.specs-view` 已是 flex column）。
- [x] **步骤 2.3:** 验证 `vue-tsc && vite build`（web/）全绿 + 手测 toggle 切换 + 文件树浏览。[✅ 已完成] `vite build` 全绿（23.64s）；`vue-tsc` **0 新错误**（HEAD baseline 13 个 pre-existing 错误，`git stash` 对比确认，全在未触碰文件：AgentAvatar/CategoryCards/ChatsView/RelayView/WikiView/i18n.spec）；手测待用户运行时验证（本环境无浏览器+后端）。

### 任务 3: 前端 Auto 轨同步
- [x] **步骤 3.1:** `src/back/api.at` 加 `specs_tree` / `specs_get_file` 的 `#[api]` 契约。[✅ 已完成] `specs_tree` 返回 `[]TreeNode`（TreeNode 已在 api.at 定义）；`specs_get_file(path)` 返回 `str`。
- [x] **步骤 3.2:** `src/front/specs_view.at` 加文件树模式（递归或扁平兜底）+ `src/front/i18n/{en,zh}.json` 加文案。[✅ 已完成] 采用**扁平渲染兜底**（plan §10）：`specs_view.at` 加 toggle（`EnterStructuredMode`/`EnterTreeMode`）+ `view_mode` + 扁平文件树面板（顶层节点列表）；`specs_store.at` 加 `spec_tree` state + `LoadSpecTree`/`LoadSpecFile`；i18n 加 4 文案（emoji 入 i18n 值，因 `.at` `text` 不支持字面量拼接）。递归渲染留 KNOWN-DEBT（见 §10）。
- [x] **步骤 3.3:** `auto build --gen-only` + gen `vue-tsc && vite build` 全绿（或扁平兜底 + KNOWN-DEBT）。[✅ 已完成] `auto build --gen-only` 成功（28 components，0 parse error）；gen `npm run build`（vue-tsc + vite）全绿（20.56s，0 错误）。

### 任务 4: 验证 + 提交
- [x] **步骤 4.1:** 全量 `cargo test -p musk` + `vue-tsc/vite build`（双前端）+ 手测。[✅ 已完成] cargo test：plan-025 相关全绿（pre-existing 债务见 §10）；web vite build 全绿（0 新错误）；gen build 全绿（28 components）；手测待用户运行时验证。
- [x] **步骤 4.2:** 更新本计划状态（`execution_done`）+ README（docs/specs/ 说明）。[✅ 已完成] 状态 → execution_done；`docs/specs/00-overview.md` 作为知识库入口说明就位。

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

- **Auto 轨递归渲染**（已用扁平兜底解决）：`.at` 无自引用递归 widget。任务 3 采用**扁平渲染**（顶层节点列表，点文件加载正文，文件夹不展开）。
  - ✅ 已实现：`specs_view.at` toggle + 扁平文件树面板（顶层节点 + 点文件看内容）。
  - **KNOWN-DEBT**：文件夹嵌套浏览（展开 `goals/` 等子目录）待 `.at` 增强递归 widget、或加 per-folder 展开状态后补齐。docs/specs/ 初期浅层（顶层 2 .md 可直接点开），对当前知识量够用；原生 web/ 前端用 `TreeView.vue` 递归组件，无此限制。
- **merge 写 docs/specs/ 的深度集成**：`/auto-plan:merge` 目前写 `specs.json` 6 区；后续可扩展直接写 `docs/specs/modules/<mod>/{spec,design,tests}.md`。留作再后续。
- **008 §5 路径表述**：008 设计写的是 `specs/`（项目根），本计划用 `docs/specs/`（与 docs/plans/、docs/designs/ 一致，进版本控制）。后续可回头更新 008 文档的路径表述。
- **plan-018 遗留测试债务（阻塞验收标准 1「`cargo test -p musk` 全绿」）**：执行中发现 HEAD 上 `cargo test -p musk` 本就跑不通 —— plan-018 把 `plans` section 从 6 区 spec 移除时，多个测试文件未同步：
  - ✅ `parity_specs.rs:27` `SectionType::Plans` dead case（编译错误）— 已删
  - ✅ `parity_specs.rs:138` `parity_derive_goal_implemented_when_plan_done`（断言 specs.rs:443-445 明确声明已移除的 derive 规则）— 已删
  - ❌ `parity_workspace_endpoints.rs:98/149/168` 往 `plans` section upsert（期望 200，实际 400）— 未修，同根因
  - 可能还有更多同根因过时测试。plan-025 新增代码（`spec_tree` 3 单测 + `parity_spec_tree` 5 集成）已**独立全绿**。待用户决策：继续清理 vs 降级验收标准 1 + 立独立债务任务。

---

*本文件为 PLAN-025，格式遵循设计文档 008（Auto-Plan 核心契约）。来源：PLAN-024 §4.3 后续演进。*
