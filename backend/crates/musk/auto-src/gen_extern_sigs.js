// gen_extern_sigs.js — 从 extern_impl.rs 的 pub fn 签名生成 extern_sigs.at
// 用法: node gen_extern_sigs.js < extern_impl.rs > extern_sigs.at
// 规则: &T 形参 → @T (引用，a2r call site 注入 &), String/&str → str, 其余原样占位
const fs = require('fs');
const lines = fs.readFileSync(process.argv[2] || '/dev/stdin', 'utf8').split(/\r?\n/);
const out = ['// extern_sigs.at — Plan 384 A3 sidecar: extern_impl.rs glue-layer signatures.',
             '// Only @T params (Rust &T) matter to a2r (triggers &arg injection at call sites).',
             '// Non-reference params use loose placeholder types (str/Value/int) — type name',
             '// itself is irrelevant; only reference-ness (presence of @) drives injection.',
             ''];

function mapParam(rustSig) {
  // rustSig 形如 "_s: &T" 或 "_u: String" 或 "_h: axum::http::HeaderMap"
  // 返回 Auto 形参声明 "name typ"
  const m = rustSig.match(/^(\w+):\s*(.+)$/);
  if (!m) return null;
  const [, name, rustTy] = m;
  const t = rustTy.trim();
  // &T (引用) → @T
  if (t.startsWith('&')) {
    const inner = t.replace(/^&mut\s+/, '').replace(/^&/, '').trim();
    // 简化 inner: 取第一个标识符
    const base = inner.replace(/[<>().*]/g, '').split(/[\s,]/)[0] || 'T';
    return `${name} @${base === 'str' ? 'str' : 'T'}`;
  }
  // String / &str → str
  if (t === 'String' || t === '&str') return `${name} str`;
  // Value → Value
  if (/^Value$/.test(t) || /serde_json::Value/.test(t)) return `${name} Value`;
  // bool/u32/u64/i32/f64 等 → int/float/bool
  if (/^bool$/.test(t)) return `${name} bool`;
  if (/^u?\d+$/.test(t) || /^i\d+$/.test(t)) return `${name} int`;
  if (/^f\d+$/.test(t)) return `${name} float`;
  // 泛型 T/U/V/W 或具名类型 → 占位（非引用，类型名不重要）
  return `${name} Value`;
}

function mapRet(rustRet) {
  // 返回类型简化（对 a2r 注入无关紧要，但 fn 声明需要）
  const t = rustRet.trim();
  if (t === '()') return '';
  if (t === 'String' || t === '&str') return ' str';
  if (/^bool$/.test(t)) return ' bool';
  if (/^u?\d+$/.test(t) || /^i\d+$/.test(t)) return ' int';
  if (/^f\d+$/.test(t)) return ' float';
  return ' Value';
}

for (const line of lines) {
  // 匹配 pub (async )?fn name<T,U>(params) -> Ret 或 pub (async )?fn name(params) -> Ret
  const m = line.match(/^pub\s+(async\s+)?fn\s+(\w+)\s*<[^>]*>\s*\(([^)]*)\)\s*(?:->\s*(.+?))?\s*\{/);
  const m2 = !m && line.match(/^pub\s+(async\s+)?fn\s+(\w+)\s*\(([^)]*)\)\s*(?:->\s*(.+?))?\s*\{/);
  const mm = m || m2;
  if (!mm) continue;
  const [, isAsync, name, paramsStr, ret] = mm;
  // 跳过 NoDaemonClient impl / StubRole（非 extern fn）——它们在文件末尾且不是 pub fn 平级
  if (name === 'complete' || name === 'new') continue;
  const params = paramsStr.split(',').map(s => s.trim()).filter(Boolean);
  const autoParams = params.map(mapParam).filter(Boolean).join(', ');
  const asyncKw = isAsync ? '~' : '';
  let retPart = ret ? mapRet(ret) : '';
  // NOTE: async fns are declared WITHOUT `~` here, because the .at sources
  // already write explicit `.await` at the call sites — a `~` return would
  // make a2r inject a *second* `.await` (→ `.await.await`). Only the param
  // reference-ness (the `@` markers) matters for call-site injection.
  out.push(`fn ${name}(${autoParams})${retPart} {}`);
}
fs.writeFileSync(process.argv[3] || '/dev/stdout', out.join('\n') + '\n');
