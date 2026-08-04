//! Parity tests — verify `auto_generated::auth` behaves identically to the
//! hand-written `auth` module.
//!
//! Same framework as `parity_specs.rs`: exercise the same scenarios on both
//! the hand-written Rust (`musk::auth`) and the a2r-transpiled Auto output
//! (`musk::auto_generated::auth`), asserting both produce equal results.
//!
//! Focus areas (per Plan 018 §9 auth row):
//! - RBAC pure logic (Role::permissions / Role::allows) — hw returns
//!   `&'static [Permission]`, ag returns `Vec<Permission>`; we compare the
//!   *sets* and the allows matrix, which is the observable behavior.
//! - Wire format (serde) — the C4 .at fixes must keep the JSON identical.
//! - AuthStore session lifecycle — the 4 Mutex-guarded methods ported in
//!   `3797fd7` (login/session_user/token_allows/logout) had compile-time
//!   verification only; these tests give them behavioral verification.

use std::path::PathBuf;

use musk::auth as hw;                 // hand-written
use musk::auto_generated::auth as ag; // a2r-transpiled Auto

// ──────────────────────────────────────────────────────────
// Role / Permission — RBAC logic parity
// ──────────────────────────────────────────────────────────

/// Serialize a permission to its wire form (both versions derive Serialize).
fn perm_json(p: &impl serde::Serialize) -> String {
    serde_json::to_string(p).unwrap()
}

#[test]
fn parity_role_permission_sets() {
    // Admin → all; Developer → run/edit/read; Viewer → read-only.
    let admin_hw: Vec<String> = hw::Role::Admin.permissions().iter().map(perm_json).collect();
    let admin_ag: Vec<String> = ag::Role::Admin.permissions().iter().map(perm_json).collect();
    let dev_hw: Vec<String> = hw::Role::Developer.permissions().iter().map(perm_json).collect();
    let dev_ag: Vec<String> = ag::Role::Developer.permissions().iter().map(perm_json).collect();
    let view_hw: Vec<String> = hw::Role::Viewer.permissions().iter().map(perm_json).collect();
    let view_ag: Vec<String> = ag::Role::Viewer.permissions().iter().map(perm_json).collect();

    assert_eq!(admin_hw, admin_ag, "Admin permission set mismatch");
    assert_eq!(dev_hw, dev_ag, "Developer permission set mismatch");
    assert_eq!(view_hw, view_ag, "Viewer permission set mismatch");

    // Sanity: the sets themselves are what the MVP spec expects.
    assert_eq!(admin_hw.len(), 4);
    assert_eq!(dev_hw, vec!["\"RunAgent\"", "\"EditSpecs\"", "\"Read\""]);
    assert_eq!(view_hw, vec!["\"Read\""]);
}

#[test]
fn parity_role_allows_matrix() {
    // Every (role, permission) pair must agree between the two versions.
    let roles_hw = [hw::Role::Admin, hw::Role::Developer, hw::Role::Viewer];
    let roles_ag = [ag::Role::Admin, ag::Role::Developer, ag::Role::Viewer];
    let perms_hw = [
        (hw::Permission::RunAgent, "RunAgent"),
        (hw::Permission::EditSpecs, "EditSpecs"),
        (hw::Permission::Read, "Read"),
        (hw::Permission::ManageUsers, "ManageUsers"),
    ];
    let perms_ag = [
        (ag::Permission::RunAgent, "RunAgent"),
        (ag::Permission::EditSpecs, "EditSpecs"),
        (ag::Permission::Read, "Read"),
        (ag::Permission::ManageUsers, "ManageUsers"),
    ];

    for (rh, ra) in roles_hw.iter().zip(roles_ag.iter()) {
        for ((ph, name), (pa, _)) in perms_hw.iter().zip(perms_ag.iter()) {
            assert_eq!(
                (*rh).allows(*ph),
                ra.allows(*pa),
                "allows mismatch for role x {name}"
            );
        }
    }
}

// ──────────────────────────────────────────────────────────
// Wire format — serde parity (C4 guarantees)
// ──────────────────────────────────────────────────────────

#[test]
fn parity_serde_wire_format() {
    // Same JSON on the wire for User/Role/Permission/UserInfo.
    let hw_u = hw::User {
        username: "alice".into(),
        role: hw::Role::Developer,
        password_hash: "cafe".into(),
        salt: "salt".into(),
    };
    let ag_u = ag::User {
        username: "alice".into(),
        role: ag::Role::Developer,
        password_hash: "cafe".into(),
        salt: "salt".into(),
    };
    assert_eq!(
        serde_json::to_string(&hw_u).unwrap(),
        serde_json::to_string(&ag_u).unwrap(),
        "User wire format mismatch"
    );
    assert_eq!(
        serde_json::to_string(&hw::Role::Viewer).unwrap(),
        serde_json::to_string(&ag::Role::Viewer).unwrap(),
        "Role wire format mismatch"
    );
    assert_eq!(
        serde_json::to_string(&hw::Permission::EditSpecs).unwrap(),
        serde_json::to_string(&ag::Permission::EditSpecs).unwrap(),
        "Permission wire format mismatch"
    );

    // UserInfo — hw uses `From<&User>`, ag uses the static `from_user`.
    let hw_info = hw::UserInfo::from(&hw_u);
    let ag_info = ag::UserInfo::from_user(ag_u.clone());
    assert_eq!(
        serde_json::to_string(&hw_info).unwrap(),
        serde_json::to_string(&ag_info).unwrap(),
        "UserInfo wire format mismatch"
    );
}

// ──────────────────────────────────────────────────────────
// AuthStore — session lifecycle parity (Mutex-guarded methods)
// ──────────────────────────────────────────────────────────

fn tmp_path(name: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("musk_auth_parity_{name}.json"));
    let _ = std::fs::remove_file(&p);
    p
}

/// A pair of freshly-seeded stores (hw + ag) on separate temp files, with the
/// files cleaned up when the test drops.
struct Stores {
    hw: hw::AuthStore,
    ag: ag::AuthStore,
    _paths: Vec<PathBuf>,
}

fn stores(name: &str) -> Stores {
    let hw_path = tmp_path(&format!("{name}_hw"));
    let ag_path = tmp_path(&format!("{name}_ag"));
    Stores {
        hw: hw::AuthStore::new(&hw_path),
        ag: ag::AuthStore::new(ag_path.clone()),
        _paths: vec![hw_path, ag_path],
    }
}

impl Drop for Stores {
    fn drop(&mut self) {
        for p in &self._paths {
            let _ = std::fs::remove_file(p);
        }
    }
}

#[test]
fn parity_default_admin_login() {
    let s = stores("login");
    let hw_s = s.hw.login("admin", "admin").expect("hw default admin should log in");
    let ag_s = s.ag.login("admin", "admin").expect("ag default admin should log in");
    assert_eq!(hw_s.username, "admin");
    assert_eq!(ag_s.username, "admin");
    assert!(!hw_s.token.is_empty());
    assert!(!ag_s.token.is_empty());
}

#[test]
fn parity_wrong_password_fails() {
    let s = stores("wrong");
    assert!(s.hw.login("admin", "wrong").is_none());
    assert!(s.ag.login("admin", "wrong").is_none());
}

#[test]
fn parity_session_user_resolves() {
    let s = stores("session");
    let hw_user = s
        .hw
        .session_user(&s.hw.login("admin", "admin").unwrap().token)
        .unwrap();
    let ag_user = s
        .ag
        .session_user(&s.ag.login("admin", "admin").unwrap().token)
        .unwrap();
    assert_eq!(hw_user.username, "admin");
    assert_eq!(hw_user.role, hw::Role::Admin);
    assert_eq!(ag_user.username, "admin");
    assert_eq!(ag_user.role, ag::Role::Admin);
}

#[test]
fn parity_token_allows_checks_permission() {
    let s = stores("allows");
    let hw_token = s.hw.login("admin", "admin").unwrap().token;
    let ag_token = s.ag.login("admin", "admin").unwrap().token;
    assert!(s.hw.token_allows(&hw_token, hw::Permission::RunAgent));
    assert!(s.hw.token_allows(&hw_token, hw::Permission::ManageUsers));
    assert!(s.ag.token_allows(&ag_token, ag::Permission::RunAgent));
    assert!(s.ag.token_allows(&ag_token, ag::Permission::ManageUsers));
    // Unknown tokens deny everything in both versions.
    assert!(!s.hw.token_allows("bogus-token", hw::Permission::Read));
    assert!(!s.ag.token_allows("bogus-token", ag::Permission::Read));
}

#[test]
fn parity_logout_invalidates_session() {
    let s = stores("logout");
    let hw_token = s.hw.login("admin", "admin").unwrap().token;
    let ag_token = s.ag.login("admin", "admin").unwrap().token;
    assert!(s.hw.session_user(&hw_token).is_some());
    assert!(s.ag.session_user(&ag_token).is_some());
    s.hw.logout(&hw_token);
    s.ag.logout(&ag_token);
    assert!(s.hw.session_user(&hw_token).is_none());
    assert!(s.ag.session_user(&ag_token).is_none());
}
