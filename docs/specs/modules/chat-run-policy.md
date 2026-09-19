# chat 运行策略（Chat Run Policy）模块规范

> PLAN-069 落地（2026-09-15，reviewed 3d94726）。会话 81b45c34 四问题综合改善：
> 会话沙箱绑定、消息块时间线、工具级审批门、SSE 订阅/运行生命周期。
> PLAN-073 Phase 2 增订（2026-09-20，reviewed e5e098e）：收束三臂统一组装
> 现场 + turn 增量落盘（pending 快照 / upsert 原位替换保树链）+ 空闲订阅
> idle 帧即收；详见"收束落盘与增量快照"节与 chat-streaming 契约。

## 会话沙箱绑定（fail-closed 不变量）

1. **单一真源**：工具沙箱根恒 = `session.workspace_id` 经服务端解析出的 workspace
   根（`WorkspaceRegistry::get_exact` 严格版：未知/缺失 → Err）。客户端
   `?workspace=` 仅作交叉校验。
2. **fail-closed**：/api/run、SSE run、ag agent_run/run_stream_handler、workflow
   run 解析不出有效 workspace → 400 拒绝启动，**无 CWD/首仓静默回退**。
3. **注入式注册**：`build_agent_from_mode(mode, client, ws_root: Option<Arc<PathBuf>>)`
   条件注入八个文件/命令工具（`with_root`，对齐 relay 路径 `build_agent_with_context`）；
   生产入口禁止传 None。`tool_safety::set_current_root/clear_current_root` 自全部
   运行路径退役（API 保留供测试）——thread-local 在 tokio 线程迁移下失效
   （PLAN-030 同类缺陷），"工具注册必须显式注入 root"为不变量。
4. **报错即事实**：`tools.rs map_path_error` 携带工具**实际 scope root**（注入根
   canonicalize），仅在未注入时回退 `project_root()`——报文不得谎报沙箱根
   （81b45c34 实证：模型据错误文案向用户转述假根）。

## 消息块时间线

- 块模型：`ChatMessage.blocks = [ {kind:"text",text} | {kind:"tool",id,name,args}
  | {kind:"tool_result",id,status,output} ]`，按严格执行序追加。
- 组装规则：每次 ReAct 迭代的叙述文本 = **新 text 块**（跨轮不合并）；tool_call
  与其 tool_result 各自成块、按序插入。
- 派生兼容：`content` = 全部 text 块按序拼接（导出/搜索兼容）；`tool_calls` =
  全部 tool 块引用（旧前端兼容）。
- 前端：blocks 非空 → 逐块渲染（工具卡原位穿插，最终回答位于最后一个工具卡
  之后）；blocks 为空 → content+tool_calls 旧渲染分支（历史会话兼容，AC-08）。
- 转写补尾：run 收束时最终 assistant 消息补入 turns.jsonl（消除"尾 turn =
  tool_result"的双存储悬尾）。

## 收束落盘与增量快照（PLAN-073 Phase 2）

- **三臂统一组装**：成功/失败/超时三种收束一律组装完整现场（叙述块封口 →
  思考首块 → 已积累工具卡 → 超时/失败的 ⚠ 尾块），共用
  `assemble_chat_run_msg`（clone 语义，不消耗累积器）。失败臂不再仅落一行
  ⚠ 文本（24a43d74 跑满 100 轮失败全现场丢失实证）；零现场时退化为仅 ⚠ 尾块。
- **turn 增量落盘**：run 起跑即定本 run 的消息 id；每个 `turn_end` 边界把
  当前积累（thinking/工具卡/叙述）以 `pending=true` 快照 upsert 落盘——
  run 进行中刷新页面即可见已完成轮次的现场。
- **终版同 id 换入**：收束终版与快照共用消息 id，`ChatStore::upsert_message`
  按 id 原位替换并清除 pending；session 内无该 id 时退化为 append（parent/
  leaf 语义同 append_message）。
- **树链不变量**：原位替换必须保留原消息的 `parent_id` 与 `created_at`——
  终版由收束组装独立构造（不重导 parent），整体换入会把消息踢出所在分支
  （`upsert_message_replaces_in_place` 测试实证）。
- **落盘不静默**：收束/快照落盘统一走 `persist_chat_run_msg`——写失败与
  session 缺失（Ok(None)）均显式 WARN（原三臂 `let _ =` 静默吞错）。
- **turns 双写粒度**：conversations turns 镜像维持收束时一次——增量快照
  不镜像（避免半轮 turn 重复入 journal）。

## 工具级审批门（human 会话）

- 触点：`run_command` 预执行 `tool_safety::confine_offending_paths(cmd)` 收集
  越界 token（保守匹配：命令行任一 token 解析出根外路径即候选）；**首个越界
  即暂停**（逐命令逐问，不批量）。
- human 判定：`!force && gate_session.is_some()`。挂起 = `tool_gate::register`
  oneshot + ProgressSink `send_gate_waiting`（SSE `tool_gate_waiting` 事件 →
  ToolGateCard 卡片：命令全文 + 越界路径）。
- 决议：`POST /api/chats/tool-gate/{gid}/approve|deny`。approve → 放行执行该命令
  **一次**（放行半径单次，不做前缀/目录授权）；deny → `[security denied]` 拒绝
  回灌、运行继续。**1800s 超时 = deny**。
- auto / force / 无会话：维持硬拒 + 继续（PLAN-027 ③口径不变），不弹门。
- 文件读写工具无门：越界恒硬拒（v1 仅命令类工具有"先问再放行"语义）。

## SSE 订阅/运行生命周期

> PLAN-071 增订（2026-09-17）：会话 `18683b29`（auto-edit）一句话两答实证 F-03
> 窥探分流的结构性缺陷——守卫状态对"POST 孵化的主体"与"1ms 后到达的订阅者"
> 同值，`if !run_owner` 反置使在途订阅并行再跑一轮。改**显式角色分离**，并将
> PLAN-069 T-06 未接线的事件双发补全。

- **运行孵化唯一入口**：`chats_message run=true` → `chat_run_try_start`
  （per-session 守卫键 `{ws_id}:{session_id}`）抢到 → spawn `chat_run_owner`
  运行主体（含持久化，守卫由其全部出口清除）；未抢到（已在途）→ 不重复孵化。
- **守卫占用（busy）语义**：未抢到守卫的请求**不孵化**，用户消息照常持久化；
  真实用户提交 → 落一条**可见 busy 提示**（持久化，替代旧静默吞掉）+ WARN
  日志 + 响应体 `busy:true`；携带 `queued=true` 的请求（前端队列自动重发）
  **只回 `busy:true` 不落提示**（重试每秒可达，落提示会刷屏）。
- **在途消息排队**（前端）：流式在途时新消息入 store 队列（画布"⏳ 已排队
  n 条"指示），收束后按 FIFO 自动重发（queued=true；busy 竞态窗口内退回
  队首下拍重试）——后端 busy 链路保留为跨标签页/竞态兜底。
- **空闲看门狗**：`AUTO_RUN_IDLE_TIMEOUT_SECS`（默认 300s）窗口内无任何
  流式事件 → drop run_stream future（中止在途 LLM 流）→ 按超时收束：错误
  事件 + **错误消息持久化** + 守卫清理。run_command 默认超时
  `AUTO_CMD_TIMEOUT_SECS`（默认 300s，超时杀树保留部分输出）。
- **取消**：`POST /api/chats/session/{id}/cancel` 置位 per-session 取消旗标
  （owner 孵化注册、出口移除）；agent 于迭代边界检查收束并落盘已有内容；
  流内 stall 由看门狗兜底。
- **生命周期日志**：spawned / finished(elapsed) / failed(原因) / busy 全打
  INFO/WARN——运行故障可从 serve 日志直接判读。
- **会话存储写串行化**：ChatStore 全部变更方法持 write_lock 串行化
  （load-modify-save 非原子；运行主体落盘回复与并发消息 POST 交错时后写
  覆盖前写，实证丢回答）。
- **显式角色分离**：SSE `chat_stream` 与 VM 桥的 `chat_run_stream`（签名不变、
  调用点零改动）**恒为附加/空闲订阅**——在途 → 附加转发 relay_bus 事件
  （run_id=session_id；chat_event / tool_update / tool_gate_waiting /
  relay_gate_waiting）直至 done；不在途 → **空闲即收**：入口以
  `chat_run_active` 只读窥探，立即回 `{"type":"idle"}` 帧并关流（PLAN-073
  P2-T4 修订；原"挂起等下一次运行"的空闲流语义退役——前端据此复位运行
  态，不再悬挂空流；收束竞态窗口内 try_recv 捞到本会话 done 尾巴则转发）。
  订阅**绝不孵化运行、不触碰守卫**。以共享守卫窥探分流主体/订阅者的 F-03
  方案（`chat_run_active` + `if !run_owner`）退役。
- **事件双发**：`chat_run_owner` 把全部 SSE 事件（含 plan-merge 短路四帧）
  镜像上 relay_bus（`event_type=chat_event`）。PLAN-069 T-06 的 emit_bus 闭包
  此前只有定义无调用点（死代码）——流式事件只进本运行 tx（chats_message
  spawn 路径恒 Null），订阅可见流曾由 F-03 缺陷（订阅方自行再跑）意外顶替。
- **订阅收束**：附加订阅 done 后必须析构桥接 channel pair（`close_channel`）
  ——Sender 常驻 HANDLES 表，tx Value 掉落不关闭通道，漏调则 SSE 永不收束。
- **无重跑**：同一会话二次订阅 / EventSource 断线自动重连不产生新运行；双订阅
  收到同一运行同序事件流；延迟重连接续在途运行。
- **运行 spawn 化**：ag chat_stream 立即返回 SSE、运行后台 spawn——事件不再积压，
  断线重连不拿"事后重放"报 malformed。
- **直播优先**：`.streaming && stream_es != None`（web 轨 SSE 已附加）期间
  PollStream 回填**整体跳过**——回填快照不含在途 assistant，LLM 长思考/迭代
  间隙超 3s 健康门放开后回填会清掉直播内容（F-04 用户实测"每轮边界删掉重显"
  根修）；done 臂落 streaming=false 后回填恢复兜底。VM 轨 stream_es=None
  （Sse.open no-op）不受影响。

## 测试口径

cargo lib：get_exact 严格性、denial-root 报注入 scope、blocks 投影顺序与 legacy
兼容、run 守卫防双跑、运行主体路径经 chats_message run=true（relay_bus 双发收
到 delta/done、恰一条助手回复、收束后守卫清除）、18683b29 回归序列（run:true
POST 后紧跟 SSE 订阅 = 附加转发同一事件流至 done、恰一条助手回复）、裸订阅恒
空闲流不终止/不孵化/不触碰守卫（PLAN-071 口径；F-05"先 try_start 取守卫再订阅
= 运行主体路径"口径随窥探机制一并退役）；E2E v3/v4：实时流 llm_alive、
human 门 t≈8s 首触暂停 / approve 200 resolved / deny 回灌、run=false 双订阅
0 字节零运行、多轮 DOM 采样直播零清空（drops=[]）。
