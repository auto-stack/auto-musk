//! auto-musk library root.
//!
//! Re-exports modules so the binary and integration tests share one source.

pub mod auth;
pub mod mode;
pub mod server;
pub mod specs;
pub mod tools;
pub mod workflow;
pub mod hello;

use std::sync::Arc;

use auto_ai_agent::Profession;

/// Owns an `Arc<dyn Profession>` and re-implements `Profession` so it can be
/// passed to `Agent::new` (which takes `P: Profession + 'static`). The agent
/// crate's `load_builtin`/`load_profession` return `Arc<dyn Profession>`, and
/// `Arc<dyn Trait>` itself doesn't implement the trait — this thin wrapper
/// bridges that. (Mirrors the private `ArcProfession` in auto-ai-agent's
/// workflow module.)
pub(crate) struct OwnedProfession(Arc<dyn Profession>);

impl OwnedProfession {
    pub(crate) fn new(inner: Arc<dyn Profession>) -> Self {
        Self(inner)
    }
}

impl Profession for OwnedProfession {
    fn name(&self) -> &str {
        self.0.name()
    }
    fn system_prompt(&self) -> &str {
        self.0.system_prompt()
    }
    fn model(&self) -> &str {
        self.0.model()
    }
    fn temperature(&self) -> f64 {
        self.0.temperature()
    }
    fn max_turns(&self) -> usize {
        self.0.max_turns()
    }
    fn allowed_tools(&self) -> Vec<String> {
        self.0.allowed_tools()
    }
    fn memory_limit(&self) -> Option<usize> {
        self.0.memory_limit()
    }
}

/// Build the standard 3-tool set (read_file/write_file/run_command), returning
/// a fresh agent configured for the given profession + client.
/// Build an agent configured by an [`crate::mode::AgentMode`].
///
/// The mode declares: profession, tool whitelist, skills on/off, context file,
/// extra system prompt. This function resolves the profession, registers only
/// the allowed tools (+ skill tool if enabled), injects context, and returns
/// the agent.
pub fn build_agent_from_mode(
    mode: &crate::mode::AgentMode,
    client: Arc<dyn auto_ai_agent::Client>,
) -> Result<auto_ai_agent::Agent, String> {
    // 1. Resolve profession from the mode's profession field.
    let profession: Arc<dyn Profession> = resolve_profession(&mode.profession)
        .map_err(|e| format!("mode '{}': {e}", mode.name))?;

    let mut agent = auto_ai_agent::Agent::new(OwnedProfession::new(profession), client);

    // 2. Register tools filtered by the mode's whitelist.
    //    Empty whitelist = register all base tools.
    let all_tools: Vec<(&str, Arc<dyn auto_ai_agent::Tool>)> = vec![
        ("read_file", Arc::new(tools::ReadFile)),
        ("write_file", Arc::new(tools::WriteFile)),
        ("edit_file", Arc::new(tools::EditFile)),
        ("batch_replace", Arc::new(tools::BatchReplace)),
        ("search", Arc::new(tools::Search)),
        ("list_dir", Arc::new(tools::ListDir)),
        ("list_symbols", Arc::new(tools::ListSymbols)),
        ("glob", Arc::new(tools::Glob)),
        ("run_command", Arc::new(tools::RunCommand)),
    ];
    for (name, tool) in &all_tools {
        if mode.tools.is_empty() || mode.tools.iter().any(|t| t == name) {
            agent.register_shared(tool.clone());
        }
    }

    // 3. Register the Skill tool if the mode enables skills.
    if mode.skills {
        if let Some(skills_dir) =
            dirs::home_dir().map(|h| h.join(".config/autoos/skills"))
        {
            let registry =
                std::sync::Arc::new(auto_ai_agent::SkillRegistry::scan(&skills_dir));
            if !registry.is_empty() {
                agent.register_skill_tool(auto_ai_agent::SkillTool::new(registry));
            }
        }
    }

    // 4. Inject context file (from mode config or auto-discovered).
    let ctx_path = if !mode.context_file.is_empty() {
        Some(std::path::PathBuf::from(&mode.context_file))
    } else {
        find_context_file()
    };
    if let Some(path) = ctx_path {
        agent = auto_ai_agent::Agent::with_context_file(agent, &path);
    }

    Ok(agent)
}

/// Resolve a profession: built-in name, or `.at` file path.
fn resolve_profession(spec: &str) -> Result<Arc<dyn Profession>, String> {
    if let Some(p) = auto_ai_agent::load_builtin(spec) {
        return Ok(p);
    }
    let content = std::fs::read_to_string(spec)
        .map_err(|e| format!("not a builtin, cannot read '{spec}': {e}"))?;
    auto_ai_agent::load_profession(&content).map_err(|e| format!("parse '{spec}': {e}"))
}

/// Search upward from CWD for `.musk.md`, then `CLAUDE.md`. Returns the first
/// found path, or None.
fn find_context_file() -> Option<std::path::PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    for dir in cwd.ancestors() {
        for name in [".musk.md", "CLAUDE.md"] {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}
