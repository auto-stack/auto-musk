// inject_styles.ts — 全局布局样式注入（Plan 022 C 类 parity）
//
// AutoUI 生成的组件 <style> 为空，自定义语义 class（chats-view/session-list/
// chats-canvas/msg-* 等）无对应 CSS。原生 web/ 把这些放在各组件 <style scoped>。
// 这里集中注入全局样式，对齐原生视觉。
//
// 逃生舱说明：AutoUI .at 无法表达 scoped CSS，用 use { fn } 在 App.Init 注入。

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
.dark .specs-view > div:first-child,
.dark .wiki-view > div:first-child {
  background: hsl(220 15% 10%) !important;
}

/* ── header 统一分隔线 + 高度 + 标题样式（对齐原版）── */
.app-header {
  height: 48px !important; padding: 0 1rem !important;
  /* 负 margin 抵消父容器 px-3(12px) 左右 padding，让 border 撑满宽度 */
  margin-left: -0.75rem; margin-right: -0.75rem;
  border-bottom: 1px solid hsl(var(--border));
  display: flex; align-items: center; flex-shrink: 0;
}
.sidebar-header, .section-nav-header, .wiki-nav-header, .chats-header {
  height: 48px !important;
  border-bottom: 1px solid hsl(var(--border)) !important;
  display: flex !important; align-items: center; flex-shrink: 0;
}
/* 内容页 title 与导航 header 高度统一（原版 content-header 48px + border-bottom） */
/* 对齐原版 .section-editor(无 padding) + .content-header(贴顶全宽自带宽) 架构：
   .specs-main/.wiki-main 去 padding，header 贴顶全宽自带水平 padding，内容容器单独 padding。 */
.section-header, .wiki-content-header {
  height: 48px;
  flex-shrink: 0;
  border-bottom: 1px solid hsl(var(--border));
  display: flex; align-items: center;
  padding: 0 1.25rem;
}
/* 标题统一为 Noto Sans SC bold 风格（覆盖各视图分散定义） */
.sidebar-title, .section-nav-title, .wiki-nav-title, .chats-title {
  font-family: 'Noto Sans SC', sans-serif !important;
  font-size: 1rem !important;
  font-weight: 700 !important;
  color: hsl(var(--foreground)) !important;
  text-transform: none !important;
  letter-spacing: normal !important;
}

/* ── 导航栏 ── */
.rail-tab {
  width: 100%; text-align: left; padding: 0.5rem 0.75rem; border-radius: 0.375rem;
  font-size: 0.875rem; color: hsl(var(--muted-foreground)); background: transparent;
  border: none; cursor: pointer; transition: all 0.15s;
  display: flex; align-items: center; gap: 0.5rem;
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
  width: 220px; flex-shrink: 0; display: flex; flex-direction: column;
  border-right: 1px solid hsl(var(--border)); background: hsl(var(--card));
}
.sidebar-header {
  display: flex; align-items: center; gap: 0.35rem;
  padding: 0.75rem 1rem; flex-shrink: 0; height: 48px;
}
.sidebar-title { flex: 1; font-size: 0.85rem; font-weight: 600; color: hsl(var(--muted-foreground)); line-height: 1; }
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
.chats-header {
  display: flex; align-items: center; justify-content: space-between;
  padding: 0.6rem 1rem; height: 48px; flex-shrink: 0;
  border-bottom: 1px solid hsl(var(--border)); background: hsl(var(--card));
}
.chats-title { font-size: 0.85rem; font-weight: 500; color: hsl(var(--muted-foreground)); text-transform: uppercase; letter-spacing: 0.04em; }
/* header 三段布局：标题左 / 搜索中 / 操作右 */
.header-title-row { display: flex; align-items: center; gap: 0.4rem; flex-shrink: 0; }
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
.chats-canvas { flex: 1; overflow-y: auto; padding: 1rem; display: flex; flex-direction: column; gap: 0.75rem; }

/* 消息样式 — 标准 chat UI（header + 气泡 + 工具栏）
   对齐原生 ChatsView.vue：每条消息含 header（role + 时间）+ 气泡内容。
   user 气泡右对齐 + primary 底色，AI 气泡左对齐 + card 底色。 */

/* 消息行（for 循环容器） */
.msg-row { display: flex; flex-direction: column; gap: 0.2rem; margin-bottom: 0.6rem; }
.msg-row-user { align-self: flex-end; max-width: 85%; }
.msg-row-ai { align-self: flex-start; max-width: 100%; }

/* header */
.msg-header {
  display: flex; align-items: center; gap: 0.5rem; padding: 0 0.25rem;
}
.msg-role-badge {
  font-size: 0.85rem; font-weight: 600;
}
/* user 的 header 右对齐 */
.chats-canvas > div:has(.msg-bubble-user) .msg-header { justify-content: flex-end; }
.chats-canvas > div:has(.msg-bubble-user) .msg-role-badge { color: hsl(var(--primary)); }
/* AI 的 header 左对齐 */
.chats-canvas > div:has(.msg-bubble-ai) .msg-role-badge { color: hsl(var(--muted-foreground)); }
.msg-time {
  font-size: 0.72rem; color: hsl(var(--muted-foreground));
}

/* 气泡 */
.msg-bubble {
  padding: 0.6rem 0.9rem; border-radius: 12px; font-size: 0.92rem; line-height: 1.6; word-break: break-word;
}
.msg-bubble-user {
  background: hsl(var(--primary)); color: hsl(var(--primary-foreground));
  border-bottom-right-radius: 4px;
}
/* .user-text 基础排版 + @mention 高亮（Plan 023 P3：原逃生舱 UserMessage.vue
   scoped 样式的全局兜底——component fn 不支持 scoped CSS）。 */
.user-text { color: inherit; line-height: 1.5; word-break: break-word; }
.user-text .inline-mention {
  background: hsl(220 90% 56% / 0.12); color: hsl(220 90% 56%);
  border-radius: 3px; padding: 0 0.2rem; font-weight: 500;
}
.msg-bubble-user .user-text { color: hsl(var(--primary-foreground)); white-space: pre-wrap; }
.msg-bubble-user .user-text .inline-mention { color: hsl(var(--primary-foreground)); font-weight: 600; background: hsl(0 0% 100% / 0.15); }

.msg-bubble-ai {
  background: hsl(var(--card)); border: 1px solid hsl(var(--border)); color: hsl(var(--foreground));
  border-bottom-left-radius: 4px;
}
.msg-bubble-ai .streaming-document { color: hsl(var(--foreground)); }
.msg-bubble-ai .streaming-document p,
.msg-bubble-ai .streaming-document li,
.msg-bubble-ai .streaming-document h1,
.msg-bubble-ai .streaming-document h2,
.msg-bubble-ai .streaming-document h3,
.msg-bubble-ai .streaming-document h4,
.msg-bubble-ai .streaming-document code,
.msg-bubble-ai .streaming-document pre,
.msg-bubble-ai .streaming-document ul,
.msg-bubble-ai .streaming-document ol,
.msg-bubble-ai .streaming-document blockquote { color: hsl(var(--foreground)); }
.msg-bubble-ai .streaming-document pre { background: hsl(var(--muted)); padding: 0.6rem 0.8rem; border-radius: 6px; overflow-x: auto; font-size: 0.85rem; margin: 0.5rem 0; }
.msg-bubble-ai .streaming-document code { background: hsl(var(--muted)); padding: 2px 5px; border-radius: 4px; font-size: 0.88rem; }
.msg-bubble-ai .streaming-document pre code { background: transparent; padding: 0; }

/* 旧版 :has(.user-text)/:has(.streaming-document) 兜底（无气泡包装时） */
.chats-canvas > div:has(.user-text):not(:has(.msg-bubble-user)) { align-self: flex-end; max-width: 85%; }
.chats-canvas > div:has(.user-text):not(:has(.msg-bubble-user)) .user-text { color: hsl(var(--foreground)); }
.chats-canvas > div:has(.streaming-document):not(:has(.msg-bubble-ai)) { align-self: flex-start; max-width: 100%; }
.chats-canvas > div:has(.streaming-document):not(:has(.msg-bubble-ai)) .streaming-document { color: hsl(var(--foreground)); }

/* 流式 draft + thinking + error */
.msg.assistant-msg.draft { align-self: flex-start; max-width: 100%; color: hsl(var(--foreground)); }
.msg-thinking { font-size: 0.82rem; color: hsl(var(--muted-foreground)); font-style: italic; padding: 0.25rem 0.5rem; align-self: flex-start; max-width: 100%; }
.msg-error { color: hsl(var(--destructive)); font-size: 0.88rem; padding: 0.5rem 0.75rem; background: hsl(var(--destructive) / 0.08); border-radius: 8px; align-self: flex-start; max-width: 100%; }

/* ── ErrandCard（Plan 023 P3：原逃生舱 scoped 样式全局兜底）── */
.errand-card { border: 1px solid hsl(var(--border)); border-radius: 8px; margin: 0.5rem 0; overflow: hidden; background: hsl(38 92% 50% / 0.03); }
.errand-header { display: flex; align-items: center; gap: 0.4rem; padding: 0.5rem 0.75rem; cursor: pointer; font-size: 0.82rem; }
.errand-header:hover { background: hsl(38 92% 50% / 0.06); }
.errand-icon { font-size: 0.9rem; }
.errand-name-prefix { font-weight: 500; }
.errand-name { font-weight: 500; flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.errand-status { font-size: 0.7rem; padding: 0.1rem 0.4rem; border-radius: 3px; }
.errand-status.running { background: hsl(var(--primary) / 0.15); color: hsl(var(--primary)); }
.errand-status.completed { background: hsl(142 71% 45% / 0.15); color: hsl(142 71% 45%); }
.errand-status.failed { background: hsl(var(--destructive) / 0.15); color: hsl(var(--destructive)); }
.errand-cost { font-size: 0.7rem; color: hsl(var(--muted-foreground)); }
.errand-body { padding: 0.5rem 0.75rem; border-top: 1px solid hsl(var(--border)); }
.errand-task { font-size: 0.8rem; margin-bottom: 0.3rem; color: hsl(var(--foreground)); }
.errand-content { font-size: 0.78rem; white-space: pre-wrap; margin: 0.3rem 0; max-height: 300px; overflow-y: auto; }
.errand-tool-calls { margin-top: 0.4rem; }
.errand-sub-tool { padding: 0.2rem 0; border-left: 2px solid hsl(var(--border)); padding-left: 0.6rem; margin: 0.2rem 0; }
.errand-sub-tool-header { display: flex; align-items: center; gap: 0.4rem; font-size: 0.75rem; }
.errand-sub-tool-name { font-family: monospace; color: hsl(var(--muted-foreground)); }
.errand-sub-tool-status { font-size: 0.68rem; padding: 0.05rem 0.3rem; border-radius: 3px; }
.errand-sub-tool-status.running { background: hsl(var(--primary) / 0.15); color: hsl(var(--primary)); }
.errand-sub-tool-status.completed { background: hsl(142 71% 45% / 0.15); color: hsl(142 71% 45%); }
.errand-sub-tool-status.failed { background: hsl(var(--destructive) / 0.15); color: hsl(var(--destructive)); }
.errand-sub-tool-result { font-size: 0.72rem; white-space: pre-wrap; margin: 0.2rem 0; max-height: 150px; overflow-y: auto; }
.errand-result { margin-top: 0.5rem; }
.errand-result-label { font-size: 0.72rem; color: hsl(var(--muted-foreground)); margin-bottom: 0.2rem; }
.errand-result-text { font-size: 0.78rem; white-space: pre-wrap; max-height: 400px; overflow-y: auto; }

/* ── TaskPlanCard（Plan 023 P3：原逃生舱 scoped 样式全局兜底）── */
.task-plan-card { border: 1px solid hsl(var(--border)); border-radius: 8px; margin: 0.5rem 0; overflow: hidden; background: hsl(280 60% 96% / 0.4); }
.tp-header { display: flex; align-items: center; gap: 0.4rem; padding: 0.5rem 0.75rem; cursor: pointer; font-size: 0.82rem; }
.tp-header:hover { background: hsl(280 60% 96% / 0.7); }
.tp-icon { font-size: 0.9rem; }
.tp-title { font-weight: 500; flex: 1; }
.tp-status { font-size: 0.7rem; padding: 0.1rem 0.4rem; border-radius: 3px; }
.tp-status.running, .tp-status.started { background: hsl(var(--primary) / 0.15); color: hsl(var(--primary)); }
.tp-status.completed { background: hsl(142 71% 45% / 0.15); color: hsl(142 71% 45%); }
.tp-status.failed { background: hsl(var(--destructive) / 0.15); color: hsl(var(--destructive)); }
.tp-progress { font-size: 0.72rem; color: hsl(var(--muted-foreground)); }
.tp-body { padding: 0.5rem 0.75rem; border-top: 1px solid hsl(var(--border)); font-size: 0.8rem; }
.tp-field { display: flex; gap: 0.4rem; margin: 0.2rem 0; }
.tp-field-label { color: hsl(var(--muted-foreground)); min-width: 60px; }
.tp-field-value { font-family: monospace; font-size: 0.75rem; }
.tp-phases { margin: 0.4rem 0; display: flex; flex-direction: column; gap: 0.2rem; }
.tp-phase { display: flex; align-items: center; gap: 0.4rem; font-size: 0.75rem; padding: 0.15rem 0; }
.tp-phase-icon { font-size: 0.75rem; }
.tp-phase-name { flex: 1; color: hsl(var(--foreground)); }
.tp-phase-status { font-size: 0.68rem; color: hsl(var(--muted-foreground)); }
.tp-empty { font-size: 0.78rem; color: hsl(var(--muted-foreground)); font-style: italic; }

/* ── GenericToolCard（Plan 023 P3：原逃生舱 scoped 样式全局兜底）── */
.tool-card { border: 1px solid hsl(var(--border)); border-radius: 8px; margin: 0.5rem 0; overflow: hidden; }
.tool-header { display: flex; align-items: center; gap: 0.4rem; padding: 0.5rem 0.75rem; cursor: pointer; font-size: 0.82rem; }
.tool-header:hover { background: hsl(var(--accent)); }
.tool-icon { font-size: 0.9rem; }
.tool-name { font-weight: 500; font-family: monospace; }
.tool-seg { font-size: 0.75rem; color: hsl(var(--muted-foreground)); max-width: 300px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.seg-path { color: hsl(190 80% 40%); font-family: monospace; }
.seg-pattern { color: hsl(280 60% 45%); }
.seg-desc { color: hsl(var(--foreground)); }
.seg-loc { color: hsl(var(--muted-foreground)); font-family: monospace; }
.tool-status { font-size: 0.7rem; padding: 0.1rem 0.4rem; border-radius: 3px; margin-left: auto; }
.tool-status.running { background: hsl(var(--primary) / 0.15); color: hsl(var(--primary)); }
.tool-status.completed { background: hsl(142 71% 45% / 0.15); color: hsl(142 71% 45%); }
.tool-status.failed { background: hsl(var(--destructive) / 0.15); color: hsl(var(--destructive)); }
.tool-chevron { font-size: 0.7rem; color: hsl(var(--muted-foreground)); }
.tool-body { padding: 0.5rem 0.75rem; border-top: 1px solid hsl(var(--border)); }
.tool-section { margin: 0.3rem 0; }
.tool-section-title { font-size: 0.72rem; color: hsl(var(--muted-foreground)); margin-bottom: 0.2rem; text-transform: uppercase; letter-spacing: 0.05em; }
.tool-code, .tool-result { font-size: 0.75rem; white-space: pre-wrap; background: hsl(var(--muted) / 0.5); padding: 0.4rem; border-radius: 4px; max-height: 300px; overflow-y: auto; font-family: monospace; }

/* ── StreamingTable（Plan 023 P3：原逃生舱 scoped 样式全局兜底）──
   codegen 给 th/td 注入 Tailwind class（border/px-4/py-2），用 !important 覆盖对齐原版。 */
.streaming-table { margin: 0.5rem 0; overflow-x: auto; }
.streaming-table table { border-collapse: collapse; width: 100%; font-size: 0.93rem; }
.streaming-table th, .streaming-table td { border: 1px solid hsl(var(--border)) !important; padding: 0.4rem 0.6rem !important; text-align: left; }
.streaming-table th { background: hsl(var(--card)); font-weight: 600; color: hsl(var(--foreground)); }
.streaming-table td { color: hsl(var(--foreground)); }
.streaming-table tbody tr:nth-child(even) { background: hsl(var(--muted) / 0.5); }
.streaming-table .loading-row td { color: hsl(var(--muted-foreground)); font-style: italic; text-align: center; }
.streaming-table .loading-dots::after { content: ''; animation: st-dots 1.4s infinite both; }
@keyframes st-dots { 0%, 80%, 100% { content: ''; } 40% { content: '.'; } 60% { content: '..'; } }

/* ── AgentAvatar（Plan 023 P3：原逃生舱 scoped 样式全局兜底）── */
.agent-avatar {
  display: inline-flex; align-items: center; justify-content: center;
  flex-shrink: 0; border-radius: 50%; font-weight: 600; line-height: 1;
  user-select: none; font-family: system-ui, -apple-system, sans-serif; overflow: hidden;
}
.agent-avatar.xs { width: 18px; height: 18px; font-size: 0.65rem; }
.agent-avatar.sm { width: 30px; height: 30px; font-size: 0.9rem; }
.agent-avatar.md { width: 28px; height: 28px; font-size: 1rem; }
.agent-avatar.lg { width: 48px; height: 48px; font-size: 1.4rem; }

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
  background: transparent !important;
  border: none !important;
  border-radius: 0 !important;
}
/* textarea 自身去边框（靠容器的边框+圆角），block+100% 宽自适应 */
.input-compose textarea,
.chats-input {
  display: block !important;
  width: 100% !important;
  border: none !important; border-radius: 0 !important; background: transparent !important;
  font-size: 0.95rem; resize: none; outline: none;
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
  width: 200px; flex-shrink: 0; display: flex; flex-direction: column; gap: 0.25rem;
  /* 容器结构与 ChatsView 二级导航一致：无 padding，header 贴顶全宽 border-bottom */
  padding: 0; border-right: 1px solid hsl(var(--border)); background: hsl(var(--card));
  overflow-y: auto;
}
.section-nav-header { display: flex; align-items: center; gap: 0.4rem; padding: 0.5rem 0.75rem; flex-shrink: 0; }
.section-nav-title { font-size: 0.95rem; font-weight: 700; color: hsl(var(--foreground)); }
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
.section-header { display: flex; align-items: center; justify-content: space-between; margin: 0 -1.25rem 0.5rem; }
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
  width: 240px; flex-shrink: 0; display: flex; flex-direction: column; gap: 0.25rem;
  /* 容器结构与 ChatsView 二级导航一致：无 padding，header 贴顶全宽 border-bottom */
  padding: 0; border-right: 1px solid hsl(var(--border)); background: hsl(var(--card));
  overflow-y: auto;
}
.wiki-nav-header { display: flex; align-items: center; justify-content: space-between; padding: 0.5rem 0.75rem; flex-shrink: 0; }
.wiki-nav-title { font-size: 0.95rem; font-weight: 700; color: hsl(var(--foreground)); }
.nav-icon-btn { width: 28px; height: 28px; display: inline-flex; align-items: center; justify-content: center; border: 1px solid hsl(var(--border)); border-radius: 6px; background: transparent; cursor: pointer; color: hsl(var(--foreground)); font-size: 1rem; }
.nav-icon-btn:hover { background: hsl(var(--accent)); }
.wiki-nav-list { flex: 1; overflow-y: auto; }
.wiki-nav-item {
  display: block; width: 100%; text-align: left;
  padding: 0.4rem 0.75rem; border: none; border-radius: 6px; font-size: 0.85rem;
  background: transparent; color: hsl(var(--foreground)); cursor: pointer;
}
.wiki-nav-item:hover { background: hsl(var(--accent)); }
.wiki-nav-item.active { background: hsl(var(--accent)); font-weight: 500; }
.wiki-main { flex: 1; overflow-y: auto; display: flex; flex-direction: column; }
.wiki-content-header { display: flex; align-items: center; justify-content: space-between; margin: 0 -1.25rem 0.5rem; }
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

/* ── RelayRunBox（Plan 023 队列 A3 原生化：逃生舱 scoped 转全局兜底）── */
.relay-box {
  border: 1px solid var(--af-border); border-radius: 8px; margin: 0.5rem 0;
  overflow: hidden; background: hsl(var(--muted-foreground) / 0.03);
}
.status-running { border-left: 3px solid hsl(var(--primary)); }
.status-completed { border-left: 3px solid hsl(142 71% 45%); }
.status-failed { border-left: 3px solid hsl(var(--af-error)); }
.status-gate { border-left: 3px solid hsl(38 92% 50%); }
.box-header {
  display: flex; align-items: center; gap: 0.4rem;
  padding: 0.5rem 0.75rem; cursor: pointer; font-size: 0.82rem; color: var(--af-fg);
}
.box-header:hover { background: hsl(var(--muted-foreground) / 0.06); }
.box-title { font-weight: 500; flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.box-progress { font-size: 0.72rem; color: var(--af-muted); }
.box-status { font-size: 0.7rem; padding: 0.1rem 0.4rem; border-radius: 3px; }
.badge-running { background: hsl(var(--primary) / 0.15); color: hsl(var(--primary)); }
.badge-completed { background: hsl(142 71% 45% / 0.15); color: hsl(142 71% 45%); }
.badge-failed { background: hsl(var(--af-error) / 0.15); color: hsl(var(--af-error)); }
.badge-gate { background: hsl(38 92% 50% / 0.15); color: hsl(38 92% 50%); }
.box-body { padding: 0.5rem 0.75rem; border-top: 1px solid var(--af-border); }
.log-entries { max-height: 400px; overflow-y: auto; font-size: 0.78rem; line-height: 1.5; }
.log-entry { padding: 0.15rem 0; }
.entry-prof { margin-right: 0.3rem; }
.entry-text { color: var(--af-fg); }
.entry-tool { display: flex; align-items: center; gap: 0.3rem; color: var(--af-muted); padding-left: 1rem; }
.tool-name { font-family: monospace; font-size: 0.74rem; }
.entry-step { color: var(--af-muted); font-size: 0.75rem; padding: 0.2rem 0; }
.entry-step.done { color: hsl(142 71% 45%); }
.entry-gate { color: hsl(38 92% 50%); padding: 0.3rem 0; font-weight: 500; }
.entry-error { color: hsl(var(--af-error)); }
.entry-done { color: hsl(142 71% 45%); font-weight: 500; padding: 0.3rem 0; }
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
  border-radius: 10px; padding: 0.75rem 1rem; margin-top: 0.5rem;
  display: flex; flex-direction: column; gap: 0.75rem;
}
.q-header { display: flex; align-items: center; gap: 0.4rem; font-size: 0.85rem; font-weight: 600; color: var(--af-primary); }
.q-item { display: flex; flex-direction: column; gap: 0.35rem; }
.q-text { font-size: 0.88rem; font-weight: 500; color: var(--af-fg); line-height: 1.4; }
.q-options { display: flex; flex-direction: column; gap: 0.15rem; }
.q-option {
  display: flex; align-items: center; gap: 0.4rem; padding: 0.35rem 0.5rem;
  border: none; border-radius: 6px; background: transparent; cursor: pointer;
  transition: background 0.1s; font-size: 0.85rem; color: var(--af-fg); text-align: left; width: 100%;
}
.q-option:hover { background: hsl(var(--primary) / 0.06); }
.q-option.checked { background: hsl(var(--primary) / 0.08); }
.q-check {
  width: 14px; height: 14px; border: 2px solid var(--af-border); border-radius: 50%;
  flex-shrink: 0; display: flex; align-items: center; justify-content: center; transition: all 0.15s;
}
.q-check.square { border-radius: 4px; }
.q-option.checked .q-check { border-color: var(--af-primary); background: var(--af-primary); }
.q-option.checked .q-check::after {
  content: ''; width: 5px; height: 5px; border-radius: 50%; background: #fff;
}
.q-option.checked .q-check.square::after {
  width: 6px; height: 3px; border-radius: 0; background: transparent;
  border-left: 2px solid #fff; border-bottom: 2px solid #fff;
  transform: rotate(-45deg); margin-bottom: 1px;
}
.q-label { line-height: 1.3; }
.q-text-input input, .q-other-input {
  width: 100%; padding: 0.4rem 0.6rem; border: 1px solid var(--af-border); border-radius: 6px;
  background: var(--af-bg); color: var(--af-fg); font-size: 0.85rem; outline: none; transition: border-color 0.15s;
}
.q-text-input input:focus, .q-other-input:focus { border-color: var(--af-primary); }
.q-other-row { display: flex; align-items: center; gap: 0.4rem; padding: 0.35rem 0.5rem; font-size: 0.85rem; color: var(--af-fg); }
.q-other-label { flex-shrink: 0; color: var(--af-muted); }
.q-other-input { flex: 1; min-width: 0; }
.q-submit {
  display: inline-flex; align-items: center; justify-content: center; gap: 0.3rem;
  align-self: flex-start; padding: 0.4rem 0.8rem; border: none; border-radius: 6px;
  background: var(--af-primary); color: #fff; font-size: 0.82rem; font-weight: 500;
  cursor: pointer; transition: opacity 0.15s;
}
.q-submit:disabled { opacity: 0.5; cursor: not-allowed; }
.q-submit:hover:not(:disabled) { opacity: 0.9; }

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
`

export function injectStyles(): void {
  if (document.getElementById('musk-global-styles')) return
  const style = document.createElement('style')
  style.id = 'musk-global-styles'
  style.textContent = STYLES
  document.head.appendChild(style)
}
