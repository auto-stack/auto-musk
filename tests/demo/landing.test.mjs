// tests/demo/landing.test.mjs — PLAN-097 T-09 V02：工程落地对账判言
// （注入清单;真实取证 collectLandingFacts 用临时仓做一遍集成验证）。
import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { execFileSync } from 'node:child_process';
import {
  linearHistoryVerdict, branchesVerdict, worktreeCleanupVerdict, worktreeListVerdict,
  landingFilesVerdict, landingVerdict, collectLandingFacts,
} from '../../scripts/demo/landing.mjs';

test('linearHistoryVerdict: empty passes; merge commits loud', () => {
  assert.equal(linearHistoryVerdict([]).ok, true);
  const v = linearHistoryVerdict(['abc123', 'def456']);
  assert.equal(v.ok, false);
  assert.match(v.reason, /merge commit/);
});

test('branchesVerdict: only default remains', () => {
  assert.equal(branchesVerdict(['master']).ok, true);
  const v = branchesVerdict(['master', 'plan-001-dev', '* master']);
  assert.equal(v.ok, false);
  assert.match(v.reason, /plan-001-dev/);
});

test('worktreeCleanupVerdict/worktreeListVerdict: run-scoped leftovers rejected', () => {
  assert.equal(worktreeCleanupVerdict(['D:/autostack/.wt/focusboard-runA-001/focusboard', 'D:/autostack/.wt/other'], 'runB').ok, true);
  const v = worktreeCleanupVerdict(['D:/autostack/.wt/focusboard-runB-001/focusboard'], 'runB');
  assert.equal(v.ok, false);
  assert.match(v.reason, /残留/);
  assert.equal(worktreeListVerdict('worktree D:/x/repo\nHEAD abc\nbranch refs/heads/master\n').ok, true);
  const two = worktreeListVerdict('worktree D:/x/repo\nworktree D:/x/.wt/fb\n');
  assert.equal(two.ok, false);
});

test('landingFilesVerdict: specs/ledger/archived/wiki 全链缺一响亮', () => {
  const good = landingFilesVerdict({
    specs: ['board.md'], ledgerPaths: ['docs/specs/index.json'],
    archived: ['001-basic-board.md'], wiki: ['usage.md'], planMarker: '001-basic-board',
  });
  assert.equal(good.ok, true, JSON.stringify(good.errors));
  const noLedger = landingFilesVerdict({ specs: ['board.md'], ledgerPaths: [], archived: ['a.md'], wiki: ['usage.md'], planMarker: 'a' });
  assert.equal(noLedger.ok, false);
  assert.ok(noLedger.errors.some(e => /ledger/.test(e)));
  const noWiki = landingFilesVerdict({ specs: ['board.md'], ledgerPaths: ['x'], archived: ['a.md'], wiki: [], planMarker: 'a' });
  assert.ok(noWiki.errors.some(e => /Wiki/.test(e)));
  const noSpecs = landingFilesVerdict({ specs: [], ledgerPaths: ['x'], archived: [], wiki: ['u.md'], planMarker: null });
  assert.ok(noSpecs.errors.some(e => /Specs/.test(e)));
});

test('landingVerdict: composes part verdicts', () => {
  const facts = {
    linearHistory: { ok: true }, branches: { ok: true }, worktreeRegistry: { ok: true },
    worktreeCleanup: { ok: false, reason: 'demo worktree 残留: x' }, files: { ok: true, errors: [] },
  };
  const v = landingVerdict(facts);
  assert.equal(v.ok, false);
  assert.match(v.errors[0], /worktreeCleanup/);
  assert.equal(landingVerdict({ linearHistory: { ok: true }, branches: { ok: true }, worktreeRegistry: { ok: true }, worktreeCleanup: { ok: true }, files: { ok: true } }).ok, true);
});

test('collectLandingFacts: real temp repo integration (linear master + files)', () => {
  const repo = fs.mkdtempSync(path.join(os.tmpdir(), 'e097-land-'));
  const run = (args, opts = {}) => execFileSync('git', ['-C', repo, ...args], { encoding: 'utf8', windowsHide: true, ...opts });
  run(['init', '-b', 'master']);
  run(['config', 'user.email', 't@local']); run(['config', 'user.name', 't']);
  fs.mkdirSync(path.join(repo, 'docs', 'specs'), { recursive: true });
  fs.mkdirSync(path.join(repo, 'docs', 'wiki'), { recursive: true });
  fs.mkdirSync(path.join(repo, 'docs', 'plans', 'archived'), { recursive: true });
  fs.writeFileSync(path.join(repo, 'docs', 'specs', 'board.md'), '# board');
  fs.writeFileSync(path.join(repo, 'docs', 'specs', 'index.json'), '{}');
  fs.writeFileSync(path.join(repo, 'docs', 'wiki', 'usage.md'), '# usage');
  fs.writeFileSync(path.join(repo, 'docs', 'plans', 'archived', '001-basic-board.md'), '# P001');
  run(['add', '-A']); run(['commit', '-m', 'p001']);
  const facts = collectLandingFacts({ repoDir: repo, wtRoot: path.join(os.tmpdir(), 'e097-nowt'), runId: 'runT', planMarker: '001-basic-board' });
  assert.equal(facts.linearHistory.ok, true, facts.linearHistory.reason);
  assert.equal(facts.branches.ok, true);
  assert.equal(facts.worktreeCleanup.ok, true);
  assert.equal(facts.files.ok, true, JSON.stringify(facts.files.errors));
  assert.equal(landingVerdict(facts).ok, true);
  fs.rmSync(repo, { recursive: true, force: true });
});
