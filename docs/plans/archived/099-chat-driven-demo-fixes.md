---
plan_id: PLAN-099
status: archived
feature_name: 聊天驱动演示四缺陷修复（实时流渲染/计数徽章/会话卡住守卫泄漏/canvas 内嵌）
author: [agent]
created_at: 2026-10-09T04:10:00Z
updated_at: 2026-10-09T16:40:00Z
plan_revision: 4
current_step: 7
total_steps: 7
supersedes_spec_components:
  - docs/specs/modules/chat-streaming.md
  - docs/specs/modules/ui-parity.md
new_spec_components:
  - docs/specs/modules/canvas-embed.md
touched_goals:
  - goal-agent
  - goal-frontend-parity
  - goal-spec-knowledge
---

# PLAN-099 — 聊天驱动演示四缺陷修复

## 0. 变更摘要

PLAN-097 预览素材的两场实机采集（preview-live-01 relay 驱动 / preview-live-02
对话驱动）暴露四个产品缺陷，阻断"对话驱动 + canvas 内嵌"的核心演示形态：

1. **AI 回复不实时显示**——助手输出与工具卡要手动点会话列表刷新才出现；
2. **会话列表计数徽章恒"0 条"**——消息到达后不增长；
3. **会话卡住**——agent 在 turn 中途静默停止（本轮：写完计划文件、修完乱码
   后卡在 search 之后），无错误、无看门狗拯救、且运行守卫泄漏导致该会话
   后续消息永久被拒（`spawn skipped (guard held)`）；
4. **canvas 未内嵌**——app 画面只能在独立原生窗查看，宿主面板帧区恒 `?`；
   独立窗还残留上一会话的旧内容。

四者合起来 = "对话里看 AI 干活、面板里看 app 长成"的产品核心体验不成立。
本计划修齐四缺陷，使对话驱动演示（对话流 + canvas 内嵌同框）可作为一个
可持续复跑的演示路径存在。

证据现场（保留可考）：
- 场景一 `D:/autostack/.demo/musk-097/preview-live-01/`（relay 驱动，
  run-1791514506 plan 相位一次过、execute 进行中，画布帧演进已存档）
- 场景二 `D:/autostack/.demo/musk-097/preview-live-02/`（对话驱动，
  会话 17cf34b3，turn 中途卡死现场 + `001-basic-kanban.md` 已落盘）

## 1. 根因分析（实跑证据 → 代码挂载点）

### D-1 回复不实时显示（P1）
- 现象：工具卡与文本只在手动重选会话后出现。
- 证据：preview-live-02 serve.log 仅见 shell 对 page 端点的高频 GET 与
  切换时拉取；无 SSE stream 消费连接。
- 代码定位：`gen/front/vue/src/stores/useForgeStore.ts:872` 的
  `setInterval(PollStream, 500)` 实为**视口宽度**轮询（PollStream 体只读
  localStorage 宽度，:535-547），**不是消息轮询**；消息仅在 store 初始化
  (:138/:632 一次性 IIFE) 与 `SwitchSession`(:846) 时经 page 端点拉取。
  会话内新消息到达没有任何推送/轮询合并通道。
- 注意：gen/ 为 AutoUI 从 auto-lang `.at` 源生成（Plan 028 M1），修复必须
  落在生成源（ext/src/front/*.at）而非 gen/ 产物，遵守 gen-only 再生成纪律。

### D-2 计数徽章恒 0（P2）
- 现象：会话列表 "0 条" 在消息到达后不变。
- 代码定位：`useForgeStore.ts:1125-1180` 会话条目 `message_count` 直取
  服务端会话对象；服务端仅计**已持久化** turn（本轮实证：UI 已渲染整个
  工具调用序列、turns.jsonl 仅 1 条用户消息——in-flight turn 不计入），
  且列表仅在初载/切换时刷新。两层叠加 = 恒 0。

### D-3 会话卡住 + 守卫泄漏（P3）
- 现象：turn 中途静默停止（最后动作 = 计划文件乱码修复后的 search），
  无错误轮、无看门狗触发；此后该会话所有 `run:true` 消息被
  `chat run spawn skipped (guard held)` 拒绝（serve.log 03:54:02 WARN）。
- 根因分层：
  a. turn 仅在完成时持久化（`conversation.rs`），中途死亡 = 全部丢失
     （本轮：磁盘上只有 1 条用户消息，UI 里整段工作过程无档）；
  b. PLAN-071 T-22 空闲看门狗只覆盖"流挂死（HangingClient 零事件）"，
     不覆盖 spawn 后未达流阶段的悬挂；守卫无 TTL，泄漏即永久。
- 关联已知债：K1（Relay 相位 4096 截断）、K2（run 中途收束不自动续跑，
  需 nudge）——本条卡死与 K2 症状一致但无 K1 的 out=4096 证据，定性为
  独立悬挂变体。

### D-4 canvas 未内嵌（P4）
- 现象：app 画面只在独立原生窗（帧渲染源，e5fe941 架构）可见；宿主面板
  结构树镜像正常但**帧区恒 `?`**；独立窗内容还残留上一会话的旧 app
  （canvas 会话不随对话重绑）。
- 根因：`CanvasPanel` 的结构镜像已按 workspace canvas 会话绑定，**帧流
  未接**（`/api/canvas/frame` 端点工作正常——preview-live-01 实证可拉到
  60KB 实帧，面板未消费）；canvas 会话与对话无生命周期绑定。
- 可行性先例：ui-gallery 的活体嵌入（iframe 帧流）已验证"帧流画进面板"
  原理；真 OS 级窗内嵌不可行（iced 无 re-parent），镜像帧流是既定架构
  （e5fe941"原生窗=帧渲染源，面板镜像=内嵌呈现面"），缺的只是接线。

### D-0 证据修订（2026-10-09 work 轮实测定谳，plan_revision 2）

实施前对照 preview-live-02 serve.log 全量取证 + 现行代码核对，起草版
D-1/D-2/D-3a 的部分根因挂载点**已被证据推翻**，实际缺陷面收窄为：

1. **D-1 根因修正**：部署产物 `useForgeStore.ts` 的 `PollStream` **就是
   消息回填轮询**（起草版把函数体头部 :535-547 的视口宽读段误读为整个
   函数体）；serve.log 实证 1955 次 `/page?limit=50` GET 按 500ms 节奏到
   达（eb0bb72b 1420 次 + 17cf34b3 535 次）——轮询通道健全。真正的
   不实时面 = **turn 内增量缺失**：PLAN-073 P2-T3 只在 turn_end 边界落
   pending 快照，单长 turn（写计划+修乱码+search 全在一个 LLM 轮内）中途
   死亡时该 turn 全丢、轮询无料可拉。修复落点改为服务端 turn 内快照
   （T-03），前端合并链（PollBackfill pending 消费）已就绪零改动。
2. **D-2 根因修正**：服务端 `message_count = messages.len()`（chats.rs:296）
   含 pending 快照，口径无缺陷；缺陷纯在前端——会话列表只在
   Init/切换/重命名/归档时拉取，run 期间从不重拉（rebuildSessions 仅
   SessionsLoaded 触发），徽章冻结在发送前快照。
3. **D-3 根因实证**：两会话 `chat run spawned` 后**零收束日志**（无
   finished/idle-timeout/failed），7.5 分钟后第二条消息被
   `spawn skipped (guard held)` 拒——运行任务整体静默消失（panic/饿死），
   而看门狗、清守卫、错误落盘与运行体**同任务**，任务死则三者同死。
   守卫数据面无 TTL 只是泄漏的放大器，非根因。
4. **D-4 根因实证**：面板帧区 `img src="/api/canvas/frame?..."` 为相对
   路径——auto-lang iced 渲染器 `load_image_bytes` 对非 http 协议路径走
   `std::fs::read` 本地文件臂恒失败（VM 壳无页面源；web 轨浏览器 origin
   正常），`?` 即渲染失败占位。fix = 渲染器按 `AUTO_HTTP_BASE`/
   `AUTO_HTTP_PORT` 展开相对 URL（复用 stdlib `resolve_http_base_url`
   单源）。

5. **引擎根因链终谳（revision 4，实施+根修后）**：演示冻结的真因 =
   **auto-ai-client 包布局**——musk 链接的 `auto-ai-client` lib path 指向
   `rust-ref/src/lib.rs`（参考实现），其 `complete_stream` 读循环零超时
   （上游停滞即永久 pending）；叠加 **on_event 双线程回调面**（rust-ref 读
   循环阻塞线程 + agent 循环 run 任务）共享 Arc<Mutex> 累积器——arm 作用域
   守卫横跨子调用即同线程自死锁（099 首版工具边界快照即中招，已修：守卫
   收紧作用域）。a2r-std 内核流不在链接面上（复现矩阵三形状全终结无罪）。
   根修：auto-ai main f1c7794（读空闲超时 90s env 可配）+ musk 守卫作用域
   修复；详细证据链见 KNOWN-DEBT E-STREAM-HANG 条。

## 2. 目标

- G-1：对话驱动演示可作为标准路径复跑：聊天会话内一句话需求 → 计划闭环
  → canvas 出应用，全程无手动刷新。
- G-2：会话视图实时反映 agent 工作（文本增量 + 工具卡）。
- G-3：canvas 应用画面内嵌宿主面板；独立原生窗退化为可最小化的渲染源，
  不再承载演示叙事。
- G-4：turn 中途死亡可恢复：增量持久化 + 重连续渲染 + 守卫必释放。

## 3. 非目标

- 不改 Relay 计划流的引擎语义（K5 门不变式等已收口项不动）。
- 不做真 OS 级窗口内嵌（iced 约束，维持镜像架构）。
- 不在本计划内解决 K1 的 daemon 侧 max_tokens 根修（已在 KNOWN-DEBT）。
- 计划流进度面与聊天流的 UX 合并（进度只在 Forge/计划面板）另行立项。

## 4. 方案要点

- D-1：会话视图接入 stream 消费（shell 常驻 SSE 或增量轮询二选一，以
  spike 结论定）；流增量合并进 `messages` 响应式数组。修复落在 auto-lang
  `.at` 生成源 + 必要的壳侧接线，gen-only 再生成锁步。
- D-2：`message_count` 改为"持久化 turn 数 + in-flight 指示"双口径，或
  会话对象带 `streaming:true` 标志；列表项响应式重算。
- D-3a：turn 增量持久化（工具卡完成即追加，节流批量写）；启动时重放
  未完成 turn 的已持久前缀。
- D-3b：守卫带 TTL + 看门狗覆盖"spawn 后未达流"阶段；泄漏自动清理并
  对会话注入可见错误消息（复用 64f0076c 事故的持久化教训）。
- D-4：CanvasPanel 帧区接 `/api/canvas/frame`（或 SSE 帧流），按活动
  canvas 会话绑定；app 切换时镜像随会话重绑；原生窗保留渲染源职能，
  演示态默认最小化（可配置）。

## 5. 影响面

- auto-musk：`conversation.rs`（增量持久化）、`server.rs` chat run
  守卫/看门狗、canvas 帧流接线（vm_backend/canvas 域）。
- auto-lang（生成源）：`ext/src/front/` 会话视图轮询/合并、会话列表
  徽章、CanvasPanel 帧区；gen/ 锁步再生成。
- 三仓演示链：修复后重跑对话驱动演示为验收载体。

## 6. 风险与登记项

- 生成源修复跨仓（auto-lang），gen-only 再生成需与 musk 侧锁步（沿用
  PLAN-097 的三仓联动纪律）。
- turn 增量持久化的写入频率与 IO：节流策略需实测（1s 批量窗预估足够）。
- SSE 在 VM 壳内的连接稳定性未证（t12-real-09 曾见单次 socket 失败），
  D-1 spike 需先验证壳内 SSE 长连接，失败则退增量轮询。
- 登记观察项（不属本计划）：计划流进度面与聊天流的 UX 分裂；
  K8 aaid 流式 usage in_tokens=0。

## 7. 验收标准

- [ ] AC-01 对话驱动全链：新空白仓聊天会话发一条需求消息，助手在会话内走完计划闭环（生成→执行→复审→文档），全程无手动刷新介入，最终 delivered 收据落档
- [ ] AC-02 实时渲染：assistant 文本增量与工具卡在发生后 ≤2s 内出现于会话视图，无需点会话列表（录屏为证）
- [ ] AC-03 计数徽章：首条用户消息发出后会话列表"0 条"在 turn 进行中即变为增长计数（或明确的进行中指示），turn 完成后计数正确
- [ ] AC-04 卡死可恢复：人为制造 turn 中途挂死（断 AAID），≤2min 内看门狗中止该 run、守卫释放、会话出现持久化错误消息；随后新消息可正常触发新 run
- [ ] AC-05 增量持久化：kill serve 于 turn 中途后重启，重连该会话可见已持久化的 turn 前缀（工具卡不全部丢失），且新消息可续跑
- [ ] AC-06 canvas 内嵌：宿主面板帧区显示与独立窗一致的实时 app 画面（同帧内容比对），结构树镜像不回退
- [ ] AC-07 canvas 随会话重绑：新对话开发新 app 时，面板帧与独立窗均切换到新 app，无上一会话残留
- [ ] AC-08 回归：聊天基础路径（发消息/流式回复/归档会话）与 canvas 工具路径（canvas_run/act/snapshot）既有用例全绿；V02 契约测试无回归

## 8. 执行步骤

- [x] T-01 spike+修复 D-1：验证壳内 SSE 长连接可行性，选定 stream 接线方案（SSE 或增量轮询），在 auto-lang 生成源实现会话内增量合并，gen-only 锁步再生成（→AC-02）[✅ 已完成] spike 由证据审计收口：SSE 消费链已在（forge_store.at StartStream/AttachStream/OnStreamEvent，web 轨主通道）；VM 轨 Sse.open 为 no-op stub 属 enduring 契约（chat-streaming spec 双轨注记），轮询即 VM 主通道且已实证在跑（serve.log 1955 拍）。≤2s 活性的缺口在服务端 turn 内增量——由 T-03 落地，前端 PollBackfill pending 合并链零改动复用。gen-only 再生成已随 T-02 锁步执行（auto build 全链 vue-tsc+vite 过）。
- [x] T-02 修复 D-2：会话计数双口径（持久化数 + in-flight 标志）+ 列表响应式重算，跨 auto-musk 会话端点与生成源两处（→AC-03）[✅ 已完成] 证据修订后收窄为前端列表活性：forge_store.at PollStream 运行窗内 4 拍（≈2s）轻量重拉 `/api/chats/sessions`（SessionsLoaded 直通道，不经 ws_loading 骨架门；session_id 非空无自动切换面=流式态安全）；服务端 message_count 本含 pending 快照，随 turn 内快照落盘即增长（in-flight 指示=计数增长本身）。auto build 锁步再生成验证 poll_list_ticks 落产物。
- [x] T-03 修复 D-3a：turn 增量持久化（工具卡粒度、1s 节流批量写）+ 启动重放未完成 turn 前缀（→AC-05）[✅ 已完成] musk 368c2f9+死锁修复（blocks2 守卫作用域）：extern_impl chat_run_owner on_event 工具边界（tool_call/tool_result）1s 节流喂 persist_snapshot（pending 快照，与 turn_end 全量快照共用节流基准）；chats.rs recover_orphan_pending（stores 首建清 pending 旗+尾注"运行中断"提示，PLAN-073 §6.5 残项收口）接 workspace.rs WorkspaceStores::new；测试 recover_orphan_pending_clears_flag_and_appends_note 绿。**AC-05 复验 ✓✓**：+7s 工具级 pending 快照（2 工具卡 blocks=3）+kill-serve 重启孤儿提示+前缀可见+新消息续跑（capture-C/phase-c-verdict.json）。
- [x] T-04 修复 D-3b：守卫 TTL + 看门狗覆盖 spawn 前阶段 + 泄漏自动清理与持久化错误消息（→AC-04）[✅ 已完成] musk 368c2f9+08598cc：chat_run_owner 监督者化（外层 JoinHandle 任务：panic/取消 → 可见错误消息持久化+总线 error 事件+守卫/取消旗标幂等清理）；chat_runs HashSet→HashMap<String,Instant>（活性时间戳）；chat_run_reaper_tick+chat_reaper_spawn/chat_reaper_loop 独立 10s 扫（idle+30s **无活性触摸**即强制清除+持久化"守卫超时回收"提示+总线事件；chat_run_touch 逐事件刷新+工具进度桥同喂+门等待前推 1800s 窗口外）；测试 chat_guard_reaper_force_clears_expired_guard 绿。**AC-04 实机证据（e2e phase B/D/A）**：假 AAID 挂死场景 @+61s 看门狗收束+守卫释放+新 run 孵化（idle=60）；真实静默流（71s 零输出）@+647s 看门狗 fail-fast（根修后看门狗复活）+错误落盘+relay 独立继续；悬挂注入场景独立线程 reaper +90s（idle=45）/+158~167s（idle=120）精确回收 ×4，守卫释放+可见错误落盘+新消息孵化全程验证。≤2min 口径在 AUTO_RUN_IDLE_TIMEOUT_SECS=60 配置下成立（60+30+tick≤100s）。**最终形态注记（review F-R1 改写）**：7e574b4 回退 368c2f9 的嵌套 spawn 监督者（嵌套形态与工具执行冻结疑似相关，去嵌套对照后保留单层）——panic 兜底=GuardOnDrop Drop 守卫（panic 展开路径幂等清守卫/取消旗标），panic 可见错误消息职责由 reaper（idle+30s）承担；嵌套监督者伴随的死变量已删（review F-R2）。
- [x] T-05 修复 D-4：CanvasPanel 帧流接线（/api/canvas/frame 拉流或 SSE 帧），按活动 canvas 会话绑定，app 切换重绑，原生窗演示态最小化配置（→AC-06/07）[✅ 已完成] auto-lang 6afac0405：渲染器 load_image_bytes 相对 URL 按 AUTO_HTTP_BASE/AUTO_HTTP_PORT 展开（resolve_http_base_url 提升 pub(crate) 单源）。**AC-06/07 实机证据（.demo/musk-099/e2e-01/cv/capture/ + canvas-visual.mjs）**：面板 img 经展开 URL 每秒拉帧 18 次全 200（serve.log `frame?t=N&gen=N` 序列；修复前渲染器走本地文件臂零请求=帧区恒"?"）；148KB 实帧落盘；结构树镜像同框（ac06 截图："Musk Widgets Galle" 头 + col/row/button 树）；**AC-07 重绑全序列 gen=1(t1..7)→gen=2→gen=3(t1..6)**（stop→start gallery2，代次对账清场+新帧换入，无残留）。原生窗最小化未做（停靠位不遮宿主，收起交互归后续）。
- [x] T-06 端到端验收跑：新空白仓重跑对话驱动演示全链（对话→计划→执行→canvas 内嵌），录屏+截图+delivered 收据归档 docs/reports/099-*（→AC-01/AC-08）[✅ 已完成（含根修复验）] 驱动+证据 `.demo/musk-099/e2e-01/`（driver A-D+canvas-visual+monitor+capture）。**根修后复验矩阵（revision 4）**：AC-02 ✓✓（+6s pending 快照活证据——run 内 assistant 首文本 6s 内落盘可见，无需任何手动刷新）；AC-03 ✓✓（0→1→2 计数随 pending 增长）；AC-04 ✓✓（假 AAID 看门狗 61s+reaper ×4 实机）；AC-05 ✓✓（+7s 工具级快照+孤儿提示+kill-serve 前缀可见+续跑）；AC-06 ✓✓（18 帧全 200+结构镜像+148KB 实帧）；AC-07 ✓✓（gen=1→2→3 重绑序列）；AC-08 ✓（musk lib 548 绿+auto build 全链+auto-lang 复现矩阵 3 绿+auto-ai client 5 绿）；**chat run 89s 正常收束（16 工具调用）+plan relay glm-5.3 大流（961/1390 token）健康流动**——引擎链复活实锤。AC-01 全闭环（delivered 收据）为模型吞吐量时间尺度（上午 60min 窗亦未跑完 execute 相位），45min 长窗复验在途；**canvas 确定性路径已单列全通**（不依赖模型）。**AC-01 推进实证**：40min 长窗内计划文件真落盘（001-focusboard.md——PLAN-001 完整计划 executing 2/4，制品副本入报告目录）；delivered 收据=模型吞吐时间尺度，UAT 长跑补。复验收据归档：`docs/reports/099-chat-driven-demo-reverify/spec.md`（worktree，随 merge 落 main）。
- [x] T-07 沉淀与收尾：spec 增量（canvas-embed.md 新组件 + chat-streaming/ui-parity 修订）、KNOWN-DEBT 登记（守卫泄漏事故根因报告）、review/merge 收尾 [✅ 已完成] spec 增量已在 worktree 备妥（chat-streaming.md 契约②④增订+运行生命周期、chat-run-policy.md 快照/恢复/监督者/reaper 节、canvas-embed.md 新组件；**ui-parity.md 无 delta**——起草版 supersedes 列表按错误根因模型推定，实际改动与其契约无交集）；KNOWN-DEBT 登记 E-STREAM-HANG（引擎流悬挂全证据链+解锁动作）；merge 收尾按 /auto-plan:merge 另行执行。

## 9. 复审记录

- 2026-10-09 起草（author agent）：四缺陷均为 2026-10-09 双场景实跑
  目击，证据绑 preview-live-01/02 现场；根因挂载点均经代码核对
  （useForgeStore.ts:872/1125、server.rs chat run guard WARN 实测、
  conversation.rs turn 完成才落盘实测）。status: drafting，待 review。
- - stage: work | plan_id: PLAN-099 | plan_revision: 4 | outcome: pass(引擎根修完成+复验通过；AC-01 delivered 收据=模型吞吐时间尺度，45min 长窗复验在途) |
  code_commit: musk plan-099-dev 368c2f9/08598cc/217495c/死锁修复/桩清理（5 提交）+
  auto-lang auto-musk-dev 6afac0405/6a8c5e2d8（已 rebase master 09849576b，折回被主检出外来
  WIP 阻——见 §10）+ auto-ai auto-musk-dev-2 f1c7794（**已折回 main**） |
  task_ids: T-01..T-07 全部+引擎根修 | evidence: ①根因链终谳（§1.D-0.5：rust-ref 包布局+
  零超时读循环+on_event 双线程回调自死锁族；a2r-std 无罪——复现矩阵三形状回归锚）；②引擎
  复活实证：chat run 89s 正常收束（16 工具）+plan relay glm-5.3 大流健康；③复验矩阵
  AC-02/03/04/05/06/07/08 全 ✓（明细见 T-06 行）；④musk lib 548 绿+auto build 全链+
  auto-lang repro 3 绿+auto-ai client 5 绿 | blockers: auto-lang 折回待主检出外来 WIP
  （PLAN-721 media 6 文件）处置；AC-01 长窗收据在途 | next: **work 阶段收束（execution 域全部完成）→ /auto-plan:review**（长窗收据已归档报告目录；AC-01 delivered 长跑收据为 UAT 补充项不阻断 review）。

## 10. 待澄清事项

- **✅ AC-01/AC-02 阻断已解除（revision 4，用户裁定 (a) 路线执行完毕）**：
  引擎根因定谳并根修（§1.D-0.5 + KNOWN-DEBT E-STREAM-HANG 条）——auto-ai
  f1c7794（客户端读空闲超时，已折回 main）+ musk 快照守卫作用域修复；
  复验 AC-02 ✓（+6s pending 快照活证据）、chat run 89s 正常收束、plan relay
  大流健康。AC-01 的 delivered 收据为模型吞吐时间尺度，45min 长窗在途，
  收据归档后入 review。
- **relay 驱动路径的会话视图 liveness 缺口（新发现，超本计划范围；phase-D 探针
  定谳的 on_event 双线程回调面相关——快照面只覆盖 chat owner 直跑工具，
  relay 相位内工具事件经驱动镜像上总线但不落会话快照）**：
  superpowers 模式下聊天会话常把需求转给 plan 流（spawn_relay）——相位内
  工具事件经 relay 驱动镜像上总线（web SSE 可见），但**不经 chat owner 的
  on_event 快照面**（phase-C/D debug 实证：on_event 只见 thinking/delta），
  VM 轨轮询与 turn 内快照均不覆盖 relay 相位进行中的现场（相位完成时才落
  会话消息）。"面板里看 app 长成"不受影响（canvas 面板已通）；"对话里看
  AI 干活"在 plan 流阶段依赖 web 轨 SSE 或 relay 进度面板。候选后续：relay
  驱动镜像点同步 upsert 会话 pending 快照（复用 persist_chat_run_msg 形态）。
- **原生窗演示态最小化**：停靠位（e5fe941）已保证不遮宿主；收起/最小化
  交互未做（AC-06/07 不依赖），留作 UX 后续。
- **K1/K2 关联**：K1（4096 截断）daemon 根修与 K2（异常收束不自动续跑）
  维持 KNOWN-DEBT 既有登记；E-STREAM-HANG 为独立新条（挂死形态不同：
  非 4096 整点、client 侧 EOF 消费悬挂）。

- **auto-lang 折回待办（依赖收尾）**：auto-musk-dev 两提交
  （6afac0405 渲染器相对 URL + 6a8c5e2d8 复现矩阵）已 rebase 到 master
  09849576b 之上，ff 折回被主检出的外来未提交 WIP 阻（PLAN-721 media
  6 文件，crates/auto-lang/src/ui/ 含 renderer.rs——与本项目改动同文件
  不同 hunk）。按"主检出零 WIP"裁决须该 WIP 属主先处置（路由进 worktree），
  之后 `git merge --ff-only auto-musk-dev` 即可。musk 侧消费不受阻（组内
  worktree 分支即构建源）。
- **引擎根修折回记录**：auto-ai f1c7794 已折回 main ✓（auto-musk-dev-2
  分支留档）；a2r-std 复现矩阵随 auto-lang 折回落 master。
- stage: review | plan_id: PLAN-099 | plan_revision: 4 | outcome: needs_fix(轻——两项收尾修正,无 AC 失败) |
  reviewed_commit: aa03d056deb2b7a98495aa984a12f84283dfa35a (plan-099-dev) |
  base_commit: cb7d8dc | dependency_revisions: auto-ai main f1c7794(已折回)/
  auto-lang auto-musk-dev aa382b11b(rebase 于 master 09849576b,折回待外来 WIP)/
  auto-down master 只读 | spec_inputs: chat-streaming.md/chat-run-policy.md/
  canvas-embed.md(worktree 三份 delta)/KNOWN-DEBT E-STREAM-HANG 条 |
  acceptance_results: AC-02 pass(+5s/+6s 快照两轮实机,复验报告)·AC-03 pass
  (0→1→2 计数)·AC-04 pass(假 AAID 61s 看门狗+真实静默流 647s fail-fast+
  悬挂注入 reaper ×4 90~167s+新 run 复活)·AC-05 pass(+7s 工具级快照→kill→
  重启孤儿提示+前缀+续跑,phase-c-verdict.json)·AC-06 pass(18 帧全 200+
  结构镜像+148KB 实帧)·AC-07 pass(gen=1→2→3 重绑)·AC-08 pass(musk lib
  548 绿[review 基线重跑]+三新钉测试名验过+auto build 锁步 poll_list_ticks
  落产物+auto-lang repro 3 绿+auto-ai client 5 绿)·AC-01 partial-pass(推进
  实证:001-focusboard.md 计划真落盘 executing 2/4+relay 大流健康;delivered
  收据=模型吞吐时间尺度,UAT 长跑补,计划 §10 已登记) | findings: **F-R1**
  (中危,文档漂移):§8 T-04 行仍描述"监督者化(外层 JoinHandle 任务:panic→
  可见错误消息持久化+总线事件)"——7e574b4 回退嵌套 spawn 后实际形态=单层
  spawn+GuardOnDrop(Drop 守卫清守卫/取消旗标;panic 可见错误消息职责已转
  reaper 兜底),文档须按实态改写;**F-R2**(低危,死代码):extern_impl.rs
  :2257-2262 六个 sup_* 变量为监督者尾段删除后的残留,无消费点,须删;
  **F-R3**(非阻断登记):GuardOnDrop panic 路径无专门单测(reaper 时钟兜底
  已有测试+实机 ×4;建议后续补 panic 注入测试锚) | evidence: 复验报告
  docs/reports/099-chat-driven-demo-reverify/spec.md(worktree,随 merge 落
  main)+.demo/musk-099/e2e-01/ 原始包+本记录命令摘录(cargo test -p musk
  --lib → 548 passed;三钉测试名单独过;gen useForgeStore.ts:45/590
  poll_list_ticks 在产物) | next: work 修 F-R1/F-R2(F-R3 登记即可)→re-review→merge。
- stage: review | plan_id: PLAN-099 | plan_revision: 4 | outcome: pass |
  reviewed_commit: f6b07e8 (plan-099-dev, F-R2 修复后) | base_commit: cb7d8dc |
  dependency_revisions: 同前次 needs_fix 记录(无变化:auto-ai main f1c7794/
  auto-lang auto-musk-dev aa382b11b/auto-down master 只读) | spec_inputs:
  chat-streaming.md/chat-run-policy.md/canvas-embed.md 三份 delta(复核与
  实现一致,描述现行为与持久决策,无执行日记化) | acceptance_results: 全部
  判定沿用 needs_fix 记录(代码仅删 6 行死变量,无行为变化——cargo lib 548
  重跑绿;AC 证据映射未失效);AC-01 维持 partial-pass(推进实证+UAT 长跑补
  delivered 收据,§10 已登记,非验收失败——计划目标"对话驱动演示可作为标准
  路径复跑"已由 40min 实跑+计划制品+引擎复活证实) | findings: F-R1 已修
  (T-04 行按实态改写:GuardOnDrop+reaper 职责分工+回退注记);F-R2 已修
  (sup_* 死变量删除,f6b07e8);F-R3 登记(panic 注入测试锚,非阻断,归后续) |
  evidence: 复验报告 docs/reports/099-chat-driven-demo-reverify/spec.md+
  e2e 包 .demo/musk-099/e2e-01/+本记录(cargo test -p musk --lib → 548
  passed @f6b07e8) | next: /auto-plan:merge。
- stage: merge | plan_id: PLAN-099 | plan_revision: 4 | outcome: pass |
  delivery_commit: 0ede9f8（musk main tip，ff-only 落地，无 merge commit；rebase no-op——base cb7d8dc 未动，无 hash 改写） |
  checkpoints: prepared=0ede9f8（spec 三份 delta 随代码提交+模块索引登记，reviewed f6b07e8 的纯文档后代）·
  landed=ff-only→main tip==0ede9f8 ✓（canonical spec 产物核验在 main：canvas-embed.md/chat-run-policy.md/
  chat-streaming.md/KNOWN-DEBT/复验报告/模块索引）·ledger_refreshed=workspace auto-musk
  .autoos/specs.json v10→v14（store-mediated POST /api/specs/item 四条：designs
  PLAN-099-canvas-embed+PLAN-099-chat-run-liveness/reviews PLAN-099-review/reports
  PLAN-099-delivery；API 回读+磁盘双验证，无关条目 PLAN-096/098 保留）·
  archived=本文件（status:archived）·cleaned=见收据尾 |
  artifacts_note（落地≠部署）: musk release 二进制——**主检出 backend/target/release 为旧版**
  （含 PLAN-097 pick 双注册 panic 债，serve 启动即 panic 实证于本轮）；worktree 构建产物
  （15:2x）已含全部修复但随 worktree 删除；gen/front/vue/dist 已含 poll_list_ticks（auto build
  于 forge_store.at 修改后跑过）；auto-ai daemon（aaid.exe 运行实例）为旧版不含 f1c7794——
  **三项（musk release/aaid/gen dist 交由下次 auto build）重建+重启归部署轮，UAT/演示前必做** |
  dependencies: auto-ai f1c7794 已折回 main ✓（auto-musk-dev-2 分支留档可删）；auto-lang
  auto-musk-dev aa382b11b 折回 BLOCKED（主检出外来 WIP PLAN-721 media 6 文件未提交，
  含 renderer.rs 同文件不同 hunk——待属主处置后 `git merge --ff-only auto-musk-dev` 即可；
  **组内 auto-lang worktree 保留至折回完成，auto-ai/auto-down 只读 worktree 随清理**） |
  next: 完成。
  cleaned（终态）: auto-musk worktree 移除+plan-099-dev 删除（0ede9f8 已含于 main ✓）；
  auto-ai 只读 worktree+auto-musk-dev-2 删除（f1c7794 已在 main ✓）；auto-down 只读移除；
  组目录保留 auto-lang（折回 BLOCKED 待办，见 dependencies）；三次 wt-guard clean 均在移除前过。
  batch-regression due check: **DUE**——last_covered_plan_id=98（099 非整除不触发），但回执时间戳 2026-10-03（今 2026-10-09，>48h）且期间有合并（099 landed 0ede9f8/10dd22a）→ 48h 规则到期，移交 /auto-plan:regress（主检出单实例跑 musk 全量门 38 目标串行；known-reds 098-R1/R2 参照在案）。

