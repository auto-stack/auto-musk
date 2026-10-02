// tests/demo/focusboard.test.mjs — PLAN-097 T-08 V02：bootstrap 白名单审计
// 与轮次停止条件分类。
import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { BOOTSTRAP_WHITELIST, bootstrapViolations, roundStopVerdict, bootstrapRepo } from '../../scripts/demo/focusboard.mjs';

test('bootstrapViolations: whitelist passes; business file/extra dir violates', () => {
  assert.deepEqual(bootstrapViolations(BOOTSTRAP_WHITELIST), []);
  const v = bootstrapViolations([...BOOTSTRAP_WHITELIST, 'src/front/app.at', 'docs/specs/board.md', 'tests/board.test.mjs']);
  assert.deepEqual(v, ['src/front/app.at', 'docs/specs/board.md', 'tests/board.test.mjs']);
});

test('roundStopVerdict: delivered completes; failed/canceled stop loudly', () => {
  assert.deepEqual(
    roundStopVerdict({ status: 'completed', plan_execution: { phase: 'delivered', outcome: 'pass' } }),
    { stop: true, terminal: true, reason: 'delivered' },
  );
  assert.equal(roundStopVerdict({ status: 'failed', plan_execution: { phase: 'document', outcome: 'blocked' } }).stop, true);
  assert.equal(roundStopVerdict({ status: 'canceled', plan_execution: {} }).stop, true);
  // failed 但无 delivered 收据 = 停且非 delivered(reason 不含 delivered)
  assert.ok(!roundStopVerdict({ status: 'failed', plan_execution: {} }).reason.includes('delivered'));
});

test('roundStopVerdict: needs_replan/blocked/repair-cap stop without nudge', () => {
  assert.equal(roundStopVerdict({ status: 'running', plan_execution: { phase: 'execute', outcome: 'needs_replan' } }).reason, 'needs_replan（授权边界,不 nudge）');
  assert.equal(roundStopVerdict({ status: 'running', plan_execution: { phase: 'review', outcome: 'blocked', blocker: 'X' } }).stop, true);
  assert.equal(roundStopVerdict({ status: 'running', plan_execution: { phase: 'execute', repair_count: 3, repair_limit: 3 } }).reason, 'repair cap reached (3/3)');
  // 未达封顶/无 stop 事实 → 继续
  assert.equal(roundStopVerdict({ status: 'running', plan_execution: { phase: 'execute', attempt: 2, repair_count: 1, repair_limit: 3 } }).stop, false);
  assert.equal(roundStopVerdict({ status: 'running' }).stop, false);
});

test('bootstrapRepo: whitelisted files only, git initialized, business-free', () => {
  const src = path.resolve(new URL('.', import.meta.url).pathname.replace(/^\/(\w:)/, '$1'), '../../examples/focusboard-demo');
  const repo = path.join(os.tmpdir(), `e097-boot-${Date.now()}`);
  const r = bootstrapRepo({ repoDir: repo, sourceDir: src });
  assert.deepEqual(r.violations, []);
  for (const f of BOOTSTRAP_WHITELIST) assert.ok(fs.existsSync(path.join(repo, f)), f);
  // 空仓断言:无业务代码/Specs/测试
  assert.ok(!fs.existsSync(path.join(repo, 'src')));
  assert.ok(!fs.existsSync(path.join(repo, 'docs/specs')));
  assert.ok(!fs.existsSync(path.join(repo, 'tests')));
  // git: 初始提交存在且工作树干净
  const log = fs.existsSync(path.join(repo, '.git'));
  assert.ok(log);
  fs.rmSync(repo, { recursive: true, force: true });
});
