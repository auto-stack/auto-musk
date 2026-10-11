# PLAN-105 证据 — UAT 87af4d4b run 生命周期与工具执行层修复

会话：`87af4d4b7e7a9c25e2d44114`（demo2，2026-10-11，/auto-plan 全流程）。
三源：`tmp/manual-uat-aaid.log`（daemon）、`tmp/manual-uat-serve.log`（musk）、
`tmp/demo2/.autoos/conversations/87af4d4b…/turns.jsonl`（353 行）。现场只读。

## T-01 定罪纪要 — P1 run 不收束（300s 看门狗死等）

### 时间线（本地时 = UTC+8）

| 时刻 | 源 | 事实 |
|---|---|---|
| 17:38:13 | serve | run spawn（elapsed 基准，824s 倒推） |
| 17:45:44 | aaid | `chat req: msgs=37 candidates=2`（本 run 最后一次请求） |
| 17:46:37 | aaid | `chat stream done … out=2647`（上游流收完，provider_task Ok 臂） |
| 17:46:37± | turns | 末轮响应 = seq131 文本「对策：…」+ 3 工具调用（write_file×2 + `start /min … http.server`），**工具批全部执行且结果落块**（b30-b32 + results 在超时收束消息内） |
| 17:46:37→17:51:37 | aaid | **零请求、零日志**（chat handler 入口即打 `chat req`，PLAN-092 T-04） |
| 17:51:37 | serve | `chat run idle-timeout: idle=300s elapsed=824s`（最后事件时刻=17:46:37，秒级吻合） |

### 排除过程

- **流收尾形态**（计划原设想的 A/B/C 三嫌疑）：末轮流完整收完且工具批执行完毕
  ——`complete_stream` 已返回、`run_inner` 已过 wants_tool 臂，非「流完成不返回」。
  musk 级复现器形态 A（文本收尾）0.07s 收束、形态 B（工具轮→文本收尾）0.06s
  收束，正常路径健康（回归钉保留）。计划预设的「双候选错峰」形态在 UAT 未出现
  （aaid 每请求恰一对 start/done，单流），不单列复现器。
- **90s 读空闲超时为何没救**：PLAN-099 根修（f1c7794）只覆盖**响应体读循环**
  （`stream.next()` 空闲窗）。挂点不在读循环——在下一轮请求的 **send 阶段**
  （`self.http.post(...).send().await`：连接/写/等响应头），该 await 无任何超时。
- **on_event 阻塞**：喂狗是 watch channel 同步 send（非阻塞）；看门狗能触发
  说明 run_stream future 干净 pending（非同步卡死）。

### 定罪

**挂点 = auto-ai-client rust-ref `AiClient::complete_stream` 的 send 阶段无超时。**
`AiClient` 以 `reqwest::Client::new()` 裸构（无 connect/read/整体超时）。下一轮
请求 `send().await` 在池化 keep-alive 连接半死/对端持连不响应时永久 pending
（POST 非幂等，reqwest 不自动重试；UAT 具体触发 = daemon 侧连接行为不可从
现有日志分辨，但修复不依赖分辨：send 阶段有界 + 失败走既有 Err 收束臂即可）。

### 复现器（红证据）

| 测试 | 层 | 修复前实测 | 判定 |
|---|---|---|---|
| `chat_run_llm_send_stall_converges_bounded`（server.rs tests） | musk run 级：真 AiClient 打 stub daemon，请求 1 正常 SSE（delta+done 带 read_file），请求 2 accept 后持连不响应 | **10.15s FAILED**（8s 轮询窗内守卫不释放，run 悬挂） | 红=UAT 形态复现 |
| `complete_stream_send_timeout_on_silent_daemon`（auto-ai-client lib.rs tests） | client 级：stub accept 后持连不响应 | **30.01s FAILED**（悬挂=stub 持连全长，错误语义断言不及） | 红=悬挂实证 |
| `chat_run_text_final_converges_without_watchdog` | musk | 0.07s ok | 绿钉（形态 A） |
| `chat_run_tool_round_then_text_converges_without_watchdog` | musk | 0.06s ok | 绿钉（形态 B） |

### 修复面清单（T-02 依据）

1. **auto-ai-client rust-ref lib.rs**：`complete_stream`/`complete` 的 send 阶段
   包 `tokio::time::timeout`（env `AUTO_AI_CLIENT_SEND_TIMEOUT_SECS`，缺省 60s
   ——盖过 daemon 侧 30s permit 最长等待；超时单次重试一次再失败 → 
   `ClientError::Http("llm send timeout …")`）；`complete` 的 `resp.json()` 体读
   同窗包界。读循环 90s 空闲窗（099）不动。
2. **auto-ai-daemon server.rs（同路径邻接缺陷，顺手加固）**：done/error 尾帧
   `tx.try_send`（mpsc 64 满 → 静默丢帧且日志已先打 done）改有界 `send().await`
   ——done 帧丢失会让 client 把带工具调用的响应当纯文本终答（假收束），EOF
   前置语义同时保证 done 先于流结束。
3. **musk 侧无改动**（收束臂已齐：Err 臂落盘可见错误+释放守卫；看门狗降为
   兜底）。P2/P3 的续跑/队列在 T-03/T-04 增量。

### 与 E-STREAM-HANG（099）的交界

099 修「流中停滞」（体读窗 90s）；P1 修「下一轮请求 send 悬死」（send 窗 60s）。
两窗互补，均终结为 Http 错误交 musk 既有失败臂。若实机再出现 099 环境变量
失效形态，另行登记不吞并。

---

## T-02～T-08 落地证据（V01–V07）

| 任务 | 验证 | 结果 |
|---|---|---|
| T-02 收束修复 | V01 红→绿 | musk 形态 D 10.15s 红 → **2.11s 绿**（守卫释放+失败消息+工具现场保留）；client 6/6（send 钉测试 ~2s=1s 窗+重试）；daemon 73/73 |
| T-03 文案分型+自动续跑 | V05 | 停滞型（点名在途工具 run_command）/挂起型两测绿；续跑注入 2 次调用恢复终答、防链式恰 2 次、关闭路径不注入；env `AUTO_RUN_AUTO_CONTINUE` |
| T-04 待发队列 | V05/V08 | guard 占用入队+⏳ 提示；收束重放两轮按序（一轮完成→提示→二轮完成）；人工边界新消息 id |
| T-05 引号保真+词法 | V03 | 引号矩阵四测绿（commit -m「fix: a b c」回读一致/git -C "space dir"/curl -o "my out.txt" 落盘/findstr /C:"two words" 命中）；tokenizer 单测+confinement 漏判补红两例+根内带空格不误伤；runner/safety 27 绿 |
| T-06 specs 诚实化 | V04 | 帕斯卡 Approved→400 带该区清单；合法转移 applied_status 回显+GET/POST 对称回写；非法 400 带 `from -> to` 因；upsert bypassed_transition 标注；specs lib 39 绿 |
| T-07 VM 白名单 | V07 | host 桥回环五点绿（增/查/错误透传/未知 ws 拒/删）+ VM serve 冒烟门 2/2（.at 全量编译含新端口） |
| T-08 进程收割 | V06 | runner 级：start /b 分离 server→job close→端口 ≤5s 释放；run 级端到端：看门狗中止→收割→端口释放（4.09s，兼验停滞型文案）。邻接真缺陷修复：超时臂读任务裸 await 悬死（15 分钟复现）→2s 有界 |

全 lib 回归：**578 passed / 0 failed / 1 ignored**（T-09 时点）。

## T-09 回归与场景复跑（V02/V08）

### V02 全量（逐目标串行）

- **绿**：--lib 578；integration 14 目标全绿（含本计划新增
  `plan105_specs_transition_honesty` 4/4、修复的 `parity_workspace_endpoints`
  4/4）。
- **红（既有，非本计划引入）**：22 个 integration 目标编译红——三类旧伤：
  ①`chat_runs` HashSet→HashMap（368c2f9/PLAN-099 漏改 tests 构造，仓内注释
  自证）；②`chats::ToolCall` 缺 escape_paths/gate_id/pending_cmd
  （PLAN-103 同款漏改）；③`StartRunRequest.authorization` 缺字段。
  **基线对照**：plan 基提交 11c5db5 干净 worktree 上 `--tests --no-run`
  = 22 目标 23 errors 同款红集；`tool_atoms::run_command_dangerous_returns_paused`
  运行红在 pre-T-05 提交 a18913b 与基线 11c5db5 同样复现（环境依赖：'/'
  解析到 D:\ 根）。**零新增红成立**；本计划顺手修复了全部 AppState 字面量
  （chat_pending/run_auto_continue/chat_procs 三字段 21+ 文件）。
- 既有测试偶发（chat_busy/cancel/failure_preserves 批跑偶 404/500）：
  Windows fs 并行竞态（`chats.get()` 无锁读 vs 原子写窗口），基线（stash
  对照）同样复现——非本计划引入，建议另行立项。

### V08 实机复跑（新演示 workspace `v08-demo` + 隔离 MUSK_CONFIG_DIR + 真栈 aaid）

| 点 | 实机证据 | 判定 |
|---|---|---|
| P6 | 帕斯卡 `Approved` → **400** `unknown status 'Approved' … valid for section 'designs': empty, draft, under_review, approved, rejected, superseded, outdated`（区专属清单+原串）；合法 `draft→under_review` → 200 `{"applied_status":"under_review"}`；非法 `under_review→draft` → 400 `invalid status transition for designs: under_review -> draft` | ✅ 实机 |
| P4 | 真 LLM（basic 模式 4 轮）：`git commit -m "v08 quote test: a b c"` 首触 PAUSED（白名单外）→ force 执行 → **root-commit b8d5808 消息原样**，`git log -1 --pretty=%B` 回读 `v08 quote test: a b c` 一致（UAT 中同款命令引号丢失中止） | ✅ 实机 |
| P1/P3/P7 | P1=send 悬死（网络竞态形态，不可确定性复演；musk 形态 D 复现器红→绿 + client 钉测试双层覆盖）；P3=队列重放（V05 集成测）；P7=host 桥回环 + VM serve 起服门 | ✅ 双层确定性证据（建议复审抽样实机） |

### 遗留（merge 阶段）

- KNOWN-DEBT K2 行改写 + Q-1（自动续跑缺省值）回用户确认。
- auto-ai 依赖 worktree（auto-musk-dev-105）保留至本计划 merge
  （musk worktree path 解析耦合组内兄弟——musk-102 组同款先例），
  merge 时随本组一并收尾；auto-lang 为只读镜像（detached）同批清理。

（T-02+ 证据随任务推进续记。）

