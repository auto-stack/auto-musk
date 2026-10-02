// tests/demo/focusboard-verify.test.mjs — PLAN-097 T-09 V02：轮次交付验证
// 复合（verifyRoundDelivery）与 P003 上下文复合（p003ContextChecks）。
// ①未交付抛错(不发下一需求);②合规假应用+真临时仓+HTTP 桩全绿;
// ③Wiki hash 错/交付缺件红。
import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { execFileSync } from 'node:child_process';
import { fakeBoard } from './helpers/fake-board.mjs';
import { verifyRoundDelivery, p003ContextChecks } from '../../scripts/demo/focusboard.mjs';

const DELIVERED = {
  status: 'completed',
  plan_execution: {
    contract_version: 1, plan_id: 'PLAN-001', plan_seq: 1, plan_path: 'docs/plans/001-basic-board.md',
    plan_revision: 1, phase: 'delivered', outcome: 'pass', repair_count: 0, repair_limit: 3,
    reviewed_commit: 'a'.repeat(40),
    delivery_checkpoints: Object.fromEntries(['prepared', 'landed', 'ledger_refreshed', 'archived', 'cleaned'].map(k => [k, { checkpoint: k }])),
  },
};

function tempRepo() {
  const repo = fs.mkdtempSync(path.join(os.tmpdir(), 'e097-verify-'));
  const git = (...a) => execFileSync('git', ['-C', repo, ...a], { encoding: 'utf8', windowsHide: true });
  git('init', '-b', 'master'); git('config', 'user.email', 't@l'); git('config', 'user.name', 't');
  fs.mkdirSync(path.join(repo, 'docs', 'specs'), { recursive: true });
  fs.mkdirSync(path.join(repo, 'docs', 'wiki'), { recursive: true });
  fs.mkdirSync(path.join(repo, 'docs', 'plans', 'archived'), { recursive: true });
  fs.writeFileSync(path.join(repo, 'docs', 'specs', 'board.md'), '# board');
  fs.writeFileSync(path.join(repo, 'docs', 'specs', 'index.json'), '{}');
  fs.writeFileSync(path.join(repo, 'docs', 'wiki', 'usage.md'), '# FocusBoard 用法\n');
  fs.writeFileSync(path.join(repo, 'docs', 'plans', 'archived', '001-basic-board.md'), '# P001');
  git('add', '-A'); git('commit', '-m', 'p001');
  return repo;
}

// HTTP 桩:canvas status/stop/start + wiki create/page
function stubHttp({ gen = 1, wikiReadback = null }) {
  let g = gen;
  let stopped = false;
  const wiki = new Map();
  return async (url, opts = {}) => {
    const u = String(url);
    const json = j => ({ status: 200, json: async () => j, text: async () => JSON.stringify(j) });
    if (u.includes('/api/canvas/status')) return json({ state: stopped ? 'running' : 'running', generation_id: g, app_path: 'src/front/app.at', frame: { valid: true } });
    if (u.includes('/api/canvas/stop')) { stopped = true; return json({ state: 'stopped', generation_id: g }); }
    if (u.includes('/api/canvas/start')) { g += 1; stopped = true; return json({ generation_id: g }); }
    if (u.includes('/api/forge/wiki/') && u.includes('/pages') && opts.method === 'POST') {
      const body = JSON.parse(opts.body);
      if (wiki.has(body.slug)) return { status: 409, text: async () => 'exists' };
      wiki.set(body.slug, body.content);
      return { status: 200, json: async () => ({}), text: async () => '{}' };
    }
    if (u.includes('/api/forge/wiki/') && opts.method === 'PUT') {
      const slug = decodeURIComponent(u.split('/page/')[1].split('?')[0]);
      wiki.set(slug, JSON.parse(opts.body).content);
      return { status: 200, json: async () => ({}), text: async () => '{}' };
    }
    if (u.includes('/api/forge/wiki/') && u.includes('/page/')) {
      const slug = decodeURIComponent(u.split('/page/')[1].split('?')[0]);
      const content = wikiReadback !== null ? wikiReadback : wiki.get(slug);
      if (content === undefined) return { status: 404 };
      return { status: 200, json: async () => ({ page: { content } }) };
    }
    throw new Error(`stubHttp: unexpected ${u}`);
  };
}

test('verifyRoundDelivery: 未交付抛错(工程未交付不发下一需求)', async () => {
  await assert.rejects(
    () => verifyRoundDelivery({ round: 'p001', planLabel: 'P001', facts: { status: 'running', plan_execution: { phase: 'execute' } }, mcp: {}, repoDir: os.tmpdir(), base: 'http://x', wikiFiles: [], runId: 'r' }),
    /未交付.*不发下一需求/,
  );
});

test('verifyRoundDelivery: 合规应用全绿(行为+wiki hash+落地对账)', async t => {
  const repo = tempRepo();
  const mcp = fakeBoard({ mode: 'p001' });
  mcp.repoDir = repo;
  const fetchImpl = stubHttp({ gen: 3 });
  // canvasRestart 走 stubHttp(注入)——verifyRoundDelivery 内 canvasRestart 默认
  // 全局 fetch,这里通过 mcp.__restart 显式绑定 stub 版
  const { canvasRestart } = await import('../../scripts/demo/assertions.mjs');
  mcp.__restart = () => canvasRestart({ base: 'http://x', fetchImpl });
  const r = await verifyRoundDelivery({
    round: 'p001', planLabel: 'P001', facts: DELIVERED, mcp, repoDir: repo,
    base: 'http://x', workspaceId: 'ws1', wikiFiles: ['docs/wiki/usage.md'],
    runId: 'runT', planMarker: '001-basic-board', log: () => {}, fetchImpl,
    wtRoot: path.join(repo, 'wt-empty'),
  });
  assert.equal(r.ok, true, JSON.stringify(r.checks.filter(c => !c.ok), null, 1));
  assert.ok(r.checks.some(c => /usage\.md/.test(c.name) && c.ok));
  assert.ok(r.checks.some(c => /landing 对账/.test(c.name) && c.ok));
  fs.rmSync(repo, { recursive: true, force: true });
});

test('verifyRoundDelivery: wiki 回读 hash 被改 → 红(不静默绿)', async () => {
  const repo = tempRepo();
  const mcp = fakeBoard({ mode: 'p001' });
  mcp.repoDir = repo;
  const fetchImpl = stubHttp({ gen: 1, wikiReadback: '# 被篡改的正文\n' });
  const r = await verifyRoundDelivery({
    round: 'p001', planLabel: 'P001', facts: DELIVERED, mcp, repoDir: repo,
    base: 'http://x', workspaceId: 'ws1', wikiFiles: ['docs/wiki/usage.md'],
    runId: 'runT', planMarker: '001-basic-board', log: () => {}, fetchImpl,
    wtRoot: path.join(repo, 'wt-empty'),
  });
  const w = r.checks.find(c => /usage\.md/.test(c.name));
  assert.equal(w.ok, false);
  assert.match(w.detail, /mismatch/);
  fs.rmSync(repo, { recursive: true, force: true });
});

test('p003ContextChecks: 三臂齐绿/各自变红', () => {
  const req = '# P003 可配置规则\n';
  const events = [{ type: 'turn_tool_call', tool_name: 'read_file', arguments: { path: 'docs/specs/board.md' } }];
  const good = p003ContextChecks({ p003SessionId: 's3', priorSessionIds: ['s1'], inputText: req, requestText: req, events });
  assert.equal(good.every(c => c.ok), true, JSON.stringify(good));
  const dup = p003ContextChecks({ p003SessionId: 's1', priorSessionIds: ['s1'], inputText: req, requestText: req, events });
  assert.equal(dup[0].ok, false);
  const injected = p003ContextChecks({ p003SessionId: 's3', priorSessionIds: ['s1'], inputText: req + '\n前情提要…', requestText: req, events });
  assert.equal(injected[1].ok, false);
  const noReads = p003ContextChecks({ p003SessionId: 's3', priorSessionIds: ['s1'], inputText: req, requestText: req, events: [] });
  assert.equal(noReads[2].ok, false);
  assert.match(noReads[2].detail, /读取证据/);
});
