// vm-link-probe.mjs — PLAN-045: auto-musk VM 链接门禁一键探针
//
// 对 musk 全量前端(src/front/app.at)做 VM 目标的 parse+codegen+link headless
// 验证(auto-lang plan442_musk_probe_tests,#[ignore] 手动门)。
//
// 用法: node scripts/vm-link-probe.mjs   (或 vm-link-probe.cmd 委托)
// 前置: sibling 检出 ../auto-lang 存在且可 cargo 构建。
//
// 勘误(PLAN-045): 探针模块门控是 feature "ui-iced";auto-lang 442 计划文档
// 头注的 "--features ui-interpreter" 已过时(该 feature 集编译失败)。
// 环境注: 直接运行测试 exe 需 RUST_MIN_STACK=16777216;经 cargo 运行时由
// auto-lang 仓 .cargo/config.toml [env] 自动提供。
//
// PLAN-046 T10: 体积门禁(阈值可 env 覆盖)。
//   实测锚点(2026-08-27): musk 全量前端 link 后合计 60614 bytes 且探针 PASS
//   ——auto-lang K1 回跳 isize 化后,历史"单模块 >32767 必回绕"前提已失效,
//   故不再以 32767 为红线上限。当前口径:
//   - 统计面 = 链接后全部模块合计字节([probe] 行,flash.memory.len;
//     单模块精确记账需上游 per-module 面——KNOWN-DEBT 046 同步义务)
//   - WARN 默认 90000(实测带 +48%,增长趋势提示)
//   - FAIL 默认 131072 = 2^17(保守包络,非实测红线)
//   - 无 [probe] 行(auto-lang master 早于 e3abde1ba)→ 注记不判失败

import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SIZE_WARN = Number(process.env.VM_PROBE_SIZE_WARN ?? 90000);
const SIZE_FAIL = Number(process.env.VM_PROBE_SIZE_FAIL ?? 131072);

const muskRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
// PLAN-049: 布局自适应——主检出 sibling ../auto-lang;worktree 布局 ../../auto-lang;
// env VM_LINK_LANG_ROOT 最优先。
const langRoot =
  process.env.VM_LINK_LANG_ROOT ||
  [resolve(muskRoot, '..', 'auto-lang'), resolve(muskRoot, '..', '..', '..', 'auto-lang')]
    .find((p) => existsSync(resolve(p, 'crates', 'auto-lang')));
if (!langRoot) {
  console.error('[vm-link-probe] auto-lang not found (env VM_LINK_LANG_ROOT 可指定)');
  process.exit(2);
}
const r = spawnSync(
  'cargo',
  ['test', '-p', 'auto-lang', '--lib', '--features', 'ui-iced', 'musk_probe',
   '--', '--ignored', '--nocapture'],
  { cwd: langRoot, encoding: 'utf8', shell: true },
);
// tee 探针原始输出(毒化/handler-codegen 失败行仍可见)
if (r.stdout) process.stdout.write(r.stdout);
if (r.stderr) process.stderr.write(r.stderr);

let fail = r.status !== 0;
const m = String(r.stdout ?? '').match(/\[probe\] synthesized\+linked modules: (\d+) bytes/);
if (m) {
  const size = Number(m[1]);
  console.log(`[vm-link-probe] linked-modules-total ${size} bytes (WARN>=${SIZE_WARN} / FAIL>=${SIZE_FAIL})`);
  if (size >= SIZE_FAIL) {
    console.log(`[vm-link-probe] FAIL-SIZE ${size} >= ${SIZE_FAIL}: 逼近/越过校准包络,安排分模块排期(PLAN-046 D6)`);
    fail = true;
  } else if (size >= SIZE_WARN) {
    console.log(`[vm-link-probe] WARN-SIZE ${size} >= ${SIZE_WARN}: 模块体积趋势告警`);
  }
} else if (r.status === 0) {
  console.log('[vm-link-probe] note: no [probe] size line — auto-lang master predates plan-046 (e3abde1ba); size gate skipped');
}
if (!fail) {
  console.log('[vm-link-probe] PASS — musk frontend links on VM target' +
    (m ? '' : ' (no size data)'));
  process.exit(0);
}
console.log('[vm-link-probe] FAIL — see [HANDLER-CODEGEN] / [CODEGEN] / FAIL-SIZE lines above');
process.exit(r.status ?? 1);
