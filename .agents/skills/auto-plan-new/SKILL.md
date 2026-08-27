---
name: auto-plan-new
description: |
  Create a new implementation plan file in docs/plans/ with an auto-assigned
  3-digit sequence number, YAML frontmatter, and a needs-analysis section seeded
  from the spec ledger overview. Use when:
  (1) User says "new plan" / "新建 plan" / "创建计划" / "建个 plan" / describes a new feature or requirement to plan
  (2) User says "/auto-plan:new" or "给这个需求建个 plan"
  (3) A new requirement arrives and no plan for it exists yet
  This skill only scans docs/plans/ for the next number and reads the spec
  overview; it never reads other plan files (avoids context pollution) and never
  starts executing — hand off to /auto-plan:work.
---

# /auto-plan:new — Create a new plan

Create a single, self-contained plan file that becomes the **sole execution
context** for `/auto-plan:work`. One skill, one session, one plan. Idempotent —
re-running with the same requirement overwrites the same draft until the user
confirms.

> **Design source:** `docs/designs/008-auto-plan.md` §6.2.

**Announce at start:** "I'm using /auto-plan:new to draft the plan."

**Input:** A requirement description (natural language). If vague, clarify first.

**Output:** `docs/plans/NNN-slug.md` with `status: drafting`, presented for
user confirmation. Execution is **not** started.

## Process

### Step 1: Assign the next sequence number (deterministic, never guess)

Scan BOTH the active and archived directories so numbers never collide with
history:

```bash
ls docs/plans/*.md docs/plans/archived/*.md 2>/dev/null \
  | sed -E 's|.*/([0-9]{3})-.*|\1|' | sort -n | tail -1
```

Take the max, add 1, zero-pad to 3 digits (max `023` → `024`). Empty directory
→ `001`. This is authoritative — `PlansStore::next_seq` uses the same rule, so a
number assigned here will not collide with one later assigned by the API.

### Step 2: Read the spec overview (background, not full specs)

Ground the plan's "needs analysis" in the project's current state:

```bash
# Preferred: structured overview from the running backend
curl -s http://127.0.0.1:8080/api/specs/overview 2>/dev/null \
  || cat .autoos/specs.json 2>/dev/null || cat backend/.autoos/specs.json
```

Extract: existing modules, goals, architecture items, anything the new plan
touches. **Do not read other plan files** — they bias the draft and waste
context (008 §6.2 constraint).

### Step 3: Clarify the requirement (only if vague)

If the requirement is clear, skip to Step 4. If vague (ambiguous scope, missing
constraints, several interpretations), ask 1-3 focused questions **one at a
time**. Prefer proposing 2-3 concrete options over open-ended questions. Record
the resolved decisions in the plan's "needs analysis" section.

### Step 4: Draft the plan file

Create `docs/plans/NNN-slug.md` where `slug` is a short English kebab-case name
derived from the feature. Use this frontmatter (008 §4.2):

```yaml
---
plan_id: PLAN-NNN              # matches the filename prefix
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: <concise name>
author: [<you>]
created_at: <ISO now>
updated_at: <ISO now>

# Leave these EMPTY here — /auto-plan:review fills them:
supersedes_spec_components: []
new_spec_components: []
touched_goals: []

current_step: 0
total_steps: <count of execution tasks>
---
```

Body sections (in order): `# [PLAN-NNN] <title>`, 变更摘要, 目标, 架构方案,
技术栈, 需求分析与背景调查 (seeded from Step 2), 详细设计, 测试设计, 验收标准,
执行步骤 (atomic tasks), 复审记录, 待澄清事项.

**Execution-task granularity (superpowers rule):** each task = a 2-5 minute
atomic action with (a) precise file paths, (b) the exact operation, (c) a
verification command. Forbidden: "TBD", "TODO", "similar to Task N",
"implement later".

### Step 5: Present for confirmation — do NOT execute

Show the drafted plan to the user. Update `updated_at`. Wait for confirmation or
edits. Starting execution is `/auto-plan:work`'s job, not this skill's.

## Rules

- **Draft on the default checkout; code lands in a worktree later.** The plan
  doc must stay visible on the default checkout (the backend and every other
  skill read `docs/plans/` from there). Actual implementation happens in a
  dedicated worktree `.worktrees/plan-<NNN>-dev`, created by `/auto-plan:work`
  when execution starts.
- **Never read other plan files.** They pollute context and bias the draft.
- **Only read the spec overview**, not full spec contents.
- **Sequence numbers are computed from the filesystem, never hardcoded.**
- **No placeholders.** Every task names real files + a real verification command.
- **Hand off, don't execute.** New ends at `status: drafting`.
- **Respect the design doc.** Plan format follows `docs/designs/008-auto-plan.md` §4.2.

## Checklist

- [ ] `docs/plans/NNN-slug.md` exists with `status: drafting`
- [ ] `plan_id: PLAN-NNN` matches the filename's 3-digit prefix
- [ ] Sequence number = max(active + archived) + 1, no collision
- [ ] Needs-analysis section references real spec modules from the overview
- [ ] Every execution task has a file path + operation + verification command
- [ ] `supersedes`/`new_spec_components`/`touched_goals` left empty (deferred to review)
- [ ] User has confirmed or edited the draft; execution not started
