# 前端组件分组清单（Plan 028 T22 / 附录 A 底稿）

> 历史来源：覆盖 gen 工程全部 29 个组件 + 2 个平台实现 + 状态层（Plan 028 T22）。
> 当前标准：以 PLAN-074 双端 Gallery 基线与全量目录扫描为准，详见 `docs/specs/modules/ui-parity.md` 与 `docs/reports/ui-parity/074-baseline.md`。
> PLAN-077 修订（2026-09-20，reviewed fbca482）：G-对话 Block 组按实际内联/抽离与事件所有权刷新（SD-02）；可达清单以 074 基线 + 077 证据为准。
> 静态清单总计 108 个声明单元、101 个有效测试用例（PLAN-077 后；`node scripts/ui-parity.mjs check` 口径）。

## G-对话 Block（✅ 已全量原生化，Plan 028 主线；PLAN-077 组合一致性落地）

| 组件 | 源 | 迁移状态 | 依赖特性 |
|---|---|---|---|
| ChatMessage | chat_message.at | ✅ component fn + fn 模块 + platform markdown；thinking/tool 块**内联**（无独立 ThinkBlock 挂载）；gate_waiting 内联审批卡（PLAN-073 F-B）；块契约=chat-streaming 契约⑧（state/tkey/gate 载荷） | F1–F9、P1 |
| ThinkBlock | think_block.at | ⚠️ 登记为历史独立件——thinking 块实际内联于 ChatMessage，展开态由父级键列表持态，无独立实例消费者 | F3 |
| ToolBlock | tool_block.at | ⚠️ 登记为历史独立件——工具块分发逻辑已内联于 ChatMessage（gate/dispatch/spawn_relay/task_plan/report/GenericToolCard），文件无实例消费者 | F1/F3 |
| GenericToolCard | generic_tool_card.at | ✅ 同上；展开键走 ForgeStore 键列表（`toggleBlockExpansion`） | F3 |
| ErrandCard | errand_card.at | ✅ 同上 | F1 |
| TaskPlanCard | task_plan_card.at | ✅ 同上 | F1 |
| RelayRunBox | relay_run_box.at | ✅ 同上 + use store: RelayStore | F1/F8 |
| QuestionnaireCard | questionnaire_card.at | ✅ 同上（Index v-model） | F5 |
| UserMessage | user_message.at | ✅ 同上；render_mentions 已 .at 原生化（mention_helpers.at），逐字符扫描用 `sub(i, i+1)`（VM/web `char_at` 语义分歧，chat-streaming 契约⑧注记） | F7 |
| StreamingTable | streaming_table.at | ✅ 同上（Math.max/min）；当前无生产消费者，077 登记可达性证据（render 单元 case） | F3 |

### 消息块事件所有权（PLAN-077 SD-02）

- 展开/折叠：子件 handler 直调 `ForgeStore.ThinkToggle/ToolToggle(key)`，
  键列表存根态（VM 子件 model 全实例共享，本地 expanded 禁用）。
- 分叉：`ChatMessage.ForkFrom(mid)` 空 handler，经实例路由
  `on_fork_from: .ForkFrom($event)` 直达宿主（**键必须 snake 全名形**：
  VM 派发键 `on`+msg 名折叠、Vue `snake_to_pascal` → `@ForkFrom`；
  `onfork` 双端皆断，077 已修）。
- 复制/停止：`CopyContent` 走 `dom.copy_text` FFI；`CancelRun` 直调
  `ForgeStore.CancelRun()`（后端 cancel 端点 + 本地退出流式态）。

## G-对话 Block·平台实现（不迁移，协议挂载）

| 组件 | 协议 | 实现 | 迁移状态 |
|---|---|---|---|
| Markdown（统一文档引擎） | `platform:markdown` / `ports/renderer.at` | Web: `@autodown/engine`；VM: 原生 `autodown` | ✅ PLAN-076 统一引擎双端接入，详见 `docs/specs/modules/autodown-consumption.md` |
| PrismCodeBlock | P2（高亮器，Markdown 内部） | prismjs | ✅ gen src/platform/PrismCodeBlock.vue |

## G-对话壳/输入（依赖 Block 组先行，下一批）

| 组件 | 源 | 迁移状态 | 依赖特性 |
|---|---|---|---|
| ChatsView | chats_view.at | ✅ component fn；mention/命令路由仍走 TS fn | F2/F3/F7 |
| MentionInput | mention_input.at | ✅ component fn；检测/插入逻辑 mention_helpers.ts | F4（回调式 replace 超子集） |
| MentionDropdown | mention_dropdown.at | ✅ component fn | F3 |
| SessionInfo | session_info.at | ✅ component fn；helpers 留 TS | F3/F6 |
| AgentAvatar | agent_avatar.at | ✅ component fn（样式已归还） | F1/F3 |

## G-审批/系统消息

| 组件 | 源 | 迁移状态 | 依赖特性 |
|---|---|---|---|
| GateCard | gate_card.at | ✅ component fn；gate_helpers.ts 留 TS | F6 |
| SecretaryMessage | secretary_message.at | ✅ component fn；useGateInbox 留 TS | F6 |
| SecretaryMessageWrapper | secretary_message_wrapper.at | ✅ component fn | — |
| ReportCard | report_card.at | ✅ component fn | F6 |

## G-导航/框架

| 组件 | 源 | 迁移状态 | 依赖特性 |
|---|---|---|---|
| NavSidebar | nav_sidebar.at | ✅ component fn | — |
| ContentHeader | content_header.at | ✅ component fn（slot） | — |
| WorkspaceSelector | workspace_selector.at | ✅ component fn；workspace_helpers.ts 留 TS | F3/F6 |
| SettingsMenu | settings_menu.at | ✅ component fn；settings_helpers.ts 留 TS | F3/F6 |
| LoginPage | login.at | ✅ component fn | F6 |

## G-文件浏览（PLAN-068）

| 组件 | 源 | 迁移状态 | 依赖特性 |
|---|---|---|---|
| FileTree | filetree.at（移植自 auto-os widgets-gallery，PLAN-614） | ✅ vue 轨（VM 待 import_aliases，登记差异） | tree_util.at / tree_icon.at |
| FilesView | files_view.at | ✅ vue 轨 | files_store.at / ports: renderer+icons+composables+files(.web.at) |
| FilesStore | files_store.at | ✅ store（helper 内联） | back.api: files_tree |
| 查看器 | MarkdownRender（双端 autodown）/ img / video(html:) / 不能打开空态 | ✅ 双端平权（PLAN-076 补齐 VM 端口） | ports/renderer.at |

## G-知识库

| 组件 | 源 | 迁移状态 | 依赖特性 |
|---|---|---|---|
| WikiView | wiki_view.at | ✅ component fn；wiki_helpers.ts 留 TS | F4（文件类型正则） |
| WikiNav | wiki_nav.at | ✅ component fn | F3 |
| RawPreview | raw_preview.at | ✅ component fn；raw_upload.ts 留 TS | — |

## G-规范 / G-计划

| 组件 | 源 | 迁移状态 | 依赖特性 |
|---|---|---|---|
| SpecsView | specs_view.at | ✅ component fn（视图态/computed 化） | — |
| PlansView | plans_view.at | ✅ component fn | — |

## G-状态层

| 模块 | 源 | 迁移状态 | 依赖特性 |
|---|---|---|---|
| ForgeStore | forge_store.at | ✅ SSE 消费原生化（Sse.open/OnStreamEvent） | F8/F9 |
| RelayStore | relay_store.at | ✅ 全量原生化（Http.* + Sse.open + gate_signal 中转） | F8 |
| AuthStore / PlansStore / SpecsStore / WikiStore | *_store.at | ✅ store 原生；各自 helpers 留 TS | F3/F8 |
| 遗留 TS | relay_commands / gate_/wiki_/settings_/workspace_/session_info_helpers / useGateInbox / useTheme / useT / useAccentColor / useAgentConfigs / useKeyboardShortcuts / inject_styles（token+非块组） | ⏳ 随各组后续立项（mention_helpers 已 .at 原生化，PLAN-077） | F4 闭包 replace 等 |

## 后续立项建议优先级

1. **G-对话壳/输入**（Block 组直系依赖；mention 域需 a2ts 支持回调式 replace 或平台化）
2. **G-审批/系统消息**（gate/secretary helpers 纯度高，成本低）
3. **G-知识库 / G-导航/框架**（helpers 各自独立）
4. **VM 渲染目标补齐**（store facade 概念 + 平台协议 VM 实现——T21 观察项）
