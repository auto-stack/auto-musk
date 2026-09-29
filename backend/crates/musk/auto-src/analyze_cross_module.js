// PLAN-093 seam analysis: enumerate bare cross-module fn calls in auto-src corpus
// under Plan 545 linker semantics (dep exports are mod-qualified only).
// Read-only analysis; output guides per-module `use <mod>: *` insertion.
const fs = require('fs');
const path = require('path');
const dir = __dirname;
const files = fs.readdirSync(dir).filter(f => f.endsWith('.at') && f !== 'extern_sigs_template.at');
const KEYWORDS = new Set(['if','else','for','while','match','return','fn','let','print','println','dep','use','impl','ext','pub','struct','enum','type','import','true','false','nil','and','or','not','in','as','spawn','await','async','defer','move','loop','break','continue','main']);
const NATIVE_PREFIXES = ['auto.', 'json.', 'http.', 'file.', 'image.', 'str.', 'list.', 'map.', 'math.', 'time.', 'ui.', 'dom.', 'os.', 'proc.', 'term.', 'rand.', 'db.', 'regex.', 'base64.', 'uuid.', 'log.'];

const mods = {}; // stem -> { exports:Set, locals:Set, text }
for (const f of files) {
  const stem = f.replace(/\.at$/, '');
  const text = fs.readFileSync(path.join(dir, f), 'utf8');
  const exports = new Set();
  for (const m of text.matchAll(/^(?:pub\s+)?fn\s+([a-zA-Z_][a-zA-Z0-9_]*)/gm)) exports.add(m[1]);
  const locals = new Set(); // all defs incl. indented methods
  for (const m of text.matchAll(/\bfn\s+([a-zA-Z_][a-zA-Z0-9_]*)/g)) locals.add(m[1]);
  mods[stem] = { exports, locals, text };
}
// owner index: exported name -> [modules]
const owner = {};
for (const [stem, m] of Object.entries(mods)) {
  for (const e of m.exports) (owner[e] ||= []).push(stem);
}

// strip comments & strings to reduce false positives
function strip(text) {
  return text
    .replace(/\/\/[^\n]*/g, '')
    .replace(/`[^`]*`/g, '""')
    .replace(/"(?:[^"\\]|\\.)*"/g, '""');
}

const crossCalls = {}; // stem -> Map(callee -> Set(owners))
for (const [stem, m] of Object.entries(mods)) {
  const t = strip(m.text);
  const calls = new Map();
  for (const c of t.matchAll(/(?<![.\w$])(?<!fn\s)([a-z_][a-z0-9_]*)\s*\(/g)) {
    const name = c[1];
    if (KEYWORDS.has(name)) continue;
    if (owner[name] && !m.locals.has(name)) {
      const os = owner[name].filter(o => o !== stem);
      if (os.length) {
        if (!calls.has(name)) calls.set(name, new Set());
        for (const o of os) calls.get(name).add(o);
      }
    }
  }
  if (calls.size) crossCalls[stem] = calls;
}

console.log('== per-module cross-module bare calls (callee -> owners) ==');
let total = 0;
for (const [stem, calls] of Object.entries(crossCalls)) {
  console.log(`\n[${stem}]`);
  for (const [name, os] of [...calls].sort()) { console.log(`  ${name} <- ${[...os].join(',')}`); total++; }
}
console.log(`\ntotal call-sites-as-names: ${total}`);

console.log('\n== ambiguous names (exported by 2+ modules) ==');
for (const [name, os] of Object.entries(owner)) if (os.length > 1) console.log(`  ${name}: ${os.join(',')}`);

// names defined both locally in module and in other modules (local shadows — fine, but report)
console.log('\n== modules NOT calling cross-module (no use needed) ==');
const all = Object.keys(mods);
console.log(all.filter(s => !crossCalls[s]).join(', '));
