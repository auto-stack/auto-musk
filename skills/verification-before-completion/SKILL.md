---
name: verification-before-completion
description: Use when about to claim work is complete, fixed, or passing — before committing or reporting success. Requires running verification commands and confirming output, not assumptions.
---

# Verification Before Completion

**Never claim "done" without evidence.** Run the verification, read its output, and only report success if the output confirms it.

## Process

1. **What does "done" mean for this task?** Identify the concrete check: a test suite, a build, a command that demonstrates the fix.

2. **Run the verification.** Use `run_command` to execute it:
   - For code changes: `cargo test`, `npm test`, `pytest`, etc.
   - For fixes: re-run the exact reproduction that previously failed.
   - For builds: `cargo build`, `npm run build`, etc.

3. **Read the actual output.** Look at what the command printed. Don't assume — read it.
   - Did it say "passed" / "0 failed"? Or did it say "error"?
   - Did the reproduction now produce the correct result?

4. **If verification fails, you are NOT done.** Do not report success. Either fix the issue and re-verify, or honestly report that it's incomplete with the failure output.

5. **Report with evidence.** When you say "done," include what you ran and what it output: "All 12 tests pass (`cargo test` output: `test result: ok. 12 passed`)".

## Rules

- **"I think it works" is not verification.** Run the command. Read the output.
- **Never report success on an unverified assumption.** If you didn't run it, you don't know.
- **Include the evidence in your report.** Name the command and quote the key output line.
- **Partial completion is fine — lying about it isn't.** "3 of 4 tests pass, test X fails because Y" is far more useful than a false "done."
- **Skipping a verification step to save time is a false economy.** The 2 minutes you skip now costs hours of debugging later.

## Terminal state

A completion claim backed by quoted command output. If you cannot produce evidence, you cannot claim completion — report what's blocking instead.
