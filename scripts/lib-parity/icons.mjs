#!/usr/bin/env node
// icons.mjs — PLAN-038 T10：auto-icons 数据层对拍（降级态）。
//
// 渲染层降级（T9 canary：.at UI 不支持 svg，见 KNOWN-DEBT Plan 038），本脚本按
// 计划 T10 括号条款执行降级对拍：gen 编译产物 icons_data.ts 的元素序列
// （tag + attrs）对 lucide dist 源数据全等断言（规范化口径同生成器：剔除 vue
// 渲染提示 key；lucide 默认六属性提为渲染层默认值不入数据层）。
// svg 能力就绪后升级为 @vue/server-renderer 双端 renderToString 对拍。

import { readFileSync } from 'node:fs';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { join } from 'node:path';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);
const ROOT = fileURLToPath(new URL('../../', import.meta.url));
const DIST_ICONS = join(ROOT, 'web/node_modules/lucide-vue-next/dist/esm/icons');

// 与 scripts/gen-icons.at-data.mjs 相同的清单/别名/规范化（对拍独立重述，防生成器
// 自身 bug 自证自明——两侧独立实现，全等才是有效对拍）
const ALIASES = {
  FileIcon: 'file', HelpCircle: 'circle-help', Loader2: 'loader-circle',
  Unlink: 'unlink-2', UploadCloud: 'upload', CopyCheck: 'copy-check',
};
const kebab = n => n.replace(/([a-z0-9])([A-Z])/g, '$1-$2').replace(/([a-zA-Z])(\d)/g, '$1-$2').toLowerCase();

const { icons_data } = await import(pathToFileURL(join(ROOT, 'gen/front/vue/src/ext/src/front/lib/icons_data.ts')).href);

const genIcons = icons_data();
let pass = 0;
const fails = [];
for (const [name, genElements] of Object.entries(genIcons)) {
  const stem = ALIASES[name] ?? kebab(name);
  let src;
  try { src = readFileSync(join(DIST_ICONS, stem + '.js'), 'utf8'); }
  catch { if (name.endsWith('Icon')) src = readFileSync(join(DIST_ICONS, kebab(name.replace(/Icon$/, '')) + '.js'), 'utf8'); else src = null; }
  if (!src) { fails.push({ name, reason: 'dist 源缺失' }); continue; }
  const m = src.match(/createLucideIcon\("[^"]+",\s*(\[[\s\S]*\])\);/);
  const distElements = new Function(`return ${m[1]}`)()
    .map(([tag, attrs]) => ({ tag, attrs: Object.fromEntries(Object.entries(attrs).filter(([k]) => k !== 'key')) }));
  const a = JSON.stringify(genElements);
  const b = JSON.stringify(distElements);
  if (a === b) pass += 1;
  else fails.push({ name, reason: `序列不等\n    gen:  ${a.slice(0, 160)}\n    dist: ${b.slice(0, 160)}` });
}

console.log(`icons data parity (degraded): ${pass}/${Object.keys(genIcons).length} 全等 (lucide dist ${require(join(ROOT, 'web/node_modules/lucide-vue-next/package.json')).version})`);
if (fails.length > 0) {
  console.error('不等图标:');
  for (const f of fails.slice(0, 10)) console.error(`  ${f.name}: ${f.reason}`);
  if (fails.length > 10) console.error(`  ... 共 ${fails.length}`);
  process.exit(1);
}
