#!/usr/bin/env node
// gen-icons.at-data.mjs — PLAN-038 T8：从 web/node_modules/lucide-vue-next dist
// 提取 musk 使用面图标集（ports 37 符号 ∪ web/src 直引差集）的 SVG path 数据，
// 生成 src/front/lib/icons_data.at（纯数据 fn 模块，零 use.web，整文件生成物）。
//
// 规范化（D3）：lucide 默认属性 stroke="currentColor" / stroke-width="2" /
// viewBox="0 0 24 24" / fill="none" / stroke-linecap="round" / stroke-linejoin="round"
// 提为渲染层默认值——数据层只存每图标的元素序列（tag + 原生属性，剔除 vue 渲染
// 提示 key）。别名（lucide 改名史）经 ALIASES 显式映射。
//
// 清单核对：脚本自证——ports/icons.web.at 与 web/src 的实际引用若与固化清单
// 不一致立即报错（防漂移）。
//
// 再生：node scripts/gen-icons.at-data.mjs（幂等，无时间戳）

import { readFileSync, writeFileSync, readdirSync } from 'node:fs';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';
import { join, dirname } from 'node:path';

const require = createRequire(import.meta.url);

const ROOT = fileURLToPath(new URL('../', import.meta.url));
const DIST_ICONS = join(ROOT, 'web/node_modules/lucide-vue-next/dist/esm/icons');
const TARGET = join(ROOT, 'src/front/lib/icons_data.at');

// ── 固化清单 ─────────────────────────────────────────────────────────────────
// 来源 1：src/front/ports/icons.web.at 的 37 符号（.at 轨经端口消费的 lucide 面）
const PORTS_SYMBOLS = [
  'BookOpen', 'Check', 'ChevronDown', 'ChevronRight', 'ChevronUp', 'Clock', 'Copy',
  'CopyCheck', 'Download', 'ExternalLink', 'Eye', 'File', 'FileIcon', 'FileText',
  'Folder', 'FolderInput', 'FolderOpen', 'FolderPlus', 'HelpCircle', 'Info',
  'ListTodo', 'MessageSquare', 'Monitor', 'Moon', 'Orbit', 'PanelLeft', 'Plus',
  'Scroll', 'Search', 'Send', 'Settings', 'Square', 'Sun', 'Terminal', 'Trash2',
  'UploadCloud', 'Wrench', 'X',
];
// 来源 2：web/src 直引 lucide-vue-next 的符号（web 轨差集补充；含多行 import）
const WEB_SYMBOLS = [
  'ArrowDown', 'ArrowUp', 'BookOpen', 'Check', 'ChevronDown', 'ChevronRight',
  'ChevronUp', 'Clipboard', 'Clock', 'Copy', 'CopyCheck', 'Download',
  'ExternalLink', 'Eye', 'File', 'FileCode', 'FileText', 'Flame', 'Folder',
  'FolderInput', 'FolderOpen', 'FolderPlus', 'HelpCircle', 'Image', 'Inbox',
  'Info', 'Link2', 'ListTodo', 'Loader2', 'MessageSquare', 'Monitor', 'Moon',
  'Orbit', 'PanelLeft', 'Pencil', 'Play', 'Plus', 'RefreshCw', 'Scroll',
  'Search', 'Send', 'Settings', 'Sun', 'TableProperties', 'Terminal', 'Trash2',
  'Unlink', 'UploadCloud', 'Wrench', 'X',
];
// lucide 改名/别名史（导出名 → dist 图标文件 stem）
const ALIASES = {
  FileIcon: 'file',          // File 的 Icon 后缀别名
  HelpCircle: 'circle-help', // 更名：HelpCircle → CircleHelp
  Loader2: 'loader-circle',  // 更名：Loader2 → LoaderCircle
  Unlink: 'unlink-2',        // Unlink（如 dist 命名不符再调整）
  UploadCloud: 'upload',     // 更名：UploadCloud → Upload
  CopyCheck: 'copy-check',
};

// ── 清单核对（防清单与实际引用漂移）────────────────────────────────────────
function extractSymbols(text) {
  return [...text.matchAll(/import\s*\{([^}]+)\}\s*from\s*['"][^'"]*lucide-vue-next['"]/g)]
    .flatMap(m => m[1].split(',').map(s => s.trim()).filter(Boolean));
}
const portsFile = readFileSync(join(ROOT, 'src/front/ports/icons.web.at'), 'utf8');
const portsLine = portsFile.match(/use\.web component ([^"]+) from "lucide-vue-next"/);
const portsActual = portsLine ? portsLine[1].split(',').map(s => s.trim()).filter(Boolean) : [];
if (portsActual.join(',') !== PORTS_SYMBOLS.join(',')) {
  console.error(`[gen-icons] ports 清单漂移：\n  固化: ${PORTS_SYMBOLS.join(',')}\n  实际: ${portsActual.join(',')}`);
  process.exit(1);
}
// web/src 直引扫描
const webDir = join(ROOT, 'web/src');
function* walk(dir) {
  for (const e of readdirSync(dir, { withFileTypes: true })) {
    const p = join(dir, e.name);
    if (e.isDirectory()) yield* walk(p);
    else if (/\.(vue|ts)$/.test(e.name)) yield p;
  }
}
const webActual = new Set();
for (const f of walk(webDir)) {
  for (const s of extractSymbols(readFileSync(f, 'utf8'))) webActual.add(s);
}
const webFixed = new Set(WEB_SYMBOLS);
const webMissing = [...webActual].filter(s => !webFixed.has(s));
const webExtra = [...webFixed].filter(s => !webActual.has(s));
if (webMissing.length || webExtra.length) {
  console.error(`[gen-icons] web 清单漂移：缺 ${webMissing.join(',')} ; 多 ${webExtra.join(',')}`);
  process.exit(1);
}

// ── 数据提取 ─────────────────────────────────────────────────────────────────
function kebab(name) {
  return name.replace(/([a-z0-9])([A-Z])/g, '$1-$2').replace(/([a-zA-Z])(\d)/g, '$1-$2').toLowerCase();
}
const ALL = [...new Set([...PORTS_SYMBOLS, ...WEB_SYMBOLS])].sort();

const data = {};
const unresolved = [];
for (const name of ALL) {
  const stem = ALIASES[name] ?? kebab(name);
  let file = join(DIST_ICONS, stem + '.js');
  // 未命中且名带 Icon 后缀 → 去后缀重试（别名约定）
  try { readFileSync(file); } catch {
    if (name.endsWith('Icon')) {
      const alt = kebab(name.replace(/Icon$/, ''));
      file = join(DIST_ICONS, alt + '.js');
      try { readFileSync(file); } catch { unresolved.push(name); continue; }
    } else { unresolved.push(name); continue; }
  }
  const src = readFileSync(file, 'utf8');
  const m = src.match(/createLucideIcon\("[^"]+",\s*(\[[\s\S]*\])\);/);
  if (!m) { unresolved.push(name); continue; }
  // dist 数组已是 JS 字面量（受信静态内容）：["path", { d: "...", key: "..." }] 形式
  const elements = new Function(`return ${m[1]}`)();
  data[name] = elements.map(([tag, attrs]) => ({
    tag,
    attrs: Object.fromEntries(Object.entries(attrs).filter(([k]) => k !== 'key')),
  }));
}
if (unresolved.length) {
  console.error(`[gen-icons] 无法解析的图标: ${unresolved.join(', ')}（补 ALIASES 映射后重跑）`);
  process.exit(1);
}

// ── 生成 .at 数据文件 ────────────────────────────────────────────────────────
function atLiteral(value, indent) {
  const pad = ' '.repeat(indent);
  const padInner = ' '.repeat(indent + 4);
  if (typeof value === 'string') return JSON.stringify(value);
  if (Array.isArray(value)) {
    if (value.length === 0) return '[]';
    const items = value.map(v => `${padInner}${atLiteral(v, indent + 4)}`);
    return `[\n${items.join(',\n')}\n${pad}]`;
  }
  const entries = Object.entries(value).map(([k, v]) => `${padInner}${JSON.stringify(k)}: ${atLiteral(v, indent + 4)}`);
  return entries.length === 0 ? '{}' : `{\n${entries.join(',\n')}\n${pad}}`;
}

const header = `// icons_data.at — auto-icons 数据层（生成物勿手改）
// 生成器：scripts/gen-icons.at-data.mjs（源：web/node_modules/lucide-vue-next dist，
// 版本 ${require(join(ROOT, 'web/node_modules/lucide-vue-next/package.json')).version}）
// 清单：ports/icons.web.at ${PORTS_SYMBOLS.length} 符号 ∪ web/src 直引差集，共 ${ALL.length} 图标。
// 规范化：lucide 默认属性（viewBox "0 0 24 24" / fill "none" / stroke "currentColor" /
// stroke-width "2" / stroke-linecap "round" / stroke-linejoin "round"）提为渲染层默认值，
// 数据层只存元素序列（tag + 原生属性；vue 渲染提示 key 已剔除）。
// PLAN-038 T8。渲染层（svg widget）降级登记见 KNOWN-DEBT（.at UI 暂不支持 svg 节点，
// T9 canary 实证）。再生幂等：node scripts/gen-icons.at-data.mjs`;

const out = `${header}

fn icons_data() map {
    return ${atLiteral(data, 4)}
}
`;
writeFileSync(TARGET, out);
console.log(`icons_data.at: ${ALL.length} icons -> ${TARGET}`);
for (const n of ALL) {
  if (!out.includes(`"${n}"`)) { console.error(`  !! ${n} 未命中`); process.exit(1); }
}
console.log(`symbol grep 核对: ${ALL.length}/${ALL.length} 命中`);
