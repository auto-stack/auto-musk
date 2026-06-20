---
name: test-driven-development
description: Use when implementing any feature or bugfix, before writing the implementation code. Write a failing test first, then implement to make it pass.
---

# Test-Driven Development

Write the test **before** the implementation. This catches design mistakes early and gives you a clear "done" signal.

## Process

1. **Understand the requirement.** Use `read_file` / `search` / `list_symbols` to study the existing code you'll touch. Don't write anything yet.

2. **Write one failing test.** Pick the smallest behavior the new feature needs. Write the test using your project's test framework. Run it with `run_command` (e.g. `cargo test`, `npm test`). **It must fail** — if it passes already, your test isn't testing the new behavior.

3. **Write the minimal implementation** to make that test pass. Resist adding anything beyond what the test requires.

4. **Run the test again.** It should pass now. If it doesn't, fix the implementation (not the test, unless the test was wrong).

5. **Repeat** for the next behavior: another failing test → implement → pass. Build the feature one test at a time.

## Rules

- **Test first, always.** Never write implementation before a failing test proves you need it.
- **One behavior per test.** If a test asserts five things, split it.
- **Run the test after every change.** Use `run_command` — don't assume.
- **Refactor only when tests are green.** If you clean up code, all tests must still pass afterward.
- If a test is hard to write, the design is probably wrong — fix the design before forcing the test.

## When NOT to use

- Pure config / data changes with no logic to test.
- Exploratory throwaway code (but switch to TDD once the approach settles).

## Terminal state

The feature is done when all tests pass and there are no untested behaviors you intended to cover. Then report which tests were added and that they pass.
