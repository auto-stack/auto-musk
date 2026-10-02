// tests/demo/rounds.test.mjs — PLAN-097 T-09 V02：三轮行为断言流在**合规
// 假应用**上全绿（驱动/判言接线正确）,在**缺陷变体**上响亮变红（拒绝缺失/
// 降限丢任务/持久化丢失）。真机 V05 属 T-12。
import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { p001Checks, p002Checks, p003Checks, runChecks } from '../../scripts/demo/assertions.mjs';
import { fakeBoard } from './helpers/fake-board.mjs';

const runAll = checks => runChecks(checks, { log: () => {} });
const tmp = p => fs.mkdtempSync(path.join(os.tmpdir(), p));
const failing = r => JSON.stringify(r.checks.filter(c => !c.ok), null, 1);

test('P001 流在合规应用上全绿(add/rename/toggle/计数/delete/跨启+磁盘)', async () => {
  const dir = tmp('e097-p001-');
  const mcp = fakeBoard({ mode: 'p001' });
  mcp.repoDir = dir;
  const r = await runAll(p001Checks({ mcp, repoDir: dir }));
  assert.equal(r.ok, true, failing(r));
  assert.ok(fs.existsSync(path.join(dir, 'src', 'front', 'data', 'board.json')), '磁盘持久化文件存在');
  fs.rmSync(dir, { recursive: true, force: true });
});

test('P001 流在缺陷变体上响亮红(重启丢任务)', async () => {
  const dir = tmp('e097-p001bad-');
  const mcp = fakeBoard({ mode: 'p001', defect: 'lose-on-restart' });
  mcp.repoDir = dir;
  const r = await runAll(p001Checks({ mcp, repoDir: dir }));
  const cross = r.checks.find(c => /跨启/.test(c.name));
  assert.equal(r.ok, false);
  assert.equal(cross.ok, false);
  assert.match(cross.detail, /丢失|不可见|disk/);
  fs.rmSync(dir, { recursive: true, force: true });
});

test('P002 流:合规上限全绿;不限流的应用在第4项检查上红', async () => {
  const dir = tmp('e097-p002-');
  const mcp = fakeBoard({ mode: 'p002' });
  mcp.repoDir = dir;
  const good = await runAll(p002Checks({ mcp, repoDir: dir }));
  assert.equal(good.ok, true, failing(good));
  fs.rmSync(dir, { recursive: true, force: true });

  const dir2 = tmp('e097-p002bad-');
  const bad = fakeBoard({ mode: 'p002' });
  bad.repoDir = dir2;
  bad.state.limit = Infinity; // 缺陷:不限流
  const r2 = await runAll(p002Checks({ mcp: bad, repoDir: dir2 }));
  const rej = r2.checks.find(c => /被拒/.test(c.name));
  assert.equal(rej.ok, false, '不限流应用必须红');
  fs.rmSync(dir2, { recursive: true, force: true });
});

test('P003 流:可配置上限合规全绿(无效拒绝/降限保留/跨启/回归)', async () => {
  const dir = tmp('e097-p003-');
  const mcp = fakeBoard({ mode: 'p003' });
  mcp.repoDir = dir;
  const setup = await runChecks(p002Checks({ mcp, repoDir: dir }).slice(0, 1), { log: () => {} });
  assert.equal(setup.ok, true, failing(setup));
  const r = await runAll(p003Checks({ mcp, repoDir: dir }));
  assert.equal(r.ok, true, failing(r));
  fs.rmSync(dir, { recursive: true, force: true });
});

test('P003 流:降限丢任务的应用红(需求 003§3 保留原列)', async () => {
  const dir = tmp('e097-p003bad-');
  const mcp = fakeBoard({ mode: 'p003', defect: 'shrink-drops' });
  mcp.repoDir = dir;
  await runChecks(p002Checks({ mcp, repoDir: dir }).slice(0, 1), { log: () => {} });
  const r = await runAll(p003Checks({ mcp, repoDir: dir }));
  const shrink = r.checks.find(c => /降限/.test(c.name));
  assert.equal(shrink.ok, false, '降限丢任务必须红');
  assert.match(shrink.detail, /消失|移出|doing/);
  fs.rmSync(dir, { recursive: true, force: true });
});
