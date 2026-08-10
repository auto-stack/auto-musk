// inject_styles.ts — 全局布局样式注入（Plan 022 C 类 parity）
//
// AutoUI 生成的组件 <style> 为空，自定义语义 class（chats-view/session-list/
// chats-canvas/msg-* 等）无对应 CSS。原生 web/ 把这些放在各组件 <style scoped>。
// 这里集中注入全局样式，对齐原生视觉。
//
// 逃生舱说明：AutoUI .at 无法表达 scoped CSS，用 use { fn } 在 App.Init 注入。

const STYLES = `
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
/* 标题统一为大写灰标签风格（对齐原版 sidebar-title / section-nav-title 等） */
.sidebar-title, .section-nav-title, .wiki-nav-title {
  font-size: 0.8rem !important;
  font-weight: 500 !important;
  color: hsl(var(--muted-foreground)) !important;
  text-transform: uppercase !important;
  letter-spacing: 0.04em !important;
}

/* ── 导航栏 ── */
.rail-tab {
  width: 100%; text-align: left; padding: 0.5rem 0.75rem; border-radius: 0.375rem;
  font-size: 0.875rem; color: hsl(var(--muted-foreground)); background: transparent;
  border: none; cursor: pointer; transition: all 0.15s;
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
  padding: 0.75rem 0.5rem; border-right: 1px solid hsl(var(--border)); background: hsl(var(--card));
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
.specs-main { flex: 1; overflow-y: auto; padding: 1.25rem; display: flex; flex-direction: column; }
.overview-content { font-size: 0.9rem; line-height: 1.6; color: hsl(var(--foreground)); }
.section-content { margin-bottom: 1rem; }
.section-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 0.5rem; }
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
  padding: 0.75rem 0.5rem; border-right: 1px solid hsl(var(--border)); background: hsl(var(--card));
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
.wiki-main { flex: 1; overflow-y: auto; padding: 1.25rem; display: flex; flex-direction: column; }
.wiki-content-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 0.75rem; }
.wiki-content-actions { display: flex; gap: 0.25rem; }
.wiki-content { font-size: 0.9rem; line-height: 1.6; color: hsl(var(--foreground)); }
.wiki-markdown { font-size: 0.9rem; line-height: 1.6; }
.wiki-editor { padding: 1rem; border: 1px solid hsl(var(--border)); border-radius: 8px; background: hsl(var(--card)); }
.wiki-empty { display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; color: hsl(var(--muted-foreground)); gap: 0.5rem; text-align: center; }
.text-muted { color: hsl(var(--muted-foreground)); }
`

export function injectStyles(): void {
  if (document.getElementById('musk-global-styles')) return
  const style = document.createElement('style')
  style.id = 'musk-global-styles'
  style.textContent = STYLES
  document.head.appendChild(style)
}
