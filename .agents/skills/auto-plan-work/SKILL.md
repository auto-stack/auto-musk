---
name: auto-plan-work
description: |
  Execute an implementation plan step by step, using the plan file as the sole
  context. Walks the task list, marks each step done, advances the status
  (drafting → executing → execution_done), and logs blockers to the plan's
  open-questions section instead of going off-script. Use when:
  (1) User says "execute plan N" / "执行 plan N" / "继续做 plan" / "work on plan 42"
  (2) User says "/auto-plan:work" or "按这个 plan 干活"
  (3) A plan in docs/plans/ is in drafting/executing status and the user wants to advance it
  Reads only the target plan — never loads specs or other plans mid-execution.
---

# /auto-plan:work — Execute a plan

Execute one plan, end to end, treating that plan as the **only** context. One
skill, one session, one plan. Resumable — if interrupted, re-run and resume from
the first unchecked task.

> **Design source:** `docs/designs/008-auto-plan.md` §6.3.

**Announce at start:** "I'm using /auto-plan:work to execute plan `<NNN>`."

**Input:** A plan reference — a number (`42`), a filename (`042-*.md`), or the
literal "执行 Plan N". If omitted, pick the newest `drafting` or `executing`
plan in `docs/plans/`.

**State gate:** The plan must be `drafting` or `executing`. If it is
`execution_done` / `review_done` / `merged`, refuse and point the user to the
right next skill (`/auto-plan:review` for `execution_done`).

## Process

### Step 1: Locate and load the target plan

```bash
# By number — match the 3-digit prefix in active OR archived dir:
ls docs/plans/<NNN>-*.md docs/plans/archived/<NNN>-*.md 2>/dev/null
```

Read that one file. It is now the sole context for this session. **Do not load
specs, other plans, or the design doc** — the plan already contains everything
needed; going off-script is how steps get dropped (008 §6.3 constraint).

### Step 2: Advance the state machine if needed

If `status: drafting`, flip it to `executing` in the frontmatter and bump
`updated_at`. If already `executing`, leave it. Legal transitions:
`drafting → executing`; `executing → execution_done` (Step 5).

### Step 3: Execute tasks in order, marking each done

Work through `## 执行步骤` top-to-bottom. For each step:

1. Do exactly what the step says — precise file path, exact operation.
2. Run the step's verification command. It must pass before moving on.
3. Append `[✅ 已完成] <one-line evidence>` next to the step in the plan file.
4. Bump `current_step` in the frontmatter.

**TDD order (superpowers rule):** when a step writes code with tests, write the
failing test first, confirm it fails, then implement, then confirm it passes.

### Step 4: Log blockers in-plan, do NOT improvise

When a step is ambiguous or blocked, **do not** go search specs or other docs to
figure it out. Instead append a bullet under `## 待澄清事项` (Open Questions)
and stop there for the user to resolve. Going off-script to "research" is how
execution drifts from the reviewed plan.

### Step 5: When all tasks are done → execution_done

Once every step has `[✅]` and every verification has passed:

1. Set `status: execution_done` in the frontmatter.
2. Run the plan's whole verification suite (acceptance criteria section) once more.
3. Hand off: tell the user the plan is ready for `/auto-plan:review`.

Do not review or merge — those are separate skills.

## When to stop and ask for help

Stop immediately and surface to the user if:
- A verification command fails repeatedly and the fix is unclear.
- A step's instructions contradict the current code and you cannot tell which is right.
- The plan references a file/path/module that does not exist and the intent is unclear.

Ask rather than guess — a wrong step propagates to every later step.

## Rules

- **Only read the target plan.** No specs, no other plans, no design docs mid-flight.
- **Every completed step gets a `[✅]` marker + `current_step` bump.** No silent progress.
- **TDD: failing test → implement → passing test**, when tests apply.
- **Blockers go to `## 待澄清事项`, not into speculative research.**
- **Follow steps exactly.** If a step looks wrong, stop and ask — do not redesign on the fly.
- **Do not start on the default branch without consent** (general safety rule).

## Checklist

- [ ] Target plan located; loaded as sole context
- [ ] `status` advanced (`drafting → executing`, or already `executing`)
- [ ] Every execution step has `[✅ 已完成]` evidence
- [ ] `current_step` reflects the furthest completed step
- [ ] All per-step verification commands passed
- [ ] Blockers (if any) recorded under `## 待澄清事项`, not silently worked around
- [ ] On completion: `status: execution_done`; user pointed to `/auto-plan:review`
