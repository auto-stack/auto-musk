#!/usr/bin/env node
// canvas-layout-live.mjs — PLAN-102 T-03 切片④: 停靠布局 VM 实机探针。
//
// 被测对象 = musk VM 前端真身（auto run --render vm，本 worktree .at 源）
// + 真实 musk 后端会话（musk serve + POST /api/canvas/start → embed 夹具
// 应用——面板随状态轮询展开，工作台挂载）。
//
// 断言面（全部经 MCP vtree bbox 实测渲染几何，非样式串）：
//   boot    : 工作台根（cv-workbench）几何 + 初始停靠列 240 / 外框 640
//             + chrome=78 派生常数校准记录（wb bbox h vs 窗口-78）
//   nudge   : 键盘步进钮 ±10 → 停靠列宽随动
//   dock    : ◨/▄/✕/◧ 四停靠几何翻转（右=停靠列 x 右移；底=decl 高驱动；
//             hidden=停靠列消失；left 恢复）
//   drag    : __mcp_drag 注入 DivH（单 widget Down␟Move␟Up 合同）——
//             +60px 精确随动；越界钳制（上限 wb_w-240 / 下限 180）
//   narrow  : DivOuter 拖至外框 min 240 → wb 派生 240 < 420 → tabs 降级
//             （solo 页签出现、停靠列退役）；拖宽 → dock 自动恢复
//
// Usage: node scripts/ui-parity/canvas-layout-live.mjs
// MUSK_EXE 解析：env → 主检出 debug 构建（布局面不需 T-02 后端新面）。
// AUTO_EXE 解析：env → musk-102 依赖构建（canvas 会话 spawn 用）→ 主检出。
// 收据：tmp/ui-parity/PLAN-102/W-03-4-canvas-layout-live-receipt.json。
// 退出 0 = 全部断言过。

import fs from 'node:fs';
import path from 'node:path';
import http from 'node:http';
import { spawn, spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const RECEIPT_DIR = path.join(ROOT, 'tmp', 'ui-parity', 'PLAN-102');
const WS_ROOT = path.join(RECEIPT_DIR, 'layout-ws');
const CFG_DIR = path.join(RECEIPT_DIR, 'layout-cfg');
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

function resolveFirst(candidates) {
  for (const c of candidates.filter(Boolean)) if (fs.existsSync(c)) return c;
  return null;
}
const MUSK_EXE = resolveFirst([
  process.env.MUSK_EXE,
  path.join(ROOT, 'backend', 'target', 'release', 'musk.exe'),
  path.join(ROOT, 'backend', 'target', 'debug', 'musk.exe'),
  'D:/autostack/auto-musk/backend/target/debug/musk.exe',
]);
const AUTO_EXE = resolveFirst([
  process.env.AUTO_EXE,
  'D:/autostack/.wt/musk-102/target/release/auto.exe',
  'D:/autostack/auto-lang/target/release/auto.exe',
]);
if (!MUSK_EXE || !AUTO_EXE) {
  console.error(`[layout-live] 二进制缺失 musk=${MUSK_EXE} auto=${AUTO_EXE}`);
  process.exit(2);
}

async function freePort() {
  return new Promise((resolve) => {
    const s = http.createServer();
    s.listen(0, '127.0.0.1', () => { const p = s.address().port; s.close(() => resolve(p)); });
  });
}
async function jfetch(url, opts) {
  const r = await fetch(url, opts);
  let body = null;
  try { body = await r.json(); } catch { /* 文本面 */ }
  return { status: r.status, body };
}

class Mcp {
  constructor(endpoint) { this.endpoint = endpoint; this.id = 100; }
  async call(name, callArgs) {
    const body = JSON.stringify({ jsonrpc: '2.0', id: this.id++, method: 'tools/call', params: { name, arguments: callArgs ?? {} } });
    const r = await fetch(this.endpoint, { method: 'POST', headers: { 'content-type': 'application/json' }, body });
    const t = await r.text();
    try {
      const j = JSON.parse(t);
      if (j?.error) throw new Error(`${name}: ${JSON.stringify(j.error)}`);
      return (j?.result?.content ?? []).map((c) => c.text).join('');
    } catch (e) { if (e instanceof SyntaxError) return t; throw e; }
  }
  vtree() { return this.call('autoui_vtree', {}); }
  action(element, action, value) { return this.call('autoui_action', { element_id: element, action, value }); }
  async pressByLabel(vt, label) {
    // dock 钮用子 text 形态（非 inline label）——按 span 含 content 定位。
    const li = vt.indexOf(`content: "${label}"`);
    if (li < 0) throw new Error(`按钮未定位: ${label}`);
    const head = vt.slice(0, li);
    const start = head.lastIndexOf('button vnode_');
    const id = start >= 0 ? vt.slice(start).match(/^button (vnode_\d+)/)?.[1] ?? '' : '';
    if (!id) throw new Error(`按钮节点未定位: ${label}`);
    await this.action(id, 'press', '');
  }
}

/** vtree Atom 探针：按 class 子串定位节点 span（括号配平），取首个
 * bbox/class。opts.within = 只在该 span 文本内搜；opts.containing = 节
 * 点 span 须含该子串（父容器定位用）。col 序列化序：bbox 先于 box/
 * style/class 与子节点。 */
function nodeByClass(vt, classSub, opts = {}) {
  const scope = opts.within ?? vt;
  const re = new RegExp(`(\\w[-\\w]*) (vnode_\\d+) \\{`, 'g');
  let m;
  while ((m = re.exec(scope))) {
    const start = m.index;
    let depth = 0, end = -1;
    for (let i = scope.indexOf('{', start); i < scope.length; i++) {
      if (scope[i] === '{') depth++;
      else if (scope[i] === '}') { depth--; if (depth === 0) { end = i; break; } }
    }
    if (end < 0) continue;
    const span = scope.slice(start, end);
    if (opts.containing && !span.includes(opts.containing)) continue;
    const cls = span.match(/class:\s*"([^"]*)"/)?.[1] ?? '';
    if (cls.includes(classSub)) {
      const bboxRaw = span.match(/bbox:\s*\{[^}]*\}/)?.[0] ?? '';
      const num = (k) => { const x = bboxRaw.match(new RegExp(`${k}:\\s*(-?[\\d.]+)`)); return x ? Number(x[1]) : null; };
      return { id: m[2], kind: m[1], cls, span, x: num('x'), y: num('y'), w: num('w'), h: num('h'), bboxRaw };
    }
  }
  return null;
}

/** 节点 span 定位（class 子串 → 完整 span 文本，供域限定）。 */
function spanByClass(vt, classSub) {
  const n = nodeByClass(vt, classSub);
  return n ? n.span : null;
}

const near = (a, e, tol) => a != null && Math.abs(a - e) <= tol;

async function main() {
  const result = { probe: 'W-03-4-canvas-layout-live', at: new Date().toISOString(), musk: MUSK_EXE, auto: AUTO_EXE, asserts: [], pass: false };
  const assert = (name, ok, detail) => {
    result.asserts.push({ name, ok, detail: detail ?? '' });
    console.log(`  ${ok ? '✓' : '✗'} ${name}${detail ? ` — ${detail}` : ''}`);
  };

  // ── 工作区与目标应用（embed 夹具拷入 wsRoot/targets） ──────────────────
  fs.mkdirSync(path.join(WS_ROOT, 'targets'), { recursive: true });
  fs.mkdirSync(CFG_DIR, { recursive: true });
  const tgt = path.join(WS_ROOT, 'targets', 'embed-fixture');
  fs.cpSync(path.join(ROOT, 'tests', 'ui-parity', 'probes', 'canvas-embed', 'embed-fixture'), tgt, { recursive: true });
  fs.rmSync(path.join(tgt, 'gen'), { recursive: true, force: true });

  // 端口：musk pac daemon 绑定 back_port=17201（auto run 启动桩死等该口，
  // freePort 无效——launch-vm.cmd 同款）；占用则如实失败（不与用户实跑
  // 实例共栈污染会话）。
  const backPort = 17201;
  const mcpPort = await freePort();
  const WINDOW_W = 1280, WINDOW_H = 800;

  // ── ① 后端 ────────────────────────────────────────────────────────────
  const back = spawn(MUSK_EXE, ['serve', '--workdir', WS_ROOT], {
    cwd: path.join(ROOT, 'backend'),
    env: { ...process.env, MUSK_CONFIG_DIR: CFG_DIR, MUSK_SERVE_ADDR: `127.0.0.1:${backPort}`, AUTO_EXE },
    windowsHide: true, shell: false,
  });
  let backOut = '';
  back.stdout.on('data', (b) => (backOut += b));
  back.stderr.on('data', (b) => (backOut += b));

  // ── ② VM 前端（本 worktree 源） ───────────────────────────────────────
  const storage = path.join(RECEIPT_DIR, `layout-storage-${Date.now()}.json`);
  const front = spawn(AUTO_EXE, ['run', '--render', 'vm'], {
    cwd: ROOT,
    env: {
      ...process.env,
      AUTOUI_MCP_PORT: String(mcpPort),
      AUTO_VM_STORAGE_FILE: storage,
      AUTO_VM_WINDOW: `${WINDOW_W}x${WINDOW_H}`,
      AUTO_REUSE_BACKEND: '1',
      AUTO_HTTP_PORT: String(backPort),
      AUTO_HTTP_BASE: `http://127.0.0.1:${backPort}`,
      RUST_MIN_STACK: '16777216',
    },
    windowsHide: true, shell: false,
  });
  let frontOut = '';
  front.stdout.on('data', (b) => (frontOut += b));
  front.stderr.on('data', (b) => (frontOut += b));
  result.frontExit ??= null;
  front.on('exit', (code, sig) => {
    result.frontExit = { code, sig, at: new Date().toISOString(), outBytes: frontOut.length };
    try { fs.mkdirSync(RECEIPT_DIR, { recursive: true }); fs.writeFileSync(path.join(RECEIPT_DIR, 'W-03-4-frontend-full.log'), frontOut, 'utf8'); } catch { /* best effort */ }
  });
  back.on('exit', (code, sig) => {
    result.backExit ??= { code, sig, at: new Date().toISOString() };
  });

  const procs = [back, front];
  const killAll = () => { for (const p of procs) { try { spawnSync('taskkill', ['/pid', String(p.pid), '/T', '/F'], { windowsHide: true }); } catch { /* best effort */ } } };

  try {
    // 后端就绪。
    let up = false;
    for (let i = 0; i < 90 && !up; i++) {
      up = (await jfetch(`http://127.0.0.1:${backPort}/api/canvas/status`).catch(() => null))?.status === 200;
      if (!up) await sleep(1000);
    }
    assert('backend-up', up, `:${backPort}`);
    if (!up) throw new Error('backend 90s 未就绪');

    // 会话（真实 spawn；面板随轮询展开——布局面不依赖帧）。
    const st = await jfetch(`http://127.0.0.1:${backPort}/api/canvas/start?workspace=`, {
      method: 'POST', headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ app_path: 'targets/embed-fixture' }),
    });
    assert('session-start', st.status === 200, `status:${st.status} gen=${st.body?.generation_id ?? '?'}`);

    // MCP 就绪 + 工作台挂载（vtree 出现 cv-workbench）。首跑含 cargo 冷
    // 编译（worktree target 空——编译产物缓存后后续跑快），窗 10 分钟。
    let mcp = null;
    for (let i = 0; i < 1200 && !mcp; i++) {
      const m = frontOut.match(/AutoUI MCP: listening on (\S+)/);
      if (m) mcp = new Mcp(m[1].replace(/\/+$/, '') + '/mcp');
      else await sleep(500);
    }
    assert('mcp-ready', !!mcp);
    if (!mcp) throw new Error('MCP 10 分钟未就绪');

    let vt = '';
    const vtRetry = async (tries = 6) => {
      for (let i = 0; i < tries; i++) {
        const t = await mcp.vtree().catch(() => '');
        if (t && !/^Error/.test(t) && t.includes('vnode_')) return t;
        await sleep(400);
      }
      return '';
    };
    let wb = null;
    for (let i = 0; i < 60 && !wb; i++) {
      vt = await vtRetry();
      if (vt) wb = nodeByClass(vt, 'cv-workbench');
      if (!wb) await sleep(1000);
    }
    assert('workbench-mounted', !!wb, wb ? `bbox=${wb.bboxRaw}` : '60s 未挂载（cv_open 未开？）');
    if (!wb) throw new Error('workbench 未挂载');
    // bbox 就绪等待：挂载首拍 vtree 可能先于布局（bbox 缺席）——轮询到
    // 工作台根有 bbox（≤10s；仍缺则转储 vtree 供诊断后失败）。
    for (let i = 0; i < 10 && wb.w == null; i++) {
      await sleep(1000);
      vt = await vtRetry(3);
      wb = vt ? nodeByClass(vt, 'cv-workbench') ?? wb : wb;
    }
    if (wb.w == null) {
      fs.writeFileSync(path.join(RECEIPT_DIR, 'W-03-4-vtree-dump.txt'), vt, 'utf8');
    }
    result.wbRoot = { w: wb.w, h: wb.h, x: wb.x, y: wb.y };

    // ── boot 几何 ────────────────────────────────────────────────────────
    const wbSpan = spanByClass(vt, 'cv-workbench');
    // 停靠列 = 工作台 span 内的 decl 串节点（水平 width: / 底部 height:）；
    // 外框 = 含工作台的 decl 节点（父容器）——域限定防两者互串。
    const dock0 = nodeByClass(vt, 'px;height:100%', { within: wbSpan });
    const outer0 = nodeByClass(vt, 'px;height:100%', { containing: 'cv-workbench' });
    assert('initial-dock-col-240', near(dock0?.w, 240, 2), `w=${dock0?.w}`);
    assert('initial-outer-640', near(outer0?.w, 640, 2), `w=${outer0?.w} cls=${outer0?.cls?.slice(0, 24)}`);
    // chrome 校准（切片④定案）：面板根即全窗高（ContentHeader 在聊天列
    // 内部）——工作台根 bbox h ≈ 窗口高；拆分容器 = 根高 - 停靠条 29。
    const chromeDelta = wb.h != null ? wb.h - WINDOW_H : null;
    result.chromeCalibration = { measuredRootH: wb.h, windowH: WINDOW_H, delta: chromeDelta };
    assert('wb-root-full-window-height', Math.abs(chromeDelta ?? 999) <= 2, `root h=${wb.h} window=${WINDOW_H}`);

    const refresh = async () => { vt = await vtRetry(); return vt; };
    const dockColNow = () => {
      const s = spanByClass(vt, 'cv-workbench');
      if (!s) return null;
      return nodeByClass(vt, 'px;height:100%', { within: s }) ?? nodeByClass(vt, 'px;width:100%', { within: s });
    };
    const dockCol = async () => {
      for (let i = 0; i < 6; i++) {
        const n = dockColNow();
        if (n && n.w != null && n.w > 0) return n;
        await sleep(350);
        vt = (await vtRetry(2)) || vt;
      }
      return dockColNow();
    };
    // 稳定读：连续两次一致（秒级 Poll 重渲染 × 半序列化 vtree 的竞态
    // 消解——切片④实测 w=0/1360 过渡帧）。
    const stableRead = async (fn) => {
      let prev = null;
      for (let i = 0; i < 10; i++) {
        const n = fn();
        if (n && n.w != null && n.w > 0 && prev && prev.w === n.w) return n;
        prev = n;
        await sleep(350);
        vt = (await vtRetry(2)) || vt;
      }
      return prev;
    };
    const outerCol = () => nodeByClass(vt, 'px;height:100%', { containing: 'cv-workbench' });
    // 状态取证：每阶段后抓 CanvasStore 关键字段（诊断 wb 派生/模式翻转）。
    const stateSnap = async (phase) => {
      const st = await mcp.call('autoui_state', {}).catch(() => '');
      const pick = (k) => new RegExp(`\\s${k}: ([^\\n]*)`).exec(st)?.[1] ?? '?';
      const snap = {
        phase,
        dock: pick('cv_dock'), dock_w: pick('cv_dock_w'), dock_h: pick('cv_dock_h'),
        outer_w: pick('cv_outer_w'), wb_w: pick('cv_wb_w'), wb_h: pick('cv_wb_h'),
        mode: pick('cv_layout_mode'), drag: pick('cv_drag_kind'),
        w_decl: pick('cv_dock_w_decl'), h_decl: pick('cv_dock_h_decl'), outer_decl: pick('cv_outer_w_decl'),
      };
      (result.states ??= []).push(snap);
      return snap;
    };

    // ── nudge ────────────────────────────────────────────────────────────
    await mcp.pressByLabel(vt, '››');
    await sleep(300); await refresh(); await stateSnap('nudge-widen');
    assert('nudge-widen+10', near((await dockCol())?.w, 250, 2), `w=${(await dockCol())?.w}`);
    await mcp.pressByLabel(vt, '‹‹');
    await sleep(300); await refresh(); await stateSnap('nudge-narrow');
    assert('nudge-narrow-10', near((await dockCol())?.w, 240, 2), `w=${(await dockCol())?.w}`);

    // ── dock 切换 ────────────────────────────────────────────────────────
    await stateSnap('pre-dock');
    const xLeft = (await dockCol())?.x;
    await mcp.pressByLabel(vt, '◨');      // right
    await sleep(300); await refresh();
    await stateSnap('dock-right');
    const dR = await dockCol();
    assert('dock-right-geometry', dR != null && dR.x > xLeft + 100, `x ${xLeft} → ${dR?.x}`);

    await mcp.pressByLabel(vt, '▄');      // bottom
    await sleep(300); await refresh();
    const dB = await dockCol();
    assert('dock-bottom-height-driven', dB != null && near(dB.h, 240, 2) && dB.w > 400, `w=${dB?.w} h=${dB?.h}`);

    await mcp.pressByLabel(vt, '✕');     // hidden
    await sleep(300); await refresh();
    assert('dock-hidden-col-gone', (await dockCol()) == null);

    await mcp.pressByLabel(vt, '◧');     // left 恢复
    await sleep(300); await refresh();
    const dL = await dockCol();
    assert('dock-left-restored', dL != null && near(dL.w, 240, 2) && Math.abs((dL.x ?? 999) - xLeft) < 4, `w=${dL?.w} x=${dL?.x}`);

    // ── drag（注入合同：单 widget Down␟Move␟Up） ─────────────────────────
    const SEP = String.fromCharCode(31);
    const wbW = wb.w ?? 640;
    const dragDiv = async (widget, pxDelta, containerW) => {
      const L = Math.round(pxDelta * 4096 / containerW);
      const spec = [widget, 'Down', 'Move', 'Up', `0,0;${Math.round(L / 2)},0;${L},0`].join(SEP);
      result.dragEcho ??= []; result.dragEcho.push(await mcp.action('aura_0', 'drag', spec));
      await sleep(400);
      await refresh();
    };
    await dragDiv('DivH', 60, wbW); await stateSnap('drag-60');
    const d60 = await stableRead(dockColNow);
    if (!near(d60?.w, 300, 2)) {
      try { fs.writeFileSync(path.join(RECEIPT_DIR, 'W-03-4-drag60-vtree.txt'), vt, 'utf8'); } catch { /* best effort */ }
    }
    assert('drag-dock-+60px', near(d60?.w, 300, 2), `w=${d60?.w} cls=${d60?.cls?.slice(0, 32)}`);

    await dragDiv('DivH', 99999, wbW);
    assert('drag-clamp-max', near((await stableRead(dockColNow))?.w, wbW - 240, 2), `w=${(await dockCol())?.w} max=${wbW - 240}`);

    await dragDiv('DivH', -99999, wbW);
    assert('drag-clamp-min-180', near((await stableRead(dockColNow))?.w, 180, 2), `w=${(await dockCol())?.w}`);

    // ── narrow 降级 + 恢复（外框拖动） ────────────────────────────────────
    const containerW = WINDOW_W - 240;    // canvasOuterContainerW(1280)=1040
    await dragDiv('DivOuter', -99999, containerW);
    await sleep(2200); await refresh();   // Poll 拍：wb 派生 240 → tabs
    const oMin = await stableRead(outerCol);
    assert('outer-drag-min-240', near(oMin?.w, 240, 2), `w=${oMin?.w}`);
    const soloBtn = (vt.includes('content: "画布"') || vt.includes('content: "Canvas"'));
    assert('narrow-outer-tabs', (await dockCol()) == null && soloBtn, `dockCol=${!!dockCol()} solo=${soloBtn}`);

    await dragDiv('DivOuter', 99999, containerW);
    await sleep(2200); await refresh();
    const oMax = await stableRead(outerCol);
    assert('outer-drag-max-720', near(oMax?.w, 720, 2), `w=${oMax?.w}（=container 1040-chat 320）`);
    const dN = await dockCol();
    assert('widen-outer-dock-restored', dN != null && dN.w >= 180, `w=${dN?.w}（偏好保留+重钳）`);

    // ── T-05 走查截图（AC-08 证据面：四停靠布局 + 紧凑树行实况） ─────────
    {
      const shot = await mcp.call('autoui_screenshot', {}).catch(() => null);
      let copied = '';
      try {
        // 回文本 "Screenshot saved to: <path>"（P1 shot() 同源形态）。
        const m = /saved to:\s*(\S+)/.exec(String(shot ?? ''));
        if (m?.[1]) {
          const src = m[1].replace(/^\\\\\?\\/, '');
          if (fs.existsSync(src)) {
            const dst = path.join(RECEIPT_DIR, 'W-03-5-walkthrough-vm.png');
            fs.copyFileSync(src, dst);
            copied = path.relative(ROOT, dst);
          }
        }
      } catch { /* best effort */ }
      assert('walkthrough-screenshot', !!copied, copied || `shot=${String(shot).slice(0, 80)}`);
    }

    result.pass = result.asserts.every((a) => a.ok);
  } catch (e) {
    result.error = String(e.message ?? e);
    result.pass = false;
  } finally {
    result.backendTail = backOut.slice(-3000);
    result.frontendTail = frontOut.slice(-3000);
    killAll();
    await sleep(1500);
    try { fs.rmSync(storage, { force: true }); } catch { /* best effort */ }
  }

  fs.mkdirSync(RECEIPT_DIR, { recursive: true });
  const receipt = path.join(RECEIPT_DIR, 'W-03-4-canvas-layout-live-receipt.json');
  fs.writeFileSync(receipt, JSON.stringify(result, null, 2), 'utf8');
  const fails = result.asserts.filter((a) => !a.ok);
  console.log(`\n[layout-live] ${result.asserts.length - fails.length}/${result.asserts.length} pass${result.error ? ` ERROR:${result.error}` : ''} → ${path.relative(ROOT, receipt)}`);
  process.exit(result.pass ? 0 : 1);
}

main().catch((e) => {
  console.error(`[layout-live] 失败: ${e.message}`);
  process.exit(1);
});
