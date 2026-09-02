#!/usr/bin/env node
// vendor-autodown-vue.mjs — PLAN-056 T7：把 auto-down 的 @autodown/engine 0.5.0
// dist 快照 vendor 进 musk 仓库 vendor/@autodown/engine/（自包含 file: 接入）。
//
// 历史：本脚本原为 vendor @autodown/vue 0.2.0（PLAN-038 T11，渲染真源切换）。
// 2026-09-02 PLAN-056 T7 起消费面升格：chat 渲染切真实 @autodown/engine
// 0.5.0（auto-down master，dist-stamp 058e5d70…），@autodown/vue 降级为
// re-export 别名 shim（生成物 platform/markdown.vue 的历史 import 零改动）。
// 上游 DEBTS 020「旧包 shim 物理归档」由此解锁（musk 不再读 packages/vue）。
//
// 再生：node scripts/vendor-autodown-vue.mjs（幂等；auto-down 侧先
// pnpm --filter @autodown/engine build；在 auto-down 仓库 worktree 内执行
// 后拷贝 dist，或直接让本脚本读上游检出路径）。

import { copyFileSync, cpSync, mkdirSync, readFileSync, rmSync, writeFileSync, existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { join } from 'node:path';

const ROOT = fileURLToPath(new URL('../../', import.meta.url));
// 允许 --src 覆盖（worktree 场景：上游构建产物在别处）；默认读主检出
const argSrcIdx = process.argv.indexOf('--src');
const ENGINE_DIST = argSrcIdx >= 0 ? process.argv[argSrcIdx + 1]
  : join(ROOT, '../auto-down/autodown/packages/engine/dist');
const ENGINE_PKG = join(ENGINE_DIST, '..', 'package.json');
const DST = join(ROOT, 'vendor/@autodown/engine');

if (!existsSync(join(ENGINE_DIST, 'index.js'))) {
  console.error(`[vendor-autodown] ${ENGINE_DIST}/index.js 不存在——先在 auto-down 侧 pnpm --filter @autodown/engine build（或用 --src 指向构建产物目录）`);
  process.exit(1);
}

rmSync(DST, { recursive: true, force: true });
mkdirSync(join(DST, 'dist'), { recursive: true });

// dist 全量拷贝：index/editor/render/parser 的 js+d.ts、code-split chunks、
// 子目录（editor/ parser/ render/）、style.css、.dist-stamp（上游构建指纹）
cpSync(ENGINE_DIST, join(DST, 'dist'), { recursive: true });

// shim package.json：声明上游运行时依赖（katex/mermaid 为 enable* 动态
// opt-in，musk 未调用仅安装不加载）；peer 保持上游口径
const srcPkg = JSON.parse(readFileSync(ENGINE_PKG, 'utf8'));
const shim = {
  name: '@autodown/engine',
  version: `${srcPkg.version}-musk-vendor`,
  description: 'PLAN-056 T7: 真实 @autodown/engine dist vendor（快照自 auto-down master）。上游包本体见 auto-down 仓库 autodown/packages/engine。',
  type: 'module',
  main: './dist/index.js',
  module: './dist/index.js',
  types: './dist/index.d.ts',
  exports: {
    '.': { import: './dist/index.js', types: './dist/index.d.ts' },
    './style.css': './dist/style.css'
  },
  dependencies: srcPkg.dependencies,
  peerDependencies: srcPkg.peerDependencies
};
writeFileSync(join(DST, 'package.json'), JSON.stringify(shim, null, 2) + '\n');

// @autodown/vue → 退役别名（re-export engine；生成物 platform/markdown.vue
// 的历史 import 零改动）。dist/style.css 为 engine 副本（CSS 无法 re-export）。
const DST_VUE = join(ROOT, 'vendor/@autodown/vue');
rmSync(DST_VUE, { recursive: true, force: true });
mkdirSync(join(DST_VUE, 'dist'), { recursive: true });
writeFileSync(join(DST_VUE, 'dist', 'index.js'),
  "// PLAN-056 T7: @autodown/vue 退役别名——真源 = @autodown/engine（../engine）。\n// 保留仅为 gen 轨生成物 platform/markdown.vue 的历史 import 零改动。\nexport * from '@autodown/engine';\n");
writeFileSync(join(DST_VUE, 'dist', 'index.d.ts'),
  "export * from '@autodown/engine';\n");
copyFileSync(join(DST, 'dist', 'style.css'), join(DST_VUE, 'dist', 'style.css'));
const vueShim = {
  name: '@autodown/vue',
  version: `${srcPkg.version}-musk-alias`,
  description: 'PLAN-056 T7: 退役别名——re-export @autodown/engine（真源 ../engine）。上游 plan 020 已 deprecate 本包；保留仅为生成物 platform/markdown.vue 历史 import 零改动，待 auto-lang 模板切 engine 后移除。',
  type: 'module',
  main: './dist/index.js',
  module: './dist/index.js',
  types: './dist/index.d.ts',
  exports: {
    '.': { import: './dist/index.js', types: './dist/index.d.ts' },
    './style.css': './dist/style.css'
  },
  dependencies: {
    '@autodown/engine': 'file:../engine'
  }
};
writeFileSync(join(DST_VUE, 'package.json'), JSON.stringify(vueShim, null, 2) + '\n');
console.log(`[vendor-autodown] engine ${srcPkg.version} dist -> vendor/@autodown/engine（+ vue 退役别名）`);
