# chat-streaming — 对话流实时性契约（PLAN-067 SD-01）

> 来源：PLAN-067 r2（reviewed 57b5dba / delivery 于 main）。行为证据：T-01 探针
> 定责 + T-02 修复实测（乐观消息 ≤2s 上屏、流式增量实时渲染）。

## 契约

1. **SSE 主通道**：chat 会话的 assistant 回复、工具 Block、`gate_waiting` 等
   run 事件必须经 `GET /api/chats/session/{id}/stream`（SSE）增量实时推送。
   后端 `chat_run_stream`（extern_impl）由订阅触发运行，事件经 mpsc→SSE 逐段
   下发；事件为无名（message）SSE 事件，前端以 `onmessage` 接收。
2. **实时性口径**：发送后乐观 user 消息立即上屏；assistant 首增量 ≤2s 内渲染；
   工具 Block / gate 卡片出现延迟 ≤2s。违反任一即缺陷，不得以"刷新后可见"替代。
3. **叶链投影一致性**：会话渲染按 `chatActivePath(messages, active_leaf)` 父链
   投影。任何写入 `.messages` 的新消息（乐观 user、流式 assistant、回填数据）
   必须保证 active_leaf 指向其路径（新增消息挂 `parent_id` = 当前叶并推进叶；
   轮询回填时同步服务端 `active_leaf`）。**只写数组不推进叶 = 渲染不可见缺陷**
   （PLAN-067 T-01 实测定责，症状"AI 回复不自动刷新"）。
4. **轮询为兜底**：`PollStream`（500ms，deadman 窗 2 分钟）仅在 SSE 不健康时
   承担回填（PLAN-067：3s 内有流事件则跳过——运行中服务端快照不含未完成
   assistant，回填会清掉流式内容）。完成启发式（回合增长守卫）保留。
5. **双轨注记**：VM 轨无 SSE，轮询即主通道（Plan 051 T10 形态），叶同步规则
   同样适用；SSE 健康门在 VM 恒开（OnStreamEvent 不触发），不影响 VM 轮询。

## 关联实现

- 前端：`src/front/forge_store.at`（StartStream/OnStreamEvent/PollStream、
  `last_sse_at` 心跳、叶推进）、`mention_input.at`（composer）。
- 后端：`auto_generated/extern_impl.rs chat_run_stream`（订阅触发 + mpsc 桥）、
  `auto_generated/server_stream.rs`（SSE 出口）。
- 历史债务：KD 059-FU1（"AI 回复了但界面不动"）——VM 轨 Sse.open 抛点问题
  （根修归上游 SSE 专项）；web 轨同症状由本契约 T-02 根修。
