// tests/demo/vm-session.test.mjs — PLAN-097 T-06 V02：会话决策函数矩阵。
// 端口冲突/归属判言/旧产物拒绝/端点发现与回退标记/占用解析。
import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  parseListeningPids, decidePortConflict, ownershipVerdict,
  fingerprintVerdict, parseMcpEndpoint,
} from '../../scripts/demo/vm-session.mjs';

const NETSTAT = [
  '  TCP    127.0.0.1:17841       0.0.0.0:0              LISTENING       1234',
  '  TCP    127.0.0.1:17842       0.0.0.0:0              LISTENING       5678',
  '  TCP    [::]:17841            [::]:0                 LISTENING       1234',
  '  TCP    127.0.0.1:17843       0.0.0.0:0              ESTABLISHED     9999',
  '',
].join('\r\n');

test('parseListeningPids: LISTENING-only, deduped, other states ignored', () => {
  assert.deepEqual(parseListeningPids(NETSTAT, 17841).sort(), [1234]);
  assert.deepEqual(parseListeningPids(NETSTAT, 17842), [5678]);
  assert.deepEqual(parseListeningPids(NETSTAT, 17843), []); // ESTABLISHED 不算监听
  assert.deepEqual(parseListeningPids(NETSTAT, 19999), []);
});

test('decidePortConflict: free passes; occupied refuses with explicit reason', () => {
  assert.deepEqual(decidePortConflict([]), { ok: true, pids: [], reason: 'free' });
  const v = decidePortConflict([1234]);
  assert.equal(v.ok, false);
  assert.ok(v.reason.includes('1234'));
  assert.ok(v.reason.includes('显式失败'));
});

test('ownershipVerdict: exact single-pid match owned; mismatch/extra refused', () => {
  assert.equal(ownershipVerdict([24812], 24812).owned, true);
  assert.equal(ownershipVerdict([9999], 24812).owned, false);
  assert.equal(ownershipVerdict([24812, 9999], 24812).owned, false);
  assert.equal(ownershipVerdict([], 24812).owned, false);
});

test('fingerprintVerdict: match ok; sha drift refused; missing field refused', () => {
  const rec = { musk_exe: { sha256: 'aaa' }, auto_exe: { sha256: 'bbb' } };
  assert.equal(fingerprintVerdict(rec, { musk_exe: { sha256: 'aaa' }, auto_exe: { sha256: 'bbb' } }).ok, true);
  const drifted = fingerprintVerdict(rec, { musk_exe: { sha256: 'aaa' }, auto_exe: { sha256: 'zzz' } });
  assert.equal(drifted.ok, false);
  assert.ok(drifted.reason.includes('stale artifact'));
  assert.equal(fingerprintVerdict(rec, { musk_exe: { sha256: 'aaa' } }).ok, false);
  assert.equal(fingerprintVerdict(null, {}).ok, false);
});

test('parseMcpEndpoint: endpoint + /mcp suffix; fallback flagged; absent null', () => {
  const ok = parseMcpEndpoint('boot...\nAutoUI MCP: listening on http://127.0.0.1:17782\nmore');
  assert.equal(ok.endpoint, 'http://127.0.0.1:17782/mcp');
  assert.equal(ok.fellBack, false);
  const fb = parseMcpEndpoint('AutoUI MCP: listening on http://127.0.0.1:17783\nAutoUI MCP: port 17782 busy — fell back to 17783');
  assert.equal(fb.endpoint, 'http://127.0.0.1:17783/mcp');
  assert.equal(fb.fellBack, true);
  assert.equal(parseMcpEndpoint('nothing here').endpoint, null);
});
