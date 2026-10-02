// scripts/demo/focusboard.mjs — PLAN-097 T-08: FocusBoard 三轮演示编排。
//
// 职责（计划 §5.4）：空仓 bootstrap（白名单审计）、三轮正式入口、交付等待
// 与停止条件。不携带解决方案：bootstrap 只物化 README/AGENTS/需求文档并
// 做白名单审计（任何业务文件 = 违规）；每轮仅提交一次已授权需求后观察
// ——needs_replan/blocked/修复封顶即停（留证），无手工 nudge、无外部修复。
//
// 正式入口（096 探针同源）：POST /api/forge/relay/runs
//   { flow_id: 'plan', task: <需求全文>, authorization: 'auto' }
//   → POST /runs/{run_id}/advance → 轮询 facts 至终态（contract.mjs 校验）。
import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { parseRunStateResponse } from './contract.mjs';
import { deliveredGate, runChecks, canvasRestart, p001Checks, p002Checks, p003Checks, p003SessionVerdict, p003InputVerdict, p003ReadEvidence } from './assertions.mjs';
import { readAgentWiki, publishWiki, readbackWiki, wikiHashVerdict } from './wiki.mjs';
import { collectLandingFacts, landingVerdict } from './landing.mjs';

// bootstrap 白名单（V02 直测）：仅需求/约定物。任何其他路径 = 违规。
export const BOOTSTRAP_WHITELIST = [
  'README.md',
  'AGENTS.md',
  'requests/001.md',
  'requests/002.md',
  'requests/003.md',
];

// 纯函数：bootstrap 产物审计——createdFiles 相对路径列表 → 违规项。
export function bootstrapViolations(createdFiles) {
  const allow = new Set(BOOTSTRAP_WHITELIST);
  return createdFiles.filter(f => !allow.has(f.replace(/\\/g, '/')));
}

// 纯函数：轮次停止条件分类（facts = contract.mjs parseRunStateResponse 产物）。
// 返回 {stop, reason, terminal}。规则（§5.4）：
//   completed+delivered = 正常交付完成；
//   failed / canceled = 响亮停止（留证）；
//   outcome ∈ needs_replan|blocked = 停（授权边界，不 nudge）；
//   repair_count >= repair_limit(>0) = 修复封顶停。
export function roundStopVerdict(facts) {
  const st = facts?.status;
  const pe = facts?.plan_execution ?? {};
  const phase = pe.phase ?? '';
  const outcome = pe.outcome ?? '';
  const repair = pe.repair_count ?? 0;
  const limit = pe.repair_limit ?? 0;
  if (st === 'completed' && phase === 'delivered') return { stop: true, terminal: true, reason: 'delivered' };
  if (st === 'failed') return { stop: true, terminal: true, reason: `run failed (phase=${phase} outcome=${outcome} blocker=${pe.blocker ?? '-'})` };
  if (st === 'canceled') return { stop: true, terminal: true, reason: 'run canceled' };
  if (outcome === 'needs_replan') return { stop: true, terminal: false, reason: 'needs_replan（授权边界,不 nudge）' };
  if (outcome === 'blocked') return { stop: true, terminal: false, reason: `blocked: ${pe.blocker ?? '(无 blocker 文本)'}` };
  if (limit > 0 && repair >= limit) return { stop: true, terminal: false, reason: `repair cap reached (${repair}/${limit})` };
  return { stop: false, terminal: false, reason: `running (status=${st} phase=${phase})` };
}

// 空仓 bootstrap：物化 README/AGENTS/requests → 白名单审计 → git init+commit。
// sourceDir = examples/focusboard-demo（仓内需求源）;repoDir = 每跑新建的
// 演示仓（workspace root,即 serve --workdir 指向）。
export function bootstrapRepo({ repoDir, sourceDir }) {
  fs.rmSync(repoDir, { recursive: true, force: true });
  fs.mkdirSync(repoDir, { recursive: true });
  const created = [];
  const materialize = (relSrc, relDst) => {
    const dst = path.join(repoDir, relDst);
    fs.mkdirSync(path.dirname(dst), { recursive: true });
    fs.copyFileSync(path.join(sourceDir, relSrc), dst);
    created.push(relDst);
  };
  materialize('README.md', 'README.md');
  for (const r of ['001', '002', '003']) materialize(`requests/${r}.md`, `requests/${r}.md`);
  // AGENTS.md：演示仓的工程约定（worktree 外置/线性合回/知识职责——README 同款口径,
  // 面向 agent 的指令形态;内容为约定,非解决方案）
  fs.writeFileSync(path.join(repoDir, 'AGENTS.md'), `# FocusBoard 工程约定

- 每个产品计划走平台 /auto-plan:new → work → review → merge 四技能闭环;
  工程 worktree 外置(.wt/focusboard-<run_id>-<NNN>/),默认分支线性合回(ff-only)。
- 业务 Specs 为权威(docs/specs/),ledger 派生;Wiki 只记用法/设计取舍
  (docs/wiki/*.md)并链接规范与计划。
- 界面跑在 Musk 原生 VM Canvas 上;数据文件放 app 目录内(app 相对路径
  实际落在 <app>/src/front/data/,跨 Canvas stop/start 存活)。
- 需求原文见 requests/;每轮需求独立提交,不带前几轮聊天上下文(P003 新会话)。
`);
  created.push('AGENTS.md');

  const violations = bootstrapViolations(created);
  if (violations.length) throw new Error(`bootstrap: whitelist violations ${JSON.stringify(violations)}（不写业务审计失败）`);

  execFileSync('git', ['init', '-b', 'master'], { cwd: repoDir, stdio: 'ignore', windowsHide: true });
  execFileSync('git', ['add', '-A'], { cwd: repoDir, stdio: 'ignore', windowsHide: true });
  execFileSync('git', ['-c', 'user.name=demo-driver', '-c', 'user.email=demo@local', 'commit', '-m', 'bootstrap: 需求与约定（空仓,无业务代码/Specs/测试）'], { cwd: repoDir, stdio: 'ignore', windowsHide: true });
  const head = execFileSync('git', ['-C', repoDir, 'rev-parse', 'HEAD'], { encoding: 'utf8' }).trim();
  return { created, violations: [], head };
}

// 三轮正式入口：提交一次已授权需求（task = 需求全文）并 kick 驱动。
export async function startRound({ base, token, round, reqText }) {
  const headers = { 'content-type': 'application/json' };
  if (token) headers.authorization = `Bearer ${token}`;
  const res = await fetch(`${base}/api/forge/relay/runs`, {
    method: 'POST', headers, body: JSON.stringify({ flow_id: 'plan', task: reqText, authorization: 'auto' }),
  });
  const text = await res.text();
  if (res.status !== 200) throw new Error(`focusboard: start round ${round} -> ${res.status}: ${text.slice(0, 200)}`);
  const { run_id: runId } = JSON.parse(text);
  const adv = await fetch(`${base}/api/forge/relay/runs/${encodeURIComponent(runId)}/advance`, { method: 'POST', headers });
  if (adv.status >= 500) throw new Error(`focusboard: advance -> ${adv.status}`);
  return { runId, reqText };
}

// 交付等待：轮询 facts 至停止条件（roundStopVerdict）；不 nudge。
// 返回 { facts, verdict }；超时 = 响亮失败。
// 合同容错（T-12 干跑实证）：启动即失败的 run 其 plan_execution 为空壳
// （serde default,plan_id/plan_path 空）——严格校验拒收会让失败轮无法
// 归类。故：校验失败时降级取原始 JSON,若 status 已 failed/canceled 则按
// 响亮停止路径返回原始事实（verdict 分类器消费 status/outcome）;仍在跑
// 则继续轮询。completed 但事实违约不降级（缺件完成不绿,保持响亮）。
export async function waitForRound({ base, token, runId, timeoutMs = 3600000, onProgress = null }) {
  const headers = token ? { authorization: `Bearer ${token}` } : {};
  const deadline = Date.now() + timeoutMs;
  let last = '';
  while (Date.now() < deadline) {
    await new Promise(r => setTimeout(r, 5000));
    const res = await fetch(`${base}/api/forge/relay/runs/${encodeURIComponent(runId)}`, { headers });
    if (res.status === 404) continue; // run 尚未可见
    const text = await res.text();
    let facts;
    try {
      facts = parseRunStateResponse(text);
    } catch (e) {
      let raw = null;
      try { raw = JSON.parse(text); } catch { /* 非 JSON:继续轮询 */ }
      if (raw?.status === 'failed' || raw?.status === 'canceled') {
        const verdict = roundStopVerdict(raw);
        const line = `status=${raw.status} (contract-invalid facts degraded for terminal state: ${e.message.slice(0, 80)})`;
        if (line !== last) { last = line; onProgress?.(line); }
        return { facts: raw, verdict };
      }
      continue; // 校验失败但非终态:继续等（可能仍在装配）
    }
    const verdict = roundStopVerdict(facts);
    const line = `status=${facts.status} phase=${facts.plan_execution?.phase ?? '-'} attempt=${facts.plan_execution?.attempt ?? '-'} repair=${facts.plan_execution?.repair_count ?? '-'}`;
    if (line !== last) { last = line; onProgress?.(line); }
    if (verdict.stop) return { facts, verdict };
  }
  throw new Error(`focusboard: round ${runId} timeout after ${timeoutMs}ms（响亮失败,现场保留）`);
}

// ── T-09: 轮次交付验证复合 ──────────────────────────────────────────────
// 每轮 waitForRound 停在 delivered 后调用：①交付门（未交付抛错——工程未
// 交付不发下一需求，由调用方据此终止）；②目标 VM 行为断言（P00X 流，
// canvasRestart 绑定为 mcp.__restart）；③Agent Wiki 正文经既有 API 发布/
// 回读（hash 一致）；④工程落地对账（Specs/ledger/归档/ff/清理）。
// 返回 {ok, checks, landing}；ok=false 时调用方不得进入下一轮。

const ROUND_BUILDERS = { p001: p001Checks, p002: p002Checks, p003: p003Checks };

export async function verifyRoundDelivery({ round, planLabel, facts, mcp, repoDir, base, token, workspaceId,
  wikiFiles = [], runId, planMarker = null, log = () => {}, fetchImpl = fetch, wtRoot }) {
  deliveredGate(facts, { planLabel }); // 未交付在此抛——不发下一需求

  const withRestart = {
    ...mcp,
    __restart: mcp.__restart ?? (() => canvasRestart({ base, token, fetchImpl })),
  };
  const builder = ROUND_BUILDERS[round];
  if (!builder) throw new Error(`focusboard: unknown round '${round}'（expected p001|p002|p003）`);
  const behavior = await runChecks(builder({ mcp: withRestart, repoDir }), { log });

  const wikiChecks = [];
  for (const rel of wikiFiles) {
    try {
      const content = readAgentWiki(repoDir, rel);
      const slug = rel.replace(/^docs[\\/]wiki[\\/]/, '').replace(/\.md$/, '');
      const pub = await publishWiki({ base, token, workspaceId, slug, title: slug, content, fetchImpl });
      const rb = await readbackWiki({ base, token, workspaceId, slug, fetchImpl });
      const v = wikiHashVerdict(pub.sourceSha256, rb.readbackSha256);
      wikiChecks.push({ name: `wiki ${rel}（${pub.action}+回读）`, ok: v.ok, detail: v.reason });
    } catch (e) {
      wikiChecks.push({ name: `wiki ${rel}`, ok: false, detail: e.message });
    }
    log(`[wiki] ${rel} → ${wikiChecks[wikiChecks.length - 1].ok ? 'PASS' : 'FAIL'}`);
  }

  let landingCheck;
  try {
    const facts2 = collectLandingFacts({ repoDir, runId, planMarker, wtRoot });
    const lv = landingVerdict(facts2);
    landingCheck = { name: `landing 对账(${planLabel}: specs/ledger/归档/ff/清理)`, ok: lv.ok, detail: lv.errors.join('; ') || '线性合入+分支/worktree 清理+Specs/ledger/归档/Wiki 齐' };
  } catch (e) {
    landingCheck = { name: `landing 对账(${planLabel})`, ok: false, detail: e.message };
  }
  log(`[landing] ${landingCheck.ok ? 'PASS' : 'FAIL'} ${landingCheck.detail}`);

  const checks = [...behavior.checks, ...wikiChecks, landingCheck];
  return { ok: checks.every(c => c.ok), checks, behavior, landing: landingCheck };
}

// P003 上下文来源复合（AC-12）：新会话 + 输入=需求原文 + 事件窗内真实读取。
export function p003ContextChecks({ p003SessionId, priorSessionIds, inputText, requestText, events }) {
  const s = p003SessionVerdict(p003SessionId, priorSessionIds ?? []);
  const i = p003InputVerdict(inputText, requestText);
  const r = p003ReadEvidence(events);
  return [
    { name: 'P003 新会话(无前两轮聊天注入)', ok: s.ok, detail: s.reason },
    { name: 'P003 会话输入=需求原文', ok: i.ok, detail: i.reason },
    { name: 'P003 真实读取 Specs/工件(事件窗证据)', ok: r.verdict.ok, detail: r.verdict.ok ? `${r.paths.length} reads: ${r.paths.slice(0, 5).join(', ')}` : r.verdict.reason },
  ];
}

// CLI：bootstrap 独立可跑（T-08 V01 验收面）
if (process.argv[1] && (import.meta.url === `file:///${process.argv[1].replace(/\\/g, '/')}` || process.argv[1]?.endsWith('focusboard.mjs'))) {
  const args = process.argv.slice(2);
  if (args[0] === 'bootstrap') {
    const opt = (n, d) => { const i = args.indexOf(n); return i >= 0 ? args[i + 1] : d; };
    const repoDir = opt('--repo-dir');
    const sourceDir = opt('--source-dir', new URL('.', import.meta.url).pathname.replace(/^\/(\w:)/, '$1') + '../../examples/focusboard-demo');
    if (!repoDir) { console.error('usage: focusboard.mjs bootstrap --repo-dir <dir> [--source-dir <examples/focusboard-demo>]'); process.exit(2); }
    const src = path.resolve(sourceDir);
    const r = bootstrapRepo({ repoDir, sourceDir: src });
    console.log(`[focusboard] bootstrap ok: files=${JSON.stringify(r.created)} head=${r.head.slice(0, 12)}`);
    console.log(`[focusboard] whitelist audit clean（无业务文件）`);
  }
}
