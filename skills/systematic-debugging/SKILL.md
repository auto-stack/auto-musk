---
name: systematic-debugging
description: Use when encountering any bug, test failure, or unexpected behavior, before proposing fixes. Investigate the root cause methodically instead of guessing at solutions.
---

# Systematic Debugging

Don't guess. Don't patch symptoms. Find the **root cause** before changing anything.

## Process

1. **Reproduce reliably.** Use `run_command` to trigger the bug. If you can't reproduce it, you can't verify the fix. Note the exact inputs and the exact (wrong) output.

2. **Read the error carefully.** What does the message actually say? What line? What values? Use `read_file` to look at the exact location. Don't skim — the answer is often in the error text.

3. **Form a hypothesis.** Based on what you read, state one specific, testable hypothesis: "I think X is null because Y didn't initialize it." Write it down.

4. **Verify the hypothesis.** Add a print/log, write a probe test, or use `search` to trace the data flow. **Prove or disprove** the hypothesis before moving on.

5. **If wrong, form a new hypothesis.** Don't stack fixes on an unverified guess. Loop back to step 3.

6. **Fix the root cause.** Once confirmed, fix the actual bug (not a symptom). Use `edit_file` for the minimal change.

7. **Verify the fix.** Re-run the reproduction (`run_command`). It should now work. Run the broader test suite to check for regressions.

## Rules

- **Never propose a fix before finding the root cause.** "Maybe try X" is guessing, not debugging.
- **One hypothesis at a time.** Change one thing, observe, repeat.
- **Read more than you change.** Use `read_file` / `search` / `list_symbols` to understand the flow before touching code.
- **The error message is your best lead.** It tells you what and where — follow it.
- **If you're stuck after 3 hypotheses, stop and report.** Don't flail — describe what you've ruled out and ask for help.

## Terminal state

The bug is fixed **and** its root cause is explained. You've verified the fix reproduces the correct behavior, and the test suite still passes.
