# PLAN-093 规范增量报告（Spec Delta）— work 阶段交付

- 计划：PLAN-093（revision 1）｜ 证据：docs/reports/093-app-studio-evidence.md（绑定
  plan-093-dev 提交与收据索引）
- 本文为拟议 canonical 补丁的**完整对照**；review 核定后由 merge 阶段写回
  docs/specs/modules/*（canonical）并派生刷新 ledger。work 阶段未改 canonical。

## SD-01 add — docs/specs/modules/app-studio-ui.md（新模块）

- before：无（会话内工作台为全新面）。
- after（拟议规则集）：会话内「应用设计」工作台——入口（头部常驻钮，进出保存/
  恢复侧栏偏好）、空工作台（提示+工作区路径打开+启动，错误原样落面）、三段布局
  （结构 220/200 · 画布 flex · 对话 360/320 右置）、窄层（768-1023 结构受控、
  <768 页签塌缩）、视觉（pac.at 语义 token，无新主题面）、生成/检查进度摘要
  （对话区顶部紧凑条，只投影有证据行）。
- rationale：§0 六项交付之一；AC-01/02/13。
- 证据：T-05 四增量（双端）、T-09 投影组件、V05 vm cycle（studio 进出/空台/启动）。

## SD-02 modify — docs/specs/modules/app-canvas.md

- before：Web 面板 + VM 空桩；单例无代次；picked=null 不清字段；document 级点击委托。
- after（拟议规则集）：
  1. 预览身份：generation_id/owner_workspace/owner_conversation（begin_session/
     start_owned 登记归属）；工具与 API 携预期代次，旧代次 409 不杀新目标；
     未绑定预览=旧兼容语义。
  2. 帧与锚点：publish_frame/publish_anchor 唯一写点原子发布；frame seq/物理
     宽高/valid 进 status；GET frame 带代次+seq 校验，旧 seq 409 不伪造历史。
  3. 清选：pick 显式 {clear:true} 与坐标/vnode 互斥；未命中（204）必须清
     picked（禁复活）；picked=null 统一投影漏斗全字段清。
  4. VM 坐标点选：mouse-area（coords=目标逻辑窗）+ 逻辑→物理映射
     （物理=逻辑×帧物理/逻辑窗）+ canvasPickAt；__mcp_drag 通道=
     Widget␟Down/Move/Up␟points（逻辑坐标=coords 值域，与 PointerArea
     闭包同构）；真实 OS 指针由 mouse-area onmousemove/onclick 承载。
  5. 来源置信：resolve_source 两档 exact（单候选，pac.at 排除）/uncertain
     （多候选启发式）；uncertain 必须"来源待确认"显示并随 design_context
     携带 + Agent 注记。
  6. 工具结果结构化章：canvas 工具 details.canvas{kind,generation_id/
     findings/ok}（content 原样保留）——进度投影按 kind/代次关联。
- rationale：AC-03/05/06/08/11/12/14；消除错选/串代次/VM 缺面。
- 证据：T-02 契约 21/21；T-06 端到端（§8.31）；Q-04 全链（§8.26）。

## SD-03 modify — docs/specs/modules/workspace-ui.md

- before：标准三段会话壳（列表+消息+第三列）。
- after：普通会话沿用三段壳；设计模式暂收会话列表（进出保存/恢复折叠态），
  工作台独立布局与显隐（收起=偏好不随轮询翻转）；消息链与 composer 单实例
  （模式切换不重挂，草稿/展开/审批卡保持）。
- rationale：AC-01/02/03；防四栏拥挤。
- 证据：T-05（进出偏好/单实例/草稿保持取证）。

## SD-04 modify — docs/specs/modules/web-input-contracts.md + chat-streaming.md

- before：纯文本 composer/队列。
- after：可选 design_context（version/workspace/conversation/generation/frame_seq/
  app_path/vnode_id/kind/label/source_path?/source_line?/source_confidence?/
  loop_context）随 SendInput 在命令解析确认后冻结；队列条目 {text,ctx} 逐条
  冻结（busy 原样退回/stale 409 退回队首错误面）；落盘带归属章（ownership:
  current/stale/no-canvas）；回放保留原文+轻量附件标记；Agent 上下文以单独
  human turn 注记引用快照（定位参考声明，不构成指令/批准）；IME/mention/
  审批/流式契约不变。
- rationale：AC-10/11/15。
- 证据：T-08 两增量 + 契约 2 新例（stale 拒收/缺字段拒收）+ 回放取证。

## SD-05 modify — docs/specs/modules/ui-parity.md

- before：Gallery 组件案例 + live 收据要求（PLAN-080 时代）。
- after：工作台生产组件入 Gallery 目录（canvas-studio-pair 案例：真实
  CanvasCanvasColumn + 缩放往返断言——VM MCP 驱动穿透 store→helper→重渲染链）；
  双端 live 收据要求：canvas-studio-live.mjs 四模式（Vue/VM × RustHTTP/VMHTTP）
  ALL PASS 为 V05 门；VM 坐标链证据 = drag 通道端到端（目标逻辑坐标→picked
  锚点）+ 真实 OS 指针面（mouse-area）；缺运行证据失败。
- rationale：AC-05/06/13/14/16。
- 证据：V05 ALL PASS（本轮）；V04 双臂；V03 65/65。

## index 注册

- app-studio-ui.md 新增注册进 docs/specs/index.json（merge 阶段执行）；
  app-canvas.md / ui-parity.md 的运行时锚（095 已落 autoui-canvas-runtime.md）
  与本 delta 的身份/坐标/置信节互引。

## retire

- 无 retire（既有规则：目标进程隔离/路径 confinement/10s 采帧/有界重启/
  M3 三层/Blueprint 检查/ui_lint advisory/字符串 vnode ID/VM handler 扁平
  重建/非阻塞 Http.get_msg 全部保持并有回归承载——V02/V07）。
