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
import '@autodown/vue/style.css'

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
/* @mention 高亮（UserMessage v-html 内部,后代选择器） */
.user-text .inline-mention {
  background: hsl(220 90% 56% / 0.12); color: hsl(220 90% 56%);
  border-radius: 3px; padding: 0 0.2rem; font-weight: 500;
}
.msg-bubble-user .user-text { color: hsl(var(--primary-foreground)); white-space: pre-wrap; }
.msg-bubble-user .user-text .inline-mention { color: hsl(var(--primary-foreground)); font-weight: 600; background: hsl(0 0% 100% / 0.15); }
.msg-bubble-ai .streaming-document { color: hsl(var(--foreground)); }
/* 会话删除按钮：默认隐藏,悬停会话项时显现（悬停显隐无法工具类化） */
.session-delete-btn { display: none; }
.session-item:hover .session-delete-btn { display: flex; align-items: center; }
.session-delete-btn:hover { opacity: 1; color: hsl(var(--destructive)); }
/* 搜索框 focus 光环 + placeholder 色 */
.header-search:focus-within {
  border-color: hsl(var(--primary) / 0.35);
}
.search-input::placeholder { color: hsl(var(--muted-foreground)); }
.wiki-search-input::placeholder { color: hsl(var(--muted-foreground)); }
/* wiki 树删除钮：悬停行显现 + 悬停钮危险色（悬停显隐无法工具类化） */
.tree-item-del { display: none; }
.tree-item:hover .tree-item-del { display: flex; }
.tree-item-del:hover { background: hsl(var(--destructive) / 0.12); color: hsl(var(--destructive)); }
/* 输入区 focus 光环 + @mention 双层文字技术（textarea 文字透明,由
   backdrop 层显示高亮文本;VM 侧 textarea 直接显字,无此技术） */
.input-compose:focus-within {
  border-color: hsl(var(--primary) / 0.45) !important;
  box-shadow: 0 0 0 3px hsl(var(--primary) / 0.08) !important;
}
.input-compose textarea,
.chats-input {
  display: block !important;
  width: 100% !important;
  border: none !important; border-radius: 0 !important; background: transparent !important;
  resize: none; outline: none;
  color: transparent !important;
  caret-color: hsl(var(--foreground));
  position: relative;
  z-index: 1;
}
.chats-input:focus { outline: none !important; box-shadow: none !important; border: none !important; }
/* send-btn 悬停/禁用反馈（透明度渐变,工具类已给 hover:/disabled: 变体,
   此处仅补 :active 缩放微交互） */
.send-btn:active { transform: scale(0.95); }
`

export function injectStyles(): void {
  if (document.getElementById('musk-global-styles')) return
  const style = document.createElement('style')
  style.id = 'musk-global-styles'
  style.textContent = STYLES
  document.head.appendChild(style)
}
