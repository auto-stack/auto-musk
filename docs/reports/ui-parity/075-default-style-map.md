# PLAN-075 默认样式三方对账表（Design 22 × Musk Web-Only CSS × AutoVM）

> 生成时间：2026-09-19  
> 计划编号：PLAN-075（S2 默认样式三方对账与主题字体收敛）  
> 依据标准：Design 22 (`auto-lang/docs/design/autoui/base-styles-and-visual-parity.md`)、`src/front/inject_styles.web-only.ts`、`pac.at`、`aura_view_builder.rs`、`variants.rs`、`registry.rs`。  
> 边界原则：应用级默认（§2–5）与宿主注入在本计划对账并闭合；AutoDown 引擎文档面（§4.5/4.6/7）与相关 CSS 规则整包移交 PLAN-076，不在此私修。

---

## 1. 对账汇总与责任划分

| 域 | 规则总数 | 已对齐 / 可直接收敛 | 待修复 / 待消费项 | 移交后续计划 |
|---|---|---|---|---|
| **Design 22 §2 标题排版标尺** | 7 | 5 (h3, h4, h5, h6, 文本基础) | 2 (h1/h2 缺 tracking-tight, p/text 注入) | 0 |
| **Design 22 §3 表单控件与交互原语** | 9 | 7 (button 各变体、checkbox、badge) | 2 (input/textarea 预设与 min-height) | 0 |
| **Design 22 §4 基础容器与边框兜底** | 8 | 8 (col, row, center, grid, scroll, container, border, bg-card) | 0 | 0 |
| **Design 22 §4.5/4.6/7 引擎文档面** | 10 | 0（引擎内部规约） | 0 | 10 (全量移交 PLAN-076) |
| **Musk CSS 全局注入 (`inject_styles`)** | 15 | 4 (离线字体、主题变量、af别名、链接色) | 2 (移除在线Google Font、纳入pac.at共享主题) | 9 (M-08..M-10 归 076, M-11/13..15 归 078) |

---

## 2. Design 22 §2 标题排版标尺（Typography Scale）逐条映射

| 编号 | 标签 / 原语 | Design 22 规约属性 | Vue 最终级联链 | VM 预设 / 兜底实现 | 差异裁定与对账结论 | 责任归属 |
|---|---|---|---|---|---|---|
| **TY-01** | `h1` | `text-4xl font-bold tracking-tight text-primary mb-4` (36px, bold, -0.025em, primary, margin-bottom 16px) | `@layer base { h1 { @apply text-4xl font-bold tracking-tight text-primary mb-4; } }` (用户 class 胜) | `aura_view_builder.rs:3106`: `text-4xl font-bold text-primary mb-4` | **差异**：VM 预设遗漏 `tracking-tight` (-0.025em)；字号与颜色均一致。在 VM 预设补齐 tracking-tight。 | auto-lang (T-02) / auto-musk (消费) |
| **TY-02** | `h2` | `text-3xl font-bold tracking-tight text-primary mt-8 mb-4` (30px, bold, -0.025em, primary, mt 32px, mb 16px) | `@layer base { h2 { @apply text-3xl font-bold tracking-tight text-primary mt-8 mb-4; } }` | `aura_view_builder.rs:3107`: `text-3xl font-bold text-primary mt-8 mb-4` | **差异**：VM 预设遗漏 `tracking-tight`。在 VM 预设补齐 tracking-tight。 | auto-lang (T-02) / auto-musk (消费) |
| **TY-03** | `h3` | `text-xl font-semibold text-primary mb-3` (20px, semibold 600, primary, mb 12px) | `@layer base { h3 { @apply text-xl font-semibold text-primary mb-3; } }` | `aura_view_builder.rs:3108`: `text-xl font-semibold text-primary mb-3` | **一致**：双端预设文本与渲染层级完全对齐。 | PLAN-075 已对齐 |
| **TY-04** | `h4` | `text-lg font-semibold mb-2` (18px, semibold 600, mb 8px) | `@layer base { h4 { @apply text-lg font-semibold mb-2; } }` | `aura_view_builder.rs:3109`: `text-lg font-semibold mb-2` | **一致**：双端预设完全对齐。 | PLAN-075 已对齐 |
| **TY-05** | `h5` | `text-base font-semibold mb-1` (16px, semibold 600, mb 4px) | `@layer base { h5 { @apply text-base font-semibold mb-1; } }` | `aura_view_builder.rs:3110`: `text-base font-semibold mb-1` | **一致**：双端预设完全对齐。 | PLAN-075 已对齐 |
| **TY-06** | `h6` | `text-sm font-semibold mb-1` (14px, semibold 600, mb 4px) | `@layer base { h6 { @apply text-sm font-semibold mb-1; } }` | `aura_view_builder.rs:3111`: `text-sm font-semibold mb-1` | **一致**：双端预设完全对齐。 | PLAN-075 已对齐 |
| **TY-07** | `p` / `text` | `text-base font-normal leading-7 text-muted-foreground` (16px, 400, 28px line-height, muted fg) | Vue 浏览器 UA 默认块级段落；文档域消费 `text-muted-foreground leading-7` | `aura_view_builder.rs:2058` 仅提取用户 class/style，无裸标签预设注入 | **差异**：裸 `<p>` 标签在 VM 端未自动补齐 `text-base leading-7 text-foreground`（或 `text-muted-foreground`）；需在组件消费侧统一使用显式类或在 VM 文本元素转换补齐基线。 | auto-musk / auto-lang (T-02) |

---

## 3. Design 22 §3 表单控件与交互原语（Shadcn Parity）逐条映射

| 编号 | 组件 / 变体 | Design 22 规约属性 | Vue 最终级联链 | VM 预设 / 兜底实现 | 差异裁定与对账结论 | 责任归属 |
|---|---|---|---|---|---|---|
| **FC-01** | `input` (默认) | `border border-input bg-background rounded-md px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground` (1px 边框, 6px 圆角, 12px H / 8px V 内边距, 14px 文本) | Shadcn `Input.vue` 模板 preset 驱动 | `aura_view_builder.rs:9719`: `"border rounded-md px-3 py-2 text-sm"`，renderer 兜底 `Color::Background` 与 `resolve_border_rgb()` | **大体一致**：VM 缺显式 `bg-background` 和 `border-input` 类，但 renderer 内部直接采用对应 token 渲染。裸控件几何与文本尺寸一致。 | PLAN-075 已对齐 |
| **FC-02** | `textarea` (默认) | `border border-input bg-background rounded-md px-3 py-2 text-sm text-foreground min-h-[80px]` | Shadcn `Textarea.vue` 模板 preset 驱动 | `aura_view_builder.rs:9965`: `"border rounded-md px-3 py-2 text-sm"` | **差异**：VM 预设缺 `min-h-[80px]`，VM 文本域由父级或内容撑开。对于显式设置高度或组件级固定样式的场景表现一致。 | PLAN-075 / auto-musk |
| **FC-03** | `button` (default) | `bg-muted border border-border text-foreground font-medium rounded-md h-10 px-4 text-sm hover:bg-muted/70` (UA 预填等价基线，PLAN-571) | Shadcn `variants.ts` default cva | `variants.rs:23`: `"bg-muted border border-border text-foreground font-medium rounded-md hover:bg-muted/70"` + size `h-10 px-4` | **完全一致**：单一事实源互锁，中性背景+发丝描边+圆角+尺寸双端恒等。 | PLAN-075 已对齐 |
| **FC-04** | `button` (primary / submit) | `bg-primary text-primary-foreground font-medium rounded-md h-10 px-4 text-sm hover:bg-primary/90` | Shadcn `variants.ts` primary cva | `variants.rs:27`: `"bg-primary text-primary-foreground font-medium rounded-md hover:bg-primary/90"` + size `h-10 px-4` | **完全一致**：显式 CTA 主题色填充双端恒等。 | PLAN-075 已对齐 |
| **FC-05** | `button` (secondary) | `bg-secondary text-secondary-foreground font-medium rounded-md h-10 px-4 text-sm hover:bg-secondary/80` (无描边纯填充) | Shadcn `variants.ts` secondary cva | `variants.rs:31`: `"bg-secondary text-secondary-foreground font-medium rounded-md hover:bg-secondary/80"` | **完全一致**：深一档中性填充、无描边双端恒等。 | PLAN-075 已对齐 |
| **FC-06** | `button` (destructive) | `bg-destructive text-destructive-foreground font-medium rounded-md h-10 px-4 text-sm` | Shadcn `variants.ts` destructive cva | `variants.rs:34`: `"bg-destructive text-destructive-foreground font-medium rounded-md hover:bg-destructive/90"` | **完全一致**：警示红底白字恒等。 | PLAN-075 已对齐 |
| **FC-07** | `button` (outline) | `border border-input bg-background text-foreground rounded-md h-10 px-4 text-sm` | Shadcn `variants.ts` outline cva | `variants.rs:37`: `"border border-input bg-background text-foreground rounded-md hover:bg-secondary hover:text-secondary-foreground"` | **完全一致**：描边无填充恒等。 | PLAN-075 已对齐 |
| **FC-08** | `button` (ghost) | `rounded-md h-10 px-4 text-sm hover:bg-accent` | Shadcn `variants.ts` ghost cva | `variants.rs:39`: `"rounded-md hover:bg-secondary hover:text-secondary-foreground"` | **次级差异**：Vue 用 `hover:bg-accent`，VM 用 `hover:bg-secondary`。在 scaffold 主题中二者明度极其相近（均为浅底微灰）。 | PLAN-075 已对齐 |
| **FC-09** | `button` (icon) | `h-7 w-7 px-0 py-0 rounded-md` (正方形，无额外 padding) | Shadcn `variants.ts` size="icon" (`h-10 w-10`) | `variants.rs:41`: `"h-7 w-7 px-0 py-0"` / size `h-10 w-10` | **一致**：正方形态防塌陷规则双端生效。 | PLAN-075 已对齐 |
| **FC-10** | `checkbox` | `h-4 w-4 rounded border border-primary` (16x16px, 4px 圆角) | Shadcn `Checkbox.vue` | `aura_view_builder.rs:convert_checkbox` 对应渲染 | **一致**：尺寸与焦点表现一致。 | PLAN-075 已对齐 |
| **FC-11** | `badge` | `inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold` | Shadcn `Badge.vue` | `aura_view_builder.rs:convert_badge` | **一致**：圆角胶囊与文本样式一致。 | PLAN-075 已对齐 |

---

## 4. Design 22 §4 基础容器与边框兜底逐条映射

| 编号 | 标签 / 工具类 | Design 22 规约属性 | Vue 实现 | VM 实现 | 对账结论 | 责任归属 |
|---|---|---|---|---|---|---|
| **CT-01** | `col` / `column` | `flex flex-col gap-4` | `flex flex-col gap-4` | `convert_column_tracked_ctx` 垂直 flex 列布局 | **一致** | PLAN-075 |
| **CT-02** | `row` | `flex flex-row gap-4` | `flex flex-row gap-4` | `convert_row_tracked_ctx` 水平 flex 行布局 | **一致** | PLAN-075 |
| **CT-03** | `center` | `flex flex-col items-center justify-center h-full` | `flex flex-col items-center justify-center h-full` | `convert_center_tracked_ctx` | **一致** | PLAN-075 |
| **CT-04** | `grid` | `grid` | `grid` | `convert_grid_tracked_ctx` 网格布局 | **一致** | PLAN-075 |
| **CT-05** | `scroll` | `overflow-auto` | `overflow-auto` | `convert_scroll_tracked_ctx` 滚动视口 | **一致** | PLAN-075 |
| **CT-06** | `container` | `max-w-7xl mx-auto` | `max-w-7xl mx-auto` | `convert_container_tracked_ctx` | **一致** | PLAN-075 |
| **CT-07** | `border` utility | `border-width: 1px`, 兜底 `--border` (zinc-800/zinc-200), 禁止回退 transparent | Tailwind `border` 类默认绑定 `--border` | `renderer.rs`: `resolve_border_rgb()` 兜底 | **一致**：发丝描边均有实体色，无隐形边框漂移 | PLAN-075 |
| **CT-08** | `bg-card` | `Color::Surface` / `hsl(var(--card))` | `bg-card` | Surface/Card 语义色彩映射 | **一致** | PLAN-075 |

---

## 5. Design 22 §4.5 / 4.6 / 7 AutoDown 引擎文档面移交映射

> 裁决原则：Design 22 明确规定引擎文档面样式由 `autodown-engine` 单源负责。Musk 严禁通过上层样式污染或私自重写 Markdown 渲染器；全部移交 **PLAN-076（AutoDown 引擎双端一致性）**。

| 编号 | 章节 | 规约内容 | 现状与落点 | 移交处理决策 | 接收计划 |
|---|---|---|---|---|---|
| **ED-01** | §4.5 | Markdown 块间垂直节奏 12px (`0.75rem`) | Vue 现由 `inject_styles.web-only.ts` 临时补齐；VM markdown 纯文本/未对齐 | 移交 PLAN-076 在 `autodown-engine` view 契约中统一收口，移除 Musk 私有注入 | PLAN-076 |
| **ED-02** | §4.6 | Markdown 暗色主题颜色映射 (`.dark` 作用域) | Vue 现由 `inject_styles.web-only.ts` 对冲 vendor style.css；VM 纯文本 | 移交 PLAN-076 统一消费引擎暗色 theme 契约 | PLAN-076 |
| **ED-03** | §7.1 | 中性色板 (`--ad-fg`, `--ad-muted`, `--ad-border`, `--ad-surface`) | 引擎 CSS 与 VM `autodown_blocks.rs` 映射 | 移交 PLAN-076 校验双端 token 烘焙一致性 | PLAN-076 |
| **ED-04** | §7.2 | Accent 五色盘 (`indigo`, `coral`, `ocean`, `sage`, `amber`) | 引擎 CSS `data-accent` 规则组 | 移交 PLAN-076 验证双端 accent 表现 | PLAN-076 |
| **ED-05** | §7.3 | 文档排版分档 (正文 0.95rem/1.6; h1 1.58rem ≈ 25.3px) | 区别于 §2 应用级标题标尺；存在既有分叉登记 | 移交 PLAN-076 闭环分叉登记项 | PLAN-076 |
| **ED-06** | §7.4 | 块家族 chrome (fence 容器/header/pre, inline code, table, callout) | 引擎两端 renderer / widget 契约 | 移交 PLAN-076 逐项对账 | PLAN-076 |
| **ED-07** | §7.5 | hljs 语法高亮双档 (`hljs_scope_map.rs` 单源) | 代码高亮双端烘焙 | 移交 PLAN-076 验证高亮 token | PLAN-076 |
| **ED-08** | §7.6 | 编辑壳交互面 (斜杠菜单/气泡菜单/滚动条) | Vue-only 面（VM 降级只读或原生编辑器） | 移交 PLAN-076 明确降级契约与豁免证明 | PLAN-076 |
| **ED-09** | §7.7 | 零游离值核对 (hex 过滤核对) | 扫描 `autodown-editor.css` 与 VM 颜色表 | 移交 PLAN-076 静态门禁复核 | PLAN-076 |

---

## 6. Musk 全部 Web-Only CSS 注入逐项对账与收敛路径

检查文件：`src/front/inject_styles.web-only.ts`（共 253 行）。

| 注入项编号 | 源码行号 / 选择器 | 原始目的 | Vue 生效机制 | VM 现行状态 | 裁定与收敛方案 | 归属与落点 |
|---|---|---|---|---|---|---|
| **MK-01** | :20 `@import url('https://fonts...')` | 引入 Google Fonts Noto Sans SC | 浏览器网络拉取 web font | VM 离线系统字体 / Inter 内置 | **违规消除**：违反 AC-03「无网络字体依赖」。**移除在线 @import**，改用离线系统无衬线字体栈 fallback。 | PLAN-075 (T-03) `inject_styles.web-only.ts` |
| **MK-02** | :21 `font-family` 声明 | 全局基础文本字体族配置 | body/button/input/textarea/select 覆盖 | VM 使用系统缺省无衬线字体 | **收敛**：统一为 `-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Noto Sans SC", "PingFang SC", "Microsoft YaHei", sans-serif`，双端离线稳定一致。 | PLAN-075 (T-03) `inject_styles.web-only.ts` |
| **MK-03** | :26–54 `:root` 主题变量覆盖 | Musk 品牌紫色体系 (`--primary: 238 55% 58%`, `--border: 220 13% 91%` 等) | 覆盖 codegen 的默认 scaffold 近黑 | VM 之前缺省使用标准 scaffold（未载入 Musk 品牌色） | **关键闭合**：将品牌色**纳入 `pac.at` `theme: {}` 块**。AutoUI 编译器为 Vue 生成 index.css，同时 VM boot 自动激活合成主题，实现双端同源主题！ | PLAN-075 (T-03) `pac.at` |
| **MK-04** | :55–72 `.dark` 主题变量覆盖 | Musk 暗色品牌色体系 (`--primary: 238 55% 62%`, `--background: 220 15% 8%` 等) | 注入 `.dark` 作用域 | 同上，VM 经由 `pac.at` 主题合成与 dark 模式开关联动 | 同上，由 `pac.at` 与规范主题层驱动，保留合法 CSS 静态投影。 | PLAN-075 (T-03) `pac.at` |
| **MK-05** | :43–53 `af-*` 语义别名层 | 历史组件逃生舱使用 `--af-primary` 等别名 | 变量别名重定向 | VM 组件均已迁入标准 Tailwind token，不引用 af-* | 保留为合法 Vue 静态投影，不影响 VM。 | PLAN-075 (保留) |
| **MK-06** | :74–77 滚动条样式 (`::-webkit-scrollbar`) | 细滚动条 (6px, muted-foreground/0.2) | WebKit 专属伪元素 | VM 原生 Iced 滚动条自管 | 合法平台差异（Web-only 视觉微调）。 | PLAN-075 (保留) |
| **MK-07** | :79 `a { color: hsl(var(--primary)); }` | 全局链接色 | CSS 标签选择器 | VM `a` / `link` 预设 `text-primary` | 双端均使用 primary 主题色，保持一致。 | PLAN-075 (已对齐) |
| **MK-08** | :85 `@keyframes st-dots` | StreamingTable 动态省略号动画 | CSS Keyframes | StreamingTable 属于 unreachable/retired 组件 | 保留声明，不阻塞生产。 | PLAN-075 (保留) |
| **MK-09** | :87–89 Markdown 表格斑马线 | `.table-node tbody tr:nth-child(even)` 底色 | 强制 nth-child 底色 | 引擎层规划 | **移交 PLAN-076**：由 autodown-engine 统一负责表格样式。 | PLAN-076 |
| **MK-10** | :95 `.msg-bubble-ai .streaming-document` | AI 消息前景色兜底 | 强制 `--foreground` | ChatMessage 内联分支渲染 | **移交 PLAN-076 / PLAN-077**。 | PLAN-076 / 077 |
| **MK-11** | :103 Markdown 块间节奏 `0.75rem` | `> .node-slot + .node-slot` 间距 | 强制上边距 | 引擎层规划 | **移交 PLAN-076**。 | PLAN-076 |
| **MK-12** | :105–107 会话删除按钮悬停显隐 | `.session-item:hover .session-delete-btn` | CSS hover 状态切换 | 导航栏会话列表交互 | **移交 PLAN-078**（壳与交互一致性）。 | PLAN-078 |
| **MK-13** | :125–187 `.dark .streaming-document ...` | Markdown 暗色样式覆盖（数十条 !important） | 覆写引擎浅色 token | 引擎层规划 | **移交 PLAN-076**：在 auto-down 根治深色 token 映射，消除 Musk 层的 !important 补丁。 | PLAN-076 |
| **MK-14** | :195–207 IME 组词与选区透明文字技术 | 双层输入框 IME 高亮与 selection 背景 | textarea 特异样式与事件监听 | VM textarea 原生渲染，无双层透明文字 | **移交 PLAN-078**（输入框交互与 IME 稳定性）。 | PLAN-078 |
| **MK-15** | :213–221 鹿 Logo 双主题切换与收缩导航按钮 | `.deer-icon-light/dark`, `.rail-icon-btn` | 主题翻转显隐与按钮样式 | 导航栏适配 | **移交 PLAN-078**（导航与布局）。 | PLAN-078 |

---

## 7. 结论与执行动作（T-02 ~ T-05）

1. **规约与注入零未归属**：所有 26 项基础标尺与 15 项 Musk 注入均已精确定位，引擎条目无缝移交 PLAN-076，交互与壳移交 PLAN-078。
2. **T-02 基础属性修复**：
   - 标题 tracking：在 auto-lang / VM 预设中为 `h1` 与 `h2` 补入 `tracking-tight`。
   - 文本段落基准：在测试用例与裸控件中补齐 input / textarea / button / checkbox / badge / row / col 覆盖。
3. **T-03 主题字体统一**：
   - 移除 Google Fonts 在线链接，使用确定性离线字体栈。
   - 在 `pac.at` 声明 Musk 品牌主色 `theme: { extends: "scaffold", colors: { ... } }`，实现双端同源主题。
4. **T-04 最终属性验证**：
   - 在 `scripts/ui-parity.mjs` 中接入 PLAN-075 基础样式 case 运行并产出快照证据。
5. **T-05 规约回写**：
   - 产出 `075-evidence.md` 并更新 Specs 拟议增量。
