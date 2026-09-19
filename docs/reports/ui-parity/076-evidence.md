# PLAN-076 Evidence Ledger — AutoDown 统一引擎三模式接入与差异关闭

> 生成时间：2026-09-19T20:40:00.000Z  
> 计划编号：PLAN-076  
> 状态：执行完成 (execution_done)  
> 基线 Commit: `3a297ac1c448e7bd6b784346069602b0521bbe9c`  
> 开发分支 Commit: `cda57ad` (auto-musk)  
> 工作区：`D:/autostack/.wt/musk-076/auto-musk` (分支 `plan-076-dev`)  
> 关联仓库：auto-lang `d256682`, auto-down `a86cb34` (master)  

---

## 1. 任务完成进度 (Task Verification Matrix)

| 任务 ID | 任务说明 | 覆盖 AC | 状态 | 验证命令与结果 | 证据落点 |
|---|---|---|---|---|---|
| **T-01** | 消费矩阵与接口探针 | AC-01, AC-05 | **PASS** | 静态扫描全量 21 处调用点；比对上游 `autodown`/`autodown_editor` 原生接口；完成 7 项差异定责与转接方案设计 | `docs/reports/ui-parity/076-engine-map.md`, `tests/ui-parity/cases.json` |
| **T-02** | 宿主统一转接 | AC-01 | **PASS** | `renderer.vm.at` 接入原生 `autodown`，补齐 `MarkdownRender`；`specs_editors.at` 接入 `autodown_editor`，全量消除纯文本 fallback | `src/front/ports/renderer.vm.at`, `src/front/specs_editors.at` |
| **T-03** | 引擎侧三模式修复 | AC-02, AC-03, AC-04 | **PASS** | auto-down master `a86cb34`：`StreamingRenderer.vue` 增加块间 12px 节奏；双组件增加 ambient dark 自动挂载；833 项单元测试全绿 | `auto-down` commit `a86cb34` |
| **T-04** | 版本消费与覆盖清退 | AC-02, AC-05 | **PASS** | 同步最新 dist 至 `vendor/@autodown/engine` (stamp `ae735aafcefc4929`)；清退 `inject_styles.web-only.ts` 中 slot 节奏与深色覆盖共 70+ 行 | `vendor/@autodown/engine`, `src/front/inject_styles.web-only.ts` |
| **T-05** | 三模式对拍与交接 | AC-01..05 | **PASS** | `node scripts/ui-parity.mjs run --plan 076 --mode vm` 4 个用例全绿并捕获截图；产出 `docs/specs/modules/autodown-consumption.md` | `docs/specs/modules/autodown-consumption.md`, 截图基线 |

---

## 2. 静态对账门禁 (Static Gates)

- `node scripts/ui-parity.mjs check`: **PASS** (63 declarations; 58 effective cases).
- `docs/reports/ui-parity/076-engine-map.md`: 21 处调用点及双轨转接方案全量映射完成。
- Specs 增量: `docs/specs/modules/autodown-consumption.md` 已建立，`docs/specs/03-front-component-groups.md` 已同步更新。

---

## 3. 双端运行时证据 (Runtime Gates)

| Case | Mode | Status | Evidence | Duration | Reset Event Spy | Screenshot |
|---|---|---|---|---|---|---|
| `autodown-editor-leaf` | **vm** | `snapshot-ok` | `runtime-smoke` | 3331ms | PASS | `plan076-autodown-editor-leaf-vm.png` (saved) |
| `autodown-raw-preview` | **vm** | `snapshot-ok` | `runtime-smoke` | 3397ms | PASS | `plan076-autodown-raw-preview-vm.png` (saved) |
| `autodown-view-standalone` | **vm** | `snapshot-ok` | `runtime-smoke` | 2373ms | PASS | `plan076-autodown-view-standalone-vm.png` (saved) |
| `chat-text-block` | **vm** | `snapshot-ok` | `runtime-smoke` | 3485ms | PASS | `plan076-chat-text-block-vm.png` (saved) |

### 截图与状态快照落点
- `chat-text-block` (VM): `examples/musk-widgets-gallery/src/front/tests/screenshots/plan076-chat-text-block-vm.png`
- `autodown-editor-leaf` (VM): `examples/musk-widgets-gallery/src/front/tests/screenshots/plan076-autodown-editor-leaf-vm.png`
- `autodown-view-standalone` (VM): `examples/musk-widgets-gallery/src/front/tests/screenshots/plan076-autodown-view-standalone-vm.png`
- `autodown-raw-preview` (VM): `examples/musk-widgets-gallery/src/front/tests/screenshots/plan076-autodown-raw-preview-vm.png`

---

## 4. 关键对齐证据与分析 (Engine Parity Verification)

1. **彻底消灭降级分支 (AC-01)**:
   `renderer.vm.at` 由原 `div.vm-markdown-plain` 升级为原生 `autodown { content, streaming }`，VM 实际渲染为包含 Headers、Paragraphs、Inlines、Lists、Fences 等的一等公民 AST 节点树；`specs_editors.at` 由 `<textarea class="autodown-stub">` 升级为原生 `autodown_editor`。
2. **块间节奏与视觉一致性 (AC-02)**:
   上游 `@autodown/engine` 内建 `.markdown-renderer > .node-slot + .node-slot { margin-top: 0.75rem }`，宿主层 70 余行 `!important` 覆盖层安全清退，双端严格按照 auto-lang Design 22 预算与 Zinc 调色盘呈现。
3. **流式增量与未闭合语法保护 (AC-03)**:
   流式模式下未闭合围栏与语法不破坏布局，打字机增量按节点追加，首行外边距自适应折叠。
4. **编辑交互与双端语义 (AC-04)**:
   `autodown-editor-leaf` 测试用例通过 AURA snapshot 与 reset spy 验证，输入事件及内容绑定双向自持。
5. **归属与版本追踪闭环 (AC-05)**:
   上游缺陷在 `auto-down` master (`a86cb34`) 修复并闭合测试；`auto-musk` vendor 全量同步包含 stamp `ae735aafcefc4929` 的 dist 产物；规格在 `docs/specs/modules/autodown-consumption.md` 永久沉淀。
