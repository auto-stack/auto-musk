//! Phase task templates for the plan flow (PLAN-030 T8; PLAN-086 fixed
//! professions; PLAN-096 T-02 contract rework).
//!
//! PLAN-096 SD-01: the templates no longer inline the four-phase discipline.
//! The `/auto-plan:{new,work,review,merge}` skills are the single discipline
//! source — their full content is frozen at run start
//! ([`crate::relay::plan_contract::snapshot_plan_skills`]) and injected
//! verbatim into the phase task as a mechanical input, so the model never
//! has to "remember to load" its discipline. What remains in the template:
//! the fixed profession, the mechanical inputs (requirement, plan file,
//! binding facts), the stage-result protocol
//! (`complete_plan_stage` — Done without a valid result is
//! `stage_incomplete`), and — for the execute phase — the Canvas generation
//! guidance (previously `mode.name == "coding"`-only, §5.1).
//!
//! `FlowStep` has no per-step prompt field (the orchestration types stay
//! generic), so the musk driver injects phase-specific instructions here:
//! `RunStore::step_context` prefers a phase template over the raw initial
//! task for runs of the `plan` flow. The `{plan_file}` placeholder is
//! substituted from the run context — fed by the create_plan binding channel
//! first, the `PLAN_FILE:` marker extraction as fallback
//! ([`plan_file_marker_write`]).

use std::collections::{BTreeMap, HashMap};

use super::plan_contract::{SkillEntry, SkillSnapshot};

/// Extract the `PLAN_FILE: <path>` marker from a step's accumulated output.
/// The plan phase must emit it as the last line; later phases' templates
/// consume the stashed path.
pub fn extract_plan_file(output: &str) -> Option<String> {
    let re = regex::Regex::new(r"(?m)^PLAN_FILE:\s*(\S+)\s*$").ok()?;
    re.captures(output)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
}

/// PLAN-086 T-03: gate the marker-fallback write — the plan_file context var
/// is the binding channel's territory (create_plan writes it tool-time, zero
/// AI involvement). When a binding already exists the `PLAN_FILE:` marker
/// must NOT overwrite it (binding > marker > hint); with no binding the
/// marker extraction runs as the fallback. Returns the value to write
/// (None = don't touch the var). Both drivers (hw run_step + ag
/// drive_submit_handoff) route through this so the order is single-sourced.
pub fn plan_file_marker_write(existing: Option<String>, output: &str) -> Option<String> {
    if existing.is_some() {
        return None;
    }
    extract_plan_file(output)
}

// ── PLAN-094 T-03: execute-gate plan-file invariant (UAT K4/K5) ────────────

/// Terminal run error when an execute-gate approve finds no plan file (plan
/// D1: loud failure instead of a gate pass — the engine's redraft reruns the
/// GATED step (execute), so a true plan-phase rerun is not expressible; the
/// recovery path is restarting the run, which the advisor handles idempotently).
pub const PLAN_GATE_FAIL_ERROR: &str =
    "plan phase ended without a plan file — restart the run (advisor reuses existing plans) or answer the advisor via reject+feedback in human mode";

/// Default audit note carried on every auto-approved human gate (K4: the
/// audit turn must say what was injected on the user's behalf).
pub const AUTO_APPROVE_NOTE: &str = "auto-approved; recorded defaults apply";

/// What the caller must do with an approve at the plan flow's execute gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecuteGateAction {
    /// Plan file present — approve proceeds unchanged (AC-04 regression arm).
    Proceed,
    /// Plan file missing — do NOT pass the gate: auto mode fails the run
    /// (plan D1), the human arm refuses the decision without consuming it.
    Fail,
}

/// The execute-gate invariant (UAT K5): an approve at the `plan` flow's
/// execute gate requires a materialized plan file — run context `plan_file`
/// via the create_plan binding or the `PLAN_FILE:` marker fallback. Returns
/// None when the approve is not at that gate (other flows/steps untouched).
/// `flow_id`/`gate_step_id` come from `RunStore::flow_of` /
/// `RunStore::pending_gate_step` (drivers may pass the WaitForHuman step_id —
/// same value). Single-sourced here so the hw and ag drivers rule identically.
pub fn execute_gate_action(
    flow_id: Option<&str>,
    gate_step_id: Option<&str>,
    plan_file: Option<&str>,
) -> Option<ExecuteGateAction> {
    if flow_id != Some("plan") || gate_step_id != Some("execute") {
        return None;
    }
    if plan_file.map(|s| !s.trim().is_empty()).unwrap_or(false) {
        return Some(ExecuteGateAction::Proceed);
    }
    Some(ExecuteGateAction::Fail)
}

/// The stage→skill mapping (§5.1): plan→auto-plan-new, execute→auto-plan-work,
/// review→auto-plan-review, document→auto-plan-merge.
pub fn stage_skill(step_id: &str) -> &'static str {
    match step_id {
        "plan" => "auto-plan-new",
        "execute" => "auto-plan-work",
        "review" => "auto-plan-review",
        "document" => "auto-plan-merge",
        _ => "",
    }
}

/// The result-protocol block every plan-flow phase carries: Done without a
/// valid `complete_plan_stage` submission is `stage_incomplete` (AC-04).
fn result_protocol(step_id: &str) -> String {
    let stage_facts = match step_id {
        "plan" => "stage=\"plan\"、outcome=\"pass\"（或受阻时 needs_replan/blocked）、\
plan_id/plan_revision 取自创建的计划 frontmatter",
        "execute" => "stage=\"execute\"、outcome=\"pass\"（复审发现问题时由 review 相位发 \
needs_fix；本相位不自行回退）、commit=开发 worktree 内已提交的完整 hash、\
acceptance_results=逐条 AC 的 pass/fail+证据、evidence=验证命令与输出路径",
        "review" => "stage=\"review\"、outcome∈pass|needs_fix|needs_replan|blocked；\
needs_fix 必须带稳定 finding id 与对应 task/AC id；pass 必须带逐 AC 判定与证据",
        "document" => "stage=\"document\"、outcome=\"pass\"（交付检查点由 plan_delivery \
工具逐项核验后登记，不要自报 delivered）",
        _ => "",
    };
    format!(
        "# 结果提交协议（机械要求）\n\n\
本相位完成时必须调用 `complete_plan_stage` 工具提交结构化结果：{stage_facts}。\n\
只输出 Done/总结而不提交有效结果 → 本相位按 stage_incomplete 处理，不进入下一相位（AC-04）。\n\
工具调用被服务器校验（回读计划文件/Git/证据工件）；伪 pass、缺提交、缺证据都会被拒绝。\n\n"
    )
}

/// Missing-discipline blocker clause (skill snapshot absent at template
/// compose time — upstream defect, same treatment as a missing plan file).
fn skill_blocker_clause(step_id: &str) -> String {
    format!(
        "# 阻断性缺陷\n\n\
技能纪律源（{skill}）未随本 run 注入。这是阻断性缺陷：立即停止并输出 \
blocker 说明技能快照缺失；不得开始或继续本相位工作、不得正常完成本相位。\n\n",
        skill = stage_skill(step_id)
    )
}

fn skill_block(step: &str, skills: &BTreeMap<String, SkillEntry>) -> String {
    match skills.get(stage_skill(step)) {
        Some(entry) => format!(
            "# 纪律源（唯一，机械注入；版本 sha256:{}）\n\n\
以下为 `{}` 技能全文——它是本相位的唯一纪律来源，逐步遵守；与本模板冲突时以技能为准：\n\n\
<header-skills>\n{}\n</header-skills>\n\n",
            entry.sha256,
            entry.name,
            entry.content,
        ),
        None => skill_blocker_clause(step),
    }
}

/// Compose the phase task for (flow_id, step_id). Returns None for flows
/// without templates (legacy behavior: raw initial task).
///
/// `initial_task` (the user's requirement) is embedded in every phase so the
/// agent always knows what it is working on. `{plan_file}` is substituted
/// from `context`; a missing value is a hard-fail blocker clause (PLAN-094
/// T-04). `skills` is the run's frozen skill snapshot (PLAN-096 §5.1); when
/// the phase's skill is absent the template degrades to a blocker clause —
/// the phase must not run without its discipline source.
pub fn phase_task(
    flow_id: &str,
    step_id: &str,
    initial_task: &str,
    context: &HashMap<String, String>,
    skills: &SkillSnapshot,
) -> Option<String> {
    // plan-merge（PLAN-034）只有 document 模板；plan 四相位全有；其它流程无。
    if flow_id == "plan-merge" {
        if step_id != "document" {
            return None;
        }
    } else if flow_id != "plan" {
        return None;
    }
    let requirement = format!("# 需求（用户原话整理）\n{initial_task}\n\n");
    // PLAN-094 T-04 (UAT K5/K7)：双驱动门前置不变式的第二道保险——计划文件
    // 缺失不再是"自行定位"的降级提示（UAT T3 实录：coder 借它空转假完成），
    // 而是硬失败条款：立即停止并输出 blocker。
    let plan_file = context.get("plan_file").cloned().unwrap_or_else(|| {
        "(缺失——上游缺陷：计划文件未落实。这是阻断性缺陷：立即停止并输出 \
         blocker 说明 plan_file 未落实；不得开始或继续本相位工作、不得正常\
         完成本相位。)"
            .into()
    });
    let mech_plan_file = format!("# 机械输入\n\n- 计划文件：{plan_file}\n");
    let template = match step_id {
        "plan" => format!(
            "{requirement}# 任务：需求整理与计划撰写（plan 相位）\n\n\
你是本需求的规划师（advisor，固定四职业之首）。技能快照与机械输入如下；\
完成后按结果提交协议调用 `complete_plan_stage`。\n\n\
{}\n\
{mech_plan_file}- 计划写入：用 `create_plan` 工具落主检出 `docs/plans/`\
（工具返回的路径即绑定通道，零 AI 参与；已存在对应计划时幂等复用，不要新建重复计划）。\n\
{}\n\
最终输出以单独一行结尾（驱动器解析）：`PLAN_FILE: docs/plans/NNN-slug.md`\n",
            skill_block(step_id, &skills.skills),
            result_protocol(step_id),
        ),
        "execute" => {
            // PLAN-096 §5.1：Canvas 生成指引补齐——原 `mode.name=="coding"`
            // 专享条件使 relay coder 有工具无纪律；现随相位模板机械注入。
            let canvas = crate::canvas::templates::generation_prompt();
            format!(
                "{requirement}# 任务：按计划实施（execute 相位）\n\n\
你是本需求的实现工程师（coder，第二相位职业）。技能快照与机械输入如下；\
完成后按结果提交协议调用 `complete_plan_stage`。\n\n\
{}\n\
{mech_plan_file}- 工作根：本相位的文件/命令工具已限定在本计划的开发 \
worktree（主检出的计划共享状态仍经 plan 工具读写，不要直接改主检出代码）。\n\
{}\n\
# Canvas 生成指引（目标工程为 Auto 应用时适用）\n\n{canvas}\n",
                skill_block(step_id, &skills.skills),
                result_protocol(step_id),
            )
        }
        "review" => {
            // PLAN-096 T-07（§5.6）：工件绑定事实（批准版本/commit/依赖冻结）
            // 作为机械输入注入——reviewer 的凭据核验面向绑定事实与真实工件，
            // 不接收 coder 的完成自述。
            let binding_facts = context
                .get("binding_facts")
                .map(|f| format!("- 批准绑定事实：{f}\n"))
                .unwrap_or_default();
            format!(
            "{requirement}# 任务：复审（review 相位）\n\n\
你是本需求的复审人（reviewer，第三相位职业——独立于起草与执行的凭据核验者，\
不接收执行相位的 history 或完成自述）。技能快照与机械输入如下；\
完成后按结果提交协议调用 `complete_plan_stage`。\n\n\
{}\n\
{mech_plan_file}{binding_facts}{}\n",
            skill_block(step_id, &skills.skills),
            result_protocol(step_id),
        )
        }
        "document" => {
            // PLAN-034：plan-merge 单相位 run 只做沉淀（执行/复审均已完成）
            let preamble = if flow_id == "plan-merge" {
                String::from(
                    "# 任务：受控交付（plan-merge 单相位 run）\n\n\
                     你是本计划的知识管理员（assistant）。目标计划见下方机械输入；\
                     本 run 只做交付——执行与复审均已完成，不要重做。\n\n",
                )
            } else {
                String::new()
            };
            format!(
            "{preamble}{requirement}# 任务：受控交付（document 相位）\n\n\
你是本计划的知识管理员（assistant，末相位职业）。技能快照与机械输入如下。\n\n\
{}\n\
{mech_plan_file}- 交付操作：用 `plan_delivery` 工具按序执行 \
prepare → land → refresh → archive → cleanup；每个检查点由服务器核验并落收据，\
不要自报 delivered、不要手拼账本 JSON、不要直接在主检出跑 Git 写操作。\n\
{}\n\
汇报 `sections_touched` / `items_created` 与 `docs/specs/` 树的具体改动；\
`emit_report` 可选（frontmatter 主信息走机械渲染，不要自编步骤/令牌/时长数字）。\n",
            skill_block(step_id, &skills.skills),
            result_protocol(step_id),
        )
        }
        _ => return None,
    };
    Some(template)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap_with(names: &[&str]) -> SkillSnapshot {
        let mut s = SkillSnapshot::default();
        for n in names {
            s.skills.insert(
                n.to_string(),
                SkillEntry {
                    name: n.to_string(),
                    path: format!("/{n}/SKILL.md"),
                    sha256: format!("hash-of-{n}"),
                    content: format!("# {n} discipline\n\n按技能步骤执行。"),
                },
            );
        }
        s
    }

    fn full_snap() -> SkillSnapshot {
        snap_with(&[
            "auto-plan-new",
            "auto-plan-work",
            "auto-plan-review",
            "auto-plan-merge",
        ])
    }

    fn ctx(plan_file: Option<&str>) -> HashMap<String, String> {
        let mut m = HashMap::new();
        if let Some(p) = plan_file {
            m.insert("plan_file".to_string(), p.to_string());
        }
        m
    }

    fn task(flow: &str, step: &str, ctx: &HashMap<String, String>) -> String {
        phase_task(flow, step, "做一个功能", ctx, &full_snap()).unwrap_or_else(|| {
            panic!("step {step} must have a template")
        })
    }

    /// 四相位模板齐全且机械嵌入需求原文。
    #[test]
    fn phase_task_covers_all_four_plan_steps() {
        for step in ["plan", "execute", "review", "document"] {
            let t = task("plan", step, &ctx(Some("docs/plans/031-x.md")));
            assert!(t.contains("做一个功能"), "{step}: requirement embedded");
        }
    }

    /// PLAN-086 T-04：每相位模板对准其固定职业称呼（advisor/coder/reviewer/
    /// assistant），plan-merge 沉淀前言同样点名 assistant。
    #[test]
    fn templates_voice_the_fixed_professions() {
        let roles = [
            ("plan", "advisor"),
            ("execute", "coder"),
            ("review", "reviewer"),
            ("document", "assistant"),
        ];
        for (step, role) in roles {
            let t = task("plan", step, &ctx(None));
            assert!(t.contains(&format!("（{role}")), "{step} template names {role}");
        }
        let pm = phase_task(
            "plan-merge",
            "document",
            "沉淀 PLAN-007",
            &ctx(None),
            &full_snap(),
        )
        .unwrap();
        assert!(pm.contains("（assistant"), "plan-merge document names assistant");
        // 旧单角色称呼不得残留。
        for (step, _) in roles {
            let t = task("plan", step, &ctx(None));
            assert!(!t.contains("plan-dev"), "{step} must not mention plan-dev");
        }
    }

    #[test]
    fn phase_task_none_for_other_flows_and_steps() {
        let empty = SkillSnapshot::default();
        assert!(phase_task("default", "advise", "t", &ctx(None), &full_snap()).is_none());
        assert!(phase_task("plan", "unknown-step", "t", &ctx(None), &full_snap()).is_none());
        assert!(phase_task("plan-merge", "execute", "t", &ctx(None), &full_snap()).is_none());
        assert!(phase_task("plan-merge", "plan", "t", &ctx(None), &empty).is_none());
    }

    /// PLAN-096 SD-01：纪律取自技能快照——相位模板注入对应技能全文与版本
    /// hash；stage→skill 映射固定（new/work/review/merge）。
    #[test]
    fn templates_inject_stage_skill_content_and_hash() {
        assert_eq!(stage_skill("plan"), "auto-plan-new");
        assert_eq!(stage_skill("execute"), "auto-plan-work");
        assert_eq!(stage_skill("review"), "auto-plan-review");
        assert_eq!(stage_skill("document"), "auto-plan-merge");
        let t = task("plan", "execute", &ctx(None));
        assert!(t.contains("# auto-plan-work discipline"), "skill content injected");
        assert!(t.contains("hash-of-auto-plan-work"), "version hash surfaced");
        assert!(t.contains("唯一纪律来源"), "discipline-source statement present");
        // 其它相位拿到的是各自技能。
        let p = task("plan", "plan", &ctx(None));
        assert!(p.contains("# auto-plan-new discipline"));
        let r = task("plan", "review", &ctx(None));
        assert!(r.contains("# auto-plan-review discipline"));
        let d = task("plan", "document", &ctx(None));
        assert!(d.contains("# auto-plan-merge discipline"));
    }

    /// 缺技能快照 → 阻断性缺陷条款（不静默降级到无纪律模板）。
    #[test]
    fn missing_skill_snapshot_is_a_blocker_clause() {
        let empty = SkillSnapshot::default();
        let t = phase_task("plan", "execute", "需求", &ctx(None), &empty).unwrap();
        assert!(t.contains("阻断性缺陷"), "blocker clause present");
        assert!(t.contains("auto-plan-work"), "names the missing skill");
        assert!(t.contains("立即停止"), "stop-now instruction present");
        assert!(!t.contains("# auto-plan-work discipline"));
    }

    /// AC-04：结果提交协议注入每个相位——Done 不算完成。
    #[test]
    fn result_protocol_injected_in_every_phase() {
        for step in ["plan", "execute", "review", "document"] {
            let t = task("plan", step, &ctx(None));
            assert!(t.contains("complete_plan_stage"), "{step}: tool named");
            assert!(t.contains("stage_incomplete"), "{step}: incomplete consequence");
        }
        let plan_t = task("plan", "plan", &ctx(None));
        assert!(plan_t.contains(r#"stage="plan""#));
        let rev_t = task("plan", "review", &ctx(None));
        assert!(rev_t.contains("needs_fix"), "review protocol mentions fix routing");
    }

    /// §5.1：execute 相位注入 Canvas 生成指引（原 coding 模式专享条件补齐）。
    #[test]
    fn execute_phase_carries_canvas_guidance() {
        let t = task("plan", "execute", &ctx(None));
        assert!(t.contains("Canvas 生成指引"), "canvas guidance block present");
        assert!(!crate::canvas::templates::generation_prompt().is_empty());
        // 其它相位不注入（advisor/reviewer 无关画布）。
        let p = task("plan", "plan", &ctx(None));
        assert!(!p.contains("Canvas 生成指引"));
    }

    /// plan-merge document 模板：merge 技能 + plan_delivery 受控交付指引。
    #[test]
    fn plan_merge_document_template_carries_delivery_contract() {
        let pm = phase_task(
            "plan-merge",
            "document",
            "沉淀 PLAN-007",
            &ctx(None),
            &full_snap(),
        )
        .unwrap();
        assert!(pm.contains("PLAN-007"), "requirement (plan id) embedded");
        assert!(pm.contains("受控交付"));
        assert!(pm.contains("# auto-plan-merge discipline"));
        assert!(pm.contains("plan_delivery"));
        assert!(pm.contains("prepare"));
        assert!(pm.contains("不要自报 delivered"));
        assert!(pm.contains("complete_plan_stage"));
    }

    #[test]
    fn plan_template_carries_plan_file_protocol() {
        let t = task("plan", "plan", &ctx(Some("docs/plans/096-x.md")));
        assert!(t.contains("PLAN_FILE: docs/plans/NNN-slug.md"));
        // 幂等复用纪律保留（技能内详解，模板保留指针）。
        assert!(t.contains("幂等复用") || t.contains("不要新建重复计划"));
    }

    #[test]
    fn later_phases_substitute_plan_file_or_hard_fail() {
        let t = task("plan", "execute", &ctx(Some("docs/plans/030-x.md")));
        assert!(t.contains("docs/plans/030-x.md"));
        assert!(!t.contains("{plan_file}"), "no dangling placeholder");

        // PLAN-094 T-04 (用例 5): plan_file 缺失 → 硬失败 blocker 条款。
        for step in ["execute", "review", "document"] {
            let t = task("plan", step, &ctx(None));
            assert!(t.contains("阻断性缺陷"), "{step}: blocker clause present");
            assert!(t.contains("立即停止"), "{step}: stop now instruction");
            assert!(!t.contains("list_plans 找到"), "{step}: locate hint retired");
        }
    }

    #[test]
    fn extract_plan_file_finds_last_line_marker() {
        let out = "分析…\n创建计划…\n\nPLAN_FILE: docs/plans/031-demo.md\n";
        assert_eq!(
            extract_plan_file(out).as_deref(),
            Some("docs/plans/031-demo.md")
        );
        assert!(extract_plan_file("no marker here").is_none());
        // 行中(非行首)出现不算
        assert!(extract_plan_file("mention PLAN_FILE: x.md inline").is_none());
    }

    /// PLAN-086 T-03 / AC-03：标记回退守门——绑定已存在不覆盖；无绑定时
    /// 标记生效；无标记不动。
    #[test]
    fn marker_fallback_never_overwrites_binding() {
        // 绑定优先：已有 plan_file 时标记输出被忽略。
        assert_eq!(
            plan_file_marker_write(
                Some("docs/plans/001-bound.md".into()),
                "分析…\nPLAN_FILE: docs/plans/002-fallback.md\n"
            ),
            None
        );
        // 回退生效：无绑定时提取标记。
        assert_eq!(
            plan_file_marker_write(None, "分析…\n\nPLAN_FILE: docs/plans/002-fallback.md\n")
                .as_deref(),
            Some("docs/plans/002-fallback.md")
        );
        // 双缺：无绑定亦无标记 → 不写（组装臂落 hint）。
        assert_eq!(plan_file_marker_write(None, "no marker"), None);
    }

    /// PLAN-094 T-03 (用例 3 的判定核)：execute 门不变式两分支——有计划文件
    /// 照常放行（AC-04）；缺失即不放行（auto 置败 / human 决议报错，plan D1
    /// 退化：引擎 redraft 只会重做被门守卫的 execute 相位而非 plan 相位，
    /// 定向重跑 plan 相位不可表达 → 直接置败响亮失败）。非 execute 门 /
    /// 非 plan 流程不适用（None）。
    #[test]
    fn execute_gate_invariant_two_way() {
        use ExecuteGateAction::{Fail, Proceed};
        // plan 流 execute 门：两判定。
        assert_eq!(
            execute_gate_action(Some("plan"), Some("execute"), Some("docs/plans/001-x.md")),
            Some(Proceed)
        );
        assert_eq!(
            execute_gate_action(Some("plan"), Some("execute"), None),
            Some(Fail)
        );
        assert_eq!(
            execute_gate_action(Some("plan"), Some("execute"), Some("")),
            Some(Fail),
            "empty plan_file counts as missing"
        );
        // 其它门/流程：不变式不适用。
        assert_eq!(execute_gate_action(Some("plan"), Some("review"), None), None);
        assert_eq!(execute_gate_action(Some("plan-merge"), Some("document"), None), None);
        assert_eq!(execute_gate_action(Some("simple"), Some("execute"), None), None);
        assert_eq!(execute_gate_action(None, Some("execute"), None), None);
    }
}
