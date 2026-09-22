---
plan_id: PLAN-086
status: archived
feature_name: plan flow 固定四角色相位流 + 相位间机械传递改造
author: [agent]
created_at: 2026-09-22T21:50:00+08:00
updated_at: 2026-09-23T01:10:00+08:00
completion_kind: delivered
plan_revision: 1
current_step: 8
total_steps: 8
supersedes_spec_components:
  - "relay/plan flow 单角色规则：PLAN-030 落地的「4 步全 plan-dev」约定（仅存于 ledger 与代码，docs/specs 未成文；由新模块 plan-flow.md 的 before 栏收录改写）"
new_spec_components:
  - "docs/specs/modules/plan-flow.md（新增）：固定四角色相位契约 + 计划文件唯一载体 + 类型化绑定机械传递 + 状态机唯一路由源 + handoff 注入于 plan/plan-merge 退役"
touched_goals: []
---

# [PLAN-086] plan flow 固定四角色相位流 + 相位间机械传递改造

## 0. 变更摘要

`plan` flow 从「单一 plan-dev 角色跑满四相位」改为**固定职业四相位**：
plan=advisor（写计划）、execute=coder（执行）、review=reviewer（复审）、
document=assistant（触发与 merge）。同时做三项配套改造：

1. **handoff 注入按 flow 级关闭**：plan/plan-merge 两流相位间不再注入
   prior handoff render（AI 生成摘要通道退役），deprecated 流
   （default/relay/simple/superpower）保留原语义供对拍。
2. **create_plan 类型化绑定替代 PLAN_FILE 正则主通道**：create_plan 工具
   在 relay run 内调用时经 ToolContext.parent_conversation_id(=run_id) 把
   计划文件路径写入 RunStore 上下文；step_context 组装时**绑定优先、
   PLAN_FILE 标记降级为回退、双缺时降级定位提示**。相位文件传递全程
   无 AI 参与。
3. **路由信号唯一化**：流程路由只认 transition_plan 状态机（reviewed 才
   进 document、非 reviewed 拒绝 merge），无任何 AI 摘要参与路由。

模型档位随 builtin 角色自动分化：advisor=Max/0.3/40、coder=Max/0.3/40、
reviewer=**Pro/0.2/50**（与计划作者异档=真模型多样性）、assistant=**Mid/
0.3/20**（merge 降档省钱），替代现行 plan-dev 全程 Max/120。

设计契约（用户裁定，2026-09-22 会话）：流程形状静态、每相位角色固定、
传递内容固定；灵活性只保留 intake 路由与 Human gate 两处；计划文件为
唯一交接载体，其章节结构（0-10 + frontmatter）按 agent 间 API 对待。

## 1. 目标

1. `plan` flow 四步 role_id 固定为 advisor/coder/reviewer/assistant（hw
   `flows.rs` + ag `auto-src/relay_flows.at` 双轨一致），deprecated flow
   定义零变更。
2. plan/plan-merge 相位输入 = 相位模板（内嵌用户原话需求）+ 计划文件路径，
   仅此两样；handoff 注入关闭且 deprecated 流行为不变。
3. create_plan 工具期绑定 run→plan 文件；引擎按 绑定 > PLAN_FILE 标记 >
   定位提示 的顺序组装；三相臂单测覆盖。
4. merge 门禁与 review 失败回环语义（status 回 executing）不回归。
5. 四相位 model tier 分化生效（builtin 直读验证）。

### 非目标

- **relay 侧工具最小权限 enforcement**：professions.json 的 allowed_tools
  现不在 relay 路径强制（factory 传空 mode.tools=全量注册，见 §4 证据），
  本轮不引入 enforcement，另立后续。
- **relay 相位挂 skill 工具**：factory `skills:false` 维持，四相位纪律
  继续内化于模板（讨论已裁定分两步走，本轮只做角色+档位+传递）。
- TaskPlan/DAG、auto-ai 仓改动（factory 为 musk 侧单点，无需动依赖仓）。
- 前端 UI 改动（RelayRunBox 按步骤显示职业若已可用则零改动，T-08 核实）。

## 2. 架构方案

### 2.1 现状 → 目标映射

| 维度 | 现状（PLAN-030） | 目标（本轮） |
|:---|:---|:---|
| 四相位角色 | 全 plan-dev（Max/120） | advisor/coder/reviewer/assistant（Max/Max/Pro/Mid） |
| 相位间信息 | handoff render 注入首条 user history + 计划文件 | 仅计划文件（引擎机械传递） |
| 计划文件路径获取 | 正则提取 agent 输出 `PLAN_FILE:` 标记 | create_plan 工具期绑定优先，标记回退 |
| 路由信号 | transition_plan 状态机 | 不变，且成为唯一源（handoff 通道退役） |
| 每相位上下文 | 已是逐相位新 agent（factory 每步新建） | 不变 |
| 相位工具 | mode.tools 空=全量（base+orch+plan 六件套） | 不变（无工具面变更） |

### 2.2 handoff 关停的实现形状（musk 侧单点）

`MuskAgentFactory::build_agent`（`relay/driver.rs:48`）已持有
`state + workspace_id + run_id`：查 RunStore 得该 run 的 flow_id，
`plan | plan-merge` 白名单命中则跳过 `prior_handoff.render()` 注入分支；
未命中走原路径。ag 侧 factory（`auto_generated/extern_impl.rs:1305`
factory_build_agent）**委托 hw factory**，故单点改动双轨生效。
run 终报回写（`relay_append_report_message_to`，extern_impl.rs:1248）
数据源为 `ws.relay.run_report(run_id)`，与逐相位注入无关，不受影响。

### 2.3 绑定通道的实现形状

create_plan 工具经 `from_ctx` 持有 ToolContext（state/workspace_id/
parent_conversation_id=run_id）。工具成功创建计划后：
`state.registry.get(ws).relay.set_context_var(run_id, "plan_file", path)`，
仅当 parent_conversation_id 命中活跃 run 时写入（chat 会话调用=自然 no-op）。
driver 侧 PLAN_FILE 标记提取改为**绑定已存在则不覆盖**。step_context 组装
顺序：绑定 > 标记 > locate-hint（现有降级文案保留）。

### 2.4 计划文件 = agent 间 API

章节 0-10 + frontmatter（status/plan_revision 等）是四角色协作的唯一
契约：advisor 产出（§0-8）、coder 消费并回写（§8 勾选+§10 阻塞）、
reviewer 回写（§9+spec-impact 三字段）、assistant 消费（merge_plan 读
status 门禁）。改章节结构属 breaking change，需独立计划。

## 3. 技术栈

- Rust（musk crate，backend/ workspace；cargo 命令一律在 `backend/` 下跑）。
- 双轨维护：hw `src/relay/flows.rs` + ag `auto-src/relay_flows.at`（a2r
  转译产出 `auto_generated/relay_flows.rs`，改 .at 后走既有再生成链）。
- auto-ai-agent 依赖零改动（builtin 角色直读即可）。

## 4. 需求分析与背景调查

### 4.1 已获授权记录

- 用户于 2026-09-22 会话中逐项裁定：①四角色定型（接受 reviewer 替代
  advisor 复审的修正；assistant 负责发起与 merge）；②「handoff 机制直接
  去掉，flow 引擎自己管理流程文件、机械传递」；③「流程固定、每阶段角色
  固定、传递内容固定」为设计原则，避免 auto-forge 流程过灵活的败因。
- 本轮授权范围：auto-musk 仓 backend relay/plan flow 改造 + 测试 + 冒烟。
  未授权：auto-ai 仓改动、工具权限 enforcement、skill 挂载。
- 未指定预算与自动续跑限额。

### 4.2 背景事实（已核验，含出处）

| 事实 | 出处 |
|:---|:---|
| plan flow 定义：4 步全 plan-dev，execute 前 Human gate | `backend/crates/musk/src/relay/flows.rs:32-37`；ag 镜像 `auto-src/relay_flows.at` / `auto_generated/relay_flows.rs:13-16` |
| factory 每步新建一次性 agent；handoff render 注入首条 user history | `src/relay/driver.rs:48-76` |
| factory 传 `tools: Vec::new()`、`skills:false`；ag factory 委托 hw | `driver.rs:53`、`auto_generated/extern_impl.rs:1302-1306` |
| 工具注册按 mode.tools 白名单（空=全量）；plan 六件套经 build_agent_with_context 无条件注册（chat+relay 同路径） | `src/lib.rs:147-235, 276-340` |
| PLAN_FILE 正则提取 + {plan_file} 三级降级组装 | `src/relay/plan_flow.rs:11-50` |
| run 终报数据源=run_report，与注入无关 | `auto_generated/extern_impl.rs:1248-1290` |
| parity 对拍用 simple 流且断言 handoff 摘要 | `backend/crates/musk/tests/parity_relay_driver.rs` |
| builtin 档位：advisor Max/0.3/40、coder Max/0.3/40、reviewer Pro/0.2/50、assistant Mid/0.3/20、plan-dev Max/0.3/120 | `auto-ai/crates/auto-ai-agent/src/builtin_roles/*.at` |
| professions.json（~/.config/autoos）allowed_tools 不被 relay 路径消费 | `src/lib.rs` 注册逻辑 + factory 空白名单 |
| PLAN-030 原文明示单角色系「初期」安排 | `docs/plans/archived/030-plan-driven-dev-flow.md` §1 |
| 环境注记：本机 `~/.config/autoos/roles/assistant.at` 覆盖（tier=min、persona "vm e2e edited"）将命中 document 相位 | 实机配置（2026-09-22 检视） |

### 4.3 风险

- coder builtin 40 turns 可能低于长 execute 相位需求（plan-dev 原为 120）
  → T-08 冒烟观察项；不足时部署级 roles 覆盖缓解（见 §10-1）。
- 本机 assistant.at 覆盖影响 dev 环境 document 相位档位/人格 → §10-2。
- flows.rs 现有单测断言「四步同角色 plan-dev」需同步改写，属预期红区。

## 5. 详细设计

### 5.1 flows 四角色替换（T-01）

`flows.rs` `plan_flow()`：`plan→advisor`、`execute→coder`（保留
`.with_gate(Human)`）、`review→reviewer`、`document→assistant`；
`plan_merge_flow()` document 步 `plan-dev→assistant`。ag 侧同步改
`auto-src/relay_flows.at` 后走既有 a2r 再生成链产出
`auto_generated/relay_flows.rs`。单测 `plan_flow_is_four_same_role_steps_
with_one_human_gate` 改写为 per-step 角色断言（gate 位置断言保留），
plan-merge 单测同步。

### 5.2 factory flow 级 handoff 开关（T-02）

`build_agent` 内：`let flow_id = state.registry.get(&workspace_id).relay.flow_of(&run_id)`（RunStore 现有查询能力，若仅能经 RunState 取则顺路补查询函数）；
`matches!(flow_id.as_deref(), Some("plan") | Some("plan-merge"))` 时跳过
注入分支。开关逻辑抽独立函数便于单测（flow_id → 是否注入）。

### 5.3 create_plan 绑定 + 组装优先级（T-03）

- `plan_tools.rs CreatePlan`：创建成功后按 §2.3 写 `plan_file` 上下文变量
  （活跃 run 命中才写）。路径来源即工具自身写盘结果，零 AI 参与。
- driver 提取侧：绑定已存在则跳过标记写入（避免回退覆盖主通道）。
- `step_context`/`phase_task`：绑定 > 标记 > hint 三臂语义不变，仅取值
  顺序固化并测试化。

### 5.4 模板职业化措辞（T-04）

`plan_flow.rs` 四段模板逐段按目标职业校准：plan 模板面向 advisor（澄清
或起草、编号章节、原子任务、PLAN_FILE 尾行协议保留）；execute 面向
coder（计划唯一上下文、逐任务+验证命令、TDD、阻塞入 §10）；review 面向
reviewer（信代码不信勾选、逐 AC 对照 file:line、填 §9+spec-impact、不过
则 transition 回 executing）；document 面向 assistant（status 门禁核对、
merge_plan 机械沉淀、更新 docs/specs 模块树、归档）。纪律条目不增删，
仅口吻与职责称呼校准。

### 5.5 档位分化验证（T-05）

单测直读四个 builtin 角色 model_tier/temperature/max_turns 并断言
（防 auto-ai 侧未来漂移）；在测试注释与本计划记录本机 roles 覆盖的影响
（document 相位 dev 环境为 min 档）。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|:---|:---|:---|:---|:---|:---|
| SD-01 | add | docs/specs/modules/plan-flow.md（新） | before：PLAN-030「4 步全 plan-dev 单角色，handoff render 注入 + PLAN_FILE 正则为主通道」（ledger 在案，specs 未成文）。after：固定 advisor/coder/reviewer/assistant 四相位；相位输入=模板+计划文件；create_plan 绑定为主通道、标记回退；路由唯一源=transition_plan 状态机；handoff 注入在 plan/plan-merge 退役、deprecated 流保留；章节 0-10 为 agent 间 API | 多模型特长分档（reviewer 异档真独立、merge 降档省本）；机械化传递消除 AI 幻觉面；固定流程防 auto-forge 灵活性败因复发 | AC-01..AC-06 |

## 6. 测试设计

- **单测（cargo lib）**：flows per-step 角色断言；factory 开关函数
  （plan/plan-merge 不注入、simple/default 注入）；create_plan 绑定三臂
  （绑定优先/无绑定回退标记/双缺 hint）；merge_plan 非 reviewed 拒绝
  （回归）；review 失败 transition 回 executing（回归）；builtin 四角色
  档位断言。
- **parity**：parity_relay_driver/parity_relay_api 全绿（simple 流
  handoff 对拍面不变；ag 委托 hw 单点改动）。
- **端到端冒烟（T-08）**：HTTP API 发起 `spawn_relay(flow_id="plan")`
  简单需求 → 走完 plan→gate 批准→execute→review→document，核对：四步
  profession_id、各相位输入无 handoff 摘要、绑定生效（run 上下文含
  plan_file）、merge 后 status=archived、报告回写正常。

## 7. 验收标准

- **AC-01 四相位固定角色**：hw 与 ag 双轨 `plan` flow 四步 role_id 依次
  = advisor/coder/reviewer/assistant（execute 步保留 Human gate），
  deprecated flow 定义逐字节不变。验证：`cd backend && cargo test
  relay::flows` per-step 断言绿 + `git diff` 确认 default/relay/simple/
  superpower 流未动。
- **AC-02 handoff 按 flow 退役**：plan/plan-merge 相位 agent 初始 history
  不含 prior handoff render；simple 流仍注入。验证：factory 开关单测
  双向断言 + `cargo test --test parity_relay_driver --test
  parity_relay_api` 全绿。
- **AC-03 机械绑定主通道**：create_plan 在活跃 run 内调用后 run 上下文
  含 plan_file 且 step_context 以其组装；无绑定时标记回退仍工作；双缺
  时降级 hint。验证：三臂单测 + T-08 冒烟中 run 上下文实证。
- **AC-04 状态机唯一路由不回归**：merge_plan 对非 reviewed 拒绝、
  review 失败回 executing 语义不变。验证：既有/新增回归单测绿。
- **AC-05 档位分化**：四 builtin 角色档位断言（advisor/coder=Max、
  reviewer=Pro、assistant=Mid）绿；dev 环境覆盖影响已记录于计划文本。
- **AC-06 端到端冒烟**：实跑一轮 plan flow 全程通过，四步各自职业身份
  可见于 run 记录，gate 生效，document 相位 merge 归档成功。验证：T-08
  冒烟产物（run 记录/日志摘录）入 §9 证据。

## 8. 执行步骤

- [x] T-01 flows 四角色替换（AC-01）[✅ 已完成]
  files：`src/relay/flows.rs`、`auto-src/relay_flows.at`（→再生成
  `src/auto_generated/relay_flows.rs`）、flows.rs 单测改写。
  verify：`cd backend && cargo test relay::flows`。
  证据：commit 9e37301；`cargo test --lib relay::flows` 3 passed
  （plan_flow_is_four_fixed_role_steps_with_one_human_gate 等）；deprecated
  四流 diff 核对零触碰；ag 侧 `.at` 补入 plan/plan-merge 流（此前仅手工
  存在于生成文件）后 a2r 再生成落位（Loop struct 语法与 `for f in flows`
  迭代形式保持手修定式，a2r 新版对 hetero 变体/迭代的产出漂移在案）。
- [x] T-02 factory flow 级 handoff 开关（AC-02，依赖：无）[✅ 已完成]
  files：`src/relay/driver.rs`（开关抽函数 + build_agent 接线）、
  `src/relay/store.rs`（flow_of）。
  verify：`cd backend && cargo test relay::driver`。
  证据：commit dbc3ab4；`handoff_injection_is_flow_gated`（六 flow 双向
  断言 + None fail-open）+ `flow_of_resolves_run_flow_id` 绿。
- [x] T-03 create_plan 绑定 + 组装优先级（AC-03，依赖：无）[✅ 已完成]
  files：`src/plan_tools.rs`、`src/relay/driver.rs`（提取侧不覆盖）、
  `src/relay/plan_flow.rs`（守门纯函数）、`src/auto_generated/extern_impl.rs`
  （ag 驱动补齐既有缺提取缺口，守门单源）。
  verify：`cd backend && cargo test plan_tools relay::plan_flow`。
  证据：commit 9f7c252；create_plan_binding_writes_active_run_context /
  create_plan_binding_noop_for_non_run_session / marker_fallback_never_
  overwrites_binding 绿；AC-04 回归面（merge_plan_gates_on_reviewed_and_
  deposits、transition 校验）同批绿。
- [x] T-04 模板职业化措辞（AC-01/AC-06，依赖：T-01）[✅ 已完成]
  files：`src/relay/plan_flow.rs` 四段模板。
  verify：`cd backend && cargo test relay::plan_flow`。
  证据：commit 01a2b33；9 tests passed（新增
  templates_voice_the_fixed_professions：四相位点名对应职业 + plan-dev
  残留反断言；纪律条目零增删）。
- [x] T-05 档位分化断言（AC-05，依赖：T-01）[✅ 已完成]
  files：flows.rs 测试模块新增 plan_flow_professions_builtin_tier_matrix。
  verify：`cd backend && cargo test`。
  证据：commit 9ce7c21；直读 load_builtin 四角色 Max/Max/Pro/Mid +
  temperature/max_turns 断言绿（绕过用户 RoleRegistry，防漂移钉死）。
- [x] T-06 parity 双轨对拍回归（AC-02，依赖：T-01/T-02）[✅ 已完成]
  verify：`cd backend && cargo test --test parity_relay_driver --test
  parity_relay_api`。
  证据：parity_relay_driver 4 passed + parity_relay_api 6 passed 1
  ignored（含 relay_professions_souls_flows_hw_vs_ag 双轨 flows 一致、
  parity_drive_run_simple_flow_matches_hw 的 simple 流 handoff 对拍面）。
- [x] T-07 全量绿（依赖：T-01..T-06）[✅ 已完成]
  verify：`cd backend && cargo build && cargo test`。
  证据：build 0 error；全量 667 passed / 1 failed——唯一红
  tool_atoms::run_command_dangerous_returns_paused 经基线比对实锤为
  540194d 既有红（PLAN-084 交接在案，沙箱根解析问题），与本计划无关。
  Cargo.lock +97/-2 登记（commit 6d4134a，组内依赖固定位的机械再生）。
- [x] T-08 端到端冒烟（AC-06，依赖：T-07）[✅ 已完成]
  动作：worktree debug 构建 serve（端口 18086）+ 隔离
  MUSK_CONFIG_DIR，HTTP 发起 plan flow 简单需求，gate 批准后
  plan→execute→review→document 走完。
  证据：run-1790093669-341d94ec——step_history 四职业链
  advisor=>coder=>reviewer=>assistant + 四职业 token 分账
  （advisor 3970/coder 4262/reviewer 5868/assistant 3576）；execute 前
  Human gate 等待并 POST approve 放行；create_plan 绑定在真实 run 内
  落 plan_file（coder/reviewer/assistant 按 seq=1 消费同一计划）；
  merge_plan 门禁通过（reviewed 才可达）→ 计划归档至
  docs/plans/archived/001-*.md（frontmatter status: archived、
  spec-impact 三字段由 reviewer 填写）；产物 hello-p086.txt 内容精确
  命中；emit_report 报告落 .autoos/reports/run-*/report.html + 
  run_completed 事件携带 report；serve 日志 0 error/panic。
  附件：docs/plans/attachments/p086-smoke-final-run.json、
  p086-smoke-serve.log。live serve 走 ag 驱动
  （auto_generated::relay_driver::drive_run），本次改动 ag 侧
  （factory 委托 + drive_submit_handoff 提取）被直接锻炼。
  注意项处置：§10-2 assistant.at 覆盖冒烟期间临时移出、完毕即还原；
  首次冒烟曾误用共享 workspace 注册表（见 §10-5），已隔离重启并
  清除生产 store 内的计划残留（backend/docs/plans/001-*.md 已删，
  store 复核仅剩 archived/）。

## 9. 复审记录

- 2026-09-22 /auto-plan:new 起草交接：stage=new，plan_revision=1，
  outcome=pass（授权范围内可执行），next=work（T-01..T-08）。
  起草前事实核验完成（§4.2 十一项出处）；两个环境级决策点已隔离至 §10
  不阻塞执行。
- 2026-09-22 /auto-plan:work 交接：stage=work，plan_revision=1，
  outcome=pass，code_commit=plan-086-dev@9ce7c21（T-01..T-05 +
  lock 登记 6d4134a，六 commit 全部落地），task_ids=T-01..T-08 全清。
  evidence：各任务行内证据 + T-08 冒烟附件
  （attachments/p086-smoke-final-run.json / p086-smoke-serve.log）。
  blockers：无阻塞（三项环境观察登记 §10-4/5/6，不阻塞 review）。
  next=review。worktree：D:/autostack/.wt/musk-086/auto-musk
  （组内只读依赖位 auto-ai@57eb44a、auto-lang@641e1b9f4，merge 时随组清理）。
- 2026-09-23 /auto-plan:review 定案：stage=review，plan_id=PLAN-086，
  plan_revision=1，outcome=pass，reviewed_commit=6d4134a5030f9214f0992
  cdd8eedb0f69e0e68c0，base_commit=540194d，dependency_revisions=
  auto-ai@57eb44a（detached）/auto-lang@641e1b9f4（detached，master
  61ccf23fc 破损退避，见 §10-5②），spec_inputs=docs/specs/modules/
  README.md 现行树（SD-01 目标 plan-flow.md 为新增，无冲突组件）。
  独立性限制声明：与 work 同会话复核，全部证据为本 review 会话新鲜
  重跑命令与持久化制品，不消费执行期结论。
  acceptance_results（全部 pass，均本会话重放）：
  - AC-01 pass——`cargo test --lib relay::flows` 4 绿
    （plan_flow_is_four_fixed_role_steps_with_one_human_gate 逐相位
    advisor/coder/reviewer/assistant + execute Human gate；diff 检查
    deprecated 四流 FlowStep 行零触碰）。
  - AC-02 pass——`handoff_injection_is_flow_gated` 绿（plan/plan-merge
    不注入、deprecated 四流+None fail-open 注入）；parity_relay_driver
    4 绿（simple 流 handoff 对拍面 parity_drive_run_simple_flow_matches_hw）
    + parity_relay_api 6 绿 1 ignored（预存手动门）。
  - AC-03 pass——三臂单测绿（create_plan_binding_writes_active_run_
    context / create_plan_binding_noop_for_non_run_session /
    marker_fallback_never_overwrites_binding）；冒烟真机消费链实证：
    advisor create_plan → coder/reviewer/assistant 按 seq=1 消费同一
    计划（run-1790093669-341d94ec，附件 JSON）。
  - AC-04 pass——merge_plan_gates_on_reviewed_and_deposits +
    transition_plan_validates_and_hints_legal_targets 绿（非 reviewed
    拒绝、状态机回环语义未动）。
  - AC-05 pass——plan_flow_professions_builtin_tier_matrix 绿
    （Max/Max/Pro/Mid + 0.3/0.3/0.2/0.3 + 40/40/50/20 直读断言）。
  - AC-06 pass——冒烟四职业 step_history 链 advisor=>coder=>reviewer=>
    assistant + 四职业 token 分账（3970/4262/5868/3576）；gate 放行
    以 serve 日志附件实锤（16:16:03 POST /gate 200，gate 前停车
    90s）；merge_plan 归档（archived/001-*.md frontmatter status:
    archived，spec-impact 三字段 reviewer 已填）；产物 hello-p086.txt
    内容精确；emit_report + run_completed report 事件在案；serve 日志
    0 error/panic。全量门禁 `cargo build && cargo test`：0 error，
    667 passed / 1 failed（唯一红 tool_atoms::run_command_dangerous_
    returns_paused = base 540194d 既有红，本 review 重跑同点复现，
    与 PLAN-086 无涉）。
  findings（均非阻断，无需回 work）：
  - F-R1（info）：冒烟附件 JSON 的 run 事件为 API 500 条窗口视图，
    早期事件（GateWaiting/advisor 工具调用）被窗口挤出——gate 证据已
    从 serve 日志附件恢复；后续冒烟取证建议落盘 /events 全量或提前
    快照。不改代码。
  - F-R2（info）：a2r 转译器产出漂移两处（hetero 变体 ExitRouting::Loop
    产元组语法不可编译；`for f in &flows` 与 `Some(f)` 所有权不兼容）
    ——auto_generated 落位时按既有手修定式矫正（本分支 diff 在案），
    未来再生成同点位需复检。auto-lang 侧改进候选，另立。
  - F-R3（info）：spec-impact 收口说明——touched_goals=[] 依据：本轮
    为 relay 流程编排内改造（flows/driver/plan_tools/plan_flow），不
    触及任何 Goal 级 spec 条目（docs/specs 现行树 grep 无 plan flow
    单角色描述组件，01-architecture.md:58 的 human gate 机制描述与
    本轮后行为一致仍准确），故无 goal 级增删改。
  spec delta 复核：SD-01 add docs/specs/modules/plan-flow.md——目标
  路径有效；before 栏（PLAN-030 单角色+handoff 注入+PLAN_FILE 正则
  主通道）与 base 代码史实一致（本分支 diff 即物证）；after 栏与
  重放后行为逐条吻合；章节 0-10=agent 间 API、绑定>标记>hint、状态机
  唯一路由均已被单测钉死。冻结：本记录 + 附件即 delta 证据包。
  evidence：本记录内命令与结果 + docs/plans/attachments/
  p086-smoke-final-run.json / p086-smoke-serve.log.txt（主检出持久）。
  next=merge（whole-workflow 已授权，review 通过即转入 merge）。
- 2026-09-23 merge（plan_revision 1，**pass — delivered**）：
  - stage: merge | PLAN-086 | r1 | **pass — delivered**
  - checkpoints:
    - prepared：reviewed 基线=6d4134a（依赖位 auto-ai@57eb44a /
      auto-lang@641e1b9f4 detached）；canonical 增量三件在 worktree 备置并
      提交——docs/specs/modules/plan-flow.md 新建（SD-01 after 栏全文，
      流程形状/相位输入/机械传递/唯一路由/agent 间 API/已知限制六节）+
      docs/specs/index.json（spec_files 增 modules/plan-flow.md，
      updated_at 2026-09-23）+ docs/plans/KNOWN-DEBT-AND-RISKS.md 086 行
      （a2r 再生成接缝两处手修定式 + F-R1 取证窗口指针）；diff vs
      reviewed commit 核对=纯 docs 3 文件（documentation-only descendant，
      实现/依赖零变更，取得 delivery 资格）。ledger 目标工作区=仓库根
      .autoos/specs.json（运行时账本，untracked，六区全空=musk 既有惯例，
      知识单源走 docs/specs/ + KNOWN-DEBT，本轮不写 specs.json）。
    - landed：main ff-only → **5e6ea765**（7×commit；rebase 到 58862d0
      后 range-diff **6/6 全等**，old→new 映射 9e37301→d1d3807 /
      dbc3ab4→67535a8 / 9f7c252→0fce4b9 / 01a2b33→8a264dc /
      9ce7c21→c79b6bd / 6d4134a→e120b2a，第 7 commit=docs 沉淀
      5e6ea76 即 delivery commit）；无 merge commit。主检出让路协议：
      在途 backend/Cargo.lock WIP（+3606 构建再生产物，归属不明）stash
      留档（stash@{0}，未回灌——内容可由任意构建再生），ff-only 无阻。
    - ledger_refreshed：docs/specs/index.json + KNOWN-DEBT 086 行随
      5e6ea76 落地主检出，读回核验在位（index 解析合法、条目在列、
      plan-flow.md 八节全文在位）；运行时 .autoos/specs.json 零写入。
    - archived：docs/plans/archived/086-plan-flow-fixed-roles.md，
      status: archived, completion_kind: delivered（见本记录）。
    - cleaned：（见下）。
  - 落地后门禁（主检出已知良好）——**部分受阻（外部漂移，非本轮引入）**：
    主检出 cargo build 红 11 错全在第三方 crates.io 包 wgpu-hal 27.0.4
    （windows 接口不兼容）；实证与本 delta 无涉——wgpu-hal 在本轮提交的
    lock、分支 worktree lock、落地前 main lock 中均为 0 条，构建时 cargo
    对 auto-lang **master**（主检出路径解析，db3fd4a42，比 086 固定位
    641e1b9f4 新多笔，含他 session 在途 PLAN-041 系）重解析 +3541 行才
    拉入；musk 代码树与全绿分支（667 passed）经 ff-only+range-diff 6/6
    等价同源。**解锁动作=auto-lang master 依赖面稳定后重跑主检出
    build+test；或临时 pin 后重建**（生产桌面 17201 跑旧二进制不受影响；
    部署重建暂缓至 auto-lang master 稳定，登记 KNOWN-DEBT 080 行防再发
    协议已覆盖启动面）。
  - evidence：本记录 checkpoints 内命令与结果；review 证据包（§9 review
    条目 + attachments/p086-smoke-*.json/.txt）随归档路径持续可解析。

## 10. 待澄清事项

- [ ] coder builtin 40 turns 对长 execute 相位是否足够：T-08 冒烟观察——
  极小任务下 coder 4262 tokens 收敛良好，40 turns 充足；长 execute 相位
  仍是观察项，不足时先以部署级 `~/.config/autoos/roles/coder.at` 覆盖
  缓解，auto-ai 侧调整（若需）另立计划。
- [x] 本机 `~/.config/autoos/roles/assistant.at`（tier=min、"vm e2e
  edited" 人格）：冒烟期间临时移出为 `assistant.at.p086-smoke`，冒烟
  完毕已原位还原（document 相位以 builtin Mid 档实跑）。后续 dev 环境
  若复跑冒烟需同样临时移出。
- [ ] RelayRunBox 步骤职业显示是否开箱可用：冒烟 serve 未构建前端
  （web dist 缺席，HTTP API 面），未核实；不立项不改前端，留待有前端
  的桌面环境顺带核实。
- [x] relay 侧 allowed_tools 最小权限 enforcement 与 skill 挂载：本轮
  非目标，未来另立计划（§1 非目标已声明，维持）。
- [ ] 环境债（冒烟插曲，非本计划处置）：① 共享 workspace 注册表
  `~/.config/autoos/workspaces.json` 机器级共用——serve 冒烟若不隔离
  会把 default workspace 解析到生产桌面登记的 root（本次实证写穿到
  主检出 backend/docs/plans，已清理并用 MUSK_CONFIG_DIR 隔离重跑；
  PLAN-080 T-1 隔离门即为此设）；② auto-lang master@61ccf23fc 的
  crates/auto-lang/Cargo.toml 存在重复 `iced` 键（9ed0da75d 引入，
  任何消费者 cargo 加载即炸）——086 组固定位退避 641e1b9f4，待
  auto-lang 侧修复；③ 主检出 backend/Cargo.lock 在途脏 WIP（+3606，
  构建再生产物，归属不明）——merge 阶段与分支 lock 登记一并处置。
