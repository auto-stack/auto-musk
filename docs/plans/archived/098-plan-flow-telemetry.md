---
plan_id: PLAN-098
status: archived
feature_name: 计划流程运行遥测与模型归因（plan-flow-uat 重跑可对比性）
author: [agent]
created_at: 2026-10-02T04:21:14Z
updated_at: 2026-10-03T12:00:00Z
plan_revision: 1
current_step: 7
total_steps: 7
supersedes_spec_components:
  - docs/specs/modules/chat-streaming.md
  - docs/specs/modules/plan-flow.md
new_spec_components:
  - docs/specs/modules/run-telemetry.md
touched_goals:
  - goal-agent
  - goal-relay
  - goal-spec-knowledge
---

# PLAN-098 — 计划流程运行遥测与模型归因（plan-flow-uat 重跑可对比性）

## 0. 变更摘要

plan-flow-uat（`docs/reports/plan-flow-uat/`，RUNBOOK 已注册）目前能逐步骤判定
"跑好/跑坏"，但**判定不了"每一步是哪个角色、实际由哪个 provider/model 服务、
花了多少 token/时间"**：daemon（aaid）每请求日志只在 stdout 且无 run/session
关联键；musk 侧全部落盘物（会话与 relay run 的 turns.jsonl）零模型信息——
尽管数据链已通（aaid 在 SSE done 尾帧携带实际服务模型元数据，PLAN-031；
musk 的 `CompletionResponse` 已有 `model_meta` 字段）。

本计划补三件事，使 plan-flow-uat 每轮产出可直接对比的模型归因数据：

1. musk 客户端遥测记录器（decorator Client）：拦截每轮 LLM 完成，记录
   时间、耗时、请求模型、实际服务 provider/model、usage、stop_reason 及
   所属上下文（会话轮次 / relay run+相位+角色）。
2. 归因落盘与事件暴露：聊天与 relay 的 turn 记录追加可选 telemetry 字段
   （serde default 兼容旧文件）；SSE `turn_end`/`done` 事件在已知时填充
   model 元数据。
3. `collect-telemetry.mjs` 摘要脚本并入 RUNBOOK：按角色/相位聚合轮数、
   工具数、时长，time-join aaid 日志行，自动标记已知失败签名
   （out=4096 截断、门同秒自动放行、循环检测击杀、plan 相位无 create_plan），
   产出每轮 telemetry 摘要供 model 选择优化对比。

扩展点优先复用 PLAN-096 已建机制（relay factory 的显式 Client 包装注入、
`plan_runtime_client.rs` 的按 run 键控记录器模式）；**本计划以 PLAN-096 合回
main 为前置**。auto-ai 只读，不改 daemon。

## 1. 目标

### 交付目标

plan-flow-uat 重跑后，对任一已完成会话或 relay run，能从磁盘工件回答：

- 每一轮 assistant 输出由哪个 provider/model 实际服务（tier 回退后仍是
  真实值）；每轮 in/out token、耗时、stop_reason。
- 按"相位/技能阶段 × 角色"聚合的轮数、工具调用数、时间跨度。
- 已知失败签名是否出现（K1 截断、K4 门自动放行、K3 循环击杀、K5 无计划）。

数据落在受版本控制的记录物（turns.jsonl 字段、telemetry 摘要 JSON/MD），
不依赖 daemon 日志存活或时间戳手工对齐。

### 范围

- 主仓库：auto-musk。`main.rs` 客户端注入点、chats 运行循环与 relay
  driver 的上下文关联、`conversation.rs` turn 持久化、SSE 事件构造、
  `docs/reports/plan-flow-uat/` 的脚本与 RUNBOOK、测试与证据。
- 依赖：auto-ai / auto-lang / auto-down 只读。不改 aaid 的日志格式、
  max_tokens 预设或 UsageTracker 聚合维度。
- PLAN-096 关系：复用其 relay factory 包装注入与按 run 键控记录器模式；
  其 `plan-execution-contract.md`（merge 阶段新建）落地后，本计划 SD-03
  在 plan-flow.md 的规则引用由 review 核定是否改挂该新模块。

### 非目标

- 不改 auto-ai：不做 daemon 日志关联键、不做按 run 的 UsageTracker 聚合、
  不做 relay/builtin 档位 max_tokens 预设（K1 根修候选仍在 KNOWN-DEBT）。
- 不做成本换算（价格表）、不做跨 run 的模型质量评分、不做模型自动选择。
- 不做前端 UI 展示模型归因；SSE 字段为消费方预留。
- 不修复 K2（不自动续跑）、不关闭工具循环防护、不改审批语义。
- 不重跑 plan-flow-uat 全套四阶段作为本计划验收；端到端实证用轻量
  实况 smoke（T-06），完整重跑是 merge 后的使用场景。

### 成功边界

AC-01～AC-07 全部有可复验结果。遥测缺失（如 aaid 无响应元数据）必须
记录为显式空值，不得伪造或静默缺行。

## 2. 架构方案

### 捕获点：装饰器 Client（单仓、零 daemon 改动）

musk 在 `main.rs` 以 `AiClient::new()` 构造真实客户端并作为
`Arc<dyn Client>` 注入 server state；relay factory（PLAN-096 T-06 起）
在计划执行 run 中显式注入包装链。本计划新增一个可组合的遥测装饰器：

```text
真实链:  Agent ──> AiClient(HTTP→aaid)
本计划:  Agent ──> TelemetryClient ──> AiClient
                   │ complete()/stream() 完成时记录:
                   │ ts、elapsed、requested_model、model_meta(provider/model)、
                   │ usage(in/out)、stop_reason、stream、correlation
                   └ 缺元数据记 None，绝不使轮次失败
```

- 聊天路径：`serve()` 注入点包一层遥测（全部聊天流量）。
- relay 路径：factory 包装链追加遥测层（与 096 的 budget/truncation 层
  组合，顺序由 T-01 冻结——budget 层改请求、遥测层只读响应，遥测在外层
  或内层均须记录到**实际发出**的请求与**实际收到**的响应）。
- correlation：上下文由 musk 侧运行循环提供——chats run 循环设置
  （workspace、session、turn seq），relay `drive_run`/`run_step` 设置
  （run_id、step_id、role/profession）。具体机制（tokio task-local 或
  显式注入）在 T-01 勘察后冻结，约束：跨 `tokio::spawn` 边界不丢、
  上下文缺失时记录 `correlation: null` 而非错配。

### 落盘点：conversation.rs 单写点

聊天会话与 relay run 共用 `conversation.rs` 的 `turns.jsonl` 写点
（run 目录即 `kind: flow` 的会话，`turns[].from` 已承载角色）。turn 记录
追加可选 `telemetry` 对象（serde default，旧文件字节兼容照读）：

```json
{"ts":..., "from":"coder", "kind":"assistant", ...,
 "telemetry": {"provider":"zhipu","model":"glm-5.3","requested_model":"tier:mid",
               "in_tokens":0,"out_tokens":699,"elapsed_ms":19615,
               "stop_reason":"end_turn","stream":true,"correlation":{...}}}
```

不新建平行文件，不改 meta.json 结构；单写点保证聊天/relay 一致。

### 暴露点：SSE 事件

`server.rs` 既有事件构造处（现测试 mock 显示 `CompletionResponse.model_meta`
已贯通类型链）在 `turn_end`/`done` 事件已知时填充模型元数据；前端不消费
不影响现状（chat-streaming.md 事件契约增量见 SD-02）。

### 摘要脚本：collect-telemetry.mjs（独立、可重复）

输入：`--session <sid>` 或 `--run <id>`、`--autoos <dir>`、可选
`--aaid-log <path> --since --until`、`--out <prefix>`。输出 JSON + Markdown：

- turns.jsonl 按 `from`（relay 再按 step）聚合：轮数、工具调用数、
  首末时间戳跨度、每轮 provider/model/token/耗时表。
- aaid 日志行解析（去 ANSI；`chat req:` / `chat stream start:` /
  `chat stream done:` / `chat ok:` / tier fallback），按时间窗与
  turn 时段做启发式对齐（daemon 无关联键，aaid 侧仅作旁证，标注置信）。
- 失败签名自动标记：收束轮 `out=4096`；`Gate .* approve` 与
  `gate_waiting` 同秒；同参工具调用 ≥4（循环击杀）；plan 相位无
  `create_plan` 调用。
- 输入缺失（会话/run 不存在、turns.jsonl 无 assistant 轮）非零退出，
  不产出空报告冒充成功。

daemon 侧 `in=0`（流式 usage 未回填）按观察项处理：脚本原样记录 0 并在
摘要标注"daemon 流式 usage 未回填"，不在本计划修 aaid。

## 3. 技术栈

| 层 | 沿用方案 | 本计划约束 |
|---|---|---|
| 客户端装饰 | `auto_ai_agent::Client` trait + async_trait；096 factory 注入模式 | 只增装饰层；不改 trait 与 auto-ai-client 类型 |
| 关联上下文 | tokio 运行循环内设置；具体载体 T-01 冻结 | 跨 spawn 不丢；缺失记 null 不错配 |
| 持久化 | conversation.rs turns.jsonl（serde_json 行） | 可选字段 + serde default；旧文件零迁移 |
| 事件 | server.rs 既有 SSE 构造 | 只填充已知值；不改帧协议形状 |
| 脚本 | Node ≥18 内置模块（与 plan-driver.mjs 同约束） | 零第三方依赖；真实工件 fixture 随仓持久 |
| 测试 | cargo test -p musk（串行跑法）；node --test 不引入 | mock Client 双路（stream/非 stream）断言 |

## 4. 需求分析与背景调查

### 已有授权与流程边界

- 用户授权（2026-10-02 会话）：基于已完成的观测面分析建立本计划，补齐
  plan-flow-uat 重跑验证所需的步骤/角色/模型归因工具。授权范围=计划撰写；
  未授权执行、预算或自动续跑。work 阶段需后续指令。
- **前置依赖：PLAN-096（executing，plan-096-dev@b3ab588，T-01～T-11 已勾）**
  尚未合回 main。本计划 T-01 的第一项是核对 096 合并状态；未合回前
  T-02～T-04 不得在 main 基座上开工（relay factory 包装注入挂点属 096）。
  若 096 的最终形态与本计划 §2 假设不符（如包装链接口变化），回到
  needs_replan 给最小修订，不绕开 096 另建平行注入。
- 计划编号分配：扫描 `docs/plans/`（活跃 096、097）与 `archived/`（最大
  095），取 098；创建时 musk serve 未运行（:17201 无监听），无并发写者；
  创建后复验唯一。
- 四阶段分工：本合同（new）→ work 在专用 worktree
  `D:/autostack/.wt/musk-098/auto-musk`（分支 `plan-098-dev`）实施 →
  review 独立复验 → merge 按 AGENTS.md rebase/ff-only/清理。禁止
  junction/symlink。

### 调查版本与关键事实（main@6a22693，2026-10-02）

| 锚点 | 事实 | 对本计划的意义 |
|---|---|---|
| backend/crates/musk/src/main.rs:118-243 | `AiClient::new()` 构造真实客户端，`Arc<dyn Client>` 注入 serve；`NoDaemonClient` 兜底 | 聊天路径包装注入点 |
| .wt/musk-096/.../relay/plan_runtime_client.rs | 096 T-06 装饰器模式：按 run_id 键控 record/records_for/clear，仅 relay factory 注入 | 复用模式；relay 注入挂点（属 096） |
| backend/crates/musk/src/conversation.rs:609-940 | 聊天与 relay run 共用 `turns.jsonl`+`meta.json` 单写点；`turns[].from` 承载角色 | 落盘单写点，零平行文件 |
| aaid 日志实测（tmp/aaid-0929.log） | 行式样：`chat req: app=auto-ai-client model=tier:mid stream=true`；`chat stream start: provider=zhipu model=glm-5.3-flash (waited 0ms)`；`chat stream done: model=... elapsed=19615ms in=0 out=699`；app 名恒为 `auto-ai-client`（不区分流量来源） | 脚本解析对象；app 名无关联力→归因必须 musk 侧做；流式 `in=0`（usage 未回填）按观察项 |
| auto-ai PLAN-031（daemon server.rs 注释） | done 尾帧携带实际服务模型元数据；musk `CompletionResponse.model_meta` 类型已贯通（现仅测试 mock 置 None） | 无需改协议；捕获即可得真值 |
| docs/specs/modules/plan-flow.md | 四阶段静态契约 + 094 门不变式；无任何模型归因规则 | SD-03 增量空间 |
| docs/specs/modules/chat-streaming.md | 流式/轮次事件契约（delta/tool_result/turn_start/turn_end/done 臂）；无 model 元数据规则 | SD-02 增量空间 |
| docs/reports/plan-flow-uat/spec.md + RUNBOOK.md | T1-T4 用例、K1-K7 对策、复现步骤已入库（97fc677/1993eaa/4a0b44e） | 脚本与落账的集成对象 |
| tmp/demo/.autoos/conversations/run-094-humanarm-8975/ | 真实 relay run 产物（meta.json+turns.jsonl，206KB 全轮记录） | V03 真实夹具来源（转脱敏 fixture 随仓持久） |

### 与既有计划的边界

- PLAN-096 已覆盖：计划执行的有界交付、截断登记与一次续做（musk 侧
  max_tokens 兜底）、阶段结构化检查点。本计划只做**观测记录与汇总**，
  不新增阶段控制逻辑。
- PLAN-097（drafting，另一会话）：VM 演示与证据呈现，消费 096 的结构化
  事实做截图展示。本计划的 telemetry 字段可为其所用，但本计划不做任何
  UI/VM 呈现。
- KNOWN-DEBT：K1 的 daemon 侧根修候选（relay 档位预设安全 max_tokens）
  仍登记在案，本计划不改 aaid，不关闭该债。

## 5. 详细设计

### 5.1 遥测记录器（TelemetryClient）

- 实现 `auto_ai_agent::Client`，持有内层 `Arc<dyn Client>` 与记录 sink；
  `complete()` 与流式路径都在**内层返回后**同步记录（流式在流收束处）。
- 单条记录（进程内 struct，落盘前形状）：
  `ts、elapsed_ms、requested_model、provider、model、in_tokens、out_tokens、
  stop_reason、stream、error、correlation`。`model_meta` 缺失→
  provider/model 记 null；usage 缺失→记 null（aaid 流式 in=0 场景如实记）。
- 记录失败（sink 写错）只 `tracing::warn`，不影响轮次结果——遥测是旁路，
  不得成为新的失败源。
- 注入：聊天路径在 `serve()` 包装；relay 路径在 factory 包装链追加。
  装饰顺序与 096 budget 层的组合由 T-01 冻结并写入证据（要求：记录的
  requested_model 是最终发出值、model/usage 是最终响应值）。

### 5.2 关联上下文

- chats 运行循环在每轮驱动前设置 `{workspace, session_id, turn_seq}`；
  relay 在 `run_step` 设置 `{run_id, step_id, role}`；作用域结束清除。
- 载体候选：tokio task-local（需验证跨 `tokio::spawn` 的 driver 结构）或
  经 factory/build_agent 显式传递。T-01 勘察后冻结；验收约束不变：
  不得把 A 会话的轮次记到 B 头上（并发反例入单测）。

### 5.3 落盘（conversation.rs）

- turn 追加时若记录器有本上下文的新记录，折叠为单个 `telemetry` 对象
  写入 assistant turn；无记录则省略字段（不是 null 行）。
- 一次 assistant 轮内多次 LLM 请求（工具循环）记 `telemetry.requests[]`
  汇总（每请求一行），轮级 out/in/elapsed 为合计——保持"一轮一 telemetry"
  的读取简单性。
- 兼容：serde 可选字段 + default；加载 094 时代旧 turns.jsonl 的回归
  测试入 V06。

### 5.4 SSE 事件

- `turn_end`/`done` 事件在已知时附 `model_meta`（provider/model/usage），
  未知省略字段；事件帧其余结构不变。前端（Vue/VM）零改动、零消费要求。

### 5.5 collect-telemetry.mjs

- 位置：`docs/reports/plan-flow-uat/collect-telemetry.mjs`（与
  plan-driver.mjs 同目录、同零依赖约束）。
- 子命令形态：单命令多参（见 §2）。优先消费 turns.jsonl 内嵌 telemetry
  （T-03 落地后为权威）；aaid 日志仅在显式传入时作旁证合并。
- fixture 随仓持久：`docs/reports/plan-flow-uat/fixtures/`——从
  run-094-humanarm-8975 与 2026-09-29 chat 会话产物提取的**脱敏样本**
  （turns.jsonl 片段 + aaid 日志片段），V03 的确定性断言跑在 fixture 上，
  真实工件只在现场跑时消费（tmp 会被工作区重置清除，不留失效引用）。
- 退出码：0=产出摘要；2=输入缺失/不可解析（并列出缺失项）；解析行数
  与聚合数须打印供人工复核。

### 5.6 RUNBOOK 集成

- RUNBOOK §4"跑完后必做"第 0 步改为：运行 collect-telemetry.mjs，摘要
  JSON/MD 存入本轮记录；结果总表回填与历史表行追加时引用该摘要路径。
- spec.md 不改判定标准（观测增强不改变 pass/fail 定义）；若 T-06 实况
  smoke 产生新观察（如流式 usage 恒 0），按其"已知问题"表流程登记。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/modules/run-telemetry.md（新） | 无 → 运行遥测契约：聊天与 relay 的 assistant 轮须尽可能记录实际服务 provider/model/usage/耗时/stop_reason；缺失记显式空值；遥测旁路不得影响轮次结果；旧记录零迁移兼容 | "每步哪个模型服务"成为可对比运行的一等事实 | AC-01, AC-02, AC-04 |
| SD-02 | modify | docs/specs/modules/chat-streaming.md | 轮次/done 事件无模型元数据 → turn_end/done 已知时附 model_meta（provider/model/usage），未知省略；前端零消费要求保持 | 事件流与落盘物同源，消费方（含 PLAN-097 演示）可选使用 | AC-03 |
| SD-03 | modify | docs/specs/modules/plan-flow.md | 运行产物无模型归因要求 → 四阶段 run 的 turn 产物满足 run-telemetry.md 记录契约（引用而非复制规则）；aaid 日志仅为旁证 | 计划流程的"可对比重跑"由规范背书，而非约定俗成 | AC-02, AC-05 |

SD-01 在规范索引注册。若 PLAN-096 的 plan-execution-contract.md 已在
main 承接 plan-flow.md 的相位产物章节，SD-03 的挂载目标由 review 核定
改挂，规则文本不变。

## 6. 测试设计

### 命令与证据约定

| 编号 | 命令／工作目录 | 预期结果 |
|---|---|---|
| V01 | worktree/backend：`cargo test -p musk --lib`（新单测所在目标） | 装饰器双路（stream/非 stream）、correlation 并发反例、折叠聚合、sink 失败不影响轮次——全绿 |
| V02 | worktree/backend：`cargo test -p musk`（**逐目标串行**，§8.27 环境事实） | 全套绿；既有测试零改动或仅夹具补字段 |
| V03 | 根：`node docs/reports/plan-flow-uat/collect-telemetry.mjs --session <fixture> --autoos docs/reports/plan-flow-uat/fixtures --out tmp/098/v03` | 退出 0；聚合行数与 fixture 断言一致；签名标记命中预期；输入缺失反例退出 2 |
| V04 | 实况 smoke（demo-1，musk serve+aaid 运行）：2 轮 chat 会话 → 读 `tmp/demo/.autoos/conversations/<sid>/turns.jsonl` | assistant 轮含 telemetry（provider/model 真值；in 允许 0 但字段存在）；V03 对该会话端到端产出摘要 |
| V05 | worktree/backend：`cargo test -p musk --test parity_relay_driver`（及 plan 契约目标） | relay 注入路径按 run/step/role 关联的断言绿；mock 响应元数据透传 |
| V06 | 回归：V02 全套 + 旧 turns.jsonl 兼容夹具（094 真实样本转 fixture）加载 | 旧记录零告警读取；telemetry 缺省行为一致 |

### 场景矩阵

| 场景组 | 输入/故障 | 证据 |
|---|---|---|
| 捕获 | 非流式/流式、tier 回退（model_meta 与请求模型不同）、usage 缺失、error 响应 | V01/V05 断言 |
| 关联 | 并发两会话交错轮次；relay run 期间普通 chat 并行 | correlation 零错配（V01 反例） |
| 落盘 | 新记录、无记录（未走遥测路径的轮）、旧文件兼容 | V01/V06 |
| 脚本 | fixture 三签名全命中/全不命中；缺 turns；缺 aaid 日志；空 assistant | V03 断言含退出码 |
| 实况 | demo-1 两轮 chat；tool-use 轮多请求折叠 | V04 产物+摘要 |

### 证据落点（均为新）

- `docs/reports/098-plan-flow-telemetry-evidence.md`：AC 结果、命令、
  注入点/载体冻结决策、失败与修复记录。
- `docs/reports/098-plan-flow-telemetry-spec-delta.md`：SD-01～03 对照。
- `docs/reports/plan-flow-uat/fixtures/`：脱敏样本与期望摘要（随仓）。
- 收据含 plan_revision、musk commit、aaid 版本、运行模式；rebase 后按
  技能重绑并记 range-diff 映射。

## 7. 验收标准

| ID | 可观察的验收结果 | 验证方法与预期 |
|---|---|---|
| AC-01 | demo 工作区真实聊天会话的每个 assistant 轮在 turns.jsonl 带 telemetry：实际服务 provider/model（回退后真值）、out_tokens、elapsed_ms、stop_reason；in_tokens 字段存在（值可为 0 并注明 aaid 流式限制） | V04 产物检查；伪造/缺行即失败 |
| AC-02 | relay run 的每相位 assistant 轮带 telemetry 且 correlation 含 run_id/step_id/role；跨并发 run 零错配 | V05 契约断言；现场 relay 记录抽查 |
| AC-03 | turn_end/done 事件已知时附 model_meta，未知省略；前端零改动不回归 | V01 事件构造断言 + V02 前端契约回归 |
| AC-04 | 旧 turns.jsonl（无 telemetry 字段）加载与回放行为不变 | V06 兼容夹具 |
| AC-05 | collect-telemetry.mjs 对 fixture 产出正确聚合与签名标记；输入缺失退出 2；aaid 旁证标注置信且不冒充权威 | V03 断言 |
| AC-06 | RUNBOOK §4 已含脚本步骤与产物路径约定；一轮完整 T1 重跑后无需手工对齐时间戳即可得到按相位/角色×模型的对比表 | 文档评审 + V04 摘要端到端 |
| AC-07 | 非 plan 流量与既有功能零回归：聊天、relay 旧流、tool 循环防护行为不变；遥测自身故障不产生用户可见错误 | V02 全套 + V01 sink 失败注入 |

## 8. 执行步骤

所有任务未开始。每项完成须记录实际命令、结果、代码版本与证据。

### [x] T-01：worktree 建立与扩展点冻结

- 依赖：无（但含 096 状态门）。关联 AC-01～AC-07。
- 核对 PLAN-096 合并状态（main 上存在 relay factory 注入与
  plan_runtime_client.rs 即视为满足）；未合回则停在明确等待，不开工。
- 勘察并冻结：装饰器组合顺序、correlation 载体（task-local 可行性 vs
  factory 显式传递）、conversation.rs turn 序列化形状、SSE 事件构造点、
  aaid done 尾帧字段实际形态（抓一次真实响应）。产出基线段入证据报告。
- 完成门：注入点/载体/顺序三决策落纸；与本计划 §2 假设不符处列出并
  触发修订判定。
- [✅ 已完成] worktree `.wt/musk-098/auto-musk`@plan-098-dev（base=main@da748ac）；
  096 合并门过（archived/096 + plan_runtime_client.rs 在 main）；三决策落纸
  （D1 每 run 包装注入、D2 显式载体否决 task-local、D3 遥测最外层）+ D4/D5
  折叠与 SSE 挂点；偏差清单 2 项（serve()→每 run 包装——§2 授权 T-01 冻结；
  relay 折叠挂点=相位边界轮——delta 碎片不可对齐，实测 1013/14）。证据：
  worktree `docs/reports/098-plan-flow-telemetry-evidence.md` §T-01（commit
  330768c）；aaid done 尾帧经 daemon 源码勘察（auto-ai@5a50a55 server.rs:412-443，
  wire 无 provider 字段→musk 侧记 null+脚本旁证 join）。

### [x] T-02：遥测装饰器与关联上下文

- 依赖：T-01。关联 AC-01、AC-02、AC-07。
- 位置：新 relay/../telemetry（或 client_telemetry.rs，T-01 定）；
  main.rs 注入；factory 链追加；chats run 循环与 run_step 上下文设置。
- V01 单测：双路捕获、并发反例（两会话交错零错配）、多请求折叠、
  sink 故障不影响轮次、model_meta/usage 缺失容忍。
- [✅ 已完成] 新模块 `src/telemetry.rs`（T-01 D1-D3 冻结形态：TelemetryClient/
  TelemetryContext/TelemetrySink/fold_records/run 键控 sink 注册表）；relay
  factory 注入=遥测最外层（budget 层外，plan/非 plan 流全包）；chat 注入=
  chat_run_owner 会话级（main.rs 不动——T-01 偏差 D1）；依赖组只读兄弟
  worktree（detach：auto-ai@5a50a55、auto-lang@d5215b477）。V01=8 新单测绿 +
  全 `cargo test -p musk --lib` 542 绿 0 红。证据：evidence §T-02（待补）、
  worktree commit e2f906c。

### [x] T-03：落盘与事件接线

- 依赖：T-02。关联 AC-01、AC-02、AC-03、AC-04。
- 位置：conversation.rs（turn append 折叠 telemetry）、server.rs 事件
  构造处填充。
- 验证 V01 扩展 + V06 兼容夹具；旧回放零告警。
- [✅ 已完成] Turn/ChatMessage 增可选 `telemetry`（serde default+省略序列化，
  旧行照读零迁移；.at 真源与生成镜像锁步）；chat 三收束臂（done/failed/
  idle_timeout）drain→fold（单请求扁平/多请求合计+requests[]）挂收束
  Message 轮；relay 折叠单点=store.submit_handoff（hw/ag 双轨委托单点，
  StepCompleted 增 serde default telemetry 字段，run 键控 sink 注册表上限
  128 防泄漏）；SSE turn_end/done 在 DTO 序列化后 Value 级追加 model_meta
  （已知才加）。V01 545 绿（+3 新投影/兼容测试）；V05
  plan098_relay_telemetry_correlates_and_folds 绿（hw+ag 双轨并发双 run
  零错配、provider 显式 null、usage 透传、折叠即 drain）。证据：worktree
  commit 2778e33。

### [x] T-04：collect-telemetry.mjs 与随仓 fixture

- 依赖：T-03（读内嵌 telemetry）+ 可独立先行解析部分（turns 聚合）。
  关联 AC-05、AC-06。
- 位置：docs/reports/plan-flow-uat/{collect-telemetry.mjs,fixtures/}；
  fixture 从 run-094-humanarm-8975 与 09-29 chat 产物脱敏提取。
- 验证 V03 全断言（含退出码矩阵）。
- [✅ 已完成] 脚本零依赖（Node18 内置）：turns 按 from 聚合+相位分割
  （StepStarted/StepCompleted 边界）、逐请求 LLM 表（telemetry 行规范化
  单请求扁平/多请求 requests[]）、K1-K5 签名（4096 截断/同参×4/门同秒/
  plan 无 create_plan）、aaid 旁证 join（去 ANSI、±5s+model+stream 窗、
  唯一候选=high；provider 取自 stream start/chat ok 行）、退出码 0/2/3
  （--expect 断言模式）。fixture 随仓脱敏持久（run-094 时间基线+签名阳性
  相位结构、chat 旧格式兼容轮、带 ANSI aaid 片段）。V03：relay 24 断言+
  chat 13 断言绿（含 4 签名全命中+provider=zhipu join+high 置信）；输入
  缺失退出 2；无 aaid=provider null+skipped 注记。证据：worktree f9ae074、
  fixtures/README.md。

### [x] T-05：实况 smoke（V04）

- 依赖：T-02、T-03。关联 AC-01、AC-06。
- demo-1 真实两轮 chat（含一次工具调用）；产物+摘要入证据报告；
  aaid 流式 in=0 观察项如复现，按 spec.md 已知问题表登记。
- [✅ 已完成] worktree release musk（:17298 专用口，生产 ：17201 未动）+
  在跑 aaid（:17654）；demo-1 会话 db7b23cedffcf99423379abc 两轮（第一轮
  list_dir 工具调用）；两 assistant 轮 telemetry 真值：model=glm-5.3-flash、
  requested=tier:mid、多请求折叠 50+64=184、stop/stream/correlation 齐；
  **in=0 观察项复现**→spec.md 已知问题表登记 K8（流式 usage 未回填，
  非流式 in=123 正常）；collect-telemetry 端到端 exit 0（6 轮/3 请求行/
  0 缺遥测）。**D6 修订**（live 证据）：daemon 无 context_window 声明时
  model_meta 恒缺，但 resp.model=解析后实际服务模型（curl 实证 tier:mid→
  glm-5.3-flash 非回显）→model 权威序 model_meta.id→resp.model→null，
  单测同步（11 个）。证据：worktree abe9c77，evidence §T-05。

### [x] T-06：RUNBOOK/spec 文档落地与回归收口

- 依赖：T-04、T-05。关联 AC-06、AC-07。
- RUNBOOK §4 更新；V02 全套串行回归；V06；回归面零新增红。
- [✅ 已完成] RUNBOOK §4 增第 0 步（collect-telemetry 摘要落账：产物路径
  `runs/<date>-<sid>`、结果总表/历史行引用摘要路径、provider null 常态、
  K8 in=0、旁证置信口径）；spec.md 判定标准未动。**V02 全套逐目标串行
  38 目标：36 绿 + 2 存量红**——`parity_handoff_store`（E0063 缺
  authorization，096 T-12 漏改夹具）与 `tool_atoms::
  run_command_dangerous_returns_paused`（'/' 环境敏感）均在
  main@da748ac 复现定谳，**PLAN-098 零新增红**；二者归属交 review。
  V06 兼容夹具（094 真实行）绿。证据：worktree ccfa573，evidence §T-06。

### [x] T-07：证据汇总与规范增量交付

- 依赖：T-01～T-06。关联 AC 全部。
- docs/reports/098-plan-flow-telemetry-{evidence,spec-delta}.md；
  交 /auto-plan:review 独立复验；work 不自行 review/merge。
- [✅ 已完成] evidence（T-01~T-06 段+命令表+AC-01~07 覆盖映射+work 收口
  记录）与 spec-delta（SD-01~03 before/after；SD-03 挂载两案并列）落纸。
  AC 全部有可复验结果；**交 review 四核定项**：① SD-03 挂载目标；② T-01
  D4 relay 折叠形状与 D6 model 权威序修订（live 证据驱动）；③ 两个存量红
  归属；④ relay 实况加强不作为完成门（§10 原留项）。canonical docs/specs
  编辑留 merge 沉淀。

### 任务覆盖与依赖检查

| 任务组 | 验收覆盖 | 规范增量 |
|---|---|---|
| T-01～T-03 | AC-01,02,03,04,07 | SD-01, SD-02 |
| T-04～T-06 | AC-05,06,07 | SD-01, SD-03 |
| T-07 | AC-01～07 | SD-01～03 |

## 9. 复审记录

### new 阶段交接（PLAN-098:r1）

- stage: new ｜ plan_id: PLAN-098 ｜ plan_revision: 1 ｜ outcome: **pass**
  （可进入 work；带前置依赖条件）
- 依据：观测面分析经用户确认（本会话）；扩展点经代码勘察锚定
  （main.rs 注入、conversation.rs 单写点、096 装饰器模式、aaid 日志
  行式实测、model_meta 类型链贯通）；编号 098 经双目录扫描+musk serve
  离线确认；与 096/097/KNOWN-DEBT 边界已划清。
- 前置条件：**PLAN-096 合回 main 后方可开始 T-02～T-04**（T-01 仅做
  状态核对与基线，可提前）。若 096 最终形态偏离 §2 假设 → needs_replan。
- next: work（/auto-plan:work；worktree musk-098/plan-098-dev；先 T-01）。

### work 阶段交接（PLAN-098:r1 → review）

- stage: work ｜ plan_id: PLAN-098 ｜ plan_revision: 1 ｜ outcome: **pass** ｜
  code_commit: plan-098-dev@319cb4f（330768c → e2f906c → 2778e33 →
  f9ae074 → abe9c77 → ccfa573 → 319cb4f，T-01～T-07 顺序提交） ｜
  task_ids: T-01～T-07 全勾（current_step=7/7） ｜
  evidence: worktree `docs/reports/098-plan-flow-telemetry-evidence.md`
  （T-01~T-06 段、命令表、AC-01~07 覆盖映射）+
  `docs/reports/098-plan-flow-telemetry-spec-delta.md` ｜
  blockers: 无 ｜ next: review（/auto-plan:review 独立复验）。
- 验证汇总：V01 lib 545 绿（含 11 遥测单测+3 投影/兼容）；V03 relay 24 +
  chat 13 断言绿（缺失 exit 2）；V04 实况两轮 chat 遥测真值 + 摘要端到端
  exit 0；V05 relay 双轨并发零错配断言绿；V02 全套逐目标串行 36/38 绿
  +2 存量红（main@da748ac 复现定谳，零新增红）；V06 兼容绿。
- 交 review 核定项：① SD-03 挂载目标（两案并列）；② T-01 D4 relay 折叠
  形状（StepCompleted 边界轮聚合）与 D6 model 权威序
  （model_meta.id→resp.model→null，live 证据）两处 §5.1 语义修订；
  ③ 存量红 parity_handoff_store / tool_atoms 的归属去向；④ §10 原留
  "relay 实况加强"不作为完成门（V05 契约断言 + 094 真实产物为准）。
- 环境与清理：worktree `D:/autostack/.wt/musk-098/auto-musk` 留给 review/
  merge（wt-guard clean）；依赖只读兄弟 worktree（auto-ai@main、
  auto-lang@master，detach）随 merge 收尾清理；aaid :17654 复用在跑实例
  （非本计划所有）；:17298 smoke serve 已停。

### review 阶段复验（PLAN-098:r1 → reviewed）

- stage: review ｜ plan_id: PLAN-098 ｜ plan_revision: 1 ｜ outcome: **pass** ｜
  reviewed_commit: plan-098-dev@319cb4f ｜ base_commit: da748ac ｜
  dependency_revisions: auto-ai@5a50a55（main；组内 detach@main）、
  auto-lang@d5215b477（master；组内 detach@master） ｜
  spec_inputs: docs/specs/modules/{chat-streaming,plan-flow}.md @main
  （canonical 未被分支触及——diff 中 docs/specs 匹配数=0）；新模块
  run-telemetry.md 未在 review 期发布（merge 沉淀） ｜
  next: merge。
- **独立性声明**：review 与 work 同会话执行——按技能要求以工件重建结论：
  全套 37 目标串行重跑、V03 矩阵重跑、关键代码 hunk 以 `git show HEAD:`
  复核、V04 实况工件（turns.jsonl + 摘要 JSON）重读，不采信实施期摘要。
- acceptance_results: AC-01 **pass**（V04 工件重读：2 assistant 轮遥测，
  model=glm-5.3-flash×3 行，折叠/in=0 注记齐）；AC-02 **pass**（V05 重跑
  绿：hw+ag 双轨并发零错配、correlation 逐相位）；AC-03 **pass**（SSE
  Value 级追加复核 + 前端契约面 parity 全绿）；AC-04 **pass**（V06 兼容
  测试在 lib 545 内绿）；AC-05 **pass**（V03 重跑：24+13 断言 OK、缺失
  exit 2 直验）；AC-06 **pass**（RUNBOOK §4 第 0 步在 HEAD 复读确认）；
  AC-07 **pass**（复跑 36 绿 + 2 存量红与 main@da748ac 同像定谳=零新增；
  sink 故障注入单测绿）。
- findings（均不阻断）：
  - F-1（存量，非本计划引入）：`parity_handoff_store` 编译错——
    `StartRunRequest` 缺 `authorization` 字段；main@da748ac 同错复现两次；
    疑 PLAN-096 T-12（4393199）锁步手修漏改该 parity 夹具。去向：merge
    阶段登记 KNOWN-DEBT（修法=夹具补 2 处字段，属 096 修补面）。
  - F-2（存量，非本计划引入）：`tool_atoms::
    run_command_dangerous_returns_paused` panic（'/' 路径解析越界先于门
    断言）——环境敏感，main 同像复现。去向：KNOWN-DEBT（断言与环境根
    解析解耦）。
  - 核定①（SD-03 挂载）：**维持 plan-flow.md**——规则源在新模块
    run-telemetry.md，plan-flow.md 仅承载"四阶段 run 产物满足该契约"的
    引用；plan-execution-contract.md 管辖受控执行（worktree 绑定/阶段路
    由），与可对比重跑产物分域；merge 时可选择性在后者加互引（非要求）。
  - 核定②（D4/D6 修订）：**接受**——AC-01/AC-02 在修订后语义下经 V04
    实况工件与 V05 契约断言实证满足，属实现机制澄清而非验收弱化；已记
    录于证据报告（不改变 plan_revision 语义合同——验收文本未动）。
  - 核定④（relay 实况加强）：**闭项**——计划 §10 明文不作为完成门；
    V05 契约断言 + 094 真实产物 + 签名阳性 fixture 已覆盖可对比面。
- evidence: review 复跑日志要点已摘录入本记录；V03 断言集与 fixture 随仓
  （`docs/reports/plan-flow-uat/fixtures/`）；V04 实况会话
  db7b23cedffcf99423379abc（tmp/demo，易失面——证据以 evidence 报告摘录
  与摘要 JSON 概要为准，已入 worktree 提交 319cb4f）。

### merge 阶段沉淀收据（PLAN-098:r1，completion_kind: delivered）

- stage: merge ｜ plan_id: PLAN-098 ｜ plan_revision: 1 ｜ outcome: **pass** ｜
  delivery_commit: main@ce37460（= plan-098-dev rebase 后 tip；8 提交 ff-only
  落地，无 merge commit）。
- **prepared**：canonical specs 在 worktree 准备并提交（2f8f65f，reviewed
  319cb4f 的纯文档后代——实现/依赖零改动）：
  `docs/specs/modules/run-telemetry.md`（新，75 行：记录契约/折叠挂点/
  关联/缺失语义权威序/SSE/摘要脚本/已知限制）；
  `chat-streaming.md` +「运行遥测暴露（SD-02）」节；
  `plan-flow.md` +「运行遥测（PLAN-098）」引用节（SD-03，review 核定①
  维持 plan-flow.md 挂载）；`docs/specs/index.json` 注册新模块；
  KNOWN-DEBT 登记 098-R1/R2（review F-1/F-2 裁定去向）。
- **landed**：rebase main（52038cd，097 会话 docs-only 簿记提交，零冲突）；
  range-diff `da748ac..319cb4f` vs `52038cd..42627ad` **7/7 全等（=）**——
  旧→新映射：330768c→6b39fa3、e2f906c→f15a9cf、2778e33→ae9f442、
  f9ae074→c025570、abe9c77→a902a0e、ccfa573→e8ebfa6、319cb4f→42627ad；
  wt-guard clean；main 检出 `git merge --ff-only` 成功（52038cd→ce37460，
  fast-forward 行确认）；落地后 main 冒烟 `cargo test -p musk --lib`
  **545 passed 0 failed**。
- **ledger_refreshed**：workspace `auto-musk`（运行时 ignored 账本
  `.autoos/specs.json`），store-mediated HTTP（worktree release musk :8080
  起、用毕停——specs_upsert 工厂路径 + transition 状态机边）：
  designs/`PLAN-098-run-telemetry`（→**approved**）、
  reviews/`PLAN-098-review`（→**published**）；读回验证通过（doc version
  5→7）；无手工 JSON 写入。
- **archived**：本计划归档至 `docs/plans/archived/098-plan-flow-telemetry.md`
  （untracked 计划随文件移动+`git add` 落库），frontmatter status: archived。
  归档提交 27c4299。
- **cleaned**：worktree `D:/autostack/.wt/musk-098/auto-musk` 移除前
  wt-guard clean 复验 + `HEAD 是 main 祖先`核验（全部提交已落地）；分支
  `plan-098-dev` 删除（was ce37460）；依赖只读兄弟 detach worktree
  （auto-ai@main、auto-lang@master，各自源仓移除）；组目录
  `D:/autostack/.wt/musk-098` 已删（空）；`git worktree list` 无 musk-098
  残留。
- **产物检查（landing is not deployment）实测**：musk release binary 主检出
  落地后重建完成（`cargo build -p musk --release` 3m04s Finished，二进制含
  遥测面）；aaid=auto-ai 只读未触及（:17654 在跑实例继续有效）；
  gen/front/vue/dist 零前端改动不适用重建；生产 :17201 serve 当时未运行
  （无运行面需重启）。
- **批量回归到期核查**：`docs/plans/.last-batch-regression.json` **缺失
  = 到期**；已按技能交 `/auto-plan:regress` 在主检出跑批量全量门（单实例）。

## 10. 待澄清事项

- 无阻断项。两点留 review 核定：① SD-03 挂载目标是否随 096 的
  plan-execution-contract.md 落地改挂；② relay 路径是否追加一次真实
  run 实况记录（V05 契约断言之外的加强项）——取决于 096 合并后 relay
  planner 稳定性，T-05 时再定，不作为本计划完成门。
