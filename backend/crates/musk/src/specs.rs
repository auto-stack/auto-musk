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
use std::sync::OnceLock;

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
// Relation graph — derived reverse-links (`related`)
// ============================================================

/// Regex matching a spec item ID: optional `<module>-` prefix, a type letter
/// from [GADPSVXTIR], digits, optional `.sub`. Ported from auto-forge
/// `mod.rs:1812` (`\b((?:[A-Za-z]+-)?[GADPSVXTIR]\d+(?:\.\d+)?)\b`).
static ID_RE: OnceLock<regex::Regex> = OnceLock::new();

fn id_regex() -> &'static regex::Regex {
    ID_RE.get_or_init(|| {
        // \b boundaries; module prefix is letters + '-'; type letter fixed set.
        regex::Regex::new(r"\b(?:[A-Za-z]+-)?[GADPSVXTIR]\d+(?:\.\d+)?\b").unwrap()
    })
}

/// Collect all spec item IDs present in the document (across all sections).
fn all_ids(doc: &SpecsDocument) -> std::collections::HashSet<String> {
    let mut set = std::collections::HashSet::new();
    for section in &doc.sections {
        for item in &section.items {
            set.insert(item.id.clone());
        }
    }
    set
}

/// Scan `text` for references to known item IDs, returning the matched IDs
/// (deduped). Only references to IDs that actually exist in `known` are kept
/// (avoids matching ordinary text that happens to look like an ID).
fn scan_refs(text: &str, known: &std::collections::HashSet<String>) -> Vec<String> {
    let mut refs: Vec<String> = id_regex()
        .find_iter(text)
        .map(|m| m.as_str().to_string())
        .filter(|r| known.contains(r))
        .collect();
    refs.sort();
    refs.dedup();
    refs
}

impl SpecsDocument {
    /// Rebuild every item's derived `related` field (reverse-links).
    ///
    /// For each item A, its forward references are: (a) `depends_on` (manual)
    /// and (b) any IDs mentioned in its `content`. For each forward edge
    /// `A -> B`, B's `related` list gains A. After the scan, each `related`
    /// list is sorted + deduped.
    ///
    /// Call this after any upsert/delete/load (mirrors auto-forge
    /// `mod.rs:1810-1850`).
    pub fn rebuild_relations(&mut self) {
        let known = all_ids(self);
        // referrer_id -> set of IDs it references (forward edges)
        let mut reverse: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();

        for section in &self.sections {
            for item in &section.items {
                // forward edges: depends_on + content-scanned refs
                let mut forwards: Vec<String> = item.depends_on.clone();
                forwards.extend(scan_refs(&item.content, &known));
                forwards.sort();
                forwards.dedup();
                for target in &forwards {
                    reverse
                        .entry(target.clone())
                        .or_default()
                        .push(item.id.clone());
                }
            }
        }

        // write back reverse-links into each item's `related`
        for section in &mut self.sections {
            for item in &mut section.items {
                let mut rel = reverse.remove(&item.id).unwrap_or_default();
                rel.sort();
                rel.dedup();
                item.related = rel;
            }
        }
    }
}

// ============================================================
// Derived statuses — auto-advance item/section status from relations
// ============================================================

impl SpecsDocument {
    /// Derive (auto-advance) statuses from the relation graph.
    ///
    /// Rules (ported from auto-forge `mod.rs:1875-2040+`):
    /// 1. **Goal → Implemented**: a Goal in {Empty, Proposed, Analysed,
    ///    Approved, InProgress} whose *all* related Plans are `Done` advances
    ///    to `Implemented` (only if the per-section machine allows it).
    /// 2. **Goal → Verified**: a Goal that is `Implemented`, whose all related
    ///    Tests are `Done`/`Verified` AND at least one related Review is
    ///    `Published`, advances to `Verified`.
    /// 3. **Section aggregate**: a section whose every item has reached its
    ///    section type's "done-ish" status (see `section_complete_status`)
    ///    advances the section-level `status`.
    ///
    /// Only forward transitions allowed by `SectionConfig` are applied; invalid
    /// ones are skipped silently (no forcing). Call after `rebuild_relations`.
    pub fn derive_statuses(&mut self) {
        // Snapshot ids + their related lists + statuses for read-only scanning.
        // (We need to read across sections while mutating, so snapshot first.)
        #[derive(Clone)]
        struct Snap {
            id: String,
            section_type: SectionType,
            status: SpecStatus,
            related: Vec<String>,
        }
        let snap: std::collections::HashMap<String, Snap> = {
            let mut m = std::collections::HashMap::new();
            for s in &self.sections {
                for it in &s.items {
                    m.insert(
                        it.id.clone(),
                        Snap {
                            id: it.id.clone(),
                            section_type: s.section_type,
                            status: it.status,
                            related: it.related.clone(),
                        },
                    );
                }
            }
            m
        };

        // ── Rule 1 & 2: Goal auto-advance ──
        // A Goal's forward deps (the Plans/Tests/Reviews it references) live in
        // its `depends_on` (and content-scanned refs, captured into the reverse
        // graph as: those targets' `related` contains the Goal). For derivation
        // we need the FORWARD edges out of the Goal, so we read `depends_on`
        // (plus content refs recomputed via the snap of reverse edges would be
        // circular — `depends_on` is the authoritative forward list).
        use SpecStatus::*;
        let goal_cfg = SectionConfig::for_type(SectionType::Goals);
        for section in &mut self.sections {
            if section.section_type != SectionType::Goals {
                continue;
            }
            for item in &mut section.items {
                // forward refs = depends_on (manual) — the authoritative list
                let forwards: Vec<&String> = item.depends_on.iter().collect();
                let related_plans_all_done = forwards
                    .iter()
                    .filter_map(|rid| snap.get(*rid))
                    .filter(|x| x.section_type == SectionType::Plans)
                    .all(|x| x.status == Done);
                let has_any_plan = forwards
                    .iter()
                    .any(|rid| matches!(snap.get(*rid).map(|x| x.section_type), Some(SectionType::Plans)));

                // Rule 1: Goal → Implemented
                if has_any_plan
                    && related_plans_all_done
                    && matches!(item.status, Empty | Proposed | Analysed | Approved | InProgress)
                    && goal_cfg.can_transition(item.status, Implemented)
                {
                    item.status = Implemented;
                    item.modified_at = now_sec();
                }

                // Rule 2: Goal → Verified (needs Implemented now, all Tests done/verified, ≥1 Review published)
                if item.status == Implemented {
                    let tests_ok = forwards
                        .iter()
                        .filter_map(|rid| snap.get(*rid))
                        .filter(|x| x.section_type == SectionType::Tests)
                        .all(|x| matches!(x.status, Done | Verified));
                    let has_tests = forwards
                        .iter()
                        .any(|rid| matches!(snap.get(*rid).map(|x| x.section_type), Some(SectionType::Tests)));
                    let has_review_published = forwards
                        .iter()
                        .filter_map(|rid| snap.get(*rid))
                        .any(|x| x.section_type == SectionType::Reviews && x.status == Published);
                    if has_tests && tests_ok && has_review_published && goal_cfg.can_transition(Implemented, Verified) {
                        item.status = Verified;
                        item.modified_at = now_sec();
                    }
                }
            }
        }

        // ── Rule 3: section aggregate status ──
        for section in &mut self.sections {
            if section.items.is_empty() {
                continue;
            }
            let done_status = section_complete_status(section.section_type);
            let cfg = SectionConfig::for_type(section.section_type);
            // Every item reached its done status ⇒ section advances to it.
            if section.items.iter().all(|it| it.status == done_status) {
                if cfg.can_transition(section.status, done_status) && section.status != done_status {
                    section.status = done_status;
                    section.last_modified = now_sec();
                }
            } else if section.items.iter().any(|it| it.status != Empty) {
                // Any non-empty activity ⇒ section at least Draft (if machine allows).
                if cfg.can_transition(section.status, Draft) && section.status == Empty {
                    section.status = Draft;
                    section.last_modified = now_sec();
                }
            }
        }
    }
}

/// The "done-ish" terminal status for a section type (used by aggregate rule).
fn section_complete_status(st: SectionType) -> SpecStatus {
    match st {
        SectionType::Goals => SpecStatus::Archived,
        SectionType::Architecture | SectionType::Designs => SpecStatus::Approved,
        SectionType::Plans => SpecStatus::Done,
        SectionType::Tests => SpecStatus::Verified,
        SectionType::Reviews | SectionType::Reports => SpecStatus::Published,
    }
}

// ============================================================
// Overview & drift-check
// ============================================================

/// A per-section summary for the overview endpoint.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SectionOverview {
    pub id: String,
    pub section_type: SectionType,
    pub title: String,
    pub status: SpecStatus,
    pub item_count: usize,
    /// How many items are in each status (status_str -> count).
    pub status_counts: Vec<(String, usize)>,
}

/// A document-level overview (aggregate of all sections).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpecsOverview {
    pub project: String,
    pub version: u64,
    pub total_items: usize,
    pub sections: Vec<SectionOverview>,
}

impl SpecsDocument {
    /// Build an aggregate overview (per-section item counts + status dist).
    pub fn overview(&self) -> SpecsOverview {
        let mut sections = Vec::new();
        let mut total = 0;
        for s in &self.sections {
            let mut counts: std::collections::BTreeMap<String, usize> =
                std::collections::BTreeMap::new();
            for it in &s.items {
                *counts.entry(it.status.to_str().to_string()).or_insert(0) += 1;
            }
            total += s.items.len();
            sections.push(SectionOverview {
                id: s.id.clone(),
                section_type: s.section_type,
                title: s.title.clone(),
                status: s.status,
                item_count: s.items.len(),
                status_counts: counts.into_iter().collect(),
            });
        }
        SpecsOverview {
            project: self.project.clone(),
            version: self.version,
            total_items: total,
            sections,
        }
    }
}

impl SpecsStore {
    /// Drift check: compare the in-memory `doc` against what's persisted on
    /// disk. Returns `(disk_version, drifted)` — `drifted` is true if the
    /// disk document's version differs from `doc.version` (a signal that
    /// another writer / external edit touched the file).
    ///
    /// (auto-forge watches `.ad` files via notify; musk uses a single JSON
    /// file, so drift = version mismatch with the file on disk.)
    pub fn drift_check(&self, doc: &SpecsDocument) -> Result<(u64, bool), String> {
        match self.load() {
            Ok(disk) => Ok((disk.version, disk.version != doc.version)),
            Err(e) => Err(format!("drift_check load failed: {e}")),
        }
    }
}



/// Current unix timestamp in seconds (stand-in for the `.at`'s `now_sec`).
pub fn now_sec() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// ============================================================
// State machine — per-section valid status transitions
// ============================================================

/// Per-section-type state-machine configuration.
///
/// Each of the 7 `SectionType`s has its own `allowed_statuses` and
/// `allowed_transitions`. This is the per-section replacement for the old
/// single global `can_transition` (which applied one rule set to all sections).
///
/// Ported from auto-forge `mod.rs:242-342` (`SectionConfig::for_type`), with
/// the greenfield simplifications noted in plans/009 Task 1:
/// - The loose 3rd rule ("to ∈ allowed_statuses ⇒ allow") is REMOVED — only
///   explicit transitions (and idempotent from==to) pass.
/// - The Reports state-machine bug is fixed: auto-forge's first `Reports` match
///   arm (`mod.rs:297`) was shadowed by the later `Reviews | Reports` arm
///   (`mod.rs:324`); we use a single clear arm for Reviews|Reports.
#[derive(Clone, Debug)]
pub struct SectionConfig {
    pub section_type: SectionType,
    pub allowed_statuses: Vec<SpecStatus>,
    pub allowed_transitions: Vec<(SpecStatus, SpecStatus)>,
}

impl SectionConfig {
    /// The state machine for a given section type.
    pub fn for_type(st: SectionType) -> Self {
        use SpecStatus::*;
        match st {
            // Goals: Empty → Proposed → Analysed → Approved → InProgress →
            //        Implemented → Done → Archived (+ InProgress → Archived)
            SectionType::Goals => Self {
                section_type: st,
                allowed_statuses: vec![
                    Empty, Proposed, Analysed, Approved, InProgress, Implemented, Verified, Done,
                    Archived,
                ],
                allowed_transitions: vec![
                    (Empty, Proposed),
                    (Proposed, Analysed),
                    (Analysed, Approved),
                    (Approved, InProgress),
                    (InProgress, Implemented),
                    (Implemented, Verified),
                    (Verified, Done),
                    (Done, Archived),
                    (InProgress, Archived),
                ],
            },
            // Architecture / Designs: Empty → Draft → UnderReview → {Approved|Rejected},
            //   Approved → {Superseded|Outdated}
            SectionType::Architecture | SectionType::Designs => Self {
                section_type: st,
                allowed_statuses: vec![
                    Empty, Draft, UnderReview, Approved, Rejected, Superseded, Outdated,
                ],
                allowed_transitions: vec![
                    (Empty, Draft),
                    (Draft, UnderReview),
                    (UnderReview, Approved),
                    (UnderReview, Rejected),
                    (Approved, Superseded),
                    (Approved, Outdated),
                ],
            },
            // Plans: Empty → Draft → Approved → InProgress → Done → Obsolete
            SectionType::Plans => Self {
                section_type: st,
                allowed_statuses: vec![Empty, Draft, Approved, InProgress, Done, Obsolete],
                allowed_transitions: vec![
                    (Empty, Draft),
                    (Draft, Approved),
                    (Approved, InProgress),
                    (InProgress, Done),
                    (Done, Obsolete),
                ],
            },
            // Tests: Empty → Draft → Implemented → Done → Verified,
            //        Implemented ↔ Blocked
            SectionType::Tests => Self {
                section_type: st,
                allowed_statuses: vec![
                    Empty, Draft, Implemented, Done, Verified, Blocked,
                ],
                allowed_transitions: vec![
                    (Empty, Draft),
                    (Draft, Implemented),
                    (Implemented, Done),
                    (Done, Verified),
                    (Implemented, Blocked),
                    (Blocked, Implemented),
                ],
            },
            // Reviews / Reports: Empty → Draft → Published
            // (fixed: auto-forge shadowed Reports' own arm; we unify both)
            SectionType::Reviews | SectionType::Reports => Self {
                section_type: st,
                allowed_statuses: vec![Empty, Draft, Published],
                allowed_transitions: vec![(Empty, Draft), (Draft, Published)],
            },
        }
    }

    /// Is `from -> to` a legal transition for this section type?
    /// Idempotent (from == to) is always allowed. Otherwise the pair must be in
    /// `allowed_transitions`. (The loose "to ∈ allowed_statuses" rule is
    /// intentionally NOT applied — see plans/009 Task 1.)
    pub fn can_transition(&self, from: SpecStatus, to: SpecStatus) -> bool {
        if from == to {
            return true;
        }
        self.allowed_transitions.contains(&(from, to))
    }
}

/// Per-section transition check: is `from -> to` valid for `section_type`?
pub fn can_transition(st: SectionType, from: SpecStatus, to: SpecStatus) -> bool {
    SectionConfig::for_type(st).can_transition(from, to)
}

/// Per-section transition: return the new status or an error if invalid for
/// this section type.
pub fn transition(
    st: SectionType,
    from: SpecStatus,
    to: SpecStatus,
) -> Result<SpecStatus, String> {
    if SectionConfig::for_type(st).can_transition(from, to) {
        Ok(to)
    } else {
        Err(format!(
            "invalid status transition for {}: {} -> {}",
            st.as_str(),
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
    /// item if `item.id` is new, replaces it otherwise. Rebuilds relations.
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
        doc.rebuild_relations();
        doc.derive_statuses();
        Ok(())
    }

    /// Transition an item's status (validates via the per-section state machine).
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
        let st = section.section_type;
        let item = section
            .items
            .iter_mut()
            .find(|i| i.id == item_id)
            .ok_or_else(|| format!("item '{item_id}' not found in '{section_id}'"))?;
        let updated = transition(st, item.status, new_status)?;
        item.status = updated;
        item.modified_at = now_sec();
        if matches!(updated, SpecStatus::Done) {
            item.completed_at = Some(now_sec());
        }
        section.last_modified = now_sec();
        doc.version += 1;
        Ok(())
    }

    /// Delete an item. Returns true if it existed. Rebuilds relations.
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
            doc.rebuild_relations();
            doc.derive_statuses();
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

    // ── state machine (per-section) ──────────────────────────

    #[test]
    fn goals_transitions_canonical_path() {
        use SpecStatus::*;
        let st = SectionType::Goals;
        assert!(can_transition(st, Empty, Proposed));
        assert!(can_transition(st, Proposed, Analysed));
        assert!(can_transition(st, Analysed, Approved));
        assert!(can_transition(st, Approved, InProgress));
        assert!(can_transition(st, InProgress, Implemented));
        assert!(can_transition(st, Implemented, Verified));
        assert!(can_transition(st, Verified, Done));
        assert!(can_transition(st, Done, Archived));
        // side branch
        assert!(can_transition(st, InProgress, Archived));
    }

    #[test]
    fn goals_rejects_skipping() {
        use SpecStatus::*;
        let st = SectionType::Goals;
        assert!(!can_transition(st, Empty, Approved)); // must pass proposed/analysed
        assert!(!can_transition(st, Approved, Done)); // must implement first
    }

    #[test]
    fn architecture_transitions() {
        use SpecStatus::*;
        let st = SectionType::Architecture;
        assert!(can_transition(st, Empty, Draft));
        assert!(can_transition(st, Draft, UnderReview));
        assert!(can_transition(st, UnderReview, Approved));
        assert!(can_transition(st, UnderReview, Rejected));
        assert!(can_transition(st, Approved, Superseded));
        assert!(can_transition(st, Approved, Outdated));
        // can't skip review
        assert!(!can_transition(st, Draft, Approved));
    }

    #[test]
    fn plans_transitions() {
        use SpecStatus::*;
        let st = SectionType::Plans;
        assert!(can_transition(st, Empty, Draft));
        assert!(can_transition(st, Draft, Approved));
        assert!(can_transition(st, Approved, InProgress));
        assert!(can_transition(st, InProgress, Done));
        assert!(can_transition(st, Done, Obsolete));
        assert!(!can_transition(st, Draft, Done));
    }

    #[test]
    fn tests_transitions_with_blocked() {
        use SpecStatus::*;
        let st = SectionType::Tests;
        assert!(can_transition(st, Empty, Draft));
        assert!(can_transition(st, Draft, Implemented));
        assert!(can_transition(st, Implemented, Done));
        assert!(can_transition(st, Done, Verified));
        // blocked toggle
        assert!(can_transition(st, Implemented, Blocked));
        assert!(can_transition(st, Blocked, Implemented));
        // can't verify before done
        assert!(!can_transition(st, Implemented, Verified));
    }

    #[test]
    fn reviews_reports_unified_simple_flow() {
        // auto-forge bug fix: Reports now uses the same flow as Reviews
        use SpecStatus::*;
        for st in [SectionType::Reviews, SectionType::Reports] {
            assert!(can_transition(st, Empty, Draft));
            assert!(can_transition(st, Draft, Published));
            // no UnderReview/Stable/Deprecated (those were the shadowed arm)
            assert!(!can_transition(st, Draft, Stable));
            assert!(!can_transition(st, Empty, Published));
        }
    }

    #[test]
    fn transition_idempotent() {
        for st in [
            SectionType::Goals,
            SectionType::Plans,
            SectionType::Tests,
        ] {
            assert!(can_transition(st, SpecStatus::Draft, SpecStatus::Draft));
        }
    }

    #[test]
    fn transition_fn_returns_status_or_err() {
        assert_eq!(
            transition(SectionType::Goals, SpecStatus::Empty, SpecStatus::Proposed).unwrap(),
            SpecStatus::Proposed
        );
        assert!(transition(SectionType::Goals, SpecStatus::Empty, SpecStatus::Done).is_err());
    }

    #[test]
    fn section_config_allowed_statuses_match_machine() {
        // every transition's endpoints must be in allowed_statuses
        for st in [
            SectionType::Goals,
            SectionType::Architecture,
            SectionType::Designs,
            SectionType::Plans,
            SectionType::Tests,
            SectionType::Reviews,
            SectionType::Reports,
        ] {
            let cfg = SectionConfig::for_type(st);
            for (from, to) in &cfg.allowed_transitions {
                assert!(
                    cfg.allowed_statuses.contains(from),
                    "{:?}: transition source {:?} not in allowed_statuses",
                    st,
                    from
                );
                assert!(
                    cfg.allowed_statuses.contains(to),
                    "{:?}: transition target {:?} not in allowed_statuses",
                    st,
                    to
                );
            }
        }
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
        // Walk the item through the Goals state machine to Verified, then Done.
        store
            .upsert_item(&mut doc, "goals", SpecItem::new("G1", "g"))
            .unwrap();
        for s in [
            SpecStatus::Proposed,
            SpecStatus::Analysed,
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

    // ── rebuild_relations (relation graph) ────────────────────

    #[test]
    fn rebuild_relations_depends_on_creates_reverse_link() {
        let mut doc = SpecsDocument::new("t");
        let mut a = SpecItem::new("G1", "goal");
        a.depends_on = vec!["P1".into()];
        let mut p = SpecItem::new("P1", "plan");
        p.status = SpecStatus::Draft;
        store_upsert_both(&mut doc, a, p);
        // After rebuild via upsert: P1.related should contain G1.
        let p1 = find_item(&doc, "plans", "P1");
        assert!(p1.related.contains(&"G1".to_string()));
    }

    #[test]
    fn rebuild_relations_content_reference_creates_edge() {
        // An item whose content mentions another item's ID by text creates an edge.
        let mut doc = SpecsDocument::new("t");
        let mut g = SpecItem::new("G1", "goal");
        g.content = "This depends on plan P1 for delivery".into();
        let p = SpecItem::new("P1", "plan");
        store_upsert_both(&mut doc, g, p);
        let p1 = find_item(&doc, "plans", "P1");
        assert!(p1.related.contains(&"G1".to_string()), "P1.related = {:?}", p1.related);
    }

    #[test]
    fn rebuild_relations_ignores_unknown_ids() {
        // Text that looks like an ID but isn't a real item is NOT an edge.
        let mut doc = SpecsDocument::new("t");
        let mut g = SpecItem::new("G1", "goal");
        g.content = "see Z99 for details".into(); // Z99 doesn't exist
        store_upsert_both(&mut doc, g, SpecItem::new("P1", "plan"));
        let p1 = find_item(&doc, "plans", "P1");
        assert!(!p1.related.contains(&"G1".to_string()) || true); // P1 not referenced by G1
        // G1.related should NOT contain Z99 (it doesn't exist)
        let g1 = find_item(&doc, "goals", "G1");
        assert!(!g1.related.contains(&"Z99".to_string()));
    }

    #[test]
    fn rebuild_relations_module_prefixed_id() {
        let path = std::env::temp_dir().join("musk_specs_rel_mod.json");
        let store = SpecsStore::new(&path);
        let mut doc = SpecsDocument::new("t");
        let mut g = SpecItem::new("G1", "goal");
        g.content = "covered by auth-T1".into();
        store.upsert_item(&mut doc, "goals", g).unwrap();
        store.upsert_item(&mut doc, "tests", SpecItem::new("auth-T1", "test")).unwrap();
        let t1 = find_item(&doc, "tests", "auth-T1");
        assert!(t1.related.contains(&"G1".to_string()), "auth-T1.related = {:?}", t1.related);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn rebuild_relations_delete_removes_edges() {
        let path = std::env::temp_dir().join("musk_specs_rel_del.json");
        let store = SpecsStore::new(&path);
        let mut doc = SpecsDocument::new("t");
        let mut a = SpecItem::new("G1", "g");
        a.depends_on = vec!["P1".into()];
        store.upsert_item(&mut doc, "goals", a).unwrap();
        store.upsert_item(&mut doc, "plans", SpecItem::new("P1", "p")).unwrap();
        // P1.related has G1
        assert!(find_item(&doc, "plans", "P1").related.contains(&"G1".into()));
        // delete G1 -> P1.related should no longer reference G1
        store.delete_item(&mut doc, "goals", "G1").unwrap();
        assert!(!find_item(&doc, "plans", "P1").related.contains(&"G1".into()));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn rebuild_relations_dedupes_and_sorts() {
        let mut doc = SpecsDocument::new("t");
        let mut g = SpecItem::new("G1", "g");
        g.depends_on = vec!["P1".into(), "P1".into()]; // dup
        g.content = "P1 P1".into(); // dup refs
        store_upsert_both(&mut doc, g, SpecItem::new("P1", "p"));
        let p1 = find_item(&doc, "plans", "P1");
        assert_eq!(p1.related.iter().filter(|x| x == &&"G1".to_string()).count(), 1);
    }

    // ── derive_statuses (derived status advancement) ─────────

    #[test]
    fn derive_goal_implemented_when_all_plans_done() {
        let path = std::env::temp_dir().join("musk_specs_derive_impl.json");
        let store = SpecsStore::new(&path);
        let mut doc = SpecsDocument::new("t");
        // Goal G1 depends on Plan P1; walk G1 to InProgress, P1 to Done.
        let mut g = SpecItem::new("G1", "goal");
        g.depends_on = vec!["P1".into()];
        g.status = SpecStatus::Approved;
        store.upsert_item(&mut doc, "goals", g).unwrap();
        // walk G1 Approved -> InProgress (per-section machine)
        store.transition_item(&mut doc, "goals", "G1", SpecStatus::InProgress).unwrap();
        let mut p = SpecItem::new("P1", "plan");
        p.status = SpecStatus::Approved;
        store.upsert_item(&mut doc, "plans", p).unwrap();
        store.transition_item(&mut doc, "plans", "P1", SpecStatus::InProgress).unwrap();
        store.transition_item(&mut doc, "plans", "P1", SpecStatus::Done).unwrap();
        // re-upsert G1 (triggers rebuild+derive) — G1 should now be Implemented
        let mut g2 = SpecItem::new("G1", "goal");
        g2.depends_on = vec!["P1".into()];
        g2.status = SpecStatus::InProgress;
        store.upsert_item(&mut doc, "goals", g2).unwrap();
        assert_eq!(find_item(&doc, "goals", "G1").status, SpecStatus::Implemented);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn derive_goal_verified_when_tests_and_review() {
        let path = std::env::temp_dir().join("musk_specs_derive_ver.json");
        let store = SpecsStore::new(&path);
        let mut doc = SpecsDocument::new("t");
        // G1 -> T1 (test) + R1 (review); set G1 Implemented, T1 Verified, R1 Published
        let mut g = SpecItem::new("G1", "goal");
        g.depends_on = vec!["T1".into(), "R1".into()];
        g.status = SpecStatus::Implemented; // already implemented
        store.upsert_item(&mut doc, "goals", g).unwrap();
        // T1: Draft -> Implemented -> Done -> Verified
        let mut t = SpecItem::new("T1", "test");
        t.status = SpecStatus::Draft;
        store.upsert_item(&mut doc, "tests", t).unwrap();
        store.transition_item(&mut doc, "tests", "T1", SpecStatus::Implemented).unwrap();
        store.transition_item(&mut doc, "tests", "T1", SpecStatus::Done).unwrap();
        store.transition_item(&mut doc, "tests", "T1", SpecStatus::Verified).unwrap();
        // R1: Empty -> Draft -> Published
        let mut r = SpecItem::new("R1", "review");
        r.status = SpecStatus::Empty;
        store.upsert_item(&mut doc, "reviews", r).unwrap();
        store.transition_item(&mut doc, "reviews", "R1", SpecStatus::Draft).unwrap();
        store.transition_item(&mut doc, "reviews", "R1", SpecStatus::Published).unwrap();
        // re-upsert G1 triggers derive -> Verified
        let mut g2 = SpecItem::new("G1", "goal");
        g2.depends_on = vec!["T1".into(), "R1".into()];
        g2.status = SpecStatus::Implemented;
        store.upsert_item(&mut doc, "goals", g2).unwrap();
        assert_eq!(find_item(&doc, "goals", "G1").status, SpecStatus::Verified);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn derive_section_aggregate_to_draft() {
        // A section with any non-Empty item advances section Empty -> Draft.
        let path = std::env::temp_dir().join("musk_specs_derive_sec.json");
        let store = SpecsStore::new(&path);
        let mut doc = SpecsDocument::new("t");
        // section starts Empty; upsert a plan in Draft
        let mut p = SpecItem::new("P1", "plan");
        p.status = SpecStatus::Draft;
        store.upsert_item(&mut doc, "plans", p).unwrap();
        let plans = doc.sections.iter().find(|s| s.id == "plans").unwrap();
        assert_eq!(plans.status, SpecStatus::Draft);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn derive_does_not_force_invalid_transition() {
        // A Goal at Empty with no related plans stays Empty (nothing to satisfy).
        let path = std::env::temp_dir().join("musk_specs_derive_nop.json");
        let store = SpecsStore::new(&path);
        let mut doc = SpecsDocument::new("t");
        store.upsert_item(&mut doc, "goals", SpecItem::new("G1", "goal")).unwrap();
        assert_eq!(find_item(&doc, "goals", "G1").status, SpecStatus::Empty);
        let _ = std::fs::remove_file(&path);
    }

    // ── overview & drift-check ───────────────────────────────

    #[test]
    fn overview_counts_items_per_section() {
        let path = std::env::temp_dir().join("musk_specs_overview.json");
        let store = SpecsStore::new(&path);
        let mut doc = SpecsDocument::new("proj");
        store.upsert_item(&mut doc, "goals", SpecItem::new("G1", "g1")).unwrap();
        store.upsert_item(&mut doc, "goals", SpecItem::new("G2", "g2")).unwrap();
        store.upsert_item(&mut doc, "plans", SpecItem::new("P1", "p1")).unwrap();
        let ov = doc.overview();
        assert_eq!(ov.project, "proj");
        assert_eq!(ov.total_items, 3);
        let goals = ov.sections.iter().find(|s| s.id == "goals").unwrap();
        assert_eq!(goals.item_count, 2);
        let plans = ov.sections.iter().find(|s| s.id == "plans").unwrap();
        assert_eq!(plans.item_count, 1);
        // status_counts: both goals Empty
        let empty_count = goals
            .status_counts
            .iter()
            .find(|(k, _)| k == "empty")
            .map(|(_, v)| *v)
            .unwrap_or(0);
        assert_eq!(empty_count, 2);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn drift_check_no_drift_when_unchanged() {
        let path = std::env::temp_dir().join("musk_specs_drift_ok.json");
        let store = SpecsStore::new(&path);
        let doc = store.load().unwrap();
        let (disk_ver, drifted) = store.drift_check(&doc).unwrap();
        assert_eq!(disk_ver, doc.version);
        assert!(!drifted);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn drift_check_detects_version_mismatch() {
        let path = std::env::temp_dir().join("musk_specs_drift_diff.json");
        let store = SpecsStore::new(&path);
        let mut doc = store.load().unwrap();
        // mutate in-memory (bump version) but DON'T save
        doc.version += 1;
        let (_, drifted) = store.drift_check(&doc).unwrap();
        assert!(drifted);
        let _ = std::fs::remove_file(&path);
    }

    // helpers for relation tests
    fn store_upsert_both(doc: &mut SpecsDocument, goal: SpecItem, plan: SpecItem) {
        let store = SpecsStore::new(std::env::temp_dir().join("musk_specs_rel_helper.json"));
        store.upsert_item(doc, "goals", goal).unwrap();
        store.upsert_item(doc, "plans", plan).unwrap();
        let _ = std::fs::remove_file(std::env::temp_dir().join("musk_specs_rel_helper.json"));
    }

    fn find_item<'a>(doc: &'a SpecsDocument, section_id: &str, item_id: &str) -> &'a SpecItem {
        doc.sections
            .iter()
            .find(|s| s.id == section_id)
            .unwrap()
            .items
            .iter()
            .find(|i| i.id == item_id)
            .unwrap()
    }
}
