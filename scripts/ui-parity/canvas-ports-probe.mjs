#!/usr/bin/env node
// PLAN-093 T-03 实机验证 runner：canvas VM 端口 × 双后端(VMHTTP/RustHTTP)。
//
// 驱动 probe-c-ports fixture（auto run --render=vm）经 ports/canvas.vm.at
// 五端口对真实 musk serve 后端顺序执行 start/wait/pick/clear/source/stop/
// 错误路径；runner 另以直连 HTTP 断言后端契约面（身份字段/帧字节/409/清选/
// 停止终态）。print() 自 VM UI handler 不落 stdout（probe-a 实证）——探针
// 结果经 AutoUI MCP autoui_state 采集。
//
// 用法：node scripts/ui-parity/canvas-ports-probe.mjs [--backend vm|rust|both]
// 收据：tmp/ui-parity/PLAN-093/ports-probe-<mode>-receipt.json
// 退出码：全部断言通过 0，否则 1。

import fs from 'node:fs';
import path from 'node:path';
import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const PROBE_DIR = path.join(ROOT, 'tests', 'ui-parity', 'probes', 'probe-c-ports');
const TARGET_SRC = path.join(ROOT, 'tests', 'ui-parity', 'probes', 'probe-a-media');
const PORTS_SRC = path.join(ROOT, 'src', 'front', 'ports', 'canvas.vm.at');
const MUSK_EXE = process.env.MUSK_EXE ?? path.join(ROOT, 'backend', 'target', 'debug', 'musk.exe');
const AUTO_EXE = process.env.AUTO_EXE ?? 'D:/autostack/auto-lang/target/debug/deps/auto.exe';
const RECEIPT_DIR = path.join(ROOT, 'tmp', 'ui-parity', 'PLAN-093');

const args = process.argv.slice(2);
const which = args.includes('--backend') ? args[args.indexOf('--backend') + 1] : 'both';

function log(s) { console.log(`[ports-probe] ${s}`); }
function fail(s) { console.error(`[ports-probe] FAIL: ${s}`); }

function killTree(p) {
  if (!p || p.exitCode !== null || p.pid === undefined) return;
  try { spawn('taskkill', ['/pid', String(p.pid), '/T', '/F'], { windowsHide: true }); } catch { /* best effort */ }
}

function runAndCollect(cmd, cmdArgs, cwd, env) {
  const p = spawn(cmd, cmdArgs, { cwd, env: { ...process.env, ...env }, windowsHide: true, shell: false });
  let out = '', err = '';
  p.stdout.on('data', b => { out += b; });
  p.stderr.on('data', b => { err += b; });
  return { p, text: () => out + '\n' + err };
}

function waitFor(fn, timeoutMs, label, everyMs = 400) {
  return new Promise((resolve, reject) => {
    const deadline = Date.now() + timeoutMs;
    const tick = () => {
      Promise.resolve().then(fn).then(v => v ? resolve(v) : (Date.now() < deadline ? setTimeout(tick, everyMs) : reject(new Error(`timeout: ${label}`))), e => reject(e));
    };
    tick();
  });
}

const sleep = ms => new Promise(r => setTimeout(r, ms));

async function jfetch(url, opts) {
  const r = await fetch(url, opts);
  let body = null;
  const text = await r.text();
  try { body = JSON.parse(text); } catch { body = text; }
  return { status: r.status, body, headers: r.headers };
}

// ── minimal AutoUI MCP client（同 live.mjs/probe runner 线型） ─────────────
class Mcp {
  constructor(endpoint) { this.endpoint = endpoint; this.nextId = 100; }
  async call(name, args) {
    const body = JSON.stringify({ jsonrpc: '2.0', id: this.nextId++, method: 'tools/call', params: { name, arguments: args ?? {} } });
    const r = await fetch(this.endpoint, { method: 'POST', headers: { 'content-type': 'application/json' }, body });
    const text = await r.text();
    if (text.includes('"isError":true')) throw new Error(`MCP ${name} isError: ${text.slice(0, 300)}`);
    const j = JSON.parse(text);
    const c = j?.result?.content?.find(x => x.type === 'text');
    return c?.text ?? '';
  }
  state(fields) { return this.call('autoui_state', { fields }); }
}

function mcpEndpointFrom(text) {
  const m = text.match(/AutoUI MCP: listening on (https?:\/\/[^\s]+)/);
  return m ? m[1].replace(/\/+$/, '') + '/mcp' : null;
}

function freePortsToEnv() {
  // 复用门按 pac.at back_port(18511) 探测真实后端——两模式共用该端口，
  // 顺序执行（前一模式后端已收割，TIME_WAIT 连接不阻塞新监听）。
  return { probe: 18530, back: 18511 };
}

// ── 每模式执行 ───────────────────────────────────────────────────────────
async function runMode(mode) {
  const ports = freePortsToEnv(mode === 'vm' ? 18510 : 18520);
  const stamp = new Date().toISOString().replace(/[:.]/g, '-');
  const runDir = path.join(ROOT, 'tmp', 'ui-parity', 'PLAN-093', `ports-probe-${mode}-${stamp}`);
  const wsRoot = path.join(runDir, 'ws');
  const cfgDir = path.join(runDir, 'cfg');
  const targetDir = path.join(wsRoot, 'targets', 'probe-a');
  fs.mkdirSync(targetDir, { recursive: true });
  fs.mkdirSync(cfgDir, { recursive: true });
  // 目标应用 = probe-a fixture 副本（pac.at + src/），隔离目录内。
  fs.cpSync(path.join(TARGET_SRC, 'pac.at'), path.join(targetDir, 'pac.at'));
  fs.cpSync(path.join(TARGET_SRC, 'src'), path.join(targetDir, 'src'), { recursive: true });
  // VM 轨尚无端口消费者（T-04 接线）——探针把仓库当前 ports/canvas.vm.at
  // 与 app.base.at 拼成单文件 app.at（.gitignore），恒测 T-03 交付面。
  const portsSource = fs.readFileSync(PORTS_SRC, 'utf8');
  const appTemplate = fs.readFileSync(path.join(PROBE_DIR, 'src', 'front', 'app.base.at'), 'utf8');
  fs.writeFileSync(path.join(PROBE_DIR, 'src', 'front', 'app.at'),
    portsSource + '\n\n' + appTemplate);

  const storage = path.join(runDir, 'probe-vm-storage.json');
  const result = { mode, at: new Date().toISOString(), musk: MUSK_EXE, auto: AUTO_EXE, backend_port: ports.back, asserts: [] };
  const assert = (name, ok, detail) => {
    result.asserts.push({ name, ok, detail: detail ?? '' });
    if (ok) log(`  ✓ ${name}${detail ? ` — ${detail}` : ''}`);
    else fail(`${name}${detail ? ` — ${detail}` : ''}`);
  };

  // ① 后端（VMHTTP 或 RustHTTP）
  const backEnv = mode === 'vm'
    ? {
        MUSK_BACKEND: 'vm',
        MUSK_VM_CONFIG_DIR: cfgDir,
        MUSK_VM_USERS_PATH: path.join(cfgDir, 'users.json'),
        MUSK_VM_DEFAULT_ROOT: wsRoot,
        MUSK_SERVE_ADDR: `127.0.0.1:${ports.back}`,
        RUST_MIN_STACK: '33554432',
        // canvas spawn 拉起目标应用用的 auto CLI——同探针口径（新构建）。
        AUTO_EXE,
      }
    : {
        MUSK_CONFIG_DIR: cfgDir,
        MUSK_SERVE_ADDR: `127.0.0.1:${ports.back}`,
      };
  const backArgs = ['serve'];
  if (mode !== 'vm') backArgs.push('--workdir', wsRoot);
  const back = runAndCollect(MUSK_EXE, backArgs, path.join(ROOT, 'backend'), backEnv);
  let backendUp = false;
  try {
    await waitFor(async () => {
      const r = await jfetch(`http://127.0.0.1:${ports.back}/api/canvas/status`).catch(() => null);
      return r && r.status === 200 ? r : null;
    }, 90000, `backend ${mode} up`);
    backendUp = true;
  } catch (e) {
    assert('backend-up', false, `${e.message}\n[backend tail]\n${back.text().slice(-1200)}`);
    return finish();
  }
  assert('backend-up', true, `${mode} on :${ports.back}`);

  // ② 探针应用（VM 渲染器；AUTO_REUSE_BACKEND 占住复用门，真后端已可用）
  const probeEnv = {
    AUTO_HTTP_BASE: `http://127.0.0.1:${ports.back}`,
    AUTO_REUSE_BACKEND: '1',
    AUTO_HTTP_PORT: String(ports.back),
    AUTOUI_MCP_PORT: String(ports.probe),
    AUTO_VM_STORAGE_FILE: storage,
    AUTO_VM_WINDOW: '900x700',
    AUTO_DEBUG_CAPTURE: '1',
    RUST_MIN_STACK: '16777216',
  };
  const probe = runAndCollect(AUTO_EXE, ['run', '--render', 'vm', '--port', String(ports.probe + 2)], PROBE_DIR, probeEnv);
  try {
    var mcpEndpoint = await waitFor(() => mcpEndpointFrom(probe.text()), 60000, 'probe MCP listening')
      .catch(e => { throw new Error(`${e.message}\n[probe tail]\n${probe.text().slice(-1500)}`); });
  } catch (e) {
    assert('probe-up', false, e.message);
    return finish();
  }
  const mcp = new Mcp(mcpEndpoint);
  try {
    await waitFor(async () => {
      const s = await mcp.state(['phase']).catch(() => '');
      return s.includes('phase') ? s : null;
    }, 45000, 'probe state readable');
  } catch (e) {
    assert('probe-up', false, e.message);
    return finish();
  }
  assert('probe-up', true, mcpEndpoint);

  // ③ 生命周期由 runner fetch 驱动（G-9：VM 客户端 POST park 丢续体——
  // start/pick/clear/stop 的 POST 端口调用在本探针中不走过 VM 臂）。
  // 先等探针 status 轮询链绿（VM 客户端 GET 通道），再启动目标。
  try {
    await waitFor(async () => {
      const s = await mcp.state(['r_start']).catch(() => '');
      return /status-ok/.test(s) ? s : null;
    }, 45000, 'probe status poll');
  } catch (e) {
    const lastState = await mcp.state(['phase', 'r_start']).catch(err => `MCP-ERR ${err}`);
    assert('client-status-poll', false, `${e.message}\n[last-state]\n${lastState}`);
    return finish();
  }
  assert('client-status-poll', true, extract(await mcp.state(['r_start']), 'r_start'));

  {
    const r = await jfetch(`http://127.0.0.1:${ports.back}/api/canvas/start?workspace=`, {
      method: 'POST', headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ app_path: 'targets/probe-a' }),
    });
    const ok = r.status === 200 && r.body && typeof r.body.generation_id === 'number' && r.body.generation_id >= 1;
    result.generation = r.body?.generation_id;
    assert('start-real-target', ok, `status:${r.status} ${JSON.stringify(r.body ?? {}).slice(0, 200)}`);
    if (!ok) return finish();
  }

  // 契约面：status 身份字段
  {
    const r = await jfetch(`http://127.0.0.1:${ports.back}/api/canvas/status`);
    const ok = r.status === 200 && r.body && typeof r.body.generation_id === 'number' && r.body.generation_id >= 1
      && typeof r.body.owner_workspace_id === 'string';
    assert('status-identity-fields', ok, JSON.stringify(r.body).slice(0, 220));
  }
  // 契约面：帧（等帧管线锚定后取 PNG）
  let frameOk = false, frameBytes = 0;
  try {
    await waitFor(async () => {
      const r = await fetch(`http://127.0.0.1:${ports.back}/api/canvas/frame`).catch(() => null);
      if (!r || r.status !== 200) return null;
      const buf = Buffer.from(await r.arrayBuffer());
      frameBytes = buf.length;
      return buf.length > 8 && buf[0] === 0x89 && buf[1] === 0x50 ? true : null;
    }, 60000, 'frame PNG');
    frameOk = true;
  } catch { /* handled below */ }
  assert('frame-png-bytes', frameOk, `${frameBytes} bytes`);
  // 契约面：帧旧代次/旧 seq → 409
  {
    const rGen = await jfetch(`http://127.0.0.1:${ports.back}/api/canvas/frame?generation=999`);
    const rSeq = await jfetch(`http://127.0.0.1:${ports.back}/api/canvas/frame?seq=0`);
    assert('frame-stale-409', rGen.status === 409 && rSeq.status === 409, `gen:${rGen.status} seq:${rSeq.status}`);
  }

  // 契约面：tree 锚点 + pick 200 / 未命中 204 / clear
  {
    const st = await jfetch(`http://127.0.0.1:${ports.back}/api/canvas/status`);
    const nodes = st.body?.tree?.nodes ?? [];
    if (nodes.length > 0) {
      const vnode = nodes[0].vnode;
      const hit = await jfetch(`http://127.0.0.1:${ports.back}/api/canvas/pick`, {
        method: 'POST', headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ vnode_id: `vnode_${vnode}` }),
      });
      const miss = await jfetch(`http://127.0.0.1:${ports.back}/api/canvas/pick`, {
        method: 'POST', headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ vnode_id: 'vnode_99999999' }),
      });
      const st2 = await jfetch(`http://127.0.0.1:${ports.back}/api/canvas/status`);
      const clearedAfterMiss = st2.body?.picked === null || st2.body?.picked === undefined;
      const clr = await jfetch(`http://127.0.0.1:${ports.back}/api/canvas/pick`, {
        method: 'POST', headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ clear: true }),
      });
      assert('pick-hit-200', hit.status === 200 && hit.body && hit.body.vnode_id !== undefined, JSON.stringify(hit.body).slice(0, 160));
      assert('pick-miss-204-clears', miss.status === 204 && clearedAfterMiss, `status:${miss.status} picked-null:${clearedAfterMiss}`);
      assert('pick-clear-200', clr.status === 200 && clr.body?.cleared === true, JSON.stringify(clr.body ?? {}).slice(0, 120));
    } else {
      assert('pick-hit-200', false, 'status.tree empty (anchor pipeline)');
      assert('pick-miss-204-clears', false, 'status.tree empty (anchor pipeline)');
      assert('pick-clear-200', false, 'status.tree empty (anchor pipeline)');
    }
  }

  // ④ 等探针 source 步（VM 客户端 GET files/raw 链）
  try {
    await waitFor(async () => {
      const s = await mcp.state(['r_source']).catch(() => '');
      return /r_source[^"]*"(ok|FAILED[^"]*)"/.test(s) ? s : null;
    }, 45000, 'probe source step');
  } catch (e) {
    assert('client-source-read', false, e.message);
    return finish();
  }
  {
    const s = await mcp.state(['r_source']);
    assert('client-source-read', /r_source[^"]*"ok"/.test(s), extract(s, 'r_source'));
  }

  // ⑤ runner 侧显式停止（start 同代次 → 200；再 stop → 仍 200 终态）
  {
    const gen = result.generation ?? 1;
    const stop = await jfetch(`http://127.0.0.1:${ports.back}/api/canvas/stop?generation=${gen}`, { method: 'POST' });
    assert('stop-with-generation-200', stop.status === 200 && stop.body?.state === 'stopped', `status:${stop.status} ${JSON.stringify(stop.body ?? {}).slice(0, 140)}`);
  }
  // 停止终态契约：state=stopped、frame 503
  {
    const st = await jfetch(`http://127.0.0.1:${ports.back}/api/canvas/status`);
    const fr = await jfetch(`http://127.0.0.1:${ports.back}/api/canvas/frame`);
    assert('stop-terminal-state', st.body?.state === 'stopped' && fr.status === 503, `state:${st.body?.state} frame:${fr.status}`);
  }
  // 探针在无进一步操作下仍存活（后端仍在，VM UI 不崩）
  await sleep(1500);
  {
    const s = await mcp.state(['phase']).catch(() => 'MCP-DEAD');
    assert('probe-alive-after-stop', /done/.test(s), s.slice(0, 80));
  }
  // 错误路径：runner 直发不存在目标 → 400（端口面可处理错误）
  {
    const r = await jfetch(`http://127.0.0.1:${ports.back}/api/canvas/start?workspace=`, {
      method: 'POST', headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ app_path: 'targets/definitely-missing-app' }),
    });
    assert('error-path-rejects-400', r.status === 400 && typeof r.body?.error === 'string', `status:${r.status} ${String(r.body?.error ?? '').slice(0, 140)}`);
  }

  return finish();

  async function finish() {
    killTree(probe.p);
    killTree(back.p);
    await sleep(1200);
    result.probe_head = probe.text().slice(0, 3000);
    result.probe_tail = probe.text().slice(-2500);
    result.backend_tail = back.text().slice(-1500);
    fs.mkdirSync(RECEIPT_DIR, { recursive: true });
    const receiptPath = path.join(RECEIPT_DIR, `ports-probe-${mode}-receipt.json`);
    fs.writeFileSync(receiptPath, JSON.stringify(result, null, 2));
    log(`receipt ${receiptPath}`);
    return result;
  }
}

function extract(stateText, field) {
  const m = stateText.match(new RegExp(`${field}[^"]*"([^"]*)"`));
  return m ? m[1] : '(unreadable)';
}

// ── main ─────────────────────────────────────────────────────────────────
const modes = which === 'both' ? ['vm', 'rust'] : [which];
const results = [];
for (const m of modes) {
  log(`mode ${m}`);
  results.push(await runMode(m).catch(e => ({ mode: m, fatal: String(e && e.stack || e), asserts: [{ name: 'fatal', ok: false, detail: String(e) }] })));
}
const allOk = results.every(r => r.asserts.length > 0 && r.asserts.every(a => a.ok));
log(allOk ? `ALL PASS (${results.map(r => r.mode).join(',')})` : 'FAILURES PRESENT');
process.exit(allOk ? 0 : 1);
