# PLAN-076 Evidence Ledger — AutoDown 统一引擎三模式接入与差异关闭

> 生成时间：2026-09-19T12:29:40.924Z  
> 计划编号：PLAN-076  
> 状态：执行中 (executing)  
> 基线 Commit: `3a297ac1c448e7bd6b784346069602b0521bbe9c`  
> 工作区：`D:/autostack/.wt/musk-076/auto-musk` (分支 `plan-076-dev`)  
> 关联仓库：auto-lang `d256682`, auto-down `84c9897`

---

## 1. 任务完成进度 (Task Verification Matrix)

| 任务 ID | 任务说明 | 覆盖 AC | 状态 | 验证命令与结果 | 证据落点 |
|---|---|---|---|---|---|
| **T-01** | 消费矩阵与接口探针 | AC-01, AC-05 | **PASS** | 静态扫描全量 21 处调用点；比对上游 `autodown`/`autodown_editor` 原生接口；完成 7 项差异定责与转接方案设计 | `docs/reports/ui-parity/076-engine-map.md`, `tests/ui-parity/cases.json` |
| **T-02** | 宿主统一转接 | AC-01 | **PASS** | `renderer.vm.at` 接入原生 `autodown`，补齐 `MarkdownRender`；`specs_editors.at` 接入 `autodown_editor`，全量消除纯文本 fallback | `src/front/ports/renderer.vm.at`, `src/front/specs_editors.at` |
| **T-03** | 引擎侧三模式修复 | AC-02, AC-03, AC-04 | 进行中 | 验证 auto-down 块节奏 12px、暗色 token、流式末尾保护与编辑事件 | `auto-down` 仓库提交与用例 |
| **T-04** | 版本消费与覆盖清退 | AC-02, AC-05 | 待执行 | 刷新 vendor 并校验 hash；清退 `inject_styles.web-only.ts` 中的深色与间距 CSS 覆盖 | `src/front/inject_styles.web-only.ts`, vendor |
| **T-05** | 三模式对拍与交接 | AC-01..05 | 进行中 | `node scripts/ui-parity.mjs run --plan 076` VM 4 个用例全绿并捕获 baseline 截图 | `tmp/ui-parity/PLAN-076/` |

---

## 2. 静态对账门禁 (Static Gates)

- `node scripts/ui-parity.mjs check`: **PASS** (63 declarations; 58 effective cases).
- `docs/reports/ui-parity/076-engine-map.md`: 21 处调用点及双轨转接方案全量映射完成。

---

## 3. 双端运行时证据 (Runtime Gates)

| Case | Mode | Status | Evidence | Duration | Reset Event Spy | Screenshot |
|---|---|---|---|---|---|---|
| `autodown-editor-leaf` | **vm** | `snapshot-ok` | `runtime-smoke` | 3326ms | PASS | `plan076-autodown-editor-leaf-vm.png` (saved) |
| `autodown-raw-preview` | **vm** | `snapshot-ok` | `runtime-smoke` | 3430ms | PASS | `plan076-autodown-raw-preview-vm.png` (saved) |
| `autodown-view-standalone` | **vm** | `snapshot-ok` | `runtime-smoke` | 2383ms | PASS | `plan076-autodown-view-standalone-vm.png` (saved) |
| `chat-text-block` | **vm** | `snapshot-ok` | `runtime-smoke` | 3151ms | PASS | `plan076-chat-text-block-vm.png` (saved) |

### 截图与状态快照落点
- `chat-text-block` (VM): `examples/musk-widgets-gallery/src/front/tests/screenshots/plan076-chat-text-block-vm.png`
- `autodown-editor-leaf` (VM): `examples/musk-widgets-gallery/src/front/tests/screenshots/plan076-autodown-editor-leaf-vm.png`
- `autodown-view-standalone` (VM): `examples/musk-widgets-gallery/src/front/tests/screenshots/plan076-autodown-view-standalone-vm.png`
- `autodown-raw-preview` (VM): `examples/musk-widgets-gallery/src/front/tests/screenshots/plan076-autodown-raw-preview-vm.png`

---

## 4. 关键对齐证据与分析 (Engine Parity Verification)

- **消除纯文本降级**: `renderer.vm.at` 由旧 `div.vm-markdown-plain` 全面替换为原生 `autodown { content, streaming }`，VM snapshot 证实生成标题、段落内联加粗/链接/代码、任务列表与围栏代码块结构，完全淘汰裸文本预排。
- **补齐 MarkdownRender 端口**: `renderer.vm.at` 声明同名 `widget MarkdownRender`，使 `files_view.at` 与 `raw_preview.at` 在 VM 轨获得平权渲染。
- **接入 autodown_editor**: `specs_editors.at` 中 `AutoDownEditor` textarea stub 替换为原生 `autodown_editor`，支持输入事件自持与双端富文本编辑。
