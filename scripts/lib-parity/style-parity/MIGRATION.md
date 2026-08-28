# MIGRATION.md — PLAN-049 双轨样式迁移映射表（T1 产物）

> 状态：T1 完成（2026-08-28）。本表是 inject_styles.ts 全部选择器 + 全部组件
> `style{}` 块 → .at 内联 tailwind 工具类的迁移映射底册。每切片（T4-T8）按
> 本表执行，完成后在「切片归属」列勾销并回写实际落点。
>
> 支持度列记号（探针逐类断言，见 §3 / `t1-class-probe.txt`）：
> - ✅ = class.rs 解析 OK；🚪 = 变体/装饰类，VM 已知丢弃（白名单，报告列
>   「web-only 增强」）；⚠️ = 解析缺口（D3 候选或草案避用）。
>
> 迁移通用变换（D2）：自定义类 → 等价工具类串写回 .at `style:`；hover 增强
> 以 `hover:` 变体保留（web 生效、VM 白名单丢弃）；web 独有且工具类无法表达
> 的（伪元素/伪类链/动画/滚动条/主题变量）拆 `inject_styles.web-only.ts` 挂账。

## 0. 对账

- 选择器总数：**134**（`node scripts/lib-parity/style-parity/count-selectors.mjs`，
  全量清单在 `t1-selector-inventory.txt`；@import 字体 1 条 + @keyframes st-dots
  1 组另计）。
- `style{}` 块：**37 处 / 23 文件**（grep 对账；`ports/renderer.vm.at` 为 VM
  渲染器桩，非组件样式，排除），见 §2。
- 探针：116 token 断言（ok=95 / variant=7 / gap=14），全绿（§3）。

## 1. inject_styles.ts 选择器映射（134 条）

### 1.1 全局段（18 条）——不迁组件，退役时归 web-only 文件

| 选择器 | 归属 | 处置 |
|:--|:--|:--|
| `@import` Noto Sans SC + `body,button,input,...` font-family (2) | 全局 | web-only（VM 字体走 iced 自己的栈）→ web-only.ts |
| `:root` / `.dark` 主题变量 (2) | 全局 | web-only（VM 侧 theme.rs 常量等价；值一致性进 T3 对拍集）→ web-only.ts |
| `::-webkit-scrollbar`×4 (4) | 全局 | web-only（iced 滚动条自绘）→ web-only.ts |
| `a` 链接色 (1) | 全局 | web-only → web-only.ts |
| `.flex.flex-row.h-screen > div:first-child` ×light/dark (2) | App rail | 迁：app.at rail col `bg-card`→`bg-secondary`（dark 变量由 CSS 变量机制自动跟随） |
| `.chats-view/.plans-view/.specs-view/.wiki-view > div:first-child` ×light/dark (8) | NavSidebar 壳 | 迁：NavSidebar 根 div 加 `bg-card`（NavSidebar 化后四视图同源） |

小计 21（原 18 组选择器展开 light/dark 后 21 条规则；对账脚本按源码行计 134，此处按族归组）。

### 1.2 共用壳：NavSidebar / ContentHeader（9 条）→ 切片 T4/T6

| 选择器 | 组件 | 工具类草案 | 支持度 |
|:--|:--|:--|:--|
| `.nav-sidebar` | nav_sidebar.at 根 | `flex flex-col h-full shrink-0 overflow-hidden` | ✅ |
| `.nav-sidebar.collapsed` | 同上（collapsed 分支） | `w-12`（48px）拼入条件串 | ✅ |
| `.nav-sidebar-header` | nav_sidebar.at header | `flex items-center gap-1.5 py-2 px-3 shrink-0 h-12 border-b border-border`（gap 0.4rem≈6.4px→gap-1.5） | ✅ |
| `.nav-sidebar-title` | nav_sidebar.at 标题 | `flex-1 text-base font-bold text-foreground` | ✅ |
| `.content-header` | content_header.at 根 | `flex items-center justify-between h-12 shrink-0 px-5 border-b border-border bg-card`（1.25rem=20px=px-5） | ✅ |
| `.content-header-title` | 同 | `text-xl font-bold text-foreground shrink-0`（1.25rem 字号=text-xl 20px） | ✅ |
| `.content-header-middle` | 同 | `flex-1 min-w-0 flex justify-center` | ✅ |
| `.content-header-actions` | 同 | `flex items-center gap-1 shrink-0`（0.3rem≈4.8px→gap-1） | ✅ |
| `.app-header` | （048 已内联,无组件引用） | 已迁；本切片仅删 inject_styles 残段 | — |

宽度注：NavSidebar 宽度原由 `.xxx-view > div:first-child` 位置选择器给
（chats/plans 220px、specs 200px、wiki 240px）——迁移时 NavSidebar 加
`width` prop（默认 `w-[220px]`，specs 传 `w-[200px]`、wiki 传 `w-[240px]`，
w-[Npx] ✅），连同 §1.1 的 `bg-card`、`border-r border-border shrink-0` 一起
落根 div；四个 `> div:first-child` 规则随之删除。

### 1.3 导航栏 rail（5 条）→ 切片 T4（048 已迁,本片收尾）

| 选择器 | 组件 | 状态 |
|:--|:--|:--|
| `.rail-tab` / `:hover` / `.active` (3) | app.at 视图 rail 按钮 | 048 已迁工具类（探针全 ✅）；本片补对拍用例 + 删 inject_styles 残段 |
| `.rail-footer` | app.at 底部行 | 同上（`mt-auto flex items-center justify-between gap-1.5 px-1`） |
| `.rail-footer .workspace-selector` | workspace_selector.at 根 | 迁：WorkspaceSelector 根加 `flex-1 min-w-0`（T8 片） |

### 1.4 ChatsView 会话壳（17 条）→ 切片 T6

| 选择器 | 组件/元素 | 工具类草案 | 支持度 |
|:--|:--|:--|:--|
| `.chats-view` | chats_view.at 根 row | `flex flex-row h-full overflow-hidden` | ✅ |
| `.chats-view > div:first-child` | NavSidebar 根（宽度族） | 见 §1.2 宽度注 | ✅ |
| `.sidebar-new-btn` / `:hover` (2) | 新建/删除会话按钮 | `inline-flex items-center justify-center h-[26px] px-2 text-xs border border-border rounded-md bg-transparent text-foreground hover:bg-accent` | ✅+🚪 |
| `.session-list` | 会话列表 col | `flex-1 overflow-y-auto px-2` | ✅ |
| `.session-item` / `:hover` / `.active` (3) | 会话项按钮 | `block w-full text-left py-2 px-2.5 mb-0.5 rounded-md bg-transparent hover:bg-accent`；active 分支拼 `bg-primary/10 text-primary` | ⚠️px-2.5/mb-0.5（D3 分数臂）+✅ |
| `.session-item.active .session-name` | active 会话名 | 条件串：active 时 name 加 `font-medium text-primary` | ✅ |
| `.session-delete-btn` / hover 显隐 / `:hover` (3) | 删除按钮 | `absolute right-1.5 top-1/2 -translate-y-1/2 hidden opacity-60`；hover 显隐无法表达 → 挂账 web-only（VM 常隐,登记） | ⚠️translate 无臂→draft 调整：`items-center` 布局近似 |
| `.session-preview` | 会话项 col | `flex flex-col gap-0.5` | ⚠️gap-0.5 同分数族 |
| `.session-name` | 会话名 | `text-sm text-foreground whitespace-nowrap overflow-hidden text-overflow-ellipsis`（0.85rem≈13.6px→text-sm 14px;text-overflow 无臂,truncate ✅ 替代） | ✅ |
| `.session-count` | 计数 | `text-xs text-muted-foreground`（0.72rem→text-xs） | ✅ |
| `.chats-body` | 主区 col | `flex-1 flex flex-col min-w-0 overflow-hidden` | ✅ |
| `.header-search` / `:focus-within` (2) | 搜索框壳 | `flex items-center gap-1.5 max-w-[320px] px-3 py-1.5 bg-muted-foreground/5 border border-muted-foreground/10 rounded-md text-muted-foreground`；focus-within → web-only.ts 挂账 | ✅+🚪 |
| `.header-search svg` | 图标 | `shrink-0` 落在 Search 组件 class | ✅ |
| `.search-input` / `::placeholder` (2) | 搜索输入 | `w-full text-sm text-foreground bg-transparent border-none outline-none`（原 font 0.82rem≈13px→text-sm）；::placeholder 色 web-only.ts | ✅+🚪 |
| `.chats-canvas` | 消息画布 col | `flex-1 overflow-y-auto p-4 flex flex-col gap-6`（1.4rem≈22.4px→gap-6 24px 近似） | ✅ |

### 1.5 消息流（12 条）→ 切片 T6（chat_message.at）

| 选择器 | 处置 |
|:--|:--|
| `.msg-row` | `flex flex-col gap-0.5 mb-2.5`（0.6rem≈9.6px→mb-2.5 10px）⚠️分数族 |
| `.msg-header` | `flex items-center gap-2 px-1` |
| `.chats-canvas > div:has(.msg-bubble-user) .msg-header` | 条件化：user 行 header 拼 `justify-end`（role 分支在 .at 可表达） |
| `.chats-canvas > div:has(...) .msg-role-badge` ×2 | 条件化：user 徽章 `text-primary`、AI 徽章 `text-muted-foreground` |
| `.msg-bubble` | `px-3.5 py-2.5 rounded-xl text-[15px] leading-[1.6] break-words`（0.92rem≈14.7px→text-[15px]✅arbitrary）⚠️分数族 |
| `.user-text` | `leading-[1.5] break-words`（color:inherit 天然） |
| `:has` 兜底 4 条 | 旧版无气泡包装兜底——ChatMessage 化后 DOM 恒有气泡,确认无引用后直接删（登记核对） |
| `.msg.assistant-msg.draft` | 条件化 `self-start max-w-full`（VM self-* 降级登记） |

### 1.6 输入区（16 条）→ 切片 T6（mention_input.at）

mention_input.at 是逃生舱组件（v-html backdrop），其 DOM 类名与 inject_styles
的 `.input-*` 族一一对应：

| 选择器 | 处置 |
|:--|:--|
| `.chats-input-bar` | 容器类迁 `bg-transparent border-t-0`（!important 由单一真源消除） |
| `.input-inner` | `max-w-[960px] mx-auto w-full` |
| `.input-row` | `flex items-end gap-1.5 w-full` |
| `.input-compose` / `:focus-within` (2) | `relative flex-1 min-w-0 w-auto flex items-center bg-muted-foreground/5 border border-primary/15 rounded-full py-1 px-2 min-h-20`；focus-within 光环 web-only.ts（0.5rem=py-2px…4px 8px→px-2 py-1；80px=min-h-20） |
| `.input-backdrop` | `absolute inset-0 px-2 py-1 bg-transparent border-none rounded-none pointer-events-none overflow-hidden whitespace-pre-wrap break-words text-foreground`（rounded-none ✅） |
| `.input-compose textarea` + `.chats-input` / `:focus` (3) | `block w-full border-none rounded-none bg-transparent text-[15px] resize-none outline-none relative z-[1]`（color:transparent caret 技法为 web mention 专属 → textarea 段落拆 web-only.ts,VM 直显文字） |
| `.send-btn` + `:active`/`:hover`/`:disabled` + 兄弟选择器 4 条 (7) | `w-9 h-9 min-w-9 rounded-full bg-primary text-primary-foreground flex items-center justify-center text-lg shrink-0 hover:opacity-85 disabled:opacity-40`；gradient 背景（brand 渐变）web-only.ts 或改 `bg-primary`（目验裁定,登记） |

### 1.7 SpecsView + 编辑面板（34 条）→ 切片 T7

| 选择器 | 草案要点 | 支持度 |
|:--|:--|:--|
| `.specs-view` / `> div:first-child` | `flex flex-row h-full overflow-hidden`；宽度族见 §1.2（specs 200px） | ✅ |
| `.section-nav-list` | `flex-1 overflow-y-auto flex flex-col gap-1 px-1` | ✅ |
| `.overview-entry` / `:hover` / `.active` (3) | `block w-full text-left py-1.5 px-3 rounded-md text-sm bg-transparent text-muted-foreground hover:bg-accent`；active 拼 `bg-accent text-foreground font-medium` | ✅（py-1.5 ⚠️分数族） |
| `.section-nav-item` / `:hover` / `.active` (3) | 同上,常态 `text-foreground`、active 无色变 | ✅ |
| `.specs-main` | `flex-1 overflow-y-auto flex flex-col` | ✅ |
| `.overview-content` | `p-5 text-sm leading-[1.6] text-foreground`（0.9rem≈14.4px→text-sm） | ✅ |
| `.section-content` | `mb-4 px-5 pb-5` | ✅ |
| `.spec-item-btn` / `:hover` (2) | `block w-full text-left py-2.5 px-3 border border-border rounded-lg mb-1.5 bg-card hover:border-primary` | ⚠️分数族+✅ |
| `.spec-item-main` | `flex items-center gap-2` | ✅ |
| `.spec-item-title` | `font-medium text-[14px] flex-1 text-foreground`（0.88rem） | ✅ |
| `.spec-item-status` | `text-xs py-0.5 px-2 rounded bg-muted text-muted-foreground` | ⚠️分数族 |
| `.spec-item-actions` | `flex gap-1` | ✅ |
| `.edit-panel` | `p-4 border border-border rounded-lg bg-card mb-4` | ✅ |
| `.form-group` / `label` (2) | `flex flex-col gap-1 mb-3`；label `text-[13px] font-medium text-muted-foreground` | ✅ |
| `.form-input` / `:focus` (2) | `py-2 px-2.5 border border-border rounded-md bg-background text-foreground text-sm focus:border-primary`（focus: 🚪） | ⚠️+✅ |
| `.content-input` | `min-h-[120px] font-mono`（resize:vertical 🚪web-only.ts） | ✅+🚪 |
| `.edit-actions` | `flex gap-2 justify-end` | ✅ |
| `.add-btn/.save-btn/.cancel-btn/.action-btn` / hover×2 / `.danger`×2 (8) | 公共 `py-1.5 px-3.5 border border-border rounded-md text-[13px] bg-card text-foreground hover:bg-accent`；add/save 变体 `hover:bg-primary hover:text-primary-foreground hover:border-primary`；danger `text-destructive hover:bg-destructive/10` | ⚠️+✅ |

### 1.8 PlansView（10 条）→ 切片 T7

| 选择器 | 草案 |
|:--|:--|
| `.plans-root` | `flex flex-col h-full overflow-hidden` |
| `.plans-view` / `> div:first-child` | 同 §1.2/§1.4（220px） |
| `.plans-main` | `flex-1 flex flex-col min-w-0 overflow-hidden` |
| `.header-actions` | `flex items-center gap-1 shrink-0` |
| `.session-info-btn` / `:hover` (2) | session_info.at style{} 同名类——见 §2 B13,迁一处删两处 |

### 1.9 WikiView（4 条）→ 切片 T7

| 选择器 | 草案 |
|:--|:--|
| `.wiki-view` / `> div:first-child` | 同上（wiki 240px） |
| `.nav-icon-btn` / `:hover` (2) | `w-7 h-7 inline-flex items-center justify-center border border-border rounded-md bg-transparent text-foreground text-base hover:bg-accent` |
| `.wiki-nav-list` | `flex-1 overflow-y-auto` |

### 1.10 markstream 表格斑马线（1 条）

`.markstream-vue .table-node tbody tr:nth-child(even) td` — markdown 渲染域、
`--ms-muted` 专用 token、nth-child 结构选择器 → web-only.ts 挂账（VM 表格
斑马纹走上游渲染器,非本计划域）。

## 2. 组件 style{} 块清单（37 处 / 23 文件,renderer.vm.at 桩已排除）

> 待澄清①default：纳入本期,gate_card 为代表验收点。体量裁定时降级「二批」
> +KD 挂账,退役条件改「inject_styles 选择器仅余 style{} 块对应项」。
> 每块迁移 = CSS 语义 → 工具类串进 view,块整体删除;无法工具类化的段
> （伪类/伪元素/动画）拆 web-only.ts 挂账。

| # | 文件 | 行 | 切片 | 类族 | 备注 |
|:--|:--|:--|:--|:--|:--|
| B1 | agent_avatar.at | 29 | T8 | avatar-* | 头像圈层 |
| B2 | gate_card.at | 48 | **T6** | gate-card/diff-*/approve-btn… | **代表验收点**：全工具类化+删块（capitalize/uppercase/hover 等 web-only 增强挂账） |
| B3 | errand_card.at | 42 | T8 | errand-* | |
| B4 | chats_view.at | 82 | T6 | branch-row/branch-btn/retry-btn | dashed 边框🚪rounded-full ✅ |
| B5 | chat_message.at | 46 | T6 | msg-bubble/msg-header/user-text… | 与 §1.5 同源迁移 |
| B6 | questionnaire_card.at | 41 | T8 | q-* | |
| B7 | mention_dropdown.at | 37 | T6 | mention-* | |
| B8 | generic_tool_card.at | 48 | T8 | tool-*/seg-* | |
| B9 | secretary_message.at | 42 | T8 | secretary-* | |
| B10 | streaming_table.at | 25 | T8 | streaming-table 外距 | keyframes st-dots 留 inject_styles（动画） |
| B11 | report_card.at | 42 | T8 | report-* | |
| B12 | relay_run_box.at | 134 | T8 | tv-* | 体量最大块之一 |
| B13 | session_info.at | 50 | T6 | session-info-* + **wiki-\*（8 类,归 T7 wiki 域迁）** | 跨域寄居:wiki 类迁 wiki 组件 |
| B14 | raw_preview.at | 53 | T8 | raw-* | |
| B15 | task_plan_card.at | 36 | T8 | tp-* | |
| B16-19 | specs_leaf.at | 29/59/95/247 | T7 | leaf-* | 4 块 |
| B20-24 | specs_editors.at | 40/107/154/355/524 | T7 | editor-*/form-* | 5 块 |
| B25 | settings_menu.at | 80 | T8 | settings-* | |
| B26 | specs_category.at | 196 | T7 | category-* | |
| B27 | think_block.at | 22 | T8 | think-* | |
| B28-35 | specs_detail.at | 38/99/219/293/394/499/570/735 | T7 | detail-* | 8 块,体量最大 |
| B36 | workspace_selector.at | 84 | T8 | ws-* | |
| B37 | wiki_nav.at | 68 | T7 | wiki-nav-item… | 与 B13 wiki 段同域合迁 |

## 3. class.rs 支持度探针（T1 输出留档）

- 断言文件：auto-lang `.worktrees/auto-musk-dev`
  `crates/auto-lang/src/plan449_style_parity_tests.rs`（`style_migration_probe`，
  116 token 逐类断言，verdict=ok/variant/gap）。
- 运行：`cargo test -p auto-lang --lib --features ui-iced style_migration_probe -- --nocapture`
- 结果：**total=116 ok=95 variant=7 gap=14，PASS**（全量输出
  `t1-class-probe.txt`）。

### gap 清单与裁定（D3 候选 vs 白名单）

| token | 裁定 | 说明 |
|:--|:--|:--|
| `px-2.5` / `py-2.5`（含 mb-0.5/py-0.5/py-1.5/px-3.5/mb-2.5 等 0.5 步进族） | **D3 已修**（T3 前完成） | p/m 族补分数步进臂（Pixels(N*4px)，对齐 gap 族先例）；TDD 双测锁定 |
| `items-baseline` | **D3 已修（降级臂）** | 解析保存为 ItemsStart（iced 无基线对齐，Plan 412 降级矩阵先例）；对拍归一化表记 baseline→flex-start 等价 |
| `border-r`（单侧边框族） | 白名单（VM 降级） | iced 边框均匀宽,无单侧概念;保留 web 效果,VM 无分隔线（048 已接受态） |
| `underline` | 白名单 | iced 文本无下划线;login 切换按钮 VM 无下划线（登记） |
| `z-[100]` | 草案避用 | 迁移时 session-info-tooltip 用 `z-50`（视觉等价,登记） |
| `select-none/uppercase/capitalize/italic/tracking-wide/resize-y/appearance-none/animate-pulse` | 白名单（web-only 增强） | iced 无对应属性 |

### 白名单（进 norm.json,报告列「web-only 增强」不计失败）

`hover:*`、`focus:*`、`disabled:*`、`placeholder:*`、`transition-*`（除
transition-colors 有臂）、`cursor-*`、`animate-*`、上表白名单行、伪类/
伪元素/动画/主题变量/滚动条段（web-only.ts 承接）。

## 4. 切片归属总表（执行时勾销）

| 切片 | 任务 | 覆盖 | 状态 |
|:--|:--|:--|:--|
| T4 | 导航栏收尾 | §1.3 全部 + §1.2 app-header 残段删除 + 对拍用例 | ☐ |
| T5 | 登录页 | login.at 核验（§3 gap 修复为前置）+ 对拍用例 | ☐ |
| T6 | 会话壳 | §1.4 + §1.5 + §1.6 + B2/B4/B5/B7/B13(session-info 段) | ☐ |
| T7 | plans/specs/wiki | §1.7 + §1.8 + §1.9 + B13(wiki 段)/B16-28/B37 | ☐ |
| T8 | 杂项+退役 | B1/B3/B6/B8-B12/B14/B15/B25/B27/B36 + §1.10 + web-only.ts 定稿 + inject_styles.ts 删除 | ☐ |
