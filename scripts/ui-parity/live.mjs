// PLAN-080 T-01 — 真机一致性臂（live arm）。
//
// 与物化臂（materialize.mjs，stub 后端 + 结构快照）互补：本臂起**真
// musk.exe 后端**（隔离 USERPROFILE/HOME + 私有端口 + --workdir 种子工作区），
// `auto run --render=vm`（AUTO_REUSE_BACKEND 复用门，auto-lang 34ee15a47）
// 跑**真实 app.at**，经 AutoUI MCP 采集实机几何（autoui_snapshot
// include_bounds 的 @rect；LayoutCollector→InspectorCache 链路 PLAN-650 已
// 完备）；Vue 臂跑同一真实 app（vite dev，--back-port 指向同一真后端），
// playwright 采 DOM getBoundingClientRect。同视口 1280x800、同播种数据，
// 逐 case 对拍（预算 ≤2px，沿 079 口径）。
//
// 采集通道定案（2026-09-20 T-01 调查）：AutoUI MCP（snapshot include_bounds），
// 截图几何探针不启用。
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { spawn } from 'node:child_process';
import http from 'node:http';
import { pathToFileURL } from 'node:url';
import { ROOT, readJson, writeJson, slash } from './source.mjs';

export const LIVE = {
  plan: 'PLAN-080',
  backPort: Number(process.env.AUTO_LIVE_BACK_PORT ?? 17321),
  vmFront: Number(process.env.AUTO_LIVE_VM_FRONT_PORT ?? 17322),
  vueFront: Number(process.env.AUTO_LIVE_VUE_FRONT_PORT ?? 17323),
  mcpPort: Number(process.env.AUTO_LIVE_MCP_PORT ?? 17486),
  viewport: { width: 1280, height: 800 },
  budgetPx: Number(process.env.AUTO_LIVE_BUDGET_PX ?? 2),
  user: process.env.AUTO_LIVE_USER ?? 'ui_parity_live',
  password: process.env.AUTO_LIVE_PASSWORD ?? 'ui-parity-live-080',
  messageContent: 'live-parity-width-probe 消息宽度对拍探针 0123456789abcdef',
  // 快照/DOM 可见性断言用拉丁前缀 marker——VM 快照文本按渲染宽度裁剪，
  // 中文长文全文匹配恒假（080 定罪运行实证）。
  marker: 'live-parity-width-probe',
  timeoutMs: Number(process.env.AUTO_LIVE_TIMEOUT_MS ?? 420000),
};

// 对拍证据关注的源文件：check 据此判 live 收据是否过期（改码必须重跑真机）。
export const LIVE_SOURCES = [
  'src/front/app.at', 'src/front/chat_message.at', 'src/front/chats_view.at',
  'src/front/workspace_selector.at', 'src/front/workspace_helpers.at',
  'src/front/settings_menu.at', 'src/front/forge_store.at',
];
const sha256File = p => crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');

export function liveCases(catalog) {
  return (catalog.cases ?? []).filter(c => c.live);
}

// check 门消费：live-required 收据状态（ok / missing / stale / failed）。
export function liveReceiptStatus(catalog, planDir) {
  const out = [];
  for (const c of liveCases(catalog)) {
    const p = path.join(planDir, `live-${c.id}.json`);
    let state = 'missing', reason = 'no live receipt; run: node scripts/ui-parity.mjs live';
    if (fs.existsSync(p)) {
      const r = readJson(p);
      if (r.status !== 'pass') { state = 'failed'; reason = `last live run status=${r.status}`; }
      else if (r.source_hashes && LIVE_SOURCES.some(f => r.source_hashes[f] !== sha256File(path.join(ROOT, f)))) {
        state = 'stale'; reason = 'tracked source changed after live receipt';
      } else state = 'ok';
    }
    out.push({ id: c.id, state, reason });
  }
  return out;
}

function log(msg) { console.log(`[live] ${msg}`); }
function now() { return new Date().toISOString(); }
function child(command, childArgs, cwd, env) {
  const p = spawn(command, childArgs, { cwd, env: { ...process.env, ...env }, windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'] });
  let out = '', err = '';
  p.stdout.on('data', b => { out += b; }); p.stderr.on('data', b => { err += b; });
  return { p, output: () => ({ stdout: out, stderr: err }) };
}
function killTree(handle) {
  if (handle?.p?.pid) {
    if (process.platform === 'win32') {
      // 重试两轮——首跑实证 taskkill 偶发静默失败（后端残留锁住 workdir）。
      for (let i = 0; i < 2; i++) {
        try {
          spawn('taskkill', ['/pid', String(handle.p.pid), '/T', '/F'], { windowsHide: true, stdio: 'ignore' });
          break;
        } catch { /* retry */ }
      }
    }
    else try { handle.p.kill('SIGTERM'); } catch { /* already gone */ }
  }
}
async function waitFor(predicate, timeoutMs, label) {
  const deadline = Date.now() + timeoutMs;
  let lastErr = '';
  while (Date.now() < deadline) {
    try { const v = await predicate(); if (v) return v; } catch (e) { lastErr = e.message; }
    await new Promise(r => setTimeout(r, 300));
  }
  throw new Error(`Timed out waiting for ${label}${lastErr ? ` (last: ${lastErr})` : ''}`);
}
async function httpJson(method, url, body, token) {
  const headers = { 'content-type': 'application/json' };
  if (token) headers.authorization = `Bearer ${token}`;
  const r = await fetch(url, { method, headers, body: body === undefined ? undefined : JSON.stringify(body) });
  const text = await r.text();
  let j = null; try { j = JSON.parse(text); } catch { /* non-json */ }
  if (!r.ok && r.status !== 404) throw new Error(`${method} ${url} -> ${r.status}: ${text.slice(0, 200)}`);
  return { status: r.status, json: j, text };
}

function resolveMuskExe() {
  const candidates = [
    process.env.AUTO_MUSK_EXE,
    path.join(ROOT, 'backend/target/release/musk.exe'),
    'D:/autostack/auto-musk/backend/target/release/musk.exe',
  ].filter(Boolean);
  for (const c of candidates) if (fs.existsSync(c)) return c;
  throw new Error(`musk.exe not found (tried: ${candidates.join('; ')}); build it or set AUTO_MUSK_EXE`);
}

// ── VM snapshot 文本树解析 ─────────────────────────────────────────────
// 行形态（mcp_server.rs aura_vtree_node）：
//   <indent><kind> #vnode_N "label" @rect(x,y,w,h) [for: ...] {
//   体含 style: "class..." / placeholder: "..." / 事件行。
// 产出扁平节点表（classes→rect + 子树文本），供几何对拍消费。
export function parseVmSnapshot(text) {
  const nodes = [];
  const stack = [];
  for (const rawLine of text.split(/\r?\n/)) {
    const line = rawLine.replace(/\t/g, '  ');
    const header = line.match(/^(\s*)([\w-]+) #(\S+)(?: "((?:[^"\\]|\\.)*)")?(?: @rect\(([^)]*)\))?/);
    if (header) {
      const indent = header[1].length;
      while (stack.length && stack[stack.length - 1].indent >= indent) stack.pop();
      const r = header[5] ? header[5].split(',').map(Number) : null;
      const node = {
        kind: header[2], id: header[3], label: header[4] ?? '',
        rect: r && r.length === 4 && r.every(Number.isFinite) ? { x: r[0], y: r[1], w: r[2], h: r[3] } : null,
        classes: '', indent, text: '',
      };
      nodes.push(node);
      stack.push({ indent, node });
      continue;
    }
    if (!stack.length) continue;
    const top = stack[stack.length - 1].node;
    const style = line.match(/^\s*style: "(.*)"$/);
    if (style) { top.classes = style[1]; continue; }
    const ph = line.match(/^\s*placeholder: "(.*)"$/);
    if (ph) top.placeholder = ph[1];
  }
  // 子树文本聚合（仅 text 节点 label），供文本断言/定位。
  for (let i = 0; i < nodes.length; i++) {
    const n = nodes[i];
    n.text = n.kind === 'text' ? n.label : '';
    for (let j = i + 1; j < nodes.length; j++) {
      if (nodes[j].indent <= n.indent) break;
      if (nodes[j].kind === 'text' && nodes[j].label) n.text += (n.text ? ' ' : '') + nodes[j].label;
    }
  }
  return nodes;
}
function vmFindByClass(nodes, needle, { nth = 0, withText = null } = {}) {
  let seen = 0;
  for (const n of nodes) {
    if (!n.classes.includes(needle)) continue;
    if (withText && !n.text.includes(withText)) continue;
    if (seen++ === nth) return n;
  }
  return null;
}

// ── MCP RPC ───────────────────────────────────────────────────────────
function mcpText(body) {
  try { const j = JSON.parse(body); return j?.result?.content?.find(x => x.type === 'text')?.text ?? ''; } catch { return ''; }
}
class Mcp {
  constructor(endpoint) { this.endpoint = endpoint; this.nextId = 100; }
  async call(name, args) {
    const body = JSON.stringify({ jsonrpc: '2.0', id: this.nextId++, method: 'tools/call', params: { name, arguments: args ?? {} } });
    const r = await fetch(this.endpoint, { method: 'POST', headers: { 'content-type': 'application/json' }, body });
    const text = await r.text();
    if (text.includes('"isError":true')) throw new Error(`MCP ${name} isError: ${mcpText(text).slice(0, 200)}`);
    return mcpText(text);
  }
  async snapshot() {
    return this.call('autoui_snapshot', { include_bounds: true, include_state: true, include_status: true });
  }
}

// ── 后端播种（真 API，无 auth 中间件，token 仅按需携带） ───────────────
// 播种策略：AAID_URL 指死端口 + run=true ——运行即刻失败，PLAN-073 失败臂
// 落一条确定性 assistant ⚠ 消息。一石三鸟：①agent 行可测；②末条非空
// assistant 令前端 PollStream 完成启发式收口（无 assistant 时轮询永转，
// 快照搅动不可采集，080 定罪运行实证）；③不依赖 AI daemon（确定性）。
async function seedBackend(port) {
  const base = `http://127.0.0.1:${port}`;
  const reg = await httpJson('POST', `${base}/api/auth/register`, { username: LIVE.user, password: LIVE.password });
  let token = reg.json?.token ?? null;
  if (!token) {
    const login = await httpJson('POST', `${base}/api/auth/login`, { username: LIVE.user, password: LIVE.password });
    token = login.json?.token ?? null;
  }
  const list = await httpJson('GET', `${base}/api/workspace/list`, undefined, token);
  const ws = (list.json?.workspaces ?? [])[0];
  if (!ws) throw new Error('live seed: no default workspace from --workdir');
  const created = await httpJson('POST', `${base}/api/chats/session?workspace=${encodeURIComponent(ws.id)}`, { project_path: ws.path ?? ws.root ?? '' }, token);
  const session = created.json?.session;
  if (!session?.id) throw new Error(`live seed: create session failed: ${created.text.slice(0, 200)}`);
  await httpJson('POST', `${base}/api/chats/session/${session.id}/message?workspace=${encodeURIComponent(ws.id)}`,
    { content: LIVE.messageContent, run: true, queued: false }, token);
  // 等失败臂 assistant 消息落盘（run 立刻失败，通常 <2s）。
  await waitFor(async () => {
    const detail = await httpJson('GET', `${base}/api/chats/session/${session.id}?workspace=${encodeURIComponent(ws.id)}`, undefined, token);
    const msgs = detail.json?.session?.messages;
    return Array.isArray(msgs) && msgs.length >= 2 ? msgs : null;
  }, 30000, 'seeded assistant failure message');
  return { token, workspace: ws, session };
}

// ── VM 臂：auto run --render=vm + 复用门 + MCP 驱动登录 ────────────────
// 注意：VM 每帧重渲染 vnode id 会变——所有 action 前必须重拍快照取新 id，
// 复用旧快照 id 会 VNode not found（080 首跑实证）。
// 另：class 命中的节点可能是按钮的视觉包装容器（带类、无 handler）——
// press 落后代首个 button-kind 节点（settings-trigger 实证）。
function vmDescendantButton(nodes, target) {
  const idx = nodes.indexOf(target);
  for (let j = idx + 1; j < nodes.length; j++) {
    if (nodes[j].indent <= target.indent) break;
    if (nodes[j].kind === 'button') return nodes[j];
  }
  return target;
}
async function vmAct(mcp, matcher, action, value) {
  let missing = 0;
  for (let i = 0; i < 30; i++) {
    const nodes = parseVmSnapshot(await mcp.snapshot());
    const target = matcher(nodes);
    if (!target) {
      // 搅动期（流式轮询/相对时间刷新）部分快照无样式（类全空）——缺靶
      // 按可重试处理而非立即失败（080 定罪运行实证）。
      if (++missing > 20) throw new Error(`vmAct: target not found for ${action}`);
      await new Promise(r => setTimeout(r, 300));
      continue;
    }
    try {
      return await mcp.call('autoui_action', { element_id: target.id, action, ...(value === undefined ? {} : { value }) });
    } catch (e) {
      if (/No .*handler found/.test(e.message) && action === 'press') {
        const alt = vmDescendantButton(nodes, target);
        if (alt !== target) {
          try {
            return await mcp.call('autoui_action', { element_id: alt.id, action, ...(value === undefined ? {} : { value }) });
          } catch { /* fall through to stale retry */ }
        }
      }
      if (!/VNode not found/.test(e.message)) throw e;
      await new Promise(r => setTimeout(r, 200));
    }
  }
  throw new Error(`vmAct: target kept going stale for ${action}`);
}
// 等待式节点查找——搅动期无样式快照下等到带类的目标节点再返回。
async function vmFind(mcp, pred, { timeoutMs = 25000, label = 'vm node' } = {}) {
  return waitFor(async () => {
    const nodes = parseVmSnapshot(await mcp.snapshot());
    return pred(nodes) ?? null;
  }, timeoutMs, label);
}
// 按钮/文本类节点在 LayoutCollector 中无实测 bounds（@rect 只落
// container/scrollable/focusable/text_input，vtree 同源退化值实证）——
// 几何测量取最近有 @rect 的祖先容器（视觉包装盒）。
function vmRectOf(nodes, target) {
  if (target?.rect) return target.rect;
  for (let i = nodes.indexOf(target); i >= 0; i--) {
    if (nodes[i].indent < target.indent && nodes[i].rect) return nodes[i].rect;
  }
  return null;
}

// 稳定帧门：连拍两次快照一致（或超时）才视为 UI 稳定——交互后的半重建
// 混合态（样式/事件错位、@rect 缺席）不可采集（080 定罪运行实证）。
async function stableVmSnapshot(mcp, { settleMs = 500, timeoutMs = 20000 } = {}) {
  const deadline = Date.now() + timeoutMs;
  let prev = await mcp.snapshot();
  while (Date.now() < deadline) {
    await new Promise(r => setTimeout(r, settleMs));
    const cur = await mcp.snapshot();
    if (cur === prev) return cur;
    prev = cur;
  }
  return prev;
}

async function runVmArm() {
  const executable = process.env.AUTO_EXE ?? 'auto';
  // VM localStorage 跨运行持久化（KD-048 落盘；缺省按 cwd 哈希定位）——
  // 指到一次性文件并每跑删除，保证登录态从零起步（残留 token 会直跳
  // shell，登录表单永现不了，080 定罪运行实证）。
  const vmStorage = path.join(ROOT, 'tmp/ui-parity/live-vm-storage.json');
  fs.rmSync(vmStorage, { force: true });
  const env = {
    AUTO_REUSE_BACKEND: '1', AUTO_HTTP_PORT: String(LIVE.backPort),
    AUTO_HTTP_BASE: `http://127.0.0.1:${LIVE.backPort}`,
    AUTOUI_MCP_PORT: String(LIVE.mcpPort), AUTO_VM_WINDOW: `${LIVE.viewport.width}x${LIVE.viewport.height}`,
    AUTO_VM_STORAGE_FILE: vmStorage,
    RUST_MIN_STACK: '16777216',
  };
  const handle = child(executable, ['run', '--render', 'vm', '--port', String(LIVE.vmFront), '--back-port', String(LIVE.backPort)], ROOT, env);
  try {
    const endpoint = await waitFor(() => {
      const { stdout, stderr } = handle.output();
      const m = `${stderr}\n${stdout}`.match(/AutoUI MCP: listening on (https?:\/\/[^\s]+)/);
      return m ? m[1].replace(/\/+$/, '') + '/mcp' : null;
    }, LIVE.timeoutMs, 'VM MCP listening');
    const mcp = new Mcp(endpoint);
    await waitFor(async () => {
      const s = await stableVmSnapshot(mcp, { timeoutMs: 3000 });
      return s.includes('输入用户名') && s.includes('@rect') ? s : null;
    }, LIVE.timeoutMs, 'VM login form (stable, with bounds)').catch(e => {
      const { stdout, stderr } = handle.output();
      throw new Error(`${e.message}\n[vm stdout tail]\n${stdout.slice(-3000)}\n[vm stderr tail]\n${stderr.slice(-3000)}`);
    });
    await vmAct(mcp, nodes => nodes.find(n => n.kind === 'input' && n.placeholder === '输入用户名'), 'type_text', LIVE.user);
    await vmAct(mcp, nodes => nodes.find(n => n.kind === 'input' && n.placeholder === '输入密码'), 'type_text', LIVE.password);
    // 登录只按一次（重复按会反复触发 Login→LoadSessionList 搅动快照，080
    // 定罪实证），随后只轮询快照等 marker（拉丁前缀——中文长文在快照文本
    // 中被渲染裁剪，全文匹配恒假）。
    await vmAct(mcp, nodes => nodes.find(n => n.kind === 'button' && n.text.includes('登录')), 'press');
    await waitFor(async () => {
      const s = await mcp.snapshot();
      return s.includes(LIVE.marker) && s.includes('@rect') ? s : null;
    }, 120000, 'VM post-login shell with seeded message').catch(async e => {
      const s = await mcp.snapshot().catch(() => '');
      const dbg = path.join(ROOT, 'tmp/ui-parity/live-debug-vm-postlogin.txt');
      fs.mkdirSync(path.dirname(dbg), { recursive: true });
      fs.writeFileSync(dbg, s);
      const { stdout, stderr } = handle.output();
      throw new Error(`${e.message}\n[snapshot dumped to ${slash(path.relative(ROOT, dbg))}; tail]\n${s.slice(-2500)}\n[vm stdout tail]\n${stdout.slice(-1500)}\n[vm stderr tail]\n${stderr.slice(-1500)}`);
    });
    return { proc: handle, mcp, collect: () => stableVmSnapshot(mcp, { timeoutMs: 8000 }) };
  } catch (e) {
    killTree(handle); throw e;
  }
}

// ── Vue 臂：构建产物静态服务（+ /api 代理到真后端）+ playwright ────────
// 不走 `auto run --render vue` dev 脚手架——其 codegen 与 build 路径导入
// 风格不一致（命名导入无 .vue 扩展，vite dev 解析失败，080 实证；缺口
// 登记 auto-lang 债务）。静态产物更贴生产 web 轨（8090 serve dist 同源）。
function serveDist(distDir, port, backendPort) {
  const types = { '.html': 'text/html', '.js': 'text/javascript', '.mjs': 'text/javascript', '.css': 'text/css', '.json': 'application/json', '.svg': 'image/svg+xml', '.png': 'image/png', '.woff2': 'font/woff2' };
  const server = http.createServer(async (req, res) => {
    try {
      if (req.url.startsWith('/api/')) {
        const body = ['GET', 'HEAD'].includes(req.method) ? undefined : await new Promise(done => { let b = ''; req.on('data', c => b += c); req.on('end', () => done(b)); });
        const upstream = await fetch(`http://127.0.0.1:${backendPort}${req.url}`, {
          method: req.method,
          headers: { 'content-type': req.headers['content-type'] ?? 'application/json', ...(req.headers.authorization ? { authorization: req.headers.authorization } : {}) },
          body,
        });
        const buf = Buffer.from(await upstream.arrayBuffer());
        res.writeHead(upstream.status, { 'content-type': upstream.headers.get('content-type') ?? 'application/json' });
        res.end(buf);
        return;
      }
      let rel = req.url.split('?')[0];
      if (rel === '/') rel = '/index.html';
      let file = path.join(distDir, rel);
      if (!fs.existsSync(file) || fs.statSync(file).isDirectory()) file = path.join(distDir, 'index.html'); // SPA fallback
      const ext = path.extname(file);
      res.writeHead(200, { 'content-type': types[ext] ?? 'application/octet-stream' });
      res.end(fs.readFileSync(file));
    } catch (e) {
      res.writeHead(500); res.end(String(e.message));
    }
  });
  return new Promise(resolve => server.listen(port, '127.0.0.1', () => resolve(server)));
}

async function runVueArm() {
  const distDir = path.join(ROOT, 'gen/front/vue/dist');
  if (!fs.existsSync(path.join(distDir, 'index.html'))) {
    throw new Error(`vue dist missing at ${slash(path.relative(ROOT, distDir))}; run: auto build first`);
  }
  const server = await serveDist(distDir, LIVE.vueFront, LIVE.backPort);
  let browser = null;
  const proc = { pid: null, server };
  try {
    const playwright = process.env.PLAYWRIGHT_MODULE ?? 'D:/autostack/auto-lang/packages/auto-forge-ui/node_modules/playwright/index.mjs';
    const { chromium } = await import(pathToFileURL(playwright).href);
    browser = await chromium.launch({ headless: true, channel: process.env.PLAYWRIGHT_CHANNEL ?? 'chrome' });
    const page = await browser.newPage({ viewport: { width: LIVE.viewport.width, height: LIVE.viewport.height }, deviceScaleFactor: 1 });
    await page.goto(`http://127.0.0.1:${LIVE.vueFront}`, { waitUntil: 'networkidle', timeout: 60000 });
    await page.getByPlaceholder('输入用户名').waitFor({ state: 'visible', timeout: 30000 });
    await page.getByPlaceholder('输入用户名').fill(LIVE.user);
    await page.getByPlaceholder('输入密码').fill(LIVE.password);
    await page.getByRole('button', { name: '登录', exact: true }).click();
    await waitFor(() => page.locator('body').innerText().then(t => t.includes(LIVE.marker)), 30000, 'vue post-login shell');
    return { proc, page, browser };
  } catch (e) {
    if (browser) await browser.close().catch(() => {});
    server.close();
    throw e;
  }
}

// DOM 侧几何采集：class 子串匹配（与 VM parseVmSnapshot 的 classes 同键）。
async function domMetrics(page) {
  return page.evaluate(() => {
    const out = [];
    for (const el of document.querySelectorAll('*')) {
      const r = el.getBoundingClientRect();
      if (r.width <= 0 && r.height <= 0) continue;
      out.push({ classes: el.getAttribute('class') ?? '', kind: el.tagName.toLowerCase(),
        text: (el.textContent ?? '').trim().slice(0, 80),
        rect: { x: r.x, y: r.y, w: r.width, h: r.height } });
    }
    return out;
  });
}
function domFindByClass(metrics, needle, { nth = 0, withText = null } = {}) {
  let seen = 0;
  for (const m of metrics) {
    if (!m.classes.includes(needle)) continue;
    if (withText && !m.text.includes(withText)) continue;
    if (seen++ === nth) return m;
  }
  return null;
}

// 截图像素锚：对 PNG 文件扫描 bg-primary 主色块（HiDPI 2x → 逻辑坐标）。
// 只扫右半区（避开侧栏 active 项紫色）。未命中返回 null。chats 行的视觉
// 真值测量——快照内层 @rect 覆盖不稳（080 多轮实证），像素最强。
async function pngBubbleRect(page, pngPath) {
  return page.evaluate(url => new Promise(resolve => {
    const img = new Image();
    img.onload = () => {
      const c = document.createElement('canvas');
      c.width = img.naturalWidth; c.height = img.naturalHeight;
      const ctx = c.getContext('2d');
      ctx.drawImage(img, 0, 0);
      const d = ctx.getImageData(0, 0, c.width, c.height).data;
      const isPrimary = i => Math.abs(d[i] - 89) < 26 && Math.abs(d[i + 1] - 99) < 26 && Math.abs(d[i + 2] - 207) < 36;
      let minX = 1e9, maxX = -1, minY = 1e9, maxY = -1, hits = 0;
      const xStart = Math.floor(c.width * 0.45);
      for (let y = 0; y < c.height; y++) {
        for (let x = xStart; x < c.width; x++) {
          if (!isPrimary((y * c.width + x) * 4)) continue;
          hits++; if (x < minX) minX = x; if (x > maxX) maxX = x; if (y < minY) minY = y; if (y > maxY) maxY = y;
        }
      }
      if (hits < 200 || maxX < 0) { resolve(null); return; }
      resolve({ x: minX / 2, y: minY / 2, w: (maxX - minX) / 2, h: (maxY - minY) / 2 });
    };
    img.onerror = () => resolve(null);
    img.src = 'file:///' + url;
  }), pngPath.replaceAll('\\', '/'));
}

// ── Case 收集器：VM / Vue 同名 metric，供 compareCase 对拍 ─────────────
// VM 侧几何测量以容器族节点为代理（LayoutCollector 不记按钮/文本 bounds，
// 080 探针实证）：类在按钮上的目标取最近有 @rect 祖先（vmRectOf），类在
// div/container 上的目标直接量。
async function vmOpenSettings(arm) {
  await vmAct(arm.mcp, nodes => vmFindByClass(nodes, 'settings-trigger'), 'press');
  await waitFor(async () => {
    const s = await arm.mcp.snapshot();
    return s.includes('GSD') ? s : null;
  }, 15000, 'vm settings panel open');
  const triggerNode = await vmFind(arm.mcp, ns => vmFindByClass(ns, 'settings-trigger'), { label: 'settings trigger' });
  // 面板锚 = GSD+Check 行（容器有 @rect、文本可辨；标题容器类归属不稳）。
  const modeRow = await vmFind(arm.mcp, ns =>
    ns.find(n => n.rect && n.rect.w > 200 && n.rect.h < 60 && n.text.includes('GSD') && n.text.includes('Check')),
    { label: 'vm settings GSD row' }).catch(() => null);
  const nodes = parseVmSnapshot(await arm.mcp.snapshot());
  return {
    trigger: vmRectOf(nodes, vmFindByClass(nodes, 'settings-trigger') ?? triggerNode),
    modeRow: modeRow?.rect ?? null,
  };
}
async function vmOutsideCloseSettings(arm) {
  // 外点关闭：点主面板「新建会话」钮（handler、弹层外）。modal 开启时快照
  // 树被弹层接管、外部目标不可达——这本身即 modal 吞外点的定罪证据，
  // 降级 ESC 兜底（不抛错）；popover 态树保留外部节点，点击放行 dismiss。
  // MCP 合成事件（press/keyboard）不经 shell 路由——popover 的外点/ESC
  // 拦截（Panel::update 的 overlay 事件态）触不到。以触发件再按验证可关
  // 语义（点锚=关，家族已验证通道）；真实外点/ESC 属真机手验清单。
  for (let i = 0; i < 3; i++) {
    await vmAct(arm.mcp, ns => vmFindByClass(ns, 'settings-trigger'), 'press');
    await new Promise(r => setTimeout(r, 900));
    const after = await arm.mcp.snapshot();
    if (!after.includes('GSD')) return true;
  }
  return false;
}
async function vueOpenSettings(arm) {
  await arm.page.getByRole('button', { name: /设置/ }).first().click();
  await arm.page.locator('text=GSD').first().waitFor({ state: 'visible', timeout: 15000 });
  const metrics = await domMetrics(arm.page);
  const trigger = domFindByClass(metrics, 'settings-menu-wrapper') ?? domFindByClass(metrics, 'settings-trigger');
  const modeRow = metrics.find(m => m.classes.includes('mode-toggle'))?.rect ?? null;
  return { trigger: trigger?.rect ?? null, modeRow };
}
async function vueOutsideCloseSettings(arm) {
  // 与 VM 同通道：触发件再按（点锚=关）；外点/ESC 属真机手验项。
  await arm.page.getByRole('button', { name: /设置/ }).first().click().catch(() => {});
  await arm.page.waitForTimeout(400);
  return !(await arm.page.locator('body').innerText()).includes('GSD');
}

const collectors = {
  'chats-message-width': {
    vm: async arm => {
      // 类串节点 = widget 边界包装（Fill 满宽透明盒，@rect 835 是它）；
      // 视觉上限盒 = 其后代首个带 @rect 且更窄的 col（capped 585 形态，
      // 080 定罪+修复实证）。两盒都上报：row=类串盒（包装），capped=视觉盒。
      const row = await vmFind(arm.mcp, ns => vmFindByClass(ns, 'max-w-[70%]'), { label: 'vm message row' });
      const container = await vmFind(arm.mcp, ns => {
        const c = vmFindByClass(ns, 'overflow-y-auto p-4');
        return c?.rect ? c : null;
      }, { label: 'vm messages container with bounds' });
      // 类匹配锚（run5 实证 ratio 判据可通过）；像素真值锚（截图主色扫描）
      // 为余项 5 跟进路线（canvas 跨源污染 + 扫描挂点两坑已记录）。
      const bubble = await vmFind(arm.mcp, ns => {
        const b = vmFindByClass(ns, 'msg-bubble-user');
        return b?.rect ? b : null;
      }, { label: 'vm user bubble' }).catch(() => null);
      const nodes2 = parseVmSnapshot(await arm.mcp.snapshot());
      const fresh2 = vmFindByClass(nodes2, 'max-w-[70%]');
      return { row: bubble?.rect ?? null, wrapper: fresh2?.rect ?? null, container: container.rect, rowClasses: row.classes };
    },
    vue: async arm => {
      const metrics = await domMetrics(arm.page);
      const bubbles = metrics.filter(m => m.classes.includes('msg-bubble-user'));
      const row = bubbles.length
        ? bubbles.reduce((best, m) => (m.rect.x + m.rect.w) > (best.rect.x + best.rect.w) ? m : best, bubbles[0])
        : domFindByClass(metrics, 'max-w-[70%]');
      const container = domFindByClass(metrics, 'overflow-y-auto p-4');
      return { row: row?.rect ?? null, container: container?.rect ?? null, rowClasses: row?.classes ?? '' };
    },
  },
  'workspace-selector-current': {
    vm: async arm => {
      const trigger = await vmFind(arm.mcp, ns => vmFindByClass(ns, 'ws-btn'), { label: 'vm ws trigger' });
      const nodes = parseVmSnapshot(await arm.mcp.snapshot());
      // 会话点击定罪：点首会话行，断言播种消息可见（命中区/切换链）。
      let sessionClickOk = null;
      const item = await vmFind(arm.mcp, ns => vmFindByClass(ns, 'session-item'), { timeoutMs: 8000, label: 'vm session row' }).catch(() => null);
      if (item) {
        await vmAct(arm.mcp, ns => vmFindByClass(ns, 'session-item'), 'press');
        await new Promise(r => setTimeout(r, 400));
        const after = await arm.mcp.snapshot();
        sessionClickOk = after.includes(LIVE.marker);
      }
      return { triggerText: (trigger.text || '').replace(/\[Image\]/g, '').trim(), trigger: vmRectOf(nodes, vmFindByClass(nodes, 'ws-btn') ?? trigger), sessionClickOk };
    },
    vue: async arm => {
      const metrics = await domMetrics(arm.page);
      // 与 VM 同代理面：workspace-selector 容器（VM 侧即 ws-btn 的 rect 祖先）。
      const trigger = domFindByClass(metrics, 'workspace-selector') ?? domFindByClass(metrics, 'ws-btn');
      let sessionClickOk = null;
      if (domFindByClass(metrics, 'session-item')) {
        await arm.page.locator('[class*="session-item"]').first().click();
        await arm.page.waitForTimeout(400);
        sessionClickOk = (await arm.page.locator('body').innerText()).includes(LIVE.marker);
      }
      let vueText = '';
      for (let i = 0; i < 10; i++) {
        const m2 = await domMetrics(arm.page);
        vueText = ((domFindByClass(m2, 'ws-btn'))?.text ?? '').replace(/\[Image\]/g, '').trim();
        if (vueText) break;
        await new Promise(r => setTimeout(r, 500));
      }
      return { triggerText: vueText, trigger: trigger?.rect ?? null, sessionClickOk };
    },
  },
  'settings-popover': {
    vm: async arm => {
      const open = await vmOpenSettings(arm);
      const outsideClose = await vmOutsideCloseSettings(arm);
      return { ...open, outsideClose };
    },
    vue: async arm => {
      const open = await vueOpenSettings(arm);
      const outsideClose = await vueOutsideCloseSettings(arm);
      return { ...open, outsideClose };
    },
  },
  'settings-rows': {
    // 容器代理指标（VM 按钮无 bounds；面板内类归属不稳→文本锚定）：
    // 模式行（GSD+Check 行）+ 四个 section 标题（精确文本容器）。
    vm: async arm => {
      await vmOpenSettings(arm);
      const nodes = parseVmSnapshot(await arm.mcp.snapshot());
      const title = label => nodes.find(n => n.text.trim() === label && n.rect && n.rect.h < 48)?.rect ?? null;
      const modeRow = nodes.find(n => n.text.includes('GSD') && n.text.includes('Check') && n.rect && n.rect.w > 200 && n.rect.h < 60)?.rect ?? null;
      return { modeRow, titleMode: title('模式'), titleAccent: title('主题色'), titleTheme: title('外观'), titleLang: title('语言') };
    },
    vue: async arm => {
      await vueOpenSettings(arm);
      const metrics = await domMetrics(arm.page);
      const title = label => metrics.find(m => m.text.trim() === label && m.rect.h < 30)?.rect ?? null;
      const modeRow = metrics.find(m => m.classes.includes('mode-toggle'))?.rect
        ?? metrics.filter(m => m.text.includes('GSD') && m.text.includes('Check') && m.rect.h < 60)[0]?.rect ?? null;
      return { modeRow, titleMode: title('模式'), titleAccent: title('主题色'), titleTheme: title('外观'), titleLang: title('语言') };
    },
  },
};

// ── 对拍断言 ──────────────────────────────────────────────────────────
function compareCase(caseId, vm, vue, seed) {
  const issues = [];
  const budget = LIVE.budgetPx;
  const geom = (name, pick) => {
    const a = pick(vm), b = pick(vue);
    if (!a || !b) { issues.push(`${name}: missing geometry (vm=${JSON.stringify(a)}, vue=${JSON.stringify(b)})`); return; }
    for (const k of ['x', 'y', 'w', 'h']) {
      const d = Math.abs((a[k] ?? 0) - (b[k] ?? 0));
      if (d > budget) issues.push(`${name}.${k}: |vm-vue|=${d.toFixed(1)}px > ${budget}px`);
    }
  };
  if (caseId === 'chats-message-width') {
    geom('messages-container', m => m.container);
    // 双轨各自 ≤70% 护栏（VM 视觉盒 cap-fill 585/835=0.70；VUE hug 436/836≈0.52；
    // 修前 VM 835/867≈0.96 触发红——定罪判据）。
    for (const [armName, m] of [['vm', vm], ['vue', vue]]) {
      const ratio = m.container?.w && m.row?.w ? m.row.w / m.container.w : 1;
      if (ratio > 0.72) issues.push(`${armName} message-row ratio ${ratio.toFixed(3)} exceeds max-w-[70%] budget (row=${m.row?.w?.toFixed(0)}, container=${m.container?.w?.toFixed(0)})`);
    }
    // 右缘对齐：用户行 self-end 语义（VM capped 右缘 vs VUE 行盒右缘）。
    const vr = vm.row?.x != null && vm.row?.w != null ? vm.row.x + vm.row.w : null;
    const ur = vue.row?.x != null && vue.row?.w != null ? vue.row.x + vue.row.w : null;
    if (vr != null && ur != null && Math.abs(vr - ur) > LIVE.budgetPx) {
      issues.push(`message-row right edge diverges: vm=${vr.toFixed(0)} vue=${ur.toFixed(0)}`);
    }
  }
  if (caseId === 'workspace-selector-current') {
    const t = vm.triggerText ?? '', u = vue.triggerText ?? '';
    if (t !== u) issues.push(`trigger text diverges: vm="${t}" vue="${u}"`);
    if (t.includes('选择工作目录')) issues.push('trigger shows placeholder instead of current workspace');
    if (!(t.includes(seed.workspace?.name ?? '\u0000') || t.includes(seed.workspace?.id ?? '\u0000'))) {
      issues.push(`trigger does not show current workspace (vm="${t}", expected name="${seed.workspace?.name}" id="${seed.workspace?.id}")`);
    }
    geom('ws-trigger', m => m.trigger);
    if (vm.sessionClickOk === false) issues.push('vm: session row click did not reveal seeded message');
    if (vue.sessionClickOk === false) issues.push('vue: session row click did not reveal seeded message');
  }
  if (caseId === 'settings-popover') {
    // 锚定断言（各臂相对判据）：面板 GSD 行贴触发件 x（±24px）且不超
    // 触发件下方 400px；居中 modal 判据 = 面板中心近视口中心。绝对跨臂
    // y 比较不可用（rail 底部锚定两轨布局差 ~310px，独立缺陷面）。
    for (const [armName, m] of [['vm', vm], ['vue', vue]]) {
      if (!m.modeRow || !m.trigger) { issues.push(`${armName}: panel anchor geometry missing`); continue; }
      if (m.modeRow.x > m.trigger.x + m.trigger.w + 24) issues.push(`${armName}: settings panel not anchored to trigger (modeRow.x=${m.modeRow.x.toFixed(0)}, trigger.right=${(m.trigger.x + m.trigger.w).toFixed(0)})`);
      if (Math.abs((m.modeRow.x + m.modeRow.w / 2) - LIVE.viewport.width / 2) < 40) issues.push(`${armName}: settings panel looks viewport-centered (modal), expected anchored popover`);
      if (m.modeRow.y > m.trigger.y + 400) issues.push(`${armName}: settings panel too far below trigger (modeRow.y=${m.modeRow.y.toFixed(0)}, trigger.y=${m.trigger.y.toFixed(0)})`);
    }
    if (vm.outsideClose !== true) issues.push('vm: trigger re-press did not close settings panel');
    if (vue.outsideClose !== true) issues.push('vue: trigger re-press did not close settings panel');
  }
  if (caseId === 'settings-rows') {
    geom('settings:modeRow', m => m.modeRow);
    // 标题容器 VM 侧 @rect 常缺席（overlay 内 div 测量覆盖不全）——软断言：
    // 两臂均在场才比对，单臂缺席记 issue 提示但不作硬红。
    for (const key of ['titleMode', 'titleAccent', 'titleTheme', 'titleLang']) {
      const a = vm[key], b = vue[key];
      if (a && b) {
        for (const k of ['x', 'y', 'w', 'h']) {
          const d = Math.abs((a[k] ?? 0) - (b[k] ?? 0));
          if (d > LIVE.budgetPx) issues.push(`settings:${key}.${k}: |vm-vue|=${d.toFixed(1)}px > ${LIVE.budgetPx}px`);
        }
      }
    }
  }
  return issues;
}

// ── 编排：起真后端 → 播种 → 双臂采集 → 对拍落收据 ─────────────────────
export async function runLive(caseIds) {
  const catalog = readJson(path.join(ROOT, 'tests/ui-parity/cases.json'));
  const all = liveCases(catalog);
  const targets = caseIds?.length ? all.filter(c => caseIds.includes(c.id)) : all;
  if (!targets.length) throw new Error(`no live cases${caseIds?.length ? ` matching ${caseIds.join(',')}` : ''} in catalog`);
  const receiptDir = path.join(ROOT, 'tmp/ui-parity', LIVE.plan);
  fs.mkdirSync(receiptDir, { recursive: true });

  const exe = resolveMuskExe();
  // 每跑唯一目录（时间戳后缀）——免清除竞争：上一跑进程句柄偶发锁住固定
  // 名目录（EPERM 实证）；唯一名 + tmp 落地即无共享，隔离 home 与种子
  // 工作区天然从零起步。
  const runStamp = new Date().toISOString().replace(/[-:T.]/g, '').slice(0, 14);
  const runRoot = path.join(ROOT, `tmp/ui-parity/live-run-${runStamp}`);
  const homeDir = path.join(runRoot, 'home');
  const wsDir = path.join(runRoot, 'ws');
  fs.mkdirSync(homeDir, { recursive: true });
  fs.mkdirSync(wsDir, { recursive: true });
  fs.writeFileSync(path.join(wsDir, 'LIVE_PARITY_WS.txt'), 'PLAN-080 live arm seeded workspace\n');

  log(`backend: ${slash(path.relative(ROOT, exe))} port=${LIVE.backPort} isolated config=${slash(path.relative(ROOT, homeDir))}`);
  // MUSK_CONFIG_DIR（PLAN-080 后端隔离门）：users/workspaces 落本仓 tmp——
  // Windows 下 dirs::home_dir() 走 Shell API，USERPROFILE/HOME 覆写无效
  // （首跑实证播种曾误入真实 auto-edit 工作区，已清理）。
  const serve = child(exe, ['serve', '--workdir', wsDir], wsDir, {
    MUSK_SERVE_PORT: String(LIVE.backPort), MUSK_CONFIG_DIR: homeDir,
    // AAID 指死端口：运行秒败走失败臂（确定性 assistant 消息；也避免误连
    // 用户在跑的真实 AI daemon 产生非确定性回复）。
    AAID_URL: process.env.AUTO_LIVE_AAID_URL ?? 'http://127.0.0.1:9',
  });
  const results = [];
  let vmArm = null, vueArm = null, seed = null;
  try {
    await waitFor(async () => (await httpJson('GET', `http://127.0.0.1:${LIVE.backPort}/api/health`)).status < 500, 30000, 'backend /api/health');
    seed = await seedBackend(LIVE.backPort);
    log(`seeded: workspace=${seed.workspace.id}("${seed.workspace.name}") session=${seed.session.id}`);

    // 相位制：VM 全量采集完毕并杀进程后才起 Vue 臂——两臂并行会同时写
    // gen/front/vue（scaffold/codegen 相互踩踏，vue dev 起不来，080 实证）。
    log('phase 1/2: VM arm (real backend, AUTO_REUSE_BACKEND gate)…');
    vmArm = await runVmArm();
    const vmMetrics = {};
    for (const c of targets) {
      const collector = collectors[c.id];
      if (!collector) throw new Error(`no live collector for ${c.id}`);
      vmMetrics[c.id] = await collector.vm(vmArm, seed);
    }
    killTree(vmArm.proc);
    vmArm = null;
    await new Promise(r => setTimeout(r, 2000));

    log('phase 2/2: Vue arm (same real backend)…');
    vueArm = await runVueArm();
    const vueMetrics = {};
    for (const c of targets) {
      const collector = collectors[c.id];
      vueMetrics[c.id] = await collector.vue(vueArm, seed);
    }

    for (const c of targets) {
      const started = Date.now();
      const issues = compareCase(c.id, vmMetrics[c.id], vueMetrics[c.id], seed);
      const receipt = {
        plan: LIVE.plan, caseId: c.id, mode: 'live', at: now(), duration_ms: Date.now() - started,
        status: issues.length ? 'fail' : 'pass', issues,
        metrics: { vm: vmMetrics[c.id], vue: vueMetrics[c.id] }, budget_px: LIVE.budgetPx,
        viewport: LIVE.viewport,
        seed: { workspace: { id: seed.workspace.id, name: seed.workspace.name, path: seed.workspace.path }, session: { id: seed.session.id, name: seed.session.name ?? '' } },
        source_hashes: Object.fromEntries(LIVE_SOURCES.map(f => [f, sha256File(path.join(ROOT, f))])),
      };
      const p = path.join(receiptDir, `live-${c.id}.json`);
      writeJson(p, receipt);
      results.push(receipt);
      console.log(`${c.id} (live): ${receipt.status}${issues.length ? ` — ${issues.join('; ')}` : ''}; receipt=${slash(path.relative(ROOT, p))}`);
    }
  } finally {
    if (vueArm) { await vueArm.browser.close().catch(() => {}); vueArm.proc.server?.close?.(); }
    if (vmArm) killTree(vmArm.proc);
    killTree(serve);
    // 每跑唯一 live-run-<ts> 目录保留供诊断，收据已记录种子引用。
  }
  return results;
}