// tests/demo/wiki.test.mjs — PLAN-097 T-09 V02：Wiki 发布/回读/hash 判言
// （fetch 桩;真机 V05 属 T-12）。
import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { readAgentWiki, publishWiki, readbackWiki, wikiHashVerdict } from '../../scripts/demo/wiki.mjs';

const MD = '# FocusBoard 用法\n\n1. 添加任务…\n';

test('readAgentWiki: verbatim read; missing file = loud error', () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'e097-wiki-'));
  fs.mkdirSync(path.join(dir, 'docs', 'wiki'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'docs', 'wiki', 'usage.md'), MD);
  assert.equal(readAgentWiki(dir, 'docs/wiki/usage.md'), MD);
  assert.throws(() => readAgentWiki(dir, 'docs/wiki/nope.md'), /缺件/);
  fs.rmSync(dir, { recursive: true, force: true });
});

test('publishWiki: create on 200; falls back to update on 409 (idempotent rerun)', async () => {
  let mode = 'create';
  const calls = [];
  const fetchImpl = async (url, opts = {}) => {
    calls.push({ url: String(url), method: opts.method });
    if (String(url).includes('/pages')) {
      if (mode === 'create') return { status: 200, text: async () => '{}' };
      return { status: 409, text: async () => 'already exists' };
    }
    // PUT page/{slug}
    assert.equal(opts.method, 'PUT');
    assert.equal(JSON.parse(opts.body).content, MD, 'update 正文逐字同源');
    return { status: 200, text: async () => '{}' };
  };
  const r1 = await publishWiki({ base: 'http://x', workspaceId: 'ws1', slug: 'usage', title: '用法', content: MD, fetchImpl });
  assert.equal(r1.action, 'created');
  mode = 'exists';
  const r2 = await publishWiki({ base: 'http://x', workspaceId: 'ws1', slug: 'usage', title: '用法', content: MD, fetchImpl });
  assert.equal(r2.action, 'updated');
  assert.ok(calls.length >= 3);
});

test('readbackWiki: 404 loud; malformed loud; content hash roundtrip', async () => {
  const fetchImpl = async (url) => {
    if (String(url).includes('missing')) return { status: 404 };
    if (String(url).includes('bad')) return { status: 200, json: async () => ({ page: {} }) };
    return { status: 200, json: async () => ({ page: { content: MD } }) };
  };
  const r = await readbackWiki({ base: 'http://x', slug: 'usage', fetchImpl });
  assert.equal(r.content, MD);
  await assert.rejects(() => readbackWiki({ base: 'http://x', slug: 'missing', fetchImpl }), /404/);
  await assert.rejects(() => readbackWiki({ base: 'http://x', slug: 'bad', fetchImpl }), /malformed/);
});

test('wikiHashVerdict: match ok; mismatch loud（错 hash 不静默绿）', () => {
  assert.equal(wikiHashVerdict('aaaa', 'aaaa').ok, true);
  const v = wikiHashVerdict('aaaa', 'bbbb');
  assert.equal(v.ok, false);
  assert.match(v.reason, /mismatch/);
});
