---
name: auto-plan-regress
description: |
  Run the batch full regression gate over recently merged plans, on the main
  checkout as a single instance, then write a due-check receipt and triage
  new reds to plans. Use for "batch regression", "批量回归",
  "/auto-plan:regress", or when auto-plan-merge detects the batch gate is
  due. Auto-lang fixed-group batch (PLAN-896/898); never runs inside
  plan worktrees and never merges or fixes code itself.
---

# /auto-plan:regress — Batch full regression over recent merges

Announce: "I'm using /auto-plan:regress to run the batch full gate."

Input: none (reads due state from the receipt), or an explicit forced run.
Output: a receipt at `docs/plans/.last-batch-regression.json` committed on
the default branch, plus a triage verdict (green / known-reds-only /
new-reds with plan attribution).

## Document-source contract (PLAN-106)

Current knowledge lives in docs/specs/ documents; archived Plans retain review and
delivery history. The application generates indexes automatically. Agents never
write specs.json or manually maintain index.json lists, and merge has no index or
ledger publication prerequisite. Missing/corrupt caches and an offline application
do not block source delivery. Read/list tools may expose a derived six-category
view; retired item-write tools must direct callers to canonical source files.


## Why this tier exists

Per-plan work/review uses counterexamples and affected `cargo t` fixed
groups in the Plan worktree. After final review, merge owns the frozen complete
fixed-group union; valid evidence may be reused. Periodic breadth is separate:
the batch runs `cargo t all` once, with the UI/Man owner legs. Heavy tests can include nested builds and long
gallery fences, so this queue runs alone on main, not alongside another gate.
Do not restore a per-plan stacked-tier habit or use this skill for repairs.

## Due check (trigger)

Read `docs/plans/.last-batch-regression.json` on the default branch. A
missing or unparseable receipt counts as due. Due when either holds:

1. A plan id divisible by 5 landed after the receipt's
   `last_covered_plan_id` (use `docs/plans/archive/` + git log since the
   receipt's `covered_commit` to enumerate), or
2. More than 48h elapsed since `timestamp` with at least one merge — plan
   or L0 `fix-*` — after it. The time rule is the safety net: L0 fixes do
   not consume plan ids and must not defer the gate indefinitely.

[merge](../auto-plan-merge/SKILL.md) performs this check after `cleaned`
and hands over to this skill; the user may invoke `/auto-plan:regress`
directly at any time.

## Preconditions

- Main checkout only, single instance. Check no other heavy tier is running
  (`tasklist | grep -iE "nextest|cargo"` on Windows) and no plan worktree is
  mid-review-gate; if one is, wait for it — do not start a second full tier.
- The main checkout is clean apart from documented shared bookkeeping.
- Use current fixed-group definitions to describe expected work; old aggregate
  timing is not a budget promise. Report preparation/cold compilation apart
  from execution. A due run is one scheduled pass, not three global rounds.

## Legs

- Run the repository's `cargo t all` queue. The language leg executes the
  explicit fixed-group union from `scripts/test-groups.toml` once.
- Keep UI/Man legs and their manual/platform conditions under their owners.
  Read actual driver receipts; do not infer their success from language PASS.
- Do not separately rerun groups after the scheduled language union.
  Keep original ignored/manual/milestone conditions; T3 is not automatically
  enabled. Conditional exclusions, awaiting decisions and failures are visible,
  not a passing whole-environment claim. No automatic deletion or disabling.
- Do not run per-plan repair checks or fix the failures here; route new reds
  to their fix-forward owner. Other repositories use their own batch commands.

## Known reds baseline

Compare actual module/atomic identities and failure shapes with the prior
driver receipts, archived Plan evidence and `docs/plans/KNOWN-DEBT-AND-RISKS.md`.
Do not grant a blanket exception from a historical tier or test-family name.
List established existing failures as `known_reds_seen`; a known-reds-only
verdict is not all-green. Pending or unexecuted slow/conditional work stays
explicit. New regressions need attribution and fix-forward handling; calling
an unmet current requirement old does not make it pass.

## Triage new reds

1. Attribute each new red to a plan: failing test module/path ↔ merged
   window (`git log <covered_commit>..HEAD`), archived plan files for
   intent.
2. Open a fix-forward plan (or an L0 `fix-*` worktree for small repairs);
   record the regression in `docs/plans/KNOWN-DEBT-AND-RISKS.md` if it must
   wait.
3. Never unmerge landed plans to chase a batch red.

## Receipt

Write and commit on the default branch (the file is the single source of
truth for the next due check; its git history is the regression log):

```json
{
  "last_covered_plan_id": 710,
  "timestamp": "2026-09-30T13:00:00Z",
  "covered_commit": "<HEAD sha at run time>",
  "command": "cargo t all",
  "legs": ["<actual driver leg names>"],
  "new_reds": [],
  "known_reds_seen": []
}
```

`last_covered_plan_id` = the highest plan id landed at run time (0 when
none). Record the actual leg names, selected groups and execution receipt paths, exclusions/pending, source/dependency/config/input
identity, and per-leg timings. Selection-only or incomplete work must not
advance the covered receipt as if the queue passed. Report triage actions.

Sibling repos may adopt the same receipt pattern with their own full gate;
this skill's legs are auto-lang's.
