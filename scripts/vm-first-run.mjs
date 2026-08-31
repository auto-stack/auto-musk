#!/usr/bin/env node
// scripts/vm-first-run.mjs — PLAN-047 D1: musk VM 首跑启动器（沿 vm-link-probe 先例）。
//
// 调用串（PLAN-047 T1-a 勘察定型）：
//   cd <musk检出根> && node scripts/vm-first-run.mjs [--keep] [--observe-ms N]
// 等价于在 musk 检出根执行 `auto run --render=vm`：
//   auto bin 默认 features 含 ui-iced；run_vm_ui 以 CWD 为 project_dir，
//   entry = src/front/app.at，运行期自切 CWD 至 src/front（rust_ui.rs:2419）。
// 栈注（T1-d）：UI 动态路径 run_file_dynamic_ui 强制 OS 主线程且不开大栈线程；
//   RUST_MIN_STACK 仅影响派生线程。若主线程栈溢出，按 D1 阶梯②建
//   tools/musk-vm-host 微 crate（/STACK 抬升主线程栈），本脚本不改。
//
// 桥接注（PLAN-053 P-053-4）：
//   默认使用 VM+VM 拆分模式（AUTO_VM_MERGE=0），#[api] 调用经 HTTP 桥发送至 AUTO_BACKEND（默认 http://127.0.0.1:9247）。
//   若显式指定 AUTO_VM_MERGE=1（merged 模式），#[api] 调用桩体会发出一次性 no-op 告警。
//
// 观察模式：默认观察 VM_FIRST_RUN_OBSERVE_MS(默认 20000ms) 后 taskkill 收尾；
//   --keep 不杀进程（留给用户实机目验），此时退出码只反映启动期 fatal。
// 日志：全量行 tee 至 tmp/plan047-firstrun.log；结尾出
//   `[first-run] summary observe_ms=N alive=… reds=K stack=a codegen=b …`
//   各分类下的签名行以 `[red:<class>]` 前缀复现（去重计数）。
// 退出码：0=存活至观察期结束且无 fatal；3=检出 fatal 红；4=进程提前自行退出。

import { spawn } from "node:child_process";
import { mkdirSync, appendFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const LOG_PATH = path.join(ROOT, "tmp", "plan047-firstrun.log");

const argv = process.argv.slice(2);
const KEEP = argv.includes("--keep");
const observeIdx = argv.indexOf("--observe-ms");
const OBSERVE_MS =
  (observeIdx >= 0 ? Number(argv[observeIdx + 1]) : 0) ||
  Number(process.env.VM_FIRST_RUN_OBSERVE_MS || 20000);

const CLASSES = [
  ["stack", /stack overflow|STACK_OVERFLOW/i],
  ["panic", /panicked at|RUST_BACKTRACE|thread '.*' has overflowed/i],
  ["codegen", /\[CODEGEN\].*dropping|\[HANDLER-CODEGEN\]\s*failed/i],
  ["link", /Undefined symbol|[Uu]nresolved symbol|link failed|linker error/i],
  ["io", /Failed to read|No such file|not found\s*$|cannot find/i],
];

mkdirSync(path.dirname(LOG_PATH), { recursive: true });
appendFileSync(LOG_PATH, `\n===== first-run ${new Date().toISOString()} =====\n`);

const child = spawn("auto", ["run", "--render=vm"], {
  cwd: ROOT,
  env: { ...process.env, RUST_MIN_STACK: process.env.RUST_MIN_STACK || "16777216" },
  shell: false,
});

let exitedEarly = false;
let earlyCode = null;
const hits = new Map(); // class -> Map(sigLine -> count)
let totalReds = 0;

function noteLine(line) {
  appendFileSync(LOG_PATH, line + "\n");
  process.stdout.write(line + "\n");
  for (const [cls, re] of CLASSES) {
    if (re.test(line)) {
      const sig = line.trim().slice(0, 220);
      if (!hits.has(cls)) hits.set(cls, new Map());
      const m = hits.get(cls);
      m.set(sig, (m.get(sig) || 0) + 1);
      totalReds += 1;
      break;
    }
  }
}

let stdoutBuf = "";
child.stdout.on("data", (d) => {
  stdoutBuf += d.toString();
  let i;
  while ((i = stdoutBuf.indexOf("\n")) >= 0) {
    noteLine("stdout | " + stdoutBuf.slice(0, i));
    stdoutBuf = stdoutBuf.slice(i + 1);
  }
});
let stderrBuf = "";
child.stderr.on("data", (d) => {
  stderrBuf += d.toString();
  let i;
  while ((i = stderrBuf.indexOf("\n")) >= 0) {
    noteLine("stderr | " + stderrBuf.slice(0, i));
    stderrBuf = stderrBuf.slice(i + 1);
  }
});

child.on("error", (e) => noteLine(`spawn-error | ${e.message}`));
let killing = false;
child.on("close", (code, signal) => {
  // taskkill 收尾触发的 close 不算提前退出(signal 在 win32 taskkill 下为 null,
  // 故用 killing 哨兵而非判 signal——首跑实测误报 exit=1 即此坑)。
  if (!KEEP && !killing && !signal) {
    exitedEarly = true;
    earlyCode = code;
  }
});

const t0 = Date.now();
await new Promise((resolve) => setTimeout(resolve, OBSERVE_MS));
const observedMs = Date.now() - t0;

let aliveAtEnd = true;
if (exitedEarly) aliveAtEnd = false;

if (!KEEP) {
  killing = true;
  try {
    if (process.platform === "win32") {
      spawn("taskkill", ["/pid", String(child.pid), "/T", "/F"], { stdio: "ignore" });
    } else {
      child.kill("SIGKILL");
    }
  } catch {}
} else {
  process.stdout.write(`--keep: 进程未收尾, pid=${child.pid}\n`);
}

const parts = [`observe_ms=${observedMs}`, `alive=${aliveAtEnd ? "yes" : "no(exit=" + earlyCode + ")"}`];
for (const [cls] of CLASSES) {
  const m = hits.get(cls);
  const n = m ? [...m.values()].reduce((a, b) => a + b, 0) : 0;
  parts.push(`${cls}=${n}`);
}
for (const [cls, m] of hits) {
  for (const [sig, n] of m) {
    process.stdout.write(`[red:${cls}] (x${n}) ${sig}\n`);
    appendFileSync(LOG_PATH, `[red:${cls}] (x${n}) ${sig}\n`);
  }
}
const summary = `[first-run] summary ${parts.join(" ")} reds=${totalReds}`;
process.stdout.write(summary + "\n");
appendFileSync(LOG_PATH, summary + "\n");

let exit = 0;
if (totalReds > 0) exit = 3;
else if (exitedEarly && earlyCode !== 0) exit = 4;
process.exit(exit);
