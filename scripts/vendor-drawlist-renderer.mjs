#!/usr/bin/env node
// vendor-drawlist-renderer.mjs — PLAN-102 T-04：把 auto-lang 的
// @auto/drawlist-renderer 源码快照 vendor 进 musk 仓库
// vendor/@auto/drawlist-renderer/（自包含 file: 接入——vendor-autodown
// 先例口径）。包为纯 TS 源形态（main: ./src/index.ts，无构建产物依赖），
// 直接整目录拷 src + package.json。
//
// 再生：node scripts/vendor-drawlist-renderer.mjs [--src <auto-lang 检出>]
//（默认 ../auto-lang 主检出；worktree 场景 --src 指依赖 worktree）。
// 上游版本漂移：重跑本脚本快照新源（Rust↔TS golden 对拍防漂移在上游侧）。

import { cpSync, mkdirSync, rmSync, writeFileSync, readFileSync, existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { join } from 'node:path';

const ROOT = fileURLToPath(new URL('../', import.meta.url));
const argSrcIdx = process.argv.indexOf('--src');
const SRC = argSrcIdx >= 0 ? process.argv[argSrcIdx + 1]
  : join(ROOT, '../auto-lang/packages/drawlist-renderer');
const DST = join(ROOT, 'vendor/@auto/drawlist-renderer');

if (!existsSync(join(SRC, 'package.json'))) {
  console.error(`[vendor-drawlist] 源缺失: ${SRC}（--src 覆盖）`);
  process.exit(1);
}

rmSync(DST, { recursive: true, force: true });
mkdirSync(DST, { recursive: true });
cpSync(join(SRC, 'src'), join(DST, 'src'), { recursive: true });
writeFileSync(join(DST, 'package.json'), readFileSync(join(SRC, 'package.json')));

const ver = JSON.parse(readFileSync(join(SRC, 'package.json'), 'utf8')).version;
console.log(`[vendor-drawlist] ${ver} → ${DST}`);
