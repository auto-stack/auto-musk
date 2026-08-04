//! TaskPlan registry — built-in + user-defined Atom plans.
//!
//! Unlike auto-forge's *global* static registry, musk's registry is
//! **per-workspace**: each workspace owns a `TaskPlanRegistry` instance (held
//! in `WorkspaceStores`) that loads user plans from `<workspace>/.autoos/task_plans/*.atom`.
//! Built-in plans are the same for every workspace and loaded via `include_str!`.
//!
//! Ported from auto-forge `relay/task_plan_registry.rs` (Plan 009 P2b.7).

use crate::relay::flows::get_builtin_flow;
use crate::relay::task_plan::TaskPlan;
use crate::relay::task_plan_parser::parse_task_plan;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ─── Built-in TaskPlan Atoms ─────────────────────────────────────────────────

const BUILTIN_TASK_PLANS: &[(&str, &str)] = &[(
    "deferred-decompose",
    include_str!("task_plans/builtin/deferred-decompose.atom"),
)];

/// Source of a TaskPlan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum TaskPlanSource {
    Builtin,
    User,
}

/// Summary returned by [`TaskPlanRegistry::list`].
#[derive(Debug, Clone, serde::Serialize)]
pub struct TaskPlanSummary {
    pub id: String,
    pub source: TaskPlanSource,
    pub phase_count: usize,
    pub run_count: usize,
}

/// Registry of all available TaskPlans (built-in + user-defined) for one workspace.
pub struct TaskPlanRegistry {
    plans: HashMap<String, (TaskPlan, TaskPlanSource)>,
    /// Directory where user plans are read from / written to.
    user_dir: PathBuf,
}

impl std::fmt::Debug for TaskPlanRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskPlanRegistry")
            .field("plan_count", &self.plans.len())
            .field("user_dir", &self.user_dir)
            .finish()
    }
}

impl TaskPlanRegistry {
    /// Create a registry for a workspace, rooted at its data dir (`<root>/.autoos`).
    /// Loads built-ins + any user `.atom` files under `<data>/task_plans/`.
    pub fn new(data_dir: impl AsRef<Path>) -> Self {
        let user_dir = data_dir.as_ref().join("task_plans");
        let mut registry = Self {
            plans: HashMap::new(),
            user_dir,
        };
        registry.load_builtin();
        registry.load_from_dir();
        registry
    }

    /// Load only built-in plans (useful for tests).
    pub fn load_builtins_only() -> Self {
        Self {
            plans: {
                let mut r = Self {
                    plans: HashMap::new(),
                    user_dir: PathBuf::new(),
                };
                r.load_builtin();
                r.plans
            },
            user_dir: PathBuf::new(),
        }
    }

    /// Get a plan by ID.
    pub fn get(&self, plan_id: &str) -> Option<TaskPlan> {
        self.plans.get(plan_id).map(|(plan, _)| plan.clone())
    }

    /// Get the source of a plan.
    pub fn source(&self, plan_id: &str) -> Option<TaskPlanSource> {
        self.plans.get(plan_id).map(|(_, source)| *source)
    }

    /// List all available plans.
    pub fn list(&self) -> Vec<TaskPlanSummary> {
        self.plans
            .values()
            .map(|(plan, source)| TaskPlanSummary {
                id: plan.id.clone(),
                source: *source,
                phase_count: plan.phases.len(),
                run_count: plan.phases.iter().map(|p| p.runs.len()).sum(),
            })
            .collect()
    }

    /// Insert or overwrite a plan in the registry.
    pub fn insert(&mut self, plan: TaskPlan, source: TaskPlanSource) {
        self.plans.insert(plan.id.clone(), (plan, source));
    }

    /// Remove a user plan. Built-in plans cannot be removed. Also deletes the
    /// backing `.atom` file from disk if present. Returns the removed plan.
    pub fn remove(&mut self, plan_id: &str) -> Option<TaskPlan> {
        match self.plans.get(plan_id) {
            Some((_, TaskPlanSource::Builtin)) => None,
            Some(_) => {
                let (plan, _) = self.plans.remove(plan_id)?;
                // Best-effort delete of the user file.
                let _ = std::fs::remove_file(self.user_dir.join(format!("{}.atom", plan_id)));
                Some(plan)
            }
            None => None,
        }
    }

    /// Validate a plan's structure + flow_id references against built-in flows.
    pub fn validate(&self, plan: &TaskPlan) -> Result<(), String> {
        plan.validate().map_err(|e| e.to_string())?;
        for phase in &plan.phases {
            for run in &phase.runs {
                if get_builtin_flow(&run.flow_id).is_none() {
                    return Err(format!(
                        "run '{}' references unknown flow '{}'",
                        run.name, run.flow_id
                    ));
                }
            }
        }
        Ok(())
    }

    /// Register a new TaskPlan from Atom source.
    ///
    /// Validates the Atom, checks that all referenced flows exist, writes the
    /// file to `<user_dir>/<id>.atom`, and inserts the plan into the registry.
    /// Returns the parsed plan on success.
    pub fn register(&mut self, atom: &str) -> Result<TaskPlan, String> {
        let plan = parse_task_plan(atom).map_err(|e| e.to_string())?;
        self.validate(&plan)?;

        if !self.user_dir.as_os_str().is_empty() {
            std::fs::create_dir_all(&self.user_dir).map_err(|e| e.to_string())?;
            let path = self.user_dir.join(format!("{}.atom", plan.id));
            std::fs::write(&path, atom).map_err(|e| e.to_string())?;
        }

        self.insert(plan.clone(), TaskPlanSource::User);
        Ok(plan)
    }

    fn load_builtin(&mut self) {
        for (_id, atom) in BUILTIN_TASK_PLANS {
            match parse_task_plan(atom) {
                Ok(plan) => {
                    if let Err(e) = plan.validate() {
                        tracing::error!(
                            "Built-in TaskPlan '{}' validation error: {}",
                            plan.id,
                            e
                        );
                        panic!("Built-in TaskPlan '{}' has validation errors", plan.id);
                    }
                    self.plans
                        .insert(plan.id.clone(), (plan, TaskPlanSource::Builtin));
                }
                Err(e) => {
                    tracing::error!("Failed to parse built-in TaskPlan: {}", e);
                }
            }
        }
    }

    fn load_from_dir(&mut self) {
        if !self.user_dir.is_dir() {
            return;
        }
        let Ok(entries) = std::fs::read_dir(&self.user_dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext != "atom" {
                continue;
            }
            match std::fs::read_to_string(&path) {
                Ok(content) => match parse_task_plan(&content) {
                    Ok(plan) => {
                        if let Err(e) = self.validate(&plan) {
                            tracing::error!(
                                "User TaskPlan '{}' validation error: {} (from {:?})",
                                plan.id,
                                e,
                                path
                            );
                        } else {
                            tracing::info!("Loaded TaskPlan '{}' from {:?}", plan.id, path);
                            self.plans
                                .insert(plan.id.clone(), (plan, TaskPlanSource::User));
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse TaskPlan {:?}: {}", path, e);
                    }
                },
                Err(e) => {
                    tracing::warn!("Failed to read TaskPlan {:?}: {}", path, e);
                }
            }
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relay::task_plan::{Phase, RunRef};
    use tempfile::TempDir;

    #[test]
    fn builtin_deferred_decompose_loads() {
        let registry = TaskPlanRegistry::load_builtins_only();
        let plan = registry.get("deferred-decompose");
        assert!(plan.is_some());
        let plan = plan.unwrap();
        assert_eq!(plan.phases.len(), 1);
        assert_eq!(plan.phases[0].runs.len(), 1);
    }

    #[test]
    fn cannot_remove_builtin() {
        let mut registry = TaskPlanRegistry::load_builtins_only();
        assert!(registry.remove("deferred-decompose").is_none());
        assert!(registry.get("deferred-decompose").is_some());
    }

    #[test]
    fn insert_and_remove_user_plan() {
        let mut registry = TaskPlanRegistry::load_builtins_only();
        let plan = TaskPlan::new("custom");
        registry.insert(plan, TaskPlanSource::User);
        assert!(registry.get("custom").is_some());
        assert!(registry.remove("custom").is_some());
        assert!(registry.get("custom").is_none());
    }

    #[test]
    fn list_includes_builtin_plus_user() {
        let mut registry = TaskPlanRegistry::load_builtins_only();
        registry.insert(TaskPlan::new("custom"), TaskPlanSource::User);
        let list = registry.list();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn register_validates_flow_id_against_builtins() {
        let dir = TempDir::new().unwrap();
        let mut registry = TaskPlanRegistry::new(dir.path());
        // Unknown flow → rejected.
        let bad = r#"task_plan(id: "bad") { phase(name: "p") { run(name: "r", flow_id: "nope") } }"#;
        assert!(registry.register(bad).is_err());
        // Known builtin flow "default" → accepted.
        let good = r#"task_plan(id: "good") { phase(name: "p") { run(name: "r", flow_id: "default") } }"#;
        let plan = registry.register(good).unwrap();
        assert_eq!(plan.id, "good");
        assert!(registry.get("good").is_some());
        // The user file was written.
        assert!(dir.path().join("task_plans").join("good.atom").exists());
    }

    #[test]
    fn new_loads_user_plans_from_dir() {
        let dir = TempDir::new().unwrap();
        let plans_dir = dir.path().join("task_plans");
        std::fs::create_dir_all(&plans_dir).unwrap();
        std::fs::write(
            plans_dir.join("my.atom"),
            r#"task_plan(id: "my") { phase(name: "p") { run(name: "r", flow_id: "default") } }"#,
        )
        .unwrap();
        let registry = TaskPlanRegistry::new(dir.path());
        assert!(registry.get("my").is_some());
        assert_eq!(registry.source("my"), Some(TaskPlanSource::User));
    }

    #[test]
    fn validate_rejects_plan_with_unknown_flow_directly() {
        let registry = TaskPlanRegistry::load_builtins_only();
        let plan = TaskPlan::new("x")
            .add_phase(Phase::new("p").add_run(RunRef::new("r", "nonexistent-flow")));
        assert!(registry.validate(&plan).is_err());
    }
}
