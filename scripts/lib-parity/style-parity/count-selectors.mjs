// count-selectors.mjs — PLAN-049 T1 对账脚本
//
// 从 src/front/inject_styles.ts 的 STYLES 模板串里抽取全部顶层选择器，
// 输出计数与清单，用于 MIGRATION.md §1 的覆盖对账（验收：清单覆盖全部选择器）。
//
// 用法: node scripts/lib-parity/style-parity/count-selectors.mjs
// 退出码恒 0；对账 = 人工比对本输出与 MIGRATION.md §1 表行数。

import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../../..');
const src = readFileSync(resolve(root, 'src/front/inject_styles.ts'), 'utf8');

// 取 STYLES 反引号模板串
const m = src.match(/const STYLES = `([\s\S]*)`/);
if (!m) {
  console.error('STYLES template not found');
  process.exit(1);
}
const css = m[1];

// 逐条剥离注释，再抽 "selector {"
const noComments = css.replace(/\/\*[\s\S]*?\*\//g, '');
const selectors = [];
// @ 规则（@import/@keyframes）单独记
const atRules = [];
for (const raw of noComments.split(/(?<=\})\s*(?=[^@\s}])/)) {
  const seg = raw.trim();
  if (!seg) continue;
  const am = seg.match(/^(@[a-z-]+[^{\s]*(?:\s[^{]+)?(?:,[^{]*)?)\{/g);
  if (seg.startsWith('@')) {
    // @import url(...)（无块）；@keyframes name { ... }
    for (const am2 of seg.matchAll(/^(@[a-z-]+\s*[^{\n]*)\{/gm)) {
      atRules.push(am2[1].trim());
    }
    if (seg.startsWith('@import')) atRules.push('@import (font)');
    continue;
  }
  // 普通规则：选择器 = 首个 { 之前的部分（可逗号分隔多条）
  const idx = seg.indexOf('{');
  if (idx < 0) continue;
  for (const part of seg.slice(0, idx).split(',')) {
    const sel = part.trim().replace(/\s+/g, ' ');
    if (!sel) continue;
    // @keyframes 内的帧选择器（40%/60%）不是选择器,跳过
    if (/^\d+%$/.test(sel)) continue;
    selectors.push(sel);
  }
}

console.log(`[count-selectors] inject_styles.ts 顶层规则选择器总数: ${selectors.length}`);
console.log('[count-selectors] @ 规则:', atRules.length ? atRules.join(' | ') : '(无)');
console.log('[count-selectors] —— 清单开始 ——');
for (const s of selectors) console.log(`  ${s}`);
console.log('[count-selectors] —— 清单结束 ——');
