//! Minimal RBAC + session auth for auto-musk (MVP).
//!
//! Scope: enough to gate the agent/workflow/specs endpoints behind a login.
//! - Users + roles stored in a JSON file (`users.json`).
//! - Passwords hashed with SHA-256 + per-user salt (NOT production-grade —
//!   swap for argon2 before real use; flagged).
//! - Sessions are bearer tokens (random 32-byte hex) kept in memory; a restart
//!   logs everyone out (acceptable for a single-user dev tool).
//!
//! Roles → permissions: admin (all), developer (run/workflows/specs read-write),
//! viewer (read-only). `Permission::allows()` decides per action.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

/// A capability a role may or may not have.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    /// Run agents and workflows.
    RunAgent,
    /// Read/write spec ledger.
    EditSpecs,
    /// Read anything (specs, professions, workflows).
    Read,
    /// Manage users (create/role-change).
    ManageUsers,
}

/// Coarse roles mapped to permission sets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Admin,
    Developer,
    Viewer,
}

impl Role {
    pub fn permissions(self) -> &'static [Permission] {
        use Permission::*;
        match self {
            Role::Admin => &[RunAgent, EditSpecs, Read, ManageUsers],
            Role::Developer => &[RunAgent, EditSpecs, Read],
            Role::Viewer => &[Read],
        }
    }

    pub fn allows(self, perm: Permission) -> bool {
        self.permissions().contains(&perm)
    }
}

/// A persisted user (password hashed + salted).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub role: Role,
    /// hex(SHA-256(salt + password)).
    pub password_hash: String,
    /// hex random 16 bytes.
    pub salt: String,
}

/// Public user shape (no secret material).
#[derive(Clone, Debug, Serialize)]
pub struct UserInfo {
    pub username: String,
    pub role: Role,
}

impl From<&User> for UserInfo {
    fn from(u: &User) -> Self {
        UserInfo {
            username: u.username.clone(),
            role: u.role,
        }
    }
}

/// A live session (bearer token → username).
#[derive(Clone, Debug)]
pub struct Session {
    pub token: String,
    pub username: String,
}

/// The auth store: persisted users + in-memory sessions.
pub struct AuthStore {
    users_path: PathBuf,
    sessions: Mutex<HashMap<String, String>>, // token -> username
}

impl AuthStore {
    /// Open at `users.json`. Seeds a default admin if the file is absent.
    pub fn new(users_path: impl Into<PathBuf>) -> Self {
        let users_path = users_path.into();
        let store = Self {
            users_path,
            sessions: Mutex::new(HashMap::new()),
        };
        store.ensure_default_admin();
        store
    }

    fn ensure_default_admin(&self) {
        if self.load_users().map(|u| !u.is_empty()).unwrap_or(false) {
            return;
        }
        let salt = random_hex(16);
        let hash = hash_password("admin", &salt);
        let admin = User {
            username: "admin".into(),
            role: Role::Admin,
            password_hash: hash,
            salt,
        };
        let _ = self.save_users(&[admin]);
        tracing::info!("auth: seeded default admin user (admin / admin) — change the password!");
    }

    fn load_users(&self) -> std::io::Result<Vec<User>> {
        match std::fs::read(&self.users_path) {
            Ok(b) => serde_json::from_slice(&b)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(e) => Err(e),
        }
    }

    fn save_users(&self, users: &[User]) -> std::io::Result<()> {
        if let Some(p) = self.users_path.parent() {
            std::fs::create_dir_all(p)?;
        }
        let bytes = serde_json::to_vec_pretty(users)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(&self.users_path, bytes)
    }

    /// Verify credentials; on success create a session + return its token.
    pub fn login(&self, username: &str, password: &str) -> Option<Session> {
        let users = self.load_users().ok()?;
        let user = users.iter().find(|u| u.username == username)?;
        let candidate = hash_password(password, &user.salt);
        if candidate != user.password_hash {
            return None;
        }
        let token = random_hex(32);
        self.sessions
            .lock()
            .unwrap()
            .insert(token.clone(), username.to_string());
        Some(Session {
            token,
            username: username.to_string(),
        })
    }

    /// Look up the user for a session token (None if unknown/expired).
    pub fn session_user(&self, token: &str) -> Option<UserInfo> {
        let username = self.sessions.lock().unwrap().get(token)?.clone();
        let users = self.load_users().ok()?;
        users
            .iter()
            .find(|u| u.username == username)
            .map(UserInfo::from)
    }

    /// Does the bearer token's user have `perm`?
    pub fn token_allows(&self, token: &str, perm: Permission) -> bool {
        self.session_user(token)
            .map(|u| u.role.allows(perm))
            .unwrap_or(false)
    }

    /// Log out a session (no-op if unknown).
    pub fn logout(&self, token: &str) {
        self.sessions.lock().unwrap().remove(token);
    }
}

// ── password hashing (MVP; swap for argon2 later) ──────────────────────────

fn hash_password(password: &str, salt: &str) -> String {
    // SHA-256(salt || password). Not constant-time, not slow — MVP only.
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(password.as_bytes());
    hex::encode(hasher.finalize())
}

fn random_hex(nbytes: usize) -> String {
    use rand::RngCore;
    let mut buf = vec![0u8; nbytes];
    rand::thread_rng().fill_bytes(&mut buf);
    hex::encode(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_store(name: &str) -> (AuthStore, PathBuf) {
        let path = std::env::temp_dir().join(format!("musk_auth_{name}.json"));
        let _ = std::fs::remove_file(&path);
        let store = AuthStore::new(&path);
        (store, path)
    }

    #[test]
    fn roles_permissions() {
        assert!(Role::Admin.allows(Permission::ManageUsers));
        assert!(!Role::Developer.allows(Permission::ManageUsers));
        assert!(Role::Developer.allows(Permission::RunAgent));
        assert!(!Role::Viewer.allows(Permission::RunAgent));
        assert!(Role::Viewer.allows(Permission::Read));
    }

    #[test]
    fn default_admin_seeded() {
        let (store, path) = tmp_store("seed");
        let session = store.login("admin", "admin").expect("default admin should log in");
        assert_eq!(session.username, "admin");
        assert!(!session.token.is_empty());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn wrong_password_fails() {
        let (store, path) = tmp_store("wrong");
        assert!(store.login("admin", "wrong").is_none());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn session_user_resolves() {
        let (store, path) = tmp_store("session");
        let session = store.login("admin", "admin").unwrap();
        let user = store.session_user(&session.token).unwrap();
        assert_eq!(user.username, "admin");
        assert_eq!(user.role, Role::Admin);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn token_allows_checks_permission() {
        let (store, path) = tmp_store("allows");
        let session = store.login("admin", "admin").unwrap();
        assert!(store.token_allows(&session.token, Permission::RunAgent));
        assert!(store.token_allows(&session.token, Permission::ManageUsers));
        assert!(!store.token_allows("bogus-token", Permission::Read));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn logout_invalidates_session() {
        let (store, path) = tmp_store("logout");
        let session = store.login("admin", "admin").unwrap();
        assert!(store.session_user(&session.token).is_some());
        store.logout(&session.token);
        assert!(store.session_user(&session.token).is_none());
        let _ = std::fs::remove_file(path);
    }
}
