//! parity_wiki_write.rs — Plan 020 Phase C: wiki.at WRITE 路径 parity
//! (create_page / update_page / delete_page / save_manifest)。
//!
//! 与 parity_wiki.rs(READ 路径)互补:同一组操作分别在 hw / ag WikiStore 上
//! 执行,比较可观察行为——返回值(排除 now() 时间戳)、缓存状态、磁盘 .md 与
//! _manifest.json。manifest 元数据含 updated_at,比较时归一化排除时间戳。

use std::sync::Arc;

use musk::auto_generated::wiki as ag;
use musk::wiki as hw;
use tempfile::TempDir;

fn temp_wiki() -> (TempDir, hw::WikiStore, ag::WikiStore) {
    let dir = TempDir::new().unwrap();
    let hw_store = hw::WikiStore::new(dir.path().join("hw").join("wiki"), dir.path().join("hw").join("raw"));
    let ag_store = ag::WikiStore::new(dir.path().join("ag").join("wiki"), dir.path().join("ag").join("raw"));
    (dir, hw_store, ag_store)
}

fn sample_page(slug: &str) -> (hw::WikiPage, ag::WikiPage) {
    let hw_p = hw::WikiPage {
        slug: slug.into(),
        title: "My Page".into(),
        content: "# Hello\n\nbody".into(),
        source_type: hw::WikiSource::Manual,
        tags: vec!["guide".into(), "api".into()],
        version: 0,
        created_at: 0,
        updated_at: 0,
    };
    let ag_p = ag::WikiPage {
        slug: slug.into(),
        title: "My Page".into(),
        content: "# Hello\n\nbody".into(),
        source_type: ag::WikiSource::Manual,
        tags: vec!["guide".into(), "api".into()],
        version: 0,
        created_at: 0,
        updated_at: 0,
    };
    (hw_p, ag_p)
}

/// Compare two created/updated pages ignoring the now() timestamps。
fn assert_page_eq_ignore_ts(ag_p: &ag::WikiPage, hw_p: &hw::WikiPage, ctx: &str) {
    assert_eq!(ag_p.slug, hw_p.slug, "{ctx}: slug");
    assert_eq!(ag_p.title, hw_p.title, "{ctx}: title");
    assert_eq!(ag_p.content, hw_p.content, "{ctx}: content");
    assert_eq!(ag_p.version, hw_p.version, "{ctx}: version");
    assert_eq!(
        serde_json::to_value(&ag_p.source_type).unwrap(),
        serde_json::to_value(&hw_p.source_type).unwrap(),
        "{ctx}: source_type"
    );
    assert_eq!(ag_p.tags, hw_p.tags, "{ctx}: tags");
    assert!(ag_p.created_at > 0, "{ctx}: created_at stamped");
    assert!(ag_p.updated_at > 0, "{ctx}: updated_at stamped");
}

#[test]
fn parity_create_page_matches_hw() {
    let (_dir, hw_store, ag_store) = temp_wiki();
    let (hw_p, ag_p) = sample_page("hello");

    let hw_created = hw_store.create_page(hw_p).unwrap();
    let ag_created = ag_store.create_page(ag_p).unwrap();

    assert_page_eq_ignore_ts(&ag_created, &hw_created, "create");
    assert_eq!(ag_created.version, 1, "fresh page starts at version 1");

    // Cache on both sides (timestamps may differ by a second — compare non-ts fields).
    let hw_cached = hw_store.get_page("hello").unwrap();
    let ag_cached = ag_store.get_page("hello").unwrap();
    assert_page_eq_ignore_ts(&ag_cached, &hw_cached, "cached");
    let hw_md = std::fs::read_to_string(_dir.path().join("hw/wiki/hello.md")).unwrap();
    let ag_md = std::fs::read_to_string(_dir.path().join("ag/wiki/hello.md")).unwrap();
    assert_eq!(ag_md, hw_md, ".md content written");

    // Duplicate create rejected identically.
    let (hw_p2, ag_p2) = sample_page("hello");
    let hw_err = hw_store.create_page(hw_p2).unwrap_err();
    let ag_err = ag_store.create_page(ag_p2).unwrap_err();
    assert_eq!(ag_err, hw_err, "duplicate-create error text");
    assert!(ag_err.contains("already exists"));
}

#[test]
fn parity_update_page_matches_hw() {
    let (_dir, hw_store, ag_store) = temp_wiki();
    let (hw_p, ag_p) = sample_page("doc");
    hw_store.create_page(hw_p).unwrap();
    ag_store.create_page(ag_p).unwrap();

    let hw_upd = hw_store
        .update_page("doc", "new content".into(), Some("New Title".into()), Some(vec!["x".into()]))
        .unwrap();
    let ag_upd = ag_store
        .update_page("doc", "new content".into(), Some("New Title".into()), Some(vec!["x".into()]))
        .unwrap();

    assert_page_eq_ignore_ts(&ag_upd, &hw_upd, "update");
    assert_eq!(ag_upd.version, 2, "version bumped");
    assert_eq!(ag_upd.title, "New Title", "title updated");
    assert_eq!(ag_upd.tags, vec!["x"], "tags updated");
    assert_eq!(
        std::fs::read_to_string(_dir.path().join("ag/wiki/doc.md")).unwrap(),
        std::fs::read_to_string(_dir.path().join("hw/wiki/doc.md")).unwrap(),
        "updated .md written identically"
    );

    // Update missing page → same error.
    let hw_err = hw_store.update_page("nope", "x".into(), None, None).unwrap_err();
    let ag_err = ag_store.update_page("nope", "x".into(), None, None).unwrap_err();
    assert_eq!(ag_err, hw_err, "update-missing error text");
    assert!(ag_err.contains("not found"));
}

#[test]
fn parity_delete_page_matches_hw() {
    let (_dir, hw_store, ag_store) = temp_wiki();
    let (hw_p, ag_p) = sample_page("gone");
    hw_store.create_page(hw_p).unwrap();
    ag_store.create_page(ag_p).unwrap();

    hw_store.delete_page("gone").unwrap();
    ag_store.delete_page("gone").unwrap();

    assert!(ag_store.get_page("gone").is_none(), "ag cache removed");
    assert!(hw_store.get_page("gone").is_none(), "hw cache removed");
    assert!(!_dir.path().join("ag/wiki/gone.md").exists(), "ag .md deleted");
    assert!(!_dir.path().join("hw/wiki/gone.md").exists(), "hw .md deleted");

    // Delete missing → same error.
    let hw_err = hw_store.delete_page("nope").unwrap_err();
    let ag_err = ag_store.delete_page("nope").unwrap_err();
    assert_eq!(ag_err, hw_err, "delete-missing error text");
}

#[test]
fn parity_save_manifest_writes_metas() {
    let (_dir, hw_store, ag_store) = temp_wiki();
    for slug in ["a", "b"] {
        let (hw_p, ag_p) = sample_page(slug);
        hw_store.create_page(hw_p).unwrap();
        ag_store.create_page(ag_p).unwrap();
    }

    let hw_json = std::fs::read_to_string(_dir.path().join("hw/wiki/_manifest.json")).unwrap();
    let ag_json = std::fs::read_to_string(_dir.path().join("ag/wiki/_manifest.json")).unwrap();
    let hw_manifest: serde_json::Value = serde_json::from_str(&hw_json).unwrap();
    let ag_manifest: serde_json::Value = serde_json::from_str(&ag_json).unwrap();

    // 排除 updated_at 时间戳后,两边的 manifest 元数据逐项等价。
    let mut hw_pages: Vec<serde_json::Value> =
        hw_manifest["pages"].as_array().unwrap().clone();
    let mut ag_pages: Vec<serde_json::Value> =
        ag_manifest["pages"].as_array().unwrap().clone();
    for p in hw_pages.iter_mut().chain(ag_pages.iter_mut()) {
        p.as_object_mut().unwrap().remove("updated_at");
    }
    hw_pages.sort_by(|a, b| a["slug"].as_str().cmp(&b["slug"].as_str()));
    ag_pages.sort_by(|a, b| a["slug"].as_str().cmp(&b["slug"].as_str()));
    assert_eq!(ag_pages, hw_pages, "manifest metas parity (minus timestamps)");
    assert_eq!(ag_pages.len(), 2, "two pages in manifest");
    assert_eq!(ag_pages[0]["version"], 1, "version recorded");
}

#[test]
fn parity_write_path_roundtrips_through_load() {
    // create → load()(fresh store, disk as source)→ list/get 行为一致。
    let (_dir, hw_store, ag_store) = temp_wiki();
    let (hw_p, ag_p) = sample_page("roundtrip");
    hw_store.create_page(hw_p).unwrap();
    ag_store.create_page(ag_p).unwrap();

    let hw_store2 =
        hw::WikiStore::new(_dir.path().join("hw").join("wiki"), _dir.path().join("hw").join("raw"));
    hw_store2.load();
    let ag_store2 =
        ag::WikiStore::new(_dir.path().join("ag").join("wiki"), _dir.path().join("ag").join("raw"));
    ag_store2.load();

    let hw_loaded = hw_store2.get_page("roundtrip").unwrap();
    let ag_loaded = ag_store2.get_page("roundtrip").unwrap();
    assert_eq!(ag_loaded.content, hw_loaded.content, "content from disk");
    assert_eq!(ag_loaded.title, hw_loaded.title, "title from manifest");
    assert_eq!(ag_loaded.tags, hw_loaded.tags, "tags from manifest");
    assert_eq!(ag_loaded.version, hw_loaded.version, "version from manifest");
    assert_eq!(ag_store2.list_pages().len(), hw_store2.list_pages().len(), "list parity");

    // Arc 化对比(与 server 端一致的使用方式)。
    let _ = Arc::new(ag_store2);
}
