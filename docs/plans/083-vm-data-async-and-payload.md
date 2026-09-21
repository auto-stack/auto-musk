---
plan_id: PLAN-083
status: drafting
feature_name: VM 数据链异步化与大载荷治理（081 刷新链债 + 082 §10-7 三方向合并）
author: [agent]
created_at: 2026-09-22T00:00:00Z
updated_at: 2026-09-22T00:00:00Z
plan_revision: 1
current_step: 0
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

## 5. 详细设计

### T-01 异步桥 spike（bounded 调查，决策工件）

auto-lang worktree spike：以 `LoadSessionList` 为试点，
- C2 形态：新增 VM 原语/桥（如 `Http.get_async(tag)` 入队即返回 +
  完成消息 `HttpDone(tag, payload)`），评估解释器/调度改动面；
- 验收 spike：试点 handler 期间 UI 可响应（MCP state 探测不冻结）；
- 产出决策工件：C1/C2 选型+改动面清单（文档落于本计划 §9 追记）。

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
| T-01 | 异步桥 spike（C1 挂起 vs C2 消息回填）+决策工件 | — | spike 代码+选型文档；U-3 试点绿 | AC-03 | [ ] |
| T-02 | 后端分页+归一化直出（契约+Rust 实现+单测） | — | U-1/U-2 绿 | AC-01/02 | [ ] |
| T-03 | musk 前端消费：两段式+分页+骨架+073 适配 | T-01/T-02 | U-4 绿；实机列表/首屏出 | AC-01/05 | [ ] |
| T-04 | 迁移收编（SetWorkspace/Reload/081 注记） | T-03 | 全链走异步桥 | AC-03 | [ ] |
| T-05 | 实机验收矩阵 V-1..V-4 | T-04 | 冻结 20.9s→<1s 实测记录 | AC-04 | [ ] |
| T-06 | SD-01 落册 + 销项 + 收尾 | T-05 | 文档核对；两仓 ff-only 合回 | AC-06 | [ ] |

## 9. 复审记录

- 2026-09-22 draft（plan_revision 1）：合并立项——082 §10-7 三方向
  实测归因 + 081 刷新链同域遗留（Choose 冻结/PollStream 桥）。
  - stage: new, PLAN-083 r1
  - outcome: pass（授权范围内可开工；T-01 spike 为首个决策点）
  - next: work（两仓 worktree：musk `plan-083-dev`；auto-lang
    组内并排；T-01/T-02 可并行起步）

## 10. 待澄清事项

| # | 事项 | 状态/归属 |
|---|---|---|
| 1 | 首屏 limit 缺省值（50？）与超大单条消息（单条即 552KB）的次级截断（块内文本截断？）——552KB 若是单条消息，分页数限制不住它 | T-02 设计点：需检查该 552KB 巨条的构成（单块巨文本 vs 多块）；单块巨文本需块级截断策略（如 fence/段落上限）|
| 2 | C1 挂起恢复是否立项为 VM 长期方向（本计划倾向 C2 起步） | T-01 spike 后按改动面数据请用户裁定 |
| 3 | 分页对 PollStream 全量回填的语义改动的 web 轨跟进节奏 | web 可暂走全量旧路（缺省参数兼容）；跟进排期用户定 |
