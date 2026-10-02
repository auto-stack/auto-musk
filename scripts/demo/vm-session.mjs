// scripts/demo/vm-session.mjs — PLAN-097 T-06: 隔离 VM 会话模块。
//
// 职责（计划 §5.3）：私有配置/端口隔离启动 musk serve + 原生 VM 宿主、
// 实际端点发现、源/二进制指纹、owned 关停（只杀自有进程树）。
// 配方 = T-01 冻结（docs/reports/097-vm-demo-baseline.md §冻结命令）：
//   serve: MUSK_SERVE_PORT + MUSK_CONFIG_DIR 隔离 + AAID_URL + AUTO_EXE + RUST_MIN_STACK
//   VM:    auto run --render vm + AUTO_REUSE_BACKEND=1 + AUTOUI_MCP_PORT +
//          AUTO_VM_WINDOW + AUTO_VM_STORAGE_FILE（一次性文件）
// 硬规则（本轮实证沉淀）：
//   - autoExe/muskExe 必须显式绝对路径且存在——VM 探针/演示须锁定当日
//     auto-lang 构建，不得沿用 PATH 上可能滞后的构建（T-05 附页②）。
//   - 端口起前必须空闲；被占 = 显式失败（不换端口硬跑、不杀占位者）。
//   - 关停只 taskkill 自有 spawn 的 Windows PID 树（Git Bash $! 是 MSYS
//     pid 的坑不复现——本模块全 Node spawn）+ 端口回收验收。
//   - VM 进程偶发无错提前退出（基线发现③）：暴露 exited 事件/状态供
//     调用方重试或报错，不无限等待。
// 纯决策函数单独导出供 V02 测试（端口冲突/归属判言/旧产物拒绝）。
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { execFileSync, spawn } from 'node:child_process';

export const sha256File = p => crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');

// ── 纯决策函数（V02 直测） ────────────────────────────────────────────

// netstat -ano LISTENING 行解析 → 占用指定端口的 PID 集合
export function parseListeningPids(netstatOutput, port) {
  const pids = new Set();
  for (const line of String(netstatOutput).split(/\r?\n/)) {
    if (!line.includes(`:${port} `)) continue;
    const cols = line.trim().split(/\s+/);
    if (cols[3] === 'LISTENING') pids.add(Number(cols[4]));
  }
  return [...pids];
}

// 端口冲突决策：起前发现占用者 → 拒绝启动。返回 {ok, pids, reason}。
// 占位者即使是自有遗留也只报错不杀（owned 关停唯一入口是 stop()）。
export function decidePortConflict(occupiedPids) {
  if (occupiedPids.length === 0) return { ok: true, pids: [], reason: 'free' };
  return { ok: false, pids: occupiedPids, reason: `port occupied by pids [${occupiedPids.join(',')}] — 显式失败（不换端口硬跑、不杀占位者）` };
}

// 归属判言：端口监听者必须是本会话 spawn 的 PID（serve → back 端口，
// VM → MCP 端口）。外部进程持有 = 环境污染，报错不清场。
export function ownershipVerdict(listeningPids, expectedPid) {
  if (listeningPids.length === 1 && listeningPids[0] === expectedPid) {
    return { owned: true, detail: `listening pid ${listeningPids[0]} == spawned ${expectedPid}` };
  }
  return { owned: false, detail: `listening pids [${listeningPids.join(',')}] != spawned ${expectedPid}` };
}

// 旧产物拒绝：指纹记录 vs 当前文件不匹配 = 拒绝复用旧运行目录的产物身份。
// mtime 允许更新（重构建）；sha 一致才算同一构建。
export function fingerprintVerdict(recorded, current) {
  if (!recorded || typeof recorded !== 'object') return { ok: false, reason: 'no fingerprint record' };
  for (const key of Object.keys(recorded)) {
    if (!current[key]) return { ok: false, reason: `fingerprint field missing in current: ${key}` };
    if (recorded[key].sha256 !== current[key].sha256) {
      return { ok: false, reason: `stale artifact: ${key} sha256 ${recorded[key].sha256.slice(0, 12)}… != current ${current[key].sha256.slice(0, 12)}…` };
    }
  }
  return { ok: true, reason: 'fingerprints match' };
}

// "AutoUI MCP: listening on http://127.0.0.1:PORT" 端点行解析（+ /mcp）；
// 端口回退行（busy — fell back）单独标记（隔离跑不允许回退，调用方判定）。
export function parseMcpEndpoint(bootOutput) {
  const m = String(bootOutput).match(/AutoUI MCP: listening on (https?:\/\/[^\s]+)/);
  if (!m) return { endpoint: null, fellBack: false };
  return { endpoint: m[1].replace(/\/+$/, '') + '/mcp', fellBack: /port \d+ busy .+ fell back/.test(String(bootOutput)) };
}

// ── 进程原语（全 Node spawn；Windows PID 树） ──────────────────────────

export function pidAlive(pid) {
  try {
    const out = execFileSync('tasklist', ['/FI', `PID eq ${pid}`], { encoding: 'utf8', windowsHide: true });
    return out.includes(String(pid));
  } catch { return false; }
}

export function listeningPids(port) {
  const out = execFileSync('netstat', ['-ano'], { encoding: 'utf8', windowsHide: true });
  return parseListeningPids(out, port);
}

// 只杀显式传入的自有 PID 树；两轮重试（首跑实证 taskkill 偶发静默失败）。
export function killTree(pid) {
  for (let i = 0; i < 2; i++) {
    try { execFileSync('taskkill', ['/pid', String(pid), '/T', '/F'], { windowsHide: true, stdio: 'ignore' }); break; } catch { /* retry */ }
  }
}

export async function waitFor(predicate, timeoutMs, label) {
  const deadline = Date.now() + timeoutMs;
  let lastErr = '';
  while (Date.now() < deadline) {
    try { const v = await predicate(); if (v) return v; } catch (e) { lastErr = e.message; }
    await new Promise(r => setTimeout(r, 300));
  }
  throw new Error(`timeout waiting ${label}${lastErr ? ` (last: ${lastErr})` : ''}`);
}

// ── 会话对象 ──────────────────────────────────────────────────────────

const mcpText = body => { try { return JSON.parse(body)?.result?.content?.find(x => x.type === 'text')?.text ?? ''; } catch { return ''; } };

export class Mcp {
  constructor(endpoint) { this.endpoint = endpoint; this.nextId = 100; }
  async call(name, args) {
    const r = await fetch(this.endpoint, { method: 'POST', headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ jsonrpc: '2.0', id: this.nextId++, method: 'tools/call', params: { name, arguments: args ?? {} } }) });
    const text = await r.text();
    if (text.includes('"isError":true')) throw new Error(`MCP ${name} isError: ${mcpText(text).slice(0, 200)}`);
    return mcpText(text);
  }
  snapshot() { return this.call('autoui_snapshot', { include_bounds: true, include_state: true, include_status: true }); }
  async stableSnapshot({ settleMs = 500, timeoutMs = 20000 } = {}) {
    const deadline = Date.now() + timeoutMs;
    let prev = await this.snapshot();
    while (Date.now() < deadline) {
      await new Promise(r => setTimeout(r, settleMs));
      const cur = await this.snapshot();
      if (cur === prev) return cur;
      prev = cur;
    }
    return prev;
  }

  // 容错轮询:等待谓词命中的快照(UI 未渲染/MCP isError 视为未就绪继续等)。
  // 超时抛最后错误(调用方决定整跑重试——基线发现③)。
  async waitForSnapshot(predicate, timeoutMs, label = 'snapshot') {
    const deadline = Date.now() + timeoutMs;
    let lastErr = 'no attempt';
    let snap = '';
    while (Date.now() < deadline) {
      try {
        snap = await this.snapshot();
        if (predicate(snap)) return snap;
      } catch (e) { lastErr = e.message; }
      await new Promise(r => setTimeout(r, 400));
    }
    throw new Error(`vm-session: timeout waiting ${label} (last: ${lastErr}; snapshot: ${snap.slice(0, 120)})`);
  }
}

export class VmSession {
  // opts: { runRoot, ports: {back,mcp,front}, muskExe, autoExe, workspaceDir,
  //        aaidUrl?, vmWindow?, credentials? }
  // autoExe/muskExe 必须显式绝对路径（锁当日构建规则，见头注）。
  constructor(opts) {
    if (!path.isAbsolute(opts.muskExe ?? '') || !fs.existsSync(opts.muskExe)) {
      throw new Error(`vm-session: muskExe must be an existing absolute path, got ${opts.muskExe}`);
    }
    if (!path.isAbsolute(opts.autoExe ?? '') || !fs.existsSync(opts.autoExe)) {
      throw new Error(`vm-session: autoExe must be an existing absolute path (锁当日 auto-lang 构建，不用 PATH), got ${opts.autoExe}`);
    }
    this.opts = opts;
    this.ports = opts.ports;
    this.procs = [];
    this.mcp = null;
    this.mcpEndpoint = null;
    this.stopped = false;
  }

  fingerprint() {
    const autoVer = execFileSync(this.opts.autoExe, ['--version'], { encoding: 'utf8', windowsHide: true }).trim();
    return {
      musk_exe: { path: this.opts.muskExe, sha256: sha256File(this.opts.muskExe), bytes: fs.statSync(this.opts.muskExe).size },
      auto_exe: { path: this.opts.autoExe, sha256: sha256File(this.opts.autoExe), version: autoVer },
      node: process.version,
    };
  }

  async start() {
    const o = this.opts;
    fs.mkdirSync(path.join(o.runRoot, 'home'), { recursive: true });
    fs.mkdirSync(o.workspaceDir, { recursive: true });
    // 起前置检：三端口必须空闲
    for (const [name, port] of Object.entries(this.ports)) {
      const v = decidePortConflict(listeningPids(port));
      if (!v.ok) throw new Error(`vm-session: ${name} port ${port} ${v.reason}`);
    }
    // serve（自有进程；隔离配置 + AAID 指向 + AUTO_EXE 传导给 canvas spawn）
    const serve = spawn(o.muskExe, ['serve', '--workdir', o.workspaceDir], {
      cwd: o.workspaceDir,
      env: { ...process.env, MUSK_SERVE_PORT: String(this.ports.back), MUSK_CONFIG_DIR: path.join(o.runRoot, 'home'),
             AAID_URL: o.aaidUrl ?? 'http://127.0.0.1:9', AUTO_EXE: o.autoExe, RUST_MIN_STACK: '16777216' },
      windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'],
    });
    this.procs.push({ role: 'serve', p: serve, out: '' });
    serve.stdout.on('data', b => { this.procs[0].out += b; });
    serve.stderr.on('data', b => { this.procs[0].out += b; });
    const base = `http://127.0.0.1:${this.ports.back}`;
    await waitFor(async () => (await fetch(`${base}/api/health`)).status < 500, 30000, 'serve /api/health');
    const backVerdict = ownershipVerdict(listeningPids(this.ports.back), serve.pid);
    if (!backVerdict.owned) throw new Error(`vm-session: back port not owned by spawned serve: ${backVerdict.detail}`);

    // 宿主 VM（复用门：back 端口已有活监听即直连）
    const vmStorage = path.join(o.runRoot, 'host-vm-storage.json');
    fs.rmSync(vmStorage, { force: true });
    const vm = spawn(o.autoExe, ['run', '--render', 'vm', '--port', String(this.ports.front), '--back-port', String(this.ports.back)], {
      cwd: o.projectDir,
      env: { ...process.env, AUTO_REUSE_BACKEND: '1', AUTO_HTTP_PORT: String(this.ports.back), AUTO_HTTP_BASE: base,
             AUTOUI_MCP_PORT: String(this.ports.mcp), AUTO_VM_WINDOW: o.vmWindow ?? '1280x800',
             AUTO_VM_STORAGE_FILE: vmStorage, RUST_MIN_STACK: '16777216' },
      windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'],
    });
    this.procs.push({ role: 'vm', p: vm, out: '' });
    vm.stdout.on('data', b => { this.procs[1].out += b; });
    vm.stderr.on('data', b => { this.procs[1].out += b; });

    this.mcpEndpoint = await waitFor(() => {
      if (vm.exitCode !== null) throw new Error(`VM exited early code=${vm.exitCode}（基线发现③;调用方应整跑重试）`);
      return parseMcpEndpoint(this.procs[1].out).endpoint;
    }, o.bootTimeoutMs ?? 300000, 'host VM MCP endpoint');
    const { fellBack } = parseMcpEndpoint(this.procs[1].out);
    if (fellBack) throw new Error('vm-session: MCP 端口回退发生（隔离跑不允许；端口规划失效）');
    const mcpVerdict = ownershipVerdict(listeningPids(this.ports.mcp), vm.pid);
    if (!mcpVerdict.owned) throw new Error(`vm-session: mcp port not owned by spawned VM: ${mcpVerdict.detail}`);
    this.mcp = new Mcp(this.mcpEndpoint);
    this.base = base;
    return { base, mcpEndpoint: this.mcpEndpoint, fingerprints: this.fingerprint(), servePid: serve.pid, vmPid: vm.pid };
  }

  // 播种：注册/登录 → 取默认工作区 → 建会话（AAID 死端口时 assistant
  // 失败臂消息确定性落盘；正式演示传真实 aaidUrl 后由调用方自行发需求）
  async seed() {
    const base = this.base;
    // 字段名 = username(/api/auth/register 422 实证);token 尽力获取——
    // 当前 build 无 auth 中间件,后续 build 需携带,seed 统一返回 token。
    const cred = this.opts.credentials ?? { username: 'demo', password: 'demo-097' };
    const reg = await (await fetch(`${base}/api/auth/register`, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(cred) })).json();
    const token = reg?.token ?? (await (await fetch(`${base}/api/auth/login`, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(cred) })).json())?.token;
    const list = await (await fetch(`${base}/api/workspace/list`, { headers: { authorization: `Bearer ${token}` } })).json();
    const ws = (list?.workspaces ?? [])[0];
    const created = await (await fetch(`${base}/api/chats/session?workspace=${encodeURIComponent(ws.id)}`, { method: 'POST', headers: { 'content-type': 'application/json', authorization: `Bearer ${token}` }, body: JSON.stringify({ project_path: ws.path ?? '' }) })).json();
    this.token = token;
    this.workspace = ws;
    this.sessionId = created?.session?.id;
    return { token, workspace: ws, sessionId: this.sessionId };
  }

  async stop() {
    if (this.stopped) return { allGone: true, portsFreed: true };
    this.stopped = true;
    const ownPids = this.procs.map(x => x.p.pid).filter(Boolean);
    for (const pid of ownPids) killTree(pid);
    await new Promise(r => setTimeout(r, 3000));
    const allGone = ownPids.every(pid => !pidAlive(pid));
    const portsFreed = Object.values(this.ports).every(port => listeningPids(port).length === 0);
    return { ownPids, allGone, portsFreed };
  }
}
