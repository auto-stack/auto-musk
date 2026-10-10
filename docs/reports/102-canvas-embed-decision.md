# PLAN-102 T-01 决策文档 — Canvas 内嵌呈现方案定案

> 阶段：T-01 能力和界面辨识探针（2026-10-10）。
> 探针：`scripts/ui-parity/canvas-embed-probe.mjs`（musk worktree
> `D:/autostack/.wt/musk-102/auto-musk`，分支 `plan-102-dev`）。
> 依赖构建：auto-lang worktree `D:/autostack/.wt/musk-102/auto-lang`，分支
> `auto-musk-dev-102`，基线 `925c3186a` + S1 接缝（`AUTO_VM_HIDDEN`），
> 产物 `D:/autostack/.wt/musk-102/target/release/auto.exe`（mtime
> 2026-10-10T05:54Z）。auto-down 兄弟 worktree 同组（`auto-musk-dev-102`，
> 基线 `895f8d0`，无改动）。

## 1. 决策结论

**选定候选 A：rq/remote 臂（headless iced + DisplayList v2 过线）作为
Musk canvas 内嵌的呈现与输入底座；候选 B（隐藏窗 + MCP 截图帧流）经实机
探针证伪。**

证伪一句话：本栈（iced 0.14 + winit 0.30 + wgpu，Windows）上隐藏窗
（boot `visible:false` 或运行期 `SW_HIDE`）的帧管线冻结——状态/输入通道
完好，但渲染图层永不更新，`iced::window::screenshot` 恒返回空白（boot
隐藏）或陈旧帧（后隐藏，且 SW_SHOW 后亦不复原）。remote 臂无窗、帧
<1KB/变更、交互完备（PLAN-690 IME 闭环/hover/光标/键盘），P5 实测帧流
与 headless census 全过。

## 2. 候选对比表（探针实测）

| 维度 | A：rq/remote 臂 | B：隐藏窗 + MCP 截图 |
|:---|:---|:---|
| 无可见小窗持续出帧 | ✅ P5：app headless（自有窗 census=0），daemon `[rqhost] perf EmbedFixture ops=29 texts=16` 帧流持续 | ❌ P2：`AUTO_VM_HIDDEN=1` census=0 ✓ 但 60s 内 12 轮 press→帧 md5 恒 1（`distinctFrames=1`）；对照可见窗 3 帧全新鲜 |
| 输入（点击/文本/中文/拖动） | ✅ 协议级（InputMsg 权威命中在 app 侧）；闭环实证=PLAN-508 T6 实机收据（在盘 `auto-lang/docs/plans/reports/assets/508/`） | 状态侧 ✅/帧侧 ❌：P3 隐藏窗上 `type_text "你好世界"`、submit、`__mcp_drag` Float 坐标（mx=80.25/my=90.5 精确）全过——输入通道与渲染臂正交，但用户看不到回显 |
| 无重启窗口切换 | 未在本轮实证（detach/attach 先例=session.rs `detach_surface_to_os_window`，desktop 会话内已落地）；Musk 侧=宿主切换消费面（T-02 设计） | ❌ P4：`SW_HIDE→SW_SHOW`×3，PID/状态保持 ✓，但 SW_HIDE 后帧冻且 SW_SHOW 不复原——切换后内嵌面死帧 |
| 帧形态 | DisplayList v2 矢量命令帧（981B/帧 @本夹具），消费端任意比例重放清晰 | PNG 像素快照（~65KB@960x1360），MCP HTTP 轮询节奏 |
| Vue 轨消费 | ✅ 现成：`packages/drawlist-renderer`（TS/Canvas2D，Rust↔TS golden 对拍）+ PLAN-516 RemoteWindow.vue 先例 | ✅ 现有 img 通道（若帧活着） |
| VM 轨消费 | ❌ 需新增消费面（见 §4 接缝 J2——`paint_ops_v2` iced 泛型画师 + `displaylist_element` 已在 broker_surface.rs，缺 .at widget 壳与传输） | ✅ 现有 img + mouse-area（若帧活着） |
| MCP（vtree/pick/state 锚定面） | ❌ **发现 D-2**：`run_vm_rqhost_client` 绕过 `run_session`，MCP server 不启动（P5 finding `findingMcpAbsentOnRqClient=true`）——需上游补 | ✅ 全活（P1/P2/P3 实证） |
| 每帧成本 | 命令帧（增量小） | 全窗 capture+PNG 编码+文件 IO |

## 3. 候选 B 冻结根因（定罪链）

1. `iced::window::screenshot` = 离屏重渲**当前 renderer 图层**
   （iced_wgpu lib.rs `screenshot()` → `self.draw(bg, &view, viewport)`），
   不是屏幕抓取。
2. 图层只在 iced_winit `RedrawRequested` 处理器内由
   `interface.update → interface.draw` 刷新。
3. winit 0.30 Windows `request_redraw()` = `RedrawWindow(RDW_INTERNALPAINT)`
   （winit window.rs:152）——隐藏窗（`WS_VISIBLE` off）不被投递 WM_PAINT，
   redraw 永不执行。
4. 实测双形态：boot `visible:false` → 图层恒空 → 截图纯背景色
   （uniq~1，`#141A29` 实拍像素统计）；可见渲染后 `SW_HIDE` → 最后一帧后
   冻结（B/B2/C/C2 md5 恒同），`SW_SHOW` 后不复原。
5. 对照组：同一夹具可见窗 3 帧全新鲜（`hidden-frame-test.mjs`，tmp/ 收据）。

推论：Plan 500 像素臂（`run_native_iced_pixels`，`Settings{visible:false}`）
按同一机制推断在本机同样无法产出新鲜帧——该臂解释态入口已被 PLAN-033
退役、native 臂无 `auto run` CLI 面，未在本轮证伪范围；登记上游疑点
（见 §6 G-1），不阻塞本决策。

## 4. 接口草案（选定路线 A 的 Musk 消费架构）

```
┌ musk backend (Rust) ─────────────┐   ┌ auto.exe app 子进程 ────────┐
│ CanvasManager                     │   │ auto run --render=vm        │
│  ├ spawn: AUTO_RQHOST_PIPE=<pipe> │←──│  + AUTO_VM_RENDER=remote    │
│  │        AUTO_VM_RENDER=remote   │   │ （headless iced，DisplayList │
│  ├ rq 宿主端（rendezvous adopt →  │pipe│   v2 帧生产 + 协议输入消费）│
│  │  per-app pipe → Hello/Welcome │──→│                             │
│  │  → FrameReady/HitTable ⇄ InputMsg）│  MCP server（需 D-2 补齐） │
│  ├ 帧中继路由 /api/canvas/*       │   └─────────────────────────────┘
│  └ 输入路由（前端事件 → InputMsg） │
└───────┬───────────────┬───────────┘
        │ HTTP/SSE      │ 同
   Vue 轨（web）    VM 轨（vm_http）
   drawlist-renderer  remote-frame .at widget（J2）
   (Canvas2D)         （displaylist_element iced 泛型画师）
```

- 呈现模式切换（embedded↔window）：宿主侧切换消费面（窗口模式=宿主把
  会话交给独立呈现面），不重启 app 进程、不换 generation；确切机制
  （Musk 侧第二消费面 or rqhost daemon 托管窗）在 T-02 定案并实机验证。
- 操作/检查双模式：操作态=前端指针/键盘/IME 事件→InputMsg；检查态=现有
  pick/overlay 通道（经 D-2 补齐的 MCP）。
- vtree/pick/state 锚定面与 DevTools 结构树：继续走 MCP（D-2 前置）。

## 5. 依赖修改点（auto-lang 接缝清单）

| ID | 接缝 | 形态 | 规模 |
|:---|:---|:---|:---|
| S1 | `AUTO_VM_HIDDEN=1` boot 隐藏位 | 已落地（renderer.rs `startup_window_hidden()` + `visible: !opts.pixels && !startup_window_hidden()`）——本轮探针仪器；产品路线 A 不依赖，是否合回上游由复审裁定 | 小（~20 行） |
| D-1 | 帧宿主可嵌入库：`run_daemon` 去窗化（或抽 host endpoint 库）供 musk backend 作 rq 宿主 | 待 T-02 起草 | 中 |
| D-2 | rq 客户端臂补 MCP server 启动（vtree/pick/state 面与渲染臂解耦） | 待实施 | 中小 |
| J2 | `.at` `remote-frame` widget（VM 轨 DisplayList 消费面：schema/codegen + `displaylist_element`/`paint_ops_v2` 复用 + pipe/WS 传输 + 输入映射） | 待实施 | 大（本计划最大单项） |
| 上游 spec 增量 | `docs/design/autoui/desktop-protocol-v1.md`（若帧宿主语义变化）/ ui overview 增补 | 随 D-1/J2 定稿 | — |

上游 spec 权威：`D:/autostack/auto-lang/docs/specs/auto-lang/ui/overview.md`
（remote 四值 PLAN-683 SD-02 + 交互完备四面 PLAN-690 SD-02 段）。

## 6. 发现与疑点登记

- **G-1**（上游疑点，非本轮证伪范围）：Plan 500 像素臂隐藏窗帧新鲜度按
  §3 机制推断存疑（解释态入口已退役，native 臂无 CLI 面）。
- **G-2**（小）：P3 `scrollThenClickDistantRow` 未过——scroll action 语义
  （`"0,1200"` 值域）或夹具列表折叠（vtree 行 bbox h=0）待 T-04 输入接线
  时以真实滚动容器复核；非架构阻塞（remote 臂滚轮走协议事件）。
- **G-3**（辨识收据缺口）：AutoUI 原生 DevTools（`render_devtools_panel`
  右侧可拖检查面板）实拍未取得——`autoui_keyboard{F12}` 派发
  `__toggle_debug` 后面板未出镜（三次尝试：MCP F12 / MCP F12+press 置脏 /
  SendKeys 焦点前置，截图字节级恒同）。辨识以代码级（renderer.rs
  `render_devtools_panel`：`[main][divider][DevTools]` 行布局 + 内外 divider
  拖动）+ PLAN-087 Musk 侧实拍（`docs/reports/assets/102-identify-canvaspanel-ref-087.png`）
  完成；原生 DevTools 活体实拍并入 T-05 视觉走查补齐。
- **环境**：auto-lang 主检出有他会在途 WIP（renderer.rs/session.rs 等 5 文件
  M 态，PLAN-738 家族）——本轮依赖 worktree 从干净 HEAD `925c3186a` 创建，
  未受污染；主检出 target/release/auto.exe（10-08 构建）只用于早期冒烟。

## 7. 辨识结论（T-01 辨识使命）

- 用户所指「Canvas 左侧 DevTools」= **Musk CanvasStructureColumn**
  （canvas_structure.at：结构/源码页签列）——本计划改造对象确认。
- AutoUI 原生 DevTools（app 自带 F12 检查面板）= 独立右侧面板，**不在**
  本计划改造范围（§10-2 预设解释成立，无扩散修改）。

## 8. 探针收据索引

- 脚本：`scripts/ui-parity/canvas-embed-probe.mjs`（worktree）。
- 收据：`tmp/ui-parity/PLAN-102/{P1-baseline-visible,P2-hidden-frames,
  P3-input-complete,P4-toggle-norestart,P5-remote-arm-census}-receipt.json`。
- 命令与退出码：
  - `node scripts/ui-parity/canvas-embed-probe.mjs --frontend both`
    （候选 A，缺省）→ **P1 PASS + P5 PASS，退出 0**。
  - `node scripts/ui-parity/canvas-embed-probe.mjs --frontend both
    --candidate B`（证伪仪器）→ P1 PASS，P2/P3/P4 FAIL（预期红=否证证据）。
- 夹具：`tests/ui-parity/probes/canvas-embed/embed-fixture/`（计数器 +
  中文输入 + 提交 + 30 行滚动列表 + 坐标拖动面）。
- 聚焦实验（tmp/，未入库）：`hidden-frame-test.mjs`（隐藏/可见对照）、
  `swhide-frame-test.mjs`（SW_HIDE 冻结链）、`capture-autoui-devtools.mjs`。

## 9. 对计划任务的影响（T-02 起的执行合同落点）

- T-02（会话呈现控制与输入通道）：spawn 契约改为 rq/remote 形态
  （`AUTO_RQHOST_PIPE`+`AUTO_VM_RENDER=remote`）；musk backend 增加 rq
  宿主端与帧/输入中继路由；D-2（rq 臂 MCP）为前置依赖项；「同会话切换、
  身份守卫、失败恢复、收割」验收面不变（AC-01/02/03/07）。
- T-04（双输入模式）：操作态输入走协议 InputMsg（坐标/键盘/IME）；检查态
  走 MCP pick；`canvas_panel.at` 帧区从 img 换为消费面（web=
  drawlist-renderer，VM=J2 widget——J2 未就绪前 VM 轨以受控降级显式登记，
  不得冒充完成）。
- T-03/T-05/T-06 不受候选选择影响（布局/视觉/回归与底座解耦）。
- AC-05「拖动后输入和高亮随之正确」的 DPI/坐标断言在消费面为矢量重放
  （A）时天然按逻辑坐标成立；仍按 AC 原文双 DPI 实测。
