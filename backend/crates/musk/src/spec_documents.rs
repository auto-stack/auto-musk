//! Canonical Spec documents. Locks protect compare-and-save, not the index.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io,
    path::{Component, Path, PathBuf},
};

pub fn hash(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
pub fn supported(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("md" | "markdown" | "ad")
    )
}
fn fail(kind: io::ErrorKind, message: &str) -> io::Error {
    io::Error::new(kind, message)
}

/// Resolve existing parents too: a missing leaf must not bypass confinement.
pub fn resolve(root: &Path, relative: &str) -> io::Result<PathBuf> {
    if relative.is_empty()
        || relative.contains('\\')
        || relative.contains(':')
        || Path::new(relative)
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(fail(
            io::ErrorKind::PermissionDenied,
            "Spec path must be relative to docs/specs",
        ));
    }
    let base = root.join("docs/specs");
    let canonical_root = root.canonicalize()?;
    // Reject links at every existing component, including docs/specs itself.
    let mut cursor = root.to_path_buf();
    for component in Path::new("docs/specs").join(relative).components() {
        cursor.push(component.as_os_str());
        match fs::symlink_metadata(&cursor) {
            Ok(meta) => {
                if meta.file_type().is_symlink()
                    || is_reparse(&meta)
                    || !cursor.canonicalize()?.starts_with(&canonical_root)
                {
                    return Err(fail(
                        io::ErrorKind::PermissionDenied,
                        "Linked Spec paths are not writable",
                    ));
                }
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
    }
    Ok(base.join(relative))
}
#[cfg(windows)]
fn is_reparse(meta: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    meta.file_attributes() & 0x400 != 0
}
#[cfg(not(windows))]
fn is_reparse(_: &fs::Metadata) -> bool {
    false
}

pub fn require_editable(root: &Path) -> io::Result<()> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(root)
        .output()?;
    if out.status.success() {
        let top = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim());
        if !top.join(".git").is_file() {
            return Err(fail(
                io::ErrorKind::PermissionDenied,
                "Switch to a dedicated Git worktree to edit Specs",
            ));
        }
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    pub content: String,
    pub body: String,
    pub preamble: String,
    pub hash: String,
    pub editable: bool,
    pub error: String,
}
/// Keep authored metadata byte-for-byte outside the rich-text serializer.
pub fn split_source(source: &str) -> (&str, &str) {
    let (bom, text) = if let Some(text) = source.strip_prefix('\u{feff}') {
        (&source[..3], text)
    } else {
        ("", source)
    };
    let mut lines = text.split_inclusive('\n');
    let Some(first) = lines.next() else {
        return (bom, text);
    };
    if first.trim_end_matches(['\r', '\n']) != "---" {
        return (bom, text);
    }
    let mut offset = bom.len() + first.len();
    for line in lines {
        offset += line.len();
        if line.trim_end_matches(['\r', '\n']) == "---" {
            return source.split_at(offset);
        }
    }
    (bom, text)
}
pub fn read(root: &Path, relative: &str) -> io::Result<Document> {
    let path = resolve(root, relative)?;
    let bytes = fs::read(&path)?;
    let content = String::from_utf8(bytes.clone())
        .map_err(|e| fail(io::ErrorKind::InvalidData, &e.to_string()))?;
    let (preamble, body) = split_source(&content);
    Ok(Document {
        body: body.into(),
        preamble: preamble.into(),
        content,
        hash: hash(&bytes),
        editable: supported(&path) && require_editable(root).is_ok(),
        error: String::new(),
    })
}

/// OS advisory lock releases on process exit. Lock files are never unlinked:
/// unlinking a lock inode would let a third process lock a different inode.
pub fn lock(root: &Path, key: &str) -> io::Result<File> {
    let dir = root.join(".autoos/spec-locks");
    for path in [root.join(".autoos"), dir.clone()] {
        if let Ok(meta) = fs::symlink_metadata(&path) {
            if meta.file_type().is_symlink() || is_reparse(&meta) {
                return Err(fail(
                    io::ErrorKind::PermissionDenied,
                    "Linked Spec lock directory rejected",
                ));
            }
        }
    }
    fs::create_dir_all(&dir)?;
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(dir.join(hash(key.as_bytes())))?;
    file.lock()?;
    Ok(file)
}
pub fn save(
    root: &Path,
    relative: &str,
    content: &str,
    expected: Option<&str>,
) -> io::Result<Document> {
    require_editable(root)?;
    let path = resolve(root, relative)?;
    if !supported(&path) {
        return Err(fail(
            io::ErrorKind::InvalidInput,
            "Unsupported Spec format (md/markdown/ad)",
        ));
    }
    // Canonical, case-folded path makes aliases share a lock on Windows.
    let key = path.to_string_lossy().to_lowercase();
    let _guard = lock(root, &key)?;
    let path = resolve(root, relative)?;
    let previous = match fs::read(&path) {
        Ok(bytes) => Some(hash(&bytes)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => None,
        Err(e) => return Err(e),
    };
    if previous.as_deref() != expected {
        return Err(fail(
            io::ErrorKind::AlreadyExists,
            "Spec changed since it was opened; reload before saving (draft retained)",
        ));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    auto_lang::state_file::atomic_write(&path, content.as_bytes())?;
    read(root, relative)
}
