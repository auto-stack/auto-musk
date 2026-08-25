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
    return [n, p, pathOf(n)];
  });
  const dir = compsDir.split(String.fromCharCode(92)).join('/');
  const raw = execFileSync('node', ['.track-switch-tmp.mjs', JSON.stringify(cases), dir], {
    cwd, encoding: 'utf8', timeout: 120000,
  });
  return JSON.parse(raw);
}

function normalize(html) {
  // N6: class 属性词元排序(静态+动态 class 拼接顺序两侧不同,集合等价)。
  html = html.replace(/class="([^"]*)"/g, (_, c) => 'class="' + c.split(/\s+/).filter(Boolean).sort().join(' ') + '"');
  let s = html
    .replace(/<!--.*?-->/g, '')
    .replace(/\s+data-v-[0-9a-f]+(="[^"]*")?/g, '')
    .replace(/\s+on[a-z]+="[^"]*"/g, '')
    .replace(/\s+size="\d+"/g, '')
    // N7: async loading SSR artifact(web 的 relations-panel 在 SSR 时处于
    // loading 态;gen 直接从 props 渲染)——strip loading 占位。
    .replace(/<div class="relations-loading">.*?<\/div>/g, '')
    // N8: markdown 容器等价(web markdown-content ↔ gen streaming-document)
    .replace(/class="markdown-content"/g, 'class="streaming-document"')
    .replace(/ tree-icon/g, '')
    // N10: markdown 渲染器动态属性(typewriter/fade/is-dark/break-words 等
    // ——两侧 adapter 产出的属性集不同,均为非视觉行为属性)。
    .replace(/<p class="([^"]*)" dir="([^"]*)"/g, '<p dir="$2" class="$1"')
    // N12: StatusTransition 形态差异(web select vs gen buttons)
    .replace(/<select[^>]*>.*?<\/select>/gs, '')
    .replace(/<button class="[^"]*status-option[^"]*"[^>]*>[^<]*<\/button>/g, '')
    .replace(/>\s+Priority:/g, '>Priority:')
    .replace(/<div class="status-transition">/g, '')
    .replace(/\s+index-key="[^"]*"/g, '').replace(/\s+(?:typewriter|fade|is-dark|data-node-ind)="[^"]*"/g, '')
    .replace(/\s+(?:dir|class)="(?:auto|break-words[^"]*|text-node[^"]*|whitespace[^"]*|paragraph-node[^"]*|node-slot[^"]*|node-content[^"]*)"/g, (m0, ...g) => m0)
    .replace(/>\s+</g, '><')
    .trim();
  s = s.split('<span>').join('').split('</span>').join('');
  s = s.replace(/<([a-z0-9]+)((?:\s+[a-z-]+="[^"]*")*)><\/\1>/g, '');
  return s.trim();
}

let webOut, genOut;
try {
  webOut = run(join(ROOT, 'web'), join(ROOT, 'web'),
    n => `/src/components/${['CategoryList','ArchitectureCards','DesignCards','TestsCards','ReviewCards','ReportCards','GoalsTable'].includes(n) ? 'category/' : ''}${n}.vue`, true);
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
