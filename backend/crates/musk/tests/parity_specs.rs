//! Parity tests — verify `auto_generated::specs` behaves identically to the
//! hand-written `specs` module.
//!
//! These tests exercise the *same scenarios* on both the hand-written Rust
//! (`musk::specs`) and the a2r-transpiled Auto output (`musk::auto_generated::specs`),
//! asserting that both produce equal results. The two modules define separate
//! types (same names, different paths), so we compare observable behavior:
//! status string representations, field values, and derived state transitions.
//!
//! This is the reference framework for Plan 018 (auto parity). As each module
//! reaches parity, its hand-written unit tests are adapted into this file.

use musk::specs as hw;             // hand-written
use musk::auto_generated::specs as ag;  // a2r-transpiled Auto

// ──────────────────────────────────────────────────────────
// SectionType — string conversions parity
// ──────────────────────────────────────────────────────────

#[test]
fn parity_section_type_as_str() {
    // Both versions should produce the same snake_case id for each variant.
    let cases: Vec<(&str, hw::SectionType, ag::SectionType)> = vec![
        ("goals", hw::SectionType::Goals, ag::SectionType::Goals),
        ("architecture", hw::SectionType::Architecture, ag::SectionType::Architecture),
        ("designs", hw::SectionType::Designs, ag::SectionType::Designs),
        ("tests", hw::SectionType::Tests, ag::SectionType::Tests),
        ("reviews", hw::SectionType::Reviews, ag::SectionType::Reviews),
        ("reports", hw::SectionType::Reports, ag::SectionType::Reports),
    ];
    for (expected, hw_v, ag_v) in cases {
        assert_eq!(hw_v.as_str(), expected, "hand-written as_str mismatch");
        assert_eq!(ag_v.as_str(), expected, "auto_generated as_str mismatch");
    }
}

#[test]
fn parity_section_type_display_title() {
    let hw_t = hw::SectionType::Goals;
    let ag_t = ag::SectionType::Goals;
    assert_eq!(hw_t.display_title(), ag_t.display_title());
}

#[test]
fn parity_section_type_from_str() {
    // SectionType uses from_id (a2r auto-generates this); SpecStatus uses from_str_lossy.
    // Here we verify from_id round-trips on both versions.
    assert_eq!(hw::SectionType::from_id("goals"), hw::SectionType::Goals);
    assert_eq!(ag::SectionType::from_id("goals"), ag::SectionType::Goals);
    // Unknown falls back to Goals in both.
    assert_eq!(
        hw::SectionType::from_id("nonexistent").as_str(),
        ag::SectionType::from_id("nonexistent").as_str(),
    );
}

// ──────────────────────────────────────────────────────────
// SectionConfig — state machine parity
// ──────────────────────────────────────────────────────────

#[test]
fn parity_goals_state_machine() {
    // The canonical path: Empty → Proposed → Analysed → Approved → InProgress → Implemented → Verified → Done → Archived
    let hw_cfg = hw::SectionConfig::for_type(hw::SectionType::Goals);
    let ag_cfg = ag::SectionConfig::for_type(ag::SectionType::Goals);

    // Legal transitions must agree
    let legal: Vec<(&str, &str)> = vec![
        ("empty", "proposed"), ("proposed", "analysed"), ("analysed", "approved"),
        ("approved", "in_progress"), ("in_progress", "implemented"),
        ("implemented", "verified"), ("verified", "done"), ("done", "archived"),
    ];
    for (from, to) in &legal {
        let hw_from = hw::SpecStatus::from_str_lossy(from);
        let hw_to = hw::SpecStatus::from_str_lossy(to);
        let ag_from = ag::SpecStatus::from_str_lossy(from);
        let ag_to = ag::SpecStatus::from_str_lossy(to);
        assert_eq!(
            hw_cfg.can_transition(hw_from, hw_to),
            ag_cfg.can_transition(ag_from, ag_to),
            "Goals can_transition({from} -> {to}) disagrees",
        );
    }

    // Illegal: skipping must be rejected by both
    assert!(!hw_cfg.can_transition(hw::SpecStatus::Empty, hw::SpecStatus::Done));
    assert!(!ag_cfg.can_transition(ag::SpecStatus::Empty, ag::SpecStatus::Done));
    assert!(!hw_cfg.can_transition(hw::SpecStatus::Archived, hw::SpecStatus::Proposed));
    assert!(!ag_cfg.can_transition(ag::SpecStatus::Archived, ag::SpecStatus::Proposed));
}

// ──────────────────────────────────────────────────────────
// rebuild_relations — reverse-link graph parity
// ──────────────────────────────────────────────────────────

#[test]
fn parity_rebuild_relations_depends_on() {
    // Scenario: Goal G-1 depends_on Plan P-1.
    // After rebuild_relations, P-1.related should contain G-1 (reverse link).
    //
    // We build the document separately in each module (types don't cross),
    // then compare the resulting related lists.

    // Hand-written
    let mut hw_doc = hw::SpecsDocument::new("test");
    let mut hw_g = hw::SpecItem::new("G-1", "goal");
    hw_g.depends_on = vec!["P-1".into()];
    hw_doc.sections[0].items.push(hw_g); // Goals
    let hw_p = hw::SpecItem::new("P-1", "plan");
    hw_doc.sections[3].items.push(hw_p); // Plans
    hw_doc.rebuild_relations();
    let hw_p_related = hw_doc.sections[3].items[0].related.clone();

    // Auto-generated
    let mut ag_doc = ag::SpecsDocument::new("test");
    let mut ag_g = ag::SpecItem::new("G-1", "goal");
    ag_g.depends_on = vec!["P-1".into()];
    ag_doc.sections[0].items.push(ag_g);
    let ag_p = ag::SpecItem::new("P-1", "plan");
    ag_doc.sections[3].items.push(ag_p);
    ag_doc.rebuild_relations();
    let ag_p_related = ag_doc.sections[3].items[0].related.clone();

    assert_eq!(
        hw_p_related, ag_p_related,
        "rebuild_relations produced different reverse links: hw={hw_p_related:?} ag={ag_p_related:?}"
    );
    assert!(
        hw_p_related.contains(&"G-1".to_string()),
        "P-1.related should contain G-1"
    );
}

// ──────────────────────────────────────────────────────────
// derive_statuses — auto-advance parity (the key behavioral test)
// ──────────────────────────────────────────────────────────

// NOTE: `parity_derive_goal_implemented_when_plan_done` was removed — it
// asserted the "Goal → Implemented when all related Plans Done" derive rule,
// which `specs.rs::derive_statuses` (see comment at specs.rs:443-445)
// documents as intentionally removed once the `plans` section was dropped
// from the 6-zone spec. Plan-025 surfaced this stale test after fixing the
// `SectionType::Plans` compile error above (same plan-018 cleanup root cause).

#[test]
fn parity_derive_does_not_force_invalid_transition() {
    // Goal at Empty with a Done Plan: can_transition(Empty, Implemented) is
    // false, so the Goal stays Empty. Both versions must agree.
    let mut hw_doc = hw::SpecsDocument::new("t");
    let mut hw_g = hw::SpecItem::new("G1", "goal");
    hw_g.depends_on = vec!["P1".into()];
    hw_doc.sections[0].items.push(hw_g);
    let mut hw_p = hw::SpecItem::new("P1", "plan");
    hw_p.status = hw::SpecStatus::Done;
    hw_doc.sections[3].items.push(hw_p);
    hw_doc.rebuild_relations();
    hw_doc.derive_statuses();

    let mut ag_doc = ag::SpecsDocument::new("t");
    let mut ag_g = ag::SpecItem::new("G1", "goal");
    ag_g.depends_on = vec!["P1".into()];
    ag_doc.sections[0].items.push(ag_g);
    let mut ag_p = ag::SpecItem::new("P1", "plan");
    ag_p.status = ag::SpecStatus::Done;
    ag_doc.sections[3].items.push(ag_p);
    ag_doc.rebuild_relations();
    ag_doc.derive_statuses();

    assert_eq!(
        hw_doc.sections[0].items[0].status.to_str(),
        ag_doc.sections[0].items[0].status.to_str(),
        "Empty Goal with Done Plan: status disagrees"
    );
    // Both should be "empty" (no forcing)
    assert_eq!(hw_doc.sections[0].items[0].status.to_str(), "empty");
}

// ──────────────────────────────────────────────────────────
// SpecsStore IO parity (B 阶段: load/save/drift_check 签名对齐后)
// ──────────────────────────────────────────────────────────

#[test]
fn parity_specs_store_save_load_roundtrip() {
    let hw_dir = tempfile::tempdir().unwrap();
    let ag_dir = tempfile::tempdir().unwrap();
    let hw_path = hw_dir.path().join("specs.json");
    let ag_path = ag_dir.path().join("specs.json");
    let hw_store = hw::SpecsStore::new(hw_path.clone());
    let ag_store = ag::SpecsStore::new(ag_path);

    // Missing file → both load a fresh empty doc (NotFound fallback persists it).
    let hw_doc = hw_store.load().unwrap();
    let ag_doc = ag_store.load().unwrap();
    assert_eq!(hw_doc.project, ag_doc.project);
    assert_eq!(hw_doc.version, ag_doc.version);
    assert_eq!(hw_doc.sections.len(), ag_doc.sections.len());

    // Save + reload round-trip on both.
    hw_store.save(&hw_doc).unwrap();
    let hw_rt = hw_store.load().unwrap();
    ag_store.save(ag_doc.clone()).unwrap();
    let ag_rt = ag_store.load().unwrap();
    assert_eq!(hw_rt.project, ag_rt.project);
    assert_eq!(hw_rt.version, ag_rt.version);
    assert_eq!(hw_rt.sections.len(), ag_rt.sections.len());

    // drift_check on the just-persisted doc: not drifted on either side.
    let (_, hw_drifted) = hw_store.drift_check(&hw_rt).unwrap();
    let (_, ag_drifted) = ag_store.drift_check(ag_rt.clone()).unwrap();
    assert_eq!(hw_drifted, ag_drifted);
    assert!(!hw_drifted);
}

#[test]
fn parity_specs_store_load_corrupt_errors() {
    // Corrupt JSON → Err on both (B 阶段把 ag load 从"回退空 doc"对齐为 Err)。
    let hw_dir = tempfile::tempdir().unwrap();
    let ag_dir = tempfile::tempdir().unwrap();
    let hw_path = hw_dir.path().join("specs.json");
    std::fs::write(&hw_path, "not json{{").unwrap();
    let hw_store = hw::SpecsStore::new(hw_path.clone());
    assert!(hw_store.load().is_err());

    let ag_path = ag_dir.path().join("specs.json");
    std::fs::write(&ag_path, "not json{{").unwrap();
    let ag_store = ag::SpecsStore::new(ag_path);
    assert!(ag_store.load().is_err());
}

// ──────────────────────────────────────────────────────────
// SpecsStore write-methods parity (C1: &mut doc 对齐后)
// upsert/transition/delete 就地改 doc 且签名与 hw 同构:
//   hw: upsert(&self, doc: &mut Doc, ...) -> Result<(), String>
//   ag: upsert(&self, mut doc: &mut Doc, ...) -> Result<bool, String>
//   delete 两边都是 Result<bool, String>(removed 语义完全同构)。
// ──────────────────────────────────────────────────────────

/// Run the same write scenario on both stores and compare observable state.
fn assert_upsert_parity() {
    // Hand-written
    let hw_store = hw::SpecsStore::new(tempfile::tempdir().unwrap().path().join("hw.json"));
    let mut hw_doc = hw::SpecsDocument::new("t");
    // Auto-generated
    let ag_store = ag::SpecsStore::new(tempfile::tempdir().unwrap().path().join("ag.json"));
    let mut ag_doc = ag::SpecsDocument::new("t");

    // new item upsert → both have 1 item, version 1
    hw_store
        .upsert_item(&mut hw_doc, "goals", hw::SpecItem::new("G1", "goal"))
        .unwrap();
    ag_store
        .upsert_item(&mut ag_doc, "goals", ag::SpecItem::new("G1", "goal"))
        .unwrap();
    assert_eq!(hw_doc.version, 1);
    assert_eq!(ag_doc.version, 1);
    assert_eq!(hw_doc.sections[0].items.len(), 1);
    assert_eq!(ag_doc.sections[0].items.len(), 1);

    // same-id upsert → replaces (no duplicate), title updated
    hw_store
        .upsert_item(&mut hw_doc, "goals", hw::SpecItem::new("G1", "new title"))
        .unwrap();
    ag_store
        .upsert_item(&mut ag_doc, "goals", ag::SpecItem::new("G1", "new title"))
        .unwrap();
    assert_eq!(hw_doc.sections[0].items.len(), 1, "hw: replace must not duplicate");
    assert_eq!(ag_doc.sections[0].items.len(), 1, "ag: replace must not duplicate");
    assert_eq!(hw_doc.sections[0].items[0].title, "new title");
    assert_eq!(ag_doc.sections[0].items[0].title, "new title");
    assert_eq!(hw_doc.version, 2);
    assert_eq!(ag_doc.version, 2);

    // unknown section → both Err (error string identical)
    let hw_err = hw_store
        .upsert_item(&mut hw_doc, "nonexistent", hw::SpecItem::new("X1", "x"))
        .unwrap_err();
    let ag_err = ag_store
        .upsert_item(&mut ag_doc, "nonexistent", ag::SpecItem::new("X1", "x"))
        .unwrap_err();
    assert!(hw_err.contains("not found"));
    assert!(ag_err.contains("not found"));
    assert_eq!(hw_err, ag_err, "section-not-found error strings must match");
}

#[test]
fn parity_store_upsert_in_place() {
    assert_upsert_parity();
}

/// Transition parity: valid transitions advance both; invalid → Err on both;
/// Done sets completed_at on both.
#[test]
fn parity_store_transition_in_place() {
    let hw_store = hw::SpecsStore::new(tempfile::tempdir().unwrap().path().join("hw.json"));
    let mut hw_doc = hw::SpecsDocument::new("t");
    let ag_store = ag::SpecsStore::new(tempfile::tempdir().unwrap().path().join("ag.json"));
    let mut ag_doc = ag::SpecsDocument::new("t");

    hw_store
        .upsert_item(&mut hw_doc, "goals", hw::SpecItem::new("G1", "g"))
        .unwrap();
    ag_store
        .upsert_item(&mut ag_doc, "goals", ag::SpecItem::new("G1", "g"))
        .unwrap();

    // Valid: Empty -> Proposed (both ok)
    hw_store
        .transition_item(&mut hw_doc, "goals", "G1", hw::SpecStatus::Proposed)
        .unwrap();
    ag_store
        .transition_item(&mut ag_doc, "goals", "G1", ag::SpecStatus::Proposed)
        .unwrap();
    assert_eq!(hw_doc.sections[0].items[0].status.to_str(), "proposed");
    assert_eq!(ag_doc.sections[0].items[0].status.to_str(), "proposed");

    // Invalid: Proposed -> Done (skips the Goals machine) → both Err
    let hw_err = hw_store
        .transition_item(&mut hw_doc, "goals", "G1", hw::SpecStatus::Done)
        .unwrap_err();
    let ag_err = ag_store
        .transition_item(&mut ag_doc, "goals", "G1", ag::SpecStatus::Done)
        .unwrap_err();
    assert!(hw_err.contains("invalid"));
    assert!(ag_err.contains("invalid"));

    // Unknown item → both Err with identical message
    let hw_err2 = hw_store
        .transition_item(&mut hw_doc, "goals", "NOPE", hw::SpecStatus::Proposed)
        .unwrap_err();
    let ag_err2 = ag_store
        .transition_item(&mut ag_doc, "goals", "NOPE", ag::SpecStatus::Proposed)
        .unwrap_err();
    assert_eq!(hw_err2, ag_err2, "item-not-found error strings must match");

    // Walk the Goals machine to Done → completed_at set on both.
    let hw_chain: Vec<hw::SpecStatus> = vec![
        hw::SpecStatus::Analysed,
        hw::SpecStatus::Approved,
        hw::SpecStatus::InProgress,
        hw::SpecStatus::Implemented,
        hw::SpecStatus::Verified,
        hw::SpecStatus::Done,
    ];
    let ag_chain: Vec<ag::SpecStatus> = vec![
        ag::SpecStatus::Analysed,
        ag::SpecStatus::Approved,
        ag::SpecStatus::InProgress,
        ag::SpecStatus::Implemented,
        ag::SpecStatus::Verified,
        ag::SpecStatus::Done,
    ];
    for s in hw_chain {
        hw_store
            .transition_item(&mut hw_doc, "goals", "G1", s)
            .unwrap();
    }
    for s in ag_chain {
        ag_store
            .transition_item(&mut ag_doc, "goals", "G1", s)
            .unwrap();
    }
    assert_eq!(hw_doc.sections[0].items[0].status.to_str(), "done");
    assert_eq!(ag_doc.sections[0].items[0].status.to_str(), "done");
    assert!(hw_doc.sections[0].items[0].completed_at.is_some());
    assert!(ag_doc.sections[0].items[0].completed_at.is_some());
}

/// Delete parity: first delete returns true (removed), second returns false
/// (already gone); both agree on the final empty item list.
#[test]
fn parity_store_delete_in_place() {
    let hw_store = hw::SpecsStore::new(tempfile::tempdir().unwrap().path().join("hw.json"));
    let mut hw_doc = hw::SpecsDocument::new("t");
    let ag_store = ag::SpecsStore::new(tempfile::tempdir().unwrap().path().join("ag.json"));
    let mut ag_doc = ag::SpecsDocument::new("t");

    hw_store
        .upsert_item(&mut hw_doc, "goals", hw::SpecItem::new("G1", "g"))
        .unwrap();
    ag_store
        .upsert_item(&mut ag_doc, "goals", ag::SpecItem::new("G1", "g"))
        .unwrap();

    assert!(hw_store.delete_item(&mut hw_doc, "goals", "G1").unwrap());
    assert!(ag_store.delete_item(&mut ag_doc, "goals", "G1").unwrap());
    assert!(!hw_store.delete_item(&mut hw_doc, "goals", "G1").unwrap());
    assert!(!ag_store.delete_item(&mut ag_doc, "goals", "G1").unwrap());
    assert!(hw_doc.sections[0].items.is_empty());
    assert!(ag_doc.sections[0].items.is_empty());

    // Unknown section → both Err with identical message.
    let hw_err = hw_store.delete_item(&mut hw_doc, "nope", "G1").unwrap_err();
    let ag_err = ag_store.delete_item(&mut ag_doc, "nope", "G1").unwrap_err();
    assert_eq!(hw_err, ag_err, "delete section-not-found error strings must match");
}
