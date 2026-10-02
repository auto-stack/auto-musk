# plan execution contract — 计划执行绑定与受控交付（PLAN-096）

> PLAN-096 定型（2026-10-02，reviewed 2ed2210）。产品内 `/auto-plan:*` 流的
> 机械执行合同：身份绑定、阶段结果、有界修复、输出预算与受控交付。
> 实现锚点：`relay/plan_contract.rs`、`relay/plan_control.rs`、
> `relay/plan_runtime_client.rs`、`plan_delivery.rs`、`plan_worktree.rs`。

## 身份与批准绑定（PlanExecutionBinding）

- 门批准（human 或 auto 桥接）时冻结：`contract_hash`（批准时字节 SHA-256）、
  `semantic_hash`（进度无关语义哈希）、`semantic_parts`（分部件诊断）、
  `skills_hashes`、`default_branch`（Git 实测：origin/HEAD →
  init.defaultBranch → 当前分支，不硬编码）、`base_commit`、
  `execution_root/dev_branch`（worktree 租约）、`authorization`（human|auto，
  取数来源记录）、`repair_limit`（默认 3）、`dependency_revisions`
  （MUSK_PLAN_DEP_DIRS 显式声明的冻结）。
- **语义哈希**覆盖 plan_id/feature_name + spec-impact 三列表 + §1/§2/§5 +
  §7/§8 checklist 行（id+规范化文本）；归一化剥除：`证据：`注记、状态
  emoji（✅/⏳/【）起的行内 tick/证据、悬挂括号/破折号、checkbox→`[_]`。
  §7/§8 节内非 checklist 行（执行记录/证据子行）不进语义。进度、时间戳、
  §9/§10 不敏感。checkbox 解析用惰性量词（任务行内第二个 `]` 不吞行）。
- 批准后**语义漂移 → needs_replan**（re-approval）；仅进度变化不失效。
  漂移报文点名 drifted parts 并输出 approved/current 逐行 diff
  （binding.approved_canonical 内存快照）。

## 阶段结果与单写者路由

- 相位推进唯一凭据 = `complete_plan_stage` 结构化结果（stage/plan_id/
  attempt 服务器盖章/outcome/commit/acceptance_results/findings/evidence/
  spec_delta_ref）；Done/handoff/绿勾不推进（stage_incomplete 置败）。
- `PlanControl::on_stage_end` 服务器核验后路由：plan/pass 回读计划合同；
  execute/pass 验语义未漂移+worktree 已提交（HEAD≠base、无未提交变更）+
  AC 全 pass 带证据+证据工件存在（无空白含 `/` 或 name.ext 的条目按路径
  核验，`cmd:` 前缀为命令记录不执行）+依赖未漂移；review/pass 另验后置
  reviewed_commit；needs_fix 有 findings 稳定 id 才受理。
- **有界修复**：needs_fix → 计划状态回 executing + `engine_rewind_to_step`
  （musk 侧唯一回退入口，gated 步保留已过人审门——AC-05 无手工 nudge）+
  attempt/repair_count+1 + findings 注入修复轮 coder 任务（`repair_findings`
  上下文，step_context 以「复审发现」块附加）。上限 3 轮；无进展早停
  （同 findings id 集+同 commit+同 evidence 跨轮比对）。attempt 为轮次制
  （execute/review 共享轮号）。
- needs_replan/blocked：停止、blocker 落事实、计划留 active 不 document。
- 单写者：per-run owner 互斥；取消旗标（POST /runs/{id}/cancel）在下一次
  阶段收束即停、保留现场；删除 run 时清旗标。
- **驱动孵化分流**：带 plan_execution 的 run 一律走 hw 受控驱动
  （`drive_run_dispatched`）——ag 转译驱动对 plan run 盲提交 handoff 会
  完全绕过受控路由（live 实证）；全部孵化点（hw/ag REST、gate-resume、
  chat owner、merge 短路、SpawnRelay、task_plan engine）已经接线。

## 输出预算与截断（PLAN-096 §5.5）

- plan 流 run 的 LLM 请求经 `PlanRuntimeClient` 局部包装：缺省补
  `max_tokens=MUSK_PLAN_MAX_TOKENS`（默认 16384；部署可覆盖，显式值不
  覆盖）；其他流/全局 daemon 不变。
- stop_reason ∈ {max_tokens, length, max_output_tokens, truncated} 登记
  截断：不准以截断跨阶段成功；每阶段一次有界续做（同相位重入、旧声明
  作废、与修复轮分开计数）；再截断 → 带工件位置的响亮失败。end_turn/
  tool_use 等不误判。

## 执行 worktree（plan_worktree.rs）

- 租约：main checkout 校验（top-level+common-dir 归属）、默认分支探测、
  layout（MUSK_PLAN_WORKTREE_ROOT → `{parent}/.musk-wt/{plan_id}/{repo}`）、
  `create_new` 式创建（外部占用/分支碰撞不接管）、reparse point 扫描
  guard（wt-guard 同语义，`dir /a:l` 输出判定不看退出码）、移除前先做
  合入核对（unmerged 保留现场报 pending）再 `worktree remove`+`branch -d`、
  空组目录才裁剪。
- ToolContext.`execution_root`：execute/review/document 相位的文件/命令
  工具根与 Canvas 路径解析限定 worktree（主检出代码不注册为可写根；
  白名单配置不变；plan 工具仍写主 PlansStore）。

## 受控交付（plan_delivery.rs，五检查点）

- prepare（worktree 内提交 docs/specs/ 增量 → delivery_commit；无增量拒；
  幂等按 HEAD 核对）→ land（guard+clean、rebase 默认分支+旧→新映射+
  range-diff 等价证明（任何 `!` 行拒绝）、主检出 `--ff-only` 无 merge
  回退、canonical tip 核对）→ refresh（SpecsStore upsert：item.file=
  canonical 路径、tags=[source:hash, commit:交付提交, plan:PLAN-NNN]；
  回读核验；load 失败 blocked 响亮、字节零破坏）→ archive
  （finalize_archived、completion_kind=delivered）→ cleanup（guard+合入
  核对+移除自有 worktree/分支+空组目录裁剪；失败=cleanup_pending，
  archive 保持 delivered，只补 cleanup）。
- durable receipt：`{ws}/.autoos/plan-delivery/{PLAN-NNN}.json`，键
  `PLAN-NNN:r{revision}`；重入对账只补缺项。每成功结果带 `next_action`
  顺序指引；前置缺失报错指名下一动作（降低同参盲重试；agent 循环防护
  不放宽）。

## 旧入口旁路封闭（AC-12）

受管计划（managed_by：非终态 run 绑定且未 delivered）拒绝：
`merge_plan`/`transition_plan` 工具与 POST /api/plans/{seq}/merge|
transition|archive HTTP（409，未受管计划保持旧语义）；plan 流 run 的
外部 handoff/rerun REST 409。`/auto-plan:merge` 短路与 ag REST start_run
走同一 bootstrap（技能快照+plan-merge 目标合同），bootstrap 失败回真实
错误不启动注定失败的 run。

## 事实面（B 轨消费合同）

RunState 可选 `plan_execution`（contract_version/plan_id/plan_seq/
plan_path/plan_revision/phase/attempt/outcome/repair_count/repair_limit/
continuations/blocker/binding/reviewed_commit/delivery_checkpoints/
receipt_ref/skills/stage_results；serde default 旧消费者兼容）；RunEvent
`plan_stage_facts`（RunPlanEvent 载荷）沿旧 bus+会话镜像。B 轨以 facts
序列还原时间线，不解析自然语言 handoff。receipt_ref 为 workspace 相对
路径。GET run events 为 500 条窗口（F-R1 既有限制），全量证据以 receipt
+会话镜像为准。

## 已知限制

- 服务重启不自动恢复 run；显式 plan-merge 重入经 receipt 对账。
- 模型全文回写计划的采样噪声（单字符损坏）会触发 needs_replan——合同
  正确行为；增量 tick 工具是消除该噪声面的后续计划候选（D-1）。
- 依赖冻结仅覆盖 MUSK_PLAN_DEP_DIRS 显式声明项。
- live 探针（scripts/plan-flow-probe.mjs）：L1 全链 delivered、L3 响亮停
  已真模型 PASS；L2 循环演示受复审随机性与采样噪声影响未取绿（机制由
  V02 矩阵确定性覆盖），模型稳定后可同探针复跑（D-2）。
