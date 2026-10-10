#!/usr/bin/env node
// canvas-vm-style-spike.mjs — PLAN-102 W-03-1: VM 轨动态宽度出程 spike。
//
// 问题：canvas_workbench 停靠列宽度随拖动连续变化，视图需要动态尺寸；
// musk spec 旧口径「VM 轨 :style 字段引用不产出」描的是 PLAN-748 之前的
// 行为（根修 a2422add2 已入 musk-102 依赖 worktree）。本 spike 以运行时
// 实测裁决 workbench 定稿形态，双证据通道：
//   ① vtree 节点 class/style prop（字符串到达面）
//   ② vtree 节点 bbox（渲染几何面——border_box，±1.5px 容差）
// 五臂（夹具 tests/ui-parity/probes/canvas-layout/spike-fixture/）：
//   A0 字面量对照（门——仪器健全性，恒 240）
//   A1 数组字面量集 [.store.ladder_cls]（门——枚举阶梯候选，240→300）
//   B1 裸引用 .store.w_decl CSS 声明串（门——连续 px 候选，280→330）
//   B2 裸引用 .store.w_cls 类名文本（非门——web :style 陷阱对照，记录
//      VM 侧能力：320→380）
//   B3 内联拼接 "width:"+int+"px"（门——indent_style 形态 store 引用
//      变体，240→300）
// cycle 动作后重测——证明运行时动态（状态变化→重渲染→几何跟随），非
// 仅静态首帧。
//
// Usage: node scripts/ui-parity/canvas-vm-style-spike.mjs
// AUTO_EXE 解析序：env → musk-102 依赖构建（含 748 根修）→ auto-lang 主
// 检出构建。收据：tmp/ui-parity/PLAN-102/W-03-1-vm-style-spike-receipt.json。
// 退出 0 = 四个门臂（A0/A1/B1/B3）初值+改宽后几何全部命中。

import fs from 'node:fs';
import path from 'node:path';
import http from 'node:http';
import { spawn, spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const FIXTURE = path.join(ROOT, 'tests', 'ui-parity', 'probes', 'canvas-layout', 'spike-fixture');
const RECEIPT_DIR = path.join(ROOT, 'tmp', 'ui-parity', 'PLAN-102');

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

function resolveAutoExe() {
  const candidates = [
    process.env.AUTO_EXE,
    'D:/autostack/.wt/musk-102/target/release/auto.exe',
    'D:/autostack/auto-lang/target/release/auto.exe',
  ].filter(Boolean);
  for (const c of candidates) {
    if (fs.existsSync(c)) return c;
  }
  console.error('[style-spike] auto.exe 未找到（AUTO_EXE 可覆盖）');
  process.exit(2);
}

function freePort() {
  return new Promise((resolve) => {
    const s = http.createServer();
    s.listen(0, '127.0.0.1', () => { const p = s.address().port; s.close(() => resolve(p)); });
  });
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
  state() { return this.call('autoui_state', {}); }
  vtree() { return this.call('autoui_vtree', {}); }
  action(element, action, value) { return this.call('autoui_action', { element_id: element, action, value }); }
}

async function spawnFixture() {
  const mcpPort = await freePort();
  const backPort = await freePort();
  const storage = path.join(RECEIPT_DIR, `storage-spike-${Date.now()}.json`);
  const dummy = http.createServer((q, r) => { r.writeHead(404); r.end('{}'); });
  await new Promise((r) => dummy.listen(backPort, '127.0.0.1', r));
  const p = spawn(AUTO_EXE, ['run', '--render', 'vm'], {
    cwd: FIXTURE,
    env: {
      ...process.env,
      AUTOUI_MCP_PORT: String(mcpPort),
      AUTO_VM_STORAGE_FILE: storage,
      AUTO_VM_WINDOW: '480x680',
      AUTO_DEBUG_CAPTURE: '1',
      AUTO_REUSE_BACKEND: '1',
      AUTO_HTTP_PORT: String(backPort),
    },
    windowsHide: true, shell: false,
  });
  let out = '';
  p.stdout.on('data', (b) => (out += b));
  p.stderr.on('data', (b) => (out += b));
  return { p, dummy, storage, mcpPort, getOut: () => out };
}

async function awaitMcp(h, timeoutMs) {
  const t0 = Date.now();
  while (Date.now() - t0 < timeoutMs) {
    const m = h.getOut().match(/AutoUI MCP: listening on (https?:\/\/\S+)/);
    if (m) return new Mcp(m[1].replace(/\/+$/, '') + '/mcp');
    if (h.p.exitCode !== null) break;
    await sleep(250);
  }
  return null;
}

async function awaitFirstFrame(mcp, timeoutMs = 30000) {
  const t0 = Date.now();
  while (Date.now() - t0 < timeoutMs) {
    const vt = await mcp.vtree();
    if (!/^Error/.test(vt) && vt.includes('A0')) return vt;
    await sleep(300);
  }
  return null;
}

/** 从 content 标记回溯最近前位 `col (vnode_N) {` 开启的节点 span（括号配
 * 平），取 span 内首个 bbox（props 序列化序：bbox 先于 box/style/class
 * 与子节点——A0 健全性门兜底误解析）与 class/style prop。text 节点序列
 * 化为 content: "X"（本轮活体校准），button 为 label: "X"。 */
function probeNode(vt, label) {
  const li = vt.indexOf(`content: "${label}"`);
  if (li < 0) return { found: false };
  const head = vt.slice(0, li);
  // 回溯最近前位 col 开启（内层包裹 col；vnode id 为 u64 十进制串）。
  const start = head.lastIndexOf('col vnode_');
  const id = start >= 0 ? vt.slice(start).match(/^col (vnode_\d+)/)?.[1] ?? '' : '';
  if (!id) return { found: false };
  // 括号配平找 span 尾。
  let depth = 0, end = -1;
  for (let i = vt.indexOf('{', start); i < vt.length; i++) {
    if (vt[i] === '{') depth++;
    else if (vt[i] === '}') { depth--; if (depth === 0) { end = i; break; } }
  }
  if (end < 0) return { found: false, id };
  const span = vt.slice(start, end);
  const bboxRaw = span.match(/bbox:\s*\{[^}]*\}/)?.[0] ?? '';
  const num = (k) => { const m = bboxRaw.match(new RegExp(`${k}:\\s*(-?[\\d.]+)`)); return m ? Number(m[1]) : null; };
  const clsRaw = span.match(/class:\s*"[^"]*"/)?.[0] ?? '';
  const styleRaw = span.match(/style:\s*\{[^}]*\}/)?.[0] ?? span.match(/style:\s*"[^"]*"/)?.[0] ?? '';
  return { found: true, id, bboxRaw, w: num('w'), h: num('h'), clsRaw, styleRaw };
}

function measureAll(vt) {
  const out = {};
  for (const lb of ['A0', 'A1', 'B1', 'B2', 'B3', 'C1']) out[lb] = probeNode(vt, lb);
  return out;
}

const near = (actual, expected, tol = 1.5) => actual != null && Math.abs(actual - expected) <= tol;

async function main() {
  const result = { probe: 'W-03-1-vm-style-spike', autoExe: AUTO_EXE, autoExeMtime: null, arms: {}, pass: false };
  try { result.autoExeMtime = fs.statSync(AUTO_EXE).mtime.toISOString(); } catch {}
  const h = await spawnFixture();
  try {
    const mcp = await awaitMcp(h, 30000);
    if (!mcp) throw new Error('MCP 端点 30s 未就绪');
    const vt1s = await awaitFirstFrame(mcp, 30000);
    if (!vt1s) throw new Error('首帧 30s 未就绪（vtree 无 A0）');
    const m1 = measureAll(vt1s);
    result.initial = m1;

    // cycle：按 label 定位按钮，press 后轮询等重渲染（A1 宽变化或超时）。
    const cyc = /button (vnode_\d+) \{label: "cycle"/.exec(vt1s)?.[1] ?? '';
    if (!cyc) throw new Error('cycle 按钮未定位');
    await mcp.action(cyc, 'press', '');
    let m2 = null, vt2 = '';
    const t0 = Date.now();
    while (Date.now() - t0 < 8000) {
      await sleep(300);
      vt2 = await mcp.vtree();
      m2 = measureAll(vt2);
      if (near(m2.A1?.w, 300) || near(m2.B1?.w, 330)) break;
    }
    result.afterCycle = m2;

    // 门臂断言（初值 → 改宽后）。B2 仅记录（web :style 陷阱对照）。
    const gate = {
      A0: [240, 240],  // 字面量对照：稳定不变
      A1: [240, 300],  // 数组字面量集
      B1: [280, 330],  // 裸引用 CSS 声明
      B3: [240, 300],  // 内联拼接 int
      C1: [280, 330],  // class 静态 + style 动态组合（workbench 目标形态）
    };
    for (const [arm, [w1, w2]] of Object.entries(gate)) {
      const a1 = m1[arm] ?? {}, a2 = m2?.[arm] ?? {};
      result.arms[arm] = {
        expect: { initial: w1, after: w2 },
        actual: { initial: a1.w ?? null, after: a2.w ?? null },
        initialHit: near(a1.w, w1),
        afterHit: near(a2.w, w2),
        classInitial: a1.clsRaw ?? '', classAfter: a2.clsRaw ?? '',
        bboxSample: a1.bboxRaw ?? '',
      };
    }
    result.arms.B2 = {
      expect: { initial: 320, after: 380 },  // 非门：VM 能力记录
      actual: { initial: m1.B2?.w ?? null, after: m2?.B2?.w ?? null },
      note: '对照臂（类名文本裸引用——web :style 内联无效；此处只记录 VM 侧是否出程）',
      classInitial: m1.B2?.clsRaw ?? '', classAfter: m2?.B2?.clsRaw ?? '',
    };
    result.pass = Object.entries(result.arms).filter(([k]) => k !== 'B2')
      .every(([, a]) => a.initialHit && a.afterHit);
    result.fixtureOut = h.getOut().slice(-2000);
  } finally {
    try { h.p.kill(); } catch {}
    try { h.dummy.close(); } catch {}
    try { fs.rmSync(h.storage, { force: true }); } catch {}
  }
  fs.mkdirSync(RECEIPT_DIR, { recursive: true });
  const receipt = path.join(RECEIPT_DIR, 'W-03-1-vm-style-spike-receipt.json');
  fs.writeFileSync(receipt, JSON.stringify(result, null, 2), 'utf8');
  console.log(`[style-spike] pass=${result.pass} → ${path.relative(ROOT, receipt)}`);
  for (const [arm, a] of Object.entries(result.arms)) {
    console.log(`  ${arm}: init ${a.actual?.initial ?? a.actual?.initial} → after ${a.actual?.after ?? a.actual?.after}` +
      (a.initialHit != null ? `  (gate ${a.initialHit && a.afterHit ? 'PASS' : 'RED'})` : ''));
  }
  process.exit(result.pass ? 0 : 1);
}

const AUTO_EXE = resolveAutoExe();
main().catch((e) => {
  console.error(`[style-spike] 失败: ${e.message}`);
  process.exit(1);
});
