#!/usr/bin/env node
// collect-telemetry.mjs — PLAN-098：plan-flow-uat 运行遥测摘要。
//
// 输入一份 run/会话的 turns.jsonl（含 PLAN-098 telemetry 字段）+ 可选 aaid
// 日志，产出 JSON + Markdown 摘要：按角色/相位聚合轮数、工具数、时长，每
// 请求 provider/model/token/耗时表，自动标记已知失败签名（K1 out=4096 截
// 断、K4 门同秒自动放行、K3 循环检测击杀、K5 plan 相位无 create_plan），
// aaid 日志仅作旁证（daemon 无关联键——按时间窗启发式对齐，标注置信）。
//
// 用法（零第三方依赖，Node ≥18 内置模块；与 plan-driver.mjs 同约束）：
//   node collect-telemetry.mjs --session <sid> --autoos <dir> --out <prefix>
//       [--aaid-log <path>] [--since <iso|epoch>] [--until <iso|epoch>]
//       [--expect <expected.json>]        # 断言摘要子集（V03 用），不符退出 3
//   node collect-telemetry.mjs --run <rid> --autoos <dir> ...（同上）
//
// 退出码：0=产出摘要；2=输入缺失/不可解析（stderr 列出缺失项）；
// 3=--expect 断言不符。解析行数与聚合数打印到 stdout 供人工复核。
import fs from 'node:fs';
import path from 'node:path';

// ── args ───────────────────────────────────────────────────────────────────
const argv = process.argv.slice(2);
const opt = {};
for (let i = 0; i < argv.length; i++) {
  const a = argv[i];
  if (a === '--session') opt.session = argv[++i];
  else if (a === '--run') opt.run = argv[++i];
  else if (a === '--autoos') opt.autoos = argv[++i];
  else if (a === '--aaid-log') opt.aaidLog = argv[++i];
  else if (a === '--since') opt.since = argv[++i];
  else if (a === '--until') opt.until = argv[++i];
  else if (a === '--out') opt.out = argv[++i];
  else if (a === '--expect') opt.expect = argv[++i];
  else { console.error(`collect-telemetry: unknown arg ${a}`); process.exit(2); }
}
if ((!opt.session && !opt.run) || !opt.autoos || !opt.out) {
  console.error('usage: collect-telemetry.mjs --session <sid>|--run <rid> --autoos <dir> --out <prefix> [--aaid-log p] [--since t] [--until t] [--expect f]');
  process.exit(2);
}

const missing = [];
const convDir = path.join(opt.autoos, 'conversations', opt.session ?? opt.run);
const turnsPath = path.join(convDir, 'turns.jsonl');
const metaPath = path.join(convDir, 'meta.json');

let turnsRaw = null;
try { turnsRaw = fs.readFileSync(turnsPath, 'utf8'); } catch { missing.push(`turns.jsonl not readable: ${turnsPath}`); }
let meta = {};
try { meta = JSON.parse(fs.readFileSync(metaPath, 'utf8')); } catch { /* meta optional */ }

if (missing.length) { console.error('collect-telemetry: missing input:'); for (const m of missing) console.error(`  - ${m}`); process.exit(2); }

// ── turns.jsonl 解析 ───────────────────────────────────────────────────────
const lines = turnsRaw.split(/\r?\n/).filter((l) => l.trim().length > 0);
const turns = [];
let badLines = 0;
for (const l of lines) {
  try { turns.push(JSON.parse(l)); } catch { badLines++; }
}
const kind = meta.kind ?? (opt.run ? 'flow' : 'chat');
const target = opt.session ?? opt.run;

const inWindow = (tsSec) => {
  const t = typeof tsSec === 'number' ? tsSec : Number(tsSec);
  if (!Number.isFinite(t)) return true;
  if (opt.since && t < toEpochSec(opt.since)) return false;
  if (opt.until && t > toEpochSec(opt.until)) return false;
  return true;
};
function toEpochSec(v) {
  if (/^\d+$/.test(String(v))) return Number(v);
  return Math.floor(Date.parse(v) / 1000);
}
const scoped = turns.filter((t) => inWindow(t.timestamp));
// 会话/run 不存在或无 assistant 产物 → 非零退出，不产出空报告冒充成功。
const assistantMsg = scoped.filter((t) => t.kind === 'message' && t.from !== 'human' && t.from !== 'system');
const toolTurns = scoped.filter((t) => t.kind === 'tool_call');
if (scoped.length === 0 || (assistantMsg.length === 0 && toolTurns.length === 0)) {
  console.error('collect-telemetry: 无 assistant 产物（无 message/tool 轮），拒绝产出空摘要：');
  console.error(`  - conversations/${path.basename(convDir)}: lines=${lines.length} parsed=${scoped.length}`);
  process.exit(2);
}

// ── telemetry 行规范化 ─────────────────────────────────────────────────────
// 单请求轮=扁平对象（含 ts_ms）；多请求轮=轮级合计 + requests[]（每行含
// ts_ms）。统一成逐请求行供表格与 aaid join。
function telemetryRows(tel) {
  if (!tel || typeof tel !== 'object') return [];
  if (Array.isArray(tel.requests) && tel.requests.length > 0) return tel.requests;
  if (tel.ts_ms != null || tel.model != null || tel.requested_model != null) return [tel];
  return [];
}

// ── 相位分割（flow）───────────────────────────────────────────────────────
// 系统轮 "Step 'X' started (role)" 开相位；StepCompleted 边界轮（content
// "Step 'X' completed: …"，遥测挂该轮）闭相位。
function phaseOf(content) {
  const m = /^Step '([^']+)' started \(([^)]+)\)/.exec(content ?? '');
  return m ? { step: m[1], role: m[2] } : null;
}
const phases = [];
let cur = null;
for (const t of scoped) {
  if (t.kind === 'system') {
    const p = phaseOf(t.content);
    if (p) {
      cur = { step_id: p.step, role: p.role, started_ts: t.timestamp, completed_ts: null, turns: 0, tool_calls: 0, has_create_plan: false, telemetry: null, telemetry_turn_seq: null };
      phases.push(cur);
      continue;
    }
    if (/^Step '.+' completed/.test(t.content ?? '') && cur) {
      cur.completed_ts = t.timestamp;
      if (t.telemetry) { cur.telemetry = t.telemetry; cur.telemetry_turn_seq = t.seq; }
      cur = null;
      continue;
    }
  }
  if (cur) {
    cur.turns++;
    if (t.kind === 'tool_call') {
      cur.tool_calls++;
      if (t.tool?.name === 'create_plan') cur.has_create_plan = true;
    }
  }
}

// ── LLM 表（逐请求行）──────────────────────────────────────────────────────
const llmRows = [];
for (const t of scoped) {
  const rows = telemetryRows(t.telemetry);
  if (rows.length === 0) continue;
  for (const r of rows) {
    llmRows.push({
      turn_seq: t.seq,
      from: t.from,
      carrier: t.kind === 'system' ? 'phase_boundary' : 'assistant_turn',
      step_id: r.correlation?.step_id ?? t.gate?.step_id ?? null,
      role: r.correlation?.role ?? t.from,
      session_id: r.correlation?.session_id ?? null,
      ts_ms: r.ts_ms ?? t.timestamp * 1000,
      elapsed_ms: r.elapsed_ms ?? null,
      requested_model: r.requested_model ?? null,
      provider: r.provider ?? null, // musk 侧恒 null（wire 无 provider）
      model: r.model ?? null,
      in_tokens: r.in_tokens ?? null,
      out_tokens: r.out_tokens ?? null,
      stop_reason: r.stop_reason ?? null,
      stream: r.stream ?? null,
      error: r.error ?? null,
      provider_source: null, confidence: null, // aaid join 回填
    });
  }
}
const turnsWithoutTelemetry = kind === 'chat'
  // chat：assistant message 轮未带 telemetry（旧数据/未走遥测路径）。
  ? assistantMsg.filter((t) => telemetryRows(t.telemetry).length === 0).length
  // flow（T-01 D4）：assistant 载体=相位边界轮——按无遥测相位计。
  : phases.filter((p) => !p.telemetry).length;

// ── 按 from 聚合 ───────────────────────────────────────────────────────────
const byFrom = {};
for (const t of scoped) {
  const f = byFrom[t.from] ??= { turns: 0, tool_calls: 0, first_ts: t.timestamp, last_ts: t.timestamp };
  f.turns++;
  if (t.kind === 'tool_call') f.tool_calls++;
  f.first_ts = Math.min(f.first_ts, t.timestamp);
  f.last_ts = Math.max(f.last_ts, t.timestamp);
  f.span_s = f.last_ts - f.first_ts;
}

// ── 失败签名 ───────────────────────────────────────────────────────────────
const signatures = { k1_truncation_4096: [], k3_loop_ge4_same_args: [], k4_gate_same_second: [], k5_plan_without_create_plan: [] };
// K1：收束（或任一请求行）out=4096。
for (const t of scoped) {
  const rows = telemetryRows(t.telemetry);
  if (rows.some((r) => r.out_tokens === 4096)) {
    signatures.k1_truncation_4096.push({ turn_seq: t.seq, from: t.from, step_id: t.telemetry?.correlation?.step_id ?? null, out_tokens: 4096, stop_reason: rows.find((r) => r.out_tokens === 4096)?.stop_reason ?? null });
  }
}
// K3：同参工具调用 ≥4（相邻同名同参；参数 JSON 串相等）。
{
  let run = [];
  const flush = () => { if (run.length >= 4) signatures.k3_loop_ge4_same_args.push({ tool: run[0].tool?.name, count: run.length, seqs: run.map((t) => t.seq) }); run = []; };
  let prevKey = null;
  for (const t of scoped) {
    if (t.kind !== 'tool_call') continue;
    const key = `${t.tool?.name}:${JSON.stringify(t.tool?.args ?? null)}`;
    if (key === prevKey) run.push(t); else { flush(); run = [t]; }
    prevKey = key;
  }
  flush();
}
// K4：gate waiting 与（同秒的 approve 轮 或 同秒的 Step started 系统轮）。
{
  const gateWaiting = scoped.filter((t) => t.kind === 'gate' && t.gate?.status === 'waiting');
  for (const g of gateWaiting) {
    const approveSameSec = scoped.some((t) => t !== g && t.timestamp === g.timestamp && (t.content ?? '').match(/^Gate .+ approve/i));
    const stepStartedSameSec = scoped.some((t) => t.kind === 'system' && t.timestamp === g.timestamp && /^Step '.+' started/.test(t.content ?? ''));
    if (approveSameSec || stepStartedSameSec) {
      signatures.k4_gate_same_second.push({ gate_step: g.gate?.step_id, ts: g.timestamp, via: approveSameSec ? 'approve_turn' : 'step_started_same_second' });
    }
  }
}
// K5：flow 的 plan 相位无 create_plan 调用。
if (kind === 'flow') {
  for (const p of phases) {
    if (p.step_id === 'plan' && !p.has_create_plan) {
      signatures.k5_plan_without_create_plan.push({ step_id: p.step_id, role: p.role, started_ts: p.started_ts });
    }
  }
}

// ── aaid 日志旁证（可选）───────────────────────────────────────────────────
const ANSI = /\x1b\[[0-9;]*m/g;
let aaid = { skipped: 'no --aaid-log provided; provider 归因不可用（musk 侧 telemetry.provider 恒 null）', in_zero_note: null };
if (opt.aaidLog) {
  let raw = '';
  try { raw = fs.readFileSync(opt.aaidLog, 'utf8'); } catch (e) {
    console.error(`collect-telemetry: aaid log unreadable: ${opt.aaidLog}（旁证跳过，不影响摘要产出）`);
  }
  if (raw) {
    const reqs = []; const starts = []; const dones = []; let bad = 0;
    // --since/--until 同样作用于日志行（epoch 秒或 ISO），缺省解析整份日志。
    const inLog = (tsMs) => {
      if (opt.since && tsMs < toEpochSec(opt.since) * 1000) return false;
      if (opt.until && tsMs > toEpochSec(opt.until) * 1000) return false;
      return true;
    };
    for (const line of raw.split(/\r?\n/)) {
      const s = line.replace(ANSI, '');
      const m = /^(\S+Z)\s+INFO\s+(\S+):\s+(.*)$/.exec(s);
      if (!m) continue;
      const ts = Date.parse(m[1]);
      if (!Number.isFinite(ts)) { bad++; continue; }
      if (!inLog(ts)) continue;
      const body = m[3];
      let mm;
      if ((mm = /^chat req: app=(\S+) model=(\S+) stream=(\S+) msgs=(\d+) candidates=(\d+)$/.exec(body))) {
        reqs.push({ ts, model: mm[2], stream: mm[3] === 'true' });
      } else if ((mm = /^chat stream start: app=(\S+) provider=(\S+) model=(\S+) \(waited (\d+)ms\)$/.exec(body))) {
        starts.push({ ts, provider: mm[2], model: mm[3] });
      } else if ((mm = /^chat stream done: app=(\S+) model=(\S+) elapsed=(\d+)ms in=(\d+) out=(\d+)$/.exec(body))) {
        dones.push({ ts, kind: 'stream_done', model: mm[2], elapsed: +mm[3], in: +mm[4], out: +mm[5], provider: null });
      } else if ((mm = /^chat ok: app=(\S+) provider=(\S+) model=(\S+) elapsed=(\d+)ms in=(\d+) out=(\d+)$/.exec(body))) {
        dones.push({ ts, kind: 'ok', model: mm[3], elapsed: +mm[4], in: +mm[5], out: +mm[6], provider: mm[2] });
      }
    }
    // 启发式 join：request 行（model+stream+±5s）→ 候选；唯一=high。
    for (const row of llmRows) {
      const cands = reqs.filter((r) => r.model === row.requested_model && r.stream === row.stream && Math.abs(r.ts - row.ts_ms) <= 5000);
      if (cands.length === 0) { row.confidence = 'none'; continue; }
      row.confidence = cands.length === 1 ? 'high' : 'low';
      row.provider_source = 'aaid-heuristic';
      const req = cands[0];
      if (row.stream) {
        // 流式 provider 在 stream start 行（done 行无 provider）——req 后
        // 最近的一条 start（同候选窗口）。
        const st = starts.filter((s) => s.ts >= req.ts && s.ts <= req.ts + (row.elapsed_ms ?? 0) + 60000)
          .sort((a, b) => a.ts - b.ts)[0];
        row.provider = st?.provider ?? null;
      } else {
        const ok = dones.filter((d) => d.kind === 'ok' && d.model === row.model && Math.abs(d.ts - req.ts) <= 5000)[0];
        row.provider = ok?.provider ?? null;
      }
    }
    const inZero = dones.filter((d) => d.kind === 'stream_done' && d.in === 0).length;
    aaid = {
      log: opt.aaidLog, parsed_lines: reqs.length + starts.length + dones.length, bad_lines: bad,
      chat_req: reqs.length, stream_start: starts.length, done_lines: dones.length,
      stream_done_in_zero: inZero,
      note: 'daemon 无关联键——时间窗（±5s）+ 模型 + 流式旗标启发式对齐，仅作旁证；confidence: high=唯一候选 low=多候选 none=未命中',
    };
  }
}

// ── 汇总 ───────────────────────────────────────────────────────────────────
const summary = {
  plan: 'PLAN-098', generated_at: new Date().toISOString(),
  input: { session: opt.session ?? null, run: opt.run ?? null, kind, turns_lines: lines.length, turns_parsed: scoped.length, bad_lines: badLines, aaid_log: opt.aaidLog ?? null },
  by_from: byFrom,
  phases,
  llm_rows: llmRows,
  turns_without_telemetry: turnsWithoutTelemetry,
  signatures,
  aaid,
  observations: [],
};
const inZeroObs = llmRows.filter((r) => r.stream === true && r.in_tokens === 0).length;
if (inZeroObs > 0) summary.observations.push(`daemon 流式 usage 未回填（telemetry in=0）×${inZeroObs}：aaid 流式请求 input token 上报恒 0（KNOWN 观察项，非本计划修复面）`);
if (turnsWithoutTelemetry > 0) summary.observations.push(`${turnsWithoutTelemetry} 个 assistant 轮无 telemetry（旧数据或未走遥测路径——如实省略字段）`);

// ── 输出 ───────────────────────────────────────────────────────────────────
fs.mkdirSync(path.dirname(opt.out), { recursive: true });
fs.writeFileSync(`${opt.out}.json`, JSON.stringify(summary, null, 2));
const md = [];
md.push(`# 运行遥测摘要 — ${target}（${kind}）`);
md.push('');
md.push(`- 输入：turns ${summary.input.turns_parsed}/${summary.input.turns_lines} 行解析（坏行 ${badLines}）`);
md.push(`- 轮数按角色：${Object.entries(byFrom).map(([k, v]) => `${k}=${v.turns}(工具 ${v.tool_calls})`).join('、')}`);
if (phases.length) {
  md.push(`- 相位：${phases.map((p) => `${p.step_id}(${p.role})轮=${p.turns}工具=${p.tool_calls}${p.telemetry ? `✓遥测` : ''}`).join('、')}`);
}
md.push('');
md.push('## 逐请求 LLM 表');
md.push('');
md.push('| seq | carrier | role/step | requested | model | provider* | in | out | elapsed_ms | stop | stream | conf |');
md.push('|---|---|---|---|---|---|---|---|---|---|---|---|');
for (const r of llmRows) {
  md.push(`| ${r.turn_seq} | ${r.carrier} | ${r.role}/${r.step_id ?? '-'} | ${r.requested_model ?? '-'} | ${r.model ?? '-'} | ${r.provider ?? '-'} | ${r.in_tokens ?? '-'} | ${r.out_tokens ?? '-'} | ${r.elapsed_ms ?? '-'} | ${r.stop_reason ?? '-'} | ${r.stream} | ${r.confidence ?? '-'} |`);
}
md.push('');
md.push(`\\* provider=null 为 musk 侧常态（daemon wire 无 provider）；有值来自 aaid 日志旁证（启发式）。`);
md.push('');
md.push('## 失败签名');
md.push('');
for (const [k, v] of Object.entries(signatures)) md.push(`- ${k}: ${v.length === 0 ? '无' : JSON.stringify(v)}`);
md.push('');
for (const o of summary.observations) md.push(`- 观察项：${o}`);
fs.writeFileSync(`${opt.out}.md`, md.join('\n') + '\n');

// stdout 供人工复核
console.log(`collect-telemetry: ${target} (${kind})`);
console.log(`  turns parsed ${summary.input.turns_parsed}/${summary.input.turns_lines} (bad ${badLines}); llm rows ${llmRows.length}; turns_without_telemetry ${turnsWithoutTelemetry}`);
console.log(`  by_from ${JSON.stringify(byFrom)}`);
console.log(`  signatures ${JSON.stringify(Object.fromEntries(Object.entries(signatures).map(([k, v]) => [k, v.length])))}`);
console.log(`  out: ${opt.out}.json / ${opt.out}.md`);

// ── --expect 断言（V03 确定性检查）────────────────────────────────────────
if (opt.expect) {
  const exp = JSON.parse(fs.readFileSync(opt.expect, 'utf8'));
  const fails = [];
  const get = (obj, p) => p.split('.').reduce((o, k) => (o == null ? o : o[k]), obj);
  for (const [p, want] of Object.entries(exp)) {
    const got = get(summary, p);
    const gotLen = Array.isArray(got) ? got.length : got;
    const wantLen = Array.isArray(want) ? want.length : want;
    const ok = Array.isArray(want)
      ? gotLen === wantLen && want.every((w, i) => JSON.stringify(got[i]).includes(JSON.stringify(w).slice(1, -1)) || JSON.stringify(got).includes(JSON.stringify(w)))
      : JSON.stringify(gotLen) === JSON.stringify(wantLen);
    if (!ok) fails.push(`  ${p}: want ${JSON.stringify(want)} got ${JSON.stringify(got)}`);
  }
  if (fails.length) { console.error(`collect-telemetry: --expect FAILED (${fails.length}):`); for (const f of fails) console.error(f); process.exit(3); }
  console.log(`  expect: ${Object.keys(exp).length} assertions OK`);
}
