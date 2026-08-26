// vm-link-probe.mjs — PLAN-045: auto-musk VM 链接门禁一键探针
//
// 对 musk 全量前端(src/front/app.at)做 VM 目标的 parse+codegen+link headless
// 验证(auto-lang plan442_musk_probe_tests,#[ignore] 手动门)。
//
// 用法: node scripts/vm-link-probe.mjs   (或 scriptsm-link-probe.cmd 委托)
// 前置: sibling 检出 ../auto-lang 存在且可 cargo 构建。
//
// 勘误(PLAN-045): 探针模块门控是 feature "ui-iced";auto-lang 442 计划文档
// 头注的 "--features ui-interpreter" 已过时(该 feature 集编译失败)。
// 环境注: 直接运行测试 exe 需 RUST_MIN_STACK=16777216;经 cargo 运行时由
// auto-lang 仓 .cargo/config.toml [env] 自动提供。

import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const muskRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const langRoot = resolve(muskRoot, '..', 'auto-lang');
if (!existsSync(resolve(langRoot, 'crates', 'auto-lang'))) {
  console.error(`[vm-link-probe] sibling auto-lang not found at ${langRoot}`);
  process.exit(2);
}
const r = spawnSync(
  'cargo',
  ['test', '-p', 'auto-lang', '--lib', '--features', 'ui-iced', 'musk_probe',
   '--', '--ignored', '--nocapture'],
  { cwd: langRoot, stdio: 'inherit', shell: true },
);
if (r.status === 0) {
  console.log('[vm-link-probe] PASS — musk frontend links on VM target');
  process.exit(0);
}
console.log('[vm-link-probe] FAIL — see [HANDLER-CODEGEN] / [CODEGEN] lines above');
process.exit(r.status ?? 1);
