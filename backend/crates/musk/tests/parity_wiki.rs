//! Parity tests — verify `auto_generated::wiki` behaves identically to the
//! hand-written `wiki` module (Plan 018 Phase 3 pilot).
//!
//! Scope: the transpiled module contains the data model (WikiSource /
//! WikiPage / WikiPageMeta / WikiManifest / TreeNode), the WikiStore READ path
//! (new / load / list_pages / get_page / search) and the pure tree builders
//! (walk_md_files / build_tree / strip_md_extensions). The WRITE path
//! (create_page / update_page / delete_page / save_manifest) is deferred —
//! it was removed from `wiki.at` while chasing an a2r parser state bug, and
//! the axum routes are hand-written boundaries anyway.
//!
//! Known deviation: `build_tree` in the ag module sets file-node `size` /
//! `modified` to `None`. The hand-written builder enriches them from
//! `fs::Metadata`, but a2r's Auto int model casts `.len()` to i32 (C9 codegen
//! limitation) which can't feed an `Option<u64>` size. Tree structure /
//! ordering / stripping — the functional surface — is asserted here.

use musk::wiki as hw;                 // hand-written
use musk::auto_generated::wiki as ag; // a2r-transpiled Auto

// ──────────────────────────────────────────────────────────
// WikiSource — wire format parity
// ──────────────────────────────────────────────────────────

#[test]
fn parity_wiki_source_wire_format() {
    for (hw_r, ag_r, expected) in [
        (hw::WikiSource::Manual, ag::WikiSource::Manual, "\"manual\""),
        (hw::WikiSource::Guide, ag::WikiSource::Guide, "\"guide\""),
        (hw::WikiSource::ApiRef, ag::WikiSource::ApiRef, "\"api_ref\""),
        (hw::WikiSource::Custom, ag::WikiSource::Custom, "\"custom\""),
    ] {
        assert_eq!(serde_json::to_string(&hw_r).unwrap(), expected);
        assert_eq!(serde_json::to_string(&ag_r).unwrap(), expected);
    }

    // Round-trip: both deserialize the same snake_case names.
    for name in ["manual", "guide", "api_ref", "custom"] {
        let hw_v: hw::WikiSource = serde_json::from_str(&format!("\"{name}\"")).unwrap();
        let ag_v: ag::WikiSource = serde_json::from_str(&format!("\"{name}\"")).unwrap();
        assert_eq!(
            serde_json::to_string(&hw_v).unwrap(),
            serde_json::to_string(&ag_v).unwrap(),
            "round-trip mismatch for {name}"
        );
    }
}

// ──────────────────────────────────────────────────────────
// Data model — wire format parity
// ──────────────────────────────────────────────────────────

#[test]
fn parity_wiki_page_wire_format() {
    let hw_p = hw::WikiPage {
        slug: "docs/arch".into(),
        title: "Architecture".into(),
        content: "markdown body".into(),
        source_type: hw::WikiSource::Guide,
        tags: vec!["design".into(), "rust".into()],
        version: 2,
        created_at: 100,
        updated_at: 200,
    };
    let ag_p = ag::WikiPage {
        slug: "docs/arch".into(),
        title: "Architecture".into(),
        content: "markdown body".into(),
        source_type: ag::WikiSource::Guide,
        tags: vec!["design".into(), "rust".into()],
        version: 2,
        created_at: 100,
        updated_at: 200,
    };
    assert_eq!(
        serde_json::to_string(&hw_p).unwrap(),
        serde_json::to_string(&ag_p).unwrap(),
        "WikiPage wire mismatch"
    );
}

#[test]
fn parity_wiki_page_meta_wire_format() {
    let hw_m = hw::WikiPageMeta {
        slug: "api".into(),
        title: "API Reference".into(),
        source_type: hw::WikiSource::ApiRef,
        tags: vec!["http".into()],
        version: 3,
        updated_at: 42,
    };
    let ag_m = ag::WikiPageMeta {
        slug: "api".into(),
        title: "API Reference".into(),
        source_type: ag::WikiSource::ApiRef,
        tags: vec!["http".into()],
        version: 3,
        updated_at: 42,
    };
    assert_eq!(
        serde_json::to_string(&hw_m).unwrap(),
        serde_json::to_string(&ag_m).unwrap(),
        "WikiPageMeta wire mismatch"
    );
}

#[test]
fn parity_tree_node_wire_format() {
    // Folder node with a nested child — both skip None size/modified.
    let hw_t = hw::TreeNode {
        name: "a".into(),
        path: "a".into(),
        node_type: "folder".into(),
        children: Some(vec![hw::TreeNode {
            name: "b.md".into(),
            path: "a/b.md".into(),
            node_type: "file".into(),
            children: None,
            size: Some(10),
            modified: Some(5),
        }]),
        size: None,
        modified: None,
    };
    let ag_t = ag::TreeNode {
        name: "a".into(),
        path: "a".into(),
        node_type: "folder".into(),
        children: Some(vec![ag::TreeNode {
            name: "b.md".into(),
            path: "a/b.md".into(),
            node_type: "file".into(),
            children: None,
            size: Some(10),
            modified: Some(5),
        }]),
        size: None,
        modified: None,
    };
    assert_eq!(
        serde_json::to_string(&hw_t).unwrap(),
        serde_json::to_string(&ag_t).unwrap(),
        "TreeNode wire mismatch"
    );
    // `type` rename must be applied on both.
    let hw_json = serde_json::to_string(&hw_t).unwrap();
    assert!(hw_json.contains("\"type\":\"folder\""));
    assert!(!hw_json.contains("node_type"));
}

// ──────────────────────────────────────────────────────────
// WikiStore — read path parity (hw vs ag)
// ──────────────────────────────────────────────────────────

/// Set up a wiki dir with two .md files and a manifest; return (wiki, raw) tmp dirs.
fn temp_wiki() -> (tempfile::TempDir, tempfile::TempDir) {
    let wiki = tempfile::tempdir().unwrap();
    let raw = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(wiki.path().join("docs")).unwrap();
    std::fs::write(wiki.path().join("docs/guide.md"), "Guide content here").unwrap();
    std::fs::write(wiki.path().join("api.md"), "API docs body").unwrap();
    std::fs::write(
        wiki.path().join("manifest.json"),
        r#"{"pages":[{"slug":"api","title":"API Reference","source_type":"api_ref","tags":["http"],"version":3,"updated_at":42}]}"#,
    )
    .unwrap();
    (wiki, raw)
}

fn page_tuples(pages: &[ag::WikiPageMeta]) -> Vec<(String, String, String, Vec<String>, u32, u64)> {
    let mut v: Vec<_> = pages
        .iter()
        .map(|p| {
            (
                p.slug.clone(),
                p.title.clone(),
                serde_json::to_string(&p.source_type).unwrap(),
                p.tags.clone(),
                p.version,
                p.updated_at,
            )
        })
        .collect();
    v.sort_by(|a, b| a.0.cmp(&b.0));
    v
}

#[test]
fn parity_wiki_store_load_and_list() {
    let (wiki, raw) = temp_wiki();
    let hw_store = hw::WikiStore::new(wiki.path().to_path_buf(), raw.path().to_path_buf());
    hw_store.load();
    let ag_store = ag::WikiStore::new(wiki.path().to_path_buf(), raw.path().to_path_buf());
    ag_store.load();

    let hw_list = hw_store.list_pages();
    let ag_list = ag_store.list_pages();
    assert_eq!(hw_list.len(), ag_list.len(), "page count mismatch");

    // Compare normalized tuples (sorted by slug) — ag meta order is arbitrary.
    let hw_norm: Vec<_> = hw_list
        .iter()
        .map(|p| {
            (
                p.slug.clone(),
                p.title.clone(),
                serde_json::to_string(&p.source_type).unwrap(),
                p.tags.clone(),
                p.version,
                p.updated_at,
            )
        })
        .collect();
    let mut hw_norm = hw_norm;
    hw_norm.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(hw_norm, page_tuples(&ag_list));

    // Manifest metadata was applied: api gets title/source_type/tags/version.
    let api = ag_list.iter().find(|p| p.slug == "api").expect("api page");
    assert_eq!(api.title, "API Reference");
    assert_eq!(serde_json::to_string(&api.source_type).unwrap(), "\"api_ref\"");
    assert_eq!(api.tags, vec!["http"]);
    assert_eq!(api.version, 3);
    // docs/guide has no manifest entry → defaults.
    let guide = ag_list.iter().find(|p| p.slug == "docs/guide").expect("guide page");
    assert_eq!(guide.title, "docs/guide", "default title is the slug");
    assert_eq!(serde_json::to_string(&guide.source_type).unwrap(), "\"custom\"");
    assert!(guide.tags.is_empty());
    assert_eq!(guide.version, 1);
}

#[test]
fn parity_wiki_store_get_page() {
    let (wiki, raw) = temp_wiki();
    let hw_store = hw::WikiStore::new(wiki.path().to_path_buf(), raw.path().to_path_buf());
    hw_store.load();
    let ag_store = ag::WikiStore::new(wiki.path().to_path_buf(), raw.path().to_path_buf());
    ag_store.load();

    let hw_page = hw_store.get_page("docs/guide").expect("hw guide page");
    let ag_page = ag_store.get_page("docs/guide").expect("ag guide page");
    assert_eq!(hw_page.slug, ag_page.slug);
    assert_eq!(hw_page.title, ag_page.title);
    assert_eq!(hw_page.content, ag_page.content, "md content must load");
    assert_eq!(
        serde_json::to_string(&hw_page.source_type).unwrap(),
        serde_json::to_string(&ag_page.source_type).unwrap()
    );
    assert_eq!(hw_page.version, ag_page.version);
    assert_eq!(hw_page.created_at, ag_page.created_at);

    // Missing page → None on both.
    assert!(hw_store.get_page("nope").is_none());
    assert!(ag_store.get_page("nope").is_none());
}

#[test]
fn parity_wiki_store_search() {
    let (wiki, raw) = temp_wiki();
    let hw_store = hw::WikiStore::new(wiki.path().to_path_buf(), raw.path().to_path_buf());
    hw_store.load();
    let ag_store = ag::WikiStore::new(wiki.path().to_path_buf(), raw.path().to_path_buf());
    ag_store.load();

    // Case-insensitive content hit ("GUIDE" matches "Guide content here").
    let hw_hits = hw_store.search("GUIDE");
    let ag_hits = ag_store.search("GUIDE");
    let mut hw_slugs: Vec<_> = hw_hits.iter().map(|p| p.slug.clone()).collect();
    hw_slugs.sort();
    let mut ag_slugs: Vec<_> = ag_hits.iter().map(|p| p.slug.clone()).collect();
    ag_slugs.sort();
    assert_eq!(hw_slugs, ag_slugs);
    assert_eq!(ag_slugs, vec!["docs/guide"]);

    // Title hit on the manifest-metadata page.
    assert_eq!(hw_store.search("api reference").len(), ag_store.search("api reference").len());
    assert_eq!(ag_store.search("api reference").len(), 1);

    // No match → empty on both.
    assert_eq!(hw_store.search("zzz_not_there").len(), 0);
    assert_eq!(ag_store.search("zzz_not_there").len(), 0);
}

#[test]
fn parity_wiki_manifest_preference() {
    // `_manifest.json` wins over `manifest.json`.
    let wiki = tempfile::tempdir().unwrap();
    let raw = tempfile::tempdir().unwrap();
    std::fs::write(wiki.path().join("a.md"), "a body").unwrap();
    std::fs::write(
        wiki.path().join("manifest.json"),
        r#"{"pages":[{"slug":"a","title":"OLD","source_type":"guide","tags":[],"version":1,"updated_at":1}]}"#,
    )
    .unwrap();
    std::fs::write(
        wiki.path().join("_manifest.json"),
        r#"{"pages":[{"slug":"a","title":"NEW","source_type":"manual","tags":["x"],"version":7,"updated_at":9}]}"#,
    )
    .unwrap();

    let hw_store = hw::WikiStore::new(wiki.path().to_path_buf(), raw.path().to_path_buf());
    hw_store.load();
    let ag_store = ag::WikiStore::new(wiki.path().to_path_buf(), raw.path().to_path_buf());
    ag_store.load();

    let hw_page = hw_store.get_page("a").unwrap();
    let ag_page = ag_store.get_page("a").unwrap();
    assert_eq!(hw_page.title, ag_page.title);
    assert_eq!(ag_page.title, "NEW");
    assert_eq!(serde_json::to_string(&ag_page.source_type).unwrap(), "\"manual\"");
    assert_eq!(ag_page.version, 7);
}

// ──────────────────────────────────────────────────────────
// Tree builders — ag-only, pinned to the hand-written semantics
// ──────────────────────────────────────────────────────────

/// Layout used by the tree tests:
///   a/            (folder)
///     z.md
///     b/          (folder)
///       n.md
///   b.md
///   c.md
///   _manifest.json   (ignored)
///   .hidden.md       (ignored)
fn temp_tree() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join("a/b")).unwrap();
    std::fs::write(tmp.path().join("b.md"), "").unwrap();
    std::fs::write(tmp.path().join("c.md"), "").unwrap();
    std::fs::write(tmp.path().join("a/z.md"), "").unwrap();
    std::fs::write(tmp.path().join("a/b/n.md"), "").unwrap();
    std::fs::write(tmp.path().join("_manifest.json"), "{}").unwrap();
    std::fs::write(tmp.path().join(".hidden.md"), "").unwrap();
    tmp
}

#[test]
fn parity_tree_walk_md_files() {
    let tmp = temp_tree();
    let mut slugs = ag::walk_md_files(tmp.path().to_path_buf(), "").unwrap();
    slugs.sort();
    // Relative paths, forward-slash separators, `.md` stripped, dotfiles and
    // manifests excluded.
    assert_eq!(slugs, vec!["a/b/n", "a/z", "b", "c"]);
}

#[test]
fn parity_tree_build_structure_and_order() {
    let tmp = temp_tree();
    let tree = ag::build_tree(tmp.path().to_path_buf(), "");

    // Root: folders first (a), then files alphabetically (b.md, c.md).
    assert_eq!(tree.len(), 3, "dotfile and manifest must be excluded");
    assert_eq!((tree[0].name.as_str(), tree[0].node_type.as_str()), ("a", "folder"));
    assert_eq!((tree[1].name.as_str(), tree[1].node_type.as_str()), ("b.md", "file"));
    assert_eq!((tree[2].name.as_str(), tree[2].node_type.as_str()), ("c.md", "file"));
    // Raw tree keeps `.md` in file names.
    assert_eq!(tree[1].path, "b.md");

    // Folder children: subfolder b first, then z.md.
    let a = &tree[0];
    assert_eq!(a.path, "a");
    assert_eq!(a.children.as_ref().unwrap().len(), 2);
    let children = a.children.as_ref().unwrap();
    assert_eq!((children[0].name.as_str(), children[0].node_type.as_str()), ("b", "folder"));
    assert_eq!(children[1].name, "z.md");
    // Deep child path is prefixed (raw tree keeps `.md`).
    let b = &children[0];
    assert_eq!(b.path, "a/b");
    assert_eq!(b.children.as_ref().unwrap()[0].path, "a/b/n.md");
}

#[test]
fn parity_tree_strip_md_extensions() {
    let tmp = temp_tree();
    let tree = ag::build_tree(tmp.path().to_path_buf(), "");
    let stripped = ag::strip_md_extensions(tree);

    // File names lose `.md`; folder names and paths stay; children recurse.
    assert_eq!(stripped[0].name, "a");
    let a_children = stripped[0].children.as_ref().unwrap();
    assert_eq!(a_children[0].name, "b");
    assert_eq!(a_children[1].name, "z");
    assert_eq!(a_children[1].path, "a/z");
    assert_eq!(stripped[1].name, "b");
    assert_eq!(stripped[2].name, "c");
    assert_eq!(a_children[0].children.as_ref().unwrap()[0].name, "n");
    assert_eq!(a_children[0].children.as_ref().unwrap()[0].path, "a/b/n");
}
