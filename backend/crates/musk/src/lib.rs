//! auto-musk library root.
//!
//! Re-exports modules so the binary and integration tests share one source.

pub mod app_config;
pub mod auth;
// PLAN-092 T-03: auto-plan 四技能 serve 启动幂等分发（musk 为真源）。
pub mod builtin_skills;
pub mod canvas;
pub mod chats;
pub mod conversation;
pub mod dev_seed;
pub mod edit_diff;
pub mod mode;
pub mod orch_tools;
pub mod relay;
pub mod server;

// PLAN-044 Phase 1: VM 后端桥接（MUSK_BACKEND=vm 时 serve 走 AutoVM）。
pub mod vm_backend;
pub mod spec_tools;
pub mod specs;
pub mod spec_tree;
pub mod files_browser;
pub mod chat_branch;
pub mod plans;
pub mod report_tools;
pub mod plan_merge;
pub mod plan_tools;
// PLAN-096 T-04: 计划执行专用 worktree（租约/守卫/安全移除）。
pub mod plan_worktree;
// PLAN-096 T-08/T-09: 受控交付（prepare/land/refresh/archive/cleanup + 收据）。
pub mod plan_delivery;
// PLAN-098: 运行遥测装饰器（每 run 模型归因；聊天/relay 注入与折叠见模块文档）。
pub mod telemetry;
pub mod command_runner;
pub mod output_accumulator;
pub mod tool_context;
pub mod tool_gate;
pub mod tool_gate_routes;
pub mod tool_safety;
pub mod tool_test;
pub mod tool_truncate;
pub mod tools;
pub mod wiki;
pub mod workspace;
pub mod hello;
// PLAN-083 T-02: 会话详情分页端点 + 归一化直出（/api/chats/session/{id}/page）。
pub mod chat_page;

/// Auto-generated Rust from .at sources (a2r transpilation). Coexists with
/// the hand-written modules above. See `auto_generated/mod.rs`.
#[allow(dead_code, unused_imports)]
pub mod auto_generated;

use std::sync::Arc;

use auto_ai_agent::Role;

/// Owns an `Arc<dyn Role>` and re-implements `Role` so it can be
/// passed to `Agent::new` (which takes `P: Role + 'static`). The agent
/// crate's `load_builtin`/`load_role` return `Arc<dyn Role>`, and
/// `Arc<dyn Trait>` itself doesn't implement the trait — this thin wrapper
/// bridges that. (Mirrors the private `ArcRole` in auto-ai-agent's
/// workflow module.)
///
/// When `extra_prompt` is set, `system_prompt()` returns the base prompt with
/// the extra appended — this is how a Mode's `extra_system_prompt` customizes
/// the Role's Soul (Plan 004).
pub(crate) struct OwnedRole {
    inner: Arc<dyn Role>,
    extra_prompt: Option<String>,
    /// Materialized base+extra prompt so system_prompt() can borrow it.
    prompt: String,
    /// Tier forced by Plan 004 tier-clamping: when a mode's allowed_tiers
    /// excludes the role's own tier, this is set to the highest allowed tier
    /// (see build_agent_from_mode). `model_tier()` returns it when set,
    /// making `allowed_tiers` actually enforce.
    override_tier: Option<auto_ai_agent::ModelTier>,
}

impl OwnedRole {
    pub(crate) fn new(inner: Arc<dyn Role>) -> Self {
        let prompt = inner.system_prompt().to_string();
        Self { inner, extra_prompt: None, prompt, override_tier: None }
    }

    /// Force the role to report `tier` from `model_tier()`, overriding the
    /// inner role's own tier. Used by the Plan 004 tier-clamp logic.
    pub(crate) fn with_override_tier(mut self, tier: auto_ai_agent::ModelTier) -> Self {
        self.override_tier = Some(tier);
        self
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

impl Role for OwnedRole {
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
        // Plan 004 tier-clamp: honor the forced tier when set (see
        // build_agent_from_mode + with_override_tier); otherwise forward.
        self.override_tier.unwrap_or_else(|| self.inner.model_tier())
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
/// a fresh agent configured for the given role + client.
/// Build an agent configured by an [`crate::mode::AgentMode`].
///
/// The mode declares: role, tool whitelist, skills on/off, context file,
/// extra system prompt. This function resolves the role, registers only
/// the allowed tools (+ skill tool if enabled), injects context, and returns
/// the agent.
pub fn build_agent_from_mode(
    mode: &crate::mode::AgentMode,
    client: Arc<dyn auto_ai_agent::Client>,
    // PLAN-069 W1：注入式沙箱根。Some = 文件/命令工具按显式根注册（生产入口
    // 必传，fail-closed）；None = 旧解析链（thread-local > startup CWD），
    // 仅测试可用——运行入口禁止传 None（PLAN-030 同类缺陷在 /api/run、
    // SSE run 与 dispatch 的复发根治）。
    // PLAN-070 T-02：单根 → 多根（registry::sandbox_roots 合成的
    // [workspace 根, *白名单] 向量；CLI 单根 CWD 亦走 Vec）。
    ws_roots: Option<&std::sync::Arc<Vec<std::path::PathBuf>>>,
) -> Result<auto_ai_agent::Agent, String> {
    // 1. Resolve role: user role (.at) > built-in name > .at file path.
    let role: Arc<dyn Role> = resolve_role(&mode.role)
        .map_err(|e| format!("mode '{}': {e}", mode.name))?;

    // Tier clamp (Plan 004): if the role declares allowed_tiers and the role's
    // own tier falls outside them, warn + clamp to the highest allowed tier.
    // The clamped tier is applied via `with_override_tier` so that
    // `model_tier()` actually returns it (previously computed-but-discarded).
    let allowed = role.allowed_tiers();
    let clamp_to = if !allowed.is_empty() {
        let tier = role.model_tier();
        if !allowed.contains(&tier) {
            let clamped = allowed
                .iter()
                .max_by_key(|t| t.order())
                .copied()
                .unwrap_or(tier);
            tracing::warn!(
                "mode '{}': role '{}' tier {:?} not in allowed_tiers {:?}; clamping to {:?}",
                mode.name,
                role.name(),
                tier,
                allowed,
                clamped
            );
            Some(clamped)
        } else {
            None
        }
    } else {
        None
    };

    // Wrap, applying the mode's extra_system_prompt as a Soul customization,
    // and the tier clamp (if any) so model_tier() honors allowed_tiers.
    // PLAN-087 T-08: coding 模式追加画布生成指导（内嵌模板池 + canvas 工具
    // 流程 + 已知坑节选）——模块化注入点，非 coding 模式不受影响。
    let extra_prompt = if mode.name == "coding" {
        format!(
            "{}\n\n{}",
            mode.extra_system_prompt,
            crate::canvas::templates::generation_prompt()
        )
    } else {
        mode.extra_system_prompt.clone()
    };
    let mut owned = OwnedRole::new(role).with_extra_prompt(&extra_prompt);
    if let Some(tier) = clamp_to {
        owned = owned.with_override_tier(tier);
    }
    let role_skills = owned.skills();
    let mut agent = auto_ai_agent::Agent::new(owned, client);

    // 2. Register tools filtered by the mode's whitelist.
    //    Empty whitelist = register all base tools.
    // PLAN-069 W1：文件/命令工具按 ws_roots 条件注入。None = 测试回退链
    // （thread-local > startup CWD）；生产运行入口一律 Some（fail-closed）。
    // PLAN-070 T-02：单根 → 多根（workspace 根 + 白名单，任一命中即放行）。
    macro_rules! scoped {
        ($new:expr, $scoped:expr) => {
            if let Some(r) = ws_roots {
                Arc::new($scoped(r.clone()))
            } else {
                Arc::new($new())
            }
        };
    }
    let all_tools: Vec<(&str, Arc<dyn auto_ai_agent::Tool>)> = vec![
        ("read_file", scoped!(tools::ReadFile::new, tools::ReadFile::with_roots)),
        ("write_file", scoped!(tools::WriteFile::new, tools::WriteFile::with_roots)),
        ("edit_file", scoped!(tools::EditFile::new, tools::EditFile::with_roots)),
        ("search", scoped!(tools::Search::new, tools::Search::with_roots)),
        ("list_dir", scoped!(tools::ListDir::new, tools::ListDir::with_roots)),
        ("list_symbols", scoped!(tools::ListSymbols::new, tools::ListSymbols::with_roots)),
        ("glob", scoped!(tools::Glob::new, tools::Glob::with_roots)),
        ("run_command", scoped!(tools::RunCommand::new, tools::RunCommand::with_roots)),
        // Spec tools (Plan 009 P1a): read/write the Spec Ledger.
        ("read_specs", Arc::new(spec_tools::ReadSpecs::new())),
        ("list_specs", Arc::new(spec_tools::ListSpecs::new())),
        ("write_spec", Arc::new(spec_tools::WriteSpec::new())),
        ("update_spec", Arc::new(spec_tools::UpdateSpec::new())),
        ("write_goals", Arc::new(spec_tools::WriteGoals::new())),
    ];
    for (name, tool) in &all_tools {
        if mode.tools.is_empty() || mode.tools.iter().any(|t| t == name) {
            agent.register_shared(tool.clone());
        }
    }

    // 3. Register the Skill tool if the mode enables skills. Plan 004: if the
    //    role declares a skills whitelist, only those skills are exposed;
    //    otherwise (empty whitelist) all installed skills are exposed.
    //    PLAN-092 T-03: 目录解析改 autoos_skills_dir 单源（MUSK_CONFIG_DIR
    //    覆盖时与 serve 技能分发目标一致）；agent 每 run 重建即重扫，
    //    serve 启动同步的新技能下一次运行即可见。
    if mode.skills {
        if let Some(skills_dir) = crate::builtin_skills::autoos_skills_dir() {
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

/// Like [`build_agent_from_mode`], but also registers orchestration tools
/// (`spawn_relay`, `dispatch`, `bring_in`) when a [`tool_context::ToolContext`]
/// is provided and the mode's tool whitelist allows them. Used by chat_stream
/// and the relay driver to give agents the ability to spawn sub-conversations.
///
/// PLAN-030 §5.5: `spawn_relay` is back for chat agents — it is the entry
/// point into the plan-driven dev flow (`flow_id="plan"`), which replaces the
/// old spec-driven relay pipelines as the canonical escalation path.
pub fn build_agent_with_context(
    mode: &crate::mode::AgentMode,
    client: Arc<dyn auto_ai_agent::Client>,
    ctx: Option<tool_context::ToolContext>,
) -> Result<auto_ai_agent::Agent, String> {
    // PLAN-069 W1：root 在 base 构建期即注入（原 base 注册非注入 + 此处覆盖
    // 注册的双跳形态退役——覆盖窗口内 thread-local 回调仍可能漏出）。
    // PLAN-070 T-02：多根 = [workspace 根, *白名单]（sandbox_roots 现读，
    // 白名单增删对后续运行即时生效）。
    // PLAN-096 T-04（§5.3）：plan 流 execute/review 相位携带显式 execution
    // scope——文件/命令工具的根 = 该 run 的开发 worktree（主检出代码不注册
    // 为可写根，白名单配置不改动）；Canvas 工具经同字段解析（见 canvas）。
    let base_roots = ctx
        .as_ref()
        .map(|c| match &c.execution_root {
            Some(root) => std::sync::Arc::new(vec![(**root).clone()]),
            None => c.state.registry.sandbox_roots(&c.workspace_id),
        });
    let mut agent = build_agent_from_mode(mode, client, base_roots.as_ref())?;
    if let Some(ctx) = ctx {
        // PLAN-030 T3: plan tools are workspace-scoped (docs/plans/), so they
        // need the ToolContext — registered alongside the orchestration tools
        // here (chat agents + relay step agents both build through this path).
        let orch_tools: Vec<(&str, Arc<dyn auto_ai_agent::Tool>)> = vec![
            ("spawn_relay", Arc::new(crate::orch_tools::SpawnRelay::new(ctx.clone()))),
            ("dispatch", Arc::new(crate::orch_tools::Dispatch::new(ctx.clone()))),
            ("bring_in", Arc::new(crate::orch_tools::BringIn::new(ctx.clone()))),
            ("spawn_task_plan", Arc::new(crate::orch_tools::SpawnTaskPlan::new(ctx.clone()))),
            ("display_image", Arc::new(crate::tools::DisplayImage::new(ctx.clone()))),
            ("register_task_plan", Arc::new(crate::orch_tools::RegisterTaskPlan::new(ctx.clone()))),
            ("list_plans", Arc::new(crate::plan_tools::ListPlans::from_ctx(&ctx))),
            ("read_plan", Arc::new(crate::plan_tools::ReadPlan::from_ctx(&ctx))),
            ("create_plan", Arc::new(crate::plan_tools::CreatePlan::from_ctx(&ctx))),
            ("update_plan", Arc::new(crate::plan_tools::UpdatePlan::from_ctx(&ctx))),
            ("transition_plan", Arc::new(crate::plan_tools::TransitionPlan::from_ctx(&ctx))),
            ("merge_plan", Arc::new(crate::plan_tools::MergePlan::from_ctx(&ctx))),
            ("complete_plan_stage", Arc::new(crate::plan_tools::CompletePlanStage::from_ctx(&ctx))),
            ("plan_delivery", Arc::new(crate::plan_delivery::PlanDelivery::from_ctx(&ctx))),
            ("emit_report", Arc::new(crate::report_tools::EmitReport::from_ctx(&ctx))),
        ];
        for (name, tool) in &orch_tools {
            if mode.tools.is_empty() || mode.tools.iter().any(|t| t == name) {
                agent.register_shared(tool.clone());
            }
        }
        // PLAN-069 W1+W3：七个文件工具已在 base 构建期按 ws_roots 注入（见上），
        // 仅 run_command 需在此覆盖注册以挂进度通道与审批门（human 会话
        // 越界首触暂停，W3）。PLAN-070 T-02：门判定与 cwd 同走多根。
        // PLAN-096 T-04：execution scope 存在时根 = worktree（cwd 同落）。
        let ws_roots: std::sync::Arc<Vec<std::path::PathBuf>> = match &ctx.execution_root {
            Some(root) => std::sync::Arc::new(vec![(**root).clone()]),
            None => ctx.state.registry.sandbox_roots(&ctx.workspace_id),
        };
        let gate_session = if ctx.approval_mode.as_deref() == Some("human") {
            Some(ctx.parent_conversation_id.clone())
        } else {
            None
        };
        // PLAN-073 F-D：auto 模式自动放行（非白名单/越界直接执行，不打断）；
        // human 挂 live 门（UI approve/deny）；其余（CLI/relay 无会话）legacy。
        let auto_approve = ctx.approval_mode.as_deref() == Some("auto");
        let scoped_run_command: Vec<(&str, Arc<dyn auto_ai_agent::Tool>)> = vec![
            (
                "run_command",
                Arc::new(crate::tools::RunCommand::with_roots_progress_policy(
                    ws_roots.clone(),
                    ctx.progress.clone(),
                    gate_session,
                    auto_approve,
                )),
            ),
        ];
        for (name, tool) in &scoped_run_command {
            if mode.tools.is_empty() || mode.tools.iter().any(|t| t == name) {
                agent.register_shared(tool.clone());
            }
        }
        // PLAN-030 复审修复：在 server/relay 场景把 5 个 spec 工具覆盖注册为
        // workspace 域（默认注册是 home 目录 store，agent 写入与 UI/审批队列
        // 互不可见）。register_shared 后写覆盖同名工具。
        let ws_spec_tools: Vec<(&str, Arc<dyn auto_ai_agent::Tool>)> = vec![
            ("read_specs", Arc::new(crate::spec_tools::ReadSpecs::from_ctx(&ctx))),
            ("list_specs", Arc::new(crate::spec_tools::ListSpecs::from_ctx(&ctx))),
            ("write_spec", Arc::new(crate::spec_tools::WriteSpec::from_ctx(&ctx))),
            ("update_spec", Arc::new(crate::spec_tools::UpdateSpec::from_ctx(&ctx))),
            ("write_goals", Arc::new(crate::spec_tools::WriteGoals::from_ctx(&ctx))),
        ];
        for (name, tool) in &ws_spec_tools {
            if mode.tools.is_empty() || mode.tools.iter().any(|t| t == name) {
                agent.register_shared(tool.clone());
            }
        }
        // PLAN-087 T-06 + PLAN-088 T-04/T-06: canvas 工具七件（同白名单过
        // 滤；coding.at 已收录 run/stop/snapshot/act/state/pick/overlay）。
        for (name, tool) in crate::canvas::tools::canvas_tool_registry(&ctx) {
            if mode.tools.is_empty() || mode.tools.iter().any(|t| t == name) {
                agent.register_shared(tool);
            }
        }
    }
    Ok(agent)
}

/// Resolve a role by spec. Order: a user Role from the RoleRegistry
/// (`.at` in ~/.config/autoos/roles), then a built-in name, then a literal
/// `.at` file path. (Plan 004 adds the RoleRegistry-first lookup.)
/// `pub(crate)`: delegated by the a2r extern_impl `resolve_role` stub.
pub(crate) fn resolve_role(spec: &str) -> Result<Arc<dyn Role>, String> {
    // 1. User role from the on-disk registry.
    let registry = auto_ai_agent::RoleRegistry::load();
    if let Some(p) = registry.resolve_role(spec) {
        return Ok(p);
    }
    // 2. Built-in name.
    if let Some(p) = auto_ai_agent::load_builtin(spec) {
        return Ok(p);
    }
    // 3. Literal .at file path.
    let content = std::fs::read_to_string(spec)
        .map_err(|e| format!("not a builtin or role, cannot read '{spec}': {e}"))?;
    auto_ai_agent::load_role(&content).map_err(|e| format!("parse '{spec}': {e}"))
}

/// Search upward from `start` for `.musk.md`, then `CLAUDE.md`. Returns the
/// first found path, or None. Shared by `find_context_file` and the a2r
/// extern_impl delegation (`find_ctx_upward`).
pub(crate) fn find_ctx_upward(start: &std::path::Path) -> Option<std::path::PathBuf> {
    for dir in start.ancestors() {
        for name in [".musk.md", "CLAUDE.md"] {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// Search upward from CWD for `.musk.md`, then `CLAUDE.md`. Returns the first
/// found path, or None. `pub(crate)`: delegated by the a2r extern_impl
/// `find_context_file` stub.
pub(crate) fn find_context_file() -> Option<std::path::PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    find_ctx_upward(&cwd)
}

#[cfg(test)]
mod tests {
    use super::*;
    use auto_ai_agent::ModelTier;

    /// Minimal Role for testing OwnedRole tier-clamping. Only `name`,
    /// `system_prompt`, and the tier methods are meaningful.
    struct MockRole {
        tier: ModelTier,
        allowed: Vec<ModelTier>,
    }

    impl Role for MockRole {
        fn name(&self) -> &str { "mock" }
        fn system_prompt(&self) -> &str { "mock soul" }
        fn model_tier(&self) -> ModelTier { self.tier }
        fn allowed_tiers(&self) -> Vec<ModelTier> { self.allowed.clone() }
    }

    #[test]
    fn owned_role_forwards_inner_tier_when_no_override() {
        let role = MockRole { tier: ModelTier::Mid, allowed: vec![] };
        let owned = OwnedRole::new(Arc::new(role));
        assert_eq!(owned.model_tier(), ModelTier::Mid);
    }

    #[test]
    fn owned_role_override_tier_wins_over_inner() {
        // Plan 004 F1: the override set by tier-clamping must take effect.
        let role = MockRole { tier: ModelTier::Max, allowed: vec![ModelTier::Mid, ModelTier::Pro] };
        let owned = OwnedRole::new(Arc::new(role))
            .with_override_tier(ModelTier::Pro); // clamp Max → highest allowed (Pro)
        assert_eq!(owned.model_tier(), ModelTier::Pro);
    }

    #[test]
    fn owned_role_override_preserves_extra_prompt() {
        // The builder chain (extra_prompt + override_tier) must compose.
        let role = MockRole { tier: ModelTier::Max, allowed: vec![ModelTier::Mid] };
        let owned = OwnedRole::new(Arc::new(role))
            .with_extra_prompt("extra")
            .with_override_tier(ModelTier::Mid);
        assert_eq!(owned.model_tier(), ModelTier::Mid);
        assert!(owned.system_prompt().ends_with("extra"));
    }
}
