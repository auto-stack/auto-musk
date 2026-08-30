# 迁移矩阵：inject_styles.web-only.ts → .at 单一真源（PLAN-050 增补 / 049 二批收口）

> 目标（用户指令 2026-08-29）：web-only CSS 注入的默认内容全部切到 Auto 组件声明，
> 让 VM 与 Vue 看到同一份声明源。本文 = 全量规则盘点 + 迁移机制 + VM 行为 + 验证点，
> 按此逐条执行即可（每条独立可验证）。
>
> VM 支持度实测（class.rs @ auto-musk-dev worktree）：
> `hidden`=解析(991)、`truncate`=解析(942)、`hover:*`=按钮样式重求支持（Plan 409/414）；
> `text-transparent`/`group-hover:`/`placeholder:`/`focus-within:`/`caret-*`/`active:`/
> `scale-*`/`opacity-*`=**不解析**（VM 静默忽略,元素落默认形态）。
> 推论：`hidden + group-hover:flex` 组合在 VM=永久隐藏（禁用）；`text-transparent`
> 内联对 VM 安全（VM 输入框保持直接显字）。

## A. 可内联（迁移后 web-only 对应规则删除）

| # | web-only 规则 | 迁移机制（.at 落点） | VM 行为 | 验证 |
|---|---|---|---|---|
| A1 | `.input-compose textarea,.chats-input` 的 `color:transparent+caret`（121-131） | mention_input.at textarea 类串补 `text-transparent caret-[hsl(var(--foreground))]`；`display/width/border/bg/outline` 段已有等价内联，删除 web-only !important 块 | VM 不解析→输入框正常显字（既有语义）；web 双层技术保持 | web：输入文字单层+光标可见；VM：输入显字 |
| A2 | `.chats-input:focus`（132） | textarea 类串补 `focus:outline-none focus:shadow-none focus:border-none` | VM 忽略（outline 本就无） | web：聚焦无环 |
| A3 | `.input-compose:focus-within`（117-120）border 部分 | 容器已有 `focus-within:border-primary/45` 内联 → web-only 的 border-color 声明删除（!important 冗余） | VM 忽略 | web：聚焦边框变色 |
| A3b | 同上 box-shadow 光环 | 容器补 `focus-within:shadow-[0_0_0_3px_hsl(var(--primary)/0.08)]`（web 解析;VM 忽略） | VM 忽略 | web：聚焦光环 |
| A4 | `.send-btn:active`（135） | 发送钮类串补 `active:scale-95`（web 解析;VM 忽略） | VM 忽略 | web：按下缩放 |
| A5 | `.search-input/.wiki-search-input::placeholder`（109-110） | 对应 input 类串补 `placeholder:text-muted-foreground` | VM 忽略（placeholder 用主题色） | web：placeholder 灰 |
| A6 | `.header-search:focus-within`（106-108） | 搜索框容器补 `focus-within:border-primary/35` | VM 忽略 | web：聚焦边框 |

落点文件：`src/front/mention_input.at`（A1-A3b）、`src/front/chats_view.at`（A4，
发送钮在 ChatInput 区）、`src/front/content_header.at` + wiki 视图搜索框（A5/A6）。

## B. 需小型重构（helper/结构变更后再迁）

| # | web-only 规则 | 阻塞点 | 迁移方案 |
|---|---|---|---|
| B1 | `.user-text .inline-mention`（94-97）+ `.msg-bubble-user .user-text(.inline-mention)`（98-99） | inline-mention span 由 `mention_helpers.at` 的 render 函数产 HTML,样式靠后代选择器按上下文（输入 backdrop / 用户气泡）区分 | render 函数加 context 参数,按上下文直接发完整内联类串（输入态=蓝 tint;气泡态=白字+白 15% 底）;`whitespace-pre-wrap`/`text-primary-foreground` 同步内联进 user_message.at 的 user-text span |
| B2 | `.msg-bubble-ai .streaming-document`（100） | 目标类在第三方 @autodown scoped 样式之后,需后代选择器强制 | 短期留 web-only（登记"第三方覆盖"类）;长期=@autodown 消费色 token 或 Markdown 挂显式色 |

## C. 留守 web-only（web 平台基座,iced 有等价物或无对应概念）

| # | 规则 | 理由 |
|---|---|---|
| C1 | 字体 @import + body font-family（19-21） | 全局文档级;VM iced 主题字体自管（theme.rs） |
| C2 | `:root/.dark` 主题变量 + af-* 别名（26-72） | web CSS 变量体系;VM 色板=theme.rs resolve_semantic_rgb（已对齐 shadcn dark,Plan 448/455） |
| C3 | 滚动条（74-77）/全局链接色（79） | web 文档级 |
| C4 | st-dots keyframes（85）/markstream 斑马线（87-89） | web 渲染技法;VM 无该组件形态 |
| C5 | `@autodown/vue/style.css` 引入（16） | 第三方依赖样式 |

## D. 特例警示（不得内联）

- **hover 显隐**（`.session-item:hover .session-delete-btn` 102-104、`.tree-item:hover .tree-item-del` 112-114）：`hidden` VM 解析但 `group-hover:flex` 不解析 → group 模式会让 VM 永久隐藏。现行 VM 形态=常显低透明度（可接受）。若要统一,改用"常显 opacity-40 + hover:opacity-100"形态内联（web 视觉变化需用户确认）,否则留守 web-only。
- **`text-transparent`**：VM 不解析恰好是正确行为（VM 输入直接显字）——这是唯一"VM 不解析反而正确"的规则,内联时必须保住这一非对称。

## 执行顺序建议

A1-A6 一批（同文件簇,web 逐项截图+VM 截图双验）→ B1 一批（helper 重构,mention
双上下文对拍）→ B2/C 留守登记（web-only.ts 头注改为"平台基座+第三方覆盖+hover 显
隐特例"三类清单）。全部完成后 web-only.ts 预计从 145 行缩至 ~60 行纯平台基座。

## E. 留守能力的 VM 对应物核验（2026-08-29 补记）

| 留守项 | VM 对应物 | 核验结果 |
|---|---|---|
| C1 字体（Noto Sans SC + 系统 fallback） | iced 内置 **Inter**（renderer.rs:1367 INTER_FONT） | ⚠️ **缺口**：Inter 无 CJK 字形,中文回退系统字体;两轨中西文形态不一致。可修：VM 侧主题字体可配/内嵌 CJK 子集 |
| C2 主题变量（web-only 覆盖的"原版品牌紫"暗色板） | theme.rs resolve_semantic_rgb（Plan 448/455 对齐**生成端 shadcn dark 默认值**） | ⚠️ **部分缺口**：background/card/primary-foreground/muted-foreground 已对齐;但 **secondary/border/muted/accent 走 shadcn 默认**,而 web-only .dark 覆盖为"原版"值（如 --secondary: 220 12% 16%）→ 两轨暗色板微妙不同（实测 rail bg 一致属巧合命中,其余令牌待逐个比对） |
| C3 滚动条/全局链接色 | iced Scrollable 原生滚动条（样式不可 CSS 化） | ✅ 形态等价（视觉细节归 iced） |
| C4 st-dots keyframes / markstream 斑马线 | 无（markdown 的 VM 渲染路径不同） | ✅ web 渲染技法,无 VM 需求 |
| C5 @autodown style.css | VM 不经该组件渲染 markdown | ✅ 不适用 |
| B2 `.msg-bubble-ai .streaming-document` 色覆盖 | 同上 | ✅ 不适用（待 markdown 一致化批次再议） |

**新墧行动项**：①theme.rs 暗色 secondary/border/muted/accent 对齐 web-only .dark
覆盖值（消除两轨底色/边线色差）；②VM 主题字体策略（CJK 回退）。两者归
auto-lang 渲染器批次（与 009 号需求说明同批立项）。
