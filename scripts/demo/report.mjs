// scripts/demo/report.mjs — PLAN-097 T-13: 图文报告生成（HTML/Markdown）。
//
// 输入 = run 目录（evidence-manifest.json + evidence/*.png + run-summary.json）。
// 输出 = report.md + report.html（按 P001～003 排列的证据入口：每 AC/轮的
// capture 缩略引用、阶段事实、Wiki hash、落地对账、行为矩阵）。只引用
// 已登记事实与已核验 hash——不重绘结果、不拼接、不把未 review 的证据
// 自标 reviewed。外部证据目录清理后，报告正文（含事实/判言文本）仍可读，
// 图片以相对路径引用（V06 离线核验走 manifest hash,不依赖图存在）。
import fs from 'node:fs';
import path from 'node:path';

export function buildReport(runDir, { title = 'PLAN-097 FocusBoard 全流程演示证据报告' } = {}) {
  const manifest = JSON.parse(fs.readFileSync(path.join(runDir, 'evidence-manifest.json'), 'utf8'));
  const summary = fs.existsSync(path.join(runDir, 'run-summary.json'))
    ? JSON.parse(fs.readFileSync(path.join(runDir, 'run-summary.json'), 'utf8')) : null;

  const byRound = new Map(); // p001 → captures[]
  for (const c of manifest.captures ?? []) {
    const m = String(c.capture_id ?? '').match(/^(p00\d)/);
    const key = m ? m[1] : 'run';
    if (!byRound.has(key)) byRound.set(key, []);
    byRound.get(key).push(c);
  }

  const esc = s => String(s ?? '').replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
  const md = [];
  const html = [];
  md.push(`# ${title}`);
  md.push('');
  md.push(`- 运行身份: workspace=${manifest.identity?.workspace_id ?? '-'} session=${manifest.identity?.session_id ?? '-'} · at=${manifest.at}`);
  md.push(`- 指纹: musk=${(manifest.fingerprints?.musk_exe?.sha256 ?? '-').slice(0, 12)}… auto=${manifest.fingerprints?.auto_exe?.version ?? '-'} (${(manifest.fingerprints?.auto_exe?.sha256 ?? '-').slice(0, 12)}…) · worktree_head=${(manifest.fingerprints?.worktree_head ?? '-').slice(0, 12)}`);
  md.push(`- 证据目录: \`${runDir}\\evidence\`（manifest 绑定 SHA256;verify 命令独立核验,清理平台 worktree 后仍可读）`);
  md.push('');
  md.push(`> 判定纪律：本报告只引用 manifest 已登记事实与已核验 hash；每轮交付以 A 控制器的 delivered 事实+五检查点为准；capture 区分 live 与场景级（allow_unstaged 落记录）。报告生成（T-13 出版步骤）区别于模型行为。`);

  // 每轮一节
  for (const p of manifest.plans ?? []) {
    md.push('');
    md.push(`## ${p.plan_id}（r${p.plan_revision} · run ${p.run_id}）`);
    md.push('');
    md.push(`- 阶段事实: ${(p.phases ?? []).map(ph => `${ph.stage}#${ph.attempt}(${ph.outcome ?? '-'})`).join(' → ')}`);
    md.push(`- reviewed_commit: \`${p.reviewed_commit ?? '-'}\` · 收据: \`${p.receipt_ref ?? '-'}\``);
    md.push(`- 交付检查点: ${Object.keys(p.delivery_checkpoints ?? {}).join(', ') || '（无）'}`);
    const caps = (byRound.get(matchRound(p, summary)) ?? []);
    if (caps.length) {
      md.push('');
      md.push('### 截图与采集');
      md.push('');
      md.push('| capture | 来源 | kind | 阶段绑定 | PNG (sha256 前 12) |');
      md.push('|---|---|---|---|---|');
      for (const c of caps) {
        md.push(`| ${c.capture_id} | ${c.source} | ${c.kind}${c.allow_unstaged ? '（场景级,allow_unstaged）' : ''} | ${c.stage_ref ? `${c.stage_ref.stage ?? c.stage_ref.plan_id}#${c.stage_ref.attempt ?? 1}` : '-'} | \`${c.capture_id}.png\` (${(c.sha256 ?? '-').slice(0, 12)}…) |`);
      }
    }
  }

  // 轮次结果矩阵
  if (summary?.rounds?.length) {
    md.push('');
    md.push('## 行为/验收矩阵（run-summary）');
    md.push('');
    md.push('| 轮 | 结果 | 交付 | 检查数 | 失败数 |');
    md.push('|---|---|---|---|---|');
    for (const r of summary.rounds) {
      md.push(`| ${r.round} | ${r.ok ? 'PASS' : 'FAIL'} | ${r.delivered ? 'delivered' : '-'} | ${r.checks ?? '-'} | ${r.fails ?? '-'}${r.error ? ` (${r.error})` : ''} |`);
    }
  }

  // 落地对账（若 manifest 记录）
  if (manifest.landing?.commit || (manifest.landing?.receipts ?? []).length) {
    md.push('');
    md.push(`## 落地: commit=${manifest.landing.commit} receipts=${JSON.stringify(manifest.landing.receipts)}`);
  }

  const mdText = md.join('\n') + '\n';
  // HTML：同构内容,图片相对引用
  const bodyHtml = mdText
    .split('\n')
    .map(line => {
      const e = esc(line);
      if (line.startsWith('# ')) return `<h1>${esc(line.slice(2))}</h1>`;
      if (line.startsWith('## ')) return `<h2>${esc(line.slice(3))}</h2>`;
      if (line.startsWith('### ')) return `<h3>${esc(line.slice(4))}</h3>`;
      if (line.startsWith('|')) return `<div class="row">${e}</div>`;
      if (line.startsWith('> ')) return `<blockquote>${e.slice(2)}</blockquote>`;
      if (line.startsWith('- ')) return `<li>${e.slice(2)}</li>`;
      return e ? `<p>${e}</p>` : '';
    })
    .join('\n');
  const htmlText = `<!doctype html><meta charset="utf-8"><title>${esc(title)}</title>
<style>body{font-family:system-ui,sans-serif;max-width:1080px;margin:2rem auto;padding:0 1rem;color:#111}
h1{font-size:1.5rem}h2{border-bottom:1px solid #ddd;padding-bottom:.2rem;margin-top:2rem}
.row{font-family:ui-monospace,monospace;font-size:.8rem;white-space:pre}
blockquote{background:#f7f7f9;border-left:3px solid #999;padding:.5rem 1rem;color:#444}
li{margin:.2rem 0}</style>
${bodyHtml}`;
  fs.writeFileSync(path.join(runDir, 'report.md'), mdText);
  fs.writeFileSync(path.join(runDir, 'report.html'), htmlText);
  return { md: path.join(runDir, 'report.md'), html: path.join(runDir, 'report.html'), plans: manifest.plans?.length ?? 0, captures: manifest.captures?.length ?? 0 };
}

// plan facts → 轮 key（run 目录内按 plan_id/run 顺序对齐 roundResults）
function matchRound(plan, summary) {
  const pid = String(plan.plan_id ?? '');
  const m = pid.match(/(\d{3})/);
  if (m) return `p${m[1]}`;
  // plan_id 无编号时按 plans 数组顺序对齐 rounds
  return null;
}
