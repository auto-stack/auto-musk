# 模块索引

> 各模块的详细 spec 待后续从代码扫描沉淀。核心架构见 `../01-architecture.md`。

## 后端模块（backend/crates/musk/src/）

| 模块 | 职责 |
|---|---|
| server.rs | HTTP 外壳 + AppState + SSE + 路由组装 |
| specs.rs | 6 区 Spec Ledger + 状态机 |
| plans.rs | Plan 文件树 + 5 态状态机 |
| plan_merge.rs | Plan→Spec 合并引擎 |
| spec_tree.rs | docs/specs/ 文件树 API |
| tools.rs | 9 本地工具 + map_path_error |
| tool_safety.rs | path confinement + run_command 安全 |
| workspace.rs | workspace 注册表 + store bundle |
| chats.rs | chat session + 审批队列 |
| conversation.rs | 统一会话模型 |
| relay/ | 编排引擎（driver/store/api/profession/flows/task_plan） |
| orch_tools.rs | 编排工具（spawn_relay/dispatch/bring_in） |
| mode.rs | agent 运行模式 |
| auto_generated/ | a2r 转译模块（server/auth/relay_api/wiki） |

## 前端模块

### web/（原生）
- views: ChatsView / SpecsView / WikiView / PlansView / RelayView / LoginView
- composables: useForge / useSpecs / useRelay / useWiki / useAuth / useGateInbox 等
- components: TreeView / MarkdownContent / WorkspaceSelector / NavSidebar / ContentHeader 等

### .at 源（Auto 轨）
- widget: app / chats_view / specs_view / plans_view / wiki_view / login
- store: forge_store / auth_store / specs_store / plans_store / wiki_store
- component fn: nav_sidebar / content_header / wiki_nav / workspace_selector / chat_message / mention_input / generic_tool_card 等
- api.at: 51 个 #[api] 契约
- inject_styles.ts: 全局 CSS 兜底
- 逃生舱: forge_stream.ts / setup_auth_fetch.ts / forge_helpers.ts / StreamingRenderer.vue
