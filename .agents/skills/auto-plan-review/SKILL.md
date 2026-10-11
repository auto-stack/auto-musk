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
  verify that claim before treating it as ready. For high-risk or repaired work,
  reconcile the readiness record and finding closure table with the actual
  contract, code, and assertions; an internal pass or test total is not the gate.
  Known unfinished work-owned criteria or required oracles return to work,
  even if the receipt says execution_done. Record the concrete gap; do not
  spend another full final-review cycle merely to reconfirm admitted unfinished
  implementation. Explicit investigation or targeted review requests can still
  examine partial work, but cannot issue final pass. A re-review is appropriate
  on explicit request or when merge detects stale evidence. Do not reopen an
  archived Plan.
- Prefer an independent review session/context when available and authorized.
  A separate role/model is optional; do not claim independence merely from a
  role name. If reviewing in the implementation session, state that limitation
  and reconstruct the verdict from artifacts rather than the executor's summary.
  Do not create another task or agent without the applicable authorization.
- Identify the recorded worktree via `git worktree list --porcelain`; verify
  its resolved path, repository, branch, and handoff commit. Internal and final
  reviewers enter the same Plan tree used by work, with independent contexts
  where assigned. Do not create a reviewer-specific checkout or synchronize
  between model-specific trees. Set tool working directories explicitly.
- During review, implementation writes are paused. Inspect/test committed code
  and record findings; route repairs to work in the same tree. Read the current
  shared Plan on main, not a stale copy in the implementation tree.
  If an active, unlanded Plan's tree is missing, resolve ownership/recovery
  before issuing pass. For an already-landed historical review, verify commit
  ancestry and record the inspected checkout and historical baseline;
  absence alone does not prove landing. Code changes go back to work.

## Establish the review baseline

Resolve `review_scope: internal | final | phase` from the request and the Plan's
assigned roles; record it in the receipt. With no split-role workflow, the
ordinary independent acceptance review is final. With a designated final
reviewer, an internal pass leaves `execution_done` and routes to that reviewer;
it cannot grant `reviewed` or authorize merge. An internal non-pass returns the
affected work to `executing`. Phase passes keep the Plan `executing`.
Use a context independent of implementation for the assigned final review when
available and authorized; do not relabel an implementation-session self-check
as the separate final reviewer. Record actual reviewer/context identity when
exposed, and limitations otherwise; never fabricate a model or session ID.

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
4. Recheck HEAD, implementation cleanliness, dependencies, and verification
   configuration before recording the verdict. Unexpected drift makes affected
   evidence stale; reconcile it instead of passing the originally named commit
   against a changed tree. Record the worktree path and branch with the receipt.

Progress markers, timestamps, and this review record do not change
`plan_revision`. Changes to the semantic contract or reviewed implementation
invalidate the affected review. A previous pass never covers future edits
automatically.

## Verify coverage and behavior

For each acceptance ID, reproduce its verification and record
`pass / partial / fail`, command or inspection method, result, and evidence.

- Run the target repository current change-scoped review gates and applicable backend/target checks. Its test-tier and serialization rules govern; do not substitute a stale global full-suite example. Documentation-only changes use applicable content/link/format checks unless repository rules require more.
- For a multi-repository Plan, identify and run each target repository own required gates; do not apply one repository tier by analogy to another.
- Exercise user-visible/API behavior where required; source inspection alone
  does not demonstrate a runtime acceptance criterion.
- Verify test assertions and negative cases, not just exit codes. Separate
  baseline/environment failures from regressions, but never turn an unverified
  required criterion into a pass.
- Reproduce verification for this review baseline. If a repeated review has
  unchanged code, dependencies, and test configuration, reuse identified
  evidence only with an explicit reason; rerun checks whose assumptions changed.

For the Plan's high-risk invariants, inspect the complete declared path/state
matrix and independently choose representative counterexamples, not only the
executor's regression inputs. Read
[verification-contract.md](../auto-plan-new/references/verification-contract.md)
when applicable. Cover the identified risk areas in the first review rather
than stopping at the first defect; mark unexamined areas pending with reasons.
New scope is a contract change, not an undisclosed acceptance requirement.
For document contracts, check rule consistency and dependency feasibility,
not only field or example counts.

On re-review, distinguish old-example closure, full-invariant closure, and
new repair regressions. Replay affected prior inputs and accepted controls
against the new baseline, with hashes when input identity matters. A changed
parser/traversal/executor needs representative combination or failure-exit
checks for its new paths; earlier helper evidence does not cover a replacement.
Keep counterexample input, expected behavior, command and actual result durable
and directly reusable by work. Describe the violated existing rule and coverage
boundary, so a new probe does not silently become a new scale promise. After
two failed final reviews, verify the diagnosis and coverage/design adjustment
from work before treating the next handoff as ready.

Check raw invocation/results against claimed case IDs and totals. For a
validator used as acceptance evidence, test meaningful valid and invalid
controls, actual mutation, and failure propagation; an ALL-PASS banner cannot
prove its own coverage. Required future merge checks remain recorded pending,
not claimed pass on the strength of a mock or a receipt template.

Map `AC IDs → task IDs → code/artifacts → evidence`. Look for missing
sub-items, configuration/call-site gaps, unapproved scope reductions,
deferrals, or workarounds. Findings need stable IDs, affected acceptance/tasks,
severity, evidence, and a concrete correction or investigation.

Recording an unmet requirement as debt does not make it complete. Distinguish
nonblocking improvements outside the agreed scope from failures within it.
An in-contract defect cannot be downgraded merely because it is pre-existing,
rare, or described as nearly unreachable. Findings identify whether they are
an original requirement gap, contract clarification/change, new regression,
input drift, or closeout defect; this determines the repair and review scope.

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

`stage: review | review_scope | reviewer_identity/limitations | plan_id |
plan_revision | outcome | worktree_path | branch | reviewed_commit | base_commit |
dependency_revisions |
spec_inputs | acceptance_results | pending_closeout_gates | findings | evidence |
next`

Keep evidence paths resolvable after worktree removal: record durable repository
artifacts or concise command/result excerpts in the Plan, not only temporary logs.
The evidence package includes a frozen copy/hash of the reviewed Spec delta.

| Outcome | Plan state | Next action |
|---|---|---|
| final `pass` | `reviewed` | merge; required implementation/review criteria and delta passed, explicit merge-owned gates remain for closeout |
| internal `pass` | `execution_done` | designated final review; no merge/archive/cleanup |
| `needs_fix` | `executing` | work with the concrete findings and affected task IDs |
| `needs_replan` | `executing` | new to revise the invalid design or task contract |
| `blocked` | `executing` | Resolve the named prerequisite/decision, then resume the appropriate stage |

For any non-pass, reopen affected task checkboxes and their dependent evidence;
recompute `current_step` from the completed task count. Preserve unaffected
completed tasks and all historical findings. Never leave a failed review in
`execution_done` while asking work to repair it.

A phase-only review records its phase/commit verdict while the overall Plan
stays `executing`; it cannot grant final `reviewed` status. Phase landing
still requires its regression gate (the scoped daily gate above — not the
batch `cargo tf` tier).

Do not merge or fix implementation within review. If the whole workflow is
already authorized, hand control to the appropriate skill and honor the repair
limit from [work](../auto-plan-work/SKILL.md); otherwise report the next action.
Any semantic fix changes the reviewed baseline: return to work, commit it,
and verify affected criteria on the resulting revision before a new final pass.
Do not cover code added after pass with a comment-only/Spec-only exception.
Outcome records are a skill contract, not new backend statuses or automatic
Relay routing.
