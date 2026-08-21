---
name: plan-driven-development
description: The canonical development loop for musk — one agent drives a feature through plan → execute → review → document on top of a numbered plan file (docs/plans/NNN-*.md) with the PLAN status machine. Use for any feature work too big for a single direct edit; replaces the old brainstorm→spec→relay pipeline. In relay runs the phase task already carries these rules — this skill is the contract reference for chat-mode development.
---

# Plan-Driven Development

One agent carries a feature end-to-end. The **plan file is the single source of
truth and the full handoff artifact** — phases never exchange summaries, they
share the file. Development touches Specs only at the end (delayed
materialization): review passes → deposit → archive.

## The Plan File Contract

`docs/plans/NNN-slug.md` (NNN = max active+archived seq + 1, never hand-pick):

- YAML frontmatter: `plan_id: PLAN-NNN`, `status:` one of
  `drafting → executing → execution_done → review_done → merged`
  (+ legal back-edges: review_done→executing, execution_done→executing;
  merged is terminal), `feature_name`, timestamps, `current_step`/`total_steps`.
  Leave `supersedes_spec_components` / `new_spec_components` / `touched_goals`
  empty until review fills them.
- Numbered sections (merge maps by number — unnumbered Chinese titles are
  tolerated but numbered is the contract):
  `## 0. 变更摘要` `## 1. 目标` `## 2. 架构方案` `## 3. 技术栈`
  `## 4. 需求分析与背景调查` `## 5. 详细设计（含代码示例）` `## 6. 测试设计`
  `## 7. 验收标准（checkbox）` `## 8. 执行步骤（原子任务）` `## 9. 复审记录`
  `## 10. 待澄清事项`
- Atomic tasks: precise file path + exact operation + a runnable verification
  command per task (2–5 minute granularity). Forbidden: TBD / TODO / "similar
  to task N".

## Phases

1. **plan** — clarify or draft. Vague requirement → ask, then stop. Clear →
   write the full plan via `create_plan`; check `list_plans` first and RESUME
   an existing plan for the same feature instead of duplicating. End the final
   message with one line: `PLAN_FILE: docs/plans/NNN-slug.md`.
2. **execute** — the plan is the only context. `transition_plan` → executing;
   walk `## 8` top-to-bottom; TDD when tests apply; tick tasks with
   `[✅ 已完成]` + one-line evidence via `update_plan`; blockers go to
   `## 10` only — never improvise off-plan. Finish with the acceptance suite,
   then → execution_done.
3. **review** — trust the code, not the checkboxes. Re-verify every
   `## 7` item against actual code (pass/partial/fail + file:line); fill
   `## 9` and the spec-impact frontmatter fields. Pass → review_done;
   fail → back to executing with a gap report.
4. **document** — gate on review_done; `merge_plan` deposits into the Spec
   ledger (§0→reports, §1→goals, §2→architecture, §5→designs, §6→tests,
   §7/§9→reviews; id-stable `P<seq>-<n>` items); then update the
   `docs/specs/` module tree to match what actually changed.

## Rules

- Verify, don't trust: a green checkbox is a claim; a passing verification
  command is evidence.
- The status machine is the flow's state: any phase aligns with the plan's
  current status before acting (idempotent resume).
- One plan per feature; re-running a phase continues the same file.
- In a `plan` relay flow the human gate before execute is the plan-confirmation
  checkpoint — never start executing before it is approved.
