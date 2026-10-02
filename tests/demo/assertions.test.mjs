// tests/demo/assertions.test.mjs — PLAN-097 T-09 V02：目标 VM 行为断言的
// 纯判言/解析/动作通道/跨启循环（注入桩,不起真机;真机 V05 属 T-12）。
import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  parseVmTree, subtreeText, findCard, descendants, actByLabel,
  restartVerdict, canvasRestart, diskVerdict,
  columnSections, headerCount, columnOfTask, cardButton,
  taskVisibleVerdict, wipRejectionVerdict, invalidInputVerdict, anchorVerdict,
  deliveredGate, p003SessionVerdict, p003InputVerdict, p003ReadEvidence, runChecks, pickAnchor,
} from '../../scripts/demo/assertions.mjs';

// 合成 FocusBoard 快照（模拟 Agent 可产出的三列 UI;结构仅测试解析/匹配）。
const SNAP = `
window #v1 "FocusBoard" @rect(0,0,480,680)
  text #v2 "任务看板" @rect(10,5,100,20)
  row #v3 "" @rect(10,30,460,30)
    input #v4 "buy milk" @rect(10,30,300,28)
      placeholder: "任务标题"
    button #v5 "添加" @rect(320,30,60,28)
  row #v6 "" @rect(0,0,0,0)
  column #v7 "todo 2" @rect(10,70,150,500)
    card #v8 "buy milk" @rect(15,80,140,40)
      button #v9 "重命名" @rect(20,85,50,20)
      button #v10 "删除" @rect(75,85,40,20)
      button #v11 "→ doing" @rect(120,85,30,20)
    card #v12 "write spec" @rect(15,125,140,40)
  column #v13 "doing 1/3" @rect(165,70,150,500)
    card #v14 "ship it" @rect(170,80,140,40)
  column #v15 "done 0" @rect(320,70,150,500)
`;

test('parseVmTree: hierarchy/labels/rects; zero-rect → null; placeholder line', () => {
  const { nodes } = parseVmTree(SNAP);
  const win = nodes[0];
  assert.equal(win.kind, 'window');
  assert.deepEqual(win.rect, { x: 0, y: 0, w: 480, h: 680 });
  const input = nodes.find(n => n.id === 'v4');
  assert.equal(input.placeholder, '任务标题');
  const ghost = nodes.find(n => n.id === 'v6');
  assert.equal(ghost.rect, null, '零 rect 必须折叠为 null(不造坐标)');
});

test('findCard: own-label priority; subtree fallback; descendants filter', () => {
  const { nodes } = parseVmTree(SNAP);
  const card = findCard(nodes, /buy milk/);
  assert.equal(card.id, 'v8', '自身 label 命中的卡片优先于窗口 subtree');
  const del = descendants(card, n => n.kind === 'button' && /删除/.test(n.label));
  assert.equal(del.length, 1);
  assert.equal(del[0].id, 'v10');
  assert.match(subtreeText(card), /重命名/);
});

test('actByLabel: fresh snapshot each attempt, vnode action, stale retry', async () => {
  let snaps = 0;
  const mcp = {
    snapshot: async () => {
      snaps++;
      // 第 1 拍目标缺失,第 2 拍出现且 id 变化(vnode 重编号语义)
      return snaps === 1 ? SNAP.replace('"添加" @rect', '"不存在" @rect') : SNAP;
    },
    call: async (name, args) => {
      assert.equal(name, 'autoui_action');
      assert.equal(args.element_id, 'v5');
      return 'ok';
    },
  };
  const r = await actByLabel({ mcp, match: n => n.kind === 'button' && /添加/.test(n.label), action: 'press', label: 'add' });
  assert.equal(r, 'ok');
  assert.ok(snaps >= 2, '必须重拍快照(vnode 每帧重编号)');
});

test('actByLabel: stale VNode retries with re-snapshot', async () => {
  let snaps = 0;
  const mcp = {
    snapshot: async () => { snaps++; return SNAP; },
    call: async (name, args) => {
      if (snaps <= 2) throw new Error('MCP autoui_action isError: VNode not found: vnode_v5');
      return 'ok';
    },
  };
  const r = await actByLabel({ mcp, match: n => n.id === 'v5', action: 'press', label: 'x', settleMs: 5 });
  assert.equal(r, 'ok');
  assert.ok(snaps >= 3);
});

test('actByLabel: missing target times out loudly', async () => {
  const mcp = { snapshot: async () => SNAP, call: async () => 'ok' };
  await assert.rejects(
    () => actByLabel({ mcp, match: () => false, action: 'press', timeoutMs: 120, settleMs: 10, label: 'ghost' }),
    /target not found/,
  );
});

test('restartVerdict: generation must change; canvasRestart drives stop/start and waits new gen', async () => {
  assert.equal(restartVerdict(3, 3).ok, false);
  assert.equal(restartVerdict(3, 4).ok, true);
  assert.equal(restartVerdict('x', 4).ok, false);

  let gen = 7;
  let stopped = false;
  const fetchImpl = async (url, opts = {}) => {
    const u = String(url);
    if (u.endsWith('/api/canvas/status')) {
      const valid = stopped;
      return { json: async () => ({ state: valid ? 'running' : stopped ? 'running' : 'running', generation_id: gen, app_path: 'src/front/app.at', frame: { valid } }) };
    }
    if (u.endsWith('/api/canvas/stop')) {
      assert.equal(JSON.parse(opts.body).generation, 7, 'stop 必须携带停止前代次');
      stopped = true;
      return { status: 200, text: async () => '{"state":"stopped","generation_id":7}' };
    }
    if (u.endsWith('/api/canvas/start')) {
      assert.equal(JSON.parse(opts.body).app_path, 'src/front/app.at', '重启同一 app_path');
      gen = 8;
      stopped = true;
      return { status: 200, text: async () => `{"generation_id":8}` };
    }
    throw new Error(`unexpected url ${u}`);
  };
  const r = await canvasRestart({ base: 'http://x', fetchImpl });
  assert.deepEqual({ oldGen: r.oldGen, newGen: r.newGen }, { oldGen: 7, newGen: 8 });
  assert.equal(r.verdict.ok, true);
});

test('canvasRestart: refuses when canvas not running', async () => {
  const fetchImpl = async () => ({ json: async () => ({ state: 'stopped', generation_id: 3, app_path: null }) });
  await assert.rejects(() => canvasRestart({ base: 'http://x', fetchImpl }), /not running/);
});

test('diskVerdict: expect present + absent absent', () => {
  assert.equal(diskVerdict({ text: 'buy milk todo\n' }, { expect: ['buy milk'] }).ok, true);
  const v = diskVerdict({ text: '' }, { expect: ['buy milk'], absent: [] });
  assert.equal(v.ok, false);
  const v2 = diskVerdict({ text: 'ghost' }, { expect: [], absent: ['ghost'] });
  assert.equal(v2.ok, false);
});

test('columnSections/headerCount/columnOfTask: 三列与计数提取', () => {
  const { nodes } = parseVmTree(SNAP);
  const cols = columnSections(nodes);
  assert.equal(cols.todo.id, 'v7');
  assert.equal(cols.doing.id, 'v13');
  assert.equal(cols.done.id, 'v15');
  assert.deepEqual(headerCount(cols.todo), { n: 2, cap: null });
  assert.deepEqual(headerCount(cols.doing), { n: 1, cap: 3 }, 'doing 列头 n/3 形态');
  assert.equal(columnOfTask(nodes, /ship it/), 'doing');
  assert.equal(columnOfTask(nodes, /buy milk/), 'todo');
});

test('cardButton: 卡片内按钮定位', () => {
  const { nodes } = parseVmTree(SNAP);
  assert.equal(cardButton(nodes, /buy milk/, /→ doing/)?.id, 'v11');
  assert.equal(cardButton(nodes, /nonexistent/, /删除/), null);
});

test('taskVisibleVerdict / wipRejectionVerdict: P002 拒绝三臂', () => {
  const { nodes } = parseVmTree(SNAP);
  assert.equal(taskVisibleVerdict(nodes, 'buy milk').ok, true);
  assert.equal(taskVisibleVerdict(nodes, 'nope').ok, false);

  // 好 case:解释可见 + 第4项留在 todo(拒绝生效) + 磁盘仍有
  const snap4 = SNAP
    .replace('"write spec" @rect(15,125,140,40)', '"write spec" @rect(15,125,140,40)\n    card #v21 "task four" @rect(15,170,140,40)')
    + '\n  text #v20 "已到上限 3/3,不能添加更多" @rect(10,600,300,20)\n';
  const snap4Nodes = parseVmTree(snap4).nodes;
  assert.equal(wipRejectionVerdict({ snapText: snap4, nodes: snap4Nodes, taskTitle: 'task four', disk: { text: 'task four todo' } }).ok, true,
    JSON.stringify(wipRejectionVerdict({ snapText: snap4, nodes: snap4Nodes, taskTitle: 'task four', disk: { text: 'task four todo' } })));
  // 臂1:解释缺失
  const noExplain = wipRejectionVerdict({ snapText: SNAP, nodes, taskTitle: 'task four', disk: { text: 'task four' } });
  assert.equal(noExplain.ok, false);
  assert.ok(noExplain.errors.some(e => /解释/.test(e)));
  // 臂2:第4项竟在 doing
  const snapBad = SNAP.replace('card #v14 "ship it"', 'card #v14 "task four"').replace(/已到上限/, 'x').replace('doing 1/3', 'doing 3/3') + '\n  text #v20 "已到上限 3/3" @rect(0,0,10,10)\n';
  const inDoing = wipRejectionVerdict({ snapText: snapBad, nodes: parseVmTree(snapBad).nodes, taskTitle: 'task four', disk: { text: 'task four' } });
  assert.equal(inDoing.ok, false);
  assert.ok(inDoing.errors.some(e => /doing/.test(e)));
  // 臂3:磁盘缺任务
  const noDisk = wipRejectionVerdict({ snapText: snap4, nodes: snap4Nodes, taskTitle: 'task four', disk: { text: '' } });
  assert.equal(noDisk.ok, false);
  assert.ok(noDisk.errors.some(e => /disk/.test(e)));
});

test('invalidInputVerdict: 错误可见 + 原设定保留', () => {
  const good = '设定上限: 3  错误:上限必须是 1~5 的整数';
  assert.equal(invalidInputVerdict({ snapText: good, originalLimit: 3, invalidInput: 'abc' }).ok, true);
  const noErr = invalidInputVerdict({ snapText: '设定上限: 3', originalLimit: 3, invalidInput: 0 });
  assert.equal(noErr.ok, false);
  const lostOrig = invalidInputVerdict({ snapText: '错误:无效', originalLimit: 3, invalidInput: 0 });
  assert.equal(lostOrig.ok, false);
  assert.ok(lostOrig.errors.some(e => /原设定/.test(e)));
});

test('anchorVerdict: source:line + nonzero bbox + confidence;零 rect 拒绝', () => {
  const good = { vnode_id: 'vnode_9', source: 'src/front/app.at:42', source_confidence: 'exact', bbox_px: { x: 1, y: 2, w: 30, h: 12 } };
  assert.equal(anchorVerdict(good).ok, true);
  assert.equal(anchorVerdict({ ...good, source: undefined }).ok, false);
  assert.equal(anchorVerdict({ ...good, bbox_px: { x: 0, y: 0, w: 0, h: 0 } }).ok, false, '零 rect 不用于锚点');
  assert.equal(anchorVerdict({ ...good, source_confidence: undefined }).ok, false, '置信度必须落载荷');
  assert.equal(anchorVerdict({ ...good, source: 'src/front/app.at' }).ok, false, '缺行号不算 file:line');
});

test('deliveredGate: delivered+五检查点放行;未交付/缺件响亮拒绝', () => {
  const ok = {
    status: 'completed',
    plan_execution: {
      phase: 'delivered', outcome: 'pass',
      delivery_checkpoints: Object.fromEntries(['prepared', 'landed', 'ledger_refreshed', 'archived', 'cleaned'].map(k => [k, { checkpoint: k }])),
    },
  };
  assert.equal(deliveredGate(ok, { planLabel: 'P001' }), true);
  assert.throws(() => deliveredGate({ status: 'running', plan_execution: { phase: 'document' } }, { planLabel: 'P001' }), /未交付.*不发下一需求/);
  assert.throws(() => deliveredGate({ status: 'completed', plan_execution: { phase: 'document' } }, { planLabel: 'P001' }), /未交付/);
  assert.throws(() => deliveredGate({ status: 'completed', plan_execution: { phase: 'delivered', delivery_checkpoints: { prepared: {} } } }, { planLabel: 'P001' }), /缺件/);
});

test('p003SessionVerdict/p003InputVerdict: 新会话与无注入', () => {
  assert.equal(p003SessionVerdict('s3', ['s1', 's2']).ok, true);
  assert.equal(p003SessionVerdict('s1', ['s1', 's2']).ok, false);
  assert.equal(p003SessionVerdict(null, []).ok, false);
  assert.equal(p003InputVerdict('# P003\n需求\r\n', '# P003\n需求\n').ok, true, '仅空白差异归一');
  const v = p003InputVerdict('# P003 需求\n前两轮上下文:…', '# P003 需求\n');
  assert.equal(v.ok, false);
  assert.match(v.reason, /注入/);
});

test('p003ReadEvidence: turn_tool_call 读取 specs/plans/wiki 提取;空=失败', () => {
  const events = [
    { type: 'turn_tool_call', tool_name: 'read_file', arguments: { path: 'docs/specs/board.md' } },
    { type: 'turn_tool_call', tool_name: 'read_file', arguments: { path: 'docs/plans/archived/p001.md' } },
    { type: 'turn_tool_call', tool_name: 'write_file', arguments: { path: 'docs/specs/x.md' } },
    { type: 'step_started', step_id: 'a' },
  ];
  const ev = p003ReadEvidence(events);
  assert.equal(ev.observed, true);
  assert.equal(ev.paths.length, 2, 'write 不算读取证据');
  assert.equal(ev.verdict.ok, true);
  const none = p003ReadEvidence([{ type: 'turn_tool_call', tool_name: 'run_command', arguments: { cmd: 'ls' } }]);
  assert.equal(none.observed, false);
  assert.equal(none.verdict.ok, false);
});

test('runChecks: 行为失败记录不抛;通道异常记 FAIL 后继续', async () => {
  const r = await runChecks([
    { name: 'a', run: async () => ({ ok: true, reason: 'fine' }) },
    { name: 'b', run: async () => ({ ok: false, errors: ['bad state'] }) },
    { name: 'c', run: async () => { throw new Error('MCP down'); } },
  ], { log: () => {} });
  assert.equal(r.ok, false);
  assert.equal(r.checks.length, 3);
  assert.equal(r.checks[1].detail, 'bad state');
  assert.match(r.checks[2].detail, /channel error: MCP down/);
});

test('pickAnchor: vnode 直选;204 未命中/409 代次冲突响亮;载荷可判言', async () => {
  const anchor = { vnode_id: 'vnode_9', kind: 'button', label: '添加', source: 'src/front/app.at:42', source_confidence: 'exact', bbox_px: { x: 1, y: 2, w: 30, h: 12 } };
  const fetchImpl = async (url, opts = {}) => {
    const body = JSON.parse(opts.body ?? '{}');
    if (body.vnode_id === 'vnode_9') assert.equal(body.expected_generation, 4, '期望代次必须携带(防跨目标)');
    if (body.vnode_id === 'vnode_ghost') return { status: 204 };
    if (body.vnode_id === 'vnode_stale') return { status: 409, json: async () => ({ generation_id: 5 }) };
    return { status: 200, json: async () => anchor };
  };
  const a = await pickAnchor({ base: 'http://x', vnodeId: 'vnode_9', generation: 4, fetchImpl });
  assert.equal(anchorVerdict(a).ok, true);
  await assert.rejects(() => pickAnchor({ base: 'http://x', vnodeId: 'vnode_ghost', generation: 4, fetchImpl }), /未命中/);
  await assert.rejects(() => pickAnchor({ base: 'http://x', vnodeId: 'vnode_stale', generation: 4, fetchImpl }), /409/);
});
