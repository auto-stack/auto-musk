import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { ROOT, DATA, ARTIFACTS, readJson, writeJson } from './source.mjs';

export async function verifyVue(caseId, url, plan = 'PLAN-077') {
  const catalog = readJson(path.join(DATA, 'cases.json'));
  const entry = catalog.cases.find(c => c.id === caseId);
  const fixture = readJson(path.join(DATA, 'fixtures', entry.fixture));
  const playwright = process.env.PLAYWRIGHT_MODULE ?? 'D:/autostack/auto-lang/packages/auto-forge-ui/node_modules/playwright/index.mjs';
  const { chromium } = await import(pathToFileURL(playwright).href);
  const browser = await chromium.launch({ headless: true, channel: process.env.PLAYWRIGHT_CHANNEL ?? 'chrome' });
  const page = await browser.newPage({ viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1 });
  const errors = [], requests = [], assertions = [];
  page.on('pageerror', error => errors.push(error.message));
  await page.route('**/api/**', async route => {
    requests.push({ method: route.request().method(), path: new URL(route.request().url()).pathname, body: route.request().postData() });
    // 形状宽容的桩：PlansStore.LoadPlans 等消费方直读响应字段（c.plans.length），
    // '{}' 会 pageerror——按后端真实形态给空表。
    await route.fulfill({ status: 200, contentType: 'application/json', body: '{"plans":[]}' });
  });
  const check = async step => {
    const text = await page.locator('body').innerText();
    return (step.visible ?? []).every(x => text.includes(x)) && (step.absent ?? []).every(x => !text.includes(x)) && (!step.request || requests.some(r => r.method === 'POST' && r.path === step.request));
  };
  let status = 'missing';
  const out = path.join(ARTIFACTS, plan, `${caseId}-vue.png`);
  const spyOf = text => { const m = text.match(/Spy events (\d+)/); return m ? Number(m[1]) : -1; };
  try {
    await page.goto(url, { waitUntil: 'networkidle' });
    await page.getByRole('button', { name: 'Reset fixture', exact: true }).click();
    await page.waitForTimeout(250);
    assertions.push({ step: 'fixture', pass: await check(fixture.expect ?? {}) });
    let spyBefore = spyOf(await page.locator('body').innerText());
    for (const step of fixture.interactions ?? []) {
      if (step.vmOnly) continue;
      await page.getByRole('button', { name: new RegExp(step.click.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')) }).nth(step.index ?? 0).click();
      await page.waitForTimeout(250);
      const bodyText = await page.locator('body').innerText();
      const spyAfter = spyOf(bodyText);
      const pass = (await check(step)) && (!step.spyIncrement || spyAfter > spyBefore);
      spyBefore = spyAfter;
      assertions.push({ step, pass });
    }
    fs.mkdirSync(path.dirname(out), { recursive: true });
    await page.screenshot({ path: out, fullPage: true });
    status = errors.length === 0 && assertions.every(a => a.pass) ? 'interaction-ok' : 'interaction-failed';
  } catch (error) { errors.push(error.message); status = 'interaction-failed'; }
  const result = { plan, caseId, mode: 'vue', status, errors, requests, assertions, screenshot: out, text: await page.locator('body').innerText().catch(() => '') };
  await browser.close();
  writeJson(path.join(ARTIFACTS, plan, `${caseId}-vue.json`), result);
  return result;
}

if (process.argv[1] && path.resolve(process.argv[1]) === path.resolve(new URL(import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, '$1'))) {
  const result = await verifyVue(process.argv[2], process.argv[3] ?? 'http://127.0.0.1:17774');
  console.log(JSON.stringify(result, null, 2));
  if (result.status !== 'interaction-ok') process.exitCode = 1;
}

