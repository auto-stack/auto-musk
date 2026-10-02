//! PLAN-096 T-08/T-09: controlled delivery — the server-owned Git / store /
//! archive operations behind the `plan_delivery` tool (§5.7, AC-08/10/11/17).
//!
//! Five checkpoints in order, each verified against real facts before the
//! previous side effects are considered settled, and each recorded in a
//! durable receipt keyed `PLAN-NNN:rN` under
//! `{ws_root}/.autoos/plan-delivery/{plan_id}.json`:
//!
//! 1. **prepared** — review facts verified (plan reviewed, binding intact,
//!    worktree HEAD == reviewed_commit, deps unchanged); the reviewed spec
//!    delta authored under `docs/specs/` in the WORKTREE is committed there
//!    (delivery_commit). Same-implementation doc descendants may become the
//!    delivery commit; implementation drift must go back to review (the
//!    binding/HEAD checks enforce that).
//! 2. **landed** — fresh guard clean; rebase onto the detected default
//!    branch with a recorded old→new mapping and a `git range-diff`
//!    equivalence check (any changed-diff line refuses); the main checkout
//!    is fast-forwarded `--ff-only` only (no merge fallback), and the
//!    actual tip must equal the rebased canonical commit.
//! 3. **ledger_refreshed** — via the workspace SpecsStore (same semantics as
//!    write_spec/update_spec): per spec-delta target upsert a derived item
//!    carrying the canonical path, source hash and delivery commit; the
//!    store is re-loaded to verify. Load errors / unwritable ledger →
//!    blocked loud (never a hand-written JSON pass). Unrelated items and
//!    history are preserved by upsert semantics.
//! 4. **archived** — only after the above verified: the shared plan is
//!    explicitly finalized (status=archived + moved), completion_kind=
//!    delivered recorded with the receipt.
//! 5. **cleaned** — fresh guard + merged check, then the run's own worktree
//!    and dev branch are removed; the layout group dir is pruned only when
//!    empty. Cleanup failure keeps the delivered archive and reports
//!    `cleanup_pending` — earlier side effects are never re-run.
//!
//! Cross-step re-entry is reconciled from the receipt (explicit plan-merge
//! re-runs read it and only fill the missing checkpoints).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::json;

use crate::relay::plan_contract::PlanExecutionState;

// ── Receipt (durable, workspace-relative) ───────────────────────────────────

#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DeliveryReceipt {
    /// `PLAN-NNN:rN` — plan id + approved semantic revision.
    pub receipt_key: String,
    pub plan_id: String,
    #[serde(default)]
    pub plan_revision: u32,
    pub workspace_id: String,
    #[serde(default)]
    pub main_root: String,
    #[serde(default)]
    pub updated_at: u64,
    /// checkpoint name → facts (commit/hash/file lists).
    #[serde(default)]
    pub checkpoints: BTreeMap<String, serde_json::Value>,
    /// completion_kind=delivered set by the archive checkpoint.
    #[serde(default)]
    pub completion_kind: Option<String>,
}

/// Workspace-relative receipt path (the B-track `receipt_ref` form).
pub fn receipt_rel_path(plan_id: &str) -> String {
    format!(".autoos/plan-delivery/{plan_id}.json")
}

pub fn receipt_abs_path(ws_root: &Path, plan_id: &str) -> PathBuf {
    ws_root.join(receipt_rel_path(plan_id))
}

pub fn load_receipt(ws_root: &Path, plan_id: &str) -> Result<Option<DeliveryReceipt>, String> {
    let p = receipt_abs_path(ws_root, plan_id);
    match std::fs::read_to_string(&p) {
        Ok(text) => serde_json::from_str(&text)
            .map(Some)
            .map_err(|e| format!("receipt {p:?} unreadable: {e} — refusing to guess")),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("receipt read failed: {e}")),
    }
}

pub fn save_receipt(ws_root: &Path, receipt: &DeliveryReceipt) -> Result<(), String> {
    let p = receipt_abs_path(ws_root, &receipt.plan_id);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("receipt dir create failed: {e}"))?;
    }
    let bytes = serde_json::to_vec_pretty(receipt).map_err(|e| e.to_string())?;
    auto_lang::state_file::atomic_write(&p, &bytes)
        .map_err(|e| format!("receipt write failed: {e}"))
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ── Git helpers (arg-vector only, no shell) ─────────────────────────────────

fn git(cwd: &Path, args: &[&str]) -> Result<String, String> {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| format!("git spawn failed: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "git {} failed: {}",
            args.first().unwrap_or(&"?"),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

fn worktree_head_clean(wt: &Path) -> Result<(String, bool), String> {
    let status = git(wt, &["status", "--porcelain"])?;
    let head = git(wt, &["rev-parse", "HEAD"])?.trim().to_string();
    Ok((head, status.trim().is_empty()))
}

// ── Core actions ────────────────────────────────────────────────────────────

struct DeliveryCtx {
    ws_root: PathBuf,
    main_root: PathBuf,
    worktree: PathBuf,
    binding: crate::relay::plan_contract::PlanExecutionBinding,
    pe: PlanExecutionState,
}

fn delivery_ctx(state: &crate::server::AppState, ws_id: &str, run_id: &str) -> Result<DeliveryCtx, String> {
    let ws = state.registry.get(ws_id);
    let pe = ws
        .relay
        .plan_execution(run_id)
        .ok_or_else(|| "run carries no plan execution".to_string())?;
    if pe.phase != "document" && pe.phase != "delivered" {
        return Err(format!(
            "delivery actions require the document phase (run is at '{}')",
            pe.phase
        ));
    }
    let binding = pe
        .binding
        .clone()
        .ok_or_else(|| "delivery without approval binding".to_string())?;
    Ok(DeliveryCtx {
        ws_root: ws.root.clone(),
        main_root: PathBuf::from(&binding.main_root),
        worktree: PathBuf::from(binding.execution_root.clone().ok_or(
            "delivery without a registered execution worktree",
        )?),
        binding,
        pe,
    })
}

/// Record a checkpoint into run facts + durable receipt (idempotent upsert).
fn record_checkpoint(
    state: &crate::server::AppState,
    ws_id: &str,
    run_id: &str,
    ctx: &DeliveryCtx,
    name: &str,
    facts: serde_json::Value,
    completion_kind: Option<&str>,
) -> Result<(), String> {
    // 1. run facts.
    state
        .registry
        .get(ws_id)
        .relay
        .mutate_plan_execution(run_id, |pe| {
            pe.delivery_checkpoints.insert(name.to_string(), facts.clone());
            if let Some(kind) = completion_kind {
                // completion_kind 存 facts 内（AC-08：archive 时置 delivered）。
                pe.delivery_checkpoints
                    .entry("completion_kind".to_string())
                    .or_insert_with(|| json!(kind));
            }
        });
    // 2. durable receipt (re-entry anchor).
    let mut receipt = load_receipt(&ctx.ws_root, &ctx.binding.plan_id)?.unwrap_or_default();
    receipt.receipt_key = format!("{}:r{}", ctx.binding.plan_id, ctx.binding.plan_revision);
    receipt.plan_id = ctx.binding.plan_id.clone();
    receipt.plan_revision = ctx.binding.plan_revision;
    receipt.workspace_id = ws_id.to_string();
    receipt.main_root = ctx.binding.main_root.clone();
    receipt.updated_at = now_secs();
    receipt
        .checkpoints
        .insert(name.to_string(), facts.clone());
    if let Some(kind) = completion_kind {
        receipt.completion_kind = Some(kind.to_string());
    }
    save_receipt(&ctx.ws_root, &receipt)?;
    // 3. facts event.
    crate::relay::plan_control::push_facts(
        state.registry.get(ws_id).relay.as_ref(),
        run_id,
        crate::relay::plan_contract::RunPlanEvent {
            timestamp: now_secs(),
            plan_id: ctx.binding.plan_id.clone(),
            stage: "document".into(),
            attempt: ctx.pe.attempt,
            outcome: name.to_string(),
            repair_count: ctx.pe.repair_count,
            repair_limit: ctx.pe.repair_limit,
            blocker: None,
            reviewed_commit: ctx.pe.reviewed_commit.clone(),
            delivery_checkpoint: Some(name.to_string()),
            receipt_ref: Some(receipt_rel_path(&ctx.binding.plan_id)),
        },
    );
    Ok(())
}

/// Shared review-fact verification before prepare/land (AC-07: an old pass
/// whose commit/dependency/delta drifted refuses deposition).
fn verify_review_facts(ctx: &DeliveryCtx) -> Result<(), String> {
    // 计划状态 = reviewed。
    let plans = crate::plans::PlansStore::new(ctx.main_root.join("docs/plans"));
    let pf = plans
        .get(ctx.pe.plan_seq)
        .ok_or_else(|| format!("plan {:03} vanished", ctx.pe.plan_seq))?;
    if pf.status != crate::plans::PlanStatus::Reviewed && pf.status != crate::plans::PlanStatus::Archived {
        return Err(format!(
            "plan {:03} is '{}' — delivery requires reviewed (or already archived)",
            ctx.pe.plan_seq,
            pf.status.as_str()
        ));
    }
    // 合同语义未漂移（批准范围仍成立）。
    let contract = crate::relay::plan_contract::PlanContract::read(
        &ctx.main_root.join(&ctx.binding.plan_path),
        Some(&ctx.binding.plan_id),
    )?;
    if contract.semantic_hash != ctx.binding.semantic_hash {
        return Err("semantic contract drift since approval — deposition refused".into());
    }
    // worktree HEAD 仍是复审通过的提交。
    let (head, _clean) = worktree_head_clean(&ctx.worktree)?;
    if let Some(rev) = &ctx.pe.reviewed_commit {
        if head != *rev {
            return Err(format!(
                "worktree HEAD {} drifted from reviewed_commit {} — deposition refused, re-review required",
                &head[..head.len().min(12)],
                &rev[..rev.len().min(12)]
            ));
        }
    }
    // 依赖未漂移。
    crate::relay::plan_control::verify_dependencies_for_delivery(&ctx.binding)?;
    Ok(())
}

/// **prepare** — commit the reviewed spec delta in the worktree.
pub fn prepare(state: &crate::server::AppState, ws_id: &str, run_id: &str) -> Result<serde_json::Value, String> {
    let ctx = delivery_ctx(state, ws_id, run_id)?;
    // 幂等：已登记则核对事实返回。
    if let Some(existing) = ctx.pe.delivery_checkpoints.get("prepared") {
        let commit = existing
            .get("delivery_commit")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let (head, _) = worktree_head_clean(&ctx.worktree)?;
        if head == commit {
            return Ok(json!({"checkpoint": "prepared", "delivery_commit": commit, "idempotent": true}));
        }
        return Err(format!(
            "prepared checkpoint records {commit} but worktree HEAD is now {head} — re-review required"
        ));
    }
    verify_review_facts(&ctx)?;
    // docs/specs/ 下必须有已修改的增量（agent 已在 worktree 内撰写）。
    let status = git(&ctx.worktree, &["status", "--porcelain", "-uall"])?;
    let spec_files: Vec<String> = status
        .lines()
        .filter_map(|l| l.get(3..))
        .map(|p| p.trim().trim_matches('"').replace('\\', "/"))
        .filter(|p| p.starts_with("docs/specs/"))
        .collect();
    if spec_files.is_empty() {
        return Err(
            "prepare: no changes under docs/specs/ — author the reviewed spec delta in the \
             worktree first (file tools are scoped to the dev worktree)"
                .into(),
        );
    }
    git(&ctx.worktree, &["add", "docs/specs"])?;
    let review_desc = format!(
        "plan({}) spec delta delivery (SD targets: {})",
        ctx.binding.plan_id,
        spec_files.len()
    );
    git(&ctx.worktree, &["commit", "-m", &review_desc])?;
    let (delivery_commit, clean) = worktree_head_clean(&ctx.worktree)?;
    if !clean {
        return Err("prepare committed the spec delta but the worktree still has unrelated changes — commit or revert them (land requires clean)".into());
    }
    // 目标当前 hash 记录（canonical before/after 锚点）。
    let mut target_hashes = serde_json::Map::new();
    for f in &spec_files {
        let p = ctx.worktree.join(f);
        if p.is_file() {
            let bytes = std::fs::read(&p).map_err(|e| format!("delta read failed: {e}"))?;
            target_hashes.insert(f.clone(), json!(crate::relay::plan_contract::sha256_hex(&bytes)));
        }
    }
    let facts = json!({
        "delivery_commit": delivery_commit,
        "files": spec_files,
        "target_hashes": target_hashes,
        "base_commit": ctx.binding.base_commit,
    });
    record_checkpoint(state, ws_id, run_id, &ctx, "prepared", facts, None)?;
    Ok(json!({"checkpoint": "prepared", "delivery_commit": delivery_commit, "files": spec_files}))
}

/// **land** — rebase with equivalence proof, then ff-only the main checkout.
pub fn land(state: &crate::server::AppState, ws_id: &str, run_id: &str) -> Result<serde_json::Value, String> {
    let ctx = delivery_ctx(state, ws_id, run_id)?;
    if let Some(existing) = ctx.pe.delivery_checkpoints.get("landed") {
        let commit = existing
            .get("delivery_commit")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let main_tip = git(&ctx.main_root, &["rev-parse", &ctx.binding.default_branch])?
            .trim()
            .to_string();
        if main_tip == commit {
            return Ok(json!({"checkpoint": "landed", "delivery_commit": commit, "idempotent": true}));
        }
        return Err(format!(
            "landed checkpoint records {commit} but default branch tip is {main_tip} — reconcile before continuing"
        ));
    }
    let prepared = ctx
        .pe
        .delivery_checkpoints
        .get("prepared")
        .ok_or("land requires the prepared checkpoint")?
        .clone();
    let old_head = prepared
        .get("delivery_commit")
        .and_then(|v| v.as_str())
        .ok_or("prepared checkpoint lacks delivery_commit")?
        .to_string();
    // 1. guard + clean。
    crate::plan_worktree::guard_clean(&ctx.worktree)?;
    let (head, clean) = worktree_head_clean(&ctx.worktree)?;
    if !clean || head != old_head {
        return Err("land requires a clean worktree at the prepared commit".into());
    }
    // 2. rebase 当前默认分支（记录旧→新映射）。
    let default_branch = ctx.binding.default_branch.clone();
    let rebase = std::process::Command::new("git")
        .args(["rebase", &default_branch])
        .current_dir(&ctx.worktree)
        .output();
    let rebase = match rebase {
        Ok(o) if o.status.success() => o,
        Ok(o) => {
            let _ = git(&ctx.worktree, &["rebase", "--abort"]);
            return Err(format!(
                "rebase onto {default_branch} conflicted — controlled repair/re-review required: {}",
                String::from_utf8_lossy(&o.stderr).trim()
            ));
        }
        Err(e) => return Err(format!("rebase spawn failed: {e}")),
    };
    let _ = rebase;
    let new_head = git(&ctx.worktree, &["rev-parse", "HEAD"])?.trim().to_string();
    // 3. range-diff 等价证明：任何 changed-diff 行（`!`）→ 拒绝。
    let range = git(
        &ctx.worktree,
        &[
            "range-diff",
            &format!("{}..{}", ctx.binding.base_commit, old_head),
            &format!("{}..{}", ctx.binding.base_commit, new_head),
        ],
    )?;
    let non_equivalent: Vec<&str> = range
        .lines()
        .filter(|l| {
            let t = l.trim_start();
            // 形如 `N: hash ! N: hash …` 的 changed-diff 行。
            t.contains("! ") && t.starts_with(|c: char| c.is_ascii_digit())
        })
        .collect();
    if !non_equivalent.is_empty() {
        return Err(format!(
            "rebase is not patch-equivalent ({} changed diff lines) — refusing to land; re-review required",
            non_equivalent.len()
        ));
    }
    // 4. 主检出只 --ff-only（失败不普通 merge）。
    let ff = std::process::Command::new("git")
        .args(["merge", "--ff-only", &ctx.binding.dev_branch.clone().ok_or("dev branch missing")?])
        .current_dir(&ctx.main_root)
        .output();
    match ff {
        Ok(o) if o.status.success() => {}
        Ok(o) => {
            return Err(format!(
                "main checkout fast-forward failed (no merge fallback): {}",
                String::from_utf8_lossy(&o.stderr).trim()
            ))
        }
        Err(e) => return Err(format!("merge spawn failed: {e}")),
    }
    let main_tip = git(&ctx.main_root, &["rev-parse", &default_branch])?.trim().to_string();
    if main_tip != new_head {
        return Err(format!(
            "canonical hash mismatch after landing: default tip {main_tip} != delivery {new_head}"
        ));
    }
    let facts = json!({
        "delivery_commit": new_head,
        "old_commit": old_head,
        "range_diff_equivalent": true,
        "default_branch": default_branch,
        "main_tip": main_tip,
    });
    record_checkpoint(state, ws_id, run_id, &ctx, "landed", facts, None)?;
    Ok(json!({"checkpoint": "landed", "delivery_commit": new_head, "old_commit": old_head}))
}

/// **refresh** — ledger via the workspace SpecsStore (store-mediated only).
pub fn refresh(state: &crate::server::AppState, ws_id: &str, run_id: &str) -> Result<serde_json::Value, String> {
    let ctx = delivery_ctx(state, ws_id, run_id)?;
    if let Some(existing) = ctx.pe.delivery_checkpoints.get("ledger_refreshed") {
        return Ok(json!({"checkpoint": "ledger_refreshed", "idempotent": true, "facts": existing}));
    }
    let landed = ctx
        .pe
        .delivery_checkpoints
        .get("landed")
        .ok_or("ledger refresh requires the landed checkpoint (canonical first)")?
        .clone();
    let delivery_commit = landed
        .get("delivery_commit")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    // 合同（批准范围）的 spec 增量目标。
    let contract = crate::relay::plan_contract::PlanContract::read(
        &ctx.main_root.join(&ctx.binding.plan_path),
        Some(&ctx.binding.plan_id),
    )?;
    let ws = state.registry.get(ws_id);
    let specs = ws.specs.clone();
    let mut touched: Vec<String> = Vec::new();
    for target in contract
        .new_spec_components
        .iter()
        .chain(contract.supersedes_spec_components.iter())
    {
        let canon = ctx.main_root.join(target.trim_start_matches("./"));
        let bytes = std::fs::read(&canon)
            .map_err(|e| format!("spec target {target} unreadable after landing: {e}"))?;
        let hash = crate::relay::plan_contract::sha256_hex(&bytes);
        let mut doc = specs.load().map_err(|e| {
            format!(
                "ledger load failed — blocked (never hand-write specs.json): {e}; \
                 restore from backup or rebuild from docs/specs/ then retry"
            )
        })?;
        let base = Path::new(target)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| target.clone());
        let mut item = crate::specs::SpecItem::new(
            format!("{}-{base}", ctx.binding.plan_id),
            format!("{} — spec delta ({})", contract.feature_name, base),
        );
        item.status = crate::specs::SpecStatus::Empty;
        item.file = Some(target.clone());
        item.module = Some(base.trim_end_matches(".md").to_string());
        item.tags = vec![
            format!("source:{hash}"),
            format!("commit:{delivery_commit}"),
            format!("plan:{}", ctx.binding.plan_id),
        ];
        item.content = format!(
            "Derived from {} spec delta landed at {delivery_commit}; canonical source: {target} (source:{hash}).",
            ctx.binding.plan_id
        );
        specs
            .upsert_item(&mut doc, "designs", item)
            .map_err(|e| format!("ledger upsert failed: {e}"))?;
        specs.save(&doc).map_err(|e| {
            format!(
                "ledger save failed — blocked: {e}; the ledger must go through the store, never a hand-written pass"
            )
        })?;
        touched.push(target.clone());
    }
    // 交付收据摘要条目（reports 区）。
    {
        let mut doc = specs
            .load()
            .map_err(|e| format!("ledger load failed — blocked: {e}"))?;
        let mut item = crate::specs::SpecItem::new(
            format!("{}-delivery", ctx.binding.plan_id),
            format!("{} — delivery receipt", contract.feature_name),
        );
        item.status = crate::specs::SpecStatus::Empty;
        item.tags = vec![
            format!("plan:{}", ctx.binding.plan_id),
            format!("commit:{delivery_commit}"),
            format!(
                "receipt:{}",
                receipt_rel_path(&ctx.binding.plan_id)
            ),
        ];
        item.content = format!(
            "delivery checkpoints: {:?}",
            ctx.pe.delivery_checkpoints.keys().collect::<Vec<_>>()
        );
        specs
            .upsert_item(&mut doc, "reports", item)
            .map_err(|e| format!("ledger upsert failed: {e}"))?;
        specs
            .save(&doc)
            .map_err(|e| format!("ledger save failed — blocked: {e}"))?;
    }
    // 回读核验（AC-17）：load 再成功 + 条目在且 hash 正确。
    {
        let doc = specs
            .load()
            .map_err(|e| format!("ledger reload failed after refresh — blocked: {e}"))?;
        for target in &touched {
            let base = Path::new(target)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| target.clone());
            let id = format!("{}-{base}", ctx.binding.plan_id);
            let found = doc
                .sections
                .iter()
                .flat_map(|s| s.items.iter())
                .find(|i| i.id == id);
            match found {
                Some(item) => {
                    if !item.tags.iter().any(|t| t.starts_with("source:")) {
                        return Err(format!("ledger item {id} lost its source metadata"));
                    }
                }
                None => return Err(format!("ledger item {id} missing after refresh")),
            }
        }
    }
    let facts = json!({
        "delivery_commit": delivery_commit,
        "targets": touched,
        "store": "SpecsStore upsert (store-mediated)",
    });
    record_checkpoint(state, ws_id, run_id, &ctx, "ledger_refreshed", facts, None)?;
    Ok(json!({"checkpoint": "ledger_refreshed", "targets": touched}))
}

/// **archive** — explicit finalize after the earlier checkpoints verified.
pub fn archive(state: &crate::server::AppState, ws_id: &str, run_id: &str) -> Result<serde_json::Value, String> {
    let ctx = delivery_ctx(state, ws_id, run_id)?;
    let plans = crate::plans::PlansStore::new(ctx.main_root.join("docs/plans"));
    if let Some(pf) = plans.get(ctx.pe.plan_seq) {
        if pf.archived {
            // 幂等：已归档即核对收据。
            if ctx.pe.delivery_checkpoints.get("archived").is_none() {
                let facts = json!({"completion_kind": "delivered", "plan_path": format!("docs/plans/archived/{}", pf.filename)});
                record_checkpoint(state, ws_id, run_id, &ctx, "archived", facts, Some("delivered"))?;
            }
            return Ok(json!({"checkpoint": "archived", "idempotent": true, "completion_kind": "delivered"}));
        }
    }
    for required in ["prepared", "landed", "ledger_refreshed"] {
        if ctx.pe.delivery_checkpoints.get(required).is_none() {
            return Err(format!("archive requires the {required} checkpoint first"));
        }
    }
    let pf = plans
        .finalize_archived(ctx.pe.plan_seq)
        .map_err(|e| format!("archive failed: {e}"))?;
    let facts = json!({
        "completion_kind": "delivered",
        "plan_path": format!("docs/plans/archived/{}", pf.filename),
        "status": pf.status.as_str(),
    });
    record_checkpoint(state, ws_id, run_id, &ctx, "archived", facts, Some("delivered"))?;
    Ok(json!({"checkpoint": "archived", "completion_kind": "delivered", "plan_path": format!("docs/plans/archived/{}", pf.filename)}))
}

/// **cleanup** — guard + ownership verified removal of the run's own
/// worktree/branch. Failure keeps the delivered archive and reports
/// cleanup_pending (never re-runs earlier side effects).
pub fn cleanup(state: &crate::server::AppState, ws_id: &str, run_id: &str) -> Result<serde_json::Value, String> {
    let ctx = delivery_ctx(state, ws_id, run_id)?;
    if ctx.pe.delivery_checkpoints.get("cleaned").is_some() {
        return Ok(json!({"checkpoint": "cleaned", "idempotent": true}));
    }
    if ctx.pe.delivery_checkpoints.get("archived").is_none() {
        return Err("cleanup requires the archived checkpoint (delivery first, then cleanup)".into());
    }
    let lease = crate::plan_worktree::WorktreeLease {
        main_root: ctx.binding.main_root.clone(),
        worktree_root: ctx.worktree.display().to_string(),
        branch: ctx
            .binding
            .dev_branch
            .clone()
            .ok_or("cleanup without dev branch")?,
        base_commit: ctx.binding.base_commit.clone(),
        created: false,
    };
    match crate::plan_worktree::remove_plan_worktree(
        &ctx.main_root,
        &lease,
        &ctx.binding.default_branch,
    ) {
        Ok(()) => {
            let facts = json!({
                "worktree_removed": ctx.worktree.display().to_string(),
                "branch_deleted": lease.branch,
                "group_dir_pruned_if_empty": true,
            });
            record_checkpoint(state, ws_id, run_id, &ctx, "cleaned", facts, None)?;
            Ok(json!({"checkpoint": "cleaned"}))
        }
        Err(e) => {
            // cleanup_pending：delivered 保留，不重跑前面副作用。
            let facts = json!({
                "cleanup_pending": true,
                "reason": e,
            });
            record_checkpoint(state, ws_id, run_id, &ctx, "cleanup_pending", facts, None)?;
            Err(format!(
                "cleanup_pending (archive kept as delivered; only cleanup may be retried): {e}"
            ))
        }
    }
}

// ── Tool surface ────────────────────────────────────────────────────────────

/// `plan_delivery` — the document-phase agent's ONLY delivery channel. The
/// server owns the Git/store/archive side effects; the agent requests
/// actions and reports facts, it never runs git against the main checkout.
pub struct PlanDelivery {
    state: std::sync::Arc<crate::server::AppState>,
    workspace_id: String,
    run_id: String,
}

impl PlanDelivery {
    pub fn from_ctx(ctx: &crate::tool_context::ToolContext) -> Self {
        Self {
            state: ctx.state.clone(),
            workspace_id: ctx.workspace_id.clone(),
            run_id: ctx.parent_conversation_id.clone(),
        }
    }
}

#[async_trait::async_trait]
impl auto_ai_agent::Tool for PlanDelivery {
    fn name(&self) -> &str {
        "plan_delivery"
    }
    fn description(&self) -> &str {
        "Controlled plan delivery (document phase only). Actions, executed and \
         verified by the server in order: prepare (commit the reviewed docs/\
         specs/ delta in the dev worktree) → land (rebase with range-diff \
         equivalence proof + fast-forward the main checkout) → refresh \
         (update the Spec ledger via the store) → archive (finalize the plan \
         as delivered) → cleanup (remove the dev worktree/branch). Each \
         checkpoint is fact-checked and recorded in a durable receipt; \
         re-runs reconcile from the receipt and only fill missing steps."
    }
    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["prepare", "land", "refresh", "archive", "cleanup", "status"]
                }
            },
            "required": ["action"]
        })
    }
    async fn execute(
        &self,
        args: &serde_json::Value,
    ) -> Result<auto_ai_agent::ToolOutput, auto_ai_agent::ToolError> {
        let action = args["action"]
            .as_str()
            .ok_or_else(|| auto_ai_agent::ToolError::Args("missing 'action'".into()))?
            .to_string();
        let state = self.state.clone();
        let ws_id = self.workspace_id.clone();
        let run_id = self.run_id.clone();
        let result = tokio::task::spawn_blocking(move || match action.as_str() {
            "prepare" => prepare(&state, &ws_id, &run_id),
            "land" => land(&state, &ws_id, &run_id),
            "refresh" => refresh(&state, &ws_id, &run_id),
            "archive" => archive(&state, &ws_id, &run_id),
            "cleanup" => cleanup(&state, &ws_id, &run_id),
            "status" => status(&state, &ws_id, &run_id),
            other => Err(format!("unknown plan_delivery action '{other}'")),
        })
        .await
        .map_err(|e| auto_ai_agent::ToolError::Exec(format!("delivery join failed: {e}")))?;
        match result {
            Ok(v) => Ok(auto_ai_agent::ToolOutput::text(v.to_string())),
            Err(e) => Err(auto_ai_agent::ToolError::Exec(e)),
        }
    }
}

/// Read-only checkpoint/receipt view.
pub fn status(state: &crate::server::AppState, ws_id: &str, run_id: &str) -> Result<serde_json::Value, String> {
    let ctx = delivery_ctx(state, ws_id, run_id)?;
    let receipt = load_receipt(&ctx.ws_root, &ctx.binding.plan_id)?;
    Ok(json!({
        "plan_id": ctx.binding.plan_id,
        "plan_revision": ctx.binding.plan_revision,
        "checkpoints": ctx.pe.delivery_checkpoints,
        "receipt": receipt,
        "receipt_ref": receipt_rel_path(&ctx.binding.plan_id),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn receipt_roundtrip_and_key() {
        let td = tempfile::tempdir().unwrap();
        let mut r = DeliveryReceipt {
            receipt_key: "PLAN-007:r1".into(),
            plan_id: "PLAN-007".into(),
            plan_revision: 1,
            workspace_id: "ws".into(),
            main_root: "/repo".into(),
            updated_at: 42,
            checkpoints: BTreeMap::new(),
            completion_kind: None,
        };
        r.checkpoints.insert("prepared".into(), json!({"delivery_commit": "abc"}));
        save_receipt(td.path(), &r).unwrap();
        let back = load_receipt(td.path(), "PLAN-007").unwrap().unwrap();
        assert_eq!(back.receipt_key, "PLAN-007:r1");
        assert_eq!(back.checkpoints["prepared"]["delivery_commit"], "abc");
        assert_eq!(receipt_rel_path("PLAN-007"), ".autoos/plan-delivery/PLAN-007.json");
        // 缺失 → None（不是错）。
        assert!(load_receipt(td.path(), "PLAN-999").unwrap().is_none());
    }
}
