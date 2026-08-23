#!/usr/bin/env node
// vendor-autodown-vue.mjs — PLAN-038 T11：把 ../auto-down 的 @autodown/vue dist
// 快照 vendor 进 musk 仓库 vendor/@autodown/vue/（自包含,web/gen 双轨 file: 接入）。
//
// 为什么 vendor 而非 file: 直链 ../auto-down：上游 package.json 声明
// `@autodown/core: workspace:*`——npm/pnpm 在 workspace 外均无法解析该协议,
// file: 直链安装即失败。dist 本身为 lib-bundle,运行时外部化仅
// vue/markstream-vue/lowlight/hast-util-to-html（dist 未引用 core/katex/mermaid,
// 类型面仅 vue）——shim 只声明这四个,均可从消费方解析。
// 版本跟进 = auto-down 侧重build后重跑本脚本（快照含源版本号,git diff 可审）。
//
// 再生：node scripts/vendor-autodown-vue.mjs（幂等；auto-down 侧需先 pnpm build）

import { copyFileSync, mkdirSync, readFileSync, rmSync, writeFileSync, existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { join } from 'node:path';

const ROOT = fileURLToPath(new URL('../', import.meta.url));
const SRC = join(ROOT, '../auto-down/autodown/packages/vue');
const DST = join(ROOT, 'vendor/@autodown/vue');

const srcPkg = JSON.parse(readFileSync(join(SRC, 'package.json'), 'utf8'));

if (!existsSync(join(SRC, 'dist/index.js'))) {
  console.error(`[vendor-autodown] ${SRC}/dist 不存在——先在 auto-down 侧 pnpm build`);
  process.exit(1);
}

rmSync(DST, { recursive: true, force: true });
mkdirSync(join(DST, 'dist'), { recursive: true });

for (const f of ['index.js', 'index.d.ts', 'style.css', 'StreamingRenderer.vue.d.ts', 'StreamingTable.vue.d.ts', 'useStreamingDocument.d.ts']) {
  copyFileSync(join(SRC, 'dist', f), join(DST, 'dist', f));
}

// shim：仅声明 dist 实际外部化的运行时依赖（上游 workspace:* 等不可解析项剔除）
const shim = {
  name: '@autodown/vue',
  version: srcPkg.version,
  description: `${srcPkg.description} (vendored dist snapshot; source ../auto-down, regen: scripts/vendor-autodown-vue.mjs)`,
  type: 'module',
  main: './dist/index.js',
  module: './dist/index.js',
  types: './dist/index.d.ts',
  exports: {
    '.': { import: './dist/index.js', types: './dist/index.d.ts' },
    './style.css': './dist/style.css',
  },
  peerDependencies: { vue: '^3.4.0' },
  dependencies: {
    'markstream-vue': '^0.0.14-beta.8',
    lowlight: '^3.3.0',
    'hast-util-to-html': '9.0.5',
  },
};
writeFileSync(join(DST, 'package.json'), JSON.stringify(shim, null, 2) + '\n');

console.log(`vendored @autodown/vue@${srcPkg.version} -> vendor/@autodown/vue (dist + shim package.json)`);
