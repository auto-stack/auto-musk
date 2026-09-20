# PLAN-078 Evidence Ledger — 非消息输入、导航与页面组合双端一致性

> 生成时间：2026-09-20T05:07:40.160Z  
> 计划编号：PLAN-078  
> 状态：执行完成 (execution_done)  
> 基线 Commit: `04eb90531643223076a5d9d2a9572ccaaf6d1c14`  
> 工作区：`D:/autostack/.wt/musk-078/auto-musk` (分支 `plan-078-dev`)  
> 关联仓库：auto-lang `3df7b21a2`（分支 `auto-musk-dev`，handler 前向符号导出预注册修复）、auto-down `d1a83b6`（detached HEAD，构建消费）

---

## 1. 任务完成进度 (Task Verification Matrix)

| 任务 ID | 任务说明 | 覆盖 AC | 状态 | 验证命令与结果 | 证据落点 |
|---|---|---|---|---|---|
| **T-01** | 补齐非消息单元场景 | AC-01 | **PASS** | 覆盖全部 108 声明与 102 场景，零未分配；为 MentionInput、MentionDropdown、TagInput、NavSidebar、WorkspaceSelector、SettingsMenu、DeleteConfirmDialog、FileTree、FilesView、SpecsView、WhitelistView、WikiView、PlansView、ChatsView 14 个核心非消息单元补齐显式场景与 fixture | `tests/ui-parity/cases.json`, `tests/ui-parity/fixtures/` |
| **T-02** | 输入能力收敛 | AC-02 | **PASS** | MentionInput composer 一体化、TagInput 键盘响应、IME 组词守卫与候选过滤无缝工作；消除了 `Value.preventDefault` 静态符号歧义；双端 observable 合同一致 | `src/front/mention_input.at`, `src/front/specs_editors.at`, `tmp/ui-parity/PLAN-078/` |
| **T-03** | 导航浮层和壳收敛 | AC-03 | **PASS** | NavSidebar 折叠收缩与展开切换正常；WorkspaceSelector popover 触发锚定、SettingsMenu 受控 dialog 模态弹层、DeleteConfirmDialog 双端确认撤销全量通过 | `src/front/nav_sidebar.at`, `src/front/workspace_selector.at`, `src/front/settings_menu.at`, `src/front/ports/delete_confirm.vm.at` |
| **T-04** | 页面组合验证 | AC-04 | **PASS** | FilesView/FileTree 树形层级、SpecsView 分类与详情骨架、WhitelistView 目录表单、WikiView 知识库结构、PlansView 计划路线图全部加载通过，零占位缺件；多 store 边界解耦适配器保证 gallery 单源性 | `src/front/files_view.at`, `src/front/specs_view.at`, `src/front/whitelist_view.at`, `src/front/wiki_view.at`, `src/front/plans_view.at` |
| **T-05** | 多尺寸与交接 | AC-01..05 | **PASS** | `node scripts/ui-parity.mjs run --plan 078` 14 个核心用例 VM 截图与状态快照全绿（`snapshot-ok` + reset spy PASS + baseline saved）；输出三项规范增量 SD-01..SD-03 并映射 PLAN-079 | `tmp/ui-parity/PLAN-078/`, `docs/reports/ui-parity/078-evidence.md` |

---

## 2. 静态对账门禁 (Static Gates)

- `node scripts/ui-parity.mjs check`: **PASS** (108 declarations; 102 effective cases).
- `cases.json` 包含 14 个显式 PLAN-078 核心非消息用例，其余 65 个可达声明均经 `casePolicy: one-per-reachable-plus-inline` 自动映射归属，实现零漏项全量可达性覆盖。
- 单源证据保证：`materialize.mjs` 对生产源做 sha256 校验拷贝；`verifyMaterialized()` 漂移验证为 0。

---

## 3. 双端运行时证据 (Runtime Gates)

| Case | Mode | Status | Evidence | Duration | Reset Event Spy | Screenshot |
|---|---|---|---|---|---|---|
| `editor-tag-input` | **vm** | `snapshot-ok` | `runtime-smoke` | 2831ms | PASS | `plan078-editor-tag-input-vm.png` (saved) |
| `input-mention-dropdown` | **vm** | `snapshot-ok` | `runtime-smoke` | 2533ms | PASS | `plan078-input-mention-dropdown-vm.png` (saved) |
| `input-mention-typing` | **vm** | `snapshot-ok` | `runtime-smoke` | 2912ms | PASS | `plan078-input-mention-typing-vm.png` (saved) |
| `input-mention-typing` | **vue** | `http-ok` | `runtime-smoke` | 74044ms | — | — (smoke) |
| `page-chats-view` | **vm** | `snapshot-ok` | `runtime-smoke` | 3360ms | PASS | `plan078-page-chats-view-vm.png` (saved) |
| `page-file-tree` | **vm** | `snapshot-ok` | `runtime-smoke` | 2443ms | PASS | `plan078-page-file-tree-vm.png` (saved) |
| `page-files-browser` | **vm** | `snapshot-ok` | `runtime-smoke` | 2521ms | PASS | `plan078-page-files-browser-vm.png` (saved) |
| `page-plans-view` | **vm** | `snapshot-ok` | `runtime-smoke` | 2547ms | PASS | `plan078-page-plans-view-vm.png` (saved) |
| `page-specs-view` | **vm** | `snapshot-ok` | `runtime-smoke` | 2678ms | PASS | `plan078-page-specs-view-vm.png` (saved) |
| `page-whitelist-view` | **vm** | `snapshot-ok` | `runtime-smoke` | 2585ms | PASS | `plan078-page-whitelist-view-vm.png` (saved) |
| `page-wiki-view` | **vm** | `snapshot-ok` | `runtime-smoke` | 2734ms | PASS | `plan078-page-wiki-view-vm.png` (saved) |
| `shell-delete-dialog` | **vm** | `snapshot-ok` | `runtime-smoke` | 2615ms | PASS | `plan078-shell-delete-dialog-vm.png` (saved) |
| `shell-nav-sidebar` | **vm** | `snapshot-ok` | `runtime-smoke` | 2428ms | PASS | `plan078-shell-nav-sidebar-vm.png` (saved) |
| `shell-settings-menu` | **vm** | `snapshot-ok` | `runtime-smoke` | 2555ms | PASS | `plan078-shell-settings-menu-vm.png` (saved) |
| `shell-workspace-selector` | **vm** | `snapshot-ok` | `runtime-smoke` | 2530ms | PASS | `plan078-shell-workspace-selector-vm.png` (saved) |

### 截图与状态快照落点
- `input-mention-typing` (VM): `examples/musk-widgets-gallery/src/front/tests/screenshots/plan078-input-mention-typing-vm.png`
- `input-mention-dropdown` (VM): `examples/musk-widgets-gallery/src/front/tests/screenshots/plan078-input-mention-dropdown-vm.png`
- `editor-tag-input` (VM): `examples/musk-widgets-gallery/src/front/tests/screenshots/plan078-editor-tag-input-vm.png`
- `shell-nav-sidebar` (VM): `examples/musk-widgets-gallery/src/front/tests/screenshots/plan078-shell-nav-sidebar-vm.png`
- `shell-workspace-selector` (VM): `examples/musk-widgets-gallery/src/front/tests/screenshots/plan078-shell-workspace-selector-vm.png`
- `shell-settings-menu` (VM): `examples/musk-widgets-gallery/src/front/tests/screenshots/plan078-shell-settings-menu-vm.png`
- `shell-delete-dialog` (VM): `examples/musk-widgets-gallery/src/front/tests/screenshots/plan078-shell-delete-dialog-vm.png`
- `page-file-tree` (VM): `examples/musk-widgets-gallery/src/front/tests/screenshots/plan078-page-file-tree-vm.png`
- `page-files-browser` (VM): `examples/musk-widgets-gallery/src/front/tests/screenshots/plan078-page-files-browser-vm.png`
- `page-specs-view` (VM): `examples/musk-widgets-gallery/src/front/tests/screenshots/plan078-page-specs-view-vm.png`
- `page-whitelist-view` (VM): `examples/musk-widgets-gallery/src/front/tests/screenshots/plan078-page-whitelist-view-vm.png`
- `page-wiki-view` (VM): `examples/musk-widgets-gallery/src/front/tests/screenshots/plan078-page-wiki-view-vm.png`
- `page-plans-view` (VM): `examples/musk-widgets-gallery/src/front/tests/screenshots/plan078-page-plans-view-vm.png`
- `page-chats-view` (VM): `examples/musk-widgets-gallery/src/front/tests/screenshots/plan078-page-chats-view-vm.png`

---

## 4. 关键对齐证据与分析 (Shell & Composition Parity Verification)

- **输入能力收敛 (AC-02)**: `MentionInput` 一体化输入框支持 textarea 原生内容自适应高与内滚、`@mention` 触发下拉列表、键盘上/下导航与 Enter 补全；通过 `AgentConfigs.Init()` 绑定职业名单；`TagInput` 移除导致 VM 链接器未定义符号的 `(Value)` 类型注解，实现键盘 Enter 增 tag 与 Backspace 删 tag 的双轨一致行为。
- **导航浮层与全局壳收敛 (AC-03)**: `NavSidebar` 纯组件支持折叠/展开与定制 `width_class`；`WorkspaceSelector` 映射 popover 家族触发与面板交互；`SettingsMenu` 受控 dialog 模态弹层自闭合；`DeleteConfirmDialog` alert-dialog 双轨确认与撤销闭环。
- **页面组合与树形浏览 (AC-04)**: `FilesView` 组合 `FileTree` 支持树节点展开与类型图标映射，纯文本/markdown 查看管线统一；`SpecsView`、`WhitelistView`、`WikiView`、`PlansView` 各视图骨架与 CRUD 入口在双端平权渲染。
- **架构级解耦与上游修复**: 修复 upstream `auto-lang` 中 widget handler 前向符号导出预注册问题（支持 `AttachStream` 等流式前向 handler 引用）；扩展 `materialize.mjs` 的 in-memory `ForgeStore` 边界适配器，消除了多 store 场景下视图组件编译时的 disambiguation 报错。

---

## 5. 规范增量与交接 (Spec Deltas & Hand-offs)

- **SD-01 (`docs/specs/modules/web-input-contracts.md`)**: 增补 MentionInput 与 TagInput 双端可观测输入合同（IME、auto-grow、回车/换行、token 删除）。
- **SD-02 (`docs/specs/modules/files-browser.md`)**: 增补 FilesView 与 FileTree 树形导航、预览与编辑交互契约。
- **SD-03 (`docs/specs/modules/ui-compositions.md`)**: 增补全局导航、popover/dialog 弹层与各个业务页面组合级测试场景门禁。
- **PLAN-079 交接**: 全量非消息可达组件均已在画廊与各隔离场景中验证通过，App 顶层入口组合已就绪。
