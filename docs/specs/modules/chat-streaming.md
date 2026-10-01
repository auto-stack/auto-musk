# chat-streaming — 对话流实时性契约（PLAN-067 SD-01）

> 来源：PLAN-067 r2（reviewed 57b5dba / delivery 于 main）。行为证据：T-01 探针
> 定责 + T-02 修复实测（乐观消息 ≤2s 上屏、流式增量实时渲染）。
> PLAN-073 Phase 2 增订（2026-09-20，reviewed e5e098e）：空闲订阅 idle 帧即收、
> 会话载入即附加（AttachStream）、完成启发式 pending 守卫。
> PLAN-077 增订（2026-09-20，reviewed fbca482）：双端 Block 状态/回放验收
> （契约⑧，SD-01）。

## 契约

1. **SSE 主通道**：chat 会话的 assistant 回复、工具 Block、`gate_waiting` 等
   run 事件必须经 `GET /api/chats/session/{id}/stream`（SSE）增量实时推送。
   后端 `chat_run_stream`（extern_impl）运行由 `chats_message run=true` 取守卫
   孵化（PLAN-069 W4/F-03 后订阅不孵化——订阅即附加；**空闲订阅立即回
   `{"type":"idle"}` 帧收流**——PLAN-073 P2-T4 修订，原"空闲流挂起"语义
   退役），事件经 mpsc→SSE 逐段下发；事件为无名（message）SSE 事件，前端以
   `onmessage` 接收。`idle` 帧属 SseEventDto 严格枚举之外的透传事件
   （server_stream 白名单），前端 `idle` 臂复位运行态并关流。
2. **实时性口径**：发送后乐观 user 消息立即上屏；assistant 首增量 ≤2s 内渲染；
   工具 Block / gate 卡片出现延迟 ≤2s。违反任一即缺陷，不得以"刷新后可见"替代。
3. **叶链投影一致性**：会话渲染按 `chatActivePath(messages, active_leaf)` 父链
   投影。任何写入 `.messages` 的新消息（乐观 user、流式 assistant、回填数据）
   必须保证 active_leaf 指向其路径（新增消息挂 `parent_id` = 当前叶并推进叶；
   轮询回填时同步服务端 `active_leaf`）。**只写数组不推进叶 = 渲染不可见缺陷**
   （PLAN-067 T-01 实测定责，症状"AI 回复不自动刷新"）。
4. **轮询为兜底**：`PollStream`（500ms tick，**deadman 活性窗=「最后一次回填
   成功后 2 分钟」**）仅在 SSE 不健康时承担回填（PLAN-067：3s 内有流事件则
   跳过）。**PLAN-069 F-04 收紧**：`.streaming && stream_es != None`（web 轨
   SSE 已附加）期间回填**整体跳过**——run 期间服务端仅存 turn 粒度增量快照
   （PLAN-073 P2-T3）、不含 turn 内直播尾部，健康门放开后的回填会清掉直播
   内容（实测每轮边界"删掉重显"）；done 臂落 streaming=false 后回填恢复
   兜底。完成启发式（回合增长守卫）保留，并增 **pending 守卫**（PLAN-073
   P2-T3）：末条为 pending 快照（非终版）时不清窗不收束，等收束终版同 id
   换入（pending 清除）后自然收束。
   **PLAN-092 T-02 r2 增订（VM 臂门控语义）**：活性窗门控**全 int tick
   计数**——`poll_seen_wins`（识别 Send/Attach 的 push 增长=活性）、
   `poll_idle_ticks`（>240 tick≈120s 无活性即清窗，**expired 臂清窗一次后
   静默**，空闲态 tick 在 wins==0 短路零日志）、`poll_inflight_ticks`（单飞
   防重入；>20 tick≈10s 未回收判响应丢失自动放行）。续窗信号=窗增长或回填
   成功复位 idle（原「`.streaming == true` 每 tick 续窗」退役——该标量 VM
   读垃圾恒真曾致 wins 无界增长）。VM 轨禁用时间戳/bool 读/列表索引读作
   门控（0927 实机定罪的平台读缺陷三件，见 vm-data-semantics 已知边界）；
   web 轨两道 SSE 门（live 门+3s 健康门）原样保留（web Date.now 为原生
   真时钟）。窗口戳仍以列表 push 承载（跨模块调用帧唯一可靠写，536 T12）。
5. **双轨注记**：VM 轨无 SSE，轮询即主通道（Plan 051 T10 形态），叶同步规则
   同样适用；SSE 健康门在 VM 恒开（OnStreamEvent 不触发），不影响 VM 轮询。
   **PLAN-092 实证**：r2 门控下 VM 轮询 2/秒全程到达、发送后零操作回复
   自动上屏（契约② VM 臂达成）、145s 空闲 0 请求 0 日志。
6. **块化组装规则（PLAN-069 W2）**：每次 ReAct 迭代的叙述文本独立成 text 块
   （跨轮不合并）；tool_call 与 tool_result 成对入块、按执行序穿插；`content`
   为全部 text 块的派生拼接、`tool_calls` 为 tool 块引用（兼容面）。前端
   blocks 非空逐块渲染，为空走旧 content+tool_calls 分支。详见
   `modules/chat-run-policy.md`。
7. **会话载入即附加（PLAN-073 P2-T4）**：会话打开（切换/刷新自动选中）即
   `AttachStream` 挂流——在途 run 的后续事件照常上屏（刷新前的前缀由 turn
   增量快照落盘补足），无在途即收 idle 帧。AttachStream **不预置
   streaming**：运行态由首个真实流事件置位（delta/thinking/tool_call/
   tool_result/turn_start/turn_end/tool_gate_waiting），done/idle/error 臂
   自行复位——旧后端（无 idle 帧）attach 后零事件即零幻态，发送路径的
   StartStream 契约不变。
8. **双端 Block 状态与回放验收（PLAN-077 SD-01）**：块投影必须携带
   thinking `state`（streaming/done——漏带会把流式思考误显"已思考"）与
   稳定块身份 `tkey`（按 `raw.id`/`块 id`，重排后不变、不串位）；gate
   载荷（`gate_id`/`pending_cmd`/`escape_paths`）随块投影直达审批卡；
   `gate_waiting` 状态直通显示，不得归并进 completed。展开态归 ForgeStore
   根态 JSON 键列表（`toggleBlockExpansion`）——多块/多消息独立展开、
   重排不串位、卸载复建保持。双端验收以 `tests/ui-parity` 同一断言集
   回放为准（VM snapshot 内容断言 + Vue 真实交互 + 事件/请求 spy），
   适用 case 双端全绿；未达项登记证据报告，不得藏入遗留桶。

## 关联实现

- 前端：`src/front/forge_store.at`（StartStream/AttachStream/OnStreamEvent/
  PollStream、`last_sse_at` 心跳、叶推进、idle 臂）、`mention_input.at`
  （composer）。
- 块消费面（PLAN-077）：`forge_helpers.at`（`messageDisplayBlocks`/
  `expandedMessageBlocks`——state/tkey/gate 载荷投影）、`forge_store.at`
  （`toggleBlockExpansion` 键列表）、`chat_message.at` / `tool_gate_card.at`
  （显式展示 props + `on_fork_from` 宿主路由）；mention 扫描统一
  `sub(i, i+1)` 取字符（VM `char_at` 按 Plan 368 W5 返回码点 int，
  web 返回 1 字符 string——双端分歧以 END 语义 `sub` 承载）。
- 后端：`auto_generated/extern_impl.rs chat_run_stream`（订阅触发 + 空闲窥探 +
  mpsc 桥）、`auto_generated/server_stream.rs`（SSE 出口 + 透传白名单含 idle）。
- 历史债务 KD 059-FU1（"AI 回复了但界面不动"）已收口（PLAN-066 T-08）：
  VM 轨 `Sse.open(url, .Handler)` 的 handler-as-value 实参在合成 fn 内改写为
  handler fn 裸引用（值位置 CLOSURE，JS this.method 语义），不再落 GET_FIELD
  抛 "Field not found" 中止 StartStream。
- **绕行层 enduring 契约（VM 轨 SSE 保持 no-op 传输期间，PLAN-066 T-08 复盘
  逐项裁定全部保留）**：①streaming 置位先于 Sse.open（AttachStream 因不
  预置 streaming 不受此约束）；②StopStream 头部直连 close；③回合增长守卫
  （pre_stream_len 基线）；④deadman 2 分钟时间窗（窗戳经列表 push 承载）——
  **立场：保留**（无真 SSE 事件流下唯一恢复路径）；⑤SSE 健康门（last_sse_at
  3 秒内不回填，067）；⑥流式期跳过回填（069）。
  前端 blocks 消费面不变。

## 附件与流式/轮询交点（PLAN-093）

- 附件快照随消息持久化（serde default 旧文件兼容）+ 分页通道
  （blocks_normalized）透传 design_context——刷新回放诚实标注（乐观
  push 无附件/持久化带标记双路径）。
- 流式期 composer 禁输 = 产品真实 UX：队列入队窗口 = 发送竞态秒级；
  busy 重入不换附件（队列条目级冻结）。
- VM 轮询轨与 web SSE 轨同构消费：tool_result details（UI 载荷）透传，
  刷新回放 details 缺省 None 兼容。
