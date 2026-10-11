# PLAN-104 证据报告（T-01 起草，随任务扩展）

> 计划：docs/plans/104-uat-vm-memory-and-spec-channel-fixes.md（r1）
> 现场：2026-10-11 手动 UAT（demo2 / 会话 87af4d4b7e7a9c25e2d44114）
> 日志留存：tmp/manual-uat-{aaid,serve,vm,vm-supervisor}.log（主检出 tmp/）

## T-01 定罪纪要

### K11 FilesView active_kind（✅ 已定谳并修复，commit eba12de）

- 现象：文件树点击抛 `RuntimeError("Field 'active_kind' not found on type
  instance App_State ...")`（tmp/manual-uat-vm.log，3 次复现；错误体携带
  App_State 全字段清单，含 ftExpanded/ftSelected 等 FilesStore 物化字段，
  独缺 active_kind）。
- 定谳：`files_store.at` computed `active_kind => files_file_kind(.active_path)`
  ——computed 内函数调用在 VM 不物化为实例字段（097 T-11 根因矩阵同族：
  computed 内函数调用/成员链在 VM 恒空）。handler 域读 `.store.active_kind`
  （files_view.at:45）即抛；模板域读同族四件（active_url/active_lang/
  active_video_html，:117-131）静默空（分支永不命中）。
- 修复：四 computed 退役改 model var，SelectFile 处理器域随路径同拍落账
  （单一写点实证）。门禁：gen-only strict EXIT=0 + vm-link-probe PASS +
  ui-parity catalog PASS 116/117。实机点击取证归 T-07。

### K9 活跃会话内存累积（静态定谳完成，runtime soak 待跑）

- 现象基线：会话活跃期 ~1MB/s 增长（用户实机）；会话停止即走平
  （0.013–0.017MB/s，两轮 60s 采样）。同轮 20 次静默退出（寿命 0.4–15min，
  退出码 0/1 混杂，WER 零记录），集中于重流式时段；本轮已用私有 MCP 口
  （9277），KD-066 端口争抢定罪不适用。
- 静态定谳（提交面=客户端拉取→合并→重渲染链）：
  1. `forge_store.at:209` `PollStream (every_ms: 500)` 无条件轮询
     （PLAN-536 D4：when 门已摘，gating 在 handler 内）；
  2. `.PollBackfill`（:1144）每拍 `GET /page?limit=50`——大会话实测
     292KB/拍（延迟 30→340ms 随会话增长），`JSON.parse` ×2；
  3. `rebuildParsedMessages(resp.session.messages, ...)` 全 50 条重建；
  4. `.messages = mergeTailWindow(...)` **无条件整表赋值**（无等值检查）
     ——内容未变的拍照样产生：新数组分配 + store 状态变更 + 下一帧全量
     vnode 重建。此即"没有新内容也在涨"的分配源。
- 落点冻结：**musk 客户端就地修**（T-02：回填增变检门——窗口指纹
  （条数+末条 id+末 content 长度+active_leaf）未变则跳过 rebuild 与赋值；
  ProfConfigsLoaded 重置指纹强制一次重建）。修后若 soak 仍见增长，再开
  auto-lang 渲染层缓存上界面（届时按依赖政策并排 worktree）。
- 待跑：soak active 场景（V01 红基线）→ T-02 后复跑（V02 斜率 ≤0.1MB/s
  + 1h 零静默退出）。20 次退出与累积是否同根，由 V02 的审计面
  （AUTO_DESKTOP_EXIT_LOG）一并回答。

### K10 大会话页空渲染（静态图景下修，矩阵兜底）

- 服务端投影已有界：chat_page.rs `page_window`（limit=50 + `before` 游标）
  + `normalize_message_for_page`（thin tool_calls + THINKING/TEXT_CAP 截断
  + flatten_blocks_budgeted 预算拍平）。292KB = 50 条窗的有界产物，非无界。
- 修正假设：长寿实例空渲染（服务端 200/292KB 正常返回、客户端零错误日志）
  更可能是 K9 累积族（重建/缓存耗尽）而非硬尺寸墙——新实例同载荷拉取正常
  支持此判断。
- 决策：T-03 的大页阈值降级机制保留为后备；先修 K9 后跑矩阵
  （50/150/350 轮 × 新/长寿实例）——长寿实例转绿则 T-03 缩为"横幅+i18n
  文档面"或按矩阵证据裁剪；新实例亦空渲染才启用完整降级。

## 命令与结果登记（随任务追加）

| 编号 | 命令 | 结果 |
|---|---|---|
| V04（T-05） | `cargo test -p musk --test specs_upsert_payload`（worktree，CARGO_TARGET_DIR=主检出 target） | 3 passed |
| V04（T-05） | `cargo test -p musk --lib` | 560 passed 0 failed 1 ignored |
| V06（T-04） | `auto build --gen-only --strict` | EXIT=0，65 components |
| V06（T-04） | `scripts\vm-link-probe.cmd` | PASS，linked 108365B（WARN 体积趋势，FAIL 阈 131072） |
| V06（T-04） | `node scripts/ui-parity.mjs check` | catalog PASS（116 declarations / 117 cases；live-required 0/5 属 live 面） |
| V06（T-02 精化后复跑） | gen-only strict + vm-link-probe + ui-parity check | EXIT=0 / PASS 108973B / catalog PASS |

## T-01/T-02 runtime 判定（soak active 场景，2026-10-11）

仪器：`scripts/soak-active-session.mjs`（隔离 serve+VM、私有端口/配置、
纯文本长流式会话、分相位 2s 采样 VM 私有内存；RED/GREEN 唯一变量=VM
启动目录编译的 .at 版本，serve 侧同为 worktree 构建）。auto-lang 二进制：
兄弟 worktree dde29cf2c `cargo build --release -p auto`（当日构建锁定）。

| 轮次 | VM .at | stream 斜率 MB/min | idle 斜率 MB/min | VM 退出 |
|---|---|---|---|---|
| RED | main（旧，无门） | +143.0（7 样本） | +48.0（41 样本） | **code=1** |
| GREEN-1 | worktree（tier2 指纹门） | +123.3 | +30.1 | **code=1** |
| GREEN-2 | worktree（+tier1 等长预检+tick 条件写） | +74.0 | +52.6 | **code=1** |
| GREEN-3 | 同上 | +40.3 | +56.0 | **code=1** |
| AUDIT | 同上 + AUTO_DESKTOP_EXIT_LOG | −14.1（噪声/GC 波动） | +72.6 | **code=1** |

判读：

1. **流式期改善方向一致**（143→40~123，样本方差大），tier1+tier2 有效
   削减 musk 客户端每拍全量重建；
2. **idle 残留 ~30-72MB/min 跨 .at 形态稳定存在**——tier1 已把回填路径的
   parse/重建全部跳过（空闲拍零 store 写入），残留只能来自每拍 HTTP 响应
   缓冲（292KB 级瞬态垃圾，~584KB/s）未被 VM GC 完全回收 + 每帧重绘的
   分配（KD-062 残留族）——**auto-lang 运行时/渲染器面，超出本计划边界**；
3. **静默退出归因定谳**：AUTO_DESKTOP_EXIT_LOG 审计**零记录** + code=1
   = 原生层 `std::process::exit`/外部终止（vm-process-stability 判读矩阵
   第 4 类），非 panic 非 main_return 非 Auto 层退出；与轮询/渲染负载强
   相关（空闲 VM 不退，仪器条件下 2-4 分钟内必退）；隔离端口、无并发
   会话——KD-066 端口争抢定罪不适用。**归属 auto-lang，随残留面一并移交**。

### V05 全套回归判定（base 28f3415 对照）

37 个集成目标逐目标串行：**11 目标绿**（含本计划新增 specs_upsert_payload
3/3 与 parity_specs/plan_flow_contract 等核心面）、**tool_atoms 1 测试失败**
（098 review F-2 存量红，环境敏感）、**20 目标编译错**——抽查定谳为
**base 预存**：`parity_plans.rs:46` 夹具 `chat_runs: HashSet::new()` vs
`server.rs:53` 字段 `HashMap<String, Instant>` 在 base 28f3415 即不匹配
（`git show` 双证），本计划 diff 零触及失败目标文件（`git diff
28f3415..HEAD --name-only` 核对）。归属=并发会话改后端字段未同步夹具，
非本计划引入；去向=夹具同步批次（登记 KNOWN-DEBT 候选，随 merge 交裁定）。

## AC 状态裁定（依计划 §10① 预案）

- **AC-01（idle 斜率 ≤0.1MB/s）：blocked on auto-lang**——musk 侧杠杆
  （重建跳过+parse 跳过+等值不写）已用尽，残留为每拍 HTTP 瞬态垃圾的
  GC 回收不全 + 每帧重绘分配，均在渲染器/运行时面。解阻动作=auto-lang
  专项（HTTP 响应缓冲生命周期 + 每帧重绘分配剖面 + 原生 exit(1) 归因；
  仪器与数据已备：scripts/soak-active-session.mjs + 本表五轮数据）。
- **AC-02/AC-03**：K10 大页矩阵与 K11 实机点击被退出面干扰（VM 2-4 分钟
  必退，无法构造长寿实例对照）——随 auto-lang 退出归因后补跑。
- **AC-04～AC-07 的 musk 侧面**：全部达成（见命令表）。

## 环境指纹

- musk worktree plan-104-dev @ base 28f3415（main 并发推进中，落地前 rebase）；
  T-05=09d9d0e、T-04=eba12de。
- 依赖只读兄弟：auto-ai@f1c7794、auto-lang@dde29cf2c、auto-down@895f8d0
  （detached；三仓 untracked Cargo.lock 为 build.rs 记账补拷，wt-guard 前
  须移除——merge 清单在案）。
- 代码生成/链接门禁二进制：auto-lang release auto.exe（10-10 构建，
  PLAN-760 登记落后观察项；.at 语法面无涉本次改动语义）。

## T-07 回归收口与 UAT 复验（2026-10-11 晚，隔离栈）

### V07 T1 简版复跑（主用例，AC-06 实证）

- **栈**：worktree 构建 musk.exe（plan-104-dev @ ac5b22d）serve :17255 +
  `MUSK_CONFIG_DIR` 私有注册表（demo-t1 → tmp/demo-t1）+ 共享 aaid :17654；
  自动门禁技能分发核验（4 技能来自 worktree，含 T-05/T-06 新文本）。
- **形态**：RUNBOOK §2 + PLAN-104 对策（分阶段新会话 / 白名单预注册
  `D:\autostack\.wt\demo-plan-001`（roots/add）/ 终检追加）。消息四条按
  msg-1 原文 + msg-2/3/4 改写（去首轮硬编码提交号）。
- **结果**：**pass**——
  - new（2aceea2c，4.2min）→ PLAN-001 drafting；
  - work resume（b815eb06，21.5min，首会话 9d569e1a 见事故①）→
    execution_done 5/5，worktree 单元提交 5c93041（4 文件 +493 行）；
  - review（0c32f001，10.8min）→ reviewed，outcome pass @r2 绑 5c93041；
  - merge（27069ba9，13.1min，首会话 fe215347 见事故③）→ main 快进
    `b779ca3→5c93041→f5ae990`、四收据齐、worktree 组+分支已清、
    `git status` 干净、`node --test` 15/15。
  - **台账 4 条全带 file+tags**（DES/TST/REV/REP；spec-blob:3acb55f6 与
    `git hash-object` 逐字一致）——AC-06 的 K12/K13 合同实证。
  - 全程 **零 K2 人工 nudge**（与 87af4d4b 轮 3 次 nudge 对比；阶段切分
    + 分会话形态下 K2 未触发）。
- **观察项（K 表候选）**：归档计划文件 frontmatter `status` 停留
  `reviewed` 未翻 `archived`（API `archived:true` 正确、目录归位正确；
  判读以 archived 标志为准，status 字段属 merge 技能簿记缺口）。
- 遥测摘要：`docs/reports/plan-flow-uat/runs/2026-10-11-{2aceea2c,
  9d569e1a,b815eb06,0c32f001,fe215347,27069ba9}*.{json,md}`。

### 事故记录（复跑期间三项）

1. **auto 模式 taskkill 自毁零防护（K 表新行）**：work 首会话（9d569e1a）
   13:07:27 连续踩 run_command 引号损坏（node -e 多行崩溃 → 下一条命令
   "文件名/目录语法错误"）后，一命令将 **musk.exe 按映像整体清除**——
   隔离栈 :17255 与上午 demo2 栈 :17201 双亡（tasklist 双证、本侧等待器
   ECONNRESET、后台任务 exit 127）。auto 审批模式（PLAN-073 F-D"不打断"）
   对宿主自杀命令无任何拦截。demo2 栈未在本轮恢复（重启属用户裁量）。
   关联：PLAN-105 P4/P5（引号保真，直接诱因）与 P8（进程收割面，缺
   宿主保护语义）；resume 会话以进程红线约束重启后正常走完。
2. **serve 硬死 → run 转录全丢**：被杀会话 turns.jsonl 仅存用户消息 1 行
   （run 事件全部未落盘；PLAN-099 T-03 工具边界快照在 serve 进程死亡
   形态下不提供任何持久化）。关联 PLAN-105 P1 家族（收束/持久化面）。
3. **上游 LLM 瞬时 500**：merge 首会话（fe215347）ev#3 即死（zhipu
   code 1234 网络错误）；新会话重发一次成功。外部因素，K 表备注即可。

### V06 复验处置

- 自动门禁（gen-only strict / vm-link-probe / ui-parity）HEAD ac5b22d
  已绿（T-02 精化后复跑，见命令表）。
- **实机冒烟（新增）**：auto-lang dde29cf2c auto.exe `run --render=vm`
  （AUTO_HTTP_PORT=17255）对隔离栈实跑 5+ 分钟，5124 行日志
  **零 RuntimeError / panic / Field not found**（StatusBackfill 流动证
  实桥接在线——:17201 已死仍存活即指向 :17255）。
- **FilesView 物理点击：本轮未执行**——computer-use 桌面驱动能力在本
  ZCode 会话未启用，无法编程点击原生窗口。K11 修复的静态门禁全绿 +
  097 T-11 同族先例 + 冒烟零错；点击复验留 review 裁定（或用户在 VM
  窗口随手点一次 Files 即可闭）。

### V05 复核

- 沿用 T-02 精化轮的全套判定（37 目标：11 绿 + tool_atoms 存量红 098
  F-2 + 20 编译错 base 预存）；本轮未改 musk 代码面（evidence/RUNBOOK/
  telemetry 文档落账不触代码），无需重跑。

## T-03 大页矩阵与 AC-01 真值重判（2026-10-12 凌晨，post-765 解阻后）

### 前置：PLAN-765 落地与本仓解阻件

- PLAN-765 已合入 auto-lang main（ff-only 4f53653fd + 归档 0eb915077，批量
  回归绿 43a816b47，worktree 已清）；post-765 `auto.exe`（auto-lang main
  release 构建）承载以下全部验证。
- **P765-D2 单位修（commit 0e06161）**：soak 仪器 slope 样本 t 已为分钟、
  误乘 60——历史报值实为 MB/hour 量级口径混乱，本报告历史斜率数字一律
  以"报值（旧仪器）"标注；新仪器为真值 MB/min。
- **F-R2 并发零串账（同 commit 0e06161）**：spec_tools 集成测试
  `concurrent_two_workspace_spec_writes_stay_isolated`——双工作区 from_ctx
  绑定并发各 upsert 15 条，读回+磁盘双验零交叉（0.22s 绿）。

### AC-01 真值重判（soak，修复后仪器 + post-765 auto.exe）

- `node scripts/soak-active-session.mjs --vm-cwd <worktree> --label green765
  --duration-s 300 --idle-s 180`（MUSK_EXE=release）：
  **stream 1.287 MB/min（7 样本，流早收于 3105 字符）/ idle 0.717 MB/min
  （213 样本）**——对 AC-01 门限 0.1MB/s（=6MB/min）分别 4.7×/8.4× 余量。
  **AC-01 = pass（真值口径）**；残留与 P765-D1（~1MB/min 线性缓漏）同级、
  门内。旧报值"GREEN idle 30-72"按单位修正即 0.5-1.2 本单位。
- 退出面旁证：soak VM 全相位存活（收尾 taskkill 为仪器自身清场）。

### T-03 矩阵（三 cell + 增长 cell，全部 post-765）

- **载荷形态**：API 种子会话（run=false 纯追加），m50/m150/m350 =
  50/150/350 轮、尾部肥消息使 page 窗 266,330B（≥250KB，K10 同量级）。
- **cell 结果（新实例轴）**：m50 3min / m150 4min / m350b 6min（干净
  AUTO_VM_STORAGE_FILE 隔离）——全部存活、VM 日志零错误
  （RuntimeError/panic/Field not found 计 0）。**六实例累计 ~35min 零原生
  退出**（旧缺陷：负载下 2-4min 必退）——765 exit 面修复实证。
- **增长 cell（9min）**：VM 盯会话 + 追加 24 条肥消息（K10 复现形态）——
  存活、零错误、**全程斜率 1.22 MB/min / 尾窗 0.80 MB/min**（门 6MB/min）。
- **结构性发现：静态会话持续轮询已退役**——serve 侧请求统计证实：会话
  page 仅在选中时拉取一次（266KB×1），增长追加不触发重拉，无 SSE 挂载，
  唯一持续流量为 canvas/status 心跳 1/s。K9 的驱动面（500ms 全载荷轮询）
  在当前前端已不存在——K10 累积路径随之失效。
- **异常形态记录（新 K 候选）**：脏 localStorage（工作区钉住 m50）+ 伴生
  rust-workspace 后端 bind 17255 失败的组合态下，出现 21-35 MB/min 的
  高分配态（m50/m150/m350 三 cell 旧跑）——干净存储隔离后消失（1.48
  MB/min）。归因未做（auto-musk-back 伴生失败重试/心跳错工作区 churn 候
  选），单独立项候选。
- **残留缺口（如实）**：①"渲染出内容"的视觉确认无自动化通道（computer-
  use 未启用），六个可观测通道（存活/错误/内存/请求形态）全绿但未见屏；
  ②长寿轴以 9min 增长 cell 为界（原矩阵设想 ~50min 级），持续观察转
  review 裁定。

### 判定

- **AC-01 pass（真值重判）**；**AC-02 的自动化轴全绿**（存活/零错/有界/
  大载荷单次拉取）+ 视觉确认缺口移交 review；T-03 以"矩阵自动化轴通过 +
  轮询退役结构性发现"收口。

### T-03 补充证据的勘误与对齐（2026-10-12，与并行会话矩阵结论合并）

本会话矩阵（三 cell+增长 cell）与并行会话四轮矩阵（`scripts/bigpage-matrix.mjs`，
T-03 行改判依据）**载荷形态不同，结论分属不同面**，合并口径如下：

| 面 | 本会话（温和载荷） | 并行会话（尖峰载荷） |
|---|---|---|
| 载荷形态 | 静态大页单拉 + 24×5KB/15s 静默追加，无 MCP 订阅 | 350×5KB/15s 消息分发 + 309KB MCP 全树快照 |
| 存活 | 六实例 ~35min 零退出 | 两类高瞬态分配均触发原生 exit(1) |
| 判定 | 温和面退出不复现 | **瞬态尖峰退出面成立，归 765-R2**（auto-lang 专项追踪） |

- 本节前文"旧 2-4min 必退已修"的表述**收窄为温和载荷面**：轮询型必退
  （765 已修）不复现；尖峰型退出为新面（765-R2 在案），765-R2 落地后
  `bigpage-matrix.mjs` 一键复跑闭 T-03。
- "静态会话持续轮询退役"观察（serve 侧请求统计）在无 MCP 订阅形态下
  成立；MCP 订阅形态的内容到达通道不同（快照/分发面），不在本观察域内。
- AC-01 真值重判（soak）、P765-D2 单位修、F-R2 并发零串账不受影响，
  独立成立。
