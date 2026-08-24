//! Plan tools — let the agent read/write the Plan ledger (`docs/plans/`).
//!
//! PLAN-030 T2: 6 tools wrapping the workspace's PlansStore / SpecsStore:
//! list_plans / read_plan / create_plan / update_plan / transition_plan /
//! merge_plan. Unlike spec_tools (home-dir default store), plan tools are
//! workspace-scoped — plans live in `{workspace}/docs/plans/` — so each tool
//! holds `Arc<PlansStore>` (+ `Arc<SpecsStore>` for merge) resolved from the
//! ToolContext workspace; tests inject temp stores directly.

use std::sync::Arc;

use async_trait::async_trait;
use auto_ai_agent::{Tool, ToolError, ToolOutput};
use serde_json::{json, Value};

use crate::plans::{merge_plan_stores, PlanStatus, PlansStore};
use crate::specs::SpecsStore;
use crate::tool_context::ToolContext;
use crate::workspace::WorkspaceStores;

/// Resolve the workspace's plan/spec stores from a tool context.
fn stores_of(ctx: &ToolContext) -> (Arc<PlansStore>, Arc<SpecsStore>) {
    let ws: Arc<WorkspaceStores> = ctx.state.registry.get(&ctx.workspace_id);
    (ws.plans.clone(), ws.specs.clone())
}

/// All legal next statuses from `status`（transition 报错时的提示用）。
fn legal_transitions_from(status: PlanStatus) -> Vec<&'static str> {
    use PlanStatus::*;
    [Drafting, Executing, ExecutionDone, Reviewed, Archived]
        .into_iter()
        .filter(|t| PlanStatus::can_transition(status, *t))
        .map(|t| t.as_str())
        .collect()
}

// ── list_plans ──────────────────────────────────────────────

/// List plans (seq / status / feature_name 索引)。
pub struct ListPlans {
    plans: Arc<PlansStore>,
}

impl ListPlans {
    pub fn from_ctx(ctx: &ToolContext) -> Self {
        Self {
            plans: stores_of(ctx).0,
        }
    }
    pub fn with_store(plans: Arc<PlansStore>) -> Self {
        Self { plans }
    }
}

#[async_trait]
impl Tool for ListPlans {
    fn name(&self) -> &str {
        "list_plans"
    }
    fn description(&self) -> &str {
        "List implementation plans with seq, status (drafting/executing/\
         execution_done/reviewed/archived) and feature name. Set \
         include_archived=true to also list archived plans."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "include_archived": {
                    "type": "boolean",
                    "description": "also list archived plans (default false)"
                }
            }
        })
    }
    async fn execute(&self, args: &Value) -> Result<ToolOutput, ToolError> {
        let include_archived = args["include_archived"].as_bool().unwrap_or(false);
        let list = self.plans.list(include_archived);
        if list.is_empty() {
            return Ok("(no plans)".into());
        }
        let mut out = String::from("# Plans\n\n");
        for p in list {
            out.push_str(&format!(
                "- {:03} [{}] {} — {}\n",
                p.seq,
                p.status.as_str(),
                p.feature_name,
                p.title
            ));
        }
        Ok(ToolOutput::text(out))
    }
}

// ── read_plan ───────────────────────────────────────────────

/// Read one plan's full content（含 frontmatter）。
pub struct ReadPlan {
    plans: Arc<PlansStore>,
}

impl ReadPlan {
    pub fn from_ctx(ctx: &ToolContext) -> Self {
        Self {
            plans: stores_of(ctx).0,
        }
    }
    pub fn with_store(plans: Arc<PlansStore>) -> Self {
        Self { plans }
    }
}

#[async_trait]
impl Tool for ReadPlan {
    fn name(&self) -> &str {
        "read_plan"
    }
    fn description(&self) -> &str {
        "Read a single plan file's full content (frontmatter + body) by its \
         3-digit sequence number."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "seq": { "type": "integer", "description": "3-digit plan number, e.g. 30 for PLAN-030" }
            },
            "required": ["seq"]
        })
    }
    async fn execute(&self, args: &Value) -> Result<ToolOutput, ToolError> {
        let seq = args["seq"]
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .ok_or_else(|| ToolError::Args("missing/invalid 'seq'".into()))?;
        let plan = self
            .plans
            .get(seq)
            .ok_or_else(|| ToolError::Exec(format!("plan {seq:03} not found")))?;
        Ok(ToolOutput::text(plan.content))
    }
}

// ── create_plan ─────────────────────────────────────────────

/// Create a new plan (auto-assigns max+1 seq, injects frontmatter,
/// status=drafting)。
pub struct CreatePlan {
    plans: Arc<PlansStore>,
}

impl CreatePlan {
    pub fn from_ctx(ctx: &ToolContext) -> Self {
        Self {
            plans: stores_of(ctx).0,
        }
    }
    pub fn with_store(plans: Arc<PlansStore>) -> Self {
        Self { plans }
    }
}

#[async_trait]
impl Tool for CreatePlan {
    fn name(&self) -> &str {
        "create_plan"
    }
    fn description(&self) -> &str {
        "Create a new implementation plan file under docs/plans/. Assigns the \
         next free 3-digit sequence number, injects the YAML frontmatter \
         (plan_id/status=drafting/feature_name/timestamps), and writes your \
         markdown body. Returns seq + file path."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "feature_name": { "type": "string", "description": "concise feature name" },
                "content": {
                    "type": "string",
                    "description": "markdown body with numbered sections: ## 0. 变更摘要 / ## 1. 目标 / ## 2. 架构方案 / ## 3. 技术栈 / ## 4. 需求分析与背景调查 / ## 5. 详细设计 / ## 6. 测试设计 / ## 7. 验收标准 / ## 8. 执行步骤 / ## 9. 复审记录 / ## 10. 待澄清事项"
                }
            },
            "required": ["feature_name", "content"]
        })
    }
    async fn execute(&self, args: &Value) -> Result<ToolOutput, ToolError> {
        let feature_name = args["feature_name"]
            .as_str()
            .ok_or_else(|| ToolError::Args("missing 'feature_name'".into()))?;
        let content = args["content"]
            .as_str()
            .ok_or_else(|| ToolError::Args("missing 'content'".into()))?;
        let pf = self
            .plans
            .create(feature_name, content)
            .map_err(ToolError::Exec)?;
        Ok(ToolOutput::text(json!({
            "seq": pf.seq,
            "plan_id": pf.id,
            "filename": pf.filename,
            "path": format!("docs/plans/{}", pf.filename),
            "status": pf.status.as_str(),
        })
        .to_string()))
    }
}

// ── update_plan ─────────────────────────────────────────────

/// Replace a plan's body（保留 plan_id 等 frontmatter 身份字段）。
pub struct UpdatePlan {
    plans: Arc<PlansStore>,
}

impl UpdatePlan {
    pub fn from_ctx(ctx: &ToolContext) -> Self {
        Self {
            plans: stores_of(ctx).0,
        }
    }
    pub fn with_store(plans: Arc<PlansStore>) -> Self {
        Self { plans }
    }
}

#[async_trait]
impl Tool for UpdatePlan {
    fn name(&self) -> &str {
        "update_plan"
    }
    fn description(&self) -> &str {
        "Replace a plan's full content (frontmatter identity like plan_id is \
         preserved; status/updated_at are managed by the store). Use e.g. to \
         tick execution checkboxes or fill review sections."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "seq": { "type": "integer" },
                "content": { "type": "string", "description": "full markdown content (frontmatter + body)" }
            },
            "required": ["seq", "content"]
        })
    }
    async fn execute(&self, args: &Value) -> Result<ToolOutput, ToolError> {
        let seq = args["seq"]
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .ok_or_else(|| ToolError::Args("missing/invalid 'seq'".into()))?;
        let content = args["content"]
            .as_str()
            .ok_or_else(|| ToolError::Args("missing 'content'".into()))?;
        let pf = self.plans.update(seq, content).map_err(ToolError::Exec)?;
        Ok(ToolOutput::text(format!(
            "updated plan {:03} ({} bytes, status {})",
            pf.seq,
            pf.content.len(),
            pf.status.as_str()
        )))
    }
}

// ── transition_plan ─────────────────────────────────────────

/// State-machine transition（drafting→executing→execution_done→reviewed；
/// 复审不过可回退）。`archived` 为终态，不经本工具进入——reviewed 沉淀走
/// `merge_plan`，搁置走 HTTP archive（PLAN-033 单一终态）。
pub struct TransitionPlan {
    plans: Arc<PlansStore>,
}

impl TransitionPlan {
    pub fn from_ctx(ctx: &ToolContext) -> Self {
        Self {
            plans: stores_of(ctx).0,
        }
    }
    pub fn with_store(plans: Arc<PlansStore>) -> Self {
        Self { plans }
    }
}

#[async_trait]
impl Tool for TransitionPlan {
    fn name(&self) -> &str {
        "transition_plan"
    }
    fn description(&self) -> &str {
        "Advance a plan's status machine: drafting → executing → \
         execution_done → reviewed (idempotent; review failure may go \
         back to executing). `archived` is terminal and NOT reachable via \
         this tool — reviewed plans must use merge_plan (deposit + \
         archive). Illegal transitions are rejected with the legal \
         target list."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "seq": { "type": "integer" },
                "to": {
                    "type": "string",
                    "enum": ["drafting", "executing", "execution_done", "reviewed"]
                }
            },
            "required": ["seq", "to"]
        })
    }
    async fn execute(&self, args: &Value) -> Result<ToolOutput, ToolError> {
        let seq = args["seq"]
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .ok_or_else(|| ToolError::Args("missing/invalid 'seq'".into()))?;
        let to = args["to"]
            .as_str()
            .ok_or_else(|| ToolError::Args("missing 'to'".into()))?;
        let new_status = PlanStatus::from_str_lossy(to);
        match self.plans.transition(seq, new_status) {
            Ok(pf) => Ok(ToolOutput::text(format!(
                "plan {:03} -> {} ({})",
                pf.seq,
                pf.status.as_str(),
                pf.filename
            ))),
            Err(e) => {
                let hint = self.plans.get(seq).map(|p| {
                    format!(
                        "; legal targets from {}: {:?}",
                        p.status.as_str(),
                        legal_transitions_from(p.status)
                    )
                }).unwrap_or_default();
                Err(ToolError::Exec(format!("{e}{hint}")))
            }
        }
    }
}

// ── merge_plan ──────────────────────────────────────────────

/// 沉淀：reviewed 门禁 → 拆解进 Spec 6 区 → archived（置终态 + 移档）。
pub struct MergePlan {
    plans: Arc<PlansStore>,
    specs: Arc<SpecsStore>,
}

impl MergePlan {
    pub fn from_ctx(ctx: &ToolContext) -> Self {
        let (plans, specs) = stores_of(ctx);
        Self { plans, specs }
    }
    pub fn with_stores(plans: Arc<PlansStore>, specs: Arc<SpecsStore>) -> Self {
        Self { plans, specs }
    }
}

#[async_trait]
impl Tool for MergePlan {
    fn name(&self) -> &str {
        "merge_plan"
    }
    fn description(&self) -> &str {
        "Deposit a reviewed plan into the Spec ledger: extracts mapped \
         sections (变更摘要→reports, 目标→goals, 架构方案→architecture, \
         详细设计→designs, 测试设计→tests, 验收标准/复审记录→reviews) as \
         id-stable items, sets the plan to archived (terminal) and moves \
         it into archived/. Gate: the plan must be reviewed."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "seq": { "type": "integer" }
            },
            "required": ["seq"]
        })
    }
    async fn execute(&self, args: &Value) -> Result<ToolOutput, ToolError> {
        let seq = args["seq"]
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .ok_or_else(|| ToolError::Args("missing/invalid 'seq'".into()))?;
        let result = merge_plan_stores(&self.plans, &self.specs, seq).map_err(ToolError::Exec)?;
        Ok(ToolOutput::text(json!({
            "plan_id": result.plan_id,
            "sections_touched": result.sections_touched,
            "items_created": result.items_created,
            "archived": true,
        })
        .to_string()))
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn tmp_stores() -> (Arc<PlansStore>, Arc<SpecsStore>) {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "musk_plan_tools_test_{}_{}",
            std::process::id(),
            n
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("docs/plans")).unwrap();
        (
            Arc::new(PlansStore::new(dir.join("docs/plans"))),
            Arc::new(SpecsStore::new(dir.join(".autoos/specs.json"))),
        )
    }

    const BODY: &str = "# [PLAN-001] 测试计划\n\n## 0. 变更摘要\n\n摘要。\n\n## 1. 目标\n\n目标内容。\n\n## 2. 架构方案\n\n架构内容。\n\n## 3. 技术栈\n\nRust。\n\n## 4. 需求分析与背景调查\n\n背景。\n\n## 5. 详细设计\n\n设计。\n\n## 6. 测试设计\n\n测试。\n\n## 7. 验收标准\n\n- [ ] A1\n\n## 8. 执行步骤\n\n- [ ] T1\n\n## 9. 复审记录\n\n复审通过。\n\n## 10. 待澄清事项\n\n无。\n";

    #[tokio::test]
    async fn create_then_list_and_read() {
        let (plans, _specs) = tmp_stores();
        let c = CreatePlan::with_store(plans.clone());
        let out = c
            .execute(&json!({ "feature_name": "演示功能", "content": BODY }))
            .await
            .unwrap();
        assert!(out.content.contains("\"seq\":1"));
        assert!(out.content.contains("docs/plans/001-"));

        let l = ListPlans::with_store(plans.clone());
        let out = l.execute(&json!({})).await.unwrap();
        let out = out.content;
        assert!(out.contains("001"));
        assert!(out.contains("[drafting]"));
        assert!(out.contains("演示功能"));

        let r = ReadPlan::with_store(plans);
        let out = r.execute(&json!({ "seq": 1 })).await.unwrap();
        let out = out.content;
        assert!(out.contains("## 1. 目标"));
        assert!(out.contains("目标内容。"));
    }

    #[tokio::test]
    async fn update_plan_replaces_body_preserving_identity() {
        let (plans, _specs) = tmp_stores();
        CreatePlan::with_store(plans.clone())
            .execute(&json!({ "feature_name": "x", "content": BODY }))
            .await
            .unwrap();
        let new_body = BODY.replace("- [ ] T1", "- [x] T1 已完成");
        let u = UpdatePlan::with_store(plans.clone());
        let out = u.execute(&json!({ "seq": 1, "content": new_body })).await.unwrap();
        let out = out.content;
        assert!(out.contains("updated plan 001"));

        let r = ReadPlan::with_store(plans);
        let out = r.execute(&json!({ "seq": 1 })).await.unwrap();
        let out = out.content;
        assert!(out.contains("- [x] T1 已完成"));
        assert!(out.contains("PLAN-001"), "plan_id preserved");
    }

    #[tokio::test]
    async fn transition_plan_validates_and_hints_legal_targets() {
        let (plans, _specs) = tmp_stores();
        CreatePlan::with_store(plans.clone())
            .execute(&json!({ "feature_name": "x", "content": BODY }))
            .await
            .unwrap();
        let t = TransitionPlan::with_store(plans.clone());

        // drafting → archived 非法（终态不经 transition），报错附合法目标集
        let err = t.execute(&json!({ "seq": 1, "to": "archived" })).await.unwrap_err();
        match err {
            ToolError::Exec(msg) => {
                assert!(msg.contains("illegal transition"));
                assert!(msg.contains("legal targets from drafting"));
            }
            other => panic!("expected Exec error, got {:?}", other),
        }

        // drafting → executing 合法
        let out = t.execute(&json!({ "seq": 1, "to": "executing" })).await.unwrap();
        let out = out.content;
        assert!(out.contains("plan 001 -> executing"));
    }

    #[tokio::test]
    async fn merge_plan_gates_on_reviewed_and_deposits() {
        let (plans, specs) = tmp_stores();
        CreatePlan::with_store(plans.clone())
            .execute(&json!({ "feature_name": "沉淀演示", "content": BODY }))
            .await
            .unwrap();

        // drafting 直接 merge → 门禁报错
        let m = MergePlan::with_stores(plans.clone(), specs.clone());
        let err = m.execute(&json!({ "seq": 1 })).await.unwrap_err();
        assert!(format!("{err:?}").contains("reviewed"));

        // drafting → reviewed（跳过执行直接复审，合法路径）
        TransitionPlan::with_store(plans.clone())
            .execute(&json!({ "seq": 1, "to": "reviewed" }))
            .await
            .unwrap();

        let out = m.execute(&json!({ "seq": 1 })).await.unwrap();
        let out = out.content;
        assert!(out.contains("\"items_created\":7"));
        assert!(out.contains("archived"));

        // specs 落 6 区：goals 有 P001- item
        let doc = specs.load().unwrap();
        let goals = doc.sections.iter().find(|s| s.id == "goals").unwrap();
        assert!(goals.items.iter().any(|i| i.id.starts_with("P001-")));

        // plan 已归档：active list 空，archived list 有
        let l = ListPlans::with_store(plans.clone());
        assert_eq!(l.execute(&json!({})).await.unwrap().content, "(no plans)");
        let out = l.execute(&json!({ "include_archived": true })).await.unwrap();
        let out = out.content;
        assert!(out.contains("[archived]"));
    }
}
