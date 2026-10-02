# Plan 流程对话驱动 UAT 测试说明

目的：可重复地验证 auto-musk 两条 plan 流程路径在对话驱动下的真实行为，并固化判定标准与已知问题对策。
本文记录 2026-09-28/29 两轮实测结果；重跑时按"复现步骤"重置环境后逐用例执行，并回填结果表。

> 重跑入口：**流程名 plan-flow-uat**——操作序列、前置检查与历史轮次账目见 [RUNBOOK.md](./RUNBOOK.md)（触发语"请重复跑 plan-flow-uat 自动流程"）；T1–T4 对话输入原文存于 [messages/](./messages/)（2026-10-02 从 tmp/ 收编入库）。

## 被验证的四个能力问题

| # | 问题 | 对应用例 |
|---|---|---|
| Q1 | Agent 能否根据裸需求自主判断"任务足够大，需要建计划"并开启 plan 流程 | T2 |
| Q2 | 调研需求 → 调用 auto-plan-new 技能写出计划，是否为流程自动触发 | T1 / T2 |
| Q3 | work / review 阶段是否自动触发；是否由不同 agent（角色）负责 | T3 / T4 |
| Q4 | merge 阶段是否同样自动触发 | T1 / T3 |

## 两条被测路径

1. **聊天 × 仓库四技能路径**（`~/.config/autoos/skills/auto-plan-{new,work,review,merge}`，启动时由 musk 从仓库 `.agents/skills/` 幂等分发）：阶段由用户对话指令显式点名，单会话单 Agent 按技能契约执行。这是设计行为（见 `skills/plan-driven-development`："one agent carries a feature end-to-end"），不是缺陷。
2. **内置 Relay plan 流程**（`relay/flows.rs::plan_flow`）：`plan(advisor) → execute(coder)【human 确认门】→ review(reviewer) → document(assistant)` 四相位四职业，阶段自动推进、计划文件为唯一交接物。聊天 Agent 判定需求够大时可自行启动（经 `/api/forge/relay/runs` 可见）。

## 环境与前置

- musk serve：`cd D:\autostack\auto-musk && set MUSK_SERVE_PORT=17201 && backend\target\release\musk.exe serve`（后台）。
- ai-daemon：`cd D:\autostack\auto-ai && target\release\aaid.exe`（后台；musk 亦有 `auto_start_daemon` 兜底）。
- 演示工作区：`tmp/demo`（工作区注册表 `~/.config/autoos/workspaces.json` 中 `demo-1`）。**重置**：删除 `tmp/demo` 全部内容 → `git init -b main` → 写 README → 基线提交（避免 Agent 把工作区误认成 auto-musk 主仓）。
- 会话配置（每个新会话创建后立即设置）：
  - `PATCH /api/chats/session/{id}/approval` `{"approval_mode":"auto"}` —— 无人值守必需；副作用见已知问题 K4。
  - `PATCH /api/chats/session/{id}/thinking` `{"thinking_level":"max"}` —— 对策 K1。
- 驱动脚本（本目录）：`plan-driver.mjs`（chat 会话驱动：create/approval/send/wait/page/lastmsg，以 SSE `done` 帧判定 run 收束）；`relay-watch.mjs`（Relay run 监视 + 门批准）。

## 用例

### T1 四阶段显式点名全流程（基线）

步骤：全新会话依次发送四条消息（每条等收束后再发下一条）：
1. `请用 /auto-plan:new 技能新建一个计划：<需求>…`（需求需含：多文件工程 + 测试 + 新建规范 `docs/specs/<name>.md`）
2. `计划已确认，请用 /auto-plan:work 技能执行 PLAN-001…`（如工作区是独立仓库，指明 worktree 放 `D:/autostack/.wt/<组名>/demo`）
3. `请用 /auto-plan:review 技能独立复审 PLAN-001…`
4. `请用 /auto-plan:merge 技能沉淀 PLAN-001…`

判定（全部满足为 pass）：
- 计划落盘 `docs/plans/001-*.md`，status 走 `drafting → reviewed → archived`；
- work 产出实现提交（worktree 分支），`node --test` 全绿；
- review 有独立取证记录（不沿用执行摘要）；merge 四收据齐（prepared/landed/ledger_refreshed/cleaned），`--ff-only` 合回 main，计划归档、worktree/分支清理；
- 终检：`/api/plans` 显示 `archived: true`；spec 文件与 ledger 哈希一致；产物无乱码、无外链依赖。

2026-09-28 结果：**pass**（PLAN-001 密码生成器；main `6ca87c0→b864746→bfecc9b→7098dd3`）。

### T2 裸需求自触发（Q1/Q2）

步骤：全新会话，只发一条**不含任何技能名/流程词**的需求（建议带 3+ 个功能点与测试要求，处于"大小模糊"区间）。观察首轮回复与动作。

判定：Agent 自主选择进入某条 plan 路径（启动 Relay 流程，或加载 plan-driven-development/auto-plan-new 技能）为 pass；直接开写代码为 fail；仅提问不动手为 inconclusive。

2026-09-29 结果：**pass** —— Agent 回复"这个需求涉及新建工程、多个模块和测试，我启动 plan 流程来做"，自启 Relay run（四相位）。

### T3 Relay 流程编排与确认门（Q3/Q4）

步骤：承接 T2 启动的 run（或经 `POST /api/forge/relay/runs` `{"task":...,"workflow":"plan"}` 直接启动），用 `relay-watch.mjs <run_id>` 跟踪到收束；如需测人工门，把会话 approval 设回 `human` 后在门处 `POST /api/forge/relay/runs/{id}/gate` `{"decision":"approve|edit|reject"}`。

判定：
- 阶段自动推进、每相位独立角色（advisor/coder/reviewer/assistant）各自留独立的会话轮次 → 编排 pass；
- 计划文件落盘 `docs/plans/NNN-*.md` 且被 execute 相位实际消费 → 交接 pass；
- 终态 delivered 且产物可用 → 全流程 pass。

2026-09-29 结果：**fail（失败级联，根因见 K5–K7）** —— advisor 产出问卷而非计划文件（未调 `create_plan`）→ human 门被 approval auto 同秒自动放行（审计轮 `Gate execute approve`）→ coder 找不到计划、六个只读调用后空转"completed"（完成摘要臆造"PLAN-086"）→ reviewer 正确判 REJECTED 但陷入对 `.autoos/specs.json` 的手工修复循环（17+ 次 edit_file/write_spec 逐字段试错），最终被 `read_file` 循环检测击杀（`run_failed`），document 相位未达。工作树无损；ledger 被 reviewer 的 `update_spec` 重写为旧六区空账本（K6），事后手工恢复并留审计。

### T4 跨会话独立复核（Q3 独立性）

步骤：全新会话（S3），只给归档计划路径："……请用 /auto-plan:review 技能对已归档的 PLAN-001 做核实性复核（no-op 路径）……"。判定：复核者不依赖原实现会话上下文，仅凭工件（git/spec/测试）得出结论且与收据一致为 pass。

2026-09-29 结果：**pass** —— 零上下文新会话自主路由到技能的 archived no-op 验证路径（"Do not reopen an archived Plan"）；全证据从工件重建：线性历史与父子提交核实（还自主发现并绕过 cmd.exe 对 `^` 的转义）、spec sha256 磁盘重算逐字符一致、ledger 与归档收据互证、`node --test` 新鲜复跑 9/9、交付零漂移（`git diff b864746..HEAD` 仅归档文件）；并正确溯源出工作区里 `docs/reviews/R2-*.md` 残留来自 T3 的失败 run（与 ledger 恢复审计对上），结论"收据与落地一致"，未改动任何状态。

## 已知问题与对策（重跑前必读）

| ID | 现象 | 对策 |
|---|---|---|
| K1 | glm-5.3-flash 思考流耗尽服务商默认 4096 输出上限，响应截断后 run 提前收束（收束时最后轮 `out=4096`） | 会话设 `thinking_level=max`（PLAN-064：显式档位会把 max_tokens 抬到 budget+1024）。**094 重跑实测补充：该变通只覆盖 chat 会话自身请求——Relay 相位（builtin 档位）的请求仍被 4096 截断**（见 2026-09-29 重跑记录 R1/R2），advisor 相位写计划轮截断即无法产出计划文件；根修候选（daemon 预设安全 max_tokens）登记于 KNOWN-DEBT |
| K2 | 长任务 run 中途收束后**不会自动续跑** | 用"继续：接着完成当前阶段…"nudge 重发（T1 当轮两次） |
| K3 | 工具循环检测（同参重复调用 4 次）会击杀 run | 属防护机制；失败后按上条 nudge 续跑或人工收敛 |
| K4 | 会话 `approval_mode=auto` 会把 Relay human 门**同秒自动放行**（含 advisor 留给用户的澄清问卷）——无人值守的既定语义，但问卷等于按缺省执行 | 需要真实人审时用 `human` 模式 + `relay-watch.mjs` 在门处决议。**PLAN-094 已收口一半**：auto 放行审计轮现带注入反馈文本（`Gate <step> <decision> — <note>`），且 execute 门有 plan_file 前置不变式（见 K5）——问卷门不再可能静默放行一个无计划 run |
| K5 | Relay 断链模式：advisor 以问卷代计划且不调 `create_plan` → 计划文件缺席 → coder 空转 → reviewer 无物可审 | **PLAN-094 已修复（2026-09-29 重跑四 run 实证）**：execute 门 approve 前置不变式——`plan_file` 缺失时 auto=直接 `run_failed`（计划 D1 退化，引擎 redraft 只重做 execute 相位、重跑 plan 相位不可表达）、human=409 不消费门；级联（coder 空转→reviewer 死循环）不再可能。恢复出路：重开 run（advisor 幂等复用）或 human 门 reject+feedback |
| K6 | **ledger schema 不兼容 bug**：Relay reviewer 的 `update_spec` 读不了聊天侧 merge 写的 v1 ledger（`missing field project`），且会用旧六区 schema **整体重写** `~<ws>/.autoos/specs.json`，抹掉既有条目 | **PLAN-094 已修复**：merge 技能钉死"只经 spec 工具写账本"（store-mediated）+ spec 工具 load 失败改为响亮错误（期望字段/六区清单/禁手改明令/恢复路径，文件字节不变，有字节不变单测）；外语格式不可能再被静默重写。094 重跑实测：document 相位经工具写出的 ledger 为原生六区格式、serde 可解析 |
| K7 | 阶段间经"完成摘要"传染臆造内容（coder 摘要臆造"PLAN-086"，reviewer 循此查证幻影计划）——印证"阶段只共享计划文件、不传摘要"的设计必要性 | 计划文件必须真实落盘（K5 修复后 auto 路径的传染源消失——无计划 run 活不过门）。094 重跑仍观察到幻影（human-arm run 的 coder `read_plan seq 86`）——模板纪律属**指令级约束**，非机制强制；登记观察项 |
| K8 | aaid 流式请求的 usage **in_tokens 恒 0**（input token 未回填）：PLAN-098 实况 smoke（2026-10-03，worktree@plan-098-dev 两轮 chat）三轮流式请求 `in=0` 全数复现，非流式请求 `in=123` 正常；out_tokens 正常 | 观察项，不阻塞判定：musk 侧 telemetry 如实记 0（collect-telemetry 摘要打"daemon 流式 usage 未回填"观察标注），cost/时长对比以 out_tokens+elapsed_ms 为准；根修在 daemon 侧（auto-ai 只读，不在 PLAN-098 范围） |

## 结果总表

| 用例 | 日期 | 结果 | 备注 |
|---|---|---|---|
| T1 | 2026-09-28 | pass | PLAN-001 archived delivered；四收据齐 |
| T2 | 2026-09-29 | pass | 自启 Relay plan 流程 |
| T3 | 2026-09-29 | fail | K5 断链级联；循环检测收束 |
| T3（094 重跑） | 2026-09-29 | **pass（门修复四断言 3/4 实证，第 4 断言受阻于 K1 预存债）** | 门不放行/ledger 零破坏/明确原因失败三断言四次 run 实证；"计划文件被 execute 消费"未能在实跑复现（advisor 相位自身产出被 K1 截断×2 + 澄清-停止×1），机制面由 driver 行为测试覆盖（hw+ag）；级联死亡 |
| T4 | 2026-09-29 | pass | 零上下文自主路由 no-op 路径，证据全重建，收据一致 |

## 复现步骤（全新一轮）

1. 重置 `tmp/demo`（见"环境与前置"）；确认 17201/17654 监听。
2. 按 T1→T2→T3→T4 顺序执行；每用例新建会话并套用 K1/K4 配置。
3. 每轮结束后：核对计划状态（`/api/plans?include_archived=true`）、spec↔ledger 哈希、`node --test`、`git status` 干净；回填结果总表与本文件的"结果"小节。

## 2026-09-29 PLAN-094 重跑记录（T3 修复验证）

环境：demo 工作区重置（基线 `8fe52ad`）；**修复版 musk**（worktree musk-094，调试构建）serve 于 ：17255（生产 ：17201 未动）；aaid 复用 ：17654；chat 会话 `approval=auto` + `thinking=max`。四断言按计划 PLAN-094 §6 用例 6 判定：

| # | 断言 | 结果 | 证据 |
|---|---|---|---|
| 1 | 计划文件出现且被 execute 相位实际消费 | ✗（实跑未复现；机制面由测试覆盖） | 四次 run 的 advisor 相位均未真正落盘计划：R1/R2 = 输出在 4096 上限截断（R2 已写计划正文至 §5 详细设计处腰斩，K1 在 Relay 相位不受会话 thinking=max 变通保护）；R3 = advisor 完整收束但按"澄清-停止"纪律停在不调 create_plan；R4 = human 门 reject+feedback 重做的 coder 相位空转至"完成"（模板纪律指令级、非强制）。"有计划文件→门照常放行→execute 消费"由 driver 行为测试 `plan094_auto_gate_with_plan_file_approves_through`（hw+ag）钉死 |
| 2 | 门不放行无计划 run | **✓（×4 实证）** | R1-R3（auto）：advisor 相位一结束、门即拒绝——`run_failed`，error=`plan phase ended without a plan file — restart the run (advisor reuses existing plans) or answer the advisor via reject+feedback in human mode`；事件序列 step_completed(plan)→gate_waiting→run_failed，**无 gate_resolved、无 coder/reviewer/assistant 轮次**。R4（human）：POST approve → 409 错误体且门保持 waiting（未消费） |
| 3 | ledger 不被外语化重写 | **✓** | R1-R3 全程零 ledger 写；R4 document 相位经 spec 工具写出的 `.autoos/specs.json` 为原生 `{project, version, sections[6]}` 格式（goals/architecture/designs/tests/reviews/reports，serde 可解析）——与 K6 事故的外语形状（flat specs/history）对比鲜明 |
| 4 | run 终态 delivered 或带明确原因失败 | **✓** | R1-R3 `failed` + 上述契约 error 文案；R4 显式人工 reject 放行后正常走完（completed） |

四 run 清单（demo-1 工作区）：R1 `run-1790654109-eccbccc`（chat 自启，auto）；R2 `run-1790654229-255398d0`（nudge 重启，auto）；R3 `run-1790654564-c3ba84c`（轻需求，auto）；R4 `run-094-humanarm-8975`（直启，human 门实测 409 + reject 重做）。

**结论**：UAT T3 的失败级联（无计划放行 → coder 空转 → reviewer 手术死循环）已不可能发生——门在 auto/human 两臂都拒绝无计划的 run。剩余缺口在 advisor 相位自身的产出可靠性（K1 截断在 Relay 相位不受现有变通保护 / 澄清-停止被沙盒语境误触发），属已登记的模型交互债，不在 PLAN-094 范围。

**顺带实测发现（登记候选）**：
- Relay 相位请求的 max_tokens 抬升缺口（K1 变通只覆盖 chat 会话）——根修候选：daemon 对 relay/builtin 档位请求预设安全 max_tokens。
- 模板纪律为指令级约束：R4 的 coder 在"计划文件缺失=阻断性缺陷，立即停止"模板 + 门反馈下仍空转至"完成"——机制级强制（如 execute 相位开始时校验 plan_file 存在性）为后续增强候选。
- ag 生成的 gate 路由对错误信封返回 HTTP 200（body 内 `{"error":{code:409}}`），与 hw 路由的真 409 状态行不同——a2r 接缝外观差异，消费者以 body error 为准。
