#!/usr/bin/env node
// scripts/vm-hangwatch.mjs — PLAN-066 T-01/T-02: VM UI 进程挂起期转储伴跑工具
// （P625-D1 指定手段 procdump -h；SD-02 常驻验证资产）。
//
// 用法（与 vm-first-run-soak.mjs 伴跑）：
//   node scripts/vm-hangwatch.mjs <总监控秒数>
//   PATH=<auto-lang release> AUTOUI_MCP_PORT=<私有端口> node scripts/vm-hangwatch.mjs 2700 &
//
// 行为：轮询命令行含 "run --render=vm" 的 auto.exe（UI 主进程），逐 PID 挂一次
// `procdump64 -accepteula -h <pid> <dmp>`——窗口停泵 >5s（AppHang 签名）即落
// 转储；无挂起则随进程终止安静退出（"Dump count not reached"）。
// env：PROCDUMP（默认 D:/autostack/tools/procdump64.exe，仓外工具，见 066
// 待澄清①）； hangwatch 日志与转储落 <root>/tmp/。

import { spawn, execFileSync } from "node:child_process";
import { appendFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const LOG = path.join(ROOT, "tmp", "plan066-hangwatch.log");
const PROCDUMP = process.env.PROCDUMP || "D:/autostack/tools/procdump64.exe";
const TOTAL_S = Number(process.argv[2] || 2700);

const log = (msg) => {
  const line = `[${new Date().toISOString()}] ${msg}`;
  console.log(line);
  appendFileSync(LOG, line + "\n");
};

const attached = new Set();
const t0 = Date.now();

const listVmAuto = () => {
  try {
    const out = execFileSync(
      "powershell",
      ["-NoProfile", "-Command",
        "Get-CimInstance Win32_Process -Filter \"Name='auto.exe'\" | " +
        "Where-Object { $_.CommandLine -match 'run.*--render=vm' } | " +
        "Select-Object ProcessId,CreationDate | ConvertTo-Json -Compress"],
      { encoding: "utf8", timeout: 15000 },
    ).trim();
    if (!out) return [];
    const j = JSON.parse(out);
    return Array.isArray(j) ? j : [j];
  } catch (e) {
    log(`poll error: ${e.message.split("\n")[0]}`);
    return [];
  }
};

log(`hangwatch start total_s=${TOTAL_S} procdump=${PROCDUMP}`);
while ((Date.now() - t0) / 1000 < TOTAL_S) {
  for (const p of listVmAuto()) {
    const pid = p.ProcessId;
    if (attached.has(pid)) continue;
    attached.add(pid);
    const dmp = path.join(ROOT, "tmp", `plan066-hang-${pid}.dmp`);
    log(`attach procdump -h pid=${pid} -> ${path.basename(dmp)}`);
    const pd = spawn(PROCDUMP, ["-accepteula", "-h", String(pid), dmp], {
      cwd: ROOT,
      stdio: ["ignore", "pipe", "pipe"],
    });
    let out = "";
    pd.stdout.on("data", (d) => (out += d));
    pd.stderr.on("data", (d) => (out += d));
    pd.on("exit", (code) => {
      const tail = out.split("\n").filter(Boolean).slice(-3).join(" | ");
      log(`procdump pid=${pid} exit=${code} ${tail ? `tail: ${tail}` : ""}`);
    });
  }
  await new Promise((r) => setTimeout(r, 2000));
}
log(`hangwatch end attached=${[...attached].join(",") || "none"}`);
