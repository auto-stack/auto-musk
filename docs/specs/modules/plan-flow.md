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
  阻塞入 §10 / 信代码不信勾选 / spec-impact 三字段硬性要求 / status 门禁。
- **prior handoff render 注入在 plan/plan-merge 退役**：`relay/driver.rs
  injects_handoff`（纯函数）按 flow_id 门控——plan/plan-merge 不注入，
  deprecated 流保留注入语义，未知 flow（run 消失）fail-open 走注入。
  ag factory（`extern_impl factory_build_agent`）委托 hw，单点改动双轨生效。
- 设计原则（用户裁定）：流程形状静态、每相位角色固定、传递内容固定；灵活性
  只保留 intake 路由与 Human gate 两处（对齐 auto-forge「流程过灵活」败因）。

## 计划文件机械传递（绑定 > 标记 > hint）

计划文件路径经 run 上下文变量 `plan_file` 传递，写入优先级固定：

1. **绑定主通道**：`create_plan` 工具创建成功即写（`plan_tools.rs CreatePlan`
   持 ToolContext 装配的 RunStore + run_id；chat 会话调用 = 未命中 relay
   run，自然 no-op）。路径来源即工具写盘结果，零 AI 参与。
2. **标记回退**：相位输出尾行 `PLAN_FILE: <path>` 由驱动提取——
   `plan_flow.rs plan_file_marker_write(existing, output)` 守门：绑定已存在
   不覆盖；hw `run_step` 与 ag `drive_submit_handoff` 双驱动共用此单源函数。
3. **降级提示**：双缺时模板 `{plan_file}` 落 list_plans 定位提示。

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
- relay factory `skills: false`：相位纪律内化于模板，不挂 skill 工具。
- 冒烟取证注意：GET run 的事件为 500 条窗口视图，早期事件（含 GateWaiting /
  首相位工具调用）会被挤出，全量证据需落盘 `/events` 或提前快照（PLAN-086
  F-R1）；coder 40 turns 对超长 execute 相位的充分性仍为运行观察项。
