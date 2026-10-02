// scripts/demo/evidence.mjs — PLAN-097 T-07: 证据清单（manifest v1）与校验。
//
// manifest v1（计划 §5.5）：运行指纹、workspace/run/plan 身份、revision、
// 各阶段/attempt/outcome、reviewed_commit、交付检查点与收据；每个 capture
// 含来源/UTC/PNG 相对路径及 SHA256/身份核验标记/kind(live|completed-
// stage-record)/阶段事实引用。外部保存（run 目录），verify 逐条核验：
// schema、文件存在、hash 一致——缺件/错 hash/删除图 = 非零语义（不静默绿）。
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';

const sha256File = p => crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const SOURCES = ['host-vm', 'target-canvas'];
const KINDS = ['live', 'completed-stage-record'];

export function createManifest({ at = new Date().toISOString(), fingerprints, identity }) {
  return {
    format_version: 1,
    at,
    fingerprints,
    identity, // { workspace_id, session_id }
    plans: [],   // { plan_id, plan_revision, run_id, phases:[{stage,attempt,outcome,reviewed_commit}], delivery_checkpoints, receipt_ref }
    captures: [],
    landing: { commit: null, receipts: [] },
  };
}

// 登记一条阶段事实（T-12 各轮调用；capture.stage_ref 必须指向已登记事实）
export function addPlanFacts(manifest, facts) {
  const required = ['plan_id', 'plan_revision', 'run_id', 'phases'];
  for (const k of required) {
    if (facts[k] === undefined) throw new Error(`evidence: plan facts missing ${k}`);
  }
  manifest.plans.push(facts);
  return manifest;
}

// 追加 capture。live 采集必须带 stage_ref（缺阶段注入在此拒绝）——
// 唯一例外 kind=live 且 stage_ref=null 需显式 allowUnstaged（用于场景级
// 整屏，如最终界面）；completed-stage-record 必须带 stage_ref。
export function addCapture(manifest, capture, { allowUnstaged = false } = {}) {
  for (const k of ['capture_id', 'source', 'at', 'png', 'sha256', 'bytes', 'kind']) {
    if (capture[k] === undefined) throw new Error(`evidence: capture missing ${k}`);
  }
  if (!SOURCES.includes(capture.source)) throw new Error(`evidence: unknown source ${capture.source}`);
  if (!KINDS.includes(capture.kind)) throw new Error(`evidence: unknown kind ${capture.kind}`);
  if (capture.kind === 'completed-stage-record' && !capture.stage_ref) {
    throw new Error('evidence: completed-stage-record requires stage_ref（历史标识必须绑定真实阶段事实）');
  }
  if (capture.kind === 'live' && !capture.stage_ref) {
    if (!allowUnstaged) throw new Error('evidence: live capture missing stage_ref（缺阶段注入拒绝;场景级整屏传 allowUnstaged）');
    capture.allow_unstaged = true; // 落到记录,verify 对账
  }
  if (capture.stage_ref) {
    const ref = capture.stage_ref;
    const ok = manifest.plans.some(p => p.plan_id === ref.plan_id
      && p.phases.some(ph => ph.stage === ref.stage && ph.attempt === ref.attempt));
    if (!ok) throw new Error(`evidence: stage_ref not in registered plan facts: ${JSON.stringify(ref)}`);
  }
  manifest.captures.push(capture);
  return manifest;
}

export function setLanding(manifest, { commit, receipts }) {
  manifest.landing = { commit, receipts };
  return manifest;
}

// 逐条校验。返回 {ok, errors}；不抛（调用方决定退出码语义）。
export function verifyManifest(manifest, evidenceDir) {
  const errors = [];
  const err = m => errors.push(m);
  if (manifest?.format_version !== 1) err('format_version != 1');
  if (!manifest?.fingerprints?.musk_exe?.sha256) err('fingerprints.musk_exe missing');
  if (!manifest?.identity?.workspace_id) err('identity.workspace_id missing');
  if (!Array.isArray(manifest?.captures) || manifest.captures.length === 0) err('no captures');
  for (const [i, c] of (manifest?.captures ?? []).entries()) {
    const png = path.join(evidenceDir, c.png ?? '');
    if (!fs.existsSync(png)) { err(`captures[${i}] ${c.capture_id}: png missing on disk（删除图）`); continue; }
    const got = sha256File(png);
    if (got !== c.sha256) err(`captures[${i}] ${c.capture_id}: sha256 mismatch（错 hash/被改动）`);
    if (c.kind === 'completed-stage-record' && !c.stage_ref) err(`captures[${i}]: completed-stage-record without stage_ref`);
    if (c.kind === 'live' && !c.stage_ref && !c.allow_unstaged) err(`captures[${i}]: live without stage_ref`);
  }
  for (const p of manifest?.plans ?? []) {
    if (!Array.isArray(p.phases) || p.phases.length === 0) err(`plan ${p.plan_id}: no phases`);
  }
  return { ok: errors.length === 0, errors };
}

// 落盘/读回（外部 run 目录；与 worktree 生命周期解耦）
export function writeManifest(manifest, runDir) {
  const p = path.join(runDir, 'evidence-manifest.json');
  fs.writeFileSync(p, JSON.stringify(manifest, null, 2));
  return p;
}
export function readManifest(runDir) {
  return JSON.parse(fs.readFileSync(path.join(runDir, 'evidence-manifest.json'), 'utf8'));
}
