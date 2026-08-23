#!/usr/bin/env node
// i18n.mjs — PLAN-038 T7：auto-i18n（src/front/lib/i18n.at 编译产物）对 vue-i18n
// 基准 fixtures 的对拍（differential testing）。
//
// 基准：scripts/lib-parity/fixtures/i18n-expected.json（i18n-fixtures.mjs 从
// web/node_modules 的 vue-i18n ^9 实测生成）。
// 被测：gen/front/vue/src/ext/src/front/lib/i18n.ts（auto build 产物；先跑
// `auto build --gen-only` 再跑本脚本）。
// 断言：全部用例输出全等 → exit 0。

import { readFileSync } from 'node:fs';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { join } from 'node:path';

const ROOT = fileURLToPath(new URL('../../', import.meta.url));
const GEN_I18N = join(ROOT, 'gen/front/vue/src/ext/src/front/lib/i18n.ts');
const FIXTURES = join(ROOT, 'scripts/lib-parity/fixtures/i18n-expected.json');

const { i18nT } = await import(pathToFileURL(GEN_I18N).href);
const fixtures = JSON.parse(readFileSync(FIXTURES, 'utf8'));

let pass = 0;
const fails = [];
for (const c of fixtures.cases) {
  let actual;
  try {
    actual = i18nT(c.locale, c.key, c.params ?? undefined);
  } catch (e) {
    actual = `<thrown: ${e.message}>`;
  }
  if (actual === c.expected) pass += 1;
  else fails.push({ ...c, actual });
}

console.log(`i18n parity: ${pass}/${fixtures.cases.length} 全等 (vue-i18n@${fixtures.source})`);
if (fails.length > 0) {
  console.error('不等用例:');
  for (const f of fails.slice(0, 20)) {
    console.error(`  [${f.locale}] ${f.key} (${f.kind})`);
    console.error(`    expected: ${JSON.stringify(f.expected)}`);
    console.error(`    actual:   ${JSON.stringify(f.actual)}`);
  }
  if (fails.length > 20) console.error(`  ... 共 ${fails.length} 例`);
  process.exit(1);
}
