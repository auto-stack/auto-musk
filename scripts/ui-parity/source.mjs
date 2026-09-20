import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';

export const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
export const GALLERY = path.join(ROOT, 'examples/musk-widgets-gallery');
export const DATA = path.join(ROOT, 'tests/ui-parity');
export const ARTIFACTS = path.join(ROOT, 'tmp/ui-parity');
export const slash = s => s.replaceAll('\\', '/');
export const hash = s => crypto.createHash('sha256').update(s).digest('hex');
export const readJson = p => JSON.parse(fs.readFileSync(p, 'utf8'));
export function writeJson(p, v) {
  fs.mkdirSync(path.dirname(p), { recursive: true });
  fs.writeFileSync(p, JSON.stringify(v, null, 2) + '\n');
}
export function files(dir) {
  if (!fs.existsSync(dir)) return [];
  return fs.readdirSync(dir, { withFileTypes: true }).flatMap(e => {
    const p = path.join(dir, e.name);
    if (fs.lstatSync(p).isSymbolicLink()) throw new Error(`Links forbidden: ${p}`);
    return e.isDirectory() ? files(p) : [p];
  });
}

// Remove comments without destroying strings, then find balanced widget bodies.
// This is an inventory parser, not a replacement Auto compiler. Unresolved references fail check.
export function uncomment(s) {
  return s.replace(/"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|\/\*[\s\S]*?\*\/|\/\/[^\n]*/g,
    x => x.startsWith('//') || x.startsWith('/*') ? x.replace(/[^\n]/g, ' ') : x);
}
export function widgets(file, source) {
  const s = uncomment(source), out = [];
  const re = /\bwidget\s+(\w+)\s*(?:\(([^)]*)\))?\s*\{/g;
  for (const m of s.matchAll(re)) {
    let i = m.index + m[0].length, depth = 1, quote = '';
    for (; i < s.length && depth; i++) {
      const ch = s[i];
      if (quote) { if (ch === '\\') i++; else if (ch === quote) quote = ''; }
      else if (ch === '"' || ch === "'") quote = ch;
      else if (ch === '{') depth++;
      else if (ch === '}') depth--;
    }
    if (depth) throw new Error(`Unbalanced widget ${file}:${m[1]}`);
    const props = (m[2] ?? '').split(',').filter(x => x.trim()).map(x => {
      const p = x.trim().match(/^(\w+)\s*:\s*(\S+)(?:\s*=\s*(.*))?$/);
      if (!p) throw new Error(`Unknown prop syntax ${file}: ${x}`);
      return { name: p[1], type: p[2], default: p[3] ?? null };
    });
    out.push({ id: m[1], source: slash(path.relative(ROOT, file)), props,
      body: s.slice(m.index + m[0].length, i - 1), offset: m.index });
  }
  return out;
}

export function inventory() {
  const units = files(path.join(ROOT, 'src/front')).filter(p => p.endsWith('.at'))
    .flatMap(p => widgets(p, fs.readFileSync(p, 'utf8')));
  const variantFiles = new Set(files(path.join(ROOT, 'src/front')).filter(p => p.endsWith('.at'))
    .map(p => slash(path.relative(ROOT, p))));
  const known = new Set(units.map(u => u.id));
  for (const u of units) {
    // Includes DSL function-style and brace-style instantiation, not bare imports.
    const code = u.body.replace(/"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'/g, '""');
    u.children = [...new Set([...code.matchAll(/\b([A-Z]\w+)\s*(?:\{|\()/g),
      ...code.matchAll(/^\s*([A-Z]\w+)\s*$/gm)]
      .map(m => m[1]).filter(n => known.has(n) && n !== u.id))];
  }
  const paths = new Map([['App', ['App']]]), queue = ['App'];
  while (queue.length) {
    const id = queue.shift();
    for (const u of units.filter(x => x.id === id)) for (const c of u.children) {
      if (!paths.has(c)) { paths.set(c, [...paths.get(id), c]); queue.push(c); }
    }
  }
  return units.map(({ body, offset, ...u }) => {
    const base = u.source.replace(/\.(?:vm|web)\.at$/, '.at');
    const vmPath = base.replace(/\.at$/, '.vm.at');
    const webPath = base.replace(/\.at$/, '.web.at');
    const hasVm = variantFiles.has(vmPath), hasWeb = variantFiles.has(webPath);
    const platforms = u.source.endsWith('.vm.at') ? (hasWeb ? ['vm', 'vue'] : ['vm'])
      : u.source.endsWith('.web.at') ? (hasVm ? ['vm', 'vue'] : ['vue']) : ['vm', 'vue'];
    return { ...u, platforms, reachable: paths.has(u.id),
      consumerPath: paths.get(u.id) ?? [],
      sourceHash: hash(fs.readFileSync(path.join(ROOT, u.source))) };
  });
}

export function portVariants() {
  const root = path.join(ROOT, 'src/front/ports');
  const groups = new Map();
  for (const p of files(root).filter(p => p.endsWith('.at'))) {
    const rel = slash(path.relative(ROOT, p));
    const m = rel.match(/^src\/front\/ports\/(.+)\.(vm|web)\.at$/);
    if (!m) continue;
    const key = m[1], g = groups.get(key) ?? { id: key, vm: null, vue: null };
    g[m[2] === 'vm' ? 'vm' : 'vue'] = rel; groups.set(key, g);
  }
  return [...groups.values()].map(g => ({ ...g, platforms: [g.vm && 'vm', g.vue && 'vue'].filter(Boolean) }));
}

export function triageFor(u) {
  if (!u.reachable) return { owner: 'retired-review', plan: 'retired-review', reason: 'Not reachable from App' };
  if (['Markdown', 'MarkdownEditor', 'RawPreview', 'AutoDownEditor'].includes(u.id)) {
    return { owner: 'PLAN-076', plan: 'PLAN-076', reason: 'AutoDown/Markdown engine parity' };
  }
  if (['ChatMessage', 'UserMessage', 'AgentAvatar', 'ErrandCard', 'RelayRunBox', 'TaskPlanCard', 'ToolGateCard', 'QuestionnaireCard', 'SecretaryMessage', 'SecretaryMessageWrapper', 'GateCard', 'ReportCard'].includes(u.id)) {
    return { owner: 'PLAN-077', plan: 'PLAN-077', reason: 'Message block and interaction parity' };
  }
  if (u.id === 'App') return { owner: 'PLAN-079', plan: 'PLAN-079', reason: 'App-level parity and release gate' };
  return { owner: 'PLAN-078', plan: 'PLAN-078', reason: 'Shell/composition parity' };
}

export function checkCatalog(catalog) {
  const actual = inventory(), issues = [];
  const declaredUnits = catalog.unitPolicy === 'auto-reachable'
    ? actual.map(u => ({ ...u, bucket: u.reachable ? (u.platforms.length === 1 ? `${u.platforms[0]}-only` : 'both') : 'unreachable',
        ...triageFor(u) }))
    : catalog.units;
  const classified = declaredUnits ?? [];
  for (const u of actual) {
    const c = classified.find(c => c.id === u.id && c.source === u.source);
    if (!c) { issues.push(`Unclassified declaration: ${u.source}:${u.id}`); continue; }
    if (c.sourceHash !== u.sourceHash) issues.push(`Stale source: ${u.id}`);
    if (JSON.stringify(c.consumerPath) !== JSON.stringify(u.consumerPath)) issues.push(`Stale consumer path: ${u.id}`);
    if (!['both', 'vue-only', 'vm-only', 'unreachable'].includes(c.bucket)) issues.push(`Invalid bucket: ${u.id}`);
    if (u.reachable && c.bucket === 'unreachable') issues.push(`Reachable incorrectly retired: ${u.id}`);
    if (u.reachable && !effectiveCases(catalog).some(x => x.unit === u.id)) issues.push(`Missing initial case: ${u.id}`);
    if (!c.owner || !c.plan || !c.reason) issues.push(`Missing triage owner: ${u.id}`);
  }
  for (const c of classified) if (!actual.some(u => u.id === c.id && u.source === c.source)) issues.push(`Removed declaration: ${c.id}`);
  const ids = catalog.cases.map(c => c.id);
  if (new Set(ids).size !== ids.length) issues.push('Duplicate case IDs');
  for (const id of ['thinking', 'text', 'tool']) if (!catalog.cases.some(c => c.logicalUnit === id)) issues.push(`Missing inline unit: ${id}`);
  const explicit = catalog.cases ?? [];
  const generated = catalog.casePolicy === 'one-per-reachable-plus-inline'
    ? actual.filter(u => u.reachable && !explicit.some(c => c.unit === u.id)).map(u => ({
        id: `inventory-${u.id}`, unit: u.id, logicalUnit: 'component', fixture: 'empty.json',
        mode: 'inventory-only', state: 'initial', ...triageFor(u)
      })) : [];
  const allCases = [...explicit, ...generated];
  for (const c of allCases) {
    if (!c.owner || !c.plan || !c.fixture || !c.state) issues.push(`Incomplete case: ${c.id}`);
    if (!classified.some(u => u.id === c.unit)) issues.push(`Unknown unit: ${c.id}`);
    if (!fs.existsSync(path.join(DATA, 'fixtures', c.fixture))) issues.push(`Missing fixture: ${c.id}`);
  }
  return issues;
}

export function effectiveCases(catalog) {
  const actual = inventory(), explicit = catalog.cases ?? [];
  const generated = catalog.casePolicy === 'one-per-reachable-plus-inline'
    ? actual.filter(u => u.reachable && !explicit.some(c => c.unit === u.id)).map(u => ({
        id: `inventory-${u.id}`, unit: u.id, logicalUnit: 'component', fixture: 'empty.json',
        mode: 'inventory-only', state: 'initial', ...triageFor(u)
      })) : [];
  return [...explicit, ...generated];
}
