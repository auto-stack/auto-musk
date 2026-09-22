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
/* ── 字体（离线系统字体栈，消除网络依赖，PLAN-075）── */
body, button, input, textarea, select { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Noto Sans SC", "PingFang SC", "Microsoft YaHei", sans-serif; }

/* Theme tokens are generated from pac.at for Vue and VM. */
:root {
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
/* r9（PLAN-073 续）：输入框 backdrop 的引用 token（@plan/<seq>、
   @spec/<relpath>）语义类——codegen __autoMentionHtml 对名单命中的非纯词
   token 附加 mention-token。紫系着色（品牌主色）区别 Agent @词（蓝）；
   零宽度样式（禁 padding/border）不改字宽，backdrop 与 textarea 逐字
   对齐不受影响。 */
.mention-token {
  color: hsl(var(--primary));
  background: hsl(var(--primary) / 0.10);
  border-radius: 4px;
}
.msg-bubble-ai .streaming-document { color: hsl(var(--foreground)); }
/* PLAN-056 T6 → PLAN-076 T-04 已退役：markdown 块间节奏已由上游 @autodown/engine
   StreamingRenderer.vue 原生内置（.streaming-document :deep(.markdown-renderer > .node-slot + .node-slot)），
   宿主层覆盖规则在此彻底清退。 */
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
/* ══ PLAN-054 B1 → PLAN-076 T-04 已退役：@autodown/engine 深色主题覆盖 ══
   上游 @autodown/engine 现已原生支持环境暗色（ambient dark）自动激活
   .is-dark 状态，Design 22 §7 全套 token（--ad-fg/border/surface/accent）与
   代码块、表格、Admonition、Mermaid、Details 等组件级深色样式均由引擎原生
   管理，宿主层覆盖规则在此彻底清退。 */

/* ══ IME 组合串可见性（PLAN-493 双层文字技术的组词缺口）══
   textarea 常态 color:transparent（显字由 backdrop 层承担）,组词中的
   composition 文字不进 v-model（Vue 组词期冻结）、不进 backdrop,浏览器
   对透明色的组合串回退系统黑——深色主题下不可读。组词期间给 textarea
   挂 .ime-composing（换主题前景色实绘,深浅色自动跟随 --foreground 翻转;
   浅色下≈深墨与原系统黑观感一致）,backdrop 兄弟层挂 .ime-composing-
   backdrop 隐藏,避免整段文字双层叠绘与 mention 底色混色。 */
textarea.chats-input.ime-composing {
  color: hsl(var(--foreground));
  -webkit-text-fill-color: hsl(var(--foreground));
}
.chats-input.ime-composing-backdrop { visibility: hidden; }
/* 选区可见性：透明 textarea 的默认选区高亮对比随浏览器/系统色模式浮动,
   暗色下近乎不可见,形似"Ctrl+A 无效"（DOM 选区实测正常）。显式给主题
   选区配色:primary 半透明底 + 前景色字——选中时 textarea 层文字也以前
   景色实绘,与 backdrop 层同色叠加,深浅主题自动翻转。 */
textarea.chats-input::selection {
  background: hsl(var(--primary) / 0.32);
  color: hsl(var(--foreground));
}
/* ═══════════════════════════════════════════════════ */

/* PLAN-071 r4（需求③-3）：鹿 logo 双主题切换 + 收缩态导航按钮。
   主题经 html.dark 翻转（useVisualStore）；light 变体默认 inline
   display:none（VM 轨无本样式表，兜底单图）。 */
.deer-icon-light { display: none; }
:root:not(.dark) .deer-icon-dark { display: none !important; }
:root:not(.dark) .deer-icon-light { display: block !important; }
.rail-icon-btn { width: 2.5rem; height: 2.5rem; display: flex; align-items: center; justify-content: center; border-radius: 0.375rem; background: transparent; border: none; color: hsl(var(--muted-foreground)); cursor: pointer; padding: 0; }
.rail-icon-btn:hover { background: hsl(var(--accent)); }
.rail-icon-btn.active { background: hsl(var(--accent)); color: hsl(var(--primary)); }
/* 收缩态底部触发器（WorkspaceSelector/SettingsMenu）——脚手架 Button 的
   [&_svg]:size-4 会把 lucide size 属性压成 16px，这里以 !important 夺回。 */
.rail-trigger-24 svg { width: 24px !important; height: 24px !important; }
`

// 组词状态钩子:compositionstart/end 在 document 冒泡段监听,只认
// "textarea.chats-input 且前邻兄弟为 backdrop div.chats-input"的双层结构
// （其余输入面零影响）。end 走冒泡段保证晚于 Vue v-model 的组词同步
// （元素上先跑）,类移除与 backdrop 文字刷新同帧落定,无闪断。
function installImeComposingHook(): void {
  const pair = (target: EventTarget | null): [HTMLTextAreaElement, HTMLElement] | null => {
    if (!(target instanceof HTMLTextAreaElement) || !target.classList.contains('chats-input')) return null
    const backdrop = target.previousElementSibling
    if (!(backdrop instanceof HTMLDivElement) || !backdrop.classList.contains('chats-input')) return null
    return [target, backdrop]
  }
  const flip = (target: EventTarget | null, on: boolean): void => {
    const pairRes = pair(target)
    if (!pairRes) return
    pairRes[0].classList.toggle('ime-composing', on)
    pairRes[1].classList.toggle('ime-composing-backdrop', on)
  }
  document.addEventListener('compositionstart', (e) => flip(e.target, true))
  document.addEventListener('compositionend', (e) => flip(e.target, false))
}

export function injectStyles(): void {
  if (document.getElementById('musk-global-styles')) return
  const style = document.createElement('style')
  style.id = 'musk-global-styles'
  style.textContent = STYLES
  document.head.appendChild(style)
  installImeComposingHook()
}
