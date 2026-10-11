#!/usr/bin/env node
// canvas-layout-contract.mjs — PLAN-102 T-03: 停靠布局纯函数合同门。
//
// 被测对象 = 生成的 ext 生产模块（gen/front/vue/src/ext/src/front/
// canvas_layout_helpers.ts，.at 单源经 use.web.fn 链产出）——不复制算法。
// 覆盖：钳制三族（外/水平/底部）、窄容器降级判定（含 0 维守卫）、拖动
// 结果换算（right 反向/钳制边界）、键盘步进（Home 复位）、偏好编解码
//（畸形段回落）、动态尺寸声明串（W-03-1 裁定形态——VM split_css_
// declarations 的 ';' 存在性与 width/height 键、web :style 真 CSS 有效性
// 都锚定在本串的逐字符形态上，故做全等断言）、VM 派生源两函数。
//
// T-03 门面：`node scripts/ui-parity/canvas-layout-contract.mjs` 预期 0。
// 退出码：全过 0，任一失败 1。收据：stdout 汇总行。

import { pathToFileURL } from 'node:url';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const HELPERS_TS = path.join(ROOT, 'gen', 'front', 'vue', 'src', 'ext', 'src', 'front', 'canvas_layout_helpers.ts');

const {
  canvasClampDockW, canvasClampDockH, canvasClampOuterW,
  canvasDockNarrow, canvasDockNarrowH, canvasDockMode,
  canvasDockWDecl, canvasDockHDecl, canvasOuterWDecl,
  canvasWbEstH, canvasOuterContainerW, canvasLogicalToPx,
  canvasDockDragSize, canvasDockNudgeSize,
  canvasLayoutPrefKey, canvasLayoutPrefEncode, canvasLayoutPrefDecode,
} = await import(pathToFileURL(HELPERS_TS).href);

let pass = 0, fail = 0;
const assert = (name, ok, detail) => {
  if (ok) { pass++; console.log(`  ✓ ${name}${detail ? ` — ${detail}` : ''}`); }
  else { fail++; console.error(`  ✗ ${name}${detail ? ` — ${detail}` : ''}`); }
};

// ── 钳制三族 ─────────────────────────────────────────────────────────────
console.log('[clamp]');
{
  // 水平：min 180，hi = container-240（不足收敛 180）。
  assert('dockW 常规', canvasClampDockW(900, 240) === 240);
  assert('dockW 上界 = container-240', canvasClampDockW(900, 700) === 660, `=${canvasClampDockW(900, 700)}`);
  assert('dockW 窄容器收敛 min', canvasClampDockW(400, 240) === 180, `=${canvasClampDockW(400, 240)}`);
  assert('dockW 零容器', canvasClampDockW(0, 500) === 180);
  assert('dockW 下界', canvasClampDockW(1000, 100) === 180);
  // 底部：min 140，hi = container-180。
  assert('dockH 常规', canvasClampDockH(800, 240) === 240);
  assert('dockH 上界 = container-180', canvasClampDockH(800, 700) === 620, `=${canvasClampDockH(800, 700)}`);
  assert('dockH 窄容器收敛 min', canvasClampDockH(300, 240) === 140, `=${canvasClampDockH(300, 240)}`);
  // 外框：min 240，hi = container-320。
  assert('outerW 常规', canvasClampOuterW(1200, 640) === 640);
  assert('outerW 上界 = container-320', canvasClampOuterW(1200, 1000) === 880, `=${canvasClampOuterW(1200, 1000)}`);
  assert('outerW 窄容器收敛 min', canvasClampOuterW(500, 640) === 240, `=${canvasClampOuterW(500, 640)}`);
  assert('outerW 下界', canvasClampOuterW(2000, 100) === 240);
}

// ── 窄容器降级判定 ───────────────────────────────────────────────────────
console.log('[dock-mode]');
{
  // 水平阈：240+180=420。
  assert('left 宽容 dock', canvasDockMode('left', 420, 800) === 'dock');
  assert('left 窄 tabs', canvasDockMode('left', 419, 800) === 'tabs');
  assert('right 同阈', canvasDockMode('right', 419, 800) === 'tabs');
  // 垂直阈：180+140=320（bottom 只看高）。
  assert('bottom 高足 dock', canvasDockMode('bottom', 100, 320) === 'dock');
  assert('bottom 高窄 tabs', canvasDockMode('bottom', 9999, 319) === 'tabs');
  // hidden 恒 dock；0 维守卫（宿主未报尺寸不降级）。
  assert('hidden 恒 dock', canvasDockMode('hidden', 0, 0) === 'dock');
  assert('0 维容器不降级', canvasDockMode('left', 0, 0) === 'dock');
  assert('半维（宽知高缺）不降级', canvasDockMode('bottom', 0, 0) === 'dock');
  // 窄判定原语直测。
  assert('narrow 阈', canvasDockNarrow(419) === true && canvasDockNarrow(420) === false);
  assert('narrowH 阈', canvasDockNarrowH(319) === true && canvasDockNarrowH(320) === false);
}

// ── 拖动结果换算 ─────────────────────────────────────────────────────────
console.log('[drag]');
{
  assert('left 正向变宽', canvasDockDragSize('left', 'h', 900, 600, 240, 60) === 300);
  assert('right 反向（正位移=变窄）', canvasDockDragSize('right', 'h', 900, 600, 240, 60) === 180, `=${canvasDockDragSize('right', 'h', 900, 600, 240, 60)}`);
  assert('right 负位移变宽', canvasDockDragSize('right', 'h', 900, 600, 300, -60) === 360);
  assert('拖动上界钳制', canvasDockDragSize('left', 'h', 900, 600, 240, 9999) === 660, `=${canvasDockDragSize('left', 'h', 900, 600, 240, 9999)}`);
  assert('拖动下界钳制', canvasDockDragSize('left', 'h', 900, 600, 240, -9999) === 180);
  assert('bottom 高拖动', canvasDockDragSize('bottom', 'b', 900, 600, 240, 100) === 340);
  assert('bottom 上界 = container-180', canvasDockDragSize('bottom', 'b', 900, 600, 240, 9999) === 420);
}

// ── 键盘步进 ─────────────────────────────────────────────────────────────
console.log('[nudge]');
{
  assert('±10 步进', canvasDockNudgeSize('left', 'h', 900, 600, 240, 10) === 250
    && canvasDockNudgeSize('left', 'h', 900, 600, 240, -10) === 230);
  assert('Home 复位初值（无关 origin）', canvasDockNudgeSize('left', 'h', 900, 600, 9999, 0) === 240);
  assert('bottom Home 复位', canvasDockNudgeSize('bottom', 'b', 900, 600, 9999, 0) === 240);
  assert('right 步进反向', canvasDockNudgeSize('right', 'h', 900, 600, 240, 10) === 230);
  assert('窄容器 Home 钳制', canvasDockNudgeSize('left', 'h', 400, 600, 240, 0) === 180);
}

// ── 偏好编解码 ───────────────────────────────────────────────────────────
console.log('[pref]');
{
  assert('key 缺省工作区', canvasLayoutPrefKey('') === 'canvas.layout.default');
  assert('key 具名工作区', canvasLayoutPrefKey('ws-a') === 'canvas.layout.ws-a');
  const enc = canvasLayoutPrefEncode('right', 300, 260, 700);
  assert('encode 形态', enc === 'right|300|260|700', enc);
  const rt = canvasLayoutPrefDecode(enc);
  assert('roundtrip', rt.dock === 'right' && rt.dock_w === 300 && rt.dock_h === 260 && rt.outer_w === 700);
  const e = canvasLayoutPrefDecode('');
  assert('空串回落缺省', e.dock === 'left' && e.dock_w === 240 && e.dock_h === 240 && e.outer_w === 640);
  const j = canvasLayoutPrefDecode('junk');
  assert('非法段数回落缺省', j.dock === 'left' && e.dock_w === 240);
  const m = canvasLayoutPrefDecode('top|-5|x|abc');
  assert('域外 dock 回落 left', m.dock === 'left', `dock=${m.dock}`);
  assert('负宽回落初值', m.dock_w === 240, `w=${m.dock_w}`);
  assert('非数高回落初值', m.dock_h === 240, `h=${m.dock_h}`);
  assert('非数外框回落初值', m.outer_w === 640, `outer=${m.outer_w}`);
  const b = canvasLayoutPrefDecode('bottom|181|141|241');
  assert('边界值保留', b.dock === 'bottom' && b.dock_w === 181 && b.dock_h === 141 && b.outer_w === 241);
}

// ── 动态尺寸声明串（W-03-1 裁定形态——全等锚定） ─────────────────────────
console.log('[decl]');
{
  // VM split_css_declarations 需要 ';' 存在 + width/height 键名；web :style
  // 需要合法 CSS。整串全等 = 双轨出程形态的逐字符合同。
  assert('dockWDecl(240)', canvasDockWDecl(240) === 'width:240px;height:100%;overflow:hidden;flex-shrink:0', canvasDockWDecl(240));
  assert('dockWDecl(333) 动态值', canvasDockWDecl(333) === 'width:333px;height:100%;overflow:hidden;flex-shrink:0');
  assert('dockHDecl(300) 高驱动', canvasDockHDecl(300) === 'height:300px;width:100%;overflow:hidden;flex-shrink:0', canvasDockHDecl(300));
  assert('outerWDecl(640)', canvasOuterWDecl(640) === 'width:640px;height:100%;overflow:hidden;flex-shrink:0');
}

// ── VM 派生源 ────────────────────────────────────────────────────────────
console.log('[vm-derive]');
{
  // chrome=29（工作台自身停靠条；切片④实机校准——面板根即全窗高）。
  assert('wb 高派生 = 窗口-29', canvasWbEstH(680) === 651, `=${canvasWbEstH(680)}`);
  assert('wb 高派生下限 1', canvasWbEstH(0) === 1 && canvasWbEstH(29) === 1);
  assert('外框容器 = 视口-240', canvasOuterContainerW(1280) === 1040, `=${canvasOuterContainerW(1280)}`);
  assert('外框容器下限 240', canvasOuterContainerW(300) === 240 && canvasOuterContainerW(400) === 240);
}

// ── 逻辑域→像素换算（分隔条 move 增量通道） ─────────────────────────────
console.log('[logical-to-px]');
{
  // 全域 = 容器宽：4096 → container。
  assert('全域映射', canvasLogicalToPx(4096, 640) === 640, `=${canvasLogicalToPx(4096, 640)}`);
  assert('半域映射', canvasLogicalToPx(2048, 640) === 320);
  assert('负向', canvasLogicalToPx(-2048, 640) === -320);
  assert('零', canvasLogicalToPx(0, 640) === 0);
  // 探针算例：wb=640 下拖宽 60px → 逻辑 384。
  assert('探针算例 60px', canvasLogicalToPx(384, 640) === 60, `=${canvasLogicalToPx(384, 640)}`);
  assert('小值截断方向', canvasLogicalToPx(100, 640) === 15, `=${canvasLogicalToPx(100, 640)}`);
}

console.log(`\n[canvas-layout-contract] ${pass} pass / ${fail} fail`);
process.exit(fail === 0 ? 0 : 1);
