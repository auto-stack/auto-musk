import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { pathToFileURL } from 'node:url';
import { ROOT, ARTIFACTS, hash, writeJson } from './source.mjs';

const generated = path.join(ROOT, 'gen/front/vue/src/ext/src/front/forge_helpers.ts');
const { messageBlocks, messageDisplayBlocks, expandedMessageBlocks } = await import(pathToFileURL(generated));
const msg = { id: 'message', role: 'assistant', blocks: [
  { id: 'think-a', kind: 'thinking', state: 'done', text: 'First thought' },
  { id: 'text-a', kind: 'text', text: 'Answer' },
  { id: 'think-b', kind: 'thinking', state: 'streaming', text: 'Live thought' },
  { kind: 'tool', tc: { id: 'call-a', name: 'run_command', arguments: { command: 'echo fixture' }, result: '', status: 'gate_waiting', gate_id: 'gate-a', pending_cmd: 'echo fixture', escape_paths: ['fixture-outside'] } },
] };
const blocks = messageDisplayBlocks(msg, true);
assert.deepEqual(blocks.map(b => b.kind), ['thinking', 'text', 'thinking', 'tool']);
assert.deepEqual(blocks.filter(b => b.kind === 'thinking').map(b => b.state), ['done', 'streaming']);
assert.equal(blocks[3].tc.gate_id, 'gate-a');
assert.equal(blocks[3].tc.pending_cmd, 'echo fixture');
assert.deepEqual(blocks[3].tc.escape_paths, ['fixture-outside']);
assert.equal(blocks[3].tc.status, 'gate_waiting');
const keys = blocks.filter(b => b.tkey).map(b => b.tkey);
const moved = messageDisplayBlocks({ ...msg, blocks: [...msg.blocks].reverse() }, true);
assert.deepEqual(moved.filter(b => b.tkey).map(b => b.tkey).sort(), [...keys].sort());
const expanded = expandedMessageBlocks(msg, true, JSON.stringify([keys[0], keys[1]]), JSON.stringify([keys[2]]));
assert.equal(expanded.filter(b => b.expanded).length, 3);
assert.equal(expandedMessageBlocks({ ...msg, id: 'other' }, true, JSON.stringify(keys), JSON.stringify(keys)).filter(b => b.expanded).length, 0);
const legacy = { id: 'old', thinking: 'Old thought', content: 'Old answer', tool_calls: [msg.blocks[3].tc] };
assert.deepEqual(messageBlocks(legacy).map(b => b.kind), ['thinking', 'text', 'tool']);
assert.equal(messageBlocks(legacy)[2].tc.status, 'gate_waiting');
assert.equal(messageBlocks({ id: 'waiting', blocks: [] }).length, 0);
for (const [status, expected] of [['error', 'failed'], ['failed', 'failed'], ['running', 'running'], ['completed', 'completed'], ['gate_waiting', 'gate_waiting']]) {
  assert.equal(messageBlocks({ id: 'status', blocks: [{ kind: 'tool', tc: { ...msg.blocks[3].tc, status } }] })[0].tc.status, expected);
}

// Execute the production functions in AutoVM too. None is the normal-mode spelling
// of the existing null token; no function implementation is translated or mocked.
const source = fs.readFileSync(path.join(ROOT, 'src/front/forge_helpers.at'), 'utf8');
const probe = `${source.replace(/\bnull\b/g, 'None')}\nlet fixture = ${JSON.stringify(msg)}\nlet projected = messageDisplayBlocks(fixture, true)\nprint(projected[0].state)\nprint(projected[2].state)\nprint(projected[3].tc.gate_id)\nprint(projected[3].tc.pending_cmd)\nprint(projected[3].tc.status)\n`;
const out = path.join(ARTIFACTS, 'PLAN-077'); fs.mkdirSync(out, { recursive: true });
const probePath = path.join(out, 'projection-probe.at'); fs.writeFileSync(probePath, probe);
const result = spawnSync(process.env.AUTO_EXE ?? 'auto', [probePath], { cwd: ROOT, encoding: 'utf8', windowsHide: true });
assert.equal(result.status, 0, result.stderr);
for (const expected of ['done', 'streaming', 'gate-a', 'echo fixture', 'gate_waiting']) assert.ok(result.stdout.includes(expected), result.stdout);
writeJson(path.join(out, 'message-contract.json'), { status: 'pass', source_hash: hash(source), generated_hash: hash(fs.readFileSync(generated)), vm_stdout: result.stdout, checks: ['block state', 'gate payload', 'status normalization', 'stable reorder keys', 'message isolation', 'independent expansion', 'legacy projection', 'empty streaming'] });
console.log('message-contract: PASS (generated Vue functions + AutoVM production projection)');
