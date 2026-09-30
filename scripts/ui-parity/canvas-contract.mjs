#!/usr/bin/env node
// PLAN-093 T-06 V03（种子）：canvas 生产 helper 边界测试。
//
// 被测对象 = 生成的 ext 生产模块（gen/front/vue/src/ext/src/front/
// canvas_helpers.ts，.at 单源经 use.web.fn 链产出）——不复制算法。
// web 轨直测；VM 轨同源 .at 由解释器消费（消费面证据见 vm-studio-cycle
// 的包装层样式断言），本文件覆盖纯函数边界与换算契约。
//
// 退出码：全过 0，任一失败 1。收据：stdout（V03 汇总行）。

import { pathToFileURL } from 'node:url';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const HELPERS_TS = path.join(ROOT, 'gen', 'front', 'vue', 'src', 'ext', 'src', 'front', 'canvas_helpers.ts');

const { canvasMapPhysical, canvasContentStyle, canvasFrameUrl, canvasPickedProjection, rebuildCanvasOverlay, canvasPickStyleFromBbox, canvasTreeVisible, canvasSourceLines, canvasProgressRows } = await import(pathToFileURL(HELPERS_TS).href);

let pass = 0, fail = 0;
const assert = (name, ok, detail) => {
  if (ok) { pass++; console.log(`  ✓ ${name}${detail ? ` — ${detail}` : ''}`); }
  else { fail++; console.error(`  ✗ ${name}${detail ? ` — ${detail}` : ''}`); }
};
const near = (a, b, tol) => Math.abs(a - b) <= tol;

// ── canvasMapPhysical：退化/边界/截断契约 ────────────────────────────────
console.log('[canvasMapPhysical]');
{
  // 退化尺寸：全零出界（不产 NaN）
  const z = canvasMapPhysical(10, 10, 0, 0, 720, 1020);
  assert('零盒尺寸 → 出界零点', z.inside === false && z.x === 0 && z.y === 0, JSON.stringify(z));
  const z2 = canvasMapPhysical(10, 10, 349, 494, 0, 0);
  assert('零帧尺寸 → 出界零点', z2.inside === false, JSON.stringify(z2));

  // 边界点：左上角含；右/下边界不含（ox<box_w 严格）
  const tl = canvasMapPhysical(0, 0, 349, 494, 720, 1020);
  assert('边界点 (0,0) 含', tl.inside === true && tl.x === 0 && tl.y === 0, JSON.stringify(tl));
  const rEdge = canvasMapPhysical(349, 100, 349, 494, 720, 1020);
  assert('右边界 (box_w) 不含', rEdge.inside === false, JSON.stringify(rEdge));
  const bEdge = canvasMapPhysical(100, 494, 349, 494, 720, 1020);
  assert('下边界 (box_h) 不含', bEdge.inside === false, JSON.stringify(bEdge));
  const brIn = canvasMapPhysical(348, 493, 349, 494, 720, 1020);
  assert('内缘 (box-1,box-1) 含', brIn.inside === true, JSON.stringify(brIn));

  // 负偏移（容器留白换算面）出界
  const neg = canvasMapPhysical(-5, 100, 349, 494, 720, 1020);
  assert('负偏移出界', neg.inside === false, JSON.stringify(neg));

  // 换算正确性 + 整除截断界（误差 < natural/box 比 = 1 css px 的物理当量）
  const m = canvasMapPhysical(175, 247, 349, 494, 720, 1020);
  const exactX = 175 * 720 / 349, exactY = 247 * 1020 / 494;
  assert('中心换算', near(m.x, exactX, 2.1) && near(m.y, exactY, 2.1), `x=${m.x}(期望≈${exactX.toFixed(1)}) y=${m.y}(≈${exactY.toFixed(1)})`);
  let maxErr = 0;
  for (let ox = 0; ox < 349; ox += 7) for (let oy = 0; oy < 494; oy += 11) {
    const r = canvasMapPhysical(ox, oy, 349, 494, 720, 1020);
    maxErr = Math.max(maxErr, Math.abs(r.x - ox * 720 / 349), Math.abs(r.y - oy * 1020 / 494));
  }
  assert('截断误差扫描 <1 css 当量(2.07px)', maxErr < 2.07, `maxErr=${maxErr.toFixed(3)}px`);

  // DPI 等价性：换算输入是 CSS 量（deviceScaleFactor 不进公式）——同
  // css 点不同 DPI 同结果（契约面：不分别猜 bbox 坐标）。
  const dpiA = canvasMapPhysical(100, 150, 349, 494, 720, 1020);
  const dpiB = canvasMapPhysical(100, 150, 349, 494, 720, 1020);
  assert('同 css 输入同输出（DPI 无关契约）', dpiA.x === dpiB.x && dpiA.y === dpiB.y);
}

// ── canvasContentStyle：双模式 + 退化 ────────────────────────────────────
console.log('[canvasContentStyle]');
{
  assert('退化（0 尺寸）占满', canvasContentStyle(0, 0, true) === 'position:relative;max-width:100%;max-height:100%;margin:auto', canvasContentStyle(0, 0, true));
  assert('负尺寸退化', canvasContentStyle(-1, 100, true).includes('max-width:100%'));
  const fit = canvasContentStyle(720, 1020, true);
  assert('fit=定比盒', fit === 'position:relative;aspect-ratio:720/1020;max-width:100%;max-height:100%;margin:auto', fit);
  const full = canvasContentStyle(720, 1020, false);
  assert('100%=物理 px 定尺寸', full === 'position:relative;width:720px;height:1020px;margin:auto', full);
}

// ── 帧身份与投影漏斗 ─────────────────────────────────────────────────────
console.log('[canvasFrameUrl / canvasPickedProjection / rebuildCanvasOverlay]');
{
  assert('帧 URL 身份+seq 双键', canvasFrameUrl(7, 2) === '/api/canvas/frame?t=7&gen=2', canvasFrameUrl(7, 2));

  const empty = canvasPickedProjection(null);
  assert('picked=null → 全空投影（统一清场）', empty.id === '' && empty.style === '' && empty.source_path === '' && empty.for_label === '', JSON.stringify(empty));

  const pk = {
    vnode_id: 'vnode_42', kind: 'button', label: '提交', source: 'src/a.at',
    source_path: 'src/a.at', source_line: 7, forctx: { index: 2 },
    bbox_pct: { x: 10, y: 20, w: 30, h: 40 },
  };
  const p = canvasPickedProjection(pk);
  assert('picked 投影 8 字段（含 source_line）', p.id === 'vnode_42' && p.kind === 'button' && p.label === '提交' && p.source_path === 'src/a.at' && p.for_label === '#2' && p.source_line === 7 && p.style.includes('left:10%'), JSON.stringify(p).slice(0, 160));
  const pkNoLine = { ...pk, source_line: undefined };
  assert('source_line 缺省 0（无有效行）', canvasPickedProjection(pkNoLine).source_line === 0);
  assert('覆盖框样式含 pointer-events:none', p.style.includes('pointer-events:none'));

  const ovs = rebuildCanvasOverlay([
    { vnode_id: 'vnode_1', bbox_pct: { x: 0, y: 0, w: 50, h: 50 } },
    { vnode_id: 'vnode_2', bbox_pct: { x: 50, y: 50, w: 50, h: 50 } },
  ]);
  assert('overlay 重建 2 条', ovs.length === 2 && ovs[0].style.includes('left:0%'), JSON.stringify(ovs).slice(0, 120));
  assert('overlay 样式不拦指针', ovs.every(o => o.style.includes('pointer-events:none')));
  assert('pick 样式蓝/overlay 琥珀分色', canvasPickStyleFromBbox(pk.bbox_pct).includes('rgb(59,130,246)') && ovs[0].style.includes('rgb(245,158,11)'));
}

// ── canvasTreeVisible：折叠/展开/has_kids/前序可见性契约 ─────────────────
console.log('[canvasTreeVisible]');
{
  const flat = [
    { vid: 'a', depth: 0, kind: 'col', label: '', for_label: '', indent_style: 'padding-left:4px' },
    { vid: 'b', depth: 1, kind: 'row', label: 'L1', for_label: '', indent_style: 'padding-left:16px' },
    { vid: 'c', depth: 2, kind: 'text', label: 'T', for_label: '#1', indent_style: 'padding-left:28px' },
    { vid: 'd', depth: 1, kind: 'button', label: 'B', for_label: '', indent_style: 'padding-left:16px' },
  ];
  const all = canvasTreeVisible(flat, {});
  assert('全展开 4 行', all.length === 4, `len=${all.length}`);
  assert('has_kids 标记（a、b 有子；c、d 无）', all[0].has_kids === true && all[1].has_kids === true && all[2].has_kids === false && all[3].has_kids === false);
  assert('open 只对有子行为真（叶行恒 false）', all.filter(r => r.has_kids).every(r => r.open === true) && all.filter(r => !r.has_kids).every(r => r.open === false));

  const colB = canvasTreeVisible(flat, { b: true });
  assert('折叠 b → 整枝 c 隐藏（3 行）', colB.length === 3 && !colB.some(r => r.vid === 'c'), `len=${colB.length}`);
  assert('折叠节点行保留且 open=false', colB.find(r => r.vid === 'b').open === false);

  const colA = canvasTreeVisible(flat, { a: true });
  assert('折叠根 → 仅根行', colA.length === 1 && colA[0].vid === 'a' && colA[0].open === false);

  const dup = canvasTreeVisible([...flat, { ...flat[1], vid: 'b2' }], { b: true });
  assert('同 vid 复用折叠键；兄弟分支不受影响', dup.length === 4 && dup.some(r => r.vid === 'b2'), `len=${dup.length}`);

  assert('空树 → 空行', canvasTreeVisible([], {}).length === 0);
  const d0 = canvasTreeVisible([{ vid: 'x', kind: 'col' }], {});
  assert('depth 缺省 0 不崩', d0.length === 1 && d0[0].depth === 0);
}

// ── canvasSourceLines：行切分契约 ────────────────────────────────────────
console.log('[canvasSourceLines]');
{
  assert('三行切分', JSON.stringify(canvasSourceLines('a\nb\nc')) === JSON.stringify(['a', 'b', 'c']));
  assert('尾行无换行照收', JSON.stringify(canvasSourceLines('a\nb')) === JSON.stringify(['a', 'b']));
  assert('空串单空行', canvasSourceLines('').length === 1 && canvasSourceLines('')[0] === '');
  assert('空行保留', JSON.stringify(canvasSourceLines('a\n\nb')) === JSON.stringify(['a', '', 'b']));
  assert('None → 空表', canvasSourceLines(null).length === 0);
  const many = canvasSourceLines('x\n'.repeat(500));
  assert('长文本 500 行（尾空行）', many.length === 501 && many[499] === 'x' && many[500] === '');
}

// ── canvasProgressRows：工具事件投影契约（T-09 §5.8）─────────────────────
console.log('[canvasProgressRows]');
{
  // 无事件不占行（不硬凑五步）：空消息 + stopped 无应用 → 0 行。
  const empty = canvasProgressRows([], 'stopped', 0, '', 0, '');
  assert('无事件+无预览 → 0 行（不硬凑步骤）', empty.length === 0, JSON.stringify(empty));

  // live 块（tc 形态）：生成完成 + 运行中帧可见 → generate done / preview
  // visible / verify 未验证 / frame info。
  const liveMsg = {
    id: 'm1',
    blocks: [
      { kind: 'text', text: 'hi' },
      { kind: 'tool', tkey: 'k1', tc: { id: 'tc-1', name: 'write_file', status: 'completed', result: 'ok', details: null } },
      { kind: 'tool', tkey: 'k2', tc: { id: 'tc-2', name: 'canvas_run', status: 'completed', result: 'starting', details: { canvas: { kind: 'run', generation_id: 3 } } } },
    ],
  };
  const rowsA = canvasProgressRows([liveMsg], 'running', 15, 'apps/demo', 3, '');
  const byKey = Object.fromEntries(rowsA.map(r => [r.key, r]));
  assert('生成行 done（write_file 完成）', byKey.generate?.state === 'done', JSON.stringify(byKey.generate));
  assert('预览行 visible（running+seq>0，CanvasStore 权威）', byKey.preview?.state === 'visible', JSON.stringify(byKey.preview));
  assert('验证行未验证（无 canvas_act/state/snapshot）', byKey.verify?.state === 'unverified', JSON.stringify(byKey.verify));
  assert('帧行 info（seq>0）', byKey.frame?.state === 'info', JSON.stringify(byKey.frame));
  assert('lint/bp 无证据不占行', byKey.lint === undefined && byKey.bp === undefined);
  assert('start 返回≠已可见：running+seq=0 → waiting', canvasProgressRows([liveMsg], 'running', 0, 'apps/demo', 3, '').find(r => r.key === 'preview')?.state === 'waiting');

  // 回放块（扁平形态）：success/error 状态映射 + bp/lint 分行。
  const replayMsg = {
    id: 'm2',
    blocks: [
      { kind: 'tool', tool_name: 'bp_list', tool_status: 'success', tool_result: 'ok', tool_id: 'tc-3' },
      { kind: 'tool', tool_name: 'bp_check', tool_status: 'error', tool_result: 'FAIL', tool_id: 'tc-4' },
      { kind: 'tool', tool_name: 'ui_lint', tool_status: 'success', tool_result: '# 2 findings', tool_id: 'tc-5' },
    ],
  };
  const rowsB = canvasProgressRows([replayMsg], 'stopped', 0, '', 0, '');
  const byKeyB = Object.fromEntries(rowsB.map(r => [r.key, r]));
  assert('生成行 done（bp_list 回放）', byKeyB.generate?.state === 'done', JSON.stringify(byKeyB.generate));
  assert('bp 行 failed（bp_check error，回放扁平形态）', byKeyB.bp?.state === 'failed', JSON.stringify(byKeyB.bp));
  assert('lint 行 pass（advisory，成功=有结果）', byKeyB.lint?.state === 'pass', JSON.stringify(byKeyB.lint));
  assert('stopped+无应用+无 canvas_run → 预览行省略', byKeyB.preview === undefined);

  // stopped+有应用描述 → 已停止（不隐去预览行）。
  const rowsC = canvasProgressRows([liveMsg], 'stopped', 0, 'apps/demo', 3, '');
  assert('stopped+有 app → 预览行 stopped', rowsC.find(r => r.key === 'preview')?.state === 'stopped');

  // degraded → 预览失败。
  assert('degraded → 预览行 failed', canvasProgressRows([liveMsg], 'degraded', 0, 'apps/demo', 3, 'spawn failed').find(r => r.key === 'preview')?.state === 'failed');

  // 陈旧代次章：canvas_run details.canvas.generation_id=1 ≠ 当前 cv_gen=2
  // 且 stopped 无应用 → 事件属被替换预览，不冒充当前状态（预览行省略）。
  const staleRun = { id: 'm3', blocks: [{ kind: 'tool', tkey: 'k3', tc: { id: 'tc-6', name: 'canvas_run', status: 'completed', result: 'starting', details: { canvas: { kind: 'run', generation_id: 1 } } } }] };
  assert('代次章失配 → 陈旧 run 不冒充预览', canvasProgressRows([staleRun], 'stopped', 0, '', 2, '').find(r => r.key === 'preview') === undefined);
  // 同代次章：保留 requested（事件仍指当前预览）。
  assert('代次章匹配 → requested 保留', canvasProgressRows([staleRun], 'stopped', 0, '', 1, '').find(r => r.key === 'preview')?.state === 'requested');

  // 验证失败与运行中。
  const verifyMsg = { id: 'm4', blocks: [{ kind: 'tool', tkey: 'k4', tc: { id: 'tc-7', name: 'canvas_act', status: 'failed', result: 'boom', details: { canvas: { kind: 'verify' } } } }, { kind: 'tool', tkey: 'k5', tc: { id: 'tc-8', name: 'canvas_state', status: 'running', result: '', details: null } }] };
  assert('验证行 failed→running 优先（任一在跑）', canvasProgressRows([verifyMsg], 'running', 5, 'a', 1, '').find(r => r.key === 'verify')?.state === 'running');
  const verifyFail = { id: 'm5', blocks: [{ kind: 'tool', tkey: 'k6', tc: { id: 'tc-9', name: 'canvas_act', status: 'failed', result: 'boom', details: null } }] };
  assert('验证行 failed（最新失败）', canvasProgressRows([verifyFail], 'running', 5, 'a', 1, '').find(r => r.key === 'verify')?.state === 'failed');

  // gate_waiting 计入运行中。
  const gateMsg = { id: 'm6', blocks: [{ kind: 'tool', tkey: 'k7', tc: { id: 'tc-10', name: 'ui_lint', status: 'gate_waiting', result: '', details: null } }] };
  assert('gate_waiting → 检查中', canvasProgressRows([gateMsg], 'stopped', 0, '', 0, '').find(r => r.key === 'lint')?.state === 'running');

  // seq=0 无帧行。
  assert('seq=0 → 无帧行', canvasProgressRows([], 'running', 0, 'a', 1, '').find(r => r.key === 'frame') === undefined);

  // label_key 为完整 i18n 键（视图 t() 直查）。
  assert('label_key 完整键', byKey.generate?.label_key === 'canvas.progRowGenerate' && byKey.frame?.label_key === 'canvas.progRowFrame');
}

console.log(`V03 canvas-contract: ${pass} pass / ${fail} fail`);
process.exit(fail ? 1 : 0);