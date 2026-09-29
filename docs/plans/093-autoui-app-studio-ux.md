---
plan_id: PLAN-093
status: executing
feature_name: AutoUI 应用设计工作台与实时 Canvas 双端 UX
author: [agent]
created_at: 2026-09-28T14:20:35Z
updated_at: 2026-09-29T21:00:00Z
plan_revision: 1
current_step: 2
total_steps: 14
supersedes_spec_components:
  - docs/specs/modules/app-canvas.md
  - docs/specs/modules/workspace-ui.md
  - docs/specs/modules/web-input-contracts.md
  - docs/specs/modules/chat-streaming.md
  - docs/specs/modules/ui-parity.md
new_spec_components:
  - docs/specs/modules/app-studio-ui.md
touched_goals:
  - goal-agent
  - goal-frontend-parity
  - goal-security
---

# PLAN-093 — AutoUI 应用设计工作台与实时 Canvas 双端 UX

## 0. 变更摘要

将当前挂在会话右侧的 520px Canvas 面板改为会话内的“应用设计”工作台：
左侧结构／源码、中间实时画布、右侧现有设计对话。保留全局导航和会话能力，
以现有 AutoUI 组件实现清晰的布局、状态和联动，不另造一套聊天系统。

这次交付包含六项相互配合的变化：

1. 独立管理工作台显隐与目标应用生命周期，“收起”和“停止”分开。
2. Vue 与 VM 都可展示真实目标 VM 的帧，并完成树选、画布点选、源码定位。
3. 修正 contain 留白下的坐标换算与覆盖框对齐，防止旧帧、旧选择串入新应用。
4. 将选中元素显示为输入框上方的可移除上下文；发送和排队时冻结其快照。
5. 用真实工具结果呈现生成、检查、运行和验证状态，提供清晰的失败恢复。
6. 建立双端、多窗口尺寸和真实进程的验收证据，并提出对应模块规范增量。

本文件是实施合同草案，尚未执行。计划文档按 AGENTS.md 写入主检出
`docs/plans/`；后续代码工作必须进入专用 worktree。

## 1. 目标

### 交付目标

用户可以在一个会话内完成“描述应用 → 查看生成过程 → 看见真实运行画面 →
选择元素 → 描述修改 → 查看更新 → 查看验证结果”，不必在会话、源码和外部
VM 窗口之间反复切换。停止、收起、等待帧、重新启动和选中元素的含义可辨认。

同一套 `.at` 布局与业务状态覆盖 Vue 和 VM；平台差异封装在 ports／宿主桥，
不得以 VM 空桩、静态截图或只有编译通过代替交付。

### 范围

- 主仓库：auto-musk。前端工作台、CanvasStore、现有会话／composer、
  Canvas API 与工具上下文、VM backend 桥、i18n、测试和证据。
- 依赖：读取当前 auto-lang／auto-ai／auto-down 能力并集成，默认不修改它们。
  若 T-01 证明必须修改 auto-lang，先给出最小缺口和依赖任务，修订本合同的
  仓库范围；不得在主检出顺带修改依赖或悄悄降低 VM 验收要求。
- 普通会话、计划、规范、知识、文件栏目保持已有功能；本计划只增加应用设计
  模式的入口和切换，不再进行五个栏目的全面改版。
- 规范沉淀由后续 merge 阶段完成；work 阶段提出规范增量及证据。

### 非目标

- 不新增第六个一级导航，不建设应用市场、应用目录或多 Canvas 并发管理。
- 不实现用户直接操作目标应用的交互模式；画面点击用于选择元素，
  真实 press／type／toggle 仍由已有 canvas_act 工具完成。
- 不实现属性面板直接编辑、拖拽布局、可拖动分栏、Blueprint 反向提取。
- 不改变目标 VM 当前 480×680 的逻辑窗口；“适应／100%”只改变展示比例。
- 不引入原生窗口嵌入、视频帧推流或新的 Agent 编排引擎。
- 不将现有六区 ledger 变为另一份需要独立维护的行为规范。

### 成功边界

AC-01～AC-16 全部有可复验结果。无法完成的必要能力必须回到 needs_replan／
blocked 的具体决策，不能从验收中删除，也不能登记为遗留后宣称完成。

## 2. 架构方案

### 布局与复用关系

```text
全局导航 64px │ 应用设计工作台
              │ 顶栏：应用名 · 运行状态 · 同步状态          [收起] [停止／运行]
              │ ┌结构／源码┐ ┌实时画布────────────────┐ ┌设计对话──────┐
              │ │组件树     │ │工具栏：选择元素 480×680 │ │现有消息与工具 │
              │ │选中详情   │ │适应／100%               │ │生成／检查摘要 │
              │ │只读源码   │ │真实 VM 帧 + 选择覆盖框   │ │元素上下文 chip│
              │ └──────────┘ └────────────────────────┘ │现有 composer │
              │                                          └──────────────┘
```

`ChatsView` 决定普通会话／设计模式及布局；进入设计模式时收起原会话列表，
退出时恢复原折叠状态、会话和草稿。保留一份消息链和一份 MentionInput 实例，
避免模式切换卸载导致草稿、展开状态和审批卡丢失。

现有 `CanvasPanel` 改为工作台的画布消费组件；结构／源码按需要拆为生产组件。
`CanvasStore` 保存规范化的状态投影、选中元素和视图偏好，不承载聊天运行引擎。
`ForgeStore` 继续负责发送、消息队列、运行守卫、流式／轮询、审批和回放。

### 单源状态与平台边界

- CanvasManager 是目标运行、当前帧、锚点索引、picked、overlay 的权威来源。
- 工作台显隐、窄屏页签、树折叠、展示比例是前端视图状态；状态轮询不能
  每秒把用户主动收起的工作台重新打开。
- 一条 Canvas 会话新增预览身份 `generation_id`、工作区及所属会话信息。
  当前仍只允许一个目标应用；身份隔离不等于新增多应用管理器。
- 前端所有异步请求携带／捕获身份，收到响应时核对当前身份；后端变更操作
  核对预期身份，旧响应和旧停止请求不能影响已经替换的新应用。
- Vue 的 DOM 几何／图像加载，与 VM 的媒体读取／指针坐标，通过平台端口
  提供共同的展示与点选契约。不得把布局全部塞入 web-only CSS。
- 图片和覆盖层共用一个实际内容矩形。图像、状态和索引的版本一致后才允许
  点选；帧更新过程中不显示错配的旧框。

## 3. 技术栈

| 层 | 沿用方案 | 本计划约束 |
|---|---|---|
| UI 真源 | AutoUI `.at`，row／col／button／label／scroll／popover 等 | 优先已支持的基本组件；不用重型设计器 |
| Vue 平台 | 生成 Vue 3 工程、现有 TypeScript 平台门面 | 只手写源目录中的平台适配；不修改 gen 的生成模板 |
| VM 平台 | AutoVM／Iced、Http.get_msg、宿主桥及现有媒体能力 | 真实异步加载帧；不得 UI 线程解码／网络阻塞 |
| 后端 | Rust／axum／tokio，CanvasManager 与现有 Auto 后端 | 保留 RustHTTP 与 VMHTTP 已支持的入口 |
| 会话 | ForgeStore、MentionInput、既有 message API 与回放 | 增加可选上下文，不改变旧请求默认行为 |
| 样式 | pac.at 主题、共享语义颜色、现有图标和 i18n | 尊重 primary 与用户主题；不给每页另配一套主题 |
| 验证 | cargo tests、Auto 构建、ui-parity Gallery、MCP／浏览器 | 使用生产组件及真实应用，证据绑定代码版本 |

`backend/crates/musk/src/auto_generated/extern_impl.rs` 虽在 auto_generated
目录下，属于现有手写 glue 层；可按当前桥接约定修改。
真正的转译产物（例如 auto_generated/chats.rs）必须由 Auto 源及既有生成链产生，
不能只补生成结果使下一次构建丢失改动。T-01 固定并记录实际生成命令。

## 4. 需求分析与背景调查

### 已有授权与流程边界

- 用户要求分析新的 AutoUI 应用设计／Canvas 功能，并按已讨论的工作台设计
  写一份详细新计划；最新指令明确使用仓库 auto-plan-new 技能。
- 已讨论范围包括简洁现代的 UI、AutoUI 能力约束、Vue／VM 两端，以及计划
  和对应 worktree 的开发方式。本计划把这些要求落实为明确的任务和验收。
- 最新授权是计划撰写，本轮不自动进入代码执行、提交、合回或部署。
  用户未指定执行预算、自动续跑次数或模型组合，不作推定。
- AGENTS.md 的计划文档例外适用；后续 worktree 预定为
  `D:/autostack/.wt/musk-093/auto-musk`，分支 `plan-093-dev`。
  禁止 junction／symlink。若修订后需要依赖 worktree，放在同组兄弟目录。
- 四阶段：new 交付合同 → work 实施并记录证据 → review 独立复验与绑定版本
  → merge 按仓库规则 rebase／快进合回，再沉淀模块规范、刷新派生 ledger、
  归档与清理。本文不代替后续评审或合回收据。

### 调查版本与权威来源

调查主仓库为 `main@9b93fe17f5b3646b373ceaa5ea7a62714702c8bc`，
工作区调查时干净；当前无匹配的在途编号计划，归档最大编号为 092。
依赖读证据版本：

| 仓库 | HEAD |
|---|---|
| auto-lang | 025fb192c0f768e9f818b0359a34e8fc23b0e924 |
| auto-ai | 5a50a55844d7aa3523b593f21ba0fb03d18eac48 |
| auto-down | 3373a5cc6e3a00336613133db51906fb0940777d |

`docs/specs/index.json` 与 `00-overview.md` 用于定位；后者及
`01-architecture.md` 部分内容仍为 2026-08-14 扫描快照，工具数量等旧描述
不能覆盖当前 app-canvas 模块规范。goals 索引中的旧“双落点”表述亦不改变
用户已同意的权威规则：**模块 docs/specs 承载当前知识，ledger 提供派生索引、
关系和历史视图。**

下表为本合同的关键规范快照，SHA-256 可用于复核；其余调查源码均受上述主仓
HEAD 固定。执行前若版本变化，T-01 对受影响结论重新核对。

| 来源 | SHA-256 |
|---|---|
| docs/specs/modules/app-canvas.md | dba0cc96d090c0b87b981a42f7e1845daa7ed816b704c36217067c4e10715423 |
| docs/specs/modules/workspace-ui.md | c1f856959440a463c1ed3bbff7d4a146434da86c9ddf9b3b3f60c7b959cdb812 |
| docs/specs/modules/vm-data-semantics.md | bcf486ce4eb9e40cbc266f9290e1b0ff299aa8dccc502d348c583e07fd28054b |
| docs/specs/modules/web-input-contracts.md | 030e6361e5abfad6bb8b267f1ed0d870ae076d328385c3d12eb9ac4beed7f235 |
| docs/specs/modules/chat-streaming.md | 832ed8e93bb9351ee33a6c65cb623a59e9158f471d82f26d59eafe9a7a1ae41d |
| docs/specs/modules/ui-parity.md | 648a9270866cd100cfafa2f731211ff66bfe320a4ed7a6c864923eb55d0b345c |

另已阅读 `docs/specs/modules/ui-compositions.md`，保留现有菜单、消息块、
composer 和侧栏交互约定。先前交互草图位于
`C:/Users/zhaop/.codex/visualizations/2026/09/09/01a08412-7116-7f20-9d7e-20c90f304bc6/auto-app-studio.html`
（SHA-256 4b97d646984d20fbbabd013c52e327847ac94f02f6b2957f54d7daab9a3789dc）。
草图仅提供视觉参考；本文包含完整合同，实施和验收不能依赖该本地附件存在。

### 当前能力与具体缺口

| 已核实位置 | 当前行为 | 本计划处理 |
|---|---|---|
| src/front/chats_view.at | 会话列表＋消息列＋CanvasPanel 第三列；已有 SendInput 与 MentionInput | 改工作台布局，复用现有聊天路径 |
| src/front/canvas_panel.at | 固定 520px，树 170px，源码底部 220px；字体低至 9px | 结构／源码移左侧；画布获得可用空间 |
| src/front/canvas_store.at | 1s Http.get_msg；非 stopped 每拍令 cv_open=true | 显隐与运行分离；身份及版本守卫 |
| 同上 ClearPick／StatusBackfill | 仅清本地；picked=null 未清字段 | 后端清选＋所有字段归零，禁止轮询复活 |
| src/front/canvas_web.ts | 以整张 img 盒换算点选；204 分支在 !response.ok 内 | 统一内容矩形；先识别 204，禁止把 null 当锚点 |
| 同上 installCanvasFrameClicks | document 全局委托按 URL 前缀匹配，缺少会话身份 | 使用受控视口事件／有卸载和身份守卫的端口 |
| src/front/ports/canvas.vm.at | start／pick／source 为 web-only 空桩 | 接通真实请求、帧与点选；作为必验项 |
| backend/.../canvas/{mod,manager,tools}.rs | 单例全局会话；未命中不清 picked；ToolContext 已有工作区／parent_conversation_id | 复用上下文建立归属和预览代次 |
| backend/.../vm_backend.rs；auto-src/vm_entry.at、server.at | VM AppState 有 CanvasManager，但 Canvas 路由未接通 | 补桥接，保持同一管理器和生命周期规则 |
| src/front/forge_store.at | Send／QueueMessage／FlushQueue 的队列为字符串 | 增加逐消息上下文快照及旧队列兼容 |
| src/front/ports/platform.vm.at | 聚焦 composer 为 no-op；窗口高度可从宿主 KV 读取 | T-01 验证原生聚焦与宽度／resize 能力 |
| ../auto-lang/schema/aura.at | Image 为 partial；ImageSurface 有媒体 ticket；mouse-area 有逻辑移动坐标 | 能力候选，不等同当前 PNG／点击坐标链已打通 |

### 要保留的既有规则

目标进程隔离、路径 confinement、10s 采帧超时、三次失败后的有界重启策略、
1s 帧采集、M3 三层生成策略、Blueprint 检查、ui_lint advisory、字符串 vnode ID、
VM handler 扁平重建及非阻塞 Http.get_msg 都来自当前规范。改变视觉布局不能
损坏这些规则；没有证据时不把同步状态或验证结果写成成功。

## 5. 详细设计

### 5.1 入口与主流程

1. 会话中增加“应用设计”入口。没有应用时进入空工作台，说明可以描述一个
   AutoUI 应用，并保留当前 composer；按钮只引导输入，不自动发送示例需求。
2. “打开已有应用”提供工作区内路径输入／选择，调用已有 app 校验与 start
   流程。越界、无 pac.at 或启动失败给出具体原因，不向未验证目录发起运行。
3. 当前会话的 canvas_run 成功建立预览身份后，可首次自动打开设计模式。
   同一代次用户收起以后，常规轮询／采帧不得强制打开；显式“打开画布”恢复。
4. 切换到其他会话或工作区时清空待发送的元素上下文，隐藏不属于当前会话的
   实况；原目标可以继续运行，回到所属会话可以重新打开。不自动杀进程。
5. 单实例正在被别的会话占用时，“打开画布”不显示为本会话应用；显示所属
   会话提示／返回入口。显式运行另一个应用才按现有替换规则停止旧目标。

### 5.2 布局、尺寸与视觉

| 窗口宽度 | 布局行为 |
|---|---|
| ≥1280px | 64px 全局导航；结构 220px；画布 flex；对话 360px |
| 1024～1279px | 64px 导航；结构 200px；画布 flex；对话 320px |
| 768～1023px | 默认画布＋对话；结构用按钮切换／受控面板；不叠加原会话列表 |
| <768px | 工作台内“画布／对话／结构”页签，单内容区；页签切换保持草稿与选择 |

以上为外层可用宽度预算，不把 480px 目标画面强行作为最小列宽。预览必须能
缩小以适应当前空间，100% 模式才允许画布内部滚动。窗口高度不足时，各内容区
独立滚动，composer 和顶栏保持可达；页面不横向溢出。

使用现有 theme 的 background／card／muted／border／foreground／primary。
深色默认维持沉静的深蓝灰层次，主色沿用 pac.at 品牌紫；选中框蓝色、Agent
高亮琥珀色、运行绿色、错误 destructive。状态同时有文本／图标，不单靠颜色。
不改变用户已选的浅色／深色主题。

顶栏 48px，画布工具栏约 36px；常规内边距 12～16px，控件圆角 6～10px，
边框 1px。主要文字 13～14px，树节点正文至少 12px，次要标签至少 11px；
状态和路径低对比但可读。普通按钮命中高度至少 28px，主要动作至少 32px。
不用强阴影、玻璃效果和复杂动画。

布局用共享 .at 结构和语义样式；仅 Web 平台能力需要的 DOM 测量、ResizeObserver、
图片加载回调可以留在平台门面。VM 用实际窗口／布局信息更新，不能硬编码
1280px 后宣称响应式完成。

### 5.3 生命周期、展示与状态矩阵

建议以整数／字符串保存 VM 敏感门控，视图 bool 只作展示投影：

| 维度 | 值／规则 |
|---|---|
| 展示 | 普通会话／设计模式；结构或窄屏页签；树折叠；适应／100% |
| 运行 | 继承 stopped／starting／running／restarting／degraded |
| 帧 | 无帧／加载中／已展示当前版本／等待新帧；绑定 generation_id＋seq |
| 生成 | 未开始／生成中／已结束／失败，由当前会话的真实工具事件投影 |
| 检查 | 未检查／检查中／通过／失败／不适用；lint、bp_check、live 验证分别记录 |

“收起”只改变展示，PID、代次、帧序号继续有效。“停止”调用 stop 并收割当前
目标，清选与失效帧；工作台保留应用标题、已停止说明和“重新运行”入口，
不得因 state=stopped 隐去全部操作。初始无应用与已停止应用是不同空状态。

启动／重启时显示原因和正在等待画面；没有新帧不显示“已同步”。运行中暂时
显示最后一帧时标记“等待新画面”，禁止交互与旧选择附带发送。
degraded 展示可执行的重新运行入口及简短原因；原始日志折叠在详情内。
前端不叠加自动重试计数器；继续沿用后端 1／2／4s、最多三次的策略。

### 5.4 预览身份与 API 增量

预览代次在每次显式 start／替换时变化；目标内部的 watchdog 自动重启保留归属，
同时清除失效帧／锚点并按新实际帧恢复。最低契约如下：

- status 增加 generation_id、owner_workspace_id、owner_conversation_id，
  frame 的 seq／像素宽高／逻辑宽高与帧有效性；原字段保持兼容。
- start 请求新增可选 conversation_id；工具从 ToolContext 取归属。
  旧无所属会话的调用为“未绑定预览”，不能自动当成任何当前会话的附件。
- pick 新增显式 `{clear:true}`，与坐标／vnode_id 互斥；未命中也清 picked。
  保留旧成功载荷及 204 未命中语义，前端把 204 规范化为“清选成功”。
- 工作台 start／stop／pick／frame 请求均携带预期身份；新客户端发送旧代次
  必须得到明确的冲突结果（409），不能对新应用执行旧操作。鉴权和路径校验
  仍按既有规则执行，归属字段不是认证替代品。
- 查询绑定到当前工作区；普通新客户端不接受别的工作区状态作为自身预览。
  工具访问同样校验其上下文归属，避免后台会话操作后来替换的应用。
- GET frame 增加代次与 seq 校验；只提供最新缓存时，旧 seq 返回冲突并要求
  重新取 status，不假装返回了所请求的历史帧。不建设历史帧服务器。
- 帧、锚点和 overlay 的发布须可确定地关联到同一 seq。前端收到 image
  loaded 后确认身份／seq 才激活对应框；迟到图像及 source 响应一律丢弃。

具体 wire 字段、错误对象和兼容策略在 T-01／T-02 的接口表固定并测试。
不能仅在 Vue 本地加 owner 标记，而让后端旧 stop 继续无条件杀新目标。

### 5.5 帧、比例与命中坐标

设容器内部可用尺寸为 Cw、Ch，帧物理像素为 Pw、Ph。适应模式
`s=min(Cw/Pw, Ch/Ph)`，实际图像尺寸 `Dw=Pw*s, Dh=Ph*s`，
偏移 `Ox=(Cw-Dw)/2, Oy=(Ch-Dh)/2`。点在容器内为 u、v 时，
仅落在内容矩形内才能映射为 `x=(u-Ox)/s, y=(v-Oy)/s`。

100% 指每个目标逻辑像素对应一个 UI 逻辑像素，显示尺寸由目标逻辑宽高计算，
不因目标屏幕 DPI 把画面放大 1.5／2 倍。点选 API 仍接收帧物理像素；实际
展示尺寸／帧尺寸的比例统一转换，不能混用逻辑 bbox 与 bbox_px。

- 图像留白点击只清选，不对目标应用发送 act。
- bbox_pct 叠层必须相对实际图像矩形；缩放、滚动、DPI 后仍对齐。
- Agent 框不截获指针；用户框和 Agent 框保持不同样式。
- 框只对已展示的版本生效；帧未加载、无索引或代次失配时禁止点选。
- 裁剪／滚动位置纳入命中坐标，不能用视口外层 rect 代替内容 rect。
- 优先构造共享内容矩形与纯换算 helper；DOM 与 VM 端口只输入测量和事件。
  若 Iced 不能可靠绝对叠层，T-01 比较现有 ImageSurface 或由共享几何生成
  展示覆盖层的最小方案，输出实证后固定实现；不得先写一套假定能力的组件。

### 5.6 组件树、详情与源码

树标题为“结构”，展示实际 vtree 的 kind 和可用 label。Blueprint 名称只有
真实来源数据存在时才显示，不能由类型猜测。循环实例显示 index／value；
vnode_id 始终为字符串，包含超过 2^53 的 ID 用例。

从 depth／父子关系预计算折叠后的可见列表。更新 label／框位置不清掉用户
折叠与滚动位置；节点删除才撤销该节点选择。行选中、图像点选、上下文 chip、
源码位置共享一个选中锚点。

左栏“结构／源码”切换；源码只读、有文件名与行号、高亮并滚动到有效 source
位置，不再从画布底部挤出 220px 抽屉。非法行号、读失败、无 source、文件位置
不确定分别显示原因。当前多文件来源启发式必须显示“来源待确认”，不得假装
已准确锚定；单文件有效来源的行定位是硬验收。

源码读取继续走工作区文件安全链，不能直接拼本地路径读取任意文件。跨代次
迟到响应不得覆盖当前选中元素的文件。

### 5.7 元素上下文与输入／消息队列

composer 上方展示紧凑 chip：“已选：Button · 新建任务”，可展开看文件、
行、循环实例等详情，并可移除。“在对话中修改”只聚焦现有输入框，不发送消息，
不覆盖用户已经输入的内容。VM 聚焦不能继续以 no-op 当成功。

发送采用可选结构化 `design_context`，与用户 `content` 分开：

```text
version, workspace_id, conversation_id, generation_id, frame_seq,
app_path, vnode_id(string), kind, label,
source_path?, source_line?, source_confidence?, loop_context?
```

- 在 SendInput 确认为普通消息后冻结上下文，斜线命令识别与 mention／Plan／
  Spec 展开规则不受上下文影响。没有选择时旧 API 请求保持兼容。
- 排队条目保存 text＋context＋归属；FlushQueue 重试 busy 时原样保留，
  不能等实际发送时才读取“当前选择”。旧字符串队列迁移为空 context。
- 后端校验 context 的归属、路径和字段；上下文是用户提供的定位数据，
  其中 label／源码片段不能提升为系统指令或批准任何工具操作。
- 回放保留用户原文和轻量附件信息。Agent 输入通过单独的上下文说明引用
  该快照，不把内部 JSON 塞进主消息气泡；不创建第二套会话历史。
- 选中元素后来失效／换帧时清除待发送 chip；已发送快照作为历史附件保留，
  标记历史定位。队列中已经冻结的旧定位不得静默换成新节点；发送前发现目标
  代次已失效时保留用户文字，提示移除附件或重新选择，不自动误发或丢弃文字。
- 切换会话／工作区清待发送选择，延续已有草稿和队列隔离规则。
- 必须覆盖中文 IME、Enter、Shift+Enter、mention 菜单、排队发送、
  run=true 单次孵化、approval gate 与 VM 轮询，不能为了新附件复制 composer。

### 5.8 生成与验证的可见进度

对话区顶部提供默认紧凑的进度摘要，细节可展开；工具卡仍留在消息流中，
避免重复铺满 Canvas。优先读取已有真实事件：

| 展示项 | 数据来源／何时算成功 |
|---|---|
| 理解／生成 | 当前会话实际运行／文件或 Blueprint 工具事件；未知时只显示“生成中” |
| 静态建议 | ui_lint 结果，advisory；不得称作硬性验收通过 |
| Blueprint 检查 | 使用 Blueprint 时的 bp_check 结果；未使用显示不适用 |
| 启动预览 | canvas_run 结果与当前代次实际帧；start 返回不等于已可见 |
| 交互验证 | canvas_act＋canvas_state／snapshot 的实际结果；未执行显示未验证 |
| 更新画面 | 热更新后收到并展示新 seq；没有因果证据只写“最近画面已更新” |

不得把工具完成顺序硬凑成每次必经五步，也不得仅凭消息里的“已通过”给绿色
勾选。映射以 tool_call_id 和预览代次关联；工具结果缺少必要标识时，在现有
输出中补充结构化元数据并保留旧文本，不另开轮询引擎。

### 5.9 性能、异常与交互细节

- 沿用 1s status／帧节奏；同代次 seq 未变不重复拉图。
- poll 单飞，响应丢失可按有界 tick 超时释放；VM 不用 Date.now 或已知不可靠
  bool／列表索引读取作活性守卫。区分视图收起与是否继续取状态／帧。
- 收起时可暂停前端下载新图，但目标采帧／进程继续；恢复时读取最新版本。
  无应用／空闲／非所属会话不得持续输出警告日志。
- 树数据按签名／版本变化更新；不因每次 status 回填重建整棵 UI 而丢折叠状态。
- loading／empty／error 不混用。网络失败保留能说明状态的旧信息并标识失联，
  不默认当 stopped 或同步成功；停止结果失败不能伪造已停止。
- 控件有中英文本、图标 tooltip 和禁用理由；Tab 能到达主要动作与输入框。
  展开浮层的 Escape 只关闭浮层，不停止 VM。帧选择模式不会截获输入框按键。
- 日志／截图不保存认证信息；测试使用独立配置、存储和端口，只收割自己创建
  的进程，不能按全机 auto.exe 数量或扫描端口批量杀进程。

### 规范增量

下列为拟议规则，review 核定最终补丁；merge 写回模块规范，ledger 仅派生刷新。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/modules/app-studio-ui.md | 新增会话内工作台布局、尺寸、视觉、入口、结构／源码／对话联动及非目标 | 为应用设计体验提供独立当前知识 | AC-01, AC-02, AC-07, AC-09, AC-13 |
| SD-02 | modify | docs/specs/modules/app-canvas.md | Web 面板＋VM 空桩、全局无代次 → 双端真实展示、归属／代次、清选、同版本帧与几何命中；保留有界生命周期／工具能力 | 消除错选、旧响应及停止串目标，兑现 VM 能力 | AC-03, AC-04, AC-05, AC-06, AC-08, AC-11, AC-12, AC-14, AC-15 |
| SD-03 | modify | docs/specs/modules/workspace-ui.md | 标准会话三段壳 → 普通会话沿用；设计模式暂收会话列表，独立布局和显隐，退出恢复 | 防止三栏加原列表造成拥挤 | AC-01, AC-02, AC-03, AC-13 |
| SD-04 | modify | docs/specs/modules/web-input-contracts.md；docs/specs/modules/chat-streaming.md | 纯文本 composer／队列 → 增量可选元素附件、冻结排队上下文、回放／失效规则；保留 IME、mention、审批及流式契约 | 元素定位可用且不破坏现有聊天 | AC-10, AC-11, AC-15 |
| SD-05 | modify | docs/specs/modules/ui-parity.md | 增加工作台生产组件 Gallery、真实 Canvas 双端／backend 模式／DPI／尺寸与版本收据要求；缺证据仍失败 | 不能把旧 Web 通过作为双端交付 | AC-05, AC-06, AC-13, AC-14, AC-16 |

SD-01 在规范索引注册；不要额外复制到 ledger 再单独编辑维护。

## 6. 测试设计

### 命令与证据约定

以下命令在后续 worktree 执行。V03／V04／V05／V07 中的新脚本／用例由本计划
创建，不能在实现前声称已通过。T-01 验证实际 CLI／生成链和依赖解析后写入证据。

| 编号 | 命令／工作目录 | 预期结果 |
|---|---|---|
| V01 | worktree 根：`auto build --gen-only --strict`；随后 gen/front/vue：`pnpm install`、`pnpm build` | Auto 生成与 Vue 类型／打包成功；无只改生成文件的补丁 |
| V02 | worktree/backend：`cargo test -p musk` | 单元／API／兼容测试成功，真实 VM ignore 项不能因此计为通过 |
| V03 | 根：`node scripts/ui-parity/canvas-contract.mjs`（新） | 调用生产 helper／store 的边界测试双端通过；不复制算法当被测对象 |
| V04 | 根：`node scripts/ui-parity.mjs run --plan 093 --case canvas-studio-pair`（新 case） | 同一生产组件 Vue／VM 快照、事件及截图均有证据 |
| V05 | 根：`node scripts/ui-parity/canvas-studio-live.mjs --plan 093 --frontend both --backend both`（新） | Vue／VM × RustHTTP／VMHTTP 实况验收，失败／缺证据非零退出 |
| V06 | 根：`node scripts/ui-parity.mjs check` | 生产单元及案例目录检查 0；不是运行态验收替代 |
| V07 | backend：`cargo test -p musk --test canvas_studio_contract`（新）；按需 `cargo test -p musk --test canvas_live -- --ignored --test-threads=1` | 新 API／归属测试及已有目标 VM 生命周期／M3 实测通过 |

现有 `scripts/ui-parity/live.mjs` 与 `check --live` 包含 PLAN-080 固定目录，
不能直接借其旧收据给 093 判绿。新 runner 必须输出自己的 093 收据，复用已有
启动／隔离 helpers 的能力时保持旧场景兼容。现有 canvas_live 的全机进程计数
不宜在有其他 Auto 进程的开发机直接用作孤儿判据；T-12 先改为本次拥有的
PID／进程树及起止收据，再做真实运行测试。

### 场景矩阵

| 场景组 | 重点输入／故障 | 证据 |
|---|---|---|
| 入口／布局 | 无应用、有应用、停止、收起／恢复；1440×900、1280×800、1024×768、900×700、640×720 | 双端截图＋可点击控件位置；无页面横向溢出 |
| 几何 | 480×680；目标 DPI 1／1.5／2；contain 横／纵留白、100% 内滚；边角／嵌套按钮 | 已知 bbox 选点和框边界，误差≤2 UI 逻辑 px |
| 选择 | 树／图点击、ID>2^53、未命中204、clear、节点删除、同 seq／新 seq | 请求和 picked 投影；等待至少3次 poll不复活 |
| 时序 | A→B 代次替换；旧 status／frame／pick／source／stop 人为迟到 | 无串帧、串文件、串选择或误杀；旧变更409 |
| 会话附件 | IME、mention、斜线命令、A元素入队后改选B、busy重试、刷新／回放、切工作区 | user原文不变；附件快照／归属正确；没有双跑 |
| 状态／生成 | lint警告、bp_check失败／不适用、spawn失败、等待帧、失联、崩溃重启／耗尽 | 真实事件投影；无假的同步／检查勾选 |
| 稳定性 | 5分钟运行／热更新／窗口resize；收起1分钟后恢复 | 单飞请求、旧响应丢弃、无刷屏、自己进程零残留 |

重要边界需要确定性测试；样式与尺寸采用真实渲染验收，不给每个样式类编写
实现镜像测试。网络延迟注入使用测试服务／拦截，不能修改生产语义只为测试通过。

### 证据落点（均为新）

- `docs/reports/093-app-studio-capability-map.md`：能力探针、API／平台决策。
- `docs/reports/093-app-studio-evidence.md`：AC 结果、截图索引、命令和失败修复。
- `docs/reports/093-app-studio-spec-delta.md`：SD 对照与拟议规范补丁。
- `tmp/ui-parity/PLAN-093/`：隔离配置、快照、截图、请求 spy、原始测试收据；
  tmp 被忽略，最终必要材料保存到报告引用的持久位置，不能归档后只剩失效链接。
- 收据包含 plan_revision、musk commit／工作树摘要、auto-lang／auto-down／auto-ai
  commit、运行模式、窗口及 DPI、实际结果、所有权 PID 和生成时间。
  rebase 后依技能重新绑定，必要时 range-diff 记录旧→新映射。

## 7. 验收标准

| ID | 可观察的验收结果 | 验证方法与预期 |
|---|---|---|
| AC-01 | 当前会话可进／出应用设计；无应用可输入需求或打开合法 app；首次运行可打开，手动收起不被 poll 复开；普通会话状态恢复 | 双端完整入口回放；草稿、会话和展开状态保持；非法目录明确失败 |
| AC-02 | 三栏与窄窗规则生效，消息／结构／画布各自滚动，composer始终可达 | §6五尺寸双端截图；无页面横溢出，640宽页签切换不丢状态 |
| AC-03 | 收起不停止进程；停止收割当前进程但保留已停止工作台和运行入口 | 记录同一PID收起前后存活；stop后该树退出；重新运行取得新代次 |
| AC-04 | 加载、等待、运行、更新、重启、失联、失败、停止可辨认；检查结果只来自真实证据 | 注入对应状态／工具结果；不出现伪造“已同步／已验证”；lint非硬门 |
| AC-05 | Vue与VM工作台都显示真实目标VM帧，并完成start／pick／source／stop；RustHTTP和VMHTTP入口可用 | V05四模式实测与截图；VM无web-only桩或静态截图替代；无静默backend回退 |
| AC-06 | 适应与100%下点选／覆盖框对准同版本实际图像，留白只清选 | DPI1／1.5／2＋内滚＋两类留白；命中预设节点，框误差≤2逻辑px |
| AC-07 | 树显示真实kind／label／循环实例，可折叠，更新不丢展开／滚动；大ID准确联动 | 双端点树↔点画面；>2^53字符串无截断；不得显示猜出的Blueprint归属 |
| AC-08 | chip关闭、留白未命中、节点消失均清后端与前端选择，旧响应不能复活 | clear后等待3拍poll；204不产生“null”锚点；框／chip／source同时复位 |
| AC-09 | 选中元素可在左栏打开只读源码，有效单文件行定位准确；不确定／非法来源给明示 | 双端源码行高亮及文件请求spy；越界失败；迟到旧source不覆盖当前文件 |
| AC-10 | chip不覆盖用户文本，修改按钮聚焦现有composer；即时和队列发送冻结context，命令／IME不变 | V03＋双端操作；A入队再选B仍保留A；busy重试不换附件；过期附件明确待用户处理 |
| AC-11 | 会话／工作区／预览代次隔离；旧stop／pick不能作用于新应用；上下文按归属校验 | A/B会话＋两个workspace＋乱序响应；旧代次409，新PID存活，用户文字不丢失 |
| AC-12 | 启动失败、失联、无帧和重启耗尽有明确恢复动作，stop失败不伪造终态 | 真故障／测试故障注入；后端重启最多3次，前端不追加无限循环 |
| AC-13 | 主题统一、文字可读、中英覆盖，主要动作键盘可达；状态不靠颜色，Escape不杀VM | 两种主题／语言双端截图；字体／命中区检查；Tab／IME／Escape实操 |
| AC-14 | seq未变不重下图片；poll单飞；窗口变化和5分钟采帧不丢选择折叠／错误串帧；收起恢复取最新帧 | V05请求统计＋5分钟收据；空闲不刷警告，UI线程无同步取图阻塞 |
| AC-15 | 现有会话消息、审批、mention、流式／VM轮询和M3生成／工具／路径沙箱保持行为 | V02/V07及相关生产场景回放；run守卫无双跑；目标仅收割本次进程 |
| AC-16 | 所有AC及SD有独立复验入口，双端证据绑定实际版本，规范权威边界明确 | V06与093报告对照无漏项；review可重跑；拟议delta只指向canonical docs，ledger为派生 |

## 8. 执行步骤

所有任务当前未开始。每项完成须记录实际命令、结果、代码版本与证据；仅写
“已完成”不能打勾。T-01 是后续工作的第一项，而非本轮已完成的运行探针。

### [x] T-01：建立工作树并验证平台能力／固定接口

- 依赖：无。关联 AC-02、AC-05、AC-06、AC-10、AC-16。
- 按 work 技能和 AGENTS.md 建／复用专用 worktree；依赖用 env／同组兄弟／
  主检出解析，不创建链接。记录四仓版本与实际 Auto CLI。
- 位置：现有 ports/canvas.*.at、canvas_web.ts、vm_backend.rs、
  auto-src/{server,vm_entry}.at、ports/platform.*.at；只读 auto-lang/schema/aura.at。
- 有界探针：最多两个最小生产能力原型，分别验证真实PNG／ImageSurface展示
  与指针＋叠层几何；同轮确认resize宽度／高度和原生composer focus接线。
  在隔离fixture跑Vue与VM；不重写渲染器来完成探针。
- 输出新 capability-map，列出已支持机制、失败证据、选定实现、API字段、
  backend Auto再生的实际命令与支持模式；执行一次V01／最小VM运行核对。
- 完成门：证明VM链可实现并固定下游任务；若缺少必须的上游能力，给出最小
  依赖合同并needs_replan，停止依赖该能力的任务，不降低AC-05／AC-10。
- [✅ 已完成] 证据（2026-09-29，commit d157387@plan-093-dev）：
  worktree `D:/autostack/.wt/musk-093/auto-musk`；capability-map
  `docs/reports/093-app-studio-capability-map.md`；双探针
  （`tests/ui-parity/probes/`，runner `scripts/ui-parity/canvas-studio-probe.mjs`）
  收据 `tmp/ui-parity/PLAN-093/probe-{a,b}-receipt.json`。
  探针A PASS：Http.request().send() 非阻塞二进制帧→body_to_file 字节忠实
  （428B 逐字节等）→open_session/current_uri 媒体 ticket→image_surface，
  park/resume 全程 UI 不冻结。探针B PASS：mouse-area(coords) 内包 image
  经 MCP drag 合成命中真实派发链（moves=3/clicks=1）；
  vm.window_inner_width/height 实窗 1024x768。
  上游缺口六项（G-1 ImageSurface onload 未接线、G-2 CSS-absolute 叠层
  hoist 丢树、G-3 dom.focus_first no-op、G-4 queue URI 无访问器、
  G-5 body_bytes 跨 park 丢 RC、G-6 合成通道 Double 位型错读）与最小依赖
  合同见 capability-map §4；T-03 定案 VMHTTP 桥走宿主 insert_http_response
  字节通道，帧运输定案 §3。Q-01/Q-02/Q-03 勘察结论落 §6。
  V01 预存阻塞（与本计划无关）：`auto build --gen-only --strict` 在 base 上
  即失败（S001 schema drift 5 条 + 静默 abort，两个 auto 二进制同症），
  解除动作见 capability-map §5；VM 运行时路径（auto run --render=vm）不受
  影响。needs_replan 不触发：无 AC 被删除或降门槛，G-1/G-2 仅约束 T-13
  不得以降级表述冒充 AC-04/AC-06 通过。

### [ ] T-02：补齐Canvas身份、清选和帧版本契约

- 依赖：T-01。关联 AC-03、AC-06、AC-08、AC-11、AC-12。
- 位置：现有 backend/.../canvas/{mod,manager,tools,anchor}.rs、
  tool_context.rs；新 tests/canvas_studio_contract.rs（crate测试目录）。
- 按§5.4增加归属／代次、原子发布帧与索引版本、expected身份检查、clear；
  清楚区分旧无身份调用的兼容行为和新工作台的强校验。
- 保留路径沙箱、单实例替换、已有watchdog、工具名称和成功文本兼容；
  工具调用使用工作区／会话上下文，不允许旧会话停掉新实例。
- 验证 V02／V07：旧请求兼容、未命中204后picked为空、乱序旧stop409且新目标
  存活、frame seq不匹配不伪造历史帧；同步更新capability-map接口表。
- [✅ 已完成] 证据（2026-09-29，commit 93bbccb@plan-093-dev）：
  契约实现 manager/mod/tools（generation/owner、begin_session、
  publish_frame/publish_anchor 唯一写点、stop_guarded、clear、frame_for、
  watchdog 复活清场）；接口表 capability-map §7。
  tests/canvas_studio_contract.rs 19 例全绿（代次冲突/归属停止守卫/未绑定
  语义/帧 seq·代次冲突不伪造历史/未命中清选/换代丢陈旧选/status 身份字段/
  路由 oneshot 409·clear 臂）。V02 全套 740 passed；唯一失败
  tool_atoms::run_command_dangerous_returns_paused 经 stash 在 base 复现
  （本机预存，与本计划无关）。V07（真实目标 VM 生命周期）按计划依赖归
  T-12 回补。环境事实：worktree 构建（check→test 序列在 Windows 产生
  metadata stub 中毒——test 前禁 cargo check；单测目标读 297MB rlib 偶发
  E0786 由杀软扫描竞态，--no-run 收敛重试可解；sccache 排除嫌疑但已禁用）。

### [ ] T-03：接通VM请求和真实媒体帧端口

- 依赖：T-01、T-02。关联 AC-05、AC-06、AC-09、AC-11。
- 位置：现有 ports/canvas.{web,vm}.at、canvas_web.ts、vm_backend.rs、
  backend/.../auto-src/{server,vm_entry}.at、src/back/api.at；
  新帧媒体适配文件只在T-01选定机制需要时创建并列入报告。
- 为start／status／pick／clear／source／stop建立双端非阻塞回填通道；
  VMHTTP桥使用同一CanvasManager，二进制PNG／媒体ticket有真实响应，
  不以打印”skipped”或固定stopped返回。
- JWT／workspace头沿用现有平台注入，避免重复query字段；加载图像携带身份
  与版本并回报loaded／failure；不在UI线程网络阻塞／decode。
- 验证 V01，V07及V05最小四模式：真实运行得到帧、树选后有锚点、读源码、
  显式停止；响应缺失、401和旧代次均有可处理错误。
- [▶ 进行中] 代码完成 + 接缝解除 + 首轮实机验证（未整体完成不勾）：
  原代码（a36413b：canvas_vm.at 五路由薄装配 + 宿主桥 canvas_*_host +
  vm_backend 注册 + extern_impl 五桥 fn；ports/canvas.vm.at 空桩退役）
  之上的本轮进展（commit 46b1173/8869f6a/18a0431@plan-093-dev）：
  ①阻塞②解除——auto-src 全语料适配 auto-lang Plan 545 链接语义
  （242 跨模块调用盘点、17 模块条目导入、wiki 方法体 3 名 shim；
  46b1173），VM serve 3601 路由启动实证；②VMHTTP 桥三处修复——宿主桥
  专职线程桥（嵌套 runtime panic）、五 handler 去 State 提取器（3 提取器
  触发 G-8 帧错位）、session.rs split_off(160) 越界（8869f6a）；
  ③五路由 curl 实证：status 200（T-02 身份字段全）/frame 503·409/stop
  200/pick clear 200/start 400 越界拒绝，0 panic；④ports 探针
  （probe-c + canvas-ports-probe.mjs）建立，VM 客户端 GET 链真实到达。
  contract 19/19 绿重绑当前树。
  **剩余阻塞（全部 auto-lang 上游，已实证登记 capability-map §8）**：
  G-8 .at 路由 handler 帧核算错位（start spawn 长宿主调用 RET 下溢崩
  VM）——VMHTTP start 实路径未通；G-9 VM 客户端 POST park 丢续体——
  canvasStart/Pick/Stop 四端口 VM 臂不可用，T-04 store POST 面同受限；
  G-10 MCP autoui_state 回读滞后——探针证据通道受限；V01 静默 abort
  维持（45min 构建 26m33s exit 1 无诊断）——T-05 起前端生成任务仍须先
  解除。解除动作与下游约束见 capability-map §8；V05 最小四模式待
  G-8/G-9 + V01 解除后回补，不得以部分证据冒充 AC-05 通过。

### [ ] T-04：重构CanvasStore投影与展示状态

- 依赖：T-02、T-03。关联 AC-01、AC-03、AC-04、AC-08、AC-11、AC-14。
- 位置：现有 canvas_store.at；新 canvas_helpers.at（共享生产投影／几何helper）。
- 将工作台显隐／前端偏好与cv_state分开；扁平重建tree／pick／overlay；
  picked=null、204、代次变化和stopped统一清场。保留停止后的应用描述。
- poll加入单飞及有界丢响应恢复；帧下载按身份＋seq，迟到回填无效；
  VM门控遵守vm-data-semantics和092的tick处方，不能照抄web Date.now。
- 验证 V03（随T-11补齐）及最小双端fixture：三拍不复选、收起不复开、
  seq不变零重复图请求、A→B迟到payload不污染B。

### [ ] T-05：实现工作台壳与响应式会话布局

- 依赖：T-01、T-04。关联 AC-01、AC-02、AC-03、AC-13。
- 位置：现有 chats_view.at、canvas_panel.at、content_header.at、nav_sidebar.at；
  新 app_studio.at（需要的壳组件）；平台宽度能力通过现有platform端口扩展。
- 加模式入口／空状态／打开现有应用；按§5.2宽度预算布列和窄屏页签。
  使用现有消息＋composer，保持一份实例或等价可证明的完整状态保留。
- 保存／恢复原会话侧栏偏好；顶部应用标题用pac.title、name、basename兜底，
  长路径只在详情内显示；收起／停止分开。
- 验证 V01／V04；五尺寸resize与模式切换，页面无横溢出，草稿／展开／
  审批卡／输入焦点状态不丢。

### [ ] T-06：统一画布几何、比例、事件和覆盖层

- 依赖：T-03、T-04、T-05。关联 AC-05、AC-06、AC-08、AC-14。
- 位置：现有 canvas_panel.at、canvas_web.ts、ports/canvas.*.at；
  新 canvas_helpers.at 中实际内容矩形／点映射函数。
- 实现适应／100%及内部滚动；使用真实帧和目标逻辑尺寸；图像loaded版本
  与框版本匹配后才启用选择；留白clear、overlay不拦指针。
- 移除或收敛document全局URL点击委托，监听必须有作用域／卸载与身份守卫；
  Vue／VM用同一换算规则，不能分别猜bbox坐标。
- 验证 V03与V05几何子场景：DPI1／1.5／2、两种留白、边界点和滚动；
  命中一致、框≤2px，旧frame加载完成不切回旧代次。

### [ ] T-07：实现组件树、选中详情与源码定位

- 依赖：T-04、T-05、T-06。关联 AC-07、AC-08、AC-09、AC-13。
- 位置：现有 canvas_store.at、canvas_panel.at、canvasLoadSource门面；
  新 canvas_structure.at、canvas_source.at（生产组件）。
- 从实际vtree生成可折叠列表，存稳定字符串ID折叠键；循环实例标签、
  选择样式与滚动保持；结构／源码切换取代底部抽屉。
- 源码按有效行高亮并定位；长文件滚动；来源不确定、读失败、非法行号与
  无source有明确状态；不引入直接写源码／属性编辑。
- 验证 V03／V04／V05：大ID、父节点折叠、节点删改、单文件有效行定位、
  多文件不确定提示、路径越界、旧source迟到等场景。

### [ ] T-08：接入元素附件与逐消息队列快照

- 依赖：T-01、T-02、T-04、T-05、T-07。关联 AC-10、AC-11、AC-15。
- 位置：现有 chats_view.at、mention_input.at、forge_store.at、
  src/back/api.at、backend/.../auto-src/{chats,server}.at、
  src/auto_generated/extern_impl.rs（手写glue）、ports/platform.*.at。
- 增量可选design_context DTO与持久化／回放；旧会话文件用默认值读取；
  用Auto模型及实际生成链更新转译结果，glue只负责必要桥接。
- chip位于现有composer上方；移除／聚焦不改文本；SendInput先解析命令；
  队列条目text＋context逐条冻结，busy重入不换附件，失效附件明确待处理。
- 保持消息原文；Agent上下文带数据来源及历史定位说明，校验归属／路径，
  不提升来自label／源码的文字为系统权限。
- 验证 V01／V02／V03及双端输入场景：IME／mention／斜线命令、A→B选中、
  busy重试、刷新回放、工作区切换；run=true只孵化一次，原生focus真实生效。

### [ ] T-09：呈现真实生成进度与运行／恢复状态

- 依赖：T-02、T-04、T-05、T-08。关联 AC-03、AC-04、AC-12、AC-15。
- 位置：现有 canvas_panel.at、canvas_store.at、forge_store.at、
  chat_message.at、canvas/{tools,bp_tools,ui_lint}.rs；
  新 app_studio.at中的摘要区域或必要的独立生产组件。
- 用tool_call_id／归属代次关联实际结果；生成、lint、Blueprint、预览、
  交互验证分别记录；没有工具结果保持未检查，不做推测的绿色完成条。
- 空／停止／starting／restarting／degraded／失联使用§5.3状态矩阵；
  失败详情可展开，运行／重试按钮带禁用理由，前端不新增重启循环。
- 验证 V03／V05故障场景及V07：lint警告不是硬门通过、bp失败明示、
  无帧不能同步成功、stop失败不伪造、重启耗尽后可显式恢复。

### [ ] T-10：收敛共享主题、中英文本和键盘交互

- 依赖：T-05、T-06、T-07、T-08、T-09。关联 AC-02、AC-13。
- 位置：现有 pac.at主题（仅必要语义扩展）、src/front/i18n/{zh,en}.json、
  inject_styles.web-only.ts（限Web适配）与相关生产.at组件。
- 按§5.2统一颜色、字号、间距、边框和命中区；全部新增状态、tooltip、
  按钮与错误提示中英覆盖；详情不暴露认证信息。
- Tab／Enter／IME／Escape行为可用，禁用选择时不会截获composer输入；
  Escape关闭展开详情／浮层，不调用stop。
- 验证 V01／V04＋双端两主题两语言实操；确认字号下限和所有主要控件可达。

### [ ] T-11：补齐确定性合同测试与Gallery用例

- 依赖：T-02～T-10（实现每个任务时先加必要边界测试，本项完成汇总）。
  关联 AC-01～AC-14、AC-16。
- 新 scripts/ui-parity/canvas-contract.mjs；
  现有tests/ui-parity/cases.json／fixtures及Gallery生产消费入口。
- 覆盖clear复活、204、代次／seq乱序、几何边界、大ID、折叠保持、
  source迟到、context冻结／队列busy／失效、状态真实性。
- 使用实际helper／store／端口响应；VM用实际Auto模块探针／生产组件消息，
  不另写相同算法作为被测实现。Gallery引用工作台生产组件，不复制模板。
- 验证 V02／V03／V04／V06：双端断言与事件spy有结果，缺运行证据失败，
  新单元全部入目录，旧Gallery案例保持通过。

### [ ] T-12：复验真实目标生命周期、工具与会话回归

- 依赖：T-02、T-03、T-08、T-09、T-11。关联 AC-03、AC-11、AC-12、AC-15。
- 位置：现有 backend/tests中的canvas_live.rs、parity_chats.rs及新
  canvas_studio_contract.rs；具体路径为backend/crates/musk/tests/。
- 首先把本次测试的进程所有权记录／孤儿判据做成PID树级；不受别的在用VM
  影响，也不能杀掉别人的进程。测试workspace／storage／端口完全隔离。
- 跑实际M3生成fixture（非真实付费模型依赖）：lint→bp_check→canvas_run→
  act／state／snapshot；验证sandbox、崩溃恢复、有界重启、停止清理。
- 同步回归会话守卫／流式或VM轮询、审批门、队列、附件回放和旧JSON兼容。
- 验证 V02／V07，保存命令、工具实际结果和自己创建的PID起止收据；
  ignore测试未实际运行不能算完成。

### [ ] T-13：完成双端实机、多尺寸、DPI与稳定性验收

- 依赖：T-10、T-11、T-12。关联 AC-01～AC-16。
- 新 scripts/ui-parity/canvas-studio-live.mjs，复用现有live隔离能力但不读取
  PLAN-080旧收据；新docs/reports/093-app-studio-evidence.md。
- V05执行四模式关键链；在Vue和VM执行§6全部尺寸、主题／语言、
  故障／乱序、DPI几何及5分钟稳定性场景。默认生产模式做完整用户流程，
  兼容backend模式至少覆盖真实帧、选择、source、stop和附件发送。
- 每项AC写实际结果及截图／snapshot／请求／PID证据；标注宿主DPI、逻辑
  尺寸、版本；只启动自己的隔离实例，不改共享服务配置。
- 完成门：所有AC可复验且通过。若依赖能力未兑现，回到明确修订／修复，
  不把VM缺口记为“尽力项”后打勾。

### [ ] T-14：整理规范增量与交付独立review

- 依赖：T-13。关联 AC-16，覆盖SD-01～SD-05。
- 新docs/reports/093-app-studio-spec-delta.md：逐条列出当前规则→新规则、
  相关实现／证据、拟议canonical模块补丁和index注册。
- work阶段不先改canonical知识／ledger来绕过review；交review技能复验
  PLAN-093当前revision及实际代码，检查acceptance、Spec delta、旧功能回归。
- 运行V01／V02／V06及此次最终改动真正影响的检查，不无理由全量重复；
  标记新旧证据是否因后续代码变化失效。
- 完成门：任务／AC／SD覆盖表完整、证据绑定待评审版本，必要截图已持久化；
  work可记录execution_done，但不自行给review pass或进入merge。
  后续合回按AGENTS.md的guard／rebase／ff-only／清理规则执行。

### 任务覆盖与依赖检查

| 任务组 | 验收覆盖 | 规范增量 |
|---|---|---|
| T-01～T-04 | AC-01,03,04,05,06,08,09,10,11,12,14,16 | SD-02,SD-03,SD-05 |
| T-05～T-07 | AC-01,02,03,05,06,07,08,09,13,14 | SD-01,SD-02,SD-03 |
| T-08～T-10 | AC-02,03,04,10,11,12,13,15 | SD-01,SD-02,SD-04 |
| T-11～T-14 | AC-01～AC-16（复验与证据） | SD-01～SD-05 |

## 9. 复审记录

### work 阶段 T-03 接缝解除与首轮实机验证交接

- stage: work
- plan_id: PLAN-093
- plan_revision: 1
- outcome: T-03 推进（阻塞②解除 + 桥修复 + 五路由 curl 实证）；
  T-03 整体验证仍 blocked（G-8/G-9/V01 上游）；整体保持 executing
- code_commit: 46b1173（语料接缝）+ 8869f6a（桥修复+探针）+
  18a0431（capability-map §8）@plan-093-dev
- task_ids: T-03（进行中）
- evidence: capability-map §8（本轮全部记录）；contract 19/19 绿
  （8869f6a 树）；VM serve 3601 路由启动；五路由 curl 要点录
  capability-map §8.3（临时收据 /tmp/vmret9.log 易失）；V01 全量构建
  26m33s exit 1 无诊断（静默 abort 维持）。依赖版本：组内
  auto-lang@auto-musk-dev ec5adb7af（运行时同探针版 e2deb4f）。
- blockers: ①G-8 .at 路由 handler 帧核算错位（VMHTTP start 实路径崩
  VM）②G-9 VM 客户端 POST park 丢续体（canvasStart/Pick/Stop 端口
  VM 臂 + T-04 store POST 面受限）③V01 静默 abort（T-05+ 前端生成
  前置）④G-10 MCP 回读滞后（证据通道）。四项均 auto-lang 上游依赖
  任务；PLAN-095（drafting）覆盖 V01 与 G-1/2/3/6，G-7/8/9/10 为本轮
  新实证，需并入 095 合同修订或另立依赖任务。
- next: work——T-03 验证回补待上游解除后补 V05 最小四模式；不受阻的
  下游任务按依赖序评估开工资格（T-04 的 POST 面受 G-9 约束、T-05 起
  受 V01 约束）。

### work 阶段 T-02/T-03 交接

- stage: work
- plan_id: PLAN-093
- plan_revision: 1
- outcome: T-02 pass；T-03 代码完成、实机验证 blocked（base 预存破坏）
- code_commit: 93bbccb（T-02）+ a36413b（T-03 wip）@plan-093-dev
- task_ids: T-02（完成）、T-03（进行中）
- evidence: tests/canvas_studio_contract.rs 19 绿；V02 全套 740 passed
  （唯一失败经 stash 验证为 base 预存）；capability-map §7 接口表 +
  环境事实（worktree check→test 中毒、杀软读竞态、sccache 禁用、依赖
  worktree 组内解析）。
- blockers: ①V01（auto build --gen-only --strict）base 即失败（S001
  schema drift + 静默 abort）——T-05 起前端生成任务开工前必须先解除；
  ②MUSK_BACKEND=vm serve base 即启动失败（auto-lang master 前进致
  extern_sigs 跨模块符号解析失效）——T-03 实机验证与 T-13 四模式矩阵
  被阻断。两项均为 auto-lang↔musk 接缝依赖任务，证据与解除动作见
  capability-map §5 与计划 T-03 条目。
- next: work（T-03 验证回补后 T-04；两项接缝对齐待用户/auto-lang 侧排期）

### work 阶段 T-01 交接

- stage: work
- plan_id: PLAN-093
- plan_revision: 1
- outcome: pass（T-01 单任务完成；整体保持 executing）
- code_commit: d157387（plan-093-dev，worktree D:/autostack/.wt/musk-093/auto-musk）
- task_ids: T-01
- evidence: docs/reports/093-app-studio-capability-map.md（worktree）+
  tmp/ui-parity/PLAN-093/probe-{a,b}-receipt.json；探针双 PASS，六项上游
  缺口与依赖合同、VMHTTP 桥定案、帧运输定案全部落报告。V01 strict 门为
  预存阻塞（base 复现，证据 capability-map §5），不阻断已开工面；T-05 起
  触前端生成的任务须先复测。
- blockers: 无阻断性待决；上游依赖任务（G-1/G-2/G-3 优先）待与 auto-lang
  侧排期，本计划继续 T-02（不依赖缺口能力）。
- next: work（T-02）

### work 阶段启动（T-01 前）

- stage: work
- plan_id: PLAN-093
- plan_revision: 1
- outcome: executing（进行中，非终态）
- 用户于 2026-09-29 明确授权实施本计划，work 自 drafting 进入 executing。
- worktree：`D:/autostack/.wt/musk-093/auto-musk`，分支 `plan-093-dev`，
  base commit `f8f99f3dc066b557b4c5b003dd2e0d3faa8bb428`；主检出预检无代码 WIP。
- 依赖版本（启动时）：auto-lang `e2deb4f879c27cf4e34597363f9866fe4bcf7ae2`（master）、
  auto-ai `5a50a55844d7aa3523b593f21ba0fb03d18eac48`（main）、
  auto-down `3373a5cc6e3a00336613133db51906fb0940777d`（master）。
  auto-lang 相对 §4 调查版 `025fb192` 已前进，T-01 需对 aura.at 能力结论重新核对。
- 依赖 env 覆盖（AUTO_LANG_ROOT 等）未设置；解析序走组内兄弟→主检出，不建链接。

### new 阶段草案交接（不是实现评审）

- stage: new
- plan_id: PLAN-093
- plan_revision: 1
- outcome: pass
- next: work
- changed_tasks: T-01～T-14（新增）
- changed_acceptance: AC-01～AC-16（新增）
- spec_deltas: SD-01～SD-05（拟议）
- 交接依据：当前规范／代码／平台门面已核对；现有路径与新路径分开；
  任务、验收和规范增量全部有覆盖。原生能力未实测的部分由T-01有界探针
  先处理，不假设VM桩已经可用，不阻碍先开展该探针。
- 授权边界：本轮交付计划，不自动执行work。next是流程交接点，不是
  Relay已解析或用户已批准自动续跑的声明。
- 编号落盘：创建前复扫主／归档目录；采用独占分配互斥及CreateNew无覆盖
  写入，并在创建后检查编号唯一。当前应用create API本身缺少跨写者分配锁，
  该既有问题不在本UI计划中悄悄扩展为后端编号重构。
- 本记录不宣称任何代码、VM帧、截图或AC已经通过；后续review需独立证据。

## 10. 待澄清事项

当前无必须由用户先回答才能撰写或开展T-01的产品问题。以下是有责任人与
下一步的技术调查项，不能当成已经验证的能力：

| 项 | 默认设计／待确认内容 | 责任人与下一步 | 影响 |
|---|---|---|---|
| Q-01 | VM现有媒体链能否消费鉴权PNG，或需ImageSurface ticket；真实click／叠层是否可用 | work执行者，T-01两个有界原型及版本证据 | 决定T-03/T-06端口；缺上游能力则修订仓库范围 |
| Q-02 | VM窗口宽度／resize与composer原生focus可用接线 | work执行者，T-01读取实际宿主能力并实机验证 | AC-02/10不能靠固定宽度或no-op降级 |
| Q-03 | Auto后端模型／glue再生链与VMHTTP Canvas二进制响应机制 | work执行者，T-01固定真实命令、T-03验四模式 | 禁止只改生成结果或静默回退RustHTTP |
| Q-04 | 多文件source仍启发式；准确性如何标记 | work执行者，T-01/07确定confidence来源，缺数据按待确认显示 | 本计划要求准确单文件定位，不承诺无证据的多文件精确映射 |
| Q-05 | 上游修复若不可避免 | work执行者提交最小依赖任务、受影响AC和新版本；按new技能修订合同 | 新增仓库／兼容承诺须明确，不能直接改依赖主检出 |

产品默认裁定已写入合同：会话内模式、单实例、480×680固定目标窗口、
用户点画面只选元素、没有可直接编辑的属性面板。若用户后续调整这些目标，
保留本草案和证据，递增plan_revision并标记受影响验证，而非悄悄改验收门槛。

