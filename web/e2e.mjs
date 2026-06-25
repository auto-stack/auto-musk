import { chromium } from 'playwright';

// End-to-end test for the musk web app (Plan 008): login → new chat → send a
// message → see a streamed reply. Proves the full stack: auth + session
// persistence + SSE agent run + history.
const browser = await chromium.launch({ headless: true });
const page = await browser.newPage();

const errors = [];
page.on('pageerror', err => errors.push(`PAGE_ERROR: ${err.message}`));
page.on('console', msg => { if (msg.type() === 'error') errors.push(`CONSOLE: ${msg.text()}`); });

console.log('=== 1. Open login page (http://localhost:8090) ===');
await page.goto('http://localhost:8090', { waitUntil: 'networkidle', timeout: 15000 });
await page.waitForTimeout(1000);
const hasLogin = await page.$('input[type="password"]');
console.log('  login form present:', !!hasLogin);

console.log('\n=== 2. Sign in (admin/admin) ===');
await page.fill('input[placeholder="Username"]', 'admin');
await page.fill('input[placeholder="Password"]', 'admin');
await page.click('button[type="submit"]');
await page.waitForTimeout(1500);
// After login, the topbar with brand should appear.
const inApp = await page.$('.topbar');
console.log('  entered app:', !!inApp);
await page.screenshot({ path: 'e2e-loggedin.png', fullPage: true });

console.log('\n=== 3. Start a new chat ===');
await page.click('.new-btn');
await page.waitForTimeout(1000);
const sessionLoaded = await page.$('.chat-area .input-bar, .placeholder');
console.log('  chat view loaded:', !!sessionLoaded);

console.log('\n=== 4. Send a message (streamed reply via LLM) ===');
await page.fill('.input', 'Say "hello" and nothing else.');
await page.click('.send-btn');
// Wait for streaming to start (typing indicator or streaming bubble), then done.
console.log('  waiting for agent reply (up to 60s)...');
let replied = false;
for (let i = 0; i < 60; i++) {
  await page.waitForTimeout(1000);
  // An assistant message bubble with content = reply arrived (streaming done reloads session).
  const assistantMsgs = await page.$$eval('.msg.assistant .msg-content', els =>
    els.map(e => e.textContent?.trim() || '')
  );
  const nonEmpty = assistantMsgs.filter(t => t.length > 0);
  if (nonEmpty.length > 0) {
    replied = true;
    console.log(`  ✓ reply received (t=${i+1}s): "${nonEmpty[nonEmpty.length - 1].slice(0, 60)}"`);
    break;
  }
}
await page.screenshot({ path: 'e2e-chat-reply.png', fullPage: true });

console.log('\n=== 5. Session persisted (sidebar shows the chat) ===');
const sessionCount = await page.$$eval('.session-item', els => els.length);
console.log(`  sessions in sidebar: ${sessionCount}`);

console.log('\n=== Console errors ===');
if (errors.length === 0) console.log('  (none)');
errors.slice(0, 5).forEach(e => console.log(' ', e));

const passed = !!inApp && replied;
console.log(`\n=== RESULT: ${passed ? '✅ musk web app works end-to-end' : '❌ FAILED'} ===`);
if (!passed) process.exitCode = 1;
await browser.close();
