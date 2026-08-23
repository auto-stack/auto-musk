#!/usr/bin/env node
// render-switch.mjs — PLAN-038 T13：渲染真源切换（ports/renderer MarkdownRender：
// markstream-vue 直绑 → MarkdownRender.vue 适配器内部消费 @autodown/vue
// StreamingRenderer）前后的 DOM 对拍。
//
// 两侧：
//   OLD = markstream-vue MarkdownRender（content/final——原端口消费形态）
//   NEW = @autodown/vue StreamingRenderer（source=content, streaming=false——
//         适配器的内部委托形态）
// fixtures = scripts/lib-parity/fixtures/render/*.md（musk 真实内容采样 + 构造边界）。
//
// 归一化（差异白名单,显式登记）：
//   W1 解包 <div class="streaming-document"> 容器（NEW 侧超集容器）
//   W2 丢弃上游 codeBlockProps 增量子树（.code-block-header 语言徽标/复制/展开钮、
//      .mermaid-block-header、.autodown-block-placeholder）
//   W3 丢弃 HTML 注释占位与全部属性（class 增量如 typewriter/batch）
// 归一后断言：标签序列 + 可见文本全等；白名单外零差异 → exit 0。
// SSR 侧 MutationObserver/onMounted 不执行——katex/mermaid/lowlight 客户端后处理
// 不在对拍面（两侧同不生效,公平）。

import { readFileSync, readdirSync, writeFileSync, rmSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { join } from 'node:path';
import { execFileSync } from 'node:child_process';

const ROOT = fileURLToPath(new URL('../../', import.meta.url));
const FIXTURE_DIR = join(ROOT, 'scripts/lib-parity/fixtures/render');

const fixtures = readdirSync(FIXTURE_DIR).filter(f => f.endsWith('.md')).sort();
if (fixtures.length === 0) {
  console.error('[render-switch] fixtures/render 无 .md 文件');
  process.exit(1);
}

// 在 web/ 内生成临时 ESM 入口（裸导入统一自 web/node_modules 解析——保证 vue 单实例），
// 执行后删除。
const TMP = join(ROOT, 'web/.render-switch-tmp.mjs');
writeFileSync(TMP, `
import { createSSRApp, h } from 'vue'
import { renderToString } from '@vue/server-renderer'
import { MarkdownRender, enableMermaid } from 'markstream-vue'

// 上游 StreamingRenderer 在 setup 里 new MutationObserver（客户端后处理）——
// SSR 无此全局,no-op polyfill（无 DOM 时观察器永不触发,行为等价于不挂载）。
globalThis.MutationObserver ??= class {
  observe() {} disconnect() {} unobserve() {} takeRecords() { return [] }
}

const { StreamingRenderer } = await import('@autodown/vue')

enableMermaid() // NEW 侧上游模块级已启用;OLD 侧显式对齐（能力等价前提）

const cases = JSON.parse(process.argv[2])

async function ssr(component, props) {
  const app = createSSRApp({ render: () => h(component, props) })
  return renderToString(app)
}

const out = []
for (const c of cases) {
  const oldHtml = await ssr(MarkdownRender, { content: c.content, final: true })
  const newHtml = await ssr(StreamingRenderer, { source: c.content, streaming: false })
  out.push({ name: c.name, oldHtml, newHtml })
}
console.log(JSON.stringify(out))
`);

let results;
try {
  const raw = execFileSync(process.execPath, [TMP, JSON.stringify(fixtures.map(f => ({
    name: f,
    content: readFileSync(join(FIXTURE_DIR, f), 'utf8'),
  })))], { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 });
  results = JSON.parse(raw);
} finally {
  rmSync(TMP, { force: true });
}

// ── 归一化器（W1/W2/W3）── 单遍标记器：drop 子树整体丢弃、unwrap 丢标签保内容、
//    其余只留标签名；文本仅在 drop 深度为 0 时保留。
const DROP_CLASS = new Set(['code-block-header', 'mermaid-block-header', 'autodown-block-placeholder']);
const UNWRAP_CLASS = new Set(['streaming-document']);
const VOID = new Set(['br', 'hr', 'img', 'input']);

function normalize(html) {
  html = html.replace(/<!--[\s\S]*?-->/g, '');
  const re = /<\/?[a-zA-Z][^>]*>|[^<]+/g;
  const tags = [];
  const texts = [];
  const stack = []; // {mode: 'drop'|'unwrap'|'keep', depth}
  let m;
  while ((m = re.exec(html)) !== null) {
    const tok = m[0];
    if (!tok.startsWith('<')) {
      if (!stack.some(s => s.mode === 'drop')) texts.push(tok);
      continue;
    }
    const isClose = tok.startsWith('</');
    const tagName = tok.replace(/^<\/?/, '').split(/[\s>/]/)[0];
    const selfClose = tok.endsWith('/>') || VOID.has(tagName);
    if (!isClose) {
      const cls = ((/class="([^"]*)"/.exec(tok) || [])[1] || '').split(/\s+/);
      if (stack.length === 0 && DROP_CLASS.has(cls.find(c => DROP_CLASS.has(c)))) {
        if (!selfClose) stack.push({ mode: 'drop', depth: 1 });
      } else if (stack.length === 0 && UNWRAP_CLASS.has(cls.find(c => UNWRAP_CLASS.has(c)))) {
        if (!selfClose) stack.push({ mode: 'unwrap', depth: 1 });
      } else {
        tags.push('<' + tagName + '>');
        if (!selfClose) stack.push({ mode: 'keep', depth: 1 });
      }
    } else {
      const top = stack[stack.length - 1];
      if (top) {
        top.depth -= 1;
        if (top.depth === 0) {
          stack.pop();
          if (top.mode === 'keep') tags.push('</' + tagName + '>');
        }
      }
    }
  }
  return tags.join('') + '\n@@TEXT@@\n' + texts.join('').replace(/\s+/g, ' ').trim();
}

let pass = 0;
const fails = [];
const w4 = [];
for (const r of results) {
  const a = normalize(r.oldHtml);
  const b = normalize(r.newHtml);
  if (a === b) { pass += 1; continue; }
  // W4 空内容边界：OLD 侧 markstream 空壳容器（单空 div、零文本）vs NEW 侧
  // segments 为空连内层 MarkdownRender 都不挂（超集语义的空态差异）
  const aTags = a.split('\n@@TEXT@@\n')[0];
  const aText = a.split('\n@@TEXT@@\n')[1] ?? '';
  const bTags = b.split('\n@@TEXT@@\n')[0];
  const bText = b.split('\n@@TEXT@@\n')[1] ?? '';
  if (aText === '' && bText === '' &&
      ((aTags === '<div></div>' && bTags === '') || (aTags === '' && bTags === '<div></div>'))) {
    pass += 1;
    w4.push(r.name);
    continue;
  }
  fails.push(r.name);
}

console.log(`render-switch parity: ${pass}/${results.length} fixtures 归一后全等`);
console.log('  白名单: W1 解包 .streaming-document / W2 丢弃 code-block-header·mermaid-block-header·placeholder 子树 / W3 去注释与属性' + (w4.length ? ` / W4 空内容空壳容器(${w4.join(',')})` : ''));
if (fails.length > 0) {
  console.error('白名单外差异 fixtures: ' + fails.join(', '));
  const first = results.find(r => r.name === fails[0]);
  if (first) {
    const fo = join(FIXTURE_DIR, '.fail-old.html');
    const fn = join(FIXTURE_DIR, '.fail-new.html');
    writeFileSync(fo, first.oldHtml);
    writeFileSync(fn, first.newHtml);
    console.error(`  首例现场已落盘: ${fo} / ${fn}`);
  }
  process.exit(1);
}
