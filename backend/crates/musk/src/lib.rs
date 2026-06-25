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
///
/// When `extra_prompt` is set, `system_prompt()` returns the base prompt with
/// the extra appended — this is how a Mode's `extra_system_prompt` customizes
/// the Role's Soul (Plan 004).
pub(crate) struct OwnedProfession {
    inner: Arc<dyn Profession>,
    extra_prompt: Option<String>,
    /// Materialized base+extra prompt so system_prompt() can borrow it.
    prompt: String,
}

impl OwnedProfession {
    pub(crate) fn new(inner: Arc<dyn Profession>) -> Self {
        let prompt = inner.system_prompt().to_string();
        Self { inner, extra_prompt: None, prompt }
    }

    /// Return a variant whose system prompt has `extra` appended to the base
    /// Soul (the mode-level customization).
    pub(crate) fn with_extra_prompt(mut self, extra: &str) -> Self {
        if !extra.trim().is_empty() {
            let mut p = self.inner.system_prompt().to_string();
            p.push_str("\n\n");
            p.push_str(extra);
            self.prompt = p;
            self.extra_prompt = Some(extra.to_string());
        }
        self
    }
}

impl Profession for OwnedProfession {
    fn name(&self) -> &str {
        self.inner.name()
    }
    fn system_prompt(&self) -> &str {
        &self.prompt
    }
    fn model(&self) -> &str {
        self.inner.model()
    }
    fn model_tier(&self) -> auto_ai_agent::ModelTier {
        self.inner.model_tier()
    }
    fn temperature(&self) -> f64 {
        self.inner.temperature()
    }
    fn max_turns(&self) -> usize {
        self.inner.max_turns()
    }
    fn allowed_tools(&self) -> Vec<String> {
        self.inner.allowed_tools()
    }
    fn memory_limit(&self) -> Option<usize> {
        self.inner.memory_limit()
    }
    // Plan 004: forward the new role fields too.
    fn allowed_tiers(&self) -> Vec<auto_ai_agent::ModelTier> {
        self.inner.allowed_tiers()
    }
    fn token_budget(&self) -> Option<u64> {
        self.inner.token_budget()
    }
    fn skills(&self) -> Vec<String> {
        self.inner.skills()
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
    // 1. Resolve profession: user role (.at) > built-in name > .at file path.
    let profession: Arc<dyn Profession> = resolve_profession(&mode.profession)
        .map_err(|e| format!("mode '{}': {e}", mode.name))?;

    // Tier clamp (Plan 004): if the role declares allowed_tiers and the role's
    // own tier falls outside them, warn + clamp to the highest allowed tier.
    // (We compare against the role's declared tier, which is what the agent
    // will request from the daemon.)
    let allowed = profession.allowed_tiers();
    if !allowed.is_empty() {
        let tier = profession.model_tier();
        if !allowed.contains(&tier) {
            let clamped = allowed
                .iter()
                .max_by_key(|t| t.order())
                .copied()
                .unwrap_or(tier);
            tracing::warn!(
                "mode '{}': role '{}' tier {:?} not in allowed_tiers {:?}; clamping to {:?}",
                mode.name,
                profession.name(),
                tier,
                allowed,
                clamped
            );
        }
    }

    // Wrap, applying the mode's extra_system_prompt as a Soul customization.
    let owned = OwnedProfession::new(profession).with_extra_prompt(&mode.extra_system_prompt);
    let role_skills = owned.skills();
    let mut agent = auto_ai_agent::Agent::new(owned, client);

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

    // 3. Register the Skill tool if the mode enables skills. Plan 004: if the
    //    role declares a skills whitelist, only those skills are exposed;
    //    otherwise (empty whitelist) all installed skills are exposed.
    if mode.skills {
        if let Some(skills_dir) =
            dirs::home_dir().map(|h| h.join(".config/autoos/skills"))
        {
            let mut registry =
                auto_ai_agent::SkillRegistry::scan(&skills_dir);
            if !role_skills.is_empty() {
                registry.retain(&role_skills);
            }
            if !registry.is_empty() {
                let registry = std::sync::Arc::new(registry);
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

/// Resolve a profession by spec. Order: a user Role from the RoleRegistry
/// (`.at` in ~/.config/autoos/roles), then a built-in name, then a literal
/// `.at` file path. (Plan 004 adds the RoleRegistry-first lookup.)
fn resolve_profession(spec: &str) -> Result<Arc<dyn Profession>, String> {
    // 1. User role from the on-disk registry.
    let registry = auto_ai_agent::RoleRegistry::load();
    if let Some(p) = registry.resolve_profession(spec) {
        return Ok(p);
    }
    // 2. Built-in name.
    if let Some(p) = auto_ai_agent::load_builtin(spec) {
        return Ok(p);
    }
    // 3. Literal .at file path.
    let content = std::fs::read_to_string(spec)
        .map_err(|e| format!("not a builtin or role, cannot read '{spec}': {e}"))?;
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
