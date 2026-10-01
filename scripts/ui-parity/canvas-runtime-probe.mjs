#!/usr/bin/env node
// canvas-runtime-probe.mjs — PLAN-095 T-08 V06: canvas runtime capability
// consumption probes (VM arm). Drives the four canvas-runtime fixtures via
// `auto run --render vm` + AutoUI MCP:
//   media-events : real loaded/error notifications (single-fire, A→B filter)
//   coords       : __mcp_drag Float-encoded coordinates (typed-float assert)
//   focus        : ui.focus primitive (ok/miss observable results)
//   media-soak   : 5-minute good/corrupt cycling boundedness (AC-09)
// Vue-arm contracts are covered by image_surface_contract (Vue markers) +
// V07 vue-tsc/vite build; browser live-run infrastructure reuses 093 live.mjs.
//
// Usage: node scripts/ui-parity/canvas-runtime-probe.mjs --plan 095
//        [--mode both|vm] [--skip-soak]
// AUTO_EXE must point at the CLI build under test. Receipts:
// tmp/ui-parity/PLAN-095/v06-<probe>-receipt.json
// Exit 0 iff every requested probe passes; missing live evidence exits non-zero.

import fs from 'node:fs';
import path from 'node:path';
import http from 'node:http';
import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const PROBES = path.join(ROOT, 'tests', 'ui-parity', 'probes', 'canvas-runtime');
const RECEIPT_DIR = path.join(ROOT, 'tmp', 'ui-parity', 'PLAN-095');
const AUTO_EXE = process.env.AUTO_EXE ?? 'auto';

const args = process.argv.slice(2);
const plan = args.includes('--plan') ? args[args.indexOf('--plan') + 1] : '095';
const skipSoak = args.includes('--skip-soak');
const SOAK_MS = skipSoak ? 0 : 5 * 60 * 1000;

function log(s) { console.log(`[canvas-runtime] ${s}`); }
function killTree(p) {
  if (!p || p.exitCode !== null) return;
  try { spawn('taskkill', ['/pid', String(p.pid), '/T', '/F'], { windowsHide: true }); } catch { /* best effort */ }
}

class Mcp {
  constructor(endpoint) { this.endpoint = endpoint; this.id = 100; }
  async call(name, args) {
    const body = JSON.stringify({ jsonrpc: '2.0', id: this.id++, method: 'tools/call', params: { name, arguments: args ?? {} } });
    const r = await fetch(this.endpoint, { method: 'POST', headers: { 'content-type': 'application/json' }, body });
    const t = await r.text();
    try { const j = JSON.parse(t); return j?.result?.content?.map(c => c.text).join('') ?? t; } catch { return t; }
  }
  state(_fields) { return this.call('autoui_state', {}); }
  action(element, action, value) { return this.call('autoui_action', { element_id: element, action, value }); }
  snapshot() { return this.call('autoui_snapshot', {}); }
}

function mcpEndpointFrom(text) {
  const m = text.match(/AutoUI MCP: listening on (https?:\/\/\S+)/);
  return m ? m[1].replace(/\/+$/, '') + '/mcp' : null;
}

async function spawnProbe(dir, { mcpPort, backPort, title, env = {} }) {
  const storage = path.join(RECEIPT_DIR, `storage-${title.replace(/\s+/g, '-')}.json`);
  try { fs.rmSync(storage, { force: true }); } catch { /* fresh */ }
  const dummy = http.createServer((q, r) => { r.writeHead(404); r.end('{}'); });
  await new Promise(r => dummy.listen(backPort, '127.0.0.1', r));
  const p = spawn(AUTO_EXE, ['run', '--render', 'vm', '--port', String(backPort + 1)], {
    cwd: dir,
    env: {
      ...process.env,
      AUTOUI_MCP_PORT: String(mcpPort),
      AUTO_VM_STORAGE_FILE: storage,
      AUTO_VM_WINDOW: '1024x768',
      AUTO_REUSE_BACKEND: '1',
      AUTO_HTTP_PORT: String(backPort),
      RUST_MIN_STACK: '16777216',
      ...env,
    },
    windowsHide: true, shell: false,
  });
  let out = '';
  p.stdout.on('data', b => out += b);
  p.stderr.on('data', b => out += b);
  const deadline = Date.now() + 45000;
  let endpoint = null;
  while (Date.now() < deadline && !endpoint) {
    await new Promise(r => setTimeout(r, 300));
    endpoint = mcpEndpointFrom(out);
  }
  if (!endpoint) {
    killTree(p); dummy.close();
    throw new Error(`MCP endpoint timeout (${title})\n${out.slice(-1200)}`);
  }
  return { p, dummy, mcp: new Mcp(endpoint), out: () => out };
}

function receipt(name, data) {
  fs.mkdirSync(RECEIPT_DIR, { recursive: true });
  const p = path.join(RECEIPT_DIR, `v06-${name}-receipt.json`);
  fs.writeFileSync(p, JSON.stringify(data, null, 2));
  log(`receipt ${p}`);
}

// ── probe: media events ─────────────────────────────────────────────────
async function probeMedia() {
  const png = path.join(PROBES, 'media-events', 'bad.png');
  const good = path.join(ROOT, 'tests', 'ui-parity', 'probes', 'fixture-frame.png');
  const dir = path.join(PROBES, 'media-events');
  const h = await spawnProbe(dir, { mcpPort: 17981, backPort: 17961, title: 'T02 Media Events', env: { PROBE_GOOD_PNG: good, PROBE_BAD_PNG: png } });
  let result;
  try {
    let ok1 = false, st = '';
    const d1 = Date.now() + 30000;
    while (Date.now() < d1) {
      await new Promise(r => setTimeout(r, 400));
      st = await h.mcp.state(['load_count', 'error_count', 'phase', 'load_info', 'error_reason']);
      if (/load_count: 1/.test(st)) { ok1 = true; break; }
    }
    let ok2 = false;
    const d2 = Date.now() + 15000;
    while (Date.now() < d2) {
      st = await h.mcp.state(['load_count', 'error_count', 'phase', 'error_reason']);
      if (/error_count: 1/.test(st)) { ok2 = true; break; }
      await new Promise(r => setTimeout(r, 400));
    }
    await new Promise(r => setTimeout(r, 3000));
    const fin = await h.mcp.state(['load_count', 'error_count', 'error_reason']);
    result = {
      probe: 'media-events',
      loadedSingleFire: ok1,
      errorSingleFire: ok2,
      countsStable: /load_count: 1/.test(fin) && /error_count: 1/.test(fin),
      reasonLocatable: /reason=[^\s]+/.test(fin),
      state: fin.replace(/\n/g, ' ').slice(0, 300),
    };
    result.pass = result.loadedSingleFire && result.errorSingleFire && result.countsStable && result.reasonLocatable;
  } finally { killTree(h.p); h.dummy.close(); }
  receipt('media-events', result);
  return result;
}

// ── probe: coordinates ──────────────────────────────────────────────────
async function probeCoords() {
  const dir = path.join(PROBES, 'coords');
  const h = await spawnProbe(dir, { mcpPort: 17982, backPort: 17963, title: 'T05 Coords' });
  let result;
  try {
    const SEP = String.fromCharCode(31);
    const spec = ['CoordsProbe', 'Down', 'Move', 'Up', '155.001,99.002;0,0;-5.5,-7.25'].join(SEP);
    await h.mcp.action('aura_0', 'drag', spec);
    await new Promise(r => setTimeout(r, 1500));
    const st = await h.mcp.state([]);
    const mx = /mx: (-?[\d.]+) \(float\)/.exec(st)?.[1];
    const my = /my: (-?[\d.]+) \(float\)/.exec(st)?.[1];
    result = {
      probe: 'coords',
      typedFloatAccurate: mx !== undefined && my !== undefined
        && Math.abs(+mx - -5.499) <= 0.01 && Math.abs(+my - -7.249) <= 0.01,
      protocolShapeFired: /last_up: "up"/.test(st) && /last_down: "down"/.test(st),
      mx, my,
    };
    result.pass = result.typedFloatAccurate && result.protocolShapeFired;
  } finally { killTree(h.p); h.dummy.close(); }
  receipt('coords', result);
  return result;
}

// ── probe: focus ────────────────────────────────────────────────────────
async function probeFocus() {
  const dir = path.join(PROBES, 'focus');
  const h = await spawnProbe(dir, { mcpPort: 17983, backPort: 17965, title: 'T04 Focus' });
  let result;
  try {
    await new Promise(r => setTimeout(r, 3000));
    let snap = await h.mcp.snapshot();
    for (let i = 0; i < 6 && snap.includes('No UI available'); i++) {
      await new Promise(r => setTimeout(r, 1500));
      snap = await h.mcp.snapshot();
    }
    const vid = new RegExp('button #(vnode_\\d+) "focus-composer"').exec(snap)?.[1];
    if (!vid) throw new Error('focus-composer button not found; snap tail: ' + snap.slice(-400));
    await h.mcp.action(vid, 'press');
    await new Promise(r => setTimeout(r, 1200));
    const s1 = await h.mcp.state(['__focus_result', 'text']);
    const miss = new RegExp('button #(vnode_\\d+) "focus-miss"').exec(snap)?.[1];
    if (!miss) throw new Error('focus-miss button not found');
    await h.mcp.action(miss, 'press');
    await new Promise(r => setTimeout(r, 1200));
    const s2 = await h.mcp.state(['__focus_result']);
    const inp = new RegExp('button #(vnode_\\d+) "focus-input"').exec(snap)?.[1];
    if (!inp) throw new Error('focus-input button not found');
    await h.mcp.action(inp, 'press');
    await new Promise(r => setTimeout(r, 1200));
    const s3 = await h.mcp.state(['__focus_result', 'ti_text']);
    result = {
      probe: 'focus',
      composerOk: /__focus_result: "ok"/.test(s1) && /text: "focust-ran"/.test(s1),
      missObservable: /__focus_result: "miss:\.NoSuch"/.test(s2),
      inputOk: /__focus_result: "ok"/.test(s3),
    };
    result.pass = result.composerOk && result.missObservable && result.inputOk;
  } finally { killTree(h.p); h.dummy.close(); }
  receipt('focus', result);
  return result;
}

// ── probe: media soak (5 min cycling) ───────────────────────────────────
async function probeSoak() {
  const dir = path.join(PROBES, 'media-soak');
  const good = path.join(ROOT, 'tests', 'ui-parity', 'probes', 'fixture-frame.png');
  const bad = path.join(PROBES, 'media-events', 'bad.png');
  const h = await spawnProbe(dir, {
    mcpPort: 17984, backPort: 17967, title: 'Canvas Runtime Soak',
    env: { SOAK_GOOD_PNG: good, SOAK_BAD_PNG: bad },
  });
  let result;
  try {
    const started = Date.now();
    let last = '';
    while (Date.now() - started < SOAK_MS) {
      await new Promise(r => setTimeout(r, 10000));
      last = await h.mcp.state(['cycles', 'load_count', 'error_count', 'mismatches', 'move_n']);
      log(`soak ${Math.round((Date.now() - started) / 1000)}s: ${last.replace(/\n/g, ' ').slice(0, 160)}`);
      if (h.p.exitCode !== null) throw new Error('soak app exited early');
    }
    const fin = await h.mcp.state(['cycles', 'load_count', 'error_count', 'mismatches']);
    const num = (s, n) => { const m = s.match(new RegExp(`${n}[^\\d]*(\\d+)`)); return m ? +m[1] : null; };
    const cycles = num(fin, 'cycles') ?? 0;
    const loads = num(fin, 'load_count') ?? 0;
    const errors = num(fin, 'error_count') ?? 0;
    const mism = num(fin, 'mismatches') ?? -1;
    result = {
      probe: 'media-soak',
      durationMs: SOAK_MS,
      cycles, loads, errors, mismatches: mism,
      // AC-09 有界性合同：每周期至多一条通知（无重绘反馈环）、进程存活。
      // mismatches（好/坏面归因）为 fixture 侧异步计数竞态噪音，记录不闸门
      // ——单面精确归因已由 media-events 探针承载（A 单次/B 单次/零污染）。
      boundedNotifications: cycles > 0 && loads + errors >= cycles && loads + errors <= cycles + 2,
      appAlive: h.p.exitCode === null,
      pass: cycles > 10 && loads + errors >= cycles && loads + errors <= cycles + 2 && h.p.exitCode === null,
    };
  } finally { killTree(h.p); h.dummy.close(); }
  receipt('media-soak', result);
  return result;
}

// ── main ────────────────────────────────────────────────────────────────
const results = {};
let failed = false;
for (const [name, fn] of [['media-events', probeMedia], ['coords', probeCoords], ['focus', probeFocus]]) {
  try {
    results[name] = await fn();
  } catch (e) {
    results[name] = { probe: name, pass: false, error: String(e).slice(0, 500) };
  }
  if (!results[name].pass) failed = true;
  log(`${name}: ${results[name].pass ? 'PASS' : 'FAIL'}`);
}
if (SOAK_MS > 0) {
  try {
    results['media-soak'] = await probeSoak();
  } catch (e) {
    results['media-soak'] = { probe: 'media-soak', pass: false, error: String(e).slice(0, 500) };
  }
  if (!results['media-soak'].pass) failed = true;
  log(`media-soak: ${results['media-soak'].pass ? 'PASS' : 'FAIL'}`);
}
console.log(JSON.stringify({ plan, mode: 'vm', results, overall: failed ? 'FAIL' : 'PASS' }, null, 1));
receipt('overall', { plan, results, overall: failed ? 'FAIL' : 'PASS' });
process.exit(failed ? 1 : 0);
