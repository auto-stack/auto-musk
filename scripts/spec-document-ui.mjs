// Real generated SpecsView → AutoDown → HTTP → source-file verification.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import { pathToFileURL } from 'node:url';

const origin = process.env.SPEC_UI_ORIGIN ?? 'http://127.0.0.1:18571';
const artifacts = process.env.SPEC_UI_ARTIFACTS ?? 'D:/autostack/plan-artifacts/plan-106';
const workspace = path.join(artifacts, 'ui-workspace');
const doc = path.join(workspace, 'docs/specs/document-ui106.md');
fs.mkdirSync(path.dirname(doc), { recursive: true });
const preamble = '---\nspec_id: ui106\ncustom: retained-verbatim\n---\n';
fs.writeFileSync(doc, preamble + '# Before\n\ninitial text\n');
fs.writeFileSync(path.join(path.dirname(doc), 'other-ui106.md'), '# Other\n');
const opened = await fetch(`${origin}/api/workspace/open`, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ path: workspace }) });
assert.equal(opened.status, 200, await opened.clone().text());
const openedBody = await opened.json();
const id = openedBody.id ?? openedBody.workspace?.id;
assert.ok(id, JSON.stringify(openedBody));
const modulePath = process.env.PLAYWRIGHT_MODULE ?? 'D:/autostack/auto-lang/packages/auto-forge-ui/node_modules/playwright/index.mjs';
const { chromium } = await import(pathToFileURL(modulePath));
const browser = await chromium.launch({ headless: true, channel: 'chrome' });
const checks = [];
let page;
try {
  page = await browser.newPage({ viewport: { width: 1280, height: 900 } });
  await page.addInitScript((id) => localStorage.setItem('musk_workspace', id), id);
  await page.goto(`${origin}/specs?workspace=${id}`, { waitUntil: 'networkidle' });
  await page.getByTitle('规范', { exact: true }).first().click();
  await page.getByText('document-ui106.md', { exact: true }).first().click();
  const editor = page.locator('[contenteditable="true"]').first();
  await editor.waitFor({ state: 'visible' });
  await editor.fill('Saved through AutoDown UI 106');
  await page.getByRole('button', { name: '保存文档', exact: true }).click();
  await page.getByText('文档已保存；索引将在下次读取时自动更新。', { exact: true }).waitFor();
  assert.ok(fs.readFileSync(doc, 'utf8').startsWith(preamble));
  assert.ok(fs.readFileSync(doc, 'utf8').includes('Saved through AutoDown UI 106'));
  checks.push('real-editor-save-and-frontmatter-preservation');
  await page.reload({ waitUntil: 'networkidle' });
  await page.getByTitle('规范', { exact: true }).first().click();
  await page.getByText('document-ui106.md', { exact: true }).first().click();
  await editor.waitFor({ state: 'visible' });
  assert.match(await editor.innerText(), /Saved through AutoDown UI 106/);
  checks.push('reload-source-roundtrip');
  fs.writeFileSync(doc, preamble + '# External newer edit\n');
  await editor.fill('Keep this conflicted draft');
  await page.getByRole('button', { name: '保存文档', exact: true }).click();
  await page.getByText(/保存未完成/).waitFor();
  assert.match(await editor.innerText(), /Keep this conflicted draft/);
  assert.match(fs.readFileSync(doc, 'utf8'), /External newer edit/);
  checks.push('conflict-preserves-draft-and-newer-file');
  await page.getByText('other-ui106.md', { exact: true }).first().click();
  await page.getByText('有未保存的更改，请先保存或放弃。', { exact: true }).waitFor();
  assert.match(await editor.innerText(), /Keep this conflicted draft/);
  checks.push('unsaved-file-navigation-blocked');
  await page.getByTitle('会话', { exact: true }).first().click();
  assert.match(await editor.innerText(), /Keep this conflicted draft/);
  assert.match(new URL(page.url()).pathname, /^\/specs/);
  checks.push('unsaved-app-navigation-blocked');
  await page.getByRole('button', { name: '放弃未保存更改', exact: true }).click();
  await page.getByText('other-ui106.md', { exact: true }).first().click();
  await page.getByRole('button', {name:'结构化', exact:true}).click();
  await page.getByText('designs', {exact:true}).first().click();
  await page.getByText('Other', {exact:true}).first().click();
  await page.getByRole('button', {name:/^(编辑|Edit)$/, exact:true}).first().click();
  await editor.waitFor({state:'visible'});
  assert.match(await editor.innerText(), /Other/);
  checks.push('structured-projection-edit-opens-canonical-source');
  await page.screenshot({ path: path.join(artifacts, 'spec-editor-web.png') });
  fs.writeFileSync(path.join(artifacts, 'ui-web-receipt.json'), JSON.stringify({ checks, workspace, origin, success: true }, null, 2));
  console.log(JSON.stringify({ success: true, checks }));
} catch (error) {
  if (page) {
    fs.writeFileSync(path.join(artifacts, 'ui-web-failure.txt'), await page.locator('body').innerText());
    await page.screenshot({ path: path.join(artifacts, 'ui-web-failure.png') });
  }
  throw error;
} finally { await browser.close(); }
