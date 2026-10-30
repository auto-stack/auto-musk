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

## Document-source contract (PLAN-106)

Current knowledge lives in docs/specs/ documents; archived Plans retain review and
delivery history. The application generates indexes automatically. Agents never
write specs.json or manually maintain index.json lists, and merge has no index or
ledger publication prerequisite. Missing/corrupt caches and an offline application
do not block source delivery. Read/list tools may expose a derived six-category
view; retired item-write tools must direct callers to canonical source files.


## Evidence storage policy (2026-10-28 user ruling)

- Evidence files — probe sources/outputs, logs, screenshots, gate/ledger
  dumps, delivery copies under `docs/plans/attachments/**` — are LOCAL-ONLY
  working artifacts. **Never `git add`, commit, or push them**: everything
  under `docs/plans/attachments/` except `*.md` is gitignored and the
  pre-commit hook rejects staged evidence paths. They matter during review
  and repair only; keep them on disk for that window.
- Record evidence as concise SUMMARIES inside the Plan file: command, exit
  code, key numbers, and the local attachment path plus content hash for
  traceability. Deliberate `*.md` summaries/contracts under `attachments/`
  may still be committed.
- Override for a genuinely load-bearing non-md artifact:
  `AUTO_LANG_ALLOW_EVIDENCE=1 git commit` with the reason recorded in the
  Plan. Already-pushed legacy evidence is intentionally left as-is.

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

For auto-lang:

| Item | Location / branch |
|---|---|
| Shared Plan and progress | Main checkout `docs/plans/NNN-slug.md` |
| Implementation worktree | `D:/autostack/.wt/lang-NNN/auto-lang` |
| Development branch | `plan-NNN-dev` |
| Dependency worktree | Same group, e.g. `D:/autostack/.wt/lang-NNN/auto-down` |
| L0 / walkthrough fix | `D:/autostack/.wt/fix-<slug>/auto-lang`, branch `fix-<slug>` |

1. Inspect `git worktree list --porcelain`. Reuse the Plan's existing
   worktree/branch throughout its lifetime. Confirm repository, Plan ownership,
   absolute path, and branch; a matching branch name alone is insufficient.
   Retain an existing legacy worktree for an in-flight Plan.
2. If none exists, create the worktree before edits. If the intended directory
   or branch belongs to other work, do not repurpose it. Apply the target
   repository's naming convention when these skills are used elsewhere;
   do not derive the group by stripping an arbitrary part of the repo name.
3. Before the first edit, assert you are inside the worktree, never the main
   checkout: `git rev-parse --show-toplevel` must resolve to the worktree path
   and `git branch --show-current` to the development branch. If either fails,
   stop and create/enter the worktree first. Re-assert after any `cd`.
4. Preflight the main checkout at session start: if its working tree carries
   uncommitted changes under code paths (`crates/**`, `test/**`, or the
   repository's equivalent), surface them to the user before any landing or
   edit of your own; another session's WIP on the main checkout violates the
   master-zero-WIP rule and needs its own routing (stash/patch handoff to its
   owner or a fix worktree), not silent inclusion.
5. All implementation, test, and canonical Spec file edits/builds take place
   in worktrees. Plan progress and handoff records remain on the main checkout.
   Only one writer updates the shared Plan; reread before patching and preserve
   concurrent user edits. An unexpected concurrent edit requires reconciliation.
6. Never create junctions/symlinks inside any worktree. Dependencies resolve
   via explicit environment override, then a group sibling, then the main
   dependency checkout. Dependency modifications require their own worktree.
7. Commit completed implementation units. Preserve unexpected uncommitted
   changes and resolve ownership before committing or merging them.
   Record base commit, worktree, and dependency revisions in the Plan.
8. If the approved Plan lands phases incrementally, each landing needs a
   recorded phase acceptance review on that commit and the required full-suite
   gate. Keep the overall Plan `executing`; do not claim final review.
   Sync the default branch back before the next phase and refresh stale evidence.
   Fold an authorized dependency change back promptly after its own verification
   and successful consumer integration, then guard and clean its worktree.

For one active Plan, implementation, internal check, and final review share
this authoritative worktree per repository, regardless of model or chat. Do not
create a per-agent copy or use Git synchronization between review and execution
trees as the handoff. Cross-repository sibling trees remain separate by repo.
Record the resolved path, repository, branch, base/current commit, dependency
revisions, and next responsible role at each handoff. Each agent explicitly
sets its tool working directory to that path; a chat's default checkout is not
proof that it is operating on the Plan tree.

Transfer write ownership sequentially. Before review, commit implementation,
resolve dirty tracked/untracked delivery files, and pause implementation writes
until the verdict. Reviewers inspect/test the frozen baseline and update the
shared Plan's evidence; implementation repairs return to work in this same tree.
Do not run concurrent writers or competing build jobs in the shared tree.
Ignored build outputs are allowed, but changed implementation, dependencies, or
verification configuration invalidate affected evidence. Before resuming work,
reread the shared Plan and verify the handoff against the tree's actual state.

The worktree mandate covers every code-writing entry, not only plan execution:
L0 trivial fixes and walkthrough/verification-session incidental fixes likewise
require a dedicated fix worktree (the `fix-<slug>` row above; auto-lang
2026-09-15 ruling 87eba67ab: the main checkout carries zero WIP code — only
Plan bookkeeping under `docs/plans/**` may be edited there). An agent editing
implementation files directly on the main checkout — with or without this
skill — violates the ruling; when you encounter such a working tree, do not
build on it: surface it to the user and help its owner route the WIP into a
worktree before proceeding.

Before any worktree removal, verify the resolved absolute target is inside its
intended group and require `bash D:/autostack/wt-guard.sh <worktree>` to report
clean. Final project cleanup belongs to [merge](../auto-plan-merge/SKILL.md).

## Execute verifiable units

Choose the next dependency-ready task. Respect ordering dependencies; continue
independent authorized work when another task is blocked.

For high-risk work or needs_fix, reconcile the next task's execution entry
before substantive edits, using the shared reference below. In the existing
contract rows, confirm the actual producer-to-consumer path, owners/lifetime,
representative states, distinguishing oracle and closing conditions against
the code. Fill evidence and ordinary implementation choices; route invalid
design or missing semantic decisions through the adaptation table. Do not
silently invent acceptance, re-ask approval for unchanged scope, or block
independent work. An ordinary local/doc fix needs no extra entry or matrix.

For those high-risk tasks, check that the oracle distinguishes a plausible wrong
implementation. Where practical, use the pre-fix failure or a controlled
mutation and a legitimate accepted control; otherwise record the evidence
and limitation. A future test name or an unconnected counter is not this
check. This is targeted verification, not a requirement for universal TDD,
fault injection infrastructure, or whole-tier regression.

For each task:

1. Implement its intended outcome. Keep work within the approved scope.
2. Run the specified or justified equivalent verification for the current
   stage, using the test-scope rules below. Record command,
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

## Test scope during implementation and repair

**A repair round verifies its findings and affected behavior; it does not restart
the Plan's complete test campaign.** This applies to every executor/model and
to scripts and standalone probe workspaces as well as the main test aliases.

- Before running checks, select the smallest set that covers each current
  finding: its durable counterexample, an accepted control, representative
  sibling paths sharing the cause, and necessary module/type/build checks.
  Keep a short `check → finding/invariant` mapping in the closure evidence.
- Finish the related fixes, then run shared affected checks once on the final
  repair revision. Do not run a global suite after each fix, commit, internal
  check, or resubmission. Fast counterexample checks during debugging remain
  appropriate; a new fix only repeats checks whose assumptions it changes.
- Do not substitute `cargo test --workspace`, the entire probe package, all
  test families, `verify -Scope all/acceptance/closeout`, or another aggregate
  script for a finding-specific test/filter. An aggregate runner's name or a
  standalone workspace does not make its full suite scoped verification.
  Broaden to an affected module/integration path only with a concrete shared
  dependency, changed configuration, or observed regression; record that reason.
- Reuse identified passing evidence for unaffected behavior after checking its
  relevant code, dependencies, configuration and inputs are unchanged. A new
  overall HEAD alone does not invalidate unrelated evidence. Do not rerun GC,
  scale/churn, full performance scenarios or unrelated compatibility families
  merely because another finding remains open. An expensive oracle that the
  repair actually affects is still required; scope is determined by impact.
- Run unified regression only after all work-owned features and in-contract
  findings are complete and the code is frozen. Keep that gate explicitly
  pending in the work handoff; its owner is final review or merge according to
  the repository workflow. Missing this future gate does not block submission
  of otherwise complete, committed repairs for acceptance review.

For auto-lang (2026-10-14 ruling), work, internal review and needs_fix re-review
must not run whole-tier cargo t/tv/tt/tb/tu/taa. Merge owns bare cargo t and the
required surface tiers after final code/acceptance review passes, before landing.
cargo tf remains main-checkout single-instance batch regression only. Other
repositories retain their own final-gate owner and timing. Do not add tests that
merely mirror wording or trivial implementation details.

## Repair the invariant, not only the reported example

For needs_fix or applicable high-risk implementation, read
[verification-contract.md](../auto-plan-new/references/verification-contract.md).
On needs_fix, reproduce the finding against the bound baseline, then list the
other applicable branches, call sites, consumers, or lifecycle states sharing
its cause. Keep one current closure row per finding:
`finding → invariant → affected paths/states → fix and permanent cases →
current results → remaining gap`. A passed original example does not close an
invariant while a declared or discovered in-contract gap remains.

Use prior review counterexamples, including accepted controls, as a shared
replay set. Consume durable inputs and expected results from the review rather
than reconstructing them from a summary; identify the input and current code.
Replay findings affected by the repair and representative sibling combinations,
and preserve assertions for previously cleared behavior. For a substantial
parser, traversal, or executor rewrite, reassess its changed grammar/lifecycle
states; do not assume the old passing corpus covers the new implementation.
This calls for representative coverage, not an exhaustive Cartesian product.

Check the required observable behavior: a final CLI error is not proof of
consumer isolation, timeout is not proof of resource cleanup, and existence is
not proof of target identity when the contract distinguishes them. Fixture
mutations must actually change the intended input and protect the specified
objects; verification totals come from executed cases, with skips/pending
explicit. An adjacent type check, earlier pipeline stage, or smaller artifact
set cannot stand in for the required oracle. Bind permanent regressions to
actual test entrypoints and assertions, not only case names or a green banner.
For each closure row, identify the durable prior input/control, applicable new
or existing check and its assertion/inspection, observed result and remaining
coverage boundary. A doc/local repair may use a content or diff inspection.
Preserve any permanent regressions required by the agreed contract.
For a targeted handoff within an unfinished Plan, report only that scope's readiness and leave
the overall Plan executing.

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
After two consecutive failed final reviews of the same Plan, diagnose before
the next repair and retain a linked diagnosis in section 9. Count across repair
commits and semantic revisions; an internal pass does not reset the sequence.
Name each remaining invariant, why earlier checks missed it, the affected
paths/states, and the design/oracle adjustment. Implement and verify that
adjustment before another ready-for-review handoff; restating the findings or
passing only the old examples is insufficient. If an architecture change is
needed, route the bounded decision through new under existing authorization.
This checkpoint does not stop authorized work or require approval again for
the unchanged scope. Aim for two or three final reviews by improving detection
and repair; the target never waives acceptance or changes the automatic retry
limit above. Do not hide extra final reviews by relabeling them internal.

## Completion and handoff

Before setting `execution_done`, verify that all work-owned tasks and criteria
are complete, required scoped checks passed on the current code, completed
changes are committed, and no blocking question remains. For needs_fix or a
high-risk Plan, include a concise readiness record in the handoff:

- Work-owned AC results, linked to current test/assertion evidence.
- Open in-contract findings, required coverage gaps, and unresolved contract
  decisions: each must be empty before a ready-for-review pass.
- Affected prior-case replay and sibling-state results; distinguish actual
  closure from a fixed original example with remaining work.
- Tests run for this repair, identified unchanged evidence reused, and unified
  regression still pending with its owner; do not demand a full-suite pass as
  an extra repair handoff prerequisite.
- Current code/dependency/configuration identity and, when triggered, the
  two-failure diagnosis plus verified adjustment.

Aggregate tests passing cannot override an admitted remaining task, partial
criterion, skipped required case, or unapproved support reduction. Keep such
work `executing`, mark affected evidence incomplete, and continue authorized
repairs or report the precise blocker. Do not hand unfinished implementation
to final review to decide whether the existing contract still applies.
Identify remaining review/merge-owned tasks and gates without marking them
complete; they do not block a work-complete handoff. Legacy Plans may record
these owners without changing their requirements. Counts include all tasks.

Append a short record under `9. 复审记录`; blockers also appear under
`10. 待澄清事项`:

`stage: work | plan_id | plan_revision | outcome | worktree_path | branch |
base_commit | code_commit | dependency_revisions | task_ids | evidence |
blockers | next_responsible_role`

- `pass`: set `execution_done`; next is review.
- `needs_replan`: keep `executing`; next is new with affected tasks.
- `blocked`: keep `executing`; state the precise unblock action.

Keep the worktree for review and merge. If the user already authorized the
whole workflow, continue with the next skill after satisfying its entry gate;
a request for work alone ends at this handoff. These outcome records are
distinct from the backend's five Plan statuses.
