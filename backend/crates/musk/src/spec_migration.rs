//! Explicit, resumable legacy import. The original ledger is never modified.
use crate::spec_documents::{self as documents, hash};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Entry {
    pub source: String,
    pub target: String,
    pub disposition: String,
    pub content: String,
    pub content_hash: String,
    pub before_hash: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub policy: String,
    pub source: PathBuf,
    pub source_hash: String,
    pub root: PathBuf,
    pub entries: Vec<Entry>,
}
fn err(message: &str) -> io::Error {
    io::Error::other(message)
}
fn json_string(value: &str) -> String {
    serde_json::to_string(value).unwrap()
}

pub fn dry_run(root: &Path, source: &Path) -> io::Result<Manifest> {
    let bytes = fs::read(source)?;
    let source_hash = hash(&bytes);
    let legacy: Value = serde_json::from_slice(&bytes).map_err(|e| {
        err(&format!(
            "Legacy JSON invalid; preserve original bytes and restore a backup: {e}"
        ))
    })?;
    let sections = legacy["sections"]
        .as_array()
        .ok_or_else(|| err("Legacy sections array missing; source left intact"))?;
    let mut entries = Vec::new();
    let mut envelope = legacy.clone();
    let envelope_sections = envelope["sections"].as_array_mut().unwrap();
    for (section_pos, section) in sections.iter().enumerate() {
        let section_id = section["id"].as_str().unwrap_or("unknown");
        if let Some(items) = section["items"].as_array() {
            envelope_sections[section_pos]["items"] = json!([]);
            for (item_pos, item) in items.iter().enumerate() {
                let id = item["id"].as_str().unwrap_or("missing-id");
                // Position preserves duplicates; source fingerprint isolates unrelated imports.
                let target = format!(
                    "docs/specs/history/legacy-ledger/{}/{section_pos}-{item_pos}.md",
                    &source_hash[..16]
                );
                let explicit_file = item["file"].as_str().filter(|p| {
                    p.starts_with("docs/specs/") || p.starts_with("docs/plans/archived/")
                });
                let canonical = explicit_file.filter(|p| root.join(p).is_file());
                let plan = id
                    .strip_prefix("PLAN-")
                    .and_then(|s| s.get(..3))
                    .filter(|s| s.chars().all(|c| c.is_ascii_digit()));
                let archived = plan.and_then(|seq| {
                    fs::read_dir(root.join("docs/plans/archived"))
                        .ok()
                        .and_then(|iter| {
                            iter.filter_map(Result::ok)
                                .find(|e| {
                                    e.file_name()
                                        .to_string_lossy()
                                        .starts_with(&format!("{seq}-"))
                                })
                                .map(|e| {
                                    format!(
                                        "docs/plans/archived/{}",
                                        e.file_name().to_string_lossy()
                                    )
                                })
                        })
                });
                let historical = matches!(section_id, "reviews" | "reports");
                let disposition = if historical {
                    "historical-preserved"
                } else if canonical.is_some() || archived.is_some() {
                    "source-linked-preserved"
                } else {
                    "needs-reconciliation"
                };
                // Archived provenance never turns an old statement into today's requirement.
                let content = format!("---\nspec_id: {}\nsection: reports\ntags: [historical, legacy-import]\n---\n\n# Legacy Spec: {}\n\nHistorical preservation; current requirements remain in canonical module Specs.\n\n- Source: `{section_id}/{id}` (position {section_pos}/{item_pos})\n- Ledger SHA256: `{source_hash}`\n- Disposition: {disposition}\n- Canonical reference: {}\n- Archived Plan: {}\n\n## Original content\n\n{}\n\n## Original record (all fields, immutable import)\n\n````json\n{}\n````\n",
                    json_string(&format!("legacy:{source_hash}:{section_pos}:{item_pos}")),
                    item["title"].as_str().unwrap_or(id), canonical.unwrap_or("unresolved"), archived.as_deref().unwrap_or("unresolved"),
                    item["content"].as_str().unwrap_or(""), serde_json::to_string_pretty(item).unwrap());
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
                });
            }
        }
    }
    let target = format!(
        "docs/specs/history/legacy-ledger/{}/envelope.md",
        &source_hash[..16]
    );
    let content = format!("# Legacy ledger envelope\n\nHistorical metadata, not current requirements. Original source SHA256: `{source_hash}`.\nAll top-level and section fields retained; individual item records are adjacent.\n\n````json\n{}\n````\n", serde_json::to_string_pretty(&envelope).unwrap());
    let before_hash = fs::read(root.join(&target)).ok().map(|b| hash(&b));
    entries.push(Entry {
        source: "envelope".into(),
        target,
        disposition: "historical-metadata".into(),
        content_hash: hash(content.as_bytes()),
        content,
        before_hash,
    });
    Ok(Manifest {
        policy: "document-source-v1".into(),
        source: source.canonicalize()?,
        source_hash,
        root: root.canonicalize()?,
        entries,
    })
}
pub fn write_manifest(manifest: &Manifest, journal: &Path) -> io::Result<()> {
    if let Some(parent) = journal.parent() {
        fs::create_dir_all(parent)?;
    }
    // Never overwrite an earlier recovery manifest or its backup.
    let bytes = serde_json::to_vec_pretty(manifest).unwrap();
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(journal)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    let backup = journal.with_extension("legacy-backup");
    let source = fs::read(&manifest.source)?;
    if hash(&source) != manifest.source_hash {
        return Err(err("Legacy changed during dry-run; regenerate manifest"));
    }
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(backup)?;
    file.write_all(&source)?;
    file.sync_all()
}
pub fn read_manifest(journal: &Path) -> io::Result<Manifest> {
    let manifest: Manifest =
        serde_json::from_slice(&fs::read(journal)?).map_err(|e| err(&e.to_string()))?;
    if manifest.policy != "document-source-v1"
        || hash(&fs::read(journal.with_extension("legacy-backup"))?) != manifest.source_hash
    {
        return Err(err(
            "Migration backup/policy mismatch; restore the exact backup or regenerate dry-run",
        ));
    }
    Ok(manifest)
}
pub fn apply(manifest: &Manifest, stopped: bool) -> io::Result<()> {
    if !stopped {
        return Err(err(
            "Stop/isolate the legacy writer first; pass --legacy-writer-stopped after verifying",
        ));
    }
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
        let current = fs::read(&target).ok().map(|b| hash(&b));
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
