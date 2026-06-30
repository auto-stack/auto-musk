//! Workspace registry — maps workspace ids to project roots and lazy-loads
//! each workspace's store bundle. See designs/006-workspace-multi-directory.md.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
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
    pub chats: Arc<ChatStore>,
    pub wiki: Arc<WikiStore>,
    pub relay: Arc<RunStore>,
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

    fn read_index(path: &PathBuf) -> WorkspaceIndex {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_default()
    }

    fn write_index(path: &PathBuf, idx: &WorkspaceIndex) {
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
        };
        let meta = match meta {
            Some(m) => m,
            None => panic!("no workspaces registered and no default"),
        };
        let root = PathBuf::from(&meta.path);
        {
            let cache = self.cache.read().unwrap();
            if let Some(stores) = cache.get(&root) {
                return stores.clone();
            }
        }
        let stores = Arc::new(WorkspaceStores::new(root.clone()));
        self.cache.write().unwrap().insert(root.clone(), stores.clone());
        stores
    }

    /// Open (or reuse) a workspace for the given project root path. Creates
    /// `.autoos/`, assigns an id (dir name, suffixed on clash), persists index.
    pub fn open(&self, root_path: &str) -> WorkspaceMeta {
        let canonical = std::fs::canonicalize(root_path)
            .unwrap_or_else(|_| PathBuf::from(root_path));
        // Reuse if path already indexed.
        {
            let idx = self.index.read().unwrap();
            if let Some(existing) = idx
                .workspaces
                .iter()
                .find(|m| m.path == canonical.to_string_lossy().to_string())
            {
                return existing.clone();
            }
        }
        let _ = std::fs::create_dir_all(autoos_dir(&canonical));
        let base_id = canonical
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "workspace".into());
        let id = self.unique_id(&base_id);
        let meta = WorkspaceMeta {
            id: id.clone(),
            path: canonical.to_string_lossy().to_string(),
            name: base_id,
            last_opened: now_secs(),
        };
        {
            let mut idx = self.index.write().unwrap();
            idx.workspaces.push(meta.clone());
            if idx.default_workspace_id.is_none() {
                idx.default_workspace_id = Some(id);
            }
        }
        self.save();
        meta
    }

    fn unique_id(&self, base: &str) -> String {
        let idx = self.index.read().unwrap();
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

impl WorkspaceStores {
    /// Instantiate all stores rooted at {root}/.autoos.
    pub fn new(root: PathBuf) -> Self {
        let data = autoos_dir(&root);
        let _ = std::fs::create_dir_all(&data);
        Self {
            specs: Arc::new(SpecsStore::new(data.join("specs.json"))),
            chats: Arc::new(ChatStore::at(data.join("chats.json"))),
            wiki: Arc::new(WikiStore::new(data.join("wiki"), data.join("raw"))),
            relay: Arc::new(RunStore::at(data.join("relay"))),
            root,
        }
    }
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
}
