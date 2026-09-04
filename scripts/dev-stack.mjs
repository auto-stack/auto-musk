#!/usr/bin/env node
// scripts/dev-stack.mjs — musk 开发栈一键编排（PLAN-059 S1：验收环境硬化）。
//
// 解决两件事：
//   ①AutoUI MCP 默认抢 9247 与后端冲突（FATAL: failed to bind → snapshot/
//     驱动取证瞎眼）——VM 前端显式 AUTOUI_MCP_PORT=9277 换口;
//   ②后端/VM/Vue 三进程手工拉起易漏 env（AUTO_BACKEND/AUTO_VM_MERGE/
//     RUST_MIN_STACK/AUTO_HTTP_PROXY）,端口占用状态各异。
//
// 用法：node scripts/dev-stack.mjs [--web] [--fresh-backend] [--fresh-vm]
//   后端  musk serve --addr 127.0.0.1:9247（CWD=tmp/musk-demo;9247 已监听则跳过）
//   VM    auto run --render=vm（AUTO_BACKEND=http://127.0.0.1:9247,
//         AUTO_VM_MERGE=0, RUST_MIN_STACK=16777216, AUTOUI_MCP_PORT=9277）
//   --web 额外起 Vue dev :3335（AUTO_HTTP_PROXY=http://127.0.0.1:9247）
//   --fresh-backend 先 taskkill 既有 musk.exe 再起（默认跳过已监听端口）
// 进程全部 detached + 日志 tee 到 tmp/dev-stack-{serve,vm,web}.log,
// 本脚本退出不牵连任何子进程（Start-Process 语义,沿 2026-09-03 会话教训）。
import { spawn } from "node:child_process";
import net from "node:net";
import { openSync, mkdirSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const argv = process.argv.slice(2);
const WITH_WEB = argv.includes("--web");
const FRESH_BACKEND = argv.includes("--fresh-backend");

const PORTS = { serve: 9247, mcp: 9277, web: 3335 };
const LOG_DIR = path.join(ROOT, "tmp");
mkdirSync(LOG_DIR, { recursive: true });

function portInUse(port) {
  return new Promise((resolve) => {
    const s = net.connect(port, "127.0.0.1");
    s.setTimeout(800);
    s.on("connect", () => { s.destroy(); resolve(true); });
    s.on("error", () => resolve(false));
    s.on("timeout", () => { s.destroy(); resolve(false); });
  });
}

function kill(name) {
  return new Promise((resolve) => {
    const p = spawn("taskkill", ["/IM", name, "/F"], { shell: false });
    p.on("exit", () => resolve());
    p.on("error", () => resolve());
  });
}

function detach(cmd, args, cwd, logPath, env) {
  const out = openSync(logPath, "a");
  const child = spawn(cmd, args, {
    cwd,
    env: { ...process.env, ...env },
    detached: true,
    stdio: ["ignore", out, out],
    shell: false,
  });
  child.unref();
  return child.pid;
}

const stamp = new Date().toISOString();
const results = [];

// 1) 后端
if (FRESH_BACKEND && (await portInUse(PORTS.serve))) {
  await kill("musk.exe");
  await new Promise((r) => setTimeout(r, 800));
}
if (await portInUse(PORTS.serve)) {
  results.push(`[serve] :${PORTS.serve} 已监听——跳过（--fresh-backend 可强制重启）`);
} else {
  const muskExe = path.join(ROOT, "backend", "target", "debug", "musk.exe");
  const pid = detach(
    muskExe, ["serve", "--addr", `127.0.0.1:${PORTS.serve}`],
    path.join(ROOT, "tmp", "musk-demo"),
    path.join(LOG_DIR, "dev-stack-serve.log"),
  );
  results.push(`[serve] musk serve :${PORTS.serve} pid=${pid}（workdir tmp/musk-demo）`);
}

// 2) VM 前端（编译轨;MCP 换口 9277）。9277 已监听 = 已有带 MCP 的 VM 实例,跳过
// （防止重跑堆窗口;--fresh-vm 先 taskkill auto.exe 再起）。
{
  if (argv.includes("--fresh-vm")) {
    await kill("auto.exe");
    await new Promise((r) => setTimeout(r, 800));
  }
  if (await portInUse(PORTS.mcp)) {
    results.push(`[vm] MCP :${PORTS.mcp} 已监听——VM 实例已在跑,跳过`);
  } else {
    // KD 059-FU1 观察项 a:静默退出——release 规避实测无效(2026-09-04 仍退),
    // 回归 debug 构建走取证路线(AUTO_EXE 可覆盖;退出码/末尾输出由调用方捕获)。
    const autoExe = process.env.AUTO_EXE || "auto";
    const log = path.join(LOG_DIR, "dev-stack-vm.log");
    const pid = detach(autoExe, ["run", "--render=vm"], ROOT, log, {
      AUTO_BACKEND: `http://127.0.0.1:${PORTS.serve}`,
      AUTO_VM_MERGE: "0",
      RUST_MIN_STACK: "16777216",
      AUTOUI_MCP_PORT: String(PORTS.mcp),
    });
    results.push(`[vm] auto run --render=vm pid=${pid}（MCP=:${PORTS.mcp}）`);
  }
}

// 3) 可选 Vue dev
if (WITH_WEB) {
  if (await portInUse(PORTS.web)) {
    results.push(`[web] :${PORTS.web} 已监听——跳过`);
  } else {
    const log = path.join(LOG_DIR, "dev-stack-web.log");
    const pid = detach(
      "cmd.exe", ["/c", "npx vite", "--port", String(PORTS.web), "--strictPort"],
      path.join(ROOT, "gen", "front", "vue"), log,
      { AUTO_HTTP_PROXY: `http://127.0.0.1:${PORTS.serve}` },
    );
    results.push(`[web] vite :${PORTS.web} pid=${pid}（proxy→serve）`);
  }
}

console.log(`[dev-stack] ${stamp}`);
for (const r of results) console.log("  " + r);
console.log(`[dev-stack] 日志: tmp/dev-stack-*.log;MCP 取证口=:${PORTS.mcp}`);
