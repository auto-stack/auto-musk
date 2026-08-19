# 前端组件分组清单（Plan 028 T22 / 附录 A 底稿）

> 覆盖 gen 工程全部 29 个组件 + 2 个平台实现 + 状态层。分组决定后续 Auto 化立项的批次与
> 依赖顺序；「迁移状态」以 Plan 028 完成时点为准（Block 组 = 本计划主线，已全量原生化）。

## G-对话 Block（✅ 已全量原生化，Plan 028 主线）

| 组件 | 源 | 迁移状态 | 依赖特性 |
|---|---|---|---|
| ChatMessage | chat_message.at | ✅ component fn + fn 模块 + platform markdown | F1–F9、P1 |
| ThinkBlock | think_block.at | ✅ component fn（样式已归还 style 块） | F3 |
| ToolBlock | tool_block.at | ✅ 同上 | F1/F3 |
| GenericToolCard | generic_tool_card.at | ✅ 同上 | F3 |
| ErrandCard | errand_card.at | ✅ 同上 | F1 |
| TaskPlanCard | task_plan_card.at | ✅ 同上 | F1 |
| RelayRunBox | relay_run_box.at | ✅ 同上 + use store: RelayStore | F1/F8 |
| QuestionnaireCard | questionnaire_card.at | ✅ 同上（Index v-model） | F5 |
| UserMessage | user_message.at | ✅ 同上（renderMentions 留 TS） | F7 |
| StreamingTable | streaming_table.at | ✅ 同上（Math.max/min） | F3 |

## G-对话 Block·平台实现（不迁移，协议挂载）

| 组件 | 协议 | 实现 | 迁移状态 |
|---|---|---|---|
| Markdown（原 StreamingRenderer） | `platform:markdown` | markstream-vue + useStreamingDocument | ✅ gen src/platform/markdown.vue |
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
| 遗留 TS | mention_helpers / relay_commands / gate_/wiki_/settings_/workspace_/session_info_helpers / useGateInbox / useTheme / useT / useAccentColor / useAgentConfigs / useKeyboardShortcuts / inject_styles（token+非块组） | ⏳ 随各组后续立项 | F4 闭包 replace 等 |

## 后续立项建议优先级

1. **G-对话壳/输入**（Block 组直系依赖；mention 域需 a2ts 支持回调式 replace 或平台化）
2. **G-审批/系统消息**（gate/secretary helpers 纯度高，成本低）
3. **G-知识库 / G-导航/框架**（helpers 各自独立）
4. **VM 渲染目标补齐**（store facade 概念 + 平台协议 VM 实现——T21 观察项）
