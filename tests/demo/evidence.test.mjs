// tests/demo/evidence.test.mjs — PLAN-097 T-07 V02：采集/证据失败矩阵。
// 删除图/错 hash/跨身份/截图超时/缺阶段注入全部失败；原图可读；有效流通过。
import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import crypto from 'node:crypto';
import {
  pngInfo, verifyIdentity, captureHostVm, captureTargetCanvas,
} from '../../scripts/demo/capture.mjs';
import {
  createManifest, addPlanFacts, addCapture, verifyManifest,
  writeManifest, readManifest, setLanding,
} from '../../scripts/demo/evidence.mjs';

// 最小合法 PNG（签名+IHDR+IDAT/IEND 骨架;IHDR 声明 4x3）——够 pngInfo/存盘校验
const PNG4x3 = Buffer.concat([
  Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]), // signature
  Buffer.from([0x00, 0x00, 0x00, 0x0d]),                          // IHDR len
  Buffer.from('IHDR'),
  Buffer.from([0, 0, 0, 4, 0, 0, 0, 3, 8, 6, 0, 0, 0]),           // 4x3, 8bit, RGBA
  Buffer.from([0, 0, 0, 0]),                                       // IDAT len 0 (骨架)
  Buffer.from('IDAT'),
  Buffer.from([0, 0, 0, 0]),
  Buffer.from('IEND'),
]);

function tmpDir() {
  return fs.mkdtempSync(path.join(os.tmpdir(), 'e097-ev-'));
}

const FACTS = {
  plan_id: 'PLAN-X1', plan_revision: 1, run_id: 'run-1',
  phases: [{ stage: 'execute', attempt: 1, outcome: 'pass', reviewed_commit: 'abc' }],
  delivery_checkpoints: {}, receipt_ref: null,
};

test('pngInfo: readable dims; bad signature throws（原图可读）', () => {
  assert.deepEqual(pngInfo(PNG4x3), { width: 4, height: 3 });
  assert.throws(() => pngInfo(Buffer.from('not a png file........')), /not a PNG/);
});

test('verifyIdentity: markers in both snapshots pass; missing fails（身份双验）', () => {
  assert.equal(verifyIdentity('A B C', 'C B A', ['A', 'B']), true);
  assert.throws(() => verifyIdentity('A B', 'A C', ['A', 'C']), /identity check failed/);
});

test('captureHostVm: happy path writes png + record; timeout throws（截图超时）', async () => {
  const dir = tmpDir();
  let calls = 0;
  const goodMcp = {
    stableSnapshot: async () => 'marker-A marker-B',
    snapshot: async () => 'marker-A marker-B',
    call: async (name) => {
      calls++;
      const p = path.join(dir, `raw-${calls}.png`);
      fs.writeFileSync(p, PNG4x3);
      return `Screenshot saved to: ${p}`;
    },
  };
  const cap = await captureHostVm({ mcp: goodMcp, evidenceDir: dir, captureId: 'host-1', markers: ['marker-A'] });
  assert.equal(cap.source, 'host-vm');
  assert.equal(cap.width, 4);
  assert.ok(fs.existsSync(path.join(dir, 'host-1.png')));
  assert.ok(!fs.existsSync(path.join(dir, 'raw-1.png'))); // 原图搬运不留残影

  const slowMcp = {
    stableSnapshot: async () => 'x', snapshot: async () => 'x',
    call: () => new Promise((_, rej) => setTimeout(() => rej(new Error('t')), 80)),
  };
  await assert.rejects(
    captureHostVm({ mcp: slowMcp, evidenceDir: dir, captureId: 'host-2', markers: ['x'], timeoutMs: 60 }),
    /screenshot timeout/,
  );
});

test('captureHostVm: identity drift between before/after fails（跨身份）', async () => {
  const dir = tmpDir();
  let n = 0;
  const mcp = {
    stableSnapshot: async () => 'gen-1 view', snapshot: async () => (++n > 0 ? 'gen-2 view' : 'gen-1 view'),
    call: async () => { const p = path.join(dir, 'r.png'); fs.writeFileSync(p, PNG4x3); return `Screenshot saved to: ${p}`; },
  };
  await assert.rejects(
    captureHostVm({ mcp, evidenceDir: dir, captureId: 'host-3', markers: ['gen-1'] }),
    /identity check failed/,
  );
});

test('captureTargetCanvas: generation mismatch fails before fetch frame（跨代次）', async () => {
  const dir = tmpDir();
  const base = 'http://127.0.0.1:1'; // 不会真的请求 frame——status 先拒
  const origFetch = global.fetch;
  global.fetch = async (url) => {
    if (String(url).includes('/api/canvas/status')) {
      return { json: async () => ({ generation_id: 7 }) };
    }
    throw new Error('should not reach frame');
  };
  try {
    await assert.rejects(
      captureTargetCanvas({ base, evidenceDir: dir, captureId: 'cv-1', generation: 3 }),
      /cross-identity/,
    );
  } finally { global.fetch = origFetch; }
});

test('evidence manifest: full flow passes; missing stage_ref injects fail（缺阶段注入）', () => {
  let m = createManifest({ fingerprints: { musk_exe: { sha256: 'x' } }, identity: { workspace_id: 'ws', session_id: 's1' } });
  addPlanFacts(m, FACTS);
  const cap = { capture_id: 'c1', source: 'host-vm', at: 't', png: 'c1.png', sha256: 'deadbeef', bytes: 10, kind: 'live', stage_ref: { plan_id: 'PLAN-X1', stage: 'execute', attempt: 1 } };
  assert.doesNotThrow(() => addCapture(m, cap));
  // 缺阶段注入:live 无 stage_ref 且未显式 allowUnstaged → 拒绝
  assert.throws(() => addCapture(m, { ...cap, capture_id: 'c2', stage_ref: null }), /missing stage_ref/);
  // 历史记录无 stage_ref → 拒绝（历史标识必须绑定事实）
  assert.throws(() => addCapture(m, { ...cap, capture_id: 'c3', kind: 'completed-stage-record', stage_ref: null }), /requires stage_ref/);
  // stage_ref 指向未登记事实 → 拒绝
  assert.throws(() => addCapture(m, { ...cap, capture_id: 'c4', stage_ref: { plan_id: 'PLAN-NOPE', stage: 'plan', attempt: 1 } }), /not in registered/);
  // 场景级整屏:显式 allowUnstaged 放行
  assert.doesNotThrow(() => addCapture(m, { ...cap, capture_id: 'c5', stage_ref: null, allow_unstaged: true }, { allowUnstaged: true }));
});

test('verifyManifest: deleted png / tampered sha both fail（删除图/错 hash）', () => {
  const dir = tmpDir();
  fs.writeFileSync(path.join(dir, 'c1.png'), PNG4x3);
  let m = createManifest({ fingerprints: { musk_exe: { sha256: 'x' } }, identity: { workspace_id: 'ws', session_id: 's' } });
  addPlanFacts(m, FACTS);
  const sha = crypto.createHash('sha256').update(PNG4x3).digest('hex');
  addCapture(m, { capture_id: 'c1', source: 'host-vm', at: 't', png: 'c1.png', sha256: sha, bytes: PNG4x3.length, kind: 'live', stage_ref: { plan_id: 'PLAN-X1', stage: 'execute', attempt: 1 } });
  setLanding(m, { commit: 'c0ffee', receipts: [] });
  assert.equal(verifyManifest(m, dir).ok, true);

  // 删除图 → fail
  fs.rmSync(path.join(dir, 'c1.png'));
  let v = verifyManifest(m, dir);
  assert.equal(v.ok, false);
  assert.ok(v.errors.some(e => e.includes('png missing')));

  // 错 hash → fail
  fs.writeFileSync(path.join(dir, 'c1.png'), Buffer.from('tampered'));
  v = verifyManifest(m, dir);
  assert.equal(v.ok, false);
  assert.ok(v.errors.some(e => e.includes('sha256 mismatch')));
});

test('manifest round-trip via write/read', async () => {
  const dir = tmpDir();
  fs.writeFileSync(path.join(dir, 'c1.png'), PNG4x3);
  let m = createManifest({ fingerprints: { musk_exe: { sha256: 'x' } }, identity: { workspace_id: 'ws', session_id: 's' } });
  addPlanFacts(m, FACTS);
  addCapture(m, { capture_id: 'c1', source: 'target-canvas', at: 't', png: 'c1.png', sha256: crypto.createHash('sha256').update(PNG4x3).digest('hex'), bytes: PNG4x3.length, kind: 'live', stage_ref: { plan_id: 'PLAN-X1', stage: 'execute', attempt: 1 } });
  const p = writeManifest(m, dir);
  assert.equal(readManifest(dir).format_version, 1);
  assert.ok(fs.existsSync(p));
});
