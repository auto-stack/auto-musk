---
name: requesting-code-review
description: Use when completing a task, implementing a major feature, or before committing, to verify the work meets requirements and quality standards.
---

# Requesting Code Review

Before declaring work done, review it yourself against a checklist. Catch your own mistakes before they become someone else's problem.

## Process

1. **Re-read the requirements.** What was the task? Use `read_file` on the spec/plan/task description. List what was asked for.

2. **Review the diff.** Use `run_command` (e.g. `git diff`) to see exactly what changed. Read every changed line. Ask: "Does this do what the task asked?"

3. **Check against this checklist:**
   - [ ] **Correctness:** Does the code do what it should? Any edge cases missed?
   - [ ] **Tests:** Are there tests? Do they pass? (`run_command`)
   - [ ] **Naming:** Are functions/variables named clearly?
   - [ ] **Errors:** Are errors handled, not silently swallowed?
   - [ ] **No dead code:** Anything unused left behind?
   - [ ] **Scope:** Did you change only what was needed, or did you creep?

4. **Fix what you found.** Use `edit_file` for any issues from the checklist. Don't skip them — "I'll fix it later" means it won't happen.

5. **Report the review.** Summarize: what was built, what tests pass, what issues you found and fixed (or chose to leave and why).

## Rules

- **Review your own diff before reporting completion.** It's not done until you've read it.
- **Be honest about gaps.** If a test is missing or an edge case is unhandled, say so — don't hide it.
- **Prefer fixing over noting.** If you spot an issue during review, fix it now, don't just list it.
- **Check scope discipline.** Unrelated "while I'm here" changes should be called out separately.

## Terminal state

A review report stating: the work meets the requirements (or what's still missing), tests pass, and known issues are listed. Then the work is ready to commit or hand off.
