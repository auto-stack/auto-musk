//! Profession Registry — the app-layer orchestration metadata (P2a).
//!
//! Each `Profession` declares an agent's role in the spec-driven relay
//! workflow: which spec sections it owns/reads, which tools it may use, who it
//! can hand off to or dispatch to, and its token budget. This is *separate*
//! from the auto-ai-agent `Profession` trait (which is purely static
//! prompt/model config) — the two are bridged by `build_agent_from_mode` +
//! `relay_routes` resolving a profession_id into both an orchestration record
//! (here) and an agent (auto-ai-agent).
//!
//! Ported from auto-forge `backend/src/relay/profession.rs`, with the
//! model-tier / thinking / skill fields dropped (those live in auto-ai-agent).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::specs::SectionType;

/// A profession defines an agent's role, scope, and constraints within a relay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profession {
    pub id: String,
    pub name: String,
    pub phase: ForgePhase,
    /// Sections this profession can write to.
    pub owned_sections: Vec<SectionType>,
    /// Sections this profession can read for context.
    pub readable_sections: Vec<SectionType>,
    /// Tool names this profession is allowed to use.
    pub allowed_tools: Vec<String>,
    /// Professions that may receive handoffs from this one.
    pub handoff_to: Vec<String>,
    /// Professions that may be dispatched to as errand agents from this one.
    pub dispatchable_to: Vec<String>,
    /// Human approval is required before handing off to these professions.
    pub approval_gates: Vec<String>,
    /// Max LLM turns before forced handoff.
    pub max_turns: u32,
    /// Default token budget for this profession.
    pub token_budget: u64,
}

/// Lifecycle phase of the spec-driven workflow. Orders the relay pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForgePhase {
    Intake,
    Discovery,
    GoalGate,
    Design,
    Planning,
    Execution,
    Verification,
    Report,
    Errand,
}

impl ForgePhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            ForgePhase::Intake => "intake",
            ForgePhase::Discovery => "discovery",
            ForgePhase::GoalGate => "goal_gate",
            ForgePhase::Design => "design",
            ForgePhase::Planning => "planning",
            ForgePhase::Execution => "execution",
            ForgePhase::Verification => "verification",
            ForgePhase::Report => "report",
            ForgePhase::Errand => "errand",
        }
    }
}

/// Registry of built-in and custom professions.
pub struct ProfessionRegistry {
    professions: HashMap<String, Profession>,
}

impl ProfessionRegistry {
    /// Load from disk, seeding the built-in defaults on first run.
    pub fn load() -> Self {
        let path = professions_path();
        let list: Vec<Profession> = if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|c| serde_json::from_str(&c).ok())
                .unwrap_or_else(default_professions)
        } else {
            let defaults = default_professions();
            let _ = save_professions(&defaults);
            defaults
        };
        let map = list.into_iter().map(|p| (p.id.clone(), p)).collect();
        Self { professions: map }
    }

    pub fn get(&self, id: &str) -> Option<&Profession> {
        self.professions.get(id)
    }

    pub fn list(&self) -> Vec<Profession> {
        let mut v: Vec<Profession> = self.professions.values().cloned().collect();
        v.sort_by(|a, b| a.id.cmp(&b.id));
        v
    }

    /// True if `from` is allowed to hand off to `to`.
    pub fn can_handoff(&self, from: &str, to: &str) -> bool {
        self.professions
            .get(from)
            .map(|p| p.handoff_to.iter().any(|h| h == to))
            .unwrap_or(false)
    }

    /// True if handing off from `from` to `to` requires human approval.
    pub fn needs_approval(&self, from: &str, to: &str) -> bool {
        self.professions
            .get(from)
            .map(|p| p.approval_gates.iter().any(|g| g == to))
            .unwrap_or(false)
    }
}

fn config_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config/autoos")
}

fn professions_path() -> PathBuf {
    config_dir().join("professions.json")
}

fn save_professions(professions: &[Profession]) -> Result<(), String> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("create dir {}: {}", dir.display(), e))?;
    let path = professions_path();
    let content =
        serde_json::to_string_pretty(professions).map_err(|e| format!("serialize: {}", e))?;
    std::fs::write(&path, content).map_err(|e| format!("write {}: {}", path.display(), e))?;
    Ok(())
}

/// Generate the built-in professions: 9 core + 3 "superpower" variants.
///
/// Handoff graph (core): assistant → advisor →(gate)→ architect → planner →
/// tester → coder →{tester,architect} → … → reviewer → documenter.
/// Gofer is the errand target (dispatchable_to) for most professions.
pub fn default_professions() -> Vec<Profession> {
    use SectionType::*;
    vec![
        Profession {
            id: "assistant".into(),
            name: "Assistant".into(),
            phase: ForgePhase::Intake,
            owned_sections: vec![],
            readable_sections: vec![],
            allowed_tools: into_tools([
                "bring_in",
                "dispatch",
                "spawn_relay",
                "spawn_task_plan",
                "run_command",
                "query_wiki",
                "list_wiki",
            ]),
            handoff_to: into_list(["advisor", "super-advisor"]),
            approval_gates: vec![],
            dispatchable_to: into_list(["gofer"]),
            max_turns: 12,
            token_budget: 2_000_000,
        },
        Profession {
            id: "advisor".into(),
            name: "Advisor".into(),
            phase: ForgePhase::Discovery,
            owned_sections: vec![Goals],
            readable_sections: vec![Goals, Architecture],
            allowed_tools: into_tools([
                "read_specs",
                "list_specs",
                "update_spec",
                "write_goals",
                "read_file",
                "query_wiki",
                "list_wiki",
                "bring_in",
                "dispatch",
                "spawn_relay",
            ]),
            handoff_to: into_list(["architect"]),
            approval_gates: into_list(["architect"]),
            dispatchable_to: into_list(["gofer"]),
            max_turns: 40,
            token_budget: 8_000_000,
        },
        Profession {
            id: "architect".into(),
            name: "Architect".into(),
            phase: ForgePhase::Design,
            owned_sections: vec![Architecture, Designs],
            readable_sections: vec![Goals, Architecture, Designs],
            allowed_tools: into_tools([
                "read_specs",
                "list_specs",
                "update_spec",
                "read_file",
                "write_file",
                "query_wiki",
                "list_wiki",
                "bring_in",
                "spawn_relay",
            ]),
            handoff_to: into_list(["planner"]),
            approval_gates: vec![],
            dispatchable_to: into_list(["gofer"]),
            max_turns: 40,
            token_budget: 12_000_000,
        },
        Profession {
            id: "planner".into(),
            name: "Planner".into(),
            phase: ForgePhase::Planning,
            owned_sections: vec![],
            readable_sections: vec![Goals, Architecture, Designs, Tests],
            allowed_tools: into_tools([
                "read_specs",
                "list_specs",
                "update_spec",
                "read_file",
                "query_wiki",
                "list_wiki",
                "bring_in",
                "register_task_plan",
            ]),
            handoff_to: into_list(["tester"]),
            approval_gates: vec![],
            dispatchable_to: into_list(["gofer"]),
            max_turns: 40,
            token_budget: 8_000_000,
        },
        Profession {
            id: "tester".into(),
            name: "Tester".into(),
            phase: ForgePhase::Planning,
            owned_sections: vec![Tests],
            readable_sections: vec![Goals, Designs, Tests],
            allowed_tools: into_tools([
                "read_specs",
                "list_specs",
                "update_spec",
                "read_file",
                "write_file",
                "edit_file",
                "run_command",
                "search",
                "query_wiki",
                "list_wiki",
                "bring_in",
            ]),
            handoff_to: into_list(["coder"]),
            approval_gates: vec![],
            dispatchable_to: into_list(["gofer"]),
            max_turns: 40,
            token_budget: 8_000_000,
        },
        Profession {
            id: "coder".into(),
            name: "Coder".into(),
            phase: ForgePhase::Execution,
            owned_sections: vec![],
            readable_sections: vec![Designs, Tests],
            allowed_tools: into_tools([
                "read_file",
                "write_file",
                "edit_file",
                "run_command",
                "search",
                "read_specs",
                "list_specs",
                "query_wiki",
                "list_wiki",
                "dispatch",
            ]),
            handoff_to: into_list(["tester", "architect"]),
            approval_gates: vec![],
            dispatchable_to: into_list(["gofer"]),
            max_turns: 50,
            token_budget: 20_000_000,
        },
        Profession {
            id: "reviewer".into(),
            name: "Reviewer".into(),
            phase: ForgePhase::Verification,
            owned_sections: vec![Reviews],
            readable_sections: vec![Goals, Architecture, Designs, Tests, Reviews, Reports],
            allowed_tools: into_tools([
                "read_file",
                "write_file",
                "edit_file",
                "run_command",
                "search",
                "read_specs",
                "list_specs",
                "update_spec",
                "query_wiki",
                "list_wiki",
                "dispatch",
            ]),
            handoff_to: into_list(["documenter"]),
            approval_gates: vec![],
            dispatchable_to: into_list(["gofer"]),
            max_turns: 40,
            token_budget: 15_000_000,
        },
        Profession {
            id: "documenter".into(),
            name: "Documenter".into(),
            phase: ForgePhase::Report,
            owned_sections: vec![Reports],
            readable_sections: vec![
                Goals, Architecture, Designs, Tests, Reviews, Reports,
            ],
            allowed_tools: into_tools([
                "read_file",
                "read_specs",
                "list_specs",
                "update_spec",
                "write_file",
                "edit_file",
                "query_wiki",
                "list_wiki",
            ]),
            handoff_to: vec![],
            approval_gates: vec![],
            dispatchable_to: vec![],
            max_turns: 20,
            token_budget: 4_000_000,
        },
        Profession {
            id: "gofer".into(),
            name: "Gofer".into(),
            phase: ForgePhase::Errand,
            owned_sections: vec![],
            readable_sections: vec![Goals, Architecture, Designs, Tests],
            allowed_tools: into_tools([
                "run_command",
                "read_file",
                "edit_file",
                "search",
                "list_specs",
                "read_specs",
                "query_wiki",
                "list_wiki",
            ]),
            handoff_to: vec![],
            approval_gates: vec![],
            dispatchable_to: vec![],
            max_turns: 20,
            token_budget: 4_000_000,
        },
        // ─── Superpower professions (autonomous long-running relays) ───────────
        Profession {
            id: "super-advisor".into(),
            name: "Super Advisor".into(),
            phase: ForgePhase::Planning,
            owned_sections: vec![Goals, Architecture, Designs, Tests],
            readable_sections: vec![
                Goals, Architecture, Designs, Tests, Reviews, Reports,
            ],
            allowed_tools: into_tools([
                "read_specs",
                "list_specs",
                "update_spec",
                "write_goals",
                "read_file",
                "write_file",
                "query_wiki",
                "list_wiki",
                "bring_in",
                "dispatch",
                "spawn_relay",
            ]),
            handoff_to: into_list(["super-coder"]),
            approval_gates: into_list(["super-coder"]),
            dispatchable_to: into_list(["gofer"]),
            max_turns: 120,
            token_budget: 15_000_000,
        },
        Profession {
            id: "super-coder".into(),
            name: "Super Coder".into(),
            phase: ForgePhase::Execution,
            owned_sections: vec![],
            readable_sections: vec![Goals, Architecture, Designs, Tests],
            allowed_tools: into_tools([
                "read_file",
                "write_file",
                "edit_file",
                "run_command",
                "search",
                "read_specs",
                "list_specs",
                "query_wiki",
                "list_wiki",
                "dispatch",
            ]),
            handoff_to: into_list(["super-tester"]),
            approval_gates: vec![],
            dispatchable_to: into_list(["gofer"]),
            max_turns: 120,
            token_budget: 20_000_000,
        },
        Profession {
            id: "super-tester".into(),
            name: "Super Tester".into(),
            phase: ForgePhase::Report,
            owned_sections: vec![Reviews, Reports],
            readable_sections: vec![
                Goals, Architecture, Designs, Tests, Reviews, Reports,
            ],
            allowed_tools: into_tools([
                "read_file",
                "run_command",
                "search",
                "read_specs",
                "list_specs",
                "update_spec",
                "query_wiki",
                "list_wiki",
                "dispatch",
            ]),
            handoff_to: vec![],
            approval_gates: vec![],
            dispatchable_to: into_list(["gofer"]),
            max_turns: 100,
            token_budget: 15_000_000,
        },
        // ─── Plan-driven profession (PLAN-030: 单角色四相位流程) ────────────────
        // One agent plays Advisor/Coder/Reviewer/Documenter across the plan
        // flow's four phases; the plan file is the full handoff artifact, so
        // handoff_to lists itself (sequential same-role steps).
        Profession {
            id: "plan-dev".into(),
            name: "Plan-Driven Developer".into(),
            phase: ForgePhase::Execution,
            owned_sections: vec![
                Goals, Architecture, Designs, Tests, Reviews, Reports,
            ],
            readable_sections: vec![
                Goals, Architecture, Designs, Tests, Reviews, Reports,
            ],
            allowed_tools: into_tools([
                "read_file",
                "write_file",
                "edit_file",
                "search",
                "list_dir",
                "run_command",
                "read_specs",
                "list_specs",
                "update_spec",
                "list_plans",
                "read_plan",
                "create_plan",
                "update_plan",
                "transition_plan",
                "merge_plan",
            ]),
            handoff_to: into_list(["plan-dev"]),
            approval_gates: vec![],
            dispatchable_to: into_list(["gofer"]),
            max_turns: 120,
            token_budget: 20_000_000,
        },
    ]
}

fn into_tools<I: IntoIterator<Item = &'static str>>(iter: I) -> Vec<String> {
    iter.into_iter().map(String::from).collect()
}

fn into_list<I: IntoIterator<Item = &'static str>>(iter: I) -> Vec<String> {
    iter.into_iter().map(String::from).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_load_and_have_handoff_graph() {
        let reg = ProfessionRegistry {
            professions: default_professions()
                .into_iter()
                .map(|p| (p.id.clone(), p))
                .collect(),
        };
        // All 12 built-ins present.
        for id in [
            "assistant", "advisor", "architect", "planner", "tester", "coder",
            "reviewer", "documenter", "gofer", "super-advisor", "super-coder", "super-tester",
        ] {
            assert!(reg.get(id).is_some(), "missing profession {id}");
        }
        // PLAN-030: plan-dev present, owns all 6 sections, self-handoff chain.
        let plan_dev = reg.get("plan-dev").expect("missing profession plan-dev");
        assert_eq!(plan_dev.owned_sections.len(), 6);
        assert!(reg.can_handoff("plan-dev", "plan-dev"));
        assert!(!reg.needs_approval("plan-dev", "plan-dev"));
        assert!(plan_dev.allowed_tools.contains(&"merge_plan".to_string()));
        assert_eq!(plan_dev.max_turns, 120);
        // Core handoff chain.
        assert!(reg.can_handoff("assistant", "advisor"));
        assert!(reg.can_handoff("advisor", "architect"));
        assert!(reg.can_handoff("architect", "planner"));
        assert!(reg.can_handoff("coder", "tester"));
        // advisor→architect requires approval.
        assert!(reg.needs_approval("advisor", "architect"));
        assert!(!reg.needs_approval("architect", "planner"));
        // documenter is terminal (no handoffs).
        assert!(reg.get("documenter").unwrap().handoff_to.is_empty());
    }

    #[test]
    fn advisor_owns_goals() {
        let reg = ProfessionRegistry {
            professions: default_professions()
                .into_iter()
                .map(|p| (p.id.clone(), p))
                .collect(),
        };
        let advisor = reg.get("advisor").unwrap();
        assert!(advisor.owned_sections.contains(&SectionType::Goals));
        assert_eq!(advisor.phase, ForgePhase::Discovery);
    }
}
