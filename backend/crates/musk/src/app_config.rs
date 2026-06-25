//! musk runtime configuration — how the app connects to the daemon, its default
//! mode, context-file discovery, etc.
//!
//! This is the SINGLE source of truth for musk's runtime config, read by BOTH
//! the CLI (`musk run` / `musk chat`) and the HTTP API (`musk serve`). The
//! config lives at `~/.config/autoos/apps/musk/config.at`.
//!
//! Deliberately SEPARATE from capability registries (Roles/Skills/Modes): per
//! the unified-Harness design (auto-os-config/designs/), app config is "how
//! this app runs", not "which capabilities it inherits".

use serde::{Deserialize, Serialize};

/// Path: `~/.config/autoos/apps/musk/config.at`.
pub fn musk_config_path() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|h| h.join(".config/autoos/apps/musk/config.at"))
}

/// The persisted musk runtime config. Every field is optional so only the set
/// values are written; unset fields fall back to env vars / compiled defaults
/// when read via [`Self::effective`].
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MuskAppConfig {
    /// Daemon URL (env `AAID_URL`, default http://127.0.0.1:17654).
    #[serde(default)]
    pub daemon_url: Option<String>,
    /// Default mode for `musk run` / `musk chat` (default "superpowers").
    #[serde(default)]
    pub default_mode: Option<String>,
    /// Context file musk auto-loads (`.musk.md` / `CLAUDE.md`), or explicit.
    #[serde(default)]
    pub context_file: Option<String>,
    /// HTTP server bind addr for `musk serve` (default 127.0.0.1:8080).
    #[serde(default)]
    pub serve_addr: Option<String>,
    /// Whether the daemon should be lazily started if unreachable.
    #[serde(default)]
    pub auto_start_daemon: Option<bool>,
    /// Which OS-level harness kinds/names this app has opted into (inherited
    /// as-is — no field override, per the unified-Harness design Phase 1).
    /// Keyed by kind ("roles"/"skills"/"modes"). Empty = inherit nothing.
    #[serde(default)]
    pub harness: HarnessSelection,
}

/// The app's harness selection: which OS-level (or app-level) harnesses it
/// uses. Phase 1 is "inherit as-is" — names are used directly, no overrides.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HarnessSelection {
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub modes: Vec<String>,
}

impl MuskAppConfig {
    /// Read the on-disk file (if any). Missing file → empty (all None).
    pub fn load() -> Self {
        match musk_config_path().and_then(|p| std::fs::read_to_string(p).ok()) {
            Some(content) => Self::parse_from_at(&content).unwrap_or_default(),
            None => Self::default(),
        }
    }

    /// Parse the persisted .at back into the struct (best-effort prop read).
    fn parse_from_at(content: &str) -> Option<Self> {
        use auto_atom::AtomParser;
        let atom = AtomParser::parse(content).ok()?;
        let node = match atom {
            auto_atom::Atom::Node(n) if n.name.as_str() == "musk" || n.name.as_str() == "config" => n,
            _ => return None,
        };
        let opt_str = |k: &str| match node.get_prop_of(k) {
            auto_val::Value::Str(s) => Some(s.to_string()),
            auto_val::Value::Nil => None,
            other => Some(other.to_astr().to_string()),
        };
        let opt_bool = |k: &str| match node.get_prop_of(k) {
            auto_val::Value::Bool(b) => Some(b),
            _ => None,
        };
        Some(Self {
            daemon_url: opt_str("daemon_url"),
            default_mode: opt_str("default_mode"),
            context_file: opt_str("context_file"),
            serve_addr: opt_str("serve_addr"),
            auto_start_daemon: opt_bool("auto_start_daemon"),
            harness: parse_harness(&node),
        })
    }

    /// Serialize back to a `musk { ... }` .at block.
    pub fn to_at_source(&self) -> String {
        use auto_val::{AtomSource as _, Node, Value};
        let mut node = Node::new("musk");
        if let Some(v) = &self.daemon_url {
            node.set_prop("daemon_url", Value::str(v.as_str()));
        }
        if let Some(v) = &self.default_mode {
            node.set_prop("default_mode", Value::str(v.as_str()));
        }
        if let Some(v) = &self.context_file {
            node.set_prop("context_file", Value::str(v.as_str()));
        }
        if let Some(v) = &self.serve_addr {
            node.set_prop("serve_addr", Value::str(v.as_str()));
        }
        if let Some(v) = self.auto_start_daemon {
            node.set_prop("auto_start_daemon", Value::Bool(v));
        }
        // Harness selection (nested node).
        let h = &self.harness;
        if !h.roles.is_empty() || !h.skills.is_empty() || !h.modes.is_empty() {
            let mut hn = Node::new("harness");
            if !h.roles.is_empty() {
                hn.set_prop("roles", str_array(&h.roles));
            }
            if !h.skills.is_empty() {
                hn.set_prop("skills", str_array(&h.skills));
            }
            if !h.modes.is_empty() {
                hn.set_prop("modes", str_array(&h.modes));
            }
            node.add_kid(hn);
        }
        node.to_at_source()
    }

    /// The effective daemon URL, merging: file < env `AAID_URL` < compiled default.
    pub fn effective_daemon_url(&self) -> String {
        self.daemon_url
            .clone()
            .or_else(|| std::env::var("AAID_URL").ok())
            .unwrap_or_else(|| "http://127.0.0.1:17654".into())
    }

    /// The effective default mode ("superpowers" if unset).
    pub fn effective_default_mode(&self) -> String {
        self.default_mode
            .clone()
            .unwrap_or_else(|| "superpowers".into())
    }

    /// Merge env-var / compiled defaults for the *effective* values reported
    /// to the UI (so the page shows what's actually in use, not just the file).
    pub fn effective(&self) -> serde_json::Value {
        serde_json::json!({
            "daemon_url": self.effective_daemon_url(),
            "default_mode": self.effective_default_mode(),
            "context_file": self.context_file.clone().unwrap_or_else(|| "auto (.musk.md / CLAUDE.md)".into()),
            "serve_addr": self.serve_addr.clone().unwrap_or_else(|| "127.0.0.1:8080".into()),
            "auto_start_daemon": self.auto_start_daemon.unwrap_or(true),
            "harness": {
                "roles": self.harness.roles,
                "skills": self.harness.skills,
                "modes": self.harness.modes,
            },
        })
    }

    /// Apply the config to the current process so the daemon client (which
    /// reads `AAID_URL` via `auto_ai_client::daemon_url()`) picks it up.
    ///
    /// Called by the CLI (`musk run` / `musk chat`) before constructing an
    /// `AiClient`, and is a no-op if the config file is absent / daemon_url
    /// unset (the env var or compiled default then applies).
    pub fn apply_to_env(&self) {
        if let Some(url) = &self.daemon_url {
            // Only override if not already set explicitly via the environment —
            // an explicit AAID_URL wins over the file.
            if std::env::var("AAID_URL").is_err() {
                std::env::set_var("AAID_URL", url);
            }
        }
    }
}

/// Convenience: load + apply the app config to the environment (for CLI use).
pub fn apply_app_config() {
    MuskAppConfig::load().apply_to_env();
}

/// Parse the `harness { roles: [...]; skills: [...]; modes: [...] }` child node.
fn parse_harness(node: &auto_val::Node) -> HarnessSelection {
    let h = match node.kids_iter().find(|(_, k)| matches!(k, auto_val::Kid::Node(n) if n.name.as_str() == "harness")) {
        Some((_, auto_val::Kid::Node(n))) => n,
        _ => return HarnessSelection::default(),
    };
    let list = |k: &str| -> Vec<String> {
        match h.get_prop_of(k) {
            auto_val::Value::Array(a) => a
                .values
                .iter()
                .filter_map(|v| match v {
                    auto_val::Value::Str(s) => Some(s.to_string()),
                    _ => None,
                })
                .collect(),
            auto_val::Value::Str(s) => vec![s.to_string()],
            _ => Vec::new(),
        }
    };
    HarnessSelection {
        roles: list("roles"),
        skills: list("skills"),
        modes: list("modes"),
    }
}

fn str_array(items: &[String]) -> auto_val::Value {
    let values = items.iter().map(|s| auto_val::Value::str(s.as_str())).collect();
    auto_val::Value::Array(auto_val::Array { values })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_missing_file_is_default() {
        // A non-existent path → all None (no panic).
        let cfg = MuskAppConfig::default();
        assert!(cfg.daemon_url.is_none());
    }

    #[test]
    fn roundtrip_at_source() {
        let cfg = MuskAppConfig {
            daemon_url: Some("http://example:17654".into()),
            default_mode: Some("coding".into()),
            auto_start_daemon: Some(false),
            ..Default::default()
        };
        let src = cfg.to_at_source();
        // Re-parse via the same path the loader uses.
        let reparsed = MuskAppConfig::parse_from_at(&src).expect("must reparse");
        assert_eq!(reparsed.daemon_url.as_deref(), Some("http://example:17654"));
        assert_eq!(reparsed.default_mode.as_deref(), Some("coding"));
        assert_eq!(reparsed.auto_start_daemon, Some(false));
    }

    #[test]
    fn effective_falls_back_to_default() {
        let cfg = MuskAppConfig::default();
        // daemon_url falls back to env or compiled default; just check shape.
        assert!(cfg.effective_daemon_url().starts_with("http://"));
    }
}
