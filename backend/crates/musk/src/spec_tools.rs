//! Spec tools — let the agent read/write the Spec Ledger.
//!
//! Implements 5 tools (Plan 009 P1a): read_specs / list_specs / write_spec /
//! update_spec / write_goals. Each holds an `Arc<SpecsStore>` so it can load +
//! mutate the JSON-backed spec document. Mirrors auto-forge `tools.rs:1929-2580`
//! (adapted to musk's SpecsStore API + per-section state machine).

use std::sync::Arc;

use async_trait::async_trait;
use auto_ai_agent::{Tool, ToolError};
use serde_json::{json, Value};

use crate::specs::{SpecItem, SpecStatus, SpecsStore};

/// The default spec-store path (matches server.rs AppState::specs).
fn default_store() -> Arc<SpecsStore> {
    let path = dirs::home_dir()
        .map(|h| h.join(".config/autoos/specs.json"))
        .unwrap_or_else(|| std::path::PathBuf::from("specs.json"));
    Arc::new(SpecsStore::new(path))
}

// ── read_specs ─────────────────────────────────────────────

/// Read one section's items (or the whole document if no section_id).
pub struct ReadSpecs {
    store: Arc<SpecsStore>,
}

impl ReadSpecs {
    pub fn new() -> Self {
        Self { store: default_store() }
    }
    pub fn with_store(store: Arc<SpecsStore>) -> Self {
        Self { store }
    }
}

#[async_trait]
impl Tool for ReadSpecs {
    fn name(&self) -> &str {
        "read_specs"
    }
    fn description(&self) -> &str {
        "Read spec items. If `section_id` is given (e.g. 'goals', 'plans'), \
         return that section's items; otherwise return a compact list of all \
         sections with their item ids + titles + statuses."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "section_id": { "type": "string", "description": "optional section id (goals/architecture/designs/plans/tests/reviews/reports)" }
            }
        })
    }
    async fn execute(&self, args: &Value) -> Result<String, ToolError> {
        let mut doc = self
            .store
            .load()
            .map_err(|e| ToolError::Exec(format!("load specs: {e}")))?;
        doc.rebuild_relations();
        doc.derive_statuses();
        let section_id = args["section_id"].as_str();
        if let Some(sid) = section_id {
            let section = doc
                .sections
                .iter()
                .find(|s| s.id == sid)
                .ok_or_else(|| ToolError::Exec(format!("section '{sid}' not found")))?;
            let mut out = format!("# {}\n\n", section.title);
            for it in &section.items {
                out.push_str(&format!(
                    "- **{}** {} [{}]\n  {}\n",
                    it.id,
                    it.title,
                    it.status.to_str(),
                    it.content.lines().next().unwrap_or("")
                ));
            }
            Ok(out)
        } else {
            let mut out = String::from("# Spec overview\n\n");
            for s in &doc.sections {
                out.push_str(&format!(
                    "## {} ({} items, {})\n",
                    s.title,
                    s.items.len(),
                    s.status.to_str()
                ));
                for it in &s.items {
                    out.push_str(&format!(
                        "  - {} {} [{}]\n",
                        it.id,
                        it.title,
                        it.status.to_str()
                    ));
                }
            }
            Ok(out)
        }
    }
}

// ── list_specs ─────────────────────────────────────────────

/// List all section ids with item counts (compact index).
pub struct ListSpecs {
    store: Arc<SpecsStore>,
}

impl ListSpecs {
    pub fn new() -> Self {
        Self { store: default_store() }
    }
    pub fn with_store(store: Arc<SpecsStore>) -> Self {
        Self { store }
    }
}

#[async_trait]
impl Tool for ListSpecs {
    fn name(&self) -> &str {
        "list_specs"
    }
    fn description(&self) -> &str {
        "List all spec sections with their item counts and aggregate statuses \
         (a compact index of the whole spec ledger)."
    }
    fn parameters(&self) -> Value {
        json!({ "type": "object", "properties": {} })
    }
    async fn execute(&self, _args: &Value) -> Result<String, ToolError> {
        let doc = self
            .store
            .load()
            .map_err(|e| ToolError::Exec(format!("load specs: {e}")))?;
        let ov = doc.overview();
        let mut out = format!(
            "project: {} (v{}, {} items)\n",
            ov.project, ov.version, ov.total_items
        );
        for s in &ov.sections {
            out.push_str(&format!(
                "  {} — {} items, {}\n",
                s.id, s.item_count, s.status.to_str()
            ));
        }
        Ok(out)
    }
}

// ── update_spec ────────────────────────────────────────────

/// Upsert / delete / patch / set_status a spec item.
pub struct UpdateSpec {
    store: Arc<SpecsStore>,
}

impl UpdateSpec {
    pub fn new() -> Self {
        Self { store: default_store() }
    }
    pub fn with_store(store: Arc<SpecsStore>) -> Self {
        Self { store }
    }
}

#[async_trait]
impl Tool for UpdateSpec {
    fn name(&self) -> &str {
        "update_spec"
    }
    fn description(&self) -> &str {
        "Modify a spec item. `action`: upsert (create/replace), delete, \
         set_status. For upsert provide section_id + item (id/title/content/\
         status/depends_on/tags). For set_status provide section_id + item_id \
         + new_status (must be a legal transition for that section type)."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": { "type": "string", "enum": ["upsert", "delete", "set_status"] },
                "section_id": { "type": "string" },
                "item_id": { "type": "string", "description": "for delete/set_status" },
                "item": { "type": "object", "description": "full SpecItem for upsert" },
                "new_status": { "type": "string", "description": "for set_status" }
            },
            "required": ["action", "section_id"]
        })
    }
    async fn execute(&self, args: &Value) -> Result<String, ToolError> {
        let action = args["action"]
            .as_str()
            .ok_or_else(|| ToolError::Args("missing 'action'".into()))?;
        let section_id = args["section_id"]
            .as_str()
            .ok_or_else(|| ToolError::Args("missing 'section_id'".into()))?;
        let mut doc = self
            .store
            .load()
            .map_err(|e| ToolError::Exec(format!("load specs: {e}")))?;

        match action {
            "upsert" => {
                let item: SpecItem = serde_json::from_value(args["item"].clone())
                    .map_err(|e| ToolError::Args(format!("invalid item: {e}")))?;
                let id = item.id.clone();
                self.store
                    .upsert_item(&mut doc, section_id, item)
                    .map_err(ToolError::Exec)?;
                self.store
                    .save(&doc)
                    .map_err(|e| ToolError::Exec(format!("save: {e}")))?;
                Ok(format!("upserted {id} into {section_id} (v{})", doc.version))
            }
            "delete" => {
                let item_id = args["item_id"]
                    .as_str()
                    .ok_or_else(|| ToolError::Args("missing 'item_id'".into()))?;
                let removed = self
                    .store
                    .delete_item(&mut doc, section_id, item_id)
                    .map_err(ToolError::Exec)?;
                self.store
                    .save(&doc)
                    .map_err(|e| ToolError::Exec(format!("save: {e}")))?;
                if removed {
                    Ok(format!("deleted {item_id} from {section_id} (v{})", doc.version))
                } else {
                    Ok(format!("{item_id} not found in {section_id}"))
                }
            }
            "set_status" => {
                let item_id = args["item_id"]
                    .as_str()
                    .ok_or_else(|| ToolError::Args("missing 'item_id'".into()))?;
                let new_status_str = args["new_status"]
                    .as_str()
                    .ok_or_else(|| ToolError::Args("missing 'new_status'".into()))?;
                let new_status = SpecStatus::from_str_lossy(new_status_str);
                self.store
                    .transition_item(&mut doc, section_id, item_id, new_status)
                    .map_err(ToolError::Exec)?;
                self.store
                    .save(&doc)
                    .map_err(|e| ToolError::Exec(format!("save: {e}")))?;
                Ok(format!(
                    "{item_id} -> {} (v{})",
                    new_status.to_str(),
                    doc.version
                ))
            }
            other => Err(ToolError::Args(format!("unknown action '{other}'"))),
        }
    }
}

// ── write_spec ─────────────────────────────────────────────

/// Write a whole section's content as free text (creates items from headings).
/// Simplified vs auto-forge: upserts one item per `## ID Title` heading found
/// in `content`. If no headings, upserts a single item with the given id.
pub struct WriteSpec {
    store: Arc<SpecsStore>,
}

impl WriteSpec {
    pub fn new() -> Self {
        Self { store: default_store() }
    }
    pub fn with_store(store: Arc<SpecsStore>) -> Self {
        Self { store }
    }
}

#[async_trait]
impl Tool for WriteSpec {
    fn name(&self) -> &str {
        "write_spec"
    }
    fn description(&self) -> &str {
        "Write a section's spec content as markdown. Each `## ID Title` heading \
         becomes/upserts a spec item with the body as its content. Requires \
         section_id + content."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "section_id": { "type": "string" },
                "content": { "type": "string", "description": "markdown with ## ID Title headings" }
            },
            "required": ["section_id", "content"]
        })
    }
    async fn execute(&self, args: &Value) -> Result<String, ToolError> {
        let section_id = args["section_id"]
            .as_str()
            .ok_or_else(|| ToolError::Args("missing 'section_id'".into()))?;
        let content = args["content"]
            .as_str()
            .ok_or_else(|| ToolError::Args("missing 'content'".into()))?;
        let mut doc = self
            .store
            .load()
            .map_err(|e| ToolError::Exec(format!("load specs: {e}")))?;

        // Parse `## ID Title` headings; body until next heading.
        let mut items: Vec<(String, String, String)> = Vec::new(); // id, title, body
        let mut cur: Option<(String, String, String)> = None;
        for line in content.lines() {
            if let Some(rest) = line.strip_prefix("## ") {
                if let Some(c) = cur.take() {
                    items.push(c);
                }
                // split "ID Title..." → id + title
                let (id, title) = match rest.split_once(char::is_whitespace) {
                    Some((i, t)) => (i.trim().to_string(), t.trim().to_string()),
                    None => (rest.trim().to_string(), String::new()),
                };
                cur = Some((id, title, String::new()));
            } else if let Some((_, _, body)) = cur.as_mut() {
                if !body.is_empty() || !line.trim().is_empty() {
                    body.push_str(line);
                    body.push('\n');
                }
            }
        }
        if let Some(c) = cur.take() {
            items.push(c);
        }

        if items.is_empty() {
            return Err(ToolError::Args(
                "no `## ID Title` headings found in content".into(),
            ));
        }

        let mut count = 0;
        for (id, title, body) in items {
            let mut item = SpecItem::new(id.clone(), title);
            item.content = body.trim_end().to_string();
            self.store
                .upsert_item(&mut doc, section_id, item)
                .map_err(ToolError::Exec)?;
            count += 1;
        }
        self.store
            .save(&doc)
            .map_err(|e| ToolError::Exec(format!("save: {e}")))?;
        Ok(format!("wrote {count} items into {section_id} (v{})", doc.version))
    }
}

// ── write_goals ────────────────────────────────────────────

/// Convenience: write goals as a bullet list (one item per `- [ ] text`).
pub struct WriteGoals {
    store: Arc<SpecsStore>,
}

impl WriteGoals {
    pub fn new() -> Self {
        Self { store: default_store() }
    }
    pub fn with_store(store: Arc<SpecsStore>) -> Self {
        Self { store }
    }
}

#[async_trait]
impl Tool for WriteGoals {
    fn name(&self) -> &str {
        "write_goals"
    }
    fn description(&self) -> &str {
        "Write goals from a markdown bullet list. Each `- [ ] text` or \
         `- text` line becomes a goal item (auto-id G1, G2, …). Replaces all \
         existing goals."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "goals": { "type": "string", "description": "markdown bullet list of goals" }
            },
            "required": ["goals"]
        })
    }
    async fn execute(&self, args: &Value) -> Result<String, ToolError> {
        let goals = args["goals"]
            .as_str()
            .ok_or_else(|| ToolError::Args("missing 'goals'".into()))?;
        let mut doc = self
            .store
            .load()
            .map_err(|e| ToolError::Exec(format!("load specs: {e}")))?;

        // clear existing goals
        if let Some(section) = doc.sections.iter_mut().find(|s| s.id == "goals") {
            section.items.clear();
        }

        let mut idx = 1;
        for line in goals.lines() {
            let trimmed = line.trim();
            let text = trimmed
                .strip_prefix("- [ ] ")
                .or_else(|| trimmed.strip_prefix("- [x] "))
                .or_else(|| trimmed.strip_prefix("- "))
                .unwrap_or(trimmed);
            if text.is_empty() {
                continue;
            }
            let id = format!("G{idx}");
            let item = SpecItem::new(id, text);
            self.store
                .upsert_item(&mut doc, "goals", item)
                .map_err(ToolError::Exec)?;
            idx += 1;
        }
        self.store
            .save(&doc)
            .map_err(|e| ToolError::Exec(format!("save: {e}")))?;
        Ok(format!("wrote {} goals (v{})", idx - 1, doc.version))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn tmp_store() -> Arc<SpecsStore> {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!(
            "musk_spec_tools_test_{}_{}.json",
            std::process::id(),
            n
        ));
        let _ = std::fs::remove_file(&path);
        Arc::new(SpecsStore::new(path))
    }

    #[tokio::test]
    async fn read_specs_empty_doc_overview() {
        let store = tmp_store();
        let t = ReadSpecs::with_store(store);
        let out = t.execute(&json!({})).await.unwrap();
        assert!(out.contains("Spec overview"));
    }

    #[tokio::test]
    async fn write_goals_then_read() {
        let store = tmp_store();
        let w = WriteGoals::with_store(store.clone());
        let out = w
            .execute(&json!({ "goals": "- [ ] first goal\n- second goal\n- [x] done one" }))
            .await
            .unwrap();
        assert!(out.contains("wrote 3 goals"));

        let r = ReadSpecs::with_store(store);
        let out = r.execute(&json!({ "section_id": "goals" })).await.unwrap();
        assert!(out.contains("G1"));
        assert!(out.contains("first goal"));
        assert!(out.contains("done one"));
    }

    #[tokio::test]
    async fn update_spec_upsert_and_set_status() {
        let store = tmp_store();
        let u = UpdateSpec::with_store(store.clone());
        let out = u
            .execute(&json!({
                "action": "upsert", "section_id": "goals",
                "item": {
                    "id": "G1", "title": "test goal", "content": "",
                    "status": "Empty", "depends_on": [], "related": [],
                    "priority": null, "assignee": null, "test_file": null,
                    "file": null, "milestone": null, "module": null, "tags": [],
                    "created_at": 0, "modified_at": 0, "completed_at": null
                }
            }))
            .await
            .unwrap();
        assert!(out.contains("upserted G1"));

        let out = u
            .execute(&json!({
                "action": "set_status", "section_id": "goals",
                "item_id": "G1", "new_status": "proposed"
            }))
            .await
            .unwrap();
        assert!(out.contains("proposed"));
    }

    #[tokio::test]
    async fn update_spec_set_status_rejects_illegal() {
        let store = tmp_store();
        let u = UpdateSpec::with_store(store.clone());
        u.execute(&json!({
            "action": "upsert", "section_id": "goals",
            "item": {
                "id": "G1", "title": "g", "content": "", "status": "Empty",
                "depends_on": [], "related": [], "priority": null, "assignee": null,
                "test_file": null, "file": null, "milestone": null, "module": null,
                "tags": [], "created_at": 0, "modified_at": 0, "completed_at": null
            }
        }))
        .await
        .unwrap();
        let err = u
            .execute(&json!({
                "action": "set_status", "section_id": "goals",
                "item_id": "G1", "new_status": "done"
            }))
            .await
            .unwrap_err();
        assert!(matches!(err, ToolError::Exec(_)));
    }

    #[tokio::test]
    async fn write_spec_parses_headings() {
        let store = tmp_store();
        let w = WriteSpec::with_store(store.clone());
        let out = w
            .execute(&json!({
                "section_id": "plans",
                "content": "## P1 First plan\nDo the thing\n## P2 Second\nOther\n"
            }))
            .await
            .unwrap();
        assert!(out.contains("wrote 2 items"));

        let r = ReadSpecs::with_store(store);
        let out = r.execute(&json!({ "section_id": "plans" })).await.unwrap();
        assert!(out.contains("P1"));
        assert!(out.contains("First plan"));
        assert!(out.contains("Do the thing"));
    }

    #[tokio::test]
    async fn list_specs_shows_counts() {
        let store = tmp_store();
        let w = WriteGoals::with_store(store.clone());
        w.execute(&json!({ "goals": "- a\n- b" })).await.unwrap();
        let l = ListSpecs::with_store(store);
        let out = l.execute(&json!({})).await.unwrap();
        assert!(out.contains("goals"));
        assert!(out.contains("2 items"));
    }
}
