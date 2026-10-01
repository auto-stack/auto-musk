# app-studio-ui — 会话内应用设计工作台

> 来源：PLAN-093（2026-10-01 落地）。会话内「应用设计」工作台：结构/源码 ·
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
  320）、结构 220（<1280 200）；768-1023 结构列受控显隐（切换钮列）；
  <768 页签塌缩（画布/对话/结构单区，切换保持草稿与选择）。宽度驱动源
  = ForgeStore.fw_win_w（PollStream 随拍）+ CanvasStore.cv_win_w（web 键
  优先 VM 窗口 KV 兜底）。
- 视觉沿 pac.at 语义 token（background/card/muted/border/foreground/
  primary）；状态同时有文本/图标不单靠颜色；树行 12px、次要 11px 下限；
  主要动作命中 ≥32px。不新增主题面。

## 结构 / 源码左列

- 页签化（结构|源码）。结构树从实际 vtree 扁平表渲染：折叠键 = 稳定
  字符串 vid（>2^53 安全），跨帧回填/折叠保持；循环节点显示 index/value。
- 源码面板：只读；行对象由 store 域构建（视图内不 parse）；拾取行高亮 +
  自动滚动定位；非法行号/读失败/无 source/来源待确认各有明确状态；
  来源置信度 exact/uncertain（多文件启发式必须显示「来源待确认」）。

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
- web 轨 computed use.web.fn 消费；VM 轨 computed 返空 → 摘要隐藏
  （已知限制，不误报）。

## 已知限制

- G-15：进度摘要 VM 轨隐藏（见上）。G-17：studio 模式 VM 快照 rect
  全零（布局渲染不受影响；坐标/几何面按 normal 模式验证）。
- 键盘：Escape 收敛（源码页签→结构/错误详情收起，不触 stop）；Tab 序
  与双主题实拍列 review 复验项。
