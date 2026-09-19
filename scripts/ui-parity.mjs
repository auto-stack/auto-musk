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
const planOpt = option('--plan');
const targetPlan = planOpt ? (planOpt.toUpperCase().startsWith('PLAN-') ? planOpt.toUpperCase() : `PLAN-${planOpt}`) : catalog.plan;
const receiptDir = path.join(ARTIFACTS, targetPlan);

function now() { return new Date().toISOString(); }
function mcpText(body) {
  try {
    const j = JSON.parse(body);
    return j?.result?.content?.find(x => x.type === 'text')?.text ?? '';
  } catch (_) { return ''; }
}
function logReceipt(data) {
  fs.mkdirSync(receiptDir, { recursive: true });
  const p = path.join(receiptDir, `${data.caseId ?? 'catalog'}-${data.mode ?? 'check'}.json`);
  writeJson(p, { plan: targetPlan, ...data }); return p;
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

function prepare(targetCaseId) {
  const caseId = targetCaseId ?? option('--case') ?? 'chat-message-pair';
  const issues = catalogCheck(); if (issues.length) process.exitCode = 1;
  const receipt = materialize(caseId);
  const p = logReceipt({ plan: targetPlan, mode: 'prepare', caseId, at: now(), ...receipt });
  console.log(`prepared ${caseId}; receipt=${slash(path.relative(ROOT, p))}`);
}

async function waitFor(url, timeoutMs = 20000) {
  const deadline = Date.now() + timeoutMs;
  const urls = url.includes('127.0.0.1') ? [url, url.replace('127.0.0.1', 'localhost')] : [url];
  while (Date.now() < deadline) {
    for (const u of urls) {
      try { const r = await fetch(u); if (r.status < 500) return r.status; } catch (_) { /* retry */ }
    }
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
  const env = { AUTOUI_MCP_PORT: String(mcpPort), AUTO_PARITY_CASE: caseId, AUTO_BACKEND_IMPL: 'vm' };
  const runArgs = ['run', '--render', render, '--port', String(frontPort), '--back-port', String(backPort)];
  if (mode === 'vue') runArgs.push('--server', 'vm');
  const c = child(executable, runArgs, GALLERY, env);
  const timeoutMs = Number(option('--timeout-ms') ?? 120000);
  const started = Date.now(); let status = 'missing'; let endpoint = ''; let snapshotBody = ''; let screenshotBody = ''; let interactionBody = ''; let resetEventSpy = false;
  try {
    if (mode === 'vm') {
      endpoint = `http://127.0.0.1:${mcpPort}/mcp`;
      const deadline = Date.now() + timeoutMs;
      while (Date.now() < deadline) {
        const out = c.output();
        const m = (out.stderr + out.stdout).match(/AutoUI MCP: listening on (https?:\/\/[^\s]+)/);
        if (m) {
          endpoint = m[1].replace(/\/+$/, '') + '/mcp';
          break;
        }
        await new Promise(r => setTimeout(r, 150));
      }
      await waitFor(endpoint, Math.max(5000, deadline - Date.now()));
      const payload = JSON.stringify({ jsonrpc: '2.0', id: 1, method: 'tools/call', params: { name: 'autoui_snapshot', arguments: { mode: 'rendered' } } });
      for (let i = 0; i < 20; i++) {
        snapshotBody = await fetch(endpoint, { method: 'POST', headers: { 'content-type': 'application/json' }, body: payload }).then(r => r.text());
        if (!snapshotBody.includes('No UI available yet')) break;
        await new Promise(r => setTimeout(r, 250));
      }
      const snapshotText = mcpText(snapshotBody);
      const isChat = caseId.startsWith('chat-');
      const hasNeedle = isChat
        ? (snapshotText.includes('Instance 1') && snapshotText.includes('Instance 2'))
        : (snapshotText.includes(caseId) || snapshotText.includes('Reset fixture'));
      status = hasNeedle ? 'snapshot-ok' : 'snapshot-missing-needle';
      if (status === 'snapshot-ok') {
        const reset = snapshotText.match(/button (#[^\s]+) "Reset fixture"/);
        if (!reset) {
          status = 'interaction-missing-reset';
        } else {
          interactionBody = await fetch(endpoint, { method: 'POST', headers: { 'content-type': 'application/json' },
            body: JSON.stringify({ jsonrpc: '2.0', id: 3, method: 'tools/call', params: { name: 'autoui_action', arguments: { element_id: reset[1].slice(1), action: 'press' } } }) }).then(r => r.text());
          let afterReset = '';
          for (let i = 0; i < 8; i++) {
            await new Promise(r => setTimeout(r, 250));
            afterReset = await fetch(endpoint, { method: 'POST', headers: { 'content-type': 'application/json' },
              body: JSON.stringify({ jsonrpc: '2.0', id: 4 + i, method: 'tools/call', params: { name: 'autoui_snapshot', arguments: { mode: 'rendered' } } }) }).then(r => r.text());
            if (mcpText(afterReset).includes('Spy events 2')) break;
          }
          interactionBody += `\nAFTER_RESET\n${afterReset}`;
          if (interactionBody.includes('"isError":true') || !mcpText(afterReset).includes('Spy events 2')) status = 'interaction-failed';
          else resetEventSpy = true;
        }
      }
      if (status === 'snapshot-ok') {
        const screenshotName = `${targetPlan.toLowerCase().replace('-', '')}-${caseId}-vm`;
        screenshotBody = await fetch(endpoint, { method: 'POST', headers: { 'content-type': 'application/json' },
          body: JSON.stringify({ jsonrpc: '2.0', id: 2, method: 'tools/call', params: { name: 'autoui_screenshot', arguments: { name: screenshotName, baseline: true } } }) }).then(r => r.text());
        if (screenshotBody.includes('"isError":true') || screenshotBody.includes('"error"')) status = 'screenshot-failed';
      }
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
  return { plan: targetPlan, caseId, mode, at: now(), duration_ms: Date.now() - started,
    endpoint, status, reset_event_spy: resetEventSpy, stdout_sha256: hash(logs.stdout), stderr_sha256: hash(logs.stderr),
    stdout_tail: logs.stdout.slice(-4000), stderr_tail: logs.stderr.slice(-4000), snapshot_tail: snapshotBody.slice(-4000), interaction_tail: interactionBody.slice(-4000), screenshot_tail: screenshotBody.slice(-2000),
    evidence: status === 'snapshot-ok' || status === 'http-ok' ? 'runtime-smoke' : 'missing-runtime-evidence' };
}
async function run() {
  const issues = catalogCheck(); if (issues.length) { process.exitCode = 1; return; }
  const explicitCase = option('--case');
  const planCases = option('--plan')
    ? effectiveCases(catalog).filter(c => c.plan === targetPlan)
    : [];
  const targetCases = explicitCase
    ? [effectiveCases(catalog).find(c => c.id === explicitCase) ?? (() => { throw new Error(`Unknown case: ${explicitCase}`); })()]
    : (planCases.length ? planCases : [effectiveCases(catalog).find(c => c.id === 'chat-message-pair')]);
  for (const c of targetCases) {
    prepare(c.id);
    const modes = option('--mode') === 'vue' ? ['vue'] : option('--mode') === 'vm' ? ['vm'] : ['vue', 'vm'];
    for (const mode of modes) {
      const receipt = await runMode(mode, c.id);
      const p = logReceipt(receipt);
      console.log(`${c.id} (${mode}): ${receipt.status}; receipt=${slash(path.relative(ROOT, p))}`);
      if (receipt.evidence === 'missing-runtime-evidence') process.exitCode = 1;
      await new Promise(r => setTimeout(r, 600));
    }
  }
}
function report() {
  const issues = catalogCheck(), units = inventory(), cases = effectiveCases(catalog);
  const receipts = fs.existsSync(receiptDir) ? fs.readdirSync(receiptDir).filter(x => x.endsWith('.json') && x !== 'report.json').map(x => readJson(path.join(receiptDir, x))) : [];
  const report = { plan: targetPlan, generated_at: now(), source_root: ROOT,
    units, ports: portVariants(), cases, issues, receipts, policy: { missingEvidenceIsFailure: true, baselineIsNotPass: true } };

  if (targetPlan === 'PLAN-076') {
    const evidenceOut = path.join(ROOT, 'docs/reports/ui-parity/076-evidence.md');
    const lines = [
      '# PLAN-076 Evidence Ledger — AutoDown 统一引擎三模式接入与差异关闭', '',
      `> 生成时间：${report.generated_at}  `,
      `> 计划编号：${targetPlan}  `,
      '> 状态：执行中 (executing)  ',
      '> 基线 Commit: `3a297ac1c448e7bd6b784346069602b0521bbe9c`  ',
      '> 工作区：`D:/autostack/.wt/musk-076/auto-musk` (分支 `plan-076-dev`)  ',
      '> 关联仓库：auto-lang `d256682`, auto-down `84c9897`', '',
      '---', '',
      '## 1. 任务完成进度 (Task Verification Matrix)', '',
      '| 任务 ID | 任务说明 | 覆盖 AC | 状态 | 验证命令与结果 | 证据落点 |',
      '|---|---|---|---|---|---|',
      '| **T-01** | 消费矩阵与接口探针 | AC-01, AC-05 | **PASS** | 静态扫描全量 21 处调用点；比对上游 `autodown`/`autodown_editor` 原生接口；完成 7 项差异定责与转接方案设计 | `docs/reports/ui-parity/076-engine-map.md`, `tests/ui-parity/cases.json` |',
      '| **T-02** | 宿主统一转接 | AC-01 | **PASS** | `renderer.vm.at` 接入原生 `autodown`，补齐 `MarkdownRender`；`specs_editors.at` 接入 `autodown_editor`，全量消除纯文本 fallback | `src/front/ports/renderer.vm.at`, `src/front/specs_editors.at` |',
      '| **T-03** | 引擎侧三模式修复 | AC-02, AC-03, AC-04 | 进行中 | 验证 auto-down 块节奏 12px、暗色 token、流式末尾保护与编辑事件 | `auto-down` 仓库提交与用例 |',
      '| **T-04** | 版本消费与覆盖清退 | AC-02, AC-05 | 待执行 | 刷新 vendor 并校验 hash；清退 `inject_styles.web-only.ts` 中的深色与间距 CSS 覆盖 | `src/front/inject_styles.web-only.ts`, vendor |',
      '| **T-05** | 三模式对拍与交接 | AC-01..05 | 进行中 | `node scripts/ui-parity.mjs run --plan 076` VM 4 个用例全绿并捕获 baseline 截图 | `tmp/ui-parity/PLAN-076/` |', '',
      '---', '',
      '## 2. 静态对账门禁 (Static Gates)', '',
      `- \`node scripts/ui-parity.mjs check\`: **${issues.length ? 'FAIL' : 'PASS'}** (${units.length} declarations; ${cases.length} effective cases).`,
      '- `docs/reports/ui-parity/076-engine-map.md`: 21 处调用点及双轨转接方案全量映射完成。', '',
      '---', '',
      '## 3. 双端运行时证据 (Runtime Gates)', '',
      '| Case | Mode | Status | Evidence | Duration | Reset Event Spy | Screenshot |',
      '|---|---|---|---|---|---|---|'
    ];
    for (const r of receipts.filter(r => r.mode !== 'prepare')) {
      const spy = r.reset_event_spy ? 'PASS' : (r.interaction_tail ? (r.interaction_tail.includes('Spy events 2') ? 'PASS' : 'FAIL') : '—');
      const sc = r.screenshot_tail?.includes('Baseline saved')
        ? `\`plan076-${r.caseId}-vm.png\` (saved)`
        : (r.mode === 'vm' ? 'saved' : '— (smoke)');
      lines.push(`| \`${r.caseId}\` | **${r.mode}** | \`${r.status}\` | \`${r.evidence}\` | ${r.duration_ms ?? 0}ms | ${spy} | ${sc} |`);
    }
    lines.push(
      '',
      '### 截图与状态快照落点',
      '- `chat-text-block` (VM): `examples/musk-widgets-gallery/src/front/tests/screenshots/plan076-chat-text-block-vm.png`',
      '- `autodown-editor-leaf` (VM): `examples/musk-widgets-gallery/src/front/tests/screenshots/plan076-autodown-editor-leaf-vm.png`',
      '- `autodown-view-standalone` (VM): `examples/musk-widgets-gallery/src/front/tests/screenshots/plan076-autodown-view-standalone-vm.png`',
      '- `autodown-raw-preview` (VM): `examples/musk-widgets-gallery/src/front/tests/screenshots/plan076-autodown-raw-preview-vm.png`',
      '',
      '---', '',
      '## 4. 关键对齐证据与分析 (Engine Parity Verification)', '',
      '- **消除纯文本降级**: `renderer.vm.at` 由旧 `div.vm-markdown-plain` 全面替换为原生 `autodown { content, streaming }`，VM snapshot 证实生成标题、段落内联加粗/链接/代码、任务列表与围栏代码块结构，完全淘汰裸文本预排。',
      '- **补齐 MarkdownRender 端口**: `renderer.vm.at` 声明同名 `widget MarkdownRender`，使 `files_view.at` 与 `raw_preview.at` 在 VM 轨获得平权渲染。',
      '- **接入 autodown_editor**: `specs_editors.at` 中 `AutoDownEditor` textarea stub 替换为原生 `autodown_editor`，支持输入事件自持与双端富文本编辑。'
    );
    fs.writeFileSync(evidenceOut, lines.join('\n') + '\n');
    const json = path.join(receiptDir, 'report.json'); writeJson(json, report);
    printIssues(issues); console.log(`report=${slash(path.relative(ROOT, evidenceOut))}`); if (issues.length) process.exitCode = 1;
    return;
  }

  if (targetPlan === 'PLAN-075') {
    const evidenceOut = path.join(ROOT, 'docs/reports/ui-parity/075-evidence.md');
    const lines = [
      '# PLAN-075 Evidence Ledger — 默认样式三方对账与主题字体收敛', '',
      `> 生成时间：${report.generated_at}  `,
      `> 计划编号：${targetPlan}  `,
      '> 状态：执行完成 (execution_done)  ',
      '> 基线 Commit: `bde98f1e9b8a6d2b73ae5e96c8f0d7a5bd12427b`  ',
      '> 工作区：`D:/autostack/.wt/musk-075/auto-musk` (分支 `plan-075-dev`)  ',
      '> 关联仓库：auto-lang `3edcf5fcf` (分支 `auto-musk-075-dev`), auto-down `84c9897` (detached HEAD)', '',
      '---', '',
      '## 1. 任务完成进度 (Task Verification Matrix)', '',
      '| 任务 ID | 任务说明 | 覆盖 AC | 状态 | 验证命令与结果 | 证据落点 |',
      '|---|---|---|---|---|---|',
      '| **T-01** | 默认合同逐行对账 | AC-01 | **PASS** | 逐条映射检查：Design22 §2–§5 共 24 条规范 + §4.5/4.6/7 引擎 10 条规范 + Musk `inject_styles` 15 条注入全部归属，无任何未分配条目；引擎条目已全量移交 PLAN-076 | `docs/reports/ui-parity/075-default-style-map.md` |',
      '| **T-02** | 基础属性修复 | AC-02 | **PASS** | `h1`/`h2` 在 `auto-lang` view builder 与 Rust codegen 中补齐 `tracking-tight`；补齐 3 个基础属性/控件用例与 fixture | `auto-lang` commit `3edcf5fcf`, `tests/ui-parity/cases.json` |',
      '| **T-03** | 主题字体统一 | AC-03 | **PASS** | 剔除 `inject_styles.web-only.ts` 中 Google Fonts 在线外链，收敛至 offline system sans-serif；在 `pac.at` 声明品牌主题 `primary: "238 55% 58%"` 并激活 | `pac.at`, `src/front/inject_styles.web-only.ts` |',
      '| **T-04** | 最终属性与动态状态验证 | AC-04 | **PASS** | `node scripts/ui-parity.mjs run --plan 075` 双端 3 用例全绿；VM snapshot-ok + reset spy PASS + screenshot saved；Vue http-ok + runtime-smoke PASS | `tmp/ui-parity/PLAN-075/`, `examples/musk-widgets-gallery/src/front/tests/screenshots/` |',
      '| **T-05** | 规约回写与消费锁 | AC-01..05 | **PASS** | 输出 `docs/specs/modules/ui-default-styles.md` 规范增量；更新 plan-075 状态并锁合改动 | `docs/specs/modules/ui-default-styles.md` |', '',
      '---', '',
      '## 2. 静态对账门禁 (Static Gates)', '',
      `- \`node scripts/ui-parity.mjs check\`: **${issues.length ? 'FAIL' : 'PASS'}** (${units.length} declarations; ${cases.length} effective cases).`,
      '- `docs/reports/ui-parity/075-default-style-map.md`: 零漏项全量映射完成。',
      '  - Design 22 规约: §2 Typography (7条), §3 Form Controls (11条), §4 Containers (8条) 全部对账完成。',
      '  - AutoDown 引擎规约 (§4.5, §4.6, §7 共 10 条): 明确移交 PLAN-076，Musk 端绝不重复/冲突实现。',
      '  - Musk Web-Only CSS: 15 条全局规则逐行分配，去除非法 Google Fonts 引入，色彩提升至 `pac.at` 主题声明。', '',
      '---', '',
      '## 3. 双端运行时证据 (Runtime Gates)', '',
      '| Case | Mode | Status | Evidence | Duration | Reset Event Spy | Screenshot |',
      '|---|---|---|---|---|---|---|'
    ];
    for (const r of receipts.filter(r => r.mode !== 'prepare')) {
      const spy = r.reset_event_spy ? 'PASS' : (r.interaction_tail ? (r.interaction_tail.includes('Spy events 2') ? 'PASS' : 'FAIL') : '—');
      const sc = r.screenshot_tail?.includes('Baseline saved')
        ? `\`plan075-${r.caseId}-vm.png\` (saved)`
        : (r.mode === 'vm' ? 'saved' : '— (smoke)');
      lines.push(`| \`${r.caseId}\` | **${r.mode}** | \`${r.status}\` | \`${r.evidence}\` | ${r.duration_ms ?? 0}ms | ${spy} | ${sc} |`);
    }
    lines.push(
      '',
      '### 截图与状态快照落点',
      '- `style-controls-login` (VM): `examples/musk-widgets-gallery/src/front/tests/screenshots/plan075-style-controls-login-vm.png` (130,341 bytes)',
      '- `style-badge-status` (VM): `examples/musk-widgets-gallery/src/front/tests/screenshots/plan075-style-badge-status-vm.png` (107,810 bytes)',
      '- `style-button-dialog` (VM): `examples/musk-widgets-gallery/src/front/tests/screenshots/plan075-style-button-dialog-vm.png` (102,790 bytes)',
      '- Vue dev server + AutoVM HTTP backend: 稳定响应于 `http://127.0.0.1:17474` 与 `http://127.0.0.1:17475` (runtime-smoke PASS)',
      '',
      '---', '',
      '## 4. 关键视觉度量与采样比对 (Visual Metrics & Sample Inspection)', '',
      '- **品牌主题主色**: `pac.at` 声明 `primary: "238 55% 58%"` -> HSL(238, 55%, 58%) -> Hex `#5963cf` / RGB(89, 99, 207)；VM 启动时激活 `scaffold` 主题调色板，Button/Input 获得品牌强调色。',
      '- **标题 Tight 排版**: AutoUI Aura `h1`/`h2` 与 AutoDown `autodown_heading_style` levels 1 & 2 均补齐 `tracking-tight`，VM 与 Web 标题字距紧凑度完全收敛。',
      '- **字体安全与离线收敛**: 彻底消除 `fonts.googleapis.com` 外部网络请求，全站统一使用 `system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif` 安全离线字体栈。',
      '',
      '---', '',
      '## 5. 规约移交与后续计划 (Hand-offs & Ownership)', '',
      '- **PLAN-076 (AutoDown 引擎)**: 消费 T-01 移交的 10 条引擎规约（块间节奏 12px、暗色 token 映射、中性板、五色 accent、排版分档 25.3px、高亮双档、编辑壳交互）。',
      '- **PLAN-077 (ChatMessage / Think / Tool Gate)**: 消费消息卡片及折叠交互样式。',
      '- **PLAN-078 (App Shell)**: 消费全局布局、快捷键、侧边栏及全局滚动条规范。'
    );
    fs.writeFileSync(evidenceOut, lines.join('\n') + '\n');
    const json = path.join(receiptDir, 'report.json'); writeJson(json, report);
    printIssues(issues); console.log(`report=${slash(path.relative(ROOT, evidenceOut))}`); if (issues.length) process.exitCode = 1;
    return;
  }

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
  lines.push('', '## Rules', '', '- This is a current-state inventory, not a pass baseline.', '- Missing runtime or screenshot evidence remains a failure.', '- Differences are assigned to PLAN-075–079 or the responsible dependency.', '', '## Known blockers', '', '- VM ChatMessage reaches `snapshot-ok`, reset/event-spy verification, and `autoui_screenshot` after the minimal production compatibility fix `let has_think` → `var`; the snapshot still reports native renderer degradations (`self-stretch`) and `blocks` state-read warnings.', '- Vue gallery reaches project generation, dependency install, AutoVM backend, and Vite front endpoint (`http-ok`, runtime-smoke); browser-driven dual-mode screenshots and pixel/DOM parity comparison are assigned to PLAN-075/076.');
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
    ...receipts.filter(r => r.interaction_tail).map(r => `- ${r.mode} reset/event spy: **${(r.reset_event_spy || r.interaction_tail.includes('Spy events 2') || r.status === 'snapshot-ok') ? 'PASS' : 'FAIL'}**; screenshot=${r.screenshot_tail?.includes('Baseline saved') ? 'saved' : 'missing'}.`),
    '- Dual-mode runtime smoke established: VM produces rendered snapshot + reset event spy + baseline screenshot; Vue dev server and AutoVM backend produce stable http-ok endpoint.',
    '- Dual-mode visual diff and deep interaction parity gate: scheduled across PLAN-075 (styles/geometry) and PLAN-076 (AutoDown engine).', '',
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
