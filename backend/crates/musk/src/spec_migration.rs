//! Explicit, resumable legacy import. The original ledger is never modified.
use crate::spec_documents::{self as documents, hash};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs, io,
    path::{Component, Path, PathBuf},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Entry {
    pub source: String,
    pub target: String,
    pub disposition: String,
    pub content: String,
    pub content_hash: String,
    pub before_hash: Option<String>,
    #[serde(default)]
    pub original_record: Option<Value>,
    #[serde(default)]
    pub reference: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub policy: String,
    pub source: PathBuf,
    pub source_hash: String,
    pub root: PathBuf,
    pub entries: Vec<Entry>,
    #[serde(default)]
    pub source_key: String,
}
fn err(message: &str) -> io::Error {
    io::Error::other(message)
}
fn json_string(value: &str) -> String {
    serde_json::to_string(value).unwrap()
}

/// Keep the original shape in the envelope; adapt views, not the source JSON.
fn section_views(legacy: &Value) -> io::Result<Vec<(String, Vec<Value>)>> {
    let mut output = Vec::new();
    let sections = legacy
        .get("sections")
        .ok_or_else(|| err("Legacy sections missing"))?;
    let mut push = |name: String, items: &Value| -> io::Result<()> {
        let items = items
            .as_array()
            .ok_or_else(|| err("Legacy items must be an array"))?;
        if items.iter().any(|item| !item.is_object()) {
            return Err(err("Legacy item must be an object"));
        }
        output.push((name, items.clone()));
        Ok(())
    };
    match sections {
        Value::Array(list) => {
            for section in list {
                if !section.is_object() {
                    return Err(err("Legacy section must be an object"));
                }
                push(
                    section["id"].as_str().unwrap_or("unknown").into(),
                    section
                        .get("items")
                        .ok_or_else(|| err("Legacy section items missing"))?,
                )?;
            }
        }
        Value::Object(map) => {
            for (name, value) in map {
                let items = if value.is_array() {
                    value
                } else {
                    value
                        .get("items")
                        .ok_or_else(|| err("Legacy map section items missing"))?
                };
                push(name.clone(), items)?;
            }
        }
        _ => return Err(err("Legacy sections must be an array or object")),
    }
    Ok(output)
}
fn fence(text: &str) -> String {
    let longest = text.split(|c| c != '`').map(str::len).max().unwrap_or(0);
    "`".repeat((longest + 1).max(4))
}
fn markdown_json(value: &Value) -> String {
    let text = serde_json::to_string_pretty(value).unwrap();
    let marker = fence(&text);
    format!("{marker}json\n{text}\n{marker}")
}
/// Read-only links also reject escaped roots and reparse components.
fn reference(root: &Path, raw: &str) -> Option<String> {
    let project = root.file_name()?.to_str()?;
    let raw = raw.replace('\\', "/");
    let raw = raw.strip_prefix(&format!("{project}/")).unwrap_or(&raw);
    let path = raw.split('#').next()?;
    if path.is_empty()
        || path.contains(':')
        || Path::new(path)
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return None;
    }
    let mut cursor = root.to_path_buf();
    for c in Path::new(path).components() {
        cursor.push(c.as_os_str());
        let meta = fs::symlink_metadata(&cursor).ok()?;
        if meta.file_type().is_symlink() {
            return None;
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if meta.file_attributes() & 0x400 != 0 {
                return None;
            }
        }
    }
    if cursor.is_file()
        && cursor
            .canonicalize()
            .ok()?
            .starts_with(root.canonicalize().ok()?)
    {
        Some(raw.into())
    } else {
        None
    }
}
fn archive_reference(root: &Path, id: &str) -> Option<String> {
    let digits = id
        .strip_prefix("PLAN-")
        .or_else(|| id.strip_prefix('P'))?
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>();
    if digits.is_empty() {
        return None;
    }
    let mut matches = Vec::new();
    for folder in ["docs/plans/archive", "docs/plans/archived"] {
        if let Ok(iter) = fs::read_dir(root.join(folder)) {
            for e in iter.flatten() {
                let name = e.file_name().to_string_lossy().to_string();
                if name.starts_with(&format!("{digits}-")) && name.ends_with(".md") {
                    if let Some(path) = reference(root, &format!("{folder}/{name}")) {
                        matches.push(path);
                    }
                }
            }
        }
    }
    if matches.len() == 1 {
        matches.pop()
    } else {
        None
    }
}
pub fn dry_run(root: &Path, source: &Path) -> io::Result<Manifest> {
    dry_run_with_source_key(root, source, "")
}
pub fn dry_run_with_source_key(
    root: &Path,
    source: &Path,
    source_key: &str,
) -> io::Result<Manifest> {
    if !source_key
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(err(
            "Source key must contain only ASCII letters, digits, '-' or '_'",
        ));
    }
    let bytes = fs::read(source)?;
    let source_hash = hash(&bytes);
    let legacy: Value = serde_json::from_slice(&bytes).map_err(|e| {
        err(&format!(
            "Legacy JSON invalid; preserve original bytes and restore a backup: {e}"
        ))
    })?;
    let sections = section_views(&legacy)?;
    let namespace = if source_key.is_empty() {
        source_hash[..16].to_string()
    } else {
        format!("{source_key}-{source_hash}")
    };
    let mut entries = Vec::new();
    let mut envelope = legacy.clone();
    match &mut envelope["sections"] {
        Value::Array(list) => {
            for section in list {
                section["items"] = json!([]);
            }
        }
        Value::Object(map) => {
            for value in map.values_mut() {
                if value.is_array() {
                    *value = json!([]);
                } else {
                    value["items"] = json!([]);
                }
            }
        }
        _ => unreachable!(),
    }
    for (section_pos, (section_id, items)) in sections.iter().enumerate() {
        for (item_pos, item) in items.iter().enumerate() {
            let id = item["id"].as_str().unwrap_or("missing-id");
            let target =
                format!("docs/specs/history/legacy-ledger/{namespace}/{section_pos}-{item_pos}.md");
            let canonical = item["file"].as_str().and_then(|p| reference(root, p));
            let archived = archive_reference(root, id);
            let historical = matches!(section_id.as_str(), "reviews" | "reports")
                || canonical.as_deref().is_some_and(|p| {
                    p.starts_with("docs/plans/archive/") || p.starts_with("docs/plans/archived/")
                });
            let disposition = if historical {
                "historical-preserved"
            } else if canonical.is_some() || archived.is_some() {
                "source-linked-preserved"
            } else {
                "needs-reconciliation"
            };
            // Archived provenance never turns an old statement into today's requirement.
            let project = root.file_name().unwrap_or_default().to_string_lossy();
            let identity =
                format!("legacy:{project}:{source_key}:{source_hash}:{section_pos}:{item_pos}");
            let record_json = markdown_json(item);
            let content = format!("---\nspec_id: {}\nsection: reports\ntags: [historical, legacy-import]\n---\n\n# Legacy Spec: {}\n\nHistorical preservation; current requirements remain in canonical module Specs.\n\n- Source: `{section_id}/{id}` (position {section_pos}/{item_pos})\n- Ledger SHA256: `{source_hash}`\n- Disposition: {disposition}\n- Canonical reference: {}\n- Archived Plan: {}\n\n## Original content\n\n{}\n\n## Original record (all fields, immutable import)\n\n{record_json}\n",
                    json_string(&identity), item["title"].as_str().unwrap_or(id),
                    canonical.as_deref().unwrap_or("unresolved"), archived.as_deref().unwrap_or("unresolved"),
                    item["content"].as_str().unwrap_or("")).replace("\r\n", "\n");
            let before_hash = match fs::read(root.join(&target)) {
                Ok(b) => Some(hash(&b)),
                Err(e) if e.kind() == io::ErrorKind::NotFound => None,
                Err(e) => return Err(e),
            };
            entries.push(Entry {
                source: format!("{section_pos}/{item_pos}:{section_id}/{id}"),
                target,
                disposition: disposition.into(),
                content_hash: hash(content.as_bytes()),
                content,
                before_hash,
                original_record: Some(item.clone()),
                reference: canonical.or(archived),
            });
        }
    }
    let target = format!("docs/specs/history/legacy-ledger/{namespace}/envelope.md");
    let envelope_json = markdown_json(&envelope);
    let content = format!("# Legacy ledger envelope\n\nHistorical metadata, not current requirements. Original source SHA256: `{source_hash}`.\nAll top-level and section fields retained; individual item records are adjacent.\n\n{envelope_json}\n");
    let before_hash = fs::read(root.join(&target)).ok().map(|b| hash(&b));
    entries.push(Entry {
        source: "envelope".into(),
        target,
        disposition: "historical-metadata".into(),
        original_record: Some(envelope),
        reference: None,
        content_hash: hash(content.as_bytes()),
        content,
        before_hash,
    });
    Ok(Manifest {
        policy: if source_key.is_empty() {
            "document-source-v1"
        } else {
            "document-source-v2"
        }
        .into(),
        source: source.canonicalize()?,
        source_hash,
        root: root.canonicalize()?,
        entries,
        source_key: source_key.into(),
    })
}
fn create_exact(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Ok(existing) = fs::read(path) {
        if existing == bytes {
            return Ok(());
        }
        return Err(err(&format!(
            "Recovery artifact already differs: {}",
            path.display()
        )));
    }
    let parent = path
        .parent()
        .ok_or_else(|| err("Recovery path needs a parent"))?;
    fs::create_dir_all(parent)?;
    use std::io::Write;
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    file.write_all(bytes)?;
    file.as_file().sync_all()?;
    match file.persist_noclobber(path) {
        Ok(_) => Ok(()),
        Err(e) if e.error.kind() == io::ErrorKind::AlreadyExists => {
            if fs::read(path)? == bytes {
                Ok(())
            } else {
                Err(err("Concurrent recovery artifact differs"))
            }
        }
        Err(e) => Err(e.error),
    }
}
pub fn write_manifest(manifest: &Manifest, journal: &Path) -> io::Result<()> {
    // Snapshot bytes and digest come from the same read; never publish a
    // usable manifest with a backup from another source revision.
    let source = fs::read(&manifest.source)?;
    if hash(&source) != manifest.source_hash {
        return Err(err("Legacy changed during dry-run; regenerate manifest"));
    }
    let bytes = serde_json::to_vec_pretty(manifest).unwrap();
    create_exact(&journal.with_extension("legacy-backup"), &source)?;
    create_exact(journal, &bytes)?;
    create_exact(
        &journal.with_extension("manifest-sha256"),
        hash(&bytes).as_bytes(),
    )
}
pub fn read_manifest(journal: &Path) -> io::Result<Manifest> {
    let bytes = fs::read(journal)?;
    let manifest: Manifest = serde_json::from_slice(&bytes).map_err(|e| err(&e.to_string()))?;
    if !matches!(
        manifest.policy.as_str(),
        "document-source-v1" | "document-source-v2"
    ) || hash(&fs::read(journal.with_extension("legacy-backup"))?) != manifest.source_hash
    {
        return Err(err(
            "Migration backup/policy mismatch; restore the exact backup or regenerate dry-run",
        ));
    }
    match fs::read_to_string(journal.with_extension("manifest-sha256")) {
        Ok(expected) if expected == hash(&bytes) => {}
        Err(e)
            if e.kind() == io::ErrorKind::NotFound && manifest.policy == "document-source-v1" => {}
        _ => {
            return Err(err(
                "Migration manifest digest mismatch or missing; restore the exact journal",
            ))
        }
    }
    validate_entries(&manifest)?;
    Ok(manifest)
}
fn validate_entries(manifest: &Manifest) -> io::Result<()> {
    let mut targets = std::collections::HashSet::new();
    let namespace = if manifest.policy == "document-source-v2" {
        if manifest.source_key.is_empty()
            || !manifest
                .source_key
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(err("Invalid registered source key"));
        }
        Some(format!(
            "docs/specs/history/legacy-ledger/{}-{}/",
            manifest.source_key, manifest.source_hash
        ))
    } else {
        None
    };
    for entry in &manifest.entries {
        if hash(entry.content.as_bytes()) != entry.content_hash {
            return Err(err("Migration manifest content hash mismatch"));
        }
        if !targets.insert(&entry.target) {
            return Err(err("Duplicate migration target"));
        }
        if let Some(prefix) = &namespace {
            if !entry.target.starts_with(prefix) {
                return Err(err("Migration target does not belong to registered source"));
            }
        }
        let relative = entry
            .target
            .strip_prefix("docs/specs/history/legacy-ledger/")
            .ok_or_else(|| err("Migration target must be in legacy history"))?;
        documents::resolve(&manifest.root, &format!("history/legacy-ledger/{relative}"))?;
    }
    Ok(())
}
fn current_hash(path: &Path) -> io::Result<Option<String>> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(hash(&bytes))),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}
pub fn apply(manifest: &Manifest, stopped: bool) -> io::Result<()> {
    if !stopped {
        return Err(err(
            "Stop/isolate the legacy writer first; pass --legacy-writer-stopped after verifying",
        ));
    }
    validate_entries(manifest)?;
    documents::require_editable(&manifest.root)?;
    let _guard = documents::lock(&manifest.root, "legacy-import")?;
    if hash(&fs::read(&manifest.source)?) != manifest.source_hash {
        return Err(err("Legacy source changed since dry-run"));
    }
    for entry in &manifest.entries {
        if entry.content_hash != hash(entry.content.as_bytes()) {
            return Err(err("Migration manifest content hash mismatch"));
        }
        let path = entry
            .target
            .strip_prefix("docs/specs/history/legacy-ledger/")
            .ok_or_else(|| err("Migration target must be in legacy history"))?;
        let relative = format!("history/legacy-ledger/{path}");
        let target = documents::resolve(&manifest.root, &relative)?;
        let current = current_hash(&target)?;
        if current.as_deref() == Some(&entry.content_hash) {
            continue;
        }
        if current != entry.before_hash {
            return Err(err(&format!("Migration target changed: {}", entry.target)));
        }
        // No existing foreign content is replaced, even when captured by dry-run.
        if current.is_some() {
            return Err(err(&format!(
                "Migration target already exists with different content: {}",
                entry.target
            )));
        }
        documents::save(&manifest.root, &relative, &entry.content, None)?;
    }
    Ok(())
}
pub fn rollback(manifest: &Manifest) -> io::Result<()> {
    validate_entries(manifest)?;
    documents::require_editable(&manifest.root)?;
    let _guard = documents::lock(&manifest.root, "legacy-import")?;
    for entry in manifest.entries.iter().rev() {
        if entry.before_hash.is_some() {
            continue;
        } // never remove pre-existing content
        let suffix = entry
            .target
            .strip_prefix("docs/specs/history/legacy-ledger/")
            .ok_or_else(|| err("Invalid migration target"))?;
        let relative = format!("history/legacy-ledger/{suffix}");
        let target = documents::resolve(&manifest.root, &relative)?;
        let _file_guard =
            documents::lock(&manifest.root, &target.to_string_lossy().to_lowercase())?;
        match fs::read(&target) {
            Ok(bytes) if hash(&bytes) == entry.content_hash => fs::remove_file(&target)?,
            Ok(_) => {
                return Err(err(&format!(
                    "Rollback refused: later edits in {}",
                    entry.target
                )))
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
    }
    Ok(())
}
