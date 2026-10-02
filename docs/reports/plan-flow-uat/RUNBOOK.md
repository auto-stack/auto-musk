# PLAN-FLOW-UAT 重跑手册（流程名：plan-flow-uat）

**触发语**：用户说"**请重复跑 plan-flow-uat 自动流程**"（变体："重跑四阶段全自动计划流程 UAT"）时，执行本手册——按 §2 重置环境并跑 T1 四阶段基线，再按 §4 回填对比记录。执行者先读完本手册与 [spec.md](./spec.md) 再动手。

- **流程定位（一段话）**：在 auto-musk 应用的**全新演示工作区**（`tmp/demo`，注册名 `demo-1`）里创建全新聊天会话，按四条预置消息依次点名 `auto-plan:new → work → review → merge` 四技能，验证应用内 Agent 能否无人值守走完"建计划 → 执行 → 复审 → 合并"；跑完回填 spec.md 结果总表，形成可对比的重复试验记录。
- **用例定义 / 判定标准 / 已知问题 K1–K7**：见 [spec.md](./spec.md)，本手册只给操作序列，不重复其内容。
- **首次执行档案**：2026-09-28/29 两轮，ZCode 会话 `sess_47ea3a2c-cdea-4d2e-a14c-5ce1045b1181`；T1 产物 PLAN-001（本地密码生成器）archived delivered，demo main 线性历史 `6ca87c0→b864746→bfecc9b→7098dd3`。该工作区在 09-29 为 094 重跑被重置，产物本体已不存在——只有本目录的记录是持久的。
- T1 四条消息点名技能属**设计行为**（聊天 × 仓库四技能路径，见 spec.md"两条被测路径"）；裸需求自触发行为由 T2 单独覆盖。

## 1. 前置条件

| 项 | 要求 | 缺失时 |
|---|---|---|
| musk.exe | `backend/target/release/musk.exe`，serve 端口 **17201** | 按仓库 README 构建 release |
| aaid.exe | auto-ai 仓 `target/release/aaid.exe`，端口 **17654**（musk 有 `auto_start_daemon` 兜底） | 构建，或依赖兜底 |
| 技能分发 | musk 启动时把仓库 `.agents/skills/auto-plan-*` 幂等分发到 `~/.config/autoos/skills/` | 起服务后抽查该目录 |
| 工作区注册 | `~/.config/autoos/workspaces.json` 含 `demo-1 → D:/autostack/auto-musk/tmp/demo` | 手动注册 |
| 驱动脚本 | 本目录 `plan-driver.mjs`（chat 驱动）、`relay-watch.mjs`（仅 T3 用） | — |
| 预置消息 | 本目录 [messages/](./messages/) 六个文件 | — |

注意：两个脚本**硬编码** `BASE=127.0.0.1:17201`、`WS=demo-1`；换端口/工作区需先改脚本常量（2026-09-29 的 094 重跑在 ：17255 用修复版 musk 时即如此处理，且当时生产 ：17201 未动）。

## 2. T1 四阶段基线（本流程主用例）

1. **重置工作区**（⚠️ 危险操作——`tmp/demo` 全部内容删除不可恢复；确认无在跑会话后执行）：
   删除 `tmp/demo` 全部内容 → `git init -b main` → 写 README（"# demo\n\n演示工作区（UAT/T 重跑沙盒）。"）→ 基线提交并**记录 hash**。基线提交的作用：避免 Agent 把工作区误认成 auto-musk 主仓。
2. **启动服务**：musk serve（`MUSK_SERVE_PORT=17201`）+ aaid，均后台；确认两端口监听。
3. **建会话并配置**（K1/K4 对策，两步缺一不可）：

   ```bash
   SID=$(node plan-driver.mjs create)          # superpowers 模式新会话
   node plan-driver.mjs approval "$SID"         # approval_mode=auto（副作用见 K4）
   curl -X PATCH "http://127.0.0.1:17201/api/chats/session/$SID/thinking?workspace=demo-1" \
        -H 'Content-Type: application/json' -d '{"thinking_level":"max"}'   # K1 对策
   ```

4. **依次发送四条消息**（每条收束后再发下一条；`msg` 子命令 = send + wait + lastmsg 一条龙，末参为等待超时分钟数）：

   ```bash
   node plan-driver.mjs msg "$SID" messages/msg-1-new.md    60   # /auto-plan:new
   node plan-driver.mjs msg "$SID" messages/msg-2-work.md   90   # /auto-plan:work
   node plan-driver.mjs msg "$SID" messages/msg-3-review.md 60   # /auto-plan:review
   node plan-driver.mjs msg "$SID" messages/msg-4-merge.md  90   # /auto-plan:merge
   ```

   中途 run 提前收束属已知现象（K1 截断/K2 不自动续跑）：按 spec.md K2 用"继续：接着完成当前阶段…"nudge 从 stdin 重发（`node plan-driver.mjs send "$SID"` + `wait`）。
5. **终检清单**（全部满足才记 pass；判定细节见 spec.md T1）：
   - `GET /api/plans?include_archived=true&workspace=demo-1` → PLAN-001 `status: archived`、`archived: true`
   - spec 文件 SHA-256 与 `.autoos/specs.json` ledger 派生哈希一致
   - demo 仓 `node --test` 全绿；`git log --oneline` 呈 new→work→archive→receipts 线性四段
   - merge 四收据齐（prepared / landed / ledger_refreshed / cleaned）；worktree 与分支已清理
   - 产物无乱码、无外链依赖

## 3. 可选扩展用例（对照数据）

- **T2 裸需求自触发**：全新会话只发 [messages/msg-t2-bare.md](./messages/msg-t2-bare.md)（不含技能名/流程词），判定 Agent 是否自主进入某条 plan 路径（判据见 spec.md T2）。
- **T3 Relay 编排与确认门**：`POST /api/forge/relay/runs {"task":...,"workflow":"plan","workspace":"demo-1"}` 直启或承接 T2 启动的 run，`node relay-watch.mjs <run_id> [timeout_min]` 跟踪；测真实人工门需先把会话 approval 设回 `human`，在门处 `POST /api/forge/relay/runs/<id>/gate {"decision":"approve|edit|reject",...}`。
- **T4 跨会话独立复核**：全新会话只发 [messages/msg-t4-review.md](./messages/msg-t4-review.md)（仅给归档计划路径）。

## 4. 跑完后必做（对比记录的落账）

0. **产出遥测摘要（PLAN-098 起，第 0 步）**：对每个跑过的会话/relay run 执行
   `collect-telemetry.mjs`（与本脚本同目录，零依赖），摘要 JSON/MD 存入本轮
   记录并在下述回填中引用其路径：

   ```bash
   node docs/reports/plan-flow-uat/collect-telemetry.mjs \
     --session <sid>|--run <rid> --autoos <demo>/.autoos \
     [--aaid-log <aaid.log> --since <t> --until <t>] \
     --out docs/reports/plan-flow-uat/runs/<date>-<sid>
   ```

   - 摘要含：按角色/相位聚合（轮数/工具数/时间跨度）、逐请求
     provider/model/token/耗时表、失败签名自动标记（K1 截断 / K3 循环 /
     K4 门同秒放行 / K5 无计划）。退出码 2=输入缺失（列出缺失项）；确定性
     断言可用 `--expect <json>`（不符退出 3）。
   - `provider` 为 null 属常态（daemon wire 不携带 provider，muskside 记
     显式空值）；传入 aaid 日志时由时间窗启发式 join 回填并标注置信——
     **旁证不冒充权威**。
   - `in=0` 为 K8 已知观察项（daemon 流式 usage 未回填，见 spec.md K 表），
     摘要已自动标注；重跑对比以 `out_tokens`+`elapsed_ms` 为准。
   - 结果总表回填与 §5 历史行追加时**引用该摘要路径**：重跑对比直接读两轮
     摘要 JSON（逐请求 model/token/耗时 + 签名计数），无需手工对齐时间戳。

1. 回填 [spec.md](./spec.md) 结果总表：用例 | 日期 | 结果 | 备注（含 demo main 提交链、run_id、耗时、遥测摘要路径）。
2. 在本文件 §5 追加一行历史记录（日期、轮次、结论、commit 指针）。
3. **记录环境指纹**：musk/aaid 构建版本（git describe）、日期、模型与 thinking 档位——缺环境指纹的对比记录无效。
4. 新发现的已知问题：登记 `docs/plans/KNOWN-DEBT-AND-RISKS.md`，并在 spec.md K 表加行。
5. 本轮工作区产物在下一轮重置时会被清除——**只有落进本目录（受版本控制）的内容才持久**；关键截图/日志先存入本目录或 `tmp/ui-parity/PLAN-<n>/` 再在记录中引用。

## 5. 历史记录

| 日期 | 轮次 | 结论 | 记录 |
|---|---|---|---|
| 2026-09-28 | 首轮 T1（ZCode 会话 sess_47ea3a2c；musks@a17201 + aaid@17654） | **pass**——PLAN-001 archived delivered，四收据齐，终检全过 | spec.md T1；驱动消息 [messages/msg-1..4](./messages/) |
| 2026-09-29 | 首轮 T2/T3/T4 | T2 pass（自启 Relay）；T3 fail（K5–K7 级联，催生 PLAN-094）；T4 pass | spec.md T2/T3/T4 与 K 表；main `97fc677` |
| 2026-09-29 | PLAN-094 修复验证重跑（demo baseline `8fe52ad`，修复版 musk@:17255） | 门修复 3/4 断言实证（门不放行无计划 run ×4 / ledger 零破坏 / 失败带明确原因）；advisor 产出可靠性受阻于 K1 预存债 | spec.md"2026-09-29 PLAN-094 重跑记录"；main `1993eaa` |
