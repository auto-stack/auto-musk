// PLAN-093 seam fix: insert Plan-545-compliant item imports into auto-src corpus.
// For each module, add `use <owner>: <fns...>` for bare cross-module fn calls.
// Ambiguity rulings (by hand):
//   task_plan_engine drive_run -> relay_driver (3-arg state+await+~Result match)
//   relay_driver build_agent_with_context -> lib (typed AgentMode/Arc/Option match)
// vm_entry.at: upgrade existing bare `use <mod>` to item form for route assemblers.
const fs = require('fs');
const path = require('path');
const dir = __dirname;
const files = fs.readdirSync(dir).filter(f => f.endsWith('.at'));
const KEYWORDS = new Set(['if','else','for','while','match','return','fn','let','print','println','dep','use','impl','ext','pub','struct','enum','type','import','true','false','nil','and','or','not','in','as','spawn','await','async','defer','move','loop','break','continue','main']);

const AMBIG = { drive_run: 'relay_driver', build_agent_with_context: 'lib' };

const mods = {};
for (const f of files) {
  const stem = f.replace(/\.at$/, '');
  const text = fs.readFileSync(path.join(dir, f), 'utf8');
  const exports = new Set();
  for (const m of text.matchAll(/^(?:pub\s+)?fn\s+([a-zA-Z_][a-zA-Z0-9_]*)/gm)) exports.add(m[1]);
  const locals = new Set();
  for (const m of text.matchAll(/\bfn\s+([a-zA-Z_][a-zA-Z0-9_]*)/g)) locals.add(m[1]);
  mods[stem] = { exports, locals, text };
}
const owner = {};
for (const [stem, m] of Object.entries(mods)) for (const e of m.exports) (owner[e] ||= []).push(stem);

function strip(text) {
  return text.replace(/\/\/[^\n]*/g, '').replace(/`[^`]*`/g, '""').replace(/"(?:[^"\\]|\\.)*"/g, '""');
}

// needed[stem] = Map(owner -> Set(fn names))
const needed = {};
for (const [stem, m] of Object.entries(mods)) {
  const t = strip(m.text);
  for (const c of t.matchAll(/(?<![.\w$])(?<!fn\s)([a-z_][a-z0-9_]*)\s*\(/g)) {
    const name = c[1];
    if (KEYWORDS.has(name) || m.locals.has(name)) continue;
    let os = owner[name] ? owner[name].filter(o => o !== stem) : [];
    if (os.length && AMBIG[name]) os = os.filter(o => o === AMBIG[name]);
    if (os.length === 1) {
      const o = os[0];
      ((needed[stem] ||= new Map()).set(o, (needed[stem].get(o) || new Set()).add(name)));
    } else if (os.length > 1) {
      console.error(`UNRESOLVED ambiguity: ${stem} :: ${name} <- ${os.join(',')}`);
      process.exitCode = 1;
    }
  }
}

// insertion point: after last header (use|dep) line before first top-level decl; else at decl
function insertAt(lines) {
  let firstDecl = lines.findIndex(l => /^(?:pub\s+)?(?:fn|type|struct|enum|impl|ext)\b/.test(l));
  if (firstDecl === -1) firstDecl = lines.length;
  let lastUse = -1;
  for (let i = 0; i < firstDecl; i++) if (/^(?:use|dep)\b/.test(lines[i])) lastUse = i;
  return lastUse >= 0 ? lastUse + 1 : firstDecl;
}

let totalLines = 0;
for (const [stem, owners] of Object.entries(needed)) {
  if (stem === 'vm_entry') continue; // handled specially below
  const f = stem + '.at';
  const lines = mods[stem].text.split('\n');
  const inserts = [];
  for (const [o, names] of [...owners].sort()) {
    inserts.push(`// PLAN-093 seam: Plan 545 linker needs explicit imports for cross-module bare calls`);
    inserts.push(`use ${o}: ${[...names].sort().join(', ')}`);
    totalLines += names.size;
  }
  lines.splice(insertAt(lines), 0, ...inserts, '');
  fs.writeFileSync(path.join(dir, f), lines.join('\n'));
  console.log(`${f}: +${inserts.filter(l => l.startsWith('use')).length} use lines`);
}

// vm_entry.at: bare use -> item form
const ve = path.join(dir, 'vm_entry.at');
let veText = fs.readFileSync(ve, 'utf8');
const veMap = { server: ['build_router'], server_stream: ['red_routes'], relay_api: ['relay_routes', 'task_plan_routes'], wiki: ['wiki_routes'], canvas_vm: ['canvas_vm_routes'] };
for (const [mo, names] of Object.entries(veMap)) {
  const re = new RegExp(`^use ${mo}$`, 'm');
  if (!re.test(veText)) { console.error(`vm_entry: missing bare use ${mo}`); process.exitCode = 1; continue; }
  veText = veText.replace(re, `use ${mo}: ${names.join(', ')}`);
}
fs.writeFileSync(ve, veText);
console.log(`vm_entry.at: 5 bare uses upgraded to item form`);
console.log(`total imported fn names: ${totalLines + 6}`);
