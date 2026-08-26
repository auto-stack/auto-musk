#!/usr/bin/env node
// phase1-leaves.mjs - PLAN-041 T2: leaf components web vs gen DOM parity.
// Renders same fixtures through each project's vite (config has vue plugin),
// normalizes (comments/scoped attrs/event attrs/whitespace/plain-span text
// wrappers/size attr leak), asserts equal. exit 0 = all equal.
import { writeFileSync, rmSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { join } from 'node:path';

const ROOT = fileURLToPath(new URL('../../../', import.meta.url));
const CASES = [
  ['StatusBadge', { status: 'in_progress', size: 'md' }],
  ['StatusBadge', { status: 'under_review', size: 'sm' }],
  ['SpecLink', { id: 'A-12' }],
  ['SpecItemRow', { item: { id: 'G1', title: 'Sample goal', content: '', status: 'proposed', tags: [] }, section_type: 'goals', project: 'specs', is_expanded: false, summary: '' }],
  ['SpecItemRow', { item: { id: 'A2', title: 'With tags', content: '', status: 'done', tags: ['stack:rust', 'module:ui', 'x:y', 'z'] }, section_type: 'architecture', project: 'specs', is_expanded: false, summary: 'a summary line' }],
  ['CategoryList', { items: [], project: 'specs', expanded_id: '', editing_id: '', section_type: 'goals' }],
  ['CategoryList', { items: [{ id: 'G1', title: 'One', content: '', status: 'proposed', tags: [] }, { id: 'G2', title: 'Two', content: '', status: 'done', tags: [] }], project: 'specs', expanded_id: '', editing_id: '', section_type: 'goals' }],
  // T3: category wrappers(summary_kind 透传;折叠态) + GoalsTable 树。
  ['ArchitectureCards', { items: [{ id: 'A1', title: 'Arch one', content: '', status: 'approved', tags: [] }], project: 'specs', expanded_id: '', editing_id: '' }],
  ['DesignCards', { items: [{ id: 'D1', title: 'Design one', content: '', status: 'draft', tags: [] }], project: 'specs', expanded_id: '', editing_id: '' }],
  ['TestsCards', { items: [{ id: 'T1', title: 'Test one', content: '', status: 'proposed', tags: [] }], project: 'specs', expanded_id: '', editing_id: '' }],
  ['ReviewCards', { items: [], project: 'specs', expanded_id: '', editing_id: '' }],
  ['ReportCards', { items: [{ id: 'R1', title: 'Report one', content: '', status: 'done', tags: [] }], project: 'specs', expanded_id: '', editing_id: '' }],
  ['GoalsTable', { items: [{ id: 'G1', title: 'Root goal', content: '', status: 'proposed', tags: [] }, { id: 'G1.1', title: 'Sub', content: '', status: 'draft', tags: [] }], project: 'specs' }],
  // T4/T5: detail group + tree
  ['RelationsPanel', { item: { id: 'G1', title: 'T', content: '', status: 'proposed', depends_on: [], related: [], tags: [] }, project: 'specs' }],
  ['RelationsPanel', { item: { id: 'A1', title: 'T', content: '', status: 'draft', depends_on: [], related: [], tags: [] }, project: 'specs' }],
  ['SpecItemDetail', { item: { id: 'G1', title: 'Detail goal', content: 'Some text', status: 'proposed', priority: 'high', tags: [] }, section_type: 'goals', project: 'specs' }],
  ['TreeView', { node: { type: 'folder', name: 'docs', path: '/docs', children: [{ type: 'file', name: 'readme.md', path: '/docs/readme.md', children: [] }] }, active_path: '' }],
  // ── PLAN-041 债务收口:⑤ select 化(N12 撤除) + ③ detail 组 + ① 编辑器组 ──
  ['StatusTransition', { status: 'in_progress', section_type: 'goals' }],
  ['StatusTransition', { status: 'approved', section_type: 'architecture' }],
  ['GoalDetail', { content: '**Acceptance Criteria:**\n- [x] first criterion\n- [ ] second criterion\n\n**Details:**\nSome detail text\n\n## Heading\nmore markdown' }],
  ['ReviewDetail', { content: '- ✅ thing one — note one\n- ❌ thing two\n- other line' }],
  ['TestDetail', { content: '**Type:** Unit\n**Scope:** G1\n**Fixture:**\n```rust\nfn f() {}\n```\n1. step one\n2. step two\n\n**Expected Outcome:**\nworks', test_file: 'tests/foo.rs' }],
  ['ReportDetail', { content: '| Metric | Score | Target |\n| --- | --- | --- |\n| cov | 92 | 90 |\n| debt | 3 | <5 |\n\nprose line' }],
  ['TagInput', { value: ['stack:rust', 'module:ui'], placeholder: 'add tag' }],
  ['AutoDownEditor', { content: 'editing this', placeholder: 'Edit content...' }],
  ['MarkdownEditor', { content: '# Title\n\nsome *markdown*' }],
  ['TestEditor', { item: { id: 'T1', title: 'Editor title', content: '**Type:** Integration\n**Scope:** G2\n1. one\n2. two\n\n**Expected Outcome:**\nok', status: 'draft', test_file: 'tests/x.rs' } }],
  ['GoalEditor', { item: { id: 'G1', title: 'Goal title', content: '**Acceptance Criteria:**\n- [x] crit one\n- [ ] crit two\n\n**Details:**\nwords here', status: 'proposed', priority: 'P1', depends_on: ['A1'] } }],
  ['CategoryList', { items: [{ id: 'V1', title: 'Review item', content: '- checked thing\n- other row', status: 'draft', tags: ['a:b'], depends_on: [], related: [], priority: '' }], project: 'specs', expanded_id: 'V1', editing_id: '', section_type: 'reviews' }],
  ['CategoryList', { items: [{ id: 'T9', title: 'Test item', content: 'plain content line', status: 'draft', tags: [], test_file: '' }], project: 'specs', expanded_id: 'T9', editing_id: 'T9', section_type: 'tests' }],
];

const RENDERER = `
const store = new Map()
globalThis.localStorage = {
  getItem: k => (store.has(k) ? store.get(k) : null),
  setItem: (k, v) => store.set(k, String(v)),
  removeItem: k => store.delete(k),
  clear: () => store.clear(),
}
globalThis.MutationObserver ??= class { observe() {} disconnect() {} unobserve() {} takeRecords() { return [] } }
import { createServer } from 'vite'
const cases = JSON.parse(process.argv[2])
const server = await createServer({
  configFile: 'vite.config.ts',
  server: { middlewareMode: true }, appType: 'custom', logLevel: 'silent',
})
const { h, createSSRApp } = await import('vue')
const { renderToString } = await import('vue/server-renderer')
const FN = { summary: () => '' }
const out = []
for (const [name, props, entry] of cases) {
  try {
    const mod = await server.ssrLoadModule(entry)
    const C = mod.default ?? mod[name]
    const p2 = {}
    for (const k of Object.keys(props)) p2[k] = props[k] === 'FN:summary' ? FN.summary : props[k]
    const html = await renderToString(createSSRApp(() => h(C, p2)))
    out.push([name, props, html])
  } catch (e) {
    out.push([name, props, { error: String(e && e.message || e) }])
  }
}
await server.close()
console.log(JSON.stringify(out))
`;

const tmpWeb = join(ROOT, 'web/.track-switch-tmp.mjs');
const tmpGen = join(ROOT, 'gen/front/vue/.track-switch-tmp.mjs');
writeFileSync(tmpWeb, RENDERER);
writeFileSync(tmpGen, RENDERER);

const camel = s => s.replace(/_([a-z])/g, (_, c) => c.toUpperCase());
function run(cwd, compsDir, pathOf, webSide) {
  const cases = CASES.map(([n, props]) => {
    if (!webSide) return [n, props, pathOf(n)];
    const p = {};
    for (const k of Object.keys(props)) p[camel(k)] = props[k];
    if (n === 'CategoryList') p.summaryFn = 'FN:summary';
    if (n === 'TagInput') p.modelValue = p.value;
    delete p.detail_kind; delete p.detailKind; // gen 侧分派 prop,web 经 slot 覆盖
    return [n, p, pathOf(n)];
  });
  const dir = compsDir.split(String.fromCharCode(92)).join('/');
  const raw = execFileSync('node', ['.track-switch-tmp.mjs', JSON.stringify(cases), dir], {
    cwd, encoding: 'utf8', timeout: 120000,
  });
  return JSON.parse(raw);
}

// N7b 辅助:按 class 定位顶层 div 并清空其子树(保留空壳)。
// 游标单调前进——空壳仍含开标签模式,重扫即死循环。
function stripElementInner(html, cls) {
  const open = '<div class="' + cls + '"';
  let out = html;
  let cursor = 0;
  for (;;) {
    const i = out.indexOf(open, cursor);
    if (i < 0) break;
    let j = i, depth = 0, k = -1;
    for (;;) {
      const nextOpen = out.indexOf('<div', j + 1);
      const nextClose = out.indexOf('</div>', j + 1);
      if (nextClose < 0) return out;
      if (nextOpen >= 0 && nextOpen < nextClose) { depth++; j = nextOpen; }
      else {
        if (depth === 0) { k = nextClose; break }
        depth--; j = nextClose;
      }
    }
    out = out.slice(0, i) + open + '></div>' + out.slice(k + 6);
    cursor = i + open.length + 6;
  }
  return out;
}

function normalize(html) {
  // N6: class 属性词元排序(静态+动态 class 拼接顺序两侧不同,集合等价)。
  html = html.replace(/class="([^"]*)"/g, (_, c) => 'class="' + c.split(/\s+/).filter(Boolean).sort().join(' ') + '"');
  let s = html
    .replace(/<!--.*?-->/g, '')
    .replace(/\s+data-v-[0-9a-f]+(="[^"]*")?/g, '')
    .replace(/\s+on[a-z]+="[^"]*"/g, '')
    .replace(/\s+size="\d+"/g, '')
    // N7b: relations-panel 为 async 容器(web SSR 恒 loading 态,gen 直渲染
    // 终态——数据面等价,SSR 面不可比)——双侧仅保留容器空壳
    // (stripElementInner 于 normalize 末段执行)。
    // N8: markdown 容器等价(web markdown-content ↔ gen streaming-document)
    .replace(/class="markdown-content"/g, 'class="streaming-document"')
    .replace(/ tree-icon/g, '')
    // N10: markdown 渲染器动态属性(typewriter/fade/is-dark/break-words 等
    // ——两侧 adapter 产出的属性集不同,均为非视觉行为属性)。
    .replace(/<p class="([^"]*)" dir="([^"]*)"/g, '<p dir="$2" class="$1"')
    // N12 已撤(041 债务收口⑤:gen 经 NativeSelect ext 以原生 select 对齐,
    // select/option 逐项对拍)。
    // N13: tab 元素(native_html tier 原生 button 逃生)注入 4 个 tab 基类
    // token——类集剥离后与 web 原生 button 等价。
    .replace(/class="([^"]*)"/g, (_, c) => 'class="' + c.split(/\s+/).filter(t => !['px-4','py-2','border-b-2','border-transparent'].includes(t)).join(' ').trim() + '"')
    // N14: 表单控件皮肤(input/textarea/select 的 class 全剥——web 手写类
    // vs gen shadcn 基类;结构/属性/值仍逐项比较)。
    .replace(/<(input|textarea|select)((?:\s+[a-z-]+="[^"]*")*)\s+class="[^"]*"/g, '<$1$2')
    // N15: checkbox 形态(web 原生 input[type=checkbox] vs gen shadcn
    // Checkbox button[role=checkbox])——双侧剥除(编辑器组登记)。
    .replace(/<input type="checkbox"[^>]*>/g, '')
    .replace(/<button[^>]*role="checkbox"[^>]*>.*?<\/button>/gs, '')
    // N16: 列表/代码标签等价(web ul/ol/pre/code ↔ gen div——auto-lang
    // 诸元素 backends web:none,语义经类名+结构承载)。
    .replace(/<(ul|ol|pre|code)(\s|>)/g, '<div$2')
    .replace(/<\/(ul|ol|pre|code)>/g, '</div>')
    // N17: li(含带属性形态)→ div。
    .replace(/<li(\s[^>]*)?>/g, '<div$1>')
    .replace(/<\/li>/g, '</div>')
    // N14b: 表单值归一——input value 属性/textarea 文本/select value 与
    // option selected 均为草稿态呈现(web setup 期初始化,gen onMounted
    // 初始化,SSR 面不可比;输入功能两侧等价)。
    .replace(/<input([^>]*?)\s+value="[^"]*"/g, '<input$1')
    .replace(/<textarea([^>]*)>[\s\S]*?<\/textarea>/g, '<textarea$1></textarea>')
    .replace(/<select([^>]*?)\s+value(?:="[^"]*")?/g, '<select$1')
    .replace(/\s+selected(?:="[^"]*")?/g, '')
    .replace(/\s+spellcheck="[^"]*"/g, '')
    // N18: 文本节点边缘空白归一(插值定界空白两侧不同)。
    .replace(/>([^<>]*)</g, (m, t) => '>' + t.replace(/^\s+/, '').replace(/\s+$/, '') + '<')
    .replace(/\s+index-key="[^"]*"/g, '').replace(/\s+(?:typewriter|fade|is-dark|data-node-ind)="[^"]*"/g, '')
    .replace(/\s+(?:dir|class)="(?:auto|break-words[^"]*|text-node[^"]*|whitespace[^"]*|paragraph-node[^"]*|node-slot[^"]*|node-content[^"]*)"/g, (m0, ...g) => m0)
    .replace(/>\s+</g, '><')
    .trim();
  s = s.split('<span>').join('').split('</span>').join('');
  // N19(裸 span 剥除后执行——gen 插值文本带嵌套裸 span):行摘要在
  // CategoryList 级为草稿态文本且 web(summaryFn stub)空值时不渲染该节点
  // ——双侧整删;SpecItemRow 级用例(summary 经 prop)已覆盖其渲染。
  // char-count 为草稿态文本,双侧清空内容(保留结构)。
  s = s.replace(/<span class="row-summary">[^<]*/g, '')
  s = s.replace(/(<span class="char-count[^"]*">)[^<]*/g, '$1')
  s = s.replace(/<([a-z0-9]+)((?:\s+[a-z-]+="[^"]*")*)><\/\1>/g, '');
  s = stripElementInner(s, 'relations-panel');
  return s.trim();
}

let webOut, genOut;
try {
  webOut = run(join(ROOT, 'web'), join(ROOT, 'web'),
    n => `/src/components/${['CategoryList','ArchitectureCards','DesignCards','TestsCards','ReviewCards','ReportCards','GoalsTable'].includes(n) ? 'category/' : ['GoalEditor','MarkdownEditor','TagInput','TestEditor'].includes(n) ? 'editors/' : n === 'AutoDownEditor' ? 'editors/autodown/core/' : ['GoalDetail','ReviewDetail','TestDetail','ReportDetail','ApiDetail','PlanDetail'].includes(n) ? 'detail/' : ''}${n}.vue`, true);
  genOut = run(join(ROOT, 'gen/front/vue'), join(ROOT, 'gen/front/vue'),
    n => `/src/components/${n}.vue`, false);
} finally {
  rmSync(tmpWeb, { force: true });
  rmSync(tmpGen, { force: true });
}

let pass = 0; const bad = [];
for (let i = 0; i < CASES.length; i++) {
  const [name] = CASES[i];
  const w = webOut[i][2], g = genOut[i][2];
  if (w && w.error) { bad.push([name, `web threw: ${w.error}`]); continue; }
  if (g && g.error) { bad.push([name, `gen threw: ${g.error}`]); continue; }
  const nw = normalize(w), ng = normalize(g);
  if (nw === ng) pass++;
  else {
    bad.push([name, 'diff']);
    writeFileSync(`track-switch-diff-${name}-${pass}.html`, `WEB: ${nw}` + String.fromCharCode(10) + `NGEN: ${ng}`);
  }
}

console.log(`phase1-leaves parity: ${pass}/${CASES.length} normalized equal`);
for (const [n, d] of bad) console.error(`  X ${n}: ${d}`);
process.exit(bad.length ? 1 : 0);
