---
name: auto-plan-merge
description: |
  Deposit a reviewed plan's knowledge into the spec ledger (the 6 sections:
  goals/architecture/designs/tests/reviews/reports) and archive the plan file to
  docs/plans/archived/, setting status to archived. Use when:
  (1) User says "merge plan N" / "沉淀 plan N" / "归档 plan N" / "把 plan 并入 spec"
  (2) User says "/auto-plan:merge" or a plan is reviewed and ready to settle
  (3) A plan has passed /auto-plan:review and the user wants its outcome to become permanent spec knowledge
  This skill refuses any plan that is not reviewed. It first folds the plan's
  execution worktree (`.worktrees/plan-<NNN>-dev`) back into the main branch
  and removes worktree + dev branch, then prefers the
  /api/plans/{seq}/merge endpoint (automated section→spec deposit) and falls back
  to manual file edits when the backend is not running.
---

# /auto-plan:merge — Deposit a plan into the spec ledger, then archive

Take a `reviewed` plan, extract its knowledge into the spec ledger's 6
sections (each new spec item traces back to the plan via `file` + `related`),
move the plan into `archived/`, and mark it `archived`. One skill, one session, one
plan.

> **Design source:** `docs/designs/008-auto-plan.md` §6.5. The deposit engine
> lives in `backend/crates/musk/src/plan_merge.rs`; this skill drives it via the
> `/api/plans/{seq}/merge` endpoint when available, else performs the equivalent
> edits by hand.

**Announce at start:** "I'm using /auto-plan:merge to deposit plan `<NNN>` into the spec ledger."

**Input:** A plan reference (number / filename).

**State gate (HARD):** The plan **must** be `reviewed`. Refuse anything else
and point the user to the right skill (`/auto-plan:work` if unfinished,
`/auto-plan:review` if not yet reviewed). Merging unreviewed work defeats the
whole checkpoint pipeline.

## Process

### Step 1: Confirm the gate + locate the plan

```bash
# Verify status == reviewed before doing anything:
head -10 docs/plans/<NNN>-*.md | grep 'status:'
```

If not `reviewed`, **stop**. Do not "merge anyway" — tell the user what status
it is and which skill to run.

### Step 2: Fold the execution worktree back into main, then clean up

Before depositing knowledge, land the reviewed code onto the main branch and
remove the scaffolding:

```bash
# Only if it still exists — idempotent skip when already folded:
git merge plan-<NNN>-dev                            # land reviewed commits on main
git worktree remove .worktrees/plan-<NNN>-dev       # delete the worktree
git branch -d plan-<NNN>-dev                        # delete the dev branch too
```

- The worktree must be clean first. If uncommitted changes remain inside it,
  **surface them to the user** instead of discarding — uncommitted work was
  never part of what review verified.
- After landing, optionally re-run a cheap acceptance check on main so main is
  known-green before archiving.
- Dependency-project worktrees opened during execution (e.g.
  `auto-lang/.worktrees/auto-musk-dev`, named after this repo) should already
  have been folded into their own projects during execution; if one still
  exists, remind the user to close it out in THAT project — do not reach into
  another repo unasked.

### Step 3: Prefer the automated endpoint

If the backend is running, the deposit is one call — `plan_merge.rs` extracts
the plan's numbered sections, maps them to spec sections (§1→goals, §2→architecture,
§5→designs, §6→tests, §7/§9→reviews, §0→reports), upserts a traced SpecItem per
section, saves `specs.json`, and moves the plan to `archived/`:

```bash
curl -s -X POST http://127.0.0.1:8080/api/plans/<seq>/merge
# → { "plan_id": "PLAN-NNN", "sections_touched": [...], "items_created": N }
```

Confirm: the response has `items_created > 0`, the plan now appears under
`docs/plans/archived/` with `status: archived`, and `specs.json` grew. If so, skip
to Step 6.

### Step 4: Manual fallback (backend not running)

Perform the equivalent edits by hand. Read the plan's body sections and, for
each mapped section, upsert one `SpecItem` into `.autoos/specs.json` under the
matching section id:

| Plan section | Spec section id |
|:---|:---|
| §0 变更摘要 | `reports` |
| §1 目标 | `goals` |
| §2 架构方案 | `architecture` |
| §5 详细设计 | `designs` |
| §6 测试设计 | `tests` |
| §7 验收标准 / §9 复审 | `reviews` |

Each generated item:
- `id` = `P<seq>-<n>` (e.g. `P024-1`) — stable, so re-merge is idempotent.
- `title` = `<plan feature_name> — <section heading>`.
- `content` = the section body verbatim.
- `file` = `docs/plans/<filename>` (traceability back to the plan).
- `related` = `[PLAN-<NNN>]`.
- `status` = a sane initial value per section (`architecture`/`designs` → `stable`,
  `tests` → `verified`, `reviews`/`reports` → `published`, `goals` → `proposed`).

Sections §3/§4/§8/§10 are process info — do not deposit them; the archived plan
keeps them. Use the `supersedes_spec_components` metadata from review to replace
(rather than duplicate) named items.

### Step 5: Move the plan to archived/

```bash
git mv docs/plans/<NNN>-*.md docs/plans/archived/
```

Then set `status: archived` in the moved file's frontmatter (this is the terminal
state — `archived` plans do not go back).

### Step 6: Verify + report

- The execution worktree branch is folded into main; `.worktrees/plan-<NNN>-dev`
  and the `plan-<NNN>-dev` branch no longer exist.
- The plan is under `docs/plans/archived/` with `status: archived`.
- The spec ledger has new items whose `file` points at the plan.
- Re-running merge is a no-op (item ids `P<seq>-<n>` are stable → upsert is idempotent).

Report: which spec sections were touched, how many items deposited, and where
the plan now lives.

## Rules

- **Never merge a plan that is not `reviewed`.** The review gate exists for a reason.
- **Land the worktree first.** Fold `.worktrees/plan-<NNN>-dev` back into main
  and delete worktree + dev branch BEFORE depositing/archiving; do it
  idempotently (skip silently when already folded). Never leave the scaffolding behind.
- **Plan content wins over stale spec text** for the items this plan touches
  (the plan was reviewed; the spec is the materialized view). But only touch the
  items this plan generated — leave unrelated spec items alone.
- **Idempotent.** Re-merging the same plan must not duplicate items (stable `P<seq>-<n>` ids).
- **Always archive + set `archived`.** An archived plan leaves `docs/plans/` for `archived/`.
- **Defer to specialists.** After merge, if the repo keeps plan indices/reports,
  the broader `/archive-plan` skill handles index updates; this skill does not
  reimplement that.

## Checklist

- [ ] Plan confirmed `reviewed` (gate passed)
- [ ] `.worktrees/plan-<NNN>-dev` branch landed on main; worktree and dev branch removed (or confirmed already gone)
- [ ] Spec items deposited into the 6 sections, each with `file` + `related` traceability
- [ ] `specs.json` saved with the new items
- [ ] Plan moved to `docs/plans/archived/` via `git mv`
- [ ] `status: archived` set in the archived plan's frontmatter
- [ ] Re-merge confirmed idempotent (no duplicate items)
