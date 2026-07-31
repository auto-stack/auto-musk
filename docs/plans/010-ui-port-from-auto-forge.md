# 010 — UI 重构：移植 auto-forge 前端（Chats/Specs/Wikis/Flows）

> **状态**：实施计划。基于 2026-06-26 对 auto-forge 前端的逐文件调研（附录 A）。
> **决策**：放弃 auto-musk 自写的简陋前端（web/，8 文件），直接移植 auto-forge 经过多轮打磨的前端。auto-musk 本质是 auto-forge 拆成 UI+client+daemon，前端界面应高度一致。
> **范围**：Chats / Specs / Wikis / Relays（→Flows）四个模块。不移植 Harness 配置（ApiSources/Agents/Professions，那些走 auto-os-config/daemon）。

---

## 0. 背景与关键事实

### 为什么移植而非重写
auto-forge 前端经过多轮调整，4 模块合计 ~6564 行视图 + ~1711 行 composable + ~3000 行组件。auto-musk 自写的 web/ 只有 8 文件，风格功能都差很多。重写既浪费又难达同等质量。

### auto-forge 前端技术栈（移植前提）
- Vue 3.5 `<script setup>` + TypeScript + **Vite 6**
- **无 vue-router**（自研 useViewState，手写 History API）
- **无 pinia**（composable 单例 ref 模式）
- **无 tailwind/shadcn**（纯 CSS 变量主题 theme.css）
- vue-i18n 9（en/zh）
- markstream-vue（流式 markdown）+ marked + mermaid
- TipTap 3（Wiki 的富文本编辑器，重型）
- lucide-vue-next（图标）

**关键**：架构轻量，移植主要搬 composables，不需引入新框架。

### 最大挑战：后端 API 缺口（调研结论）
| 模块 | 后端就绪度 | 主要缺口 |
|---|---|---|
| **Specs** | 🟡 接近 | 数据模型对齐，但端点形状差异（按 project vs 扁平；缺 /related /rebuild-relations 端点）|
| **Chats** | 🟡 部分 | 路径错位（/api/forge/chats vs /api/chats）；SSE 事件丰富度（前端 22 种 vs 后端 4 种）；approve/reject 协议 |
| **Wikis** | 🔴 全缺 | 后端无任何 /api/wiki /api/raw 路由 |
| **Flows** | 🔴 几乎全缺 | 后端只有简单 workflow，前端的 run 持久化/gate/handoff/SSE 事件协议全缺 |

---

## 1. 总体策略

```
Phase 0  打 tag 保存现状（v0.2-pre-rewrite）
   │
Phase 1  基础设施层移植（theme/i18n/types/useAuth/useViewState/共享组件/App shell）
   │     ← 纯前端搬代码，后端基本不动
   ├─→ Phase 2  Specs（后端最接近，见效快）
   │
   ├─→ Phase 3  Chats（后端有基础，SSE 事件适配）
   │
   ├─→ Phase 4  Wikis（前端就绪，需新建后端 wiki 模块）
   │
   └─→ Phase 5  Flows（最后，后端 relay 运行时工作量最大）
```

**原则**：
1. 每个 Phase 独立交付、可验证、可 commit。
2. 前端移植（搬代码）与后端补缺并行——前端先搬、能编译，后端逐步补端点。
3. 端点路径统一：前端 composable 的 `API_BASE` 统一改成 auto-musk 的 `/api/...`（去掉 `/forge` 前缀）。
4. SSE 事件协议：后端逐步向前端期望的事件类型靠拢（Chats 先补 turn_start/tool_result）。

---

## 2. Phase 0 — 保存现状

- [ ] `git tag v0.2-pre-rewrite`（标记自写前端最后状态）
- [ ] 把当前 `web/` 重命名为 `web-legacy/`（保留参考，不删除）
- [ ] 新建 `web/`（移植目标，从 auto-forge frontend 复制结构）

---

## 3. Phase 1 — 基础设施层（纯前端搬代码）

**目标**：移植 auto-forge 前端骨架，能编译、能显示 App shell（空 4 tab），登录可用。

### Tasks
- [ ] 复制 `frontend/package.json` → `web/package.json`，调整 name 为 musk-web，保留依赖（vue/vite/vue-i18n/markstream-vue/marked/mermaid/lucide/tiptap）
- [ ] 复制 `frontend/vite.config.ts`，改 base path（auto-forge 是 `/forge/`，musk 用 `/`）+ dev port 3333 + proxy → :8888
- [ ] 复制 `frontend/index.html`、`frontend/tsconfig*.json`
- [ ] 复制 `src/styles/theme.css`（109 行 CSS 变量主题）
- [ ] 复制 `src/i18n/`（index.ts + locales/{en,zh}.json），改 brandName 等键
- [ ] 复制 `src/types/`（forge.ts/specs.ts/wiki.ts/tool.ts）
- [ ] 移植 `src/composables/useAuth.ts`：融合 auto-musk 现有 auth（JWT key `autoforge_jwt` → `musk_jwt`；fetch monkey-patch 改 key；登录端点 `/api/auth/login` 对齐）
- [ ] 移植 `src/composables/useViewState.ts`：base path `/forge/` → `/`
- [ ] 移植共享组件：StatusBadge / AgentAvatar / MarkdownContent / TreeView / DropZone / StreamingRenderer / StreamingTable
- [ ] 改写 `src/App.vue`：保留 4 tab（Chats/Specs/Wikis/Flows）+ Login，去掉 agents-config/professions/skills/apis/explorer
- [ ] 复制 `src/main.ts`（i18n + fetch 拦截 + mermaid init）
- [ ] `npm install && npm run build` 验证编译通过（视图先放占位）

### 验收
- web/ 能编译、dev server 起来
- App shell 显示 4 tab 导航 + 登录页
- 登录走通 auto-musk 后端 `/api/auth/login`

---

## 4. Phase 2 — Specs（后端最接近）

**目标**：移植 SpecsView 全套，接通 auto-musk specs 后端。

### 前端（搬代码）
- [ ] `composables/useSpecs.ts` + `useItemRelations.ts`
- [ ] `views/SpecsView.vue`（1396 行）
- [ ] `components/category/`（CategoryList + 7 个 *Cards.vue + GoalsTable）
- [ ] `components/detail/`（GoalDetail/PlanDetail/ApiDetail/ReportDetail/ReviewDetail/TestDetail）
- [ ] `components/GoalDetailModal.vue`、`GateBanner.vue`、`RelationsPanel.vue`
- [ ] `utils/itemTemplates.ts`、`categorySummary.ts`、`goalParser.ts`

### 后端适配（auto-musk server.rs + specs.rs）
- [ ] useSpecs 的 API_BASE 从 `/api/forge/specs/{project}` 改为 `/api/specs`（去掉 project 命名空间，musk 单项目）
- [ ] 补端点 `GET /api/specs/related/{item_id}`（返回某 item 的 related，后端 rebuild_relations 已有逻辑，加 HTTP 暴露）
- [ ] 补端点 `POST /api/specs/rebuild-relations`（手动触发重建）
- [ ] 整 section 保存（`PUT /api/specs/section/{id}` body=content）适配或前端改用 item 级 upsert
- [ ] 确认 SpecItem 类型字段对齐（created_at/modified_at/completed_at/tags）

### 验收
- Specs tab 显示 7 类 section，卡片/详情/状态切换/关系面板全可用
- 状态切换经后端 per-section 状态机校验
- overview / drift-check 端点工作

---

## 5. Phase 3 — Chats（后端有基础，SSE 适配）

**目标**：移植 ChatsView 全套，接通 chats 后端 + SSE 流式。

### 前端（搬代码）
- [ ] `composables/useForge.ts`（519 行，核心）+ `useSessions.ts` + `useEventRouter.ts` + `useGateInbox.ts`
- [ ] `views/ChatsView.vue`（2704 行）
- [ ] 组件：StreamingRenderer / StreamingTable / MentionDropdown / GateCard / SecretaryMessage / ReportCard / QuestionnaireCard
- [ ] useProject 简化（musk 单项目，去掉"打开项目"模型或硬编码默认项目）

### 后端适配（SSE 事件 + approve/reject）
- [ ] useForge API_BASE `/api/forge/chats` → `/api/chats`
- [ ] approve/reject 协议对齐（前端 body 传 edited_specs vs 后端按 index；二选一统一）
- [ ] **SSE 事件丰富度**：后端 chat_stream 当前只发 delta/tool/done/error，前端 useForge 期望 22 种（turn_start/delta/thinking/tool_call/tool_result/errand_*/relay_*/phase_change...）。策略：前端容忍（多余分支不触发），后端逐步补发关键事件（turn_start/tool_result/thinking）
- [ ] 补端点 `GET /api/chats/session/{id}/history`（前端用，后端可返回完整 session）

### 验收
- Chats tab 新建会话、多轮对话、SSE 流式逐 token 显示
- 工具调用内联展示
- spec 审批 gate（approve/reject）在 chat 流中工作

---

## 6. Phase 4 — Wikis（前端就绪，需新建后端）

**目标**：移植 WikiView 全套 + 新建 wiki 后端模块。

### 前端（搬代码）
- [ ] `composables/useWiki.ts`（231 行）
- [ ] `views/WikiView.vue`（865 行）
- [ ] `components/TreeView.vue`、`DropZone.vue`、`MarkdownContent.vue`（已在 Phase 1）
- [ ] `components/editors/autodown/`（TipTap 富文本子树，重型）

### 后端新建（wiki 模块）
- [ ] 新建 `backend/crates/musk/src/wiki.rs`：WikiStore（pages 持久化，JSON 或文件）
- [ ] 端点 `/api/wiki/pages`（CRUD：list/get/create/update/delete/search/tree）
- [ ] 端点 `/api/raw`（文件树：tree/file/upload/mkdir/delete）
- [ ] 注册路由到 server.rs

### 验收
- Wiki tab 显示页面树、新建/编辑/删除页面
- markdown 渲染（MarkdownContent）
- 文件上传（DropZone）
- 搜索

---

## 7. Phase 5 — Flows（最后，工作量最大）

**目标**：移植 RelayView 全套 + 新建 relay 后端运行时。

### 前端（搬代码）
- [ ] `composables/useRelay.ts`（595 行）+ `useTaskPlan.ts`（192 行）
- [ ] `views/RelayView.vue`（1599 行，重命名为 FlowsView）
- [ ] 组件：GatePanel / SegmentedProgressBar / TaskPlanPanel
- [ ] 去掉 useRelay 里的 loadProfessions/loadSouls（Harness，不移植）

### 后端新建（relay 运行时 = Plan 009 P2b）
- [ ] 这是 Plan 009 P2b（Relay 编排引擎）的前端对应。后端需新建完整 relay 运行时（pipeline/handoff/gate/checkpoint/budget + run 持久化 + SSE 事件协议）。
- [ ] **建议**：Flows 前端移植等 Plan 009 P2b 后端就绪后再做（前端期望的事件协议与后端 workflow 差距巨大，硬移植会卡住）。

### 验收
- Flows tab 显示 run 列表、启动 run、gate 审批、handoff 传递
- SSE 事件流（turn_delta/tool_call/gate_waiting）实时更新

---

## 8. 不移植（明确排除）

| 项 | 理由 |
|---|---|
| ApiSourcesView / useApiSources | Harness，走 daemon |
| ProfessionsView / useProfessions | Harness，走 auto-os-config |
| SkillsView / useSkills | Harness |
| AgentsConfigView / useAgentConfigs | Harness |
| ExplorerView / useProject（项目模型） | auto-musk 单项目，简化掉 |
| WelcomeView | auto-musk 直接进登录 |
| useRelay 的 loadProfessions/loadSouls | Harness |

---

## 9. 风险与注意

1. **markstream-vue 是 beta 包**（0.0.14-beta.8），可能版本漂移。Phase 1 时确认能装、能 build。
2. **TipTap 子树大**（Wiki 的 AutoDownEditor），Phase 4 工作量集中在它。
3. **SSE 事件协议**是 Chats/Flows 最大的适配点——前端期望的事件类型多，后端要逐步补。
4. **端点路径全局替换**：auto-forge 全用 `/api/forge/*`，移植时 composable 里要批量改 API_BASE。建议保留 `/api/forge` 前缀？或改 musk 风格。**决策**：去掉 `/forge`，用 `/api/chats` `/api/specs` 等（musk 风格），composable 改 API_BASE。
5. **i18n 键**：brandName 等要改，但大部分（specs.*/relay.*/wiki.*）可直接用。

---

## 附录 A：调研数据（本计划依据）

### 4 模块代码量
| 模块 | View 行数 | Composable 行数 | 组件数 |
|---|---|---|---|
| Chats | 2704 | 519+103+91+94 | 9 |
| Relays/Flows | 1599 | 595+192+39+55 | 5 |
| Specs | 1396 | 130+44 | 15+（category+detail）|
| Wikis | 865 | 231 | 4（+TipTap 子树）|
| 共享 infra | App 304 | useAuth 191 + useViewState 142 + useProject 110 | theme 109 + i18n 750 |

### API 缺口（auto-forge 期望 vs auto-musk 提供）
- Specs：🟡 端点形状差异（按 project vs 扁平；缺 /related /rebuild-relations 端点）
- Chats：🟡 路径错位 + SSE 事件 22 vs 4 + approve/reject 协议
- Wikis：🔴 后端全缺（无 /api/wiki /api/raw）
- Flows：🔴 后端几乎全缺（只有简单 workflow，relay 运行时全无）

### 移植顺序依据
后端就绪度 × 前端依赖链：基础设施 → Specs（后端最近）→ Chats（后端有基础）→ Wikis（前端就绪+新建后端）→ Flows（后端最大缺口，等 P2b）。
