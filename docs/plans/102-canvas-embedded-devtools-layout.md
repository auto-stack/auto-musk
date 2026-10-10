---
plan_id: PLAN-102
status: executing
feature_name: Canvas 内嵌交互预览与 DevTools 可停靠布局
author: [agent]
created_at: 2026-10-10T05:20:55Z
updated_at: 2026-10-10T11:30:00Z
plan_revision: 1
current_step: 1
total_steps: 6
supersedes_spec_components:
  - docs/specs/modules/canvas-embed.md
  - docs/specs/modules/app-canvas.md
  - docs/specs/modules/app-studio-ui.md
  - docs/specs/modules/autoui-canvas-runtime.md
new_spec_components: []
touched_goals:
  - goal-frontend-parity
---

# PLAN-102 — Canvas 内嵌交互预览与 DevTools 可停靠布局

## 0. 变更摘要

将 Canvas 从「独立运行窗 + 宿主截图检查面」升级为默认在 Musk 内显示、可以操作的应用预览。提供「内嵌 / 独立窗口」切换；保留「操作应用 / 检查元素」两种输入用途。把 Canvas 左侧结构/源码工具栏改为可停靠的 DevTools：左侧、右侧、底部、隐藏，分隔条可拖动，正常聊天与应用设计模式共用布局组件。

本阶段只建立执行合同，不实现代码，不修改权威 Specs，不宣称运行验证通过。执行前先完成有界能力探针，选择可持续出帧且能回传输入的宿主呈现方案。

## 1. 目标

- 应用在 Musk 内容框中显示和操作；默认无需看到第二个小窗口。
- 切换独立窗口时保持同一个应用会话、模型状态和 generation；返回内嵌继续操作。
- DevTools 可左右/上下排布或隐藏；工具区与预览区之间可拖动调整尺寸。
- 提升工具区的信息密度、对齐、滚动和主题一致性；保留树选中、源码定位、Agent 高亮和设计上下文能力。
- Vue/VM 前端与 RustHTTP/VMHTTP 后端行为一致，不能只交付浏览器端。

范围：auto-musk Canvas 前端、会话控制、双后端桥接与测试。若需要运行时能力，限定修改 auto-lang 的 Canvas 呈现/输入/布局接缝；不以本计划全面重构 AutoUI 全局 DevTools。

非目标：OS HWND re-parent、任意外部程序嵌入、跨网络远程桌面、视频流平台、复制 Chrome 全部调试功能、重构聊天组件或生成引擎。

## 2. 架构方案

### 2.1 两个正交开关

1. 呈现模式 `embedded | window`：控制应用显示在哪；不等于 start/stop。
2. 输入模式 `operate | inspect`：操作模式将输入发给应用；检查模式使用现有 pick/overlay/source 通道。默认 operate，工具栏明确显示当前模式。

DevTools 停靠 `left | right | bottom | hidden` 独立于上述开关。隐藏后保留重新打开入口，不停止应用，不丢弃选择/页签。

### 2.2 呈现方案决策门

T-01 优先评估 auto-lang 已有 `desktop_render: remote` 的 headless iced / DisplayList v2 路径与宿主消费面，复用真实组件布局、输入、聚焦和 IME。已有能力属于桌面协议，不直接假设 Canvas 已能接入；验证 VM 和 Vue 的嵌入接缝、会话隔离、是否仍需额外可见宿主窗。

备选为保持独立渲染进程、用可隐藏但持续出帧的 surface 向 Musk 输送帧，并建立真实输入回传。不得仅调用最小化或移到屏幕外就判完成；不得让静止截图冒充交互内嵌。选择只由双端探针证据决定，T-01 产出对比表、接口草案、依赖修改点及取舍。

`CanvasManager` 继续管理单会话、generation、重启、收割；呈现切换不调用 start 的替换语义。新控制请求绑定 workspace + generation，陈旧请求不能控制新应用。前端状态以 status 实际确认值为准；请求中禁重复点击，失败回显并保持可恢复入口。

### 2.3 共用布局

新增 `CanvasWorkbench`（名称暂定）组合 CanvasCanvasColumn 与 CanvasStructureColumn；CanvasPanel / CanvasStudioSlot 仅配置宿主。新建布局纯函数和双端指针适配，业务偏好仍由 CanvasStore 持有，不在 TS 另建业务状态。正常模式外框取消不可调整的固定 520px 宽，增加与聊天区之间的外部分隔条；与内部 DevTools 分隔条分别管理尺寸。

## 3. 技术栈

- .at 单源视图/store；Vue 生成目标、VM/Iced 原生目标；沿用 pac.at 主题 token 和 zh/en i18n。
- Rust CanvasManager / SessionHandle / axum routes；VMHTTP .at 路由 + vm_backend 宿主桥。
- AutoUI 现有 remote/headless 或持续帧 surface、真实指针/键盘/IME 能力：接入形式在 T-01 定案。
- Node 探针 + Rust 合同测试 + 双端实机截图/交互收据。只改真源，不手改生成产物。

## 4. 需求分析与背景调查

### 4.1 授权与工作区规则

用户于 2026-10-10 授权分析内嵌/窗口切换、DevTools UI、方向切换及拖动分隔条，并建立计划。本轮授权为调查与计划编写，尚未要求执行实现、合并或部署；未指定预算、并行 agent 或自动续跑。计划文档按 AGENTS.md 共享状态例外写主检出 docs/plans；代码实现必须进入 `D:/autostack/.wt/musk-102/auto-musk`、分支 `plan-102-dev`。如需改 auto-lang，使用同组兄弟 worktree、分支 `auto-musk-dev`，禁止 junction/symlink；依赖改动经消费验证后尽快按约定快进合回并清理。

扫描活跃计划：仅 PLAN-097 仍为 executing，内容为演示/证据，不覆盖本项用户新需求，不覆盖其文档。PLAN-099 已归档，交付的是截图镜像与相对 URL 修复，不代表完整交互内嵌。

### 4.2 当前事实与根因

- `src/front/canvas_panel.at`：CanvasPanel 固定 520px、结构列 170px；CanvasStudioSlot 按断点固定 200/220px；没有可停靠/可拖动的共用布局。
- `src/front/canvas_structure.at`：结构/源码页签，树节点纵向堆放 kind/label/循环值；窄列与长文本容易占用高度，需调整密度和截断。
- `src/front/canvas_panel.at` CanvasCanvasColumn：预览是 img + mouse-area + overlay；FrameClick 走 canvasPickAt。`canvas_web.ts` 的点击委托同样 POST pick。它是元素检查入口，不能直接操作应用。
- `backend/crates/musk/src/canvas/session.rs` SessionHandle::spawn：`auto run --render=vm`，480x680 原生窗；MUSK_CANVAS_WINDOW_POS 只决定独立窗位置。
- `canvas/manager.rs` 看门狗消费截图；最小化的 Screenshot skipped 不算失败，但无首帧有超时护栏。隐藏窗口不是已证明可行的连续出帧方案。
- `canvas-embed.md` 定义原生窗为帧源、宿主为镜像，记录 iced 无 re-parent、动态帧快照语义；本计划改变呈现和输入合同，需要明确 Spec 增量。
- `app-studio-ui.md` 记录 VM 动态 style 字段限制、360x510 静态回退和 studio 快照 rect 全零；动态拖动必须以运行时真实几何验证，不能仅生成 CSS 或读取该旧仪表判绿。
- 另有 AutoUI 原生 DevTools：auto-lang `ui/session.rs` DevToolsState 与 `ui/iced/renderer.rs`，现有外部/内部 divider 拖动，计算偏向右侧停靠。这不是 Musk 左侧 CanvasStructureColumn。本计划主对象是用户所指 Canvas 工作台工具区；T-01 实拍标注两者，若出现原生工具重复露出，Canvas 模式仅显示一套工具入口。全局调试器重做另立范围。
- auto-lang `ui/overview.md` 已记录 PLAN-683 remote 渲染与 PLAN-690 IME/hover/光标/键盘回传；`ui/desktop_protocol/remote.rs` 是桌面远程镜像入口。相关能力值得复用，但不等于已具备 Musk 内嵌 API。

### 4.3 调查基线

代码基线：auto-musk `753731c174bd277c914538b1a61c00bb914826a8`；auto-lang `c212e1c7a71b0bfea6972b0d1c9b86d78eeb794b`。执行阶段重新核对差异。

权威来源 SHA256：

| 来源 | hash |
|---|---|
| docs/specs/modules/canvas-embed.md | EC007C1D6FC37D13B165F8ED39ACD9EDDB9A89B25C6CECF8E7EA24FC0785EDBC |
| docs/specs/modules/app-studio-ui.md | AB84903AD70F77027C579B1E719D08DE11E4C2527A1399246824B1FE41768A98 |
| docs/specs/modules/app-canvas.md | A2E5BD4048363783212CE16BF2BD9EE679D7E9D532B6A8E65D52707D98DEF47D |
| docs/specs/modules/autoui-canvas-runtime.md | 55CE73AB1FA02F57FE728130070E3041783C3D84E202E2D0CDA51B1271561D95 |

另读 `docs/specs/00-overview.md`、`modules/ui-parity.md`、`goals/README.md`；上游权威 `D:/autostack/auto-lang/docs/specs/auto-lang/ui/overview.md`；设计背景 `D:/autostack/auto-lang/docs/design/autoui/rq-remote-renderer.md`。后者用于理解接缝，不覆盖权威规范。

## 5. 详细设计

### 5.1 窗口与输入

- 默认 embedded；应用首次出帧直接在 Musk 内容区出现，外部 app 窗不露出抢焦点。window 模式入口放在预览顶部工具栏，带文字或 tooltip。
- 模式切换保持会话身份及应用数据；如果当前运行时只能重启才切换，视为探针未通过，不能静默清空表单来降低验收门槛。
- 操作态回传点击、拖动、滚轮、文本输入、中文 IME、聚焦、常见编辑按键及应用声明的键盘导航。检查态只点选，不触发按钮副作用。被动 overlay 不截获输入。
- 内容坐标统一使用真实内容矩形和帧/surface 尺寸，包含缩放、DPI、工具区拖动、底部停靠后的变化；留白不回传目标输入。
- 断开/重启/无帧时明确禁用输入，显示状态和重试；迟到帧/输入不能污染下一 generation。

### 5.2 工具栏和 DevTools

预览顶部：应用名与状态、操作/检查、适应/100%、呈现切换、DevTools 停靠菜单、停止。低宽度下次要操作收入菜单，主要操作命中区域至少 32px。

DevTools 顶部使用紧凑页签「结构 / 源码」，标题区不重复大块应用信息。树行优先一行：展开按钮、kind、短 label、实例提示；长文本截断，完整值用 tooltip，选中使用主题 primary 的细条/浅底。深层节点支持横向滚动或受控缩进，不能把文字压到零宽。源码正文与树分别滚动，工具条固定；空态/错误态沿用现有语义，统一边框、文字层级和 hover/focus。

停靠图示（对话区由宿主安排）：

```text
left:   [ DevTools ] | [     Canvas     ]
right:  [     Canvas     ] | [ DevTools ]
bottom: [          Canvas              ]
        --------------------------------
        [          DevTools            ]
hidden: [          Canvas              ]  ← 工具栏可重新打开
```

### 5.3 拖动、响应式和偏好

- divider 可见线 1px、实际命中约 6px，横向使用 col-resize、底部使用 row-resize；hover/拖动反馈沿用主题。pointer capture 或等效 VM 窗口事件保证越界继续拖动；释放、失焦、切换方向、卸载均结束拖动。
- 初始左右工具区 240px，最小 180px；底部初始 240px，最小 140px；Canvas 最小内容区 240x180。尺寸按实际可用内容区域钳制，不能使用整个窗口宽度替代嵌入容器宽度。
- 正常聊天模式外框初始约 640px，并依据可用空间保证聊天区至少 320px；宽度不足进入页签单区。默认数字是初值，执行阶段允许在约束不变前提下依据双端实拍微调并记录。
- 狭窄容器不足以满足两侧最小尺寸时，降为画布/工具单区页签，保存用户停靠和尺寸偏好；变宽恢复原方向。优先看工作台容器，不能只看 cv_win_w。
- 左/右共用或各自记忆宽度须在 T-01 定案；至少水平宽度与底部高度分别记忆。隐藏/恢复保留方向、页签、选择和树折叠。
- 偏好按 workspace + 前端轨保存：停靠、水平宽度、底部高度、正常模式外框宽度；轮询不覆写。工作区切换读取对应偏好并重新钳制。呈现切换首次默认内嵌；不持久化运行中的 generation/input 焦点。
- divider 支持键盘调整（方向键每次 10px，Home 恢复初值），有可读名称、方向和当前尺寸；Escape 取消本次拖动恢复起始尺寸，不停止应用。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/modules/canvas-embed.md | 固定原生窗帧源+镜像 → 可持续呈现的默认内嵌与无重启窗口切换；保留不可 re-parent 约束 | 移除独立小窗依赖，记录最终证据支持的呈现机制 | AC-01,02,07 |
| SD-02 | modify | docs/specs/modules/app-canvas.md | start/stop/status → 增加带身份守卫的呈现控制/输入消费，隔离与收割不变 | 防止切换重建会话和陈旧输入串会话 | AC-02,03,07 |
| SD-03 | modify | docs/specs/modules/app-studio-ui.md | 固定左列/520px 面板 → 共用可停靠工具区、内外拖动、偏好及窄容器规则 | 解决空间分配和视觉问题 | AC-04,05,06,08 |
| SD-04 | modify | docs/specs/modules/autoui-canvas-runtime.md | 静态回退盒与检查点击 → 实际内容几何、操作/检查输入隔离、双端拖动及聚焦/IME消费 | 动态尺寸下正确交互 | AC-03,05,07,08 |

若修改 auto-lang，上游 spec delta 的精确目标由 T-01 按真实能力定位，在本计划补充仓库限定路径、before/after 与 AC 映射后再实施；不假设不存在的 API。语义合同变化递增 plan_revision。

## 6. 测试设计

- 纯逻辑：布局钳制、横/纵尺寸独立、窄容器恢复、非法偏好、拖动取消、generation 守卫；测试生产 helper，不复制算法。
- Rust 路由/manager 合同：呈现切换不调用 start、不变 generation；陈旧控制/输入拒绝；stop/restart 中切换、控制失败回滚。
- 双端真实探针：同一个计数器和含中文输入/滚动/拖动的表单夹具，覆盖 embedded↔window×3、两种输入模式、各停靠位置和隐藏；绑定 PID/generation/seq、应用 typed 状态和截图，不能只断言样式字符串。
- 几何矩阵：工作台容器 640/900/1280px、左右及底部极限拖动、窗口变小再变大；Windows DPI 100%/150%，适应与100%模式；输入位置与 overlay 偏差不超过 2 个逻辑 px。
- 生命周期：初始无应用、starting/running/restarting/degraded/stopped；切换期间 stop、A→B、前端卸载；目标进程与端口归零，不残留独立窗或失效输入捕获。
- 可视走查：双主题、zh/en、长标题/深树/长源码、普通聊天和设计模式；每端至少左右、底部、隐藏四张有实际内容的截图。中文 IME 候选框位置、真实 OS 拖动另留实机记录。
- 回归复用 `canvas-contract.mjs`、`canvas-ports-probe.mjs`、`canvas-studio-live.mjs`。后者当前引用未入库的 `tmp/vm-studio-cycle.mjs`，T-06 将必要驱动固化到 scripts，不能依赖机器残留文件，也不能只改 --plan 标签复用 PLAN-093 旧收据。

## 7. 验收标准

| ID | 可观察结果 | 验证与预期 |
|---|---|---|
| AC-01 | 默认内嵌应用可见并持续更新，无额外可见 app 小窗 | Vue/VM 实拍+窗口 census；操作计数器5次，应用状态+5且帧更新；连续运行60秒无无故 degraded |
| AC-02 | 内嵌/独立窗口往返保留同会话状态 | 往返3次，PID/generation及输入值/计数不变；只存在应可见的窗，不重新 spawn |
| AC-03 | 操作态与检查态分工正确 | 操作按钮/文本/中文IME/滚动/拖动及编辑键成功；检查点击只改变 picked、计数不变；留白无应用副作用 |
| AC-04 | 左/右/底部/隐藏可选择并恢复 | 两宿主两前端逐项切换，Canvas与DevTools相对位置正确；隐藏后画布回收空间，重新打开保留选择/页签 |
| AC-05 | 内外分隔条真可拖，输入和高亮随之正确 | OS拖动/MCP辅助覆盖最小/最大/越界释放/取消；尺寸不负不塌；100%/150%DPI点选与overlay误差≤2逻辑px；拖动不误触应用 |
| AC-06 | 响应式、偏好与聊天状态保持 | 三种容器宽度+工作区A/B+关闭重开；尺寸钳制、窄屏页签、扩宽恢复通过；对话草稿/消息/审批卡不重挂丢失 |
| AC-07 | 会话失败与陈旧事件安全且四轨一致 | Vue/VM×RustHTTP/VMHTTP新收据全部通过；A→B迟到帧/输入拒绝，stop/restart错误明确，无孤儿进程/端口 |
| AC-08 | 工具区视觉与无障碍可用 | 双主题/语言截图无主要动作溢出或零宽文本；树/源码各自滚动；键盘调divider、Tab焦点、tooltip/名称可用 |

## 8. 执行步骤

所有实现都在专用 worktree；以下路径除标注新增外已核实存在。每项完成写产物、验证命令/版本/实际退出码、关联 AC，不以代码写完替代证据。

- [x] **T-01 能力和界面辨识探针**（无依赖；AC-01/02/03/05/07；SD-01/02/04）`[✅ 已完成 2026-10-10]`：**候选 B（隐藏窗+MCP 截图）实机证伪、候选 A（rq/remote DisplayList v2）选定**。产出：探针 `scripts/ui-parity/canvas-embed-probe.mjs` + 夹具 `tests/ui-parity/probes/canvas-embed/embed-fixture/`（P1 基线/P2 隐藏帧/P3 输入全家桶/P4 显隐往返/P5 remote 臂 census，收据 `tmp/ui-parity/PLAN-102/`）；决策文档 `docs/reports/102-canvas-embed-decision.md`（主检出）。关键证据：`--frontend both`（缺省候选 A）退出 0（P1+P5 pass——app headless census=0 + daemon 帧流 `ops=29 texts=16`）；`--candidate B` 预期红 = 否证（隐藏窗 60s 12 轮 press 帧 md5 恒 1、SW_HIDE 后冻结且 SW_SHOW 不复原；根因 = winit 隐藏窗不投递 RedrawRequested → iced 图层永不刷新，详见决策文档 §3）。上游接缝：S1 `AUTO_VM_HIDDEN`（探针仪器，已落地依赖 worktree）；依赖缺口 D-1（rq 宿主可嵌入库）/D-2（rq 客户端臂无 MCP——P5 finding）；VM 轨消费面 J2（`.at` remote-frame widget）为最大单项。辨识结论：用户所指 DevTools = Musk CanvasStructureColumn（本计划对象）；AutoUI 原生 DevTools 为独立右侧面板（代码级辨识，活体实拍归 T-05，G-3）。依赖 worktree：`.wt/musk-102/auto-lang`（基线 925c3186a + S1）与 `.wt/musk-102/auto-down`（基线 895f8d0 无改动），构建 `D:/autostack/.wt/musk-102/target/release/auto.exe`。
- [ ] **T-02 会话呈现控制和输入通道**（依赖T-01；AC-01/02/03/07；SD-01/02/04）：改 `backend/crates/musk/src/canvas/{session,manager,mod,mcp_client}.rs`，`backend/crates/musk/auto-src/canvas_vm.at`、`backend/crates/musk/src/vm_backend.rs` 及 host extern 对应注册（以T-01定位为准），补 `canvas_studio_contract.rs`。同会话切换、身份守卫、失败恢复、停止收割全部过门；`cargo test --manifest-path backend/Cargo.toml -p musk --test canvas_studio_contract` 预期0；探针再次确认AC-01/02/03/07。`[⏳ 定位已闭合 2026-10-10]`：接口=候选 A（决策文档 §4/§5）——musk 控制面 spawn bridge daemon（`auto rqhost --bridge-ws`，D-1 已落地）+ rq remote app（D-2 MCP 共位已落地），数据面前端直连 bridge WS（status 携端点+token）；AC-02 窗口切换遗留设计位见 §5（rq 客户端 exit-on-EOF 的重连预算 seam）。
- [ ] **T-03 共用停靠布局与偏好**（依赖T-01；AC-04/05/06；SD-03/04）：新增 `src/front/canvas_workbench.at`、`canvas_layout_helpers.at`；改 `canvas_panel.at`、`canvas_store.at`、`chats_view.at` 与两轨 ports。接入左右/底部/隐藏、内外分隔条、持久化和窄容器降级；新增 `scripts/ui-parity/canvas-layout-contract.mjs`，`node scripts/ui-parity/canvas-layout-contract.mjs` 预期0且钳制/偏好/取消全通过。VM必须真实拖动有效，不接受web-only门面。`[⏳ 切片③ 2026-10-10]`：W-03-1 spike 闭合 + workbench 定稿（切片②，见 §9）；切片③ = CanvasPanel 宿主化（520 固定宽退役 → cv_outer_w_decl）+ DivOuter 外部分隔条 + chats_view contentRow 捕获层（mouse-area 单子件包裹——VM 多子件强制 col 化规避）+ 容器尺寸上报（web 实测 installCanvasLayoutSync→canvas.wb_* 键 / VM 派生 outer_w+window_h-78）+ canvas-layout-contract.mjs 合同门 53/0 绿 + gen 69 组件绿。余项（切片④）：VM 实机拖动/停靠/窄容器探针（live 驱动沿 canvas-studio-live 形态，T-06 固化前置）、chats_view 挂载链实机验证、chrome=78 派生常数按 bbox 校准。
- [ ] **T-04 内嵌工具栏和双输入模式**（依赖T-02/03；AC-01/02/03/05/07；SD-01/02/04）：改 `canvas_panel.at`、`canvas_helpers.at`、`canvas_web.ts`、`ports/canvas.{web,vm}.at` 和 `canvas_store.at`，连接模式请求/确认、聚焦/IME、内容坐标、旧帧禁输入。探针 `node scripts/ui-parity/canvas-embed-probe.mjs --frontend both` 预期0，typed状态证明操作与检查隔离。
- [ ] **T-05 DevTools 视觉和可访问交互**（依赖T-03/04；AC-04/05/06/08；SD-03）：改 `canvas_structure.at`、`canvas_source.at`、`canvas_panel.at`、`src/front/i18n/{zh,en}.json`；完成紧凑树行、页签、滚动、长文本、菜单、焦点/tooltip/键盘divider。`auto build --gen-only --strict` 与 `node scripts/ui-parity.mjs check` 预期0；可视走查截图覆盖AC-08。
- [ ] **T-06 四轨回归和交付收据**（依赖T-02～05；全部AC与SD）：改 `scripts/ui-parity/canvas-studio-live.mjs`、`canvas-ports-probe.mjs`，将缺失的VM驱动作为新增受管理文件固化；新增PLAN-102场景/收据目录参数，生产运行不得共享旧临时输出。`node scripts/ui-parity/canvas-contract.mjs`、`node scripts/ui-parity/canvas-studio-live.mjs --plan 102 --frontend both --backend both`、上述合同测试与严格生成均预期0；提交新的截图/typed状态/窗口与进程census/输入证据索引。受影响上游测试命令由T-01定案后补入，不虚构既有测试名。完成后交 `/auto-plan:review`，不得自行宣称复审/合并通过。

执行验证使用T-01绑定hash的实际 AUTO_EXE；生成后 helper 才能运行。当前机器有无相应二进制、依赖或实机权限由执行阶段记录；本次仅计划检查。

## 9. 复审记录

2026-10-10 草稿交接：`stage: new`，`plan: PLAN-102`，`revision: 1`，`outcome: pass`（计划完整性检查，非产品测试/独立复审通过），`next: work`（用户要求执行后）。新增 T-01～06、AC-01～08、SD-01～04。每项AC和规范增量均有任务覆盖。实现路线的能力未知交由有界T-01调查，不以未证实引擎能力作既定方案。

2026-10-10 T-01 执行记录：`stage: work`，`plan: PLAN-102`，`revision: 1`，`outcome: executing（T-01 done，继续 T-02）`，`code_commit`: musk worktree（probe+fixture）/ auto-lang worktree（S1 接缝），`task_ids`: T-01，`evidence`: `docs/reports/102-canvas-embed-decision.md` + `tmp/ui-parity/PLAN-102/*-receipt.json`（P1/P5 pass 退出 0；P2/P3/P4 否证红），`blockers`: 无阻塞；J2（VM 轨 remote-frame widget）为 T-04 前置大项，`next`: T-02（后端 rq 宿主 + D-2 前置）。呈现方案决策门闭合：候选 A 选定，候选 B 证伪（隐藏窗帧冻结——winit 不向隐藏窗投递 Redraw，MCP 截图=离屏重渲恒空图层；SW_HIDE 同因且 SW_SHOW 不复原）。

2026-10-10 T-02 前置 D-1/D-2 执行记录：`stage: work`，`plan: PLAN-102`，`revision: 1`，`outcome: executing（上游接缝全落地，T-02 musk 侧待实施）`，`code_commit`: auto-lang worktree `d3042f5f6`（D-2 MCP 共位）+ `9b45a6c82`（D-1 bridge 宿主），`task_ids`: T-02 前置，`evidence`: `tmp/ui-parity/PLAN-102/P5-remote-arm-census-receipt.json`（D-2：rq 臂 MCP 261ms 上线/vtree 4056B/action count 0→1/截图快速失败）+ `P6-bridge-host-receipt.json`（D-1：宿主零窗 census/Welcome/FrameReadyV2/DisplayList 字节定位 inc(32,29)/**WS 坐标点击→app count 0→1→新帧回推全链闭环**）——`--frontend both` 三探针全 PASS 退出 0，`blockers`: 无；AC-02 无重启窗口切换的宿主切换机制为遗留设计位（rq 客户端 exit-on-EOF → 需重连预算或第二呈现面，见决策文档 §5），`next`: T-02 musk 侧（bridge 会话 spawn + status 携 WS 端点 + 看门狗健康化 + canvas_studio_contract 适配）。

2026-10-10 T-02 musk 侧执行记录：`stage: work`，`plan: PLAN-102`，`revision: 1`，`outcome: executing（后端 embedded 会话落地，T-02 部分红）`，`code_commit`: musk worktree `402234d`，`task_ids`: T-02（部分），`evidence`: 合同门 `cargo test -p musk --test canvas_studio_contract` 21/21 绿（含修 PLAN-099 遗留 chat_runs 基线红）；新增 `embedded_spawn_contract_live`（#[ignore] 实机）spawn/status/presentation{mode=embedded,bridge_ws_url}/vtree 断言全过，`blockers`: **F-02-1（开放）**：am split-mode 在 `-q` 形态遗留脱离 `taskkill /T` 树的 `run -r vm` 子进程（幸存者 cmdline 实证：`auto.exe run -r vm`，父进程已亡）——零孤儿 census 断言红；夹具已对齐 canvas 产线 pac 形态（无 api 字段）后仍现，疑 rq 臂进程拓扑（am 装配）与 legacy 不同，需下程定罪（对照 legacy spawn 同夹具是否同样遗留 + am 装配链），`next`: F-02-1 定罪 → embedded_spawn_contract_live 全绿 → T-02 收口（present 切换路由 + canvas_vm.at/vm_backend 外接线）。

2026-10-10 T-02 收口记录：`stage: work`，`plan: PLAN-102`，`revision: 1`，`outcome: executing（T-02 主体收口）`，`code_commit`: musk worktree（present 路由 + census 收窄两提交），`task_ids`: T-02，`evidence`: **F-02-1 结案=环境污染假红**（对照实验 legacy/rq 两臂同收割零新孤儿；"孤儿"实为他会话进程——另一 ZCode 会话 bash 的 legacy 实验与 vm_matrix.mjs 的子进程，父进程链实证）；`embedded_spawn_contract_live` 全绿（spawn/running/presentation{embedded,ws_url 可连}/vtree/停止收窄 census 零残留）；present 路由 3 合同（stale gen 409 / no-session 400 / embedded 幂等 + window 诚实 501 + 非法 400）；**合同门 24/24 绿**。`blockers`: T-02 余项（登记非阻塞）：VMHTTP 轨 present 路由镜像（extern_impl.rs 为生成产物——需走 extern_sigs.at + gen_extern_sigs.js 管线，下程实施）；canvas_live.rs 实机臂（#[ignore]）按 embedded 形态适配归 T-06 回归批。`next`: T-03（前端共用停靠布局——CanvasWorkbench/停靠/分隔条/偏好）。

2026-10-10 T-03 执行记录：`stage: work`，`plan: PLAN-102`，`revision: 1`，`outcome: executing（T-03 切片①）`，`code_commit`: musk worktree `ff66612`，`task_ids`: T-03（部分），`evidence`: `auto build --gen-only --strict` 65 组件通过（canvas_layout_helpers.at 纯函数 + canvas_store.at 偏好/拖动/钳制动作族；load-once 轮询不覆写；persist 内联）。`blockers`: **W-03-1（设计前置）**：组合件视图稿（canvas_workbench.at.draft）暂缓编译——动态宽度需先验证 VM 轨出程形态（§8.14 四形态限制：`style: <字段>` VM 不产出；既有可行先例 = CanvasStudioSlot `style: [.structureColClass]` 方括号字面量集），下程第一步 = 宽度阶梯 vs 内联 style 双轨 spike（build+探针实测），定稿 workbench；随后 canvas_panel.at 宿主化改造 + 外部分隔条 + i18n 键 + canvas-layout-contract.mjs 合同门。

2026-10-10 W-03-1 spike + T-03 切片②执行记录：`stage: work`，`plan: PLAN-102`，`revision: 1`，`outcome: executing（T-03 切片②）`，`code_commit`: musk worktree（本切片提交），`task_ids`: T-03（部分：W-03-1 + workbench 定稿），`evidence`: **W-03-1 出程 spike 闭合**——探针 `scripts/ui-parity/canvas-vm-style-spike.mjs` + 夹具 `tests/ui-parity/probes/canvas-layout/spike-fixture/`（五臂对照 + cycle 动作改宽），收据 `tmp/ui-parity/PLAN-102/W-03-1-vm-style-spike-receipt.json` 退出 0：A1 数组字面量集 `[.store.cls]` 240→300 ✔、B1 裸引用 CSS 声明串 `.store.w_decl` 280→330 ✔、B3 内联拼接 int 240→300 ✔（均 MCP vtree bbox 实测初值+改宽后两轮）；B2 类名文本裸引用 VM 侧 320→380 亦活（**web 侧 :style 内联类文本无效——双轨不对称坐实，禁用**）；C1 组合形态（class 静态 + style 动态）VM 红 472 撑满——**VM extract class 优先短路 style（aura_view_builder.rs extract_string_with or_else 链），一节点一通道**。web 侧同夹具 `--gen-only` 产物路由形态归档（B1/B3 → `:style` 真 CSS、A1 → `:class` join、B2 → `:style` 死类文本）。**旧口径修正**：spec「VM 轨 :style 字段引用不产出」描的是 PLAN-748 之前行为——748 根修 `a2422add2` 已入 musk-102 依赖 worktree（二进制 15:32 构建），裸引用在 row/col 等绑定感知臂已产出（auto-lang p748_t00_style_faces 同证）。**裁定**：动态尺寸走 store 域组合 CSS 声明串（`width:Npx;height:100%;overflow:hidden;flex-shrink:0`，canvas_layout_helpers canvasDockWDecl/HDecl/OuterWDecl 单源；web 专属键 flex-shrink VM 词法扫描落 remainder 静默跳过）+ `style: .store.<decl>` 单通道；视觉分隔线走 DivH/DivB 自带 w-px 线（停靠列零 border 依赖）；捕获层 = 常驻包裹式 mouse-area（iced 无绝对定位，inset 解析保存渲染降级——避开；嵌套交互子件不吞事件有 chats_view 会话行先例）+ store.cv_drag_kind 门控 + 起拖首 move 定基线（4096 域×容器实际尺寸换算像素）。切片②交付：canvas_workbench.at（四停靠/分隔条/步进钮组/tabs 降级/hidden 重开）、store 三声明串字段 + cv_layout_mode 投影（视图域 fn 调用不冒险）+ 全写点重算、helpers 模式 0 维守卫、i18n 11 键（zh/en）、`--gen-only --strict` **68 组件绿**（+Workbench/DivH/DivB；生成面 `:style` 路由核验 ✔）。`blockers`: 无阻塞。Escape 取消拖动 = store.DockDragCancel 已备、键盘接线归 T-05（AC-08 面）。`next`: T-03 切片③——canvas_panel 宿主化（cv_outer_w_decl 外框 + 外部分隔条）、chats_view 挂载 CanvasWorkbench + LayoutContainer 容器上报、canvas-layout-contract.mjs 纯函数合同门（ext TS 已生成）、VM 实机拖动/停靠/窄容器探针。

2026-10-10 T-03 切片③执行记录：`stage: work`，`plan: PLAN-102`，`revision: 1`，`outcome: executing（T-03 切片③：宿主化+外分隔条+容器上报+合同门）`，`code_commit`: musk worktree `c2028e4`（切片②）+ 本切片提交，`task_ids`: T-03（部分），`evidence`: ① **CanvasPanel 宿主化**——固定 520px 退役，外框 = `style: .store.cv_outer_w_decl`（W-03-1 裁定通道）内挂 CanvasWorkbench；canvas_panel↔canvas_workbench 文件级循环引用经生成器验证可解析（gen 69 绿）。② **外部分隔条 DivOuter**（canvas_workbench.at，与 DivH/DivB 同族）+ **chats_view contentRow 捕获层**：mouse-area **单子件包裹**（VM convert_mouse_area 多子件强制 `View::col()` 实证——row 布局职责留内层 row；捕获仅 outer 拖动态消费，嵌套交互子件不吞事件沿本件会话行先例）；OuterCapMove 起拖首 move 定基线、行宽换算（视口-侧栏：展开 240/收起 w-12=48——本件视图域两态皆知）；跨 store 调用走限定式 `CanvasStore.OuterDragMove`（PLAN-101 毒签先例，裸别名 canvasStore.X() 禁）。③ **容器尺寸上报双源**：web = installCanvasLayoutSync（canvas_web.ts 500ms 轮询 querySelector('.cv-workbench') 写 canvas.wb_w/h；ports 双轨门面 + workbench setup 装载 + 根类标记）实测优先；VM = store Poll 派生（wb_w=outer_w 外框即容器宽 / wb_h=vm.window_inner_height-78 chrome 常数——canvasWbEstH/canvasOuterContainerW 纯函数单源，renderer KV 面普查无元素级键）；变化即重钳+模式重算。**切片①遗留修正**：OuterDragMove 容器误用 wb_w+outer_w → canvasOuterContainerW(cv_win_w)；load-once 外框钳制同改。④ **canvas-layout-contract.mjs 合同门**：ext TS 直测 53/0 退出 0（钳制三族边界/降级判定含 0 维守卫/right 反向拖动/Home 复位/偏好畸形段/声明串全等锚定/VM 派生两函数）——**抓到真缺陷一处**：模块级 const 不进 ext TS 发射面（ReferenceError），helpers 常量改字面量内联+注释单源表（canvas_helpers 先例本就无常量）。⑤ `auto build --gen-only --strict` **69 组件绿**（+DivOuter）；回归 canvas-contract.mjs 65/0 绿。`blockers`: 无阻塞；**遗留观察项**：VM 轨 contentRow 捕获层/mouse-area 嵌套（capture×workbench capture×canvas frame 三层）与 chrome=78 派生常数待切片④实机探针按 vtree bbox 验证/校准；studio 模式 CanvasStudioSlot 未迁 CanvasWorkbench（本切片范围=正常模式宿主化；studio 迁移随 T-05 视觉走查定）。`next`: T-03 切片④——VM 实机探针（live 驱动沿 canvas-studio-live 形态：真实拖动/四停靠/窄容器降级/外框拖动几何断言；T-06 驱动固化前置）；全绿后 T-03 收口转 T-04/T-05。

## 10. 待澄清事项

1. ~~用户未要求本轮实现~~ → 用户已授权执行（2026-10-10 `/auto-plan:work 102`）。
2. 「DevTools」辨识：T-01 已闭合——用户所指 = Musk Canvas 左侧结构/源码工具区（CanvasStructureColumn）；AutoUI 原生 DevTools（app 自带右侧检查面板）不在本计划范围（决策文档 §7）。原生 DevTools 活体实拍（G-3）并入 T-05 视觉走查。
3. ~~remote或持续帧方案待T-01裁定~~ → 已裁定：remote/rq 臂选定；持续帧（隐藏窗）方案证伪（决策文档 §1/§3）。VM 轨消费面 J2（.at remote-frame widget）与 D-2（rq 臂 MCP 缺席）为主要上游接缝，规模与排期见决策文档 §5——若 J2 排期超出本计划容量，需向用户呈报降级方案（VM 轨受控降级）再定，不得静默缩减 AC。
4. 默认内嵌、默认操作态、工具区默认左停靠是本计划产品假设；提供独立窗口与其他停靠选项满足用户选择。工具尺寸初值以可读性走查修正，不改变最小空间和验收要求。
