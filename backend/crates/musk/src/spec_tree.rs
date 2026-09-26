//! Spec module-tree browser (PLAN-025; PLAN-091 T-01/T-02).
//!
//! hw escape-hatch routes mirroring `plans.rs` (PLAN-024): serves the
//! `docs/specs/` file-tree knowledge layer (design 008 §5) by deriving paths
//! from the workspace root — **no store added** to `workspace.rs`.
//!
//! Three endpoints:
//!   GET /api/specs/tree          → `Vec<FilesNode>` over `docs/specs/`
//!   GET /api/specs/text/{*path}  → `{ content, error }` JSON (PLAN-091 T-02)
//!   GET /api/specs/file/{*path}  → raw file body (path-traversal guarded;
//!                                  kept for direct/browser links)
//!
//! PLAN-091 T-01: the tree payload switched from wiki `TreeNode`
//! (name/path/type, leaf `children` omitted) to the fs-shaped `FilesNode`
//! the FileTree widget consumes (`id`/`label`/`children` always present) —
//! the same schema contract as `files_browser`, whose doc comment records
//! the mismatch defect this fixes ("TypeError: reading 'length'" on leaf
//! nodes). Ordering keeps the wiki semantics: folders first, then
//! alphabetical; dotfiles + manifests skipped. Unlike the wiki tree, spec
//! files keep their real names (`.md` is **not** stripped) — this is a
//! knowledge layer browsed by filename, not a slug store.

use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use std::path::PathBuf;

use crate::files_browser::{read_text_confined, FilesNode, FilesTextResponse};
use crate::server::AppState;
use crate::wiki::{guess_mime, validate_path_pub};
use crate::workspace::WorkspaceQuery;

/// Flatten `WorkspaceQuery` so `?workspace=<id>` works the same way as every
/// other route (see `plans::PlansQuery`).
#[derive(Deserialize)]
pub struct SpecTreeQuery {
    #[serde(flatten)]
    pub workspace: WorkspaceQuery,
}

/// Resolve the `docs/specs/` directory for the requested workspace.
fn specs_dir_for(state: &AppState, q: &SpecTreeQuery) -> PathBuf {
    let ws = state.registry.get(&q.workspace.id_or_default(&state.registry));
    ws.root.join("docs").join("specs")
}

pub fn spec_tree_routes() -> Router<AppState> {
    Router::new()
        .route("/api/specs/tree", get(spec_tree))
        .route("/api/specs/text/{*path}", get(spec_text))
        .route("/api/specs/file/{*path}", get(spec_file))
}

/// Spec-tree build: wiki `build_tree` ordering semantics (folders first +
/// alphabetical, dotfiles/manifests skipped) emitting fs-shaped `FilesNode`s.
fn build_spec_tree(root: &std::path::Path, prefix: &str) -> Vec<FilesNode> {
    let mut entries: Vec<FilesNode> = Vec::new();
    let Ok(dir) = std::fs::read_dir(root) else {
        return entries;
    };
    let mut dir_entries: Vec<_> = dir.flatten().collect();
    // Folders first, then alphabetical — same ordering contract as the wiki
    // tree and the workspace files tree so all three browsers feel identical.
    dir_entries.sort_by(|a, b| {
        let a_is_dir = a.path().is_dir();
        let b_is_dir = b.path().is_dir();
        b_is_dir
            .cmp(&a_is_dir)
            .then(a.file_name().to_string_lossy().cmp(&b.file_name().to_string_lossy()))
    });
    for entry in &dir_entries {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name == "_manifest.json" || name == "manifest.json" {
            continue;
        }
        let id = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", prefix, name)
        };
        if entry.path().is_dir() {
            let children = build_spec_tree(&entry.path(), &id);
            entries.push(FilesNode {
                label: name,
                id,
                children,
                kind: "dir".into(),
                icon: String::new(),
                is_leaf: false,
                badge: String::new(),
            });
        } else {
            entries.push(FilesNode {
                label: name,
                id,
                children: Vec::new(),
                kind: "file".into(),
                icon: String::new(),
                is_leaf: true,
                badge: String::new(),
            });
        }
    }
    entries
}

/// GET /api/specs/tree — nested file/folder tree over `docs/specs/`.
///
/// A fresh workspace without `docs/specs/` simply yields an empty tree
/// (`read_dir` fails → `[]`) — no 404, the frontend renders an empty browser.
async fn spec_tree(
    State(state): State<AppState>,
    Query(q): Query<SpecTreeQuery>,
) -> Json<Vec<FilesNode>> {
    let dir = specs_dir_for(&state, &q);
    Json(build_spec_tree(&dir, ""))
}

/// GET /api/specs/text/{*path} — spec file body as `{ content, error }` JSON
/// (PLAN-091 T-02, mirrors `files_text`): the generated `#[api]` bindings fix
/// `response.json()`, so raw text bodies cannot be consumed by the frontend.
/// Errors stay non-2xx (the VM `get_json` wrapper reshapes them to
/// `{error, status}`; the web generated binding throws into the catch arm) —
/// on success `error` is always "" (honest contract).
async fn spec_text(
    State(state): State<AppState>,
    Query(q): Query<SpecTreeQuery>,
    Path(path): Path<String>,
) -> Result<Json<FilesTextResponse>, (StatusCode, String)> {
    let dir = specs_dir_for(&state, &q);
    let content = read_text_confined(&dir, &path)?;
    Ok(Json(FilesTextResponse {
        content,
        error: String::new(),
    }))
}

/// GET /api/specs/file/{*path} — read a file under `docs/specs/`.
///
/// Rejects path traversal (`..`, leading `/` or `\`) via `validate_path_pub`,
/// then streams the raw body with a MIME type from `guess_mime`
/// (`.md` → `text/markdown`).
async fn spec_file(
    State(state): State<AppState>,
    Query(q): Query<SpecTreeQuery>,
    Path(path): Path<String>,
) -> Result<Response, StatusCode> {
    validate_path_pub(&path).map_err(|_| StatusCode::BAD_REQUEST)?;
    let dir = specs_dir_for(&state, &q);
    let file_path = dir.join(&path);
    let data = std::fs::read(&file_path).map_err(|_| StatusCode::NOT_FOUND)?;
    let mime = guess_mime(&file_path);
    Ok(([(header::CONTENT_TYPE, mime)], data).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_dir(root: &std::path::Path, rel: &str) -> std::path::PathBuf {
        let p = root.join(rel);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    fn write(root: &std::path::Path, rel: &str, content: &str) {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, content).unwrap();
    }

    /// Tree build: fs-shaped nodes (`id`/`label`/`children` always present);
    /// folders sort before files, alphabetical within each group; dotfiles +
    /// manifests are skipped; ids are paths relative to `docs/specs/`.
    #[test]
    fn build_spec_tree_folders_first_and_nests() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // files at top level
        write(root, "00-overview.md", "# o");
        write(root, "01-architecture.md", "# a");
        write(root, ".hidden.md", "skip");
        write(root, "_manifest.json", "{}");
        // a folder with one child
        write(root, "goals/README.md", "# g");
        write(root, "goals/z-last.md", "# z");

        let tree = build_spec_tree(root, "");

        // top-level order: folder(s) first then files alphabetically;
        // dotfile + manifest dropped.
        assert_eq!(tree.len(), 3, "got {tree:?}");
        assert_eq!(tree[0].kind, "dir");
        assert_eq!(tree[0].label, "goals");
        assert_eq!(tree[0].id, "goals");
        assert!(!tree[0].is_leaf);
        // `children` is always present (FileTree contract: leaf nodes carry
        // an empty array, never a missing key).
        assert_eq!(tree[0].children.len(), 2, "goals children: {:?}", tree[0].children);
        // `.md` is NOT stripped (spec files keep real names); child ids nest.
        let goals_children = &tree[0].children;
        assert!(
            goals_children
                .iter()
                .any(|n| n.label == "README.md" && n.id == "goals/README.md"),
            "expected goals/README.md in {goals_children:?}"
        );
        for child in goals_children {
            assert_eq!(child.kind, "file");
            assert!(child.is_leaf);
            assert!(child.children.is_empty(), "leaf children must be []");
        }

        assert_eq!(tree[1].kind, "file");
        assert_eq!(tree[1].label, "00-overview.md");
        assert_eq!(tree[1].id, "00-overview.md");
        assert_eq!(tree[2].label, "01-architecture.md");

        // Missing root → empty tree (fresh workspace contract), no panic.
        assert!(build_spec_tree(&make_dir(tmp.path(), "nope-nope"), "").is_empty());
    }

    /// `validate_path_pub` rejects traversal and absolute paths, accepts normal.
    #[test]
    fn validate_path_rejects_traversal() {
        assert!(validate_path_pub("../etc/passwd").is_err());
        assert!(validate_path_pub("a/../../b").is_err());
        assert!(validate_path_pub("/etc/passwd").is_err());
        assert!(validate_path_pub("\\windows\\system32").is_err());
        assert!(validate_path_pub("goals/README.md").is_ok());
        assert!(validate_path_pub("00-overview.md").is_ok());
    }

    /// File read via the same join + guess_mime logic the raw handler uses:
    /// valid path returns bytes; missing file would error.
    #[test]
    fn file_read_returns_body() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(root, "goals/README.md", "# goals index");

        let path = "goals/README.md";
        validate_path_pub(path).unwrap();
        let body = std::fs::read(root.join(path)).unwrap();
        assert_eq!(body, b"# goals index");
        assert_eq!(guess_mime(std::path::Path::new(path)), "text/markdown");
    }

    /// PLAN-091 T-02: `read_text_confined` (shared with files_text) powers
    /// the specs text channel — success returns the body, missing files and
    /// traversal are typed errors, directory reads are rejected.
    #[test]
    fn spec_text_confined_read_contract() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(root, "modules/plan-flow.md", "# plan flow\n\nbody");
        make_dir(root, "modules/nested");

        // Happy path: full body, in-band error "".
        let content = read_text_confined(root, "modules/plan-flow.md").unwrap();
        assert_eq!(content, "# plan flow\n\nbody");

        // Missing file → typed error (NOT_FOUND arm).
        assert!(read_text_confined(root, "modules/nope.md").is_err());

        // Traversal → typed error (BAD_REQUEST arm).
        assert!(read_text_confined(root, "../outside.md").is_err());

        // Directory read → rejected.
        assert!(read_text_confined(root, "modules/nested").is_err());
    }

    /// PLAN-091 T-02: the JSON wrapper shape the generated bindings consume
    /// (`response.json()` → `{content, error}`) — same honest contract as
    /// files_text (error "" on success).
    #[test]
    fn spec_text_response_shape_wraps_content() {
        let resp = FilesTextResponse {
            content: "# specs\n".to_string(),
            error: String::new(),
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json, serde_json::json!({ "content": "# specs\n", "error": "" }));
    }
}
