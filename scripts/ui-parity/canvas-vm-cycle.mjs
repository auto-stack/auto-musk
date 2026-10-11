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
import crypto from 'node:crypto';
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

  // 泄漏清场（P-embed 前置）：早前探针残留的同题 musk/auto 进程会持最大化
  // 窗抢占标题定位（实测 -7,-7,1926,1238 点击入残留窗）。路径过滤只杀本
  // 组（*musk-102*）——studio-live cleanupBetween 同款口径。
  try {
    spawnSync('powershell', ['-NoProfile', '-Command',
      "Get-Process auto,musk -ErrorAction SilentlyContinue | Where-Object { $_.Path -like '*musk-102*' } | ForEach-Object { Stop-Process -Id $_.Id -Force }"],
      { timeout: 30000, windowsHide: true });
  } catch { /* best effort */ }

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

  // 前端 spawn 延后到后端就绪之后（VMHTTP 臂实测根因：vm 后端 AutoVM
  // 初始化 ~40s，前端先起时 boot 检测 17201 不通 → daemon-ensure 自起
  // rust 模式 musk 抢同口 → 前端轮询打到错误实例。后端先就绪则前端
  // AUTO_REUSE_BACKEND 复用本后端——定向复现序）。
  const storage = path.join(RECEIPT_DIR, `cycle-storage-${Date.now()}.json`);
  let front = null;
  const spawnFront = () => {
    front = spawn(AUTO_EXE, ['run', '--render', 'vm'], {
      cwd: ROOT,
      env: { ...process.env, AUTOUI_MCP_PORT: String(mcpPort), AUTO_VM_STORAGE_FILE: storage, AUTO_VM_WINDOW: '1280x800', AUTO_REUSE_BACKEND: '1', AUTO_HTTP_PORT: String(PORT), AUTO_HTTP_BASE: `http://127.0.0.1:${PORT}`, RUST_MIN_STACK: '16777216', AUTO_RF_DEBUG: '1' },
      windowsHide: true, shell: false,
    });
  };
  let frontOut = ''; const attachFront = () => { if (front) { front.stdout.on('data', (b) => (frontOut += b)); front.stderr.on('data', (b) => (frontOut += b)); } };
  const killAll = () => { for (const p of [back, front]) { if (p) { try { spawnSync('taskkill', ['/pid', String(p.pid), '/T', '/F'], { windowsHide: true }); } catch {} } } };

  try {
    let up = false;
    for (let i = 0; i < 90 && !up; i++) { up = (await jfetch(`http://127.0.0.1:${PORT}/api/canvas/status`).catch(() => null))?.status === 200; if (!up) await sleep(1000); }
    assert(`backend-up-${backend}`, up, `:${PORT}`);
    if (!up) throw new Error('backend 90s 未就绪');
    spawnFront();
    attachFront();

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
          return { id: m[2], cls: span.match(/class:\s*"([^"]*)"/)[1], span, x: num('x'), y: num('y'), w: num('w'), h: num('h') };
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
          const bb2 = sp.match(/bbox:\s*\{[^}]*\}/)?.[0] ?? '';
            const num2 = (k) => { const q = bb2.match(new RegExp(k + ':\s*(-?[\d.]+)')); return q ? Number(q[1]) : null; };
            return { id: m[2], x: num2('x'), y: num2('y'), w: x ? Number(x[1]) : null, h: num2('h') };
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

    // ── P-embed（PLAN-102 T-04/T-06 收口臂）：remote-frame 元件 + 操作输
    // 入闭环（760 e2e 同口径）。① 元件在树且几何 laid out；② OS SendInput
    // 点夹具 inc 钮位（vtree bbox + 窗口屏幕原点 + 480x680 fit 换算）→
    // 夹具 MCP 直读 count 递增（status.mcp_base）——remote-frame 输入→
    // bridge WS→app 的完整操作链；③ 截图 md5 变化（帧内容随动）。
    {
      // vtree 宿主叶零 bbox（content "[RemoteFrame]"）——在场断言按类标记；
      // 帧盒由外框 decl 节点（width:640px 类、已证精确）与停靠列 decl 推导：
      // 帧盒 = 外框 − 停靠列宽 − 分隔条 6 − 画布列头部 36。
      const rfPresent = vt.includes('cv-embed-frame') && vt.includes('[RemoteFrame]');
      // 外框 = 含 cv-workbench 的 decl 节点（layout-live outerCol 同口径）。
      let outerBox = null;
      {
        const re2 = /(\w[-\w]*) (vnode_\d+) \{/g; let m2;
        while ((m2 = re2.exec(vt))) {
          const st2 = m2.index; let d2 = 0, e2 = -1;
          for (let i = vt.indexOf('{', st2); i < vt.length; i++) { if (vt[i] === '{') d2++; else if (vt[i] === '}') { d2--; if (d2 === 0) { e2 = i; break; } } }
          if (e2 < 0) continue;
          const sp2 = vt.slice(st2, e2);
          const cl2 = sp2.match(/class:\s*"([^"]*)"/)?.[1] ?? '';
          if (cl2.startsWith('width:') && sp2.includes('cv-workbench')) {
            const bb3 = sp2.match(/bbox:\s*\{[^}]*\}/)?.[0] ?? '';
            const num3 = (k) => { const q = bb3.match(new RegExp(k + ':\\s*(-?[\\d.]+)')); return q ? Number(q[1]) : null; };
            outerBox = { x: num3('x'), y: num3('y'), w: num3('w'), h: num3('h') };
            break;
          }
        }
      }
      assert('embed-remote-frame-present', rfPresent && outerBox != null && (outerBox.w ?? 0) > 300, rfPresent ? `outer bbox=${outerBox?.w}@${outerBox?.x},${outerBox?.y}` : '未找到');
      const sNow2 = (await jfetch(`http://127.0.0.1:${PORT}/api/canvas/status`)).body ?? {};
      if (rfPresent && outerBox && sNow2.mcp_base) {
        const fxState = async () => {
          try {
            const r = await fetch(`http://${String(sNow2.mcp_base).replace(/^https?:\/\//, '').replace(/\/$/, '')}/mcp`, {
              method: 'POST', headers: { 'content-type': 'application/json' },
              body: JSON.stringify({ jsonrpc: '2.0', id: 7, method: 'tools/call', params: { name: 'autoui_state', arguments: {} } }),
            });
            const t = await r.text();
            return /count: (\d+)/.exec(t)?.[1] ?? null;
          } catch { return null; }
        };
        const count0 = await fxState();
        // 窗口屏幕原点 + 点击（PS1 user32；委托形态沿 canvas-embed-probe
        // PS1_SRC——显式 EnumWindowsProc 委托，System.Func 内联 PS 编组失败）。
        const ps1 = path.join(RECEIPT_DIR, 'vm-cycle-click.ps1');
        const PS1C = [
          'param([int]$TargetPid,[int]$X,[int]$Y)',
          'Add-Type -Namespace M102C -Name W -MemberDefinition @"',
          '[DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc cb, IntPtr l);',
          '[DllImport("user32.dll")] public static extern bool SetProcessDPIAware();',
          'public delegate bool EnumWindowsProc(IntPtr h, IntPtr l);',
          '[DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);',
          '[DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);',
          '[DllImport("user32.dll")] public static extern int GetWindowTextLength(IntPtr h);',
          '[DllImport("user32.dll")] public static extern int GetWindowText(IntPtr h, System.Text.StringBuilder s, int n);',
          '[DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);',
          '[DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr h, out RECT r);',
          '[DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr h, ref POINT p);',
          'public struct POINT { public int X; public int Y; }',
          '[DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);',
          '[DllImport("user32.dll")] public static extern bool PostMessageW(IntPtr h, uint m, UIntPtr w, IntPtr l);',
          '[DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr h);',
          '[DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();',
          '[DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);',
          '[DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, IntPtr pid);',
          '[DllImport("kernel32.dll")] public static extern uint GetCurrentThreadId();',
          '[DllImport("user32.dll")] public static extern bool AttachThreadInput(uint a, uint b, bool at);',
          '[DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, uint data, UIntPtr extra);',
          'public struct RECT { public int L; public int T; public int R; public int B; }',
          '"@',
          '# DPI 感知（2x 显示器实测：非感知坐标错 2 倍——点击落聊天区）。',
          '[M102C.W]::SetProcessDPIAware() | Out-Null',
          '$hits = New-Object System.Collections.ArrayList',
          '$cb = {',
          '  param($h, $l)',
          '  if ([M102C.W]::IsWindowVisible($h)) {',
          '    $sb = New-Object System.Text.StringBuilder 256',
          '    [void][M102C.W]::GetWindowText($h, $sb, 256)',
          '    $t = $sb.ToString()',
          '    if ($t -like "*Musk*") { [void]$hits.Add("$h|" + $t) }',
          '  }',
          '  return $true',
          '}',
          '[M102C.W]::EnumWindows($cb, [IntPtr]::Zero) | Out-Null',
          '# 非零 rect 候选（同名零矩形代理窗实测存在）。',
          '$h2 = [IntPtr]::Zero',
          'foreach ($e in $hits) {',
          '  $c = [IntPtr]::new([int64](($e -split "\\|")[0]))',
          '  $rr = New-Object M102C.W+RECT',
          '  [M102C.W]::GetWindowRect($c, [ref]$rr) | Out-Null',
          '  if ($rr.R - $rr.L -gt 100 -and $rr.B - $rr.T -gt 100) { $h2 = $c; break }',
          '}',
          'if ($h2 -eq [IntPtr]::Zero) { Write-Output "nowin"; exit 1 }',
          '$r = New-Object M102C.W+RECT',
          '[M102C.W]::GetClientRect($h2, [ref]$r) | Out-Null',
          '$pt = New-Object M102C.W+POINT',
          '$pt.X = 0; $pt.Y = 0',
          '[M102C.W]::ClientToScreen($h2, [ref]$pt) | Out-Null',
          '$r.L = $pt.X; $r.T = $pt.Y; $r.R = $pt.X + $r.R; $r.B = $pt.Y + $r.B',
          'Write-Output ("rect " + $r.L + "," + $r.T + "," + $r.R + "," + $r.B + "," + [int64]$h2)',
          '# 前台化（AttachThreadInput 旁路；非前台全局注入 press 被丢——760 e2e 注记）。',
          '$fg = [M102C.W]::GetForegroundWindow()',
          'if ($fg -ne $h2) {',
          '  $ct = [M102C.W]::GetCurrentThreadId()',
          '  $ft = [M102C.W]::GetWindowThreadProcessId($fg, [IntPtr]::Zero)',
          '  $tt = [M102C.W]::GetWindowThreadProcessId($h2, [IntPtr]::Zero)',
          '  [void][M102C.W]::AttachThreadInput($ct, $ft, $true)',
          '  [void][M102C.W]::AttachThreadInput($ct, $tt, $true)',
          '  [void][M102C.W]::SetForegroundWindow($h2)',
          '  [void][M102C.W]::AttachThreadInput($ct, $ft, $false)',
          '  [void][M102C.W]::AttachThreadInput($ct, $tt, $false)',
          '  Start-Sleep -Milliseconds 250',
          '}',
          '# PostMessage 直投（WM_MOUSEMOVE/0x200、DOWN/0x201、UP/0x202）——',
          '# lparam = 窗口逻辑域（DPI 由 winit 消化，760 e2e 逐点对账实测）。',
          '# 真移光标（canvas 只吃真实 move；760 e2e 同款 SetCursorPos+Post）。',
          '$dpi = [M102C.W]::GetDpiForWindow($h2) / 96.0',
          'if ($dpi -lt 0.5) { $dpi = 1.0 }',
          '[M102C.W]::SetCursorPos([int]($r.L + $X * $dpi), [int]($r.T + $Y * $dpi)) | Out-Null',
          'Start-Sleep -Milliseconds 150',
          '$lp = [IntPtr]::new((($Y -shl 16) -bor ($X -band 0xFFFF)))',
          '[void][M102C.W]::PostMessageW($h2, 0x200, [UIntPtr]::Zero, $lp)',
          'Start-Sleep -Milliseconds 60',
          '[void][M102C.W]::PostMessageW($h2, 0x201, [UIntPtr]::new([uint64]1), $lp)',
          'Start-Sleep -Milliseconds 60',
          '[void][M102C.W]::PostMessageW($h2, 0x202, [UIntPtr]::Zero, $lp)',
          'Start-Sleep -Milliseconds 60',
          '[void][M102C.W]::PostMessageW($h2, 0x200, [UIntPtr]::Zero, $lp)',
          'Write-Output "clicked"',
        ].join('\r\n');
        fs.writeFileSync(ps1, PS1C, 'utf8');
        const psRun = (args2) => {
          const r = spawnSync('powershell', ['-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-File', ps1, ...args2], { encoding: 'utf8', timeout: 20000, windowsHide: true });
          return (r.stdout ?? '') + (r.stderr ?? '');
        };
        // 窗口定位（标题 × 非零客户区；rect 仅取证——点击走 PostMessage
        // lparam = 窗口逻辑域，无需屏幕换算）。
        const rectOut = psRun([`-TargetPid ${front.pid}`, '-X 0', '-Y 0']);
        const rect = /rect (-?\d+),(-?\d+),(-?\d+),(-?\d+)/.exec(rectOut);
        const hwndInt = Number(/rect (-?\d+),(-?\d+),(-?\d+),(-?\d+),(\d+)/.exec(rectOut)?.[5] ?? 0);
        result.embedRectOut = rectOut.slice(0, 300);
        // 截图/md5 取证助手（先声明后使用；正则反斜杠曾被历史 sed 削毁——
        // shotFile 恒 null → md5 断言退化为 'nof' 前缀比较（F-06-2 家族））。
        const shotFile = (t) => { const m = /saved to:\s*(\S+)/.exec(String(t)); if (!m) return null; let f = m[1]; if (f.startsWith(String.fromCharCode(92, 92, 63, 92))) f = f.slice(4); return f; };
        const fileMd5 = (f) => { try { return crypto.createHash('md5').update(fs.readFileSync(f)).digest('hex'); } catch { return null; } };
        const md5 = (t) => { const f = shotFile(t); return f ? fileMd5(f) : 'nof' + String(t).slice(0, 16); };
        // 外框 decl 解析（vtree → {x,y,w,h}；点击重试时以最新几何重算点位）。
        const outerOf = (vtText) => {
          const re2 = /(\w[-\w]*) (vnode_\d+) \{/g; let m2;
          while ((m2 = re2.exec(vtText))) {
            const st2 = m2.index; let d2 = 0, e2 = -1;
            for (let i = vtText.indexOf('{', st2); i < vtText.length; i++) { if (vtText[i] === '{') d2++; else if (vtText[i] === '}') { d2--; if (d2 === 0) { e2 = i; break; } } }
            if (e2 < 0) continue;
            const sp2 = vtText.slice(st2, e2);
            const cl2 = sp2.match(/class:\s*"([^"]*)"/)?.[1] ?? '';
            if (cl2.startsWith('width:') && sp2.includes('cv-workbench')) {
              const bb3 = sp2.match(/bbox:\s*\{[^}]*\}/)?.[0] ?? '';
              const num3 = (k) => { const q = bb3.match(new RegExp(k + ':\\s*(-?[\\d.]+)')); return q ? Number(q[1]) : null; };
              return { x: num3('x'), y: num3('y'), w: num3('w'), h: num3('h') };
            }
          }
          return null;
        };
        // 截图基线（帧内容随动证据）。
        const shot0 = await mcp.call('autoui_screenshot', {}).catch(() => '');
        // 点击（≤3 次重测重试：boot/布局毛刺下单测点位可能漂移；命中
        // 判据 = 夹具 count 实际递增，点击器返回值仅取证）。
        let clicked = false, detail = `rect=${rectOut.slice(0, 40)}`, count1 = null;
        for (let attempt = 0; attempt < 3 && count1 == null; attempt++) {
          const vtN = attempt === 0 ? vt : (await vtRetry() || vt);
          const ob = attempt === 0 ? outerBox : (outerOf(vtN) ?? outerBox);
          if (!ob) break;
          // PostMessage lparam = 窗口逻辑域直通：帧盒 = 外框 bbox − 停靠列
          // − 分隔条 6 − 停靠条29 + 头部36；夹具 480x680 contain 居中。
          const dockW = dockCol(vtN)?.w ?? 240;
          const fx0 = (ob.x ?? 0) + dockW + 6;
          const fy0 = (ob.y ?? 0) + 29 + 36;
          const fw = (ob.w ?? 640) - dockW - 6;
          const fh = (ob.h ?? 800) - 29 - 36;
          const s = Math.min(fw / 480, fh / 680);
          const ox = (fw - 480 * s) / 2, oy = (fh - 680 * s) / 2;
          const lx = Math.round(fx0 + ox + 48 * s);
          const ly = Math.round(fy0 + oy + 36 * s);
          const clickOut = spawnSync('python', [path.join(ROOT, 'scripts/ui-py/click-hwnd.py'), String(lx), String(ly), '2.0'], { encoding: 'utf8', timeout: 30000, windowsHide: true });
          clicked = (clickOut.stdout ?? '').includes('clicked');
          detail = `lp=${lx},${ly} dockW=${dockW} try=${attempt + 1} ${clicked ? 'posted' : String(clickOut.stdout ?? clickOut.stderr ?? '').slice(0, 30)}`;
          for (let i = 0; i < 6; i++) { const c = await fxState(); if (c != null && Number(c) > Number(count0 ?? 0)) { count1 = c; break; } await sleep(500); }
        }
        await sleep(2500);
        if (count1 == null) count1 = await fxState();
        const shot1 = await mcp.call('autoui_screenshot', {}).catch(() => '');
        // 帧随动取证：两帧落盘 + md5（编码确定——md5 相异即内容真变）。
        try {
          const f0 = shotFile(shot0), f1 = shotFile(shot1);
          if (f0) fs.copyFileSync(f0, path.join(RECEIPT_DIR, 'W-06-pembed-shot0.png'));
          if (f1) fs.copyFileSync(f1, path.join(RECEIPT_DIR, 'W-06-pembed-shot1.png'));
          result.embedShots = { md5_0: md5(shot0), md5_1: md5(shot1) };
        } catch { /* best effort */ }
        result.embedOperate = { count0, count1, clicked, detail };
        assert('embed-operate-input-roundtrip',
          count0 != null && count1 != null && Number(count1) > Number(count0),
          `count ${count0}→${count1}（OS 点击→remote-frame→WS→夹具）${detail}`);
        assert('embed-frame-follows-input', md5(shot0) !== md5(shot1), `截图 md5 随输入变化（帧内容更新）`);
      } else {
        assert('embed-operate-input-roundtrip', false, sNow2.mcp_base ? 'rfNode 缺' : 'mcp_base 缺');
      }
    }

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
    // 定位锚 = events onclick: ".Stop"（定向复现实证：handler 直达
    // CanvasCanvasColumn.Stop → cv_state=stopped；文本 × 首匹配会误中
    // 他钮——F-06-2 定罪）。
    vt = await vtRetry();
    {
      const re = new RegExp('button (vnode_' + String.fromCharCode(92) + 'd+) ' + String.fromCharCode(92) + '{[^{}]*?label: "×"[^{}]*?events: ' + String.fromCharCode(92) + '{[^}]*onclick: "' + String.fromCharCode(92) + '.Stop"');
      let sid = re.exec(vt)?.[1] ?? '';
      if (!sid) {
        const j = vt.indexOf('.Stop');
        const st2 = j >= 0 ? vt.slice(0, j).lastIndexOf('button vnode_') : -1;
        sid = st2 >= 0 ? vt.slice(st2).match(/^button (vnode_\d+)/)?.[1] ?? '' : '';
      }
      result.stopPress = { sid, echo: null };
      if (sid) { result.stopPress.echo = String(await mcp.action(sid, 'press', '').catch((e) => `ERR ${e.message}`)).slice(0, 90); } else { await pressByContent(vt, '×'); }
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
    result.backendTail = backOut.slice(-9000);
    result.frontendTail = frontOut.slice(-9000);
    try { fs.writeFileSync(path.join(RECEIPT_DIR, 'W-06-vm-cycle-' + backend + '-frontend-full.log'), frontOut, 'utf8'); } catch { /* best effort */ }
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
