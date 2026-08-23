#!/usr/bin/env node
// deps-guard.mjs — 第三方依赖白名单守卫（PLAN-038 T3 / 设计 D6）。
//
// 白名单（内置,新增第三方依赖须显式改这里并说明理由）：
//   - 普查结论表：vue-i18n / lucide-vue-next / markstream-vue / prismjs / mermaid
//   - 运行时：vue
//   - 测试基建：vitest / @vue/server-renderer（package.json 声明层的 @types/* 不在
//     import 扫描面内,随 devDependencies 评审）
//   - gen 轨 cn()：clsx / tailwind-merge
//   - Phase 3 渲染真源：@autodown/vue
//
// 扫描面：
//   1. web/src 的非相对、非 @/ 别名 import（from / 副作用 import / 动态 import()）
//   2. gen/front/vue/src 同上
//   3. src/front 全部 `use.web ... from "<target>"` 中形如 npm 包名的目标
//      （src/ 前缀本地路径、platform: 协议目标豁免）
//
// 超白名单 → exit 1 并打印完整清单（CI 可挂）。
//
// 过渡区（TRANSITIONAL）：T2 已裁定转责（2026-08-23,选项 (ii)）——auto-lang 442
// Phase 0 P0-1（依赖按使用裁剪）落地前,共享模板仍生成 codemirror 系死依赖声明与
// 死文件 CodeEditor.vue 的 import。守卫放行但单列打印,442 P0-1 落地后删除。

import { readFileSync, readdirSync, existsSync } from 'node:fs';
import { join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = fileURLToPath(new URL('../../', import.meta.url));

const WHITELIST = new Set([
  // 普查结论表（PLAN-038 需求分析）
  'vue-i18n', 'lucide-vue-next', 'markstream-vue', 'prismjs', 'mermaid',
  // 运行时
  'vue',
  // 测试基建
  'vitest', '@vue/server-renderer', '@vue/test-utils',
  // gen 轨 cn()（待澄清 #6,VM 轨样式模型未定）
  'clsx', 'tailwind-merge',
  // auto-man 脚手架 ui 组件库（gen 轨 Button/Input 真实运行面,全量 build 时按需生成）
  'class-variance-authority', 'reka-ui',
  // Phase 3 渲染真源切换（T11 接入）
  '@autodown/vue',
]);

const TRANSITIONAL = new Set([
  'vue-codemirror', 'codemirror',
  '@codemirror/view', '@codemirror/state', '@codemirror/language', '@codemirror/search',
  '@codemirror/lang-rust', '@codemirror/lang-python', '@codemirror/lang-javascript',
  '@codemirror/lang-markdown', '@codemirror/lang-json',
]);

const SCAN_DIRS = ['web/src', 'gen/front/vue/src'];
const USE_WEB_DIR = 'src/front';
const CODE_EXT = /\.(ts|mts|js|mjs|vue|at)$/;

function* walk(dir) {
  if (!existsSync(dir)) return;
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const p = join(dir, entry.name);
    if (entry.isDirectory()) yield* walk(p);
    else if (CODE_EXT.test(entry.name)) yield p;
  }
}

function pkgRoot(spec) {
  const parts = spec.split('/');
  return spec.startsWith('@') ? parts.slice(0, 2).join('/') : parts[0];
}

// 相对/别名/协议/资源目标豁免
function isExempt(spec) {
  return spec.startsWith('.') || spec.startsWith('@/') || spec.startsWith('/') ||
    spec.startsWith('src/') || spec.includes(':') || spec.endsWith('.css') ||
    spec.endsWith('.json') || /\.(png|svg|jpg|jpeg|gif|webp|woff2?)$/.test(spec);
}

function* importSpecs(text) {
  yield* text.matchAll(/from\s+['"]([^'"]+)['"]/g);
  yield* text.matchAll(/(?:^|\n)\s*import\s+['"]([^'"]+)['"]/g);
  yield* text.matchAll(/import\(\s*['"]([^'"]+)['"]\s*\)/g);
}

const violations = new Map(); // pkg -> [{ file, spec }]
const transitionalHits = new Set();

function record(pkg, file, spec) {
  if (WHITELIST.has(pkg)) return;
  if (TRANSITIONAL.has(pkg)) { transitionalHits.add(pkg); return; }
  if (!violations.has(pkg)) violations.set(pkg, []);
  violations.get(pkg).push({ file, spec });
}

for (const dir of SCAN_DIRS) {
  for (const file of walk(join(ROOT, dir))) {
    const text = readFileSync(file, 'utf8');
    for (const m of importSpecs(text)) {
      const spec = m[1];
      if (isExempt(spec)) continue;
      record(pkgRoot(spec), relative(ROOT, file), spec);
    }
  }
}

for (const file of walk(join(ROOT, USE_WEB_DIR))) {
  const text = readFileSync(file, 'utf8');
  for (const m of text.matchAll(/use\.web[^\n]*?from\s+['"]([^'"]+)['"]/g)) {
    const spec = m[1];
    if (isExempt(spec)) continue;
    record(pkgRoot(spec), relative(ROOT, file), spec);
  }
}

if (transitionalHits.size > 0) {
  console.warn(`[deps-guard] 过渡放行（待 auto-lang 442 P0-1 落地清零）: ${[...transitionalHits].sort().join(', ')}`);
}

if (violations.size > 0) {
  console.error('[deps-guard] 超白名单第三方依赖:');
  for (const [pkg, hits] of [...violations.entries()].sort()) {
    console.error(`  ${pkg}`);
    for (const h of hits.slice(0, 5)) console.error(`    ${h.file}  (${h.spec})`);
    if (hits.length > 5) console.error(`    ... 共 ${hits.length} 处`);
  }
  console.error('\n处理：移除引用,或将其加入本脚本 WHITELIST 并注明理由。');
  process.exit(1);
}

console.log(`[deps-guard] OK — 扫描面 ${SCAN_DIRS.join(' + ')} + ${USE_WEB_DIR} use.web,零超白名单依赖。`);
