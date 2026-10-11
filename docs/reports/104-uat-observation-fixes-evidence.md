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
