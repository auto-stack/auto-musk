//! Wiki Knowledge Layer — markdown pages with metadata manifest + a raw
//! resource tree (arbitrary files / folders).
//!
//! Ported from auto-forge's `backend/src/forge/wiki.rs`, flattened to
//! auto-musk's single-project model: the `{project}` path segment in every
//! route is accepted but ignored (there is exactly one wiki store per musk
//! instance). The store is injected via [`crate::server::AppState`] as an
//! `Arc<WikiStore>` instead of auto-forge's global `OnceLock` singleton, so it
//! stays testable with `tempfile`.

use axum::{
    extract::{DefaultBodyLimit, Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::server::AppState;
use crate::workspace::WorkspaceQuery;

// ─── Data Model ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiPage {
    pub slug: String,
    pub title: String,
    pub content: String,
    pub source_type: WikiSource,
    pub tags: Vec<String>,
    pub version: u32,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WikiSource {
    Manual,
    Guide,
    ApiRef,
    Custom,
}

impl Default for WikiSource {
    fn default() -> Self {
        WikiSource::Custom
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WikiManifest {
    pages: Vec<WikiPageMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiPageMeta {
    pub slug: String,
    pub title: String,
    pub source_type: WikiSource,
    pub tags: Vec<String>,
    pub version: u32,
    pub updated_at: u64,
}

// ─── Tree Node ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
    pub name: String,
    pub path: String,
    #[serde(rename = "type")]
    pub node_type: String, // "file" | "folder"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<TreeNode>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<u64>,
}

pub(crate) fn build_tree(root: &std::path::Path, prefix: &str) -> Vec<TreeNode> {
    let mut entries: Vec<TreeNode> = Vec::new();
    let Ok(dir) = std::fs::read_dir(root) else {
        return entries;
    };
    let mut dir_entries: Vec<_> = dir.flatten().collect();
    // Folders first, then alphabetical — matches auto-forge + frontend expectations.
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
        let path = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", prefix, name)
        };
        let meta = entry.metadata().ok();
        if entry.path().is_dir() {
            let children = build_tree(&entry.path(), &path);
            entries.push(TreeNode {
                name,
                path,
                node_type: "folder".into(),
                children: Some(children),
                size: None,
                modified: None,
            });
        } else {
            entries.push(TreeNode {
                name,
                path,
                node_type: "file".into(),
                children: None,
                size: meta.as_ref().map(|m| m.len()),
                modified: meta
                    .and_then(|m| m.modified().ok())
                    .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()),
            });
        }
    }
    entries
}

/// Recursively strip `.md` extensions from file nodes — wiki trees show pages
/// by slug (no extension), while raw trees keep the real filenames.
pub(crate) fn strip_md_extensions(nodes: &mut [TreeNode]) {
    for node in nodes.iter_mut() {
        if node.node_type == "file" && node.name.ends_with(".md") {
            node.name = node.name.trim_end_matches(".md").to_string();
            node.path = node.path.trim_end_matches(".md").to_string();
        }
        if let Some(ref mut children) = node.children {
            strip_md_extensions(children);
        }
    }
}

// ─── Path Validation ─────────────────────────────────────────────────────────

fn validate_path(path: &str) -> Result<(), (StatusCode, String)> {
    if path.contains("..") || path.starts_with('/') || path.starts_with('\\') {
        return Err((StatusCode::BAD_REQUEST, "Invalid path".into()));
    }
    Ok(())
}

/// `pub(crate)`: 供 a2r `extern_impl`(wiki_raw_file / raw_upload)复用同一套
/// 路径校验 + MIME 判定,避免复制两份语义。
pub(crate) fn validate_path_pub(path: &str) -> Result<(), (StatusCode, String)> {
    validate_path(path)
}

// ─── Wiki Store ──────────────────────────────────────────────────────────────

/// Single-project wiki store. `wiki_dir` holds page `.md` files + a
/// `_manifest.json`; `raw_dir` holds arbitrary uploaded resources. Both are
/// created on construction. The in-memory `pages` map is a lazily-loaded
/// cache keyed by slug; disk is the source of truth, written on every mutation.
pub struct WikiStore {
    /// Directory holding page `.md` files + `_manifest.json`.
    pub wiki_dir: PathBuf,
    /// Directory holding uploaded raw resources.
    pub raw_dir: PathBuf,
    pages: Mutex<HashMap<String, WikiPage>>,
}

impl WikiStore {
    /// Create a new store rooted at the given wiki/raw directories (created
    /// if missing). Accept arbitrary paths so tests can use `tempfile`.
    pub fn new(wiki_dir: PathBuf, raw_dir: PathBuf) -> Self {
        let _ = std::fs::create_dir_all(&wiki_dir);
        let _ = std::fs::create_dir_all(&raw_dir);
        Self {
            wiki_dir,
            raw_dir,
            pages: Mutex::new(HashMap::new()),
        }
    }

    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Load (or refresh) the in-memory page cache from disk: read the manifest
    /// for metadata and every `.md` file for content. Idempotent — safe to
    /// call before any read.
    pub fn load(&self) {
        let mut pages = self.pages.lock().unwrap();
        pages.clear();

        let metas: HashMap<String, WikiPageMeta> = std::fs::read_to_string(self.wiki_dir.join("_manifest.json"))
            .or_else(|_| std::fs::read_to_string(self.wiki_dir.join("manifest.json")))
            .ok()
            .and_then(|c| serde_json::from_str::<WikiManifest>(&c).ok())
            .map(|m| m.pages.into_iter().map(|p| (p.slug.clone(), p)).collect())
            .unwrap_or_default();

        let Ok(slugs) = walk_md_files(&self.wiki_dir, "") else { return };
        for slug in slugs {
            let page_path = self.wiki_dir.join(format!("{}.md", &slug));
            if let Ok(content) = std::fs::read_to_string(&page_path) {
                let meta = metas.get(&slug);
                let page = WikiPage {
                    slug: slug.clone(),
                    title: meta.map(|m| m.title.clone()).unwrap_or_else(|| slug.clone()),
                    content,
                    source_type: meta.map(|m| m.source_type.clone()).unwrap_or_default(),
                    tags: meta.map(|m| m.tags.clone()).unwrap_or_default(),
                    version: meta.map(|m| m.version).unwrap_or(1),
                    created_at: meta.map(|m| m.updated_at).unwrap_or(0),
                    updated_at: meta.map(|m| m.updated_at).unwrap_or(0),
                };
                pages.insert(slug, page);
            }
        }
    }

    pub fn list_pages(&self) -> Vec<WikiPageMeta> {
        let pages = self.pages.lock().unwrap();
        pages
            .iter()
            .map(|(_, p)| WikiPageMeta {
                slug: p.slug.clone(),
                title: p.title.clone(),
                source_type: p.source_type.clone(),
                tags: p.tags.clone(),
                version: p.version,
                updated_at: p.updated_at,
            })
            .collect()
    }

    pub fn get_page(&self, slug: &str) -> Option<WikiPage> {
        self.pages.lock().unwrap().get(slug).cloned()
    }

    pub fn create_page(&self, page: WikiPage) -> Result<WikiPage, String> {
        let mut pages = self.pages.lock().unwrap();
        if pages.contains_key(&page.slug) {
            return Err(format!("Page '{}' already exists", page.slug));
        }

        let now = Self::now();
        let page = WikiPage {
            created_at: now,
            updated_at: now,
            version: 1,
            ..page
        };

        let page_path = self.wiki_dir.join(format!("{}.md", page.slug));
        if let Some(parent) = page_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(&page_path, &page.content)
            .map_err(|e| format!("Failed to write page: {}", e))?;

        pages.insert(page.slug.clone(), page.clone());
        drop(pages);
        self.save_manifest();

        Ok(page)
    }

    pub fn update_page(
        &self,
        slug: &str,
        content: String,
        title: Option<String>,
        tags: Option<Vec<String>>,
    ) -> Result<WikiPage, String> {
        let mut pages = self.pages.lock().unwrap();
        let page = pages
            .get_mut(slug)
            .ok_or_else(|| format!("Page '{}' not found", slug))?;

        page.content = content;
        if let Some(t) = title {
            page.title = t;
        }
        if let Some(t) = tags {
            page.tags = t;
        }
        page.version += 1;
        page.updated_at = Self::now();

        let updated = page.clone();
        drop(pages);

        let page_path = self.wiki_dir.join(format!("{}.md", slug));
        if let Some(parent) = page_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(&page_path, &updated.content)
            .map_err(|e| format!("Failed to write page: {}", e))?;

        self.save_manifest();
        Ok(updated)
    }

    pub fn delete_page(&self, slug: &str) -> Result<(), String> {
        let mut pages = self.pages.lock().unwrap();
        if pages.remove(slug).is_none() {
            return Err(format!("Page '{}' not found", slug));
        }
        drop(pages);

        let page_path = self.wiki_dir.join(format!("{}.md", slug));
        if page_path.exists() {
            let _ = std::fs::remove_file(&page_path);
        }
        self.save_manifest();
        Ok(())
    }

    pub fn search(&self, query: &str) -> Vec<WikiPage> {
        let query_lower = query.to_lowercase();
        let pages = self.pages.lock().unwrap();
        pages
            .iter()
            .filter(|(_, p)| {
                p.content.to_lowercase().contains(&query_lower)
                    || p.title.to_lowercase().contains(&query_lower)
            })
            .map(|(_, p)| p.clone())
            .collect()
    }

    fn save_manifest(&self) {
        let pages = self.pages.lock().unwrap();
        let metas: Vec<WikiPageMeta> = pages
            .iter()
            .map(|(_, p)| WikiPageMeta {
                slug: p.slug.clone(),
                title: p.title.clone(),
                source_type: p.source_type.clone(),
                tags: p.tags.clone(),
                version: p.version,
                updated_at: p.updated_at,
            })
            .collect();
        drop(pages);

        let manifest = WikiManifest { pages: metas };
        if let Ok(json) = serde_json::to_string_pretty(&manifest) {
            let _ = std::fs::write(self.wiki_dir.join("_manifest.json"), json);
        }
    }
}

fn walk_md_files(root: &std::path::Path, prefix: &str) -> Result<Vec<String>, String> {
    let mut result = Vec::new();
    let dir = std::fs::read_dir(root).map_err(|e| e.to_string())?;
    for entry in dir.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name == "_manifest.json" || name == "manifest.json" {
            continue;
        }
        let path = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", prefix, name)
        };
        if entry.path().is_dir() {
            result.extend(walk_md_files(&entry.path(), &path)?);
        } else if name.ends_with(".md") {
            // Strip .md extension to get the slug.
            result.push(path.trim_end_matches(".md").to_string());
        }
    }
    Ok(result)
}

// ─── MIME Helper ─────────────────────────────────────────────────────────────

/// `pub(crate)`: 供 a2r `extern_impl` 的 raw_file 复用 MIME 判定。
pub(crate) fn guess_mime(path: &std::path::Path) -> &'static str {    match path.extension().and_then(|e| e.to_str()) {
        Some("md") => "text/markdown",
        Some("txt") => "text/plain",
        Some("pdf") => "application/pdf",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("json") => "application/json",
        Some("csv") => "text/csv",
        Some("html") => "text/html",
        Some("js") => "application/javascript",
        Some("css") => "text/css",
        Some("xml") => "application/xml",
        Some("zip") => "application/zip",
        _ => "application/octet-stream",
    }
}

// ─── API DTOs ────────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct WikiListResponse {
    pages: Vec<WikiPageMeta>,
}

#[derive(Serialize)]
struct WikiPageResponse {
    page: WikiPage,
}

#[derive(Deserialize)]
struct CreatePageRequest {
    slug: String,
    title: String,
    content: String,
    #[serde(default)]
    source_type: WikiSource,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Deserialize)]
struct UpdatePageRequest {
    content: String,
    title: Option<String>,
    tags: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct SearchRequest {
    query: String,
}

#[derive(Serialize)]
struct SearchResponse {
    results: Vec<WikiPage>,
}

#[derive(Deserialize)]
struct MkdirRequest {
    path: String,
}

#[derive(Deserialize)]
struct UploadQuery {
    #[serde(default)]
    prefix: String,
    #[serde(default)]
    workspace: Option<String>,
}

// ─── API Handlers ────────────────────────────────────────────────────────────

/// `GET /api/forge/wiki/{project}/tree` — directory tree of wiki pages.
/// `project` is accepted but ignored (single-project model).
async fn wiki_tree(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
    Path(_project): Path<String>,
) -> Json<Vec<TreeNode>> {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    let wiki_dir = ws.wiki.wiki_dir.clone();
    let mut tree = build_tree(&wiki_dir, "");
    strip_md_extensions(&mut tree);
    Json(tree)
}

/// `GET /api/forge/raw/{project}/tree` — directory tree of raw resources.
async fn raw_tree(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
    Path(_project): Path<String>,
) -> Json<Vec<TreeNode>> {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    let raw_dir = ws.wiki.raw_dir.clone();
    let _ = std::fs::create_dir_all(&raw_dir);
    let tree = build_tree(&raw_dir, "");
    Json(tree)
}

/// `GET /api/forge/wiki/{project}/pages` — list all page metadata.
async fn list_pages(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
    Path(_project): Path<String>,
) -> Json<WikiListResponse> {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    ws.wiki.load();
    let pages = ws.wiki.list_pages();
    Json(WikiListResponse { pages })
}

/// `GET /api/forge/wiki/{project}/page/{*slug}` — read a single page.
async fn get_page(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
    Path((_project, slug)): Path<(String, String)>,
) -> Result<Json<WikiPageResponse>, StatusCode> {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    validate_path(&slug).map_err(|_| StatusCode::BAD_REQUEST)?;
    ws.wiki.load();
    match ws.wiki.get_page(&slug) {
        Some(page) => Ok(Json(WikiPageResponse { page })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// `POST /api/forge/wiki/{project}/pages` — create a new page.
async fn create_page(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
    Path(_project): Path<String>,
    Json(req): Json<CreatePageRequest>,
) -> Result<Json<WikiPageResponse>, (StatusCode, String)> {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    validate_path(&req.slug)?;
    ws.wiki.load();
    let page = WikiPage {
        slug: req.slug,
        title: req.title,
        content: req.content,
        source_type: req.source_type,
        tags: req.tags,
        version: 0,
        created_at: 0,
        updated_at: 0,
    };
    ws.wiki
        .create_page(page)
        .map(|p| Json(WikiPageResponse { page: p }))
        .map_err(|e| (StatusCode::CONFLICT, e))
}

/// `PUT /api/forge/wiki/{project}/page/{*slug}` — update page content/meta.
async fn update_page(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
    Path((_project, slug)): Path<(String, String)>,
    Json(req): Json<UpdatePageRequest>,
) -> Result<Json<WikiPageResponse>, (StatusCode, String)> {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    validate_path(&slug)?;
    ws.wiki.load();
    ws.wiki
        .update_page(&slug, req.content, req.title, req.tags)
        .map(|p| Json(WikiPageResponse { page: p }))
        .map_err(|e| (StatusCode::NOT_FOUND, e))
}

/// `DELETE /api/forge/wiki/{project}/page/{*slug}` — delete a page.
async fn delete_page(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
    Path((_project, slug)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, String)> {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    validate_path(&slug)?;
    ws.wiki.load();
    ws.wiki
        .delete_page(&slug)
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::NOT_FOUND, e))
}

/// `POST /api/forge/wiki/{project}/search` — full-text search across pages.
async fn search(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
    Path(_project): Path<String>,
    Json(req): Json<SearchRequest>,
) -> Json<SearchResponse> {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    ws.wiki.load();
    let results = ws.wiki.search(&req.query);
    Json(SearchResponse { results })
}

/// `POST /api/forge/raw/{project}/upload?prefix=` — multipart upload of files.
async fn raw_upload(
    State(state): State<AppState>,
    Path(_project): Path<String>,
    Query(query): Query<UploadQuery>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let ws = state
        .registry
        .get(&WorkspaceQuery { workspace: query.workspace.clone() }.id_or_default(&state.registry));
    let raw_dir = ws.wiki.raw_dir.clone();
    if !query.prefix.is_empty() {
        validate_path(&query.prefix)?;
    }

    let mut uploaded = Vec::new();
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    {
        let filename = field.file_name().unwrap_or("unnamed").to_string();
        validate_path(&filename)?;
        let data = field
            .bytes()
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

        let target_dir = if query.prefix.is_empty() {
            raw_dir.clone()
        } else {
            raw_dir.join(&query.prefix)
        };
        let _ = std::fs::create_dir_all(&target_dir);
        let file_path = target_dir.join(&filename);

        std::fs::write(&file_path, &data)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let relative = if query.prefix.is_empty() {
            filename.clone()
        } else {
            format!("{}/{}", query.prefix, filename)
        };
        uploaded.push(relative);
    }
    Ok(Json(serde_json::json!({ "uploaded": uploaded })))
}

/// `GET /api/forge/raw/{project}/file/{*path}` — serve a raw file by path.
async fn raw_file(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
    Path((_project, path)): Path<(String, String)>,
) -> Result<Response, StatusCode> {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    validate_path(&path).map_err(|_| StatusCode::BAD_REQUEST)?;
    let file_path = ws.wiki.raw_dir.join(&path);

    let data = std::fs::read(&file_path).map_err(|_| StatusCode::NOT_FOUND)?;
    let mime = guess_mime(&file_path);

    Ok(([(header::CONTENT_TYPE, mime)], data).into_response())
}

/// `DELETE /api/forge/raw/{project}/file/{*path}` — delete a raw file or folder.
async fn raw_delete(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
    Path((_project, path)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, String)> {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    validate_path(&path)?;
    let file_path = ws.wiki.raw_dir.join(&path);
    if !file_path.exists() {
        return Err((StatusCode::NOT_FOUND, "Not found".into()));
    }
    if file_path.is_dir() {
        std::fs::remove_dir_all(&file_path)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    } else {
        std::fs::remove_file(&file_path)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }
    Ok(StatusCode::NO_CONTENT)
}

/// `POST /api/forge/raw/{project}/mkdir` — create a folder (recursively).
async fn raw_mkdir(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
    Path(_project): Path<String>,
    Json(req): Json<MkdirRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    validate_path(&req.path)?;
    let target = ws.wiki.raw_dir.join(&req.path);
    std::fs::create_dir_all(&target)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

// ─── Router ──────────────────────────────────────────────────────────────────

/// All wiki/raw routes. Routes use the full `/api/forge/...` paths the frontend
/// (`useWiki.ts`) hardcodes, so this is `.merge`-ed into the main router without
/// any prefix.
pub fn wiki_routes() -> Router<AppState> {
    Router::new()
        // Wiki tree + CRUD
        .route("/api/forge/wiki/{project}/tree", get(wiki_tree))
        .route(
            "/api/forge/wiki/{project}/pages",
            get(list_pages).post(create_page),
        )
        .route("/api/forge/wiki/{project}/search", post(search))
        .route(
            "/api/forge/wiki/{project}/page/{*slug}",
            get(get_page).put(update_page).delete(delete_page),
        )
        // Raw tree + CRUD
        .route("/api/forge/raw/{project}/tree", get(raw_tree))
        .route(
            "/api/forge/raw/{project}/upload",
            post(raw_upload).layer(DefaultBodyLimit::max(50 * 1024 * 1024)),
        )
        .route("/api/forge/raw/{project}/mkdir", post(raw_mkdir))
        .route(
            "/api/forge/raw/{project}/file/{*path}",
            get(raw_file).delete(raw_delete),
        )
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_store() -> WikiStore {
        let wiki = std::env::temp_dir().join(format!(
            "musk-wiki-test-{}-wiki",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let raw = std::env::temp_dir().join(format!(
            "musk-wiki-test-{}-raw",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        WikiStore::new(wiki, raw)
    }

    fn page(slug: &str, title: &str, content: &str) -> WikiPage {
        WikiPage {
            slug: slug.into(),
            title: title.into(),
            content: content.into(),
            source_type: WikiSource::Manual,
            tags: vec![],
            version: 0,
            created_at: 0,
            updated_at: 0,
        }
    }

    #[test]
    fn create_get_update_delete_roundtrip() {
        let store = temp_store();
        // Create
        store.create_page(page("intro", "Intro", "# Hello")).unwrap();
        // Get
        let got = store.get_page("intro").expect("page should exist");
        assert_eq!(got.title, "Intro");
        assert_eq!(got.content, "# Hello");
        assert_eq!(got.version, 1);
        // Update
        store
            .update_page("intro", "# Hello v2".into(), Some("Intro v2".into()), None)
            .unwrap();
        let updated = store.get_page("intro").unwrap();
        assert_eq!(updated.title, "Intro v2");
        assert_eq!(updated.content, "# Hello v2");
        assert_eq!(updated.version, 2);
        // Delete
        store.delete_page("intro").unwrap();
        assert!(store.get_page("intro").is_none());
    }

    #[test]
    fn create_duplicate_conflicts() {
        let store = temp_store();
        store.create_page(page("dup", "Dup", "x")).unwrap();
        let err = store.create_page(page("dup", "Dup", "y")).unwrap_err();
        assert!(err.contains("already exists"));
    }

    #[test]
    fn list_and_search() {
        let store = temp_store();
        store.create_page(page("arch", "Architecture", "frontend backend")).unwrap();
        store.create_page(page("api", "API Guide", "REST endpoints")).unwrap();
        let list = store.list_pages();
        assert_eq!(list.len(), 2);
        let hits = store.search("backend");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].slug, "arch");
        let title_hits = store.search("guide");
        assert_eq!(title_hits.len(), 1);
        assert_eq!(title_hits[0].slug, "api");
    }

    #[test]
    fn persists_across_reload() {
        let wiki = std::env::temp_dir().join(format!(
            "musk-wiki-persist-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let raw = std::env::temp_dir().join(format!(
            "musk-wiki-persist-raw-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        {
            let store = WikiStore::new(wiki.clone(), raw.clone());
            store.create_page(page("kept", "Kept", "persisted")).unwrap();
        }
        // A fresh store over the same dir must reload the page from disk.
        let store2 = WikiStore::new(wiki, raw);
        store2.load();
        let got = store2.get_page("kept").expect("page should persist");
        assert_eq!(got.title, "Kept");
        assert_eq!(got.content, "persisted");
    }

    #[test]
    fn nested_slug_creates_subdirs() {
        let store = temp_store();
        store.create_page(page("docs/arch", "Arch", "deep")).unwrap();
        let got = store.get_page("docs/arch").unwrap();
        assert_eq!(got.content, "deep");
        // .md file lives under a subdir on disk
        let md = store.wiki_dir.join("docs/arch.md");
        assert!(md.exists(), "nested page file should exist at {md:?}");
    }

    #[test]
    fn validate_path_rejects_traversal() {
        assert!(validate_path("../etc/passwd").is_err());
        assert!(validate_path("/abs/path").is_err());
        assert!(validate_path(r"\windows").is_err());
        assert!(validate_path("ok/nested").is_ok());
    }
}
