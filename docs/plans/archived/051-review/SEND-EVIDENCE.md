# PLAN-051 T8 实测证据（发送往返 + 渲染现状）

环境：worktree plan-051-dev（musk .at 修正后）× auto-lang worktree release exe
（C1-C4 + retain 修复 + 回归锁）；后端 :8081（沿用 PID 30976 实例）；MCP :9252。
账号 plan048user。会话 = "nihao"（7a7e6788d44fbedb92114e9a）。

## 发送链路（✅ 全通，数据面）

- 会话切换：NavListItem.activate → [VM-EMIT] → ChatsView.SelectSession →
  store.SwitchSession → session_id 变更 + chats_get_session 回读（messages
  vmref 入态）。切换后 nihao 的 2 条消息驱动气泡渲染。
- 按钮发送：MCP press → MentionInput.send(.text)（draft 清空实证）→ C2①
  声明式 onsend → ChatsView.SendInput(text)（实参契约修正后）→
  store.Send + StartStream → streaming=true（state 实证）→
  chats_send_message POST → 后端 count 2→3，末条 user
  'plan051 t8 send test'（curl 实证）。
- 材料：01-chat-before-send.txt / 02-after-send.txt（MCP 快照）、
  vm-run-t8.log（运行日志）、t8-01-chat-bubbles.png / t8-02-after-send.png
  （截图，待多模态分析——样式细节：左右分向/圆角/hairline 壳/正文降级形态）。

## 渲染现状（部分）

- ✅ 气泡结构：🧑 You / 🤖 AI 徽章、分叉重试钮（⑂ 重试）、气泡容器、
  gate/run 卡片共存结构。
- ⚠️ 正文空白：messageDisplayBlocks 返回列表在 for 回退解引用为 0 行。
  勘察链（单位级二分）：let 绑定调用结果再迭代的形状通过；直接
  `for b in fn(...)` 与 musk 真实函数体（循环体含 if/else-if/else + 嵌套
  let + ?? + 调用）仍触发 VM 栈失衡（返回值错位）→ 上游债（见计划待澄清6）。
- ⚠️ 时间戳空：msgTimeLabel 依赖 Date.format native，VM 无此形态（上游项）。
- i18n 参数模板（{count} 条/${runId}）未插值（T1 已记录侧观察）。

## 视觉核对（2026-08-30 多模态补录，T8 收口）

t8-01（发送前）逐项（图像分析 × vtree 双证）：
- ✅ **左右分向**：You 徽章右对齐 / AI 徽章左对齐（一右一左相反确认），
  与 self-end items-end / self-stretch 类串契约一致。
- ✅ **AI hairline 壳**：AI 气泡区上下细线可见（border-t border-b 消费成立，
  PLAN-050 T4 单侧边框能力在会话页生效）。
- ✅ **gate/run 卡**：🔒278 + Approve & Execute（绿色实心）/ Reject &
  Redraft / Review 三钮 + ✅ run 摘要行，渲染在位。
- ✅ **输入区**：textarea（placeholder 完整）+ 发送按钮结构在位。
- ⚠️ 用户气泡圆角底色不可见：气泡容器随正文空白塌缩（与正文空白同根因，
  非独立缺陷）。

t8-02（发送后）：
- ⚠️ **发送的用户气泡不可见**（聊天区仅 gate/run 卡 + "AI 思考中..."指示）：
  与数据面证据（后端 count 2→3）合流定责——乐观 push 只存在于 web 轨
  forge_stream.ts（Sse.open 建连前），VM 轨无 SSE 无轮询 → 发送后无本地
  入列也无增量拉取。**定责 T10（PollStream 轮询）范围**，非 C1-C4 回归。
- ⚠️ ${runId}/${durationLabel}/${confidenceLabel} 原样显示（i18n/插值上游项，
  待澄清6 连带登记）。

## 截图视觉核对（2026-08-30 多模态分析：t8-01 / t8-02）

| 验收项 | 结论 | 证据 |
|---|---|---|
| 左右分向 | ✅ 正确——"🧑 You" 徽章右上对齐，"🤖 AI" 徽章左侧，与 vue 版一致 | t8-01 右侧消息区 |
| hairline 壳 | ✅ 可见——You 行上方与 AI 行下方有细分隔线（msg-blocks hairline utilities 生效） | t8-01 |
| 圆角/气泡壳 | ⚠️ 无法核对——正文空白（上游 for-in 栈失衡债），气泡填充体不存在，圆角无从谈起；徽章+分隔线结构本身正确 | t8-01 |
| Markdown 降级形态 | ❌ 同上无法核对——正文区空白（内容可读性验收待上游债清偿后复验） | t8-01 |
| 名徽章 | ✅ 🧑 You / 🤖 AI 渲染正确（emoji+文案） | t8-01 |
| 时间戳 | ❌ 空白——msgTimeLabel 依赖 Date.format，VM 无此 native | 快照无时间文本 |
| 侧栏 active 态 | ✅ nihao 项高亮边框+主色文字，切换态正确 | t8-01/t8-02 左栏 |
| gate 卡 | ⚠️ 渲染在但 "278" 为数字（应 gate 标题）；Reject & Redraft / Review 按钮**深底深字**几乎不可见（outline/secondary variant 文字色缺失） | t8-01 |
| run 卡 | ⚠️ `${runId}${durationLabel}${confidenceLabel}` 裸模板（i18n/模板插值未生效，T1 已登记） | t8-01 |
| composer | ⚠️ placeholder 不可见（textarea 区域空），右下蓝色圆形发送钮可见；draft 回写/清空在数据面已证 | t8-01 |

## 发送后状态（t8-02）与新根因

- t8-02：气泡**整体消失**（含历史 2 条）+ "AI 思考中..." 指示器在（streaming
  派生文案渲染 OK——T10 相关能力活着）。
- **日志铁证**：`[VM-EMIT] MentionInput.send -> ChatsView.SendInput failed:
  Field 'OnStreamEvent' not found on type instance App_State (crash ip=0xe1dd
  in handler_ChatsView_SendInput)`——C2 emit 派发正确、store.Send 落库成功
  （后端 2→3 条），**StartStream 内 `Sse.open(path, .OnStreamEvent)` 的 msg
  变体引用被当 state 字段 GET_FIELD → RuntimeError 中断 handler**。
  KD-047 G1（SSE 无 VM 形态）的实机形态比"静默断链"更重：**崩溃中断 +
  疑似级联腐坏**（其后 messages=[] 的写入者未定位——唯一精确写入点
  NewSession/DeleteAllSessions 均未触发；伴发 600+ 次 GET_FIELD
  field=length non-i32 刷屏）。T10 的 platform.vm.at Sse no-op 须**连同
  解决 msg 变体引用求值**（裸 .MsgVariant 在表达式位 → Nil/msg-ref，而非
  GET_FIELD 崩溃），StartStream 路径被 PollStream 替换后复验 messages=[]。
- 附：复核时误触过期 vnode 触发 NewSession（新建 1ff667 空会话）——复现了
  `.messages = []` 语义本身正确（新会话清空视图）。
