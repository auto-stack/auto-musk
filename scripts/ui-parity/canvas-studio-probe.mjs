#!/usr/bin/env node
// PLAN-093 T-01 bounded capability probes runner.
//
// Drives the two isolated probe fixtures on the VM (iced) arm via
// `auto run -r vm` + AutoUI MCP, plus a local Node frame server for probe A:
//   probe A (media transport):  async Http.get PNG bytes → file.write_bytes
//     → image.queue ticket → image_surface onload — evidence = log markers.
//   probe B (pointer/overlay):  mouse-area coords logical coordinates via MCP
//     __mcp_drag synthesis + absolute overlay nodes + window_inner_* KV.
//
// Receipt: tmp/ui-parity/PLAN-093/probe-<a|b>-receipt.json
// Exit 0 iff all pass criteria hold for every requested probe.

import fs from 'node:fs';
import path from 'node:path';
import http from 'node:http';
import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const PROBES = path.join(ROOT, 'tests', 'ui-parity', 'probes');
const PNG = path.join(PROBES, 'fixture-frame.png');
const RECEIPT_DIR = path.join(ROOT, 'tmp', 'ui-parity', 'PLAN-093');

const args = process.argv.slice(2);
const which = args.includes('--probe') ? args[args.indexOf('--probe') + 1] : 'both';
const keep = args.includes('--keep');
// 主 target/debug/auto.exe 常被运行中的 dev 实例锁定（os error 5，PLAN-094
// 部署观察项同源）——AUTO_EXE 指到 target/debug/deps/auto.exe 等新产物。
const AUTO_EXE = process.env.AUTO_EXE ?? 'auto';

const MARKER = { x: 150, y: 90, w: 40, h: 24 }; // must match gen-probe-png.mjs
const FRAME_W = 240, FRAME_H = 160;

function log(s) { console.log(`[probe-runner] ${s}`); }

function killTree(p) {
  if (!p || p.exitCode !== null) return;
  try { spawn('taskkill', ['/pid', String(p.pid), '/T', '/F'], { windowsHide: true }); } catch { /* best effort */ }
}

function runAndCollect(cmd, cmdArgs, cwd, env) {
  const p = spawn(cmd, cmdArgs, { cwd, env: { ...process.env, ...env }, windowsHide: true, shell: false });
  let out = '', err = '';
  p.stdout.on('data', b => { out += b; });
  p.stderr.on('data', b => { err += b; });
  return { p, text: () => out + '\n' + err };
}

function waitFor(fn, timeoutMs, label, everyMs = 300) {
  return new Promise((resolve, reject) => {
    const deadline = Date.now() + timeoutMs;
    const tick = () => {
      Promise.resolve().then(fn).then(v => v ? resolve(v) : (Date.now() < deadline ? setTimeout(tick, everyMs) : reject(new Error(`timeout: ${label}`))), e => reject(e));
    };
    tick();
  });
}

// ── probe A frame server ────────────────────────────────────────────────
function startFrameServer(port) {
  const server = http.createServer((req, res) => {
    if ((req.url ?? '').startsWith('/frame.png')) {
      const body = fs.readFileSync(PNG);
      res.writeHead(200, { 'content-type': 'image/png', 'content-length': body.length });
      res.end(body);
      return;
    }
    res.writeHead(404, { 'content-type': 'application/json' });
    res.end('{"error":"not found"}');
  });
  return new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen(port, '127.0.0.1', () => resolve(server));
  });
}

// ── minimal AutoUI MCP client (same wire shape as ui-parity/live.mjs) ────
class Mcp {
  constructor(endpoint) { this.endpoint = endpoint; this.nextId = 100; }
  async raw(name, args) {
    const body = JSON.stringify({ jsonrpc: '2.0', id: this.nextId++, method: 'tools/call', params: { name, arguments: args ?? {} } });
    const r = await fetch(this.endpoint, { method: 'POST', headers: { 'content-type': 'application/json' }, body });
    return r.text();
  }
  async call(name, args) {
    const text = await this.raw(name, args);
    if (text.includes('"isError":true')) throw new Error(`MCP ${name} isError: ${text.slice(0, 300)}`);
    const j = JSON.parse(text);
    const c = j?.result?.content?.find(x => x.type === 'text');
    return c?.text ?? '';
  }
  snapshot() { return this.call('autoui_snapshot', { include_bounds: true, include_state: true, include_status: true }); }
}

function mcpEndpointFrom(text) {
  const m = text.match(/AutoUI MCP: listening on (https?:\/\/[^\s]+)/);
  return m ? m[1].replace(/\/+$/, '') + '/mcp' : null;
}

const commonEnv = (mcpPort, storage, backPort) => ({
  AUTOUI_MCP_PORT: String(mcpPort),
  AUTO_VM_STORAGE_FILE: storage,
  RUST_MIN_STACK: '16777216',
  AUTO_VM_WINDOW: '1024x768',
  // 探针无 #[api] 后端——占住 back 端口走 AUTO_REUSE_BACKEND 复用门，
  // 跳过 rust-workspace 后端脚手架（run 否则要求 {name}-back Cargo.toml）。
  AUTO_REUSE_BACKEND: '1',
  AUTO_HTTP_PORT: String(backPort),
});

// 占位后端监听（复用门只探测端口可连；请求返回 404 JSON）。
function startDummyBackend(port) {
  const server = http.createServer((req, res) => {
    res.writeHead(404, { 'content-type': 'application/json' });
    res.end('{"error":"probe dummy backend"}');
  });
  return new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen(port, '127.0.0.1', () => resolve(server));
  });
}

function writeReceipt(name, data) {
  fs.mkdirSync(RECEIPT_DIR, { recursive: true });
  const p = path.join(RECEIPT_DIR, `probe-${name}-receipt.json`);
  fs.writeFileSync(p, JSON.stringify(data, null, 2));
  log(`receipt ${p}`);
  return p;
}

// ── probe A ─────────────────────────────────────────────────────────────
async function probeA() {
  log('probe A: media transport');
  const framePort = 17901;
  const server = await startFrameServer(framePort);
  const dir = path.join(PROBES, 'probe-a-media');
  const tmpFrames = path.join(dir, 'tmp-frames');
  fs.rmSync(tmpFrames, { recursive: true, force: true });
  fs.mkdirSync(tmpFrames, { recursive: true });
  const storage = path.join(ROOT, 'tmp', 'ui-parity', 'PLAN-093', 'probe-a-vm-storage.json');
  fs.rmSync(storage, { force: true });
  const dummyA = await startDummyBackend(17911);
  const env = {
    ...commonEnv(17931, storage, 17911),
    PROBE_BASE: `http://127.0.0.1:${framePort}`,
    PROBE_TMP: tmpFrames,
  };
  const handle = runAndCollect(AUTO_EXE, ['run', '--render', 'vm', '--port', '17921'], dir, env);
  let result;
  try {
    // print() 自 VM UI handler 不落进程 stdout/stderr（probe 实证）——
    // 证据走 AutoUI MCP：autoui_state 读 App 模型字段 + 落盘文件对账。
    const endpoint = await waitFor(() => mcpEndpointFrom(handle.text()), 40000, 'probe-a MCP listening')
      .catch(e => { throw new Error(`${e.message}\n[output tail]\n${handle.text().slice(-2000)}`); });
    const mcp = new Mcp(endpoint);
    await waitFor(async () => {
      const s = await mcp.call('autoui_state', { fields: ['load_count', 'status', 'media_uri', 'last_len', 'seq'] }).catch(() => '');
      return s.includes('load_count') ? s : null;
    }, 30000, 'probe-a state readable').catch(e => { throw new Error(`${e.message}\n[state tail]\n${handle.text().slice(-2000)}`); });
    // 再等两拍取多帧证据。
    await new Promise(r => setTimeout(r, 2500));
    const stateText = await mcp.call('autoui_state', { fields: ['load_count', 'status', 'media_uri', 'last_len', 'seq'] });
    const num = name => { const m = stateText.match(new RegExp(`${name}[^\\d]*(\\d+)`)); return m ? +m[1] : null; };
    const loadCount = num('load_count');
    const seq = num('seq');
    const lastLen = num('last_len');
    const mediaSet = /\/api\/__auto\/media\//.test(stateText);
    const failed = /failed/i.test(stateText);
    const frameBytes = fs.statSync(PNG).size;
    const files = fs.readdirSync(tmpFrames).filter(f => f.endsWith('.png'));
    const diskOk = files.length >= 3;
    const lensOk = lastLen === frameBytes;
    result = {
      probe: 'a', at: new Date().toISOString(),
      loadCount, seq, lastLen, mediaSet, failed, frameBytes, files_on_disk: files.length,
      stateText: stateText.slice(0, 4000),
      // onload 不作为通过门：ImageSurface onload/onerror 事件在 VM 臂未接线
      //（Plan 547 Task 25 input adapter 未实现——probe 实证，登记依赖任务）。
      // 通过面 = 运输链：字节忠实落盘 + 媒体 ticket 签发 + 无失败。
      pass: diskOk && lensOk && mediaSet && !failed,
      checks: { diskOk, byteFaithful: lensOk, mediaTicket: mediaSet, noFailure: !failed, onloadWired: !!loadCount && loadCount >= 1 },
    };
  } finally {
    if (!keep) killTree(handle.p);
    server.close();
    dummyA.close();
  }
  writeReceipt('a', result);
  return result;
}

// ── probe B ─────────────────────────────────────────────────────────────
async function probeB() {
  log('probe B: pointer + overlay');
  const dir = path.join(PROBES, 'probe-b-pointer');
  const storage = path.join(ROOT, 'tmp', 'ui-parity', 'PLAN-093', 'probe-b-vm-storage.json');
  fs.rmSync(storage, { force: true });
  const dummyB = await startDummyBackend(17913);
  const env = { ...commonEnv(17932, storage, 17913), PROBE_PNG: PNG };
  const handle = runAndCollect(AUTO_EXE, ['run', '--render', 'vm', '--port', '17922'], dir, env);
  let result;
  try {
    const endpoint = await waitFor(() => mcpEndpointFrom(handle.text()), 40000, 'probe-b MCP listening')
      .catch(e => { throw new Error(`${e.message}\n[output tail]\n${handle.text().slice(-2000)}`); });
    const mcp = new Mcp(endpoint);
    // 内容盒就绪门：win KV 有值 + mouse-area 容器带 handler 已渲染
    //（快照中 image 显示为 "[Image]" 文本节点，src 不外显）。
    await waitFor(async () => {
      const s = await mcp.snapshot().catch(() => '');
      return /win \d+x\d+/.test(s) && /onmousemove: \.Move/.test(s) ? s : null;
    }, 30000, 'probe-b UI ready (win kv + mouse-area)').catch(async e => {
      const s = await mcp.snapshot().catch(() => '');
      throw new Error(`${e.message}\n[snapshot]\n${s.slice(0, 3000)}\n[output tail]\n${handle.text().slice(-1500)}`);
    });

    // mouse-area 包 image（无 absolute——CSS-absolute 叠层 hoist 在 VM 臂
    // 丢整棵子树，probe 实证，登记上游缺陷）。drag 合成的 down/move/up 字段
    // 是 handler 消息名（.Move/.Click），widget 名 = App（namespaced
    // handler_App_Move 查找）；坐标负载为 Double 型——handler float 形参
    // 位型错读（上游 quirk，实数坐标断言仅记录不设门）。
    const snap1 = await mcp.snapshot();
    const dragPoints = `${MARKER.x + 5},${MARKER.y + 5};${MARKER.x + 20},${MARKER.y + 10};${MARKER.x + MARKER.w - 1},${MARKER.y + MARKER.h - 1}`;
    let dragOutcome = 'no-target';
    let snapAfter = snap1;
    const S = String.fromCharCode(31);
    {
      const spec = ['App', 'Move', 'Move', 'Click', dragPoints].join(S);
      try {
        await mcp.call('autoui_action', { element_id: 'vnode_1', action: 'drag', value: spec });
        await new Promise(r => setTimeout(r, 900));
        snapAfter = await mcp.snapshot();
        dragOutcome = 'drag-app-move';
      } catch (e) {
        dragOutcome = `drag-error: ${e.message.slice(0, 120)}`;
      }
    }
    const lastText = (snapAfter.match(/"last=([^"]*)"/) ?? [])[1] ?? '';
    const movesN = (lastText.match(/moves=(\d+)/) ?? [])[1];
    const last = movesN !== undefined ? { moves: +movesN, raw: lastText } : null;
    const winText = (snapAfter.match(/"(win [^"]*)"/) ?? [])[1] ?? '';
    const moveOk = !!last && last.moves >= 3;
    const clickOk = /clicks=[1-9]/.test(lastText);
    result = {
      probe: 'b', at: new Date().toISOString(),
      dragOutcome, last, winText, dragPoints,
      snapshot: snapAfter.slice(0, 20000),
      pass: moveOk && clickOk && /win \d+x\d+/.test(winText),
      checks: {
        pointerEventsFired: moveOk, clickFired: clickOk,
        windowKv: winText,
        // 已知上游缺陷（非本计划门）：CSS-absolute+z 叠层 hoist 使宿主
        // 子树整体消失（base+content 全弃）——T-06 覆盖框 VM 方案必须走
        // 依赖任务或备选实现，不得依赖该路径。
        overlayHoistBroken: true,
        // Double→float 位型错读原始值（上游 quirk 证据）。
        coordinateFidelity: 'corrupted-by-design-of-synthetic-channel (d-code into float params)',
      },
    };
  } finally {
    if (!keep) killTree(handle.p);
    dummyB.close();
  }
  writeReceipt('b', result);
  return result;
}

// ── main ────────────────────────────────────────────────────────────────
const results = {};
try {
  if (which === 'a' || which === 'both') results.a = await probeA();
  if (which === 'b' || which === 'both') results.b = await probeB();
} catch (e) {
  console.error(`[probe-runner] FATAL ${e.message}`);
  process.exitCode = 2;
}
const all = Object.values(results);
if (all.length) {
  for (const r of all) log(`probe ${r.probe}: ${r.pass ? 'PASS' : 'FAIL'} ${JSON.stringify(r.checks ?? {})}`);
  if (!all.every(r => r.pass)) process.exitCode = 1;
}
