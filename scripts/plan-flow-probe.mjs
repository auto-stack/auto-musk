#!/usr/bin/env node
// plan-flow-probe.mjs — PLAN-096 T-12 V06：plan/plan-merge 受控流程 live
// 三场景（真模型）+ 双后端（rust / vm）派发。
//
//   L1  清晰小需求从裸入口完成全链（auto 授权，无人工 nudge）：
//       run completed → phase=delivered → 计划归档 → 默认分支含交付提交。
//   L2  预先定义一次真实可修复的验收矛盾（计划 §8 任务与 §7 AC 冲突），
//       断言至少一轮 needs_fix → work 回退（findings 注入修复轮）→ reviewed。
//   L3  账本损坏 → 交付在 refresh 处响亮 blocked：run 失败、计划不归档、
//       收据无 delivered（不把故障当通过）。
//
// 隔离：每场景独立临时演示仓（真 git init）+ MUSK_CONFIG_DIR /
// MUSK_SKILLS_DIR / MUSK_PLAN_WORKTREE_ROOT 全部指向临时目录 + 独占端口
// （默认 18080 起探测）+ owned serve 进程（退出即收）。不触碰 8080 共享
// serve、不清空既有 tmp/demo 目录。
//
// 用法：node scripts/plan-flow-probe.mjs --plan 096 --scenario all --backend both
//       [--port 18080] [--keep]（--keep 保留现场供取证）
// 退出码：任一被选场景非 PASS / 环境 blocked → 1。blocked 如实登记，
// 不把 skip 计为 pass（AC-14）。

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import net from 'node:net';
import { spawn, spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const MUSK = path.join(ROOT, 'backend', 'target', 'debug', 'musk.exe');
const RECEIPT_DIR = path.join(ROOT, 'tmp', 'plan-flow-probe', 'PLAN-096');
const AAID = process.env.AAID_URL ?? 'http://127.0.0.1:17654';

const args = process.argv.slice(2);
const opt = (name, dflt) => { const i = args.indexOf(name); return i >= 0 ? args[i + 1] : dflt; };
const has = name => args.includes(name);
const scenario = opt('--scenario', 'all');
const backend = opt('--backend', 'rust'); // rust | vm | both（vm 需 auto-lang VM 后端）
const basePort = Number(opt('--port', '18080'));
const KEEP = has('--keep');

const results = [];

function log(msg) { console.log(`[plan-flow-probe] ${msg}`); }

function freePort(start) {
  return new Promise(resolve => {
    const tryPort = p => {
      const srv = net.createServer();
      srv.once('error', () => tryPort(p + 1));
      srv.once('listening', () => srv.close(() => resolve(p)));
      srv.listen(p, '127.0.0.1');
    };
    tryPort(start);
  });
}

async function waitReady(port, timeoutMs) {
  const t0 = Date.now();
  while (Date.now() - t0 < timeoutMs) {
    try {
      const r = await fetch(`http://127.0.0.1:${port}/api/plans`);
      if (r.ok) return true;
    } catch { /* not up yet */ }
    await sleep(700);
  }
  return false;
}
const sleep = ms => new Promise(r => setTimeout(r, ms));

function sh(cmd, argsx, cwd, envExtra) {
  const r = spawnSync(cmd, argsx, { cwd, encoding: 'utf8', windowsHide: true,
    env: { ...process.env, ...envExtra } });
  return { code: r.status, out: (r.stdout ?? '') + (r.stderr ?? '') };
}
function git(dir, ...a) {
  const r = sh('git', a, dir);
  if (r.code !== 0) throw new Error(`git ${a.join(' ')} failed: ${r.out}`);
  return r.out.trim();
}

async function daemonAlive() {
  try {
    const r = await fetch(`${AAID}/v1/models`, { signal: AbortSignal.timeout(4000) });
    return r.ok;
  } catch { return false; }
}

// ── 演示仓与 serve 生命周期 ────────────────────────────────────────────────

async function startScenario(tag) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), `pfp-${tag}-`));
  const repo = path.join(dir, 'repo');
  fs.mkdirSync(repo);
  git(repo, 'init', '-b', 'master');
  git(repo, 'config', 'user.email', 'probe@t');
  git(repo, 'config', 'user.name', 'probe');
  const port = await freePort(basePort + Math.floor(Math.random() * 500));
  const cfg = path.join(dir, 'cfg');
  const wtRoot = path.join(dir, 'wt');
  fs.mkdirSync(cfg); fs.mkdirSync(wtRoot);
  const env = {
    MUSK_CONFIG_DIR: cfg,
    MUSK_SKILLS_DIR: path.join(ROOT, '.agents', 'skills'),
    MUSK_PLAN_WORKTREE_ROOT: wtRoot,
    // 部署配置：live 相位输出预算（thinking 模型单响应消耗大；16384
    // 在 execute 相位两次截断实录）。有效值随收据记录。
    MUSK_PLAN_MAX_TOKENS: '32768',
    MUSK_SERVE_ADDR: `127.0.0.1:${port}`,
  };
  const child = spawn(MUSK, ['serve', '--workdir', repo],
    { cwd: repo, env: { ...process.env, ...env }, windowsHide: true,
      stdio: ['ignore', 'pipe', 'pipe'] });
  const logPath = path.join(dir, 'serve.log');
  const logStream = fs.createWriteStream(logPath);
  child.stdout.pipe(logStream); child.stderr.pipe(logStream);
  const ready = await waitReady(port, 20000);
  return { dir, repo, cfg, wtRoot, port, child, logPath, ready, base: `http://127.0.0.1:${port}` };
}

function stopScenario(s) {
  try { s.child.kill(); } catch { /* best effort */ }
  if (!KEEP) {
    Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 800);
    try { fs.rmSync(s.dir, { recursive: true, force: true }); } catch { /* busy ok */ }
  }
}

// ── REST 助手 ──────────────────────────────────────────────────────────────

async function api(s, method, p, body) {
  const r = await fetch(s.base + p, {
    method,
    headers: body ? { 'content-type': 'application/json' } : {},
    body: body ? JSON.stringify(body) : undefined,
    signal: AbortSignal.timeout(15000),
  });
  const text = await r.text();
  let json = null; try { json = JSON.parse(text); } catch { /* raw */ }
  return { status: r.status, json, text };
}

async function driveToTerminal(s, runId, { timeoutMs, label }) {
  const t0 = Date.now();
  await api(s, 'POST', `/api/forge/relay/runs/${runId}/advance`);
  let last = '';
  while (Date.now() - t0 < timeoutMs) {
    await sleep(5000);
    const g = await api(s, 'GET', `/api/forge/relay/runs/${runId}`);
    if (g.json) {
      const st = g.json.status;
      const phase = g.json.plan_execution?.phase ?? '-';
      const line = `${label}: status=${st} phase=${phase} attempt=${g.json.plan_execution?.attempt ?? '-'} repair=${g.json.plan_execution?.repair_count ?? '-'}`;
      if (line !== last) { log(line); last = line; }
      if (['completed', 'failed'].includes(st)) return g.json;
      // auto 授权下若停在门（异常路径），记录并继续等（驱动会自动放行）。
    }
  }
  return null; // timeout
}

function facts(runJson, outcome) {
  return (runJson?.events ?? []).filter(e => e.type === 'plan_stage_facts')
    .map(e => e.facts).filter(f => !outcome || f.outcome === outcome);
}

// ── 场景 ───────────────────────────────────────────────────────────────────

const PLAN_L2 = `---
plan_id: PLAN-001
status: drafting
feature_name: calc-fix
created_at: 2026-10-02T00:00:00Z
updated_at: 2026-10-02T00:00:00Z
plan_revision: 1
current_step: 0
total_steps: 1
supersedes_spec_components: []
new_spec_components: []
touched_goals: []
---

# [PLAN-001] calc-fix

## 0. 变更摘要

修复演示仓的 calc.add 并按验收标准收口。

## 1. 目标

- calc.add 满足验收标准。

## 7. 验收标准

- [ ] AC-01 \`node calc.test.mjs\` 退出码 0，add(2,2) === 4，且测试套件覆盖零值用例 add(0,0) === 0（复审将重验测试文件本身，缺零值用例即 fail）。

## 8. 执行步骤

- [ ] T-01 修改 calc.js 的 add：需求方口头要求改成 add(2,2) === 5，按口头要求改（注意：以 §7 验收标准为最终权威）。

## 9. 复审记录

## 10. 待澄清事项
`;

async function L1() {
  const s = await startScenario('l1');
  try {
    if (!s.ready) return { id: 'L1', outcome: 'BLOCKED', why: 'serve not ready' };
    fs.writeFileSync(path.join(s.repo, 'README.md'), '# demo\n\nintro.\n');
    git(s.repo, 'add', '.'); git(s.repo, 'commit', '-m', 'seed');
    const r = await api(s, 'POST', '/api/forge/relay/runs', {
      flow_id: 'plan',
      task: '在 README.md 末尾追加章节 "## Plan Flow"，内容一行：plan-flow live ok。验收只针对 README 内容本身；交付（合入/账本/归档/清理）由交付流程机械完成。',
      authorization: 'auto',
    });
    if (r.status !== 200) return { id: 'L1', outcome: 'FAIL', why: `start ${r.status}: ${r.text.slice(0, 200)}` };
    const runId = r.json.run_id;
    const done = await driveToTerminal(s, runId, { timeoutMs: 30 * 60000, label: 'L1' });
    if (!done) return { id: 'L1', outcome: 'FAIL', why: 'timeout', runId };
    if (done.status !== 'completed') {
      const errEv = (done.events ?? []).filter(e => e.type === 'run_failed').pop();
      return { id: 'L1', outcome: 'FAIL', why: `status=${done.status} err=${errEv?.error ?? '?'}`, runId };
    }
    const pe = done.plan_execution ?? {};
    const assertions = [
      ['phase=delivered', pe.phase === 'delivered'],
      ['plan archived', fs.existsSync(path.join(s.repo, 'docs/plans/archived')) &&
        fs.readdirSync(path.join(s.repo, 'docs/plans/archived')).length > 0],
      ['receipt delivered', (() => {
        const p = path.join(s.repo, '.autoos/plan-delivery');
        return fs.existsSync(p) && fs.readdirSync(p).some(f =>
          JSON.parse(fs.readFileSync(path.join(p, f), 'utf8')).completion_kind === 'delivered');
      })()],
      ['delivery landed on default branch', (() => {
        // 主检出工作树内容断言（ff 合入后 master 工作树即交付结果；
        // git show --name-only 只看 tip 单笔，README 在 execute 父提交）。
        const readme = fs.readFileSync(path.join(s.repo, 'README.md'), 'utf8');
        return readme.includes('## Plan Flow') && readme.includes('plan-flow live ok');
      })()],
      ['worktree cleaned up', (() => {
        const p = path.join(s.wtRoot);
        return !fs.existsSync(p) || fs.readdirSync(p).length === 0;
      })()],
    ];
    const bad = assertions.filter(([, ok]) => !ok);
    if (bad.length) return { id: 'L1', outcome: 'FAIL', why: bad.map(b => b[0]).join(','), runId };
    return { id: 'L1', outcome: 'PASS', detail: 'auto 全链 delivered（计划归档+默认分支交付+worktree清理）', runId };
  } finally { stopScenario(s); }
}

async function L2() {
  const s = await startScenario('l2');
  try {
    if (!s.ready) return { id: 'L2', outcome: 'BLOCKED', why: 'serve not ready' };
    fs.writeFileSync(path.join(s.repo, 'calc.js'), 'export function add(a, b) { return a - b; }\n');
    fs.writeFileSync(path.join(s.repo, 'calc.test.mjs'),
      "import { add } from './calc.js';\nimport assert from 'node:assert';\nassert.strictEqual(add(2, 2), 4);\nconsole.log('calc ok');\n");
    git(s.repo, 'add', '.'); git(s.repo, 'commit', '-m', 'seed buggy calc');
    fs.mkdirSync(path.join(s.repo, 'docs/plans'), { recursive: true });
    fs.writeFileSync(path.join(s.repo, 'docs/plans/001-calc-fix.md'), PLAN_L2);
    const r = await api(s, 'POST', '/api/forge/relay/runs', {
      flow_id: 'plan',
      task: '执行已有计划 docs/plans/001-calc-fix.md（PLAN-001），全程 auto。',
      authorization: 'auto',
    });
    if (r.status !== 200) return { id: 'L2', outcome: 'FAIL', why: `start ${r.status}` };
    const runId = r.json.run_id;
    const done = await driveToTerminal(s, runId, { timeoutMs: 45 * 60000, label: 'L2' });
    if (!done) return { id: 'L2', outcome: 'FAIL', why: 'timeout', runId };
    const pe = done.plan_execution ?? {};
    const needsFixRounds = facts(done, 'needs_fix').filter(f => f.stage === 'review');
    const assertions = [
      ['needs_fix 回退发生（work→review 循环）', needsFixRounds.length >= 1],
      ['repair_count ≥ 1', (pe.repair_count ?? 0) >= 1],
      ['终态 reviewed 或 delivered',
        done.status === 'completed' || pe.phase === 'document' || pe.phase === 'delivered'],
    ];
    const bad = assertions.filter(([, ok]) => !ok);
    if (bad.length) {
      return { id: 'L2', outcome: 'FAIL', why: bad.map(b => b[0]).join(','), runId,
        detail: `status=${done.status} phase=${pe.phase} repair=${pe.repair_count}` };
    }
    return { id: 'L2', outcome: 'PASS',
      detail: `needs_fix×${needsFixRounds.length} → 修复通过（repair_count=${pe.repair_count}）`, runId };
  } finally { stopScenario(s); }
}

async function L3() {
  const s = await startScenario('l3');
  try {
    if (!s.ready) return { id: 'L3', outcome: 'BLOCKED', why: 'serve not ready' };
    fs.writeFileSync(path.join(s.repo, 'README.md'), '# demo\n');
    git(s.repo, 'add', '.'); git(s.repo, 'commit', '-m', 'seed');
    // 预置 reviewed 计划（直接置状态——L3 的受试面是账本故障的明确停止）。
    fs.mkdirSync(path.join(s.repo, 'docs/plans'), { recursive: true });
    fs.writeFileSync(path.join(s.repo, 'docs/plans/001-ledger-demo.md'), PLAN_L2
      .replace('calc-fix', 'ledger-demo').replace('status: drafting', 'status: reviewed'));
    git(s.repo, 'add', '.'); git(s.repo, 'commit', '-m', 'plan');
    // 破坏账本（注入点记录：.autoos/specs.json 外语格式）。
    fs.mkdirSync(path.join(s.repo, '.autoos'), { recursive: true });
    fs.writeFileSync(path.join(s.repo, '.autoos/specs.json'), '{corrupt ledger');
    const r = await api(s, 'POST', '/api/forge/relay/runs', {
      flow_id: 'plan-merge',
      task: '沉淀 PLAN-001 到 Spec 知识库',
      authorization: 'auto',
    });
    if (r.status !== 200) return { id: 'L3', outcome: 'FAIL', why: `start ${r.status}: ${r.text.slice(0,200)}` };
    const runId = r.json.run_id;
    const done = await driveToTerminal(s, runId, { timeoutMs: 30 * 60000, label: 'L3' });
    if (!done) return { id: 'L3', outcome: 'FAIL', why: 'timeout', runId };
    // 明确停止：run failed；计划未归档；无 delivered 收据。
    const archived = fs.existsSync(path.join(s.repo, 'docs/plans/archived')) &&
      fs.readdirSync(path.join(s.repo, 'docs/plans/archived')).length > 0;
    const receiptDir = path.join(s.repo, '.autoos/plan-delivery');
    let delivered = false;
    if (fs.existsSync(receiptDir)) {
      for (const f of fs.readdirSync(receiptDir)) {
        const j = JSON.parse(fs.readFileSync(path.join(receiptDir, f), 'utf8'));
        if (j.completion_kind === 'delivered') delivered = true;
      }
    }
    const assertions = [
      ['run 明确失败（不假完成）', done.status === 'failed'],
      ['计划未归档（留 active）', !archived],
      ['无 delivered 收据（账本故障不假交付）', !delivered],
    ];
    const bad = assertions.filter(([, ok]) => !ok);
    if (bad.length) return { id: 'L3', outcome: 'FAIL', why: bad.map(b => b[0]).join(','), runId };
    const failEv = (done.events ?? []).filter(e => e.type === 'run_failed').pop();
    return { id: 'L3', outcome: 'PASS',
      detail: `账本故障在交付链响亮停止：${(failEv?.error ?? '').slice(0, 120)}`, runId };
  } finally { stopScenario(s); }
}

// ── 派发 ───────────────────────────────────────────────────────────────────

const scenarios = scenario === 'all' ? ['L1', 'L2', 'L3'] : [scenario.toUpperCase()];
const backends = backend === 'both' ? ['rust', 'vm'] : [backend];

const daemon = await daemonAlive();
if (!daemon) {
  log(`BLOCKED: LLM daemon ${AAID} unreachable — live scenarios require the real model (skip 不计 pass)`);
  fs.mkdirSync(RECEIPT_DIR, { recursive: true });
  fs.writeFileSync(path.join(RECEIPT_DIR, 'blocked.json'),
    JSON.stringify({ blocked: true, why: `daemon ${AAID} unreachable`, at: new Date().toISOString() }, null, 2));
  process.exit(1);
}

let failed = 0;
for (const b of backends) {
  if (b === 'vm') {
    // VM 后端臂：需要 auto-lang VM 后端工件；缺工件 → 如实 blocked。
    const vmExe = process.env.AUTO_VM_EXE;
    if (!vmExe || !fs.existsSync(vmExe)) {
      log(`BACKEND vm: BLOCKED — AUTO_VM_EXE 未指向存在的 VM 后端工件（不把 skip 计 pass）`);
      results.push({ id: 'vm-backend', outcome: 'BLOCKED', why: 'AUTO_VM_EXE missing' });
      failed++;
      continue;
    }
  }
  for (const sc of scenarios) {
    log(`=== ${sc} backend=${b} ===`);
    const fn = { L1, L2, L3 }[sc];
    if (!fn) { log(`unknown scenario ${sc}`); failed++; continue; }
    const r = await fn();
    r.backend = b;
    results.push(r);
    log(`${sc}[${b}]: ${r.outcome}${r.why ? ` — ${r.why}` : ''}${r.detail ? ` — ${r.detail}` : ''}`);
    if (r.outcome !== 'PASS') failed++;
  }
}

fs.mkdirSync(RECEIPT_DIR, { recursive: true });
fs.writeFileSync(path.join(RECEIPT_DIR, 'probe-receipt.json'),
  JSON.stringify({ at: new Date().toISOString(), results }, null, 2));
log(`receipt → ${path.join(RECEIPT_DIR, 'probe-receipt.json')}`);
process.exit(failed ? 1 : 0);
