#!/usr/bin/env node
// scripts/demo-focusboard.mjs — PLAN-097 演示驱动入口。
//
//   node scripts/demo-focusboard.mjs preflight [--run-dir D] [--auto-exe P] [--musk-exe P]
//                                              [--ports back=N,mcp=N,front=N] [--keep]
//   （run/verify/report 子命令属 T-08/T-12/T-13，后续增量）
//
// preflight（T-06，V01）：不启动模型任务；静态能力检 + 短 live serve 冒烟，
// 输出明确能力/缺件报告（JSON，落 run-dir/preflight.json），缺件非零退出。
// 检查面（全部为 T-01/T-02 已冻结事实的机械复验）：
//   S1 二进制存在 + 指纹（musk=worktree 自建 release；auto=显式绝对路径——
//      锁当日 auto-lang 构建，PATH 滞后构建是 T-05 误诊根因，硬规则）
//   S2 端口空闲（默认 17841-17843 私有段；占用=显式失败）
//   S3 worktree 无链接（wt-guard 闸门）
//   S4 消费合同 hash 对账（contract.mjs 冻结来源 vs 当前工作树——
//      漂移 = 演示身份失真，硬失败并提示重冻结）
//   S5 examples/focusboard-demo 需求件齐备（T-08 产物）
//   S6 staging 目录可写（D:/autostack/.demo-build/musk-097）
//   L1 live serve 冒烟：私有端口起 serve → /api/health → relay 列表路由 →
//      canvas 状态路由 → owned 关停 + 端口回收
import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { VmSession, listeningPids, decidePortConflict, sha256File } from './demo/vm-session.mjs';
import { CONTRACT_FROZEN } from './demo/contract.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const args = process.argv.slice(2);
const opt = (name, dflt) => { const i = args.indexOf(name); return i >= 0 ? args[i + 1] : dflt; };
const command = args[0] ?? 'preflight';

const RUN_ROOT = opt('--run-dir', `D:/autostack/.demo/musk-097/preflight-${new Date().toISOString().replace(/[-:T.]/g, '').slice(0, 14)}`);
const MUSK_EXE = opt('--musk-exe', path.join(ROOT, 'backend/target/release/musk.exe'));
// 硬规则：auto-exe 必须显式传入（默认拒绝 PATH 兜底——锁当日构建）
const AUTO_EXE = opt('--auto-exe', process.env.AUTO_EXE_DEMO ?? '');
const PORTS = (() => {
  const raw = opt('--ports', 'back=17841,mcp=17842,front=17843');
  const o = {};
  for (const kv of raw.split(',')) { const [k, v] = kv.split('='); o[k.trim()] = Number(v); }
  return o;
})();

const report = { command, at: new Date().toISOString(), runRoot: RUN_ROOT, checks: [], ok: true };
const check = (id, name, ok, detail) => {
  report.checks.push({ id, name, ok, detail });
  console.log(`[preflight] ${ok ? 'PASS' : 'FAIL'} ${id} ${name} — ${detail}`);
  if (!ok) report.ok = false;
};

async function preflight() {
  fs.mkdirSync(RUN_ROOT, { recursive: true });

  // S1 二进制
  const muskOk = fs.existsSync(MUSK_EXE);
  check('S1a', 'musk.exe 存在(worktree 自建 release)', muskOk, MUSK_EXE);
  const autoOk = path.isAbsolute(AUTO_EXE) && fs.existsSync(AUTO_EXE);
  check('S1b', 'auto.exe 显式绝对路径存在(锁当日构建,不用 PATH)', autoOk, AUTO_EXE || '(未提供——传 --auto-exe 或 AUTO_EXE_DEMO)');
  let fingerprints = null;
  if (muskOk && autoOk) {
    const sess = new VmSession({ runRoot: RUN_ROOT, ports: PORTS, muskExe: MUSK_EXE, autoExe: AUTO_EXE, workspaceDir: path.join(RUN_ROOT, 'ws'), projectDir: ROOT });
    fingerprints = sess.fingerprint();
    report.fingerprints = fingerprints;
    check('S1c', '指纹可取(sha256+版本)', true, `auto=${fingerprints.auto_exe.version} musk=${fingerprints.musk_exe.sha256.slice(0, 12)}…`);
  }

  // S2 端口
  for (const [name, port] of Object.entries(PORTS)) {
    const v = decidePortConflict(listeningPids(port));
    check(`S2-${name}`, `端口 ${port} 空闲`, v.ok, v.reason);
  }

  // S3 worktree 无链接
  let guardOut = '';
  try { guardOut = execFileSync('bash', ['D:/autostack/wt-guard.sh', ROOT], { encoding: 'utf8', windowsHide: true }); } catch (e) { guardOut = String(e.stdout ?? e.message); }
  check('S3', 'worktree 无 reparse point(wt-guard)', /clean/.test(guardOut), guardOut.trim().split('\n')[0] ?? '');

  // S4 合同 hash 对账
  const frozen = CONTRACT_FROZEN.sources;
  const drift = [];
  for (const [rel, want] of Object.entries(frozen)) {
    const p = path.join(ROOT, rel);
    if (!fs.existsSync(p)) { drift.push(`${rel} missing`); continue; }
    const got = sha256File(p).slice(0, 16);
    if (got !== want) drift.push(`${rel} ${want}→${got}`);
  }
  // 说明:frozen 为 路径→hash 映射(contract.mjs CONTRACT_FROZEN.sources)
  check('S4', '消费合同 hash 对账(冻结来源 vs 当前树)', drift.length === 0,
    drift.length ? `漂移:${drift.join('; ')}——重冻结或回退后重跑` : '全部一致');

  // S5 需求件
  const reqs = ['examples/focusboard-demo/README.md', 'examples/focusboard-demo/requests/001.md', 'examples/focusboard-demo/requests/002.md', 'examples/focusboard-demo/requests/003.md'];
  const missingReqs = reqs.filter(r => !fs.existsSync(path.join(ROOT, r)));
  check('S5', 'FocusBoard 需求件齐备', missingReqs.length === 0, missingReqs.length ? `缺:${missingReqs.join(',')}` : reqs.join(','));

  // S6 staging 可写
  const staging = 'D:/autostack/.demo-build/musk-097';
  try { fs.mkdirSync(staging, { recursive: true }); fs.accessSync(staging, fs.constants.W_OK); check('S6', 'build staging 可写', true, staging); }
  catch (e) { check('S6', 'build staging 可写', false, String(e.message)); }

  // L1 live serve 冒烟（不起 VM、不发模型任务）
  if (report.ok && muskOk) {
    let sess = null;
    try {
      sess = new VmSession({ runRoot: RUN_ROOT, ports: PORTS, muskExe: MUSK_EXE, autoExe: AUTO_EXE, workspaceDir: path.join(RUN_ROOT, 'ws'), projectDir: ROOT });
      // 只起 serve：直接借用 start() 的前半段不合适——手工起 serve 复用同类
      const { spawn } = await import('node:child_process');
      fs.mkdirSync(path.join(RUN_ROOT, 'home'), { recursive: true });
      fs.mkdirSync(path.join(RUN_ROOT, 'ws'), { recursive: true });
      const serve = spawn(MUSK_EXE, ['serve', '--workdir', path.join(RUN_ROOT, 'ws')], {
        cwd: path.join(RUN_ROOT, 'ws'),
        env: { ...process.env, MUSK_SERVE_PORT: String(PORTS.back), MUSK_CONFIG_DIR: path.join(RUN_ROOT, 'home'), AAID_URL: 'http://127.0.0.1:9', AUTO_EXE: AUTO_EXE, RUST_MIN_STACK: '16777216' },
        windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'],
      });
      const base = `http://127.0.0.1:${PORTS.back}`;
      await (async () => {
        const dl = Date.now() + 30000;
        while (Date.now() < dl) {
          try { if ((await fetch(`${base}/api/health`)).status < 500) return; } catch { }
          await new Promise(r => setTimeout(r, 300));
        }
        throw new Error('health timeout');
      })();
      const runs = await fetch(`${base}/api/forge/relay/runs`);
      const canvas = await fetch(`${base}/api/canvas/status`);
      const liveOk = runs.status < 500 && canvas.status < 500;
      check('L1', 'live serve 冒烟(health/relay/canvas 路由)', liveOk, `relay=${runs.status} canvas=${canvas.status}`);
      execFileSync('taskkill', ['/pid', String(serve.pid), '/T', '/F'], { windowsHide: true, stdio: 'ignore' });
      await new Promise(r => setTimeout(r, 2500));
      const freed = listeningPids(PORTS.back).length === 0;
      check('L2', 'owned 关停 + 端口回收', freed, freed ? `port ${PORTS.back} freed` : `port ${PORTS.back} still held`);
    } catch (e) {
      check('L1', 'live serve 冒烟', false, String(e.message));
      if (sess) await sess.stop();
    }
  } else {
    check('L1', 'live serve 冒烟(跳过:静态检未全过)', false, '先修静态缺件');
  }

  fs.writeFileSync(path.join(RUN_ROOT, 'preflight.json'), JSON.stringify(report, null, 2));
  console.log(`[preflight] report: ${path.join(RUN_ROOT, 'preflight.json')}`);
  console.log(report.ok ? '[preflight] CAPABILITY OK' : '[preflight] MISSING CAPABILITY(缺件非零退出)');
  process.exit(report.ok ? 0 : 1);
}

if (command === 'preflight') {
  await preflight();
} else {
  console.error(`unknown command: ${command}(run/verify/report 属 T-08/T-12/T-13 增量)`);
  process.exit(2);
}
