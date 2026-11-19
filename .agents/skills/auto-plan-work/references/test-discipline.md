# Test cost, build discipline, and session reports

User ruling, 2026-11-14. Apply in work, review, and merge. These rules govern
test execution and its evidence; they do not authorize wider test selection.

## New tests: at most 10 seconds each

- Inventory every test obligation added by the Plan: native tests, document
  cases/checks, parameterized inputs, and script/recipe or wrapper children.
  Compare with the actual base, rather than counting new filenames. An
  unchanged legacy test moved with an identity mapping is not a new obligation;
  a new state/input or a renamed new test still is.
- Every new test case must complete in **<=10.000 seconds**. Measure elapsed
  wall time from test entry through its setup, child work, assertions and
  cleanup, including waits inside the test. Also measure an executable wrapper
  added by the Plan as a test: packaging many children cannot hide its cost.
  Record precise values; do not round 10.001 down to 10, use an average/P90 to
  conceal an overrun, or omit a failed/slow attempt.
- Work measures new cases and investigates overruns before handing off. Review
  must return **needs_fix**, not pass or a nonblocking suggestion, for a new
  case exceeding 10 seconds. Give the programmer the full case ID, source,
  command/filter, baseline/tool/profile, observed duration, relevant phases and
  the next investigation/replay. Missing per-case timing cannot receive pass.
  A repair must provide current valid evidence while retaining the first result.
- Moving a new slow test to PERF/special, marking it ignored, splitting away
  assertions, or recording debt does not waive this new-case limit. Existing
  formal tests keep their original samples/scale/conditions; their long runtimes
  are reported, not silently rewritten to meet a rule for new cases.

## Daily tests do not build their prerequisites

- Prepare source-bound tools, workers, fixtures, providers, generated projects
  and independent compiled oracles in an explicit **pre-test build stage**.
  Daily test bodies, helpers, child processes and recipes must not launch
  Cargo/rustc, C/C++ compilation/linking, project/package builds, or lazy
  ensure-built/rebuild fallbacks. Review follows the reachable call path,
  including cache-miss and error/recovery branches, rather than only searching
  the test function for build commands.
- Use prepared artifacts whose source/toolchain/profile/features/target and
  inputs are bound to the run. Missing or stale prerequisites return a named
  preparation failure and route back to the build stage; tests never compile
  them on demand. Do not reuse an unbound executable just to avoid a build.
- Exercising the compiler's in-process parser/type checker/IR/source emitter
  is valid test behavior. It is not permission to compile a host tool,
  generated project, SDK/provider, or independent oracle inside the test.
  A test whose purpose requires an actual clean build or compiler/linker
  subprocess belongs to an explicit build/integration verification lane,
  outside the daily test union. Preserve its oracle and report its separate
  cost; the 10-second rule still applies if the Plan adds that case.
- A reachable implicit build inside a daily test is a review `needs_fix`, even
  if the reviewed warm run happened to hit a cache and did not compile.

## Debug/incremental for daily work

- Daily preparation, builds, tests and review/merge regression use dev/debug
  or the project's ordinary test profile, with incremental compilation where
  supported. Do not trigger `--release`, a Cargo profile inheriting release,
  release workers or production bundle builds as routine verification, and do
  not switch profile merely to make a slow test pass the timing gate.
- Keep required runtime properties such as panic=abort in the appropriate
  development profile. A nextest scheduling profile is not itself a Cargo
  release build; inspect the actual build argv/profile and artifact identity.
- A routine daily release-build path is also `needs_fix`; successful results
  do not waive the required debug/test verification configuration.
- A release/package/deployment explicitly required by the user's delivery
  scope is a separate operation, reported separately from daily verification.
  Existing explicit formal-PERF configuration is also separate. Neither is
  implicitly required just because a Plan reaches review or merge.

## Check machine occupancy before a unified run

Before review/merge starts a unified test union **or its preparation builds**:

1. Check active test/build jobs across agents, repositories and worktrees on
   the machine, using available coordinator/lease state and real processes.
   On Windows inspect PID + creation time, parent chain and command line for
   relevant Cargo/nextest/test-runner/compiler/linker/worker jobs. An idle
   shell or an unrelated app alone is not a competing build/test job.
2. If another agent is testing or compiling, **wait and notify the user**.
   Identify the observed Plan/worktree, actual groups or command, phase/PID,
   elapsed wait and what must finish. Do not launch the union or its builds,
   stop another agent's process, or say only "Group union is still running".
   Continue independent read-only work while waiting when useful; waits/checks
   are bounded to <=60 seconds so progress remains visible.
3. Recheck immediately before starting; use the existing shared execution lease
   if available and honor a competing holder. A check of this session alone
   does not prove machine-wide availability. If occupancy cannot be established,
   report that uncertainty and keep the union pending rather than claiming idle.
4. Record the check and any wait separately from test execution time. At start,
   name the requested groups, actual merged invocations and current stage.
   This gate applies as well when work owns a unified run; it does not force
   another full run when valid frozen evidence can be reused.

## Required session report

The user-facing work-completion, review and merge reports, and their durable
Plan records, include the **tests actually run in this session**:

| Test module / actual scope | Result and executed/excluded counts | Preparation/build | Discovery | Tests | Cleanup | Total wall time |
|---|---|---|---|---|---|---|
| Exact group(s), target/filter or script | Preserve failures, skips and NOT_RUN | Measured seconds | Seconds | Seconds | Seconds | Seconds |

For added cases, also include:

| New case ID | What behavior and oracle it tests | Profile | Actual elapsed seconds | Result / <=10s gate |
|---|---|---|---|---|
| Full native or suite/case/check identity | Concrete assertion, not just a name | Actual profile | Including setup and cleanup | Pass or needs_fix |

Mark unobserved phase times explicitly; never invent per-module times by
copying one union total or summing concurrent test durations. Show union totals
and resource-wait time separately. Identify reused evidence as reused, with its
baseline/scope, rather than claiming a session rerun. Report "no new cases"
when applicable. For docs-only work, name and time the content/link/format
checks and state that product tests were not run. A terse "tests passed" or
"group union passed" without module names and times is insufficient.
