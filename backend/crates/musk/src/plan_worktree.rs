//! PLAN-096 T-04: Plan execution worktrees — the lease that binds a plan run
//! to a dedicated git worktree of the target repository (§5.3).
//!
//! Contract:
//! - The **main root** must be a real git checkout (`rev-parse --show-toplevel`
//!   resolves to it) — the plan's shared state (docs/plans/, docs/specs/,
//!   ledger) lives there; implementation work does not.
//! - The **execution worktree** is created under a run-scoped layout root
//!   (`MUSK_PLAN_WORKTREE_ROOT` env → `{main_root.parent}/.musk-wt/`), named
//!   `{plan_id}/{repo_name}`, on branch `plan-{seq}-dev`. Existing occupied
//!   directories or branches are never taken over (loud refusal).
//! - **No junctions/symlinks**: before any create/reuse/remove the tree is
//!   scanned for reparse points (the wt-guard discipline — Plan 529 incident);
//!   a positive scan blocks the operation.
//! - Removal only after (a) fresh guard scan clean, (b) every branch commit
//!   is contained in the default branch (`merge-base --is-ancestor`), then
//!   `git worktree remove` + `git branch -d`; an empty layout group dir is
//!   pruned. Failures surface as errors — the caller records
//!   `cleanup_pending`, never retries the landed side effects.
//!
//! All git invocations pass argument vectors (no shell interpolation of
//! model-provided strings).

use std::path::{Path, PathBuf};
use std::process::Command;

/// A registered worktree lease for one plan run.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorktreeLease {
    /// Main checkout absolute path (canonicalized).
    pub main_root: String,
    /// Worktree absolute path (canonicalized at creation time).
    pub worktree_root: String,
    /// Development branch (`plan-{seq}-dev`).
    pub branch: String,
    /// Commit the worktree was created from (approval-time base).
    pub base_commit: String,
    /// True when this call created the worktree (false = reused).
    pub created: bool,
}

fn git<I, S>(cwd: &Path, args: I) -> Result<String, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let out = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| format!("git spawn failed: {e}"))?;
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    if !out.status.success() {
        return Err(format!(
            "git {} failed ({}): {}",
            "command",
            out.status,
            stderr.trim()
        ));
    }
    Ok(stdout)
}

/// Validate `main_root` is a real git checkout top-level (not a subdirectory,
/// not itself a linked worktree of another repo — the shared-state owner).
pub fn validate_main_checkout(main_root: &Path) -> Result<PathBuf, String> {
    if !main_root.is_dir() {
        return Err(format!("main root not a directory: {}", main_root.display()));
    }
    let toplevel = git(main_root, ["rev-parse", "--show-toplevel"])?;
    let toplevel = PathBuf::from(toplevel.trim());
    let want = std::fs::canonicalize(main_root)
        .unwrap_or_else(|_| main_root.to_path_buf());
    let got = std::fs::canonicalize(&toplevel).unwrap_or(toplevel);
    if got != want {
        return Err(format!(
            "main root is not a checkout top-level (git reports {})",
            got.display()
        ));
    }
    // 主检出本身不得是别人的链接 worktree（common-dir 必须在自己内部）。
    let common = git(main_root, ["rev-parse", "--git-common-dir"])?.trim().to_string();
    let common_path = if Path::new(&common).is_absolute() {
        PathBuf::from(&common)
    } else {
        main_root.join(&common)
    };
    let want_common = std::fs::canonicalize(main_root.join(".git"))
        .unwrap_or_else(|_| main_root.join(".git"));
    let got_common = std::fs::canonicalize(&common_path).unwrap_or(common_path);
    if got_common != want_common {
        return Err(format!(
            "main root looks like a linked worktree (common dir {} ≠ {}) — refusing to bind",
            got_common.display(),
            want_common.display()
        ));
    }
    Ok(want)
}

/// The target repository's default branch — detected, never hardcoded
/// (§1: "消费工程默认分支由 Git 实际识别，不硬编码 main"). Order:
/// `refs/remotes/origin/HEAD` symbolic target → `init.defaultBranch` config
/// → current branch of the main checkout (local-only repos).
pub fn default_branch_of(main_root: &Path) -> Result<String, String> {
    if let Ok(head) = git(main_root, ["symbolic-ref", "refs/remotes/origin/HEAD"]) {
        let t = head.trim();
        if let Some(branch) = t.strip_prefix("refs/remotes/origin/") {
            if !branch.is_empty() {
                return Ok(branch.to_string());
            }
        }
    }
    if let Ok(cfg) = git(main_root, ["config", "--get", "init.defaultBranch"]) {
        let b = cfg.trim();
        if !b.is_empty() {
            return Ok(b.to_string());
        }
    }
    let cur = git(main_root, ["rev-parse", "--abbrev-ref", "HEAD"])?
        .trim()
        .to_string();
    if cur.is_empty() || cur == "HEAD" {
        return Err("cannot detect default branch (detached HEAD, no origin)".into());
    }
    Ok(cur)
}

/// Layout for one plan's execution worktree: dir + branch. Overridable via
/// `MUSK_PLAN_WORKTREE_ROOT` (tests point it into a tempdir); default places
/// the tree beside the main checkout (`.musk-wt/{plan_id}/{repo}`) — outside
/// the repository, never inside it.
pub fn worktree_layout(main_root: &Path, plan_id: &str, seq: u32) -> (PathBuf, String) {
    let repo_name = main_root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "repo".into());
    let base = std::env::var("MUSK_PLAN_WORKTREE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            main_root
                .parent()
                .map(|p| p.join(".musk-wt"))
                .unwrap_or_else(|| std::env::temp_dir().join(".musk-wt"))
        });
    let dir = base.join(plan_id.to_lowercase()).join(&repo_name);
    let branch = format!("plan-{seq:03}-dev");
    (dir, branch)
}

/// Scan `root` for reparse points (junctions/symlinks), the wt-guard
/// discipline: any hit blocks create/reuse/remove (Plan 529 incident:
/// `git worktree remove` recursion follows junctions and deletes targets).
/// Implementation mirrors wt-guard.sh (`dir /s /b /a:l`) on Windows; on
/// other targets it walks with `symlink_metadata` (covers symlinks/junctions
/// via `FileType::is_symlink`).
pub fn scan_reparse_points(root: &Path) -> Result<Vec<PathBuf>, String> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    #[cfg(windows)]
    {
        let wpath = std::ffi::OsString::from(root);
        // d://wt-guard 实测定式：MSYS_NO_PATHCONV=1 + 参数分传（dir 原生枚举）。
        // 退出码不判定（见下）。
        let out = Command::new("cmd")
            .arg("/c")
            .arg("dir")
            .arg("/s")
            .arg("/b")
            .arg("/a:l")
            .arg(&wpath)
            .output()
            .map_err(|e| format!("reparse scan spawn failed: {e}"))?;
        let text = String::from_utf8_lossy(&out.stdout);
        let links: Vec<PathBuf> = text
            .lines()
            .map(str::trim)
            .filter(|l| {
                !l.is_empty()
                    && !l.to_ascii_lowercase().contains("file not found")
            })
            .map(PathBuf::from)
            .collect();
        // 无命中时 dir 以非零码退出并在 stderr 报 "File Not Found"——这正是
        // clean 情形（wt-guard.sh 只按输出行判定，不看退出码）。
        Ok(links)
    }
    #[cfg(not(windows))]
    {
        let mut hits = Vec::new();
        fn walk(dir: &Path, hits: &mut Vec<PathBuf>) -> std::io::Result<()> {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                let ft = std::fs::symlink_metadata(&path)?.file_type();
                if ft.is_symlink() {
                    hits.push(path);
                } else if ft.is_dir() {
                    walk(&path, hits)?;
                }
            }
            Ok(())
        }
        walk(root, &mut hits).map_err(|e| format!("reparse scan failed: {e}"))?;
        Ok(hits)
    }
}

/// Guard: refuse when any reparse point exists under `root`.
pub fn guard_clean(root: &Path) -> Result<(), String> {
    let links = scan_reparse_points(root)?;
    if !links.is_empty() {
        return Err(format!(
            "reparse points under {} — remove them manually (never rm -rf): {}",
            root.display(),
            links.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(", ")
        ));
    }
    Ok(())
}

/// List the main repo's registered worktrees: `path branch` pairs
/// (`git worktree list --porcelain`).
fn registered_worktrees(main_root: &Path) -> Result<Vec<(PathBuf, String)>, String> {
    let out = git(main_root, ["worktree", "list", "--porcelain"])?;
    let mut result = Vec::new();
    let mut path: Option<PathBuf> = None;
    for line in out.lines() {
        if let Some(p) = line.strip_prefix("worktree ") {
            path = Some(PathBuf::from(p));
        } else if let Some(b) = line.strip_prefix("branch ") {
            if let Some(p) = path.take() {
                result.push((p, b.trim_start_matches("refs/heads/").to_string()));
            }
        }
    }
    Ok(result)
}

/// Create (or reuse a registered, branch-matching) execution worktree for
/// `plan_id`. Never takes over a foreign directory or an existing branch.
pub fn ensure_plan_worktree(
    main_root: &Path,
    plan_id: &str,
    seq: u32,
    base_commit: &str,
) -> Result<WorktreeLease, String> {
    let main = validate_main_checkout(main_root)?;
    let (dir, branch) = worktree_layout(&main, plan_id, seq);
    // 目录占用核：存在且未登记为本仓同名 worktree → 拒绝接管。
    if dir.exists() {
        let registered = registered_worktrees(&main)?
            .into_iter()
            .find(|(p, b)| {
                let same = std::fs::canonicalize(p).unwrap_or_else(|_| p.clone())
                    == std::fs::canonicalize(&dir).unwrap_or_else(|_| dir.clone());
                same && *b == branch
            });
        if registered.is_none() {
            return Err(format!(
                "worktree path {} exists but is not this repo's registered '{branch}' worktree — refusing to take over",
                dir.display()
            ));
        }
        guard_clean(&dir)?;
        // 复用：核对分支与 top-level。
        let got_branch = git(&dir, ["rev-parse", "--abbrev-ref", "HEAD"])?.trim().to_string();
        if got_branch != branch {
            return Err(format!(
                "worktree {} is on '{got_branch}', expected '{branch}' — refusing",
                dir.display()
            ));
        }
        let toplevel = PathBuf::from(git(&dir, ["rev-parse", "--show-toplevel"])?.trim());
        let toplevel = std::fs::canonicalize(&toplevel).unwrap_or(toplevel);
        return Ok(WorktreeLease {
            main_root: main.display().to_string(),
            worktree_root: toplevel.display().to_string(),
            branch,
            base_commit: base_commit.to_string(),
            created: false,
        });
    }
    // 新建：base 必须存在。
    let base = git(&main, ["rev-parse", "--verify", &format!("{base_commit}^{{commit}}")])
        .map_err(|e| format!("base commit {base_commit} not found: {e}"))?;
    let _ = base;
    if let Some(parent) = dir.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("worktree layout dir create failed: {e}"))?;
        guard_clean(parent)?;
    }
    git(&main, ["worktree", "add", "-b", &branch, &dir.to_string_lossy(), base_commit])
        .map_err(|e| format!("worktree add failed: {e}"))?;
    guard_clean(&dir)?;
    let toplevel = PathBuf::from(git(&dir, ["rev-parse", "--show-toplevel"])?.trim());
    let toplevel = std::fs::canonicalize(&toplevel).unwrap_or(toplevel);
    Ok(WorktreeLease {
        main_root: main.display().to_string(),
        worktree_root: toplevel.display().to_string(),
        branch,
        base_commit: base_commit.to_string(),
        created: true,
    })
}

/// True when every commit of `branch` is contained in `target` (all work
/// merged — removal precondition).
pub fn branch_fully_merged(main_root: &Path, branch: &str, target: &str) -> Result<bool, String> {
    let branch_head = git(main_root, ["rev-parse", "--verify", &format!("refs/heads/{branch}")])?
        .trim()
        .to_string();
    let target_head = git(main_root, ["rev-parse", "--verify", &format!("{target}")])?
        .trim()
        .to_string();
    if branch_head == target_head {
        return Ok(true);
    }
    let out = Command::new("git")
        .args(["merge-base", "--is-ancestor", &branch_head, &target_head])
        .current_dir(main_root)
        .status()
        .map_err(|e| format!("merge-base spawn failed: {e}"))?;
    Ok(out.success())
}

/// Remove the plan worktree + its dev branch. Preconditions (AC-11):
/// fresh guard scan clean on the tree, all branch commits merged into the
/// target branch. The layout group dir is pruned only when empty.
pub fn remove_plan_worktree(
    main_root: &Path,
    lease: &WorktreeLease,
    default_branch: &str,
) -> Result<(), String> {
    // 合入核对在前（AC-11：清理失败保留现场——worktree 不动，报 pending）。
    let merged = branch_fully_merged(main_root, &lease.branch, default_branch)?;
    if !merged {
        return Err(format!(
            "branch '{}' has unmerged commits — keeping worktree and branch, reporting cleanup_pending",
            lease.branch
        ));
    }
    let wt = PathBuf::from(&lease.worktree_root);
    if wt.exists() {
        guard_clean(&wt)?;
        // 注册归属核对：必须是本仓登记的、分支匹配的 worktree。
        let registered = registered_worktrees(main_root)?
            .into_iter()
            .any(|(p, b)| {
                let same = std::fs::canonicalize(&p).unwrap_or_else(|_| p.clone())
                    == std::fs::canonicalize(&wt).unwrap_or_else(|_| wt.clone());
                same && b == lease.branch
            });
        if !registered {
            return Err(format!(
                "worktree {} is not registered on branch '{}' — refusing removal",
                wt.display(),
                lease.branch
            ));
        }
        git(main_root, ["worktree", "remove", &wt.to_string_lossy()])
            .map_err(|e| format!("worktree remove failed: {e}"))?;
    }
    git(main_root, ["branch", "-d", &lease.branch])
        .map_err(|e| format!("branch delete failed: {e}"))?;
    // 组目录空则删（layout = base/{plan}/{repo}；父两级为组）。
    if let Some(plan_dir) = wt.parent() {
        if dir_is_empty(plan_dir) {
            let _ = std::fs::remove_dir(plan_dir);
        }
        if let Some(group) = plan_dir.parent() {
            if dir_is_empty(group) {
                let _ = std::fs::remove_dir(group);
            }
        }
    }
    Ok(())
}

fn dir_is_empty(dir: &Path) -> bool {
    std::fs::read_dir(dir).map(|mut rd| rd.next().is_none()).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Init a tiny git repo with one commit on the default branch.
    fn init_repo(dir: &Path) -> String {
        std::fs::create_dir_all(dir).unwrap();
        let _ = git(dir, ["init", "-b", "master"]).unwrap();
        git(dir, ["config", "user.email", "t@t"]).unwrap();
        git(dir, ["config", "user.name", "t"]).unwrap();
        std::fs::write(dir.join("README.md"), "seed\n").unwrap();
        git(dir, ["add", "."]).unwrap();
        git(dir, ["commit", "-m", "seed"]).unwrap();
        git(dir, ["rev-parse", "HEAD"]).unwrap().trim().to_string()
    }

    #[test]
    fn validate_main_checkout_accepts_repo_rejects_subdir() {
        let td = tempfile::tempdir().unwrap();
        let repo = td.path().join("repo");
        init_repo(&repo);
        assert!(validate_main_checkout(&repo).is_ok());
        // 子目录不是 top-level。
        let sub = repo.join("docs").join("plans");
        std::fs::create_dir_all(&sub).unwrap();
        assert!(validate_main_checkout(&sub).is_err());
        // 非 git 目录拒绝。
        let plain = td.path().join("plain");
        std::fs::create_dir_all(&plain).unwrap();
        assert!(validate_main_checkout(&plain).is_err());
    }

    #[test]
    fn default_branch_detection_prefers_origin_head() {
        let td = tempfile::tempdir().unwrap();
        let repo = td.path().join("repo");
        init_repo(&repo); // on master, no origin
        // 无 origin：回退当前分支。
        assert_eq!(default_branch_of(&repo).unwrap(), "master");
        // 建一个 origin remote 指针（bare clone）后优先 origin/HEAD。
        let bare = td.path().join("origin.git");
        git(td.path(), ["clone", "--bare", &repo.to_string_lossy(), &bare.to_string_lossy()]).unwrap();
        // bare 仓的 HEAD 默认指向其分支；把 origin 加为 remote 并 fetch。
        git(&repo, ["remote", "add", "origin", &bare.to_string_lossy()]).unwrap();
        git(&repo, ["fetch", "origin"]).unwrap();
        git(&repo, ["remote", "set-head", "origin", "-a"]).unwrap();
        assert_eq!(default_branch_of(&repo).unwrap(), "master");
    }

    #[test]
    fn worktree_layout_uses_env_root_and_plan_branch() {
        std::env::remove_var("MUSK_PLAN_WORKTREE_ROOT");
        let td = tempfile::tempdir().unwrap();
        let repo = td.path().join("repo");
        std::fs::create_dir_all(&repo).unwrap();
        let (dir, branch) = worktree_layout(&repo, "PLAN-007", 7);
        assert_eq!(branch, "plan-007-dev");
        assert!(dir.ends_with("plan-007/repo"));
        std::env::set_var("MUSK_PLAN_WORKTREE_ROOT", "/tmp/wt-override");
        let (dir2, _) = worktree_layout(&repo, "PLAN-007", 7);
        assert!(dir2.starts_with("/tmp/wt-override"));
        std::env::remove_var("MUSK_PLAN_WORKTREE_ROOT");
    }

    /// 创建→复用→提交→合回→安全移除全链（真 Git）。
    #[test]
    fn ensure_reuse_and_safe_removal_roundtrip() {
        std::env::remove_var("MUSK_PLAN_WORKTREE_ROOT");
        let td = tempfile::tempdir().unwrap();
        let repo = td.path().join("repo");
        let base = init_repo(&repo);
        std::env::set_var("MUSK_PLAN_WORKTREE_ROOT", td.path().join("wt"));
        let lease = ensure_plan_worktree(&repo, "PLAN-009", 9, &base).unwrap();
        assert!(lease.created);
        assert!(Path::new(&lease.worktree_root).is_dir());
        assert_eq!(lease.branch, "plan-009-dev");

        // 复用（同参再取 → created=false，同一路径）。
        let again = ensure_plan_worktree(&repo, "PLAN-009", 9, &base).unwrap();
        assert!(!again.created);
        assert_eq!(again.worktree_root, lease.worktree_root);

        // 外部目录占用同名路径 → 拒绝接管。
        // （先注销 worktree，模拟目录被他人占用。）
        let _ = git(&repo, ["worktree", "remove", &lease.worktree_root]);
        // 重建同路径的普通目录（模拟外部占用者）。
        std::fs::create_dir_all(&lease.worktree_root).unwrap();
        std::fs::write(
            std::path::Path::new(&lease.worktree_root).join("stranger.txt"),
            "hands off",
        )
        .unwrap();
        let err = ensure_plan_worktree(&repo, "PLAN-009", 9, &base).unwrap_err();
        assert!(err.contains("refusing to take over"), "{err}");
        std::fs::remove_dir_all(&lease.worktree_root).unwrap();

        // 分支已存而目录不在：碰撞不接管（该分支可能载有未回合并的工作）。
        let err = ensure_plan_worktree(&repo, "PLAN-009", 9, &base).unwrap_err();
        assert!(err.contains("worktree add failed"), "{err}");

        // 未合入提交 → 拒绝删分支、保留现场（cleanup_pending 语义）。
        // （新计划号：上一段的 plan-009-dev 分支仍在，属已知占用。）
        let lease2 = ensure_plan_worktree(&repo, "PLAN-010", 10, &base).unwrap();
        std::fs::write(
            std::path::Path::new(&lease2.worktree_root).join("wip.txt"),
            "x",
        )
        .unwrap();
        git(
            std::path::Path::new(&lease2.worktree_root),
            ["add", "."],
        )
        .unwrap();
        git(
            std::path::Path::new(&lease2.worktree_root),
            ["commit", "-m", "wip"],
        )
        .unwrap();
        let err = remove_plan_worktree(&repo, &lease2, "master").unwrap_err();
        assert!(err.contains("unmerged"), "{err}");
        assert!(std::path::Path::new(&lease2.worktree_root).exists(), "scene kept");

        // 合回后安全移除。
        git(&repo, ["merge", "--ff-only", &lease2.branch]).unwrap();
        remove_plan_worktree(&repo, &lease2, "master").unwrap();
        assert!(!std::path::Path::new(&lease2.worktree_root).exists());
        // 分支已删。
        assert!(git(&repo, ["rev-parse", "--verify", &format!("refs/heads/{}", lease2.branch)]).is_err());
        std::env::remove_var("MUSK_PLAN_WORKTREE_ROOT");
    }

    /// reparse 扫描：普通目录 clean；模拟违规（普通目录内建 symlink——
    /// 不在任何 git worktree 内创建，遵守 AGENTS 红线）→ guard 拒绝。
    #[test]
    fn guard_blocks_reparse_points() {
        let td = tempfile::tempdir().unwrap();
        let inner = td.path().join("inner");
        std::fs::create_dir_all(&inner).unwrap();
        assert!(guard_clean(td.path()).is_ok());
        // 建 symlink（unix）或 junction（windows）在普通目录中。
        #[cfg(unix)]
        std::os::unix::fs::symlink(&inner, td.path().join("link")).unwrap();
        #[cfg(windows)]
        {
            // mklink /J 建 junction（cmd 内建，无额外依赖）。
            let out = Command::new("cmd")
                .arg("/c")
                .arg("mklink")
                .arg("/J")
                .arg(td.path().join("link"))
                .arg(&inner)
                .output()
                .unwrap();
            assert!(out.status.success(), "junction create failed");
        }
        let err = guard_clean(td.path()).unwrap_err();
        assert!(err.contains("reparse"), "{err}");
        // 清理链接本身（只删链接，不穿透）。
        #[cfg(unix)]
        std::fs::remove_file(td.path().join("link")).unwrap();
        #[cfg(windows)]
        {
            let _ = Command::new("cmd")
                .arg("/c")
                .arg("rmdir")
                .arg(td.path().join("link"))
                .status();
        }
        assert!(guard_clean(td.path()).is_ok());
    }
}
