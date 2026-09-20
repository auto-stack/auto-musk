# ui-parity — 前端组件对齐契约与双端 Gallery 基线（PLAN-074 SD-01）

> 来源：PLAN-074 r1（reviewed 4cbd5b5 / delivery 于 main）。
> 核心交付：同源 Gallery 基线、静态目录覆盖门、双端 runtime-smoke 证据与分阶段定责。

## 契约

1. **同源消费原则**：
   - 双端 Gallery（`examples/musk-widgets-gallery`）必须直接挂载生产源码组件（`src/front/*.at`），严禁克隆业务模板或另写假 UI。
   - 数据与副作用隔离通过专用 Store 适配器（如 `forge_store.at` 内存事件记录）和确定性 fixtures（`tests/ui-parity/fixtures/*.json`）完成，禁止触碰真实工作区、网络 API 或审批流。
2. **全量静态门**：
   - 清单必须覆盖全部前端声明与内联分支；`node scripts/ui-parity.mjs check` 必须为 0 退出（0 missing items）。
   - 当前基准：62 个声明单元（54 个可达单元、8 个不可达历史单元、8 个端口变体组、57 个有效测试用例）。
3. **双端运行时证据门**：
   - 任何声称可达的组件用例必须具备双端运行时凭据或明确缺件失败记录：
     - **Vue 端**：工程生成、依赖安装、AutoVM 后端与 Vite 前端在有界时间内启动并返回 `status: http-ok`。
     - **VM 端**：AutoUI MCP 成功挂载并在 `autoui_snapshot` 中呈现对应 UI 节点（`status: snapshot-ok`）；Reset fixture 按钮经 `autoui_action` 触发后事件 spy 计数递增（`reset_event_spy: PASS`）；基线截图成功保存。
   - 缺少任一端证据或出现未分类回归时，runner 必须非零退出，报告不得伪装通过。
4. **渐进式路线与责任归属**：
   - **PLAN-074**（基线与同源 Gallery）：建立全量清单、同源双实例、副作用隔离、双端 runtime-smoke。
   - **PLAN-075**（默认样式与几何）：Tailwind/CSS 规则收敛至 VM 原生布局支持，消除 `self-stretch` 降级，达成关键边界 ≤2px。
   - **PLAN-076**（AutoDown 引擎与编辑器）：对齐 Markdown 流式渲染、代码块高亮及 Bubble/Slash 菜单。
   - **PLAN-077**（卡片交互与流式）：ChatMessage、ToolBlock、Thinking、GateCard 展开/折叠与状态流转。
   - **PLAN-078**（导航与宿主 App）：App 整体外壳、Workspace 切换、会话列表与全局状态对齐。
   - **PLAN-079**（发布门与全量套件）：自动化 App 全流程回放、多后端模式矩阵（Vue+VMHTTP / VM+VMHTTP / VMmerged / RustHTTP）、持续回归门固化与两阶段像素目标沉淀。
5. **持续回归与发布门禁规则（PLAN-079）**：
   - **静态对账门**：`node scripts/ui-parity.mjs check` 必须覆盖 100% 声明（108 声明单元，105 用例），出现未分类、无 fixture 或重复 ID 时非零失败。
   - **运行时发布门**：`node scripts/ui-parity.mjs run --plan 079` 双端（Vue http-ok + VM snapshot-ok）100% 通过；缺失截图、状态漂移或 MCP 丢失必须显式置失败。
   - **三仓版本锁定**：发布基线必须同时绑定 auto-musk、auto-lang、auto-down 三仓 HEAD SHA 及 CLI 工具链版本，禁止浮动版本验收。

## 关联实现与工具

- Gallery 入口：`examples/musk-widgets-gallery/pac.at`、`src/front/app.at`。
- 自动化编排：`scripts/ui-parity.mjs`（`check`、`run`、`report` 命令接口）。
- 测试夹具：`tests/ui-parity/cases.json`、`tests/ui-parity/fixtures/*.json`。
- 差异与基线报告：`docs/reports/ui-parity/074-baseline.md`、`docs/reports/ui-parity/074-evidence.md`。
- 默认样式契约与对账映射：`docs/specs/modules/ui-default-styles.md`、`docs/reports/ui-parity/075-default-style-map.md`、`docs/reports/ui-parity/075-evidence.md`。
- AutoDown 引擎与编辑器对账：`docs/reports/ui-parity/076-engine-map.md`、`docs/reports/ui-parity/076-evidence.md`。
- 消息卡片与流式交互对账：`docs/reports/ui-parity/077-evidence.md`。
- 全局壳与组合对账：`docs/reports/ui-parity/078-evidence.md`。
- App 全流程与发布门禁证据：`docs/reports/ui-parity/079-evidence.md`。
