---
name: auto-plan-review
description: |
  Independently verify a Plan against agreed acceptance criteria and actual
  code, bind evidence to revisions, and validate its proposed Spec delta.
  Use for "review plan", "复审计划", "/auto-plan:review", or work completion.
  Returns pass, needs_fix, needs_replan, or blocked; never merges code.
---

# /auto-plan:review — Verify the implementation and Spec delta

Announce: "I'm using /auto-plan:review to verify plan NNN."

Input: a Plan reference, optionally a named phase for an approved incremental
landing. Output: reproducible evidence and an unambiguous next action.

## Entry, authority, and independence

- Read the Plan from the main checkout. Read the implementation and diff in
  its existing worktree, plus relevant `docs/specs/`, tests, and repository
  rules. Module Specs are authoritative project knowledge; ledger entries are
  derived navigation/history, not competing requirements.
- Compare the approved intent, actual behavior, and proposed change. Trust code
  as evidence of what exists; do not treat a mismatch as permission to weaken
  the agreed requirement.
- Normal entry is `execution_done`. If `executing` claims all tasks done,
  verify that claim before treating it as ready. A draft or incomplete Plan
  returns to work. A re-review is appropriate on explicit request or when
  merge detects stale evidence. Do not reopen an archived Plan.
- Prefer an independent review session/context when available and authorized.
  A separate role/model is optional; do not claim independence merely from a
  role name. If reviewing in the implementation session, state that limitation
  and reconstruct the verdict from artifacts rather than the executor's summary.
  Do not create another task or agent without the applicable authorization.
- Identify the actual worktree via `git worktree list --porcelain`.
  If absent, verify landing through commit ancestry and recorded history;
  absence alone does not prove a merge. A review requiring code changes goes
  back to work in a worktree.

## Establish the review baseline

1. Record Plan revision, full Git commit SHA, diff base, dependency revisions,
   and relevant Spec source versions/hashes. Migrate a legacy Plan's missing
   revision as described in [new](../auto-plan-new/SKILL.md); do not fabricate
   old approvals or evidence.
   For legacy tasks/criteria without stable IDs, assign IDs while preserving
   their meaning and progress; record that normalization in the review baseline.
2. Require reviewed implementation changes to be committed. Inventory dirty
   changes and route them for resolution; do not issue a pass tied only to HEAD
   while uncommitted implementation is being tested.
3. Check authorization for the actual scope and any plan revisions. Preserve
   acceptance IDs and prior review records.

Progress markers, timestamps, and this review record do not change
`plan_revision`. Changes to the semantic contract or reviewed implementation
invalidate the affected review. A previous pass never covers future edits
automatically.

## Verify coverage and behavior

For each acceptance ID, reproduce its verification and record
`pass / partial / fail`, command or inspection method, result, and evidence.

- Run the repository-required full suite for code changes, including relevant
  backend/target checks. In auto-lang this includes `cargo tf`, plus
  `cargo tv`, `cargo tt`, or `cargo tb` for affected VM/transpiler/book work.
  Documentation-only changes use applicable content, link, and format checks
  unless repository rules require more.
- Exercise user-visible/API behavior where required; source inspection alone
  does not demonstrate a runtime acceptance criterion.
- Verify test assertions and negative cases, not just exit codes. Separate
  baseline/environment failures from regressions, but never turn an unverified
  required criterion into a pass.
- Reproduce verification for this review baseline. If a repeated review has
  unchanged code, dependencies, and test configuration, reuse identified
  evidence only with an explicit reason; rerun checks whose assumptions changed.

Map `AC IDs → task IDs → code/artifacts → evidence`. Look for missing
sub-items, configuration/call-site gaps, unapproved scope reductions,
deferrals, or workarounds. Findings need stable IDs, affected acceptance/tasks,
severity, evidence, and a concrete correction or investigation.

Recording an unmet requirement as debt does not make it complete. Distinguish
nonblocking improvements outside the agreed scope from failures within it.

## Review the knowledge delta

Read `### 规范增量` under the Plan's detailed design, or add it for a legacy
Plan using the verified implementation. For each add/modify/retire operation:

- Verify the repository-relative `docs/specs/...` target and the before/after
  rule, rationale, acceptance IDs, and current target version.
- Ensure the proposed text describes current behavior and enduring decisions,
  not abandoned implementation plans or an execution diary.
- Resolve conflicts with current canonical Specs through evidence and approved
  intent; concurrent changes require reconciliation, never blind Plan priority.
- Finalize `supersedes_spec_components`, `new_spec_components`, and
  `touched_goals`. Use exact `docs/specs/...` paths and real goal IDs.
  Empty impact requires a written explanation.
- Review any prepared Spec diff in the worktree. The ledger will be derived
  from these canonical documents and review/history artifacts at merge.

Do not publish canonical Specs or modify the live ledger during review.
If correcting the delta changes the semantic contract, increment its revision
and verify the resulting contract before issuing the final verdict.

## Record and route

Append to `9. 复审记录`:

`stage: review | plan_id | plan_revision | outcome | reviewed_commit |
base_commit | dependency_revisions | spec_inputs | acceptance_results |
findings | evidence | next`

Keep evidence paths resolvable after worktree removal: record durable repository
artifacts or concise command/result excerpts in the Plan, not only temporary logs.
The evidence package includes a frozen copy/hash of the reviewed Spec delta.

| Outcome | Plan state | Next action |
|---|---|---|
| `pass` | `reviewed` | merge; every required criterion and the delta passed |
| `needs_fix` | `executing` | work with the concrete findings and affected task IDs |
| `needs_replan` | `executing` | new to revise the invalid design or task contract |
| `blocked` | `executing` | Resolve the named prerequisite/decision, then resume the appropriate stage |

For any non-pass, reopen affected task checkboxes and their dependent evidence;
recompute `current_step` from the completed task count. Preserve unaffected
completed tasks and all historical findings. Never leave a failed review in
`execution_done` while asking work to repair it.

A phase-only review records its phase/commit verdict while the overall Plan
stays `executing`; it cannot grant final `reviewed` status. Phase landing
still requires its regression gate.

Do not merge or fix implementation within review. If the whole workflow is
already authorized, hand control to the appropriate skill and honor the repair
limit from [work](../auto-plan-work/SKILL.md); otherwise report the next action.
Outcome records are a skill contract, not new backend statuses or automatic
Relay routing.
