// tests/demo/contract.test.mjs — PLAN-097 T-02 V02 子集：B 轨消费合同的
// 显式拒绝矩阵（未知版本/缺件/坏枚举/越界收据路径/坏事件形态）。
// 运行：node --test tests/demo/contract.test.mjs
import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  PLAN_EXECUTION_CONTRACT_VERSION, SUCCESS_SAMPLE, FAILURE_SAMPLE,
  validatePlanExecution, validatePlanEvent, validateRunState,
  parseRunStateResponse, discoverPlanRun,
} from '../../scripts/demo/contract.mjs';

const clone = o => JSON.parse(JSON.stringify(o));

test('frozen contract version is 1', () => {
  assert.equal(PLAN_EXECUTION_CONTRACT_VERSION, 1);
});

test('success sample (delivered shape) passes', () => {
  const r = validatePlanExecution(SUCCESS_SAMPLE.plan_execution);
  assert.deepEqual(r, { ok: true, errors: [] });
  const rs = validateRunState({ ...SUCCESS_SAMPLE });
  assert.equal(rs.ok, true);
});

test('failure sample (L3 blocker shape) passes', () => {
  const r = validatePlanExecution(FAILURE_SAMPLE.plan_execution);
  assert.deepEqual(r, { ok: true, errors: [] });
});

test('unknown contract_version is explicitly rejected', () => {
  const pe = clone(SUCCESS_SAMPLE.plan_execution);
  pe.contract_version = 2;
  const r = validatePlanExecution(pe);
  assert.equal(r.ok, false);
  assert.ok(r.errors.some(e => e.includes('unknown contract_version')));
});

test('missing required fields are rejected (not defaulted to success)', () => {
  for (const k of ['plan_id', 'plan_path', 'phase']) {
    const pe = clone(SUCCESS_SAMPLE.plan_execution);
    delete pe[k];
    const r = validatePlanExecution(pe);
    assert.equal(r.ok, false, `${k} missing must fail`);
    assert.ok(r.errors.some(e => e.includes(k)));
  }
});

test('bad phase/outcome enums rejected', () => {
  const a = clone(SUCCESS_SAMPLE.plan_execution); a.phase = 'running';
  assert.ok(!validatePlanExecution(a).ok);
  const b = clone(SUCCESS_SAMPLE.plan_execution); b.outcome = 'green';
  assert.ok(!validatePlanExecution(b).ok);
});

test('delivered with missing checkpoint is rejected (缺件完成不绿)', () => {
  const pe = clone(SUCCESS_SAMPLE.plan_execution);
  delete pe.delivery_checkpoints.cleaned;
  const r = validatePlanExecution(pe);
  assert.equal(r.ok, false);
  assert.ok(r.errors.some(e => e.includes('cleaned') && e.includes('delivered')));
});

test('unknown checkpoint key rejected', () => {
  const pe = clone(SUCCESS_SAMPLE.plan_execution);
  pe.delivery_checkpoints.extra = {};
  assert.ok(!validatePlanExecution(pe).ok);
});

test('absolute receipt_ref rejected (workspace-relative contract)', () => {
  const pe = clone(SUCCESS_SAMPLE.plan_execution);
  pe.receipt_ref = 'D:/autostack/auto-musk/.autoos/plan-delivery/PLAN-XXX.json';
  const r = validatePlanExecution(pe);
  assert.equal(r.ok, false);
  assert.ok(r.errors.some(e => e.includes('workspace-relative')));
});

test('valid plan_stage_facts event passes; missing outcome rejected', () => {
  const ev = { type: 'plan_stage_facts', timestamp: 1, plan_id: 'PLAN-XXX', stage: 'execute', attempt: 1, outcome: 'pass', repair_count: 0, repair_limit: 3 };
  assert.deepEqual(validatePlanEvent(ev), { ok: true, errors: [] });
  const bad = clone(ev); delete bad.outcome;
  assert.ok(!validatePlanEvent(bad).ok);
  const badStage = clone(ev); badStage.stage = 'coding';
  assert.ok(!validatePlanEvent(badStage).ok);
});

test('run-level: plan_stage_facts in events window validated; non-array events rejected', () => {
  const run = { run_id: 'r1', status: 'running', plan_execution: null, events: [{ type: 'step_started' }, { type: 'plan_stage_facts', plan_id: 'PLAN-XXX', stage: 'review', outcome: 'needs_fix', attempt: 2, repair_count: 1 }] };
  assert.equal(validateRunState(run).ok, true);
  assert.ok(!validateRunState({ run_id: 'r1', events: 'nope' }).ok);
  // T-12 live 修订:事件违约降级为警告（不拒收整个 RunState;权威面是
  // plan_execution），载荷嵌套在 facts 键（RunEvent serde 形态）
  const badEv = clone(run); badEv.events[1] = { type: 'plan_stage_facts', facts: { plan_id: '', stage: 'review', outcome: 'pass' } };
  const v = validateRunState(badEv);
  assert.ok(v.ok, 'bad event no longer rejects state');
  assert.ok(v.warnings.some(w => /events\[1\]/.test(w)), `warning recorded: ${JSON.stringify(v.warnings)}`);
});

test('parseRunStateResponse: non-JSON throws; null (404) throws; valid passes', () => {
  assert.throws(() => parseRunStateResponse('<html>502</html>'));
  assert.throws(() => parseRunStateResponse('null'));
  assert.throws(() => parseRunStateResponse("run 'x' not found"), /run not found/);
  const ok = parseRunStateResponse(JSON.stringify({ ...SUCCESS_SAMPLE }));
  assert.equal(ok.plan_execution.phase, 'delivered');
  const bad = clone(SUCCESS_SAMPLE); bad.plan_execution.contract_version = 9;
  assert.throws(() => parseRunStateResponse(JSON.stringify({ ...bad })), /unknown contract_version/);
});

test('discoverPlanRun: skips failed, fetches details, returns first with plan_execution', async () => {
  const summaries = [{ run_id: 'a', status: 'failed' }, { run_id: 'b', status: 'running' }, { run_id: 'c', status: 'running' }];
  const fetched = [];
  const detail = await discoverPlanRun(summaries, async rid => {
    fetched.push(rid);
    if (rid === 'b') return { run_id: 'b', status: 'running' };
    return { run_id: rid, status: 'running', plan_execution: { contract_version: 1, plan_id: 'P', plan_path: 'x.md', phase: 'plan' } };
  });
  assert.equal(detail?.run_id, 'c');
  assert.deepEqual(fetched, ['b', 'c']);
  assert.equal(await discoverPlanRun([], async () => { throw new Error('x'); }), null);
});

test('continuation-stage event degrades to warning, not rejection (T-12 live drift)', () => {
  const run = {
    run_id: 'r1', status: 'running',
    plan_execution: { contract_version: 1, plan_id: 'PLAN-001', plan_path: 'docs/plans/x.md', phase: 'execute', outcome: null },
    events: [
      { type: 'plan_stage_facts', facts: { plan_id: 'PLAN-001', stage: 'execute', outcome: 'continuation' } },
    ],
  };
  const parsed = parseRunStateResponse(JSON.stringify(run));
  assert.equal(parsed.status, 'running', 'state accepted');
  assert.ok((parsed._contract_warnings ?? []).some(w => /continuation|not in/.test(w)), 'warning recorded');
  // 权威面枚举仍严格:plan_execution.outcome 非法仍响亮拒
  const bad = { ...run, plan_execution: { ...run.plan_execution, outcome: 'continuation' } };
  assert.throws(() => parseRunStateResponse(JSON.stringify(bad)), /contract validation/);
});
