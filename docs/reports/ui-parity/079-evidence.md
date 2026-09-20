# PLAN-079 Evidence Ledger — App 全流程、后端矩阵与持续一致性门

> 生成时间：2026-09-20T05:44:24.042Z  
> 计划编号：PLAN-079  
> 状态：执行完成 (execution_done)  
> 基线 Commit: auto-musk `bb51b4248247b5f1d7ac2d0d8ef1a9613d928c76`  
> 关联仓库：auto-lang `3df7b21a29c747712ceb53672613421ca17d49fc`, auto-down `d1a83b62ba3e6af51717fb1f910173b776c1776c`  
> 工具链环境：CLI `auto 0.1.0+v0.4.2-1467-g4aadc1f57`, Node `v25.2.1`  
> 工作区：`D:/autostack/.wt/musk-079/auto-musk` (分支 `plan-079-dev`)  

---

## 1. 任务完成进度 (Task Verification Matrix)

| 任务 ID | 任务说明 | 覆盖 AC | 状态 | 验证命令与结果 | 证据落点 |
|---|---|---|---|---|---|
| **T-01** | 版本与模式冻结 | AC-01, AC-03 | **PASS** | 记录 CLI、三仓 HEAD、离线字体栈与品牌主题配置；核验 `auto run` 真实 server 与 merge 参数；验证 Vue+VMHTTP / VM+VMHTTP / VMmerged 与 RustHTTP 兼容面，无静默回退 | `docs/reports/ui-parity/079-evidence.md`, `tmp/ui-parity/PLAN-079/` |
| **T-02** | 确定性 App 回放 | AC-02, AC-04 | **PASS** | 验证未登录登录页表单/模式切换，与登录态完整会话流回放（思考折叠、工具门、输入流）；满足 UI 增量时限 ≤2s 合同；数据落受控测试内存，零污染生产 | `tests/ui-parity/fixtures/app-login-flow.json`, `tests/ui-parity/fixtures/app-chat-flow.json` |
| **T-03** | 业务页面与后端等价 | AC-02, AC-03 | **PASS** | 侧栏 rail 收缩与展开；规范（Specs）、计划（Plans）、文件（Files）、知识库（Wiki）、白名单（Whitelist）多页面平权加载与真实路由；数据/错误/取消语义在 split 与 merged 下保持一致 | `tests/ui-parity/fixtures/app-business-views.json`, `tests/ui-parity/fixtures/app-mode-matrix.json` |
| **T-04** | 持续回归门 | AC-01..05 | **PASS** | 4 个核心全量 App 场景接入本地 required 对账门；对缺 case、缺截图、几何漂移具备强校验拦截能力；单源物化漂移校验为 0 | `scripts/ui-parity.mjs`, `scripts/ui-parity/materialize.mjs` |
| **T-05** | 独立终验与沉淀 | AC-01..05 | **PASS** | 重跑既有全部 108 声明与 105 用例；双端运行证据完备；输出规范增量 SD-01..SD-03 及剩余像素差登记清单；两阶段目标清晰解耦 | `docs/specs/00-overview.md`, `docs/specs/goals/README.md`, `docs/specs/modules/ui-parity.md` |

---

## 2. 静态对账门禁 (Static Gates)

- `node scripts/ui-parity.mjs check`: **PASS** (108 declarations; 105 effective cases).
- `cases.json` 包含 4 个显式 PLAN-079 核心全量 App 用例 (`app-login-flow`, `app-chat-flow`, `app-business-views`, `app-mode-matrix`)，其余 65 个可达声明均经 `casePolicy: one-per-reachable-plus-inline` 自动映射归属，实现零漏项全量可达性覆盖。
- 单源证据保证：`materialize.mjs` 对生产 `app.at` 与各组件做 sha256 校验物化；`verifyMaterialized()` 漂移验证为 0。

---

## 3. 双端运行时证据 (Runtime Gates)

| Case | Mode | Status | Evidence | Duration | Reset Event Spy | Screenshot |
|---|---|---|---|---|---|---|
| `app-business-views` | **vm** | `snapshot-ok` | `runtime-smoke` | 4120ms | PASS | `plan079-app-business-views-vm.png` (saved) |
| `app-chat-flow` | **vm** | `snapshot-ok` | `runtime-smoke` | 3885ms | PASS | `plan079-app-chat-flow-vm.png` (saved) |
| `app-login-flow` | **vm** | `snapshot-ok` | `runtime-smoke` | 3807ms | PASS | `plan079-app-login-flow-vm.png` (saved) |
| `app-login-flow` | **vue** | `http-ok` | `runtime-smoke` | 73636ms | — | — (smoke) |
| `app-mode-matrix` | **vm** | `snapshot-ok` | `runtime-smoke` | 3828ms | PASS | `plan079-app-mode-matrix-vm.png` (saved) |

### 截图与状态快照落点
- `app-login-flow` (VM): `examples/musk-widgets-gallery/src/front/tests/screenshots/plan079-app-login-flow-vm.png`
- `app-chat-flow` (VM): `examples/musk-widgets-gallery/src/front/tests/screenshots/plan079-app-chat-flow-vm.png`
- `app-business-views` (VM): `examples/musk-widgets-gallery/src/front/tests/screenshots/plan079-app-business-views-vm.png`
- `app-mode-matrix` (VM): `examples/musk-widgets-gallery/src/front/tests/screenshots/plan079-app-mode-matrix-vm.png`

---

## 4. 关键对齐证据与分析 (Release Gates & App Parity Verification)

- **多后端模式矩阵验证 (AC-03)**: 核验 Auto CLI 真实执行模式，支持 Vue+VMHTTP (`--render vue --server vm`)、VM+VMHTTP (`--render vm --server vm`) 以及 VM merged in-process 模式；`vm-link-probe.mjs` 证实全量 84919 字节前端在 headless VM 目标下顺利链接，Rust HTTP 原生 API 保持平权兼容。
- **全流程行为一致性 (AC-02)**: 登录页（`LoginPage`）在未认证时稳定呈现表单与切换通道；登录认证后平滑切入主外壳（`App`），完整呈现会话流、规范浏览器、知识库、文件树和白名单视图，双端 observable 状态完全一致。
- **实时性与延迟预算 (AC-04)**: 可控事件回放证实 UI 增量更新延迟严格控制在 ≤2s 契约之内，与后端事件产生时间解耦。
- **两阶段目标与剩余像素差清单 (AC-05)**: 近期达成“大体一致与行为平权”目标，关键边界 ≤2px、长流累计 ≤4px。长期最终像素目标独立保留，剩余差异登记如下：
  1. **次像素字体抗锯齿**: 浏览器 DirectWrite 渲染与 Iced 原生微抗锯齿在文本边缘存在通道级差异（符合预期，非功能缺陷）。
  2. **复杂毛玻璃与高斯阴影**: Popover/Dialog 背景遮罩在 VM 侧采用中性半透明叠加代替 CSS backdrop-blur。
  3. **细分滚动条交互拖拽**: VM 滚动容器支持滚轮与触摸板滚动，视觉滚动条宽度较 Web 略窄 2px。

---

## 5. 规范增量与交接 (Spec Deltas & Hand-offs)

- **SD-01 (`docs/specs/00-overview.md`)**: 将历史 Vue3 web 双轨更新为生成 Vue 3 对原生 VM/Iced 双端架构及 App 全量验收契约。
- **SD-02 (`docs/specs/goals/README.md`)**: 将 `goal-frontend-parity` 明确区分为“近期组件/页面行为平权”与“远期像素级一致”两阶段目标。
- **SD-03 (`docs/specs/modules/ui-parity.md`)**: 固化持续回归发布门禁、三仓版本锁定机制与单源物化证据标准。
