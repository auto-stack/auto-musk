//! Stateless document projection: no cache file, no stale generation publication.
use crate::{
    spec_documents::{hash, supported},
    specs::{SpecItem, SpecStatus, SpecsDocument},
};
use serde::Serialize;
use std::{
    collections::{BTreeMap, HashSet},
    fs, io,
    path::Path,
};
use yaml_rust::{Yaml, YamlLoader};

#[derive(Serialize)]
pub struct Projection {
    pub document: SpecsDocument,
    pub source_hash: String,
    pub fresh: bool,
    pub diagnostics: Vec<String>,
}
fn collect(root: &Path, path: &Path, files: &mut BTreeMap<String, Vec<u8>>) -> io::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let meta = fs::symlink_metadata(entry.path())?;
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if meta.file_attributes() & 0x400 != 0 {
                continue;
            }
        }
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            collect(root, &entry.path(), files)?;
        } else if supported(&entry.path()) {
            let rel = entry
                .path()
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            files.insert(rel, fs::read(entry.path())?);
        }
    }
    Ok(())
}
fn snapshot(root: &Path) -> io::Result<BTreeMap<String, Vec<u8>>> {
    let mut files = BTreeMap::new();
    for folder in ["docs/specs", "docs/plans/archived"] {
        // Do not follow an escaped directory even at the top of the traversal.
        let p = root.join(folder);
        if p.exists() && !p.canonicalize()?.starts_with(root.canonicalize()?) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Spec index root escapes workspace",
            ));
        }
        collect(root, &p, &mut files)?;
    }
    Ok(files)
}
pub fn build(root: &Path) -> io::Result<Projection> {
    // A second snapshot detects external changes during construction. Never
    // publish a mixed generation as fresh; the caller may retry the read.
    let files = snapshot(root)?;
    let mut doc = SpecsDocument::new(root.file_name().unwrap_or_default().to_string_lossy());
    let mut diagnostics = Vec::new();
    let mut ids = HashSet::new();
    let mut fingerprint = Vec::new();
    for (path, bytes) in &files {
        fingerprint.extend_from_slice(path.as_bytes());
        fingerprint.push(0);
        fingerprint.extend_from_slice(hash(bytes).as_bytes());
        fingerprint.push(0);
        let text = match std::str::from_utf8(bytes) {
            Ok(s) => s,
            Err(e) => {
                diagnostics.push(format!("{path}: {e}"));
                continue;
            }
        };
        let (preamble, _) = crate::spec_documents::split_source(text);
        let normalized = preamble
            .trim_start_matches('\u{feff}')
            .replace("\r\n", "\n");
        let yaml = if normalized.starts_with("---\n") {
            let lines: Vec<_> = normalized.lines().collect();
            let fm = lines[1..lines.len() - 1].join("\n");
            match YamlLoader::load_from_str(&fm) {
                Ok(v) => v.first().cloned().unwrap_or(Yaml::Null),
                Err(e) => {
                    diagnostics.push(format!("{path}: invalid metadata: {e}"));
                    Yaml::Null
                }
            }
        } else {
            Yaml::Null
        };
        let historical =
            path.starts_with("docs/plans/archived/") || path.starts_with("docs/specs/history/");
        let section = if historical {
            "reports"
        } else {
            yaml["section"].as_str().unwrap_or_else(|| {
                if path.contains("/goals/") {
                    "goals"
                } else if path.contains("architecture") {
                    "architecture"
                } else if path.contains("/tests/") {
                    "tests"
                } else if path.contains("/reviews/") {
                    "reviews"
                } else if path.contains("/reports/") {
                    "reports"
                } else {
                    "designs"
                }
            })
        };
        let id = yaml["spec_id"]
            .as_str()
            .map(str::to_string)
            .unwrap_or_else(|| format!("file:{path}"));
        if !ids.insert(id.clone()) {
            diagnostics.push(format!("duplicate spec_id {id}: {path}"));
            continue;
        }
        let title = yaml["title"]
            .as_str()
            .or_else(|| text.lines().find_map(|l| l.strip_prefix("# ")))
            .unwrap_or(path);
        let mut item = SpecItem::new(id, title);
        item.created_at = 0;
        item.modified_at = 0;
        item.file = Some(path.clone());
        item.content = text.to_string();
        item.tags = yaml["tags"]
            .as_vec()
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        item.tags.push(format!("source:{}", hash(bytes)));
        if historical {
            item.tags.push("historical".into());
        }
        item.depends_on = yaml["depends_on"]
            .as_vec()
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        if let Some(s) = yaml["status"].as_str() {
            if historical {
                // A Plan lifecycle is not a Spec approval state.
                item.tags.push(format!("source_status:{s}"));
            } else if let Some(status) = SpecStatus::parse_exact(s) {
                item.status = status;
            } else {
                diagnostics.push(format!("{path}: unknown status {s}"));
            }
        }
        if let Some(s) = doc.sections.iter_mut().find(|s| s.id == section) {
            s.items.push(item);
        } else {
            diagnostics.push(format!("{path}: unknown section {section}"));
        }
    }
    for section in &mut doc.sections {
        section.last_modified = 0;
    }
    doc.rebuild_relations();
    // Explicit dependencies are also relations; legacy body-ID scanning alone
    // does not understand path-based identities.
    let edges: Vec<_> = doc
        .sections
        .iter()
        .flat_map(|s| &s.items)
        .flat_map(|i| i.depends_on.iter().map(move |d| (i.id.clone(), d.clone())))
        .collect();
    for (from, to) in edges {
        if let Some(target) = doc
            .sections
            .iter_mut()
            .flat_map(|s| &mut s.items)
            .find(|i| i.id == to)
        {
            if !target.related.contains(&from) {
                target.related.push(from);
            }
        } else {
            diagnostics.push(format!("{from}: unresolved dependency {to}"));
        }
    }
    let source_hash = hash(&fingerprint);
    // Compatibility display field fits both VM i32 and JS numbers. Identity
    // and freshness use the full source_hash, not this shortened display value.
    doc.version = u64::from_str_radix(&source_hash[..7], 16).unwrap();
    let fresh = files == snapshot(root)?;
    if !fresh {
        diagnostics.push("Source changed during indexing; retry read".into());
    }
    Ok(Projection {
        document: doc,
        source_hash,
        fresh,
        diagnostics,
    })
}
