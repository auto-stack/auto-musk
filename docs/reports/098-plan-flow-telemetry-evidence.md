# PLAN-098 证据报告 — 计划流程运行遥测与模型归因

- plan_revision: 1
- musk worktree: `D:/autostack/.wt/musk-098/auto-musk`（分支 `plan-098-dev`）
- musk 基线: main@da748ac
- 依赖（只读）: auto-ai@5a50a55（PLAN-092 T-04 后）、auto-lang@d5215b477
- 运行模式: 本文件随 plan-098-dev 提交；rebase 后按技能重绑并记 range-diff 映射

## T-01 扩展点勘察与冻结（基线段）

### 前置门核对：PLAN-096 已合回 main ✅

- `docs/plans/archived/096-plan-flow-contract-and-bounded-delivery.md` 存在（归档即已 merge）。
- main 上存在 `backend/crates/musk/src/relay/plan_runtime_client.rs`（096 T-06 产物）；
  `MuskAgentFactory::build_agent` 内已有 relay factory 包装注入（driver.rs:113-139，
  plan 流 + plan_execution 的 run 包 `PlanRuntimeClient`）。
- musk-096 worktree 已清理（`git worktree list` 仅 main 与 musk-097）。
- 结论：T-02～T-04 开工门满足。

### 勘察事实（main@da748ac 代码锚点）

| # | 锚点 | 事实 |
|---|---|---|
| F1 | `src/lib.rs:49-52` | musk 为双轨仓：`src/*.rs` 手写轨 + `auto_generated/`（a2r 转译）。`chat_run_owner` 等胶合函数**无 .at 真源**（`grep -rln chat_run_owner auto-src/*.at` 空），属手维护胶合层；PLAN-096 提交（4393199 等）直接手修 extern_impl.rs 是既定实践 |
| F2 | `auto_generated/extern_impl.rs:1983`（chat_run_owner） | 聊天运行**唯一孵化入口**（PLAN-071 T-02：chats_message run=true 抢守卫后 spawn）；2210 行取 `s.0.client.clone()` → 2240 行 `build_agent_with_context`；成功/失败/空闲超时三臂均经 `assemble_chat_run_msg` + `persist_chat_run_msg`（upsert 原位替换）+ `chat_message_to_turns` 双写 turns |
| F3 | `src/relay/driver.rs:81-161` | `MuskAgentFactory::build_agent`：plan 流且 plan_execution 时包 `PlanRuntimeClient`（budget 层：补 max_tokens + 截断登记），否则裸 `state.client`；`role_id` 与 run 上下文在 factory 构造期可得 |
| F4 | `src/relay/driver.rs:331-498`（run_step） | 步内驱动 `agent.run_stream`；on_event 把 Delta/Tool 映射为 `RunEvent::TurnDelta`/`TurnToolCall`（**每个 delta 一条 Message turn**——run-094-humanarm-8975 实测 1013 条 message vs 14 组 tool_turn）；收束经 `submit_handoff_for` → `store.submit_handoff`（push `StepCompleted` + `TokenSpend` 并 mirror 成 turns） |
| F5 | `src/relay/store.rs:662-700, 799-850` | `submit_handoff` 构造 `RunEvent::StepCompleted{timestamp, step_id, handoff_summary}`；`push_event`/`mirror_events` → `run_event_to_turns` → `conv.append_turn`（turns.jsonl 单写点）。RunEntry.events 持久化且 serde 回读（加变体有旧版互读风险；**给既有变体加 `#[serde(default)]` 可选字段双向兼容**） |
| F6 | `src/conversation.rs:86-109, 358-…` | `Turn` 结构（可选字段模式 `#[serde(default, skip_serializing_if)]`）；`chat_message_to_turns`（聊天投影）与 `run_event_to_turns`（relay 投影）两个纯函数投影 |
| F7 | `src/chats.rs:95-132, 633-660` | `ChatMessage` 全可选增量字段模式（design_context 先例）；`upsert_message` 原位替换/追加 |
| F8 | auto-ai `ai-config/rust-ref/src/wire.rs:277-332` | `ModelMeta{id, context_window, max_output_tokens?}`——**wire 上无 provider 字段**；`CompletionResponse{content, tool_calls, stop_reason?, usage?, model, error?, model_meta?}` |
| F9 | auto-ai `auto-ai-daemon/src/server.rs:412-443`（done 尾帧） | 流式 done 帧：`{type:"done", model, model_meta, usage{input,output,cache_read,cache_write}, tool_calls, stop_reason}`——勘察 daemon 源码确认实际形态（等价于"抓一次真实响应"的 wire 契约证据；auto-ai@5a50a55 只读）。**provider 不上 wire** → musk 侧记显式 null，provider 归因由 aaid 日志旁证 join（脚本标置信） |
| F10 | auto-ai `auto-ai-client/rust-ref/src/lib.rs:154-231` | musk 侧流式路径已解析 done 帧的 model/model_meta/usage 入 `CompletionResponse`（PLAN-031 贯通属实） |
| F11 | auto-ai `auto-ai-agent/rust-ref/src/agent.rs:61-80` | musk 编译用的 `Client` trait：`complete(&self, &CompletionRequest)` / `complete_stream(&self, &CompletionRequest, on_event)`（按引用；`rust/` 生成轨按值，**非 musk 编译面**）。`StreamEvent::TurnEnd{turn, usage:Option<Usage>, tool_count}` 已携带每 ReAct 轮 usage |
| F12 | `src/auto_generated/server_stream.rs:54-66` | `SseEventDto` 严格枚举（ag 聊天 SSE 帧形状）；加字段的低风险路径 = 序列化后对 Value 追加可选键，不改枚举 |
| F13 | `src/server.rs:477-582`（run_stream_handler） | hw `/api/run` 无状态流：直接用 `state.client`，无会话落盘消费方 |
| F14 | driver spawn 面 | relay 驱动孵化点多（PLAN-096 a1f7f7d 列 8 个：hw advance/gate-resume、ag REST advance/gate-resume、chat owner、merge 短路、SpawnRelay、task_plan engine），但**全部收束到 hw `drive_run`/`run_step`**（ag 轨受控分流后委托） |

### 冻结决策（三决策落纸）

**D1 注入点 —— 每 run 包装（与 §2"serve() 全聊天流量"的偏差，已列偏差记录）**

- 聊天：`chat_run_owner` 内包装（F2 唯一孵化入口；ctx={kind:"chat", workspace_id, session_id}）。
- relay：`MuskAgentFactory::build_agent` 内包装（F3；plan 流时包在 budget 层**外层**）。
- `main.rs`/`serve()` 不动；`/api/run`、workflow、CLI 路径不包装（无 turns.jsonl 消费方，遥测不落盘即无意义——不产出孤儿记录）。
- 偏差理由：§2 假设 serve() 层可携带关联上下文，但共享 client 无法承载 per-session 上下文；每 run 实例化包装使"并发零错配"成为结构保证而非运行时约定。计划 §2 明文授权"具体机制 T-01 勘察后冻结"。

**D2 关联载体 —— 显式注入（否决 task-local）**

- `TelemetryClient` 构造期携带 `TelemetryContext`，持有实例私有 sink（`Arc<Mutex<Vec<TelemetryRecord>>>`）；fold 点 drain 实例 sink。
- 否决 task-local 的证据：relay 8 个孵化点（F14）+ chat 的 `tokio::select!` 看门狗任务结构（F2）——task-local 需要在每个 spawn 边界重设 scope，遗漏即静默 null；显式注入跨 spawn 结构无损。
- 计划约束"上下文缺失时记 null 不错配"的落实：包装路径 correlation 恒为构造期真值（无缺失面）；未包装路径**不产生记录**（强于记 null——零错配、零孤儿行）。

**D3 装饰顺序 —— 遥测最外层**

- relay plan 流：`TelemetryClient( PlanRuntimeClient( AiClient ) )`。
- 依据：budget 层只改 `max_tokens`（不改 model 字段），遥测在任一层记录的 requested_model 相同；取最外层使"记录实际发出的请求与实际收到的响应"成为结构性质（响应经 budget 层原样透传）。
- 聊天：`TelemetryClient( AiClient )` 单层。

**D4（补充）折叠挂点**

- 聊天：chat_run_owner 成功/失败/超时三臂收束处 drain → `ChatMessage.telemetry`（新可选字段）→ `chat_message_to_turns` 投影到 assistant `Turn.telemetry`。多请求折叠：轮级 totals + `requests[]` 逐请求行（§5.3）。
- relay：`RunEvent::StepCompleted` 增 `#[serde(default)] telemetry: Option<Value>`（F5 双向 wire 兼容），`run_step` 收束后 drain 注入，`run_event_to_turns` 投影到 StepCompleted 系统轮。
- 与 §5.3"写入 assistant turn"的偏差记录：relay Message turn 是 **delta 碎片**（F4 实测 1013/14），流式碎片与请求边界在 append-only log 上无法对齐（记录在响应收束时才定形，晚于碎片落盘）；相位级聚合（requests[] 全行含 correlation）落在相位边界轮上，满足 AC-02 可观察面与脚本"按 from（relay 再按 step）聚合"。

**D5（补充）SSE 暴露形状**

- ag 聊天路径（F2 on_event）：`serde_json::to_value(&dto)` 后按 sink 末记录追加 `model_meta:{provider?, model, usage{in,out}}` 键（已知才加，未知省略）；DTO 枚举不动（F12）。
- hw `/api/run` 无遥测包装 → 永不附加（字段省略，wire 兼容）。
- relay：StepCompleted 事件的 telemetry 经既有 `publish` 自动上 `/runs/{id}/events` SSE（零增量工作）。

### 与 §2 假设不符处清单（触发修订判定用）

| § | 假设 | 实际 | 判定 |
|---|---|---|---|
| §2/§5.1 | 聊天路径在 serve() 包装"全部聊天流量" | 每 run 包装（D1）；/api/run 等无落盘消费方路径不包 | **偏差已授权**（§2 明文"T-01 冻结"）；语义收敛为"全部会话聊天流量" |
| §5.1 | provider/model 记真值 | provider 不上 wire（F9），恒 null；model 真值可得 | 非偏差：§5.1 已预置"缺失记 null"；provider 归由脚本 aaid 旁证 join（计划 §2 本就这么设计） |
| §5.2 | 载体候选 task-local 或显式传递 | 显式注入（D2） | 计划列出的候选之一，非偏差 |
| §5.3 | relay 遥测写入 assistant turn | 相位边界轮承载相位聚合（D4） | **偏差**：流式碎片不可对齐（F4 实测）；AC-02 可观察面不变，V05 断言按 D4 形状书写 |

## T-02 遥测装饰器与关联上下文（worktree e2f906c）

- 新模块 `backend/crates/musk/src/telemetry.rs`：`TelemetryClient`（装饰器，
  双路捕获 complete/complete_stream）、`TelemetryContext`（Chat/Relay 显式
  关联）、`TelemetrySink`（实例私有 Arc<Mutex<Vec>>；poison 恢复）、
  `fold_records`（§5.3 折叠）、run 键控 sink 注册表（relay factory 与
  store.submit_handoff 折叠点解耦共享；上限 128 驱逐防泄漏——旁路语义）。
- 注入：relay = `MuskAgentFactory::build_agent` 遥测最外层
  （`TelemetryClient(PlanRuntimeClient(AiClient))`，D3；hw factory 为双轨
  唯一 step-agent 构造点——ag drive_submit_handoff/`factory_build_agent`
  均委托 hw，实证见 T-03 V05）；chat = `chat_run_owner`（唯一孵化入口）
  会话级包装；`main.rs`/`serve()` 不动（T-01 D1 偏差）。
- V01：8 新单测（双路/回退真值/显式空值/带内+传输错误透传/poison 恢复/
  并发双会话零错配/折叠形状/relay correlation）+ 全 lib 542 绿。
- 依赖组兄弟 worktree（只读，detach）：auto-ai@5a50a55、auto-lang@d5215b477
  （相对路径 deps 解析需组内兄弟；无 junction；098 不改依赖，收尾即删）。

## T-03 落盘与事件接线（worktree 2778e33）

- `Turn.telemetry` / `ChatMessage.telemetry` 可选字段（serde default +
  skip_serializing_if）：旧行照读零迁移；**.at 真源
  （auto-src/conversation.at）与生成镜像（auto_generated/conversation.rs）
  锁步手修**（KNOWN-DEBT 086 定式；镜像profession_id 已先例漂移）。
- chat 折叠：`chat_run_owner` 三收束臂（done/failed/idle_timeout）drain→
  fold→挂收束 Message 轮（`chat_message_to_turns` 投影到最后一个 Message
  轮）；turn 快照臂不带遥测（终版同 id 原位替换）。
- relay 折叠单点 = `RunStore::submit_handoff`（hw/ag 双轨委托单点）：
  drain(run_id, step_id 过滤)→fold→`StepCompleted` 新增 serde default
  telemetry 字段（双向 wire 兼容）→`run_event_to_turns` 投影到相位边界轮
  （T-01 D4）。
- SSE：ag chat on_event 在 DTO 序列化后 Value 级追加 `model_meta`
  （provider/model/usage；turn_end/done 已知才加）；DTO 枚举与帧形状其余
  不变；hw `/api/run` 无遥测包装 → 永不附加（D5）。
- 测试：V01 545 绿（+3 投影/兼容）；V05
  `plan098_relay_telemetry_correlates_and_folds` 绿——hw+ag 双轨、并发双
  run 零错配、correlation(run/step/role) 逐相位断言、model/requested/out
  透传、provider 显式 null、折叠即 drain；parity_conversation/chats/
  relay_store、plan_flow_contract、plan_delivery_contract 全绿。

## T-04 collect-telemetry.mjs 与 fixture（worktree f9ae074）

- 脚本契约见文件头；要点：退出码 0（产出）/2（输入缺失列出）/3（--expect
  不符）；签名 K1-K5；aaid 去 ANSI 解析四种行（`chat req`/`stream start`/
  `stream done`/`chat ok`——provider 仅在 start/ok 行）；join=±5s+model+
  stream 旗标，唯一候选=high；`--since/--until` 同时作用于轮与日志行。
- fixture：签名阳性 relay run（K1-K5 全命中路径）+ chat 旧格式兼容样本 +
  带 ANSI aaid 片段 + expect 断言集；时间基线/行形状取自 run-094 真实产物
  与 daemon 源码实测格式。
- V03 矩阵：relay+expect=24 断言 OK（4 签名全命中、provider=zhipu join、
  high 置信、K1 行 out=4096/stop=max_tokens）；chat+expect=13 断言 OK；
  无 aaid 变体 exit 0 且 provider=null+skipped 注记；`--session no-such`
  → exit 2 列缺失项。

## T-05 实况 smoke（V04）与 D6 语义修订

- 环境：worktree release musk（:17298 专用端口，生产 :17201 未动）+
  在跑 aaid（:17654，auto-ai@5a50a55 构建的进程，zhipu 池）。demo-1 会话
  `db7b23cedffcf99423379abc`：auto 审批，两轮（第一轮含 list_dir 工具调用，
  第二轮追问）。
- **AC-01 实证**（turns.jsonl 原文）：
  - 轮 1（seq3）：`model=glm-5.3-flash`、`requested=tier:mid`、`in=0`、
    `out=184`（=50+64）、`elapsed=12165`、`stop=end_turn`、`stream=true`、
    `requests[]=2`（tool_use→end_turn 折叠）、correlation{chat,demo-1,sid}。
  - 轮 2（seq5）：单请求行，model 同、out=59。
  - 端到端：collect-telemetry 对该会话 exit 0，6 轮解析、3 请求行、
    0 无遥测轮、签名全 0（健康 run）。
- **K8 观察项复现并登记**（spec.md 已知问题表）：三轮流式请求 `in=0`
  全数复现（非流式 probe `in=123` 正常）——musk 侧如实记 0，摘要标注。
- **D6 修订（live 证据驱动的 §5.1 语义修正，rev 不变）**：daemon 在 config
  未声明 context_window 时 model_meta 恒缺（/v1/models metadata 全空 +
  非流式响应无 model_meta 字段，curl 直探实证）；但 daemon 对 `tier:mid`
  请求回 `model="glm-5.3-flash"`——**resp.model 即解析后的实际服务模型**
  （tier 回退已反映，非请求回显）。故 model 权威序改为：model_meta.id →
  resp.model（非空）→ null；provider 恒 null 不变。原规则（model_meta 缺
  失即 null）会使 AC-01 在部署态 daemon 上不可满足且丢弃可得真值；修订记
  录于此，交 review 核定。

## 命令记录

（各任务完成后追加）

| 任务 | 命令 | 预期 | 实际 | 版本锚点 |
|---|---|---|---|---|
| T-01 | `git -C D:/autostack/auto-musk worktree list --porcelain` | main + musk-097 | 一致；新建 musk-098@plan-098-dev（base=da748ac） | musk main@da748ac |
| T-01 | `ls docs/plans/archived/096-*.md` + `ls backend/crates/musk/src/relay/plan_runtime_client.rs` | 存在 | 存在（096 合并门过） | musk main@da748ac |
| T-01 | `grep -rln chat_run_owner backend/crates/musk/auto-src/*.at` | 空（无 .at 真源） | 空 | musk main@da748ac |
| T-02 | `cargo test -p musk --lib`（worktree/backend） | 全绿 | 542 passed 0 failed（含 8 新遥测单测） | plan-098-dev@e2f906c 前工作树 |
| T-03 | `cargo test -p musk --lib` + `--test parity_relay_driver` | 全绿 | 545 passed；parity 7 passed（含 V05 新测试） | plan-098-dev@2778e33 前工作树 |
| T-03 | `--test parity_conversation --test parity_chats --test parity_relay_store --test plan_flow_contract --test plan_delivery_contract` | 全绿 | 20/10/7/7/9 passed | 同上 |
| T-04 | `node collect-telemetry.mjs --run run-fixture-098 --autoos fixtures --aaid-log fixtures/aaid-fixture.log --out tmp/098/v03/relay --expect fixtures/expect-relay.json` | exit 0 断言全过 | 24 assertions OK，exit 0 | plan-098-dev@f9ae074 |
| T-04 | 同上（chat-fixture-098/expect-chat.json） | exit 0 | 13 assertions OK，exit 0 | 同上 |
| T-04 | `--session no-such-session …` | exit 2 列缺失 | exit 2 | 同上 |
| T-04 | chat fixture 无 `--aaid-log` | exit 0，provider null | exit 0，provider null+skipped | 同上 |
| T-05 | `cargo build -p musk --release`（worktree） | 构建成功 | Finished release 3m50s（D6 后重建 33.7s） | plan-098-dev 工作树 |
| T-05 | musk serve :17298 + demo-1 两轮 chat | 两轮收束 | 会话 db7b23cedffcf99423379abc，两轮 assistant 轮带遥测真值 | aaid@17654（5a50a55 构建） |
| T-05 | `collect-telemetry.mjs --session db7b23… --autoos D:/autostack/auto-musk/tmp/demo/.autoos --out tmp/098/v04/live` | exit 0 摘要 | 6 轮/3 请求行/0 缺遥测/签名 0，exit 0 | 同上 |
| T-05 | `curl POST :17654/v1/chat/completions`（tier:mid 与显式 model） | 观察 resp.model/model_meta | resp.model=解析后真值；model_meta 字段缺席（config 无 context_window）→ D6 修订 | aaid@17654 |
