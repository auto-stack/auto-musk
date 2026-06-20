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
pub fn build_agent(
    profession: Arc<dyn Profession>,
    client: Arc<dyn auto_ai_agent::Client>,
) -> auto_ai_agent::Agent {
    let mut agent = auto_ai_agent::Agent::new(OwnedProfession::new(profession), client);
    agent.register_tool(tools::ReadFile);
    agent.register_tool(tools::WriteFile);
    agent.register_tool(tools::RunCommand);
    agent.register_tool(tools::EditFile);
    agent.register_tool(tools::Search);

    // Register the Skill tool if any skills are configured under
    // ~/.config/autoos/skills/. (EditFile/Search referenced above are added in
    // Phase C; if absent at this point the build would fail — they exist.)
    if let Some(skills_dir) = dirs::home_dir().map(|h| h.join(".config/autoos/skills")) {
        let registry = std::sync::Arc::new(auto_ai_agent::SkillRegistry::scan(&skills_dir));
        if !registry.is_empty() {
            agent.register_skill_tool(auto_ai_agent::SkillTool::new(registry));
        }
    }
    agent
}
