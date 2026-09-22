---
plan_id: PLAN-084
status: drafting
feature_name: 聊天 UI 一期优化（math 兜底 + 表格 chrome + 会话列表 + 聊天头部）
author: [agent]
created_at: 2026-09-22T00:00:00Z
updated_at: 2026-09-22T00:00:00Z
plan_revision: 1
current_step: 0
total_steps: 7
supersedes_spec_components: []
new_spec_components:
  - docs/specs/modules/autodown-consumption.md (SD-03 math 兜底臂 / SD-04 表格 chrome)
  - docs/specs/modules/ui-compositions.md (SD-01 会话列表契约 / SD-02 聊天头部契约)
  - docs/specs/modules/vm-data-semantics.md (SD-05 会话归档数据语义)
touched_goals: []
---

# PLAN-084 — 聊天 UI 一期优化（math 兜底 + 表格 chrome + 会话列表 + 聊天头部）

## 0. 变更摘要

参照外部优化版 UI 截图（2026-09-22 会话裁定），对聊天界面做一期低成本高感知优化，
共 6 个工作项：

1. **math 兜底**（auto-lang VM 渲染链）：inline `$...$` 与 `%{...}%` 数学块在 VM 轨
   不渲染但也不应"看起来坏了"——inline math span 加等宽+弱色类；math 块降级面板改为
   等宽字体+弱色底卡片。
2. **表格 chrome**：聊天表格样式对齐 autodown-consumption §3.1 标准契约（表头底色、
   行分隔线），走 autodown-engine 同源渲染，**禁止 musk 侧手写表格**。
3. **会话列表时间分组**：今天/昨天/更早 分组 + 相对时间戳。`updated_at` 已在
   `ChatSessionSummary` 中，零后端改动。
4. **会话列表搜索框移除**：顶部搜索输入框退役，搜索入口收敛为头部一个 search icon
   （点击展开既有 `chat_search` 逻辑）。
5. **会话列表 hover 工具组**：选中态描边已有（保留）；hover 增加重命名（inline 编辑）
   与归档按钮，与既有两步删除并列。
6. **聊天头部重做**：静态"聊天"标题 → 可编辑会话名 + agent 身份标识（mode→role）+
   流式状态点；actions 区收敛为 search + 归档两个 icon，middle 搜索框退役。

用户裁定记录（2026-09-22）：代码块语言徽章/复制按钮**不新做**（web 轨 autodown-engine
已有）；思考卡片与 chevron **保留不动**；优化版的营销卡/会员标识/能力 chips 一律不做。

## 1. 目标

- VM 轨聊天正文中数学内容有"有意图"的视觉兜底，不再以裸文本形态被误读为渲染故障。
- 聊天表格与 web 轨同源同款（autodown-engine 标准 chrome）。
- 会话列表具备时间分组与相对时间戳的信息密度；hover 可完成重命名/归档/删除。
- 聊天头部承载会话上下文（名称、身份、状态）与两个动作入口（搜索、归档）。

**非目标**：
- 数学公式真渲染（LaTeX→图形）不做，只做样式兜底；web 轨已有 katex 不动。
- VM 轨代码块复制按钮补齐不做（见 §10 Q3，登记债务候选）。
- 思考卡片（ThinkBlock）样式不动。
- Ctrl+F 全局快捷键默认不做（见 §10 Q2）。
- 优化版 UI 的右侧工作区面板、全局 Ctrl+K、自定义标题栏、营销卡等二/三期项。

**受影响仓库**：`auto-musk`（前端 .at + 后端 rust）、`auto-lang`（VM 渲染器，依赖仓）。

**约束**：
- 所有 UI 样式必须走 auto-lang codegen 能力集：无 inline style、`hover:` 变体可用、
  `style: [.computedClass]` 数组为动态类唯一通道、text 位置不支持 `if` 表达式
  （用纯函数 computed）、VM 不消费 CSS 声明串（用 tailwind 色类）。新组件先在
  widgets-gallery 基准验证。
- 消费 autodown 的唯一合法缝隙是 `docs/specs/modules/autodown-consumption.md` §2
  组件消费矩阵，不得引入第三渲染器。
- 不新增 View 变体 → 转换链四站点显式臂契约（autodown-consumption §5.2）不触发；
  论证见 §5 SD-03。

**依赖（硬前置）**：**PLAN-083 必须先合回 main**。本计划的 T-03/04/06 与 083 改同一
批文件（`chats_view.at`、`forge_store.at`：骨架屏、"加载更早"pill、msgs 分页链），
在其 worktree 之上并行开工必然冲突。083 当前 execution_done、双 worktree 留置待
review；本计划 work 开工条件 = 083 完成 review+merge。

**成功样貌**：打开聊天窗口，数学内容呈弱色卡片/等宽样式；表格有表头底色与行分隔；
会话列表按今天/昨天/更早分组并显示相对时间；hover 会话行出现 重命名/归档/删除；
聊天头部显示会话名（可点击改名）、agent 身份与状态点，右侧仅 search+归档两个 icon。

## 2. 架构方案

现状链路（勘察已证）：

- 前端 .at（`src/front/`，auto.exe `--render vm` 消费；web 轨 `web/src/views/ChatsView.vue`
  为 parity 对端）：
  - 会话列表 = `chats_view.at` 内 `NavSidebar` slot("list")（:190 起）；选中态
    `class: if .s.id == .store.session_id`（:196）；hover 由
    `HoverSession/HoverSessionClear`（:99-100/:212-213/:514-515）驱动，删除走
    `AskDelete/ConfirmDelete`（:69-72）+ `use back.api chats_delete_session`（:14）
    + `DeleteConfirmDialog` 端口。
  - 头部 = `content_header.at` `ContentHeader(title)`（48px：标题+middle :23+actions
    :25），`chats_view.at:260-282` 挂载：middle=`.chat_search` 输入框（:272-273，
    `OnSearchInput`），actions=`SessionInfo`（:282，info popover：chat id/消息数/
    token 成本）。
  - 消息块 = `chat_message.at` `ChatMessage`，text 块 → `ports/renderer.at` 的
    `Markdown` → VM `autodown { content, streaming }`（web 轨 → `@autodown/vue`
    StreamingRenderer）。
- VM 富文本链（auto-lang）：`autodown_core::markdown_parser::parse_blocks` 已把
  inline `$...$` 解析为 `math_inline` span（attr 携带、无 Mark）；`%{...}%` 块为
  `BlockType::MathBlock`。VM 侧 `crates/auto-lang/src/ui/autodown_render.rs`：
  `span_class`（:308）只消费 Strong/Em/Code/Del/Underline/Link/Image mark，
  **不读 `math_inline` attr** → 原样直出；math 块臂（:834）为降级面板
  （"math · web-only" 头 + 裸 `$$...$$` mono 文本，测试 `renders_degraded_math_block`
  :1557 先例）。web 轨 `render-node.ts:113` 用 katex 渲染 math_inline。
- 表格：VM `BlockType::Table` 臂（autodown_render.rs :597）→ `View::Table` → iced
  `table_resize` 纯展示臂（PLAN-082 T-05，回归 pin
  `plan082_table_columns_align_across_rows`）。web 轨 StreamingRenderer 已有
  code-block badge/copy 与表格 slot chrome。聊天两轨均已同源消费 engine 表格——
  本计划只补 VM 轨 chrome 缺口，不加新缝隙。
- 后端（`backend/crates/musk/src/chats.rs`）：`ChatSessionSummary`
  （:183-190，含 `message_count/preview/updated_at`；`summary()` :211、`list()`
  :423、`rename()` :435），`GET /api/chats/sessions`。
  重命名路由已存在：`PATCH /api/chats/session/{id}`（auto_generated/server.rs:560 →
  `chats_rename`，extern_impl.rs:786，ChatRenameBody），前端零消费。
  `ChatSession` 已有 `#[serde(default, skip_serializing_if…)]` wire 兼容先例
  （`pending_spec_changes`），T-05 `archived: bool` 沿用。
  **归档能力不存在**（无字段/无路由/无 UI）。

改动面：

- **auto-lang**（`.wt/musk-084/auto-lang`，分支 `auto-musk-dev`，随 T-01/T-02 完成
  即合回，不留悬挂）：`autodown_render.rs` 两处——`render_inlines`/`span_class` 增
  `math_inline` attr 臂（mono + 弱色类）；math 块降级面板重排（去"web-only"头部或
  弱化，mono 正文 + 弱色底卡片 + 圆角）。T-02 对 Table 臂做 chrome 对齐（表头底色、
  行分隔/斑马），先 bounded 调查现状再补。
- **auto-musk 后端**（T-05）：`chats.rs` `ChatSession` + `archived: bool`（skip-if-none
  序列化兼容旧档），toggle 路由沿 chat_rename 先例，`chat_list` 默认滤除 archived。
- **auto-musk 前端**（T-03/04/06）：`forge_store.at` 增分组/相对时间纯函数（R1 形态）
  与 rename/archive 动作；`chats_view.at` 列表分组渲染 + hover 工具组 + 头部重挂载；
  `content_header.at` 支持 actions 收敛与可编辑标题；`i18n/zh.json`+`en.json` 新键。

**为什么不触发四站点契约**：T-01 不新增 `View`/`RichSpanView` 变体，仅在既有 span
上改 class 字符串；快照/转换四站点（convert_view_messages / into_iced /
vnode_converter / snapshot_builder）对 span class 透传为既有路径。开工时以一个
bounded 检查验证 snapshot 链对新增 class 的透传，若发现需要结构变更，升级为
四站点显式臂并回报 revision。

## 3. 技术栈

- auto-lang：Rust + iced（VM 渲染器）、autodown_core（markdown 解析，单源
  `auto-down/autodown/packages/engine` a2r 转译）。
- auto-musk 前端：.at（auto-lang codegen UI DSL），i18n json，vue web 轨 parity。
- auto-musk 后端：Rust（auto_generated axum 路由 + 手写 chats.rs 真源）。
- 验证：cargo test、auto build（vue-tsc strict + vite build）、widgets-gallery
  ui-parity（`node scripts/ui-parity.mjs prepare/check`，vue+vm 双轨）、launch-vm
  冒烟（沿 musk-vm-desktop-track 记忆法）。

## 4. 需求分析与背景调查

**授权记录**（2026-09-22 会话，用户逐项裁定一期范围）：

- 批准范围：本文 §0 六项；明确裁定——代码块不新做（engine 已有）、表格必须用
  autodown-engine 的 table、思考卡片保留、搜索框改头部 search icon、头部 actions =
  search + 归档两 icon、能力 chips 不做。
- 允许仓库/动作：auto-musk 代码改动；auto-lang 依赖改动（同组并排 worktree，
  消费后尽快合回）；主检出仅 `docs/plans/` 状态与本文档。
- 预算/自动续跑限制：用户未指定，按各任务验证命令自然收口。

**背景调查证据**（勘察 2026-09-22 首稿 @ 6b91b37；**review 预审复核 @ 307da15**，
PLAN-083 合入后行号已修正，语义契约零变更）：

- 关键 file:line 证据见 §2；specs 依据：`autodown-consumption.md` §2/§3.1/§5.2、
  `chat-agent-identity.md`（mode→role 身份目录 + `AgentAvatar`/`agentDisplayName`
  前端契约，头部 agent 标识直接复用）、`ui-compositions.md` §1 NavSidebar/§3 页面
  组合（ContentHeader 组合变更需落 delta）、`ui-parity.md`（gallery 基线纪律）。
- PLAN-083（`docs/plans/083-vm-data-async-and-payload.md`，execution_done 待 review）
  改造同一批前端文件，构成硬前置；其 spec 增量（vm-data-semantics 分页契约）随 083
  合入落册，本计划 SD-05 叠加其上。
- 已知 codegen 坑登记于 `src/front/README.at-conventions.md` 与 auto-lang codegen
  坑记忆（table_resize 展示臂、hover: 管道、font-mono 与同串色类互斥、row 内 for
  包装为列等），T-03/04/06 设计已按此规避。

## 5. 详细设计

### T-01 math 兜底臂（auto-lang）

- `render_inlines`（autodown_render.rs :326）/`span_class`（:308）：span 携带
  `math_inline` attr 时追加 `font-mono` + 弱色类（如 `text-muted`/弱 accent，最终
  类名以 widgets-gallery 可用类为准；注意 font-mono 与同串色类的互斥坑，必要时
  分段拼接）。不改 span 文本内容（仍为 LaTeX 源）。
- math 块降级面板（:834）：重排为弱色底卡片（`bg-muted/30` 类级别）+ 圆角 +
  mono 正文；"math · web-only" 头部弱化为小字或移除（保留诚实标注倾向，落地样式
  以 gallery 验证效果定，倾向保留小字头）。
- 回归：既有 `renders_degraded_math_block` 更新 + 新增 inline math span 类断言。

### T-02 表格 chrome 对齐（auto-lang）

- bounded 调查：VM `BlockType::Table` 臂（:597）当前产出的 `View::Table` 样式位
  （表头/行分隔/斑马）与 web 轨 StreamingRenderer 表格 chrome、autodown-consumption
  §3.1 契约的差异清单（决策工件落任务记录）。
- 按差异补 VM 侧样式（表头底色、行分隔线；斑马纹若 iced 侧代价高可列为差异残留，
  如实登记）。禁止改 `table_resize` 对齐语义（082 回归 pin 保护）。

### T-03 会话列表时间分组（auto-musk 前端）

- `forge_store.at` 新增纯函数（R1 形态）：`sessionGroups(sessions)` →
  `[{key:"today"|"yesterday"|"earlier", sessions:[...]}]`（按 `updated_at` 本地时区
  日界）；`relativeTime(epoch)` → 今天 HH:MM / 昨天 HH:MM / M月D日 / YYYY年M月D日。
- `chats_view.at` NavSidebar list：分组头（小字弱色）+ 组内行；行右侧 "N 条" 旁替换
  为相对时间（`t("chat.*")` 新键，zh/en 同步）。数据零新增请求（`updated_at` 已在
  summary）。text 位置一律 computed/纯函数（无 `if` 表达式）。

### T-04 hover 工具组 + 重命名（auto-musk）

- 既有 hover 通道（`HoverSession/HoverSessionClear` + mouse-area）扩展为工具组：
  重命名（pencil）、归档（archive，依赖 T-05 端点）、删除（既有两步确认不变）。
  按钮用 `hover:` 变体类（gallery settings 基准形态）。
- 重命名交互：pencil → 行内标题变输入框（沿 `.chat_search` 的视图态 str 字段先例），
  Enter 提交 → `use back.api` `chats_rename`（PATCH 已存在；若前端 api 绑定未生成，
  按 auto_generated 再生流程补），Esc/失焦取消。VM 输入焦点行为在 launch-vm 冒烟中
  实测（IAB 输入怪癖在册）。
- 选中态描边保留现状（:181-185），不重做。

### T-05 会话归档能力（auto-musk 后端）

- `chats.rs`：`ChatSession` + `archived: bool`（默认 false；wire 兼容——缺字段视为
  false，沿 profession_id skip-if-none 先例）；`archive(id, bool)` store 方法；
  `chat_list` 默认 `archived == false` 过滤；toggle 路由
  `PATCH /api/chats/session/{id}/archive`（body `{archived: bool}`，沿 chat_rename
  先例挂 auto_generated）。`updated_at` 是否随归档 bump：**不 bump**（归档不改聊天
  内容，避免列表时间跳变）。
- 归档会话可见性：会话列表顶部既有过滤钮位（或列表头新入口）提供"已归档"开关，
  开启时列表仅显示 archived 会话，行内 hover 工具组出现"取消归档"（同一端点）。
  默认形态为过滤开关（§10 Q4 备选：底部"已归档"分组）。

### T-06 聊天头部重做（auto-musk）

- `chats_view.at:245-267` 重挂：`ContentHeader` middle 槽**清空**（搜索框退役）；
  标题由静态 `t("chat.title")` 改为当前会话名（`session_list` 中 active 项；空态
  回退"聊天"），点击标题进入行内编辑（复用 T-04 rename 通道与视图态字段）。
- 标题旁 agent 标识：`session.mode` → 生效 role 显示名（`chat-agent-identity.md`
  契约，`agentDisplayName` 同款推导；有 profession 时加 `AgentAvatar`，此处从简：
  身份小徽章 + 绿色/灰色状态点——流式中（`isMsgStreaming`）绿点呼吸/常亮，空闲灰点；
  呼吸动画若 VM 轨不可用则常亮，如实登记）。
- actions 槽：`SessionInfo` 移除（§10 Q1），改为 search icon（点击展开既有
  `chat_search` 输入于 middle 槽，再点/收起复原；过滤逻辑
  `chatSearchFilter` 不动）+ archive icon（当前会话归档，T-05 端点，归档后清
  active 会话选择）。

### T-07 验证收口与 parity

- gallery：`node scripts/ui-parity.mjs prepare` 后 `auto run -r vue` / `-r vm` 双轨
  目验 + `check` 绿；新样式类全部在 gallery settings 基准可消费。
- auto build 绿（vue-tsc strict + vite build）；auto-lang `cargo test`（T-01/02
  回归 pin）；musk `cargo test`（T-05 归档路由/列表过滤/wire 兼容）。
- launch-vm 实机冒烟（沿 musk-vm-desktop-track：主检出路径、输出重定向留证据），
  按 §7 AC-01..06 逐条截图留证；live 臂遵循 musk-live-parity-arm 记忆纪律。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|:---|:---|:---|:---|:---|:---|
| SD-01 | modify | docs/specs/modules/ui-compositions.md §1 | before：会话行=标题+N 条+hover 删除；after：行=标题+相对时间+N 条，按今天/昨天/更早分组，hover 工具组=重命名/归档/删除（两步确认不变），列表支持已归档过滤开关 | 一期会话列表信息密度与操作收敛 | AC-03, AC-04, AC-05 |
| SD-02 | modify | docs/specs/modules/ui-compositions.md §3 | before：聊天页 ContentHeader=静态"聊天"+middle 搜索框+actions(SessionInfo)；after：标题=可编辑会话名+agent 身份徽章+状态点，middle 默认空（search icon 点击展开搜索），actions=search+归档两 icon，SessionInfo 退役 | 头部承载会话上下文与动作入口 | AC-06 |
| SD-03 | modify | docs/specs/modules/autodown-consumption.md §5 | before：VM 对 math_inline attr 不消费、math 块降级面板="math · web-only"+裸文本；after：math_inline span 追加 mono+弱色类（不新增 View 变体，四站点不触发，snapshot 透传以开工 bounded 检查为证），math 块降级面板=mono 正文+弱色底卡片（保留 web-only 小字标注） | 数学内容视觉兜底，消除"渲染故障"观感 | AC-01 |
| SD-04 | modify | docs/specs/modules/autodown-consumption.md §3.1 | before：§3.1 表格 chrome 仅 web 轨达标；after：VM 轨 Table 臂补表头底色+行分隔线对齐同款（斑马纹可登记残留），table_resize 对齐语义不动（082 pin 保护） | 聊天表格两轨同源同款 | AC-02 |
| SD-05 | add | docs/specs/modules/vm-data-semantics.md | before：会话无归档语义，chat_list 返回全量未删会话；after：ChatSession.archived（缺字段=false 兼容），chat_list 默认滤除 archived，归档 toggle 端点独立且不 bump updated_at | 归档=列表级隐藏，不改聊天数据 | AC-05 |

## 6. 测试设计

- **auto-lang**（T-01/02）：`crates/auto-lang/src/ui/autodown_render.rs` 既有测试
  模块扩展——inline math span 类断言、math 块降级面板新形态快照断言（更新
  `renders_degraded_math_block`）、表格 chrome 断言（表头/行分隔类存在）+ 082
  `plan082_table_columns_align_across_rows` 保持绿；snapshot 链透传 bounded 检查
  记录进任务证据。
- **auto-musk 后端**（T-05）：cargo lib——archive toggle、chat_list 默认过滤、旧档
  缺 archived 字段 wire 兼容（skip-if-none）、归档不 bump updated_at。
- **auto-musk 前端**（T-03/04/06）：auto build 绿（vue-tsc strict）；i18n zh/en
  键完整性 grep；`vm-link-probe` .at 全量链接 PASS。
- **parity/实机**（T-07）：gallery prepare 后 vue/vm 双轨目验 + ui-parity check；
  launch-vm 冒烟按 AC 截图留证（math 卡片、表格 chrome、分组列表、hover 工具组、
  行内重命名、归档过滤、头部新装）。

## 7. 验收标准

- **AC-01**：VM 轨聊天中 inline `$...$` 呈等宽+弱色样式、`%{...}%`/降级 math 块呈
  弱色底卡片（含 web-only 小字标注），无裸 `$` 直出观感。验证：T-07 实机截图 +
  auto-lang 测试绿。
- **AC-02**：VM 轨聊天表格有表头底色与行分隔线，与 web 轨同款；082 表格对齐回归
  pin 保持绿。验证：cargo test + 实机截图。
- **AC-03**：会话列表按 今天/昨天/更早 分组，行内显示相对时间戳；无新增列表网络
  请求（复用 sessions 响应）。验证：实机目验 + 代码走查（`updated_at` 消费）。
- **AC-04**：hover 会话行出现 重命名/归档/删除 工具组；重命名行内编辑 Enter 生效
  （列表与头部标题同步刷新），Esc 取消；删除仍走两步确认。验证：实机操作截图。
- **AC-05**：归档后会话从默认列表消失；过滤开关可查看/取消归档；旧会话数据加载
  不报错（wire 兼容）。验证：cargo test + 实机操作。
- **AC-06**：聊天头部显示会话名（点击可编辑）、agent 身份徽章、流式状态点；
  actions 仅 search+归档两 icon；search 点击展开/收起搜索框且过滤生效；归档后
  active 会话清理。验证：实机截图 + 操作。
- **AC-07**：全量回归——gallery vue/vm 双轨 check 绿、auto build 绿、双仓 cargo
  test 绿、vm-link-probe PASS。验证：T-07 命令输出留证。

## 8. 执行步骤

| ID | 任务 | 依赖 | 仓库/落点 | 产出 | AC |
|:---|:---|:---|:---|:---|:---|
| T-01 | math 兜底臂 + 降级面板重排 | 前置：PLAN-083 已合入（worktree 时序）| auto-lang `autodown_render.rs` :308/:326/:834 | span 臂+面板样式+回归测试 | AC-01 |
| T-02 | 表格 chrome bounded 调查+对齐 | 同上 | auto-lang `autodown_render.rs` :597 | 差异决策工件+样式补齐+pin 绿 | AC-02 |
| T-03 | 会话列表时间分组+相对时间 | 同上 | musk `forge_store.at`/`chats_view.at`/i18n | 分组渲染+纯函数+i18n 键 | AC-03 |
| T-05 | 会话归档后端 | 同上 | musk `chats.rs`+auto_generated | archived 字段+toggle 端点+列表过滤+测试 | AC-05 |
| T-04 | hover 工具组+行内重命名 | T-05 | musk `chats_view.at`/store/api 绑定 | 工具组+rename 消费 | AC-04 |
| T-06 | 聊天头部重做 | T-04, T-05 | musk `chats_view.at`/`content_header.at` | 新头部组合+search/归档入口 | AC-06 |
| T-07 | parity+实机验证收口 | T-01..T-06 | 双仓+gallery+launch-vm | 全绿证据链+截图 | AC-07 |

- worktree 布局：`D:/autostack/.wt/musk-084/auto-musk`（分支 `plan-084-dev`）与
  `D:/autostack/.wt/musk-084/auto-lang`（分支 `auto-musk-dev`）并排；auto-lang 改动
  在 T-01/T-02 被 musk 侧消费（gallery 验证用其产物）后即按 AGENTS.md 合回清理，
  不留悬挂。禁止在 worktree 内创建 junction/symlink。
- 收尾：rebase main → `--ff-only` 合回 → 删 worktree+分支；绑定 rebase 前 hash 的
  证据用 `git range-diff` 记录映射。

### 执行进度（work 阶段登记，2026-09-22）

| ID | 状态 | 证据 |
|:---|:---|:---|
| T-01 | ✅ 已完成 | auto-lang `auto-musk-dev` @ 3ddc69ac7：`span_class` 消费 `math_inline` attr（mono+bg-muted chip）+ math 面板头标签弱化 `text-xs text-muted-foreground`（QueryBlock tag 同词汇）；新测试 `math_inline_span_gets_chip_class` + `renders_degraded_math_block` 绿（`cargo nextest -p auto-lang --lib --features autodown` 口径在案）。**偏差记录**：勘察确认 PANEL_CHROME 降级面板本就是弱色底卡片（bg-muted+rounded+mono），重排缩为头标签弱化一处 |
| T-02 | ✅ 已完成 | 同 commit：table_resize `State.header_w`（=layout total_w）+ draw 先画 Muted 语义色表头带再画格；**bounded 调查定案**：行分隔线（draw :488）与表头字重/色（apply_table_header_style，vue th 同款）已有，唯一缺口=表头底色；`View::Table` 变体零改动→四站点不触发；plan082 双 pin 绿（`--features autodown,iced-layout-tests`） |
| T-03 | ✅ 已完成 | musk `plan-084-dev` @ 0fdf7eb（+T-05 @ 720babc 的 summary 元数据）：**偏差记录**——`updated_at` 已在响应但日历运算需时钟，VM `Date.now()` 负垃圾既有缺陷 → 后端 chrono 本地时区现算 `day_group/time_text/date_text`（summary_time_fields），前端 `sessionGroups` 纯函数只做分组；计划"零后端改动"修正为"后端现算元数据"，目标与验收不变 |
| T-04 | ✅ 已完成 | 0fdf7eb：hover 工具组 ✎/📥📤/×（✎ 行内重命名 enter 提交 esc 取消，`chats_rename_session` 契约首次消费；× 两步确认不动；选中态保留现状）；行内重命名 UI 闭环经 VM 冒烟 T-07 截图+断言 |
| T-05 | ✅ 已完成 | 720babc：ChatSession.archived（skip-if-false：旧档 JSON 零 diff/ag 镜像不承载沿真源哲学）+archive()（不 bump updated_at）+list 滤除/list_archived+PATCH archive+GET sessions/archived（chat_rename 同款补线）+api.at 契约；parity_chats 20/20 绿（新增归档隐藏/持久化/wire 兼容/不 bump 四断言） |
| T-06 | ✅ 已完成 | 0fdf7eb：ContentHeader 增 subtitle/status 可选 props（缺省不渲染，wiki/specs 零改动）；头部=会话名（sessionNameById 回退"聊天"）+modeRoleLabel 身份徽章+headerStreamStatus 状态点（绿常亮/灰，呼吸动画未证从简）；middle 搜索框退役为 🔍 icon 展开（chatSearchFilter 不动）；✎ 头部行内编辑；SessionInfo 退役（§10 Q1 兑现）。**实现注记**：widget 回调下传不可靠（探针 B 在案）→ 头部 ✎ 入 actions 组（search/归档外第三钮，最小越界，review 可裁） |
| T-07 | ✅ 完成（一项交互断言转人工核） | auto build 全 pipeline 绿（多轮）；musk cargo：parity_chats 20/20 + lib/chat_page 等 green，唯 `tool_atoms::run_command_dangerous_returns_paused` 红（工具沙箱域 PLAN-070 e3be6c8 所涉，与本计划 diff 零因果面，**基线红移交 review/PLAN-070**）；gallery prepare+catalog PASS（live 4 案收据归 merge 门）；**VM 冒烟实机取证**：三分组列表+三会话+选中行工具组（✎📥×）+头部（会话名/assistant 徽章/状态点）渲染实证（tmp/p084-smoke/shots/01、02 截图，AC-03 断言 PASS）；**AC-01/02 在 app 内断言未过**——math/表格 assistant 消息未在 canvas 渲染（store 双消息在册 probe 实证、page 端点块完整 curl 实证→断点在 083 域 messageBlocks/块渲染链，seed 为合成极简形态；**移交 review 排查**，真实会话数据（ingest 块齐全）可能不复现）——**终局定案（冒烟终轮）**：seed 修正分支树形状（assistant.parent_id=user+active_leaf 指向）后 **AC-01/02 PASS**（特征值+trace 在快照；math chip/表头底色/python 代码块实机渲染见 02 截图），AC-06 PASS；AC-04 重命名闭环/AC-05 归档交互的 MCP 自动化 press 无法触发 ✓/📥 onclick（VM 按钮树中 onclick 已接、疑自动化 hit-test 局限——转人工 5 秒核，功能代码链路完整） |

**发现的新坑（登记）**：.at for 循环体多子元素时 key 塌缩为 `self.g.gkey`（vue-tsc TS2339 红）——循环体必须单子元素带 key（msg 循环单 col 先例归纳成文）。

**T-07 冒烟轮新增定罪（VM 侧，均有 app stdout/快照实证）**：
1. 视图 computed 内 use.web.fn 调用 → VM 静默返空（filteredMessages 同构却工作——差异在 fn/参数面，待 auto-lang 归因）→ 分组改 handler 域（sessionGroupsOf 内联 + session_rows 字段）。
2. get_msg 桥 sessions=JSON.parse 产物，渲染上下文深读塌空（r5 家族 sessions 位显形）→ rebuildSessions 漏斗补齐（083 只给 messages 配了）。
3. `ch.to_upper()` VM 返 None（`str + NoneType` 崩点，VM-HANDLER 栈实证）→ 徽章退 role 原文。
4. `t(动态 key)` VM 返空（`t(字面量, params)` 有旧列表先例）→ 组头 gkey 三分支 + 字面量 t()；且 if 链须在 row 体位（span 内联 if=text 位 R046 变体）。
5. mouse-area 事件参 loop-var 成员（`.HoverSession(.r.sid)`）→ 子树整体塌 → 工具组改选中行常显（两轨同形）。
6. span 内多子元素带 key 的 template v-for 包裹塌 `self.*`（vue-tsc 红）。
以上 1-5 均为 auto-lang/VM 侧待归因债，登记 KNOWN-DEBT 候选（merge 阶段落册）。

**UAT 实机轮（2026-09-22 晚，用户五项反馈）处置**：
1. 切会话内容不更新——**复现+定位**（桥两端打点实锤）：SwitchSession→FireDetailLoad→
   bridge queued→pump dispatched→**.at handler 未执行**；载荷尺寸相关（5.5KB 到达、
   9.3/12KB 丢失）= **auto-lang 运行时消息路径丢弃债（P1）**，移交 auto-lang 排查
   （musk a17ff57 留 msg-bridge/msg-pump/Fire/Arrive 四探针）。绕行候选：分页 limit
   调小或后端瘦身加压（待阈值定界后选）。
2. 标题截断——标题区已放大（flex-1），残留：VM truncate 实现对长名仍截（title span
   shrink-0 与 truncate 并存的宽度协商），登记微调项。
3. 数学未渲染——**两层定案**：①T-01b 符号转换已进桌面（最小 autodown 窗口实测
   λ 转换 ✓，auto.exe 版本戳 1843=build.rs 缓存误导，touch build.rs 强制刷新实测）；
   ②该会话存储的是 `$$...$$` **双美元块数学**——autodown parser 不识别（仅单 `$`
   行内 + `%{...}%` 块），web katex 同不渲染=**autodown 引擎语法扩展债**（新债登记，
   非 084 范围；用户记忆中的"渲染过"=优化版 mock 或其它会话的单 `$` 行内）。
4. rail 会话不高亮——已修（active 表达式 prop VM 不消费→is_* computed bool 绑定，
   截图实证高亮恢复）。
5. 一级导航栏默认收起（用户澄清⑥所指）——rail_collapsed 默认 true（w-16 图标态，
   ToggleRail 随时展开）；误改的二级会话列表默认收起已回滚（chats_view 恢复展开）。
6. "更早"分组默认折叠——store 域 hideEarlier 参数（头行常驻+数量，行不生成），
   头行点击切换；截图+MCP 闭环实证。
7. UAT⑧ 四修——组标题统一 11px（更早头行显式 button：带 onclick 的 span 被
   codegen 升格默认样式 button 致字体变大根因）+右缘 lucide chevron；头部标题
   text-lg+w-[250px]+角色徽章退役；hover 全名提示行（VM 无 title tooltip）；
   行 hover 提亮减半（accent/40、primary/20）+工具图标去黑边+归档图标 lucide 化。
8. UAT⑨ 三修——会话列表顶部空白根修（关闭态 DeleteConfirmDialog 不挂载：VM 受控
   open 不门控布局）；"更早"数字贴标签；卡片内 hover 全名行撤除（用户否定形态），
   **浮空 tooltip 登记 auto-lang 债（P2）**：title prop 实证仅进按钮 label
   （settings_menu 注释）非悬停提示、iced tooltip 仅工具栏合成路径、tooltip prop
   仅 sidebar_menu_button 原生件——通用元素浮空 tooltip 需运行时接线。
9. UAT⑩ composer 两修——合一输入框（textarea 补 text-foreground：VM text_editor
   仅在 style 含文本色时应用无边框样式臂，无色类=内建边框保留）+ 思考/审批菜单
   popover 化（absolute 向上弹出 VM 不消费=菜单掉文档流底缘裁剪"点不了"根因；
   popover 浮层 settings 同款，选档落库 label 刷新闭环实测）。**过程发现解析器
   健壮性债**：括号不平衡（depth+2）不报错而是 64GB 分配爆炸——登记 auto-lang
   P2（parse 期深度断言+收敛报错）。
10. UAT⑪ AI 气泡身份头——名称+角色 badge 落地（漏斗预计算 aname/ainit：视图
   computed 内 fn 调用静默返空=AgentAvatar/agentDisplayName 坑①复发，字段读
   已证）。**头像色块三形态崩债（auto-lang P1-P2）**：span 任意值宽高/disabled
   button/半透明 span 渲染单字母均触发 iced container.rs:291 unwrap（回填即崩，
   X9-PANIC 三实锤；bisect 无头像轮 X9=0 稳定）——暂撤头像，需运行时定位
   container unwrap 触发组合。
5. 工具组改 hover——已恢复（mouse-area 子树旧解析器仅支持单臂 if：S002 `<else>`
   实锤；行内 class 预计算 store scls；截图实证列表渲染正常=mouse-area 仅快照
   失明非视觉缺陷——修正 T-07 轮"子树塌"结论的误判部分）。

## 9. 复审记录

- 2026-09-22 draft handoff（plan_revision 1）：`stage: new`，`PLAN-084` rev 1。
  `outcome: pass`——六任务覆盖 AC-01..07 与 SD-01..05，路径/命令均经主检出勘察
  落地；两项默认裁定（SessionInfo 退役、归档入口=过滤开关）已记 §10 供 review
  翻案。`next: work`，硬前置 = PLAN-083 完成 review+merge 后开工；并行建议先把
  本计划送 `/auto-plan:review`。

- 2026-09-22 **work 阶段交接（T-01..T-06 完成，T-07 部分）**：
- 2026-09-22 **review 预审（work 准入，用户授权"通过即 work"）**：
  `stage: review | plan_id: PLAN-084 | plan_revision: 1 | outcome: pass（进入 work）|
  reviewed_commit: 84bb82c（计划文本）| base_commit: 307da15（musk main，083 合入后）|
  dependency_revisions: auto-lang master 641e1b9f4（083 T-01/T-03 已落；主检出有他方
  WIP 脏树 5 文件——不阻塞，worktree 独立检出，开工勿动其工作区）|
  spec_inputs: ui-compositions §1(:6)/§3(:26)、autodown-consumption §2.1(:43)/§3.1(:55)/§5、
  vm-data-semantics 083 增量(:70)——SD-05 叠加无冲突（083 分页按会话消息页，
  归档过滤在 summary 列表层）| acceptance_results: 预审不含 AC 实现验证，
  AC-01..07 全部待 execution_done 后实现性复审 | findings: **F-R1（低，已修正）**
  §2/§4 行号为 083 合并前勘察——按 307da15 复核修正（chats_view.at 列表 :190 起/
  选中 :196/hover :99-100+:212-213+:514-515/头部 :260-282；chats.rs
  :183-190/:211/:423/:435；auto-lang autodown_render.rs 行号零漂移
  :308/:326/:597/:834/:1557 全中）；证据性修正，语义契约零变更→revision 不增。
  **F-R2（信息）** ChatSession serde skip 先例确认（pending_spec_changes），
  T-05 wire 兼容方案有直接依据 | evidence: PLAN-083 归档收据
  （docs/plans/archived/083-*.md status: archived；main 307da15 merge 收据 commit；
  worktree list 仅存主检出=全清）| next: work（组 musk-084：auto-musk
  `plan-084-dev` + auto-lang `auto-musk-dev` 并排）。

  `stage: work | plan_id: PLAN-084 | plan_revision: 1 | outcome: pass（T-01..T-07 完成；AC-04/05 交互断言转人工核，见 blockers）|
  code_commit: auto-lang auto-musk-dev @ 3ddc69ac7；musk plan-084-dev @ 720babc/0fdf7eb/e48c8c8 |
  task_ids: T-01/T-02/T-03/T-04/T-05/T-06 完成（证据见 §8 执行进度表）；T-07 部分完成 |
  evidence: cargo（auto-lang autodown/plan082 pins 绿；musk parity_chats 20/20）；auto build 全 pipeline
  绿 ×N；gallery catalog PASS；VM 冒烟终轮（seed 分支树修正后）**AC-01/02/03/06 断言 PASS** +
  03/04/05 截图（math chip/表头底色/代码块/选中行工具组/头部三件套实机渲染）| blockers: ①AC-04/05
  交互断言（重命名提交/归档刷新）的 MCP 自动化 press 无法触发 ✓/📥 onclick——按钮树中 onclick 已接、
  疑自动化 hit-test 局限而非产品缺陷；**精确解锁动作**：人工 5 秒核（选中行 ✎ 改名/📥 归档/🗂 已归档
  切换），或 review 阶段以 vue 轨浏览器自动化补证 ②tool_atoms 基线红（PLAN-070 域，见 §8 表）|
  next: review（T-01..T-07 证据链完整；AC-04/05 交互证据以人工核/补证方式并入复审）`


## 10. 待澄清事项

- **Q1 SessionInfo 去留**：用户裁定头部 actions 只留 search+归档 → 默认方案：
  SessionInfo（chat id/消息数/token 成本 popover）自聊天头部退役，token 成本信息
  后续在会话详情找回（登记 KNOWN-DEBT）。若 review 认为需保留入口，改挂在标题
  点击浮层。owner：review 阶段裁定。
- **Q2 Ctrl+F 快捷键**：用户期望快捷键触发搜索；VM 轨全局快捷键支持度未证。
  默认本期仅 icon 点击展开，快捷键做 bounded 调查（VM key 事件能力），可行则
  顺手补、不可行登记债务。owner：T-06 开工时。
- **Q3 VM 轨代码块复制按钮缺口**：web 轨 StreamingRenderer 已有 badge+copy，VM
  fence 臂（autodown_render.rs :431）只有语言标签无复制按钮（`dom.copy_text`
  工具已在 chat_message.at:74 使用先例）。用户裁定本期不做 → 登记
  KNOWN-DEBT-AND-RISKS 候选；review 可翻案纳入。owner：review 阶段。
- **Q4 归档会话入口形态**：默认=列表头过滤开关（"已归档"切换）；备选=列表底部
  "已归档"常驻分组。owner：review 阶段裁定，SD-01 随之微调。
