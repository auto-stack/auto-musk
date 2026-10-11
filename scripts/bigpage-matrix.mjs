#!/usr/bin/env node
// scripts/bigpage-matrix.mjs — PLAN-104 T-03 验证仪器（UAT-K10 大会话矩阵）。
//
// 合成 350 轮大会话（run:false 本地消息，无 LLM 成本）→ VM（worktree .at +
// 当日 auto.exe）MCP 快照打开会话 → 断言尾部窗口渲染（fresh 判定）→
// 长寿相位保持轮询 15min 后复验（long-lived 判定 + 进程存活）。
// 350 压力格 ⊃ 50/150 格（同载荷机制单调）；50 格由 UAT 全程日常面覆盖。
import { spawn, spawnSync } from "node:child_process";
import fs, { openSync, mkdirSync } from "node:fs";
import path from "node:path";

const arg = (k, d) => { const i = process.argv.indexOf("--" + k); return i > 0 ? process.argv[i + 1] : d; };
const REPO = path.resolve(arg("repo", path.join(path.dirname(new URL(import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, "$1")), "..")));
const VM_CWD = path.resolve(arg("vm-cwd", REPO));
const OUT = arg("out", "tmp/bigpage-matrix.json");
const TURNS = Number(arg("turns", 350));
const SOAK_MIN = Number(arg("soak-min", 15));
const MUSK_EXE = process.env.MUSK_EXE || path.join(REPO, "backend", "target", "debug", "musk.exe");
const AUTO_EXE = process.env.AUTO_EXE || "D:/autostack/auto-lang/target/release/auto.exe";
const PORT = 17302, MCP_PORT = 9288;
const BASE = `http://127.0.0.1:${PORT}`;
const log = (m) => console.log(`[bigpage] ${m}`);
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
async function until(fn, ms, what) { const t0 = Date.now(); while (Date.now() - t0 < ms) { try { const v = await fn(); if (v) return v; } catch { } await sleep(600); } throw new Error(`timeout: ${what}`); }
const killTree = (pid) => { try { spawnSync("taskkill", ["/PID", String(pid), "/T", "/F"], { stdio: "ignore" }); } catch { } };

// ── MCP（097 vm-session 同协议） ─────────────────────────────────────
const mcpText = (b) => { try { return JSON.parse(b)?.result?.content?.find((x) => x.type === "text")?.text ?? ""; } catch { return ""; } };
class Mcp {
  constructor(endpoint) { this.endpoint = endpoint; this.nextId = 100; }
  async call(name, args) {
    const r = await fetch(this.endpoint, { method: "POST", headers: { "content-type": "application/json" },
      body: JSON.stringify({ jsonrpc: "2.0", id: this.nextId++, method: "tools/call", params: { name, arguments: args ?? {} } }) });
    const text = await r.text();
    if (text.includes('"isError":true')) throw new Error(`MCP ${name} isError: ${mcpText(text).slice(0, 200)}`);
    return mcpText(text);
  }
  snapshot() { return this.call("autoui_snapshot", { include_state: true, include_status: true }); }
  // 765-R2 交接修①：本构建 autoui_action 的 click 要求 value:"x,y"（部件
  // 局部坐标）——缺省即 isError，曾被误读为 VM 退出（纪要后记⑦）。
  async action(element_id, action = "click", value = "12,12") { return this.call("autoui_action", { element_id, action, value }); }
}
function parseTree(text) {
  const nodes = []; const stack = [];
  for (const raw of String(text ?? "").split(/\r?\n/)) {
    const line = raw.replace(/\t/g, "  ");
    const h = line.match(/^(\s*)([\w-]+) #(\S+)(?: "((?:[^"\\]|\\.)*)")?/);
    if (!h) continue;
    const indent = h[1].length;
    while (stack.length && stack[stack.length - 1].indent >= indent) stack.pop();
    const node = { kind: h[2], id: h[3], text: h[4] ?? "", indent, children: [] };
    while (stack.length && stack[stack.length - 1].indent >= indent) stack.pop();
    if (stack.length) stack[stack.length - 1].node.children.push(node);
    nodes.push(node); stack.push({ indent, node });
  }
  return nodes;
}
const findByText = (nodes, sub) => nodes.find((n) => n.text.includes(sub));

const procs = [];
mkdirSync(path.dirname(path.resolve(OUT)), { recursive: true });
const workdir = fs.mkdtempSync(path.join(path.dirname(path.resolve(OUT)), `bigpage-ws-`));
const home = fs.mkdtempSync(path.join(path.dirname(path.resolve(OUT)), `bigpage-home-`));
const vmLog = openSync(`${OUT}.vm.log`, "a");
let vmPid = null, vmDied = false, cleaningUp = false;
try {
  const serve = spawn(MUSK_EXE, ["serve", "--workdir", workdir], {
    cwd: workdir, env: { ...process.env, MUSK_SERVE_PORT: String(PORT), MUSK_CONFIG_DIR: home,
      MUSK_SKILLS_DIR: path.join(REPO, ".agents", "skills"), MUSK_PLAN_MAX_TOKENS: "65536",
      AAID_URL: "http://127.0.0.1:17654", AUTO_EXE: AUTO_EXE, RUST_MIN_STACK: "16777216" },
    stdio: ["ignore", "ignore", "ignore"], shell: false });
  procs.push(serve.pid);
  await until(async () => (await fetch(`${BASE}/api/health`)).status < 500, 30000, "serve health");
  log(`serve up pid=${serve.pid}`);
  // 合成 350 轮大会话（run:false 纯本地落盘）
  const ws = (await (await fetch(`${BASE}/api/workspace/list`)).json()).workspaces[0];
  const sid = (await (await fetch(`${BASE}/api/chats/session?workspace=${encodeURIComponent(ws.id)}`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ mode: "superpowers" }) })).json()).session.id;
  await fetch(`${BASE}/api/chats/session/${sid}/rename?workspace=${encodeURIComponent(ws.id)}`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ name: "BIGPAGE" }) });
  const pad = "lorem-ipsum-deep-water.".repeat(220); // ~5KB/条 → 50 窗 ≈ 250KB（真实剖面 292KB 同量级）
  for (let i = 1; i <= TURNS; i++) {
    const r = await fetch(`${BASE}/api/chats/session/${sid}/message?workspace=${encodeURIComponent(ws.id)}`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ content: `BIGPAGE-msg ${i} deep-water-marker-${i} ${pad}`, run: false, queued: false }) });
    if (r.status !== 200) throw new Error(`post #${i} -> ${r.status}`);
    if (i % 100 === 0) log(`posted ${i}/${TURNS} @${new Date().toISOString()}`);
  }
  const plen = Number((await (await fetch(`${BASE}/api/chats/session/${sid}/page?limit=50&workspace=${encodeURIComponent(ws.id)}`)).text()).length);
  log(`session ${sid} synthesized: ${TURNS} turns, page payload ${plen} bytes`);

  // PLAN-104 T-03 归因修正：合成在 VM 启动前完成（爆发写不入 VM 生命期——
  // 前三轮死亡均发生于爆发+列表轮询叠加窗，765 无爆发 soak 1h 存活）。
  const vm = spawn(AUTO_EXE, ["run", "--render", "vm", "--back-port", String(PORT)], {
    cwd: VM_CWD, env: { ...process.env, AUTO_REUSE_BACKEND: "1", AUTOUI_MCP_PORT: String(MCP_PORT), RUST_MIN_STACK: "16777216" },
    stdio: ["ignore", vmLog, vmLog], shell: false });
  vmPid = vm.pid; procs.push(vmPid);
  // 765-R2 交接修②：收尾击杀与真实退出区分——cleaningUp 置位后的 exit
  // 是仪器自身 taskkill（恰码 1），不得再作为"VM EXITED"失败信号上报。
  vm.on("exit", (c) => { vmDied = true; if (!cleaningUp) log(`VM EXITED code=${c}`); });
  await sleep(20000);

  const mcp = new Mcp(`http://127.0.0.1:${MCP_PORT}/mcp`);
  // 端点发现：本构建 MCP 监听根路径（/mcp 返回 404 空文——本轮 run1 空转实证）
  if ((await mcp.snapshot()) === "") mcp.endpoint = `http://127.0.0.1:${MCP_PORT}`;
  let snap = await mcp.snapshot();
  log(`mcp up (${snap.length} bytes snapshot) @${new Date().toISOString()}`);

  // fresh 判定：最小快照策略（P765 残留面：高频 autoui_snapshot 与 VM 原生退出
  // 强相关——关键节点单拍，间隔 ≥8s，快照数压到个位数）
  let entry = null;
  for (let i = 0; i < 8 && !entry; i++) {
    await sleep(8000);
    if (vmDied) break;
    const nodes = parseTree(await mcp.snapshot());
    entry = findByText(nodes, "BIGPAGE");
  }
  if (vmDied) throw new Error("VM died before open");
  if (!entry) throw new Error("session entry not found in snapshot");
  // click 仅支持 autodown_editor 元素（本构建语义，765-R2 判别台同结论）
  // ——会话行不可点；本工作区单会话由前端"自动选首个"打开，点击容错续行。
  try { await mcp.action(entry.id, "click"); log("session clicked; waiting chat render…"); }
  catch (e) { log(`click unavailable (${String(e.message).slice(0, 90)}) — auto-selected session continues`); }
  let freshOk = false, freshDetail = "";
  for (let i = 0; i < 6 && !freshOk; i++) {
    await sleep(8000);
    if (vmDied) break;
    const t = await mcp.snapshot();
    freshOk = t.includes("deep-water-marker-" + TURNS) && t.includes("BIGPAGE-msg " + TURNS);
    if (freshOk) { const c = (t.match(/deep-water-marker-/g) || []).length; freshDetail = `${c} markers visible`; }
  }
  if (!freshOk && !vmDied) throw new Error(`tail marker #${TURNS} not rendered`);
  log(`FRESH PASS: ${freshDetail}`);

  // long-lived 判定：SOAK_MIN 分钟保持轮询，每 5min 复验渲染 + 进程存活
  const checks = [];
  for (let m = 5; m <= SOAK_MIN; m += 5) {
    await sleep(5 * 60000);
    const alive = !!privateMB(vmPid);
    const t = vmDied ? "" : await mcp.snapshot();
    const ok = alive && !vmDied && t.includes("deep-water-marker-" + TURNS);
    checks.push({ min: m, alive, rendered: ok });
    log(`long-lived +${m}min: alive=${alive} rendered=${ok}`);
    if (!ok) break;
  }
  const result = { turns: TURNS, page_payload_bytes: plen, vm_pid: vmPid, vm_died: vmDied,
    fresh: { ok: freshOk, detail: freshDetail }, long_lived: checks,
    verdict: freshOk && checks.length > 0 && checks.every((c) => c.ok) && !vmDied ? "PASS" : "FAIL" };
  fs.writeFileSync(OUT, JSON.stringify(result, null, 1));
  log(`verdict=${result.verdict} → ${OUT}`);
} finally {
  cleaningUp = true; // 先置标记再击杀：区分收尾 killTree 与真实退出
  for (const pid of procs) killTree(pid);
  await sleep(1500);
}
function privateMB(pid) {
  const r = spawnSync("powershell", ["-NoProfile", "-Command", `$p=Get-Process -Id ${pid} -ErrorAction SilentlyContinue; if($p){$p.PrivateMemorySize64/1MB}else{''}`], { encoding: "utf8", timeout: 8000 });
  const v = parseFloat((r.stdout || "").trim());
  return Number.isFinite(v) ? v : null;
}
