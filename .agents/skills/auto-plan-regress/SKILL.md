---
name: auto-plan-regress
description: |
  Run the batch full regression gate over recently merged plans, on the main
  checkout as a single instance, then write a due-check receipt and triage
  new reds to plans. Use for "batch regression", "批量回归",
  "/auto-plan:regress", or when auto-plan-merge detects the batch gate is
  due. Auto-lang tiering (fix-test-tiering, 2026-09-30); never runs inside
  plan worktrees and never merges or fixes code itself.
---

# /auto-plan:regress — Batch full regression over recent merges

Announce: "I'm using /auto-plan:regress to run the batch full gate."

Input: none (reads due state from the receipt), or an explicit forced run.
Output: a receipt at `docs/plans/.last-batch-regression.json` committed on
the default branch, plus a triage verdict (green / known-reds-only /
new-reds with plan attribution).

## Why this tier exists

Per-plan review runs the scoped daily gate (`cargo t` + surface tiers
`tv`/`tt`/`tb`/scoped `taa`) inside the plan worktree. Full tiers
(`cargo tf`) are machine-monopoly workloads — in-test nested cargo builds,
the ~800s zero-cache gallery fence — so they are batched here: run alone,
on the main checkout, never in parallel with plan worktrees or other full
tiers (see the 2026-09-30 parallel-`cargo tf` saturation incident).

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
- Warm-cache budget ≈ 15 minutes (the gallery fence dominates); a cold
  target dir costs more — say so in the report rather than guessing.

## Legs

- Always: `cargo tf`, `cargo tt`, `cargo tb`. The latter two (~24s each)
  close the gap of surface tiers that per-plan review only runs "when
  touched".
- Conditional: bare `cargo taa` only when any change merged since the last
  receipt touched aavm trigger paths (`auto/lib/*.at`, `test/vm/aavm2/**`,
  `parity/**`, `crates/auto-lang/src/tests/aavm2_*` / `aavm_runner_tests.rs`,
  `lib.rs` aavm lib sources). Window diff:
  `git log --name-only <covered_commit>..HEAD`.
- Never here: `cargo t3` (milestone-only), per-plan scoped tests, or fixes
  to the failures found (those route to fix-forward plans).

## Known reds baseline

Compare results against recorded pre-existing failures before reporting:
the daily-tier pre-existing reds on file (AGENTS 564-Q6 list), taa's
`charts_gallery` pre-existing red, and anything already recorded in
`docs/plans/KNOWN-DEBT-AND-RISKS.md`. Only NEW reds act; known reds are
listed in the receipt as `known_reds_seen` without blocking.

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
  "legs": ["tf", "tt", "tb"],
  "new_reds": [],
  "known_reds_seen": []
}
```

`last_covered_plan_id` = the highest plan id landed at run time (0 when
none). Report the verdict, leg timings, and any triage actions taken.

Sibling repos may adopt the same receipt pattern with their own full gate;
this skill's legs are auto-lang's.
