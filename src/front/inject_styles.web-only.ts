// inject_styles.web-only.ts — web 专属全局样式（PLAN-049 T8 退役产物）
//
// PLAN-049 双轨收敛后,组件自定义类已全量迁 .at 内联 tailwind 工具类
// （单一样式源）;原 inject_styles.ts 退役,余量按 D4 判据拆入本文件：
//   1. 全局段（字体/主题变量/滚动条/链接色/@autodown/vue/style.css 引入）
//      ——iced 无对应概念,web 专属;
//   2. web-only 增强（伪类链/伪元素/后代选择器/悬停显隐/@mention 双层文字
//      技术/斑马线/动画 keyframes）——工具类无法表达,VM 白名单登记。
// 二批挂账：各组件 style{} 块（specs_leaf/editors/detail/category、errand/ws/
// settings 等余 30 块）仍为 scoped 生效,迁移归二批（KNOWN-DEBT 049 行）。
// 平台绑定：platformInjectStyles（platform.web.at → 本文件;VM 侧 no-op）。

// PLAN-038 T12 → 0.2.0 收口:渲染样式唯一来源 = @autodown/vue/style.css
// （vendor 0.2.0 起 markstream-vue 消灭,其全局 index.css 随依赖移除;
// 上游样式改 scoped data-attr 形态,表格/代码块等 design token 内含）。
import '@autodown/engine/style.css'

const STYLES = `
/* ── 字体（Noto Sans SC）── */
@import url('https://fonts.googleapis.com/css2?family=Noto+Sans+SC:wght@400;500;700&display=swap');
body, button, input, textarea, select { font-family: 'Noto Sans SC', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; }

/* ── 主题变量覆盖（对齐原版 theme.css，Plan 022 视觉对齐）──
   codegen 生成的 index.css 用 shadcn 默认值（primary 近黑），
   这里覆盖为原版的品牌紫色体系 + af-* 语义别名层 + 滚动条。 */
:root {
  --primary: 238 55% 58%;
  --primary-foreground: 0 0% 100%;
  --foreground: 220 15% 20%;
  --card: 0 0% 100%;
  --card-foreground: 220 15% 20%;
  --secondary: 220 14% 96%;
  --secondary-foreground: 220 15% 20%;
  --muted: 220 14% 96%;
  --muted-foreground: 220 9% 46%;
  --accent: 220 14% 96%;
  --accent-foreground: 220 15% 20%;
  --destructive: 0 72% 51%;
  --border: 220 13% 91%;
  --input: 220 13% 91%;
  --ring: 238 55% 58%;
  --radius: 0.5rem;
  /* af-* 语义别名（原版组件逃生舱 CSS 用这些） */
  --af-bg: hsl(var(--background));
  --af-fg: hsl(var(--foreground));
  --af-card: hsl(var(--card));
  --af-muted: hsl(var(--muted-foreground));
  --af-border: hsl(var(--border));
  --af-input: hsl(var(--input));
  --af-primary: hsl(var(--primary));
  --af-primary-fg: hsl(var(--primary-foreground));
  --af-primary-soft: hsl(var(--primary) / 0.08);
  --af-secondary: hsl(var(--secondary));
}
.dark {
  --background: 220 15% 8%;
  --foreground: 220 10% 92%;
  --card: 220 15% 10%;
  --card-foreground: 220 10% 92%;
  --primary: 238 55% 62%;
  --primary-foreground: 220 15% 8%;
  --secondary: 220 12% 16%;
  --secondary-foreground: 220 10% 92%;
  --muted: 220 12% 16%;
  --muted-foreground: 220 9% 58%;
  --accent: 220 12% 16%;
  --accent-foreground: 220 10% 92%;
  --destructive: 0 62% 45%;
  --border: 220 12% 18%;
  --input: 220 12% 18%;
  --ring: 238 55% 62%;
}
/* 滚动条（对齐原版） */
::-webkit-scrollbar { width: 6px; height: 6px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: hsl(var(--muted-foreground) / 0.2); border-radius: 3px; }
::-webkit-scrollbar-thumb:hover { background: hsl(var(--muted-foreground) / 0.35); }
/* 全局链接色 */
a { color: hsl(var(--primary)); }
/* ── 三列渐变背景层次 / plans / specs / wiki 布局 ──
   PLAN-049 T7：三域布局骨架与编辑面板表单全部迁对应 .at 内联工具类
   （NavSidebar width_class 参数化;编辑面板段为 plans/specs/wiki 共用,
   已在各消费者落同串工具类）,原全局规则整段删除。 */
/* ── StreamingTable keyframes（023 P3,动画 web-only）── */
@keyframes st-dots { 0%, 80%, 100% { content: ''; } 40% { content: '.'; } 60% { content: '..'; } }
/* ── Markdown 表格补充：斑马线背景（0.2.0 scoped 样式已含边框/表头底色,斑马纹为 musk 增补） ── */
.markstream-vue .table-node tbody tr:nth-child(even) td {
  background: hsl(var(--ms-muted) / 0.55);
}
/* ══ PLAN-049 web-only 增强暂存（T8 拆出 inject_styles.web-only.ts）══
   工具类无法表达：伪类链/伪元素/后代选择器/悬停显隐/输入透明文字技术。
   VM 轨对这些无映射（登记白名单）,仅 web 生效。 */
/* PLAN-050 B1: mention 高亮已内联 mention_helpers.at（按上下文发完整类串）
   与 user_message.at（user-text 显式气泡内文字色）——后代选择器规则删除。 */
.msg-bubble-ai .streaming-document { color: hsl(var(--foreground)); }
/* PLAN-056 T6（T7 后复核仍必需）：markdown 块间节奏。engine 0.5.0 升级后
   实测 DOM 仍为 .streaming-document > .markdown-renderer(单个) > .node-slot
   兄弟流，上游 segment 规则 .streaming-document > *+* 只管"多 segment"场景
   （命中不了单文档内的 slot），slot 间距段在上游 CSS 中缺位——本规则即该
   缺位的 musk 侧实现。规约登记 auto-lang
   docs/design/autoui/base-styles-and-visual-parity.md §4.5（VM 侧按该节
   对齐）；同族暗色映射见 §4.6。上游若后续内置 slot 节奏，本规则可退役。 */
.streaming-document .markdown-renderer > .node-slot + .node-slot { margin-top: 0.75rem; }
/* 会话删除按钮：默认隐藏,悬停会话项时显现（悬停显隐无法工具类化） */
.session-delete-btn { display: none; }
.session-item:hover .session-delete-btn { display: flex; align-items: center; }
.session-delete-btn:hover { opacity: 1; color: hsl(var(--destructive)); }

/* PLAN-059 T9:内联删除确认行兜底与 .session-delete-strip 抑制规则已退役
   （536 绑定根修合回,alert-dialog 单源双轨成立）。 */
/* PLAN-050: search/tree 系钩子类在 gen 轨零元素（仅匹配已冻结 web/ 轨）
   ——死规则删除;placeholder 色已内联 wiki_nav.at/chats_view.at。 */
/* 输入区 focus 光环 + @mention 双层文字技术（textarea 文字透明,由
   backdrop 层显示高亮文本;VM 侧 textarea 直接显字,无此技术） */
/* PLAN-050: 输入区双层技术已全量内联 mention_input.at（text-transparent/
   caret/focus-within 光环）——VM 不解析这些类,恰好保持"VM 直接显字"的
   平台非对称;此处原规则块删除（迁移矩阵 docs/designs/010 A1-A3b）。 */
/* PLAN-050: send-btn 钩子在 gen 轨零元素,死规则删除。 */
/* ══ PLAN-054 B1：@autodown/vue 深色主题覆盖（最小集）══
   vendor style.css 全硬编码浅色 design token（正文/标题 #111827、
   表格/代码块 #e5e7eb/#f8f9fa、blockquote、details、admonition）,
   .dark 下深底浅字不可读。统一改挂 musk 主题变量（.dark 域内自动
   取暗值）;admonition 保色相降明度。vendor scoped data-attr 与本块
   特异性打平,注入顺序（head 末尾 <style>）取胜。 */
.dark .streaming-document .markdown-renderer,
.dark .streaming-document h1,
.dark .streaming-document h2,
.dark .streaming-document h3,
.dark .streaming-document td,
.dark .streaming-document th,
.dark .streaming-document pre code,
.dark .streaming-document .mermaid-source-code,
.dark .streaming-document details summary { color: hsl(var(--foreground)); }
.dark .streaming-document table th,
.dark .streaming-document table td,
.dark .streaming-document th,
.dark .streaming-document td,
.dark .streaming-document hr { border-color: hsl(var(--border)); }
.dark .streaming-document table th,
.dark .streaming-document th { background: hsl(var(--muted) / 0.5); }
.dark .streaming-document tr:nth-child(2n),
.dark .streaming-document table tr:nth-child(2n) { background: hsl(var(--muted) / 0.35); }
.dark .streaming-document code {
  background: hsl(var(--muted) / 0.6);
  color: hsl(var(--foreground));
}
.dark .streaming-document pre[data-language],
.dark .streaming-document pre:not([data-language]),
.dark .streaming-document .mermaid-source-panel { background: hsl(var(--card)); border-color: hsl(var(--border)); }
.dark .streaming-document .codeblock-language-badge,
.dark .streaming-document .mermaid-block-header { background: hsl(var(--muted) / 0.5); border-color: hsl(var(--border)); color: hsl(var(--muted-foreground)); }
.dark .streaming-document pre[data-language] .codeblock-language-badge { background: hsl(var(--muted) / 0.5); border-color: hsl(var(--border)); }
.dark .streaming-document .code-block-container { background: hsl(var(--card)) !important; border-color: hsl(var(--border)) !important; }
.dark .streaming-document .code-block-header { background: hsl(var(--muted) / 0.6); border-color: hsl(var(--border)); color: hsl(var(--foreground)); }
.dark .streaming-document blockquote { border-left-color: hsl(var(--border)); color: hsl(var(--muted-foreground)); }
.dark .streaming-document details { border-color: hsl(var(--border)); background: hsl(var(--card)); }
.dark .streaming-document details summary { background: hsl(var(--muted) / 0.5); }
.dark .streaming-document details summary:hover { background: hsl(var(--muted) / 0.8); }
.dark .streaming-document details[open] summary { border-bottom-color: hsl(var(--border)); }
.dark .streaming-document details summary:before { color: hsl(var(--muted-foreground)); }
.dark .streaming-document details .details-content { background: hsl(var(--muted) / 0.25); }
/* admonition：保色相、暗底亮字（vendor !important 需同量级对冲） */
.dark .streaming-document .admonition-legend { background: hsl(var(--card)) !important; }
.dark .streaming-document .admonition-content { color: hsl(var(--foreground)) !important; }
.dark .streaming-document .admonition-note { background: hsl(222 60% 18%) !important; border-color: hsl(222 60% 30%) !important; }
.dark .streaming-document .admonition-note .admonition-legend { color: #93c5fd !important; }
.dark .streaming-document .admonition-info { background: hsl(199 60% 16%) !important; border-color: hsl(199 60% 28%) !important; }
.dark .streaming-document .admonition-info .admonition-legend { color: #7dd3fc !important; }
.dark .streaming-document .admonition-tip { background: hsl(142 55% 14%) !important; border-color: hsl(142 55% 26%) !important; }
.dark .streaming-document .admonition-tip .admonition-legend { color: #86efac !important; }
.dark .streaming-document .admonition-warning,
.dark .streaming-document .admonition-caution { background: hsl(38 65% 15%) !important; border-color: hsl(38 65% 28%) !important; }
.dark .streaming-document .admonition-warning .admonition-legend,
.dark .streaming-document .admonition-caution .admonition-legend { color: #fcd34d !important; }
.dark .streaming-document .admonition-danger,
.dark .streaming-document .admonition-error { background: hsl(0 55% 16%) !important; border-color: hsl(0 55% 28%) !important; }
.dark .streaming-document .admonition-danger .admonition-legend,
.dark .streaming-document .admonition-error .admonition-legend { color: #fca5a5 !important; }
.dark .streaming-document .mermaid-block-container,
.dark .streaming-document .mermaid-preview-area { background: hsl(var(--card)); border-color: hsl(var(--border)); }
.dark .streaming-document .mermaid-block-container>div:not(.mermaid-block-header) .absolute.top-2.right-2>.flex { background: hsl(var(--card) / 0.9); }
.dark .streaming-document .mermaid-mode-btn,
.dark .streaming-document .mermaid-action-btn { color: hsl(var(--muted-foreground)); }
.dark .streaming-document .mermaid-mode-btn.is-active,
.dark .streaming-document .mermaid-mode-btn:hover,
.dark .streaming-document .mermaid-action-btn:hover { background: hsl(var(--muted) / 0.6); color: hsl(var(--foreground)); }

/* ── 主导航 hover（PLAN-061 T14, D20）──
   NavItem active 项 class-token 契约不含 hover（Plan 482 build-time
   either/or,ITEM_ACTIVE 无 hover:*,VM builder 同源）——active 项悬停无
   高亮。web 侧兜底:nav-item token 类所有导航项都带,统一中性灰 hover
   （active/非 active 一致）。上游契约放开后撤本条（C 组 D23）。 */
.nav-item:hover { background-color: hsl(var(--accent)); }
/* ═══════════════════════════════════════════════════ */
`

export function injectStyles(): void {
  if (document.getElementById('musk-global-styles')) return
  const style = document.createElement('style')
  style.id = 'musk-global-styles'
  style.textContent = STYLES
  document.head.appendChild(style)
}
