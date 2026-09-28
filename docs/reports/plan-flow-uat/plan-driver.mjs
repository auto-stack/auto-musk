#!/usr/bin/env node
// 对话驱动器（2026-09-28 demo 全流程）：向 demo-1 工作区 chat 会话发消息，
// 通过 SSE `done` 事件判断 run 收束。用法：
//   node plan-driver.mjs create
//   node plan-driver.mjs approval <sid>
//   node plan-driver.mjs send <sid>          # 消息文本从 stdin 读入
//   node plan-driver.mjs wait <sid> [min]    # 订阅 SSE 等待 done（默认 45 分钟）
//   node plan-driver.mjs page <sid>          # 打印消息概要
//   node plan-driver.mjs msg <sid> <file>    # send + wait 一条龙
const BASE = 'http://127.0.0.1:17201';
const WS = 'demo-1';
const qs = `workspace=${WS}`;
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function jfetch(url, opts) {
  const r = await fetch(url, opts);
  const t = await r.text();
  try { return JSON.parse(t); } catch { return { raw: t.slice(0, 400), status: r.status }; }
}

const cmd = process.argv[2];
const sid = process.argv[3];

async function create() {
  const d = await jfetch(`${BASE}/api/chats/session?${qs}`, {
    method: 'POST', headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ mode: 'superpowers' }),
  });
  console.log(d?.session?.id ?? JSON.stringify(d));
}

async function approval(id) {
  const d = await jfetch(`${BASE}/api/chats/session/${id}/approval?${qs}`, {
    method: 'PATCH', headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ approval_mode: 'auto' }),
  });
  console.log(JSON.stringify(d?.session ? { approval_mode: d.session.approval_mode } : d));
}

async function send(id, content) {
  for (let i = 0; i < 40; i++) {
    const d = await jfetch(`${BASE}/api/chats/session/${id}/message?${qs}`, {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ content, run: true, queued: true }),
    });
    if (d?.busy) { console.error(`[send] busy, retry #${i + 1} in 15s`); await sleep(15000); continue; }
    console.log(JSON.stringify({ ok: !!d?.session, id: d?.session?.id ?? null }));
    return;
  }
  throw new Error('send failed: busy too long');
}

function summarize(ev) {
  if (ev.type === 'chat_event') {
    const p = ev.payload ?? ev;
    if (p.delta) return String(p.delta).slice(0, 80).replace(/\n/g, ' ');
    return p.status ?? p.phase ?? '';
  }
  if (ev.tool_name) return `${ev.tool_name} ${ev.tool_status ?? ''}`;
  return Object.keys(ev).filter((k) => k !== 'type').slice(0, 3).join(',');
}

async function wait(id, timeoutMin = 45) {
  const ctrl = new AbortController();
  const res = await fetch(`${BASE}/api/chats/session/${id}/stream?${qs}`, { signal: ctrl.signal });
  const reader = res.body.getReader();
  const dec = new TextDecoder();
  let buf = '', events = 0, tools = 0, doneType = null;
  const t0 = Date.now();
  try {
    while (!doneType) {
      if (Date.now() - t0 > timeoutMin * 60000) { console.error('[wait] TIMEOUT'); process.exitCode = 2; return; }
      const { value, done: rdDone } = await reader.read();
      if (rdDone) { console.error('[wait] stream closed without done'); break; }
      buf += dec.decode(value, { stream: true });
      let idx;
      while ((idx = buf.indexOf('\n\n')) >= 0 && !doneType) {
        const frame = buf.slice(0, idx); buf = buf.slice(idx + 2);
        const line = frame.split('\n').find((l) => l.startsWith('data:'));
        if (!line) continue;
        let ev; try { ev = JSON.parse(line.slice(5)); } catch { continue; }
        events++;
        const ty = ev.type ?? '?';
        if (ty === 'tool_update') { tools++; if (tools % 25 === 0) process.stderr.write(`  ..${tools} tool updates\n`); continue; }
        if (ty === 'idle') { doneType = 'idle'; process.stderr.write(`[wait] idle (no active run) after ${events} events\n`); break; }
        process.stderr.write(`[ev#${events}] ${ty} ${summarize(ev)}\n`);
        if (ty === 'done') doneType = 'done';
      }
    }
  } finally { try { ctrl.abort(); } catch {} }
  console.log(JSON.stringify({ result: doneType ?? 'stream_closed', events, tool_updates: tools, elapsed_min: ((Date.now() - t0) / 60000).toFixed(1) }));
}

function textOf(m) {
  if (typeof m.content === 'string' && m.content) return m.content;
  if (Array.isArray(m.blocks)) return m.blocks.filter((b) => b.kind === 'text').map((b) => b.text).join('\n');
  return '';
}

async function page(id) {
  const d = await jfetch(`${BASE}/api/chats/session/${id}/page?limit=500&${qs}`);
  const msgs = d?.messages ?? d?.session?.messages ?? [];
  for (const m of msgs) {
    const tcs = (m.tool_calls ?? []).map((t) => `${t.name}:${t.status ?? '?'}`).join(' ');
    const txt = textOf(m).replace(/\s+/g, ' ').slice(0, 260);
    console.log(`— #${m.seq ?? '?'} ${m.role} ${tcs ? `[${tcs}]` : ''}`);
    if (txt) console.log(`  ${txt}`);
  }
}

async function lastmsg(id) {
  const d = await jfetch(`${BASE}/api/chats/session/${id}/page?limit=500&${qs}`);
  const msgs = d?.messages ?? d?.session?.messages ?? [];
  const asst = [...msgs].reverse().find((m) => m.role === 'assistant');
  console.log(textOf(asst ?? {}) || JSON.stringify(asst).slice(0, 800));
}

if (cmd === 'create') await create();
else if (cmd === 'approval') await approval(sid);
else if (cmd === 'send') await send(sid, (await readStdin()).trim());
else if (cmd === 'wait') await wait(sid, Number(process.argv[4] ?? 45));
else if (cmd === 'page') await page(sid);
else if (cmd === 'lastmsg') await lastmsg(sid);
else if (cmd === 'msg') {
  const content = (await import('node:fs/promises')).readFile(process.argv[4], 'utf8');
  await send(sid, (await content).trim());
  await wait(sid, Number(process.argv[5] ?? 45));
  await lastmsg(sid);
} else { console.error('unknown cmd'); process.exit(1); }

async function readStdin() {
  const chunks = [];
  for await (const c of process.stdin) chunks.push(c);
  return Buffer.concat(chunks).toString('utf8');
}
