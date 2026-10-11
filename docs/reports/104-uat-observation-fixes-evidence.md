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

## 环境指纹

- musk worktree plan-104-dev @ base 28f3415（main 并发推进中，落地前 rebase）；
  T-05=09d9d0e、T-04=eba12de。
- 依赖只读兄弟：auto-ai@f1c7794、auto-lang@dde29cf2c、auto-down@895f8d0
  （detached；三仓 untracked Cargo.lock 为 build.rs 记账补拷，wt-guard 前
  须移除——merge 清单在案）。
- 代码生成/链接门禁二进制：auto-lang release auto.exe（10-10 构建，
  PLAN-760 登记落后观察项；.at 语法面无涉本次改动语义）。
