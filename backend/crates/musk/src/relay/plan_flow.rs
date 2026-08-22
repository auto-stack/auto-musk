//! Phase task templates for the plan flow (PLAN-030 T8).
//!
//! `FlowStep` has no per-step prompt field (the orchestration types stay
//! generic), so the musk driver injects phase-specific instructions here:
//! `RunStore::step_context` prefers a phase template over the raw initial
//! task for runs of the `plan` flow. The `{plan_file}` placeholder is
//! substituted from the run context — the driver extracts the `PLAN_FILE:`
//! marker the plan phase emits ([`extract_plan_file`]) and stashes it via
//! `RunStore::set_context_var`.
//!
//! The four templates internalize the `/auto-plan:*` skill disciplines
//! (008 §6): new (clarify-or-draft, numbered sections, atomic tasks),
//! work (plan as sole context, tick + verify, blockers to 待澄清事项),
//! review (trust the code, re-verify acceptance, fill spec-impact), merge
//! (gate on reviewed, deposit, archive).

use std::collections::HashMap;

/// Extract the `PLAN_FILE: <path>` marker from a step's accumulated output.
/// The plan phase must emit it as the last line; later phases' templates
/// consume the stashed path.
pub fn extract_plan_file(output: &str) -> Option<String> {
    let re = regex::Regex::new(r"(?m)^PLAN_FILE:\s*(\S+)\s*$").ok()?;
    re.captures(output)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
}

/// Compose the phase task for (flow_id, step_id). Returns None for flows
/// without templates (legacy behavior: raw initial task).
///
/// `initial_task` (the user's requirement) is embedded in every phase so the
/// agent always knows what it is working on. `{plan_file}` is substituted
/// from `context`; a missing value degrades to a locate-it-yourself hint
/// instead of a dangling placeholder.
pub fn phase_task(
    flow_id: &str,
    step_id: &str,
    initial_task: &str,
    context: &HashMap<String, String>,
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
    let plan_file = context
        .get("plan_file")
        .cloned()
        .unwrap_or_else(|| "(未知——用 list_plans 找到本需求对应的 plan 再继续)".into());
    let template = match step_id {
        "plan" => format!(
            "{requirement}# 任务：需求整理与计划撰写（plan 相位）\n\n\
你是本需求的负责人（plan-dev），全程以计划文件为唯一事实源。请产出一份可直接执行的实施计划：\n\n\
1. 先用 `list_plans` 检查是否已有对应此需求的 plan（按 feature 与 status 判断）——幂等续跑：已存在则复用它（输出其路径即可），**不要新建重复计划**。\n\
2. 若需求模糊、缺关键约束：列出澄清问题（编号、一次问全），然后**停止**，不要开始写计划。用户会在审批门用「拒绝 + 反馈」回答你，届时重跑本相位。\n\
3. 需求清晰则用 `create_plan` 写完整计划，正文章节**必须带编号**（merge 引擎按编号映射沉淀）：\n\
   `## 0. 变更摘要` / `## 1. 目标` / `## 2. 架构方案` / `## 3. 技术栈` / `## 4. 需求分析与背景调查` / `## 5. 详细设计（含关键代码示例）` / `## 6. 测试设计` / `## 7. 验收标准（checkbox，逐条可独立验证）` / `## 8. 执行步骤（原子任务：精确文件路径 + 操作 + 验证命令；禁止 TBD/TODO）` / `## 9. 复审记录（留空）` / `## 10. 待澄清事项（留空）`\n\
4. 写完自审一遍：章节齐全、任务原子、验证命令真实可跑、frontmatter 完整——**必须含 `current_step: 0` 与 `total_steps`（= §8 任务数）**。\n\
5. 最终输出必须以单独一行结尾（驱动器解析它路由后续相位）：\n\
   `PLAN_FILE: docs/plans/NNN-slug.md`\n"
        ),
        "execute" => format!(
            "{requirement}# 任务：按计划实施（execute 相位）\n\n\
执行计划文件：{plan_file}\n\n\
1. `read_plan` 载入上述计划——它是你唯一的工作上下文。\n\
2. `transition_plan` 到 `executing`。\n\
3. 逐项执行 `## 8. 执行步骤`：严格按任务描述操作（精确文件路径 + 动作）；每完成一项就跑它的验证命令，通过后用 `update_plan` 勾选（`[✅ 已完成]` + 一行证据）并推进 frontmatter 的 `current_step`。\n\
4. TDD：涉及代码与测试的任务，先写失败测试、确认失败，再实现到通过。\n\
5. 受阻或歧义：只把问题追加进 `## 10. 待澄清事项`，继续做下一个不受阻的任务；**不要脱离计划即兴调研或改设计**。\n\
6. 全部任务完成后，完整跑一遍 `## 7. 验收标准` 的验证；然后 `transition_plan` 到 `execution_done`，汇报逐项证据。\n"
        ),
        "review" => format!(
            "{requirement}# 任务：复审（review 相位）\n\n\
复审计划文件：{plan_file}\n\n\
Trust the code, not the checkboxes：\n\n\
1. `read_plan` 载入计划，逐条重验 `## 7. 验收标准`——对照实际代码与真实命令输出（记录 pass/partial/fail + `file:line` 证据）。绿勾是主张，不是证据。\n\
2. 检查执行丢项、workaround、行为偏差——登记为债务候选。\n\
3. 用 `update_plan` 填写 `## 9. 复审记录`（复审人 / 时间 / 逐标准判定表 / 债务候选）。\n\
4. 用 `update_plan` 填 frontmatter 的 spec-impact 三字段（E2E 实测易漏，
   **硬性要求**）：`supersedes_spec_components` / `new_spec_components` /
   `touched_goals` 必须给出具体条目列表；确实无关联时保留 `[]` 并在
   `## 9.` 中写明原因。同时核对 `total_steps` 与 §8 任务数一致、
   `current_step` 反映实际进度（merge 相位会逐字消费这些字段）。\n\
5. 全部通过 → `transition_plan` 到 `reviewed`；有不通过 → `transition_plan` 回 `executing`，并在输出中列明缺口与建议（run 会正常结束，用户决定是否重开续跑修复）。\n"
        ),
        "document" => {
            // PLAN-034：plan-merge 单相位 run 只做沉淀（执行/复审均已完成）
            let preamble = if flow_id == "plan-merge" {
                String::from(
                    "# 任务：智能沉淀（plan-merge 单相位 run）\n\n\
                     目标计划：见下方需求中的 PLAN 编号（用 `read_plan` 按编号读取）。\n\
                     本 run 只做沉淀——执行与复审均已完成，不要重做。\n\n",
                )
            } else {
                String::new()
            };
            format!(
            "{preamble}{requirement}# 任务：知识沉淀（document 相位）\n\n\
沉淀计划文件：{plan_file}\n\n\
1. `read_plan` 检查 status 必须是 `reviewed`；不是则输出「复审未通过/未完成，跳过沉淀」并结束——**不要强行 merge**。\n\
2. `merge_plan` 把计划按章节映射沉淀进 Spec ledger 6 区（幂等 upsert，`P<seq>-<n>` 稳定 id）。\n\
3. 按 frontmatter 的 spec-impact 三字段，用文件工具更新 `docs/specs/` 模块树 markdown：改了哪个模块就更新哪个模块文档，新增模块建档，移除的模块标注。\n\
4. 汇报 `sections_touched` / `items_created` 与 `docs/specs/` 树的具体改动。\n\
5. `emit_report` 生成本 Run 汇报报告（format=html）：自包含单文件、内联 CSS、\
**无任何 `<script>`/iframe/外链资源**；分节：封面（标题+日期+run 概要）/\
需求与方案 / 各阶段成果（plan/execute/review/document）/ 指标（步骤·工具\
调用·令牌·时长）/ 交付物清单 / 结尾；视觉基调类 PPT 分节卡片（大标题、\
留白、16:9 心智）。同时给同结构 markdown 源。\n"
            )
        }
        _ => return None,
    };
    Some(template)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(plan_file: Option<&str>) -> HashMap<String, String> {
        let mut m = HashMap::new();
        if let Some(p) = plan_file {
            m.insert("plan_file".to_string(), p.to_string());
        }
        m
    }

    #[test]
    fn phase_task_covers_all_four_plan_steps() {
        for step in ["plan", "execute", "review", "document"] {
            let t = phase_task("plan", step, "做一个功能", &ctx(Some("docs/plans/031-x.md")))
                .unwrap_or_else(|| panic!("step {step} must have a template"));
            assert!(t.contains("做一个功能"), "{step}: requirement embedded");
        }
    }

    #[test]
    fn phase_task_none_for_other_flows_and_steps() {
        assert!(phase_task("default", "advise", "t", &ctx(None)).is_none());
        assert!(phase_task("plan", "unknown-step", "t", &ctx(None)).is_none());
    }

    /// PLAN-034：plan-merge 只有 document 模板，且带智能沉淀前言。
    #[test]
    fn plan_merge_flow_document_template_has_smart_deposit_preamble() {
        let t = phase_task("plan-merge", "document", "沉淀 PLAN-007", &ctx(None)).unwrap();
        assert!(t.contains("PLAN-007"), "requirement (plan id) embedded");
        assert!(t.contains("智能沉淀"));
        assert!(t.contains("read_plan"));
        assert!(t.contains("merge_plan"));
        assert!(t.contains("emit_report"));
        assert!(t.contains("不要重做"));
        // 其余步骤无模板（流程只有 document 一步）
        assert!(phase_task("plan-merge", "execute", "t", &ctx(None)).is_none());
        assert!(phase_task("plan-merge", "plan", "t", &ctx(None)).is_none());
        // plan 流程的 document 模板不带前言（行为不变）
        let t2 = phase_task("plan", "document", "需求", &ctx(None)).unwrap();
        assert!(!t2.contains("智能沉淀"));
    }

    #[test]
    fn plan_template_carries_plan_file_protocol() {
        let t = phase_task("plan", "plan", "需求", &ctx(None)).unwrap();
        assert!(t.contains("PLAN_FILE: docs/plans/NNN-slug.md"));
        // 澄清-停止纪律 + 幂等复用
        assert!(t.contains("不要新建重复计划"));
        assert!(t.contains("停止"));
    }

    #[test]
    fn later_phases_substitute_plan_file_or_degrade() {
        let t = phase_task("plan", "execute", "需求", &ctx(Some("docs/plans/030-x.md"))).unwrap();
        assert!(t.contains("docs/plans/030-x.md"));
        assert!(!t.contains("{plan_file}"), "no dangling placeholder");

        let t = phase_task("plan", "review", "需求", &ctx(None)).unwrap();
        assert!(t.contains("list_plans"), "missing plan_file degrades to locate hint");
    }

    #[test]
    fn templates_reference_status_machine_actions() {
        let ex = phase_task("plan", "execute", "t", &ctx(None)).unwrap();
        assert!(ex.contains("`executing`") && ex.contains("`execution_done`"));
        let rv = phase_task("plan", "review", "t", &ctx(None)).unwrap();
        assert!(rv.contains("`reviewed`") && rv.contains("spec-impact"));
        let dc = phase_task("plan", "document", "t", &ctx(None)).unwrap();
        assert!(dc.contains("`reviewed`") && dc.contains("`merge_plan`"));
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
}
