#!/usr/bin/env node
// highlight-compare.mjs — PLAN-038 T15：prismjs / lowlight(@autodown/vue 内置,
// common 集) / syntect(two-face,auto-lang code_editor 同内核) 三方案在 musk
// 实际语言集上的 token/scopes 一致性矩阵（决策数据,不设全等断言）。
//
// 方法：三引擎对同一 fixtures 代码（scripts/lib-parity/fixtures/highlight/code/
// <lang>.txt）各产出「逐字符 token 类别流」（文本锚定,规避分词差异）,经
// scope→token 近似映射表（CATEGORY_MAP,登记于报告）归到统一类别集
// （comment/string/keyword/number/function/type/operator/punctuation/property/
// constant/variable/builtin/plain）,两两逐字符比对得一致率矩阵。
// 输出：scripts/lib-parity/fixtures/highlight/report.md + matrix.json。

import { readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { join } from 'node:path';
import { execFileSync } from 'node:child_process';

const ROOT = fileURLToPath(new URL('../../', import.meta.url));
const CODE_DIR = join(ROOT, 'scripts/lib-parity/fixtures/highlight/code');
const OUT_DIR = join(ROOT, 'scripts/lib-parity/fixtures/highlight');

const langs = readdirSync(CODE_DIR).filter(f => f.endsWith('.txt')).map(f => f.replace(/\.txt$/, '')).sort();

// 1) syntect 侧（cargo 工程产物）
execFileSync(join(ROOT, 'scripts/highlight-rs/target/release/highlight-rs.exe'),
  [CODE_DIR, join(OUT_DIR, 'syntect.json')], { stdio: 'inherit' });
const syntect = JSON.parse(readFileSync(join(OUT_DIR, 'syntect.json'), 'utf8')).languages;

// 2) prism + lowlight 侧：web/ 内临时 ESM 入口（裸导入统一解析）
const TMP = join(ROOT, 'web/.highlight-compare-tmp.mjs');
writeFileSync(TMP, `
import Prism from 'prismjs'
import 'prismjs/components/prism-typescript.js'
import 'prismjs/components/prism-javascript.js'
import 'prismjs/components/prism-json.js'
import 'prismjs/components/prism-bash.js'
import 'prismjs/components/prism-python.js'
import 'prismjs/components/prism-markdown.js'
import 'prismjs/components/prism-yaml.js'
import 'prismjs/components/prism-toml.js'
import 'prismjs/components/prism-sql.js'
import 'prismjs/components/prism-java.js'
import 'prismjs/components/prism-c.js'
import 'prismjs/components/prism-cpp.js'
import 'prismjs/components/prism-go.js'
import { createLowlight, common } from 'lowlight'

// 与 @autodown/vue 相同的 lowlight 装配（common 集）
const lowlight = createLowlight(common)

const cases = JSON.parse(process.argv[2])
const out = []
for (const c of cases) {
  // prism：token 树 → 逐字符类别
  const stream = new Array(c.code.length).fill('plain')
  const grammar = Prism.languages[c.lang] ?? Prism.languages.plain
  ;(function walk(tokens, offset) {
    for (const t of tokens) {
      if (typeof t === 'string') { offset += t.length; continue }
      const type = Array.isArray(t.alias) ? t.alias[0] : t.alias || t.type
      if (typeof t.content === 'string') {
        for (let i = 0; i < t.content.length; i++) stream[offset + i] = String(type)
        offset += t.content.length
      } else if (Array.isArray(t.content)) {
        walk(t.content, offset)
        offset += (function len(ts) { let n = 0; for (const x of ts) n += typeof x === 'string' ? x.length : len(x.content); return n })(t.content)
      }
    }
  })(Prism.tokenize(c.code, grammar), 0)

  // lowlight：hast 树 → 逐字符类别（内层 className 优先;未注册语言输出 null）
  let hlStream = null
  if (lowlight.registered(c.lang)) {
    hlStream = new Array(c.code.length).fill('plain')
    const tree = lowlight.highlight(c.lang, c.code)
    ;(function walkHast(node, offset, cls) {
      for (const child of node.children ?? []) {
        if (child.type === 'text') {
          for (let i = 0; i < child.value.length; i++) {
            if (cls) hlStream[offset + i] = cls
          }
          offset += child.value.length
        } else if (child.type === 'element') {
          const inner = (child.properties?.className?.[1]) ?? (child.properties?.className?.[0]) ?? cls
          offset = walkHast(child, offset, inner)
        }
      }
      return offset
    })(tree, 0, null)
  }

  out.push({ lang: c.lang, prism: stream, lowlight: hlStream })
}
console.log(JSON.stringify(out))
`);

let prismLow;
try {
  const raw = execFileSync(process.execPath, [TMP, JSON.stringify(langs.map(l => ({
    lang: l,
    code: readFileSync(join(CODE_DIR, l + '.txt'), 'utf8'),
  })))], { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 });
  prismLow = JSON.parse(raw);
} finally {
  const { rmSync } = await import('node:fs');
  rmSync(TMP, { force: true });
}

// 3) syntect html → 逐字符类别（span 栈,内层首个 class = scope 根原子）
function syntectStream(html) {
  // 行结构:<span class="...">text</span> 嵌套。逐 token 解析;文本先做 HTML 实体
  // 解码（&amp;/&lt;/&gt;/&quot;/&#39;——生成器转义了文本,不解码会虚增长度）。
  const decode = s => s
    .replace(/&lt;/g, '<').replace(/&gt;/g, '>')
    .replace(/&quot;/g, '"').replace(/&#39;/g, "'")
    .replace(/&amp;/g, '&');
  const code = [];
  const stack = [];
  const re = /<span class="([^"]*)">|<\/span>|[^<]+|<[^>]+>/g;
  let m;
  while ((m = re.exec(html)) !== null) {
    if (m[1] !== undefined) {
      stack.push(m[1].split(/\s+/));
    } else if (m[0] === '</span>') {
      stack.pop();
    } else if (m[0].startsWith('<')) {
      // 其他标签(br 等)按零宽处理
    } else {
      const text = decode(m[0]);
      const cls = stack.length ? stack[stack.length - 1][0] : 'plain';
      for (let i = 0; i < text.length; i++) code.push(cls);
    }
  }
  return code;
}

// 4) 统一类别映射（近似,决策粒度足够;登记于报告）
const CATEGORY_MAP = {
  // prism token type → 类别
  prism: {
    comment: 'comment', prolog: 'comment', cdata: 'comment',
    string: 'string', char: 'string', regex: 'string', 'string-interpolation': 'string',
    keyword: 'keyword', 'keyword-type': 'keyword', atrule: 'keyword', important: 'keyword',
    number: 'number', boolean: 'constant',
    function: 'function', 'function-variable': 'variable',
    'class-name': 'type', tag: 'type',
    operator: 'operator',
    punctuation: 'punctuation',
    'attr-name': 'property', property: 'property', attrvalue: 'string',
    constant: 'constant', symbol: 'constant',
    variable: 'variable', 'template-variable': 'variable',
    builtin: 'builtin', selector: 'builtin', entity: 'builtin', url: 'string',
    'attr-value': 'string',
  },
  // hljs class(去 hljs- 前缀) → 类别
  lowlight: {
    comment: 'comment', quote: 'comment',
    string: 'string', subst: 'string', regexp: 'string',
    keyword: 'keyword', meta: 'keyword', 'meta-keyword': 'keyword', doctag: 'comment',
    number: 'number', literal: 'constant', symbol: 'constant',
    'title-function': 'function', 'title-function-invocation': 'function',
    'title-class': 'type', type: 'type', 'title-class-inherited': 'type', tag: 'type',
    operator: 'operator',
    punctuation: 'punctuation',
    attr: 'property', property: 'property', 'params-arguments': 'variable',
    variable: 'variable', 'variable-language': 'variable', 'variable-constant': 'constant',
    'built_in': 'builtin', 'selector-tag': 'builtin', name: 'function', title: 'function',
    section: 'type', bullet: 'operator',
  },
  // syntect scope 根原子 → 类别
  syntect: {
    comment: 'comment',
    string: 'string', constant_character_escape: 'string',
    keyword: 'keyword', storage_modifier: 'keyword', storage_type: 'type', meta_preprocessor: 'keyword',
    constant_numeric: 'number', constant_language: 'constant', constant_character: 'string',
    entity_name_function: 'function', support_function: 'function',
    entity_name_type: 'type', entity_name_class: 'type', entity_name_struct: 'type', entity_name_enum: 'type', support_type: 'type', support_class: 'type',
    keyword_operator: 'operator',
    punctuation: 'punctuation', punctuation_accessor: 'punctuation', punctuation_definition: 'punctuation', punctuation_separator: 'punctuation', punctuation_terminator: 'punctuation',
    variable_other_member: 'property', meta_property_name: 'property', string_quoted_other: 'string',
    variable: 'variable', variable_parameter: 'variable', variable_function: 'variable', variable_language: 'builtin',
    support_function_builtin: 'builtin', constant_other_symbol: 'constant',
    entity_name_tag: 'type', meta_tag: 'type',
  },
};

function categorize(engine, raw) {
  if (raw == null || raw === 'plain') return 'plain';
  const key = String(raw).replace(/^hljs-/, '');
  return CATEGORY_MAP[engine][key] ?? CATEGORY_MAP[engine][raw] ?? 'plain';
}

// 5) 矩阵
const matrix = {};
for (const pl of prismLow) {
  const lang = pl.lang;
  const p = pl.prism.map(c => categorize('prism', c));
  const l = pl.lowlight ? pl.lowlight.map(c => categorize('lowlight', c)) : null;
  const sHtml = syntect[lang]?.html;
  const s = sHtml ? syntectStream(sHtml).map(c => categorize('syntect', c)) : null;

  const entry = {
    prism_support: true,
    lowlight_support: l !== null,
    syntect_support: s !== null && s.length === p.length,
    lengths: { prism: p.length, lowlight: l?.length ?? null, syntect: s?.length ?? null },
  };
  const agree = (a, b) => {
    const n = Math.min(a.length, b.length);
    let same = 0;
    for (let i = 0; i < n; i++) if (a[i] === b[i]) same += 1;
    return n === 0 ? null : +(same / n * 100).toFixed(1);
  };
  entry.agreement = {
    prism_vs_lowlight: l ? agree(p, l) : null,
    prism_vs_syntect: s && entry.syntect_support ? agree(p, s) : null,
    lowlight_vs_syntect: l && s && entry.syntect_support ? agree(l, s) : null,
  };
  matrix[lang] = entry;
}

const langsAll = Object.keys(matrix);
const mean = k => {
  const vals = langsAll.map(l => matrix[l].agreement[k]).filter(v => v != null);
  return vals.length ? +(vals.reduce((a, b) => a + b, 0) / vals.length).toFixed(1) : null;
};
const summary = {
  prism_vs_lowlight: mean('prism_vs_lowlight'),
  prism_vs_syntect: mean('prism_vs_syntect'),
  lowlight_vs_syntect: mean('lowlight_vs_syntect'),
};

writeFileSync(join(OUT_DIR, 'matrix.json'), JSON.stringify({ summary, matrix }, null, 2) + '\n');

// 6) 报告
const fmt = v => (v == null ? '—' : v + '%');
let md = `# 高亮三方案一致性矩阵（PLAN-038 T15）

- 引擎：prismjs ^1.29（web/vue 轨现状,PrismCodeBlock 同语言注册面）·
  lowlight@3 common 集（@autodown/vue 内置同装配）·
  syntect 5 + two-face 0.4（auto-lang code_editor 同内核同版本,scripts/highlight-rs）
- 方法：同一 fixtures 代码,三引擎各产出逐字符 token 流,经 scope→token 近似映射表
  （CATEGORY_MAP,见脚本头）归一类别集后两两逐字符一致率。**近似映射,决策粒度数据,
  非全等断言**。
- fixtures：scripts/lib-parity/fixtures/highlight/code/（musk 实际语言集 14 语言,
  覆盖计划点名 11 语言 + PrismCodeBlock 实际注册的 cpp/go）
- 再生：node scripts/lib-parity/highlight-compare.mjs（内含 cargo 侧调用）

## 汇总（各语言一致率均值）

| 对比 | 一致率 |
|---|---|
| prism vs lowlight | ${fmt(summary.prism_vs_lowlight)} |
| prism vs syntect | ${fmt(summary.prism_vs_syntect)} |
| lowlight vs syntect | ${fmt(summary.lowlight_vs_syntect)} |

## 分语言矩阵

| 语言 | prism | lowlight | syntect | p–l | p–s | l–s |
|---|---|---|---|---|---|---|
`;
for (const l of langsAll) {
  const e = matrix[l];
  md += `| ${l} | ✓ | ${e.lowlight_support ? '✓' : '✗ 未注册'} | ${e.syntect_support ? '✓' : (e.lengths.syntect != null ? '✗ 长度不符' : '✗')} | ${fmt(e.agreement.prism_vs_lowlight)} | ${fmt(e.agreement.prism_vs_syntect)} | ${fmt(e.agreement.lowlight_vs_syntect)} |\n`;
}
md += `\n长度校验（逐字符流长度应 = 代码长度,三引擎一致）:\n`;
for (const l of langsAll) md += `- ${l}: ${JSON.stringify(matrix[l].lengths)}\n`;
md += `\n（决策解读见 PLAN-038 T16 复审记录登记。）\n`;
writeFileSync(join(OUT_DIR, 'report.md'), md);

console.log(`highlight-compare: ${langsAll.length} 语言 × 3 方案矩阵 -> fixtures/highlight/report.md`);
console.log(`  prism vs lowlight ${fmt(summary.prism_vs_lowlight)} | prism vs syntect ${fmt(summary.prism_vs_syntect)} | lowlight vs syntect ${fmt(summary.lowlight_vs_syntect)}`);
