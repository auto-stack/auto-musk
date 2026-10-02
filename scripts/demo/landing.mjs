// scripts/demo/landing.mjs — PLAN-097 T-09: 工程落地对账（Specs/ledger/归档/
// ff 线性/清理,AC-10）。
//
// 合同（计划 §5.4 + AC-10）：每产品计划四技能语义真实落地——默认分支线性
// 合入（无 merge commit）、agent 自有外置 worktree 与分支清理齐全、业务
// Specs 为权威 + ledger 派生、计划归档、Wiki 产出存在。
//
// 形态：collect* 用真实 fs/git 取证（T-12 调用）;verdict* 为纯函数（注入
// 清单,V02 直测）。演示仓约定 = examples/focusboard-demo/AGENTS.md（worktree
// 外置 .wt/focusboard-<run_id>-<NNN>/,ff-only 合回 master,Specs 权威/ledger
// 派生,Wiki 只记用法）。
import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';

// ── 纯判言（V02 直测） ─────────────────────────────────────────────────

// 线性历史：master 上 merge commit 清单必须为空（ff-only 合回的实证）。
export function linearHistoryVerdict(mergeCommits) {
  if (!Array.isArray(mergeCommits)) return { ok: false, reason: 'merge commit listing missing' };
  if (mergeCommits.length) return { ok: false, reason: `发现 ${mergeCommits.length} 个 merge commit（非线性合入）: ${mergeCommits.slice(0, 3).join(',')}` };
  return { ok: true, reason: 'no merge commits (linear)' };
}

// 分支清理：本地分支只剩默认分支（agent 计划分支已删）。
export function branchesVerdict(branchNames, { defaultBranch = 'master' } = {}) {
  const leftover = branchNames.filter(b => b.replace(/^\*?\s*/, '') !== defaultBranch);
  if (leftover.length) return { ok: false, reason: `残留分支: ${leftover.join(',')}（计划分支未清理）` };
  return { ok: true, reason: `only ${defaultBranch} remains` };
}

// worktree 清理：约定外置目录（focusboard-<run_id>-*）与后端托管计划目录
// （MUSK_PLAN_WORKTREE_ROOT/<plan_id>/<repo>，plan_worktree.rs 布局）都必须
// 已删或已空（merge 收尾实证）。
export function worktreeCleanupVerdict(existingDirs, runId, { backendManagedDirs = [] } = {}) {
  const re = new RegExp(`focusboard-${escapeRe(runId)}-`);
  const leftover = existingDirs.filter(d => re.test(d.replace(/\\/g, '/')));
  if (leftover.length) return { ok: false, reason: `demo worktree 残留: ${leftover.join(',')}` };
  const dirty = backendManagedDirs.filter(d => d.files.length > 0);
  if (dirty.length) return { ok: false, reason: `计划 worktree 未清: ${dirty.map(d => d.dir).join(',')}` };
  return { ok: true, reason: `no worktree leftovers for run ${runId}` };
}

// git worktree 注册表：除主检出外不得有注册项（删目录不等于注销,双验）。
export function worktreeListVerdict(listOutput) {
  const entries = String(listOutput ?? '').split(/\r?\n/).filter(Boolean);
  // porcelain: worktree <path> / HEAD … / branch … 成组;数 worktree 行
  const paths = entries.filter(l => l.startsWith('worktree ')).map(l => l.slice('worktree '.length));
  if (paths.length > 1) return { ok: false, reason: `git worktree 注册 ${paths.length} 项: ${paths.join(',')}` };
  return { ok: true, reason: 'worktree registry clean' };
}

// 落地文件对账：Specs 权威（≥1 md）、ledger 派生（约定路径之一存在）、
// 计划归档（有 marker 则命中；无 marker 要求归档目录非空——agent 计划
// 文件名先行未知，归档非空即 AC-10 归档证据）、Wiki 产出存在。
export function landingFilesVerdict({ specs = [], ledgerPaths = [], archived = [], wiki = [], planMarker = null }) {
  const errors = [];
  if (!specs.length) errors.push('docs/specs/ 无任何 .md（业务 Specs 权威缺位）');
  if (!ledgerPaths.length) errors.push('ledger 派生缺位（docs/specs/index.json 与 specs-ledger 均无）');
  if (planMarker && !archived.some(a => a.includes(planMarker))) errors.push(`归档缺本轮计划（docs/plans/archived 无 ${planMarker}）`);
  if (!planMarker && !archived.length) errors.push('docs/plans/archived 为空（归档缺件）');
  if (!wiki.length) errors.push('docs/wiki/ 无产出（用法/取舍 Wiki 缺件）');
  return { ok: errors.length === 0, errors };
}

// ── 取证（T-12 调用;每项失败响亮,不静默跳过） ─────────────────────────

const git = (repoDir, args) => execFileSync('git', ['-C', repoDir, ...args], { encoding: 'utf8', windowsHide: true });

export function collectLandingFacts({ repoDir, wtRoot, runId, planMarker = null }) {
  if (!fs.existsSync(path.join(repoDir, '.git'))) throw new Error(`landing: not a git repo: ${repoDir}`);
  if (!wtRoot) throw new Error('landing: wtRoot required（不得默认扫共享 .wt 根——会误伤其他 plan 的在途 worktree）');
  const merges = git(repoDir, ['log', '--merges', '--format=%H']).split(/\r?\n/).filter(Boolean);
  const branches = git(repoDir, ['branch', '--format=%(refname:short)']).split(/\r?\n/).map(s => s.trim()).filter(Boolean);
  const worktreeList = git(repoDir, ['worktree', 'list', '--porcelain']);
  const wtRootAbs = path.resolve(wtRoot);
  const wtDirs = fs.existsSync(wtRootAbs)
    ? fs.readdirSync(wtRootAbs).map(d => path.join(wtRootAbs, d).replace(/\\/g, '/'))
    : [];
  // 后端托管计划目录（<plan_id>/<repo>）内容清点（plan_worktree.rs 布局）
  const backendManagedDirs = [];
  for (const d of wtDirs) {
    let subs = [];
    try { subs = fs.readdirSync(d); } catch { /* not a dir */ }
    for (const sub of subs) {
      const inner = path.join(d, sub);
      let files = [];
      try { files = fs.readdirSync(inner); } catch { /* not a dir */ }
      backendManagedDirs.push({ dir: inner.replace(/\\/g, '/'), files });
    }
  }
  const specsDir = path.join(repoDir, 'docs', 'specs');
  const specs = fs.existsSync(specsDir)
    ? (function walk(dir, base = '') {
        const out = [];
        for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
          const rel = base ? `${base}/${e.name}` : e.name;
          if (e.isDirectory()) out.push(...walk(path.join(dir, e.name), rel));
          else if (e.isFile() && e.name.endsWith('.md')) out.push(rel);
        }
        return out;
      })(specsDir)
    : [];
  const ledgerPaths = ['docs/specs/index.json', 'docs/specs-ledger.md', 'docs/specs/specs-ledger.md']
    .filter(rel => fs.existsSync(path.join(repoDir, rel)));
  const archivedDir = path.join(repoDir, 'docs', 'plans', 'archived');
  const archived = fs.existsSync(archivedDir) ? fs.readdirSync(archivedDir) : [];
  const wikiDir = path.join(repoDir, 'docs', 'wiki');
  const wiki = fs.existsSync(wikiDir) ? fs.readdirSync(wikiDir).filter(f => f.endsWith('.md')) : [];
  return {
    repoDir,
    runId,
    linearHistory: linearHistoryVerdict(merges),
    branches: branchesVerdict(branches),
    worktreeRegistry: worktreeListVerdict(worktreeList),
    worktreeCleanup: worktreeCleanupVerdict(wtDirs, runId, { backendManagedDirs }),
    files: landingFilesVerdict({ specs, ledgerPaths, archived, wiki, planMarker }),
  };
}

export function landingVerdict(facts) {
  const parts = ['linearHistory', 'branches', 'worktreeRegistry', 'worktreeCleanup', 'files'];
  const errors = [];
  for (const k of parts) {
    if (!facts?.[k]?.ok) errors.push(`${k}: ${facts?.[k]?.reason ?? (facts?.[k]?.errors ?? []).join('; ')}`);
  }
  return { ok: errors.length === 0, errors };
}

function escapeRe(s) { return String(s).replace(/[.*+?^${}()|[\]\\]/g, '\\$&'); }
