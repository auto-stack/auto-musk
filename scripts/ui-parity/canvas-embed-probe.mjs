#!/usr/bin/env node
// canvas-embed-probe.mjs — PLAN-102 T-01: canvas 内嵌呈现/输入能力探针。
//
// 决策门证据（docs/reports/102-canvas-embed-decision.md 消费本探针收据）：
//   P1 baseline-visible : 现状基线——可见原生窗 + MCP 截图/状态（对照臂）。
//   P2 hidden-frames    : AUTO_VM_HIDDEN=1（auto-lang S1 接缝）→ 无可见窗
//                         census + 60s 连续（计数器 ×12 → 帧随动、typed 态
//                         断言）——候选 B 主证【T-01 实证证伪：隐藏窗帧管线
//                         冻结（blank/stale，winit 不向隐藏窗投递 Redraw），
//                         收据 = 决策文档否证证据】。
//   P3 input-complete   : 隐藏窗上 MCP 输入全家桶——press/toggle/type_text
//                         (中文)/submit/scroll/drag(Float 坐标)（输入通道与
//                         渲染臂正交的证据面——状态侧全通、帧侧冻结）。
//   P4 toggle-norestart : 宿主侧 ShowWindow ×3 往返——PID/状态保持实证，
//                         帧连续断言（同因冻结；SW_SHOW 后亦不复原）。
//   P5 remote-arm-census: auto rqhost + AUTO_VM_RENDER=remote——app 无自有窗
//                         census + daemon [rqhost] perf 帧流观测行（夹具
//                         ops/texts 语义过线）——候选 A 上臂证据（选定路线）。
//                         finding：rq 客户端不启动 MCP（依赖缺口 D-2）。
//
// Usage: node scripts/ui-parity/canvas-embed-probe.mjs --frontend both
//                            [--candidate A|B|both] [--quick]
//   --candidate A（缺省，T-01 选定路线）: P1 + P5；
//   --candidate B（证伪仪器，预期红）    : P1 + P2 + P3 + P4；
//   --candidate both                      : 全量。
//   --frontend both|web|vm : T-01 阶段两轨消费同一能力层（MCP/pipe/HTTP 端点
//     同源）；探针盖章「capability 层」，双轨前端接线断言在 T-04 扩展。
//   --quick : P2 连续窗 60s→20s、P4 往返 3→1（迭代用；正式收据不用）。
// AUTO_EXE 解析序：env → musk-102 依赖构建（含 AUTO_VM_HIDDEN 接缝）→
//   auto-lang 主检出构建（P2 会因无接缝显式红）。
// 收据：tmp/ui-parity/PLAN-102/<probe>-receipt.json（含命令、退出码、时间戳、
//   auto.exe 路径与 mtime、逐项断言）。退出 0 = 所请求候选的探针全 pass。

import fs from 'node:fs';
import path from 'node:path';
import http from 'node:http';
import net from 'node:net';
import crypto from 'node:crypto';
import { spawn, spawnSync, execSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const FIXTURE = path.join(ROOT, 'tests', 'ui-parity', 'probes', 'canvas-embed', 'embed-fixture');
const RECEIPT_DIR = path.join(ROOT, 'tmp', 'ui-parity', 'PLAN-102');

const args = process.argv.slice(2);
const frontend = args.includes('--frontend') ? args[args.indexOf('--frontend') + 1] : 'both';
const quick = args.includes('--quick');
if (!['both', 'web', 'vm'].includes(frontend)) {
  console.error(`[canvas-embed] --frontend 取值非法: ${frontend}（both|web|vm）`);
  process.exit(2);
}

// AUTO_EXE 解析（探针专用序：依赖构建优先——P2 需 AUTO_VM_HIDDEN 接缝）。
function resolveAutoExe() {
  const candidates = [
    process.env.AUTO_EXE,
    'D:/autostack/.wt/musk-102/target/release/auto.exe',
    'D:/autostack/auto-lang/target/release/auto.exe',
  ].filter(Boolean);
  for (const c of candidates) {
    if (fs.existsSync(c)) return c;
  }
  return null;
}
const AUTO_EXE = resolveAutoExe();
if (!AUTO_EXE) {
  console.error('[canvas-embed] auto.exe 不可达：设 AUTO_EXE 或构建 auto-lang');
  process.exit(2);
}
const AUTO_EXE_MTIME = fs.statSync(AUTO_EXE).mtime.toISOString();

function log(s) { console.log(`[canvas-embed] ${s}`); }
function killTree(p) {
  if (!p || p.exitCode !== null) return;
  try { execSync(`taskkill /T /F /PID ${p.pid}`, { stdio: 'ignore', windowsHide: true }); } catch { /* best effort */ }
}
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

function receipt(name, obj) {
  fs.mkdirSync(RECEIPT_DIR, { recursive: true });
  const payload = {
    probe: name, plan: 102, frontend, auto_exe: AUTO_EXE, auto_exe_mtime: AUTO_EXE_MTIME,
    quick, ts: new Date().toISOString(), ...obj,
  };
  fs.writeFileSync(path.join(RECEIPT_DIR, `${name}-receipt.json`), JSON.stringify(payload, null, 2));
  log(`${name}: ${obj.pass ? 'PASS' : 'FAIL'} → ${path.relative(ROOT, path.join(RECEIPT_DIR, `${name}-receipt.json`))}`);
}

async function freePort() {
  return new Promise((res, rej) => {
    const s = net.createServer();
    s.listen(0, '127.0.0.1', () => { const p = s.address().port; s.close(() => res(p)); });
    s.on('error', rej);
  });
}

// ── win32 窗口 census / ShowWindow（宿主侧呈现切换原语） ──────────────────
// PS 脚本落盘 + 参数数组调用（-Command 串过 cmd.exe 会被引号/变量搅碎——
// '$vis' not recognized 实证）。
const PS1 = path.join(RECEIPT_DIR, 'win-census.ps1');
const PS1_SRC = String.raw`
param(
  [int]$TargetPid,
  [ValidateSet("census", "show")] [string]$Mode,
  [int]$Cmd = 0
)
Add-Type -Namespace M102 -Name W -MemberDefinition @"
[DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc cb, IntPtr l);
public delegate bool EnumWindowsProc(IntPtr h, IntPtr l);
[DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
[DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
[DllImport("user32.dll")] public static extern int GetWindowTextLength(IntPtr h);
[DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
"@
$hits = New-Object System.Collections.ArrayList
$cb = {
  param($h, $l)
  $wpid = 0
  [M102.W]::GetWindowThreadProcessId($h, [ref]$wpid) | Out-Null
  if ($wpid -eq $TargetPid) {
    $vis = [M102.W]::IsWindowVisible($h)
    $tl = [M102.W]::GetWindowTextLength($h)
    [void]$hits.Add("$h|$vis|$tl")
  }
  return $true
}
[M102.W]::EnumWindows($cb, [IntPtr]::Zero) | Out-Null
if ($Mode -eq "census") {
  $hits -join ";"
} else {
  $targets = @($hits | Where-Object { $_ -match "^\d+\|True\|\d+$" })
  if ($targets.Count -eq 0) {
    $all = @($hits | Where-Object { $_ -match "^\d+\|(True|False)\|\d+$" })
    if ($all.Count -eq 0) { "nofind" ; exit 1 }
    $targets = $all
  }
  $first = [string]$targets[0]
  $hwnd = [int64]($first.Split("|")[0])
  [M102.W]::ShowWindow([IntPtr]$hwnd, $Cmd) | Out-Null
  "shown $hwnd $Cmd"
}
`;
function psInvoke(targetPid, mode, cmd) {
  fs.mkdirSync(RECEIPT_DIR, { recursive: true });
  fs.writeFileSync(PS1, PS1_SRC, 'utf8');
  const r = spawnSync('powershell', [
    '-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass',
    '-File', PS1, '-TargetPid', String(targetPid), '-Mode', mode, '-Cmd', String(cmd ?? 0),
  ], { encoding: 'utf8', windowsHide: true, timeout: 20000 });
  return { status: r.status, out: (r.stdout ?? '') + (r.stderr ?? '') };
}
/** pid 的顶层窗表 [{hwnd, visible, titleLen}]。 */
function windowCensus(pid) {
  const { out } = psInvoke(pid, 'census');
  return out.split(String.fromCharCode(59)).map((l) => l.trim()).filter((l) => /^\d+\|/.test(l)).map((l) => {
    const [hwnd, vis, tl] = l.split('|');
    return { hwnd: Number(hwnd), visible: vis === 'True', titleLen: Number(tl) };
  });
}
/** 对 pid 的真实窗 ShowWindow(cmd)（优先可见窗，兜底任意标题窗）。 */
function showWindowByPid(pid, cmd) {
  const r = psInvoke(pid, 'show', cmd);
  return /^shown /m.test(r.out);
}
const SW_HIDE = 0, SW_SHOW = 5;

// ── 夹具 spawn + MCP 客户端 ───────────────────────────────────────────────
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

async function spawnFixture({ env = {}, storageTag = 'p' }) {
  const mcpPort = await freePort();
  const backPort = await freePort();
  const storage = path.join(RECEIPT_DIR, `storage-${storageTag}-${Date.now()}.json`);
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
      ...env,
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

/** 首帧就绪：vtree 非 Error（窗口已绘）。 */
async function awaitFirstFrame(mcp, timeoutMs = 30000) {
  const t0 = Date.now();
  while (Date.now() - t0 < timeoutMs) {
    const vt = await mcp.vtree();
    if (!vt.startsWith('Error:')) return vt;
    await sleep(400);
  }
  return null;
}

/** autoui_screenshot → {file, md5}；删旧帧防堆积（musk watchdog 同款）。 */
let lastShot = null;
async function shot(mcp) {
  const t = await mcp.call('autoui_screenshot', {});
  const m = /Screenshot saved to: (.+)/.exec(t);
  if (!m) return { ok: false, raw: t.slice(0, 200) };
  const file = m[1].trim();
  const buf = fs.readFileSync(file);
  const md5 = crypto.createHash('md5').update(buf).digest('hex');
  if (lastShot && lastShot !== file) { try { fs.unlinkSync(lastShot); } catch { /* best effort */ } }
  lastShot = file;
  return { ok: buf.length > 1000, file, md5, bytes: buf.length };
}

function stateField(st, name) {
  return new RegExp(`\\s${name}: (.*) \\((\\w+)\\)`).exec(st)?.[1];
}

/** vtree 定位：夹具关键节点 vnode id（vtree Atom 形态实证——本轮活体校准）。 */
function locateIds(vt) {
  // scrollCol：class 含 overflow-y-auto 的节点属最近的前置 `col vnode_`
  //（col 的 props 花括号内含嵌套 box/style 块，正则不可跨 —— 后向搜索）。
  let scrollCol = '';
  const oi = vt.indexOf('overflow-y-auto');
  if (oi >= 0) {
    const head = vt.slice(0, oi);
    const m = /col (vnode_\d+) \{[^]*$/.exec(head.slice(Math.max(0, head.lastIndexOf('col vnode_'))));
    scrollCol = head.slice(Math.max(0, head.lastIndexOf('col vnode_'))).match(/^col (vnode_\d+)/)?.[1] ?? '';
  }
  return {
    incBtn: /button (vnode_\d+) \{label: "inc"/.exec(vt)?.[1] ?? '',
    toggleBtn: /button (vnode_\d+) \{label: "toggle"/.exec(vt)?.[1] ?? '',
    input: /input (vnode_\d+)/.exec(vt)?.[1] ?? '',
    commitBtn: /button (vnode_\d+) \{label: "commit"/.exec(vt)?.[1] ?? '',
    // 行渲染为可点 button（text+onclick 在 VM 轨物化为 button vnode——实证）。
    row29: /button (vnode_\d+) \{label: "row-29"/.exec(vt)?.[1] ?? '',
    scrollCol,
  };
}

// ── P1 baseline-visible ──────────────────────────────────────────────────
async function probeBaseline() {
  const result = { probe: 'P1-baseline-visible' };
  const h = await spawnFixture({ storageTag: 'p1' });
  try {
    const t0 = Date.now();
    const mcp = await awaitMcp(h, 30000);
    result.mcpReadyMs = mcp ? Date.now() - t0 : -1;
    if (!mcp) throw new Error('MCP 端点 30s 未就绪');
    const vt = await awaitFirstFrame(mcp);
    result.firstFrame = !!vt && vt.includes('inc');
    const wins = windowCensus(h.p.pid);
    result.visibleWindowCount = wins.filter((w) => w.visible && w.titleLen > 0).length;
    const s = await shot(mcp);
    result.screenshotOk = s.ok === true;
    const ids = locateIds(vt);
    const before = stateField(await mcp.state(), 'count');
    await mcp.action(ids.incBtn, 'press', '');
    await sleep(400);
    const after = stateField(await mcp.state(), 'count');
    result.pressWorks = before === '0' && after === '1';
    result.pass = result.firstFrame && result.visibleWindowCount >= 1 && result.screenshotOk && result.pressWorks;
  } catch (e) {
    result.error = String(e.message ?? e);
    result.pass = false;
  } finally {
    killTree(h.p); h.dummy.close();
    try { fs.rmSync(h.storage, { force: true }); } catch { /* best effort */ }
  }
  receipt('P1-baseline-visible', result);
  return result;
}

// ── P2 hidden-frames（60s 连续出帧，无可见小窗） ────────────────────────
async function probeHidden(keep) {
  const rounds = quick ? 4 : 12;
  const roundMs = 5000;
  const result = { probe: 'P2-hidden-frames', rounds, roundMs };
  const h = keep;
  try {
    const t0 = Date.now();
    const mcp = await awaitMcp(h, 30000);
    result.mcpReadyMs = mcp ? Date.now() - t0 : -1;
    if (!mcp) throw new Error('MCP 端点 30s 未就绪');
    const vt = await awaitFirstFrame(mcp);
    if (!vt) throw new Error('30s 无首帧 vtree');
    // census：无可见窗（有标题的可见顶层窗必须为 0——AUTO_VM_HIDDEN 接缝主断言）。
    const wins = windowCensus(h.p.pid);
    result.windowCensus = wins;
    result.hiddenNoVisibleWindow = wins.filter((w) => w.visible && w.titleLen > 0).length === 0;
    if (!result.hiddenNoVisibleWindow) {
      result.hint = '可见窗非零：AUTO_EXE 可能不含 AUTO_VM_HIDDEN 接缝（musk-102 依赖构建才支持）';
    }
    const ids = locateIds(vt);
    // 60s 连续：每轮 press → typed count 断言 + 帧随动（md5 变化）。
    const frameMd5s = [];
    const counts = [];
    const tStart = Date.now();
    let mcpErrors = 0;
    for (let i = 1; i <= rounds; i++) {
      try {
        await mcp.action(ids.incBtn, 'press', '');
        await sleep(350);
        counts.push(stateField(await mcp.state(), 'count'));
        const s = await shot(mcp);
        if (s.ok) frameMd5s.push(s.md5);
      } catch (e) { mcpErrors++; result[`round${i}Error`] = String(e.message ?? e).slice(0, 160); }
      const remain = roundMs - (Date.now() - tStart - (i - 1) * roundMs);
      if (remain > 0) await sleep(remain);
    }
    result.durationMs = Date.now() - tStart;
    result.countsProgressed = counts.length === rounds && counts.every((c, i) => c === String(i + 1));
    result.distinctFrames = new Set(frameMd5s).size;
    result.framesFollowState = result.distinctFrames >= Math.min(rounds - 1, 6);
    result.mcpErrors = mcpErrors;
    result.aliveAtEnd = h.p.exitCode === null;
    result.pass = result.hiddenNoVisibleWindow && result.countsProgressed && result.framesFollowState && mcpErrors === 0 && result.aliveAtEnd;
  } catch (e) {
    result.error = String(e.message ?? e);
    result.pass = false;
  }
  receipt('P2-hidden-frames', result);
  return result;
}

// ── P3 input-complete（隐藏窗上操作态输入全家桶） ───────────────────────
async function probeInput(h) {
  const result = { probe: 'P3-input-complete' };
  try {
    const mcp = await awaitMcp(h, 5000) ?? h.__mcp;
    if (!mcp) throw new Error('MCP 不在');
    h.__mcp = mcp;
    const vt = await mcp.vtree();
    const ids = locateIds(vt);
    const inputId = ids.input;
    const row29Id = ids.row29;
    const scrollColId = ids.scrollCol;
    result.locators = { inputId: !!inputId, row29Id: !!row29Id, scrollColId: !!scrollColId, incBtn: !!ids.incBtn, toggleBtn: !!ids.toggleBtn };

    // 1) toggle（press 布尔翻转——目标 = toggle 按钮）。
    await mcp.action(ids.toggleBtn, 'press', '');
    await sleep(300);
    result.toggleWorks = stateField(await mcp.state(), 'on') === 'true';

    // 2) 中文 type_text（提交级 IME——组合在宿主、提交串进 app）。
    await mcp.action(inputId, 'type_text', '你好世界');
    await sleep(400);
    const stName = stateField(await mcp.state(), 'name');
    result.chineseTypeText = stName === '"你好世界"';
    result.nameEcho = stName;

    // 3) submit（onenter/onsubmit 派发 → committed）。
    let submitted = false;
    try {
      await mcp.action(inputId, 'submit', '');
      await sleep(350);
      submitted = stateField(await mcp.state(), 'committed').includes('你好世界');
    } catch { /* 旧二进制无 submit 则按钮兜底 */ }
    if (!submitted) {
      await mcp.action(ids.commitBtn, 'press', '');
      await sleep(350);
      submitted = stateField(await mcp.state(), 'committed').includes('你好世界');
    }
    result.submitWorks = submitted;

    // 4) scroll + 视口外行点击（滚动行为真断言：row-29 默认在视口外）。
    await mcp.action(scrollColId, 'scroll', '0,1200');
    await sleep(400);
    await mcp.action(row29Id, 'press', '');
    await sleep(400);
    result.scrollThenClickDistantRow = stateField(await mcp.state(), 'picked_row') === '29';

    // 5) __mcp_drag Float 坐标（逻辑坐标 = coords 值域；typed 斁言 ≤0.5px）。
    const SEP = String.fromCharCode(31);
    const spec = ['EmbedFixture', 'Down', 'Move', 'Up', '120.5,40.0;80.25,90.5'].join(SEP);
    await mcp.action('aura_0', 'drag', spec);
    await sleep(500);
    const st = await mcp.state();
    const mx = parseFloat(/mx: (-?[\d.]+) \(float\)/.exec(st)?.[1] ?? 'NaN');
    const my = parseFloat(/my: (-?[\d.]+) \(float\)/.exec(st)?.[1] ?? 'NaN');
    result.dragTypedFloat = Math.abs(mx - 80.25) <= 0.5 && Math.abs(my - 90.5) <= 0.5;
    result.dragEcho = { mx, my, dragDone: stateField(st, 'drag_done') === 'true' };

    result.pass = result.toggleWorks && result.chineseTypeText && result.submitWorks && result.scrollThenClickDistantRow && result.dragTypedFloat;
  } catch (e) {
    result.error = String(e.message ?? e);
    result.pass = false;
  }
  receipt('P3-input-complete', result);
  return result;
}

// ── P4 toggle-norestart（宿主侧 ShowWindow 往返，无重启） ────────────────
async function probeToggle(h) {
  const cycles = quick ? 1 : 3;
  const result = { probe: 'P4-toggle-norestart', cycles };
  try {
    const mcp = await awaitMcp(h, 5000) ?? h.__mcp;
    if (!mcp) throw new Error('MCP 不在');
    h.__mcp = mcp;
    const pid = h.p.pid;
    const ids = locateIds(await mcp.vtree());
    const rounds = [];
    let count = Number(stateField(await mcp.state(), 'count') ?? '0');
    for (let i = 1; i <= cycles; i++) {
      // → 独立窗（SHOW）。
      const shown = showWindowByPid(pid, SW_SHOW);
      await sleep(600);
      const visShown = windowCensus(pid).filter((w) => w.visible && w.titleLen > 0).length;
      count += 1;
      await mcp.action(ids.incBtn, 'press', '');
      await sleep(350);
      const stShown = stateField(await mcp.state(), 'count');
      const sShotShown = await shot(mcp);
      // → 内嵌（HIDE）。
      const hidden = showWindowByPid(pid, SW_HIDE);
      await sleep(600);
      const visHidden = windowCensus(pid).filter((w) => w.visible && w.titleLen > 0).length;
      const sShotHidden = await shot(mcp);
      rounds.push({
        shown, hidden,
        visibleInWindowMode: visShown === 1,
        visibleInEmbeddedMode: visHidden === 0,
        statePreservedAcrossToggle: stShown === String(count),
        framesInWindowMode: sShotShown.ok === true,
        framesInEmbeddedMode: sShotHidden.ok === true,
      });
    }
    result.rounds = rounds;
    result.pidUnchanged = h.p.exitCode === null;
    result.pass = result.pidUnchanged && rounds.every((r) =>
      r.shown && r.hidden && r.visibleInWindowMode && r.visibleInEmbeddedMode &&
      r.statePreservedAcrossToggle && r.framesInWindowMode && r.framesInEmbeddedMode);
  } catch (e) {
    result.error = String(e.message ?? e);
    result.pass = false;
  }
  receipt('P4-toggle-norestart', result);
  return result;
}

// ── P5 remote-arm-census（rqhost + remote 臂，候选 A 上臂证据） ──────────
// 实证口径（T-01 勘定）：rq 客户端臂**不启动 MCP**（run_vm_rqhost_client
// 绕过 run_session，MCP/vtree/pick 面缺席）——记录为依赖缺口 finding，
// 非探针失败面；输入闭环以 PLAN-508 T6 实机收据在案佐证
//（auto-lang docs/plans/reports/assets/508/p508-t6-browser-after.png 等）。
async function probeRemoteArm() {
  const result = { probe: 'P5-remote-arm-census' };
  const daemon = spawn(AUTO_EXE, ['rqhost', '--pipe', 'musk102-p5-rqhost'], {
    cwd: FIXTURE, env: { ...process.env }, windowsHide: true, shell: false,
  });
  let daemonOut = '';
  daemon.stdout.on('data', (b) => (daemonOut += b));
  daemon.stderr.on('data', (b) => (daemonOut += b));
  const h = await spawnFixture({
    storageTag: 'p5',
    env: { AUTO_RQHOST_WELLKNOWN: 'musk102-p5-rqhost', AUTO_RQHOST_PIPE: 'musk102-p5-rqhost', AUTO_VM_RENDER: 'remote' },
  });
  try {
    // 就绪判据 = daemon 收到本夹具的帧流观测行（app 被 adopt + DisplayList 过线）。
    const t0 = Date.now();
    while (Date.now() - t0 < 30000 && !/perf `EmbedFixture`/.test(daemonOut)) await sleep(400);
    result.adoptMs = /perf `EmbedFixture`/.test(daemonOut) ? Date.now() - t0 : -1;
    if (result.adoptMs < 0) throw new Error('daemon 30s 未收到 EmbedFixture 帧流');
    // app 自身无窗（headless）——rqhost daemon 承载呈现。
    await sleep(1500);
    const wins = windowCensus(h.p.pid);
    result.appWindowCensus = wins;
    result.appHeadless = wins.filter((w) => w.titleLen > 0).length === 0;
    result.appAlive = h.p.exitCode === null;
    // daemon 帧流观测行（ops/texts>0 = 夹具真实渲染语义过线，非空帧）。
    result.daemonFrameLines = (daemonOut.match(/[^\r\n]*\[rqhost\] perf[^\r\n]*/g) ?? []).slice(-6);
    const m = /ops=(\d+) texts=(\d+)/.exec(result.daemonFrameLines.at(-1) ?? '');
    result.daemonEmbedsFixture = !!m && Number(m[1]) > 0 && Number(m[2]) > 0;
    // daemon 呈现窗在册（其自绘图面）。
    const dwins = windowCensus(daemon.pid);
    result.daemonWindowCount = dwins.filter((w) => w.visible && w.titleLen > 0).length;
    result.daemonAlive = daemon.exitCode === null;
    // finding：rq 客户端 MCP 缺席（vtree/pick/state 面不可用——依赖缺口 D-2）。
    result.findingMcpAbsentOnRqClient = !/AutoUI MCP: listening on/.test(h.getOut());
    result.pass = result.appHeadless && result.appAlive && result.daemonEmbedsFixture &&
      result.daemonAlive && result.adoptMs >= 0;
  } catch (e) {
    result.error = String(e.message ?? e);
    result.pass = false;
    result.daemonOutTail = daemonOut.slice(-600);
  } finally {
    killTree(h.p); killTree(daemon); h.dummy.close();
    try { fs.rmSync(h.storage, { force: true }); } catch { /* best effort */ }
  }
  receipt('P5-remote-arm-census', result);
  return result;
}

// ── 主流程 ───────────────────────────────────────────────────────────────
// 候选选择（T-01 决策门产物，docs/reports/102-canvas-embed-decision.md）：
//   A（缺省）：remote/rq 臂——P1 + P5。B 证伪后选定路线的Capability面。
//   B：隐藏窗 + MCP——P1 + P2 + P3 + P4（本栈实证冻结——保留为证伪仪器，
//      收据即决策文档的否证证据；预期红）。
//   both：全量。
async function main() {
  const candIdx = args.indexOf('--candidate');
  const candidate = candIdx >= 0 ? args[candIdx + 1] : 'A';
  if (!['A', 'B', 'both'].includes(candidate)) {
    console.error('[canvas-embed] --candidate 取值非法: ' + candidate + '（A|B|both）');
    process.exit(2);
  }
  log(`AUTO_EXE = ${AUTO_EXE} (mtime ${AUTO_EXE_MTIME})`);
  log(`frontend = ${frontend}, candidate = ${candidate}${quick ? ' [quick]' : ''}`);
  const results = [];

  if (candidate === 'A' || candidate === 'both') {
    results.push(await probeBaseline());
    results.push(await probeRemoteArm());
  }

  if (candidate === 'B' || candidate === 'both') {
    if (candidate === 'B') results.push(await probeBaseline());
    // P2/P3/P4 共享一个隐藏会话（候选 B 产品形态：embedded 起步 → 窗口往返）。
    const hidden = await spawnFixture({ storageTag: 'p2', env: { AUTO_VM_HIDDEN: '1' } });
    try {
      results.push(await probeHidden(hidden));
      results.push(await probeInput(hidden));
      results.push(await probeToggle(hidden));
    } finally {
      killTree(hidden.p); hidden.dummy.close();
      try { fs.rmSync(hidden.storage, { force: true }); } catch { /* best effort */ }
    }
  }

  const failed = results.filter((r) => !r.pass);
  log(`——— ${results.length - failed.length}/${results.length} pass ———`);
  if (lastShot) { try { fs.unlinkSync(lastShot); } catch { /* best effort */ } }
  process.exit(failed.length === 0 ? 0 : 1);
}

main().catch((e) => { console.error('[canvas-embed] fatal:', e); process.exit(1); });
