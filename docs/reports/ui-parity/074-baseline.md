# PLAN-074 Gallery baseline

Generated: 2026-09-19T09:18:50.690Z

Units: 62 (54 reachable)
Cases: 57

## Inventory

| Unit | Source | Platforms | Reachability | Consumer path | Triage |
|---|---|---|---|---|---|
| `AgentAvatar` | `src/front/agent_avatar.at` | vm/vue | reachable | App → ChatsView → ChatMessage → AgentAvatar | PLAN-077 |
| `App` | `src/front/app.at` | vm/vue | reachable | App | PLAN-079 |
| `ChatsView` | `src/front/chats_view.at` | vm/vue | reachable | App → ChatsView | PLAN-078 |
| `ChatMessage` | `src/front/chat_message.at` | vm/vue | reachable | App → ChatsView → ChatMessage | PLAN-077 |
| `ContentHeader` | `src/front/content_header.at` | vm/vue | reachable | App → PlansView → ContentHeader | PLAN-078 |
| `ErrandCard` | `src/front/errand_card.at` | vm/vue | reachable | App → ChatsView → ChatMessage → ErrandCard | PLAN-077 |
| `FilesView` | `src/front/files_view.at` | vm/vue | reachable | App → FilesView | PLAN-078 |
| `FileTree` | `src/front/filetree.at` | vm/vue | reachable | App → FilesView → FileTree | PLAN-078 |
| `GateCard` | `src/front/gate_card.at` | vm/vue | reachable | App → ChatsView → GateCard | PLAN-077 |
| `GenericToolCard` | `src/front/generic_tool_card.at` | vm/vue | unreachable | — | retired-review |
| `Icon` | `src/front/lib/icon.at` | vm/vue | unreachable | — | retired-review |
| `LoginPage` | `src/front/login.at` | vm/vue | reachable | App → LoginPage | PLAN-078 |
| `MentionDropdown` | `src/front/mention_dropdown.at` | vm/vue | reachable | App → ChatsView → MentionInput → MentionDropdown | PLAN-078 |
| `MentionInput` | `src/front/mention_input.at` | vm/vue | reachable | App → ChatsView → MentionInput | PLAN-078 |
| `NavListItem` | `src/front/nav_item.at` | vm/vue | unreachable | — | retired-review |
| `NavSidebar` | `src/front/nav_sidebar.at` | vm/vue | reachable | App → PlansView → NavSidebar | PLAN-078 |
| `PlansView` | `src/front/plans_view.at` | vm/vue | reachable | App → PlansView | PLAN-078 |
| `DeleteConfirmDialog` | `src/front/ports/delete_confirm.vm.at` | vm/vue | reachable | App → ChatsView → DeleteConfirmDialog | PLAN-078 |
| `Markdown` | `src/front/ports/renderer.vm.at` | vm/vue | reachable | App → PlansView → Markdown | PLAN-076 |
| `QuestionnaireCard` | `src/front/questionnaire_card.at` | vm/vue | reachable | App → ChatsView → QuestionnaireCard | PLAN-077 |
| `RawPreview` | `src/front/raw_preview.at` | vm/vue | reachable | App → WikiView → RawPreview | PLAN-076 |
| `RelayRunBox` | `src/front/relay_run_box.at` | vm/vue | reachable | App → ChatsView → ChatMessage → RelayRunBox | PLAN-077 |
| `ReportCard` | `src/front/report_card.at` | vm/vue | reachable | App → ChatsView → ReportCard | PLAN-078 |
| `SecretaryMessage` | `src/front/secretary_message.at` | vm/vue | reachable | App → ChatsView → SecretaryMessageWrapper → SecretaryMessage | PLAN-077 |
| `SecretaryMessageWrapper` | `src/front/secretary_message_wrapper.at` | vm/vue | reachable | App → ChatsView → SecretaryMessageWrapper | PLAN-077 |
| `SessionInfo` | `src/front/session_info.at` | vm/vue | reachable | App → ChatsView → SessionInfo | PLAN-078 |
| `SettingsMenu` | `src/front/settings_menu.at` | vm/vue | reachable | App → SettingsMenu | PLAN-078 |
| `ArchitectureCards` | `src/front/specs_category.at` | vm/vue | reachable | App → SpecsView → ArchitectureCards | PLAN-078 |
| `DesignCards` | `src/front/specs_category.at` | vm/vue | reachable | App → SpecsView → DesignCards | PLAN-078 |
| `ReportCards` | `src/front/specs_category.at` | vm/vue | reachable | App → SpecsView → ReportCards | PLAN-078 |
| `ReviewCards` | `src/front/specs_category.at` | vm/vue | reachable | App → SpecsView → ReviewCards | PLAN-078 |
| `TestsCards` | `src/front/specs_category.at` | vm/vue | reachable | App → SpecsView → TestsCards | PLAN-078 |
| `GoalsTable` | `src/front/specs_category.at` | vm/vue | reachable | App → SpecsView → GoalsTable | PLAN-078 |
| `StatusTransition` | `src/front/specs_detail.at` | vm/vue | reachable | App → SpecsView → GoalDetailModal → StatusTransition | PLAN-078 |
| `RelationsPanel` | `src/front/specs_detail.at` | vm/vue | reachable | App → SpecsView → GoalDetailModal → RelationsPanel | PLAN-078 |
| `GoalDetail` | `src/front/specs_detail.at` | vm/vue | reachable | App → SpecsView → GoalDetailModal → GoalDetail | PLAN-078 |
| `ReviewDetail` | `src/front/specs_detail.at` | vm/vue | reachable | App → SpecsView → TestsCards → SpecItemRow → SpecItemDetail → ReviewDetail | PLAN-078 |
| `TestDetail` | `src/front/specs_detail.at` | vm/vue | reachable | App → SpecsView → TestsCards → SpecItemRow → SpecItemDetail → TestDetail | PLAN-078 |
| `ReportDetail` | `src/front/specs_detail.at` | vm/vue | reachable | App → SpecsView → TestsCards → SpecItemRow → SpecItemDetail → ReportDetail | PLAN-078 |
| `SpecItemDetail` | `src/front/specs_detail.at` | vm/vue | reachable | App → SpecsView → TestsCards → SpecItemRow → SpecItemDetail | PLAN-078 |
| `GoalDetailModal` | `src/front/specs_detail.at` | vm/vue | reachable | App → SpecsView → GoalDetailModal | PLAN-078 |
| `TagInput` | `src/front/specs_editors.at` | vm/vue | reachable | App → SpecsView → GoalDetailModal → GoalEditor → TagInput | PLAN-078 |
| `AutoDownEditor` | `src/front/specs_editors.at` | vm/vue | reachable | App → SpecsView → TestsCards → SpecItemRow → AutoDownEditor | PLAN-076 |
| `TestEditor` | `src/front/specs_editors.at` | vm/vue | reachable | App → SpecsView → TestsCards → SpecItemRow → TestEditor | PLAN-078 |
| `GoalEditor` | `src/front/specs_editors.at` | vm/vue | reachable | App → SpecsView → GoalDetailModal → GoalEditor | PLAN-078 |
| `MarkdownEditor` | `src/front/specs_editors.at` | vm/vue | unreachable | — | retired-review |
| `StatusBadge` | `src/front/specs_leaf.at` | vm/vue | reachable | App → SpecsView → GoalsTable → StatusBadge | PLAN-078 |
| `SpecLink` | `src/front/specs_leaf.at` | vm/vue | unreachable | — | retired-review |
| `SpecItemRow` | `src/front/specs_leaf.at` | vm/vue | reachable | App → SpecsView → TestsCards → SpecItemRow | PLAN-078 |
| `CategoryList` | `src/front/specs_leaf.at` | vm/vue | reachable | App → SpecsView → CategoryList | PLAN-078 |
| `TreeView` | `src/front/specs_tree.at` | vm/vue | unreachable | — | retired-review |
| `SpecsView` | `src/front/specs_view.at` | vm/vue | reachable | App → SpecsView | PLAN-078 |
| `StreamingTable` | `src/front/streaming_table.at` | vm/vue | unreachable | — | retired-review |
| `TaskPlanCard` | `src/front/task_plan_card.at` | vm/vue | reachable | App → ChatsView → ChatMessage → TaskPlanCard | PLAN-077 |
| `ToolBlock` | `src/front/tool_block.at` | vm/vue | unreachable | — | retired-review |
| `ToolGateCard` | `src/front/tool_gate_card.at` | vm/vue | reachable | App → ChatsView → ChatMessage → ToolGateCard | PLAN-077 |
| `TreeIcon` | `src/front/tree_icon.at` | vm/vue | reachable | App → FilesView → FileTree → TreeIcon | PLAN-078 |
| `UserMessage` | `src/front/user_message.at` | vm/vue | reachable | App → ChatsView → ChatMessage → UserMessage | PLAN-077 |
| `WhitelistView` | `src/front/whitelist_view.at` | vm/vue | reachable | App → WhitelistView | PLAN-078 |
| `WikiNav` | `src/front/wiki_nav.at` | vm/vue | reachable | App → WikiView → WikiNav | PLAN-078 |
| `WikiView` | `src/front/wiki_view.at` | vm/vue | reachable | App → WikiView | PLAN-078 |
| `WorkspaceSelector` | `src/front/workspace_selector.at` | vm/vue | reachable | App → WorkspaceSelector | PLAN-078 |

## Port variants

| Port | VM source | Vue source | Platforms |
|---|---|---|---|
| `composables` | `src/front/ports/composables.vm.at` | `src/front/ports/composables.web.at` | vm/vue |
| `delete_confirm` | `src/front/ports/delete_confirm.vm.at` | `src/front/ports/delete_confirm.web.at` | vm/vue |
| `files` | `src/front/ports/files.vm.at` | `src/front/ports/files.web.at` | vm/vue |
| `icons` | — | `src/front/ports/icons.web.at` | vue |
| `platform` | `src/front/ports/platform.vm.at` | `src/front/ports/platform.web.at` | vm/vue |
| `renderer` | `src/front/ports/renderer.vm.at` | `src/front/ports/renderer.web.at` | vm/vue |
| `upload` | `src/front/ports/upload.vm.at` | `src/front/ports/upload.web.at` | vm/vue |
| `whitelist` | `src/front/ports/whitelist.vm.at` | `src/front/ports/whitelist.web.at` | vm/vue |

## Cases

| Case | Unit | State | Owner | Mode |
|---|---|---|---|---|
| `chat-message-pair` | `ChatMessage` | thinking-open-and-tool-closed | PLAN-077 | render-and-interact |
| `chat-thinking-streaming` | `ChatMessage` | streaming-thinking-tail | PLAN-077 | fixture-contract |
| `chat-text-block` | `ChatMessage` | completed-text-with-markdown | PLAN-076 | fixture-contract |
| `chat-tool-gate` | `ChatMessage` | gate-waiting | PLAN-077 | fixture-contract |
| `inventory-AgentAvatar` | `AgentAvatar` | initial | PLAN-077 | inventory-only |
| `inventory-App` | `App` | initial | PLAN-079 | inventory-only |
| `inventory-ChatsView` | `ChatsView` | initial | PLAN-078 | inventory-only |
| `inventory-ContentHeader` | `ContentHeader` | initial | PLAN-078 | inventory-only |
| `inventory-ErrandCard` | `ErrandCard` | initial | PLAN-077 | inventory-only |
| `inventory-FilesView` | `FilesView` | initial | PLAN-078 | inventory-only |
| `inventory-FileTree` | `FileTree` | initial | PLAN-078 | inventory-only |
| `inventory-GateCard` | `GateCard` | initial | PLAN-077 | inventory-only |
| `inventory-LoginPage` | `LoginPage` | initial | PLAN-078 | inventory-only |
| `inventory-MentionDropdown` | `MentionDropdown` | initial | PLAN-078 | inventory-only |
| `inventory-MentionInput` | `MentionInput` | initial | PLAN-078 | inventory-only |
| `inventory-NavSidebar` | `NavSidebar` | initial | PLAN-078 | inventory-only |
| `inventory-PlansView` | `PlansView` | initial | PLAN-078 | inventory-only |
| `inventory-DeleteConfirmDialog` | `DeleteConfirmDialog` | initial | PLAN-078 | inventory-only |
| `inventory-Markdown` | `Markdown` | initial | PLAN-076 | inventory-only |
| `inventory-QuestionnaireCard` | `QuestionnaireCard` | initial | PLAN-077 | inventory-only |
| `inventory-RawPreview` | `RawPreview` | initial | PLAN-076 | inventory-only |
| `inventory-RelayRunBox` | `RelayRunBox` | initial | PLAN-077 | inventory-only |
| `inventory-ReportCard` | `ReportCard` | initial | PLAN-078 | inventory-only |
| `inventory-SecretaryMessage` | `SecretaryMessage` | initial | PLAN-077 | inventory-only |
| `inventory-SecretaryMessageWrapper` | `SecretaryMessageWrapper` | initial | PLAN-077 | inventory-only |
| `inventory-SessionInfo` | `SessionInfo` | initial | PLAN-078 | inventory-only |
| `inventory-SettingsMenu` | `SettingsMenu` | initial | PLAN-078 | inventory-only |
| `inventory-ArchitectureCards` | `ArchitectureCards` | initial | PLAN-078 | inventory-only |
| `inventory-DesignCards` | `DesignCards` | initial | PLAN-078 | inventory-only |
| `inventory-ReportCards` | `ReportCards` | initial | PLAN-078 | inventory-only |
| `inventory-ReviewCards` | `ReviewCards` | initial | PLAN-078 | inventory-only |
| `inventory-TestsCards` | `TestsCards` | initial | PLAN-078 | inventory-only |
| `inventory-GoalsTable` | `GoalsTable` | initial | PLAN-078 | inventory-only |
| `inventory-StatusTransition` | `StatusTransition` | initial | PLAN-078 | inventory-only |
| `inventory-RelationsPanel` | `RelationsPanel` | initial | PLAN-078 | inventory-only |
| `inventory-GoalDetail` | `GoalDetail` | initial | PLAN-078 | inventory-only |
| `inventory-ReviewDetail` | `ReviewDetail` | initial | PLAN-078 | inventory-only |
| `inventory-TestDetail` | `TestDetail` | initial | PLAN-078 | inventory-only |
| `inventory-ReportDetail` | `ReportDetail` | initial | PLAN-078 | inventory-only |
| `inventory-SpecItemDetail` | `SpecItemDetail` | initial | PLAN-078 | inventory-only |
| `inventory-GoalDetailModal` | `GoalDetailModal` | initial | PLAN-078 | inventory-only |
| `inventory-TagInput` | `TagInput` | initial | PLAN-078 | inventory-only |
| `inventory-AutoDownEditor` | `AutoDownEditor` | initial | PLAN-076 | inventory-only |
| `inventory-TestEditor` | `TestEditor` | initial | PLAN-078 | inventory-only |
| `inventory-GoalEditor` | `GoalEditor` | initial | PLAN-078 | inventory-only |
| `inventory-StatusBadge` | `StatusBadge` | initial | PLAN-078 | inventory-only |
| `inventory-SpecItemRow` | `SpecItemRow` | initial | PLAN-078 | inventory-only |
| `inventory-CategoryList` | `CategoryList` | initial | PLAN-078 | inventory-only |
| `inventory-SpecsView` | `SpecsView` | initial | PLAN-078 | inventory-only |
| `inventory-TaskPlanCard` | `TaskPlanCard` | initial | PLAN-077 | inventory-only |
| `inventory-ToolGateCard` | `ToolGateCard` | initial | PLAN-077 | inventory-only |
| `inventory-TreeIcon` | `TreeIcon` | initial | PLAN-078 | inventory-only |
| `inventory-UserMessage` | `UserMessage` | initial | PLAN-077 | inventory-only |
| `inventory-WhitelistView` | `WhitelistView` | initial | PLAN-078 | inventory-only |
| `inventory-WikiNav` | `WikiNav` | initial | PLAN-078 | inventory-only |
| `inventory-WikiView` | `WikiView` | initial | PLAN-078 | inventory-only |
| `inventory-WorkspaceSelector` | `WorkspaceSelector` | initial | PLAN-078 | inventory-only |

## Runtime evidence

- prepare chat-message-pair: **prepared** (source)
- vm chat-message-pair: **snapshot-ok** (runtime-smoke)
- vue chat-message-pair: **http-ok** (runtime-smoke)

## Rules

- This is a current-state inventory, not a pass baseline.
- Missing runtime or screenshot evidence remains a failure.
- Differences are assigned to PLAN-075–079 or the responsible dependency.

## Known blockers

- VM ChatMessage reaches `snapshot-ok`, reset/event-spy verification, and `autoui_screenshot` after the minimal production compatibility fix `let has_think` → `var`; the snapshot still reports native renderer degradations (`self-stretch`) and `blocks` state-read warnings.
- Vue gallery reaches project generation, dependency install, AutoVM backend, and Vite front endpoint (`http-ok`, runtime-smoke); browser-driven dual-mode screenshots and pixel/DOM parity comparison are assigned to PLAN-075/076.
