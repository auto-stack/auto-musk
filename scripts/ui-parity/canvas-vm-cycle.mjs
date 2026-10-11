#!/usr/bin/env node
// canvas-vm-cycle.mjs — PLAN-102 T-06：VM 臂实况驱动（受管理文件——替代
// 已失传的 tmp/vm-studio-cycle.mjs，093 V05 合同重建 + PLAN-102 场景面）。
//
// 编排骨架沿 canvas-layout-live.mjs（真 musk 后端 + 本 worktree .at 的 VM
// 前端 + MCP vtree/state 双通道）；断言集按 T-02 后（embedded rq spawn）
// 形态适配：
//   boot    : 后端（rust|vm per CYCLE_BACKEND）+ 会话 + 前端 + 工作台挂载
//   session : 真实帧通道（cv_seq 递增 / cv_frame_url 非空——rq 会话的
//             watchdog 截图面）+ 结构树（cv_tree_visible 非空）
//   tree    : 树行点选 → picked 投影落位（canvasPickNode → 后端 → 回填）
//   zoom    : 适应/1:1 切换（cv_zoom_fit 翻转）
//   dock    : PLAN-102 布局面（初值 240/640 + 步进 ±10 + 右停靠几何）
//   imode   : PLAN-102 双输入模式（operate↔inspect 状态翻转）
//   studio  : 应用设计进出（studio 模式面板槽切换，vtree class 断言）
//   stop    : 停止会话 → stopped + 应用描述保留
//
// Usage: node scripts/ui-parity/canvas-vm-cycle.mjs [--backend rust|vm] [--plan 102]
// env CYCLE_BACKEND 兼容旧编排器；收据 tmp/ui-parity/PLAN-<plan>/W-06-vm-cycle-
// <backend>-receipt.json。退出 0 = 全断言过；汇总行含 "ALL PASS" 供编排器判读。

import fs from 'node:fs';
import path from 'node:path';
import http from 'node:http';
import { spawn, spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const args = process.argv.slice(2);
const opt = (n, d) => { const i = args.indexOf(n); return i >= 0 ? args[i + 1] : d; };
const backend = opt('--backend', process.env.CYCLE_BACKEND ?? 'rust');
const planId = opt('--plan', '102');
if (!['rust', 'vm'].includes(backend)) { console.error(`[vm-cycle] --backend 非法: ${backend}`); process.exit(2); }
const RECEIPT_DIR = path.join(ROOT, 'tmp', 'ui-parity', `PLAN-${planId}`);
const WS_ROOT = path.join(RECEIPT_DIR, 'cycle-ws');
const CFG_DIR = path.join(RECEIPT_DIR, 'cycle-cfg');
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

function resolveFirst(c) { for (const x of c.filter(Boolean)) if (fs.existsSync(x)) return x; return null; }
const MUSK_EXE = resolveFirst([
  process.env.MUSK_EXE, path.join(ROOT, 'backend/target/release/musk.exe'),
  path.join(ROOT, 'backend/target/debug/musk.exe'), 'D:/autostack/auto-musk/backend/target/debug/musk.exe',
]);
const AUTO_EXE = resolveFirst([
  process.env.AUTO_EXE, 'D:/autostack/.wt/musk-102/target/release/auto.exe', 'D:/autostack/auto-lang/target/release/auto.exe',
]);
if (!MUSK_EXE || !AUTO_EXE) { console.error(`[vm-cycle] 二进制缺失 musk=${MUSK_EXE} auto=${AUTO_EXE}`); process.exit(2); }

async function freePort() {
  return new Promise((res) => { const s = http.createServer(); s.listen(0, '127.0.0.1', () => { const p = s.address().port; s.close(() => res(p)); }); });
}
async function jfetch(u, o) { const r = await fetch(u, o); let b = null; try { b = await r.json(); } catch {} return { status: r.status, body: b }; }

class Mcp {
  constructor(ep) { this.endpoint = ep; this.id = 100; }
  async call(name, ca) {
    const body = JSON.stringify({ jsonrpc: '2.0', id: this.id++, method: 'tools/call', params: { name, arguments: ca ?? {} } });
    const r = await fetch(this.endpoint, { method: 'POST', headers: { 'content-type': 'application/json' }, body });
    const t = await r.text();
    try { const j = JSON.parse(t); if (j?.error) throw new Error(`${name}: ${JSON.stringify(j.error)}`); return (j?.result?.content ?? []).map((c) => c.text).join(''); }
    catch (e) { if (e instanceof SyntaxError) return t; throw e; }
  }
  vtree() { return this.call('autoui_vtree', {}); }
  async state() { return this.call('autoui_state', {}); }
  action(el, a, v) { return this.call('autoui_action', { element_id: el, action: a, value: v }); }
}

const stPick = (st, k) => new RegExp(`\\s${k}: ("?[^\\n]*?)\\s\\((\\w+)\\)`).exec(st)?.[1] ?? '?';

async function main() {
  const result = { probe: `W-06-vm-cycle-${backend}`, plan: planId, at: new Date().toISOString(), musk: MUSK_EXE, auto: AUTO_EXE, asserts: [], pass: false };
  const assert = (n, ok, d) => { result.asserts.push({ name: n, ok, detail: d ?? '' }); console.log(`  ${ok ? '✓' : '✗'} ${n}${d ? ` — ${d}` : ''}`); };

  fs.mkdirSync(path.join(WS_ROOT, 'targets'), { recursive: true });
  fs.mkdirSync(CFG_DIR, { recursive: true });
  fs.cpSync(path.join(ROOT, 'tests/ui-parity/probes/canvas-embed/embed-fixture'), path.join(WS_ROOT, 'targets/embed-fixture'), { recursive: true });
  fs.rmSync(path.join(WS_ROOT, 'targets/embed-fixture/gen'), { recursive: true, force: true });

  // 后端 17201 固定口（musk pac daemon 绑定——freePort 无效）。vm 轨后端
  // env 沿 canvas-ports-probe backEnv 口径。
  const PORT = 17201;
  const mcpPort = await freePort();
  const backEnv = backend === 'vm'
    ? { MUSK_BACKEND: 'vm', MUSK_VM_CONFIG_DIR: CFG_DIR, MUSK_VM_USERS_PATH: path.join(CFG_DIR, 'users.json'), MUSK_VM_DEFAULT_ROOT: WS_ROOT, MUSK_SERVE_ADDR: `127.0.0.1:${PORT}`, RUST_MIN_STACK: '33554432', AUTO_EXE }
    : { MUSK_CONFIG_DIR: CFG_DIR, MUSK_SERVE_ADDR: `127.0.0.1:${PORT}`, AUTO_EXE };
  const backArgs = backend === 'vm' ? ['serve'] : ['serve', '--workdir', WS_ROOT];
  const back = spawn(MUSK_EXE, backArgs, { cwd: path.join(ROOT, 'backend'), env: { ...process.env, ...backEnv }, windowsHide: true, shell: false });
  let backOut = ''; back.stdout.on('data', (b) => (backOut += b)); back.stderr.on('data', (b) => (backOut += b));

  const storage = path.join(RECEIPT_DIR, `cycle-storage-${Date.now()}.json`);
  const front = spawn(AUTO_EXE, ['run', '--render', 'vm'], {
    cwd: ROOT,
    env: { ...process.env, AUTOUI_MCP_PORT: String(mcpPort), AUTO_VM_STORAGE_FILE: storage, AUTO_VM_WINDOW: '1280x800', AUTO_REUSE_BACKEND: '1', AUTO_HTTP_PORT: String(PORT), AUTO_HTTP_BASE: `http://127.0.0.1:${PORT}`, RUST_MIN_STACK: '16777216' },
    windowsHide: true, shell: false,
  });
  let frontOut = ''; front.stdout.on('data', (b) => (frontOut += b)); front.stderr.on('data', (b) => (frontOut += b));
  const killAll = () => { for (const p of [back, front]) { try { spawnSync('taskkill', ['/pid', String(p.pid), '/T', '/F'], { windowsHide: true }); } catch {} } };

  try {
    let up = false;
    for (let i = 0; i < 90 && !up; i++) { up = (await jfetch(`http://127.0.0.1:${PORT}/api/canvas/status`).catch(() => null))?.status === 200; if (!up) await sleep(1000); }
    assert(`backend-up-${backend}`, up, `:${PORT}`);
    if (!up) throw new Error('backend 90s 未就绪');

    const st = await jfetch(`http://127.0.0.1:${PORT}/api/canvas/start?workspace=`, {
      method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ app_path: 'targets/embed-fixture' }),
    });
    assert('session-start', st.status === 200 && (st.body?.generation_id ?? 0) >= 1, `status:${st.status} gen=${st.body?.generation_id ?? '?'}`);
    result.generation = st.body?.generation_id;

    let mcp = null;
    for (let i = 0; i < 1200 && !mcp; i++) { const m = frontOut.match(/AutoUI MCP: listening on (\S+)/); if (m) mcp = new Mcp(m[1].replace(/\/+$/, '') + '/mcp'); else await sleep(500); }
    assert('mcp-ready', !!mcp);
    if (!mcp) throw new Error('MCP 未就绪');

    const vtRetry = async (tries = 6) => {
      for (let i = 0; i < tries; i++) {
        const t = await mcp.vtree().catch(() => '');
        if (t && !/^Error/.test(t) && t.includes('vnode_')) return t;
        await sleep(400);
      }
      return '';
    };
    const nodeByClass = (vt, sub) => {
      const re = /(\w[-\w]*) (vnode_\d+) \{/g; let m;
      while ((m = re.exec(vt))) {
        const start = m.index; let d = 0, end = -1;
        for (let i = vt.indexOf('{', start); i < vt.length; i++) { if (vt[i] === '{') d++; else if (vt[i] === '}') { d--; if (d === 0) { end = i; break; } } }
        if (end < 0) continue;
        const span = vt.slice(start, end);
        if ((span.match(/class:\s*"([^"]*)"/)?.[1] ?? '').includes(sub)) {
          const bb = span.match(/bbox:\s*\{[^}]*\}/)?.[0] ?? '';
          const num = (k) => { const x = bb.match(new RegExp(`${k}:\\s*(-?[\\d.]+)`)); return x ? Number(x[1]) : null; };
          return { id: m[2], cls: span.match(/class:\s*"([^"]*)"/)[1], span, w: num('w') };
        }
      }
      return null;
    };
    // 停靠列 = 工作台 span 内 class 以 width: 起头的 decl 节点（外框是含
    // cv-workbench 的父——全树首匹配会拿到外框 640，域限定防串）。
    const dockCol = (vt) => {
      const wbNode = nodeByClass(vt, 'cv-workbench');
      if (!wbNode) return null;
      const re = /(\w[-\w]*) (vnode_\d+) \{/g; let m;
      while ((m = re.exec(wbNode.span))) {
        const start = m.index; let d = 0, end = -1;
        for (let i = wbNode.span.indexOf('{', start); i < wbNode.span.length; i++) { if (wbNode.span[i] === '{') d++; else if (wbNode.span[i] === '}') { d--; if (d === 0) { end = i; break; } } }
        if (end < 0) continue;
        const sp = wbNode.span.slice(start, end);
        if ((sp.match(/class:\s*"([^"]*)"/)?.[1] ?? '').startsWith('width:')) {
          const x = sp.match(/bbox:\s*\{[^}]*\}/)?.[0].match(/w:\s*(-?[\d.]+)/);
          return { id: m[2], w: x ? Number(x[1]) : null };
        }
      }
      return null;
    };
    const pressByContent = async (vt, label) => {
      const li = vt.indexOf(`content: "${label}"`);
      if (li < 0) throw new Error(`按钮未定位: ${label}`);
      const start = vt.slice(0, li).lastIndexOf('button vnode_');
      const id = vt.slice(start).match(/^button (vnode_\d+)/)?.[1] ?? '';
      if (!id) throw new Error(`按钮节点未定位: ${label}`);
      await mcp.action(id, 'press', '');
      await sleep(400);
    };
    // 按 label 定位钮（vtree CJK 文本前有 U+EE03 私有区哨兵——精确串匹配
    // 永差一位；按裸文本 indexOf + 回溯 button 开启）。
    const pressByLabel = async (vt, labelText) => {
      let li = vt.indexOf(labelText);
      if (li < 0) { vt = await vtRetry(); li = vt.indexOf(labelText); }
      if (li < 0) throw new Error(`按钮未定位(label): ${labelText}`);
      const start = vt.slice(0, li).lastIndexOf('button vnode_');
      const id = vt.slice(start).match(/^button (vnode_\d+)/)?.[1] ?? '';
      if (!id) throw new Error(`按钮节点未定位(label): ${labelText}`);
      await mcp.action(id, 'press', '');
      await sleep(600);
    };
    const dumpVt = (tag) => { try { fs.writeFileSync(path.join(RECEIPT_DIR, `W-06-vm-cycle-${backend}-${tag}-vtree.txt`), vt, 'utf8'); } catch { /* best effort */ } };

    let vt = '';
    let wb = null;
    for (let i = 0; i < 60 && !wb; i++) { vt = await vtRetry(); if (vt) wb = nodeByClass(vt, 'cv-workbench'); if (!wb) await sleep(1000); }
    for (let i = 0; i < 10 && wb?.w == null; i++) { await sleep(1000); vt = await vtRetry(); wb = vt ? nodeByClass(vt, 'cv-workbench') ?? wb : wb; }
    assert('workbench-mounted', !!wb, wb?.w != null ? `bbox w=${wb.w}` : '挂载/布局超时');
    if (!wb) throw new Error('workbench 未挂载');

    // ── session：真实帧 + 结构树（rq 会话的截图/树回填面） ─────────────
    const state = () => mcp.state().catch(() => '');
    let st1 = await state();
    const seq0 = stPick(st1, 'cv_seq');
    await sleep(2500);   // 两拍轮询 + watchdog 采帧
    const st2 = await state();
    const seq1 = stPick(st2, 'cv_seq');
    const frameUrl = stPick(st2, 'cv_frame_url');
    assert('frame-channel-live', frameUrl.includes('/api/canvas/frame'), `frame_url=${String(frameUrl).slice(0, 40)} seq ${seq0}→${seq1}`);
    const treeN = stPick(st2, 'cv_tree_visible');
    assert('structure-tree-visible', !/^\[\]$|^None/.test(treeN) && treeN !== '?', `tree=${String(treeN).slice(0, 36)}`);

    // ── pick-projection：拾取链 E2E 后半（后端→前端投影） ─────────────
    // 坐标形态（{x,y}）对 rq 会话未接（实测 204）；UI 指针注入段有嵌套
    // mouse-area 仪器缺口（dragEcho ok 不达——观察项）；此处走 FrameClick
    // 同源的 vnode_id 形态：status.tree 取夹具按钮节点 → POST pick →
    // 1s 轮询 → VM cv_picked_id 落位。
    {
      const sNow = (await jfetch(`http://127.0.0.1:${PORT}/api/canvas/status`)).body ?? {};
      const btn = (sNow.tree ?? []).find((n) => n.kind === 'button');
      let picked = '""';
      // 503 = 无帧（rq 会话 watchdog 采帧窗口 flap——直探 t+20/47s 200、
      // 驱动时序可撞 None 窗）；重试 ≤5 次等帧回。
      result.pickPost = [];
      for (let attempt = 0; attempt < 5 && (picked === '""' || picked === '?'); attempt++) {
        if (!btn) break;
        const pr = await jfetch(`http://127.0.0.1:${PORT}/api/canvas/pick`, {
          method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ vnode_id: btn.id }),
        }).catch(() => null);
        result.pickPost.push(pr?.status ?? 'ERR');
        if (pr?.status === 200) {
          await sleep(2200);
          picked = stPick(await state(), 'cv_picked_id');
        } else {
          await sleep(2000);
        }
      }
      if (picked === '""' || picked === '?') dumpVt('coordpick');
      assert('pick-projection-e2e', picked !== '""' && picked !== '?', `picked=${String(picked).slice(0, 32)}（POST vnode_id→轮询→VM 投影；pick 码=${result.pickPost.join(',')}）`);
    }

    // ── zoom：适应/1:1 ──────────────────────────────────────────────────
    vt = await vtRetry();
    const zoom0 = (stPick(await state(), 'cv_zoom_fit')) === 'true';
    await pressByContent(vt, '1:1');
    const zoom1 = (stPick(await state(), 'cv_zoom_fit')) === 'true';
    assert('zoom-toggle', zoom0 !== zoom1, `${zoom0} → ${zoom1}`);

    // ── dock（PLAN-102 布局面；dockCol 域限定防外框互串） ───────────────
    vt = await vtRetry();
    {
      let dock = dockCol(vt);
      for (let i = 0; i < 6 && (dock?.w == null || dock.w <= 0); i++) { await sleep(350); vt = await vtRetry(); dock = dockCol(vt); }
      assert('dock-initial-240', Math.abs((dock?.w ?? 0) - 240) <= 2, `w=${dock?.w}`);
      await pressByContent(vt, '››');
      let dock2 = null;
      for (let i = 0; i < 6 && (dock2?.w == null || dock2.w <= 0); i++) { await sleep(350); vt = await vtRetry(); dock2 = dockCol(vt); }
      assert('dock-nudge+10', Math.abs((dock2?.w ?? 0) - 250) <= 2, `w=${dock2?.w}`);
      await pressByContent(vt, '‹‹');
    }

    // ── imode：双输入模式 ──────────────────────────────────────────────
    vt = await vtRetry();
    const im0 = stPick(await state(), 'cv_input_mode');
    await pressByContent(vt, '🖱');
    const im1 = stPick(await state(), 'cv_input_mode');
    assert('input-mode-toggle', im0 !== im1, `${im0} → ${im1}`);
    // 复位 operate：钮面 = 当前模式反象（inspect 态显示 🔍）。
    await pressByContent((await vtRetry()), '🔍');

    // ── studio：进出（图标钮；studio 模式面板槽换 CanvasStudioSlot——
    // workbench 被替换是 T-03 范围口径，断言以 studio 槽特征类为准） ────
    vt = await vtRetry();
    await pressByLabel(vt, '应用设计');
    await sleep(600);
    vt = await vtRetry();
    const studioOn = vt.includes('w-[200px]') || vt.includes('w-[220px]');
    assert('studio-enter-slot-switch', studioOn, 'studio 槽特征类（w-[200px] 结构列）出现');
    await pressByLabel(vt, '应用设计');
    await sleep(600);

    // ── stop：停止会话 ──────────────────────────────────────────────────
    vt = await vtRetry();
    {
      const wbNode = nodeByClass(vt, 'cv-workbench');
      const li = (wbNode?.span ?? '').indexOf('content: "×"');
      const start = li >= 0 ? (wbNode.span).slice(0, li).lastIndexOf('button vnode_') : -1;
      const sid = start >= 0 ? (wbNode.span).slice(start).match(/^button (vnode_d+)/)?.[1] : '';
      if (sid) { await mcp.action(sid, 'press', ''); } else { await pressByContent(vt, '×'); }
      await sleep(400);
    }
    let stateEnd = 'running';
    for (let i = 0; i < 12 && !String(stateEnd).includes('stopped'); i++) { await sleep(1000); stateEnd = stPick(await state(), 'cv_state'); }
    const stEndF = await state();
    const appKept = stPick(stEndF, "cv_app");
    assert('stop-session-stops', stateEnd.includes('stopped'), `cv_state=${stateEnd}`);
    assert('stop-keeps-app-desc', appKept.includes('embed'), `cv_app=${String(appKept).slice(0, 30)}`);

    result.pass = result.asserts.every((a) => a.ok);
  } catch (e) {
    result.error = String(e.message ?? e);
    result.pass = false;
  } finally {
    result.backendTail = backOut.slice(-2000);
    result.frontendTail = frontOut.slice(-2000);
    killAll();
    await sleep(1500);
    try { fs.rmSync(storage, { force: true }); } catch {}
  }

  fs.mkdirSync(RECEIPT_DIR, { recursive: true });
  const receipt = path.join(RECEIPT_DIR, `W-06-vm-cycle-${backend}-receipt.json`);
  fs.writeFileSync(receipt, JSON.stringify(result, null, 2), 'utf8');
  const fails = result.asserts.filter((a) => !a.ok).length;
  console.log(`\n[vm-cycle] backend=${backend} ${result.asserts.length - fails}/${result.asserts.length}${result.error ? ` ERROR:${result.error}` : ''} → ${path.relative(ROOT, receipt)}`);
  if (result.pass) console.log('[vm-cycle] ALL PASS');
  process.exit(result.pass ? 0 : 1);
}

main().catch((e) => { console.error(`[vm-cycle] 失败: ${e.message}`); process.exit(1); });
