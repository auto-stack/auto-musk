#!/usr/bin/env node
// Relay run 监视器：轮询 run 状态，打印阶段迁移；human 门到达时以批准决议放行。
// 用法: node relay-watch.mjs <run_id> [timeout_min]
const BASE = 'http://127.0.0.1:17201';
const WS = 'demo-1';
const runId = process.argv[2];
const timeoutMin = Number(process.argv[3] ?? 50);
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function getRun() {
  const r = await fetch(`${BASE}/api/forge/relay/runs/${runId}?workspace=${WS}`);
  return r.json();
}
async function approve(feedback) {
  const r = await fetch(`${BASE}/api/forge/relay/runs/${runId}/gate?workspace=${WS}`, {
    method: 'POST', headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ decision: 'approve', feedback }),
  });
  return r.text();
}

let lastSig = '';
const t0 = Date.now();
let gateApproved = false;
while (Date.now() - t0 < timeoutMin * 60000) {
  let d;
  try { d = await getRun(); } catch (e) { console.error('[watch] fetch err', e.message); await sleep(15000); continue; }
  const waiting = d.waiting_for_gate;
  const steps = (d.steps ?? []).map((s) => `${s.id}(${s.role_id},${s.status}${s.gate === 'human' ? ',GATE' : ''})`).join(' → ');
  const sig = `${d.status}|${d.current_step}|${steps}|${JSON.stringify(waiting)}`;
  if (sig !== lastSig) {
    console.log(`[step] status=${d.status} step=${d.current_step}/${d.total_steps} ${steps}${waiting ? ` WAITING_GATE=${JSON.stringify(waiting).slice(0, 200)}` : ''}`);
    lastSig = sig;
  }
  if (waiting && !gateApproved) {
    console.log('[gate] human gate reached — approving as user...');
    console.log(await approve('计划确认：按此执行。界面用简单网页即可。'));
    gateApproved = true;
    await sleep(3000);
    continue;
  }
  if (d.status === 'completed' || d.status === 'failed' || d.status === 'cancelled') {
    console.log(`[final] status=${d.status}`);
    const evs = (d.events ?? []).filter((e) => ['step_completed', 'gate_resolved', 'run_completed', 'run_failed', 'report_emitted'].includes(e.type));
    for (const e of evs) console.log(`  ev: ${JSON.stringify(e).slice(0, 220)}`);
    break;
  }
  await sleep(20000);
}
