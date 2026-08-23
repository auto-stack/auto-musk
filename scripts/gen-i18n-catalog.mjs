#!/usr/bin/env node
// gen-i18n-catalog.mjs — PLAN-038 T6：把 src/front/i18n/{zh,en}.json 的目录数据
// 写入 src/front/lib/i18n.at 的 @gen 标记区块（fn i18n_catalog()）。
//
// 区块外的实现逻辑为手写区，脚本只做标记区间替换。幂等：无时间戳、键序稳定，
// 连跑两次 diff 为空。
//
// 再生：node scripts/gen-i18n-catalog.mjs

import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { join } from 'node:path';

const ROOT = fileURLToPath(new URL('../', import.meta.url));
const TARGET = join(ROOT, 'src/front/lib/i18n.at');

const zh = JSON.parse(readFileSync(join(ROOT, 'src/front/i18n/zh.json'), 'utf8'));
const en = JSON.parse(readFileSync(join(ROOT, 'src/front/i18n/en.json'), 'utf8'));

// .at map 字面量：引号键 + JSON 字符串转义（\" \\ \n \uXXXX 双方一致）
function atLiteral(value, indent) {
  const pad = ' '.repeat(indent);
  const padInner = ' '.repeat(indent + 4);
  if (typeof value === 'string') return JSON.stringify(value);
  const entries = Object.entries(value).map(([k, v]) => `${padInner}${JSON.stringify(k)}: ${atLiteral(v, indent + 4)}`);
  if (entries.length === 0) return '{}';
  return `{\n${entries.join(',\n')}\n${pad}}`;
}

const block = `// @gen:i18n-catalog-begin（生成物勿手改——scripts/gen-i18n-catalog.mjs 再生）
fn i18n_catalog() map {
    return {
        "zh": ${atLiteral(zh, 8)},
        "en": ${atLiteral(en, 8)},
    }
}
// @gen:i18n-catalog-end`;

const src = readFileSync(TARGET, 'utf8');
const BEGIN = '// @gen:i18n-catalog-begin';
const END = '// @gen:i18n-catalog-end';
const beginIdx = src.indexOf(BEGIN);
const endIdx = src.indexOf(END);
if (beginIdx === -1 || endIdx === -1 || endIdx < beginIdx) {
  console.error(`[gen-i18n-catalog] ${TARGET} 中未找到 ${BEGIN} ... ${END} 标记区间`);
  process.exit(1);
}

const out = src.slice(0, beginIdx) + block + src.slice(endIdx + END.length);
writeFileSync(TARGET, out);
console.log(`i18n catalog block updated in src/front/lib/i18n.at (zh ${Object.keys(zh).length} sections, en ${Object.keys(en).length} sections)`);
