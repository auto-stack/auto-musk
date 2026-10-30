// Two real service processes, one source directory: no lost CAS updates.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';

const artifacts = process.env.SPEC_UI_ARTIFACTS ?? 'D:/autostack/plan-artifacts/plan-106';
const binary = process.env.MUSK_EXE ?? path.join(artifacts, 'ui-server.exe');
const workspace = path.join(artifacts, 'process-workspace');
fs.mkdirSync(path.join(workspace, 'docs/specs'), { recursive: true });
fs.writeFileSync(path.join(workspace, 'docs/specs/shared.md'), '# Original\n');
const services = [];
const roots = ['http://127.0.0.1:18581', 'http://127.0.0.1:18582'];
const pause = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
try {
  for (let i = 0; i < 2; i++) {
    const child = spawn(binary, ['serve', '--addr', `127.0.0.1:${18581+i}`, '--workdir', workspace], {
      windowsHide: true,
      env: { ...process.env, MUSK_CONFIG_DIR: path.join(artifacts, `process-config-${i}`), MUSK_SERVE_PORT: String(18581+i) },
      stdio: ['ignore', 'pipe', 'pipe'],
    });
    services.push(child);
    let output = '';
    child.stdout.on('data', (b) => { output += b; });
    child.stderr.on('data', (b) => { output += b; });
    let ready = false;
    for (let n = 0; n < 100; n++) {
      if (child.exitCode !== null) throw new Error(`Backend exited: ${output}`);
      try { ready = (await fetch(roots[i] + '/api/specs/index')).ok; } catch {}
      if (ready) break;
      await pause(200);
    }
    assert.ok(ready, output);
  }
  const initial = await (await fetch(roots[0] + '/api/specs/document/shared.md')).json();
  const save = (origin, file, body) => fetch(`${origin}/api/specs/document/${file}`, { method: 'PUT', headers: { 'content-type': 'application/json' }, body: JSON.stringify(body) });
  const results = await Promise.all(roots.map((origin, i) => save(origin, 'shared.md', { content: `# Writer ${i}\n`, expected_hash: initial.hash })));
  assert.deepEqual(results.map((r) => r.status).sort(), [200, 409]);
  for (let i=0;i<2;i++) fs.rmSync(path.join(workspace, `docs/specs/independent-${i}.md`), {force:true});
  const independent = await Promise.all(roots.map((origin, i) => save(origin, `independent-${i}.md`, { content: `# Independent ${i}\n`, expected_hash: null })));
  assert.deepEqual(independent.map((r) => r.status), [200, 200]);
  const read = await Promise.all(roots.map((origin) => fetch(origin + '/api/specs/index').then((r) => r.json())));
  assert.equal(read[0].fresh, true);
  assert.equal(read[0].source_hash, read[1].source_hash);
  assert.equal(fs.existsSync(path.join(workspace, '.autoos/specs.json')), false);
  const receipt = { sameFile: results.map((r) => r.status), independentFiles: independent.map((r) => r.status), sourceHash: read[0].source_hash, processIds: services.map((p) => p.pid), success: true };
  fs.writeFileSync(path.join(artifacts, 'http-process-receipt.json'), JSON.stringify(receipt, null, 2));
  console.log(JSON.stringify(receipt));
} finally {
  for (const child of services) {
    if (child.exitCode === null) {
      child.kill();
      await new Promise((resolve) => child.once('exit', resolve));
    }
  }
}
