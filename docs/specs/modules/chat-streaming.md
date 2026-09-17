# chat-streaming — 对话流实时性契约（PLAN-067 SD-01）

> 来源：PLAN-067 r2（reviewed 57b5dba / delivery 于 main）。行为证据：T-01 探针
> 定责 + T-02 修复实测（乐观消息 ≤2s 上屏、流式增量实时渲染）。

## 契约

1. **SSE 主通道**：chat 会话的 assistant 回复、工具 Block、`gate_waiting` 等
   run 事件必须经 `GET /api/chats/session/{id}/stream`（SSE）增量实时推送。
   后端 `chat_run_stream`（extern_impl）运行由 `chats_message run=true` 取守卫
   孵化（PLAN-069 W4/F-03 后订阅不孵化——订阅即附加/空闲流），事件经
   mpsc→SSE 逐段下发；事件为无名（message）SSE 事件，前端以 `onmessage` 接收。
2. **实时性口径**：发送后乐观 user 消息立即上屏；assistant 首增量 ≤2s 内渲染；
   工具 Block / gate 卡片出现延迟 ≤2s。违反任一即缺陷，不得以"刷新后可见"替代。
3. **叶链投影一致性**：会话渲染按 `chatActivePath(messages, active_leaf)` 父链
   投影。任何写入 `.messages` 的新消息（乐观 user、流式 assistant、回填数据）
   必须保证 active_leaf 指向其路径（新增消息挂 `parent_id` = 当前叶并推进叶；
   轮询回填时同步服务端 `active_leaf`）。**只写数组不推进叶 = 渲染不可见缺陷**
   （PLAN-067 T-01 实测定责，症状"AI 回复不自动刷新"）。
4. **轮询为兜底**：`PollStream`（500ms，deadman 窗 2 分钟）仅在 SSE 不健康时
   承担回填（PLAN-067：3s 内有流事件则跳过）。**PLAN-069 F-04 收紧**：
   `.streaming && stream_es != None`（web 轨 SSE 已附加）期间回填**整体跳过**——
   回填快照不含在途 assistant，健康门放开后的回填会清掉直播内容（实测每轮
   边界"删掉重显"）；done 臂落 streaming=false 后回填恢复兜底。完成启发式
   （回合增长守卫）保留。
5. **双轨注记**：VM 轨无 SSE，轮询即主通道（Plan 051 T10 形态），叶同步规则
   同样适用；SSE 健康门在 VM 恒开（OnStreamEvent 不触发），不影响 VM 轮询。
6. **块化组装规则（PLAN-069 W2）**：每次 ReAct 迭代的叙述文本独立成 text 块
   （跨轮不合并）；tool_call 与 tool_result 成对入块、按执行序穿插；`content`
   为全部 text 块的派生拼接、`tool_calls` 为 tool 块引用（兼容面）。前端
   blocks 非空逐块渲染，为空走旧 content+tool_calls 分支。详见
   `modules/chat-run-policy.md`。

## 关联实现

- 前端：`src/front/forge_store.at`（StartStream/OnStreamEvent/PollStream、
  `last_sse_at` 心跳、叶推进）、`mention_input.at`（composer）。
- 后端：`auto_generated/extern_impl.rs chat_run_stream`（订阅触发 + mpsc 桥）、
  `auto_generated/server_stream.rs`（SSE 出口）。
- 历史债务 KD 059-FU1（"AI 回复了但界面不动"）已收口（PLAN-066 T-08）：
  VM 轨 `Sse.open(url, .Handler)` 的 handler-as-value 实参在合成 fn 内改写为
  handler fn 裸引用（值位置 CLOSURE，JS this.method 语义），不再落 GET_FIELD
  抛 "Field not found" 中止 StartStream。
- **绕行层 enduring 契约（VM 轨 SSE 保持 no-op 传输期间，PLAN-066 T-08 复盘
  逐项裁定全部保留）**：①streaming 置位先于 Sse.open；②StopStream 头部直连
  close；③回合增长守卫（pre_stream_len 基线）；④deadman 2 分钟时间窗（窗戳
  经列表 push 承载）——**立场：保留**（无真 SSE 事件流下唯一恢复路径）；
  ⑤SSE 健康门（last_sse_at 3 秒内不回填，067）；⑥流式期跳过回填（069）。
  前端 blocks 消费面不变。
