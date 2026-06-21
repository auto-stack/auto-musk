//! Agent 运行模式 —— 每种模式是一个 `.at` 配置,声明该模式的全部设置。
//!
//! 模式 = 一组 agent 配置的声明式集合:profession、工具集、技能开关、
//! 可选的 workflow、上下文文件、额外 system prompt。用户可以在
//! `~/.config/autoos/modes/` 放自定义模式 `.at` 文件,不用改代码。
//!
//! 内置模式(superpowers/basic/coding/review)嵌入二进制,用户模式覆盖同名内置。

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// 一种运行模式的配置。
#[derive(Clone, Debug)]
pub struct AgentMode {
    /// 模式名(唯一 key),如 "superpowers"。
    pub name: String,
    /// 描述(CLI help 用)。
    pub description: String,
    /// 使用的 profession(内置名或 .at 路径)。
    pub profession: String,
    /// 是否启用技能系统(Superpowers)。
    pub skills: bool,
    /// 工具白名单(空 = 所有已注册工具)。名字匹配 Tool::name()。
    pub tools: Vec<String>,
    /// 可选:绑定的 workflow 名称(如 "feature-dev")。
    pub workflow: Option<String>,
    /// 可选:上下文文件(如 ".musk.md"),空 = 自动发现。
    pub context_file: String,
    /// 可选:额外 system prompt(拼在 profession prompt 之后)。
    pub extra_system_prompt: String,
}

/// 内置模式名(嵌入二进制的 .at 文件)。
pub const BUILTIN_MODES: &[(&str, &str)] = &[
    ("superpowers", include_str!("../modes/superpowers.at")),
    ("basic", include_str!("../modes/basic.at")),
    ("coding", include_str!("../modes/coding.at")),
    ("review", include_str!("../modes/review.at")),
];

/// 模式注册表:内置 + 用户自定义(`~/.config/autoos/modes/`)。
/// 用户模式同名时覆盖内置。
#[derive(Default)]
pub struct ModeRegistry {
    modes: HashMap<String, AgentMode>,
}

impl ModeRegistry {
    /// 加载所有模式:内置先加载,用户目录覆盖同名。
    pub fn load() -> Self {
        let mut modes = HashMap::new();

        // 1. 内置模式(嵌入二进制)。
        for (name, content) in BUILTIN_MODES {
            if let Ok(mode) = parse_mode_at(content) {
                modes.insert(name.to_string(), mode);
            } else {
                tracing::warn!("mode: failed to parse builtin '{name}'");
            }
        }

        // 2. 用户自定义模式(覆盖同名内置)。
        if let Some(dir) = dirs::home_dir().map(|h| h.join(".config/autoos/modes")) {
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("at") {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Ok(mode) = parse_mode_at(&content) {
                                tracing::info!(
                                    "mode: loaded '{}' from {}",
                                    mode.name,
                                    path.display()
                                );
                                modes.insert(mode.name.clone(), mode);
                            }
                        }
                    }
                }
            }
        }

        tracing::info!("mode registry: {} mode(s) loaded", modes.len());
        Self { modes }
    }

    /// 查找一个模式。None 时回退到默认(superpowers)。
    pub fn get(&self, name: &str) -> Option<&AgentMode> {
        self.modes.get(name)
    }

    /// 所有模式名(字母排序)。
    pub fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.modes.keys().cloned().collect();
        names.sort();
        names
    }

    /// 默认模式名。
    pub const DEFAULT: &'static str = "superpowers";
}

/// 解析一个 `mode { ... }` .at 文件。
fn parse_mode_at(content: &str) -> Result<AgentMode, String> {
    use auto_atom::{Atom, AtomParser};

    let atom = AtomParser::parse(content)
        .map_err(|e| format!("parse error: {e}"))?;

    let node = match atom {
        Atom::Node(n) if n.name.as_str() == "mode" => n,
        Atom::Node(n) => return Err(format!("expected 'mode' root, found '{}'", n.name)),
        other => return Err(format!("expected 'mode' node, found {other:?}")),
    };

    let name = opt_str(&node, "name").ok_or("missing 'name'")?;
    let description = opt_str(&node, "description").unwrap_or_default();
    let profession = opt_str(&node, "profession").unwrap_or_else(|| "coder".into());
    let skills = opt_bool(&node, "skills").unwrap_or(false);
    let tools = opt_str_list(&node, "tools");
    let workflow = opt_str(&node, "workflow");
    let context_file = opt_str(&node, "context_file").unwrap_or_default();
    let extra_system_prompt = opt_str(&node, "extra_system_prompt").unwrap_or_default();

    Ok(AgentMode {
        name,
        description,
        profession,
        skills,
        tools,
        workflow,
        context_file,
        extra_system_prompt,
    })
}

// ── auto-atom 导航辅助(与 workflow.rs 的模式一致) ───────────────────────

fn opt_str(node: &auto_val::Node, key: &str) -> Option<String> {
    match node.get_prop_of(key) {
        auto_val::Value::Str(s) => Some(s.to_string()),
        auto_val::Value::Nil => None,
        other => Some(other.to_astr().to_string()),
    }
}

fn opt_bool(node: &auto_val::Node, key: &str) -> Option<bool> {
    match node.get_prop_of(key) {
        auto_val::Value::Bool(b) => Some(b),
        auto_val::Value::Nil => None,
        other => Some(other.to_astr().to_string() == "true"),
    }
}

fn opt_str_list(node: &auto_val::Node, key: &str) -> Vec<String> {
    match node.get_prop_of(key) {
        auto_val::Value::Array(arr) => arr
            .values
            .iter()
            .map(|v| v.to_astr().to_string())
            .collect(),
        auto_val::Value::Nil => Vec::new(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_mode_superpowers() {
        let src = BUILTIN_MODES[0].1; // superpowers.at
        let mode = parse_mode_at(src).unwrap();
        assert_eq!(mode.name, "superpowers");
        assert!(mode.skills);
        assert_eq!(mode.profession, "coder");
        assert!(!mode.tools.is_empty()); // has tool list
    }

    #[test]
    fn parse_mode_basic() {
        let mode = parse_mode_at(BUILTIN_MODES[1].1).unwrap();
        assert_eq!(mode.name, "basic");
        assert!(!mode.skills); // basic has no skills
    }

    #[test]
    fn parse_mode_coding() {
        let mode = parse_mode_at(BUILTIN_MODES[2].1).unwrap();
        assert_eq!(mode.name, "coding");
        assert!(mode.skills); // coding has TDD skill
    }

    #[test]
    fn parse_mode_review() {
        let mode = parse_mode_at(BUILTIN_MODES[3].1).unwrap();
        assert_eq!(mode.name, "review");
        assert_eq!(mode.profession, "reviewer");
    }

    #[test]
    fn registry_loads_all_builtins() {
        let reg = ModeRegistry::load();
        let names = reg.names();
        assert!(names.contains(&"superpowers".into()));
        assert!(names.contains(&"basic".into()));
        assert!(names.contains(&"coding".into()));
        assert!(names.contains(&"review".into()));
    }

    #[test]
    fn registry_get_returns_mode() {
        let reg = ModeRegistry::load();
        let m = reg.get("basic").unwrap();
        assert!(!m.skills);
    }

    #[test]
    fn registry_unknown_returns_none() {
        let reg = ModeRegistry::load();
        assert!(reg.get("nonexistent").is_none());
    }

    #[test]
    fn parse_rejects_non_mode_root() {
        let src = "workflow { name : \"x\" }";
        assert!(parse_mode_at(src).is_err());
    }

    #[test]
    fn parse_missing_name_errors() {
        let src = "mode { description : \"x\" }";
        assert!(parse_mode_at(src).is_err());
    }
}
