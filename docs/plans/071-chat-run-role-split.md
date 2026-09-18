---
plan_id: PLAN-071
status: executing
feature_name: chat 一句话双回答回归修复（SSE 订阅与运行主体显式角色分离）+ 可用性修改需求跟踪
author: zhaop / zcode
created_at: 2026-09-17T22:50:00+08:00
updated_at: 2026-09-18T20:10:00+08:00
plan_revision: 16
current_step: 60
total_steps: 60
supersedes_spec_components:
  - docs/specs/modules/chat-run-policy.md
new_spec_components:
  - docs/specs/modules/chat-agent-identity.md
touched_goals: [goal-relay]
---

# PLAN-071 — chat 双回答回归修复（运行主体/订阅显式角色分离）+ 可用性需求跟踪

## 会话交接摘要（2026-09-18，供新会话接续）

**状态**：15 个需求全部实施完毕（T-01..T-61 全勾，AC-01..40 就绪），分支
`plan-071-dev` 领先 main（需求②-⑮未落地，需求①已随 phase-1 于
3c2b178 落地 main）。**下一步 = 用户终验 → /auto-plan:review → /auto-plan:merge**
（合并时随 SD-01/02/03 发布 specs + ledger 挂载 + worktree/依赖快照清理）。

**环境**：worktree `D:/autostack/.wt/musk-071/auto-musk`（分支 plan-071-dev，
与 main 的同步方式 = 在 worktree 内 `git merge main`）；依赖快照
auto-ai@9d2102c（`.wt/musk-071/auto-ai`）+ auto-lang@844ff9c81（`.wt/
musk-071/auto-lang`，曾因外部清理丢失后按原提交重建，detach 挂载）。
**运行态**：musk serve 在 http://127.0.0.1:8090（worktree 产物 +
MUSK_WEB_DIST 指 worktree dist；8080 已被无关进程 jade-garden-back 占用）；
aaid 在 17654。前端部署后需整页刷新（构建产物非哈希文件名 + SPA 长驻页面，
登记为已知事项）。

**需求一览（全部已完成待复审）**：①chat 双回答回归根修（角色分离+双发
接线+订阅收束）②Agent 职业身份（profession_id 三落点+前端头像）③UI 三条
（70% 对称/工具卡标题左对齐/收缩态侧边栏+鹿logo+24px）④二级菜单四条
（截断/×移二行/首盒gap/规范盒统一+计数）⑤挂死运行防护（看门狗/命令超时/
busy指示/cancel端点/生命周期日志）⑥流式状态可见性（常驻状态行，后按用户
裁定删除冗余组件由 ThinkBlock 半开态独占）⑦思考块沉淀与渲染修复
⑧流式 UX 六步规格（乐观标题栏/半开态/回填防清除）⑨流式等待点组件+
停止按钮归位工具栏（T-42 顺带修复 cancel 端点未接真实路由的 T-25 缺陷）
⑩思考档位默认低（未设置即 low 生效，显式关持久化 off）⑪计划二级导航
两行盒改造（session-item 同构 + 状态 badge 副标题）⑫计划页默认选中
第一篇并加载内容（高亮以 store.current 为准）⑬@ 提及三源扩展（Agent/
计划/规范，@plan/@spec token + 运行时解析注入）+ 下拉左对齐修复
⑭@ 交互收敛（默认选中第一项/Enter 补全优先不发送/引用 token 气泡内联
组件；composer 内着色留上游改造，登记）⑮运行在途消息排队自动执行
（替代拒绝+请重发；连带 ChatStore 写互斥修复并发丢写）。

**关键教训（新会话必读）**：① `auto build` 可能因二进制锁/中断静默跳过
codegen 或 vite 阶段——构建后必须核对生成产物（grep 标记类串）再验证；
② .at 转译器剥冗余括号（+/- 混合运算优先级错位→TS 报错）且无 list.insert
透传（TS2339）——用中间变量/重建数组规避；③ .at 的 button 全部编译为
脚手架 Button（含 [&_svg]:size-4），store 别名仅支持 `store` 单名；④
生成代码的两处同型锚点（agent_run_stream vs chat_run_owner）易误插——
补丁后必须编译+定点核对；⑤ worktree 前端验证直接用 IAB 面板 + DOM
采样（EventSource 钩子包装 addEventListener 不含 onmessage——应用走
onmessage，计数需另行包装）。

## 0. 变更摘要

会话 `18683b29…`（workspace=auto-edit）实测：**用户发一句话，收到两条助手回复**
（seq 1/seq 2 相差 6 秒、措辞近似——同一轮跑了两遍）。根因已实证为 PLAN-069
F-03 根修（提交 f0677cd）的**守卫谓词反置回归**：

- `chats_message {run:true}` 先 `chat_run_try_start` 占守卫、再 spawn `chat_run_stream`
  （运行体 A）；
- 前端 EventSource 1ms 后 `GET /stream`，`chat_stream` spawn **同一个**
  `chat_run_stream`（运行体 B）；
- `chat_run_stream` 用 `chat_run_active` 只读窥探分流：`if !run_owner { 附加; return }`
  ——守卫在途时两路都跳过附加块、都往下执行 agent。**窥探共享守卫在结构上无法
  区分"POST 孵化的主体"和"后来的订阅者"（两者看到的守卫状态相同）**，f0677cd
  只是把 F-03 前的"订阅抢守卫变身主体"挪成了"订阅在途即并行再跑"。
- 佐证：f0677cd 提交信息自记"正路径（run=true）……待干净会话复测"；现有测试
  `ag_chat_stream_persists_and_streams`（复审 F-05 口径）"先 try_start 抢守卫再
  订阅、断言订阅方运行"，恰好把缺陷语义钉成绿灯。

修复方向：**显式角色分离**——运行孵化唯一入口保持在 `chats_message run=true`
（try_start → spawn 运行主体函数，持守卫、出口清理）；`GET /stream` 与 VM 桥的
`chat_run_stream` 恒为**附加/空闲订阅**，绝不孵化运行。与
`docs/specs/modules/chat-run-policy.md`「订阅即附加」「订阅绝不孵化运行」原文对齐
（当前代码违反规范，规范为权威）。

本计划同时作为**可用性修改需求的跟踪计划**：后续用户实测暴露的修改需求以新
T-/AC-/SD- ID 追加进来（计划文档共享流程状态，留在主检出），按
`plan_revision` 递增语义修订。首个需求即本次双回复修复。

**需求②（2026-09-17 用户追加，r3）**：AI 回答不区分是哪个 Agent 在回答——
auto-forge 有完整设计（AgentConfig：name/profession_id/avatar_url + 消息携带
profession_id + 职业色头像），auto-musk 迁移时简化掉了。用户裁定：**不同类型
agent 的身份标识不能简化掉**。调查结论：部件大半在场——后端
`~/.config/autoos/professions.json` 有完整花名册（assistant/advisor/architect/
planner/tester/coder/reviewer/documenter/gofer + super 三变体，12 职业），
`/api/forge/relay/professions` 端点在场，前端 `AgentAvatar` 组件（职业 HSL 色 +
首字母 fallback）与 `AgentConfigs` store（MentionInput 挂载时 Init 拉取）已移植
（mention_dropdown/secretary_message 在消费）。缺的链路：①消息模型无身份字段；
②三个 assistant 落盘点不填身份；③chat_message.at 头部渲染写死 `"🤖 AI"`。
mode→role 映射在场（superpowers→"assistant"，modes/*.at）。

**需求③（2026-09-18 用户追加，r4，随长任务验证截图提出）**：时间线顺序确认
无问题后，三条 UI 改进：
1. **消息宽度对称**：AI 回复不再占满全宽，与用户消息对称限宽 70%（用户侧
   现为 85%，用户感知约 70%——两侧统一 70% 达成真对称）。
2. **工具卡标题栏**：操作对象（如 write_file 的 summary.md）从靠右改为
   **紧跟工具名左对齐**（中间留 gap）；status+chevron 保持靠右。
3. **侧边栏收缩态**：导航栏加收缩 icon；收缩后全部内容 icon 化且**放大到
   32px**（现 icon 20px 配合文字）；AutoMusk 标题 → 虚拟桌面配置的小鹿 icon
   （auto-os assets/icons/{dark,light}/auto-musk.png，512px 彩色插画，明暗
   两变体需随主题切换；hover 显示 "Auto Musk v0.1.0"，点击展开）；
   workspace 选择器 → 文件夹 icon（hover 显示当前 workspace 目录，点击弹出
   与现款相同的下拉）；设置 → 齿轮 icon。
   技术判定：icon 元素 codegen 支持 `size: 32`（px 内联样式）；`img` 标签
   原生映射；主题经 `html.dark` class 翻转（useVisualStore 实证）→ 双 img +
   inject_styles 全局 CSS 切换（VM 轨 inject_styles 不跑，light 变体默认
   inline display:none 兜底单图）。

**需求④（2026-09-18 用户追加，r5，随二级菜单截图提出）**：二级菜单四条改进：
1. 会话 item 标题过长时突出 box 右缘——改单行 `...` 截断 + hover（title 属性）
   显示全名（现 break-all 换行形态在该场景失效）。
2. 会话 item 的 hover `×` 删除钮从标题行（absolute 右上）移到**第二行右缘**
   （与 "n 条" 同行）。
3. 二级菜单第一个 item box 顶部无 gap——列表容器补顶部间距（会话 + 规范两处）。
4. 规范二级菜单 item box 与会话统一（session-item 盒形态：左对齐标题 + 第二
   行计数），并显示各类型规范文档数（数据源 = store.document.sections 经
   specSectionItems 计数；概览盒不计）。

**需求⑤（2026-09-18 用户报告会话 64f0076c 零回答，r6 立项）**：长任务消息发出
后 AI 一直无回答（发送 09:34:54、报告 ~09:36，等待约 1-2 分钟——初诊误判
"5 小时"，以 git/日志时间戳更正为**即时卡住**）。诊断（日志+探针+DOM 实证）：
09:34:54.564 run 孵化成功（单组 registry 日志）后**一分钟内零产出、永不收束**
——运行体挂死，per-session 守卫被其占住；期间所有新消息被 chats_message
**静默吞掉**（try_start 失败不孵化也不回报），且 Err 收束臂只发瞬态 SSE 不落
盘——用户侧永远零反馈。即时卡死形态 → 头号嫌疑 = **首个 LLM 请求流 stall**
（无读超时即永挂）；工具审批门/长命令需先有模型响应，概率次之。处理：重启
serve 清内存守卫解锁（未持久化的挂死运行工作丢失），探针会话 6s 收到回复确认
通道恢复。修复面五条：
1. **LLM 流读超时**：流 stall 超时（如 120s 无字节）→ run 报错收束 + 守卫
   清理 + **错误消息持久化**（当前 Err 臂不落盘 = 失败对用户不可见）。
2. **run_command 执行超时**：挂起命令（dev server 类永不退出）可配置超时 +
   部分输出回灌，不再永挂运行。
3. **守卫占用可见**：run:true 遇守卫占用时不再静默吞——响应带 busy 指示，
   前端显示"运行中…"。
4. **取消运行**：run_stream 的 cancel flag 接线为 POST cancel 端点 + UI 停止
   按钮（现无任何取消途径，只能重启 serve）。
5. **运行生命周期日志**：spawn/finish/fail（含原因）打 INFO——本次诊断只能靠
   registry 加载日志倒推，太隐晦。

**需求⑨（2026-09-18 用户追加，r10，随流式截图提出）**：流式等待态与停止按钮
归位，两条：
1. **等待态组件**：发送后 AI 正在启动（首个 thinking/delta 到达前），
   assistant 气泡内应显示类似 "..." 的动态等待组件，告知用户后台正在干活
   （截图现状 = 空白气泡，仅身份头部 + 工具栏，无任何活动反馈）。
2. **停止按钮归位**："停止"按钮从消息画布底部的独立按钮移入 **AI 回答气泡
   底部工具栏**（与复制/分叉 icon 并排），仅流式中的消息显示。
3. 用户同时确认：流式 think block 与 markdown 文本 block 已正常显示
   （需求⑧回验通过，该面无需改动）。
技术判定：等待窗口 = `isMsgStreaming` 为真且 `messageDisplayBlocks` 为空
（乐观气泡 T-34 保证流式期最后一条消息恒为 assistant，窗口判定可靠）；
三点跳动动画走组件 style 块（`@keyframes` + `:nth-child` 延时，
relay_run_box.at rb-spin / streaming_table.at nth-child 先例）；停止钮直调
`ForgeStore.CancelRun()`（ToggleThink 跨件直调先例）。

**需求⑩（2026-09-18 用户追加，r11，随档位截图提出）**：思考档位默认值从
"关"改为"低"——现状发送前要手动把档位从"关"切到"低"。语义裁定：**未设置
（None/空串）一律按 "low" 生效与显示**；显式关闭改为持久化 "off"（原"关"
菜单发 ""，被后端 set_thinking_level 归一成 None=跟随默认，与未设置同值、
无法真正关闭——需求⑩起 "off" 为显式关闭，未设置即低）。三处协同：
①后端运行路径 None → `or_else(Some("low"))`（extern_impl 手改先例区）；
②前端四处回填/初始 `?? ""` → `?? "low"`（store 初值/NewSession/
SwitchSession/BranchTo）；③档位菜单"关"项 `pick("")` → `pick("off")`。

**需求⑪（2026-09-18 用户追加，r12，随计划页截图提出）**：计划二级导航
改造——现状是普通单行按钮（"001 — bootstrap-auto-edit […"截断形态），
改成与"会话"/"规范"同款的 Sidebar 两行盒（session-item 形态）；每个
item 加副标题行，用 **badge 展示计划文件的状态**（draft/executing/
execution_done/reviewed/archived 等后端原值，样式对齐详情页 meta badge
`px-2 py-0.5 rounded-md text-xs bg-accent`）。标题行保留 truncate +
hover（title 属性）全名。

**需求⑫（2026-09-18 用户追加，r13，随截图提出）**：计划页打开时列表
第一篇没有默认选中、内容空白（"从侧栏选择一个计划"）。**默认选中第一篇
并加载对应内容**。根因：PlansStore.LoadPlans 只填列表，无自动选中
（会话轨 ForgeStore.Init 有"自动选中首会话"口径，计划轨缺失）；且生成
store handler 为 async（Init 不等 LoadPlans），视图侧 Init 同步读不到
选中结果——自动选中必须落在 store 的 LoadPlans 链内，高亮改以
`store.current` 为准（computed selectedSeq），视图 `selected_seq` 保留
为无选中回退。

**需求⑬（2026-09-18 用户追加，r14，随 @ 弹窗截图提出，含设计讨论）**：
两条。**①对齐 bug**：@ 弹出行内容没有左对齐（截图实证行居中且偏高）——
根因同 r4 教训：.at button 编译为脚手架 Button，预设 `justify-center
h-10` 未被覆盖；修复 = 行按钮补 `justify-start h-auto`。**②@ 菜单三源
扩展**：传统 CodingAgent 的 @ 多用于选文件，建议除 Agent 外增加计划与
规范搜索（如 `@001` 指向计划 001）。设计裁定（用户征询意见后按推荐
落地）：@ 引用的下游语义 = **运行时解析注入**——菜单选择插入
`@plan/<id>` / `@spec/<relpath>` token（用户消息原文保留），chat 运行
主体启动时解析 token 并把解析注记块（文件路径 + 计划标题/状态，提示用
read_plan/文件工具自取全文）附加进 agent 输入；不整篇内联（上下文成本
可控、agent 按需读取）。数据面：MentionInput 无 store 绑定（codegen 单
别名限制），plans/specs 经 back.api 直取 + 首次触发 @ 懒加载；过滤/键盘
导航沿用既有扁平管线（三源合并列表）。

**需求⑭（2026-09-18 用户追加，r15，随 @ 体验回验提出）**：三条交互
收敛。**①token 识别确认 + 内联组件**：`@plan/001` 后端已能解析（r14
注入链路），但须显示为特殊 inline 组件（类似 Agent @词）；**②默认
选中**：候选框打开时第一项、以及任何筛选后结果的第一项，都应为选中态；
**③Enter 补全优先**：候选框打开时 Enter/Tab 都应自动完成（把选中项插入
输入框），只有正常输入态 Enter 才发送——现状 Enter 直接发送（根因：
textarea 上 `onkeydown` 与 `onkeydown.enter.exact.prevent: .send` 两个
监听并存，Enter 同时触发补全与发送）。修复 = mentionHandled 标记：
Keydown 补全置位 → send 读到即跳过；`.Input` 即重置 `mentionIndex=0`
（默认/筛选后选中第一项）。composer 内 token 着色需上游 `__autoMentionHtml`
（auto-lang ui_gen，仅匹配 `@\w+`，斜杠中断）改造——登记为已知项，
本轮气泡侧完整渲染（mention_helpers render_mentions 扩展 token 扫描）。

**需求⑮（2026-09-18 用户追加，r16，随 70719db 会话 busy 输出提问引出）**：
会话 70719db 实证 busy 提示机制工作正常（在途运行时第二条消息被拒并落
可见提示），用户裁定：**正常消息应排队而非拒绝**——运行在途时消息入队，
当前运行收束后自动执行。设计：**前端 store 级队列**（单标签页主场景
全覆盖；后端 busy 链路保留为跨标签页/竞态兜底）。实现三件套：
①store 队列（pending_msgs + QueueMessage/FlushQueue，done/error/
PollStream 完成/SetError 四处收束挂点自动排空，FIFO 逐条）；
②SendInput 在途入队 + 画布"⏳ 已排队 n 条"指示条；③收束竞态修复——
FlushQueue 直发 chats_send_message(queued=true)（done 已见而守卫未清的
8µs 竞态窗口内后端只回 busy:true 不落提示），前端退回队首由 PollStream
每拍重试。**连带修复（排队实测暴露的既有缺陷）**：ChatStore 变更方法
load-modify-save 非原子，运行主体落盘回复与并发消息 POST 交错时后写覆盖
前写（实证：排队消息落盘覆盖丢失首个回答）——ChatStore 加 write_lock
互斥（12 个变更方法串行化，去 Clone 改 Arc 共享不受影响）。

## 1. 目标

1. `run:true` 发送一条消息（含紧跟的 SSE 订阅）后，会话**恰好新增一条**助手回复。
2. SSE 订阅（`GET /api/chats/session/{id}/stream`）在任何时刻都**不孵化运行**：
   在途 → 附加转发 relay_bus 事件直至 done；不在途 → 空闲流挂起。订阅不触碰
   per-session 守卫（chat_runs 集合不因订阅增删键）。
3. 无重跑不变量保持：连续两次 `POST run:true` 只跑一次（现有守卫语义）；
   EventSource 断线重连不产生新运行。
4. VM 轨桥接 host `chat_run_stream`（vm_backend.rs，当前无 front 调用方）语义
   对齐"订阅即附加"，签名不变。
5. 规范同步：`chat-run-policy.md`「SSE 订阅/运行生命周期」与「测试口径」章节
   更新为显式角色分离语义（移除"只读窥探分流"表述与 F-05 缺陷口径）。
6. **（r3）助手消息携带 Agent 职业身份**：assistant 消息（持久化 + turns 双写
   + wire）携带 `profession_id`；chat 界面助手消息头部显示职业头像（HSL 色+
   首字母）与职业名（professions 目录解析，缺失时 fallback profession_id）。
   填点：chat 运行主体（session mode→role）、plan-merge 短路、relay 报告回写
   三处。用户消息/系统消息不填（无身份）。

**非目标**：SSE wire 契约与事件形状（chat-streaming.md 不动）；chats/conversations
双存储结构；VM 前端行为改造；PLAN-069 W2 块时间线等已交付面。

**涉及仓库**：仅 auto-musk（main 检出的 worktree 副本）。**依赖**：无。

## 2. 架构方案

```
chats_message {run:true}                     GET /api/chats/session/{id}/stream
        │ try_start 守卫（唯一孵化入口）              │
        │ 抢到 → spawn                                │ spawn
        ▼                                            ▼
chat_run_owner（新，主体）               chat_run_stream（恒为订阅）
  持守卫、运行 ReAct 内核                    在途 → 附加转发 relay_bus 直至 done
  持久化 + 双写 turns                        不在途 → 空闲流（挂起等下一次运行）
  全部出口 chat_run_finish                   全程不触碰守卫、不孵化
        │
        └── relay_bus 事件 ──► 订阅方转发给 SSE 客户端
```

要点：
- 现有 `chat_run_stream` 运行主体躯干（session 解析、plan-merge 短路、ReAct
  运行、持久化、各出口 `chat_run_finish`）整体迁入 `chat_run_owner`；
  `chats_message` 的 spawn 目标改为它。
- `chat_run_stream` 保留函数名与四参签名（server_stream.rs / vm_backend.rs
  调用点零改动），函数体只留现附加/空闲分支。
- plan-merge 短路属运行主体职责，随躯干迁入 owner；订阅路径遇到 plan-merge
  消息只是转发 owner 发出的 bus 事件。

**r2 增补**（实施中实证的两处前序缺陷，随角色分离一并补全，目标/验收不变）：
- **事件双发接线**：PLAN-069 T-06 的 `emit_bus` 闭包自创建起无调用点（死代码）
  ——流式事件只进本运行 tx，而 `chats_message` spawn 路径 tx 恒为 `Null`，
  订阅方永远收不到 delta/done（"订阅在途可见流"此前由 F-03 缺陷意外顶替）。
  owner 现把全部 SSE 事件（含 plan-merge 短路四帧）镜像上 relay_bus——不补
  则本计划会杀掉唯一的直播通道。
- **订阅收束**：桥接通道 Sender 常驻 HANDLES 表，tx Value 掉落不关闭通道；
  附加分支 break 后补 `close_channel`（旧附加路径从未真正走通，漏调使 SSE
  永不收束，测试实证）。

## 3. 技术栈

Rust（axum + tokio + mpsc/broadcast）；测试 `cargo nextest -p musk` +
axum `tower::ServiceExt::oneshot`（MockClient）；前端零改动。

## 4. 需求分析与背景调查

**授权**（用户 2026-09-17 当面指示）：记录双回复 bug 到新计划并立即用
auto-plan-work 实施修复；后续其他修改需求继续追加到本计划跟踪。允许仓库动作：
auto-musk 代码/测试/规范修改（worktree 内）+ docs/plans 计划簿记（主检出）。
无额外预算/续跑限制。

**实证证据**：
- 会话数据：`D:/autostack/auto-edit/.autoos/conversations/18683b29f0d04a4c7cced4f7/meta.json`
  ——seq 0 用户消息（ts …238）、seq 1/seq 2 两条助手消息（ts …244 / …250）。
- 服务端日志：`tmp/logs/musk-serve.out.log`——14:27:18.742 恰一次
  `POST /message`、.743 一次 `GET /stream`、.745 mode/role/skill registry
  加载日志全部成对（两运行体并发初始化）。
- 代码：`backend/crates/musk/src/auto_generated/extern_impl.rs:853-862`
  （chats_message try_start→spawn）、`:1737-1746`（窥探 + `if !run_owner`
  反置分支）、`auto_generated/server_stream.rs:233-246`（chat_stream spawn
  同函数）、`vm_backend.rs:237-243`（VM 桥 host，src/front 与 gen front 均
  无调用方——grep 实证）。
- 测试：`backend/crates/musk/src/server.rs:2700-2757`（F-05 口径：抢守卫+订阅
  =运行主体，绿灯钉死缺陷语义）、`:3119`（连续 POST 防双跑，语义保留）。
- 历史：f0677cd（F-03 根修，自记"待干净会话复测"）；规范
  `docs/specs/modules/chat-run-policy.md:49-75` 为权威语义，代码违反之。

**已知事实**：`auto_generated/*.rs` 沿 PLAN-069 F-03 先例直接手改
（extern 真实化实践）；修复后无再生成步骤会覆盖（无 .at 源对应此函数）。

## 5. 详细设计

### 角色分离

- `pub async fn chat_run_owner(s, q, p, tx)`：现运行主体躯干原样迁入
  （含 `run_key` 守卫出口清理、plan-merge 短路、ReAct、持久化、双写 turns）。
  仅由 `chats_message` try_start 成功后 spawn——守卫必然在途，函数内不再
  窥探判定。
- `pub async fn chat_run_stream(s, q, p, tx)`：删除窥探判定与运行躯干，函数体
  = 现附加/空闲循环原样保留（`chat_run_active` 窥探可一并移除——订阅恒附加）。
  server_stream.rs:244 与 vm_backend.rs:243 两个调用点零改动即获得新语义。
- 守卫生命周期不变：`chats_message` 置位 → owner 出口 `chat_run_finish`；
  订阅路径全程 no-op。

### 规范增量

| delta_id | 操作 | 目标 | before/after | rationale | AC |
|:---|:---|:---|:---|:---|:---|
| SD-01 | modify | docs/specs/modules/chat-run-policy.md「SSE 订阅/运行生命周期」 | before：SSE chat_stream 以 chat_run_active 只读窥探分流（在途→附加/不在途→空闲）；after：显式角色分离——运行孵化唯一入口 chats_message run=true→spawn chat_run_owner（持守卫）；chat_run_stream 恒为附加/空闲订阅，绝不孵化、不触碰守卫 | 窥探共享守卫无法区分主体/订阅者（18683b29 双回复实证）；spec 原文"订阅绝不孵化运行"与代码对齐 | AC-01/02 |
| SD-02 | modify | docs/specs/modules/chat-run-policy.md「测试口径」 | before：ag_chat_stream 运行主体路径（先 try_start 取守卫再订阅——F-05 口径）；after：运行主体路径经 chat_run_owner / chats_message run=true 验证；裸订阅恒空闲流不终止；新增回归口径：run:true POST + 紧跟订阅 = 恰一条助手回复 | F-05 口径把缺陷语义钉成绿灯，随语义一并退役 | AC-03/06 |
| SD-03 | add | docs/specs/modules/chat-agent-identity.md（新模块） | 新增：assistant 消息身份契约——ChatMessage/Turn 可选 `profession_id`（skip-if-none，旧数据兼容）；三个落盘点（运行主体= session mode→role、plan-merge 短路、relay 报告回写）必须填身份；前端助手头部渲染职业头像+名字（AgentConfigs 目录解析，缺失 fallback id）；身份目录 = `~/.config/autoos/professions.json`（单一真源）经 `/api/forge/relay/professions` | 用户裁定 agent 身份不可简化（auto-forge 设计回归）；部件在场只缺接线 | AC-09/10/11 |

## 6. 测试设计

- **回归测试（核心）**：`post_run_true_then_subscribe_appends_single_reply`
  ——POST `{content, run:true}` 后立即 GET /stream（复刻 18683b29 序列），
  等待运行收束，断言 session 恰好新增 1 条 assistant 消息。
- **订阅不孵化**：裸订阅（不在途）保持空闲流不终止（保留现 F-05 裸订阅口径）；
  在途订阅：先 try_start 模拟在途 + 订阅 → 断言不新增 assistant 消息（订阅
  不再运行）、流保持挂起。
- **主体路径**：`ag_chat_stream_persists_and_streams` 改造——运行+流式+持久化
  断言改经 `chats_message run=true` 路由（或直呼 `chat_run_owner`）验证。
- **守卫防双跑**：`chat_message_run_guard_prevents_double_run` 原样保持绿。
- 门禁：`cargo nextest -p musk` 全绿（基线 638 绿 4 skip + 新增），
  `cargo build -p musk` 通过。

## 7. 验收标准

| ID | 可观察行为 | 验证方法 |
|:---|:---|:---|
| AC-01 | run:true 消息 + 紧跟 SSE 订阅 → 恰一条助手回复 | 新增回归测试绿 + T-05 实机会话复测 |
| AC-02 | 订阅在途仅附加转发，不孵化、守卫键数不因订阅变化 | 在途订阅测试绿 |
| AC-03 | 裸订阅（不在途）空闲流不终止、不运行 | 现有裸订阅口径测试保持绿 |
| AC-04 | 连续两次 POST run:true 只跑一次 | `chat_message_run_guard_prevents_double_run` 绿 |
| AC-05 | VM 桥 chat_run_stream 语义=附加订阅，签名不变 | 编译通过 + 调用点零改动 review |
| AC-06 | 全量门禁绿 | `cargo nextest -p musk`（基线+新增全绿） |
| AC-07 | 实机：auto-edit 工作区发一句话只有一条回答 | 重启 dev 栈后用户验收 |
| AC-08 | 规范与新语义一致 | SD-01/SD-02 落地，文档 diff review |
| AC-09 | assistant 持久化消息携带 `profession_id`（superpowers 会话 = "assistant"）；无身份的旧消息照常渲染（skip-if-none wire 兼容） | lib 测试 + wire 断言 |
| AC-10 | turns.jsonl 主 turn 同步携带身份（双存储投影一致） | lib 测试 |
| AC-11 | chat 界面助手消息头部显示职业头像（HSL 色+首字母）与职业名；身份缺失回退现有 "🤖 AI" 徽章 | `auto build` 产物检查 + 实机 |
| AC-12 | 全量门禁绿：cargo nextest -p musk 全绿 + `vm-link-probe` 绿（front 源门禁） | 两门禁命令 |
| AC-13 | AI 回复列与用户气泡统一 70% 限宽（AI 不再占满全宽） | 截图/产物类串检查 |
| AC-14 | 工具卡标题：操作对象左对齐紧随工具名（有 gap），status+chevron 靠右 | 截图/产物检查 |
| AC-15 | 侧边栏可收缩：收缩态 32px 纯 icon 导航 + 鹿 logo（hover=名称+版本，点击展开）+ workspace 文件夹 icon（hover=目录）+ 设置齿轮 | 实机截图 |
| AC-16 | 需求③门禁绿：vm-link-probe + auto build + nextest | 三门禁命令 |
| AC-17 | 会话标题单行 `...` 截断，hover（title）显示全名 | 浏览器实测 |
| AC-18 | 会话 hover `×` 位于第二行右缘（与 n 条 同行） | 浏览器实测 |
| AC-19 | 会话/规范二级菜单首 box 顶部有间距 | 浏览器实测 |
| AC-20 | 规范 item box 与会话同构（两行盒）且显示各类型规范文档数 | 浏览器实测 |
| AC-21 | LLM 流 stall 超时后 run 报错收束、错误消息持久化、守卫清理 | 超时注入测试 |
| AC-22 | run_command 超时可配，超时后部分输出回灌、run 继续收束 | 测试 |
| AC-23 | 守卫占用时 run:true 返回 busy 指示（不再静默） | 测试 |
| AC-24 | cancel 端点 + UI 停止按钮可终止在途运行并清守卫 | 测试 + 实机 |
| AC-26 | 发送后 assistant 标题栏立即出现（含时间戳），气泡内显示等待态 | 浏览器实测 |
| AC-27 | 流式期 ThinkBlock 半开态：一行滚动显示最新思考；收束后收缩为已思考 | 浏览器实测 |
| AC-28 | 流式期回填不清空在途内容；收束后无需刷新即换入持久化消息 | 浏览器实测 |
| AC-25 | 运行生命周期 INFO 日志（spawn/finish/fail+原因/busy）可从 serve 日志直接判读 | 日志断言 |
| AC-29 | 发送后首个 thinking/delta 到达前，assistant 气泡内显示动态等待点；块到达后消失 | 浏览器实测 |
| AC-30 | 流式期停止按钮位于流式消息工具栏（复制/分叉旁），画布底部独立按钮退役；点击取消运行，收束后消失 | 浏览器实测 |
| AC-31 | 新会话与未设置会话的思考档位默认显示并生效为"低"（发送产出思考块）；显式选"关"持久化为 off 且生效（无思考块）；档位选择跨刷新/重进会话保持 | 浏览器实测 |
| AC-32 | 计划二级导航为两行盒（与会话/规范同构）：标题行 truncate + hover 全名，副标题行状态 badge；选中态 primary 边框底色；点选加载详情正常 | 浏览器实测 + 截图 |
| AC-33 | 打开计划页时第一篇默认选中（primary 高亮）且右侧详情自动加载，空态提示不出现；用户点选切换行为不变；列表刷新（流转/归档）不抢当前选中 | 浏览器实测 |
| AC-34 | @ 弹出行内容左对齐（justify-start），行高正常（h-auto） | 浏览器 DOM 实测 |
| AC-35 | @ 菜单三源：Agent + 计划（@plan/<id>）+ 规范文件（@spec/<relpath>）；过滤跨三源（如 @001 命中计划）；选中插入对应 token | 浏览器实测 |
| AC-36 | 运行时引用解析：发含 @plan/001 的消息，agent 无需读文件即能从注入块答出计划文件路径；用户可见/持久化消息保持原文 | 浏览器端到端实测 |
| AC-37 | @ 候选框打开时与任何筛选后，第一项均为选中态（高亮） | 浏览器 DOM 实测 |
| AC-38 | 候选框打开时 Enter/Tab 均为补全（插入选中项）且不发送；关闭候选框后 Enter 正常发送 | 浏览器实测 |
| AC-39 | 发送后的用户气泡中 @plan/001、@spec/<path> 渲染为 📋/📄 前缀的特殊 inline 组件 | 浏览器 DOM 实测 |
| AC-40 | 运行在途时发送的消息入队（画布显示"⏳ 已排队 n 条"，无 busy 拒绝）；当前运行收束后队列按 FIFO 自动发送并独立成轮回答；chats.json 落盘完整无丢失 | 浏览器端到端实测 |

## 8. 执行步骤

- [x] **T-01** worktree 准备：`.wt/musk-071/auto-musk`（分支 `plan-071-dev`，
  基座 ba90b30）+ 依赖快照 `auto-ai@9d2102c` / `auto-lang@844ff9c81`
  （auto-musk-dev 分支；musk 的 `../../../../auto-{ai,lang}` 相对路径依赖）。
  [✅ 2026-09-17]
- [x] **T-02** 角色分离实现（SD-01 代码面）：extern_impl.rs 运行躯干迁入
  `chat_run_owner`，`chat_run_stream` 恒订阅化（含 break 后 `close_channel`），
  `chats_message` spawn 目标改 owner；emit_bus 双发接线（r2 增补）。
  [✅ 2026-09-17] commit 5ca6cfe；`cargo build -p musk` 通过。→ AC-01/02/05
- [x] **T-03** 测试改造与新增（SD-02 口径）：F-05 窥探口径测试退役；替换为
  `ag_chat_message_run_true_persists_single_reply`（总线双发+单回复+守卫清除）、
  `ag_chat_run_then_subscribe_single_reply`（18683b29 回归序列，SlowClient
  300ms 保证订阅时在途）、`ag_chat_stream_bare_subscribe_stays_idle`（空闲流
  不终止/不孵化/不触碰守卫）；守卫防双跑两测保留。新增 `SlowClient` 桩。
  [✅ 2026-09-17] 目标档 6/6 绿。→ AC-01/02/03/04
- [x] **T-04** 全量门禁：`cargo nextest -p musk`。
  [✅ 2026-09-17] 640 passed / 4 skipped（基线 638+4 + 净增 2），零失败。
  → AC-06
- [x] **T-05** 实机验收：worktree 产物 `musk.exe`（23:13 构建）热替换主检出
  serve（workdir tmp/musk-demo 不变；worktree 构建的 default_dist 为编译期
  worktree 路径而 dist 不入 worktree，按文档开关 `MUSK_WEB_DIST` 指向主检出
  `gen/front/vue/dist`），auto-edit 工作区实机复刻 18683b29 序列（POST
  run:true + 紧跟订阅，真实 aaid→zhipu 链路）：新会话
  `44278619070376350dddc6de` 一句话**恰一条助手回复**，SSE 直播完整
  （28 delta / 54 thinking / 1 turn_end / 1 done）并正常收束。
  [✅ 2026-09-17] → AC-07（用户侧复验可随时在 http://127.0.0.1:8080 进行）
- [ ] **T-06** 沉淀与收尾：SD-01/SD-02 已**预备**于 worktree
  （chat-run-policy.md，commit 3b2e72d）；发布到主检出 + ledger 挂载 +
  review/merge（worktree 合回 + 依赖快照清理）走 /auto-plan:review 与
  /auto-plan:merge。依赖：T-05。→ AC-08

**需求②（r3，2026-09-17 追加）：Agent 身份标识回归**

- [x] **T-07** 背景调查与决策：auto-forge 设计（AgentAvatar + AgentConfig +
  消息 profession_id）；auto-musk 部件盘点（professions.json 花名册/端点/
  AgentAvatar 组件/AgentConfigs store 均在场）；决策 = 身份字段用可选
  `profession_id`（沿 forge 词汇，skip-if-none 兼容旧数据），填点三处，前端
  头像+名字走既有 AgentAvatar/AgentConfigs。[✅ 2026-09-17]
- [x] **T-08** 后端身份字段与填点：chats.rs ChatMessage + src/conversation.rs
  Turn（手写真源）+ auto_generated/conversation.rs Turn（ag 镜像）增可选
  `profession_id`；hw to_turns Message 主 turn/text 块 turn 透传（ag 侧恒
  None，镜像哲学一致）；三落点按 session mode→role 填充。
  [✅ 2026-09-17] commit 173e492；lib 测试断言 basic→"coder" 双存储同步绿。
  → AC-09/10
- [x] **T-09** 前端接线：forge_helpers.at 增纯 fn `agentDisplayName`
  （id 推导显示名——**方案变更**：原计划 AgentConfigs store 查名经
  ChatsView prop 下传，实施发现 vue codegen 仅支持 `store` 单别名绑定，
  跨 store 链 vue-tsc 绑定缺失，改 id 推导 + 目录名动态显示登记后续）；
  chat_message.at 头部 assistant 臂 AgentAvatar + 职业名，无身份回退
  "🤖 AI"；chats_view.at 零改动（prop 链撤销）。[✅ 2026-09-17] → AC-11
- [x] **T-10** 构建与门禁：worktree `auto build` 绿（vue-tsc strict + vite，
  dist 重建）；`vm-link-probe` PASS（77262 bytes < WARN 90000，MUSK_APP_PATH
  指向 worktree 语料 + VM_LINK_LANG_ROOT 主检出热 target）；`cargo nextest
  -p musk` 640 passed / 4 skipped。[✅ 2026-09-17] → AC-12
- [x] **T-11** 实机验收：serve 换新产物（MUSK_WEB_DIST 指 worktree dist），
  auto-edit 实会话 `f2b21016303352e73d4af75e`（superpowers）一句话：wire
  assistant 消息带 `profession_id: "assistant"`、user 恒 None、仍恰一条
  回复（需求①不回退）；serve bundle 含 agent-avatar 代码，professions
  端点在线。[✅ 2026-09-17]（头像/名字的视觉确认可在 http://127.0.0.1:8080
  直接发消息验证——历史旧消息无身份字段按契约回退旧渲染）
- [ ] **T-12** 沉淀：SD-03 已**预备**于 worktree（新模块
  chat-agent-identity.md，commit 02d7d30）；发布到主检出 + ledger 挂载 +
  review/merge 走 /auto-plan:review 与 /auto-plan:merge。依赖：T-11。→ AC-08 扩展

**需求③（r4，2026-09-18 追加）：UI 三条改进**

- [x] **T-13** 消息宽度对称：chat_message.at rowClass 用户 85%→70%、AI
  max-w-full→70%。[✅ 2026-09-18] → AC-13
- [x] **T-14** 工具卡标题左对齐：name 去 flex-1（内容宽），status 加 ml-auto
  （status+chevron 靠右）；chat_message 内联两处 + generic_tool_card 一处。
  [✅ 2026-09-18]

**需求⑦（2026-09-18 用户报告会话 4062c66e 思考消失，r8）：思考沉淀为块 +
ThinkBlock 渲染修复**

- [x] **T-31** 后端：Ok 收束时 thinking 非空 → blocks 前置
  {kind:thinking} 块（须在 W2 时间线组装之后——组装整体重写 blocks，
  首版插在组装前被覆盖，实测修正）。[✅ 2026-09-18]
- [x] **T-32** 前端：messageBlocks 归一化 thinking 透传（原 else 兜底折
  成 text，ThinkBlock 分支永不可达）+ 旧消息自愈（blocks 非空但无
  thinking 块且 msg.thinking 有值 → 前置合成）。[✅ 2026-09-18]
- [x] **T-33** 实施注记：.at 无 list.insert 透传（TS2339）——重建数组
  等价实现；浏览器实测：流式期思考块实时增长、收束后 💭 已思考·N tokens
  折叠块渲染、展开可见完整思考。[✅ 2026-09-18] → AC-14
- [x] **T-15** 侧边栏收缩态：app.at rail_collapsed + ToggleRail + railClass
  计算类（w-48/w-16）；收缩态 6 导航 32px 纯 icon（rail-icon-btn + title）；
  鹿 logo 双主题 img（64px 量化 base64 内嵌 + inject_styles 随 html.dark
  切换，VM 兜底单图；title=Auto Musk v0.1.0，点击展开）；WorkspaceSelector/
  SettingsMenu collapsed 形态。[✅ 2026-09-18] → AC-15
- [x] **T-16** 构建与门禁：vm-link-probe PASS（77457B）；auto build 绿；
  dist 标记齐全（rail-icon-btn ×4 / deer 双 base64 / collapsed 臂 ×8）；后端
  零改动无需重启，浏览器刷新即生效。[✅ 2026-09-18] commit f8c8ca4 → AC-16

**需求④（r5，2026-09-18 追加）：二级菜单四条改进**

- [x] **T-17** 会话标题截断：chats_view.at 标题 span break-all→truncate +
  title=.s.name。[✅ 2026-09-18] → AC-17
- [x] **T-18** × 移第二行：hover × 从标题行 absolute 改第二行右缘
  （justify-between 与计数同行）。[✅ 2026-09-18] → AC-18
- [x] **T-19** 首盒顶部 gap：会话列表容器 +pt-1.5；规范 +pt-1。[✅ 2026-09-18] → AC-19
- [x] **T-20** 规范盒统一 + 计数：specs_helpers.at 增 specSectionRows；
  specs_view.at 概览/类型盒 session-item 同构两行盒 + "n 篇" 计数（i18n
  specs.docs）；实施中补 h-auto（脚手架 Button h-10 定高压扁两行内容）。
  [✅ 2026-09-18] → AC-20
- [x] **T-21** 构建与实机：vm-link-probe PASS（77565B）+ auto build 绿 +
  浏览器 DOM 实测四条全过（topPad 6px/4px、ellipsis、× 同行右贴、两行盒
  齐全）+ 截图确认。[✅ 2026-09-18] → AC-17..20

**需求⑤（r6，2026-09-18 追加）：挂死运行防护与可取消——Phase 2 设计与实施**

### 架构方案（Phase 2）

①**LLM 空闲看门狗**（T-22）：chat_run_owner 的 run_stream 改 `tokio::select!`
包裹——事件回调每帧 touch watch 通道（Instant）；看门狗臂 sleep_until(最后事件
+ 空闲窗) 到点即赢得 select，**drop run_stream future 中止在途 LLM 流**（不依
赖 agent 内部 cancel 语义）。空闲窗 = `AppState.run_idle_timeout`（env
`AUTO_RUN_IDLE_TIMEOUT_SECS`，默认 300s；测试注入短窗）。到点 = 超时收束：
错误事件上总线/SSE + **错误 assistant 消息持久化**（双写 turns + 身份）+
守卫清理。agent Err 臂同步补**落盘**（现只发瞬态 SSE）。
②**命令默认超时**（T-23）：run_command 工具层 `timeout_secs` 缺省时不再传
None（=永挂），改缺省 `AUTO_CMD_TIMEOUT_SECS`（默认 300s）；超时路径已有
（杀树+保留输出+timed_out 标记），补结果文本标注与文档。
③**busy 指示**（T-24）：chats_message 守卫占用分支响应体加 `"busy": true` +
WARN 日志；前端发送路径遇 busy 追加**本地未持久化**提示条（"已有运行在途，
本条未执行；结束后请重发"），不再无声。
④**取消**（T-25）：AppState 增 `chat_cancels: Mutex<HashMap<run_key,
Arc<AtomicBool>>>`——owner 孵化时注册自己的 cancel flag（出口移除）；新端点
`POST /api/chats/session/{id}/cancel?workspace=` 置位并应答。语义：agent 于
迭代边界检查（agent.at 619/665/678/714 实证），**流内 stall 由①看门狗兜底**；
前端流式期间 composer 旁显示停止钮。
⑤**生命周期日志**（T-26）：owner spawn/finish(ok|err|timeout|cancelled)/busy
全打 INFO/WARN（含会话、耗时、原因）。

- [x] **T-22** 看门狗 + Err/超时落盘 + run_idle_timeout 配置（env
  AUTO_RUN_IDLE_TIMEOUT_SECS 默认 300s）。[✅ 2026-09-18]
- [x] **T-23** run_command 默认超时（AUTO_CMD_TIMEOUT_SECS，默认 300s）。
  [✅ 2026-09-18]
- [x] **T-24** busy 指示：守卫占用落可见 busy 提示消息（持久化）+ 响应体
  busy:true + WARN 日志（前端零改动方案——生成客户端丢弃响应体，resp.busy
  不可达；提示条走持久化消息同构于超时/失败落盘）。[✅ 2026-09-18]
- [x] **T-25** cancel 端点（AppState::chat_cancel + chat_cancels 注册表）+
  api.at chats_cancel_session + forge_store CancelRun + 会话页停止钮。
  [✅ 2026-09-18]
- [x] **T-26** 生命周期日志：spawned/finished(耗时)/failed/timeout/busy。
  [✅ 2026-09-18]
- [x] **T-27** 门禁：cargo nextest 644 passed/4 skipped（新增 4 测）+
- [x] **T-28** 需求⑥：流式状态指示常驻（长思考期不再空白）——chats_view
  指示条件改 current_draft=="" 即显示；深度思考期显示 🤔 状态行 + 思考流尾
  片段（streamThinkingTail，R1/R2 合规）。实测：会话 2882117a 运行 467s
  （glm-5.3 长思考大任务）全程有状态反馈，收束后回复正常落盘。
  [✅ 2026-09-18] commit 需求⑥提交 → 新增 AC：流式期零空白。
- [x] **T-29** PollStream deadman 窗随流式自动续期——用户第二轮回验定位
  第二根因：120s 硬过期使长任务（467s）失去回填兜底，SSE 任何迟滞即永久
  空白。修复后流式进行中每 tick 续窗；浏览器回归正常（指示→流式→收束）。
  [✅ 2026-09-18] commit deadman 续期提交
- [x] **T-30** 需求⑥回验：主导航"文件"图标 folder→folder-open，与底部
  workspace 文件夹图标区分（展开/收缩同步）。[✅ 2026-09-18]
  vm-link-probe PASS + auto build 绿 + 实机验证（spawned→finished 7s 日志、
  回复正常）。[✅ 2026-09-18] → AC-21..25

**需求⑧（r9，2026-09-18 用户提出）：流式 UX 重构（6 步规格，待实施）**

- [x] **T-34** 乐观 assistant 气泡：Send 即创建（profession=modeRoleOf
  (会话 mode)；api.at ForgeSession 契约补 mode 字段）。[✅ 2026-09-18] → AC-26
- [x] **T-35** 流式期 ThinkBlock 半开态：header(💭 思考中·N tokens)+一行
  滚动尾（streamThinkingTail）；收束后收缩为已思考 chip。[✅ 2026-09-18] → AC-27
- [x] **T-36** 回填防清除：PollStream 流式期快照末消息非 assistant 时跳过
  回填；done 不再抢先关 deadman 窗。[✅ 2026-09-18] → AC-28
- [x] **T-37** 构建门禁 + 浏览器全流程实测：auto build 绿；实机采样——
  标题栏 0.6s 即现、ThinkBlock 常驻、正文流式增长、4.5s 收束；刷新后渲染
  一致（用户并行测试消息 13:53:27 同样完整渲染）。[✅ 2026-09-18] → AC-26..28
  注：AC-22 命令超时沿既有基建接缺省（专项测试已有 timeout_kills_* 覆盖）；
  T-25 VM host 桥未接线（web 为主，登记后续）。

**需求⑨（r10，2026-09-18 用户提出）：流式等待态与停止按钮归位**

- [x] **T-38** forge_helpers.at 增 `msgIsWaiting(msg, streaming)`：流式接收中
  且展示块为空 = AI 启动窗口（乐观气泡保证窗口判定可靠）。[✅ 2026-09-18] → AC-29
- [x] **T-39** chat_message.at：①等待点组件——msg-bubble-ai 内 showWaiting
  条件渲染三点跳动动画（style 块 @keyframes + :nth-child 延时）；
  ②工具栏停止按钮——is_streaming 时渲染在复制/分叉右侧（同款 ghost 方钮，
  ■ 字形 + destructive 色），直调 ForgeStore.CancelRun()。[✅ 2026-09-18] → AC-29/30
- [x] **T-40** chats_view.at：移除画布底部独立停止按钮 + 孤儿 CancelRun
  msg/handler。[✅ 2026-09-18] → AC-30
- [x] **T-41** auto build + 浏览器全流程实测。[✅ 2026-09-18] → AC-29/30
  实测证据（新会话 70bdf78f，worktree 构建即时生效）：采样时间线 t=0.6s
  等待点 3 粒即现 + 工具栏停止钮在场 + 独立按钮全程未出现；t=4.6s 首个
  thinking 到达、等待点消失、ThinkBlock 半开态接管；收束后停止钮消失、
  回复完整渲染（💭 已思考折叠块 + 全文）。工具栏 DOM 结构实证 = Copy →
  ⑂ (Fork) → ■ (Stop) 同行三钮；截图确认等待点 + 红色停止钮形态。
- [x] **T-42** cancel 端点接入真实路由表（**T-25 遗留缺陷修复**，T-41 实测
  暴露）。[✅ 2026-09-18] → AC-30
  缺陷：T-25 的 `POST /api/chats/session/{id}/cancel` 只注册进 server.rs 的
  **测试路由器**（1413 行，#[tokio::test] 内），真实 `auto_generated/
  server.rs::build_router()` 从未挂载——POST 落 SPA fallback 返回 405，
  前端 `await chats_cancel_session` 抛错、`streaming` 永不落 false、停止钮
  点击无效。修复：build_router() 补一行路由（handler = AppState::chat_cancel，
  已在场）。修复后需重编后端 + 重启 serve（8090 已换新二进制）。
  取消链路终验：点击 → POST cancel → serve 日志 "chat cancel requested
  session=70bdf78f… (in-flight=true)" → 前端停止钮 901ms 消失 → run 于
  迭代边界干净收束（elapsed=72s，守卫清除）。门禁：cargo nextest 644
  passed / 4 skipped + vm-link-probe PASS（77560B）+ auto build 绿。
  注：取消语义仍为 T-25 口径（旗标于 agent 迭代边界检查；单次 LLM 调用
  中途不中断，调用收束后 agent 见旗标即停）；流内硬中断归看门狗域，登记
  后续可选增强（select! 臂监听 cancel 旗标 drop run_stream）。

**需求⑩（r11，2026-09-18 用户提出）：思考档位默认低**

- [x] **T-43** 后端运行路径兜底：extern_impl.rs `session_thinking2` 增
  `.or_else(|| Some("low".to_string()))`——None/旧会话一律按 low 生效
  （手改先例区，regen 安全）。[✅ 2026-09-18] → AC-31
- [x] **T-44** 前端：①forge_store.at 四处回填/初始 `?? ""` → `?? "low"`
  （store 初值 :54 / NewSession :176 / SwitchSession :210 / BranchTo :158）；
  ②mention_input.at 档位菜单"关"项 `pick("")` → `pick("off")`，选中条件
  收紧为 `== "off"`（空串语义退役）；③chats.at 结构注释同步。[✅ 2026-09-18] → AC-31
- [x] **T-45** 构建 + 门禁 + 浏览器四场景实测。[✅ 2026-09-18] → AC-31
  auto build 绿 + nextest 644/4 + vm-link-probe PASS（77563B）；后端重编、
  8090 serve 已换新二进制。实测（新会话）：①新建即显示"低"（未设置旧会话
  同样走兜底显示低）②默认低发消息产出思考块（💭 已思考 · 11 tokens，低档
  生效）③显式选"关"发消息无思考块 ④整页刷新 + 重进会话后"关"保持
  （= 后端已持久化 "off"）。commit fac6757。

**需求⑪（r12，2026-09-18 用户提出）：计划二级导航两行盒改造**

- [x] **T-46** plans_view.at：列表容器 `plans-nav-list` → 会话轨同款
  （`flex-1 overflow-y-auto px-2 pt-1.5`）；item 改 session-item 两行盒
  （选中/未选中双态 class，规格对齐 specs_view/chats_view），标题行
  truncate + title 悬停全名，副标题行状态 badge。实施注记：`text` 以
  点路径开头的多段拼接（`text .plan.id + " — " + …`）触发 codegen R016
  （元素标签 "." 泄漏）——标题行改回 span 内多 text 子节点（PLAN-061 T9
  验证安全形）；attr 拼接（`title:`）有 specs_category 先例可用。
  [✅ 2026-09-18] → AC-32
- [x] **T-47** 构建 + 门禁 + 浏览器实测。[✅ 2026-09-18] → AC-32
  auto build 绿 + vm-link-probe PASS（77545B，纯前端后端零改动）；实机
  截图 + DOM 实证：两行盒高 64px（标题行 "001 — bootstrap-auto-edit" +
  副标题 badge "drafting"）、选中态 border-primary/25 + bg-primary/10、
  点选详情 Markdown 正常加载。commit 701c013。

**需求⑫（r13，2026-09-18 用户提出）：计划页默认选中第一篇**

- [x] **T-48** plans_store.at LoadPlans：列表到位后 `current == None` 则
  `LoadPlan(.plans[0].seq)`（与会话轨 Init 自动选中首会话同口径；刷新
  不抢当前选中）。[✅ 2026-09-18] → AC-33
- [x] **T-49** plans_view.at：computed `selectedSeq` 读 store.current，
  class 高亮条件改用之（async 链使视图 Init 同步不可见，store 为单一
  真源）；`selected_seq` 保留回退。[✅ 2026-09-18] → AC-33
- [x] **T-50** 构建 + 门禁 + 浏览器实测。[✅ 2026-09-18] → AC-33
  auto build 绿 + vm-link-probe PASS（77591B，纯前端）；实测——打开计划
  页第一篇即选中（primary 高亮）+ 详情自动渲染（Plan 001 正文）、空态
  提示不再出现、点选切换行为不变。commit 6fbfc2c。

**需求⑬（r14，2026-09-18 用户提出）：@ 三源扩展 + 对齐修复**

- [x] **T-51** 对齐修复：mention_dropdown.at 行按钮补 `justify-start
  h-auto`（脚手架 Button 预设覆盖，r4 教训同源）。[✅ 2026-09-18] → AC-34
- [x] **T-52** 前端三源：mention_helpers.at `mention_combined_items`
  （Agent+计划+规范文件合并）+ `mention_insert` kind 感知（agent 插
  @显示名 / plan·spec 插 @token）；mention_input.at 经 back.api 懒加载
  plans_list/specs_tree（type==file 过滤），过滤与键盘导航沿用扁平管线；
  dropdown kind 分支渲染（Avatar/📋/📄）。[✅ 2026-09-18] → AC-35
- [x] **T-53** 后端解析注入：server.rs `resolve_chat_mention_refs`
  （@plan/<seq> 查 PlansStore 含归档 → 路径+标题+状态；@spec/<relpath> →
  docs/specs/ 路径；空串=不改写）；extern_impl chat_run_owner 解析块
  附加进 agent.run_stream 输入。[✅ 2026-09-18] → AC-36
- [x] **T-54** 构建 + 门禁 + 端到端实测。[✅ 2026-09-18] → AC-34..36
  auto build 绿 + nextest 644/4 + vm-link-probe PASS（77602B）；serve 已
  换新二进制。实测——菜单 13 行（9 Agent + 1 计划 + 3 规范）左对齐
  42px；@001 过滤唯一命中、插入 `@plan/001 `；端到端：发"@plan/001 引用
  的计划文件名是什么（不要读文件）"→ agent 从注入块答出
  `docs/plans/001-bootstrap-auto-edit.md`，可见消息保持原文。
  commit ab6645b。

**需求⑭（r15，2026-09-18 用户提出）：@ 交互收敛（默认选中/补全优先/token 组件）**

- [x] **T-55** 默认选中 + Enter 补全优先：mention_input.at `.Input` 即重置
  `mentionIndex=0`；`mentionHandled` 标记（Keydown 补全置位 → `.send` 读到
  跳过），正常输入态 Enter 发送不变。[✅ 2026-09-18] → AC-37/38
- [x] **T-56** 气泡内联组件：mention_helpers.at `render_mentions` 扩展
  token 扫描（@plan/<seq>、@spec/<relpath>，路径字符 \w./- 空白终止），
  渲染 📋/📄 前缀特殊 inline 组件；`mention_is_path_char` 辅助。
  [✅ 2026-09-18] → AC-39
- [x] **T-57** 构建 + 门禁 + 浏览器实测。[✅ 2026-09-18] → AC-37..39
  auto build 绿 + vm-link-probe PASS（纯前端后端零改动）。实测——@ 打开
  与筛选 @00 后第一项均为选中态；Enter 补全为 @plan/001 且不发送、正常态
  发送正常；用户气泡渲染 `📋 @plan/001` 高亮 token；并行回验观察到需求⑤
  busy 提示正常触发。已知项：composer 内 token 着色需上游 __autoMentionHtml
  匹配器支持非 \w 词（auto-lang），登记后续。commit 23c5add。

**需求⑮（r16，2026-09-18 用户提出）：运行在途消息排队自动执行**

- [x] **T-58** store 队列：forge_store.at pending_msgs + QueueMessage/
  FlushQueue；done/error/PollStream 完成/SetError 四处收束挂点自动排空
  （FIFO；streaming 真时 no-op 防双发）。[✅ 2026-09-18] → AC-40
- [x] **T-59** 入队 + 指示条：chats_view.at SendInput 在途入队（不走
  busy 拒绝链路）+ 画布"⏳ 已排队 n 条，当前回复结束后自动发送"。
  [✅ 2026-09-18] → AC-40
- [x] **T-60** 收束竞态修复 + 并发丢写修复。[✅ 2026-09-18] → AC-40
  竞态：FlushQueue 首测在 done 已见/守卫未清窗口（实证 8µs）内重发被
  拒——FlushQueue 改直发 chats_send_message(queued=true)（api.at 返回
  any 暴露响应体；server.rs ChatMessageBody 加 queued 字段；busy 分支
  queued=true 免落提示），busy 即退回队首由 PollStream 每拍重试。
  连带缺陷：ChatStore load-modify-save 非原子（排队消息落盘覆盖丢失
  首个回答，chats.json 实证）——ChatStore 加 write_lock，12 个变更方法
  串行化（去 Clone；Arc 共享不受影响）。
- [x] **T-61** 构建 + 门禁 + 端到端实测。[✅ 2026-09-18] → AC-40
  nextest 644/4 + vm-link-probe PASS + auto build 绿（serve 已换新
  二进制）。端到端：长任务在途发第二条 → "⏳ 已排队 1 条"且无 busy；
  首轮收束后自动发送、独立成轮回答（"补充句"得到上下文衔接的回应）；
  chats.json 四 turn 齐全无丢失。commit 4d156c4。
  （全量门禁曾现 parity_plans 单测偶发失败，隔离重跑+复跑全绿——与本次
  改动无关的并行噪声，登记备查。）

## 9. 复审记录

- 2026-09-17T23:00+08:00 `stage: new` PLAN-071 r1 起草完成。背景调查四类证据
  （会话数据/服务端日志/代码行号/历史提交与测试口径）齐备，路径均实测存在。
  `outcome: pass` ——任务覆盖全部 AC 与 SD；无阻塞待澄清。
  `next: work`（用户已授权直接实施）。
- 2026-09-17T23:20+08:00 `stage: work` | PLAN-071 | r2 | `outcome: pass` |
  code_commit: 5ca6cfe（实现）+ 3b2e72d（spec 预备），分支 plan-071-dev
  （worktree `.wt/musk-071/auto-musk`，基座 ba90b30；依赖快照
  auto-ai@9d2102c / auto-lang@844ff9c81，无改动待 merge 清理）|
  task_ids: T-01..T-05 完成、T-06 预备完毕 | evidence: 全量门禁 640 passed /
  4 skipped；实机会话 44278619070376350dddc6de 单回复 + SSE 直播收束 |
  blockers: 无 | next: review（r2 增补 = emit_bus 双发接线 + close_channel
  收束，均为达成 AC-01 的必要前序缺陷修复，记录于第 2 节；AC 未变）。
- 2026-09-17T17:40+08:00 `stage: work` | PLAN-071 | r3 | `outcome: pass` |
  code_commit: 173e492（需求②实现）+ 02d7d30（SD-03 预备），分支 plan-071-dev |
  task_ids: T-07..T-11 完成、T-12 预备完毕 | evidence: 全量门禁 640 passed /
  4 skipped；vm-link-probe PASS（77262B）；auto build 绿；实机会话
  f2b21016303352e73d4af75e wire 带 profession_id="assistant" 且需求①单回复
  不回退 | blockers: 无 | next: review（两需求一并复审；r3 方案变更 =
  前端职业名走 id 推导而非 AgentConfigs 查名——生成器 `store` 单别名限制，
  实测记录于 T-09 与 SD-03 KD 行）。
- 2026-09-18T01:37+08:00 `stage: work` | PLAN-071 | r4 | `outcome: pass` |
  code_commit: f8c8ca4，分支 plan-071-dev | task_ids: T-13/T-14/T-15/T-16 完成 |
  evidence: 需求①②③全部实现完毕；vm-link-probe PASS（77457B）+ auto build 绿
  + dist 标记齐全；后端零改动 | blockers: 无 | next: review（三个需求一并复审
  后 merge；需求③-3 鹿 logo 明暗双变体随 html.dark 切换、VM 兜底单图的机制
  记录于 SD-03 同文件注释与 commit message）。
- 2026-09-18T01:49+08:00 `stage: work` | PLAN-071 | r4 用户回验修复 |
  `outcome: pass` | code_commit: dcc3b18 | evidence: 用户截图回验三问题修正
  ——①双鹿并显根因 = img class 与 CSS 选择器不一致 + img style:prop 被
  codegen 并进 class（两处修：类名对齐 deer-icon-dark/-light；去失效
  style:prop，VM 双图并显随 inject_styles 家族口径记为已知项）②收缩态
  主导航 6 icon 32→18px ③workspace/设置 icon 32→18px；auto build 绿 |
  blockers: 无 | next: review。
- 2026-09-18T01:54+08:00 `stage: work` | PLAN-071 | r4 用户回验二（图标尺寸
  对齐 VS Code）| `outcome: pass` | code_commit: dcc3b18 后续（24px 统一提交）|
  evidence: 用户对拍 VS Code 活动栏（24px 口径）——收缩态主导航/workspace/
  设置 icon 18→24px，小鹿 32→24px（w-6）；展开态收起钮保持 18px 不随动；
  auto build 绿 | blockers: 无 | next: review。
- 2026-09-18T02:06+08:00 `stage: work` | PLAN-071 | r4 用户回验三（底部图标
  真 24px）| `outcome: pass` | code_commit: 24px 统一 + rail-trigger-24 两提交 |
  evidence: 浏览器 DOM 实测（getBoundingClientRect）坐实底部 Folder/Settings
  渲染 16px——.at button 全编译为脚手架 Button（[&_svg]:size-4 压制 lucide
  size 属性；导航图标靠 size:N 内联 style 免疫）。修复 = 全局 CSS
  .rail-trigger-24 svg { 24px !important }；复测全列 24px（含鹿 24 单显）+
  截图确认；探针账号 uitest-probe 已从 users.json 移除 | blockers: 无 |

- 2026-09-18T10:50+08:00 `stage: work` | PLAN-071 | r6 需求⑤实施完成 |
  `outcome: pass` | code_commit: 需求⑤实施提交（分支 plan-071-dev）|
  task_ids: T-22..T-27 完成 | evidence: 644/4 门禁 + 实机 spawned/finished
  日志 + 64f0076c 事故诊断与解锁（重启清守卫）| blockers: 无 | next: review
  （Phase 2 = 需求⑤；与 phase-1 已落地四需求一并复审）。
- 2026-09-18T11:40+08:00 `stage: work` | PLAN-071 | 需求③回验二（工具卡参数
  仍右对齐）| `outcome: pass` | code_commit: scoped .tool-name flex:1 删除 |
  evidence: 根因 = r4 只删工具类 flex-1、scoped 样式块 .tool-name 规则残留
  flex:1 照常生效；浏览器 DOM 实测三卡参数紧随名称左对齐（gap ~8px）、
  status+chevron 靠右 + 截图确认 | blockers: 无 | next: review。
- 2026-09-18T11:00+08:00 `stage: work` | PLAN-071 | 需求⑥（流式状态可见性）|
  `outcome: pass` | code_commit: 需求⑥提交 | evidence: 会话 2882117a 实况
  （467s 长思考任务全程有状态反馈、收束正常）| blockers: 无 | next: review。
- 2026-09-18T11:50+08:00 `stage: work` | PLAN-071 | 需求⑥ T-29（deadman 续期）|
  `outcome: pass` | code_commit: deadman 续期提交 | evidence: 用户回验定位第二
  根因（120s 硬过期 → 长任务永久沉默）+ 浏览器回归全链路正常 | blockers: 无 |

- 2026-09-18T12:20+08:00 `stage: work` | PLAN-071 | r8 需求⑦实施完成 |
  `outcome: pass` | code_commit: 思考块沉淀 + 渲染修复提交 | task_ids:
  T-31..T-33 完成 | evidence: 浏览器实测流式期思考块实时增长、收束后
  ThinkBlock 折叠渲染（💭 已思考·100 token）、正文/工具卡不受影响 |
  blockers: 无 | next: review（需求①-⑦全数就绪）。
- 2026-09-18T11:20+08:00 **浏览器实机复证（新构建）**：新建会话发送数学任务，
  700ms 粒度采样 DOM——深度思考期状态行常驻（0.7–8.4s 连续可见），7s 起正文
  流式增长（页面文本 5223→5311→5324B），11.5s 收束停止钮消失。**流式全链路
  在新构建实证可用**；用户报告的空白 = 旧 bundle 行为（需求⑥修复前）+ 页面
  未刷新加载新构建。用户侧操作：刷新一次加载新 bundle 即可获得持续反馈。
- 2026-09-18T02:20+08:00 `stage: work` | PLAN-071 | r5 | `outcome: pass` |
  code_commit: r5 提交（需求④），分支 plan-071-dev | task_ids: T-17..T-21 完成 |
  evidence: 浏览器 DOM 实测四条（会话标题 ellipsis+title、× 第二行右缘 gap=0、
  首盒 topPad 6px/4px、规范盒两行+计数 goals 11/architecture 8/designs 10）+
  截图确认；实施补丁 h-auto（Button h-10 定高） | blockers: 无 | next: review
- 2026-09-18T09:09+08:00 `stage: work` | PLAN-071 | r5 用户回验（hover 高度
  跳动）| `outcome: pass` | code_commit: 计数行 text-sm 提交 | evidence: 用户
  截图实证 hover 时卡片被顶高——"n 条" text-xs 行高 16px < × 盒 20px；计数行
  改 text-sm + leading-5（14px/20px 同高），浏览器实测 hover 前后卡高 62→62
  差 0；规范面板计数行同步保持一致 | blockers: 无 | next: review。
- 2026-09-18T09:34+08:00 **阶段落地收据（phase-1，用户指示先合后审）**：
  main 合并 plan-071-dev --no-ff → merge commit `3c2b178`（10 提交，22 文件
  +625/-179）；落地门禁 cargo nextest -p musk 640 passed/4 skipped +
  vm-link-probe PASS（77565B）+ auto build 绿。计划簿记随合并上 main
  （dcd254c 建档+簿记）。**计划保持 executing**（用户：后续还有需求继续
  追加、review 后置）；worktree `.wt/musk-071/auto-musk` 与分支
  plan-071-dev 保留续用；SD-01/02/03 仍为分支预备态（发布+ledger 挂载随
  正式 review/merge）。运行态：:8080 serve = worktree 产物（含全部修复），
  main 检出若要自跑需重编后端 + 重建 dist。
  （需求①②③④全数就绪，可一并复审 merge）。

- 2026-09-18T12:45+08:00 `stage: work` | PLAN-071 | 运维事项登记（用户回验
  "仍无流式"澄清）| `outcome: pass` | evidence: 会话 2de6c8a7 数据正常
  （12:34:37 发问 / 12:34:45 收束含思考块）+ 干净标签页渲染完整（Assistant
  身份 + 💭 已思考 57 tokens + 全文）；用户空白 = SPA 长驻页面跑旧 bundle
  （整页刷新即加载新前端，会话内切换不触发加载）| 已知事项：部署新前端后需
  整页刷新（非代码缺陷，登记备查）| blockers: 无 | next: review。

- 2026-09-18T12:35+08:00 `stage: work` | PLAN-071 | 需求⑦终验（用户会话
  f402ca5a 实拍）| `outcome: pass` | evidence: 浏览器实拍——Assistant 身份头 +
  💭 已思考·73 tokens 折叠块 + 完整回复渲染正常；4062c66e 旧会话为修复前数据
  （无思考块属预期，自愈逻辑仅对 msg.thinking 非空的旧消息生效）| blockers: 无 |

- 2026-09-18T13:55+08:00 `stage: work` | PLAN-071 | r9 需求⑧实施完成 |
  `outcome: pass` | code_commit: 需求⑧实施提交（分支 plan-071-dev）|
  task_ids: T-34..T-37 完成 | evidence: 浏览器全流程实测——乐观标题栏 0.6s
  即现、ThinkBlock 半开态常驻、正文流式增长、收束一致；用户并行测试消息
  渲染完整 | blockers: 无 | next: review（需求①-⑧全数就绪，一并复审
  merge）。
- 2026-09-18T12:20+08:00 `stage: work` | PLAN-071 | 需求⑥流式诊断埋点 |
  `outcome: pass` | code_commit: 调试埋点提交 | 前端加 [SSE]/[POLL] console
  日志（事件分发/块追加/PollStream 决策/窗口过期），供用户浏览器实测定位
  "流式不渲染"断点；临时日志 merge 前决定去留 | next: 用户复测回报 console
  输出。
- 2026-09-18T14:50+08:00 `stage: work` | PLAN-071 | 需求⑧回验（用户
  实测流式已通，裁定双流式输出冗余）| `outcome: pass` | code_commit:
  状态行移除提交 | evidence: 用户实测确认流式渲染已通；ThinkBlock 半开态
  独占思考期反馈，状态行组件删除 | blockers: 无 | next: review。
- 2026-09-18T15:35+08:00 `stage: work` | PLAN-071 | r10 需求⑨实施完成
  `outcome: pass` | code_commit: bfcc152（worktree，分支 plan-071-dev）|
  task_ids: T-38..T-42 完成 | evidence: 门禁 nextest 644/4 + vm-link-probe
  PASS 77560B + auto build 绿；浏览器实测——等待点 0.6s 即现/思考到达即消、
  工具栏 Copy/⑂/■ 三钮并排、独立停止钮退役、取消链路端到端（serve 日志
  "chat cancel requested (in-flight=true)" + 前端按钮 901ms 消失 + run 72s
  干净收束）；T-42 = T-25 cancel 端点漏接真实路由表的缺陷修复（8090 serve
  已换新二进制）| blockers: 无 | next: review（需求①-⑨全数就绪，一并复审
  merge）。
- 2026-09-18T16:05+08:00 `stage: work` | PLAN-071 | r11 需求⑩实施完成
  `outcome: pass` | code_commit: fac6757（worktree，分支 plan-071-dev）|
  task_ids: T-43..T-45 完成 | evidence: 门禁 nextest 644/4 + vm-link-probe
  PASS 77563B + auto build 绿；浏览器四场景实测——新建会话默认显示低、
  默认低发消息产出思考块（11 tokens）、显式关无思考块、刷新重进关保持；
  8090 serve 已换新二进制 | blockers: 无 | next: review（需求①-⑩全数
  就绪，一并复审 merge）。
- 2026-09-18T16:35+08:00 `stage: work` | PLAN-071 | r12 需求⑪实施完成
  `outcome: pass` | code_commit: 701c013（worktree，分支 plan-071-dev）|
  task_ids: T-46..T-47 完成 | evidence: 纯前端（后端零改动）auto build 绿
  + vm-link-probe PASS 77545B；实机截图 + DOM 实证——计划导航两行盒 64px
  （标题 truncate + hover 全名 / 副标题 badge "drafting"）、选中态 primary
  双类、点选详情正常；实施注记 R016（text 点路径开头多段拼接 → span 内
  多 text 子节点规避）登记 T-46 | blockers: 无 | next: review（需求①-⑪
  全数就绪，一并复审 merge）。
- 2026-09-18T17:05+08:00 `stage: work` | PLAN-071 | r13 需求⑫实施完成
  `outcome: pass` | code_commit: 6fbfc2c（worktree，分支 plan-071-dev）|
  task_ids: T-48..T-50 完成 | evidence: 纯前端 auto build 绿 +
  vm-link-probe PASS 77591B；实测——打开计划页第一篇即选中 + 详情自动
  渲染、空态提示消失、点选切换不变；技术注记：生成 store handler 为
  async，自动选中须落在 LoadPlans 链内、高亮以 store.current 为准
  （登记 T-49）| blockers: 无 | next: review（需求①-⑫全数就绪，一并
  复审 merge）。
- 2026-09-18T18:40+08:00 `stage: work` | PLAN-071 | r14 需求⑬实施完成
  `outcome: pass` | code_commit: ab6645b（worktree，分支 plan-071-dev）|
  task_ids: T-51..T-54 完成 | evidence: nextest 644/4 + vm-link-probe PASS
  77602B + auto build 绿（serve 已换新二进制）；实测——@ 菜单 13 行三源
  （9 Agent+1 计划+3 规范）左对齐 42px、@001 过滤唯一命中插入 @plan/001、
  端到端 agent 从注入块答出 docs/plans/001-bootstrap-auto-edit.md 且
  可见消息保持原文 | blockers: 无 | next: review（需求①-⑬全数就绪，
  一并复审 merge）。
- 2026-09-18T19:05+08:00 `stage: work` | PLAN-071 | r15 需求⑭实施完成
  `outcome: pass` | code_commit: 23c5add（worktree，分支 plan-071-dev）|
  task_ids: T-55..T-57 完成 | evidence: 纯前端 auto build 绿 +
  vm-link-probe PASS；实测——@ 打开与筛选后第一项均为选中态、Enter 补全
  不发送/正常态发送正常、用户气泡渲染 📋 @plan/001 高亮 token；用户并行
  回验触发需求⑤ busy 提示（正向旁证）。已知项：composer 内 token 着色
  需上游 __autoMentionHtml 匹配器改造（auto-lang），登记后续 |
  blockers: 无 | next: review（需求①-⑭全数就绪，一并复审 merge）。
- 2026-09-18T20:10+08:00 `stage: work` | PLAN-071 | r16 需求⑮实施完成
  `outcome: pass` | code_commit: 4d156c4（worktree，分支 plan-071-dev）|
  task_ids: T-58..T-61 完成 | evidence: nextest 644/4 + vm-link-probe
  PASS + auto build 绿（serve 已换新二进制）；端到端——长任务在途发
  第二条 → "⏳ 已排队 1 条"且无 busy、首轮收束后自动发送独立成轮回答、
  chats.json 四 turn 齐全；连带修复 ChatStore 并发丢写（write_lock 12
  方法串行化）。登记：parity_plans 单测偶发失败为并行噪声（隔离+复跑
  全绿）| blockers: 无 | next: review（需求①-⑮全数就绪，一并复审
  merge）。
## 10. 待澄清事项
- 无阻塞项。备注①：vm_backend.rs `chat_run_stream` host 当前无 front 调用方
  （grep 实证），若后续 VM 前端接线，按新语义即订阅；备注②：后续追加的可用性
  修改需求在本计划以新 T-/AC-/SD- 追加并递增 plan_revision，不另开档（用户
  指定的跟踪方式）。
