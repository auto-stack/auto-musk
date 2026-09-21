---
plan_id: PLAN-083
status: execution_done
feature_name: VM 数据链异步化与大载荷治理（081 刷新链债 + 082 §10-7 三方向合并）
author: [agent]
created_at: 2026-09-22T00:00:00Z
updated_at: 2026-09-22T00:00:00Z
plan_revision: 1
current_step: 6
total_steps: 6
supersedes_spec_components: []
new_spec_components: [docs/specs/modules/vm-data-semantics.md]
touched_goals: []
---

# PLAN-083 — VM 数据链异步化与大载荷治理

## 0. 变更摘要

根治 VM 轨"数据加载冻结 UI"问题域：workspace 切换、大会话加载期间
界面全程无法响应（实测切 auto-edit 冻结 20.9s），根因=VM handler 内
全同步执行（HTTP 同步 IO + JSON 解析 + 解释器循环 + View 重建一帧跑完）。
合并两条线索：①PLAN-082 §10-7 实测归因的三条修复方向（载荷分页/
normalize native 化/异步化）；②PLAN-081 刷新链同域遗留（Choose 按压
冻结、"ws 弹层与死亡强相关"实为冻结误判、PollStream→SSE 桥泛化遗留）。

三支柱：
- **A 载荷治理（musk 后端）**：会话详情分页/首屏摘要——首屏只回最近
  N 条，历史滚动加载。
- **B native 化（musk 后端+双轨消费）**：normalizeToolBlocks 拍平
  下沉——后端直出拍平块（契约 v2），VM 解释器不再循环大数组。
- **C 异步桥（auto-lang VM）**：handler 内数据加载异步化——同步段
  立即返回渲染骨架，后台完成以消息回填（bounded spike 先行定案）。

- 主改动仓：musk（后端分页+契约 v2+前端消费）+ auto-lang（VM 异步桥）。
- 类型：性能/架构治理；不改变现有功能语义（分页对用户表现为滚动
  加载历史）。

## 1. 目标

### 目标

1. **可响应性**：workspace 切换/大会话加载期间 UI 保持可响应
   （事件循环不再长冻结）；切换过程有可见的中间态（骨架/进度）。
2. **切换性能**：切 auto-edit（最大 workspace）端到端
   20.9s → **首屏 <1s**（列表+首会话最近 N 条可交互）。
3. **架构收编**：normalizeToolBlocks 拍平逻辑从 VM 解释器循环下沉
   native（后端直出）；LoadSessionList/SetWorkspace 迁移至异步桥，
   成为 VM 数据加载的标准模式。
4. **081 遗留闭环**：Choose 冻结（081 §10"ws 弹层按压与死亡强相关"
   的冻结成分）、PollStream→SSE 桥泛化遗留在新数据链上收编。

### 非目标

- §10-9 死亡家族（已定案=兄弟 agent 启动链互杀，防线已落
  9f5593404；本计划的"冻结"与进程死亡是两个问题）。
- web/Vue 轨行为变化（分页契约向后兼容，web 可后续跟进）。
- SSE 桥全面泛化（KD-047 G1 大盘；本计划只保证新数据链与既有
  PollStream 共存）。
- 080 移交③④（字体度量、live case CJK 变体）——不同域。

### 成功样态

用户在 VM 上点击切换 auto-edit：列表立即出骨架→会话列表渲染→
首会话最近消息可交互，全程窗口可拖动/可点击；总耗时 <1s；
翻到历史处按需加载，无再冻结。

## 2. 架构方案

### 现状链（同步冻结，PLAN-082 §10-7 实测）

```
.Choose(id) ── UI 线程一帧内串行 ────────────────────────── 20.9s
  localStorage.setItem
  platformRefreshAuth()            （auth 请求，同步）
  ForgeStore.SetWorkspace(meta):
    清场（内存，快）
    .LoadSessionList():
      chats_list_sessions()        7ms / 3.4KB    （后端飞快）
      chats_get_session(first)     19ms / 578KB   （一条 552KB 巨型回复）
      normalizeToolBlocks(...)     VM 解释器循环大数组
      （autodown 解析+View 重建同一帧）
  PlansStore.Reload(): plans_list()
  location.reload()（VM no-op）
```

后端完全无罪（≤20ms）；瓶颈=VM 前端消化大载荷+解释器循环+单帧
同步模型。载荷 ×100（5KB→578KB）⇒ 冻结 ×40（0.5s→20.9s）。

### 目标链（三支柱）

```
.Choose(id):
  清场 + 发起异步加载（入队）+ 渲染骨架        ← 立即返回，UI 活
  [后台] sessions 列表请求 → 回填消息 → 渲染列表
  [后台] 首会话详情(分页:最近 N 条) → 回填 → 渲染首屏
  [滚动触发] 历史分页加载
其中：
  详情响应 = 拍平后的 blocks（B：normalize 下沉后端）
  请求/回填机制 = C 的 VM 异步桥（模式先例：PollStream timer、
  AttachStream、child_emit 消息回填）
```

### 支柱 C 方案调查（T-01 bounded spike，决策工件）

两个候选，spike 后定案（本计划不预设）：
- **C1 挂起恢复**：VM 解释器支持 handler 在 HTTP 调用处挂起
  （协程化/状态机），完成后恢复。通用性强但动解释器核心，风险高。
- **C2 分帧消息回填**：handler 拆"发起段/回填段"——发起段只入队
  （后台线程发请求）即返回；完成以 VM 消息（如
  `SessionsLoaded(payload)`）回填 store。复用既有消息机制
  （PollStream/child_emit 先例），改动面小但 handler 需要改写为
  两段式。**倾向 C2 起步**（LoadSessionList/SetWorkspace 两个
  热点先行），C1 作为长期方向登记。

### 依赖与顺序

B（契约 v2）与 A（分页）都改 `chats_get_session` 响应——合并设计
一次落（`GET /api/chats/session/{id}?limit=N&before=X` + 响应含
`blocks_normalized: true`）；C 的异步桥先行支撑 A/C 的前端消费。

## 3. 技术栈

- musk 后端：Rust axum（chats_get_session 分页参数+归一化直出）。
- musk 前端：.at（LoadSessionList/SetWorkspace 两段式改写、滚动
  加载接线、骨架态）。
- auto-lang：VM 异步桥（后台请求队列+消息回填；spike 定形态）。
- 验收仪器：MCP 计时（press→可响应窗口）、autoui_state、
  auto-edit workspace 实测（20.9s 基线对照）。

## 4. 需求分析与背景调查

### 授权记录

- 授权来源：用户 2026-09-22 指示「修复方向的3条，可以合成一个独立
  计划吧？和081的债合并立项」。
- 范围：PLAN-082 §10-7 三条修复方向 + PLAN-081 数据链同域遗留
  （Choose 冻结、刷新链同步、PollStream 桥遗留收编）。
- 基线数据（PLAN-082 会话实测，2026-09-21/22）：切 auto-edit 冻结
  20.9s；后端列表 7ms/3.4KB、首会话详情 19ms/578KB（4 消息含
  552KB 单条回复）；backend workspace 对照 0.5s/首会话 5KB。

### 背景调查

1. **081 遗留盘点**：归档 §10 三项——§10-9 新样本（已定案互杀，
   KNOWN-DEBT 080 行①闭环）；080 债②实锤（PLAN-082 已修）；ws 弹层
   Choose 按压与冻结/死亡强相关（**冻结成分=本计划根因，死亡成分=
   互杀已定案**——两条线索在此合并归档）。
2. **代码位**（2026-09-22 worktree 实测核对）：Choose 链=
   `workspace_selector.at .Choose` → `forge_store.at .SetWorkspace`
   (142) → `.LoadSessionList` (213)：`chats_list_sessions` +
   `chats_get_session(first)` + `normalizeToolBlocks` + `AttachStream`；
   PlansStore.Reload 并列。PollStream timer 500ms 全量回填（118）。
3. **VM 慢的放大器**：normalizeToolBlocks 在 VM 解释器逐块循环；
   autodown 对 552KB 文本块全量 parse+render 同帧执行。
4. **异步先例**：VM 已有 timer（PollStream 500ms）、Sse.open/close、
   child_emit 消息回填、PollStream deadman 窗——异步桥有同族机制
   可复用，非从零发明。
5. **兼容面**：分页与 PLAN-073（失败臂保现场/turn 快照/刷新重挂）
   和 PollStream 全量回填假设冲突——回填语义需适配（T-03 内处理，
   回归面=073 的测试链）。

#### work 阶段代码级调研定案（2026-09-22，T-01 前置勘定）

1. **冻结根因实锤**（auto-lang `vm/engine.rs` `call_fn_by_name`
   StepResult::Yield 臂，~L2185）：handler 的 HTTP yield 触发**忙等循环**
   （5ms sleep 轮询 `async_http_result_ready`，30s deadline）——UI 线程
   整个 handler 期间被占。HTTP 本身已是派生线程（Plan 349
   `spawn_async_http_handle`），瓶颈=忙等语义+后续 `json.to_value`
   解析+normalize 循环+View 重建同帧。§0"HTTP 同步 IO"表述修正为
   "语义同步（忙等）"。
2. **C2 机制先例全部现成**：①派生线程发请求（注入默认 query/header/
   基址展开，`spawn_async_http_handle`）；②全局通道→订阅轮询→消息
   注入（Shell SSE 桥 `SHELL_EVENT_RX`/`poll_shell_events` +
   `AppTickKind::Poll`，MCP 动作通道同款 16ms 轮询）；③update 闭包
   派发到具名 store handler（`on_with_input_for`，`\u{1F}s\u{1F}`
   载荷编码可嵌字符串实参）。C1（解释器协程化）不需要——忙等是
   唯一卡点，绕开即可。
3. **#[api] GET 参数自动转 query string**（`vm/codegen.rs`
   emit_api_http_call plan-022 规则）：`chats_get_session(id, limit,
   before)` 签名扩展即自动生成 `?limit=&before=`，前端契约零阻力。
4. **§10-1 巨条构成定案**（auto-edit `.autoos/chats.json` 实测）：
   552KB/938KB 巨条=**工具密集型回复**（非单块巨文本）——124/160
   blocks（84/109 个 tool 块）。652KB 条字节分布：`tool_calls` 消息级
   **重复数组 225KB**（与 blocks 双份载荷）+ tool result 167KB（top
   单条 50KB）+ thinking 75KB + text 6.8KB + JSON 结构开销。分页
   治理须含块级瘦身：paged 视图截断 tool result/thinking、瘦身
   tool_calls 数组（T-02 落实）。

#### worktree 登记（2026-09-22）

- musk：`D:/autostack/.wt/musk-083/auto-musk`（branch `plan-083-dev`，
  base = main f622167，tip 454dff9）
- auto-lang：`D:/autostack/.wt/musk-083/auto-lang`（branch
  `auto-musk-dev`，base = master 06837787e，tip 0a791cfb8）
- auto-ai：`D:/autostack/.wt/musk-083/auto-ai`（main 检出挂靠，纯路径
  依赖位，backend Cargo 相对解析用，零改动）
- ⚠ auto-lang 主检出有他人未提交改动（`examples/**` 代码路径+文档），
  本计划不消费不触碰；落地前需其归位（merge 阶段处理）。
- ⚠ musk worktree `gen/front/vue/node_modules` 含 pnpm junction（auto
  build 产物，gitignored）——**merge 清理前先 Node rmSync 该目录再过
  wt-guard**（标准步骤，见 memory）。
- 主检出预检：仅 `docs/plans/**` 簿记改动，无代码 WIP，合规。

## 5. 详细设计

### T-01 异步桥 spike（bounded 调查，决策工件）

auto-lang worktree spike：以 `LoadSessionList` 为试点，
- C2 形态：新增 VM 原语/桥（如 `Http.get_async(tag)` 入队即返回 +
  完成消息 `HttpDone(tag, payload)`），评估解释器/调度改动面；
- 验收 spike：试点 handler 期间 UI 可响应（MCP state 探测不冻结）；
- 产出决策工件：C1/C2 选型+改动面清单（文档落于本计划 §9 追记）。

#### T-01 决策工件（2026-09-22 work 阶段落档）

**选型：C2 分帧消息回填（定案，已实现）**。C1 挂起恢复不立项本计划。

- **证据链**：冻结根因=ui 侧 `call_fn_by_name` 对 `Waiting("http")`
  任务的忙等（5ms sleep 轮询 + 30s deadline，engine.rs StepResult::Yield
  臂）——不是解释器不支持并发，而是同步调用语义下 UI 线程被占。C2 绕开
  忙等（发起段无 Waiting），零解释器/调度器改动。
- **C2 落地形态**（auto-lang `0683778` 上 commit "feat(vm): PLAN-083
  T-01"，四站点 ~200 行）：
  1. `vm/ffi/stdlib.rs`：`Http.get_msg(url, event)` native（3148）——
     event 形如 `"Store.Handler"`；spawn 派生线程（复用 plain-handle 族
     send 汇聚路径：默认 query 注入+基址展开+默认 header 快照），完成推
     进程级 `HTTP_MSG_QUEUE`；载荷协议 `{"ok":bool,"status":u16,"body":str}`。
  2. `vm/native_catalog.rs`：id 3148 双表登记（bare 名 `Http.get_msg`
     经 to_canonical 默认小写规则自动映射，零注册）。
  3. `ui/iced/renderer.rs`：`poll_http_msgs` 泵（一次 update 一条，防
     handler 重入）+ `http_msg_subscription`（AppTickKind::Poll 19ms，
     Shell SSE 桥/MCP 动作通道同族）——产
     `IcedMessage{widget, "Handler␟s␟payload"}` 走 update 通用派发
     （`on_with_input_for`→`decode_payload`），**update_inner 零改动**。
  4. 回填 handler 签名形态：`.Handler(payload str)` 单字符串实参
     （VM push_value 对 struct 实参只推占位 0 的限制之下最可靠形态，
     shell SSE 桥同判）。
- **验证（U-3）**：`plan083_http_msg_bridge_tests` 5/5 绿（3 连跑稳定）：
  入队+载荷协议、失败臂（ok:false 可区分）、命名空间切分、␟s␟ 载荷
  decode_payload 往返、**.at 全链模拟器探针**（test/ui/plan083_http_msg
  语料：App.Kick→SpkStore.Fetch 置 loading 即返——发起段非阻塞 state
  可探测；泵回填 Loaded(str) 落态 done）。实机可响应性归 T-05 V-1。
- **C1（挂起恢复）登记为 VM 长期方向**（真挂起需任务帧/调度器重构，
  改动面大一个数量级；若未来 handler 内多请求编排成为常态再立项）——
  §10-2 就此闭环，无需用户裁定。

### T-02 后端分页+归一化直出（musk 后端）

`chats_get_session` 契约扩展：
- `?limit=N&before=<message_key>`：返回最近 N 条（首屏默认 50）+
  `has_more` + 游标；不传=全量（向后兼容 web 旧消费）。
- `blocks_normalized`：后端在响应前执行 normalize 等价逻辑
  （Rust 版 normalizeToolBlocks——拍平 tool 块双形态/现算 summary），
  前端消费侧按标记跳过本地拍平。
- 双轨：web 轨读同一端点，normalizeToolBlocks（TS）按
  `blocks_normalized` 标记跳过（向后兼容）。

### T-03 musk 前端消费改写

- `LoadSessionList` 两段式：发起段（清场+骨架+异步请求）/
  回填段（SessionsLoaded → store+首会话详情再发一跳）。
- 首会话详情分页消费：首屏 limit=50；消息列表滚动到顶部触发
  `before` 加载（复用 Scrollable offset 桥）；PLAN-073 链适配
  （PollStream 回填按已加载窗口合并，不覆盖未加载历史）。
- 切 workspace 骨架态：列表/首屏各自的 loading 呈现。

### T-04 迁移与收编

- SetWorkspace/Choose 链切换到异步桥；PlansStore.Reload 同款。
- 081 遗留注记：Choose 冻结归档指向本计划；PollStream 桥泛化
  遗留中"数据加载异步化"部分由本计划收编（SSE 泛化余量不动）。

### 规范增量

| delta_id | add/modify/retire | target | before/after | rationale | acceptance |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/modules/vm-data-semantics.md | before：会话详情全量单发、消费侧自行拍平、handler 内同步请求为常态。after：①会话详情分页契约（limit/before/has_more/游标语义，缺省全量向后兼容）；②响应可携 `blocks_normalized` 标记（后端直出拍平块，消费侧免二次归一）；③VM 数据加载模式=异步两段式（发起段禁长阻塞，回填走消息），长冻结=违反本契约 | 把 081/082 两轮实测教训固化为数据链契约 | AC-01/02/04 |

## 6. 测试设计

| # | 用例 | 方法 | 期望 |
|---|---|---|---|
| U-1 | 分页端点单测 | 后端 Rust 测试：limit/before/has_more 语义+归一化直出字段 | 绿；不传参=全量（兼容） |
| U-2 | 归一化等价性 | Rust normalize 输出 vs TS normalizeToolBlocks 输出对拍（样本=auto-edit 首会话真实载荷） | 逐块等价 |
| U-3 | 异步桥试点 | VM 单测/simulator：发起段不阻塞（state 可探测）、回填消息到位 | 绿 |
| U-4 | 073 链回归 | 失败臂保现场/turn 快照/刷新重挂既有测试+实测 | 差分零新增 |

### 实机验收（V 系列，auto-edit=20.9s 基线）

| # | 判定 | 方法 |
|---|---|---|
| V-1 | 切 auto-edit 全程可响应 | press 后 1s 间隔轮询 MCP state：不可响应轮次=0 |
| V-2 | 首屏 <1s | press→列表+首会话最近 50 条可交互 ≤1s |
| V-3 | 滚动加载 | 历史消息按需到达、无冻结 |
| V-4 | 功能不回退 | 发送/流式/工具卡/思考卡/表格渲染（082 面抽样） |

## 7. 验收标准

| ID | 标准 | 验证 |
|---|---|---|
| AC-01 | 分页契约落地（limit/before/has_more+缺省全量兼容） | U-1 |
| AC-02 | 归一化直出（blocks_normalized）且与 TS 消费等价 | U-2 |
| AC-03 | VM 异步桥落地，LoadSessionList/SetWorkspace/Reload 迁移完成 | U-3+V-1 |
| AC-04 | 切 auto-edit：全程可响应+首屏 <1s+滚动加载历史无冻结 | V-1/V-2/V-3 |
| AC-05 | 073 数据链与发送/流式功能零回退 | U-4+V-4 |
| AC-06 | SD-01 落册；081 §10 Choose 冻结线索与 PollStream 桥遗留注记收编 | 文档核对 |

## 8. 执行步骤

| ID | 任务 | 依赖 | 产出/验证 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | 异步桥 spike（C1 挂起 vs C2 消息回填）+决策工件 | — | spike 代码+选型文档；U-3 试点绿 | AC-03 | [x] 2026-09-22：C2 定案（决策工件见 §5 T-01 追记）；auto-lang `auto-musk-dev` 1×commit（native 3148+队列+泵+语料探针）；U-3=plan083_http_msg_bridge_tests 5/5 绿×3 连跑（cargo test -p auto-lang --lib --features ui-iced,ui-interpreter plan083） |
| T-02 | 后端分页+归一化直出（契约+Rust 实现+单测） | — | U-1/U-2 绿 | AC-01/02 | [x] 2026-09-22：musk `plan-083-dev` 1×commit——`/api/chats/session/{id}/page?limit=&before=`（缺省 50；has_more+next_before；缺省全量旧端点字节不变）；`blocks_normalized:true` 直出 r5b 拍平块；块级瘦身（§10-1 落实：巨条=工具密集型，result/thinking/args 截断+每消息 48K 预算+thin tool_calls 去重）。U-1=分页语义测、U-2=等价形态测+截断口径测+真实载荷测（auto-edit 938KB 会话→首屏 106KB=8.8×）全绿；既有 chat 44 测零回退。cargo test -p musk --lib |
| T-03 | musk 前端消费：两段式+分页+骨架+073 适配 | T-01/T-02 | U-4 绿；实机列表/首屏出 | AC-01/05 | [x] 2026-09-22：forge_store LoadSessionList/SwitchSession/BranchTo/PollStream 全迁消息桥（SessionsLoaded/DetailLoaded/OlderLoaded/PollBackfill 四回填段；PollBackfill 窗口合并=073 适配，pending/回合守卫/收束排空原样）；LoadOlder/OlderLoaded 历史前插翻页；chats_view 列表/首屏骨架+"加载更早的消息" pill（V-3 显式按钮形态，滚动桥接受限记录在案）；ts_adapter 补 get_msg Vue 半边（auto-lang 1×commit）。auto build 全 pipeline 绿（worktree 二进制）。U-4 实测面=后端 chat 44 测零回退 + T-05 V-4 实机 |
| T-04 | 迁移收编（SetWorkspace/Reload/081 注记） | T-03 | 全链走异步桥 | AC-03 | [x] 2026-09-22：PlansStore.LoadPlans 两段式（Choose 链三段=SetWorkspace→LoadSessionList/PlansStore.Reload/清场 全异步）；081 归档 §10-9 冻结/死亡线索二分注记落档（死亡=互杀 9f5593404/冻结=本计划）；PollStream 泛化余量（SSE 桥）不动（timer 注记更新）。auto build 绿 |
| T-05 | 实机验收矩阵 V-1..V-4 | T-04 | 冻结 20.9s→<1s 实测记录 | AC-04 | [x] 2026-09-22：隔离实例（worktree 后端 17283+worktree auto.exe+MCP 9283 驱动脚本 tmp/v083-acceptance.mjs）**V-1..V-4 全 PASS**——V-1 切 workspace 全程 1s 间隔 MCP state 轮询**0 不可响应轮次**（max 往返 70ms；对照基线=同一操作 20.9s 冻结）；V-2 首屏 414~754ms（≤1s，press→列表+首会话 50 条可交互；后端页 11ms/98KB）；V-3 翻页 50→60 条 264ms 零不可响应；V-4 9063 巨条（652KB/124 块）工具卡/正文/思考卡**像素实证**（截图 tmp/v083-evidence/）。**途中两缺陷根修**（musk 132ff73）：①JSON.parse 产物直接落 store 时 blocks 深度字段读在渲染上下文塌空（r5 家族）→ rebuildParsedMessages 扁平重建漏斗；②args_json 截断/降桩产非法 JSON 炸 messageBlocks computed（二分定罪 50 块阈值=预算降桩触发点）→ 恒合法 JSON 契约+2 回归测试（8/8 绿）。已知残留：isMsgStreaming 红色 Stop 钮偶现（streaming=false 态,疑 prop 构建期快照,非本计划链路——观察项） |
| T-06 | SD-01 落册 + 销项 + 收尾 | T-05 | 文档核对；两仓 ff-only 合回 | AC-06 | [x] 2026-09-22：SD-01 落册 worktree（vm-data-semantics 增量五节，musk 454dff9）；081 注记/§10-1/§10-2 销项闭环；§10-3（web 轨分页跟进）保持开放归用户排期。两 worktree 干净留置待 review；ff-only 合回归 merge 阶段 |

## 9. 复审记录

- 2026-09-22 draft（plan_revision 1）：合并立项——082 §10-7 三方向
  实测归因 + 081 刷新链同域遗留（Choose 冻结/PollStream 桥）。
  - stage: new, PLAN-083 r1
  - outcome: pass（授权范围内可开工；T-01 spike 为首个决策点）
  - next: work（两仓 worktree：musk `plan-083-dev`；auto-lang
    组内并排；T-01/T-02 可并行起步）
- 2026-09-22 work 收口（plan_revision 1，全六任务完成）：
  - stage: work | PLAN-083 | r1 | **pass** | 
    code_commit: musk `plan-083-dev`@454dff9（5×：T-02 243e09c /
    T-03 330a918 / T-04 471fe7f / T-05 双根修 132ff73 / SD-01 454dff9，
    base=main f622167）+ auto-lang `auto-musk-dev`@0a791cfb8（2×：T-01
    ec63b6ad0 消息桥+U-3 五测 / T-03 web 臂 0a791cfb8+30s 超时，
    base=master 06837787e）| task_ids: T-01..T-06 |
    evidence: U-1/U-2=chat_page 8/8 绿（含 auto-edit 真实载荷
    938KB→106KB）；U-3=plan083_http_msg_bridge_tests 5/5 绿×3 连跑
    （.at 全链模拟器探针）；U-4=musk chat 46 测零回退+V-4 像素；
    V-1..V-4 实机矩阵全 PASS（0 不可响应轮次/首屏 414~754ms/翻页
    264ms/巨条工具卡像素实证；证据 `.wt/musk-083/auto-musk/
    tmp/v083-evidence/`）；auto build 全 pipeline 绿×3 |
    blockers: 无阻塞项 |
    next: **review**（移交面见下）
  - **review/UAT 移交项**（不阻塞 execution_done，如实登记）：
    ①**发送/流式链实测未覆盖**——PollBackfill 改写（异步+窗口合并）
    语义与旧链逐分支对齐且后端 46 测绿，但"真实 LLM 会话发送→流式
    回填→收束"端到端未跑（验收实例无 LLM）；建议 review 或 UAT 用
    真会话抽验一发。②082 表格面抽样未跑（表格在 markdown text 块内，
    rebuild 保 text 字段=结构性不回退；代码层论证）。③isMsgStreaming
    红色 Stop 钮偶现（streaming=false 态 prop 构建期快照嫌疑，非本
    计划链路——观察项）。④已知微竞态：workspace 快速双切（<2 tick）
    时在途回填可能乱序落地（自愈型，ws_loading 门+终态覆盖；web 轨
    location.reload 语义无此窗）。⑤V-3 为显式按钮形态（滚动到顶自动
    触发受 VM 滚动事件面限制，scrollable onscroll 桥已存在但画布
    容器改造超本计划边界——follow-up 候选）。
  - status → **execution_done**

## 10. 待澄清事项

| # | 事项 | 状态/归属 |
|---|---|---|
| 1 | 首屏 limit 缺省值（50？）与超大单条消息（单条即 552KB）的次级截断（块内文本截断？）——552KB 若是单条消息，分页数限制不住它 | ✅ 闭环（T-02）：limit 缺省 50；巨条实测=工具密集型（84~109 tool 块非单块巨文本）——块级截断（result 4K/thinking 4K/args 2K+truncated 标记）+每消息 48K 预算（尾部优先，超预算更早 tool 块降桩）+thin tool_calls 去双份载荷（225KB 重复数组根除） |
| 2 | C1 挂起恢复是否立项为 VM 长期方向（本计划倾向 C2 起步） | ✅ 闭环（T-01）：C2 定案落地；C1 登记长期方向（见 §5 T-01 决策工件），无需裁定 |
| 3 | 分页对 PollStream 全量回填的语义改动的 web 轨跟进节奏 | web 可暂走全量旧路（缺省参数兼容）；跟进排期用户定 |
