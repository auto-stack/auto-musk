use musk::{spec_documents as docs, spec_index, spec_migration as migration};
use std::{
    fs,
    path::Path,
    process::Command,
    sync::{Arc, Barrier},
};

fn git(root: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
#[test]
fn same_file_cas_and_independent_files() {
    let temp = tempfile::tempdir().unwrap();
    let d = docs::save(temp.path(), "a.md", "original", None).unwrap();
    let barrier = Arc::new(Barrier::new(3));
    let handles: Vec<_> = ["one", "two"]
        .into_iter()
        .map(|text| {
            let barrier = barrier.clone();
            let root = temp.path().to_path_buf();
            let expected = d.hash.clone();
            std::thread::spawn(move || {
                barrier.wait();
                docs::save(&root, "a.md", text, Some(&expected))
            })
        })
        .collect();
    barrier.wait();
    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(
                |r| r.as_ref().err().map(|e| e.kind()) == Some(std::io::ErrorKind::AlreadyExists)
            )
            .count(),
        1
    );
    let a = temp.path().to_path_buf();
    let b = a.clone();
    let first = std::thread::spawn(move || docs::save(&a, "one.ad", "one", None));
    let second = std::thread::spawn(move || docs::save(&b, "two.markdown", "two", None));
    assert!(first.join().unwrap().is_ok());
    assert!(second.join().unwrap().is_ok());
}

#[test]
fn process_save_worker() {
    if let Ok(root) = std::env::var("SPEC_CAS_ROOT") {
        let result = docs::save(
            Path::new(&root),
            "a.md",
            &std::env::var("SPEC_CAS_TEXT").unwrap(),
            Some(&std::env::var("SPEC_CAS_HASH").unwrap()),
        );
        if let Err(e) = result {
            assert_eq!(e.kind(), std::io::ErrorKind::AlreadyExists);
            std::process::exit(23);
        }
    }
}
#[test]
fn cross_process_compare_and_save() {
    let temp = tempfile::tempdir().unwrap();
    let initial = docs::save(temp.path(), "a.md", "old", None).unwrap();
    let mut children: Vec<_> = ["one", "two"]
        .into_iter()
        .map(|text| {
            Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "process_save_worker", "--nocapture"])
                .env("SPEC_CAS_ROOT", temp.path())
                .env("SPEC_CAS_TEXT", text)
                .env("SPEC_CAS_HASH", &initial.hash)
                .spawn()
                .unwrap()
        })
        .collect();
    let codes: Vec<_> = children
        .iter_mut()
        .map(|c| c.wait().unwrap().code().unwrap())
        .collect();
    assert_eq!(codes.iter().filter(|&&c| c == 0).count(), 1);
    assert_eq!(codes.iter().filter(|&&c| c == 23).count(), 1);
}
#[test]
fn main_checkout_denied_worktree_allowed_and_paths_confined() {
    let temp = tempfile::tempdir().unwrap();
    git(temp.path(), &["init", "-b", "main"]);
    fs::write(temp.path().join("seed"), "x").unwrap();
    git(temp.path(), &["add", "seed"]);
    git(
        temp.path(),
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "commit",
            "-m",
            "seed",
        ],
    );
    assert_eq!(
        docs::save(temp.path(), "a.md", "x", None)
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::PermissionDenied
    );
    let wt = temp.path().join("worktree");
    git(
        temp.path(),
        &["worktree", "add", "-b", "test-dev", wt.to_str().unwrap()],
    );
    docs::save(&wt, "nested/a.md", "accepted", None).unwrap();
    for path in [
        "../escape.md",
        "/absolute.md",
        "D:/escape.md",
        "a/../../escape.md",
        "..\\escape.md",
    ] {
        assert!(docs::save(&wt, path, "x", None).is_err(), "{path}");
    }
}
#[test]
fn automatic_index_offline_changes_identity_history_and_diagnostics() {
    let temp = tempfile::tempdir().unwrap();
    docs::save(
        temp.path(),
        "a.md",
        "---\nspec_id: stable\nsection: designs\nstatus: draft\n---\n# A",
        None,
    )
    .unwrap();
    let first = spec_index::build(temp.path()).unwrap();
    assert!(first.fresh);
    fs::rename(
        temp.path().join("docs/specs/a.md"),
        temp.path().join("docs/specs/b.md"),
    )
    .unwrap();
    let renamed = spec_index::build(temp.path()).unwrap();
    assert_ne!(renamed.source_hash, first.source_hash);
    assert_eq!(renamed.document.sections[2].items[0].id, "stable");
    docs::save(
        temp.path(),
        "duplicate.md",
        "---\nspec_id: stable\n---\n# Duplicate",
        None,
    )
    .unwrap();
    let duplicate = spec_index::build(temp.path()).unwrap();
    assert!(duplicate
        .diagnostics
        .iter()
        .any(|s| s.contains("duplicate")));
    fs::remove_file(temp.path().join("docs/specs/duplicate.md")).unwrap();
    fs::remove_file(temp.path().join("docs/specs/b.md")).unwrap();
    fs::create_dir_all(temp.path().join("docs/plans/archived")).unwrap();
    fs::write(
        temp.path().join("docs/plans/archived/001-example.md"),
        "---\nstatus: archived\n---\n# Old Plan",
    )
    .unwrap();
    let deleted = spec_index::build(temp.path()).unwrap();
    assert!(deleted.document.sections[2].items.is_empty());
    assert!(deleted.diagnostics.is_empty(), "{:?}", deleted.diagnostics);
    assert!(deleted.document.version <= i32::MAX as u64);
    assert_eq!(deleted.document.sections[5].items.len(), 1);
    assert!(deleted.document.sections[5].items[0]
        .tags
        .iter()
        .any(|s| s == "historical"));
    // Cache/legacy bytes cannot poison source reconstruction.
    fs::create_dir_all(temp.path().join(".autoos")).unwrap();
    fs::write(temp.path().join(".autoos/specs.json"), "broken").unwrap();
    assert_eq!(
        spec_index::build(temp.path()).unwrap().source_hash,
        deleted.source_hash
    );
}
fn migration_source(root: &Path) -> std::path::PathBuf {
    let source = root.join("legacy.json");
    fs::write(&source,r#"{"project":"old","extra":{"keep":42},"sections":[{"id":"designs","notes":"retain","items":[{"id":"same","title":"A","content":"unique knowledge","custom":{"x":1}},{"id":"same","status":"FutureStatus","content":"B"}]},{"id":"reviews","items":[{"id":"review","content":"old verdict"}]}]}"#).unwrap();
    source
}
#[test]
fn migration_lossless_idempotent_conflict_resume_and_rollback() {
    let temp = tempfile::tempdir().unwrap();
    let source = migration_source(temp.path());
    let original = fs::read(&source).unwrap();
    let manifest = migration::dry_run(temp.path(), &source).unwrap();
    assert_eq!(manifest.entries.len(), 4);
    assert!(!temp.path().join("docs").exists());
    assert!(migration::apply(&manifest, false).is_err());
    // Simulated partial application: recovery recognizes its exact output.
    let first = &manifest.entries[0];
    docs::save(
        temp.path(),
        first.target.strip_prefix("docs/specs/").unwrap(),
        &first.content,
        None,
    )
    .unwrap();
    migration::apply(&manifest, true).unwrap();
    migration::apply(&manifest, true).unwrap();
    for entry in &manifest.entries {
        assert_eq!(
            docs::hash(&fs::read(temp.path().join(&entry.target)).unwrap()),
            entry.content_hash
        );
    }
    assert!(manifest.entries[0].content.contains("custom"));
    assert!(manifest.entries[1].content.contains("FutureStatus"));
    assert!(manifest.entries[3].content.contains("retain"));
    assert!(manifest.entries[3].content.contains("keep"));
    fs::write(temp.path().join(&first.target), "subsequent user edit").unwrap();
    assert!(migration::apply(&manifest, true).is_err());
    assert!(migration::rollback(&manifest).is_err());
    assert_eq!(
        fs::read_to_string(temp.path().join(&first.target)).unwrap(),
        "subsequent user edit"
    );
    assert_eq!(fs::read(&source).unwrap(), original);
}
#[test]
fn migration_source_change_corruption_and_clean_rollback() {
    let temp = tempfile::tempdir().unwrap();
    let source = migration_source(temp.path());
    let original = fs::read(&source).unwrap();
    let manifest = migration::dry_run(temp.path(), &source).unwrap();
    fs::write(&source, "corrupt").unwrap();
    assert!(migration::dry_run(temp.path(), &source).is_err());
    assert!(migration::apply(&manifest, true).is_err());
    fs::write(&source, &original).unwrap();
    migration::apply(&manifest, true).unwrap();
    migration::rollback(&manifest).unwrap();
    for entry in &manifest.entries {
        assert!(!temp.path().join(&entry.target).exists());
    }
    assert_eq!(fs::read(&source).unwrap(), original);
}

#[test]
fn rich_editor_body_roundtrip_keeps_crlf_metadata_verbatim() {
    let temp = tempfile::tempdir().unwrap();
    let prefix = "\u{feff}---\r\nspec_id: stable\r\nunknown: {nested: 42}\r\n---\r\n";
    let original = docs::save(
        temp.path(),
        "a.md",
        &(prefix.to_string() + "# Original\r\n"),
        None,
    )
    .unwrap();
    assert_eq!(original.preamble, prefix);
    let edited = docs::save(
        temp.path(),
        "a.md",
        &(original.preamble + "# Edited\n"),
        Some(&original.hash),
    )
    .unwrap();
    assert!(edited.content.starts_with(prefix));
    assert_eq!(edited.body, "# Edited\n");
}

#[test]
fn migration_requires_exact_backup_at_the_cli_manifest_boundary() {
    let temp = tempfile::tempdir().unwrap();
    let source = migration_source(temp.path());
    let manifest = migration::dry_run(temp.path(), &source).unwrap();
    let journal = temp.path().join("journal.json");
    migration::write_manifest(&manifest, &journal).unwrap();
    migration::read_manifest(&journal).unwrap();
    fs::write(journal.with_extension("legacy-backup"), "wrong backup").unwrap();
    assert!(migration::read_manifest(&journal).is_err());
    assert!(!temp.path().join("docs/specs").exists());
}

#[tokio::test]
async fn workspace_tools_read_sources_and_reject_retired_writes() {
    use auto_ai_agent::Tool;
    let temp = tempfile::tempdir().unwrap();
    docs::save(
        temp.path(),
        "tools.md",
        "# Source-owned tool knowledge\n",
        None,
    )
    .unwrap();
    let store = Arc::new(musk::specs::SpecsStore::for_documents(
        temp.path().to_path_buf(),
    ));
    assert!(musk::spec_tools::ReadSpecs::with_store(store.clone())
        .execute(&serde_json::json!({}))
        .await
        .is_ok());
    let writers: Vec<Box<dyn Tool>> = vec![
        Box::new(musk::spec_tools::WriteSpec::with_store(store.clone())),
        Box::new(musk::spec_tools::UpdateSpec::with_store(store.clone())),
        Box::new(musk::spec_tools::WriteGoals::with_store(store)),
    ];
    for tool in writers {
        let error = tool
            .execute(&serde_json::json!({}))
            .await
            .err()
            .expect("retired tool fails before mutation");
        assert!(error.to_string().contains("retired"));
    }
    assert!(!temp.path().join(".autoos/specs.json").exists());
}

#[cfg(windows)]
#[test]
fn linked_source_and_lock_parents_do_not_escape_the_workspace() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("workspace");
    let external = fixture.path().join("external");
    fs::create_dir_all(root.join("docs/specs")).unwrap();
    fs::create_dir_all(&external).unwrap();
    fs::write(external.join("sentinel"), "unchanged").unwrap();
    let link = root.join("docs/specs/linked");
    let output = Command::new("cmd")
        .args(["/c", "mklink", "/J"])
        .arg(link.to_str().unwrap().replace('/', "\\"))
        .arg(external.to_str().unwrap().replace('/', "\\"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(docs::save(&root, "linked/escape.md", "denied", None).is_err());
    let output = Command::new("cmd")
        .args(["/c", "mklink", "/J"])
        .arg(root.join(".autoos").to_str().unwrap().replace('/', "\\"))
        .arg(external.to_str().unwrap().replace('/', "\\"))
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(docs::save(&root, "ordinary.md", "denied", None).is_err());
    assert_eq!(
        fs::read_to_string(external.join("sentinel")).unwrap(),
        "unchanged"
    );
    assert!(!external.join("escape.md").exists());
    fs::remove_dir(&link).unwrap();
    fs::remove_dir(root.join(".autoos")).unwrap();
}
