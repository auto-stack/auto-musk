// PLAN-107: navigate the native file tree and inspect the real AutoDown editor.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
const directory=process.env.SPEC_MIGRATION_ARTIFACTS??'D:/autostack/plan-artifacts/plan-107';
const origin=process.env.SPEC_MIGRATION_VM??'http://127.0.0.1:18774/mcp';
let id=0;
async function call(name,args={}){
  const r=await fetch(origin,{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({jsonrpc:'2.0',id:++id,method:'tools/call',params:{name,arguments:args}})}).then(r=>r.json());
  assert.ok(!r.error&&!r.result.isError,JSON.stringify(r));
  return r.result.content.map(c=>c.text??'').join('\n');
}
const snapshot=()=>call('autoui_snapshot',{include_state:false});
const press=element_id=>call('autoui_action',{element_id,action:'press'});
const row=JSON.parse(fs.readFileSync(path.join(directory,'source-registry.json'))).sources.find(r=>r.key==='auto-musk-backend');
const manifest=JSON.parse(fs.readFileSync(path.join(directory,'journals',row.key,row.sha256+'.json')));
const sample=manifest.entries.find(e=>e.source!=='envelope'&&e.original_record.content?.length>100);
let text=await snapshot();
const workspace=text.split('\n').find(l=>l.includes('button #')&&l.includes(`musk-107\\${row.project}`));
assert.ok(workspace,'registered native workspace');await press(workspace.match(/#(vnode_\d+)/)[1]);
for(let attempt=0;attempt<40;attempt++){
  const state=await call('autoui_state',{fields:['ws_loading','loading']});
  if(state.includes('ws_loading: false')&&state.includes('loading: false'))break;
  await new Promise(resolve=>setTimeout(resolve,250));
}
text=await snapshot();await press(text.split('\n').find(l=>l.includes('button #')&&l.includes('规范')).match(/#(vnode_\d+)/)[1]);
const names=sample.target.slice('docs/specs/'.length).split('/');
for(let i=0;i<names.length;i++){
  // Snapshot is a public rendered tree, not VM state or a handler invocation.
  let lines,pos=-1;
  for(let attempt=0;attempt<40;attempt++){
    text=await snapshot();lines=text.split('\n').map(l=>l.trimEnd());
    pos=lines.findIndex(l=>l.includes('text #')&&l.endsWith(JSON.stringify(names[i])));
    if(pos>=0)break;
    await new Promise(resolve=>setTimeout(resolve,250));
  }
  assert.ok(pos>=0,`visible native file-tree node ${names[i]}`);
  let containers=[];for(let n=pos-1;n>=0&&containers.length<2;n--)if(/container #(vnode_\d+)/.test(lines[n]))containers.push(lines[n].match(/#(vnode_\d+)/)[1]);
  if(i===names.length-1){await press(containers[0]);break;}
  if(!lines.some(l=>l.includes('text #')&&l.endsWith(JSON.stringify(names[i+1]))))await press(containers[1]);
}
for(let attempt=0;attempt<40;attempt++){
  text=await snapshot();if(text.includes(sample.original_record.title))break;
  await new Promise(resolve=>setTimeout(resolve,250));
}
assert.ok(text.includes(sample.original_record.title),'native document rendered');
const entry=text.match(/textarea #(vnode_\d+)/);
assert.ok(entry,'native AutoDown shell');
const core=await call('autoui_editor_state',{element_id:entry[1]});
assert.ok(core.includes(sample.original_record.title));assert.ok(core.includes('Original record'));
fs.writeFileSync(path.join(directory,'read-vm-core.txt'),core);
fs.writeFileSync(path.join(directory,'read-vm-snapshot.txt'),text);
const state=await call('autoui_state',{fields:['spec_draft','spec_dirty','active_spec_file_path']});
assert.ok(state.includes(row.sha256));assert.ok(state.includes('Original record'));assert.ok(state.includes('false'));
const editors=[entry[1]];
fs.writeFileSync(path.join(directory,'read-vm-state.txt'),state);
assert.equal(fs.readFileSync(path.join(row.worktree,sample.target),'utf8'),sample.content);
fs.writeFileSync(path.join(directory,'read-vm-receipt.json'),JSON.stringify({success:true,source:sample.target,nativeAutoDown:true,unchanged:true},null,2));
console.log(JSON.stringify({success:true,source:sample.target,editors}));
