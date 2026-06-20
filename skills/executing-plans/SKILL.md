---
name: executing-plans
description: Use when you have a written implementation plan (from writing-plans) to carry out. Executes the plan task by task, verifying each step before moving on.
---

# Executing Plans

Carry out a written plan, one task at a time. Follow the plan; don't improvise a different approach mid-execution.

## Process

1. **Load the plan.** `read_file` the plan (e.g. `plans/<feature>.md`). Re-read the relevant design doc if a task is unclear.

2. **Review critically.** If the plan has a real problem (wrong file, missing dependency, contradicts the design), raise it before starting rather than executing a broken plan. Otherwise proceed.

3. **Execute each task in order:**
   - Do step 1 (often a failing test), then step 2 (implement), then step 3 (verify). Don't skip the verify step.
   - Use `edit_file` for targeted changes and `write_file` only for new files. Use `run_command` to run the verification (e.g. `cargo test`).
   - If a verification fails, fix the issue and re-run before moving to the next task. Do not declare a task done on an unverified assumption.

4. **Report when complete.** Summarize what was done and the final verification result (e.g. "all tests pass"). If you stopped early (a task failed and you couldn't resolve it), say so plainly — don't claim success without evidence.

## Rules

- Follow the plan's tasks and steps exactly. If reality differs from the plan (a file moved, an API changed), adapt minimally and note the deviation.
- Never claim a task is complete without its verification step passing.
- If you hit a genuine blocker (missing dependency, instruction unclear, test fails repeatedly), stop and report — don't guess around it.
- Keep changes scoped to the current task; don't do work from future tasks early.

## When to Stop

- A task's verification fails repeatedly and you can't fix it → stop, report the failure with the error.
- The user asks you to pause → stop after the current task's verification.
- All tasks done → report completion with the final verification result.
