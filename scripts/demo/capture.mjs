// scripts/demo/capture.mjs — PLAN-097 T-07: 采集模块（宿主 VM / 目标 Canvas）。
//
// 合同（计划 §5.5）：PNG 是实际运行截图（原图直存，不重绘不拼接）；采集
// 前后读身份做二次核验（拒绝跨工作区/跨计划/重启代次的错误归属）；等
// 稳定帧再拍；截图超时 = 显式失败（不静默跳过）。产出 capture 记录：
//   { capture_id, source: 'host-vm'|'target-canvas', at(UTC), viewport,
//     png(相对路径), sha256, bytes, width, height, kind: 'live'|
//     'completed-stage-record', stage_ref?, snapshot_sha256 }
// mcp 接口注入（VmSession.mcp 同形：call/snapshot/stableSnapshot），
// 测试以桩注入（V02:超时/身份漂移注入全部失败）。
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';

const sha256 = buf => crypto.createHash('sha256').update(buf).digest('hex');

// PNG 可读性核验：签名 + IHDR 尺寸（原图可读的最低断言；不重绘不缩放）。
export function pngInfo(buf) {
  if (!buf || buf.length < 24 || buf.readUInt32BE(0) !== 0x89504e47) {
    throw new Error('capture: not a PNG (bad signature)');
  }
  return { width: buf.readUInt32BE(16), height: buf.readUInt32BE(20) };
}

// 身份二次核验：采集前后两次快照都必须包含全部期望标记（跨身份/重启
// 代次的错误归属在此拒绝）。
export function verifyIdentity(before, after, markers) {
  const missing = markers.filter(m => !before.includes(m) || !after.includes(m));
  if (missing.length) {
    throw new Error(`capture: identity check failed — markers missing ${JSON.stringify(missing)}`);
  }
  return true;
}

async function writeCapture(evidenceDir, rec, buf) {
  const { width, height } = pngInfo(buf);
  const pngPath = path.join(evidenceDir, `${rec.capture_id}.png`);
  fs.writeFileSync(pngPath, buf);
  return { ...rec, png: path.basename(pngPath), sha256: sha256(buf), bytes: buf.length, width, height };
}

// 宿主 VM 采集：稳定帧 → 身份双验 → autoui_screenshot → 原图搬运。
export async function captureHostVm({ mcp, evidenceDir, captureId, markers, kind = 'live', stageRef = null, timeoutMs = 30000 }) {
  const before = await mcp.stableSnapshot({ timeoutMs: 15000 });
  const shotText = await Promise.race([
    mcp.call('autoui_screenshot', {}),
    new Promise((_, rej) => setTimeout(() => rej(new Error(`capture: screenshot timeout after ${timeoutMs}ms`)), timeoutMs)),
  ]);
  const m = shotText.match(/Screenshot saved to: (.+)/);
  if (!m) throw new Error(`capture: screenshot unparsable: ${shotText.slice(0, 120)}`);
  const src = m[1].trim();
  const buf = fs.readFileSync(src);
  const after = await mcp.snapshot();
  verifyIdentity(before, after, markers);
  fs.rmSync(src, { force: true }); // 原图搬运进 evidence（不留 tmp 残影）
  const rec = {
    capture_id: captureId, source: 'host-vm', at: new Date().toISOString(),
    viewport: '1280x800', kind, stage_ref: stageRef,
    snapshot_sha256: sha256(Buffer.from(before, 'utf8')),
  };
  return writeCapture(evidenceDir, rec, buf);
}

// 目标 Canvas 采集：status 身份核验（generation 必须等于预期——跨计划/
// 重启代次在此拒绝）→ /api/canvas/frame?generation= PNG 直出。
export async function captureTargetCanvas({ base, evidenceDir, captureId, generation, kind = 'live', stageRef = null }) {
  const stRes = await fetch(`${base}/api/canvas/status`);
  const st = await stRes.json();
  if (Number(st?.generation_id ?? 0) !== Number(generation)) {
    throw new Error(`capture: cross-identity — canvas generation ${st?.generation_id} != expected ${generation}`);
  }
  const res = await fetch(`${base}/api/canvas/frame?generation=${generation}`);
  if (res.status === 409) throw new Error(`capture: cross-identity — frame 409 (generation/seq mismatch)`);
  if (res.status !== 200) throw new Error(`capture: frame ${res.status}`);
  const buf = Buffer.from(await res.arrayBuffer());
  const rec = {
    capture_id: captureId, source: 'target-canvas', at: new Date().toISOString(),
    viewport: `${pngInfo(buf).width}x${pngInfo(buf).height}`, kind, stage_ref: stageRef,
  };
  return writeCapture(evidenceDir, rec, buf);
}
