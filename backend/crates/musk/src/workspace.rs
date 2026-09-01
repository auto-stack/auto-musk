//! Workspace registry — maps workspace ids to project roots and lazy-loads
//! each workspace's store bundle. See designs/006-workspace-multi-directory.md.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use crate::chats::ChatStore;
use crate::relay::store::RunStore;
use crate::specs::SpecsStore;
use crate::wiki::WikiStore;

/// One entry in the global workspaces.json index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMeta {
    pub id: String,
    pub path: String, // canonical project root
    pub name: String,
    pub last_opened: u64,
    /// True if the project directory has no user files yet (ignoring dotfiles
    /// like `.autoos`/`.git` and an empty `specs/` subdir). When true, the
    /// frontend shows the new-project onboarding dialog.
    #[serde(default)]
    pub is_empty: bool,
}

/// The on-disk index file at ~/.config/autoos/workspaces.json.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct WorkspaceIndex {
    #[serde(default)]
    workspaces: Vec<WorkspaceMeta>,
    #[serde(default)]
    default_workspace_id: Option<String>,
}

/// All musk data stores for one workspace, rooted at {root}/.autoos.
pub struct WorkspaceStores {
    pub root: PathBuf,
    pub specs: Arc<SpecsStore>,
    /// Plans live at `{root}/docs/plans/` (D1: workspace root, not .autoos).
    pub plans: Arc<crate::plans::PlansStore>,
    pub chats: Arc<ChatStore>,
    pub wiki: Arc<WikiStore>,
    pub relay: Arc<RunStore>,
    pub conversations: Arc<crate::conversation::ConversationStore>,
    pub handoffs: Arc<crate::relay::handoff_store::HandoffStore>,
    pub task_plans: Arc<std::sync::Mutex<crate::relay::task_plan_registry::TaskPlanRegistry>>,
}

pub struct WorkspaceRegistry {
    index: RwLock<WorkspaceIndex>,
    index_path: PathBuf,
    cache: RwLock<HashMap<PathBuf, Arc<WorkspaceStores>>>,
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn autoos_dir(root: &std::path::Path) -> PathBuf {
    root.join(".autoos")
}

/// Decide whether a project directory counts as "empty" for onboarding.
///
/// A directory is empty if it contains nothing but:
/// - dotfiles/dotdirs (`.autoos`, `.git`, `.vscode`, …) — always ignored
/// - an empty `specs/` subdir (a freshly-seeded spec ledger)
///
/// Any other file or non-empty directory → not empty.
///
/// Additionally, if `.autoos/initialized` exists, the project is considered
/// non-empty even with no source files yet — this prevents the onboarding
/// dialog from re-appearing after the user already completed it (the agent
/// may not have written files to the project root yet).
pub fn is_workspace_empty(root: &std::path::Path) -> bool {
    // Onboarding already completed? Then not empty.
    if autoos_dir(root).join("initialized").exists() {
        return false;
    }
    let Ok(dir) = std::fs::read_dir(root) else {
        return true; // missing / unreadable → treat as empty
    };
    for entry in dir.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue; // ignore all dotfiles/dotdirs
        }
        if entry.path().is_dir() {
            // An empty `specs/` is allowed (seeded ledger); anything else is content.
            if name == "specs" {
                let inner = std::fs::read_dir(entry.path());
                if inner.map_or(true, |mut d| d.next().is_none()) {
                    continue;
                }
            }
            return false;
        }
        return false; // any regular file → not empty
    }
    true
}

/// Move a single file, falling back to copy+delete if a direct rename fails
/// (e.g. across filesystems / devices). Best-effort: errors are swallowed.
fn move_file(src: &std::path::Path, dst: &std::path::Path) {
    if std::fs::rename(src, dst).is_ok() {
        return;
    }
    if std::fs::copy(src, dst)
        .and_then(|_| std::fs::remove_file(src))
        .is_ok()
    {
        return;
    }
}

/// Recursively move the contents of `src` into `dst` (dst is created if
/// missing). Existing entries in `dst` are not overwritten. Best-effort.
fn move_dir_contents(src: &std::path::Path, dst: &std::path::Path) {
    let entries = match std::fs::read_dir(src) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let from = entry.path();
        let to = dst.join(entry.file_name());
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        if metadata.is_dir() {
            if !to.exists() {
                // Try a direct rename first (cheap when dst fs matches).
                if std::fs::rename(&from, &to).is_ok() {
                    continue;
                }
            }
            let _ = std::fs::create_dir_all(&to);
            move_dir_contents(&from, &to);
            let _ = std::fs::remove_dir(&from);
        } else {
            if !to.exists() {
                move_file(&from, &to);
            }
        }
    }
}

/// Pick a unique workspace id under the lock held by `open()`. Returns `base`
/// if free, otherwise `{base}-{n}` for the smallest n that is free.
fn unique_id_locked(idx: &WorkspaceIndex, base: &str) -> String {
    if !idx.workspaces.iter().any(|m| m.id == base) {
        return base.to_string();
    }
    let mut n = 1;
    loop {
        let candidate = format!("{base}-{n}");
        if !idx.workspaces.iter().any(|m| m.id == candidate) {
            return candidate;
        }
        n += 1;
    }
}

impl WorkspaceRegistry {
    /// Load the index from `index_path`; if empty, seed a default workspace
    /// rooted at `default_root` (startup cwd / --workdir).
    pub fn load(index_path: PathBuf, default_root: PathBuf) -> Self {
        let index = Self::read_index(&index_path);
        let index = if index.workspaces.is_empty() {
            let canonical = std::fs::canonicalize(&default_root).unwrap_or(default_root.clone());
            let id = canonical
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "default".into());
            let meta = WorkspaceMeta {
                id: id.clone(),
                path: canonical.to_string_lossy().to_string(),
                name: id,
                last_opened: now_secs(),
                is_empty: is_workspace_empty(&canonical),
            };
            let seeded = WorkspaceIndex {
                default_workspace_id: Some(meta.id.clone()),
                workspaces: vec![meta],
            };
            let _ = Self::write_index(&index_path, &seeded);
            seeded
        } else {
            index
        };

        let reg = Self {
            index: RwLock::new(index.clone()),
            index_path: index_path.clone(),
            cache: RwLock::new(HashMap::new()),
        };
        if let Some(default_id) = &index.default_workspace_id {
            let _ = reg.get(default_id);
        }
        reg
    }

    fn read_index(path: &Path) -> WorkspaceIndex {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_default()
    }

    fn write_index(path: &Path, idx: &WorkspaceIndex) {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(idx) {
            let _ = std::fs::write(path, json);
        }
    }

    fn save(&self) {
        let idx = self.index.read().unwrap().clone();
        Self::write_index(&self.index_path, &idx);
    }

    /// Resolve a workspace id to its store bundle. Falls back to the default
    /// workspace if the id is missing/empty. Lazy-loads + caches by root path.
    pub fn get(&self, ws_id: &str) -> Arc<WorkspaceStores> {
        let meta = {
            let idx = self.index.read().unwrap();
            idx.workspaces
                .iter()
                .find(|m| m.id == ws_id)
                .cloned()
                .or_else(|| {
                    idx.default_workspace_id
                        .as_ref()
                        .and_then(|did| idx.workspaces.iter().find(|m| &m.id == did).cloned())
                })
                // Recover from a corrupted/stale index: if the requested id
                // and the default id both miss, fall back to the first entry
                // rather than panicking (which would crash startup via load()).
                .or_else(|| idx.workspaces.first().cloned())
        };
        let meta = match meta {
            Some(m) => m,
            None => panic!("no workspaces registered and no default"),
        };
        let root = PathBuf::from(&meta.path);
        // Double-checked locking: acquire the write lock, re-check the cache,
        // and only construct if still absent. This guarantees a single
        // WorkspaceStores per root even under concurrent get() calls.
        let mut cache = self.cache.write().unwrap();
        if let Some(stores) = cache.get(&root).cloned() {
            return stores;
        }
        let stores = Arc::new(WorkspaceStores::new(root.clone()));
        cache.insert(root, stores.clone());
        stores
    }

    /// Open (or reuse) a workspace for the given project root path. Creates
    /// `.autoos/`, assigns an id (dir name, suffixed on clash), persists index.
    pub fn open(&self, root_path: &str) -> WorkspaceMeta {
        let canonical = std::fs::canonicalize(root_path)
            .unwrap_or_else(|_| PathBuf::from(root_path));
        // Reuse if path already indexed — but re-check is_empty + refresh
        // last_opened, since the dir's contents (or the feature set) may have
        // changed since the entry was first recorded.
        {
            let mut idx = self.index.write().unwrap();
            if let Some(existing) = idx
                .workspaces
                .iter_mut()
                .find(|m| m.path == canonical.to_string_lossy().to_string())
            {
                existing.is_empty = is_workspace_empty(&canonical);
                existing.last_opened = now_secs();
                let out = existing.clone();
                drop(idx);
                self.save();
                return out;
            }
        }
        let _ = std::fs::create_dir_all(autoos_dir(&canonical));
        let base_id = canonical
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "workspace".into());
        let empty = is_workspace_empty(&canonical);
        // Compute the unique id AND push the new entry inside the same write
        // lock so two concurrent open() calls with the same dirname cannot
        // both receive the same id (TOCTOU).
        let meta = {
            let mut idx = self.index.write().unwrap();
            let id = unique_id_locked(&idx, &base_id);
            let meta = WorkspaceMeta {
                id: id.clone(),
                path: canonical.to_string_lossy().to_string(),
                name: base_id,
                last_opened: now_secs(),
                is_empty: empty,
            };
            idx.workspaces.push(meta.clone());
            if idx.default_workspace_id.is_none() {
                idx.default_workspace_id = Some(id);
            }
            meta
        };
        self.save();
        meta
    }

    /// List workspaces, most-recently-opened first.
    pub fn list(&self) -> Vec<WorkspaceMeta> {
        let idx = self.index.read().unwrap();
        let mut v = idx.workspaces.clone();
        v.sort_by(|a, b| b.last_opened.cmp(&a.last_opened));
        v
    }

    /// Update last_opened for a workspace (called on every switch).
    pub fn touch(&self, ws_id: &str) {
        let mut idx = self.index.write().unwrap();
        if let Some(m) = idx.workspaces.iter_mut().find(|m| m.id == ws_id) {
            m.last_opened = now_secs();
        }
        drop(idx);
        self.save();
    }

    /// Best-effort: if old global data exists at `global_dir` and the default
    /// workspace's .autoos/ has no specs.json yet, move the old files/dirs into
    /// it. Idempotent (guarded by the top-level specs.json check).
    pub fn migrate_global_data(&self, global_dir: &std::path::Path) {
        let idx = self.index.read().unwrap().clone();
        let Some(default_id) = &idx.default_workspace_id else { return };
        let Some(default_meta) = idx.workspaces.iter().find(|m| &m.id == default_id) else { return };
        let autoos = autoos_dir(&PathBuf::from(&default_meta.path));
        // Only migrate if .autoos has no specs.json yet (idempotent guard).
        // Note: the store constructors may have pre-created empty wiki/raw/relay
        // subdirs, so the per-subdir guard alone is not enough; this top-level
        // specs.json check is what makes the whole operation idempotent.
        if autoos.join("specs.json").exists() {
            return;
        }
        let _ = std::fs::create_dir_all(&autoos);
        // Move loose JSON files.
        for name in ["specs.json", "chats.json"] {
            let src = global_dir.join(name);
            let dst = autoos.join(name);
            if src.exists() && !dst.exists() {
                move_file(&src, &dst);
            }
        }
        // Move data subdirectories' contents. The dst dirs may already exist
        // (created empty by the store constructors), so move entry-by-entry.
        for sub in ["wiki", "raw", "relay"] {
            let src = global_dir.join(sub);
            if src.is_dir() {
                let dst = autoos.join(sub);
                let _ = std::fs::create_dir_all(&dst);
                move_dir_contents(&src, &dst);
            }
        }
    }

    /// Rename a workspace's display name.
    pub fn rename(&self, ws_id: &str, name: &str) -> Option<WorkspaceMeta> {
        let mut idx = self.index.write().unwrap();
        let m = idx.workspaces.iter_mut().find(|m| m.id == ws_id)?;
        m.name = name.to_string();
        let out = m.clone();
        drop(idx);
        self.save();
        Some(out)
    }
}

/// Query extractor: `?workspace=<id>` on business endpoints. Empty/absent →
/// falls back to the default workspace inside `registry.get`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct WorkspaceQuery {
    #[serde(default)]
    pub workspace: Option<String>,
}

impl WorkspaceQuery {
    /// Resolve to a concrete id (empty → default workspace id).
    pub fn id_or_default(&self, registry: &WorkspaceRegistry) -> String {
        match &self.workspace {
            Some(id) if !id.is_empty() => id.clone(),
            _ => registry
                .index
                .read()
                .ok()
                .and_then(|idx| idx.default_workspace_id.clone())
                .unwrap_or_default(),
        }
    }
}

impl WorkspaceStores {
    /// Instantiate all stores rooted at {root}/.autoos.
    pub fn new(root: PathBuf) -> Self {
        let data = autoos_dir(&root);
        let _ = std::fs::create_dir_all(&data);
        let stores = Self {
            specs: Arc::new(SpecsStore::new(data.join("specs.json"))),
            plans: Arc::new(crate::plans::PlansStore::new(root.join("docs/plans"))),
            chats: Arc::new(ChatStore::at(data.join("chats.json"))),
            wiki: Arc::new(WikiStore::new(data.join("wiki"), data.join("raw"))),
            relay: Arc::new(RunStore::at(data.join("relay"))),
            conversations: Arc::new(crate::conversation::ConversationStore::new(
                data.join("conversations"),
            )),
            handoffs: Arc::new(crate::relay::handoff_store::HandoffStore::new(&data)),
            task_plans: Arc::new(std::sync::Mutex::new(
                crate::relay::task_plan_registry::TaskPlanRegistry::new(&data),
            )),
            root,
        };
        // Migrate old chat sessions into the unified conversation model (idempotent).
        stores.conversations.migrate_chats(&stores.chats);
        // Link the run store to the conversation store so relay events are
        // dual-written as turns into a Flow conversation sharing the run id.
        stores
            .relay
            .link_conversations(stores.conversations.clone());
        stores
    }
}

// ============================================================
// Native folder picker route (hw 轨,serve() merge 进 ag 主 router)。
// 浏览器拿不到用户所选目录的绝对路径(File System Access API 只暴露
// handle 名),而 /api/workspace/open 需要绝对路径——由本机 serve 进程经
// rfd 弹原生系统文件夹选择器,选中即回路径。与 plans::plans_routes 同
// 模式(hw 路由;KNOWN-DEBT: a2r 对齐后可迁 ag 轨)。
// ============================================================

use axum::{
    routing::post,
    Json, Router,
};

use crate::server::AppState;

/// `POST /api/workspace/pick` — 弹出系统文件夹选择器,返回 `{"path": "..."}`;
/// 用户取消/失败返回 `{"path": null}`。对话框模态等待用户,阻塞放在
/// spawn_blocking(独占 blocking 线程,不占 worker)。
async fn workspace_pick() -> Json<serde_json::Value> {
    let picked = tokio::task::spawn_blocking(|| rfd::FileDialog::new().pick_folder())
        .await
        .ok()
        .flatten();
    Json(serde_json::json!({
        "path": picked.map(|p| p.to_string_lossy().to_string())
    }))
}

/// workspace 扩展路由(原生文件夹选择)。
pub fn pick_routes() -> Router<AppState> {
    Router::new().route("/api/workspace/pick", post(workspace_pick))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_registry() -> (WorkspaceRegistry, PathBuf) {
        let dir = std::env::temp_dir().join(format!(
            "musk-ws-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let index_path = dir.join("workspaces.json");
        let reg = WorkspaceRegistry::load(index_path.clone(), dir.clone());
        (reg, dir)
    }

    #[test]
    fn loads_with_default_workspace_when_index_empty() {
        let (reg, dir) = tmp_registry();
        let list = reg.list();
        assert_eq!(list.len(), 1, "default workspace seeded");
        assert_eq!(list[0].path, std::fs::canonicalize(&dir).unwrap().to_string_lossy());
    }

    #[test]
    fn open_new_workspace_assigns_dirname_id() {
        let (reg, _default) = tmp_registry();
        let new_root = std::env::temp_dir().join("ws-target-unique");
        std::fs::create_dir_all(&new_root).unwrap();
        let meta = reg.open(&new_root.to_string_lossy());
        assert_eq!(meta.id, "ws-target-unique");
        assert!(meta.path.contains("ws-target-unique"));
        // .autoos created
        assert!(new_root.join(".autoos").exists());
    }

    #[test]
    fn open_duplicate_path_reuses_id() {
        let (reg, _default) = tmp_registry();
        let new_root = std::env::temp_dir().join("ws-dup-unique");
        std::fs::create_dir_all(&new_root).unwrap();
        let m1 = reg.open(&new_root.to_string_lossy());
        let m2 = reg.open(&new_root.to_string_lossy());
        assert_eq!(m1.id, m2.id);
        assert_eq!(reg.list().iter().filter(|m| m.id == m1.id).count(), 1);
    }

    #[test]
    fn open_clashing_dirname_gets_suffix() {
        let (reg, _default) = tmp_registry();
        let a = std::env::temp_dir().join("ws-clash/one/myproj");
        let b = std::env::temp_dir().join("ws-clash/two/myproj");
        std::fs::create_dir_all(&a).unwrap();
        std::fs::create_dir_all(&b).unwrap();
        let m1 = reg.open(&a.to_string_lossy());
        let m2 = reg.open(&b.to_string_lossy());
        assert_eq!(m1.id, "myproj");
        assert_eq!(m2.id, "myproj-1", "second clash gets -1 suffix");
    }

    #[test]
    fn get_returns_default_bundle_and_caches_it() {
        let (reg, dir) = tmp_registry();
        let default = reg.list().into_iter().next().unwrap();
        let canonical_root = std::fs::canonicalize(&dir).unwrap();

        // First get(): root matches the canonical default root.
        let bundle = reg.get(&default.id);
        assert_eq!(bundle.root, canonical_root);

        // Second get(): returns the same bundle (cache hit).
        let bundle_again = reg.get(&default.id);
        assert!(
            Arc::ptr_eq(&bundle, &bundle_again),
            "get() must return the same Arc on a cache hit"
        );

        // Fallback: a nonexistent id resolves to the default workspace's bundle.
        let fallback = reg.get("does-not-exist");
        assert_eq!(fallback.root, canonical_root);
        assert!(
            Arc::ptr_eq(&fallback, &bundle),
            "fallback to default must reuse the cached default bundle"
        );
    }

    #[test]
    fn migrate_moves_global_data_into_default_autoos() {
        let dir = std::env::temp_dir().join(format!(
            "musk-ws-migrate-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        // Simulate old global data.
        let global = dir.join("global");
        std::fs::create_dir_all(&global).unwrap();
        std::fs::write(global.join("specs.json"), "{\"old\":true}").unwrap();
        std::fs::create_dir_all(global.join("wiki")).unwrap();
        std::fs::write(global.join("wiki/page.md"), "# old").unwrap();
        // Default workspace root = a fresh dir under `dir`.
        let ws_root = dir.join("myproj");
        std::fs::create_dir_all(&ws_root).unwrap();
        let reg = WorkspaceRegistry::load(dir.join("workspaces.json"), ws_root.clone());
        reg.migrate_global_data(&global);
        // Files moved into {ws_root}/.autoos/
        assert!(ws_root.join(".autoos/specs.json").exists());
        assert!(ws_root.join(".autoos/wiki/page.md").exists());
        // Idempotent: second call is a no-op (specs.json now exists in .autoos).
        reg.migrate_global_data(&global);
    }

    #[test]
    fn migrate_skips_if_autoos_already_has_specs() {
        let dir = std::env::temp_dir().join(format!(
            "musk-ws-migrate-skip-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let global = dir.join("global");
        std::fs::create_dir_all(&global).unwrap();
        std::fs::write(global.join("specs.json"), "{\"old\":true}").unwrap();
        let ws_root = dir.join("myproj2");
        std::fs::create_dir_all(&ws_root).unwrap();
        // Pre-seed .autoos/specs.json so migration should skip.
        std::fs::create_dir_all(ws_root.join(".autoos")).unwrap();
        std::fs::write(ws_root.join(".autoos/specs.json"), "{\"new\":true}").unwrap();
        let reg = WorkspaceRegistry::load(dir.join("workspaces.json"), ws_root.clone());
        reg.migrate_global_data(&global);
        // .autoos/specs.json NOT overwritten by the old one.
        let content = std::fs::read_to_string(ws_root.join(".autoos/specs.json")).unwrap();
        assert!(content.contains("new"), "existing .autoos data must not be overwritten");
    }

    #[test]
    fn is_empty_truly_empty_dir() {
        let dir = tmp_dir();
        assert!(is_workspace_empty(&dir));
    }

    #[test]
    fn is_empty_ignores_dotfiles_and_autoos() {
        let dir = tmp_dir();
        std::fs::create_dir_all(dir.join(".autoos")).unwrap();
        std::fs::create_dir_all(dir.join(".git")).unwrap();
        std::fs::write(dir.join(".gitignore"), "*").unwrap();
        assert!(is_workspace_empty(&dir), "dotfiles/.autoos must not count as content");
    }

    #[test]
    fn is_empty_ignores_empty_specs_dir() {
        let dir = tmp_dir();
        std::fs::create_dir_all(dir.join("specs")).unwrap(); // empty specs/
        assert!(is_workspace_empty(&dir), "empty specs/ must not count as content");
    }

    #[test]
    fn is_empty_false_when_source_file_present() {
        let dir = tmp_dir();
        std::fs::write(dir.join("README.md"), "# hi").unwrap();
        assert!(!is_workspace_empty(&dir));
    }

    #[test]
    fn is_empty_false_when_non_empty_specs_present() {
        let dir = tmp_dir();
        std::fs::create_dir_all(dir.join("specs")).unwrap();
        std::fs::write(dir.join("specs/goal.md"), "# G1").unwrap();
        assert!(!is_workspace_empty(&dir), "non-empty specs/ counts as content");
    }

    #[test]
    fn is_empty_false_when_src_dir_present() {
        let dir = tmp_dir();
        std::fs::create_dir_all(dir.join("src")).unwrap();
        assert!(!is_workspace_empty(&dir), "any non-specs dir counts as content");
    }

    #[test]
    fn is_empty_false_after_initialized_marker() {
        let dir = tmp_dir();
        // Empty dir with only .autoos → empty.
        std::fs::create_dir_all(dir.join(".autoos")).unwrap();
        assert!(is_workspace_empty(&dir));
        // After onboarding drops the marker → not empty.
        std::fs::write(dir.join(".autoos/initialized"), "1").unwrap();
        assert!(!is_workspace_empty(&dir), "initialized marker must prevent re-onboarding");
    }

    /// Helper: a fresh empty temp dir for is_empty tests.
    fn tmp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "musk-ws-empty-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }
}
