// tests/demo/report.test.mjs — PLAN-097 T-13 V02：报告生成（合成 manifest;
// 只引用已登记事实,不依赖真实 run）。
import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { buildReport } from '../../scripts/demo/report.mjs';

test('buildReport: 轮次节/采集表/摘要矩阵/落盘 md+html', () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'e097-rep-'));
  fs.writeFileSync(path.join(dir, 'evidence-manifest.json'), JSON.stringify({
    format_version: 1,
    at: '2026-10-03T00:00:00Z',
    fingerprints: { musk_exe: { sha256: 'a'.repeat(64) }, auto_exe: { sha256: 'b'.repeat(64), version: 'auto-test' }, worktree_head: 'c'.repeat(40) },
    identity: { workspace_id: 'ws1', session_id: 's1' },
    plans: [{
      plan_id: 'PLAN-001', plan_revision: 1, run_id: 'run-1',
      phases: [{ stage: 'plan', attempt: 1, outcome: 'pass' }, { stage: 'delivered', attempt: 1, outcome: 'pass' }],
      delivery_checkpoints: { prepared: {}, landed: {} }, receipt_ref: '.autoos/x.json', reviewed_commit: 'd'.repeat(40),
    }],
    captures: [
      { capture_id: 'p001-00-requirement', source: 'host-vm', at: 't', png: 'p001-00-requirement.png', sha256: 'e'.repeat(64), bytes: 1, kind: 'live', allow_unstaged: true },
      { capture_id: 'p001-live-execute', source: 'host-vm', at: 't', png: 'p001-live-execute.png', sha256: 'f'.repeat(64), bytes: 2, kind: 'live', stage_ref: { plan_id: 'PLAN-001', stage: 'execute', attempt: 1 } },
    ],
    landing: { commit: 'abc123', receipts: [] },
  }));
  fs.writeFileSync(path.join(dir, 'run-summary.json'), JSON.stringify({
    rounds: [{ round: 'p001', ok: true, delivered: true, checks: 7, fails: 0 }],
  }));
  const r = buildReport(dir);
  const md = fs.readFileSync(r.md, 'utf8');
  assert.match(md, /## PLAN-001/);
  assert.match(md, /plan#1\(pass\) → delivered#1\(pass\)/);
  assert.match(md, /p001-live-execute/);
  assert.match(md, /\| p001 \| PASS \| delivered \| 7 \| 0 \|/);
  assert.match(md, /commit=abc123/);
  const html = fs.readFileSync(r.html, 'utf8');
  assert.match(html, /<h2>PLAN-001/);
  assert.match(html, /charset="utf-8"/);
  fs.rmSync(dir, { recursive: true, force: true });
});
