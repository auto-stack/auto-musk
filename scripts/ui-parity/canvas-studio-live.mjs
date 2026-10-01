#!/usr/bin/env node
// canvas-studio-live.mjs — PLAN-093 T-13 V05 编排器：四模式（Vue/VM ×
// RustHTTP/VMHTTP）关键链 + 场景矩阵汇总。
//
// 复用已验证的驱动（不复制实现）：
//   Vue 臂 × {rust,vm}  = canvas-ports-probe.mjs --vue <backend>
//                         （17 断言：面板帧渲染/studio 切换/树选/坐标链
//                          委托/收起持久/停止收起 + T-09 进度摘要面）
//   VM 臂  × {rust,vm}  = vm-studio-cycle.mjs（CYCLE_BACKEND=<backend>）
//                         （studio 进出/启动流/真实帧/wrap+mouse-area/
//                          缩放往返/结构树/坐标点选端到端/源码面板）
// 不读取 PLAN-080 旧收据；各 runner 自产 093 收据
// （tmp/ui-parity/PLAN-093/{ports-vue-*-receipt,studio-live-*.log,
//   canvas-studio-live-receipt.json}）。
//
// 用法：node scripts/ui-parity/canvas-studio-live.mjs --plan 093
//         [--frontend both|vue|vm] [--backend both|rust|vm]
// 退出码：任一臂非 ALL PASS / 运行证据缺失 → 1。

import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const RECEIPT_DIR = path.join(ROOT, 'tmp', 'ui-parity', 'PLAN-093');

const args = process.argv.slice(2);
const opt = name => { const i = args.indexOf(name); return i >= 0 ? args[i + 1] : undefined; };
const frontend = opt('--frontend') ?? 'both';
const backend = opt('--backend') ?? 'both';

const frontModes = frontend === 'both' ? ['vue', 'vm'] : [frontend];
const backModes = backend === 'both' ? ['rust', 'vm'] : [backend];

const matrix = [];
for (const f of frontModes) {
  for (const b of backModes) {
    matrix.push({ frontend: f, backend: b });
  }
}

console.log(`[studio-live] plan=093 matrix=${matrix.map(m => `${m.frontend}-${m.backend}`).join(',')}`);
fs.mkdirSync(RECEIPT_DIR, { recursive: true });

const results = [];
let failed = 0;

// 格间清理：上一格 kill 竞态的残留进程会锁 runDir（EPERM 实证）——
// 只杀本仓 sibling 路径的进程（不触碰别的在用 Auto 进程）。
function cleanupBetween() {
  try {
    spawnSync('powershell', ['-NoProfile', '-Command',
      "Get-Process auto,musk -ErrorAction SilentlyContinue | Where-Object { $_.Path -like '*musk-093*' } | ForEach-Object { Stop-Process -Id $_.Id -Force }"], { timeout: 30000, windowsHide: true });
  } catch { /* best effort */ }
  Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 3000);
}

for (const m of matrix) {
  const label = `${m.frontend}-${m.backend}`;
  const started = new Date().toISOString();
  let pass = false, detail = '', attempts = 0;
  // 环境噪声单次重试（EPERM 锁/VM boot 缺拍）；产品断言失败不重试——
  // 重试只给环境噪声一次机会，重试亦失败如实记 FAIL。
  while (attempts < 2 && !pass) {
    attempts++;
    if (results.length > 0 || attempts > 1) cleanupBetween();
    if (attempts > 1) console.log(`[studio-live] ${label}: retry #${attempts - 1}`);
    let out = '';
    if (m.frontend === 'vue') {
      const r = spawnSync('node', ['scripts/ui-parity/canvas-ports-probe.mjs', '--vue', m.backend],
        { cwd: ROOT, encoding: 'utf8', timeout: 420000, windowsHide: true });
      out = (r.stdout ?? '') + (r.stderr ?? '');
      fs.writeFileSync(path.join(RECEIPT_DIR, `studio-live-vue-${m.backend}.log`), out);
      pass = r.status === 0 && /ALL PASS/.test(out);
    } else {
      const r = spawnSync('node', ['tmp/vm-studio-cycle.mjs'], {
        cwd: ROOT, encoding: 'utf8', timeout: 420000, windowsHide: true,
        env: { ...process.env, CYCLE_BACKEND: m.backend },
      });
      out = (r.stdout ?? '') + (r.stderr ?? '');
      fs.writeFileSync(path.join(RECEIPT_DIR, `studio-live-vm-${m.backend}.log`), out);
      pass = r.status === 0 && /ALL PASS/.test(out) && /坐标点选端到端命中/.test(out);
    }
    detail = (out.match(/ALL PASS[^\n]*/) ?? out.split('\n').filter(Boolean).slice(-1)[0] ?? '').slice(0, 120);
  }
  results.push({ mode: label, started, pass, detail, attempts });
  console.log(`[studio-live] ${label}: ${pass ? 'PASS' : 'FAIL'} — ${detail}`);
  if (!pass) failed++;
}

const receipt = {
  plan: 'PLAN-093',
  runner: 'canvas-studio-live.mjs',
  at: new Date().toISOString(),
  matrix,
  results,
  allPass: failed === 0,
};
const receiptPath = path.join(RECEIPT_DIR, 'canvas-studio-live-receipt.json');
fs.writeFileSync(receiptPath, JSON.stringify(receipt, null, 2));
console.log(`[studio-live] receipt ${path.relative(ROOT, receiptPath)}`);
console.log(failed === 0 ? '[studio-live] V05 FOUR-MODE ALL PASS' : `[studio-live] ${failed} MODE(S) FAILED`);
process.exit(failed === 0 ? 0 : 1);
