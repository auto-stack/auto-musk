// Classify cross-module bare calls: method-context (inside ext/impl/type block)
// vs top-level-fn context. Only method-context calls need hoisted shims.
const fs = require('fs');
const path = require('path');
const dir = __dirname;
const files = fs.readdirSync(dir).filter(f => f.endsWith('.at'));
const KEYWORDS = new Set(['if','else','for','while','match','return','fn','let','print','println','dep','use','impl','ext','pub','struct','enum','type','import','true','false','nil','and','or','not','in','as','spawn','await','async','defer','move','loop','break','continue','main']);
const AMBIG = { drive_run: 'relay_driver', build_agent_with_context: 'lib' };

// VM-loaded module set: vm_entry + transitive module uses (incl. new item imports)
function loadedSet() {
  const load = new Set();
  const stack = ['vm_entry'];
  while (stack.length) {
    const stem = stack.pop();
    if (load.has(stem)) continue;
    const f = path.join(dir, stem + '.at');
    if (!fs.existsSync(f)) continue;
    load.add(stem);
    const text = fs.readFileSync(f, 'utf8');
    for (const m of text.matchAll(/^use ([a-z_][a-z0-9_]*)/gm)) stack.push(m[1]);
  }
  return load;
}
const LOADED = loadedSet();

const mods = {};
for (const f of files) {
  const stem = f.replace(/\.at$/, '');
  const text = fs.readFileSync(path.join(dir, f), 'utf8');
  const locals = new Set();
  for (const m of text.matchAll(/\bfn\s+([a-zA-Z_][a-zA-Z0-9_]*)/g)) locals.add(m[1]);
  mods[stem] = { locals, text };
}
const owner = {};
for (const [stem, m] of Object.entries(mods)) {
  for (const mm of m.text.matchAll(/^(?:pub\s+)?fn\s+([a-zA-Z_][a-zA-Z0-9_]*)/gm)) (owner[mm[1]] ||= []).push(stem);
}
function strip(text) {
  return text.replace(/\/\/[^\n]*/g, '').replace(/`[^`]*`/g, '""').replace(/"(?:[^"\\]|\\.)*"/g, '""');
}
// classify each line: 0 = top-level structural, 1 = inside method block
function contexts(text) {
  const stripped = strip(text).split('\n');
  const ctx = []; let depth = 0; let inMethod = false;
  for (const line of stripped) {
    const t = line.trim();
    if (/^(?:pub\s+)?(?:fn|type|struct|enum|var|const|ext|impl|dep|use)\b/.test(t) && depth === 0) {
      inMethod = /^(?:ext|impl)\b/.test(t);
      ctx.push(0);
    } else if (/^(?:pub\s+)?fn\b/.test(t) && depth > 0) {
      ctx.push(1); // method defs inside type blocks
    } else {
      ctx.push(inMethod && depth > 0 ? 1 : (inMethod ? 1 : 0));
    }
    // brace depth (approx, strings/comments stripped)
    for (const ch of t) { if (ch === '{') depth++; else if (ch === '}') depth--; }
    if (depth <= 0) { depth = 0; inMethod = false; }
  }
  return ctx;
}

console.log(`VM-loaded modules: ${[...LOADED].sort().join(', ')}\n`);
const report = {};
for (const [stem, m] of Object.entries(mods)) {
  const strippedLines = strip(m.text).split('\n');
  const ctx = contexts(m.text);
  const calls = new Map(); // name -> {owners, lines:[], method:bool}
  for (let i = 0; i < strippedLines.length; i++) {
    const line = strippedLines[i];
    for (const c of line.matchAll(/(?<![.\w$])(?<!fn\s)([a-z_][a-z0-9_]*)\s*\(/g)) {
      const name = c[1];
      if (KEYWORDS.has(name) || m.locals.has(name)) continue;
      let os = owner[name] ? owner[name].filter(o => o !== stem) : [];
      if (!os.length) continue;
      if (AMBIG[name]) os = os.filter(o => o === AMBIG[name]);
      if (os.length !== 1) continue;
      if (!calls.has(name)) calls.set(name, { owner: os[0], lines: [], method: false });
      const e = calls.get(name);
      e.lines.push(i + 1);
      if (ctx[i] === 1) e.method = true;
    }
  }
  if (calls.size) report[stem] = calls;
}

let shimNames = 0, shimLoaded = 0, sitesMethod = 0;
console.log('== per-module: cross calls needing shims (method-context) ==');
for (const [stem, calls] of Object.entries(report)) {
  const need = [...calls].filter(([, e]) => e.method);
  if (!need.length) continue;
  const inLoaded = LOADED.has(stem) ? 'LOADED' : 'transpile-only';
  console.log(`\n[${stem}] (${inLoaded})`);
  for (const [name, e] of need.sort()) { console.log(`  ${name} <- ${e.owner} @ lines ${e.lines.join(',')}`); shimNames++; sitesMethod += e.lines.length; if (LOADED.has(stem)) shimLoaded++; }
}
const plainOnly = Object.entries(report).filter(([s, c]) => ![...c].some(([, e]) => e.method)).map(([s]) => s);
console.log(`\nmethod-context names total: ${shimNames} (loaded modules: ${shimLoaded}), call sites: ${sitesMethod}`);
console.log(`modules with plain-fn-only calls (no shims needed): ${plainOnly.join(', ') || '(none)'}`);
