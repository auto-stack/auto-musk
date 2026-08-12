//! Spec module-tree browser (PLAN-025).
//!
//! hw escape-hatch routes mirroring `plans.rs` (PLAN-024): serves the
//! `docs/specs/` file-tree knowledge layer (design 008 §5) by deriving paths
//! from the workspace root — **no store added** to `workspace.rs`. Reuses
//! `wiki::build_tree` (folders-first + alphabetical sort) and
//! `validate_path_pub` + `guess_mime` so the tree/path/mime semantics stay
//! identical to the wiki knowledge base.
//!
//! Two endpoints:
//!   GET /api/specs/tree          → `Vec<TreeNode>` over `docs/specs/`
//!   GET /api/specs/file/{*path}  → file body (path-traversal guarded)
//!
//! Unlike the wiki tree, spec files keep their real names (`.md` is **not**
//! stripped) — this is a knowledge layer browsed by filename, not a slug store.

use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use std::path::PathBuf;

use crate::server::AppState;
use crate::wiki::{build_tree, guess_mime, validate_path_pub, TreeNode};
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
        .route("/api/specs/file/{*path}", get(spec_file))
}

/// GET /api/specs/tree — nested file/folder tree over `docs/specs/`.
///
/// A fresh workspace without `docs/specs/` simply yields an empty tree
/// (`build_tree` returns `[]` when `read_dir` fails) — no 404, the frontend
/// renders an empty browser.
async fn spec_tree(
    State(state): State<AppState>,
    Query(q): Query<SpecTreeQuery>,
) -> Json<Vec<TreeNode>> {
    let dir = specs_dir_for(&state, &q);
    Json(build_tree(&dir, ""))
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

    /// Tree build: folders sort before files; entries are alphabetical within
    /// each group; dotfiles + manifest are skipped; children nest.
    #[test]
    fn build_tree_folders_first_and_nests() {
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

        let tree = build_tree(root, "");

        // top-level order: folder(s) first then files alphabetically;
        // dotfile + manifest dropped.
        assert_eq!(tree.len(), 3, "got {tree:?}");
        assert_eq!(tree[0].node_type, "folder");
        assert_eq!(tree[0].name, "goals");
        assert_eq!(
            tree[0].children.as_ref().unwrap().len(),
            2,
            "goals children: {:?}",
            tree[0].children
        );
        // `.md` is NOT stripped (spec files keep real names).
        let goals_children = tree[0].children.as_ref().unwrap();
        assert!(
            goals_children.iter().any(|n| n.name == "README.md"),
            "expected README.md in {goals_children:?}"
        );

        assert_eq!(tree[1].node_type, "file");
        assert_eq!(tree[1].name, "00-overview.md");
        assert_eq!(tree[2].name, "01-architecture.md");
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

    /// File read via the same join + guess_mime logic the handler uses:
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
}
