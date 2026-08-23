#!/usr/bin/env node
// i18n-fixtures.mjs — PLAN-038 T4：用 npm 原库 vue-i18n 生成期望输出（对拍基准）。
//
// 对 src/front/i18n/{zh,en}.json 的全部叶子键生成三类用例：
//   1. plain      — t(key) 无参（含 {'@'} 字面量转义键、{count} 缺参键的真实行为锁定）
//   2. interp     — 值含 {name} 占位符的键，t(key, {count: 42}) 具名插值
//   3. missing    — 固定缺键集合，锁定 vue-i18n「返回 key 本身」回退行为
// 输出 scripts/lib-parity/fixtures/i18n-expected.json（入库）。
//
// 注：计划普查口径为 81 键，实测叶子键 72/语言（见 T4 证据行）；fixtures 以实际文件为准。

import { createRequire } from 'node:module';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const require = createRequire(import.meta.url);
const ROOT = fileURLToPath(new URL('../../', import.meta.url));

// 以 web/node_modules 的 vue-i18n 为基准（普查声明的 ^9.14）
const { createI18n } = require(join(ROOT, 'web/node_modules/vue-i18n/dist/vue-i18n.cjs.js'));

const zh = require(join(ROOT, 'src/front/i18n/zh.json'));
const en = require(join(ROOT, 'src/front/i18n/en.json'));

function* leafKeys(obj, prefix = '') {
  for (const [k, v] of Object.entries(obj)) {
    const key = prefix ? `${prefix}.${k}` : k;
    if (v && typeof v === 'object' && !Array.isArray(v)) yield* leafKeys(v, key);
    else yield key;
  }
}

const i18n = createI18n({
  legacy: false,
  locale: 'zh',
  fallbackLocale: 'zh',
  messages: { zh, en },
});
const t = i18n.global.t.bind(i18n.global);

const MISSING_KEYS = [
  'app.nonexistent',
  'chat.msgs.deep',
  'totally.missing.key',
  'nav',
  '',
];

const cases = [];
for (const locale of ['zh', 'en']) {
  i18n.global.locale.value = locale; // Composer.t 无 locale 位置参数,切换全局 locale
  const keys = [...leafKeys(locale === 'zh' ? zh : en)];
  for (const key of keys) {
    cases.push({ locale, key, kind: 'plain', expected: t(key) });
    const raw = String(key.split('.').reduce((o, p) => (o == null ? o : o[p]), locale === 'zh' ? zh : en));
    const params = {};
    for (const m of raw.matchAll(/\{([a-zA-Z_][a-zA-Z0-9_]*)\}/g)) params[m[1]] = m[1] === 'count' ? 42 : 'X';
    if (Object.keys(params).length > 0) {
      cases.push({ locale, key, kind: 'interp', params, expected: t(key, params) });
    }
  }
  for (const key of MISSING_KEYS) {
    cases.push({ locale, key, kind: 'missing', expected: t(key) });
  }
}

const out = {
  generated_at: new Date().toISOString(),
  source: 'vue-i18n@' + require(join(ROOT, 'web/node_modules/vue-i18n/package.json')).version,
  leaf_keys: { zh: [...leafKeys(zh)].length, en: [...leafKeys(en)].length },
  cases,
};

const outPath = fileURLToPath(new URL('fixtures/i18n-expected.json', import.meta.url));
mkdirSync(dirname(outPath), { recursive: true });
writeFileSync(outPath, JSON.stringify(out, null, 2) + '\n');

console.log(`i18n fixtures: ${cases.length} cases ` +
  `(zh ${out.leaf_keys.zh} keys, en ${out.leaf_keys.en} keys; ` +
  `plain=${cases.filter(c => c.kind === 'plain').length}, ` +
  `interp=${cases.filter(c => c.kind === 'interp').length}, ` +
  `missing=${cases.filter(c => c.kind === 'missing').length}) -> ${outPath}`);
