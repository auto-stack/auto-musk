// scripts/demo/run.mjs — PLAN-097 T-12: 三轮真实模型演示编排（V05）。
//
// 形态（§5.3/§5.4/§5.5）：每跑唯一 run_id/目录；自有 musk serve+原生 VM
// （VmSession,真实 AAID 只读复用）；空仓 bootstrap；三轮正式 plan 入口
// （startRound,零 nudge）→ waitForRound（停止条件分类）→ deliveredGate
// （工程未交付不发下一需求）→ verifyRoundDelivery（行为断言+Wiki 发布
// 回读+落地对账）；阶段切换/轮终采集（capture.mjs,身份双验+原图 hash）；
// manifest v1（evidence.mjs）；取消/失败保留现场非零退出。
import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { VmSession } from './vm-session.mjs';
import { bootstrapRepo, startRound, waitForRound } from './focusboard.mjs';
import { parseRunStateResponse } from './contract.mjs';
import { createManifest, addPlanFacts, addCapture, writeManifest, verifyManifest } from './evidence.mjs';
import { captureHostVm, captureTargetCanvas } from './capture.mjs';
import { verifyRoundDelivery, p003ContextChecks } from './focusboard.mjs';
import { deliveredGate } from './assertions.mjs';

const sleep = ms => new Promise(r => setTimeout(r, ms));

const DEMO_ROOT = 'D:/autostack/.demo/musk-097';
const REQUEST_FILES = { p001: 'requests/001.md', p002: 'requests/002.md', p003: 'requests/003.md' };

function listWikiFiles(repoDir) {
  const dir = path.join(repoDir, 'docs', 'wiki');
  if (!fs.existsSync(dir)) return [];
  const out = [];
  const walk = (d, base) => {
    for (const e of fs.readdirSync(d, { withFileTypes: true })) {
      const rel = base ? `${base}/${e.name}` : e.name;
      if (e.isDirectory()) walk(path.join(d, e.name), rel);
      else if (e.isFile() && e.name.endsWith('.md')) out.push(`docs/wiki/${rel.replace(/\\/g, '/')}`);
    }
  };
  walk(dir, '');
  return out;
}

// 新建 UI 会话（P002/P003 新会话面;relay run 本身不绑会话——上下文证据
// 由 p003ContextChecks 的输入原文比对与事件窗读取证据承载）。
async function createSession(base, token, workspaceId) {
  const res = await fetch(`${base}/api/chats/session?workspace=${encodeURIComponent(workspaceId)}`, {
    method: 'POST', headers: { 'content-type': 'application/json', ...(token ? { authorization: `Bearer ${token}` } : {}) },
    body: JSON.stringify({ project_path: '' }),
  });
  const j = await res.json();
  const id = j?.session?.id;
  if (!id) throw new Error(`run: create session failed -> ${res.status}: ${JSON.stringify(j).slice(0, 160)}`);
  return id;
}

export async function runDemo({ runId, autoExe, muskExe, ports, aaidUrl, rounds, roundTimeoutMs, root }) {
  if (!path.isAbsolute(autoExe ?? '') || !fs.existsSync(autoExe)) {
    console.error(`[run] --auto-exe 必须显式绝对路径（锁当日构建硬规则），got: ${autoExe}`);
    return 2;
  }
  const runRoot = path.join(DEMO_ROOT, runId);
  if (fs.existsSync(runRoot)) { console.error(`[run] run 目录已存在（不覆盖旧跑）: ${runRoot}`); return 2; }
  const repoDir = path.join(runRoot, 'focusboard');
  const evidenceDir = path.join(runRoot, 'evidence');
  const wtRoot = path.join(runRoot, 'wt');
  fs.mkdirSync(evidenceDir, { recursive: true });
  fs.mkdirSync(wtRoot, { recursive: true });

  const log = m => console.log(`[run] ${m}`);
  log(`run_id=${runId} root=${runRoot}`);

  const boot = bootstrapRepo({ repoDir, sourceDir: path.join(root, 'examples', 'focusboard-demo') });
  log(`bootstrap head=${boot.head.slice(0, 12)} files=${boot.created.length}`);

  const sess = new VmSession({
    runRoot, ports, muskExe, autoExe,
    workspaceDir: repoDir, projectDir: root, aaidUrl,
    extraServeEnv: {
      MUSK_SKILLS_DIR: path.join(root, '.agents', 'skills'),
      MUSK_PLAN_WORKTREE_ROOT: wtRoot,
      MUSK_PLAN_MAX_TOKENS: '32768',
    },
  });

  const manifest = createManifest({ identity: null });
  const roundResults = [];
  let session = null;
  try {
    const started = await sess.start();
    manifest.fingerprints = { ...started.fingerprints, worktree_head: execHead(root) };
    log(`session up base=${started.base} mcp=${started.mcpEndpoint}`);
    const seeded = await sess.seed();
    manifest.identity = { workspace_id: seeded.workspace.id, session_id: seeded.sessionId };
    session = { ...seeded, sessionId: seeded.sessionId, sessionIds: [seeded.sessionId] };
    log(`seeded workspace=${seeded.workspace.id} session=${seeded.sessionId}`);
    // 等 shell 渲染（宿主 VM 首帧;早采集在"No UI available"上必败——干跑实证）
    await sess.mcp.waitForSnapshot(s => /Musk|composer|描述你想构建|新建会话/i.test(s), 120000, 'host shell render');
    log('host shell rendered');

    for (const round of rounds) {
      log(`── round ${round} ──`);
      const r = await runOneRound({ round, sess, session, runRoot, repoDir, evidenceDir, wtRoot, root, manifest, roundTimeoutMs, log });
      roundResults.push(r);
      if (!r.ok) {
        log(`round ${round} NOT ok —— 工程未交付或验收失败,停止演示（失败留证,不覆盖现场）`);
        break;
      }
    }
  } catch (e) {
    log(`FATAL: ${e.message}（现场保留:${runRoot}）`);
    roundResults.push({ round: 'session', ok: false, error: e.message });
  } finally {
    const stop = await sess.stop();
    log(`owned 关停: allGone=${stop.allGone} portsFreed=${stop.portsFreed}`);
    (manifest.summary ??= {}).stop = stop;
  }

  // p003 上下文检查（三轮完成后判定;失败只记录到 summary,不掩盖轮次结果）
  const p003 = roundResults.find(r => r.round === 'p003');
  if (p003?.context) manifest.summary ??= {};
  (manifest.summary ??= {}).rounds = roundResults.map(r => ({ round: r.round, ok: r.ok, delivered: r.delivered ?? false, checks: r.checks?.length, fails: r.checks?.filter(c => !c.ok).length ?? 0, error: r.error }));

  const mPath = writeManifest(manifest, runRoot);
  fs.writeFileSync(path.join(runRoot, 'run-summary.json'), JSON.stringify({ runId, at: new Date().toISOString(), rounds: manifest.summary?.rounds, portsFreed: manifest.summary?.stop?.portsFreed }, null, 2));
  const v = verifyManifest(manifest, evidenceDir);
  log(`manifest=${mPath} verify=${v.ok ? 'OK' : `FAIL(${v.errors.length})`}`);
  for (const e of v.errors) log(`  verify: ${e}`);

  const allRoundsOk = roundResults.length === rounds.length && roundResults.every(r => r.ok);
  log(allRoundsOk && v.ok ? `DEMO PASS (${roundResults.length}/${rounds.length} rounds)` : `DEMO FAIL(现场保留:${runRoot})`);
  return allRoundsOk && v.ok ? 0 : 1;
}

function execHead(dir) {
  try { return execFileSync('git', ['-C', dir, 'rev-parse', 'HEAD'], { encoding: 'utf8' }).trim(); } catch { return null; }
}

async function runOneRound({ round, sess, session, runRoot, repoDir, evidenceDir, wtRoot, root, manifest, roundTimeoutMs, log }) {
  const base = sess.base;
  const token = sess.token;
  const reqText = fs.readFileSync(path.join(root, 'examples', 'focusboard-demo', REQUEST_FILES[round]), 'utf8');

  // 新 UI 会话（P002/P003 尤其;全部轮记录 session id 供 P003 判言）
  const uiSessionId = await createSession(base, token, session.workspace.id);
  session.sessionIds.push(uiSessionId);
  log(`ui session=${uiSessionId}`);

  const captures = [];   // {capture rec} 成功采集
  const captureErrors = [];
  const capture = async (captureId, kind, stageRef, allowUnstaged = false) => {
    try {
      const rec = await captureHostVm({ mcp: sess.mcp, evidenceDir, captureId, markers: [], kind, stageRef });
      captures.push(rec);
      log(`capture ${captureId} ok (${rec.bytes}B)`);
      return rec;
    } catch (e) {
      captureErrors.push({ captureId, error: e.message });
      log(`capture ${captureId} FAILED: ${e.message}`);
      return null;
    }
  };

  // 需求提交前场景帧（场景级整屏,allowUnstaged）
  await capture(`${round}-00-requirement`, 'live', null, true);

  const seenPhases = new Set();
  const { runId: relayRunId, facts } = await startAndWait({
    base, token, round, reqText, timeoutMs: roundTimeoutMs,
    onPhase: async phase => {
      if (seenPhases.has(phase)) return;
      seenPhases.add(phase);
      await capture(`${round}-live-${phase}`, 'live', { plan_id: `PLAN-${round.toUpperCase()}`, stage: phase, attempt: 1 });
    },
    log,
  });

  deliveredGate(facts, { planLabel: round }); // 未交付抛错——停止不 nudge

  // 登记 plan 事实（阶段时间线来自 run 事件窗,不是叙述）
  const stages = (facts.events ?? []).filter(e => e.type === 'plan_stage_facts')
    .map(e => ({ stage: e.facts?.stage, attempt: e.facts?.attempt ?? 1, outcome: e.facts?.outcome }));
  const seen = new Set();
  const phases = stages.filter(s => s.stage && !seen.has(`${s.stage}:${s.attempt}`) && seen.add(`${s.stage}:${s.attempt}`));
  addPlanFacts(manifest, {
    plan_id: facts.plan_execution?.plan_id ?? `PLAN-${round.toUpperCase()}`,
    plan_revision: facts.plan_execution?.plan_revision ?? 1,
    run_id: relayRunId,
    phases,
    delivery_checkpoints: facts.plan_execution?.delivery_checkpoints ?? {},
    receipt_ref: facts.plan_execution?.receipt_ref ?? null,
    reviewed_commit: facts.plan_execution?.reviewed_commit ?? null,
  });
  log(`plan facts: ${facts.plan_execution?.plan_id} phases=[${phases.map(p => p.stage).join('→')}]`);

  // live 采集回填 stage_ref（采集发生在阶段进行中,登记在事实落账后）
  for (const c of captures) {
    try {
      if (c.stage_ref && !c.stage_ref.plan_id) c.stage_ref.plan_id = manifest.plans[manifest.plans.length - 1].plan_id;
      addCapture(manifest, { ...c }, { allowUnstaged: !c.stage_ref });
    } catch (e) {
      captureErrors.push({ captureId: c.capture_id, error: `stage_ref bind: ${e.message}` });
    }
  }

  // 交付后验证：行为断言 + Wiki + 落地对账
  const vr = await verifyRoundDelivery({
    round, planLabel: round, facts, mcp: sess.mcp, repoDir,
    base, token, workspaceId: session.workspace.id,
    wikiFiles: listWikiFiles(repoDir), runId, wtRoot,
    log: m => log(m),
  });

  // 交付后采集：最终 UI（场景级）+ 目标 Canvas 帧（generation 归属核验）
  let canvasGen = null;
  try {
    const st = await (await fetch(`${base}/api/canvas/status`)).json();
    canvasGen = st?.generation_id ?? null;
    if (st?.state === 'running' && canvasGen) {
      const rec = await captureTargetCanvas({ base, evidenceDir, captureId: `${round}-90-canvas`, generation: canvasGen, kind: 'live', stageRef: { plan_id: manifest.plans[manifest.plans.length - 1].plan_id, stage: 'delivered', attempt: facts.plan_execution?.attempt ?? 1 } });
      captures.push(rec);
      addCapture(manifest, rec);
      log(`target canvas captured gen=${canvasGen}`);
    } else {
      captureErrors.push({ captureId: `${round}-90-canvas`, error: `canvas not running (state=${st?.state})` });
    }
  } catch (e) { captureErrors.push({ captureId: `${round}-90-canvas`, error: e.message }); }
  await capture(`${round}-95-final`, 'live', null, true);

  // P003 上下文判言（AC-12;证据=p003 会话输入原文+事件窗读取）
  let context = null;
  if (round === 'p003') {
    const checks = p003ContextChecks({
      p003SessionId: session.sessionIds[session.sessionIds.length - 1],
      priorSessionIds: session.sessionIds.slice(0, -1),
      inputText: reqText, requestText: reqText,
      events: facts.events ?? [],
    });
    context = { ok: checks.every(c => c.ok), checks };
    for (const c of checks) log(`[p003-context] ${c.ok ? 'PASS' : 'FAIL'} ${c.name} — ${c.detail}`);
  }

  const ok = vr.ok && (context ? context.ok : true);
  return { round, ok, delivered: true, checks: vr.checks, context, captures: captures.length, captureErrors };
}

// startRound + waitForRound 组合;onPhase 在相位变化时回调（采集钩子）。
async function startAndWait({ base, token, round, reqText, timeoutMs, onPhase, log }) {
  let lastPhase = '';
  const { runId } = await startRound({ base, token, round, reqText });
  log(`relay run=${runId}`);
  const { facts } = await waitForRound({
    base, token, runId, timeoutMs,
    onProgress: line => {
      log(`${round}: ${line}`);
      const m = line.match(/phase=(\w+)/);
      if (m && m[1] !== lastPhase && m[1] !== '-') {
        lastPhase = m[1];
        onPhase?.(m[1]);
      }
    },
  });
  // 校验过的事实（contract.validateRunState 在 waitForRound 内已验）
  return { runId, facts: facts };
}

// ── verify 子命令（V06 独立证据核验;不依赖运行中进程） ─────────────────
export async function verifyCommand({ manifestPath, evidenceDir }) {
  if (!manifestPath || !fs.existsSync(manifestPath)) {
    console.error(`[verify] manifest 不存在: ${manifestPath}`);
    return 2;
  }
  const evDir = evidenceDir || path.dirname(manifestPath);
  const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
  const v = verifyManifest(manifest, evDir);
  for (const e of v.errors) console.error(`[verify] FAIL ${e}`);
  console.log(`[verify] ${v.ok ? 'MANIFEST OK' : `${v.errors.length} error(s)`}`);
  console.log(`[verify] plans=${manifest.plans?.length ?? 0} captures=${manifest.captures?.length ?? 0} landing_commit=${manifest.landing?.commit ?? '-'}`);
  return v.ok ? 0 : 1;
}
