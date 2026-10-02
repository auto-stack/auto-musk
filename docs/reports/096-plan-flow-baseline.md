# PLAN-096 T-01 基线报告 — plan-flow 契约改造前置核查

> plan_id: PLAN-096 · plan_revision: 1 · 任务 T-01（AC-02/03/09/12/15）
> 执行位置：worktree `D:/autostack/.wt/musk-096/auto-musk`（分支 `plan-096-dev`）
> 报告落盘：`docs/reports/096-plan-flow-baseline.md`（worktree 内新文件）
> 本报告只登记核查事实与固定设计决策；不改共享生产、不动 canonical Spec。

## 1. 依赖冻结（执行时重冻结，按 AGENTS 解析序）

| 仓库 | 冻结版本 | 检出位置 | 用途 |
|---|---|---|---|
| auto-musk | main@39870da（worktree 基点，含计划文档落盘提交） | `D:/autostack/.wt/musk-096/auto-musk`（分支 plan-096-dev） | 实施仓 |
| auto-ai | main@5a50a55（=计划§4 基线一致） | `D:/autostack/.wt/musk-096/auto-ai`（只读 detached） | Client/角色/编排引擎合同 |
| auto-lang | master@986e765ac（计划基线 142458d2 之后前移；§4 预授权"执行时重新冻结当前版本"） | `D:/autostack/.wt/musk-096/auto-lang`（只读 detached） | auto-atom/auto-val/auto-lang 编译+VM |
| auto-down | master@895f8d0 | `D:/autostack/.wt/musk-096/auto-down`（只读 detached） | auto-lang 可选依赖 autodown-core 路径补齐 |

- 解析链实测：musk `Cargo.toml` 相对路径 `../../../../auto-ai/...` 自 worktree
  `backend/crates/musk/` 解析到组内兄弟 `D:/autostack/.wt/musk-096/auto-ai`；
  auto-ai-agent 内 `../../../auto-lang/...` 同理落组内 `auto-lang`。三兄弟均为
  `git worktree add --detach`（不占用分支名，无 junction/symlink）。
- 全部只读：本期零外仓写入；auto-ai daemon/Agent 库/通用 PipelineEngine 不改。

## 2. V01 基线与既有测试基线

- **V01** `cargo check --manifest-path backend/Cargo.toml -p musk`：**PASS**
  （dev profile，1m42s，exit 0；3 个 warning 均为既有 dead-code/suggestion 类）。
- **V05 scoped parity 基线**（parity_relay/driver/store/api + parity_plans）：
  命令同计划 §6 V05；结果见 §8 附录（执行记录）。
- **V07 lib 基线**：同附录记录；基线红逐一归因（受影响必需 AC 不豁免）。

## 3. 技能快照核验（AC-01 事实源）

四技能 SKILL.md SHA-256 与计划 §4 表逐字节一致（2026-10-02 复核）：

| 文件 | SHA-256（前 8） |
|---|---|
| .agents/skills/auto-plan-new/SKILL.md | 23543fc7 |
| .agents/skills/auto-plan-work/SKILL.md | 8ee80140 |
| .agents/skills/auto-plan-review/SKILL.md | dab315e3 |
| .agents/skills/auto-plan-merge/SKILL.md | 46d7399b |

四个规范文件（plan-flow / specs-ledger / workspace-sandbox / chat-run-policy）
hash 同样逐一命中计划 §4 表。`builtin_skills.rs::skills_source_root` 的源解析链
（`MUSK_SKILLS_DIR` env → CWD `.agents/skills` → CARGO_MANIFEST_DIR 回溯）在
worktree 内命中本计划 worktree 的 `.agents/skills/`（serve 分发目标 =
`MUSK_CONFIG_DIR` 覆盖或 `~/.config/autoos/skills`，既有单测钉住幂等语义）。
T-02 据此实现"启动时四技能内容+hash 快照、缺源/读失败/不一致即硬失败"。

## 4. 阶段引擎公开状态与受控回退（Q-01 结论）

`auto_ai_agent::orchestration::PipelineEngine`（rust-ref/src/orchestration/
pipeline.rs）字段全部 `pub`：`flow / current_step / status / run_id /
step_history / loop_counters / pending_gate / gate_feedback /
gate_resolved_for_step / resumed_step_id / cumulative_tokens / budget_tracker /
mode`；状态机 `PipelineStatus::{Idle, Running, WaitingForHuman, Completed,
Failed, Paused}`。`RunStore` 的 RunEntry 直接持有 engine（musk 已有多处
`entry.engine.<field>` 直读直写先例：store.rs/plan_flow.rs/驱动）。

**固定决策（Q-01）**：受控回退走 **musk 侧唯一入口**
`plan_control.rs::engine_rewind_to_step(engine, step_id)`——把 `current_step`
拨回目标相位索引、`status` 置 Idle、清 `pending_gate`/`gate_resolved_for_step`
（+按需清该步 gate_feedback），`step_history` 截断至该相位之前的记录
（旧记录由调用方决定保留进 §9 日志字段，不冒充引擎历史）。不修改 auto-ai
引擎；调用方仅限 PlanControl 路由（needs_fix → execute、超限停止），
不得作为通用 API 暴露。旧 history/失败证据保留在 PlanControl 的
attempt 记录里（AC-05 计数可回读）。

## 5. YAML 完整 frontmatter 读取（Q-02 结论）

- 现状：`plans.rs::parse_frontmatter` 是手写标量解析器——跳过列表项与空行
  （`frontmatter_skips_list_items` 测试即证），**列表字段丢失**
  （supersedes_spec_components / new_spec_components / touched_goals 读不到）。
- 锁文件核查：`yaml-rust 0.4.5` 已在 `backend/Cargo.lock`（经 syntect 传递
  引入，`cargo tree -p musk -i yaml-rust` 实证）。**固定决策**：musk 直接
  声明 `yaml-rust = "0.4"` 依赖（锁内已有版本，无新包进入锁文件），新增
  `plan_contract.rs` 独立完整解析 `PlanContract`（frontmatter 全字段含三列表
  + 必要章节/任务/AC 结构），**不替换** plans.rs 旧标量 parser（旧显示/兼容
  路径不动）。解析失败（非法 YAML、id 不匹配、章节缺失）→ 明确拒绝。
  锁文件 diff 预期：仅 `musk` 依赖表新增 `yaml-rust` 条目。

## 6. Auto 桥再生与双轨边界（Q-04 结论）

- 既有再生命令链：`auto-src/*.at` → `auto trans <x>.at rust`（a2r）→
  `src/auto_generated/<x>.rs`，已知手修定式两处（Loop struct 语法 /
  `for f in flows` 按值迭代，KNOWN-DEBT 086 条）；`auto` CLI 可用
  （`D:/autostack/auto-lang/target/debug/auto`，0.1.0+v0.4.2-2511）。
- `auto_generated/extern_impl.rs`（3944 行）是 ag 轨的 hw 委托层：
  `factory_build_agent`（→hw MuskAgentFactory）、`drive_submit_handoff`
  （双驱动共用的 plan_file 标记守门）、`relay_resolve_gate`（execute 门
  不变式 hw/ag 同规则）已是单源委托先例。
- **固定决策（Q-04）**：本期新增的共享核（PlanContract 读取、阶段结果校验、
  PlanControl 路由、交付检查点、worktree scope）全部落在 **hw 新模块**
  （`relay/plan_contract.rs`、`relay/plan_control.rs`、
  `relay/plan_runtime_client.rs`、`plan_worktree.rs`、`plan_delivery.rs`）；
  ag 轨消费走 extern_sigs.at 声明 + extern_impl.rs 委托（单点改动双轨生效，
  沿 094 先例）。**T-10 只对必须改的 .at（relay_driver/relay_flows/relay_store/
  relay_api/extern_sigs/tool_context）做最小再生或等价手改+委托**；不再生全
  模块（有 drift 在案）；VMHTTP 轨真实消费在 T-12/V06 live 实证，不以 ag
  测试冒充 VM 实跑。生成产物不作独立事实源——修改以 .at/委托层为准。

## 7. Client 输出上限（Q-03 前置核）

- `CompletionRequest`（ai-config rust/src/wire.rs:159）含 `max_tokens:
  Option<u32>`；`CompletionResponse` 含 `stop_reason: Option<String>`；
  `auto_ai_agent::Client` trait（rust-ref/src/agent.rs:62）暴露
  `complete` / `complete_stream`，均可被 musk 侧包装（委托 inner Client，
  改写请求、透传事件/usage/取消）。`Agent::build_request` 固定发
  `max_tokens: None`（agent.rs:984）——mus 局部包装在 plan 流请求上填默认
  16384（env `MUSK_PLAN_MAX_TOKENS` 可覆盖；已显式值不覆盖）。
- **边界**：实际模型对 max_tokens 的接受度、计费、thinking 语义在 T-12 live
  前置核对（L1 先行验证），不预设所有供应商接受；live 未就绪按 blocked 登记。

## 8. 附录：基线命令执行记录

- **V05 基线**（worktree@39870da，2026-10-02）：
  `cargo test … --test parity_relay --test parity_relay_driver --test parity_relay_store --test parity_relay_api --test parity_plans -- --test-threads=1`
  → 全绿无既有红：parity_plans 5 passed / parity_relay_api 6 passed+1 ignored /
  parity_relay_driver 6 passed / parity_relay_store 7 passed（exit 0）。
- **V07 基线**（main 检出@39870da 纯净树，2026-10-02）：
  `cargo test --manifest-path backend/Cargo.toml -p musk --lib -- --test-threads=1`
  → **505 passed / 0 failed** / 1 ignored（exit 0）。无既有红；本计划引入的
  红必须全数归因。
  （注：worktree 内首跑 V07 与 T-02 在途编辑相竞产生 4 个瞬时编译错误，
  已作废；纯净基线以 main 检出重跑为准，T-02 落盘后同口径
  518 passed / 0 failed。）

## 10. 现有红登记

V05/V07 基线均无既有红（0 failed）。本计划引入的红一律在任务证据中归因，
不允许豁免受影响必需 AC。

## 9. 临时端口与 scope 方案（T-12 用）

- live 探针（V06）沿用 093 的 canvas-studio-live.mjs 隔离模式：临时目录
  自建演示仓 + `MUSK_CONFIG_DIR`/`MUSK_SKILLS_DIR` env 隔离 + 独占端口
  （默认从 18080 起探空闲，`--port` 可指定）+ owned 子进程（退出即收），
  禁止触碰 8080 共享 serve 与既有 tmp/demo 目录。
- scope：T-04 的 worktree 授权根按 run 登记（ToolContext 扩展），主检出
  代码不注册为可写根；Canvas app_path 允许落 run worktree。

## 10. 现有红登记

（V05/V07 基线执行后回填；若有红，逐条记录测试名与归因——计划 §6 要求
"基线红逐一鉴别，受影响必需 AC 不允许豁免"。）

## 11. B 消费合同初稿（T-02 定稿，T-13 定版）

RunState 新增可选 `plan_execution`（contract_version / plan_id / revision /
phase / attempt / outcome / repair_count / repair_limit / blocker /
reviewed_commit / delivery_checkpoints / receipt_ref），RunEvent 新增
`plan_stage_facts` 等价事实事件（旧字段保留，serde default 兼容）；
receipt_ref 为 workspace 相对路径。**初稿字段以 T-02 的
`relay/plan_contract.rs` serde 形状为准；定版在 T-13 报告冻结样例。**
计划 B 只允许消费 A 交付后的字段（AC-13）；A 的 WIP 不作为 B 依赖。
