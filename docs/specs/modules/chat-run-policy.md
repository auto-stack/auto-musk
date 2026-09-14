# chat 运行策略（Chat Run Policy）模块规范

> PLAN-069 落地（2026-09-15，reviewed 3d94726）。会话 81b45c34 四问题综合改善：
> 会话沙箱绑定、消息块时间线、工具级审批门、SSE 订阅/运行生命周期。

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

- **运行孵化唯一入口**：`chats_message run=true` → `chat_run_try_start`
  （per-session 守卫键 `{ws_id}:{session_id}`）抢到 → spawn `chat_run_stream`
  运行主体（含持久化）；未抢到（已在途）→ 不重复孵化。
- **订阅即附加**：SSE `chat_stream` 以 `chat_run_active` **只读窥探**分流——
  在途 → 附加转发 relay_bus 事件（run_id=session_id；chat_event / tool_update /
  tool_gate_waiting / relay_gate_waiting）直至 done；不在途 → **空闲流**（挂起
  等下一次运行）。订阅**绝不孵化运行**（F-03 根修：订阅抢守卫变身运行主体 =
  4 连跑实证，已根除）。
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
兼容、run 守卫防双跑、ag_chat_stream 运行主体路径（先 `chat_run_try_start` 取守卫
再订阅——裸订阅恒空闲流不终止，复审 F-05 口径）；E2E v3/v4：实时流 llm_alive、
human 门 t≈8s 首触暂停 / approve 200 resolved / deny 回灌、run=false 双订阅
0 字节零运行、多轮 DOM 采样直播零清空（drops=[]）。
