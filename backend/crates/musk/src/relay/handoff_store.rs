//! Cross-run handoff storage for TaskPlan execution.
//!
//! When a relay run that belongs to a TaskPlan completes, its final handoff
//! is persisted here keyed by (task_plan_id, phase_name, run_name). Later
//! phases can reference it via `input_from: "phase.run.handoff.field"`.
//!
//! Ported from auto-forge `relay/handoff_store.rs` (Plan 009 P2b.7).
//! musk adaptations: `.autoforge/` → `.autoos/` paths; uses musk's
//! `HandoffDocument` (field `target` not `to`; `TokenUsage.step_tokens` not
//! `step_input/step_output`); resolves handoffs via `RunStore::last_handoff`.

use crate::relay::HandoffDocument;
use crate::relay::RunStore;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

/// Persist and query handoffs across TaskPlan runs.
///
/// Rooted at a workspace's data directory (`.autoos/`); one instance per
/// workspace, held in `WorkspaceStores`.
#[derive(Debug)]
pub struct HandoffStore {
    data_dir: PathBuf,
    /// Optional in-memory cache keyed by (task_plan_id, phase, run).
    cache: std::sync::Mutex<HashMap<(String, String, String), HandoffDocument>>,
}

impl HandoffStore {
    /// Create a store rooted at the workspace data directory (e.g. `<root>/.autoos`).
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            data_dir: data_dir.into(),
            cache: std::sync::Mutex::new(HashMap::new()),
        }
    }

    /// Directory where handoffs are persisted for a given task plan.
    fn handoffs_dir(&self, task_plan_id: &str) -> PathBuf {
        self.data_dir
            .join("task_plans")
            .join(".handoffs")
            .join(task_plan_id)
    }

    /// File path for a specific handoff.
    fn handoff_path(&self, task_plan_id: &str, phase: &str, run: &str) -> PathBuf {
        self.handoffs_dir(task_plan_id)
            .join(phase)
            .join(format!("{}.json", run))
    }

    /// Save a handoff to disk and cache.
    pub fn save(
        &self,
        task_plan_id: &str,
        phase: &str,
        run: &str,
        handoff: &HandoffDocument,
    ) -> Result<(), String> {
        let path = self.handoff_path(task_plan_id, phase, run);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("failed to create handoff dir: {}", e))?;
        }
        let json = serde_json::to_string_pretty(handoff)
            .map_err(|e| format!("failed to serialize handoff: {}", e))?;
        std::fs::write(&path, json)
            .map_err(|e| format!("failed to write handoff {:?}: {}", path, e))?;

        self.cache.lock().unwrap().insert(
            (
                task_plan_id.to_string(),
                phase.to_string(),
                run.to_string(),
            ),
            handoff.clone(),
        );
        Ok(())
    }

    /// Load a handoff from cache or disk.
    pub fn load(&self, task_plan_id: &str, phase: &str, run: &str) -> Option<HandoffDocument> {
        let key = (
            task_plan_id.to_string(),
            phase.to_string(),
            run.to_string(),
        );
        if let Some(doc) = self.cache.lock().unwrap().get(&key) {
            return Some(doc.clone());
        }
        let path = self.handoff_path(task_plan_id, phase, run);
        let content = std::fs::read_to_string(&path).ok()?;
        let doc: HandoffDocument = serde_json::from_str(&content).ok()?;
        self.cache.lock().unwrap().insert(key, doc.clone());
        Some(doc)
    }

    /// Resolve a path like `task_plan_id.phase.run.handoff.field` to a JSON value.
    ///
    /// Supported first-level fields: `summary`, `decisions`, `open_questions`,
    /// `work_product`, `context_for_next`, `token_usage` (with arbitrary nested
    /// access via serde_json pointer semantics, e.g. `.token_usage.cumulative`).
    ///
    /// Returns `None` if the handoff or field does not exist.
    pub fn resolve_path(&self, path: &str) -> Option<Value> {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.len() < 5 || parts[3] != "handoff" {
            return None;
        }
        let task_plan_id = parts[0];
        let phase = parts[1];
        let run = parts[2];
        let handoff = self.load(task_plan_id, phase, run)?;

        let doc_json = serde_json::to_value(&handoff).ok()?;
        let mut value = &doc_json;
        for part in &parts[4..] {
            value = value.get(part)?;
        }
        Some(value.clone())
    }

    /// Collect the final handoff from a completed relay run and save it.
    /// Uses musk's `RunStore::last_handoff`. Returns the saved handoff if found.
    pub fn save_from_run(
        &self,
        store: &RunStore,
        task_plan_id: &str,
        phase: &str,
        run_name: &str,
        run_id: &str,
    ) -> Option<HandoffDocument> {
        let handoff = store.last_handoff(run_id)?;
        self.save(task_plan_id, phase, run_name, &handoff)
            .map_err(|e| tracing::warn!("Failed to save handoff: {}", e))
            .ok()?;
        Some(handoff)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relay::HandoffDocument;
    use tempfile::TempDir;

    #[test]
    fn save_and_load_handoff() {
        let dir = TempDir::new().unwrap();
        let store = HandoffStore::new(dir.path());
        let handoff = HandoffDocument::new("coder", "tester");
        store.save("tp", "phase", "run", &handoff).unwrap();
        let loaded = store.load("tp", "phase", "run").unwrap();
        assert_eq!(loaded.from, "coder");
        assert_eq!(loaded.to, "tester");
    }

    #[test]
    fn resolve_path_summary_and_nested_token() {
        let dir = TempDir::new().unwrap();
        let store = HandoffStore::new(dir.path());
        let mut handoff = HandoffDocument::new("coder", "tester");
        handoff.summary = "Implemented auth".to_string();
        handoff.token_usage.step_tokens = 100;
        handoff.token_usage.cumulative = 150;
        store.save("tp", "phase", "run", &handoff).unwrap();

        let summary = store.resolve_path("tp.phase.run.handoff.summary");
        assert_eq!(summary, Some(Value::String("Implemented auth".to_string())));

        let cumulative = store.resolve_path("tp.phase.run.handoff.token_usage.cumulative");
        assert_eq!(cumulative, Some(Value::Number(150.into())));
    }

    #[test]
    fn missing_handoff_returns_none() {
        let dir = TempDir::new().unwrap();
        let store = HandoffStore::new(dir.path());
        assert!(store.load("tp", "phase", "run").is_none());
        assert!(store
            .resolve_path("tp.phase.run.handoff.summary")
            .is_none());
    }

    #[test]
    fn rejects_path_without_handoff_segment() {
        let dir = TempDir::new().unwrap();
        let store = HandoffStore::new(dir.path());
        // Third segment is not "handoff" → None.
        assert!(store.resolve_path("tp.phase.run.output.summary").is_none());
    }
}
