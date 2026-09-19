import fs from 'node:fs';
import path from 'node:path';
import { ROOT, GALLERY, DATA, files, hash, slash, readJson, writeJson, inventory, effectiveCases } from './source.mjs';

export function at(v) {
  if (Array.isArray(v)) return '[' + v.map(at).join(', ') + ']';
  if (v && typeof v === 'object') return '{ ' + Object.entries(v).map(([k, val]) => `${k}: ${at(val)}`).join(', ') + ' }';
  return v === null ? 'None' : JSON.stringify(v);
}

export function materialize(caseId = 'chat-message-pair') {
  const catalog = readJson(path.join(DATA, 'cases.json'));
  const c = effectiveCases(catalog).find(c => c.id === caseId);
  if (!c) throw new Error(`Unknown case: ${caseId}`);
  const fixture = readJson(path.join(DATA, 'fixtures', c.fixture));
  const receipt = { version: 1, caseId, sourceFiles: [], boundaryChanges: [], fixtureHash: hash(JSON.stringify(fixture)) };
  // Paths inside use.web are project-root absolute (src/front/...). Mirror that root,
  // byte-for-byte; no rewritten widget, alternate template, junction or symlink.
  const old = fs.existsSync(path.join(GALLERY, 'materialized.json')) ? readJson(path.join(GALLERY, 'materialized.json')) : null;
  const inputs = [...files(path.join(ROOT, 'src')), ...files(path.join(ROOT, 'vendor'))];
  const wanted = new Set(inputs.map(p => slash(path.relative(ROOT, p))));
  for (const entry of old?.sourceFiles ?? []) {
    if (!wanted.has(entry.path)) fs.unlinkSync(path.join(GALLERY, entry.path));
  }
  for (const p of inputs) {
    const rel = slash(path.relative(ROOT, p));
    if (rel === 'src/front/app.at') continue; // The only UI replacement is the gallery host itself.
    const dest = path.join(GALLERY, rel), data = fs.readFileSync(p);
    fs.mkdirSync(path.dirname(dest), { recursive: true });
    if (!fs.existsSync(dest) || !fs.readFileSync(dest).equals(data)) {
      fs.writeFileSync(dest, data);
    }
    receipt.sourceFiles.push({ path: rel, sha256: hash(data) });
  }
  // Preserve API signatures + return semantics. Contract stubs have no backend implementation.
  // Instrument their data boundary to make calls observable in VM merged mode.
  const api = path.join(GALLERY, 'src/back/api.at');
  const original = fs.readFileSync(api, 'utf8');
  const instrumented = original.replace(/(pub fn\s+(\w+)\([^)]*\)[^{]*\{)/g,
    (all, prefix, name) => `${prefix}\n    print("GALLERY_API:${name}")`);
  if (!fs.existsSync(api) || fs.readFileSync(api, 'utf8') !== instrumented) {
    fs.writeFileSync(api, instrumented);
  }
  receipt.boundaryChanges.push({ path: 'src/back/api.at', purpose: 'instrument contract-only stub calls', sha256: hash(instrumented) });

  // ChatMessage writes expansion/cancel events through ForgeStore. The real
  // store also boots the production stream and session graph, which would
  // violate the gallery's no-network/no-workspace contract and makes an
  // isolated component case depend on unrelated app startup code. Keep the
  // production ChatMessage source byte-for-byte intact and replace only this
  // side-effect boundary with a deterministic in-memory store. This is an
  // adapter, not a second rendering implementation.
  const galleryStore = `// GENERATED gallery side-effect adapter; ChatMessage remains production source.\nstore ForgeStore {\n    model {\n        var think_open str = ""\n        var tool_open str = ""\n        var cancel_count int = 0\n        var event_count int = 0\n        var events []Value = []\n    }\n    msg Msg { ThinkToggle(str), ToolToggle(str), CancelRun, LoadSessionList }\n    on {\n        .ThinkToggle(key str) -> {\n            if .think_open == key { .think_open = "" } else { .think_open = key }\n            .events.push({ kind: "think", key: key })\n            .event_count = .event_count + 1\n        }\n        .ToolToggle(key str) -> {\n            if .tool_open == key { .tool_open = "" } else { .tool_open = key }\n            .events.push({ kind: "tool", key: key })\n            .event_count = .event_count + 1\n        }\n        .CancelRun -> {\n            .cancel_count = .cancel_count + 1\n            .events.push({ kind: "cancel" })\n            .event_count = .event_count + 1\n        }\n        .LoadSessionList -> {\n            .events.push({ kind: "load_session_list" })\n            .event_count = .event_count + 1\n        }\n    }\n}\n`;
  const storePath = path.join(GALLERY, 'src/front/forge_store.at');
  if (!fs.existsSync(storePath) || fs.readFileSync(storePath, 'utf8') !== galleryStore) {
    fs.writeFileSync(storePath, galleryStore);
  }
  receipt.boundaryChanges.push({ path: 'src/front/forge_store.at', purpose: 'isolate expansion/cancel side effects', sha256: hash(galleryStore) });

  const unit = inventory().find(u => u.id === c.unit);
  if (c.unit === 'App') {
    throw new Error('App is a composition case owned by PLAN-079, not mounted recursively in the gallery host');
  }
  const module = unit.source.replace('src/front/', '').replace(/\.at$/, '').replaceAll('/', '.');
  const props = fixture.props ?? {};
  const fields = Object.entries(props).map(([key, v]) => `var fixture_${key} Value = ${at(v)}`).join('\n        ');
  const bindings = Object.keys(props).map(k => `${k}: .fixture_${k}`).join('\n                ');
  const chat = c.unit === 'ChatMessage';
  const instances = fixture.instances ?? [props];
  const fieldsChat = instances.map((x, i) => `var sample${i} Value = ${at(x.msg)}`).join('\n        ');
  const chatViews = instances.map((x, i) => `col {
                style: "w-full gap-2"
                text "Instance ${i + 1}"
                ChatMessage {
                    msg: .sample${i}
                    is_streaming: ${at(x.is_streaming ?? false)}
                    errands: .empty_record
                    relays: .empty_record
                    task_plans: .empty_record
                    think_open: store.think_open
                    tool_open: store.tool_open
                }
            }`).join('\n');
  const reset = chat ? instances.map((x, i) => `.sample${i} = ${at(x.msg)}`).join('\n            ') :
    Object.entries(props).map(([k,v]) => `.fixture_${k} = ${at(v)}`).join('\n            ');
  const source = `// GENERATED gallery host. Production widget files are hash-checked byte copies.
use ${module}: ${c.unit}
use store: ForgeStore
use.web platformInjectStyles from "src/front/ports/platform.at"
widget App {
    model {
        var empty_record Value = {}
        var gallery_case str = ${at(c.id)}
        var reset_count int = 0
        ${chat ? fieldsChat : fields}
    }
    msg Msg { Init, Reset }
    on {
        .Init -> { platformInjectStyles() }
        .Reset -> {
            ${reset}
            ForgeStore.ThinkToggle("")
            ForgeStore.ToolToggle("")
            .reset_count = .reset_count + 1
        }
    }
    view {
        col {
            style: "w-full h-screen bg-background text-foreground p-4 gap-4"
            row {
                style: "w-full gap-4 items-center"
                text .gallery_case
                text "Spy events " + store.event_count
                button {
                    text "Reset fixture"
                    onclick: .Reset
                }
            }
            col {
                style: "w-full gap-4"
                ${chat ? chatViews : `${c.unit} {\n                ${bindings}\n                }`}
            }
        }
    }
}
`;
  const entry = path.join(GALLERY, 'src/front/app.at');
  if (!fs.existsSync(entry) || fs.readFileSync(entry, 'utf8') !== source) {
    fs.writeFileSync(entry, source);
  }
  receipt.entryHash = hash(source);
  writeJson(path.join(GALLERY, 'materialized.json'), receipt);
  return receipt;
}

export function verifyMaterialized() {
  const p = path.join(GALLERY, 'materialized.json');
  if (!fs.existsSync(p)) return ['No materialization; run prepare'];
  const r = readJson(p), errors = [];
  for (const f of r.sourceFiles) {
    const src = path.join(ROOT, f.path), dest = path.join(GALLERY, f.path);
    if (!fs.existsSync(src) || hash(fs.readFileSync(src)) !== f.sha256) errors.push(`Source drift: ${f.path}`);
    const expected = r.boundaryChanges.find(b => b.path === f.path)?.sha256 ?? f.sha256;
    if (!fs.existsSync(dest) || hash(fs.readFileSync(dest)) !== expected) errors.push(`Materialized drift: ${f.path}`);
  }
  if (hash(fs.readFileSync(path.join(GALLERY, 'src/front/app.at'))) !== r.entryHash) errors.push('Generated host modified');
  return errors;
}
