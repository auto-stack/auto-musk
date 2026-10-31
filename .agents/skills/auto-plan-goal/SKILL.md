---
name: auto-plan-goal
description: |
  Deliver an existing Plan end-to-end in one goal session: chain work, review,
  bounded repair, and merge, and stop only when the Plan is delivered or a
  concrete blocker needs the user. Use for "complete the plan", "自动完成计划",
  "/auto-plan-goal PLAN-NNN", or a `/goal ... 完成计划 NNN` objective.
---

# /auto-plan-goal — Deliver a Plan end-to-end

Announce: "I'm using /auto-plan-goal to deliver plan NNN."

Input: one Plan reference (`NNN`, `PLAN-NNN`, or `docs/plans/NNN-slug.md`),
resolved from the goal objective or the request and located with the host's plan
listing (`list_plans`) or `docs/plans/NNN-*.md`. If the reference is missing or
matches more than one Plan, ask which Plan; never pick one by recency.

## Document-source contract (PLAN-106)

Current knowledge lives in docs/specs/ documents; archived Plans retain review and
delivery history. The application generates indexes automatically. Agents never
write specs.json or manually maintain index.json lists, and merge has no index or
ledger publication prerequisite. Missing/corrupt caches and an offline application
do not block source delivery. Read/list tools may expose a derived six-category
view; retired item-write tools must direct callers to canonical source files.


## Purpose

The four stage skills each end in a handoff and assume the next stage is invoked
with the same authorization. Without that authorization a Plan needs five manual
commands: work, review, (repair, re-review)*, merge. This skill supplies the
missing piece — stage order, entry gates, repair caps, durable receipts, stop
conditions — so one `/goal` objective can carry a Plan from `drafting` to
`archived`, or stop exactly where the user's judgment is required.

It is an orchestrator, not a fifth stage:

- At every stage boundary, load that stage's skill (`auto-plan-work`,
  `auto-plan-review`, `auto-plan-merge`, or `auto-plan-new` for a revision) with
  the `skill` tool and follow it in full. Do not paraphrase, shorten, or
  re-derive its rules here; this file decides only *when* and *whether*.
- Never edit implementation files, canonical Specs, the ledger, or the Plan's
  task rows directly — those belong to the stage skills and their worktree
  discipline.
- Do not drive the relay `plan` flow (`spawn_relay`, `complete_plan_stage`) for
  the same Plan. Relay's `PlanControl` router and this loop are alternative
  entry points; running both would double-write the Plan. If a relay run is
  already driving this Plan, stop and report the conflict.

## Authorization (one grant, inherited by every stage)

`/goal 用 /auto-plan-goal 完成计划 NNN` is the user's authorization for the
whole chain: work in the Plan's worktree (including creating it), review,
bounded repair within the approved contract, and merge (canonical Specs,
automatic index diagnostics (non-blocking), archive, worktree cleanup, landing checks). Record the grant in
§4 and in the first goal receipt; later stages inherit it and must not ask
again.

It does not authorize:

- changing the goal, acceptance criteria or thresholds, compatibility promises,
  repository/action scope, or budget;
- merging without a final `pass` bound to the current `plan_revision` and
  reviewed commit;
- bypassing the worktree rules, the evidence policy, or a stage's stop
  conditions.

Anything in that list stops the loop (see Stop conditions).

## Goal lifecycle in the host

- Active goal already present (the usual `/goal` case): use it. Its objective
  names the Plan; the stage loop below is how you satisfy it.
- No goal and the host has a goal capability: call `create_goal` with the
  delivery objective ("Deliver PLAN-NNN end-to-end: work, review, bounded
  repair, merge") and `max_goal_rounds: 12`, then start.
- A goal exists for a different objective: do not hijack it; ask.
- After a session resume or fork the goal may be disarmed. If `get_goal` shows
  it inactive, ask the user to resume it (`/goal resume`); do not claim
  progress. Never create a second goal for the same Plan.
- The objective is achieved only after merge verified delivery, archive, and
  cleanup (Done criteria). Then `update_goal` `complete` with the receipt.
- A genuine blocker is reported immediately; mark the goal `blocked` only with
  the concrete condition and only once the host allows it (it rejects an early
  mark — see Caps for the three-round waiting rule).
- Host without goal continuation: run the same loop sequentially in one turn
  until delivered or blocked. Do not invent goal state.

## Round protocol

Every round, including the first, re-derives state from durable artifacts — not
from chat memory and not from the previous round's summary:

1. Read the Plan on the main checkout: frontmatter (`status`, `plan_revision`,
   `current_step`/`total_steps`), §9 receipts from the bottom up, §10.
2. `git worktree list --porcelain`: confirm the Plan's tree and branch exist,
   belong to this Plan, and the main checkout carries no unexpected WIP code
   under code paths. Surface an unowned or unexpected state instead of editing
   it.
3. Reconcile `status` with the last §9 receipt (see Stage map). A mismatch means
   a handoff was interrupted mid-write: trust the receipt plus tree/commit
   facts, record the discrepancy, then run the stage the receipt requires.
4. Load that stage skill and execute exactly one stage.
5. Append the goal receipt to §9, then end the round with `next: <stage>`.
   With goal continuation the host starts the next round automatically; never
   ask "shall I continue?".

One stage per round. The only exception is a round that merely verifies and
routes (e.g. an already-archived Plan): record the receipt, do not write.

## Stage map

Derive the next stage from `status`, confirmed by the last final-review and
merge receipts:

| status | next stage | entry note |
|---|---|---|
| `drafting` | work | the goal authorizes the execution scope; work records that grant |
| `executing` | work | resume open tasks, or repair the latest `needs_fix` findings |
| `execution_done` | review | `review_scope: final`; if the Plan assigns a distinct final reviewer, an internal pass ends the round with that handoff instead of granting `reviewed` |
| `reviewed` | merge | requires the final `pass` bound to this revision and commit; if the recorded pass was internal while the Plan assigns a distinct final reviewer, run that final review instead of merging |
| `archived` | done | verify delivery; complete the goal without new writes |

Receipt-vs-status reconciliation:

- last final review `needs_fix`/`needs_replan` but status is `execution_done`:
  the review did not finish reopening the Plan — run the stage that outcome
  requires and record the status repair.
- last receipt `pass` but status is `executing`: the status write was lost —
  re-read §9 to decide (review pass → merge, work pass → review) before
  repeating any stage.
- last receipt `outcome: blocked` or `waiting`: the loop is waiting on the user,
  not on a stage. Re-read §10 for an answer; run no stage work and no checks.
  With an answer, resume that stage; without one, append a `waiting` receipt and
  count the waiting round (see Caps).
- worktree missing for an active `executing`/`execution_done` Plan: do not
  fabricate one under another Plan's name; investigate ownership (merge recovery
  rule) or stop.

## Outcome routing

| Stage outcome | Next action |
|---|---|
| work `pass` (`execution_done`) | review |
| work `needs_replan` | new (bounded revision) → work |
| work `blocked` | stop |
| review final `pass` (`reviewed`) | merge |
| review `needs_fix` (`executing`) | work with the findings |
| review `needs_replan` (`executing`) | new → work → review |
| review `blocked` | stop |
| merge `pass` (`archived`) | done |
| merge `needs_fix` / `needs_replan` (`executing`) | work / new → affected re-review |
| merge `blocked` (`reviewed` or `archived`) | stop, keep checkpoints |

`needs_replan` is allowed only as a revision that keeps the goal, acceptance
criteria, scope, and repository set unchanged (path/symbol corrections, or a
design that no longer works for the same agreed outcome). A revision that would
change any of those is a user decision: stop.

## Caps and progress

Count from §9 receipts so a resumed session cannot reset them:

- At most three work↔review repair cycles; then stop with the diagnosis the work
  skill requires — no silent fourth attempt.
- Two consecutive failed final reviews: require work's diagnosis checkpoint and
  its verified design/oracle adjustment before the next repair commit; do not
  spend another review on the same failing evidence.
- No progress across two rounds (same findings, same failing checks, no new
  evidence): stop and diagnose instead of retrying.
- Waiting rounds: when a user decision is required, the round does no work and
  runs no checks; it re-reads the Plan/§10 for the answer. After three
  consecutive waiting rounds, mark the goal `blocked` with the exact question.
- Goal round cap (12 by default): if reached with work remaining, report the
  last verified stage and stop; do not extend the cap yourself.

## Evidence and receipts

Follow the stage skills' evidence policy exactly: raw evidence is local,
transient scratch kept in the single scratch home
`D:/autostack/plan-artifacts/plan-<NNN>/` — never inside any repo checkout
(`docs/plans/attachments/` is fully git-ignored and must not be recreated;
the `.wt/<group>/evidence/` convention is retired); the durable form is
concise markdown summaries (commands, exit codes, key numbers, short
code/log excerpts) written into the Plan file itself, and receipts follow
the same shape with the local path plus hash. At closure, merge consolidates
all evidence into the Plan file and deletes the Plan's whole plan-artifacts
scope — only named unresolved-issue holds survive (2026-10-31 rulings).

The goal adds exactly one record type, appended to §9 after each stage:

`stage: goal | plan_id: PLAN-NNN | plan_revision: rN | round: k | from: <stage> | to: <stage> | status: <plan status> | outcome: <pass|needs_fix|needs_replan|blocked|waiting> | worktree: <path> | commit: <sha> | next: <stage|done|user>`

Keep §9 chronological and append-only; never rewrite stage receipts.

## Worktree and ownership

Delegated to work/review/merge, verified at each handoff:

- one authoritative tree per repository — `D:/autostack/.wt/musk-NNN/auto-musk`,
  branch `plan-NNN-dev` (cross-repository work: sibling tree in the same group);
- writes only inside that tree; the shared Plan stays on the main checkout;
- independent review means an independent context on the same tree — never a
  second checkout, copy, or Git-synchronized tree;
- no junctions/symlinks anywhere in a worktree; `wt-guard.sh` clean before
  removal;
- never run concurrent writers or competing builds in the tree.

## Stop conditions

Stop, write the §10 question/blocker plus a goal receipt with
`outcome: blocked|waiting`, and hand back when:

- the Plan is missing, ambiguous, or archived (unless it is this goal's delivery
  to verify);
- a required user decision appears (goal/acceptance/scope/budget/repository
  change, unowned worktree, main-checkout WIP, conflicting relay run);
- a stage returns `blocked`, or its own stop condition fires (merge: guard
  failure, archive gate failure → cleanup pending, canonical source verification failed);
- caps fire (repair cycles, waiting rounds, goal rounds).

Report the exact unblock action, not a general "needs attention". Preserve
successful checkpoints; a stopped loop must be resumable by the same objective.

## Done criteria

The goal is complete only when all of these are verified on disk, not merely
reported:

1. merge `pass` for the delivered revision, delivery commit on the default
   branch, fast-forward history intact;
2. canonical Specs verified in the landed commit per the merge skill (or its
   approved no-impact justification); application indexes are automatic and non-blocking;
3. Plan moved to `docs/plans/archived/` with `status: archived`, docs committed;
4. worktree removed and branch deleted after a clean `wt-guard.sh`, group
   directory removed when empty;
5. main checkout clean; landing checks recorded (rebuild/restart or an explicit
   stale-artifact observation).

Then `update_goal complete`, with a final receipt listing: Plan, revision,
delivery commit, Spec paths, archive path, cleanup state, and any pending
merge-owned gates.

## Report each round

One short block: Plan/revision, stage just finished, outcome, evidence anchor
(commit + command), next stage, and — only when stopping — the decision or
unblock action. Do not repeat the Plan's content.
