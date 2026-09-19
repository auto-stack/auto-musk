#!/usr/bin/env node
// PLAN-074 gallery inventory and deterministic gate.
// This command deliberately separates source/catalog checks from a launched UI:
// a missing screenshot, MCP endpoint, or browser is reported as missing evidence.

import fs from 'node:fs';
import path from 'node:path';
import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { ROOT, GALLERY, DATA, ARTIFACTS, readJson, writeJson, inventory, portVariants, triageFor, checkCatalog, effectiveCases, hash, slash } from './ui-parity/source.mjs';
import { materialize, verifyMaterialized } from './ui-parity/materialize.mjs';

const args = process.argv.slice(2);
const command = args[0] ?? 'check';
const option = name => { const i = args.indexOf(name); return i >= 0 ? args[i + 1] : undefined; };
const has = name => args.includes(name);
const catalogPath = path.join(DATA, 'cases.json');
const catalog = readJson(catalogPath);
const receiptDir = path.join(ARTIFACTS, catalog.plan);

function now() { return new Date().toISOString(); }
function logReceipt(data) {
  fs.mkdirSync(receiptDir, { recursive: true });
  const p = path.join(receiptDir, `${data.caseId ?? 'catalog'}-${data.mode ?? 'check'}.json`);
  writeJson(p, data); return p;
}
function printIssues(issues) {
  for (const issue of issues) console.error(`❌ ${issue}`);
  if (issues.length) console.error(`ui-parity: ${issues.length} issue(s)`);
}
function catalogCheck() {
  const issues = checkCatalog(catalog);
  if (!fs.existsSync(DATA) || !fs.existsSync(path.join(DATA, 'fixtures'))) issues.push('Fixture directory missing');
  printIssues(issues);
  return issues;
}

function list() {
  const units = inventory(), cases = effectiveCases(catalog), ports = portVariants();
  console.log(JSON.stringify({ schema_version: 1, plan: catalog.plan,
    units: units.map(u => ({ id: u.id, source: u.source, reachable: u.reachable,
      platforms: u.platforms, consumerPath: u.consumerPath, sourceHash: u.sourceHash })),
    ports,
    cases: cases.map(c => ({ id: c.id, unit: c.unit, mode: c.mode, state: c.state,
      owner: c.owner, plan: c.plan })) }, null, 2));
}

function prepare() {
  const caseId = option('--case') ?? 'chat-message-pair';
  const issues = catalogCheck(); if (issues.length) process.exitCode = 1;
  const receipt = materialize(caseId);
  const p = logReceipt({ plan: catalog.plan, mode: 'prepare', caseId, at: now(), ...receipt });
  console.log(`prepared ${caseId}; receipt=${slash(path.relative(ROOT, p))}`);
}

async function waitFor(url, timeoutMs = 20000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    try { const r = await fetch(url); if (r.status < 500) return r.status; } catch (_) { /* retry */ }
    await new Promise(r => setTimeout(r, 250));
  }
  throw new Error(`Timed out waiting for ${url}`);
}
function child(command, childArgs, cwd, env) {
  const p = spawn(command, childArgs, { cwd, env: { ...process.env, ...env }, windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'] });
  let out = '', err = '';
  p.stdout.on('data', b => { out += b; }); p.stderr.on('data', b => { err += b; });
  return { p, output: () => ({ stdout: out, stderr: err }) };
}
async function runMode(mode, caseId) {
  const mcpPort = Number(option('--port') ?? (mode === 'vm' ? 17476 : 17477));
  const frontPort = Number(process.env.AUTO_GALLERY_FRONT_PORT ?? 17474);
  const backPort = Number(process.env.AUTO_GALLERY_BACK_PORT ?? 17475);
  const render = mode === 'vm' ? 'vm' : 'vue';
  const executable = process.env.AUTO_EXE ?? 'auto';
  const env = { AUTOUI_MCP_PORT: String(mcpPort), AUTO_PARITY_CASE: caseId };
  const c = child(executable, ['run', '--render', render, '--port', String(frontPort), '--back-port', String(backPort)], GALLERY, env);
  const timeoutMs = Number(option('--timeout-ms') ?? 20000);
  const started = Date.now(); let status = 'missing'; let endpoint = ''; let snapshotBody = '';
  try {
    if (mode === 'vm') {
      endpoint = `http://127.0.0.1:${mcpPort}/mcp`;
      await waitFor(endpoint, timeoutMs);
      const payload = JSON.stringify({ jsonrpc: '2.0', id: 1, method: 'tools/call', params: { name: 'autoui_snapshot', arguments: { mode: 'rendered' } } });
      for (let i = 0; i < 20; i++) {
        snapshotBody = await fetch(endpoint, { method: 'POST', headers: { 'content-type': 'application/json' }, body: payload }).then(r => r.text());
        if (!snapshotBody.includes('No UI available yet')) break;
        await new Promise(r => setTimeout(r, 250));
      }
      status = snapshotBody.includes('Instance 1') && snapshotBody.includes('Instance 2') ? 'snapshot-ok' : 'snapshot-missing-needle';
    } else {
      endpoint = `http://127.0.0.1:${frontPort}`;
      await waitFor(endpoint, timeoutMs);
      status = 'http-ok';
    }
  } catch (e) { status = `missing:${e.message}`; }
  const logs = c.output();
  const combined = `${logs.stdout}\n${logs.stderr}`;
  if (status.startsWith('missing:') && /Undefined symbol|link failed|handler synthesis failed|DynamicComponent init failed|VM UI error/i.test(combined)) {
    status = 'startup-failed';
  }
  // The CLI can spawn a backend/frontend pair. Kill only the process tree
  // started by this runner; never scan or terminate a port range.
  if (process.platform === 'win32') spawn('taskkill', ['/pid', String(c.p.pid), '/T', '/F'], { windowsHide: true, stdio: 'ignore' });
  else c.p.kill('SIGTERM');
  return { plan: catalog.plan, caseId, mode, at: now(), duration_ms: Date.now() - started,
    endpoint, status, stdout_sha256: hash(logs.stdout), stderr_sha256: hash(logs.stderr),
    stdout_tail: logs.stdout.slice(-4000), stderr_tail: logs.stderr.slice(-4000), snapshot_tail: snapshotBody.slice(-4000),
    evidence: status === 'snapshot-ok' || status === 'http-ok' ? 'runtime-smoke' : 'missing-runtime-evidence' };
}
async function run() {
  const issues = catalogCheck(); if (issues.length) { process.exitCode = 1; return; }
  const caseId = option('--case') ?? 'chat-message-pair';
  if (!effectiveCases(catalog).some(c => c.id === caseId)) throw new Error(`Unknown case: ${caseId}`);
  prepare();
  const modes = option('--mode') === 'vue' ? ['vue'] : option('--mode') === 'vm' ? ['vm'] : ['vue', 'vm'];
  for (const mode of modes) { const receipt = await runMode(mode, caseId); const p = logReceipt(receipt); console.log(`${mode}: ${receipt.status}; receipt=${slash(path.relative(ROOT, p))}`); if (receipt.evidence === 'missing-runtime-evidence') process.exitCode = 1; }
}
function report() {
  const issues = catalogCheck(), units = inventory(), cases = effectiveCases(catalog);
  const receipts = fs.existsSync(receiptDir) ? fs.readdirSync(receiptDir).filter(x => x.endsWith('.json') && x !== 'report.json').map(x => readJson(path.join(receiptDir, x))) : [];
  const report = { plan: catalog.plan, generated_at: now(), source_root: ROOT,
    units, ports: portVariants(), cases, issues, receipts, policy: { missingEvidenceIsFailure: true, baselineIsNotPass: true } };
  const out = path.join(ROOT, 'docs/reports/ui-parity/074-baseline.md'); fs.mkdirSync(path.dirname(out), { recursive: true });
  const lines = [`# PLAN-074 Gallery baseline`, '', `Generated: ${report.generated_at}`, '', `Units: ${units.length} (${units.filter(u => u.reachable).length} reachable)`, `Cases: ${cases.length}`, '', '## Inventory', '', '| Unit | Source | Platforms | Reachability | Consumer path | Triage |', '|---|---|---|---|---|---|'];
  for (const u of units) {
    const triage = triageFor(u).owner;
    lines.push(`| \`${u.id}\` | \`${u.source}\` | ${u.platforms.join('/')} | ${u.reachable ? 'reachable' : 'unreachable'} | ${u.consumerPath.length ? u.consumerPath.join(' → ') : '—'} | ${triage} |`);
  }
  lines.push('', '## Port variants', '', '| Port | VM source | Vue source | Platforms |', '|---|---|---|---|');
  for (const p of report.ports) lines.push(`| \`${p.id}\` | ${p.vm ? `\`${p.vm}\`` : '—'} | ${p.vue ? `\`${p.vue}\`` : '—'} | ${p.platforms.join('/')} |`);
  lines.push('', '## Cases', '', '| Case | Unit | State | Owner | Mode |', '|---|---|---|---|---|');
  for (const c of cases) lines.push(`| \`${c.id}\` | \`${c.unit}\` | ${c.state} | ${c.owner} | ${c.mode} |`);
  lines.push('', '## Runtime evidence', '');
  for (const r of receipts) lines.push(`- ${r.mode ?? 'unknown'} ${r.caseId ?? ''}: **${r.status ?? 'prepared'}** (${r.evidence ?? 'source'})`);
  lines.push('', '## Rules', '', '- This is a current-state inventory, not a pass baseline.', '- Missing runtime or screenshot evidence remains a failure.', '- Differences are assigned to PLAN-075–079 or the responsible dependency.', '', '## Known blockers', '', '- VM ChatMessage mounting currently stops before snapshot: the copied production `forge_helpers.at` fails VM handler synthesis at `let has_think = false` followed by reassignment, leaving `forge_helpers.messageBlocks` undefined. This is evidence for the responsible upstream VM/compiler fix; the gallery does not rewrite the production renderer.', '- Vue generation reaches project scaffolding/component generation; the local dependency installation step may require the normal network/cache and is not treated as visual evidence.');
  fs.writeFileSync(out, lines.join('\n') + '\n');
  const evidenceOut = path.join(ROOT, 'docs/reports/ui-parity/074-evidence.md');
  const evidence = [
    '# PLAN-074 evidence ledger', '',
    `Generated: ${report.generated_at}`, '',
    '## Static gates', '',
    `- node scripts/ui-parity.mjs check: **${issues.length ? 'FAIL' : 'PASS'}** (${units.length} declarations; ${cases.length} effective cases).`,
    '- Source inventory rejects symlinks and records a SHA-256 hash for every production `.at` file.',
    '- Fixture materialization copies production source byte-for-byte and records adapter hashes in materialized.json.', '',
    '## Runtime gates', '',
    ...receipts.filter(r => r.mode !== 'prepare').map(r => `- ${r.mode} / ${r.caseId}: **${r.status ?? 'missing'}**; evidence=${r.evidence ?? 'none'}; stdout=${r.stdout_sha256 ?? 'n/a'}; stderr=${r.stderr_sha256 ?? 'n/a'}.`),
    '- No screenshot or layout evidence is recorded until both renderers produce a stable gallery surface.', '',
    '## Ownership', '',
    '- VM handler/codegen and AutoUI MCP runtime failures: auto-lang / the VM responsibility in the next parity plan.',
    '- Markdown/AutoDown rendering and editor behavior: PLAN-076 via autodown-engine.',
    '- Think/tool/gate message behavior: PLAN-077.',
    '- Default style contract: PLAN-075.',
    '- App shell and release gate aggregation: PLAN-078/079.'
  ];
  fs.writeFileSync(evidenceOut, evidence.join('\n') + '\n');
  const json = path.join(receiptDir, 'report.json'); writeJson(json, report);
  printIssues(issues); console.log(`report=${slash(path.relative(ROOT, out))}`); if (issues.length) process.exitCode = 1;
}
try {
  if (command === 'list') list();
  else if (command === 'check') { const issues = catalogCheck(); if (!issues.length) console.log(`ui-parity: catalog PASS (${inventory().length} declarations, ${effectiveCases(catalog).length} cases)`); }
  else if (command === 'prepare') prepare();
  else if (command === 'run') await run();
  else if (command === 'report') report();
  else throw new Error(`Usage: node scripts/ui-parity.mjs list|check|prepare|run|report`);
} catch (e) { console.error(`ui-parity: ${e.stack ?? e}`); process.exitCode = 1; }
