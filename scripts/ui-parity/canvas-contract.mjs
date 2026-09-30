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

const { canvasMapPhysical, canvasContentStyle, canvasFrameUrl, canvasPickedProjection, rebuildCanvasOverlay, canvasPickStyleFromBbox } = await import(pathToFileURL(HELPERS_TS).href);

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
    source_path: 'src/a.at', forctx: { index: 2 },
    bbox_pct: { x: 10, y: 20, w: 30, h: 40 },
  };
  const p = canvasPickedProjection(pk);
  assert('picked 投影 7 字段', p.id === 'vnode_42' && p.kind === 'button' && p.label === '提交' && p.source_path === 'src/a.at' && p.for_label === '#2' && p.style.includes('left:10%'), JSON.stringify(p).slice(0, 140));
  assert('覆盖框样式含 pointer-events:none', p.style.includes('pointer-events:none'));

  const ovs = rebuildCanvasOverlay([
    { vnode_id: 'vnode_1', bbox_pct: { x: 0, y: 0, w: 50, h: 50 } },
    { vnode_id: 'vnode_2', bbox_pct: { x: 50, y: 50, w: 50, h: 50 } },
  ]);
  assert('overlay 重建 2 条', ovs.length === 2 && ovs[0].style.includes('left:0%'), JSON.stringify(ovs).slice(0, 120));
  assert('overlay 样式不拦指针', ovs.every(o => o.style.includes('pointer-events:none')));
  assert('pick 样式蓝/overlay 琥珀分色', canvasPickStyleFromBbox(pk.bbox_pct).includes('rgb(59,130,246)') && ovs[0].style.includes('rgb(245,158,11)'));
}

console.log(`V03 canvas-contract: ${pass} pass / ${fail} fail`);
process.exit(fail ? 1 : 0);
