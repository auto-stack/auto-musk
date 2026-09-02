#!/usr/bin/env node
// vm-safe-lint.mjs — PLAN-057 T1：VM-safe .at 子集静态门禁
//
// 五模式（对应 KNOWN-DEBT「VM/TS 语义等价性缺陷族」①–⑥）：
//   P1 新键赋值启发式    —— 点号新键写入在 VM 为 RuntimeError（缺陷族①；
//                          T2 根修 SET_FIELD 插入语义后 JS 同语义合法化）
//   P2 for-in 直接调用源 —— 迭代源为调用表达式时 VM 静默零迭代（缺陷族②）
//   P3 web 内建调用      —— 调用名不在 VM 已实现白名单（缺陷族③：静默 None。
//                          白名单=tmp/vmprobe/wl_probe*.at 实机探针实证，
//                          T6 落地 natives 后增补）
//   P4 字符接收者        —— for-in 字符串循环变量直接调方法落 None（缺陷族④）
//   P5 Array.isArray     —— VM 恒 None（缺陷族③特化单列：JS 恒布尔语义下
//                          None 使真列表也走 else 臂，静默分叉最阴险）
//
// 豁免：命中行或其上一行带 `// vm-safe-allow <原因>`（行级，审计留痕）。
// P1 启发式分层（外部对象键集静态不可知，按 census⑥ 口径分层）：
//   - 接收者是本 fn 可见对象字面量：字面量缺该键 → 命中（真新键，可静态证伪）
//   - 接收者是 for-in 循环变量：fn 内先读后写 → 放行，否则命中（外部 schema）
//   - 其余接收者（fn 参数/中间局部）：放行（后端契约既有键，人为判定域）
// 退出码：存在未豁免命中 → 1；否则 0。独立执行，不挂 package.json。

import { readdirSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { join, relative } from 'node:path';

const ROOT = fileURLToPath(new URL('..', import.meta.url));
const FRONT = join(ROOT, 'src', 'front');

// ── VM 已实现 web 内建白名单（2026-09-02 实机探针实证；PLAN-057 T6 后增补）──
// 注：JSON.parse 数组形态已在 T6 接线（shim_str_len 堆句柄判定扩展）。
const VM_IMPLEMENTED = new Set([
  'Math.abs', 'Math.min', 'Math.max', 'Math.floor', 'Math.ceil',
  'Math.round', 'Math.sqrt', 'Math.random', 'Math.trunc', 'Math.imul',
  'Object.keys', 'Object.values',
  'JSON.parse', 'JSON.stringify',
  'Array.isArray',
]);

const ALLOW_MARK = /\/\/\s*vm-safe-allow\b/;
const BUILTIN_CALL = /(?:Math|JSON|Object|Array)\.[A-Za-z_]\w*\s*\(/g;
// 点号字段赋值：接收者为普通标识符（排除 .state 前导点路径与 a.b.c 嵌套），
// 排除 == / != / <= / >= 比较形态。
const FIELD_WRITE = /(^|[^\w.])((?:let |var )?)([A-Za-z_]\w*)\.([A-Za-z_]\w*)(\s*)=(?![=>])(?!\s*[<>])\s*/g;
const FOR_IN = /for\s+([A-Za-z_]\w*)\s+in\s+([^{]+?)\s*\{/g;
const FN_SIG = /^fn\s+([A-Za-z_]\w*)\s*\(([^)]*)\)\s/;
const LITERAL_START = /(?:let|var)\s+([A-Za-z_]\w*)(?:\s+(?:map|obj))?\s*=\s*\{/;

/** 逐行剥离注释（引号感知；块注释跨行）。返回 [codeLines, rawLines]。 */
function stripComments(rawLines) {
  const code = [];
  let inBlock = false;
  for (const raw of rawLines) {
    let out = '';
    let quote = null; // '"' | "'" | null
    for (let i = 0; i < raw.length; i++) {
      const c = raw[i];
      if (inBlock) {
        if (c === '*' && raw[i + 1] === '/') { inBlock = false; i++; }
        continue;
      }
      if (quote) {
        out += c;
        if (c === '\\') { out += raw[i + 1] ?? ''; i++; }
        else if (c === quote) quote = null;
        continue;
      }
      if (c === '"' || c === "'") { quote = c; out += c; continue; }
      if (c === '/' && raw[i + 1] === '/') break; // 行注释
      if (c === '/' && raw[i + 1] === '*') { inBlock = true; i++; continue; }
      out += c;
    }
    code.push(out.trimEnd());
  }
  return code;
}

/** 收集 .at 文件。 */
function collectAtFiles(dir) {
  const out = [];
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const p = join(dir, entry.name);
    if (entry.isDirectory()) out.push(...collectAtFiles(p));
    else if (entry.name.endsWith('.at')) out.push(p);
  }
  return out;
}

function lintFile(absPath) {
  const rawLines = readFileSync(absPath, 'utf8').split(/\r?\n/);
  const code = stripComments(rawLines);
  const hits = []; // {pattern, line, text, reason}

  // fn 上下文状态
  let fnStart = 0;
  const literals = new Map(); // name -> Set(keys)，本 fn 可见局部字面量
  const forInVars = new Set(); // 本 fn 的 for-in 循环变量名
  const charFrames = []; // {v, endDepth} —— for-in 字符循环活跃帧
  let depth = 0;
  let literalCapture = null; // {name, keys, depthAtStart}

  const rel = relative(ROOT, absPath).replaceAll('\\', '/');
  const exempted = (i) =>
    ALLOW_MARK.test(rawLines[i] ?? '') || ALLOW_MARK.test(rawLines[i - 1] ?? '');

  const addHit = (pattern, i, reason) => {
    if (exempted(i)) return;
    hits.push({ pattern, file: rel, line: i + 1, text: rawLines[i].trim(), reason });
  };

  for (let i = 0; i < code.length; i++) {
    const line = code[i];
    const raw = rawLines[i];
    if (FN_SIG.test(line)) {
      fnStart = i;
      literals.clear();
      forInVars.clear();
      charFrames.length = 0;
      literalCapture = null;
    }

    // 对象字面量跨行收集（浅层：键名扫描直到括号配平）
    if (!literalCapture && LITERAL_START.test(line)) {
      const name = line.match(LITERAL_START)[1];
      literalCapture = { name, keys: new Set(), opened: line.split('{').length - 1, closed: line.split('}').length - 1 };
      collectKeys(line, literalCapture.keys);
    } else if (literalCapture) {
      collectKeys(line, literalCapture.keys);
      literalCapture.opened += line.split('{').length - 1;
      literalCapture.closed += line.split('}').length - 1;
    }
    if (literalCapture && literalCapture.closed >= literalCapture.opened) {
      literals.set(literalCapture.name, literalCapture.keys);
      literalCapture = null;
    }

    // P3/P5：web 内建调用白名单（P5=isArray 特化：白名单外才报——恒 None
    // 分叉雷仅在未实现时成立，T6 native 落地后isArray 已恒布尔）
    for (const m of line.matchAll(BUILTIN_CALL)) {
      const name = m[0].replace(/\s*\($/, '').replace(/\s*\($/, '');
      const call = name;
      if (!VM_IMPLEMENTED.has(call)) {
        if (call === 'Array.isArray') {
          addHit('P5', i, 'Array.isArray VM 恒 None：真列表也走 else 臂（静默分叉雷）');
        } else {
          addHit('P3', i, `${call} 不在 VM 已实现白名单（静默 None 桩）`);
        }
      }
    }

    // P2：for-in 直接调用源（仅调用表达式；括号分组/`(x ?? [])` 实测迭代正常——wl_probe4）
    for (const m of line.matchAll(FOR_IN)) {
      const [, v, src] = m;
      if (/\w+\s*\(/.test(src)) addHit('P2', i, `for-in 源含调用表达式（${src.trim()}）→ VM 静默零迭代`);
    }

    // P1：点号字段赋值（分层启发式）
    for (const m of line.matchAll(FIELD_WRITE)) {
      const recv = m[3];
      const field = m[4];
      if (literals.has(recv)) {
        if (!literals.get(recv).has(field)) {
          addHit('P1', i, `字面量 {${recv}} 缺键 ${field}：真新键写入，VM RuntimeError`);
        }
      } else if (forInVars.has(recv)) {
        // 先读证据：== 比较算读，仅排除单 = 赋值形态自身
        const priorRead = new RegExp(`${recv}\\.${field}\\b(?!\\s*=(?!=))`);
        let read = false;
        for (let j = fnStart; j < i; j++) {
          if (priorRead.test(code[j])) { read = true; break; }
        }
        if (!read) addHit('P1', i, `for-in 变量 ${recv}.${field} 写前无读：新键风险（外部 schema 键集不可知）`);
      }
      // 其余接收者：后端契约既有键判定域，放行（census⑥ 口径）
    }

    // for-in 循环变量登记 + 字符循环帧（源为 str 型参数）
    for (const m of line.matchAll(FOR_IN)) {
      const [, v, src] = m;
      forInVars.add(v);
      const srcId = src.trim().match(/^[A-Za-z_]\w*$/)?.[0];
      if (srcId && strParams.has(srcId)) {
        charFrames.push({ v, endDepth: depth + countAfter(line, m.index, '{', '}') });
      }
    }

    // P4：字符循环变量直接调方法（仅未修方法——T5 后 char_code_at/charCodeAt
    // 走 Char 码点恒等臂已同值；其余方法在裸字符接收者上仍静默 None）
    depth += (line.split('{').length - 1) - (line.split('}').length - 1);
    // endDepth=for 行配平后的深度；严格 < 才弹帧（for 行本身 depth===endDepth 保活，
    // 循环收口行 depth 回到 endDepth-1 才失效；单行循环体净配平 0 会悬挂到 fn 尾，
    // 现源无此形态，出现时由豁免机制兜底）
    while (charFrames.length && depth < charFrames[charFrames.length - 1].endDepth) charFrames.pop();
    const P4_FIXED_METHODS = new Set(['char_code_at', 'charCodeAt']);
    for (const f of charFrames) {
      const call = new RegExp(`(?<![\\w.])${f.v}\\.([A-Za-z_]\\w*)\\s*\\(`, 'g');
      for (const cm of line.matchAll(call)) {
        if (!P4_FIXED_METHODS.has(cm[1])) {
          addHit('P4', i, `字符循环变量 ${f.v}.${cm[1]}() 直调 → VM 落 None（缺陷族④）`);
        }
      }
    }
  }
  return hits;
}

function collectKeys(line, into) {
  for (const m of line.matchAll(/(?:[{,]\s*)(?:([A-Za-z_]\w*)|"([^"]+)")\s*:/g)) {
    into.add(m[1] ?? m[2]);
  }
}

function countAfter(line, fromIdx, open, close) {
  let n = 0;
  for (let i = fromIdx; i < line.length; i++) {
    if (line[i] === open) n++;
    else if (line[i] === close) n--;
  }
  return n;
}

// str 型参数表（P4 字符循环判定）：fn 签名 (a str, b obj) → {'a'}
// 简化：文件级收集（跨 fn 极小概率误报，豁免机制兜底）。
const strParams = new Set();

function collectStrParams(absPath) {
  strParams.clear();
  const code = stripComments(readFileSync(absPath, 'utf8').split(/\r?\n/));
  for (const line of code) {
    const m = FN_SIG.exec(line);
    if (!m) continue;
    for (const p of m[2].split(',')) {
      if (/^\s*[A-Za-z_]\w*\s+str\s*$/.test(p)) strParams.add(p.trim().split(/\s+/)[0]);
    }
  }
}

// ── main ──
const files = collectAtFiles(FRONT).sort();
const all = [];
for (const f of files) {
  collectStrParams(f);
  all.push(...lintFile(f));
}

const order = ['P1', 'P2', 'P3', 'P4', 'P5'];
const titles = {
  P1: 'P1 新键赋值（点号新键 → VM RuntimeError，缺陷族①）',
  P2: 'P2 for-in 直接调用源（静默零迭代，缺陷族②）',
  P3: 'P3 web 内建调用不在 VM 白名单（静默 None，缺陷族③）',
  P4: 'P4 字符接收者直调方法（落 None，缺陷族④）',
  P5: 'P5 Array.isArray（恒 None 静默分叉，缺陷族③特化）',
};
let red = 0;
for (const p of order) {
  const group = all.filter((h) => h.pattern === p);
  if (!group.length) {
    console.log(`✓ ${titles[p]} —— 0 命中`);
    continue;
  }
  red += group.length;
  console.log(`✗ ${titles[p]} —— ${group.length} 命中`);
  for (const h of group) console.log(`   ${h.file}:${h.line}  ${h.reason}\n   | ${h.text}`);
}
const exemptCount = countExemptions(files);
console.log(`\n小计：未豁免命中 ${red}；vm-safe-allow 豁免行 ${exemptCount}（豁免行见源码留痕）`);
if (red > 0) {
  console.log('vm-safe-lint：FAIL（存在未豁免命中）');
  process.exit(1);
}
console.log('vm-safe-lint：PASS');

function countExemptions(files) {
  let n = 0;
  for (const f of files) {
    const lines = readFileSync(f, 'utf8').split(/\r?\n/);
    for (const l of lines) if (ALLOW_MARK.test(l)) n++;
  }
  return n;
}
