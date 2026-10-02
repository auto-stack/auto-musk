// scripts/demo/wiki.mjs — PLAN-097 T-09: Wiki 发布/回读（既有 workspace
// Wiki API,AC-13）。
//
// 合同（计划 §5.4 末段）：三轮的 docs/wiki/*.md 为 Agent 产出并经复审的
// 用法/取舍正文;驱动在真实交付后通过既有 Wiki API 发布**同一正文**（不
// 改写、不代写知识）,保存源 hash 与回读 hash。本模块是确定性出版步骤——
// 报告须披露该步骤区别于模型行为。
//
// API 面（backend/crates/musk/src/wiki.rs 冻结,project 段被忽略/单项目模型）：
//   POST /api/forge/wiki/{project}/pages?workspace=<id>  {slug,title,content}
//     → 200 | 409（已存在）
//   GET  /api/forge/wiki/{project}/page/{slug}?workspace=<id> → {page:{content,...}} | 404
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';

const sha256Text = s => crypto.createHash('sha256').update(Buffer.from(s, 'utf8')).digest('hex');

// 读取 Agent 仓内的 wiki 正文（原样,不重排）。
export function readAgentWiki(repoDir, relPath) {
  const p = path.join(repoDir, relPath);
  if (!fs.existsSync(p)) throw new Error(`wiki: agent wiki file missing: ${relPath}（交付缺件——要求产出 docs/wiki/*.md）`);
  return fs.readFileSync(p, 'utf8');
}

// 发布：不存在则 create;409 已存在则 update（幂等重跑）。正文逐字同源。
export async function publishWiki({ base, token, workspaceId, slug, title, content, project = 'focusboard', fetchImpl = fetch }) {
  const headers = { 'content-type': 'application/json', ...(token ? { authorization: `Bearer ${token}` } : {}) };
  const qs = workspaceId ? `?workspace=${encodeURIComponent(workspaceId)}` : '';
  let res = await fetchImpl(`${base}/api/forge/wiki/${encodeURIComponent(project)}/pages${qs}`, {
    method: 'POST', headers, body: JSON.stringify({ slug, title, content }),
  });
  if (res.status === 409) {
    res = await fetchImpl(`${base}/api/forge/wiki/${encodeURIComponent(project)}/page/${slug.split('/').map(encodeURIComponent).join('/')}${qs}`, {
      method: 'PUT', headers, body: JSON.stringify({ content, title }),
    });
    if (res.status !== 200) throw new Error(`wiki: update ${slug} -> ${res.status}: ${(await res.text()).slice(0, 160)}`);
    return { slug, action: 'updated', sourceSha256: sha256Text(content) };
  }
  if (res.status !== 200) throw new Error(`wiki: create ${slug} -> ${res.status}: ${(await res.text()).slice(0, 160)}`);
  return { slug, action: 'created', sourceSha256: sha256Text(content) };
}

// 回读：GET 页面 → 正文 hash。404 = 缺件（响亮）。
export async function readbackWiki({ base, token, workspaceId, slug, project = 'focusboard', fetchImpl = fetch }) {
  const headers = token ? { authorization: `Bearer ${token}` } : {};
  const qs = workspaceId ? `?workspace=${encodeURIComponent(workspaceId)}` : '';
  const res = await fetchImpl(`${base}/api/forge/wiki/${encodeURIComponent(project)}/page/${slug.split('/').map(encodeURIComponent).join('/')}${qs}`, { headers });
  if (res.status === 404) throw new Error(`wiki: readback ${slug} -> 404（发布后回读缺件）`);
  if (res.status !== 200) throw new Error(`wiki: readback ${slug} -> ${res.status}`);
  const j = await res.json();
  const content = j?.page?.content;
  if (typeof content !== 'string') throw new Error(`wiki: readback ${slug} malformed (page.content missing)`);
  return { slug, content, readbackSha256: sha256Text(content) };
}

// hash 判言：源与回读必须一致（错 hash = 出版链路失真,判失败）。
export function wikiHashVerdict(sourceSha256, readbackSha256) {
  if (sourceSha256 !== readbackSha256) {
    return { ok: false, reason: `wiki hash mismatch: source ${sourceSha256.slice(0, 12)}… != readback ${readbackSha256.slice(0, 12)}…` };
  }
  return { ok: true, reason: `hash match ${sourceSha256.slice(0, 12)}…` };
}
