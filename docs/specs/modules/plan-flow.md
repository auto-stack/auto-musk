# plan flow — 计划驱动开发流程（relay 编排）

> PLAN-086 定型。前史：PLAN-030 立形（单角色 plan-dev 四相位，handoff render
> 注入 + `PLAN_FILE:` 正则为主通道）；本轮固定职业分档，并把相位间传递机械化。
> 本文为该流程的规范性描述；实现锚点见各节行内引用。

## 流程形状（静态契约）

`plan` flow 四相位固定职业，顺序与门不可配置：

| 相位 | step_id | 职业 | gate | builtin 档位 |
|---|---|---|---|---|
| 计划 | plan | advisor | auto | Max / 0.3 / 40 turns |
| 执行 | execute | coder | **Human**（计划确认门） | Max / 0.3 / 40 turns |
| 复审 | review | reviewer | auto | Pro / 0.2 / 50 turns |
| 沉淀 | document | assistant | auto | Mid / 0.3 / 20 turns |

- 流定义单源：hw `src/relay/flows.rs`（`plan_flow` / `plan_merge_flow`）；ag 轨
  经 `auto-src/relay_flows.at` a2r 再生成到 `src/auto_generated/relay_flows.rs`，
  双轨一致由 `relay_professions_souls_flows_hw_vs_ag`（tests/parity_relay_api）
  钉死。
- `plan-merge` flow：单相位 document（assistant），由 Chats
  `/auto-plan:merge PLAN-NNN` 触发，只做沉淀不重做执行/复审。
- deprecated 流（default/relay/simple/superpower）保持 PLAN-030 退役语义逐字
  不动，仅供对拍（parity 走 simple 流）。
- 档位随 builtin 角色分化（auto-ai-agent `builtin_roles/*.at`），由
  `plan_flow_professions_builtin_tier_matrix`（relay/flows.rs tests）直读
  钉死防漂移；角色解析序 = 用户 RoleRegistry（`~/.config/autoos/roles`）>
  builtin——部署级覆盖可临时改档（如 coder 40 turns 对超长 execute 相位不足时）。

## 相位输入（仅两样）

每相位 agent 的输入 = 相位模板（内嵌用户原话需求）+ 计划文件路径，此外无物：

- 相位模板：`relay/plan_flow.rs phase_task`，按目标职业校准口吻；纪律条目为
  模板固定内容——澄清-停止 / 幂等复用 / `PLAN_FILE:` 尾行协议 / TDD /
  阻塞入 §10 / 信代码不信勾选 / spec-impact 三字段硬性要求 / status 门禁 /
  **计划文件缺失硬失败**（PLAN-094：`{plan_file}` 双缺时不再是定位提示，
  而是阻断性缺陷条款——立即停止并输出 blocker，不得开始/继续本相位、
  不得正常完成本相位）。
- **prior handoff render 注入在 plan/plan-merge 退役**：`relay/driver.rs
  injects_handoff`（纯函数）按 flow_id 门控——plan/plan-merge 不注入，
  deprecated 流保留注入语义，未知 flow（run 消失）fail-open 走注入。
  ag factory（`extern_impl factory_build_agent`）委托 hw，单点改动双轨生效。
- 设计原则（用户裁定）：流程形状静态、每相位角色固定、传递内容固定；灵活性
  只保留 intake 路由与 Human gate 两处（对齐 auto-forge「流程过灵活」败因）。

## 计划文件机械传递（绑定 > 标记 > 硬失败）

计划文件路径经 run 上下文变量 `plan_file` 传递，写入优先级固定：

1. **绑定主通道**：`create_plan` 工具创建成功即写（`plan_tools.rs CreatePlan`
   持 ToolContext 装配的 RunStore + run_id；chat 会话调用 = 未命中 relay
   run，自然 no-op）。路径来源即工具写盘结果，零 AI 参与。
2. **标记回退**：相位输出尾行 `PLAN_FILE: <path>` 由驱动提取——
   `plan_flow.rs plan_file_marker_write(existing, output)` 守门：绑定已存在
   不覆盖；hw `run_step` 与 ag `drive_submit_handoff` 双驱动共用此单源函数。
3. **硬失败条款**（PLAN-094）：双缺时模板 `{plan_file}` 落阻断性缺陷条款
   （立即停止 + 输出 blocker），不再是"list_plans 自行定位"提示——UAT T3
   实录该提示曾放任 coder 空转假完成（K5 级联面）。

## execute 门前置不变式（PLAN-094，UAT K4/K5）

`plan` 流 execute 门（唯一 Human gate）上的 **approve 前置不变式**：run 上下文
`plan_file` 必须非空（绑定主通道或标记回退）。判定核单源
`plan_flow.rs execute_gate_action`（纯函数），hw/ag 双驱动与两个人口共用：

- **auto 臂**（会话 `approval_mode=auto`，`relay/driver.rs` 与 ag
  `auto_generated/relay_driver.rs` 的放行分支）：`plan_file` 缺失 → **不放行，
  直接 `fail_run`**，error =
  `plan phase ended without a plan file — restart the run (advisor reuses
  existing plans) or answer the advisor via reject+feedback in human mode`。
  有计划文件 → 照常放行（行为不变，AC-04）。
- **human 臂**（`POST …/gate`，hw `relay/api.rs` + ag
  `extern_impl relay_resolve_gate`）：`plan_file` 缺失 → 409 错误**不消费门**
  （提示改用 reject+feedback 或重开 run）。
- **D1 退化记录**（计划 PLAN-094 §10 预授权）：设计原文为"auto 缺失时转
  reject+feedback 定向重跑 plan 相位一次"。work 实测：引擎 redraft 语义
  （`PipelineEngine::resolve_gate` Reject → 重做**被门守卫的步骤**）只会重跑
  execute 相位，"重跑 plan 相位"在现引擎上不可表达（无 rewind API；让 coder
  带反馈补写计划又与模板硬失败条款及 K7"相位不做计划外发挥"冲突）→ 按预授权
  退化为直接 `run_failed`，仍满足"不放行"。恢复出路 = 重开 run（advisor 幂等
  复用既有计划）或 human 模式下 reject+feedback。
- **auto 放行审计（K4）**：auto 通过**任何** human 门时，审计事件/审计轮携带
  注入反馈文本（`RunEvent::GateResolved` 新增 `note` 字段，缺省
  `auto-approved; recorded defaults apply`；会话镜像轮正文
  `Gate <step> <decision> — <note>`，`conversation.rs`）。
- **门反馈送达（配套修复）**：`RunStore::step_context` 现消费引擎
  `feedback_for(step_id)`——reject(feedback) 重做相位时，反馈以「门反馈」块
  附加在相位模板之后（P2b.2 起该反馈从未被消费，定向反馈等同盲重放；此为
  前置缺陷的顺手修复）。已知边界：引擎 redraft 的目标是**被门守卫的相位**
  （execute），非 plan 相位——human 门的 reject+feedback 反馈到达 coder 而非
  advisor（引擎语义，见 D1 记录）。

## 路由唯一源 = transition_plan 状态机

相位间路由不消费任何 AI 摘要，只认 `plans.rs` 状态机：
drafting → executing → execution_done → reviewed；`merge_plan` 门禁 = 必须
reviewed，沉淀后置 archived（终态）；复审不过 → transition 回 executing
（run 正常结束，用户决定是否续跑修复）。计划页直达的 endpoint 式
merge/archive 操作不用于本工作流的收尾（canonical specs 先行、显式归档殿后）。

## 计划文件章节 = agent 间 API

正文 §0-§10（变更摘要 / 目标 / 架构方案 / 技术栈 / 需求分析与背景调查 /
详细设计 / 测试设计 / 验收标准 / 执行步骤 / 复审记录 / 待澄清事项）+
frontmatter（status / plan_revision / current_step / total_steps /
spec-impact 三字段）是四职业协作的唯一契约：advisor 产出 §0-§8，coder 消费
并回写（§8 勾选 + §10 阻塞），reviewer 回写（§9 + spec-impact），assistant
消费 status 门禁执行 merge。改章节结构属 breaking change，须独立计划。

## 产品内技能自带与轮次续跑（PLAN-092 SD-03/04）

**技能真源与分发**：auto-plan 四技能（new/work/review/merge）以仓内
`.agents/skills/` 为单一维护点（与 ZCode 侧同文件）；`musk serve` 启动
序列（`builtin_skills::sync_builtin_skills`）把四技能**幂等**同步到
`<config_dir>/skills/auto-plan-*/`（内容相同跳过、不同覆盖——真源单一
策略；源解析 `MUSK_SKILLS_DIR` env → CWD → 构建期仓根，缺源 warn 不阻断
serve）。产品 agent 侧无需额外拾取机制：`build_agent_from_mode` **每次
run 重建 agent 即重扫**技能目录（`MUSK_CONFIG_DIR` 覆盖时经
`autoos_skills_dir` 与分发目标单源），serve 同步的新技能下一次运行即可见。

**技能驱动的轮次续跑契约**：agent 经 skill 工具加载技能内容后，轮次必须
续跑至产出或显式报错——静默中断（无后续块、无 write、无错误）即缺陷。
技能流程中的**等待用户输入属正常暂停**（如 auto-plan:new 无设计文档时
问卷澄清），不算中断。产品内「/auto-plan:new 建计划」端到端须产出真实
`docs/plans/NNN-*.md`；计划落盘后计划栏/文件栏切换即可见（files 树端点
`/api/files/tree` 与 `/api/plans` 同源盘上数据）。诊断基建：aaid 对每笔
LLM 请求打最小观测行（入口 model/stream/candidates + 结果 elapsed/usage
+ 流式起止 + 候选链穷尽）——轮次时序归因不得再依赖会话文件考古。
（0927 实证教训：「trace 不完整」≠「run 中断」——先核落盘时机与观测面，
再定罪轮次状态。）

## 已知限制

- relay 工具面 = mode.tools 空 = 全量注册（plan 六件套 + 文件/命令 + orch），
  无 per-职业最小权限 enforcement（`professions.json` 的 allowed_tools 不在
  relay 路径消费）——最小权限另立计划。
- ~~relay factory `skills: false`：相位纪律内化于模板，不挂 skill 工具。~~
  **PLAN-096 修订**：纪律源=四技能快照机械注入相位任务（factory 仍
  skills:false 不挂 skill 工具，但模板内嵌技能全文+版本 hash；缺源硬失败）
  ——执行合同详见 `plan-execution-contract.md`。
- 冒烟取证注意：GET run 的事件为 500 条窗口视图，早期事件（含 GateWaiting /
  首相位工具调用）会被挤出，全量证据需落盘 `/events` 或提前快照（PLAN-086
  F-R1）；coder 40 turns 对超长 execute 相位的充分性仍为运行观察项。

## PLAN-096 修订（受控执行合同，2026-10-02）

本节取代上文与 PLAN-096 冲突的行为描述；未冲突部分（流程形状/固定职业/
门位置/handoff 注入门控/计划文件机械传递）继续有效。

- **相位输入**：模板 = 固定职业 + 技能全文快照（`plan_contract::
  snapshot_plan_skills`，缺源硬失败）+ 机械输入（需求/plan_file/绑定事实）
  + 结果提交协议（`complete_plan_stage`）。phase→skill 固定映射：
  plan→auto-plan-new、execute→auto-plan-work、review→auto-plan-review、
  document→auto-plan-merge；execute 另注入 Canvas 生成指引（原
  `mode.name=="coding"` 专享条件补齐）。plan 相位是计划创建者——不适用
  plan_file 缺失阻断条款（该条款仅约束 execute/review/document）。
- **路由唯一源**：~~transition_plan 状态机~~ → `PlanControl::on_stage_end`
  单写者路由（阶段结果+服务器事实核验），计划状态机推进成为路由的
  **结果**而非输入；transition_plan/merge_plan 工具对受管计划拒绝（409）。
  review 失败不再"run 结束等用户"——needs_fix 自动回退（≤3 轮+无进展
  早停+findings 注入），needs_replan/blocked 响亮停止。细节见
  `plan-execution-contract.md`。
- **沉淀**：~~`merge_plan` 章节复制+立即归档~~ → `plan_delivery` 五检查点
  （prepare/land/refresh/archive/cleanup，canonical 先合入、账本经 store
  刷新、显式归档殿后、清理失败=cleanup_pending）。SD-03。
- **execute 门**：approve 即冻结批准绑定（合同/Git 事实/worktree 租约）
  并推进计划 drafting→executing；authorization=auto 桥接 approval_mode
  （会话 auto 语义等价；REST 显式授权同路径）。
- **执行作用域**：execute/review/document 相位文件/命令/Canvas 工具落
  run 的开发 worktree（execution_root）；主检出代码不注册为可写根。SD-04。

## 运行遥测（PLAN-098，2026-10-03）

四阶段 run 的 turn 产物满足 `modules/run-telemetry.md` 记录契约（引用而非
复制规则）：每相位收束（`StepCompleted` 边界轮）携带该相位聚合遥测，
`correlation` 含 run_id/step_id/role；聊天会话收束轮同契约。aaid 日志仅为
旁证（daemon 无关联键，时间窗启发式 join 并标注置信）；重跑对比以
turns.jsonl 内嵌遥测与 `collect-telemetry.mjs` 摘要为权威记录物——
RUNBOOK §4 第 0 步（每 run 产出摘要并在结果总表/历史记录引用其路径）是
落账前置。失败签名自动标记（K1/K3/K4/K5）与逐请求 provider/model/token/
耗时表见 run-telemetry.md 与 RUNBOOK。
