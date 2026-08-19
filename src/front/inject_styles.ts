// inject_styles.ts — 全局布局样式注入（Plan 022 C 类 parity）
//
// AutoUI 生成的组件 <style> 为空，自定义语义 class（chats-view/session-list/
// chats-canvas/msg-* 等）无对应 CSS。原生 web/ 把这些放在各组件 <style scoped>。
// 这里集中注入全局样式，对齐原生视觉。
//
// 逃生舱说明：AutoUI .at 无法表达 scoped CSS，用 use { fn } 在 App.Init 注入。

// markstream-vue 渲染器样式（表格边框/表头背景/行内代码配色等 design token）。
// 原生 web/ 在 main.ts 引入；gen 工程的 main.ts 由 codegen 生成不可改，
// 故在此引入（模块加载即注入，效果等价）。
import 'markstream-vue/index.css'

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
/* ── 三列渐变背景层次（对齐原版视觉层次）──
   第一列（view-rail App 侧边栏）：最深的浅灰（secondary）
   第二列（各视图 sub-nav）：中间层（card 偏灰）
   第三列（main panel）：白底（background）
   深色模式对应三档深色。 */
/* 第一列：App.vue 侧边栏（bg-card Tailwind class → 覆盖） */
.flex.flex-row.h-screen > div:first-child {
  background: hsl(var(--secondary)) !important;
}
/* 第二列：各视图的 sub-nav（session-sidebar / section-nav / wiki-nav） */
.chats-view > div:first-child,
.plans-view > div:first-child,
.specs-view > div:first-child,
.wiki-view > div:first-child {
  background: hsl(var(--card)) !important;
}
/* 第三列：main panel 保持 background 默认白/深 */
/* 深色模式三列对应 */
.dark .flex.flex-row.h-screen > div:first-child {
  background: hsl(220 15% 8%) !important;
}
.dark .chats-view > div:first-child,
.dark .plans-view > div:first-child,
.dark .specs-view > div:first-child,
.dark .wiki-view > div:first-child {
  background: hsl(220 15% 10%) !important;
}
/* ── 共用 NavSidebar + ContentHeader（Plan 023 §3.1 单一真源）──
   替代三视图分散的 sidebar-header/section-nav-header/wiki-nav-header/chats-header/
   section-header/wiki-content-header 规则 + !important 覆盖。 */
.app-header {
  height: 48px !important; padding: 0 1rem !important;
  /* 负 margin 抵消父容器 px-3(12px) 左右 padding，让 border 撑满宽度 */
  margin-left: -0.75rem; margin-right: -0.75rem;
  border-bottom: 1px solid hsl(var(--border));
  display: flex; align-items: center; flex-shrink: 0;
}
/* NavSidebar：二级导航外壳（header 48px 贴顶全宽 border + list slot） */
.nav-sidebar {
  display: flex; flex-direction: column; height: 100%; flex-shrink: 0;
  overflow: hidden;
}
.nav-sidebar.collapsed { width: 48px; }
.nav-sidebar-header {
  display: flex; align-items: center; gap: 0.4rem;
  padding: 0.5rem 0.75rem; flex-shrink: 0; height: 48px;
  border-bottom: 1px solid hsl(var(--border));
}
.nav-sidebar-title {
  flex: 1; font-family: 'Noto Sans SC', sans-serif;
  font-size: 1rem; font-weight: 700; color: hsl(var(--foreground));
}
/* ContentHeader：内容标题栏（贴顶全宽 48px border，title + middle + actions） */
.content-header {
  display: flex; align-items: center; justify-content: space-between;
  height: 48px; flex-shrink: 0; padding: 0 1.25rem;
  border-bottom: 1px solid hsl(var(--border)); background: hsl(var(--card));
}
.content-header-title {
  font-size: 1.25rem; font-weight: 700; color: hsl(var(--foreground));
  flex-shrink: 0;
}
.content-header-middle { flex: 1; min-width: 0; display: flex; justify-content: center; }
.content-header-actions { display: flex; align-items: center; gap: 0.3rem; flex-shrink: 0; }
/* ── 导航栏 ── */
.rail-tab {
  width: 100%; text-align: left; padding: 0.5rem 0.75rem; border-radius: 0.375rem;
  font-size: 0.875rem; color: hsl(var(--muted-foreground)); background: transparent;
  border: none; cursor: pointer; transition: all 0.15s;
  display: flex; align-items: center; justify-content: flex-start; gap: 0.5rem;
}
.rail-tab:hover { background: hsl(var(--accent)); color: hsl(var(--accent-foreground)); }
.rail-tab.active {
  background: hsl(var(--primary) / 0.08); color: hsl(var(--primary)); font-weight: 500;
}
/* rail-footer：导航栏底部（WorkspaceSelector + SettingsMenu 并排） */
.rail-footer {
  margin-top: auto;
  display: flex !important;
  align-items: center;
  justify-content: space-between;
  gap: 0.4rem;
  padding: 0 0.3rem;
}
/* WorkspaceSelector 填满剩余空间，SettingsMenu 固定宽度 */
.rail-footer .workspace-selector { flex: 1; min-width: 0; }
/* ── ChatsView 布局 ── */
.chats-view { display: flex; flex-direction: row; height: 100%; overflow: hidden; }
.chats-view > div:first-child {
  width: 220px; flex-shrink: 0;
  border-right: 1px solid hsl(var(--border)); background: hsl(var(--card));
}
.sidebar-new-btn {
  display: inline-flex; align-items: center; justify-content: center;
  height: 26px; padding: 0 0.5rem; font-size: 0.75rem;
  border: 1px solid hsl(var(--border)); border-radius: 6px;
  background: transparent; color: hsl(var(--foreground)); cursor: pointer;
}
.sidebar-new-btn:hover { background: hsl(var(--accent)); }
.session-list { flex: 1; overflow-y: auto; padding: 0 0.5rem; }
.session-item {
  display: block; width: 100%; text-align: left;
  padding: 0.5rem 0.6rem; margin-bottom: 2px; border: none; border-radius: 6px;
  background: transparent; cursor: pointer;
}
.session-item:hover { background: hsl(var(--accent)); }
.session-item.active {
  background: hsl(var(--primary) / 0.08);
  color: hsl(var(--primary));
}
.session-item.active .session-name { font-weight: 500; color: hsl(var(--primary)); }
.session-item { position: relative; }
.session-delete-btn {
  position: absolute; right: 0.4rem; top: 50%; transform: translateY(-50%);
  display: none; cursor: pointer; font-size: 0.85rem; opacity: 0.6;
  background: transparent; border: none; padding: 0.2rem;
}
.session-item:hover .session-delete-btn { display: flex; align-items: center; }
.session-delete-btn:hover { opacity: 1; color: hsl(var(--destructive)); }
.session-preview { display: flex; flex-direction: column; gap: 2px; }
.session-name { font-size: 0.85rem; color: hsl(var(--foreground)); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.session-count { font-size: 0.72rem; color: hsl(var(--muted-foreground)); }
.chats-body { flex: 1; display: flex; flex-direction: column; min-width: 0; overflow: hidden; }
/* ── PlansView 布局（PLAN-026 续：对齐 ChatsView，补此前缺失的 CSS）── */
.plans-root { display: flex; flex-direction: column; height: 100%; overflow: hidden; }
.plans-view { display: flex; flex-direction: row; height: 100%; overflow: hidden; }
.plans-view > div:first-child {
  width: 220px; flex-shrink: 0;
  border-right: 1px solid hsl(var(--border)); background: hsl(var(--card));
}
.plans-main { flex: 1; display: flex; flex-direction: column; min-width: 0; overflow: hidden; }
/* header 三段布局：标题左 / 搜索中 / 操作右 */
.header-actions { display: flex; align-items: center; gap: 0.3rem; flex-shrink: 0; }
.session-info-btn {
  display: flex; align-items: center; justify-content: center;
  width: 28px; height: 28px; border: none; border-radius: 6px;
  background: transparent; color: hsl(var(--muted-foreground)); cursor: pointer;
}
.session-info-btn:hover { background: hsl(var(--accent)); color: hsl(var(--foreground)); }
/* 搜索框（对齐原版 .header-search） */
.header-search {
  display: flex; align-items: center; gap: 0.35rem;
  max-width: 320px; flex: 0 1 320px;
  padding: 0.35rem 0.75rem;
  background: hsl(var(--muted-foreground) / 0.06);
  border: 1px solid hsl(var(--muted-foreground) / 0.12);
  border-radius: 6px;
  color: hsl(var(--muted-foreground));
}
.header-search:focus-within {
  border-color: hsl(var(--primary) / 0.35);
}
.header-search svg { flex-shrink: 0; }
.search-input {
  border: none !important; background: transparent !important; outline: none !important;
  font-size: 0.82rem; color: hsl(var(--foreground)); width: 100%;
}
.search-input::placeholder { color: hsl(var(--muted-foreground)); }
.chats-canvas { flex: 1; overflow-y: auto; padding: 1rem; display: flex; flex-direction: column; gap: 1.4rem; }
/* 消息样式 — 标准 chat UI（header + 气泡 + 工具栏）
   对齐原生 ChatsView.vue：每条消息含 header（role + 时间）+ 气泡内容。
   user 气泡右对齐 + primary 底色，AI 气泡左对齐 + card 底色。 */

/* 消息行（for 循环容器） */
.msg-row { display: flex; flex-direction: column; gap: 0.2rem; margin-bottom: 0.6rem; }
/* header */
.msg-header {
  display: flex; align-items: center; gap: 0.5rem; padding: 0 0.25rem;
}
/* user 的 header 右对齐 */
.chats-canvas > div:has(.msg-bubble-user) .msg-header { justify-content: flex-end; }
.chats-canvas > div:has(.msg-bubble-user) .msg-role-badge { color: hsl(var(--primary)); }
/* AI 的 header 左对齐 */
.chats-canvas > div:has(.msg-bubble-ai) .msg-role-badge { color: hsl(var(--muted-foreground)); }
/* 气泡 */
.msg-bubble {
  padding: 0.6rem 0.9rem; border-radius: 12px; font-size: 0.92rem; line-height: 1.6; word-break: break-word;
}
/* .user-text 基础排版 + @mention 高亮（Plan 023 P3：原逃生舱 UserMessage.vue
   scoped 样式的全局兜底——component fn 不支持 scoped CSS）。 */
.user-text { color: inherit; line-height: 1.5; word-break: break-word; }
/* 旧版 :has(.user-text)/:has(.streaming-document) 兜底（无气泡包装时） */
.chats-canvas > div:has(.user-text):not(:has(.msg-bubble-user)) { align-self: flex-end; max-width: 85%; }
.chats-canvas > div:has(.user-text):not(:has(.msg-bubble-user)) .user-text { color: hsl(var(--foreground)); }
.chats-canvas > div:has(.streaming-document):not(:has(.msg-bubble-ai)) { align-self: flex-start; max-width: 100%; }
.chats-canvas > div:has(.streaming-document):not(:has(.msg-bubble-ai)) .streaming-document { color: hsl(var(--foreground)); }
/* 流式 draft + thinking + error */
.msg.assistant-msg.draft { align-self: flex-start; max-width: 100%; color: hsl(var(--foreground)); }
/* ── ErrandCard（Plan 023 P3：原逃生舱 scoped 样式全局兜底）── */
.errand-card { border: 1px solid hsl(var(--border)); border-radius: 8px; margin: 0; overflow: hidden; background: hsl(38 92% 50% / 0.03); width: 100%; }
/* ── TaskPlanCard（Plan 023 P3：原逃生舱 scoped 样式全局兜底）── */
.task-plan-card { border: 1px solid hsl(var(--border)); border-radius: 8px; margin: 0; overflow: hidden; background: hsl(280 60% 96% / 0.4); width: 100%; }
/* ── GenericToolCard（Plan 023 P3）──
   .tool-card 框架 + .seg-* 颜色已迁至 generic_tool_card.at 的组件 style 块
   （scoped 打得到自己的 DOM，且与 .tool-* 其余规则同源）。此处不再保留。 */
/* ── StreamingTable（Plan 023 P3）──
   .streaming-table 外距已迁至 streaming_table.at 组件 style 块（0.35rem，
   统一块间距节奏）；此处仅保留 scoped 块外的 keyframes。 */
@keyframes st-dots { 0%, 80%, 100% { content: ''; } 40% { content: '.'; } 60% { content: '..'; } }
/* ── AgentAvatar（Plan 023 P3：原逃生舱 scoped 样式全局兜底）── */
.agent-avatar {
  display: inline-flex; align-items: center; justify-content: center;
  flex-shrink: 0; border-radius: 50%; font-weight: 600; line-height: 1;
  user-select: none; font-family: system-ui, -apple-system, sans-serif; overflow: hidden;
}
/* ── ReportCard（Plan 023 P3：原逃生舱 scoped 样式全局兜底）── */
.report-card { border: 1px solid hsl(142 70% 45% / 0.25); border-radius: 10px; background: hsl(142 70% 45% / 0.04); margin: 0.5rem 0; overflow: hidden; transition: all 0.2s; }
.report-header { display: flex; align-items: center; gap: 0.5rem; padding: 0.6rem 0.8rem; cursor: pointer; user-select: none; }
.report-header:hover { background: hsl(142 70% 45% / 0.06); }
.report-status { font-size: 1rem; flex-shrink: 0; }
.report-title-prefix, .report-title { flex: 1; font-size: 0.93rem; font-weight: 500; color: hsl(var(--foreground)); }
.report-confidence { font-size: 0.73rem; padding: 0.15rem 0.4rem; border-radius: 4px; font-weight: 500; text-transform: uppercase; }
.report-confidence.high { background: hsl(142 70% 45% / 0.15); color: hsl(142 70% 35%); }
.report-confidence.medium { background: hsl(38 90% 50% / 0.15); color: hsl(38 80% 40%); }
.report-confidence.low { background: hsl(0 70% 50% / 0.15); color: hsl(0 70% 45%); }
.report-chevron { color: hsl(var(--muted-foreground)); flex-shrink: 0; }
.report-body { padding: 0.5rem 0.8rem 0.75rem; border-top: 1px solid hsl(142 70% 45% / 0.15); display: flex; flex-direction: column; gap: 0.6rem; }
.report-metrics { display: grid; grid-template-columns: 1fr 1fr; gap: 0.4rem; }
.metric-row { display: flex; justify-content: space-between; align-items: center; padding: 0.3rem 0.4rem; background: hsl(220 14% 50% / 0.04); border-radius: 5px; font-size: 0.83rem; }
.metric-label { color: hsl(var(--muted-foreground)); }
.metric-value { font-weight: 500; color: hsl(var(--foreground)); }
.metric-value.drift { color: hsl(var(--destructive)); }
.report-deliverables .section-title, .section-title { font-size: 0.78rem; font-weight: 600; text-transform: uppercase; color: hsl(var(--muted-foreground)); letter-spacing: 0.03em; margin-bottom: 0.2rem; }
.report-deliverables { font-size: 0.88rem; color: hsl(var(--foreground)); line-height: 1.5; }
.deliverable-item { margin: 0.1rem 0; }
.report-actions { display: flex; flex-wrap: wrap; gap: 0.35rem; }
.report-btn { display: inline-flex; align-items: center; gap: 0.3rem; padding: 0.35rem 0.6rem; border: 1px solid hsl(var(--border)); border-radius: 5px; background: transparent; color: hsl(var(--foreground)); font-size: 0.83rem; font-weight: 500; cursor: pointer; transition: all 0.15s; }
.report-btn:hover { background: hsl(220 14% 50% / 0.06); border-color: hsl(var(--primary) / 0.3); }
/* ── 输入区（对齐原生版 .input-compose + .send-btn）──
   DOM: .chats-input-bar > .input-inner > .input-row(flex) > [.input-compose(block,fixed px), button.send-btn]
   问题：.input-compose 被设成 block+固定px宽度，textarea 被设成 inline-block+固定px宽度。
   修复：让 .input-compose 在 flex 行内 flex:1 自适应；textarea width:100% block。 */
/* .chats-input-bar：原生版透明无 border-top（融入背景），Auto 版 scoped 有白底+border-top */
.chats-input-bar {
  background: transparent !important;
  border-top: none !important;
}
/* .input-inner：原生版居中限宽 */
.input-inner {
  max-width: 960px;
  margin: 0 auto;
  width: 100%;
}
.input-row { display: flex !important; align-items: flex-end; gap: 0.4rem; width: 100%; }
.input-compose {
  position: relative !important;      /* 让 .input-backdrop absolute 相对它（高亮层重叠 textarea）*/
  flex: 1 1 auto !important;          /* 在 .input-row 内填满剩余空间 */
  min-width: 0 !important;            /* 允许收缩（flex 子项防溢出） */
  width: auto !important;             /* 覆盖 codegen 注入的固定 px */
  display: flex !important;           /* 让内部 textarea 自适应 */
  align-items: center;
  background: hsl(var(--muted-foreground) / 0.04) !important;
  border: 1px solid hsl(var(--primary) / 0.15) !important;
  border-radius: 20px !important;
  padding: 4px 8px !important;
  min-height: 80px !important;   /* 原版最小高度 */
}
/* focus 时边框+光环（对齐原版 :focus-within） */
.input-compose:focus-within {
  border-color: hsl(var(--primary) / 0.45) !important;
  box-shadow: 0 0 0 3px hsl(var(--primary) / 0.08) !important;
}
/* .input-backdrop（@mention v-html 高亮层）：原生版透明背景+无边框，
   纯粹只是 mention 高亮覆盖层。Auto 版 scoped style 给了灰底+灰边+6px圆角，
   叠加在外层紫色圆角 .input-compose 上造成"双层"视觉混乱——清零。 */
.input-backdrop {
  position: absolute !important;
  top: 0; left: 0; right: 0; bottom: 0;
  padding: 4px 8px !important;        /* 对齐 textarea（与 input-compose padding 一致）*/
  background: transparent !important;
  border: none !important;
  border-radius: 0 !important;
  pointer-events: none;
  overflow: hidden;
  white-space: pre-wrap;
  word-break: break-word;
  color: hsl(var(--foreground));
}
/* textarea 自身去边框（靠容器的边框+圆角），block+100% 宽自适应 */
.input-compose textarea,
.chats-input {
  display: block !important;
  width: 100% !important;
  border: none !important; border-radius: 0 !important; background: transparent !important;
  font-size: 0.95rem; resize: none; outline: none;
  color: transparent !important;       /* 文字透明：只 caret 可见，文字由 backdrop 高亮层显示 */
  caret-color: hsl(var(--foreground));
  position: relative;
  z-index: 1;
}
.chats-input:focus { outline: none !important; box-shadow: none !important; border: none !important; }
/* Send 按钮：圆形 + 紫色底 + 白字（对齐原生版 .send-btn） */
.input-row button[class*="send"],
.input-compose button[type="submit"],
.input-compose button:last-child,
.send-btn {
  width: 36px !important; height: 36px !important;
  min-width: 36px !important;
  border-radius: 50% !important;
  background: linear-gradient(135deg, var(--vp-c-brand-1, hsl(var(--primary))), var(--vp-c-brand-2, hsl(var(--primary) / 0.85))) !important;
  color: #fff !important;
  border: none; cursor: pointer; display: flex; align-items: center; justify-content: center;
  font-size: 1.1rem; flex-shrink: 0; transition: opacity 0.15s, transform 0.1s;
}
.send-btn:active { transform: scale(0.95); }
.input-row button[class*="send"]:hover,
.input-compose button:last-child:hover,
.send-btn:hover { opacity: 0.85; }
.input-row button[class*="send"]:disabled,
.input-compose button:last-child:disabled,
.send-btn:disabled { opacity: 0.4; cursor: not-allowed; }
/* ── SpecsView 布局 ── */
.specs-view { display: flex; flex-direction: row; height: 100%; overflow: hidden; }
.specs-view > div:first-child {
  width: 200px; flex-shrink: 0;
  /* 布局（flex column + header + list）由共用 NavSidebar 承担 */
  border-right: 1px solid hsl(var(--border)); background: hsl(var(--card));
}
.section-nav-list {
  flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 0.25rem;
  padding: 0 0.25rem;
}
.overview-entry {
  display: block; width: 100%; text-align: left;
  padding: 0.4rem 0.75rem; border: none; border-radius: 6px; font-size: 0.85rem;
  background: transparent; color: hsl(var(--muted-foreground)); cursor: pointer;
}
.overview-entry:hover { background: hsl(var(--accent)); }
.overview-entry.active { background: hsl(var(--accent)); color: hsl(var(--foreground)); font-weight: 500; }
.section-nav-item {
  display: block; width: 100%; text-align: left;
  padding: 0.4rem 0.75rem; border: none; border-radius: 6px; font-size: 0.85rem;
  background: transparent; color: hsl(var(--foreground)); cursor: pointer;
}
.section-nav-item:hover { background: hsl(var(--accent)); }
.section-nav-item.active { background: hsl(var(--accent)); font-weight: 500; }
.specs-main { flex: 1; overflow-y: auto; display: flex; flex-direction: column; }
.overview-content { padding: 1.25rem; font-size: 0.9rem; line-height: 1.6; color: hsl(var(--foreground)); }
.section-content { margin-bottom: 1rem; padding: 0 1.25rem 1.25rem; }
.spec-item-btn { display: block; width: 100%; text-align: left; padding: 0.6rem 0.75rem; border: 1px solid hsl(var(--border)); border-radius: 8px; margin-bottom: 0.4rem; background: hsl(var(--card)); cursor: pointer; }
.spec-item-btn:hover { border-color: hsl(var(--primary)); }
.spec-item-main { display: flex; align-items: center; gap: 0.5rem; }
.spec-item-title { font-weight: 500; font-size: 0.88rem; flex: 1; color: hsl(var(--foreground)); }
.spec-item-status { font-size: 0.72rem; padding: 2px 8px; border-radius: 4px; background: hsl(var(--muted)); color: hsl(var(--muted-foreground)); }
.spec-item-actions { display: flex; gap: 0.25rem; }
/* 编辑面板 */
.edit-panel { padding: 1rem; border: 1px solid hsl(var(--border)); border-radius: 8px; background: hsl(var(--card)); margin-bottom: 1rem; }
.form-group { display: flex; flex-direction: column; gap: 0.3rem; margin-bottom: 0.75rem; }
.form-group label { font-size: 0.82rem; font-weight: 500; color: hsl(var(--muted-foreground)); }
.form-input { padding: 0.5rem 0.65rem; border: 1px solid hsl(var(--border)); border-radius: 6px; background: hsl(var(--background)); color: hsl(var(--foreground)); font-size: 0.88rem; }
.form-input:focus { outline: none; border-color: hsl(var(--primary)); }
.content-input { min-height: 120px; resize: vertical; font-family: monospace; }
.edit-actions { display: flex; gap: 0.5rem; justify-content: flex-end; }
.add-btn, .save-btn, .cancel-btn, .action-btn {
  padding: 0.4rem 0.85rem; border: 1px solid hsl(var(--border)); border-radius: 6px; font-size: 0.82rem; cursor: pointer; background: hsl(var(--card)); color: hsl(var(--foreground));
}
.add-btn:hover, .save-btn:hover { background: hsl(var(--primary)); color: hsl(var(--primary-foreground)); border-color: hsl(var(--primary)); }
.cancel-btn:hover, .action-btn:hover { background: hsl(var(--accent)); }
.action-btn.danger { color: hsl(var(--destructive)); }
.action-btn.danger:hover { background: hsl(var(--destructive) / 0.1); }
/* ── WikiView 布局 ── */
.wiki-view { display: flex; flex-direction: row; height: 100%; overflow: hidden; }
.wiki-view > div:first-child {
  width: 240px; flex-shrink: 0;
  /* 布局（flex column + header + list）由共用 NavSidebar 承担 */
  border-right: 1px solid hsl(var(--border)); background: hsl(var(--card));
}
.nav-icon-btn { width: 28px; height: 28px; display: inline-flex; align-items: center; justify-content: center; border: 1px solid hsl(var(--border)); border-radius: 6px; background: transparent; cursor: pointer; color: hsl(var(--foreground)); font-size: 1rem; }
.nav-icon-btn:hover { background: hsl(var(--accent)); }
.wiki-nav-list { flex: 1; overflow-y: auto; }
/* ── WikiNav 树/搜索/DropZone（Plan 023 队列 B6 原生化：逃生舱 scoped 转全局兜底）── */
.wiki-search {
  display: flex; align-items: center; gap: 0.35rem;
  margin: 0.5rem 0.5rem; padding: 0.3rem 0.6rem;
  background: hsl(var(--muted-foreground) / 0.06);
  border: 1px solid hsl(var(--muted-foreground) / 0.12);
  border-radius: 6px; color: hsl(var(--muted-foreground));
}
.wiki-search:focus-within { border-color: hsl(var(--primary) / 0.35); }
.wiki-search-input { border: none; background: transparent; outline: none; font-size: 0.8rem; color: hsl(var(--foreground)); width: 100%; }
.wiki-search-input::placeholder { color: hsl(var(--muted-foreground)); }
.wiki-nav-list { padding: 0 0.25rem; }
.tree-section { margin-bottom: 0.25rem; }
.tree-section-header {
  display: flex; align-items: center; gap: 0.3rem;
  padding: 0.35rem 0.5rem; cursor: pointer; border-radius: 4px;
  color: hsl(var(--muted-foreground)); font-size: 0.75rem; font-weight: 600;
}
.tree-section-header:hover { background: hsl(var(--accent)); }
.tree-section-title { flex: 1; }
.tree-section-body { padding-left: 0.75rem; }
.tree-item {
  display: flex; align-items: center; gap: 0.35rem;
  width: 100%; padding: 0.25rem 0.5rem; border: none; border-radius: 4px;
  background: transparent; color: hsl(var(--foreground)); font-size: 0.8rem;
  cursor: pointer; text-align: left;
}
.tree-item:hover { background: hsl(var(--accent)); }
.tree-item.active { background: hsl(var(--primary) / 0.08); color: hsl(var(--primary)); }
.tree-item-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; flex: 1; }
.tree-item-del {
  display: none; align-items: center; justify-content: center;
  width: 18px; height: 18px; border: none; border-radius: 4px;
  background: transparent; color: hsl(var(--muted-foreground)); cursor: pointer; padding: 0;
}
.tree-item:hover .tree-item-del { display: flex; }
.tree-item-del:hover { background: hsl(var(--destructive) / 0.12); color: hsl(var(--destructive)); }
.tree-empty { display: flex; align-items: center; gap: 0.4rem; padding: 0.5rem; color: hsl(var(--muted-foreground)); font-size: 0.75rem; }
.drop-zone {
  display: flex; flex-direction: column; align-items: center; justify-content: center;
  gap: 0.3rem; padding: 0.5rem; margin: 0.25rem 0.25rem 0.5rem;
  border: 1px dashed hsl(var(--border)); border-radius: 6px;
  color: hsl(var(--muted-foreground)); transition: all 0.15s; cursor: pointer;
}
.drop-zone.active { border-color: hsl(var(--primary)); background: hsl(var(--primary) / 0.04); color: hsl(var(--primary)); }
.drop-text { font-size: 0.72rem; }
.progress-bar { width: 100%; height: 3px; background: hsl(var(--muted-foreground) / 0.1); border-radius: 2px; overflow: hidden; }
.progress-fill { height: 100%; background: hsl(var(--primary)); transition: width 0.2s; }
/* ── SessionInfo（Plan 023 队列 B 续原生化：逃生舱 scoped 转全局兜底）── */
.session-info-wrapper { position: relative; }
.session-info-btn {
  display: flex; align-items: center; justify-content: center;
  width: 28px; height: 28px; border: none; border-radius: 6px;
  background: transparent; color: hsl(var(--muted-foreground)); cursor: pointer;
}
.session-info-btn:hover { background: hsl(var(--accent)); color: hsl(var(--foreground)); }
.session-info-tooltip {
  position: absolute; top: calc(100% + 0.5rem); right: 0; min-width: 280px;
  background: hsl(var(--background)); border: 1px solid hsl(var(--border));
  border-radius: 0.5rem; padding: 0.75rem; box-shadow: 0 4px 12px rgba(0,0,0,0.15);
  z-index: 100; display: flex; flex-direction: column; gap: 0.5rem;
}
.session-info-row { display: flex; align-items: center; gap: 0.5rem; }
.session-info-label { font-size: 0.75rem; color: hsl(var(--muted-foreground)); min-width: 5rem; }
.session-info-value { font-size: 0.82rem; color: hsl(var(--foreground)); flex: 1; }
.session-info-id { font-family: monospace; font-size: 0.75rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.session-info-copy {
  display: flex; align-items: center; justify-content: center;
  width: 24px; height: 24px; border: none; border-radius: 4px;
  background: transparent; color: hsl(var(--muted-foreground)); cursor: pointer; flex-shrink: 0;
}
.session-info-copy:hover { background: hsl(var(--accent)); color: hsl(var(--foreground)); }
.wiki-nav-item {
  display: block; width: 100%; text-align: left;
  padding: 0.4rem 0.75rem; border: none; border-radius: 6px; font-size: 0.85rem;
  background: transparent; color: hsl(var(--foreground)); cursor: pointer;
}
.wiki-nav-item:hover { background: hsl(var(--accent)); }
.wiki-nav-item.active { background: hsl(var(--accent)); font-weight: 500; }
.wiki-main { flex: 1; overflow-y: auto; display: flex; flex-direction: column; }
.wiki-content-actions { display: flex; gap: 0.25rem; }
.wiki-content { padding: 0 1.25rem 1.25rem; font-size: 0.9rem; line-height: 1.6; color: hsl(var(--foreground)); }
.wiki-raw { padding: 0 1.25rem 1.25rem; }
.wiki-markdown { font-size: 0.9rem; line-height: 1.6; }
.wiki-editor { margin: 1.25rem; padding: 1rem; border: 1px solid hsl(var(--border)); border-radius: 8px; background: hsl(var(--card)); }
.wiki-empty { display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; color: hsl(var(--muted-foreground)); gap: 0.5rem; text-align: center; }
.text-muted { color: hsl(var(--muted-foreground)); }
/* ── RawPreview（Plan 023 队列 A1 原生化：逃生舱 scoped 转全局兜底）── */
.raw-preview { padding: 1rem; }
.raw-preview-img { max-width: 100%; max-height: 60vh; border-radius: 6px; }
.raw-preview-pdf { width: 100%; height: 70vh; border: none; border-radius: 6px; }
.raw-preview-text { font-size: 0.875rem; }
.raw-download {
  display: flex; flex-direction: column; align-items: center; gap: 0.5rem;
  padding: 2rem; color: hsl(var(--muted-foreground));
}
.download-link { color: hsl(var(--primary)); text-decoration: underline; }
/* ── WorkspaceSelector（Plan 023 队列 A2 原生化：逃生舱 scoped 转全局兜底）── */
.workspace-selector { position: relative; margin-top: auto; }
.ws-btn {
  display: flex; align-items: center; gap: 0.35rem; width: 100%;
  padding: 0.35rem 0.5rem; border: 1px solid hsl(var(--border)); border-radius: 6px;
  background: transparent; color: hsl(var(--foreground)); font-size: 0.75rem; cursor: pointer;
  transition: background 0.15s;
}
.ws-btn:hover { background: hsl(var(--accent)); }
.ws-name { flex: 1; text-align: left; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.ws-panel {
  position: absolute; bottom: 100%; left: 0; right: 0; margin-bottom: 4px;
  background: hsl(var(--card)); border: 1px solid hsl(var(--border)); border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1); padding: 0.5rem; z-index: 50;
  max-height: 320px; overflow-y: auto;
}
.ws-panel-header {
  display: flex; align-items: center; justify-content: space-between;
  font-size: 0.75rem; font-weight: 600; color: hsl(var(--muted-foreground)); margin-bottom: 0.4rem;
}
.ws-close { border: none; background: transparent; cursor: pointer; color: hsl(var(--muted-foreground)); padding: 2px; }
.ws-close:hover { color: hsl(var(--foreground)); }
.ws-section-label { font-size: 0.7rem; color: hsl(var(--muted-foreground)); padding: 0.2rem 0.3rem; }
.ws-item {
  display: flex; align-items: center; gap: 0.35rem; width: 100%;
  padding: 0.35rem 0.4rem; border: none; border-radius: 4px; background: transparent;
  cursor: pointer; font-size: 0.75rem; text-align: left;
}
.ws-item:hover { background: hsl(var(--accent)); }
.ws-item.active { background: hsl(var(--primary) / 0.1); color: hsl(var(--primary)); }
.ws-item-name { flex-shrink: 0; font-weight: 500; }
.ws-item-path {
  flex: 1; font-size: 0.68rem; color: hsl(var(--muted-foreground));
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
.ws-empty { padding: 0.5rem; text-align: center; font-size: 0.72rem; color: hsl(var(--muted-foreground)); }
/* browse 选目录（打开其他文件夹）—— 对齐 web WorkspaceSelector scoped 样式 */
.ws-divider { height: 1px; background: hsl(var(--border)); margin: 0.4rem 0; }
.ws-input {
  width: 100%; box-sizing: border-box; background: hsl(var(--background));
  border: 1px solid hsl(var(--border)); border-radius: 4px;
  padding: 0.35rem 0.5rem; color: hsl(var(--foreground)); font-size: 0.8rem;
}
.ws-suggest { max-height: 120px; overflow-y: auto; }
.ws-suggest-item {
  display: block; width: 100%; background: none; border: none;
  padding: 0.3rem 0.5rem; text-align: left; cursor: pointer;
  color: hsl(var(--foreground)); font-size: 0.78rem; border-radius: 4px;
}
.ws-suggest-item:hover { background: hsl(var(--muted-foreground) / 0.08); }
.ws-open-btn {
  display: flex; align-items: center; gap: 0.4rem; width: 100%; justify-content: center;
  margin-top: 0.4rem; background: hsl(var(--primary)); color: hsl(var(--primary-foreground));
  border: none; border-radius: 4px; padding: 0.4rem; cursor: pointer; font-size: 0.82rem;
}
.ws-open-btn:disabled { opacity: 0.5; cursor: not-allowed; }
/* AI 思考中等待指示（streaming 但 draft 还没开始）—— pulse 动画 */
.thinking-dots {
  color: hsl(var(--muted-foreground));
  font-size: 0.85rem;
  padding: 0.5rem 0.75rem;
  animation: thinking-pulse 1.5s ease-in-out infinite;
}
@keyframes thinking-pulse {
  0%, 100% { opacity: 0.35; }
  50% { opacity: 1; }
}
/* ── RelayRunBox（Plan 023 队列 A3 原生化：逃生舱 scoped 转全局兜底）── */
.relay-box {
  border: 1px solid var(--af-border); border-radius: 8px; margin: 0; width: 100%;
  overflow: hidden; background: hsl(var(--muted-foreground) / 0.03);
}
.status-running { border-left: 3px solid hsl(var(--primary)); }
.status-completed { border-left: 3px solid hsl(142 71% 45%); }
.status-failed { border-left: 3px solid hsl(var(--af-error)); }
.status-gate { border-left: 3px solid hsl(38 92% 50%); }
.badge-running { background: hsl(var(--primary) / 0.15); color: hsl(var(--primary)); }
.badge-completed { background: hsl(142 71% 45% / 0.15); color: hsl(142 71% 45%); }
.badge-failed { background: hsl(var(--af-error) / 0.15); color: hsl(var(--af-error)); }
.badge-gate { background: hsl(38 92% 50% / 0.15); color: hsl(38 92% 50%); }
.log-entries { max-height: 400px; overflow-y: auto; font-size: 0.78rem; line-height: 1.5; }
.log-entry { padding: 0.15rem 0; }
.gate-actions {
  display: flex; align-items: center; gap: 0.5rem; padding: 0.5rem 0;
  border-top: 1px solid var(--af-border); margin-top: 0.5rem;
}
.gate-prompt { font-size: 0.78rem; color: hsl(38 92% 50%); flex: 1; }
.gate-btn { padding: 0.25rem 0.8rem; border-radius: 4px; border: 1px solid var(--af-border); cursor: pointer; font-size: 0.78rem; }
.gate-btn.approve { background: hsl(142 71% 45%); color: #fff; border-color: transparent; }
.gate-btn.reject { background: hsl(var(--af-error) / 0.1); color: hsl(var(--af-error)); }
.gate-btn:disabled { opacity: 0.5; cursor: not-allowed; }
/* ── SettingsMenu（Plan 023 队列 A4 原生化：逃生舱 scoped 转全局兜底）── */
.settings-menu-wrapper { position: relative; }
.settings-trigger {
  display: flex; align-items: center; justify-content: center;
  width: 32px; height: 32px; border: none; border-radius: 6px;
  background: transparent; color: hsl(var(--muted-foreground)); cursor: pointer;
  transition: all 0.15s;
}
.settings-trigger:hover { background: hsl(var(--accent)); color: hsl(var(--foreground)); }
.settings-trigger.open { background: hsl(var(--accent)); color: hsl(var(--primary)); }
.settings-panel {
  position: absolute; bottom: 100%; left: 0; margin-bottom: 8px; min-width: 220px;
  background: hsl(var(--card)); border: 1px solid hsl(var(--border)); border-radius: 10px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12); padding: 0.6rem; z-index: 100;
}
.settings-section { padding: 0.4rem 0; }
.settings-section + .settings-section { border-top: 1px solid hsl(var(--border)); }
.settings-section-title {
  font-size: 0.7rem; font-weight: 600; color: hsl(var(--muted-foreground));
  text-transform: uppercase; letter-spacing: 0.04em; margin-bottom: 0.4rem; padding: 0 0.3rem;
}
.mode-toggle { display: flex; gap: 0.35rem; padding: 0 0.3rem; }
.mode-btn {
  flex: 1; padding: 0.3rem 0; border: 1px solid hsl(var(--border)); border-radius: 6px;
  background: transparent; color: hsl(var(--foreground)); font-size: 0.78rem; font-weight: 600;
  cursor: pointer; transition: all 0.15s;
}
.mode-btn:hover { background: hsl(var(--accent)); }
.mode-btn.active { background: hsl(var(--primary)); border-color: hsl(var(--primary)); color: hsl(var(--primary-foreground)); }
.accent-swatches { display: flex; gap: 0.5rem; padding: 0 0.3rem; }
.accent-swatch {
  width: 24px; height: 24px; border-radius: 50%; border: 2px solid transparent;
  cursor: pointer; display: flex; align-items: center; justify-content: center;
  color: #fff; transition: transform 0.1s;
}
.accent-swatch:hover { transform: scale(1.1); }
.accent-swatch.active { border-color: hsl(var(--foreground)); }
.theme-options { display: flex; flex-direction: column; gap: 2px; }
.theme-option {
  display: flex; align-items: center; gap: 0.5rem; padding: 0.35rem 0.5rem;
  border: none; border-radius: 6px; background: transparent; color: hsl(var(--foreground));
  font-size: 0.82rem; cursor: pointer; text-align: left; width: 100%;
}
.theme-option:hover { background: hsl(var(--accent)); }
.theme-option.active { background: hsl(var(--primary) / 0.08); color: hsl(var(--primary)); }
.theme-option .check { margin-left: auto; }
.theme-option-label { flex: 1; }
.language-options { display: flex; flex-direction: column; gap: 2px; }
.language-option {
  display: flex; align-items: center; gap: 0.5rem; padding: 0.35rem 0.5rem;
  border: none; border-radius: 6px; background: transparent; color: hsl(var(--foreground));
  font-size: 0.82rem; cursor: pointer; text-align: left; width: 100%;
}
.language-option:hover { background: hsl(var(--accent)); }
.language-option.active { background: hsl(var(--primary) / 0.08); color: hsl(var(--primary)); }
.lang-code { font-weight: 700; min-width: 1.5rem; }
.lang-name { flex: 1; }
.language-option .check { margin-left: auto; }
.deep-link-btn {
  display: flex; align-items: center; gap: 0.5rem; width: 100%; padding: 0.35rem 0.5rem;
  border: 1px solid hsl(var(--border)); border-radius: 6px; background: transparent;
  color: hsl(var(--foreground)); font-size: 0.78rem; cursor: pointer; text-align: left;
  transition: background 0.15s;
}
.deep-link-btn:hover { background: hsl(var(--accent)); }
.deep-link-label { flex: 1; }
.deep-link-error {
  margin-top: 0.3rem; padding: 0.2rem 0.5rem; font-size: 0.7rem;
  color: hsl(var(--destructive)); background: hsl(var(--destructive) / 0.08); border-radius: 4px;
}
/* ── QuestionnaireCard（Plan 023 队列 B1 原生化：逃生舱 scoped 转全局兜底）── */
.questionnaire-card {
  background: hsl(var(--primary) / 0.04); border: 1px solid hsl(var(--primary) / 0.15);
  border-radius: 10px; padding: 0.75rem 1rem; margin-top: 0.55rem; width: 100%; box-sizing: border-box;
  display: flex; flex-direction: column; gap: 0.75rem;
}
/* ── GateCard（Plan 023 队列 B2 原生化：逃生舱 scoped 转全局兜底）── */
.gate-card {
  display: flex; flex-direction: column; gap: 0.6rem;
  padding: 0.75rem 1.25rem; border-top: 1px solid var(--af-border, hsl(220 13% 91%));
  flex-shrink: 0; background: hsl(220 14% 50% / 0.02);
}
.gate-card-header { display: flex; align-items: center; gap: 0.5rem; font-size: 0.93rem; color: var(--af-fg, hsl(220 14% 10%)); }
.gate-icon { font-size: 1.18rem; }
.gate-title { flex: 1; }
.gate-profession {
  font-size: 0.78rem; padding: 0.15rem 0.4rem; border-radius: 4px;
  background: hsl(220 90% 56% / 0.08); color: hsl(220 90% 56%); font-weight: 500;
}
.gate-diff-list { display: flex; flex-direction: column; gap: 0.35rem; max-height: 300px; overflow-y: auto; }
.diff-card { border: 1px solid var(--af-border, hsl(220 13% 91%)); border-radius: 8px; overflow: hidden; }
.diff-header {
  display: flex; align-items: center; gap: 0.5rem;
  padding: 0.4rem 0.6rem; cursor: pointer; user-select: none;
}
.diff-header:hover { background: hsl(220 14% 50% / 0.03); }
.diff-title { font-size: 0.88rem; font-weight: 500; color: var(--af-fg, hsl(220 14% 10%)); text-transform: capitalize; flex: 1; }
.diff-status { font-size: 0.73rem; font-weight: 500; color: hsl(38 92% 50%); }
.diff-status.approved { color: hsl(142 71% 45%); }
.diff-chevron { color: var(--af-muted, hsl(220 9% 46%)); }
.diff-body {
  display: grid; grid-template-columns: 1fr 1fr; gap: 0.5rem;
  padding: 0.5rem 0.6rem; background: hsl(220 14% 50% / 0.02); border-top: 1px solid var(--af-border, hsl(220 13% 91%));
}
.diff-side { display: flex; flex-direction: column; gap: 0.2rem; }
.diff-label { font-size: 0.73rem; font-weight: 500; text-transform: uppercase; color: var(--af-muted, hsl(220 9% 46%)); letter-spacing: 0.02em; }
.diff-content {
  font-size: 0.83rem; font-family: 'JetBrains Mono', 'Fira Code', monospace;
  background: var(--af-bg, #fff); border: 1px solid var(--af-border, hsl(220 13% 91%));
  border-radius: 4px; padding: 0.35rem; overflow-x: auto;
  white-space: pre-wrap; word-break: break-word;
  color: var(--af-fg, hsl(220 14% 10%)); margin: 0;
}
.diff-content.old { color: var(--af-muted, hsl(220 9% 46%)); }
.diff-editor {
  font-size: 0.83rem; font-family: 'JetBrains Mono', 'Fira Code', monospace;
  background: var(--af-bg, #fff); border: 1px solid var(--af-border, hsl(220 13% 91%));
  border-radius: 4px; padding: 0.35rem; color: var(--af-fg, hsl(220 14% 10%));
  resize: vertical; outline: none; width: 100%; box-sizing: border-box;
}
.diff-editor:focus { border-color: hsl(220 90% 56% / 0.4); }
.gate-actions { display: flex; gap: 0.5rem; }
.approve-btn, .reject-btn, .review-btn {
  display: inline-flex; align-items: center; gap: 0.35rem;
  padding: 0.4rem 0.9rem; border: none; border-radius: 6px;
  font-size: 0.88rem; font-weight: 500; cursor: pointer; transition: opacity 0.15s;
}
.approve-btn { background: hsl(142 71% 45%); color: #fff; }
.reject-btn { background: transparent; color: var(--af-fg, hsl(220 14% 10%)); border: 1px solid var(--af-border, hsl(220 13% 91%)); }
.review-btn { background: hsl(220 14% 50% / 0.08); color: var(--af-fg, hsl(220 14% 10%)); border: 1px solid var(--af-border, hsl(220 13% 91%)); }
.approve-btn:hover, .reject-btn:hover, .review-btn:hover { opacity: 0.85; }
/* ── MentionDropdown（Plan 023 队列 B3 原生化：逃生舱 scoped 转全局兜底）── */
.mention-dropdown {
  min-width: 180px; max-height: 220px; overflow-y: auto;
  background: var(--af-card, #fff); border: 1px solid var(--af-border, hsl(220 13% 91%));
  border-radius: 8px; box-shadow: 0 4px 16px rgba(0, 0, 0, 0.12); padding: 4px; z-index: 200;
}
.mention-item {
  display: flex; align-items: center; gap: 0.5rem; width: 100%;
  padding: 6px 10px; border: none; border-radius: 5px; background: transparent;
  color: var(--af-fg, hsl(220 14% 10%)); font-size: 0.88rem; cursor: pointer; text-align: left;
  transition: background 0.1s;
}
.mention-item:hover, .mention-item.active { background: hsl(var(--primary, 220 90% 56%) / 0.08); }
.mention-item.active { color: var(--af-primary, hsl(220 90% 56%)); }
.mention-name { font-weight: 600; font-family: monospace; }
.mention-label { color: var(--af-muted, hsl(220 9% 46%)); font-size: 0.83rem; }
/* ── SecretaryMessage（Plan 023 队列 B4 原生化：逃生舱 scoped 转全局兜底）── */
.secretary-message {
  display: flex; flex-direction: column; gap: 0.5rem;
  padding: 0.6rem 0.8rem; border-radius: 10px;
  border: 1px solid hsl(var(--primary) / 0.2); background: hsl(var(--primary) / 0.04);
  margin: 0.5rem 0; transition: opacity 0.3s, transform 0.3s;
}
.secretary-message.dismissed { opacity: 0; transform: translateX(20px); pointer-events: none; }
.secretary-header { display: flex; align-items: flex-start; gap: 0.5rem; }
.secretary-info { flex: 1; min-width: 0; }
.secretary-title { font-size: 0.93rem; font-weight: 500; color: var(--af-fg); line-height: 1.3; }
.secretary-meta { display: flex; align-items: center; gap: 0.4rem; margin-top: 0.15rem; }
.secretary-profession { font-size: 0.78rem; padding: 0.1rem 0.35rem; border-radius: 4px; background: hsl(var(--primary) / 0.1); color: var(--af-primary); font-weight: 500; }
.secretary-waiting { font-size: 0.78rem; color: var(--af-muted); }
.secretary-dismiss {
  display: inline-flex; align-items: center; justify-content: center;
  width: 24px; height: 24px; background: transparent; border: none; border-radius: 4px;
  color: var(--af-muted); cursor: pointer; transition: all 0.15s; flex-shrink: 0;
}
.secretary-dismiss:hover { background: hsl(var(--muted-foreground) / 0.08); color: var(--af-fg); }
.secretary-actions { display: flex; flex-wrap: wrap; gap: 0.35rem; }
.secretary-btn {
  display: inline-flex; align-items: center; gap: 0.25rem;
  padding: 0.3rem 0.6rem; border: none; border-radius: 5px;
  font-size: 0.83rem; font-weight: 500; cursor: pointer; transition: opacity 0.15s;
}
.secretary-btn.approve { background: hsl(142 70% 45% / 0.15); color: hsl(142 70% 35%); }
.secretary-btn.reject { background: hsl(0 70% 45% / 0.1); color: hsl(0 70% 45%); }
.secretary-btn.snooze { background: hsl(38 90% 50% / 0.1); color: hsl(38 80% 40%); }
.secretary-btn.review { background: hsl(var(--muted-foreground) / 0.08); color: var(--af-fg); }
.secretary-btn:hover { opacity: 0.85; }
.secretary-queue { font-size: 0.78rem; color: var(--af-muted); padding-top: 0.2rem; }
/* ThinkBlock（有序 block 渲染模型）：默认折叠只显示 token 数；
   展开后内容区滚动 + 最大高度。 */
.think-block {
  width: 100%; box-sizing: border-box;
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
  background: hsl(var(--muted) / 0.4);
  max-width: 100%;
  overflow: hidden;
}
/* ── Markdown 表格补充：斑马线背景（markstream index.css 已含边框/表头底色） ── */
.markstream-vue .table-node tbody tr:nth-child(even) td {
  background: hsl(var(--ms-muted) / 0.55);
}
`

export function injectStyles(): void {
  if (document.getElementById('musk-global-styles')) return
  const style = document.createElement('style')
  style.id = 'musk-global-styles'
  style.textContent = STYLES
  document.head.appendChild(style)
}
