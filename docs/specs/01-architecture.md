# 全局架构

> 基于 2026-08-14 全量代码扫描。所有结论以代码为依据。

## 1. 后端（backend/crates/musk/src/）

Rust + axum 0.8 HTTP 服务。`lib.rs:5-29` 声明手写模块 + `lib.rs:28` `auto_generated`（a2r 转译）+ `relay/` 子模块。

### 核心模块

| 模块 | 职责 | 关键 pub item |
|---|---|---|
| `server.rs` | HTTP 外壳 + `AppState` + SSE handler + 路由组装 | `AppState{client,auth,registry}` / `serve()` / `stream_event_to_json()` |
| `specs.rs` | 6 区 Spec Ledger 数据模型 + 状态机 + JSON 持久化 | `SectionType`(6) / `SpecStatus`(23) / `SpecsStore`(upsert/transition/delete/drift_check) / `derive_statuses()` |
| `plans.rs` | Plan 文件树 + 5 态状态机 + HTTP 路由（hw） | `PlanStatus`(5) / `PlansStore`(create/transition/archive/merge) / `plans_routes()` |
| `plan_merge.rs` | Plan→Spec 合并（§N→section 映射） | `plan_to_items()` / `upsert_items_into_doc()` |
| `spec_tree.rs` | docs/specs/ 文件树 API（hw，复用 wiki::build_tree） | `spec_tree_routes()` / `spec_tree` / `spec_file` |
| `tools.rs` | 8 本地工具 + `map_path_error`(SecurityDenied)；RunCommand 为 PLAN-040 流式版（timeout 参数/临时文件尾注/pi 退出码语义，经 CommandRunner） | ReadFile/WriteFile/EditFile/Search/ListDir/Glob/RunCommand/DisplayImage |
| `edit_diff.rs` | edit_file 匹配/应用核心（PLAN-039，pi edit-diff.ts 移植） | `normalize_for_fuzzy_match()` / `fuzzy_find_text()` / `apply_edits_to_normalized_content()`（行级保留+重叠检测+五类自愈报错） |
| `tool_truncate.rs` | 共享截断模块（PLAN-039，字符边界安全） | `truncate_head()/truncate_tail()/truncate_line()` / `TruncationResult` |
| `command_runner.rs` | 执行后端接缝 + 本地 tokio 实现（PLAN-040，Ash 后座契约见模块文档） | `CommandRunner` / `LocalRunner` / `ExecOptions`(on_data/timeout/env) / `ExecOutcome` / `kill_process_tree()`（Win taskkill /T /F；Unix killpg） |
| `output_accumulator.rs` | 有界内存流式累积器（PLAN-040，pi output-accumulator 移植） | `OutputAccumulator`（滚动尾部 2×maxBytes / 流式 UTF-8 解码 / 超限临时文件转储） / `OutputSnapshot` |
| `tool_context.rs` | 工具执行上下文 + 实时进度通道（PLAN-040） | `ToolContext{state,workspace_id,parent_conversation_id,progress}` / `ProgressSink::for_run()/send()`（100ms 节流由工具侧） |
| `tool_safety.rs` | path confinement + run_command 分级 + cmd 路径校验 | `resolve_within_project()` / `project_root()` / `classify_command()` / `confine_command_paths()` |
| `workspace.rs` | workspace 注册表 + store bundle + 数据迁移 | `WorkspaceRegistry` / `WorkspaceStores` / `WorkspaceQuery` |
| `chats.rs` | chat session 持久化 + spec-change 审批队列 | `ChatStore` / `ChatSession` / queue_spec_change/approve/reject |
| `conversation.rs` | 统一会话（chat+flow 抽象）+ jsonl + broadcast SSE | `ConversationStore` / `Turn` / `chat_message_to_turns()` |
| `relay/` | Relay 编排引擎（driver/store/api/feature_dev/profession/flows/task_plan） | `MuskAgentFactory` / `RunStore` / `RunEvent`(17：含 PLAN-040 `ToolUpdate` 流式 partial——SSE-only 易态，不落 run.events/不镜像 turns) / `drive_loop()` |
| `orch_tools.rs` | 编排工具（spawn_relay/dispatch/bring_in/task_plan） | `SpawnRelay` / `Dispatch` / `BringIn` / `run_errand_agent()` |
| `mode.rs` | agent 运行模式（.at 配置） | `AgentMode` / `BUILTIN_MODES`(superpowers/basic/coding/review) |
| `auto_generated/` | a2r 转译模块（server/auth/relay_api/wiki 等） | `build_router()`(38 路由) / `extern_impl`(委托 hw) / `server_stream`(6 SSE handler) |

### API 端点（按功能分组）

| 功能 | 端点 | 来源 |
|---|---|---|
| **Specs ledger** | GET/POST /api/specs, /api/specs/item, /api/specs/transition, /api/specs/overview, /api/specs/drift-check | build_router (ag) |
| **Specs 文件树** | GET /api/specs/tree, /api/specs/file/{*path} | spec_tree.rs (hw) |
| **Plans** | GET/POST /api/plans, /api/plans/{seq}/transition\|archive\|merge | plans.rs (hw) |
| **Chats** | /api/chats/session(s), message, approve, reject | build_router (ag) |
| **Conversations** | /api/conversations, /api/conversations/{id} | build_router (ag) |
| **Workspace** | /api/workspace/list\|open\|status\|browse\|initialize | build_router (ag) |
| **Run/Stream** | POST /api/run, /api/run/stream, /api/workflow/run/stream | server.rs (hw .route) |
| **Chat SSE** | GET /api/chats/session/{id}/stream, /api/conversations/{id}/stream | server_stream (ag) |
| **Relay** | /api/forge/relay/runs, advance, gate, events, professions, flows | relay_api (ag) |
| **Wiki** | /api/forge/wiki/{project}/tree\|pages\|page\|search, /api/forge/raw/... | wiki.rs (ag) |
| **Auth** | /api/auth/login\|register\|me\|logout | build_router (ag) |
| **Config** | /api/config, /api/modes, /api/roles, /api/app-config, /api/app-harness | build_router (ag) |

### hw vs ag 架构（escape-hatch）

主 router = `auto_generated::server::build_router()`（38 路由，ag 转译）。转译 handler 经 `extern_impl` 委托到 hw store/registry。hw escape-hatch 直接挂载（plans/spec_tree），因 a2r 转译器 drift（KNOWN-DEBT）。

## 2. 前端（web/ + src/front/ .at → gen/）

### 双前端

| 轨 | 位置 | 栈 | 产物 |
|---|---|---|---|
| 原生 web | `web/` | Vue3 + TS + Vite（手写 SPA，单例 ref store，无路由/pinia） | web/dist（musk serve 托管） |
| Auto 轨 | `src/front/*.at` + `src/back/api.at` | AutoLang .at（widget/component fn/store）→ auto build | gen/front/vue/（完整 Vue 工程） |

### web/ 视图

- **ChatsView**（2767 行）：session-sidebar + 消息 canvas（气泡/thinking/tool_cards 4 类）+ 流式 draft + MentionInput
- **SpecsView**（1515 行）：structured（6 section + item CRUD）/ tree 双模式
- **WikiView**（865 行）：wiki-nav + page 渲染/编辑 + raw 树
- **PlansView**（519 行）：plan 列表 + 详情 + 状态流转 + merge
- **RelayView**（1599 行）：runs 列表 + 步骤推进 + gate

### .at 源（Auto 轨）

- **widget**：app/chats_view/specs_view/plans_view/wiki_view/login
- **store**：forge_store/auth_store/specs_store/plans_store/wiki_store
- **component fn**：~~已退役~~（PLAN-037 Phase 4 全量迁移为 widget，widget 为唯一 UI 单元：6 页面 + 24 子组件）
- **api.at**：51 个 `#[api]` 契约（codegen → gen/lib/api.ts）
- **inject_styles.ts**（1017 行）：全局 CSS 兜底（gen 组件 `<style>` 空）
- **逃生舱**：forge_stream.ts（SSE 消费）/ setup_auth_fetch.ts（fetch monkey-patch）/ forge_helpers.ts / StreamingRenderer.vue

### codegen 流程

`auto build`（auto-lang）→ `.at` → `gen/front/vue/`（Vue SFC + store composable + api.ts + ext 复制）。shadcn 模式（`VueGenerator::new_shadcn`）。web 依赖经顶层 `use.web` 语句声明（PLAN-037；use 块为废弃别名）后复制/转译到 `gen/ext/`；非 web 后端遇 `use.web` 显式报错。**跨后端 facade**：`src/front/ports/*.web.at` 五域端口（platform/icons/renderer/composables/upload，Plan 424 扩展至 component/composable 符号 re-export 转发）承载全部 web 绑定——调用面 use.web 恒引 `.at` 端口名，非 `.at` 目标零命中；构建期按目标解析 `X.<target>.at` adapter，缺源显式报错。**v-model**：子 widget 的 model 变量 = 双向通道（defineModel 编译），调用点传可写状态槽自动折叠 `v-model:name`，非槽目标硬错误（PLAN-037 Phase 1）。

## 3. 核心数据流

### Chat（用户消息 → LLM → SSE）

前端 `POST /api/chats/session/{id}/message` 排队 → `GET .../stream`（SSE）→ musk `set_current_root(ws.root)` → `build_agent_with_context` → `agent.run_stream` → ReAct loop（工具本地执行）→ StreamEvent → SSE JSON → 前端 `forge_stream.ts` 路由 → 持久化 assistant + dual-write conversation。

### Spec CRUD（双落点）

- **Ledger**（`.autoos/specs.json`）：6 区 `SpecsDocument`，每次 upsert → `rebuild_relations` + `derive_statuses` + version++
- **文件树**（`docs/specs/`）：`spec_tree.rs` 复用 `wiki::build_tree`
- **Merge 桥接**：review_done plan → `plan_to_items`（§N→section 映射）→ `upsert_items_into_doc`

### Plan 状态机

`Drafting→Executing|ReviewDone` → `ExecutionDone→ReviewDone` → `ReviewDone→Merged`（merge = `plan_to_items` + `upsert_items_into_doc` + archive）。磁盘唯一事实源（`docs/plans/NNN-*.md` frontmatter）。

### Tool 执行安全

1. `resolve_within_project(path)` — path confinement（workspace root + canonicalize + starts_with）
2. `map_path_error(e)` — "outside project root" → `ToolError::SecurityDenied{kind:"path_confined"}`
3. `classify_command(cmd)` — 白名单/危险模式分级（Allowed/NeedsApproval/PAUSED）
4. `confine_command_paths(cmd)` — cmd 路径参数 confinement（堵 cat/type 绕过）
5. `.current_dir(project_root())` — run_command cwd 限制

## 4. 关键设计

### Path confinement 三层 root

`ROOT_OVERRIDE`（thread 测试）> `CURRENT_ROOT`（thread workspace，agent 驱动入口 set/clear）> `PROJECT_ROOT`（OnceLock 进程级，init_project_root 在 main.rs:90）。

### Workspace registry

全局索引 `~/.config/autoos/workspaces.json`（default_workspace_id + Vec<WorkspaceMeta>）。`WorkspaceStores::new` 装配 `{root}/.autoos/` 全部 store。`get(ws_id)` 按 canonical path 缓存。`?workspace=<id>` 统一查询。

### ToolError SecurityDenied + driver 短路

`SecurityDenied{kind,path,root,hint}`（auto-ai-agent）让 driver/前端识别安全拒绝。agent loop 连续 ≥3 次 security error → 注入"换思路"强 hint（短路无效重试）。
