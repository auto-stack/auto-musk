# app-studio-ui — 会话内应用设计工作台

> 来源：PLAN-093（2026-10-01 落地）+ PLAN-097（G-15 消除与 VM 渲染合同，
> 2026-10-12 沉淀）。会话内「应用设计」工作台：结构/源码 ·
> 实时画布 · 设计对话三段布局；与实况画布模块（app-canvas.md）共用
> CanvasStore 单源与生命周期，本模块只约束界面布局、状态投影与交互契约。
> 实现真源：`src/front/{chats_view,canvas_panel,canvas_structure,canvas_source,
> canvas_progress,canvas_helpers}.at`。

## 入口与模式

- 头部常驻「应用设计」入口钮（actions 行，激活态反色）；进出保存/恢复
  侧栏折叠偏好（sidebar_prev）。
- 设计模式暂收会话列表；消息链与 composer 单实例不重挂——草稿、展开态、
  审批卡跨模式切换保持。
- 空工作台（无会话）：描述提示 + 工作区内路径输入 + 启动钮；路径经既有
  校验（越界/无 pac.at 错误原样落 `studio_start_err`），不向未验证目录发起。
- 「收起」（偏好，轮询不复开）与「停止」（收割当前目标）分离；已停止
  保留应用标题与「重新运行」入口。

## 布局与视觉

- 内容行 studio 模式 row-reverse（对话右置）；宽度分层：对话 360（<1280
  320）；<768 页签塌缩（画布/对话/工具单区，切换保持草稿与选择）。宽度
  驱动源 = ForgeStore.fw_win_w（PollStream 随拍）+ CanvasStore.cv_win_w
  （web 键优先 VM 窗口 KV 兜底）。
- **共用可停靠 DevTools 工作台（PLAN-102，替换固定结构 220 左列）**：
  CanvasWorkbench 外框（正常聊天初值 ~640、聊天区 ≥320 保底）内含工具
  区（左/右/底部/隐藏四停靠 + 停靠条 29）+ 分隔条 + Canvas 预览列。
  工具区初值 240（底部高 240），最小 180（底部 140）；canvas 最小内容
  240x180；窄容器两侧最小不满足 → 画布/工具单区页签降级，变宽恢复原
  方向（偏好保持）。
- **尺寸通道（W-03-1 裁定）**：store 域 CSS 声明串（如
  `width:250px;height:100%;overflow:hidden;flex-shrink:0`）+
  `style: .store.<decl>` 单通道下发；**禁 class:+style: 并存**（VM
  extract class 优先短路 style）与类名文本裸引用。浮点→px 收整用
  `.to_int()`（三语句字符串中转；`Math.floor` 外源浮点穿 int 边界 +
  float→str 拼接位型缺陷曾写出 `width:0px` 致 0 宽静默退出——F-03-1）。
- **拖动契约**：分隔条可见 1px/命中 ~6px，自持 Down/Move/Up 三 handler
  （__mcp_drag 注入合同同构）；越界续拖经 capture 层（contentRow 包裹
  式）；Escape 取消本次拖动恢复起始尺寸（app.at GlobalKeydown 限定式
  汇流）；键盘 divider = 步进钮组（±10/Home，title 可读名）。
- **偏好**：停靠方向/水平宽/底部高/外框宽按 workspace + 前端轨持久
  （轮询不覆写）；隐藏/恢复保留方向、页签、树折叠。
- 视觉沿 pac.at 语义 token（background/card/muted/border/foreground/
  primary）；状态同时有文本/图标不单靠颜色；树行 12px、次要 11px 下限；
  主要动作命中 ≥32px。不新增主题面（VM 轨上游 dark-only——双主题证据
  面在 web 轨，F-RF 复审补证）。

## 结构 / 源码工具区

- 页签化（结构|源码）。结构树从实际 vtree 扁平表渲染：折叠键 = 稳定
  字符串 vid（>2^53 安全），跨帧回填/折叠保持；循环节点显示 index/value；
  树行一行化（kind/label/for + 截断 + title tooltip）。
- 源码面板：只读；行对象由 store 域构建（视图内不 parse）；拾取行高亮 +
  自动滚动定位；非法行号/读失败/无 source/来源待确认各有明确状态；
  来源置信度 exact/uncertain（多文件启发式必须显示「来源待确认」）；
  树/源码各自滚动（scroll 双界 flex-1 min-h-0）。

## 帧区与坐标点选

- 内容包装层（cv-frame-wrap）为覆盖框百分比定位与点选换算的统一内容盒：
  web 轨动态 :style（aspect-ratio 适应 / 物理px 100% + 内滚）；VM 轨动态
  :style 字段引用不产出（codegen 已知四形态限制）→ 静态等比回退尺寸
  （360x510 = 480x680×0.75）承载，web 侧 inline 优先覆盖等比一致。
- mouse-area（coords=目标逻辑窗 480x680）包帧内容：真实 OS 指针走
  onmousemove 记迹 + onclick 取最后位置；MCP 测试注入走 __mcp_drag
  （Widget␟Down(str)␟Move(float,float)␟Up()␟points，逻辑坐标=coords
  值域，与 PointerArea 闭包同构）。逻辑→物理：物理 = 逻辑 × 帧物理/逻辑窗。
- 留白（内容盒外）点击 = 清选，不对目标应用发送 act（web 委托承载）。

## 生成/检查进度摘要

- 对话区顶部紧凑条（可展开）：只投影有证据的行——生成（文件/Blueprint
  工具事件）、静态建议（ui_lint，advisory 非硬门）、Blueprint 检查、
  启动预览（CanvasStore 实际状态权威，start 返回 ≠ 已可见；代次章失配
  的陈旧事件不冒充）、交互验证（未执行显式「未验证」）、画面更新
  （seq 变化，无因果证据只写「最近画面已更新」）。状态只来自工具
  status 字段与实时状态，不嗅探文本（绿勾禁令）。
- **G-15 消除（PLAN-097 已实证）**：进度行数据由 CanvasStore 在 Poll 拍内
  handler 域预计算（`cp_prog_rows/cp_prog_has`，try 界定），摘要组件
  `CanvasProgressSummary` 自持 store（own-store 读），视图仅做 for-源列表 +
  循环局部成员读 + prop 直渲——零 computed、零模板插值、零成员链判言。
  VM 与 Vue 双轨同义可见（probe-g15 SMOKE PASS + T-11 真机 snapshot-ok）。

## VM 轨渲染合同（PLAN-097 T-11 真机实证）

- 文本位的 store 成员读（模板插值 `${store.x}` 与 `text store.x` 直读）
  被 VM 渲染器丢弃——须走 handler 域预构建列表 + for 局部读。
- computed 内 `.length` 成员链恒空——布尔化后在 handler 域落账。
- 单 widget 多个顶层 `on` 块只有一块生效（CanvasProgressSummary 双块
  实证：Init 被丢弃）——每 widget 一个 on 块。
- 模块 fn 进模板须 `use.web` 形态（plain use 不暴露 Vue 符号；VM 轨
  两者皆通）。
- store handler 域调用模块级 fn 返回空（同文件与跨模块同症，归
  auto-lang VM 调用语义）——运行时依赖的 fn 以同文件内联副本承载
  （`canvasProgressRows` 内联先例，与 canvas_helpers.at 单源同步由
  注释锚定）。

## 已知限制

- ~~G-15：进度摘要 VM 轨隐藏~~ **已消除（PLAN-097）**，见「生成/检查
  进度摘要」。G-17：studio 模式 VM 快照 rect
  全零（布局渲染不受影响；坐标/几何面按 normal 模式验证）。
- 键盘：Escape 收敛（源码页签→结构/错误详情收起，不触 stop）；Tab 序
  与双主题实拍列 review 复验项。

## Composer 发送双轨合同（PLAN-101）

- composer 发送在 VM 轨必须可用：`ChatsView` 的 `SendInput`/`CtxRemove`
  handler **不得引用第二 store 裸别名**（`canvasStore` 等）——VM 轨
  handler/模块 fn 域的第二 store 别名解析为合成断点（poison-drop 实证，
  auto-lang 上游缺口已登记 PLAN-101 §10）；跨 store 动作用限定式 emit
  （`CanvasStore.ClearPick()`，形态同 `PlansStore.Reload()` 先例）。
- 拾取 ctx 快照（PLAN-093 Q-04 DTO）经双轨同名端口 `canvasPickSnapshot`
  构造：无态直查 `GET /api/canvas/status`（与 CanvasStore 投影同源），
  web 实现在 `canvas_web.ts`、VM 实现在 `ports/canvas.vm.at`，DTO 逐字段
  对齐（forctx 缺席 = index -1 三字段空）；picked 空/请求失败返空值 →
  不带 ctx 发送，不阻断。
- 拾取 chip 的 VM 可见性（`chats_view.at` 模板跨 store 条件同族断点）
  登记为引擎模板条件族既有边界，不属本模块承诺；上游修复后自然恢复。
