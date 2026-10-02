# PLAN-096 规范增量报告（SD-01～SD-04）与 B 消费合同定版

> plan_id: PLAN-096 · plan_revision: 1 · 任务 T-13（AC-01～14,17 映射）
> 本报告按计划 §5「规范增量」表逐条给出可审 before/after；canonical Spec
> 修改在 merge 阶段落地（本报告不提前发布）。B 合同字段/事件以 §4 定版。

## SD-01 modify `docs/specs/modules/plan-flow.md`

**before（现行规范要点）**：相位纪律内化于 `relay/plan_flow.rs` 模板
（"relay factory `skills: false`：相位纪律内化于模板，不挂 skill 工具"）；
review 失败 → transition 回 executing、"run 正常结束，用户决定是否续跑修复"
（无有界修复）；document 相位调 `merge_plan`（章节复制沉淀+立即归档）。

**after（本期落地）**：
- 固定四职业消费四技能 hash 快照（`plan_contract::snapshot_plan_skills`，
  启动冻结、缺源硬失败、中途技能变化不静默改变本次运行）——"relay factory
  skills:false" 语义升级为"纪律单一真源=技能快照注入相位任务"。
- 相位推进唯一凭据 = `complete_plan_stage` 结构化结果 + 服务器事实核验
  （`plan_control::on_stage_end`）；Done/handoff/绿勾不推进（stage_incomplete）。
- review/needs_fix 自动 work→新 review：≤3 修复轮（repair_count/limit 可回读）、
  无进展早停（同 findings/commit/evidence 跨轮比对）、findings 注入修复轮
  coder 任务；needs_replan/blocked 停止且计划留 active。
- document 相位 = `plan_delivery` 受控交付（prepare/land/refresh/archive/
  cleanup 五检查点），`merge_plan` 对受管计划拒绝（AC-12）。
- 实现锚点：`relay/plan_contract.rs`、`relay/plan_control.rs`、
  `relay/plan_runtime_client.rs`、`plan_delivery.rs`、`plan_worktree.rs`、
  `relay/plan_flow.rs`（模板=职业/机械输入/结果协议）、`relay/driver.rs`
  （受控收束+runtime client 注入+worktree scope）。
- 验收：AC-01,02,04,05,06,09,10,12,16（证据：V02 19 绿/V04 9 绿/V05 回归，
  commit 7f4bdbb..54f02fa）。

## SD-02 add `docs/specs/modules/plan-execution-contract.md`（新模块）

新增规范内容（机械行为与 Agent 主张分开）：
- **PlanExecutionBinding**：contract_version/workspace_id/main_root/plan_id/
  plan_path/plan_revision/contract_hash（批准时字节 hash）/semantic_hash
  （进度无关语义 hash）/skills_hashes/default_branch（Git 实测，不硬编码）/
  base_commit/execution_root/dev_branch/authorization（human|auto 记录取数
  来源）/repair_limit/dependency_revisions（MUSK_PLAN_DEP_DIRS 冻结）。
- **语义 hash 规范化**：plan_id/feature_name + 三 spec-impact 列表 +
  §1/§2/§5/§7/§8（勾选→`[_]`、`[✅…]` 尾标删除、`证据：` 后截断）；
  进度/时间戳/§9/§10 不敏感（正反样例钉死：tests/plan_flow_contract.rs
  `semantic_hash_progress_positive_and_semantic_negative`）。
- **阶段结果**：StageResult{stage,plan_id,attempt(服务器盖章),plan_revision,
  outcome,commit,acceptance_results,findings,evidence(工件路径或`cmd:`命令
  记录),spec_delta_ref}；服务器核验=回读计划/Git 状态/证据工件存在性，
  语义正确性归 reviewer 真实测试。
- **受控回退**：`engine_rewind_to_step`（musk 侧唯一入口，保已过人审门），
  不修改 auto-ai 引擎。
- **交付检查点**：prepared→landed→ledger_refreshed→archived→cleaned，
  receipt keyed `PLAN-NNN:rN`；跨步非事务、重入对账只补缺项；cleanup 失败
  = cleanup_pending（archive 保持 delivered）。
- **取消/单写者**：per-run owner 互斥 + cancel 旗标（POST /runs/{id}/cancel），
  取消保留现场。
- 验收：AC-02～14,16,17。

## SD-03 modify `docs/specs/modules/specs-ledger.md`

**before**："写入通道只经 store 语义" 已定；但 plan-flow 的 document 相位
走 `merge_plan` 章节复制 upsert + 归档（规范与通道不匹配）；无源 metadata
写入约定。

**after**：
- 正式交付的 ledger 刷新唯一通道 = `plan_delivery refresh`（SpecsStore
  upsert，item.file=canonical 路径、tags=[source:<sha256>, commit:<交付提交>,
  plan:<PLAN-NNN>]；回读核验）。受管计划的 `merge_plan`/章节复制沉淀拒绝。
- load 失败 → 交付 blocked 响亮（不假 delivered、字节零破坏）——复用
  K6 教训条款并落地到交付链（V04 `refresh_store_mediated_…corrupt_ledger`）。
- 重复刷新零增长（幂等 upsert 测试钉住）；未涉及条目/历史保留。
- 验收：AC-08,11,17。

## SD-04 modify `docs/specs/modules/workspace-sandbox.md`

**before**：多根判定 = [workspace 根, *白名单]；无 run 级作用域概念。

**after**：
- plan 流 run 的 **execution scope**（`ToolContext.execution_root`）：
  execute/review/document 相位的文件/命令工具与 Canvas 路径解析限定在
  该 run 的开发 worktree（`plan_worktree::ensure_plan_worktree` 租约：
  main checkout 校验/默认分支探测不硬编码/占用不接管/reparse guard/
  合入核对前置的安全移除）。
- 主检出代码**不注册为可写根**；白名单语义与用户配置不变（新增的是
  run 级注入根，非全局白名单扩权）；plan 工具仍写主 PlansStore（共享
  状态例外）。
- Canvas：`resolve_within_sandbox` 优先 execution_root——目标 App 可落
  run worktree 并显示归属；越界拒绝报文列出授权根。
- 验收：AC-03,10,11,12。

## B 消费合同（定版样例，AC-13）

**RunState 可选字段**（serde skip_serializing_if=None，旧消费者无感）：

```json
"plan_execution": {
  "contract_version": 1,
  "plan_id": "PLAN-001",
  "plan_seq": 1,
  "plan_path": "docs/plans/001-calc-fix.md",
  "plan_revision": 1,
  "phase": "document",            // plan|execute|review|document|delivered
  "attempt": 1,                    // 轮次号（execute/review 共享，needs_fix +1）
  "outcome": "pass",
  "repair_count": 0,               // 已完成的 work→review 修复轮
  "repair_limit": 3,
  "continuations": {"review": 0},  // 每阶段截断续做计数（≤1）
  "blocker": null,                 // needs_replan/blocked/cancel 的原因
  "binding": { "…": "见 SD-02" },
  "reviewed_commit": "<sha>",
  "delivery_checkpoints": {"prepared": {…}, "landed": {…}},
  "receipt_ref": ".autoos/plan-delivery/PLAN-001.json",
  "skills": {"auto-plan-new": {"sha256": "…"}},
  "stage_results": [ { "stage": "plan", "attempt": 1, "outcome": "pass", "…": "…" } ]
}
```

**RunEvent 新变体**（沿旧 bus + 会话镜像；`event_type=plan_stage_facts`）：

```json
{"type": "plan_stage_facts", "timestamp": 1760000000, "facts": {
  "plan_id": "PLAN-001", "stage": "review", "attempt": 2,
  "outcome": "needs_fix", "repair_count": 1, "repair_limit": 3,
  "blocker": null, "reviewed_commit": null,
  "delivery_checkpoint": null, "receipt_ref": null }}
```

成功样例序列：`approve/bound → plan/pass → execute/pass → review/pass →
document/pass [checkpoint:all]`；失败样例：`execute/needs_replan`（blocker
带原因）、`review/no_progress`、`review/repair_limit`、`plan/stage_incomplete`、
`cleanup_pending` 检查点。消费规则：B 轨以 facts 序列还原阶段时间线，不解析
自然语言 handoff；`receipt_ref` 为 workspace 相对路径。GET run 的 events 为
500 条窗口（既有已知限制 F-R1）——全量证据以 durable receipt + 会话镜像为
准。旧字段（status/steps/…）不变；V04 `plan_run_external_handoff_rerun_…`
断言事实面形状。

**B 依赖边界**：B 只消费 A 交付后的字段/事件（本报告为定版）；A 的 WIP
不作 B 依赖。VM UI 投影与截图由 B 接入（A 只提供真实数据面）。

## 已知限制

- 服务重启不自动恢复 run：显式 plan-merge 重入经 durable receipt 对账
  （§5.7）；归档后的旧 run 无足够凭据时只读核实。
- 依赖冻结通过 `MUSK_PLAN_DEP_DIRS`（name=path;…）显式声明；未声明的
  依赖不入绑定（不做隐式全仓扫描）。
- 输出上限为 musk 局部包装（plan 流默认 16384，`MUSK_PLAN_MAX_TOKENS`
  覆盖）；供应商对上限的接受度由 L1 live 实证，不承诺计费语义。
