---
name: auto-plan-work
description: |
  Execute or repair an identified Plan in its dedicated worktree, using the
  Plan as primary context and relevant code and Specs as evidence. Use for
  "execute plan", "执行计划", "/auto-plan:work", or a needs_fix handoff.
  Records verified progress, bounded adjustments, and structured blockers.
---

# /auto-plan:work — Implement and repair a Plan

Announce: "I'm using /auto-plan:work to execute plan NNN."

Input: a Plan reference, optionally with review findings. Resolve it from the
request or active task context; if multiple candidates remain, ask which Plan
instead of selecting the newest unrelated draft.

## Entry and context

- Read the Plan on the main checkout, repository instructions, task evidence,
  review findings, and any pending decision. `docs/specs/` is authoritative
  for agreed project knowledge; the ledger is a derived lookup/history view.
- The Plan is the primary context. Read relevant code, tests, module Specs, and
  directly cited historical decisions when needed. Record evidence for any
  discrepancy; current code does not automatically override an agreed requirement.
- Enter from `drafting` once the scope is authorized, or `executing`.
  Ordinary `execution_done` work goes to [review](../auto-plan-review/SKILL.md).
  An explicit repair request or `needs_fix` finding permits
  `execution_done/reviewed → executing`; record why and invalidate affected
  review evidence. Archived work needs a new Plan.
- Follow [new](../auto-plan-new/SKILL.md) for contract revisions. Honor prior
  approval within its scope; never infer authorization from a status field.
- Reconcile task marks with code, commits, and evidence before resuming.
  An unchecked task may already have been implemented before an interruption;
  verify it before repeating side effects. A checked task with stale evidence
  needs verification.

## Worktree and write ownership

Follow the repository's `AGENTS.md`. For auto-musk:

| Item | Location / branch |
|---|---|
| Shared Plan and progress | Main checkout `docs/plans/NNN-slug.md` |
| Implementation worktree | `D:/autostack/.wt/musk-NNN/auto-musk` |
| Development branch | `plan-NNN-dev` |
| Dependency worktree | Same group, e.g. `D:/autostack/.wt/musk-NNN/auto-lang` |
| Dependency branch | `auto-musk-dev`, subject to existing ownership checks |

1. Inspect `git worktree list --porcelain`. Reuse the Plan's existing
   worktree/branch throughout its lifetime. Confirm repository, Plan ownership,
   absolute path, and branch; a matching branch name alone is insufficient.
   Retain an existing legacy worktree for an in-flight Plan.
2. If none exists, create the worktree before edits. If the intended directory
   or branch belongs to other work, do not repurpose it. Apply the target
   repository's naming convention when these skills are used elsewhere;
   do not derive the group by stripping an arbitrary part of the repo name.
3. All implementation, test, and canonical Spec file edits/builds take place
   in worktrees. Plan progress and handoff records remain on the main checkout.
   Only one writer updates the shared Plan; reread before patching and preserve
   concurrent user edits. An unexpected concurrent edit requires reconciliation.
4. Never create junctions/symlinks inside any worktree. Dependencies resolve
   via explicit environment override, then a group sibling, then the main
   dependency checkout. Dependency modifications require their own worktree.
5. Commit completed implementation units. Preserve unexpected uncommitted
   changes and resolve ownership before committing or merging them.
   Record base commit, worktree, and dependency revisions in the Plan.
6. If the approved Plan lands phases incrementally, each landing needs a
   recorded phase acceptance review on that commit and the required full-suite
   gate. Keep the overall Plan `executing`; do not claim final review.
   Sync the default branch back before the next phase and refresh stale evidence.
   Fold an authorized dependency change back promptly after its own verification
   and successful consumer integration, then guard and clean its worktree.

Before any worktree removal, verify the resolved absolute target is inside its
intended group and require `bash D:/autostack/wt-guard.sh <worktree>` to report
clean. Final project cleanup belongs to [merge](../auto-plan-merge/SKILL.md).

## Execute verifiable units

Choose the next dependency-ready task. Respect ordering dependencies; continue
independent authorized work when another task is blocked.

For each task:

1. Implement its intended outcome. Keep work within the approved scope.
2. Run the specified or justified equivalent verification. Record command,
   expected result, actual result, and evidence tied to the code/dependency
   revision. In TDD, an expected failing test is evidence of the red phase;
   the final implementation still requires the passing result.
3. Replace its open checkbox with `[x]` and append concise evidence
   (`[✅ 已完成]` may be retained for legacy Plans). Do not count a checkbox
   and a legacy completion marker as separate tasks.
4. Set `current_step` to the number of completed executable tasks and
   `total_steps` to their total count. These are counts, not a resume cursor;
   stable task IDs, dependencies, and evidence determine remaining work.
5. Update the proposed Spec delta for actual changes. Canonical Spec edits may
   be prepared in the worktree, but do not publish them or edit the live ledger
   as an independent source of requirements.

Use scoped verification during implementation. Full suites belong at review
and pre-landing gates unless new failures, changes, or repository requirements
justify another run. Do not add tests that merely mirror wording or trivial
implementation details.

## Adapt or route from evidence

| Situation | Action |
|---|---|
| Renamed file/symbol, missing call site, equivalent local implementation | Investigate relevant sources, record adjustment, update the semantic revision if needed, and continue within existing authorization |
| Design is invalid but the goal still stands | Record evidence and affected IDs; return `needs_replan` to new for a bounded revision |
| Scope, acceptance, compatibility, or authorized actions must change | Record the concrete proposal; return `blocked` pending the required user decision |
| External dependency, permission, or environment prevents progress | Record failed prerequisite, evidence, and exact unblock action; continue unaffected work |

Do not weaken acceptance criteria or silently move required work to another
Plan. Reconcile revised tasks and their dependent evidence instead of clearing
the entire task list.

Retries must have a reason to work: new evidence, an implementation fix, or a
changed transient condition. Record attempts by task/finding. For authorized
automatic continuation, use the user's limit; if none was set, cap automatic
work/review repair cycles at three, then diagnose or hand back the blocker.
Repeated identical failure without progress stops blind retries sooner.
This is a skill-level bound, not a background scheduler.

## Completion and handoff

Before setting `execution_done`, verify that all tasks and acceptance mappings
are accounted for, required scoped checks passed on the current code, completed
changes are committed, and no blocking question remains. Incomplete work stays
`executing`.

Append a short record under `9. 复审记录`; blockers also appear under
`10. 待澄清事项`:

`stage: work | plan_id | plan_revision | outcome | code_commit |
task_ids | evidence | blockers | next`

- `pass`: set `execution_done`; next is review.
- `needs_replan`: keep `executing`; next is new with affected tasks.
- `blocked`: keep `executing`; state the precise unblock action.

Keep the worktree for review and merge. If the user already authorized the
whole workflow, continue with the next skill after satisfying its entry gate;
a request for work alone ends at this handoff. These outcome records are
distinct from the backend's five Plan statuses.
