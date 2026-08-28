// run.mjs — PLAN-049 style-parity 双轨解释对拍门禁
//
// 单一样式源 = .at 源码内的 tailwind 工具类;同一类串双轨解释逐属性 diff:
//   web 侧: 读 gen/front/vue/dist/assets/*.css(tailwind 生成规则),对用例类串
//           逐 token 静态匹配规则、展开为属性表(确定性,无浏览器)。
//   VM  侧: cargo test -p auto-lang --lib --features ui-iced style_parity_dump
//           -- --nocapture 抓 JSON 行(class.rs 解析输出,case/token/属性表)。
//   diff:   属性值归一化(rem→px、hsl(var(--x)[/a])→color(x@a)、flex 简写表)
//           后逐属性对比;web-only 增强(box-shadow/transition/user-select 等
//           norm.webOnlyProps)报告计数不判失败;0 diff 为绿。
//
// 用法: node scripts/lib-parity/style-parity/run.mjs
// 前置: gen/front/vue/dist 存在(auto build / pnpm build 产物);
//       auto-lang 可构建(默认 sibling ../auto-lang,env STYLE_PARITY_LANG_ROOT 覆盖)。
// 退出码: diff=0 → 0;否则 1。

import { spawnSync } from 'node:child_process';
import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, '../../..'); // auto-musk root（style-parity → lib-parity → scripts → 根）
const norm = JSON.parse(readFileSync(join(here, 'norm.json'), 'utf8'));
const casesDoc = JSON.parse(readFileSync(join(here, 'fixtures/cases.json'), 'utf8'));
const cases = casesDoc.cases ?? [];

// ── web 侧:dist CSS → token 规则表 ───────────────────────────────────────────

function loadDistCss() {
  const dir = join(root, 'gen/front/vue/dist/assets');
  if (!existsSync(dir)) return null;
  return readdirSync(dir)
    .filter((f) => f.endsWith('.css'))
    .map((f) => readFileSync(join(dir, f), 'utf8'))
    .join('\n');
}

// selector 去转义(.bg-primary\/10:hover → .bg-primary/10:hover)
function unescapeSel(sel) {
  return sel.replace(/\\([/:.[\]%()#])/g, '$1').trim();
}

// 从 selector 提取可与 token 对比的规范名:取最后一个复合段,剥尾部状态伪类/
// 伪元素(.hover\:bg-accent:hover → hover:bg-accent;.search-input::placeholder → search-input)
function selToName(sel) {
  const un = unescapeSel(sel);
  const lastCompound = un.split(/\s+>+|\s+/).pop();
  let name = lastCompound.replace(/::+[a-z-]+$/i, '');
  name = name.replace(/:(hover|focus|active|disabled|focus-within|focus-visible|visited|placeholder)$/i, '');
  return name.replace(/^\./, '');
}

function parseRules(css) {
  const rules = new Map(); // name → Map(prop→rawValue)
  const re = /([^{}]+)\{([^{}]*)\}/g;
  let m;
  while ((m = re.exec(css))) {
    const decls = m[2];
    if (!decls.includes(':')) continue;
    for (const selPart of m[1].split(',')) {
      const sel = selPart.trim();
      if (!sel.startsWith('.') && !sel.includes('\\')) continue;
      if (!sel.startsWith('.')) continue;
      const name = selToName(sel);
      if (!name) continue;
      if (rules.has(name)) continue; // 首个规则为准
      const props = new Map();
      for (const d of decls.split(';')) {
        const i = d.indexOf(':');
        if (i < 0) continue;
        props.set(d.slice(0, i).trim().toLowerCase(), d.slice(i + 1).trim());
      }
      rules.set(name, props);
    }
  }
  return rules;
}

// web 值归一 → VM 记法
const LEN_PROPS = /^(padding|margin|gap|row-gap|column-gap|top|right|bottom|left|width|min-|max-|border-radius|border-width|height|inset)/;

function normWebValue(prop, val) {
  let v = val;
  // shadcn radius 体系:borderRadius 档位挂在 --radius(rem) 的 calc 上
  if (norm.radiusVarPx && /calc\(var\(--radius\)\s*[+-]?\s*[\d.]+px\)|^var\(--radius\)$/.test(v)) {
    const m = v.match(/calc\(var\(--radius\)\s*([+-])\s*([\d.]+)px\)/);
    if (m) {
      const delta = m[1] === '-' ? -parseFloat(m[2]) : parseFloat(m[2]);
      v = `${fmtNum(norm.radiusVarPx + delta)}px`;
    } else {
      v = `${fmtNum(norm.radiusVarPx)}px`;
    }
  }
  // 长度族零值:tailwind 写 0,VM 记 0px
  if (LEN_PROPS.test(prop) && (v === '0' || v === '0px')) v = '0px';
  // 字面透明色:VM 记 color(transparent@1)
  if (v === 'transparent') v = 'color(transparent@1)';
  // rem → px(逐 token 内联)
  v = v.replace(/(-?[\d.]+)rem/g, (_, n) => `${fmtNum(parseFloat(n) * norm.remPx)}px`);
  // hsl(var(--x)) / hsl(var(--x) / .1)
  const hv = v.match(/^hsl\(var\(--([\w-]+)\)\s*(?:\/\s*([\d.]+))?\)$/);
  if (hv) return `color(${hv[1]}@${fmtNum(hv[2] ? parseFloat(hv[2]) : 1)})`;
  // .6 → 0.6
  if (/^-?\.\d+$/.test(v)) v = v.replace(/^(-?)\./, '$10.');
  if (prop === 'flex' && norm.flexShorthand[v]) v = norm.flexShorthand[v];
  return v;
}

function fmtNum(n) {
  const s = String(n);
  return s;
}

// ── VM 侧:cargo dump → case JSON ────────────────────────────────────────────

function runVmDump() {
  // 布局自适应:主检出 sibling ../auto-lang;worktree 布局 ../../auto-lang;
  // env STYLE_PARITY_LANG_ROOT 最优先。
  const langRoot =
    process.env.STYLE_PARITY_LANG_ROOT ||
    [resolve(root, '..', 'auto-lang'), resolve(root, '..', '..', '..', 'auto-lang')]
      .find((p) => existsSync(join(p, 'crates', 'auto-lang')));
  if (!langRoot) {
    console.error('[style-parity] auto-lang not found (env STYLE_PARITY_LANG_ROOT 可指定)');
    process.exit(2);
  }
  const env = { ...process.env, STYLE_PARITY_CASES: join(here, 'fixtures/cases.json') };
  const r = spawnSync(
    'cargo',
    ['test', '-p', 'auto-lang', '--lib', '--features', 'ui-iced', 'style_parity_dump', '--', '--nocapture'],
    { cwd: langRoot, encoding: 'utf8', shell: true, env, maxBuffer: 64 * 1024 * 1024 },
  );
  if (r.stderr) process.stderr.write(r.stderr.split('\n').filter(l => !l.includes('warning')).join('\n'));
  const out = String(r.stdout ?? '');
  const lines = out.split('\n').filter((l) => l.startsWith('[style-parity-dump] '));
  const payload = lines.map((l) => l.slice('[style-parity-dump] '.length));
  const begin = payload.findIndex((s) => s.startsWith('BEGIN'));
  const end = payload.findIndex((s) => s.startsWith('END'));
  if (begin < 0 || end < 0 || r.status !== 0) {
    console.error('[style-parity] FAIL: no dump payload (auto-lang master 早于 plan-049? 设 STYLE_PARITY_LANG_ROOT 指向 auto-musk-dev worktree;或 cargo 构建失败,见上方输出)');
    process.exit(2);
  }
  return payload.slice(begin + 1, end).map((s) => JSON.parse(s));
}

// ── diff ────────────────────────────────────────────────────────────────────

function isVariantToken(t) {
  return /^(hover|focus|active|disabled|placeholder|dark|group|peer|sm|md|lg|xl|2xl):/.test(t);
}

function isWhitelisted(t) {
  const wl = norm.tokenWhitelist ?? { names: [], prefixes: [] };
  if ((wl.names ?? []).includes(t)) return true;
  return (wl.prefixes ?? []).some((p) => t.startsWith(p));
}

function webOnlyProp(prop) {
  if (norm.webOnlyProps && Object.prototype.hasOwnProperty.call(norm.webOnlyProps, prop)) return true;
  return false;
}

const report = {
  cases: 0,
  tokens: 0,
  webOnlyEnhanced: [], // token(web-only 增强,白名单丢弃/降级)
  webOnlyProps: {}, // prop → count
  whitelistedGaps: [], // token(VM 未支持且登记白名单)
  diffs: [], // {case, token, kind, detail}
};

function diffCase(caseEntry, vmEntry, rules) {
  const caseId = caseEntry.id;
  const vmTokens = new Map((vmEntry?.tokens ?? []).map((t) => [t.raw, t]));
  for (const raw of caseEntry.classes.split(/\s+/).filter(Boolean)) {
    report.tokens++;
    if (isVariantToken(raw)) {
      // web-only 增强:确认 web 侧规则存在(生成链健康),不比属性
      if (!rules.has(raw)) {
        report.diffs.push({ case: caseId, token: raw, kind: 'web-variant-rule-missing', detail: 'tailwind 未生成该变体规则(content 扫描缺失?)' });
      } else {
        report.webOnlyEnhanced.push(`${caseId}/${raw}`);
      }
      continue;
    }
    const vm = vmTokens.get(raw);
    if (!vm) {
      report.diffs.push({ case: caseId, token: raw, kind: 'vm-dump-missing', detail: 'dump 未覆盖该 token' });
      continue;
    }
    if (!vm.ok) {
      if (isWhitelisted(raw)) {
        report.whitelistedGaps.push(`${caseId}/${raw}`);
      } else {
        report.diffs.push({ case: caseId, token: raw, kind: 'vm-parse-gap', detail: vm.err ?? 'class.rs 解析失败(D3 候选或草案避用)' });
      }
      continue;
    }
    const webProps = rules.get(raw);
    if (!webProps) {
      report.diffs.push({ case: caseId, token: raw, kind: 'web-rule-missing', detail: 'tailwind 生成 CSS 无此规则(类串笔误或 content 扫描缺失)' });
      continue;
    }
    // VM 属性表:去掉 _ 前缀报告键
    const vmProps = new Map(Object.entries(vm.props ?? {}).filter(([k]) => !k.startsWith('_')));
    for (const [prop, webValRaw] of webProps) {
      if (prop.startsWith('--tw-') || (norm.dropPropPrefixes && Object.keys(norm.dropPropPrefixes).some((p) => prop.startsWith(p)))) continue;
      if (vmProps.has(prop)) {
        let webVal = normWebValue(prop, webValRaw);
        const vmVal = String(vmProps.get(prop));
        // VM 布局降级等价（Plan 412 降级矩阵）:web 值按 norm.valueDegrades 折算后再比
        const deg = norm.valueDegrades?.[prop]?.[webVal];
        if (deg !== undefined) webVal = deg;
        if (webVal !== vmVal) {
          report.diffs.push({ case: caseId, token: raw, kind: 'value-mismatch', detail: `${prop}: web=${webVal} vm=${vmVal}` });
        }
      } else if (webOnlyProp(prop)) {
        report.webOnlyProps[prop] = (report.webOnlyProps[prop] ?? 0) + 1;
      } else if (norm.dropProps && Object.prototype.hasOwnProperty.call(norm.dropProps, prop)) {
        // 双方约定不比(如 text-* 附带行高)
      } else {
        report.diffs.push({ case: caseId, token: raw, kind: 'missing-in-vm', detail: `web 有 ${prop}=${webValRaw}, VM 无(映射缺口或白名单漏登记)` });
      }
    }
    for (const [prop, vmVal] of vmProps) {
      if (!webProps.has(prop)) {
        report.diffs.push({ case: caseId, token: raw, kind: 'missing-in-web', detail: `vm 有 ${prop}=${vmVal}, web 无(类串/norm 错位)` });
      }
    }
  }
}

// ── main ────────────────────────────────────────────────────────────────────

const css = loadDistCss();
if (!css && cases.length > 0) {
  console.error('[style-parity] FAIL: gen/front/vue/dist/assets 不存在——先 auto build / pnpm build(web 侧规则源)');
  process.exit(2);
}
const rules = css ? parseRules(css) : new Map();
const vmCases = runVmDump();
const vmById = new Map(vmCases.map((c) => [c.case, c]));

for (const c of cases) {
  report.cases++;
  diffCase(c, vmById.get(c.id), rules);
}

// ── report ──────────────────────────────────────────────────────────────────
console.log('[style-parity] ═══ style-parity 对拍报告 ═══');
console.log(`[style-parity] cases=${report.cases} tokens=${report.tokens}`);
console.log(`[style-parity] web-only 增强 token: ${report.webOnlyEnhanced.length}`);
if (report.webOnlyEnhanced.length) console.log('  ' + report.webOnlyEnhanced.join(', '));
console.log(`[style-parity] 白名单 VM 缺口 token: ${report.whitelistedGaps.length}`);
if (report.whitelistedGaps.length) console.log('  ' + report.whitelistedGaps.join(', '));
const wProps = Object.entries(report.webOnlyProps);
console.log(`[style-parity] web-only 增强属性: ${wProps.map(([p, n]) => `${p}×${n}`).join(', ') || '(无)'}`);
if (report.diffs.length) {
  console.log(`[style-parity] ═══ DIFF ${report.diffs.length} 条(非白名单,判失败)═══`);
  for (const d of report.diffs) {
    console.log(`  [${d.kind}] ${d.case} :: ${d.token} — ${d.detail}`);
  }
  process.exit(1);
}
console.log('[style-parity] PASS — diff=0(白名单外)');
