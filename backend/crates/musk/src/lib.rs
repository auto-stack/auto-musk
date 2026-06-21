//! auto-musk library root.
//!
//! Re-exports modules so the binary and integration tests share one source.

pub mod auth;
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
/// Build the standard tool set + skill tool (if enabled + configured) for the
/// given profession + client.
///
/// `skills_enabled`: when true (default), the SkillTool is registered and the
/// agent gets the `<available_skills>` block (Superpowers mode). When false,
/// the agent has only the base tools (read/write/edit/search/run/etc.) — a
/// simpler "basic" mode, like Claude without superpowers.
pub fn build_agent(
    profession: Arc<dyn Profession>,
    client: Arc<dyn auto_ai_agent::Client>,
    skills_enabled: bool,
) -> auto_ai_agent::Agent {
    let mut agent = auto_ai_agent::Agent::new(OwnedProfession::new(profession), client);
    agent.register_tool(tools::ReadFile);
    agent.register_tool(tools::WriteFile);
    agent.register_tool(tools::RunCommand);
    agent.register_tool(tools::EditFile);
    agent.register_tool(tools::Search);
    agent.register_tool(tools::ListDir);
    agent.register_tool(tools::ListSymbols);
    agent.register_tool(tools::Glob);
    agent.register_tool(tools::BatchReplace);

    // Register the Skill tool only in Superpowers mode (skills_enabled).
    if skills_enabled {
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

    // Auto-discover project context: search upward from CWD for `.musk.md`
    // (then `CLAUDE.md`), like git finds `.git`. Injected into the system
    // prompt so the agent starts knowing the project's tech stack/conventions.
    let context_path = find_context_file();
    if let Some(path) = context_path {
        agent = auto_ai_agent::Agent::with_context_file(agent, &path);
    }

    agent
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
