---
name: writing-plans
description: Use when you have an approved design or requirements for a multi-step task, before touching code. Produces a bite-sized, file-mapped implementation plan that the executing-plans skill then carries out.
---

# Writing Plans

Turn an approved design (or requirements) into a concrete, bite-sized implementation plan. **Do not implement yet** — this skill only writes the plan.

## Process

1. **Read the design.** `read_file` the design doc (from brainstorming) or the user's requirements. If none exists, ask for one or do a quick brainstorm first.

2. **Map the files.** Use `search` and `read_file` to find exactly which files will change. Name them. A plan that doesn't say which files it touches is not ready.

3. **Break into bite-sized tasks.** Each task is one coherent change (2-5 minutes): a single function, a single file edit, a single test. If a task is "add the feature," it's too big — split it.

4. **Write the plan to `plans/<feature>.md`.** Use this shape per task:
   ```
   ### Task N: <one-line title>
   Files: <exact paths>
   - [ ] <step 1 — often a failing test>
   - [ ] <step 2 — implement>
   - [ ] <step 3 — verify (run command)>
   ```

5. **Self-review the plan.** Check:
   - Every requirement in the design maps to at least one task.
   - No "TBD" / "fill in later" placeholders.
   - Types and names are consistent across tasks (a struct named `Foo` in task 2 is `Foo` in task 5).
   - Each task lists the files it touches.

6. **Offer execution.** Tell the user the plan is ready and that you'll invoke **executing-plans** to carry it out.

## Rules

- No code changes in this phase (the plan is the output).
- Prefer an ordered task list (task N depends on N-1). Note explicit dependencies if non-linear.
- Include a verification step (a `run_command` like `cargo test` / `npm test`) per task where applicable.
- The terminal state is a saved plan file + an offer to execute it via executing-plans.

## Output

A plan file at `plans/<feature>.md` with bite-sized, file-mapped tasks. Then: "Plan written — invoke executing-plans to carry it out task by task."
