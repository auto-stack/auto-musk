// PLAN-107: real derived indexes and actual generated AutoDown Web reader.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {pathToFileURL} from 'node:url';
const origin=process.env.SPEC_MIGRATION_ORIGIN??'http://127.0.0.1:18771';
const directory=process.env.SPEC_MIGRATION_ARTIFACTS??'D:/autostack/plan-artifacts/plan-107';
const registry=JSON.parse(fs.readFileSync(path.join(directory,'source-registry.json'),'utf8'));
const projects={};const apiChecks=[];
for(const row of registry.sources){
  if(!projects[row.project]){
    const opened=await fetch(origin+'/api/workspace/open',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({path:row.worktree})});
    assert.equal(opened.status,200,await opened.clone().text());
    const result=await opened.json();projects[row.project]=result.id??result.workspace?.id;
    assert.ok(projects[row.project]);
  }
}
for(const [project,id] of Object.entries(projects)){
  const response=await fetch(origin+`/api/specs/index?workspace=${id}`);
  assert.equal(response.status,200,project+': '+await response.clone().text());
  const index=await response.json();assert.equal(index.fresh,true,project);
  const ids=index.document.sections.flatMap(section=>section.items.map(item=>item.id));
  for(const row of registry.sources.filter(row=>row.project===project)){
    const prefix=row.skip?`legacy:${row.sha256}:`:`legacy:${project}:${row.key}:${row.sha256}:`;
    const found=ids.filter(id=>id.startsWith(prefix));
    assert.equal(found.length,row.records,`${row.key} index identity count`);
    if(!row.skip){
      const manifest=JSON.parse(fs.readFileSync(path.join(directory,'journals',row.key,row.sha256+'.json'),'utf8'));
      const sample=manifest.entries[0];const relative=sample.target.slice('docs/specs/'.length);
      const read=await fetch(origin+`/api/specs/document/${relative}?workspace=${id}`).then(r=>r.json());
      assert.equal(read.content,sample.content,row.key+' actual source read');
      assert.equal(read.hash,sample.content_hash,row.key+' actual source hash');
    }
    apiChecks.push({source:row.key,records:found.length,fresh:index.fresh,diagnostics:index.diagnostics.length});
  }
}
fs.writeFileSync(path.join(directory,'runtime-projects.json'),JSON.stringify(projects,null,2));
fs.writeFileSync(path.join(directory,'vm-storage.json'),JSON.stringify({musk_workspace:projects['auto-down']}));
if(process.argv.includes('--api-only')){
  fs.writeFileSync(path.join(directory,'read-api-receipt.json'),JSON.stringify({success:true,checks:apiChecks},null,2));
  console.log(JSON.stringify({success:true,sources:apiChecks.length,records:apiChecks.reduce((sum,x)=>sum+x.records,0)}));
  process.exit(0);
}
const candidates=[process.env.PLAYWRIGHT_MODULE,'D:/autostack/auto-lang/packages/auto-forge-ui/node_modules/playwright/index.mjs','D:/autostack/auto-ui/node_modules/playwright/index.mjs'].filter(Boolean);
const module=candidates.find(p=>fs.existsSync(p));assert.ok(module,'Playwright module unavailable');
const {chromium}=await import(pathToFileURL(module));const browser=await chromium.launch({headless:true,channel:'chrome'});
let page;const web=[];
try{
  const row=registry.sources.find(r=>r.project==='auto-down');
  const manifest=JSON.parse(fs.readFileSync(path.join(directory,'journals',row.key,row.sha256+'.json'),'utf8'));
  const sample=manifest.entries[0];const relative=sample.target.slice('docs/specs/'.length);
  page=await browser.newPage({viewport:{width:1400,height:1000}});
  await page.addInitScript(id=>localStorage.setItem('musk_workspace',id),projects['auto-down']);
  await page.goto(origin+'/specs',{waitUntil:'networkidle'});
  await page.getByTitle('规范',{exact:true}).first().click();
  const segments=relative.split('/');
  for(let i=0;i<segments.length-1;i++){
    const current=page.getByText(segments[i],{exact:true}).first();await current.waitFor({state:'visible'});
    const next=page.getByText(segments[i+1],{exact:true}).first();
    if(!await next.isVisible()){
      const outer=current.locator('xpath=ancestor::div[contains(@class,"items-center")][2]');
      await outer.locator(':scope > div.w-5.h-5').click();
    }
  }
  await page.getByText(segments.at(-1),{exact:true}).first().click();
  const editor=page.locator('[contenteditable="true"]').first();await editor.waitFor({state:'visible'});
  const text=await page.locator('[data-engine-editor]').innerText();assert.ok(text.includes(sample.original_record.title),text.slice(0,300));
  assert.ok(text.includes(row.sha256));assert.ok(text.includes('Original record'));
  web.push('generated-specs-tree-to-real-autodown-editor');
  web.push('legacy-map-source-chinese-and-json-record-rendered');
  assert.equal(fs.readFileSync(path.join(row.worktree,sample.target),'utf8'),sample.content);
  web.push('view-did-not-mutate-source');
  await page.screenshot({path:path.join(directory,'migration-autodown-web.png')});
  fs.writeFileSync(path.join(directory,'read-web-receipt.json'),JSON.stringify({success:true,api:apiChecks,web,source:sample.target},null,2));
  console.log(JSON.stringify({success:true,web,apiSources:apiChecks.length}));
}catch(error){
  if(page){fs.writeFileSync(path.join(directory,'read-web-failure.txt'),await page.locator('body').innerText());await page.screenshot({path:path.join(directory,'read-web-failure.png')});}
  throw error;
}finally{await browser.close();}
