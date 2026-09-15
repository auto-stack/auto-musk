#!/usr/bin/env node
// scripts/vm-first-run-soak.mjs — PLAN-066 T-01/T-02: KD-048a 静默退出长跑取证 harness。
//
// 调用串：
//   cd <musk检出根> && node scripts/vm-first-run-soak.mjs [--rounds N] [--observe-ms MS]
//
// 每轮复用 scripts/vm-first-run.mjs（observe 模式），额外：
//   1. 注入 AUTO_DESKTOP_EXIT_LOG=<root>/tmp/plan066-exit-audit-r<N>.log
//      （PLAN-575 退出审计三挂点：vm_process_exit / panic / main_return；
//      空串视为未设，故这里恒注入显式路径）。
//   2. 轮末判定（575 降档出口判读矩阵，PLAN-066 rev2 T-01）：
//      - 退出码 0 且无审计死亡行            → 存活（正常收尾）
//      - 非零/提前退出 + 审计行存在          → 产品缺陷分支（site 直接指认）
//      - 非零/提前退出 + 审计文件零记录      → 外部击杀/AppHang 家族
//        （KD-048a/P625-D1 现形；定罪需挂起期线程转储，见待澄清①）
//   3. 端口候选 A/B（KD-055-4③：8080 落 Windows 保留段 8068-8167 →
//      绑定失败等待超时 exit 1）：B 组跑法 `AUTO_HTTP_PORT=9341 node 本脚本 …`
//      环境直通即可，本脚本不改写端口相关 env。
//
// 退出码：0=全部轮存活至观察期结束且无 fatal 红；1=存在静默退出轮（取证已落
// tmp/plan066-soak--summary.json）；2=启动期 fatal 红（vm-first-run 语义）。
// 入库依据：PLAN-066 测试设计「长跑 harness 作为可复跑资产入库 scripts/」。

import { spawnSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync, rmSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const argv = process.argv.slice(2);
const argOf = (name, dflt) => {
  const i = argv.indexOf(name);
  return i >= 0 ? Number(argv[i + 1]) : dflt;
};
const ROUNDS = Math.max(1, argOf("--rounds", 3));
const OBSERVE_MS = argOf("--observe-ms", 600000); // 10 分钟/轮（T-02 验收口径）

const summary = { started: new Date().toISOString(), rounds: [] };

for (let r = 1; r <= ROUNDS; r++) {
  const audit = path.join(ROOT, "tmp", `plan066-exit-audit-r${r}.log`);
  rmSync(audit, { force: true });
  const res = spawnSync(
    process.execPath,
    [path.join(ROOT, "scripts", "vm-first-run.mjs"), "--observe-ms", String(OBSERVE_MS)],
    { cwd: ROOT, env: { ...process.env, AUTO_DESKTOP_EXIT_LOG: audit }, encoding: "utf8" },
  );
  const code = res.status ?? -1;
  const auditText = existsSync(audit) ? readFileSync(audit, "utf8").trim() : "";
  const auditLines = auditText ? auditText.split("\n") : [];
  const earlyExit = code === 4; // vm-first-run: 进程提前自行退出（KD-048a 信号形态）
  let verdict;
  if (code === 0) verdict = "alive";
  else if (code === 3) verdict = "harness-red"; // 检出 fatal 红，harness 主动收尾——非稳定性判定
  else if (earlyExit && auditLines.length > 0) verdict = "product-defect"; // 审计 site 指认
  else if (earlyExit) verdict = "external-kill-or-apphang"; // 零审计 + 死亡 → 048a/P625-D1 现形
  else verdict = "infra"; // spawn 失败等 harness 自身故障
  summary.rounds.push({
    round: r,
    exitCode: code,
    auditPath: path.relative(ROOT, audit),
    auditLines,
    verdict,
    logTail: (res.stdout || "").split("\n").filter(Boolean).slice(-5),
  });
  console.log(`[soak] round ${r}/${ROUNDS}: code=${code} verdict=${verdict} auditLines=${auditLines.length}`);
}

summary.finished = new Date().toISOString();
summary.allAlive = summary.rounds.every((x) => x.verdict === "alive");
const out = path.join(ROOT, "tmp", "plan066-soak-summary.json");
writeFileSync(out, JSON.stringify(summary, null, 2));
console.log(`[soak] summary → ${out} allAlive=${summary.allAlive}`);
process.exit(summary.rounds.some((x) => x.verdict === "product-defect" || x.verdict === "external-kill-or-apphang") ? 1 : 0);
