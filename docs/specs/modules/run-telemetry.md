# run telemetry — 运行遥测与模型归因（PLAN-098）

> PLAN-098 定型（2026-10-03，reviewed 319cb4f）。聊天会话与 relay run 的
> 每轮 LLM 请求的模型归因、用量与耗时记录契约：让 plan-flow-uat 重跑的
> "每一步由哪个模型服务、花了多少"成为可对比运行的一等事实，数据落在受
> 版本控制的记录物（turns.jsonl 字段 + collect-telemetry 摘要），不依赖
> daemon 日志存活或时间戳手工对齐。实现锚点：`src/telemetry.rs`
> （TelemetryClient/TelemetryContext/TelemetrySink/fold_records）、
> `relay/driver.rs`（factory 注入）、`auto_generated/extern_impl.rs::
> chat_run_owner`（聊天注入与折叠）、`relay/store.rs::submit_handoff`
> （relay 折叠单点）、`docs/reports/plan-flow-uat/collect-telemetry.mjs`。

## 记录契约

- **每轮 assistant 产物尽可能携带 `telemetry` 对象**（turns.jsonl 单写点的
  可选字段，serde default + 省略序列化——旧文件零迁移照读）。字段：
  `provider`、`model`、`requested_model`、`in_tokens`、`out_tokens`、
  `elapsed_ms`、`stop_reason`、`stream`、`error`、`correlation`；一次
  assistant 轮内多请求（工具循环）折叠为轮级合计 + `requests[]` 逐请求行
  （每行含 `ts_ms`），保持"一轮一 telemetry"的读取简单性。
- **载体与折叠挂点**：
  - 聊天：收束 assistant 轮（`chat_run_owner` 三收束臂 drain→fold，投影到
    最后一个 Message 轮）。
  - relay：**相位边界轮**（`RunEvent::StepCompleted` 携带相位聚合遥测，
    `submit_handoff` 双轨委托单点折叠）。流式 Message 轮是 delta 碎片，
    与请求边界在 append-only log 上不可对齐——不做逐碎片挂载。
- **关联（correlation）**：构造期显式注入，跨 spawn 结构无损——chat=
  `{kind:"chat", workspace_id, session_id}`（chat_run_owner 唯一孵化入口，
  每 run 实例私有 sink）；relay=`{kind:"relay", run_id, step_id, role}`
  （hw `MuskAgentFactory` 为双轨唯一 step-agent 构造点）。并发零错配是
  结构性质；未包装路径（`/api/run`、workflow、CLI——无 turns.jsonl 消费方）
  不产生记录，强于记 null。
- **装饰顺序**：遥测为最外层（relay plan 流=
  `TelemetryClient(PlanRuntimeClient(AiClient))`）——记录的 requested_model
  是最终发出值、model/usage 是最终响应值。

## 缺失语义（显式空值，不伪造）

- `provider` **恒 null**：daemon wire 不携带 provider 字段（auto-ai
  PLAN-031 `ModelMeta{id, context_window, max_output_tokens}` 无此域）；
  provider 归因由 collect-telemetry 对 aaid 日志的启发式 join 补强
  （去 ANSI；±5s+model+stream 时间窗；唯一候选=high）——旁证，标注置信，
  不冒充权威。
- `model` 权威序：`model_meta.id`（PLAN-031 实际服务元数据）→
  `resp.model`（daemon 先解析 tier 再调 provider，该字段即回退后实际服务
  模型——PLAN-098 实况 curl 实证 `tier:mid`→`glm-5.3-flash` 非回显）→
  null（双缺才空）。
- `usage` 缺失记 null；aaid 流式请求 `in_tokens` 恒 0（daemon 流式 usage
  未回填，plan-flow-uat spec.md K8 观察项）**如实记 0**，摘要自动标注。
- 遥测是旁路：sink 故障只 `tracing::warn`（锁 poison 恢复），**绝不影响
  轮次结果**，不得成为新的失败源。

## SSE 暴露（SD-02，详见 chat-streaming.md）

`turn_end`/`done` 事件在模型事实已知时附 `model_meta`
（provider/model/usage.{in,out}），未知省略字段；DTO 枚举与帧协议形状其余
不变（DTO 序列化后 Value 级增量追加）；前端零消费要求。

## 摘要脚本（collect-telemetry.mjs）

`docs/reports/plan-flow-uat/` 内零依赖 Node ≥18 脚本：按角色/相位聚合、
逐请求 LLM 表、失败签名自动标记（K1 out=4096 截断 / K3 同参工具 ≥4 循环 /
K4 门同秒自动放行 / K5 plan 相位无 create_plan）、aaid 旁证 join。退出码
0=产出 / 2=输入缺失（列出）/ 3=--expect 断言不符。随仓脱敏 fixture 与
`--expect` 断言集提供确定性检查（fixtures/README.md 记录来源与脱敏口径）。
RUNBOOK §4 第 0 步：每 run 先产出摘要并在落账中引用其路径。

## 已知限制

- relay 相位遥测是相位级聚合（单请求轮为扁平对象）；逐请求事实在
  `requests[]`，不逐 delta 碎片挂载。
- relay sink 注册表上限 128（fail 路径不留 run 条目，超限驱逐最旧）——
  旁路语义下极端并发的记录缺失可接受，不报错。
- `provider` 在 daemon 侧补关联键/上 wire 前，musk 侧只能经 aaid 日志
  启发式回填（KNOWN 观察项）。
