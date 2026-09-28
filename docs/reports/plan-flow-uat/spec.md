# Plan 流程对话驱动 UAT 测试说明

目的：可重复地验证 auto-musk 两条 plan 流程路径在对话驱动下的真实行为，并固化判定标准与已知问题对策。
本文记录 2026-09-28/29 两轮实测结果；重跑时按"复现步骤"重置环境后逐用例执行，并回填结果表。

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
| K1 | glm-5.3-flash 思考流耗尽服务商默认 4096 输出上限，响应截断后 run 提前收束（收束时最后轮 `out=4096`） | 会话设 `thinking_level=max`（PLAN-064：显式档位会把 max_tokens 抬到 budget+1024） |
| K2 | 长任务 run 中途收束后**不会自动续跑** | 用"继续：接着完成当前阶段…"nudge 重发（T1 当轮两次） |
| K3 | 工具循环检测（同参重复调用 4 次）会击杀 run | 属防护机制；失败后按上条 nudge 续跑或人工收敛 |
| K4 | 会话 `approval_mode=auto` 会把 Relay human 门**同秒自动放行**（含 advisor 留给用户的澄清问卷）——无人值守的既定语义，但问卷等于按缺省执行 | 需要真实人审时用 `human` 模式 + `relay-watch.mjs` 在门处决议 |
| K5 | Relay 断链模式：advisor 以问卷代计划且不调 `create_plan` → 计划文件缺席 → coder 空转 → reviewer 无物可审 | 待修复；重跑 T3 时检查 `docs/plans/` 是否出现新计划文件作为交接健康的首要信号 |
| K6 | **ledger schema 不兼容 bug**：Relay reviewer 的 `update_spec` 读不了聊天侧 merge 写的 v1 ledger（`missing field project`），且会用旧六区 schema **整体重写** `~<ws>/.autoos/specs.json`，抹掉既有条目 | 重跑后核对 ledger；被抹可按归档计划 §9 收据手工恢复（本目录 spec 记录 2026-09-29 恢复一例）。README 已声明"应用内置流程尚未接入新的沉淀契约"，此为其具体表现 |
| K7 | 阶段间经"完成摘要"传染臆造内容（coder 摘要臆造"PLAN-086"，reviewer 循此查证幻影计划）——印证"阶段只共享计划文件、不传摘要"的设计必要性 | 计划文件必须真实落盘（K5 修复后此传染源消失） |

## 结果总表

| 用例 | 日期 | 结果 | 备注 |
|---|---|---|---|
| T1 | 2026-09-28 | pass | PLAN-001 archived delivered；四收据齐 |
| T2 | 2026-09-29 | pass | 自启 Relay plan 流程 |
| T3 | 2026-09-29 | fail | K5 断链级联；循环检测收束 |
| T4 | 2026-09-29 | pass | 零上下文自主路由 no-op 路径，证据全重建，收据一致 |

## 复现步骤（全新一轮）

1. 重置 `tmp/demo`（见"环境与前置"）；确认 17201/17654 监听。
2. 按 T1→T2→T3→T4 顺序执行；每用例新建会话并套用 K1/K4 配置。
3. 每轮结束后：核对计划状态（`/api/plans?include_archived=true`）、spec↔ledger 哈希、`node --test`、`git status` 干净；回填结果总表与本文件的"结果"小节。
