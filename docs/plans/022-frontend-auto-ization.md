# 022 — auto-musk 前端 Auto 化（web/ SPA → AutoUI）

> **状态**：🟡 进行中（2026-08-07 启动）。
> **前置**：Plan 020/021（后端业务端点 100% ag + 严格 parity 闭环，方法论来源）；auto-lang 的 a2vue codegen（examples/ui/015-notes 基准）。
> **仓库**：auto-musk（`web/` + 新建 `src/front/` + `src/back/api.at`）+ auto-lang（codegen 三处扩展：SSE 多事件 / i18n / markdown-mermaid tag）。
> **目标**：为 `web/` 主 SPA（~1.85 万行 Vue3+TS）创建 AutoUI `.at` 源，使 `auto build` 生成的 vue 工程在**功能/交互/外观**上与原生 `web/` 达到**行为+视觉一致**（对标后端 parity 标准）。一致性口径选定**纯 AutoUI 原生表达**——遇 codegen 缺口优先扩展生成器。

---

## 0. 实施日志

| 阶段 | 内容 | 提交 | 验收 |
|---|---|---|---|
| **Phase 0** | 工具链就绪 + 桥接演示：清理 gen 孤儿；建 `src/front/app.at` 最小计数器 widget；`auto build --gen-only` 生成 vue 工程通过；`pnpm install + build` 产出 dist（index.js 82.78KB）；web/ 基线 `vite build` 通过 | `7ced2b9` | ✅ 生成工程可构建；⚠️ 发现 web/ 既有 `vue-tsc` 类型错误（见 §9） |
| **Phase 1** | 生成器扩展：SSE 多事件 discriminator。`StreamEndpoint` 加 `discriminator`+`variants`；`resolve_stream_variants` 解析 `#[serde(tag="type",rename_all="snake_case")] pub tag`（嵌套 braces 配平）；vue.rs 数据驱动 if-chain + 多端点 per-path guard；legacy fallback 保留；26 单测绿；auto-musk forge `SseEventDto`(6变体)端到端验证生成 `data.type` dispatch 正确 | `auto-lang 45361b10` | ✅ cargo test 全绿；demo 6 变体全捕获 |
| **Phase 2** | 生成器扩展：i18n 工程级原生支持。`I18nConfig`+`parse_i18n`(true/单路径/数组)；`VueProject.i18n`+`copy_locale_files`；`generate_package_json` 加 vue-i18n；`generate_main_ts` 注入 createI18n+import locales+app.use(i18n)（重构 router 分支为统一 app 构造）；18 vue 单测绿（含 7 新增 i18n）；端到端验证 pac.at `i18n:[en,zh]` 生成正确（package.json/src/locales/main.ts）。widget 级文本 `t('key')` 替换留待 Phase 4 按需设计 | `auto-lang f0b9735e` | ✅ 工程级 i18n 注入验证通过 |
| **Phase 3** | 生成器扩展：markdown/mermaid 内置 tag + a2vue golden 测试基建。`BackendMapping.npm_package` 字段；`register_rich_text_widgets`（Markdown→MarkdownStream/markstream-vue，Mermaid→mermaid）；`detect_npm_packages_from_code` 扫生成代码 import 自动补 package.json 依赖；端到端验证 `markdown {}` 生成 MarkdownStream + 自动加 markstream-vue。**a2vue golden 基建**：`test_a2vue` helper（仿 test_a2ark）+ 001_counter case golden 基线，填补 vue codegen 无 golden 测试的长期缺口。4 npm 检测 + 1 a2vue golden 测试绿 | `auto-lang d14c3283` | ✅ markdown tag 端到端；golden 机制可用 |

> **里程碑 M1–M3 达成（2026-08-07）**：三个 codegen 缺口（SSE 多事件 / i18n / markdown-mermaid）全部扩展完成 + 单测/golden 覆盖。auto-lang 三个工作分支（sse-multi-event 45361b10 / i18n-support f0b9735e / markdown-mermaid-tag d14c3283）可合并。下一步进入纯 auto-musk 的 .at 编写阶段（Phase 4-7）。
| **Phase 4** | .at 骨架 + LoginView 生成构建闭环。`src/front/{app,auth_store,login}.at` + `src/back/api.at`(auth 端点+AuthUser/AuthResponse 类型)；app.at auth guard(v-if authenticated)；auth_store 单例(login/register/me/logout + localStorage)；LoginPage 表单(username/password + 登录/注册切换)；纯 store 驱动(无 emit callback,比原生简洁)。`vue-tsc && vite build` 全绿产出 dist。**修复 2 个 codegen bug**(auto-lang 4fc26bbe)：SSE 全局接线按 api_imports 过滤(AuthStore 不再误接 chat_stream) + store 自调用语法(bare Me()) | `auto-lang 4fc26bbe` | ✅ 生成工程 vue-tsc+vite build 通过 |
| **Phase 5a** | SpecsView 骨架 + specs 数据层。`specs_store.at`（loadDocument/loadOverview/saveItem/deleteItem/rebuildRelations）；`src/back/api.at` 加 specs 端点 + SpecItem/SpecsSection/SpecsDocument 类型；`specs_view.at`（section-nav 7 类导航 + overview + item 列表）；app.at 加 view rail（chats/specs/wiki 切换）；`setup_auth_fetch.ts`（fetch 拦截器逃生舱，注入 musk_jwt + workspace）。vue-tsc + vite build 全绿。**解决多个语法/codegen 问题**：`view` 保留字不能做字段名（改 current_view）、handler 多语句需分号、SSE 过滤（Phase 4 修复）+ store 自调用（bare Me()） | `1c410e1` | ✅ 生成工程构建通过；3 components |
| **Phase 5b** | SpecsView CRUD + 编辑面板。`use { fn }` 引入 itemTemplates 工具（getNextId/getDefaultStatus，逃生舱）；AddItem/StartEditItem/SaveEditItem/DeleteItem/CancelEdit msg + handler；编辑面板（title/status/content input + textarea）；v-for `key:` 修复（U24）。src/front/utils/（itemTemplates + categorySummary 复制，去 @/types 依赖）。vue-tsc + vite build 全绿。剩余：搜索过滤、module accordion、7 类 category 卡片细化（留后续） | _待提交_ | ✅ CRUD + 编辑面板构建通过 |
| **Phase 6** | WikiView | _待填_ | _待填_ |
| **Phase 7** | ChatsView（最难）+ 全量 parity 闭环 + 文档归档 | _待填_ | _待填_ |

---

## 1. 为什么需要本计划

后端已于 Plan 020/021 完成 Auto 化（业务端点 100% ag handler + 严格 parity）。前端仍是原生 Vue3（`web/`，移植自 auto-forge，Plan 010），未经 Auto 化。本计划把前端也纳入"Auto 为单一真源"的体系，使 auto-musk 全栈（后端逻辑 + 前端 UI）都能由 `.at` 源复现。

历史背景：早期 Plan 002 曾走 a2vue 路线（产物在 `gen/front/vue/`），后被 Plan 010"直接移植 vue"路线取代，源 `.at` 已在 `7c14be6`（2026-08-06）清理。本计划是 a2vue 路线的**重启**，目标更大（覆盖完整 web/ 而非简陋骨架），且要求生成器扩展到位以支撑纯原生表达。

## 2. 调研基线（2026-08-07 实测）

**目标前端**：`web/`（musk-web SPA）。有效代码 ~1.85 万行（剔除 RelayView 死代码 1599 行）。
- 53 vue 组件 + 19 composable + 4 类型文件 + 3 utils
- 视图：ChatsView(2767) / SpecsView(1396) / WikiView(865) / LoginView(206)；RelayView(1599) 死代码跳过
- 无 vue-router（`useViewState` 单例 + `v-if` + Ctrl+1/2/3）
- 后端托管：`server.rs` ServeDir 同源托管 `web/dist` + SPA fallback；产物必须落 `web/dist/index.html`

**工具链**：`auto build` / `auto run` / `auto watch`；输出 `gen/front/vue/`；源约定 `pac.at` + `src/front/*.at` + `src/back/*.at`。`auto.exe` 在 `D:\autostack\auto-lang\target\debug\auto.exe`。

**3 个 codegen 缺口（纯原生表达必须扩展）**：

| 缺口 | 现状 | 扩展落点 | 难度 |
|---|---|---|---|
| SSE discriminator | 硬编码 `command_output`/`command_result`（vue.rs:9845-9860）；forge 流 20+ 事件 | `aura/types.rs:389 StreamEndpoint` 加 `variants` + `ui_gen/api.rs:109` 解析 T + `vue.rs:9845` 生成 if-chain | 中 |
| i18n | 无任何支持（324 key zh/en） | `auto-man/vue.rs:415` main.ts + `:156` package.json + `ui_gen/vue.rs:3558` text 节点 + `:1648` script | 中 |
| markdown/mermaid tag | 无内置 tag | `ui_gen/widget/registry.rs:857` 注册 + `widget/spec.rs:23 BackendMapping.npm_package` | 小(注册)/中(npm携带) |

**逃生舱可用**：`use { fn/component/composable }`（生成器合同 C2 保障）。本计划原则：优先扩展生成器；极高成本特性可走逃生舱 + 登记 KNOWN-DEBT。

## 3. 目标与验收标准

1. **行为+视觉一致**：生成的 vue 工程 `pnpm build` 后，各视图功能/交互/外观与原生 `web/` 逐项对齐。
2. **纯 AutoUI 原生表达**：3 个 codegen 缺口经生成器扩展后用 `.at` 表达；逃生舱仅用于极少数记录在 KNOWN-DEBT 的特性。
3. **零 drift**：重新 `auto build` 后 diff 产物 = 0。
4. **无回归**：auto-lang `cargo test -p auto-lang` 全绿（含新增单测 + a2vue golden）。

## 4. 实施阶段

### Phase 0（M0）：工具链就绪 + 桥接演示
- **0.1** 确认根 `pac.at`（已 `scene:ui`/`render:vue`/`api:rust`）；清理退役 `gen/front/vue/` 孤儿。
- **0.2** 建 `src/front/`，放最小 `app.at`（参照 examples/ui/001），`auto build` 跑通 → `gen/front/vue/` 可 `pnpm build`。
- **0.3** 建立 parity 比对基线：`cd web && npm run build` 产出当前 `web/dist`，存档基准快照（截图 + 关键 DOM 断言）。
- **验收**：根目录 `auto build` 成功生成可构建 vue 工程；web/dist 基线存档。

### Phase 1（M1）：生成器扩展——SSE 多事件 discriminator
- **1.1** `aura/types.rs`：`StreamEndpoint` 加 `variants: Vec<(String,String)>`。
- **1.2** `ui_gen/api.rs:109`：抓 `~Stream<T>` 的 T 后解析其 union/enum 定义，snake→Pascal 填 variants。
- **1.3** `ui_gen/vue.rs:9845`：硬编码两行 → 遍历 `ep.variants` 拼 if-chain；评估多端点（`stream_ep.first()` → 遍历）。
- **1.4** 更新单测 `test_store_composable_wires_sse_stream`（13770）/`test_store_composable_sse_type_driven`（13853）；新增多 variant + 多端点用例。
- **1.5** 最小 demo（`~Stream<FooEvent>` + 3 变体）验证生成。
- **验收**：`cargo test -p auto-lang` 绿；demo store onmessage 含全部变体。
- **降级**：若解析 T 太复杂，改用 `#[stream(events="a,b,c")]` 手写标注。

### Phase 2（M2）：生成器扩展——i18n 原生支持
- **2.1** `pac.at` 加 `i18n` 字段；`auto-man/vue.rs` 新增 `parse_i18n_flag`（仿 `parse_npm_deps:824`）。
- **2.2** `generate_package_json`（156）：i18n 开启加 `vue-i18n`。
- **2.3** `generate_main_ts`（415）：加参数，注入 `createI18n({messages}) + app.use(i18n)`；新增 `generate_i18n_index_ts`。
- **2.4** `ui_gen/vue.rs:1648 generate_script`：注入 `const { t } = useI18n()`；`vue.rs:3558` text 节点识别 `t('key')`。
- **2.5** 单测：package.json/main.ts/text 节点。
- **验收**：`cargo test -p auto-lang` 绿；demo 切换 zh/en 正确。
- **降级**：若 text t() 复杂，先做 main.ts/package.json 注入，widget 文本用 `use{fn:t}` 兜底。

### Phase 3（M3）：生成器扩展——markdown/mermaid tag + a2vue golden
- **3.1** `widget/spec.rs:23 BackendMapping` 加 `npm_package: Option<(String,String)>`。
- **3.2** `widget/registry.rs` 新增 `register_rich_text_widgets`：`markdown`→MarkdownStream(markstream-vue)、`mermaid`/`markdown_static`。
- **3.3** `VueProject` 收集已用 builtin tag 的 npm 依赖合并进 `extra_deps`。
- **3.4** props 映射：markstream `content`/`stream`/`typewriter`/`batch-rendering` 透传。
- **3.5** **a2vue golden 基建**（高价值副产物）：新建 `crates/auto-lang/test/a2vue/`，仿 a2ark `test_a2ark`（input.at → 生成 → 对比 input.expected.vue）。Phase 1/2 扩展顺带补 golden。
- **验收**：`cargo test -p auto-lang` 绿；demo `markdown { content: .text }` 渲染；golden 机制可用。
- **降级**：mermaid 手写 DOM 后处理若无法 tag 化，走 `use{component:MarkdownContent}` + KNOWN-DEBT。

> **里程碑 M1–M3 检查点**：三缺口扩展完成 + 单测/golden 覆盖。建议暂停合并 auto-lang 改动，再进纯 auto-musk .at 阶段。

### Phase 4（M4）：.at 骨架 + LoginView 最小 parity 闭环
- **4.1** `src/front/app.at`（auth guard + view 切换壳，对应 App.vue）+ `types.at`。
- **4.2** `src/back/api.at`：声明 web/ 所有 `#[api]` 端点签名 + `pub type`（从 `web/src/types/*.ts` 1:1 映射）。仅服务前端 codegen，不重复生成后端。
- **4.3** `src/front/auth_store.at`（对应 useAuth.ts：token/user/authFetch/login/logout）。
- **4.4** `src/front/pages/login.at`（对应 LoginView.vue 206 行表单）。
- **4.5** **托管对接**：生成工程 vite.config.ts（base/proxy/alias）、main.ts（fetch 拦截器 musk_jwt + i18n）、index.html；确保产物落 `web/dist`。评估生成器是否需支持自定义 vite/main.ts 注入（若需，记 4.6 小扩展）。
- **4.6** parity 比对 LoginView（截图 + 登录流程）。
- **验收**：LoginView 行为+视觉一致；登录流程通；auth_store token 管理正确。

### Phase 5（M5）：store 全量 + SpecsView
- **5.1** composable→store 映射表：15 单例→`store`；4 工厂（useStreamingDocument/useProfessionSegments/useItemRelations/useWorkspaceId）→ `view fn`/widget computed。store 间互调限制（无 store-level use）：合并强耦合（useForge+useEventRouter）或 widget `on` 协调。store 级 watch 上移 widget。
- **5.2** 核心 store：`forge_store.at`（最大）、`specs_store.at`、`wiki_store.at`、`view_state_store.at`、其余轻量。
- **5.3** `src/front/pages/specs.at`（1396 行）+ 子组件：7 类 category 卡片 + detail + GoalDetailModal + StatusBadge/GateBanner。
- **5.4** 富文本：SpecsView MarkdownContent（markdown+mermaid）用 Phase 3 内置 tag。
- **5.5** parity 比对 SpecsView（CRUD 表单、7 section、状态徽章、drift-check）。
- **验收**：SpecsView 一致；store 单例共享；CRUD 全通。
- **降级**：复杂表单（GoalDetailModal 334 行）若成本过高走 `use{component}` + KNOWN-DEBT。

### Phase 6（M6）：WikiView
- **6.1** `src/front/pages/wiki.at` + 子组件（TreeView/DropZone/AutoDownEditor-stub）。
- **6.2** 拖拽上传：AutoUI DOM 事件（`ondragover`/`ondrop`）。
- **6.3** 文件树（TreeView 142 行）：递归 `for` + 折叠态。
- **6.4** AutoDownEditor stub（当前是桩）。
- **6.5** parity 比对 WikiView。
- **验收**：WikiView 一致；wiki CRUD + 拖拽通。

### Phase 7（M7）：ChatsView（最难）+ 全量闭环
- **7.1** forge 流消费：Phase 1 SSE 多事件，forge_store `on` 处理 20+ 事件（turn_start/delta/thinking/tool_call/tool_result/agent_handoff/phase_change/done/errand_*/relay_*/task_plan_spawned）。
- **7.2** 流式 markdown：Phase 3 markdown tag + StreamingRenderer 逻辑。
- **7.3** `src/front/pages/chats.at` + 子组件：MentionDropdown（defineExpose+键盘导航）、AgentAvatar、GateCard、SecretaryMessage、ReportCard、QuestionnaireCard、RelayRunBox、StreamingRenderer。
- **7.4** 复杂输入框：@mention + `/relay`/`/spawn` 命令解析、v-html mention 高亮。
- **7.5** useStreamingDocument 增量 JSON：评估 store 表达 or `use{fn}` 逃生舱（高概率逃生舱 + KNOWN-DEBT）。
- **7.6** **全量 parity 闭环**：逐视图比对（截图+DOM+交互）；零 drift；端到端冒烟（参照 015-notes acceptance.atd，可选）。
- **7.7** 文档归档：本文件 §0 填日志 + hash；README 前端章节更新；KNOWN-DEBT 登记（RelayView 死代码 + 逃生舱点）。
- **验收**：全量视图一致；`auto build` 零 drift；全流程冒烟通过。

## 5. 关键架构决策

1. **视图切换**：保持 `useViewState` 单例 + `v-if`（不引入 vue-router），App.at 用 `if .store.view == "chats" {...}`。
2. **SSE 鉴权**：保留 EventSource query 参数传 token/workspace（不能设 header）。
3. **fetch 拦截器**：main.ts 全局拦截器（注入 musk_jwt）若生成器不便注入，走 post-init 钩子或 `use{fn}` 手写片段。
4. **API 客户端**：`src/back/api.at` 的 `#[api]` 仅服务前端 codegen（生成 api.ts），后端已 Rust 实现。
5. **产物落点**：生成工程 `pnpm build` 产物落 `web/dist`，后端零改动。
6. **逃生舱原则**：优先扩展生成器；极高成本特性走 `use{fn/component}` + KNOWN-DEBT，不阻塞。

## 6. 风险登记

| 风险 | 级别 | 降级路径 |
|---|---|---|
| SSE 多端点（codegen 只取 first）需扩展 | 🟡 | Phase 1 顺带改 stream_ep 遍历 |
| ChatsView 2767 行单文件转译规模大 | 🔴 | 拆子组件逐个攻破 + 必要时逃生舱 |
| useStreamingDocument 增量 JSON 难纯原生 | 🟡 | use{fn} 逃生舱 + KNOWN-DEBT |
| i18n text 节点 t() 识别复杂 | 🟡 | main.ts/package.json 注入 + use{fn:t} 兜底 |
| 生成器改动引入回归 | 🟡 | Phase 3 建 a2vue golden 测试 |
| 跨两仓库协调（auto-lang 需先合并） | 🟢 | M1-M3 完成后设检查点合并 |

## 7. 与 KNOWN-DEBT 的关系

实施中识别的新条目将登记到 `docs/plans/KNOWN-DEBT-AND-RISKS.md`：
- RelayView（1599 行）死代码不转译
- 可能的逃生舱点（useStreamingDocument 等）及理由

| **Phase 6a** | WikiView 数据层 + 视图骨架。`wiki_store.at`（loadPages/loadPage/createPage/updatePage/deletePage/search/loadTree/loadRawTree）；`wiki_view.at`（wiki-nav pages 列表 + 主内容区：创建表单/编辑/markdown 渲染）；api.at 加 wiki 端点 + WikiPage/WikiPageMeta/TreeNode 类型；app.at 接入 WikiView（wiki tab）；pac.at 加 markstream-vue npm_dep。vue-tsc + vite build 全绿（4 components）。**修复**：markdown 组件名 MarkdownStream→MarkdownRender（auto-lang df5c2e37）；store 局部变量遮蔽（pages→loaded）；markstream-vue 依赖声明 | `auto-lang df5c2e37` | ✅ 4 components 构建通过；markdown 渲染接入 |
| **Phase 7a** | ChatsView 数据层 + 视图骨架。`forge_store.at`（session/messages/sessionList + send/stop）；`chats_view.at`（session 列表 + 消息列表 + 流式 draft + 输入框 + 发送）；`forge_stream.ts`（SSE 消费逃生舱）；api.at 加 chats 端点 + Forge 类型。vue-tsc + vite build 全绿（5 components, 4 stores）。**发现 codegen 关键限制**：store 不支持 use 块（SSE 逃生舱移 widget 层）、store handler 不能传 msg 名作回调、复杂控制流 parser 边界 → 用逃生舱绕开。SSE 事件→store 回写留 7b | _见 7b_ | ✅ 5 components 构建通过 |
| **Phase 7b** | ChatsView SSE 回写机制 + 流式渲染。forge_stream.ts import `useForgeStoreStore()` singleton，SSE onmessage 直接操作 store ref（current_draft/thinking/streaming/error），处理核心事件（delta 追加/thinking/tool_call/tool_result/done/error）。chats_view widget 加 `use { fn: startForgeStream }`，Send 时启动 SSE。**绕开"store 不能传回调"限制**：用 store singleton ref 直接操作（store 模块级 ref 虽不 export，但 useForgeStoreStore() 返回同一份 singleton）。vue-tsc + vite build 全绿 | `1dc360d` | ✅ SSE 回写 + 流式 markdown 渲染构建通过 |
| **Phase 7c** | ChatsView mention + errand/relay/task_plan 内联卡片 + §10 根治。①**数据层**：forge_store.at 加 errands/relays/task_plans 三个 Value ref；forge_stream.ts 补全 errand_*(5)/relay_*(4)/task_plan_spawned(1) 共 10 事件回写（1:1 移植 useForge.ts:362-433）+ 修正 7b tool_call 回写（真正累积到消息 tool_calls 数组，7b 只拼文本）+ ensureAssistantMsg/currentAssistantMsg 追踪当前 assistant 消息。②**逃生舱组件**（7 个 .vue + forge_helpers.ts）：ErrandCard/RelayCard/TaskPlanCard/GenericToolCard 4 类卡片 + MentionInput（v-html backdrop + MentionDropdown 键盘导航 + 命令解析）+ MentionDropdown/AgentAvatar（精简版，去 useAgentConfigs）。③**chats_view.at 重构**：use{component} 引入 5 卡片 + MentionInput；消息列表 role 分支（assistant→markdown/user→text）+ tool_calls 用 for+if-else-if 编排 dispatch/spawn_relay/task_plan/通用 4 类；输入框替换为 MentionInput。④**auto-lang §10 根治**（`60294454`）：移除 lexer.rs 的 markdown raw-string 特殊捕获（根因：把 `{ content: .x }` 吞成 opaque string，content prop 从未走 prop 解析）；vue.rs generate_shadcn_attrs 加 markdown 分支（content→:content 绑定）；a2vue golden 002_markdown。**关键发现**：codegen ext 复制只看 use 块不递归逃生舱内部相对 import（MentionDropdown/AgentAvatar/forge_helpers 需在 use 块声明触发复制）。vue-tsc + vite build 全绿；auto-lang `cargo test` 2876 passed/22 failed（22 均为既有，0 回归） | `1ff0f81`/`897a72f`/`c65fe9b` + auto-lang `60294454` | ✅ mention + 3 类卡片 + §10 根治，构建通过 |
| **Phase 5c** | SpecsView 细化（搜索/module accordion/7类category卡片）——低优先级，移除无效 key 避免多余 div，剩余 R006 警告不阻塞 | `782da1f` | ⚠️ 留后续迭代 |

## 7.5 auto-lang 分支整合（2026-08-07）

Phase 0-5a 期间在 auto-lang 形成了两条并行 Plan 022 分支：
- `plan-musk-022/sse-multi-event`（后端 SSE + 017-chat playwright 验证）
- `plan-musk-022/markdown-mermaid-tag`（i18n + markdown/mermaid + golden + SSE 过滤）

已整合到单一分支 `plan-musk-022/sse-multi-event`（cherry-pick markdown-mermaid-tag 的 i18n/markdown/golden/SSE过滤）：
- `0e3aad9d` Phase 4 SSE 过滤
- `d18e230f` Phase 3 markdown/mermaid + golden
- `8fcfdea5` Phase 2 i18n

旧的 markdown-mermaid-tag / i18n-support 分支已删除（工作已整合）。
worktree 全部清理（plan012-a/b/c + label-class + plan398）。
整合后 Phase 5a 验证仍通过（3 components，vue-tsc+vite build 绿）。

## 8. 后续待做（本计划不实施）

- `frontend/`（配置远程页面）Auto 化（已确认本次仅 web/）
- 逃生舱特性的渐进原生化（若 KNOWN-DEBT 登记）
- a2vue golden 测试全量覆盖（Phase 3 起步）

## 9. Phase 0 发现：web/ 既有 vue-tsc 类型错误（影响验收口径）

**现象**（2026-08-07 实测，仓库 git 状态干净 = 既有问题，非本计划引入）：
- `web/` 的 `npm run build`（`vue-tsc -b && vite build`）**在 `vue-tsc` 阶段失败**，约 12 处 TS 错误：
  - `MarkdownEditor` 的 `modelValue` prop 在多个 category 组件 + WikiView 调用处缺失（ArchitectureCards/CategoryList/DesignCards/PlanCards/ReportCards/ReviewCards/WikiView）
  - `AgentAvatar` 类型 `{} → string`
  - `ChatsView` profession_id `unknown` 类型推断（3 处）
  - `RelayView` 只读 value 赋值（死代码内）
  - `i18n/__tests__/i18n.spec.ts` 找不到 vitest 模块
- 但 `vite build`（跳过类型检查）**成功**，产出可用 dist（视觉/功能基线可建立）。

**对 Plan 022 验收口径的影响（待决）**：
- 生成工程的 `pnpm build` 默认也含 `vue-tsc`（见 Phase 0 生成的 package.json）。若要求生成工程 `vue-tsc` 全绿，需让 .at 生成出比原生更严格的类型——可能不公平（原生自己都不绿）。
- **候选口径**：(a) 生成工程对齐原生（原生 vue-tsc 失败则生成工程也允许跳过类型检查，parity 以 vite build + 视觉/行为为准）；(b) 生成工程要求 vue-tsc 全绿（更严格，需修复或规避原生 TS 错误的对应路径）。

**建议**：口径 (a)——parity 以"vite build 产物 + 视觉/行为一致"为准，类型严格度不作为 parity 硬门槛（与原生对齐）。本条在 Phase 4 首次 parity 比对时最终确认。

## 10. Phase 3 发现：markdown tag 的 props 映射 ~~待完善（Phase 7 深入）~~ ✅ 已修复（Phase 7c）

**现象**（2026-08-07 端到端验证）：`markdown { content: "...", style: "..." }` 生成的 `<MarkdownStream>` 标签能正确识别 + 导入 markstream-vue + 自动加 npm 依赖，但 **props 没有正确转成 Vue 属性绑定**——Aura prop 语法（`content: "..."`）被原样输出到标签内，而非 `content="..."` / `:content="..."`。

**根因**（Phase 7c 重新定位，原根因判断有误）：**不是** registry 的 `props: HashMap::new()` 空映射，而是 `lexer.rs identifier_or_special_block` 有一段 markdown 专属的 raw-string 捕获——遇到 `markdown {` 就把整个 brace 体吞成一个 opaque string token，导致 `content: .x` 从未进入标准 prop 解析路径。

**影响**：~~markdown tag 的**机制**（识别/import/npm依赖）已工作，但**实际渲染**需 props 映射完善。~~ 已无影响（Phase 7c 修复后 `markdown { content: .x }` 正确生成 `<MarkdownRender :content="x">`）。

**处置**：✅ **Phase 7c 已根治**（auto-lang `60294454`）：①移除 lexer.rs 的 markdown raw-string 特殊捕获，`markdown` 现为普通 ident，`{ content: .x }` 走标准 view-node prop 路径；②vue.rs `generate_shadcn_attrs` 加 `"markdown"` 分支（content→`:content` 绑定 / 字面量 `content`，镜像 autodown_editor）；③a2vue golden `002_markdown`（input.at + input.expected.vue）确定性验证。`cargo test` 2876 passed / 22 failed（22 均为 master 既有，0 回归）。
