#!/usr/bin/env node
// scripts/vm-mcp-census.mjs — PLAN-066 T-03: MCP 会话前后 auto 子进程普查
// （SD-02 常驻验证资产；KD-062 残留观察：autoui_snapshot 拉起 ~66MB 子 auto
// 进程不退 + ~43MB 瞬态子进程）。
//
// 用法（App 已在跑，端口为其 AutoUI MCP 端口）：
//   node scripts/vm-mcp-census.mjs --port 9741 [--calls 3] [--grace-ms 5000]
//
// 行为：census-before（auto.exe 全量：pid/ppid/WS MB/cmdline）→ 对 /mcp 连发
// `calls` 次 autoui_snapshot（无状态 JSON-RPC）→ 等 grace → census-after →
// 差集报告（新增 pid：滞留=会话后仍存活 / 已退=grace 内消失）。
// 退出码：0=无滞留新增；1=存在滞留新增（泄漏现形）；2=census/请求基础设施错误。

import { execFileSync } from "node:child_process";

const argv = process.argv.slice(2);
const argOf = (name, dflt) => {
  const i = argv.indexOf(name);
  return i >= 0 ? Number(argv[i + 1]) : dflt;
};
const PORT = argOf("--port", 9247);
const CALLS = Math.max(1, argOf("--calls", 3));
const GRACE_MS = argOf("--grace-ms", 5000);

const census = () => {
  const out = execFileSync(
    "powershell",
    ["-NoProfile", "-Command",
      "Get-CimInstance Win32_Process -Filter \"Name='auto.exe'\" | " +
      "Select-Object ProcessId,ParentProcessId,WorkingSetSize,CommandLine | ConvertTo-Json -Compress"],
    { encoding: "utf8", timeout: 15000 },
  ).trim();
  if (!out) return [];
  const j = JSON.parse(out);
  return (Array.isArray(j) ? j : [j]).map((p) => ({
    pid: p.ProcessId,
    ppid: p.ParentProcessId,
    mb: Math.round((p.WorkingSetSize || 0) / (1024 * 1024)),
    cmd: (p.CommandLine || "").slice(0, 120),
  }));
};

const callSnapshot = async (id) => {
  const res = await fetch(`http://127.0.0.1:${PORT}/mcp`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0", id,
      method: "tools/call",
      params: { name: "autoui_snapshot", arguments: { include_status: true } },
    }),
  });
  const body = await res.text();
  return { status: res.status, len: body.length, ok: body.includes('"result"') };
};

const fmt = (p) => `pid=${p.pid} ppid=${p.ppid} ${p.mb}MB ${p.cmd}`;
const key = (p) => `${p.pid}`;

const before = census();
console.log(`[census] before: ${before.length} auto.exe`);
before.forEach((p) => console.log(`  ${fmt(p)}`));

let httpErrors = 0;
for (let i = 1; i <= CALLS; i++) {
  try {
    const r = await callSnapshot(i);
    console.log(`[census] snapshot call ${i}/${CALLS}: http=${r.status} bytes=${r.len} ok=${r.ok}`);
    if (!r.ok) httpErrors++;
  } catch (e) {
    console.log(`[census] snapshot call ${i}/${CALLS}: FAILED ${e.message.split("\n")[0]}`);
    httpErrors++;
  }
  await new Promise((r) => setTimeout(r, 500));
}
if (httpErrors === CALLS) {
  console.error(`[census] all ${CALLS} MCP calls failed — is the app listening on ${PORT}?`);
  process.exit(2);
}

await new Promise((r) => setTimeout(r, GRACE_MS));

const after = census();
console.log(`[census] after (+${GRACE_MS}ms): ${after.length} auto.exe`);
after.forEach((p) => console.log(`  ${fmt(p)}`));

const beforeIds = new Set(before.map(key));
const added = after.filter((p) => !beforeIds.has(key(p)));
const persisted = added.filter((p) => after.some((q) => key(q) === key(p)));
console.log(`[census] added during session: ${added.length}`);
added.forEach((p) => console.log(`  ${fmt(p)}`));

// 滞留判定：会话期新增、census-after 时仍在（66MB/43MB 两形态均属此）
if (persisted.length > 0) {
  console.error(`[census] LEAK: ${persisted.length} child auto.exe persisted after MCP session`);
  process.exit(1);
}
console.log(`[census] clean: no persisted children after MCP session`);
process.exit(0);
