# PLAN-098 规范增量对照（SD-01～SD-03）

- plan_revision: 1 ｜ 基线: main@da748ac（docs/specs 现行文本）
- 交付物实现锚点: plan-098-dev@abe9c77（T-02 e2f906c / T-03 2778e33 /
  T-04 f9ae074 / T-05 abe9c77）
- 本文件为 review 可核的 before/after 对照；canonical `docs/specs/` 编辑由
  merge 阶段按本文落地（work 不直接改写主检出规范）。

## SD-01 add — docs/specs/modules/run-telemetry.md（新模块）

- **before**: 无任何运行遥测/模型归因规则（plan-flow.md 与 chat-streaming.md
  均无 model 元数据规则；aaid 日志是唯一模型观测面且无关联键）。
- **after（新模块全文要点，merge 时落地为独立文件并在 docs/specs/index.json
  注册）**:
  - 每轮 assistant 产物（chat 收束轮 / relay 相位边界轮）**尽可能**记录实际
    服务模型事实：`telemetry` 对象（provider、model、requested_model、
    in/out_tokens、elapsed_ms、stop_reason、stream、error、correlation、
    多请求时轮级合计+`requests[]` 逐请求行）。
  - 载体：聊天与 relay 共用 `turns.jsonl` 单写点（`turns[].telemetry` 可选
    字段）；relay 相位遥测折叠在 StepCompleted 边界轮（流式 delta 碎片不可
    对齐请求边界——PLAN-098 T-01 D4）。
  - 关联：correlation 构造期显式注入（chat=workspace+session；
    relay=run_id+step_id+role）；并发零错配为结构性质（每 run 实例私有
    sink），未包装路径不产生记录（强于记 null——零错配、零孤儿行）。
  - 缺失语义：daemon wire 不携带 provider → 恒显式 null，不伪造；model 权威
    序 = `model_meta.id`（PLAN-031）→ `resp.model`（daemon 解析 tier 后的
    实际服务值，PLAN-098 T-05 live 实证）→ null；usage 缺失记 null，流式
    `in=0`（K8）如实记 0。遥测为旁路：sink 故障只 warn，绝不影响轮次结果。
  - 兼容：serde default + 省略序列化，旧 turns.jsonl 零迁移照读（V06）。
  - 旁证：aaid 日志解析（去 ANSI）仅作 provider/耗时的启发式补强
    （±5s+model+stream 时间窗，唯一候选=high），永不冒充权威。
- rationale: "每步哪个模型服务"成为可对比运行的一等事实（重跑对比不依赖
  daemon 日志存活或手工时间戳对齐）。
- acceptance: AC-01、AC-02、AC-04 ｜ 实证: V01/V03/V04/V05/V06（evidence
  报告命令表）

## SD-02 modify — docs/specs/modules/chat-streaming.md

- **before**（现行契约 §1/事件枚举）: 轮次与收束事件
  （`turn_start`/`turn_end`/`done` 等 SseEventDto 枚举）无任何模型元数据
  字段。
- **after（在契约清单追加一条，事件枚举其余不变）**:
  - `turn_end`/`done` 事件在模型事实已知时附 `model_meta` 对象
    （provider/model/usage.{in,out}）；未知时**省略该字段**（不写 null 占位，
    旧前端/消费方零感知）。实现为 DTO 序列化后的 Value 级增量追加，DTO
    枚举与帧协议形状其余不变（wire 兼容增量）。
  - 前端零消费要求保持：Vue/VM 不读取该字段，行为零回归（AC-03 的 V02
    前端契约回归面）；字段为消费方预留（含 PLAN-097 演示类用途）。
- rationale: 事件流与落盘物同源（同一 sink），实时侧可获得与 turns.jsonl
  一致的归因。
- acceptance: AC-03 ｜ 实证: V01 事件构造断言 + V02 全套（前端契约零回归）

## SD-03 modify — docs/specs/modules/plan-flow.md

- **before**: 四阶段 run 的产物规则（相位输入/计划文件/门不变式）不含任何
  模型归因要求；运行对比依赖 daemon 日志与手工时间戳对齐。
- **after（追加一节"运行遥测（PLAN-098）"）**:
  - 四阶段 run 的 turn 产物满足 run-telemetry.md 记录契约（**引用而非复制
    规则**）；每相位收束（StepCompleted 边界轮）携带该相位聚合遥测，
    correlation 含 run_id/step_id/role。
  - aaid 日志仅为旁证（daemon 无关联键，时间窗启发式，置信标注）；重跑
    对比以 turns.jsonl 内嵌遥测与 collect-telemetry 摘要为权威记录物。
  - RUNBOOK §4 第 0 步（PLAN-098 起）为落账前置：每 run 产出遥测摘要并在
    结果总表/历史记录中引用其路径。
- **挂载目标（plan §10 留 review 核定项）**: plan-execution-contract.md
  已随 PLAN-096 落地 main。两案并列：
  ① 维持本计划默认——规则挂在 plan-flow.md（计划流程的可对比重跑语义）；
  ② 改挂 plan-execution-contract.md（若 review 认定"运行产物契约"归受控
  执行合同域）。规则文本不变，仅挂载点不同。
- acceptance: AC-02、AC-05、AC-06 ｜ 实证: V03/V04/V05 + RUNBOOK §4 评审
