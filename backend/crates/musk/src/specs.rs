//! Spec Ledger — the data model layer (ported from `src/back/specs.at`).
//!
//! The canonical source of truth for spec data is auto-forge's
//! `backend/src/forge/mod.rs`; this is a Rust port of the `.at` model that
//! lived in this repo. Range: pure data model (enums/structs + string
//! conversions + factories). State-machine, parsing, and persistence are later
//! phases.
//!
//! Port notes vs. the `.at` version:
//! - The AutoVM "enum methods aren't called" hack is GONE — Rust enum methods
//!   work normally, so `to_str`/`from_str`/`as_str` are real `impl` methods
//!   (not module-level functions).
//! - 23 Status variants retained (serialization compatibility).
//! - `related` is a derived field (filled by `rebuild_relations`, later).

use serde::{Deserialize, Serialize};

// ============================================================
// SectionType — the 7 spec-section kinds
// ============================================================

/// The 7 kinds of spec section. Serializes as snake_case.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SectionType {
    Goals,
    Architecture,
    Designs,
    Plans,
    Tests,
    Reviews,
    Reports,
}

impl SectionType {
    /// → snake_case string id (e.g. "goals", "architecture").
    pub fn as_str(self) -> &'static str {
        match self {
            SectionType::Goals => "goals",
            SectionType::Architecture => "architecture",
            SectionType::Designs => "designs",
            SectionType::Plans => "plans",
            SectionType::Tests => "tests",
            SectionType::Reviews => "reviews",
            SectionType::Reports => "reports",
        }
    }

    /// → display title with emoji (for `.ad` file `# <title>` headers).
    pub fn display_title(self) -> &'static str {
        match self {
            SectionType::Goals => "🎯 Goals",
            SectionType::Architecture => "🏗️ Architecture",
            SectionType::Designs => "🎨 Designs",
            SectionType::Plans => "📋 Plans",
            SectionType::Tests => "🧪 Tests",
            SectionType::Reviews => "🔍 Reviews",
            SectionType::Reports => "📊 Reports",
        }
    }

    /// Parse from a section id (file-name stem). Unknown → Goals
    /// (matches auto-forge `mod.rs:106-116`).
    pub fn from_id(id: &str) -> Self {
        match id {
            "goals" => SectionType::Goals,
            "architecture" => SectionType::Architecture,
            "designs" => SectionType::Designs,
            "plans" => SectionType::Plans,
            "tests" => SectionType::Tests,
            "reviews" => SectionType::Reviews,
            "reports" => SectionType::Reports,
            _ => SectionType::Goals,
        }
    }
}

// ============================================================
// SpecStatus — spec item/section lifecycle (23 variants)
// ============================================================

/// Lifecycle status of a spec item or section. Full 23-variant set retained
/// for serialization compatibility; the state machine (later) uses ~17.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpecStatus {
    Empty,
    Proposed,
    Draft,
    UnderReview,
    Approved,
    InProgress,
    InImplementation,
    Implemented,
    Verified,
    Done,
    Archived,
    Rejected,
    Backlog,
    Ready,
    InReview,
    Blocked,
    Superseded,
    Outdated,
    Stable,
    Deprecated,
    Published,
    Analysed,
    Obsolete,
}

impl SpecStatus {
    /// → snake_case string.
    pub fn to_str(self) -> &'static str {
        match self {
            SpecStatus::Empty => "empty",
            SpecStatus::Proposed => "proposed",
            SpecStatus::Draft => "draft",
            SpecStatus::UnderReview => "under_review",
            SpecStatus::Approved => "approved",
            SpecStatus::InProgress => "in_progress",
            SpecStatus::InImplementation => "in_implementation",
            SpecStatus::Implemented => "implemented",
            SpecStatus::Verified => "verified",
            SpecStatus::Done => "done",
            SpecStatus::Archived => "archived",
            SpecStatus::Rejected => "rejected",
            SpecStatus::Backlog => "backlog",
            SpecStatus::Ready => "ready",
            SpecStatus::InReview => "in_review",
            SpecStatus::Blocked => "blocked",
            SpecStatus::Superseded => "superseded",
            SpecStatus::Outdated => "outdated",
            SpecStatus::Stable => "stable",
            SpecStatus::Deprecated => "deprecated",
            SpecStatus::Published => "published",
            SpecStatus::Analysed => "analysed",
            SpecStatus::Obsolete => "obsolete",
        }
    }

    /// Parse from a string. Unknown → Draft (lossy, matches auto-forge).
    pub fn from_str_lossy(s: &str) -> Self {
        match s {
            "empty" => SpecStatus::Empty,
            "proposed" => SpecStatus::Proposed,
            "draft" => SpecStatus::Draft,
            "under_review" => SpecStatus::UnderReview,
            "approved" => SpecStatus::Approved,
            "in_progress" => SpecStatus::InProgress,
            "in_implementation" => SpecStatus::InImplementation,
            "implemented" => SpecStatus::Implemented,
            "verified" => SpecStatus::Verified,
            "done" => SpecStatus::Done,
            "archived" => SpecStatus::Archived,
            "rejected" => SpecStatus::Rejected,
            "backlog" => SpecStatus::Backlog,
            "ready" => SpecStatus::Ready,
            "in_review" => SpecStatus::InReview,
            "blocked" => SpecStatus::Blocked,
            "superseded" => SpecStatus::Superseded,
            "outdated" => SpecStatus::Outdated,
            "stable" => SpecStatus::Stable,
            "deprecated" => SpecStatus::Deprecated,
            "published" => SpecStatus::Published,
            "analysed" => SpecStatus::Analysed,
            "obsolete" => SpecStatus::Obsolete,
            _ => SpecStatus::Draft,
        }
    }
}

// ============================================================
// SpecItem — the minimal spec unit
// ============================================================

/// The minimal spec unit (one Goal, one Architecture decision, …).
///
/// - `id`: type-prefixed number (G1, A3, S1.1).
/// - `depends_on`: manually-declared forward deps (item IDs).
/// - `related`: derived reverse-links (IDs of items referencing this one).
/// - `tags`: label list.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpecItem {
    pub id: String,
    pub title: String,
    pub content: String,
    pub status: SpecStatus,
    pub depends_on: Vec<String>,
    pub related: Vec<String>,
    pub priority: Option<String>,
    pub assignee: Option<String>,
    pub test_file: Option<String>,
    pub file: Option<String>,
    pub milestone: Option<String>,
    pub module: Option<String>,
    pub tags: Vec<String>,
    pub created_at: u64,
    pub modified_at: u64,
    pub completed_at: Option<u64>,
}

impl SpecItem {
    /// Factory: create with id + title, status Empty, timestamps = now.
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        let now = now_sec();
        Self {
            id: id.into(),
            title: title.into(),
            content: String::new(),
            status: SpecStatus::Empty,
            depends_on: Vec::new(),
            related: Vec::new(),
            priority: None,
            assignee: None,
            test_file: None,
            file: None,
            milestone: None,
            module: None,
            tags: Vec::new(),
            created_at: now,
            modified_at: now,
            completed_at: None,
        }
    }
}

// ============================================================
// SpecsSection — a container for one of the 7 section kinds
// ============================================================

/// One section container (e.g. all Goals).
///
/// - `status`: section-level aggregate status (derived, later).
/// - `content`: legacy whole-section text (kept for migration).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpecsSection {
    pub id: String,
    pub section_type: SectionType,
    pub title: String,
    pub items: Vec<SpecItem>,
    pub status: SpecStatus,
    pub content: String,
    pub depends_on: Vec<String>,
    pub last_modified: u64,
    pub last_verified: Option<u64>,
}

impl SpecsSection {
    /// Factory: empty section of the given type (title = display_title).
    pub fn new(st: SectionType) -> Self {
        Self {
            id: st.as_str().to_string(),
            section_type: st,
            title: st.display_title().to_string(),
            items: Vec::new(),
            status: SpecStatus::Empty,
            content: String::new(),
            depends_on: Vec::new(),
            last_modified: now_sec(),
            last_verified: None,
        }
    }
}

// ============================================================
// SpecsDocument — the root (Document → Section[] → Item[])
// ============================================================

/// Spec document root. `version` increments on each upsert/delete
/// (persistence layer, later).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpecsDocument {
    pub project: String,
    pub version: u64,
    pub sections: Vec<SpecsSection>,
}

impl SpecsDocument {
    /// Factory: a document with all 7 empty sections.
    pub fn new(project: impl Into<String>) -> Self {
        Self {
            project: project.into(),
            version: 0,
            sections: vec![
                SpecsSection::new(SectionType::Goals),
                SpecsSection::new(SectionType::Architecture),
                SpecsSection::new(SectionType::Designs),
                SpecsSection::new(SectionType::Plans),
                SpecsSection::new(SectionType::Tests),
                SpecsSection::new(SectionType::Reviews),
                SpecsSection::new(SectionType::Reports),
            ],
        }
    }
}

// ============================================================
// SpecChange — a pending spec change (for LLM-proposed edits)
// ============================================================

/// A pending spec change proposed by the LLM, awaiting approval.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpecChange {
    pub section_id: String,
    pub item_id: String,
    pub title: Option<String>,
    pub content: Option<String>,
    pub status: Option<SpecStatus>,
    pub reason: String,
}

// ============================================================
// helpers
// ============================================================

/// Current unix timestamp in seconds (stand-in for the `.at`'s `now_sec`).
pub fn now_sec() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// ============================================================
// State machine — valid status transitions
// ============================================================

/// Is the transition `from -> to` valid? Models the canonical spec lifecycle:
/// empty → proposed → draft → review → approved → implemented → verified → done,
/// with rejection/archive/done side-branches at sensible points.
///
/// Unknown "back to draft" is allowed from most states (specs get reopened).
pub fn can_transition(from: SpecStatus, to: SpecStatus) -> bool {
    use SpecStatus::*;
    if from == to {
        return true; // idempotent
    }
    match from {
        Empty => matches!(to, Proposed | Draft | Backlog),
        Proposed => matches!(to, Draft | Backlog | Rejected),
        Draft => matches!(to, UnderReview | InReview | Approved | Backlog),
        UnderReview | InReview => {
            matches!(to, Approved | Rejected | Draft | Blocked)
        }
        Approved => matches!(to, Ready | InProgress | InImplementation),
        Ready => matches!(to, InProgress | InImplementation | Backlog),
        InProgress | InImplementation => matches!(to, Implemented | Blocked),
        Implemented => matches!(to, Verified | InProgress),
        Verified => matches!(to, Done | Archived),
        Done => matches!(to, Archived | Draft),
        Backlog => matches!(to, Ready | Proposed | Draft),
        Blocked => matches!(to, InProgress | InReview | Backlog | Rejected),
        // terminal-ish states: allow reopen-to-draft/backlog for correction,
        // or proceed to Archived.
        Archived | Rejected | Superseded | Outdated | Stable | Deprecated
        | Published | Analysed | Obsolete => matches!(to, Draft | Backlog | Archived),
    }
}

/// Transition a status, returning the new status or an error if invalid.
pub fn transition(from: SpecStatus, to: SpecStatus) -> Result<SpecStatus, String> {
    if can_transition(from, to) {
        Ok(to)
    } else {
        Err(format!(
            "invalid status transition: {} -> {}",
            from.to_str(),
            to.to_str()
        ))
    }
}

// ============================================================
// SpecsStore — JSON file persistence + CRUD
// ============================================================

/// A file-backed spec store. The whole document is serialized to one JSON file
/// at `path`. Good enough for single-user; a real DB is a later phase.
pub struct SpecsStore {
    path: std::path::PathBuf,
}

impl SpecsStore {
    /// Open (or create) a store at `path`.
    pub fn new(path: impl Into<std::path::PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Load the document; create an empty one (persisted) if absent.
    pub fn load(&self) -> std::io::Result<SpecsDocument> {
        match std::fs::read(&self.path) {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, e)
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let doc = SpecsDocument::new(
                    self.path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("project"),
                );
                self.save(&doc)?;
                Ok(doc)
            }
            Err(e) => Err(e),
        }
    }

    /// Persist the document.
    pub fn save(&self, doc: &SpecsDocument) -> std::io::Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let bytes = serde_json::to_vec_pretty(doc)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(&self.path, bytes)
    }

    /// Upsert an item into a section, bumping the document version. Creates the
    /// item if `item.id` is new, replaces it otherwise.
    pub fn upsert_item(
        &self,
        doc: &mut SpecsDocument,
        section_id: &str,
        item: SpecItem,
    ) -> Result<(), String> {
        let section = doc
            .sections
            .iter_mut()
            .find(|s| s.id == section_id)
            .ok_or_else(|| format!("section '{section_id}' not found"))?;
        let now = now_sec();
        if let Some(existing) = section.items.iter_mut().find(|i| i.id == item.id) {
            *existing = item;
        } else {
            section.items.push(item);
        }
        section.last_modified = now;
        doc.version += 1;
        Ok(())
    }

    /// Transition an item's status (validates via the state machine).
    pub fn transition_item(
        &self,
        doc: &mut SpecsDocument,
        section_id: &str,
        item_id: &str,
        new_status: SpecStatus,
    ) -> Result<(), String> {
        let section = doc
            .sections
            .iter_mut()
            .find(|s| s.id == section_id)
            .ok_or_else(|| format!("section '{section_id}' not found"))?;
        let item = section
            .items
            .iter_mut()
            .find(|i| i.id == item_id)
            .ok_or_else(|| format!("item '{item_id}' not found in '{section_id}'"))?;
        let updated = transition(item.status, new_status)?;
        item.status = updated;
        item.modified_at = now_sec();
        if matches!(updated, SpecStatus::Done) {
            item.completed_at = Some(now_sec());
        }
        section.last_modified = now_sec();
        doc.version += 1;
        Ok(())
    }

    /// Delete an item. Returns true if it existed.
    pub fn delete_item(
        &self,
        doc: &mut SpecsDocument,
        section_id: &str,
        item_id: &str,
    ) -> Result<bool, String> {
        let section = doc
            .sections
            .iter_mut()
            .find(|s| s.id == section_id)
            .ok_or_else(|| format!("section '{section_id}' not found"))?;
        let before = section.items.len();
        section.items.retain(|i| i.id != item_id);
        let removed = section.items.len() < before;
        if removed {
            section.last_modified = now_sec();
            doc.version += 1;
        }
        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn section_type_roundtrip() {
        for st in [
            SectionType::Goals,
            SectionType::Architecture,
            SectionType::Designs,
            SectionType::Plans,
            SectionType::Tests,
            SectionType::Reviews,
            SectionType::Reports,
        ] {
            assert_eq!(SectionType::from_id(st.as_str()), st);
        }
    }

    #[test]
    fn section_type_unknown_id_falls_back_to_goals() {
        assert_eq!(SectionType::from_id("nonsense"), SectionType::Goals);
    }

    #[test]
    fn section_type_display_titles_have_emoji() {
        assert!(SectionType::Goals.display_title().contains("🎯"));
        assert!(SectionType::Tests.display_title().contains("🧪"));
    }

    #[test]
    fn status_roundtrip_all_23() {
        let all = [
            SpecStatus::Empty,
            SpecStatus::Proposed,
            SpecStatus::Draft,
            SpecStatus::UnderReview,
            SpecStatus::Approved,
            SpecStatus::InProgress,
            SpecStatus::InImplementation,
            SpecStatus::Implemented,
            SpecStatus::Verified,
            SpecStatus::Done,
            SpecStatus::Archived,
            SpecStatus::Rejected,
            SpecStatus::Backlog,
            SpecStatus::Ready,
            SpecStatus::InReview,
            SpecStatus::Blocked,
            SpecStatus::Superseded,
            SpecStatus::Outdated,
            SpecStatus::Stable,
            SpecStatus::Deprecated,
            SpecStatus::Published,
            SpecStatus::Analysed,
            SpecStatus::Obsolete,
        ];
        assert_eq!(all.len(), 23);
        for s in all {
            assert_eq!(SpecStatus::from_str_lossy(s.to_str()), s);
        }
    }

    #[test]
    fn status_unknown_falls_back_to_draft() {
        assert_eq!(SpecStatus::from_str_lossy("nonexistent"), SpecStatus::Draft);
    }

    #[test]
    fn spec_item_new_defaults() {
        let item = SpecItem::new("G1", "First goal");
        assert_eq!(item.id, "G1");
        assert_eq!(item.title, "First goal");
        assert_eq!(item.status, SpecStatus::Empty);
        assert!(item.depends_on.is_empty());
        assert!(item.related.is_empty());
        assert!(item.tags.is_empty());
        assert!(item.completed_at.is_none());
        assert!(item.created_at > 0);
        assert_eq!(item.created_at, item.modified_at);
    }

    #[test]
    fn section_new_has_correct_title() {
        let s = SpecsSection::new(SectionType::Goals);
        assert_eq!(s.id, "goals");
        assert_eq!(s.section_type, SectionType::Goals);
        assert!(s.title.contains("Goals"));
        assert!(s.items.is_empty());
    }

    #[test]
    fn document_new_has_7_sections() {
        let doc = SpecsDocument::new("test-project");
        assert_eq!(doc.project, "test-project");
        assert_eq!(doc.version, 0);
        assert_eq!(doc.sections.len(), 7);
        // Sections are in canonical order.
        assert_eq!(doc.sections[0].section_type, SectionType::Goals);
        assert_eq!(doc.sections[6].section_type, SectionType::Reports);
    }

    #[test]
    fn spec_item_serializes_with_status_string() {
        let item = SpecItem::new("A1", "arch");
        let json = serde_json::to_string(&item).unwrap();
        // status serializes as the variant name (serde default); the to_str/
        // from_str_lossy pair is the canonical wire form for .ad files.
        assert!(json.contains("Empty"));
    }

    #[test]
    fn now_sec_is_recent() {
        let t = now_sec();
        // After 2024-01-01 (a safe lower bound).
        assert!(t > 1704067200);
    }

    // ── state machine ──────────────────────────────────────────

    #[test]
    fn transition_empty_to_proposed_ok() {
        assert!(can_transition(SpecStatus::Empty, SpecStatus::Proposed));
        assert!(can_transition(SpecStatus::Proposed, SpecStatus::Draft));
        assert!(can_transition(SpecStatus::Draft, SpecStatus::UnderReview));
        assert!(can_transition(SpecStatus::Approved, SpecStatus::InProgress));
        assert!(can_transition(SpecStatus::Verified, SpecStatus::Done));
    }

    #[test]
    fn transition_rejects_skipping() {
        // can't go Empty -> Approved (must pass proposed/draft/review)
        assert!(!can_transition(SpecStatus::Empty, SpecStatus::Approved));
        // can't verify before implementing
        assert!(!can_transition(SpecStatus::Approved, SpecStatus::Verified));
    }

    #[test]
    fn transition_blocked_can_proceed() {
        assert!(can_transition(SpecStatus::Blocked, SpecStatus::InProgress));
        assert!(can_transition(SpecStatus::Blocked, SpecStatus::Backlog));
    }

    #[test]
    fn transition_terminal_reopens_to_draft() {
        assert!(can_transition(SpecStatus::Done, SpecStatus::Draft));
        assert!(can_transition(SpecStatus::Rejected, SpecStatus::Draft));
    }

    #[test]
    fn transition_idempotent() {
        for s in [SpecStatus::Draft, SpecStatus::Done, SpecStatus::Backlog] {
            assert!(can_transition(s, s));
        }
    }

    #[test]
    fn transition_fn_returns_status_or_err() {
        assert_eq!(
            transition(SpecStatus::Empty, SpecStatus::Proposed).unwrap(),
            SpecStatus::Proposed
        );
        assert!(transition(SpecStatus::Empty, SpecStatus::Done).is_err());
    }

    // ── SpecsStore ─────────────────────────────────────────────

    #[test]
    fn store_creates_empty_doc_if_absent() {
        let dir = std::env::temp_dir().join("musk_specs_test_new");
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("specs.json");
        let store = SpecsStore::new(&path);
        let doc = store.load().unwrap();
        assert_eq!(doc.sections.len(), 7);
        assert!(path.exists()); // persisted
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn store_upsert_then_load_roundtrip() {
        let dir = std::env::temp_dir().join("musk_specs_test_upsert");
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("specs.json");
        let store = SpecsStore::new(&path);
        let mut doc = store.load().unwrap();

        let item = SpecItem::new("G1", "first goal");
        store.upsert_item(&mut doc, "goals", item).unwrap();
        assert_eq!(doc.version, 1);
        store.save(&doc).unwrap();

        // Reload from disk.
        let reloaded = store.load().unwrap();
        let goals = reloaded.sections.iter().find(|s| s.id == "goals").unwrap();
        assert_eq!(goals.items.len(), 1);
        assert_eq!(goals.items[0].id, "G1");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn store_upsert_replaces_existing() {
        let path = std::env::temp_dir().join("musk_specs_test_replace.json");
        let store = SpecsStore::new(&path);
        let mut doc = SpecsDocument::new("t");
        store
            .upsert_item(&mut doc, "goals", SpecItem::new("G1", "old"))
            .unwrap();
        store
            .upsert_item(&mut doc, "goals", SpecItem::new("G1", "new title"))
            .unwrap();
        let goals = doc.sections.iter().find(|s| s.id == "goals").unwrap();
        assert_eq!(goals.items.len(), 1); // not duplicated
        assert_eq!(goals.items[0].title, "new title");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn store_transition_item_validates() {
        let path = std::env::temp_dir().join("musk_specs_test_trans.json");
        let store = SpecsStore::new(&path);
        let mut doc = SpecsDocument::new("t");
        store
            .upsert_item(&mut doc, "goals", SpecItem::new("G1", "g"))
            .unwrap();
        // Empty -> Proposed (valid)
        store
            .transition_item(&mut doc, "goals", "G1", SpecStatus::Proposed)
            .unwrap();
        // Proposed -> Done (invalid, skips)
        let err = store
            .transition_item(&mut doc, "goals", "G1", SpecStatus::Done)
            .unwrap_err();
        assert!(err.contains("invalid"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn store_transition_to_done_sets_completed_at() {
        let path = std::env::temp_dir().join("musk_specs_test_done.json");
        let store = SpecsStore::new(&path);
        let mut doc = SpecsDocument::new("t");
        // Walk the item through to Verified, then Done.
        store
            .upsert_item(&mut doc, "goals", SpecItem::new("G1", "g"))
            .unwrap();
        for s in [
            SpecStatus::Proposed,
            SpecStatus::Draft,
            SpecStatus::Approved,
            SpecStatus::InProgress,
            SpecStatus::Implemented,
            SpecStatus::Verified,
            SpecStatus::Done,
        ] {
            store.transition_item(&mut doc, "goals", "G1", s).unwrap();
        }
        let goals = doc.sections.iter().find(|s| s.id == "goals").unwrap();
        assert_eq!(goals.items[0].status, SpecStatus::Done);
        assert!(goals.items[0].completed_at.is_some());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn store_delete_item() {
        let path = std::env::temp_dir().join("musk_specs_test_del.json");
        let store = SpecsStore::new(&path);
        let mut doc = SpecsDocument::new("t");
        store
            .upsert_item(&mut doc, "goals", SpecItem::new("G1", "g"))
            .unwrap();
        assert!(store.delete_item(&mut doc, "goals", "G1").unwrap());
        assert!(!store.delete_item(&mut doc, "goals", "G1").unwrap()); // gone
        let goals = doc.sections.iter().find(|s| s.id == "goals").unwrap();
        assert!(goals.items.is_empty());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn store_unknown_section_errors() {
        let path = std::env::temp_dir().join("musk_specs_test_unknown.json");
        let store = SpecsStore::new(&path);
        let mut doc = SpecsDocument::new("t");
        let err = store
            .upsert_item(&mut doc, "nonexistent", SpecItem::new("X1", "x"))
            .unwrap_err();
        assert!(err.contains("not found"));
        let _ = std::fs::remove_file(&path);
    }
}
