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
}
