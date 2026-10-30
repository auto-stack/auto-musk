// Exercise the real native SpecsView and AutoDown core through its public MCP.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
const artifacts = process.env.SPEC_UI_ARTIFACTS ?? 'D:/autostack/plan-artifacts/plan-106';
const endpoint = process.env.SPEC_VM_MCP ?? 'http://127.0.0.1:18574/mcp';
let id = 100;
const pause = ms => new Promise(r => setTimeout(r, ms));
async function call(name, args = {}) {
  const response = await fetch(endpoint, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ jsonrpc: '2.0', id: id++, method: 'tools/call', params: { name, arguments: args } }) });
  const result = (await response.json()).result;
  const text = result.content.filter(c => c.type === 'text').map(c => c.text).join('\n');
  if (result.isError || text.startsWith('Error:')) throw new Error(text);
  return text;
}
async function state() {
  const text = await call('autoui_state', { fields: ['current_view','spec_draft','spec_dirty','spec_hash','spec_notice','spec_saving','active_spec_file_path','file_loading'] });
  const output = {};
  for (const line of text.split('\n')) {
    const m = line.match(/^  (\w+): (.*) \([^)]*\)$/);
    if (m) output[m[1]] = m[2] === 'nil' ? null : JSON.parse(m[2]);
  }
  return output;
}
async function until(fn, label) {
  for (let n=0;n<100;n++) { if (await fn()) return; await pause(100); }
  throw new Error('Timed out: '+label);
}
async function snapshot() { return call('autoui_snapshot', { include_state: false }); }
async function button(label) {
  const text = await snapshot();
  const line = text.split('\n').find(l => l.includes('button #') && l.includes(`"${label}"`));
  assert.ok(line, `Missing button ${label}`);
  return call('autoui_action', { element_id: line.match(/#(vnode_\d+)/)[1], action:'press' });
}
async function selectFile(filename) {
  const text = await snapshot();
  const stack = [];
  let target;
  for (const line of text.split('\n')) {
    const indent = line.length - line.trimStart().length;
    while (stack.length && stack.at(-1).indent >= indent) stack.pop();
    if (line.includes(`"${filename}"`)) { target = stack.findLast(n => n.kind === 'container')?.id; break; }
    const m = line.match(/^\s*(\w+) #(vnode_\d+).*\{$/);
    if (m) stack.push({ kind:m[1],id:m[2],indent });
  }
  assert.ok(target, 'Missing source-file mouse-area');
  await call('autoui_action', { element_id:target, action:'press' });
  await until(async()=>{const s=await state();return s.active_spec_file_path===filename && !s.file_loading;},'source loaded');
}
async function editor() {
  const text = await snapshot();
  const match = text.match(/textarea #(vnode_\d+)/);
  assert.ok(match, 'Missing native document editor');
  return match[1];
}
const file = path.join(artifacts,'ui-workspace/docs/specs/document-ui106.md');
const preamble = '---\nspec_id: ui106\ncustom: retained-verbatim\n---\n';
const checks = [];
const rail = (await snapshot()).split('\n').find(l => l.includes('button #') && l.includes('规范'));
assert.ok(rail, 'Missing Spec navigation');
await call('autoui_action', { element_id:rail.match(/#(vnode_\d+)/)[1], action:'press' });
await until(async()=>(await snapshot()).includes('other-ui106.md'),'Spec file tree');
await selectFile('other-ui106.md');
fs.writeFileSync(file, preamble+'Native baseline 106\n');
await selectFile('document-ui106.md');
let element = await editor();
const core = JSON.parse(await call('autoui_editor_state',{element_id:element}));
assert.match(core.text,/Native baseline 106/);
checks.push('full-native-autodown-core-enabled');
await call('autoui_action',{element_id:element,action:'editor_drag',value:'20,10;20,10'});
await call('autoui_action',{element_id:await editor(),action:'key_press',value:'end'});
await pause(200);
await call('autoui_action',{element_id:await editor(),action:'key_press',value:'c:!'});
await until(async()=> (await state()).spec_dirty,'native core input');
let edited=await state();
assert.ok(edited.spec_draft.includes('!'),JSON.stringify(edited));
assert.ok(edited.spec_draft.includes('Native baseline 106'),JSON.stringify(edited));
checks.push('native-core-key-input-reaches-document-draft');
const navigation = (await snapshot()).split('\n');
const chatHandler = navigation.findIndex(l => l.includes('onclick: .ShowChats'));
assert.ok(chatHandler > 0, 'Missing native chat navigation');
const chatButton = navigation.slice(0,chatHandler).findLast(l => l.includes('button #'));
await call('autoui_action',{element_id:chatButton.match(/#(vnode_\d+)/)[1],action:'press'});
assert.equal((await state()).current_view,'specs');
checks.push('native-unsaved-app-navigation-blocked');
await button('保存文档');
await until(async()=>{const s=await state();return !s.spec_saving && !s.spec_dirty;},'native save');
assert.equal(fs.readFileSync(file,'utf8'),preamble+edited.spec_draft);
checks.push('native-save-and-exact-metadata-preservation');
await selectFile('other-ui106.md');
await selectFile('document-ui106.md');
assert.equal((await state()).spec_draft,edited.spec_draft);
checks.push('native-reload-source-roundtrip');
fs.writeFileSync(file,preamble+'External newer native edit\n');
await call('autoui_type',{element_id:await editor(),text:'Conflicted native draft',clear_first:true});
await until(async()=>(await state()).spec_dirty,'native dirty draft');
await button('保存文档');
await until(async()=>{const s=await state();return !s.spec_saving && s.spec_notice.includes('保存未完成');},'native conflict');
assert.equal((await state()).spec_draft,'Conflicted native draft');
assert.match(fs.readFileSync(file,'utf8'),/External newer native edit/);
checks.push('native-conflict-preserves-draft-and-source');
await button('放弃未保存更改');
await until(async()=>!(await state()).spec_dirty,'native discard');
const receipt={success:true,checks,source:file,endpoint};
fs.writeFileSync(path.join(artifacts,'ui-vm-receipt.json'),JSON.stringify(receipt,null,2));
console.log(JSON.stringify(receipt));
