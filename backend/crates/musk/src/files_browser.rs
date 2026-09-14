//! Workspace file browser (PLAN-068).
//!
//! hw escape-hatch routes mirroring `spec_tree.rs` (PLAN-025): serves the
//! **whole workspace root** instead of `docs/specs/` only, generalized from
//! the same `wiki` primitives (`TreeNode`, path validation, MIME guessing).
//!
//! Two endpoints:
//!   GET /api/files/tree          → `{ tree: Vec<TreeNode>, truncated }`
//!   GET /api/files/raw/{*path}   → raw bytes + MIME (confinement-guarded)
//!
//! Differences vs the specs tree (all server-enforced; the frontend is not a
//! security boundary):
//!   * directory ignore list — heavy build/vendor dirs never reach the tree
//!     (dot-entries are already dropped by the shared dotfile rule);
//!   * node/depth budgets — oversized workspaces answer a `truncated` flag
//!     instead of hanging the response;
//!   * canonicalize prefix double-check — a symlink pointing outside the
//!     workspace root is rejected (the name-based checks alone don't cover
//!     links);
//!   * 20 MB read cap — larger files answer 413, the frontend shows a
//!     "file too large" empty state.
//!
//! MIME note: `.html` is deliberately served as `text/plain` — a read-only
//! browser must not hand workspace files script execution on the app origin.
//! `.svg` keeps `image/svg+xml` (safe inside `<img>`, same surface as the
//! existing wiki raw endpoints).

use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::server::AppState;
use crate::wiki::{guess_mime, validate_path_pub};
use crate::workspace::WorkspaceQuery;

/// Directory names excluded from the workspace tree (PLAN-068 §5.1).
/// Dot-directories (`.git`, `.autoos`, `.worktrees`, `.pnpm`, …) are already
/// skipped by the dotfile rule shared with `wiki::build_tree`; this list
/// covers the non-dotted heavy build/vendor dirs.
const IGNORED_DIRS: &[&str] = &["node_modules", "target", "dist", "tmp", "vendor"];

/// Node budget for one tree response.
const MAX_NODES: usize = 5000;

/// Max nesting depth: folders beyond this are listed but not descended.
const MAX_DEPTH: usize = 12;

/// Raw read cap; larger files answer 413 (frontend "file too large" state).
const MAX_FILE_BYTES: u64 = 20 * 1024 * 1024;

/// Flatten `WorkspaceQuery` so `?workspace=<id>` works like every other route
/// (`spec_tree::SpecTreeQuery` same shape).
#[derive(Deserialize)]
pub struct FilesQuery {
    #[serde(flatten)]
    pub workspace: WorkspaceQuery,
}

/// FileTree 消费的 fs 形态节点 schema（PLAN-614 gallery 契约）：
/// `id` = 相对路径，`children` **恒存在**（叶子为空数组——flatten_tree 对
/// 缺键字段取 `.len()` 会踩 undefined），`icon`/`badge` 留空走组件内自动
/// 映射。刻意不用 wiki 的 `TreeNode`（name/type、叶子缺 children，schema
/// 不同——实证 TypeError: reading 'length'）。
#[derive(Serialize, Clone, Debug)]
pub struct FilesNode {
    pub id: String,
    pub label: String,
    pub children: Vec<FilesNode>,
    pub kind: String,    // "dir" | "file"
    pub icon: String,    // "" = 组件按 kind/扩展名自动映射
    pub is_leaf: bool,
    pub badge: String,
}

#[derive(Serialize)]
pub struct FilesTreeResponse {
    pub tree: Vec<FilesNode>,
    pub truncated: bool,
}

pub fn files_routes() -> Router<AppState> {
    Router::new()
        .route("/api/files/tree", get(files_tree))
        .route("/api/files/raw/{*path}", get(files_raw))
}

fn ws_root_for(state: &AppState, q: &FilesQuery) -> std::path::PathBuf {
    let ws = state.registry.get(&q.workspace.id_or_default(&state.registry));
    ws.root.clone()
}

/// Extended MIME table: media/code types the wiki table doesn't cover;
/// falls back to `wiki::guess_mime` (md/txt/pdf/png/json/...).
fn guess_mime_ext(path: &std::path::Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        // Safety override: no text/html from workspace files (see module doc).
        Some("html") | Some("htm") => "text/plain",
        Some("webp") => "image/webp",
        Some("bmp") => "image/bmp",
        Some("ico") => "image/x-icon",
        Some("avif") => "image/avif",
        Some("mp4") | Some("m4v") => "video/mp4",
        Some("webm") => "video/webm",
        Some("mov") => "video/quicktime",
        Some("mp3") => "audio/mpeg",
        Some("wav") => "audio/wav",
        Some("ogg") => "audio/ogg",
        Some("markdown") => "text/markdown",
        Some("at") | Some("rs") | Some("toml") | Some("yaml") | Some("yml")
        | Some("ts") | Some("tsx") | Some("vue") | Some("sh") | Some("py")
        | Some("sql") | Some("cmd") | Some("ps1") | Some("lock") => "text/plain",
        _ => guess_mime(path),
    }
}

/// Workspace-root tree: wiki `build_tree` semantics (folders-first +
/// alphabetical, dotfiles skipped) plus the ignore list and node/depth
/// budgets. `budget` counts down across the whole response; `truncated`
/// flips once anything had to be left out.
fn build_ws_tree(
    root: &std::path::Path,
    prefix: &str,
    depth: usize,
    budget: &mut usize,
    truncated: &mut bool,
) -> Vec<FilesNode> {
    let mut entries: Vec<FilesNode> = Vec::new();
    if depth > MAX_DEPTH {
        *truncated = true;
        return entries;
    }
    let Ok(dir) = std::fs::read_dir(root) else {
        return entries;
    };
    let mut dir_entries: Vec<_> = dir.flatten().collect();
    // Folders first, then alphabetical — same ordering contract as the wiki
    // tree so both browsers feel identical.
    dir_entries.sort_by(|a, b| {
        let a_is_dir = a.path().is_dir();
        let b_is_dir = b.path().is_dir();
        b_is_dir
            .cmp(&a_is_dir)
            .then(a.file_name().to_string_lossy().cmp(&b.file_name().to_string_lossy()))
    });
    for entry in &dir_entries {
        if *budget == 0 {
            *truncated = true;
            break;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name == "_manifest.json" || name == "manifest.json" {
            continue;
        }
        let is_dir = entry.path().is_dir();
        if is_dir && IGNORED_DIRS.iter().any(|d| d.eq_ignore_ascii_case(&name)) {
            continue;
        }
        let id = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", prefix, name)
        };
        *budget -= 1;
        if is_dir {
            let children = build_ws_tree(&entry.path(), &id, depth + 1, budget, truncated);
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

/// Resolve a workspace-relative path to a canonical file path, rejecting
/// traversal (`validate_path_pub`) and symlink escapes (canonicalized file
/// must stay under the canonicalized root). Missing paths → 404.
fn resolve_confined(root: &std::path::Path, rel: &str) -> Result<std::path::PathBuf, (StatusCode, String)> {
    validate_path_pub(rel)?;
    let canonical_root = root
        .canonicalize()
        .map_err(|_| (StatusCode::NOT_FOUND, "workspace root missing".into()))?;
    let canonical = root
        .join(rel)
        .canonicalize()
        .map_err(|_| (StatusCode::NOT_FOUND, "file missing".into()))?;
    if !canonical.starts_with(&canonical_root) {
        return Err((StatusCode::BAD_REQUEST, "path escapes workspace".into()));
    }
    Ok(canonical)
}

/// GET /api/files/tree — workspace-root tree with ignore rules + budgets.
///
/// A fresh/empty workspace yields an empty tree (no 404), same contract as
/// the specs tree.
async fn files_tree(
    State(state): State<AppState>,
    Query(q): Query<FilesQuery>,
) -> Json<FilesTreeResponse> {
    let root = ws_root_for(&state, &q);
    let mut budget = MAX_NODES;
    let mut truncated = false;
    let tree = build_ws_tree(&root, "", 0, &mut budget, &mut truncated);
    Json(FilesTreeResponse { tree, truncated })
}

/// GET /api/files/raw/{*path} — raw bytes + MIME under workspace confinement.
async fn files_raw(
    State(state): State<AppState>,
    Query(q): Query<FilesQuery>,
    Path(path): Path<String>,
) -> Result<Response, (StatusCode, String)> {
    let root = ws_root_for(&state, &q);
    let canonical = resolve_confined(&root, &path)?;
    let meta = std::fs::metadata(&canonical)
        .map_err(|_| (StatusCode::NOT_FOUND, "file missing".into()))?;
    if meta.is_dir() {
        return Err((StatusCode::BAD_REQUEST, "path is a directory".into()));
    }
    if meta.len() > MAX_FILE_BYTES {
        return Err((StatusCode::PAYLOAD_TOO_LARGE, "file too large".into()));
    }
    let data =
        std::fs::read(&canonical).map_err(|_| (StatusCode::NOT_FOUND, "unreadable".into()))?;
    let mime = guess_mime_ext(&canonical);
    Ok(([(header::CONTENT_TYPE, mime)], data).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(root: &std::path::Path, rel: &str, content: &str) {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, content).unwrap();
    }

    fn names(nodes: &[FilesNode]) -> Vec<&str> {
        nodes.iter().map(|n| n.label.as_str()).collect()
    }

    fn build(root: &std::path::Path) -> (Vec<FilesNode>, bool) {
        let mut budget = MAX_NODES;
        let mut truncated = false;
        let tree = build_ws_tree(root, "", 0, &mut budget, &mut truncated);
        (tree, truncated)
    }

    /// Ignore list + dotfile rule: build/vendor/dot dirs never appear.
    #[test]
    fn tree_skips_ignored_and_hidden_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(root, "README.md", "# r");
        write(root, "src/front/app.at", "widget A {}");
        for ignored in ["node_modules", "target", "dist", "tmp", "vendor"] {
            write(root, &format!("{ignored}/junk.bin"), "x");
        }
        write(root, ".git/HEAD", "ref: x");
        write(root, ".autoos/chats.json", "{}");

        let (tree, truncated) = build(root);
        assert!(!truncated);
        assert_eq!(names(&tree), vec!["src", "README.md"], "got {tree:?}");
        let src_children = &tree[0].children;
        assert_eq!(names(src_children), vec!["front"]);
    }

    /// Folders sort before files, alphabetical within each group.
    #[test]
    fn tree_folders_first_alphabetical() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(root, "z-file.txt", "z");
        write(root, "a-file.txt", "a");
        write(root, "m-dir/x.txt", "x");

        let (tree, _) = build(root);
        assert_eq!(names(&tree), vec!["m-dir", "a-file.txt", "z-file.txt"]);
    }

    /// Node budget: once exhausted the walk stops and reports truncation.
    #[test]
    fn tree_budget_truncates() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        for i in 0..10 {
            write(root, &format!("f{i:02}.txt"), "x");
        }
        let mut budget = 3usize;
        let mut truncated = false;
        let tree = build_ws_tree(root, "", 0, &mut budget, &mut truncated);
        assert!(truncated);
        assert_eq!(tree.len(), 3);
    }

    /// Depth cap: deep chains are cut off and reported as truncated.
    #[test]
    fn tree_depth_cap_truncates() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let mut rel = String::from("deep");
        for i in 0..(MAX_DEPTH + 3) {
            rel = format!("{rel}/l{i}");
        }
        write(root, &format!("{rel}/leaf.txt"), "x");

        let (tree, truncated) = build(root);
        assert!(truncated, "deep chain must report truncation");
        // Walk down: every level on the way has exactly one child until cut.
        let mut node = &tree[0];
        let mut depth_seen = 1usize;
        loop {
            if node.children.is_empty() {
                break;
            }
            node = &node.children[0];
            depth_seen += 1;
        }
        assert!(depth_seen <= MAX_DEPTH + 1, "walked {depth_seen} levels");
    }

    /// Extended MIME: media types + safety override + wiki fallback.
    #[test]
    fn mime_ext_covers_media_and_overrides_html() {
        let m = |name: &str| guess_mime_ext(std::path::Path::new(name));
        assert_eq!(m("a.mp4"), "video/mp4");
        assert_eq!(m("a.webm"), "video/webm");
        assert_eq!(m("a.webp"), "image/webp");
        assert_eq!(m("a.svg"), "image/svg+xml"); // wiki fallback, <img>-safe
        assert_eq!(m("a.md"), "text/markdown"); // wiki fallback
        assert_eq!(m("a.at"), "text/plain");
        assert_eq!(m("a.toml"), "text/plain");
        // Safety: no text/html from workspace files on the app origin.
        assert_eq!(m("a.html"), "text/plain");
        assert_eq!(m("a.unknownext"), "application/octet-stream");
    }

    /// Confinement: traversal rejected; canonicalize keeps symlinks/escapes
    /// under the root; missing files 404.
    #[test]
    fn resolve_confined_rejects_escape_and_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("wsroot");
        std::fs::create_dir_all(root.join("src")).unwrap();
        write(&root, "src/a.rs", "fn x() {}");
        // A file OUTSIDE the workspace root.
        write(&tmp.path().to_path_buf(), "outside/secret.txt", "s");

        // Inside → ok.
        let ok = resolve_confined(&root, "src/a.rs").unwrap();
        assert!(ok.ends_with("src/a.rs"));

        // Name-based traversal → rejected before any fs access.
        assert!(resolve_confined(&root, "../outside/secret.txt").is_err());
        assert!(resolve_confined(&root, "/etc/passwd").is_err());

        // Directory resolves, but the handler's is_dir guard rejects it
        // before serving (asserted via the same metadata check).
        let dir_path = resolve_confined(&root, "src").expect("existing dir resolves");
        assert!(
            std::fs::metadata(&dir_path).unwrap().is_dir(),
            "handler is_dir guard rejects directory raw reads"
        );

        // Missing → 404-style error.
        assert!(resolve_confined(&root, "src/nope.rs").is_err());
    }

}
