// scripts/demo/assertions.mjs — PLAN-097 T-09: 目标 VM 行为断言基础设施。
//
// 职责（计划 T-09 行 + §6 业务验证）：P001/P002/P003 三轮业务行为断言、
// 跨启保存（Canvas stop/start 循环 + 磁盘交叉验证）、Canvas 锚点判言
// （零 rect 不造坐标——本模块只走 vnode 动作，坐标臂不存在）、工程未
// 交付不发下一需求的门、P003 上下文来源判言（新会话/无注入/真实读取）。
//
// 协议事实（T-01 探针冻结，baseline「驱动协议要点」）：
//   - 快照行 `<indent><kind> #vnode_N "label" @rect(x,y,w,h)` + placeholder 行；
//     vnode id 每帧重渲染会变 → 一切 action 前必须重拍快照（actByLabel 每次尝试
//     都 fresh snapshot）。
//   - 等锚用短中文词/拉丁 marker（CJK 长文按渲染宽度裁剪）。
//   - 拒收文案见 examples/focusboard-demo/requests/*.md「验收口径」——匹配器
//     用容错关键词集合，UI 形态由 Agent 决定；匹配失败 = 验收失败（响亮）。
//
// 断言纪律（§6）：行为不匹配返回 {ok:false, detail}（由调用方聚合退出码），
// 只有通道/协议错误才抛——不静默绿、不因单条行为失败丢现场。
import fs from 'node:fs';
import path from 'node:path';

// ── 快照树解析（T-01 probe-launch.mjs 解析器的硬化移植） ────────────────

// 解析宿主 VM 快照文本 → 节点树。节点：{kind,id,label,text,placeholder,rect,
// indent,children,parent}。rect 缺失/零值 → rect=null（AC-04：零 rect 不可
// 用于几何，判言层拒绝；本模块动作只走 vnode id，不存在坐标臂）。
export function parseVmTree(text) {
  const nodes = [];
  const stack = []; // {indent, node}
  let root = null;
  for (const rawLine of String(text ?? '').split(/\r?\n/)) {
    const line = rawLine.replace(/\t/g, '  ');
    const h = line.match(/^(\s*)([\w-]+) #(\S+)(?: "((?:[^"\\]|\\.)*)")?(?: @rect\(([^)]*)\))?/);
    if (h) {
      const indent = h[1].length;
      while (stack.length && stack[stack.length - 1].indent >= indent) stack.pop();
      let rect = null;
      if (h[5] !== undefined) {
        const parts = h[5].split(',').map(s => Number(s.trim()));
        if (parts.length === 4 && parts.every(n => Number.isFinite(n))) {
          rect = { x: parts[0], y: parts[1], w: parts[2], h: parts[3] };
          if (rect.w <= 0 || rect.h <= 0) rect = null; // 零 rect = 无几何（不造坐标）
        }
      }
      const node = {
        kind: h[2], id: h[3], label: h[4] ?? '', text: h[4] ?? '',
        placeholder: '', rect, indent, children: [], parent: stack.length ? stack[stack.length - 1].node : null,
      };
      if (stack.length) stack[stack.length - 1].node.children.push(node);
      else if (root) { /* 多根：挂到隐式根序列 */ }
      else root = node;
      nodes.push(node);
      stack.push({ indent, node });
      continue;
    }
    if (!stack.length) continue;
    const top = stack[stack.length - 1].node;
    const ph = line.match(/^\s*placeholder: "(.*)"$/);
    if (ph) top.placeholder = ph[1];
    const st = line.match(/^\s*style: "(.*)"$/);
    if (st) top.style = st[1];
  }
  return { nodes, root };
}

// 节点自身 + 全部后代的 label/placeholder 拼接（卡片/列 subtree 匹配用）。
export function subtreeText(node) {
  const parts = [];
  const walk = n => {
    if (n.label) parts.push(n.label);
    if (n.placeholder) parts.push(n.placeholder);
    for (const c of n.children) walk(c);
  };
  walk(node);
  return parts.join(' | ');
}

export function descendants(node, pred = null) {
  const out = [];
  const walk = n => { for (const c of n.children) { if (!pred || pred(c)) out.push(c); walk(c); } };
  walk(node);
  return out;
}

// 最小匹配卡片：自身 label 命中者优先（其中带 button 后代者为"任务卡"——
// 输入框等同文节点让位）;否则取 subtree 命中的最小深度节点。
export function findCard(nodes, titleRe) {
  const own = nodes.filter(n => titleRe.test(n.label));
  if (own.length) {
    const withButtons = own.filter(n => descendants(n, c => c.kind === 'button').length > 0);
    const pool = withButtons.length ? withButtons : own;
    return pool.sort((a, b) => a.indent - b.indent)[0];
  }
  let best = null;
  for (const n of nodes) {
    if (titleRe.test(subtreeText(n)) && (!best || n.indent < best.indent)) best = n;
  }
  return best;
}

// ── 动作通道（mcp 注入：.call/.snapshot；V02 用桩） ─────────────────────

// 按谓词对最新快照找 vnode 并 autoui_action。每次尝试都重拍快照（vnode
// 重编号规则）;match(node, nodes) 拿到同一拍全节点表(可做卡片内定位);
// VNode not found 重试;目标持续缺失 = 超时错误（响亮）。
export async function actByLabel({ mcp, match, action, value, timeoutMs = 15000, settleMs = 400, label = 'action' }) {
  const deadline = Date.now() + timeoutMs;
  let lastErr = 'target never found';
  while (Date.now() < deadline) {
    let nodes;
    try {
      nodes = parseVmTree(await mcp.snapshot()).nodes;
    } catch (e) { lastErr = e.message; await new Promise(r => setTimeout(r, settleMs)); continue; }
    const target = nodes.find(n => match(n, nodes));
    if (!target) { await new Promise(r => setTimeout(r, settleMs)); continue; }
    try {
      return await mcp.call('autoui_action', { element_id: target.id, action, ...(value === undefined ? {} : { value }) });
    } catch (e) {
      if (!/VNode not found|not in current/i.test(e.message)) throw e;
      lastErr = e.message;
      await new Promise(r => setTimeout(r, settleMs));
    }
  }
  throw new Error(`assertions: ${label} — target not found / stale after ${timeoutMs}ms (last: ${lastErr})`);
}

// ── 跨启保存：Canvas stop/start 循环 ────────────────────────────────────

// 重启判言：generation 必须变化（不变得代 = 未重启，判失败）。
export function restartVerdict(oldGen, newGen) {
  if (!Number.isFinite(oldGen) || !Number.isFinite(newGen)) return { ok: false, reason: `generation not numeric: ${oldGen}→${newGen}` };
  if (newGen === oldGen) return { ok: false, reason: `generation unchanged (${oldGen}) — canvas 未重启，跨启验收无效` };
  return { ok: true, reason: `generation ${oldGen} → ${newGen}` };
}

// 停止 → 启动 → 等新代次 running+帧有效。app_path 取自停止前 status（同
// 一应用重启,不是另起目标）。expected_generation 防旧 stop 杀新目标。
// fetchImpl 可注入（V02 桩序列;默认全局 fetch）。
export async function canvasRestart({ base, token, fetchImpl = fetch }) {
  const headers = { 'content-type': 'application/json', ...(token ? { authorization: `Bearer ${token}` } : {}) };
  const stRes = await fetchImpl(`${base}/api/canvas/status`);
  const st = await stRes.json();
  const oldGen = Number(st?.generation_id ?? 0);
  if (st?.state !== 'running' || !st?.app_path) {
    throw new Error(`assertions: canvasRestart — canvas not running (state=${st?.state}, app_path=${st?.app_path}); 拒绝对未运行应用做跨启验收`);
  }
  const stopRes = await fetchImpl(`${base}/api/canvas/stop`, { method: 'POST', headers, body: JSON.stringify({ generation: oldGen }) });
  if (stopRes.status !== 200) throw new Error(`assertions: canvas stop -> ${stopRes.status}: ${(await stopRes.text()).slice(0, 160)}`);
  const startRes = await fetchImpl(`${base}/api/canvas/start`, { method: 'POST', headers, body: JSON.stringify({ app_path: st.app_path }) });
  const startText = await startRes.text();
  if (startRes.status !== 200) throw new Error(`assertions: canvas start -> ${startRes.status}: ${startText.slice(0, 160)}`);
  const newGen = Number(JSON.parse(startText)?.generation_id ?? NaN);
  // 等 running + 帧有效（旧代次帧不算——generation 必须等于 start 返回值）
  const deadline = Date.now() + 60000;
  while (Date.now() < deadline) {
    await new Promise(r => setTimeout(r, 600));
    try {
      const cur = await (await fetchImpl(`${base}/api/canvas/status`)).json();
      if (cur?.state === 'running' && Number(cur?.generation_id) === newGen && cur?.frame?.valid) break;
    } catch { /* 端口瞬断重试 */ }
  }
  const fin = await (await fetchImpl(`${base}/api/canvas/status`)).json();
  if (!(fin?.state === 'running' && Number(fin?.generation_id) === newGen && fin?.frame?.valid)) {
    throw new Error(`assertions: canvas restart did not reach running+valid frame (state=${fin?.state}, gen=${fin?.generation_id}, want ${newGen})`);
  }
  const verdict = restartVerdict(oldGen, newGen);
  if (!verdict.ok) throw new Error(`assertions: ${verdict.reason}`);
  return { oldGen, newGen, appPath: st.app_path, verdict };
}

// ── 磁盘交叉验证（app 数据落 <repo>/src/front/data，T-01 实证跨启存活） ──

const MAX_DISK_SCAN_BYTES = 2 * 1024 * 1024;

// 扫描 repo 数据目录 → {files:[相对路径], text}（拼接文本,上限截断）。
export function scanRepoData(repoDir, { dataRel = path.join('src', 'front', 'data') } = {}) {
  const root = path.join(repoDir, dataRel);
  const files = [];
  let text = '';
  if (!fs.existsSync(root)) return { files, text, root };
  const walk = dir => {
    for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
      const p = path.join(dir, e.name);
      if (e.isDirectory()) walk(p);
      else if (e.isFile()) {
        const rel = path.relative(repoDir, p).replace(/\\/g, '/');
        files.push(rel);
        if (text.length < MAX_DISK_SCAN_BYTES) {
          text += fs.readFileSync(p, 'utf8').slice(0, MAX_DISK_SCAN_BYTES - text.length) + '\n';
        }
      }
    }
  };
  walk(root);
  return { files, text, root };
}

// 磁盘判言：期望存在的标题必须都能在数据文本中找到（跨启后磁盘侧证据）。
export function diskVerdict(disk, { expect = [], absent = [] } = {}) {
  const errors = [];
  for (const t of expect) if (!disk.text.includes(t)) errors.push(`disk: 缺期望任务 "${t}"`);
  for (const t of absent) if (disk.text.includes(t)) errors.push(`disk: 不应存在 "${t}" 仍在`);
  return { ok: errors.length === 0, errors };
}

// ── UI 观察原语（列/卡片/计数） ─────────────────────────────────────────

const COLUMN_RES = { todo: /^todo\b|待办/i, doing: /^doing\b|进行/i, done: /^done\b|完成/i };

// 找三列头：label 命中列名；返回 {todo, doing, done} 各为节点或 null。
export function columnSections(nodes) {
  const out = { todo: null, doing: null, done: null };
  for (const [name, re] of Object.entries(COLUMN_RES)) {
    out[name] = nodes.find(n => re.test(n.label)) ?? null;
  }
  return out;
}

// 列头计数（需求 001§5：列标题显示当前任务数；002§4：doing 显示 n/3）。
// 从列头 label 或其 subtree 提取首个数字。
export function headerCount(columnNode) {
  if (!columnNode) return null;
  const m = subtreeText(columnNode).match(/(\d+)\s*(?:\/\s*(\d+))?/);
  if (!m) return null;
  return { n: Number(m[1]), cap: m[2] ? Number(m[2]) : null };
}

// 任务标题当前落在哪列（找含该标题的卡片 → 其最近列祖先）。返回列名|null。
export function columnOfTask(nodes, titleRe) {
  const card = findCard(nodes, titleRe);
  if (!card) return null;
  for (const [name, re] of Object.entries(COLUMN_RES)) {
    for (let p = card.parent; p; p = p.parent) {
      if (re.test(p.label)) return name;
    }
  }
  // 列头不是任务祖先（布局扁平）时退化为顺序归属：卡片之前最后一个
  // 列名命中节点（无论缩进）即所属列。
  const idx = nodes.indexOf(card);
  let current = null;
  for (let i = 0; i < idx; i++) {
    for (const [name, re] of Object.entries(COLUMN_RES)) {
      if (re.test(nodes[i].label)) current = name;
    }
  }
  return current;
}

// 卡片上的操作按钮：卡片后代中 kind=button 且 label 命中谓词。
export function cardButton(nodes, titleRe, buttonPred) {
  const card = findCard(nodes, titleRe);
  if (!card) return null;
  return descendants(card, n => n.kind === 'button' && buttonPred.test(n.label))[0] ?? null;
}

// ── 判言（纯函数,V02 直测） ─────────────────────────────────────────────

// P001 单项可见 + 列头计数一致判言。
export function taskVisibleVerdict(nodes, title) {
  const re = new RegExp(escapeRe(title));
  const card = findCard(nodes, re);
  if (!card) return { ok: false, reason: `任务 "${title}" 在快照中不可见` };
  return { ok: true, reason: `visible (node ${card.id})` };
}

// P002 拒绝判言：解释可见（上限关键词+数字 3）AND 第 4 项不在 doing AND 磁盘仍有该任务。
export function wipRejectionVerdict({ snapText, nodes, taskTitle, disk = null, limit = 3 }) {
  const errors = [];
  const reExplain = new RegExp(`(上限|最多|拒绝|不能|无法|WIP|limit).{0,40}${limit}|${limit}.{0,10}(上限|最多|个)`, 'i');
  if (!reExplain.test(snapText)) errors.push('可见解释缺失（未匹配 上限/最多/拒绝/WIP + 3）');
  const col = columnOfTask(nodes, new RegExp(escapeRe(taskTitle)));
  if (col === null) errors.push(`任务 "${taskTitle}" 在 UI 不可见`);
  else if (col === 'doing') errors.push(`任务 "${taskTitle}" 竟在 doing（拒绝未生效）`);
  if (disk) {
    const d = diskVerdict(disk, { expect: [taskTitle] });
    if (!d.ok) errors.push(...d.errors);
  }
  return { ok: errors.length === 0, errors };
}

// P003 无效输入判言：错误解释可见 AND 原设定值仍在 UI。
export function invalidInputVerdict({ snapText, originalLimit, invalidInput }) {
  const errors = [];
  if (!/(无效|非法|错误|不合法|must be|invalid|1[~～到到-]5|介于)/i.test(snapText)) {
    errors.push(`无效输入 ${JSON.stringify(invalidInput)} 后未见可见解释`);
  }
  if (!snapText.includes(String(originalLimit))) {
    errors.push(`原设定 ${originalLimit} 从 UI 消失（无效输入后设定被改动?）`);
  }
  return { ok: errors.length === 0, errors };
}

// Canvas 锚点判言（AC-04）：canvas_pick 载荷必须有 source（file:line）且
// bbox_px 非零（零 rect 造坐标在此拒绝）；置信度必须落载荷，不伪称精确。
export function anchorVerdict(anchorJson) {
  const errors = [];
  if (!anchorJson || typeof anchorJson !== 'object') return { ok: false, errors: ['anchor payload missing'] };
  if (!anchorJson.vnode_id) errors.push('vnode_id missing');
  if (!anchorJson.source || !/:\d+$/.test(String(anchorJson.source))) errors.push(`source anchor missing/not file:line: ${JSON.stringify(anchorJson.source)}`);
  if (!anchorJson.bbox_px || !(anchorJson.bbox_px.w > 0 && anchorJson.bbox_px.h > 0)) errors.push('bbox_px 零/缺失（零 rect 不用于锚点展示）');
  if (!anchorJson.source_confidence) errors.push('source_confidence missing（置信度必须落载荷）');
  return { ok: errors.length === 0, errors };
}

// 工程未交付不发下一需求（AC-10 门）：facts 必须是 contract 校验过的
// RunState（上游 parseRunStateResponse 已验）；非 completed+delivered 即抛。
export function deliveredGate(facts, { planLabel = 'round' } = {}) {
  const pe = facts?.plan_execution ?? {};
  if (facts?.status !== 'completed' || pe.phase !== 'delivered') {
    throw new Error(`assertions: ${planLabel} 未交付（status=${facts?.status} phase=${pe.phase} outcome=${pe.outcome}）——工程未交付不发下一需求`);
  }
  const cps = Object.keys(pe.delivery_checkpoints ?? {});
  const missing = ['prepared', 'landed', 'ledger_refreshed', 'archived', 'cleaned'].filter(k => !cps.includes(k));
  if (missing.length) throw new Error(`assertions: ${planLabel} delivered 但检查点缺件 ${missing.join(',')}（缺件完成不绿）`);
  return true;
}

// ── P003 上下文来源（AC-12） ────────────────────────────────────────────

// 新会话判言：P003 session id 不得出现在前两轮会话集合。
export function p003SessionVerdict(sessionId, priorSessionIds) {
  if (!sessionId) return { ok: false, reason: 'P003 sessionId missing' };
  if (priorSessionIds.includes(sessionId)) return { ok: false, reason: `P003 session ${sessionId} 复用旧会话（聊天注入嫌疑）` };
  return { ok: true, reason: `fresh session ${sessionId}` };
}

// 输入无注入判言：驱动提交文本 == 需求原文（normalize 空白差异）。
export function p003InputVerdict(inputText, requestText) {
  const norm = s => String(s ?? '').replace(/\r\n/g, '\n').trim();
  if (norm(inputText) !== norm(requestText)) {
    const a = norm(inputText); const b = norm(requestText);
    return { ok: false, reason: `P003 会话输入与需求原文不一致（len ${a.length} vs ${b.length}）——上下文注入嫌疑` };
  }
  return { ok: true, reason: 'input == request text verbatim' };
}

// 真实读取证据：RunState.events 窗口内 turn_tool_call（tool_name=read_file）
// 且参数路径命中 specs/plans/wiki —— 命中集合即上下文来源证据；空集合 = 无
// 读取证据（判失败,不用 Agent 声称补图）。
export function p003ReadEvidence(events, { pathRe = /docs[\\/]specs|docs[\\/]plans|docs[\\/]wiki/i } = {}) {
  const paths = new Set();
  for (const ev of Array.isArray(events) ? events : []) {
    if (ev?.type !== 'turn_tool_call') continue;
    if (!/read|view|open/i.test(String(ev.tool_name ?? ''))) continue;
    const argsStr = JSON.stringify(ev.arguments ?? {});
    if (pathRe.test(argsStr)) {
      for (const m of argsStr.matchAll(/(?:docs[\\/](?:specs|plans|wiki)[\w\-\\/.]*)/g)) paths.add(m[0].replace(/\\\\/g, '/'));
    }
  }
  const list = [...paths];
  return {
    observed: list.length > 0,
    paths: list,
    verdict: list.length > 0
      ? { ok: true, reason: `${list.length} distinct spec/plan/wiki reads` }
      : { ok: false, reason: 'run 事件窗内无 specs/plans/wiki 读取证据（AC-12:声称已读不算）' },
  };
}

function escapeRe(s) { return String(s).replace(/[.*+?^${}()|[\]\\]/g, '\\$&'); }

// ── 轮次行为断言编排（V05 用;此处通道可注入,V02 用桩测判言） ────────────

// 单轮断言运行器：checks = [{name, run: async () => ({ok, reason/errors})}]。
// 行为失败记 check 不抛;通道异常记 failed check(含错误文本)后继续后续项,
// 最终 {ok, checks} 由调用方决定退出码。
export async function runChecks(checks, { log = () => {} } = {}) {
  const results = [];
  for (const c of checks) {
    let rec;
    try {
      const r = await c.run();
      rec = { name: c.name, ok: !!r.ok, detail: r.reason ?? (r.errors ?? []).join('; ') };
    } catch (e) {
      rec = { name: c.name, ok: false, detail: `channel error: ${e.message}` };
    }
    results.push(rec);
    log(`[${rec.ok ? 'PASS' : 'FAIL'}] ${rec.name} — ${rec.detail}`);
  }
  return { ok: results.every(r => r.ok), checks: results };
}

// 生成一轮 VM 快照观察集（nodes/snapText/columns）。
export async function observe({ mcp }) {
  const snapText = await mcp.stableSnapshot ? await mcp.stableSnapshot() : await mcp.snapshot();
  const { nodes } = parseVmTree(snapText);
  return { snapText, nodes, columns: columnSections(nodes) };
}

// ── Canvas 锚点通道（AC-04:点选→源码联动;vnode 直选,零坐标） ─────────────

// POST /api/canvas/pick {vnode_id} → anchor JSON。期望代次携带防跨目标。
export async function pickAnchor({ base, vnodeId, generation, fetchImpl = fetch }) {
  const res = await fetchImpl(`${base}/api/canvas/pick`, {
    method: 'POST', headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ vnode_id: vnodeId, ...(generation === undefined ? {} : { expected_generation: generation }) }),
  });
  if (res.status === 409) throw new Error(`assertions: pick 409 generation conflict（先取 /api/canvas/status）`);
  if (res.status === 204) throw new Error(`assertions: pick ${vnodeId} 未命中（204——vnode 不在当前帧）`);
  if (res.status !== 200) throw new Error(`assertions: pick -> ${res.status}`);
  return res.json();
}

// ── 三轮行为断言流（验收口径 = examples/focusboard-demo/requests/*.md; ──
// ── V05 由 run 命令在每轮 delivered 后执行;matcher 容错,命中失败=验收失败）─

// 容错匹配器：UI 形态由 Agent 决定,这里冻结"必须可辨认的操作关键词"。
export const MATCHERS = {
  addInput: n => (n.kind === 'input' || n.kind === 'textarea') && /标题|任务|new task|title|添加/i.test(`${n.label} ${n.placeholder}`),
  addButton: n => n.kind === 'button' && /添加|新增|add|新建/i.test(n.label),
  renameButton: n => n.kind === 'button' && /重命名|改名|rename|编辑|edit/i.test(n.label),
  deleteButton: n => n.kind === 'button' && /删除|移除|delete|remove/i.test(n.label),
  moveToDoing: n => n.kind === 'button' && /→\s*doing|开始|进行|to doing|doing/i.test(n.label),
  moveToDone: n => n.kind === 'button' && /→\s*done|完成|to done|done/i.test(n.label),
  moveBackTodo: n => n.kind === 'button' && /←\s*todo|退回|to todo|todo/i.test(n.label),
  limitInput: n => (n.kind === 'input' || n.kind === 'textarea') && /上限|并行|limit|wip/i.test(`${n.label} ${n.placeholder}`),
  limitSave: n => n.kind === 'button' && /保存|应用|设置|save|apply|set/i.test(n.label),
  confirmButton: n => n.kind === 'button' && /确定|确认|保存|ok|confirm|save/i.test(n.label),
  clearAll: n => n.kind === 'button' && /清空|clear all|全部删除/i.test(n.label),
};

// 快照→动作→再观察小步（flow 共用）。
async function doAct(mcp, matcher, action, value, label) {
  return actByLabel({ mcp, match: matcher, action, value, label });
}

// 卡片内按钮匹配工厂：n 必须是 title 命中卡片的 button 后代且命中 btnMatcher。
// 每拍以(match 收到的)同帧 nodes 找卡片——vnode 重编号安全。
export function inCardOf(titleRe, btnMatcher) {
  return (n, nodes = []) => {
    if (!btnMatcher(n)) return false;
    const card = findCard(nodes, titleRe);
    if (!card) return false;
    let has = false;
    (function walk(c) { if (c === n) has = true; for (const k of c.children) walk(k); })(card);
    return has;
  };
}

// P001 基础看板（CRUD/三列/状态双向/计数一致/跨启保存）。
export function p001Checks({ mcp, repoDir }) {
  const A = '验收任务A', A2 = '验收任务A改', B = '验收任务B';
  const reA = new RegExp(escapeRe(A));
  const reB = new RegExp(escapeRe(B));
  return [
    { name: 'P001 add→todo', run: async () => {
        await doAct(mcp, MATCHERS.addInput, 'type_text', A, 'add-input');
        await doAct(mcp, MATCHERS.addButton, 'press', undefined, 'add-button');
        const { nodes } = await observe({ mcp });
        return taskVisibleVerdict(nodes, A);
      } },
    { name: 'P001 rename', run: async () => {
        await doAct(mcp, inCardOf(reA, MATCHERS.renameButton), 'press', undefined, 'rename');
        // 编辑输入:label 含旧标题或 placeholder 是重命名语义——不匹配加任务输入
        await doAct(mcp, n => (n.kind === 'input' || n.kind === 'textarea')
          && (n.label.includes(A) || /新标题|新名称|rename|new (name|title)/i.test(n.placeholder)), 'type_text', A2, 'rename-input');
        try { await doAct(mcp, MATCHERS.confirmButton, 'press', undefined, 'rename-confirm'); } catch { /* 无确认钮=回车提交 */ }
        const { nodes } = await observe({ mcp });
        const okNew = taskVisibleVerdict(nodes, A2);
        return okNew.ok ? okNew : { ok: false, errors: [`重命名后 "${A2}" 不可见: ${okNew.reason}`] };
      } },
    { name: 'P001 toggle 双向(A2: todo→doing→done→todo)', run: async () => {
        const reA2 = new RegExp(escapeRe(A2));
        for (const [m, want] of [[MATCHERS.moveToDoing, 'doing'], [MATCHERS.moveToDone, 'done'], [MATCHERS.moveBackTodo, 'todo']]) {
          await doAct(mcp, inCardOf(reA2, m), 'press', undefined, `toggle→${want}`);
          const { nodes } = await observe({ mcp });
          const col = columnOfTask(nodes, reA2);
          if (col !== want) return { ok: false, errors: [`切换到 ${want} 失败（实际 ${col}）`] };
        }
        return { ok: true, reason: 'todo→doing→done→todo 双向可达' };
      } },
    { name: 'P001 列头计数一致', run: async () => {
        const { nodes, columns } = await observe({ mcp });
        const errors = [];
        for (const name of ['todo', 'doing', 'done']) {
          const cnt = headerCount(columns[name]);
          if (cnt === null) { errors.push(`${name} 列头无计数`); continue; }
          const actual = descendants(columns[name], n => /card|item|row/i.test(n.kind) || (n.kind === 'button' && false)).length
            || descendants(columns[name], n => MATCHERS.deleteButton(n) || MATCHERS.moveToDoing(n) || MATCHERS.moveToDone(n)).length;
          if (cnt.n !== actual) errors.push(`${name} 列头 ${cnt.n} != 卡片数 ${actual}`);
        }
        return errors.length ? { ok: false, errors } : { ok: true, reason: '三列计数一致' };
      } },
    { name: 'P001 add+delete(B)', run: async () => {
        await doAct(mcp, MATCHERS.addInput, 'type_text', B, 'add-input-b');
        await doAct(mcp, MATCHERS.addButton, 'press', undefined, 'add-button-b');
        let { nodes } = await observe({ mcp });
        const added = taskVisibleVerdict(nodes, B);
        if (!added.ok) return added;
        await doAct(mcp, inCardOf(reB, MATCHERS.deleteButton), 'press', undefined, 'delete-b');
        try { await doAct(mcp, MATCHERS.confirmButton, 'press', undefined, 'delete-confirm'); } catch { /* 无二次确认 */ }
        ({ nodes } = await observe({ mcp }));
        const gone = !findCard(nodes, reB);
        return gone ? { ok: true, reason: '删除后消失' } : { ok: false, errors: [`删除后 "${B}" 仍可见`] };
      } },
    { name: 'P001 跨启保存(restart 后 A2 仍在+磁盘仍有)', run: async () => {
        const before = scanRepoData(repoDir);
        const { nodes: pre } = await observe({ mcp });
        if (!taskVisibleVerdict(pre, A2).ok) return { ok: false, errors: [`重启前 "${A2}" 已不可见(前序步骤失败延续)`] };
        const io = mcp.__restart ?? (async () => { throw new Error('assertions: P001 跨启检查需要 mcp.__restart 注入(run 编排提供 canvasRestart 绑定)'); });
        const r = await io();
        const { nodes } = await observe({ mcp });
        const ui = taskVisibleVerdict(nodes, A2);
        const disk = diskVerdict(scanRepoData(repoDir), { expect: [A2] });
        const diskText = disk.ok ? '' : `; disk=${JSON.stringify(disk.errors)}; files=${JSON.stringify(before.files)}`;
        return ui.ok && disk.ok ? { ok: true, reason: `重启 gen ${r.oldGen}→${r.newGen} 后 UI+磁盘均保留` }
          : { ok: false, errors: [ui.ok ? '' : `UI: ${ui.reason}`, diskText].filter(Boolean) };
      } },
  ];
}

// P002 限制并行（doing=3 拒第 4 项;释放后再进;P001 回归抽查）。
export function p002Checks({ mcp, repoDir }) {
  const T = ['WIP任务1', 'WIP任务2', 'WIP任务3', 'WIP任务4'];
  const reT = i => new RegExp(escapeRe(T[i]));
  const addTask = async title => {
    await doAct(mcp, MATCHERS.addInput, 'type_text', title, `add ${title}`);
    await doAct(mcp, MATCHERS.addButton, 'press', undefined, `add-btn ${title}`);
  };
  const moveToDoing = async (i) => {
    await doAct(mcp, inCardOf(reT(i), MATCHERS.moveToDoing), 'press', undefined, `wip ${T[i]}`);
  };
  return [
    { name: 'P002 setup: 3 任务进 doing', run: async () => {
        for (let i = 0; i < 3; i++) { await addTask(T[i]); await moveToDoing(i); }
        const { nodes } = await observe({ mcp });
        const cols = [0, 1, 2].map(i => columnOfTask(nodes, reT(i)));
        return cols.every(c => c === 'doing') ? { ok: true, reason: '3 任务在 doing' } : { ok: false, errors: `doing 成员异常: ${JSON.stringify(cols)}` };
      } },
    { name: 'P002 第 4 项被拒(解释可见+原状态不变+磁盘保留)', run: async () => {
        await addTask(T[3]);
        let rejected = false;
        try { await moveToDoing(3); } catch { rejected = true; } // 按钮消失/拒绝弹层都算拒绝面
        const { snapText, nodes } = await observe({ mcp });
        if (!rejected && columnOfTask(nodes, reT(3)) === 'doing') {
          return { ok: false, errors: ['第 4 项进入了 doing（限制未生效）'] };
        }
        return wipRejectionVerdict({ snapText, nodes, taskTitle: T[3], disk: scanRepoData(repoDir) });
      } },
    { name: 'P002 释放后再进(移出一个→第 4 项可进)', run: async () => {
        await doAct(mcp, inCardOf(reT(0), MATCHERS.moveToDone), 'press', undefined, 'release-1');
        await moveToDoing(3);
        const { nodes } = await observe({ mcp });
        const col = columnOfTask(nodes, reT(3));
        return col === 'doing' ? { ok: true, reason: '释放额度后可再进入' } : { ok: false, errors: [`释放后第 4 项仍在 ${col}`] };
      } },
    { name: 'P002 P001 回归抽查(add)', run: async () => {
        await addTask('WIP回归R');
        const { nodes } = await observe({ mcp });
        return taskVisibleVerdict(nodes, 'WIP回归R');
      } },
  ];
}

// P003 可配置规则(降限保留/无效输入/跨启保存/回归)。
export function p003Checks({ mcp, repoDir }) {
  const setLimit = async v => {
    await doAct(mcp, MATCHERS.limitInput, 'type_text', String(v), `limit-input ${v}`);
    try { await doAct(mcp, MATCHERS.limitSave, 'press', undefined, `limit-save ${v}`); } catch { /* 输入即生效形态 */ }
  };
  return [
    { name: 'P003 找到上限设置控件', run: async () => {
        const { nodes } = await observe({ mcp });
        const found = nodes.some(MATCHERS.limitInput) || nodes.some(MATCHERS.limitSave);
        return found ? { ok: true, reason: '设置控件可见' } : { ok: false, errors: ['上限设置控件未找到(需求 003§1 要求可配置)'] };
      } },
    { name: 'P003 无效输入(0/6/abc)拒绝且原设定保留', run: async () => {
        const errors = [];
        for (const bad of ['0', '6', 'abc']) {
          await setLimit(bad);
          const { snapText } = await observe({ mcp });
          const v = invalidInputVerdict({ snapText, originalLimit: 3, invalidInput: bad });
          if (!v.ok) errors.push(...v.errors.map(e => `[${bad}] ${e}`));
        }
        return errors.length ? { ok: false, errors } : { ok: true, reason: '0/6/abc 全部拒绝且设定未变' };
      } },
    { name: 'P003 3→2 降限:现有 doing 保留+新增被阻+解释可见', run: async () => {
        // 降限前快照:记当前 doing 成员(降限到 2 以下时必须原列保留)
        const { nodes: pre } = await observe({ mcp });
        const doingMembers = ['WIP任务1', 'WIP任务2', 'WIP任务3', 'WIP任务4', 'WIP回归R']
          .filter(t => columnOfTask(pre.nodes ?? pre, new RegExp(escapeRe(t))) === 'doing');
        await setLimit(2);
        const probe = 'WIP降限探针';
        await doAct(mcp, MATCHERS.addInput, 'type_text', probe, 'add-probe');
        await doAct(mcp, MATCHERS.addButton, 'press', undefined, 'add-probe-btn');
        try {
          await doAct(mcp, inCardOf(new RegExp(escapeRe(probe)), MATCHERS.moveToDoing), 'press', undefined, 'probe→doing');
        } catch { /* 按钮被拒消失也算拒绝面 */ }
        const { snapText, nodes } = await observe({ mcp });
        const errors = [];
        const reProbe = new RegExp(escapeRe(probe));
        if (columnOfTask(nodes, reProbe) === 'doing') errors.push('降限后探针任务进入了 doing（阻止未生效）');
        for (const t of doingMembers) {
          const col = columnOfTask(nodes, new RegExp(escapeRe(t)));
          if (col === null) errors.push(`降限后现有 doing 任务 "${t}" 消失（降限不删任务）`);
          else if (col !== 'doing') errors.push(`降限后 "${t}" 被移出 doing（${col}）——需求要求保留原列`);
        }
        const wip = wipRejectionVerdict({ snapText, nodes, taskTitle: probe, limit: 2 });
        if (!wip.ok) errors.push(...wip.errors.map(e => `[降限拒绝] ${e}`));
        return errors.length ? { ok: false, errors } : { ok: true, reason: `降限后 ${doingMembers.length} 个现有任务保留原列+新增被阻+解释可见` };
      } },
    { name: 'P003 设定 1~5 合法值可保存(2)', run: async () => {
        await setLimit(2);
        const { snapText } = await observe({ mcp });
        return snapText.includes('2') ? { ok: true, reason: '设定 2 已生效(UI 含 2)' } : { ok: false, errors: ['设定 2 未生效'] };
      } },
    { name: 'P003 跨启保存(设定+任务)+P001/P002 回归', run: async () => {
        const io = mcp.__restart ?? (async () => { throw new Error('assertions: P003 跨启检查需要 mcp.__restart 注入'); });
        const r = await io();
        const { snapText, nodes } = await observe({ mcp });
        const errors = [];
        if (!/(上限|limit).*2|2\s*\/\s*2|设定.*2/i.test(snapText)) errors.push('重启后上限设定 2 不可见');
        const ui = taskVisibleVerdict(nodes, 'WIP任务2');
        if (!ui.ok) errors.push(`重启后任务丢失: ${ui.reason}`);
        const disk = diskVerdict(scanRepoData(repoDir), { expect: ['WIP任务2', 'WIP降限探针'] });
        if (!disk.ok) errors.push(...disk.errors);
        return errors.length ? { ok: false, errors } : { ok: true, reason: `重启 gen ${r.oldGen}→${r.newGen} 后设定+任务均保留` };
      } },
  ];
}
