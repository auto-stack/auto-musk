#!/usr/bin/env node
// scripts/soak-active-session.mjs — PLAN-104 T-01/T-02 验证仪器（UAT-K9）。
//
// 隔离起栈（serve+VM，私有端口/配置），驱动一个纯文本长流式会话，分相位
// 采样 VM 进程私有内存并输出斜率：
//   stream 相位 = 内容增长期（两形态都在重建，对照面）；
//   idle   相位 = 收束后会话仍打开（指纹门生效面：RED 每拍全量重建/绿门跳过）。
// 用法：
//   node scripts/soak-active-session.mjs --vm-cwd <checkout> --label red \
//        [--duration-s 90] [--idle-s 60] [--out tmp/soak-red.json]
//   --vm-cwd 决定 VM 编译哪份 src/front（main=旧 .at=RED；worktree=新 .at=GREEN）。
// aaid 复用 :17654 在跑实例；musk.exe 取 <repo>/backend/target/debug（调用方先建）。
import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

const arg = (k, d) => {
  const i = process.argv.indexOf("--" + k);
  return i > 0 ? process.argv[i + 1] : d;
};
const VM_CWD = path.resolve(arg("vm-cwd", process.cwd()));
const LABEL = arg("label", "run");
const DURATION_S = Number(arg("duration-s", 90));
const IDLE_S = Number(arg("idle-s", 60));
const OUT = arg("out", `tmp/soak-${LABEL}.json`);
const REPO = path.resolve(arg("repo", path.resolve(path.dirname(new URL(import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, "$1")), "..")));
const MUSK_EXE = process.env.MUSK_EXE || path.join(REPO, "backend", "target", "debug", "musk.exe");
const AUTO_EXE = process.env.AUTO_EXE || "D:/autostack/auto-lang/target/release/auto.exe";
const PORT = 17301, MCP = 9287;
const BASE = `http://127.0.0.1:${PORT}`;
const log = (m) => console.log(`[soak:${LABEL}] ${m}`);

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
async function until(fn, ms, what) {
  const t0 = Date.now();
  while (Date.now() - t0 < ms) {
    try { const v = await fn(); if (v) return v; } catch { /* retry */ }
    await sleep(700);
  }
  throw new Error(`timeout: ${what}`);
}
const killTree = (pid) => { try { spawnSync("taskkill", ["/PID", String(pid), "/T", "/F"], { stdio: "ignore" }); } catch { } };
function privateMB(pid) {
  const r = spawnSync("powershell", ["-NoProfile", "-Command",
    `$p=Get-Process -Id ${pid} -ErrorAction SilentlyContinue; if($p){$p.PrivateMemorySize64/1MB}else{''}`],
    { encoding: "utf8", timeout: 8000 });
  const v = parseFloat((r.stdout || "").trim());
  return Number.isFinite(v) ? v : null;
}
function slopeMBPerMin(samples) {
  if (samples.length < 3) return null;
  const n = samples.length, sx = samples.reduce((a, s) => a + s.t, 0), sy = samples.reduce((a, s) => a + s.mb, 0);
  const sxy = samples.reduce((a, s) => a + s.t * s.mb, 0), sxx = samples.reduce((a, s) => a + s.t * s.t, 0);
  const denom = n * sxx - sx * sx;
  if (denom === 0) return null;
  return (n * sxy - sx * sy) / denom * 60; // t in minutes
}

const procs = [];
fs.mkdirSync(path.dirname(path.resolve(OUT)), { recursive: true });
const workdir = fs.mkdtempSync(path.join(path.dirname(path.resolve(OUT)), `soak-ws-${LABEL}-`));
const home = fs.mkdtempSync(path.join(path.dirname(OUT), `soak-home-${LABEL}-`));
let vmPid = null, vmDied = false;
try {
  // 1) serve（worktree 二进制；页面端点红绿同源，唯一变量=VM 编译的 .at）
  const serve = spawn(MUSK_EXE, ["serve", "--workdir", workdir], {
    cwd: workdir,
    env: { ...process.env, MUSK_SERVE_PORT: String(PORT), MUSK_CONFIG_DIR: home,
      MUSK_SKILLS_DIR: path.join(REPO, ".agents", "skills"), MUSK_PLAN_MAX_TOKENS: "65536",
      AAID_URL: "http://127.0.0.1:17654", AUTO_EXE: AUTO_EXE, RUST_MIN_STACK: "16777216" },
    stdio: ["ignore", "ignore", "ignore"], shell: false,
  });
  procs.push(serve.pid);
  await until(async () => (await fetch(`${BASE}/api/health`)).status < 500, 30000, "serve health");
  log(`serve up pid=${serve.pid} workdir=${workdir}`);

  // 2) VM（--vm-cwd 决定编译哪份 src/front）
  const vm = spawn(AUTO_EXE, ["run", "--render", "vm", "--back-port", String(PORT)], {
    cwd: VM_CWD,
    env: { ...process.env, AUTO_REUSE_BACKEND: "1", AUTOUI_MCP_PORT: String(MCP), RUST_MIN_STACK: "16777216" },
    stdio: ["ignore", "ignore", "ignore"], shell: false,
  });
  vmPid = vm.pid; procs.push(vmPid);
  vm.on("exit", (c) => { vmDied = true; log(`VM EXITED code=${c}`); });
  log(`vm up pid=${vmPid} cwd=${VM_CWD}`);
  await sleep(20000); // 窗口创建+shell 渲染（T-12 配方同款等待）

  // 3) 会话：approval=auto + thinking=max（K1/K4 对策），长流式纯文本需求
  const ws = (await (await fetch(`${BASE}/api/workspace/list`)).json()).workspaces[0];
  const sid = (await (await fetch(`${BASE}/api/chats/session?workspace=${encodeURIComponent(ws.id)}`, {
    method: "POST", headers: { "content-type": "application/json" },
    body: JSON.stringify({ mode: "superpowers" }) })).json()).session.id;
  await fetch(`${BASE}/api/chats/session/${sid}/approval?workspace=${encodeURIComponent(ws.id)}`, { method: "PATCH", headers: { "content-type": "application/json" }, body: JSON.stringify({ approval_mode: "auto" }) });
  await fetch(`${BASE}/api/chats/session/${sid}/thinking?workspace=${encodeURIComponent(ws.id)}`, { method: "PATCH", headers: { "content-type": "application/json" }, body: JSON.stringify({ thinking_level: "max" }) });
  log(`session ${sid} ready (workspace ${ws.id})`);

  const contentLen = async () => {
    const j = await (await fetch(`${BASE}/api/chats/session/${sid}?workspace=${encodeURIComponent(ws.id)}`)).json();
    const ms = j.session?.messages ?? [];
    const a = ms.filter((m) => m.role === "assistant");
    return a.reduce((s, m) => s + (m.content || "").length, 0);
  };
  await fetch(`${BASE}/api/chats/session/${sid}/message?workspace=${encodeURIComponent(ws.id)}`, {
    method: "POST", headers: { "content-type": "application/json" },
    body: JSON.stringify({ content: "请写一篇至少 2000 字的科幻短篇小说，主题：深海工作站 last 信号。直接开始正文，不要提纲。", run: true, queued: true }),
  });
  log("prompt sent; waiting for stream content…");
  await until(async () => (await contentLen()) > 200, 180000, "stream content >200 chars");

  // 4) 相位采样
  const samples = [];
  const t0 = Date.now();
  let phase = "stream", lastLen = -1, stableSince = 0;
  while (true) {
    const elapsed = (Date.now() - t0) / 1000;
    if (phase === "stream" && elapsed >= DURATION_S) { phase = "idle"; log(`→ idle phase (${IDLE_S}s)`); }
    if (phase === "idle" && elapsed >= DURATION_S + IDLE_S) break;
    const mb = vmPid ? privateMB(vmPid) : null;
    const len = await contentLen();
    if (phase === "stream") {
      if (len === lastLen) { if (!stableSince) stableSince = Date.now(); } else { stableSince = 0; }
      lastLen = len;
      if (stableSince && Date.now() - stableSince > 12000 && len > 500) { phase = "idle"; log(`→ idle phase early (stream done at ${len} chars)`); }
    }
    samples.push({ t: +(elapsed / 60).toFixed(3), mb, len, phase, vmDied });
    if (mb === null && vmPid) log(`WARN: vm pid ${vmPid} gone`);
    await sleep(2000);
  }
  const by = (p) => samples.filter((s) => s.phase === p && s.mb !== null);
  const result = {
    label: LABEL, vm_cwd: VM_CWD, vm_pid: vmPid, vm_died: vmDied, session: sid,
    stream_samples: by("stream").length, idle_samples: by("idle").length,
    slope_stream_mb_per_min: slopeMBPerMin(by("stream")),
    slope_idle_mb_per_min: slopeMBPerMin(by("idle")),
    final_len_chars: samples.at(-1)?.len ?? 0,
    samples,
  };
  log(`stream slope=${result.slope_stream_mb_per_min?.toFixed(3)} MB/min (${result.stream_samples} samples)`);
  log(`idle   slope=${result.slope_idle_mb_per_min?.toFixed(3)} MB/min (${result.idle_samples} samples)`);
  fs.mkdirSync(path.dirname(path.resolve(OUT)), { recursive: true });
  fs.writeFileSync(OUT, JSON.stringify(result, null, 1));
  log(`written ${OUT}`);
} finally {
  for (const pid of procs) killTree(pid);
  await sleep(1500);
}
