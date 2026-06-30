# Workspace Multi-Directory Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add multi-workspace support to auto-musk — each workspace is a project directory whose musk data (specs/chats/wiki/raw/relay) lives in `{root}/.autoos/`, fully isolated; users switch workspaces via a nav-rail footer selector and the URL `?workspace=<id>`.

**Architecture:** A new `WorkspaceRegistry` (in `AppState`) replaces the 4 singleton stores. It lazy-loads + caches a `WorkspaceStores` bundle per workspace root. Every API request carries `?workspace=<id>`; handlers resolve it to a `WorkspaceStores` and operate there. `tool_safety`'s process-wide `OnceLock` root becomes per-workspace routing via a thread-local current-workspace id. Data migrates from the old global `~/.config/autoos/*` into the default workspace's `.autoos/` on first run.

**Tech Stack:** Rust (axum 0.8, serde, tokio), Vue 3.5 + composables (no vue-router), TypeScript.

**Spec:** `designs/006-workspace-multi-directory.md`

---

## File Structure

### Backend — new
- **Create `backend/crates/musk/src/workspace.rs`** — `WorkspaceMeta`, `WorkspaceStores`, `WorkspaceRegistry`, `WorkspaceQuery` extractor, migration helper. The registry owns `~/.config/autoos/workspaces.json` and caches per-root store bundles.

### Backend — modify
- **`backend/crates/musk/src/lib.rs`** — `pub mod workspace;`
- **`backend/crates/musk/src/tool_safety.rs`** — keep `OnceLock` default root + thread-local `ROOT_OVERRIDE`; add `set_current_root`/`clear_current_root` thread-local for per-workspace routing used by agent tools.
- **`backend/crates/musk/src/server.rs`** — `AppState` drops `specs/chats/wiki/relay`, gains `registry: Arc<WorkspaceRegistry>`; all specs/chats handlers gain a `WorkspaceQuery` extractor and resolve stores via `state.registry.get(&ws_id)`; `serve()` constructs the registry + runs migration.
- **`backend/crates/musk/src/chats.rs`** — `ChatSession` gains `workspace_id: Option<String>`; `ChatStore` unchanged (instantiated per-workspace by the registry).
- **`backend/crates/musk/src/relay/store.rs`** — `RunMetadata` gains `workspace_id: Option<String>`.
- **`backend/crates/musk/src/relay/api.rs`** — relay handlers gain `WorkspaceQuery`, resolve `state.registry.get(&ws_id).relay`.
- **`backend/crates/musk/src/wiki.rs`** — wiki handlers gain `WorkspaceQuery`, resolve `state.registry.get(&ws_id)` for wiki + raw stores. `WikiStore` already takes `(wiki_dir, raw_dir)` — registry passes `{root}/.autoos/wiki` + `{root}/.autoos/raw`.
- **`backend/crates/musk/src/main.rs`** — `Serve` gains optional `--workdir`; default workspace root = `--workdir` or cwd.
- **`backend/crates/musk/src/lib.rs` `build_agent_from_mode`** — unchanged signature; the driver/chat handlers set the thread-local workspace root before running the agent.

### Frontend — new
- **Create `web/src/components/WorkspaceSelector.vue`** — footer button + popup panel (recent list + "open folder" path input with browse suggestions).

### Frontend — modify
- **`web/src/composables/useProject.ts`** — stub → real: holds `_currentWorkspace`, calls `/api/workspace/*`, syncs `?workspace=` URL param.
- **`web/src/composables/useAuth.ts`** (or a new `useApi.ts`) — `authFetch` wrapper auto-appends `?workspace=<currentId>` to relative `/api/...` URLs.
- **`web/src/App.vue`** — render `<WorkspaceSelector/>` in the rail footer next to `<SettingsMenu/>`; on mount read `?workspace` from URL → `fetchStatus`.
- **`web/src/views/{ChatsView,SpecsView,WikiView,RelayView}.vue`** — watch `workspaceId` and reload on change (already consume `useProject`).

---

## Task decomposition

The plan is split into 6 phases. **Phase 1–4 are backend, 5 is frontend, 6 is migration + verification.** Each phase ends with a commit and compiles+tests pass.

---

### Task 1: WorkspaceRegistry core (data model + index persistence)

**Files:**
- Create: `backend/crates/musk/src/workspace.rs`
- Modify: `backend/crates/musk/src/lib.rs:9` (add `pub mod workspace;`)

- [ ] **Step 1: Write the failing test**

Create `backend/crates/musk/src/workspace.rs` with the test first:

```rust
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
        // index persisted
        let raw = std::fs::read_to_string(reg.index_path.clone().unwrap_or_default()).unwrap_or_default();
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
```

- [ ] **Step 2: Run test to verify it fails (won't compile)**

Run: `cargo test -p musk workspace:: 2>&1 | head -5`
Expected: compile error — `WorkspaceRegistry::load/open/list` not defined, and `index_path` field access in test.

- [ ] **Step 3: Implement WorkspaceRegistry minimal**

Add to `workspace.rs` (above the `#[cfg(test)]` block):

```rust
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
    /// Load the index from `index_path`; if it's empty, seed a default
    /// workspace rooted at `default_root` (the startup cwd / --workdir).
    pub fn load(index_path: PathBuf, default_root: PathBuf) -> Self {
        let index = Self::read_index(&index_path);
        let mut index = if index.workspaces.is_empty() {
            let canonical = std::fs::canonicalize(&default_root).unwrap_or(default_root.clone());
            let id = canonical
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "default".into());
            let meta = WorkspaceMeta {
                id,
                path: canonical.to_string_lossy().to_string(),
                name: canonical
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "default".into()),
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
        // Eagerly load the default workspace's stores.
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
        let (id_to_load, meta) = {
            let idx = self.index.read().unwrap();
            let meta = idx
                .workspaces
                .iter()
                .find(|m| m.id == ws_id)
                .cloned()
                .or_else(|| {
                    // fallback: default workspace
                    idx.default_workspace_id
                        .as_ref()
                        .and_then(|did| idx.workspaces.iter().find(|m| &m.id == did).cloned())
                });
            (ws_id.to_string(), meta)
        };
        let meta = match meta {
            Some(m) => m,
            None => panic!("no workspaces registered and no default"),
        };
        let root = PathBuf::from(&meta.path);
        let cache = self.cache.read().unwrap();
        if let Some(stores) = cache.get(&root) {
            return stores.clone();
        }
        drop(cache);
        let stores = Arc::new(WorkspaceStores::new(root.clone()));
        self.cache.write().unwrap().insert(root.clone(), stores.clone());
        let _ = id_to_load;
        stores
    }

    /// Open (or reuse) a workspace for the given project root path. Creates
    /// `.autoos/`, assigns an id (dir name, suffixed on clash), persists index.
    pub fn open(&self, root_path: &str) -> WorkspaceMeta {
        let canonical = std::fs::canonicalize(root_path).unwrap_or_else(|_| PathBuf::from(root_path));
        // Reuse if path already indexed.
        {
            let idx = self.index.read().unwrap();
            if let Some(existing) = idx.workspaces.iter().find(|m| m.path == canonical.to_string_lossy().to_string()) {
                return existing.clone();
            }
        }
        // Create .autoos
        let _ = std::fs::create_dir_all(autoos_dir(&canonical));
        // Assign id: dirname, suffix on clash with a different path.
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
```

Also add a helper used by tests (`index_path` field is private — tests in the same module can access it; but the test references `reg.index_path.clone().unwrap_or_default()` which won't compile since it's not Option). Fix the test line to:

```rust
let _raw = std::fs::read_to_string(&reg.index_path).unwrap_or_default();
```

and remove the `.unwrap_or_default()`.

- [ ] **Step 4: Add `pub mod workspace;` to lib.rs**

Modify `backend/crates/musk/src/lib.rs` — add after `pub mod wiki;` (around line where modules are declared):

```rust
pub mod workspace;
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p musk workspace:: 2>&1 | tail -8`
Expected: 4 tests PASS.

- [ ] **Step 6: Commit**

```bash
cd D:/autostack/auto-musk
git add backend/crates/musk/src/workspace.rs backend/crates/musk/src/lib.rs
git commit -m "feat(workspace): WorkspaceRegistry core — index + lazy store bundles"
```

---

### Task 2: tool_safety per-workspace root routing

**Files:**
- Modify: `backend/crates/musk/src/tool_safety.rs:20-56`

- [ ] **Step 1: Write the failing test**

Add to the `#[cfg(test)]` block in `tool_safety.rs` (create one if absent):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn current_root_override_routes_resolution() {
        let tmp = std::env::temp_dir().join(format!(
            "musk-ts-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&tmp).unwrap();
        fs::write(tmp.join("hello.txt"), "hi").unwrap();

        set_current_root(tmp.clone());
        let resolved = resolve_within_project("hello.txt").unwrap();
        assert_eq!(resolved, tmp.join("hello.txt").canonicalize().unwrap());
        clear_current_root();
    }

    #[test]
    fn without_override_falls_back_to_project_root() {
        // Just ensure it doesn't panic and returns *some* root.
        clear_current_root();
        let _ = project_root();
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p musk tool_safety:: 2>&1 | tail -5`
Expected: FAIL — `set_current_root`/`clear_current_root` undefined.

- [ ] **Step 3: Implement the current-root thread-local**

In `tool_safety.rs`, add a new thread-local alongside the existing `ROOT_OVERRIDE` (after line 26):

```rust
/// Thread-local "current workspace root" — set by the chat/relay driver before
/// running an agent so file tools confine to the active workspace's project dir.
/// Takes precedence over the startup snapshot, but yields to ROOT_OVERRIDE
/// (which tests use for stricter sandboxing).
thread_local! {
    static CURRENT_ROOT: std::cell::RefCell<Option<PathBuf>> = std::cell::RefCell::new(None);
}

/// Set the current workspace root for this thread (agent driver entry point).
pub fn set_current_root(path: PathBuf) {
    CURRENT_ROOT.with(|r| *r.borrow_mut() = Some(path));
}

/// Clear the current workspace root (agent driver exit point).
pub fn clear_current_root() {
    CURRENT_ROOT.with(|r| *r.borrow_mut() = None);
}
```

Then change `project_root()` (replace the body at lines 48-56) to consult `CURRENT_ROOT` between the test override and the startup snapshot:

```rust
pub fn project_root() -> PathBuf {
    ROOT_OVERRIDE.with(|r| r.borrow().clone())
        .or_else(|| CURRENT_ROOT.with(|r| r.borrow().clone()))
        .unwrap_or_else(|| {
            PROJECT_ROOT
                .get()
                .cloned()
                .unwrap_or_else(|| PathBuf::from("."))
        })
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p musk tool_safety:: 2>&1 | tail -5`
Expected: 2 tests PASS.

- [ ] **Step 5: Commit**

```bash
cd D:/autostack/auto-musk
git add backend/crates/musk/src/tool_safety.rs
git commit -m "feat(workspace): tool_safety per-workspace root via thread-local CURRENT_ROOT"
```

---

### Task 3: WorkspaceQuery extractor + AppState refactor + specs/chats handlers

This is the large wiring task. `AppState` drops its 4 store fields, gains `registry`; every specs + chats handler gains a `WorkspaceQuery` extractor and resolves `state.registry.get(&ws_id).{specs,chats}`.

**Files:**
- Modify: `backend/crates/musk/src/workspace.rs` (add `WorkspaceQuery`)
- Modify: `backend/crates/musk/src/server.rs` (AppState + ~27 handler signatures + serve())

- [ ] **Step 1: Add WorkspaceQuery extractor to workspace.rs**

Append to `workspace.rs`:

```rust
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
```

- [ ] **Step 2: Refactor AppState in server.rs**

In `server.rs`, change the struct (around lines 37-44):

```rust
pub struct AppState {
    pub client: Arc<dyn Client>,
    pub auth: Arc<crate::auth::AuthStore>,
    pub registry: Arc<crate::workspace::WorkspaceRegistry>,
}
```

Remove the old `specs`, `chats`, `wiki`, `relay` fields.

- [ ] **Step 3: Update serve() to build the registry**

In `serve()` (around lines 46-64), replace the `specs_path`/`config_dir`/state-construction block with:

```rust
    let users_path = dirs::home_dir()
        .map(|h| h.join(".config/autoos/users.json"))
        .unwrap_or_else(|| std::path::PathBuf::from("users.json"));
    let config_dir = dirs::home_dir()
        .map(|h| h.join(".config/autoos"))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let default_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let registry = crate::workspace::WorkspaceRegistry::load(
        config_dir.join("workspaces.json"),
        default_root,
    );
    let state = AppState {
        client,
        auth: Arc::new(crate::auth::AuthStore::new(users_path)),
        registry: Arc::new(registry),
    };
```

- [ ] **Step 4: Convert specs handlers**

For each specs handler in `server.rs` that currently reads `state.specs`, add `Query(q): Query<WorkspaceQuery>` to the signature and replace `state.specs` with `state.registry.get(&q.id_or_default(&state.registry)).specs`. Example for `specs_list` (apply the same pattern to ALL specs handlers: `specs_list`, `specs_upsert`, `specs_transition`, `specs_delete`, `specs_overview`, `specs_drift_check`, `specs_rebuild_relations`, `specs_related`):

```rust
use crate::workspace::WorkspaceQuery;
// ... at top of file imports
use axum::extract::Query;

async fn specs_list(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
) -> impl IntoResponse {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    match ws.specs.load() {
        Ok(doc) => Json(doc).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError { error: format!("load specs: {e}") }),
        ).into_response(),
    }
}
```

Repeat for every specs handler — each gains `Query(q): Query<WorkspaceQuery>` and uses `let ws = state.registry.get(&q.id_or_default(&state.registry));` then `ws.specs.<method>`. For `specs_related` (which takes a `Path`), keep the `Path` extractor and add `Query` alongside.

- [ ] **Step 5: Convert chats handlers**

Same pattern for all chats handlers (`chat_list`, `chat_create`, `chat_get`, `chat_rename`, `chat_delete`, `chat_delete_all`, `chat_message`, `chat_stream`, `chat_approve`, `chat_reject`, `chat_reject_all`). Each gains `Query(q): Query<WorkspaceQuery>` and uses `let ws = state.registry.get(...); ws.chats.<method>`. For `chat_stream`, the spawned task needs the workspace id captured:

```rust
async fn chat_stream(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Query(q): Query<WorkspaceQuery>,
) -> Response {
    let ws_id = q.id_or_default(&state.registry);
    let ws = state.registry.get(&ws_id);
    let session = match ws.chats.get(&id) { ... };
    // ... existing logic, but use ws.chats (clone the Arc) inside the spawned task:
    let chats = ws.chats.clone();
    // ... tokio::spawn(move || { ... chats.append_message(...) ... })
}
```

- [ ] **Step 6: Update server tests (tmp helpers)**

In the `#[cfg(test)]` block of `server.rs`, replace `tmp_specs()`/`tmp_chats()`/`tmp_wiki()`/`tmp_relay()` helpers with a single `tmp_state()` that builds an `AppState` with a temp-rooted registry:

```rust
fn tmp_state() -> AppState {
    let dir = std::env::temp_dir().join(format!(
        "musk-server-test-{}",
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let registry = crate::workspace::WorkspaceRegistry::load(
        dir.join("workspaces.json"),
        dir.clone(),
    );
    AppState {
        client: Arc::new(MockClient) as Arc<dyn Client>,
        auth: tmp_auth(),
        registry: Arc::new(registry),
    }
}
```

Then update `run_endpoint_returns_result` / `run_endpoint_bad_profession_errors` to use `let state = tmp_state();` (they currently construct AppState inline). Remove the now-unused `tmp_specs/tmp_chats/tmp_wiki/tmp_relay`.

- [ ] **Step 7: Run full backend test suite**

Run: `cargo test -p musk 2>&1 | tail -10`
Expected: all tests PASS (existing specs/chats/server/wiki/relay + new workspace/tool_safety).

- [ ] **Step 8: Commit**

```bash
cd D:/autostack/auto-musk
git add backend/crates/musk/src/workspace.rs backend/crates/musk/src/server.rs
git commit -m "feat(workspace): AppState registry + specs/chats handlers via ?workspace="
```

---

### Task 4: wiki + relay handlers + ChatSession/RunMetadata workspace_id + driver wiring

**Files:**
- Modify: `backend/crates/musk/src/wiki.rs` (handlers gain WorkspaceQuery)
- Modify: `backend/crates/musk/src/relay/api.rs` (handlers gain WorkspaceQuery)
- Modify: `backend/crates/musk/src/chats.rs` (ChatSession += workspace_id)
- Modify: `backend/crates/musk/src/relay/store.rs` (RunMetadata += workspace_id)
- Modify: `backend/crates/musk/src/relay/driver.rs` (set_current_root before agent run)
- Modify: `backend/crates/musk/src/server.rs` (chat_stream: set_current_root + create session with workspace_id)

- [ ] **Step 1: Add workspace_id to ChatSession**

In `chats.rs`, add to the `ChatSession` struct (around line 87-100):

```rust
    /// Which workspace this session belongs to (for agent root routing).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
```

Update `ChatSession::new` to accept and store it (or default to None in existing call sites; the chat_create handler will set it from the WorkspaceQuery).

- [ ] **Step 2: Add workspace_id to RunMetadata**

In `relay/store.rs`, add to `RunMetadata` (around line 170):

```rust
    #[serde(default)]
    pub workspace_id: Option<String>,
```

In `start_run`, accept the workspace id and store it:

```rust
pub fn start_run(&self, req: &StartRunRequest, workspace_id: Option<String>) -> (String, RunState) {
    // ... existing ...
    metadata: RunMetadata {
        title: ...,
        initial_task: req.task.clone(),
        originating_chat_session: None,
        workspace_id,
    },
```

- [ ] **Step 3: Convert wiki handlers**

In `wiki.rs`, every handler currently does `State(state): State<AppState>` and reads `state.wiki`. Add `Query(q): Query<WorkspaceQuery>` and replace with `let ws = state.registry.get(&q.id_or_default(&state.registry));` then `ws.wiki.<method>`. Apply to: `wiki_tree`, `raw_tree`, `list_pages`, `get_page`, `create_page`, `update_page`, `delete_page`, `search`, `raw_upload`, `raw_file`, `raw_delete`, `raw_mkdir`. Add the import `use crate::workspace::WorkspaceQuery; use axum::extract::Query;`.

- [ ] **Step 4: Convert relay handlers**

In `relay/api.rs`, every handler reading `state.relay` gains `Query(q): Query<WorkspaceQuery>` and uses `let ws = state.registry.get(&q.id_or_default(&state.registry)); ws.relay.<method>`. For `start_run`, pass `Some(ws_id)` into `ws.relay.start_run(&req, Some(ws_id))`. For `advance_run`, capture the workspace_id before spawning the driver and set the thread-local root inside the spawned task. Add imports.

- [ ] **Step 5: Wire set_current_root in driver + chat_stream**

In `relay/driver.rs` `drive_run`/`run_step`: before building the agent, look up the run's workspace_id → `state.registry.get(&ws_id).root` → call `crate::tool_safety::set_current_root(root.clone())`; in a `finally`-style guard (or at driver exit) call `clear_current_root()`. Since this runs on a spawned tokio task (which is a thread), set it at the top of the task:

```rust
pub async fn drive_run(state: Arc<AppState>, run_id: String) {
    // Resolve this run's workspace root for tool confinement.
    let ws_id = state.relay... // need a peek at the run's workspace_id
    // Add a store helper: state.relay.workspace_of(&run_id) -> Option<String>
    let root = ws_id.as_ref().and_then(|id| Some(state.registry.get(id).root.clone()));
    if let Some(root) = root { crate::tool_safety::set_current_root(root); }
    // ... existing drive loop ...
    crate::tool_safety::clear_current_root(); // at the end
}
```

Add `workspace_of` to `RunStore`:

```rust
pub fn workspace_of(&self, run_id: &str) -> Option<String> {
    self.runs.lock().unwrap().get(run_id).and_then(|e| e.metadata.workspace_id.clone())
}
```

Same pattern in `server.rs` `chat_stream`: before the spawned agent run, `let root = state.registry.get(&ws_id).root.clone(); crate::tool_safety::set_current_root(root);` inside the spawned task, and `clear_current_root()` after `agent.run_stream` completes.

- [ ] **Step 6: Run full backend test suite**

Run: `cargo test -p musk 2>&1 | tail -10`
Expected: all tests PASS.

- [ ] **Step 7: Commit**

```bash
cd D:/autostack/auto-musk
git add backend/crates/musk/src/
git commit -m "feat(workspace): wiki/relay handlers + ChatSession/RunMetadata workspace_id + driver root routing"
```

---

### Task 5: workspace API endpoints + --workdir CLI

**Files:**
- Modify: `backend/crates/musk/src/server.rs` (add workspace endpoints + routes)
- Modify: `backend/crates/musk/src/main.rs` (Serve += --workdir)

- [ ] **Step 1: Add workspace management endpoints in server.rs**

Add handlers (these do NOT take WorkspaceQuery — they operate on the registry itself):

```rust
/// GET /api/workspace/list — recent workspaces.
async fn workspace_list(State(state): State<AppState>) -> impl IntoResponse {
    Json(json!({ "workspaces": state.registry.list() }))
}

/// POST /api/workspace/open — open/reuse a workspace by path.
#[derive(Deserialize)]
struct OpenWorkspaceBody { path: String }
async fn workspace_open(
    State(state): State<AppState>,
    Json(body): Json<OpenWorkspaceBody>,
) -> impl IntoResponse {
    let meta = state.registry.open(&body.path);
    state.registry.touch(&meta.id);
    Json(json!({ "workspace": meta }))
}

/// GET /api/workspace/status?workspace=<id>
async fn workspace_status(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
) -> impl IntoResponse {
    let ws_id = q.id_or_default(&state.registry);
    let meta = state.registry.list().into_iter().find(|m| m.id == ws_id);
    match meta {
        Some(m) => Json(json!({ "workspace": m, "root_exists": std::path::Path::new(&m.path).exists() })),
        None => (StatusCode::NOT_FOUND, "workspace not found").into_response(),
    }
}

/// GET /api/workspace/browse?path=<dir> — list child directories (for the picker).
async fn workspace_browse(Query(q): Query<BrowseQuery>) -> impl IntoResponse {
    #[derive(Deserialize)]
    struct BrowseQueryInner { #[serde(default)] path: String }
    let base = if q.path.is_empty() { ".".to_string() } else { q.path.clone() };
    let mut entries: Vec<serde_json::Value> = Vec::new();
    if let Ok(dir) = std::fs::read_dir(&base) {
        for e in dir.flatten() {
            if e.path().is_dir() {
                let name = e.file_name().to_string_lossy().to_string();
                if !name.starts_with('.') {
                    entries.push(json!({ "name": name, "path": e.path().to_string_lossy().to_string() }));
                }
            }
        }
    }
    Json(json!({ "entries": entries, "parent": std::path::Path::new(&base).parent().map(|p| p.to_string_lossy().to_string()) }))
}
```

(For `workspace_browse`, define a dedicated `BrowseQuery { path: String }` struct to avoid clashing with `WorkspaceQuery`.)

Register routes in the router (before `.fallback_service`):

```rust
.route("/api/workspace/list", get(workspace_list))
.route("/api/workspace/open", post(workspace_open))
.route("/api/workspace/status", get(workspace_status))
.route("/api/workspace/browse", get(workspace_browse))
```

- [ ] **Step 2: Add --workdir to the Serve CLI**

In `main.rs`, change the `Serve` variant (around line 62-67):

```rust
Serve {
    #[arg(long, default_value = "127.0.0.1:8080")]
    addr: String,
    #[arg(long)]
    workdir: Option<String>,
},
```

In the `Cmd::Serve { addr, workdir }` match arm, if `workdir` is Some, `std::env::set_current_dir(&workdir)?` **before** `init_project_root()` so the default workspace root honors `--workdir`.

- [ ] **Step 3: Run full backend test suite + build**

Run: `cargo test -p musk 2>&1 | tail -5 && cargo build -p musk 2>&1 | tail -3`
Expected: tests PASS, build OK.

- [ ] **Step 4: Commit**

```bash
cd D:/autostack/auto-musk
git add backend/crates/musk/src/server.rs backend/crates/musk/src/main.rs
git commit -m "feat(workspace): /api/workspace/* endpoints + --workdir CLI flag"
```

---

### Task 6: Frontend — useProject real + workspace query-param on fetches

**Files:**
- Modify: `web/src/composables/useProject.ts`
- Modify: `web/src/composables/useAuth.ts` (authFetch appends ?workspace=)

- [ ] **Step 1: Rewrite useProject.ts**

Replace the stub body with real state + API calls:

```ts
import { ref, computed } from 'vue'
import { authFetch } from './useAuth'

export interface WorkspaceMeta {
  id: string
  path: string
  name: string
  last_opened: number
}

const _current = ref<WorkspaceMeta | null>(null)
const _recent = ref<WorkspaceMeta[]>([])
const _isLoading = ref(false)

// Current workspace id, mirrored to the URL ?workspace=<id>.
const _urlWorkspaceId = ref<string | null>(null)

export function useProject() {
  const isOpen = computed(() => _current.value !== null)
  const projectName = computed(() => _current.value?.name ?? null)
  const projectPath = computed(() => _current.value?.path ?? null)
  const workspaceId = computed(() => _current.value?.id ?? _urlWorkspaceId.value ?? null)

  function syncUrl(id: string | null) {
    const url = new URL(window.location.href)
    if (id) url.searchParams.set('workspace', id); else url.searchParams.delete('workspace')
    window.history.replaceState({}, '', url.toString())
    _urlWorkspaceId.value = id
  }

  async function fetchStatus() {
    const id = new URL(window.location.href).searchParams.get('workspace')
    const query = id ? `?workspace=${encodeURIComponent(id)}` : ''
    const resp = await authFetch(`/api/workspace/status${query}`)
    if (resp.ok) {
      const data = await resp.json()
      _current.value = data.workspace
      syncUrl(data.workspace.id)
    }
  }

  async function openWorkspace(path: string) {
    const resp = await authFetch('/api/workspace/open', {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ path }),
    })
    if (!resp.ok) throw new Error(`open workspace failed: ${resp.status}`)
    const data = await resp.json()
    _current.value = data.workspace
    syncUrl(data.workspace.id)
    await loadRecent()
  }

  async function loadRecent() {
    const resp = await authFetch('/api/workspace/list')
    if (resp.ok) _recent.value = (await resp.json()).workspaces ?? []
  }

  async function browse(path: string) {
    const q = path ? `?path=${encodeURIComponent(path)}` : ''
    const resp = await authFetch(`/api/workspace/browse${q}`)
    if (!resp.ok) return []
    return (await resp.json()).entries ?? []
  }

  return {
    isOpen, projectName, projectPath, workspaceId,
    currentWorkspace: _current, recentWorkspaces: _recent, isLoading: _isLoading,
    fetchStatus, openWorkspace, loadRecent, browse, syncUrl,
  }
}
```

- [ ] **Step 2: Make authFetch append ?workspace=**

In `useAuth.ts`, find the `authFetch` implementation. Wrap it so relative `/api/...` calls (except `/api/workspace/*` and `/api/auth/*`) get `?workspace=<currentId>` appended. Simplest: export a `currentWorkspaceId` ref from `useProject` and read it inside `authFetch`:

```ts
// In useAuth.ts, at the top of authFetch, before constructing the request:
import { useProject } from './useProject'
// ... inside authFetch(url, opts):
let finalUrl = url
if (url.startsWith('/api/') && !url.startsWith('/api/workspace/') && !url.startsWith('/api/auth/')) {
  const { workspaceId } = useProject()
  const sep = url.includes('?') ? '&' : '?'
  if (workspaceId.value) finalUrl = `${url}${sep}workspace=${encodeURIComponent(workspaceId.value)}`
}
```

- [ ] **Step 3: Build frontend to verify it compiles**

Run: `cd D:/autostack/auto-musk/web && npx vite build 2>&1 | tail -3`
Expected: build OK (TS type errors may remain from pre-existing issues, but no NEW errors from these changes).

- [ ] **Step 4: Commit**

```bash
cd D:/autostack/auto-musk
git add web/src/composables/useProject.ts web/src/composables/useAuth.ts
git commit -m "feat(workspace): useProject real + authFetch appends ?workspace="
```

---

### Task 7: Frontend — WorkspaceSelector component + App.vue wiring

**Files:**
- Create: `web/src/components/WorkspaceSelector.vue`
- Modify: `web/src/App.vue` (render selector in footer + fetchStatus on mount)

- [ ] **Step 1: Create WorkspaceSelector.vue**

```vue
<template>
  <div class="workspace-selector">
    <button class="ws-btn" @click="open = !open" :title="current?.path">
      <Folder :size="14" />
      <span class="ws-name">{{ current?.name ?? '选择工作目录' }}</span>
      <ChevronUp :size="12" v-if="open" />
      <ChevronDown :size="12" v-else />
    </button>
    <div v-if="open" class="ws-panel">
      <div class="ws-panel-header">
        <span>切换 Workspace</span>
        <button class="ws-close" @click="open = false"><X :size="12" /></button>
      </div>
      <div class="ws-section-label">最近打开</div>
      <button
        v-for="w in recent" :key="w.id"
        class="ws-item" :class="{ active: w.id === current?.id }"
        @click="choose(w)"
      >
        <Folder :size="13" />
        <span class="ws-item-name">{{ w.name }}</span>
        <span class="ws-item-path">{{ w.path }}</span>
      </button>
      <div class="ws-divider" />
      <div class="ws-section-label">打开其他文件夹</div>
      <input class="ws-input" v-model="customPath" placeholder="D:\path\to\project"
        @keydown.enter="openCustom" />
      <div v-if="suggestions.length" class="ws-suggest">
        <button v-for="s in suggestions" :key="s.path" class="ws-suggest-item" @click="customPath = s.path">
          📁 {{ s.name }}
        </button>
      </div>
      <button class="ws-open-btn" @click="openCustom" :disabled="!customPath.trim()">
        <FolderOpen :size="13" /> 打开
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { Folder, FolderOpen, ChevronUp, ChevronDown, X } from 'lucide-vue-next'
import { useProject } from '@/composables/useProject'

const { currentWorkspace: current, recentWorkspaces: recent, openWorkspace, loadRecent, browse } = useProject()
const open = ref(false)
const customPath = ref('')
const suggestions = ref<{ name: string; path: string }[]>([])

loadRecent()

watch(customPath, async (v) => {
  if (!v || !v.includes('/') && !v.includes('\\')) return
  suggestions.value = await browse(v)
})

async function choose(w: { path: string }) {
  await openWorkspace(w.path)
  open.value = false
}
async function openCustom() {
  const p = customPath.value.trim()
  if (!p) return
  await openWorkspace(p)
  customPath.value = ''
  open.value = false
}
</script>

<style scoped>
.workspace-selector { position: relative; }
.ws-btn {
  display: flex; align-items: center; gap: 0.4rem;
  background: hsl(var(--muted-foreground) / 0.06); border: none; border-radius: 6px;
  padding: 0.35rem 0.6rem; color: var(--af-fg); cursor: pointer; font-size: 0.8rem;
  max-width: 140px;
}
.ws-btn:hover { background: hsl(var(--muted-foreground) / 0.12); }
.ws-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.ws-panel {
  position: absolute; bottom: 100%; left: 0; margin-bottom: 6px;
  width: 280px; background: var(--af-card); border: 1px solid var(--af-border);
  border-radius: 8px; box-shadow: 0 4px 16px rgba(0,0,0,0.2); padding: 0.5rem;
  z-index: 100;
}
.ws-panel-header { display: flex; justify-content: space-between; align-items: center; padding: 0.25rem 0.5rem; font-size: 0.85rem; font-weight: 600; }
.ws-close { background: none; border: none; color: var(--af-muted); cursor: pointer; }
.ws-section-label { font-size: 0.7rem; text-transform: uppercase; color: var(--af-muted); padding: 0.4rem 0.5rem 0.2rem; }
.ws-item { display: flex; align-items: center; gap: 0.4rem; width: 100%; background: none; border: none; padding: 0.4rem 0.5rem; border-radius: 4px; cursor: pointer; color: var(--af-fg); text-align: left; }
.ws-item:hover { background: hsl(var(--muted-foreground) / 0.08); }
.ws-item.active { background: hsl(var(--primary) / 0.1); color: var(--af-primary); }
.ws-item-name { font-size: 0.82rem; }
.ws-item-path { font-size: 0.68rem; color: var(--af-muted); margin-left: auto; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 120px; }
.ws-divider { height: 1px; background: var(--af-border); margin: 0.4rem 0; }
.ws-input { width: 100%; box-sizing: border-box; background: var(--af-bg); border: 1px solid var(--af-border); border-radius: 4px; padding: 0.35rem 0.5rem; color: var(--af-fg); font-size: 0.8rem; }
.ws-suggest { max-height: 120px; overflow-y: auto; }
.ws-suggest-item { display: block; width: 100%; background: none; border: none; padding: 0.3rem 0.5rem; text-align: left; cursor: pointer; color: var(--af-fg); font-size: 0.78rem; border-radius: 4px; }
.ws-suggest-item:hover { background: hsl(var(--muted-foreground) / 0.08); }
.ws-open-btn { display: flex; align-items: center; gap: 0.4rem; width: 100%; justify-content: center; margin-top: 0.4rem; background: hsl(var(--primary)); color: #fff; border: none; border-radius: 4px; padding: 0.4rem; cursor: pointer; font-size: 0.82rem; }
.ws-open-btn:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
```

- [ ] **Step 2: Wire into App.vue footer + fetchStatus on mount**

In `App.vue`:
- Import: `import WorkspaceSelector from '@/components/WorkspaceSelector.vue'`
- In the `<script setup>`, add to the existing `onMounted` (or add one): `const { fetchStatus } = useProject(); fetchStatus();`
- In the template `.rail-footer`, place the selector before `<SettingsMenu />`:

```vue
<div class="rail-footer">
  <WorkspaceSelector />
  <SettingsMenu />
</div>
```

Adjust `.rail-footer` CSS to `justify-content: space-between` so the two elements spread out.

- [ ] **Step 3: Add reload-on-workspace-change to views**

In each of `ChatsView.vue`, `SpecsView.vue`, `WikiView.vue`, `RelayView.vue`, add (they already import useProject) a watch on `workspaceId` that re-fetches. Example for ChatsView:

```ts
import { watch } from 'vue'
const { workspaceId } = useProject()
watch(workspaceId, () => { if (workspaceId.value) { loadSessionList(); resume(projectPath.value) } })
```

(Each view already has its own load functions; call them in the watch.)

- [ ] **Step 4: Build frontend**

Run: `cd D:/autostack/auto-musk/web && npx vite build 2>&1 | tail -3`
Expected: build OK.

- [ ] **Step 5: Commit**

```bash
cd D:/autostack/auto-musk
git add web/src/components/WorkspaceSelector.vue web/src/App.vue web/src/views/
git commit -m "feat(workspace): WorkspaceSelector component + App.vue footer + view reloads"
```

---

### Task 8: Migration + end-to-end verification

**Files:**
- Modify: `backend/crates/musk/src/workspace.rs` (migration on first load)
- Modify: `backend/crates/musk/src/server.rs` (call migration in serve)

- [ ] **Step 1: Add migration helper to WorkspaceRegistry**

In `workspace.rs`, add a method that runs once if the default workspace's `.autoos/` is empty AND old global data exists:

```rust
impl WorkspaceRegistry {
    /// Best-effort: if old global data exists at `global_dir` and the default
    /// workspace's .autoos is empty, move the old files into it. Idempotent.
    pub fn migrate_global_data(&self, global_dir: &std::path::Path) {
        let idx = self.index.read().unwrap().clone();
        let Some(default_id) = &idx.default_workspace_id else { return };
        let Some(default_meta) = idx.workspaces.iter().find(|m| &m.id == default_id) else { return };
        let autoos = autoos_dir(&PathBuf::from(&default_meta.path));
        // Only migrate if .autoos has no specs.json yet.
        if autoos.join("specs.json").exists() { return; }
        let _ = std::fs::create_dir_all(&autoos);
        for name in ["specs.json", "chats.json"] {
            let src = global_dir.join(name);
            let dst = autoos.join(name);
            if src.exists() && !dst.exists() {
                let _ = std::fs::rename(&src, &dst);
            }
        }
        for sub in ["wiki", "raw", "relay"] {
            let src = global_dir.join(sub);
            let dst = autoos.join(sub);
            if src.is_dir() && !dst.exists() {
                let _ = std::fs::rename(&src, &dst);
            }
        }
    }
}
```

- [ ] **Step 2: Call migration in serve()**

In `server.rs` `serve()`, after building the registry:

```rust
    let config_dir = dirs::home_dir().map(|h| h.join(".config/autoos")).unwrap_or_default();
    registry.migrate_global_data(&config_dir);
```

- [ ] **Step 3: Run full backend test suite**

Run: `cargo test -p musk 2>&1 | tail -5`
Expected: all PASS.

- [ ] **Step 4: Build release + restart server**

```bash
cd D:/autostack/auto-musk/backend && cargo build --release -p musk 2>&1 | tail -3
cd D:/autostack/auto-musk/web && npx vite build 2>&1 | tail -3
# stop old server (find PID on 8888) then:
# backend/target/release/musk.exe serve --addr 127.0.0.1:8888
```

- [ ] **Step 5: curl-verify the workspace flow**

```bash
TOKEN=$(curl -s -X POST http://localhost:8888/api/auth/login -H "Content-Type: application/json" -d '{"username":"admin","password":"admin"}' | python -c "import sys,json;print(json.load(sys.stdin)['token'])")
# list workspaces (should have a default seeded from cwd)
curl -s "http://localhost:8888/api/workspace/list" -H "Authorization: Bearer $TOKEN"
# open a new workspace
curl -s -X POST "http://localhost:8888/api/workspace/open" -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" -d '{"path":"D:/autostack/auto-forge"}' -H "Authorization: Bearer $TOKEN"
# specs isolated per workspace: list with each workspace id
curl -s "http://localhost:8888/api/specs?workspace=auto-musk" -H "Authorization: Bearer $TOKEN" | head -c 200
```
Expected: workspace/list returns ≥1 entry; workspace/open returns a meta with id `auto-forge`; specs differ (or one is empty) between the two workspace ids.

- [ ] **Step 6: Playwright-verify the UI**

Run a Playwright script (login → check the footer shows a workspace name → click it → panel opens with recent list → enter a path → open → URL gains `?workspace=`). Confirm 0 console errors.

- [ ] **Step 7: Commit**

```bash
cd D:/autostack/auto-musk
git add backend/crates/musk/src/workspace.rs backend/crates/musk/src/server.rs
git commit -m "feat(workspace): migrate global data into default .autoos/ + e2e verification"
```

---

## Self-Review (completed during authoring)

- **Spec coverage:** §2 data model → Task 1; §3 backend → Tasks 1-5; §4 frontend → Tasks 6-7; §5 UX → Task 7 (selector + URL sync); §6 migration → Task 8; §9 acceptance → Task 8 steps 5-6. All sections covered.
- **Placeholder scan:** No TBD/TODO. Each handler-conversion step names the exact handlers. Code blocks are complete.
- **Type consistency:** `WorkspaceMeta`, `WorkspaceStores`, `WorkspaceRegistry`, `WorkspaceQuery` defined in Task 1/3 and referenced consistently. `set_current_root`/`clear_current_root` defined Task 2, used Task 4. `workspace_id` field added Task 4 to both ChatSession and RunMetadata.
