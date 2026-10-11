# Verification contracts for high-risk Plans

Read when a Plan changes compiler/IR semantics, trusted admission/validation,
resource lifetimes, publication transactions, or a contract used by multiple
consumers. Use the applicable parts, not a universal exhaustive checklist.
Ordinary low-impact changes use their scoped checks.

## Put decisions and oracles in the contract

Keep the repository's numbered Plan sections and five lifecycle states.
In sections 5–8, distinguish the current contract from linked historical
receipts. Specify behavior and externally checkable results; leave routine
implementation choices to work. Unresolved architecture belongs in a bounded
investigation task, not invented detailed pseudocode.

For each risky rule, explain what it protects and choose representative cases
that would fail under a plausible wrong implementation. Section 6 can use:

| Case ID | AC / invariant | Input or setup | Entry point | Observable expected result | Assertion / evidence | Coverage boundary |
|---|---|---|---|---|---|---|
| V-01 | AC-01, mapped argument types | Different parameter types, reordered evaluation with valid bindings | Actual binder/verifier and native runner | Valid mapping accepted; native result matches; side effects retain evaluation order | Result, trace, diagnostic stage | Declared profile; not every future type |
| V-02 | AC-01, mapped argument types | Same input with invalid bindings | Actual verifier | Controlled rejection before backend | Nonzero plus specific diagnostic; no unchecked consumer | Mapping/type gate |
| V-03 | AC-02, failed publication preserves old output | Existing valid artifact set; representative failure during publication | Public build/publish entry | Old bytes preserved if not committed; cleanup or accurate residual/recovery report | Hashes and old executable result, owned residual paths | Specified artifact set and failure stages |

These are illustrative cases, not mandatory IDs or universal feature requests.
Select inputs from the real syntax/API. A proposed interface can name future
fixtures and the task that will make them runnable; do not report them as
executed before implementation. Existing supported inputs should use actual
code or source evidence.

For each applicable risk, cover representative axes rather than every Cartesian
combination:

- Identity/type: both ownership directions; heterogeneous mappings; values
  crossing local, parameter, return, and branch boundaries when supported.
- Graph structure: tiny cycles, disconnected components, valid nesting; ensure
  invalid structure is rejected before an unsafe traversal.
- Resource/publication: preparation, execution, output collection, publication,
  rollback and cleanup failure exits; distinguish timeout from successful reap.
- Trusted data: shape, semantic validity, freshness, uniqueness and applicability;
  check downstream admission if promised, not only the final CLI exit code.
- Classification: the declared binding/owner forms and ambiguity policy;
  legitimate controls must remain accepted.
- Workflow/validator: partial and complete states, active and archived paths,
  required references and exact target identity; mutations occur and a failing
  executed control makes the overall gate fail.

Explicitly state exclusions and pending decisions. A later review must identify
whether a finding violates this contract, clarifies it, or proposes new scope.
Do not retroactively treat every new case as an original explicit requirement.

## Resolve support and implementation feasibility early

For an input-processing Plan where depth or size can change admission, state
whether a limit is part of the approved profile. Define any material limit's
unit, applicable entrypoints, boundary behavior and diagnostic classification;
choose valid/invalid controls on both sides. Do not add a fixed syntax exclusion
as an incidental crash repair. No artificial syntax limit is distinct from
infinite memory, arbitrary-scale performance, or termination guarantees.

When the design is uncertain, assign a bounded probe before dependent work.
For recursive compiler structures, inspect representative flat chains,
continuous and interleaved nesting, calls and blocks, including malformed
partial trees. Check relevant parsing, resolution, validation, emission and
construction/destruction paths; iterating one traversal does not establish the
whole pipeline. Select feasible representative scales and record what was
actually established, rather than creating a universal stress threshold.

For subprocess or publication work, identify an existing verified facility and
its actual consumers before creating another implementation. If replacement is
necessary, give it representative success, failure and cleanup controls; it
cannot inherit the prior helper's pass merely by keeping its interface.

## Make mandatory oracles traceable

Give required supplemental cases stable IDs, linked ACs, work owners, entrypoints
and distinguishing assertions. Add current evidence/status during execution;
a narrative claim or corpus total does not mean the supplement ran. Proposed
case expectations must agree with the prose contract before implementation.
A skip or unavailable required oracle remains a gap unless an approved contract
change removes it; review/merge-owned future operations remain separately pending.

Use controls that isolate the promised property. For a scope test, a same-type
out-of-scope reference avoids an unrelated type error proving the wrong rule.
For a Bundle gate, mutate the actual Bundle and prove mutation occurred. For
three-file rollback, start with three real old files and exercise the promised
failure stage, then compare all three. For a process deadline, observe complete
output/error propagation and owned child/reader cleanup, not only caller return.
These are examples for the applicable contract, not requirements for every Plan.

Readiness evidence should name the actual assertion and current artifact.
Preserve raw review counterexamples and accepted controls as a shared replay
set. A repair replays affected cases plus representative sibling combinations;
changed implementation states need fresh controls. Reuse unchanged evidence
only with the explicit dependency/configuration justification required by review.

## Separate task ownership and review roles

Use `owner_stage` in the Markdown task/AC tables or handoff. These are workflow
instructions; they do not claim new backend schema or parser support.

| Task | owner_stage | Outcome and linked AC | Evidence |
|---|---|---|---|
| T-01 | work | Implementation and regression cases | Current committed result |
| T-02 | review | Independent acceptance decision | Final reviewed commit and cases |
| T-03 | merge | Actual archive references/counts/cleanup | Actual-state checks and receipts |

All unique tasks count toward total_steps. current_step counts actually completed
tasks; a pending review or cleanup is not marked done to make handoff look
complete. work enters execution_done when its owned tasks are complete, final
review enters reviewed when its criteria pass, merge finishes the remaining
explicit closeout gates before delivered pass.
Merge-owned criteria concern closeout, not deferring implementation correctness.
At the archive checkpoint, cleanup may still be pending and counts must say so;
only after actual cleanup are those tasks completed and final counts verified.

For an explicitly split workflow, record roles and routing in section 6:

```text
Implementation: assigned implementation agent.
Internal check: assigned checker; pass -> execution_done, final review next.
Final review: assigned independent reviewer; pass -> reviewed, merge next.
Closeout: merger; actual final-state check precedes cleanup.
```

Preserve user-assigned model choices; do not create or contact another agent
without authority. With no split roles, the normal independent review remains
final. An implementation-session check states its limitation and cannot stand
in for an explicitly assigned separate final reviewer.

## Share the checkout, separate the contexts

Record an intended worktree path/branch when planning; confirm the actual
resolved path, repository, branch, base/current commit and dependencies in work.
One active Plan has one implementation tree per repository. The implementation
agent, internal checker and final reviewer all operate on that tree, in sequence;
a separate review context does not require another checkout. Multi-repo Plans
use sibling trees per repo, not copies per model. Shared Plan bookkeeping stays
on the main checkout according to repository rules.

A handoff identifies the current committed baseline and next responsible role.
Implementation pauses writes during review; reviewers inspect/test and record
evidence without repairing implementation. Changes return to work in the same
tree and invalidate affected evidence. Verify path/branch/HEAD at entry and
baseline stability before verdict. Chat creation or skill invocation does not
move tool working directories automatically. Keep the tree until authorized
closeout completes, then clean it under the repository's guard rules.

## Keep evidence verifiable and the contract readable

Record case IDs, commands, observed values/diagnostics, implementation and
dependency fingerprints, relevant artifact identity, and raw result location.
For split/high-risk workflows, identify the skill source/content version and
exposed reviewer/context identity so stale instructions or routing are visible.
Counts derive from actual invocations/results. List skipped or future gates as
such. A helper must consume the intended artifact; an arbitrary cached binary
or an ALL-PASS banner is not provenance.

During repair, use a concise closure table:

| Finding | Violated invariant | Applicable paths/consumers/states | Fix and permanent cases | Current result / remaining gap |
|---|---|---|---|---|

After two consecutive failed final reviews of the same Plan, diagnose this
table and the oracle before another repair. Count across repair commits and
semantic revisions. Link a concise diagnosis in section 9 with:

| Remaining invariant | Why the earlier check missed it | Affected paths/states | Design or oracle adjustment | Current result / blocker |
|---|---|---|---|---|

Verify the adjustment before the next ready-for-review handoff. Known open
in-contract findings, mandatory oracle gaps or unresolved support decisions
prevent execution_done regardless of aggregate green tests. Partial work stays
executing; future review/merge-owned tasks remain pending without blocking a
work-complete handoff. Preserve scope authorization and continue repairs;
ask only for a genuinely changed requirement or missing decision.

Two or three final reviews is a convergence target, not permission to pass
unfinished work. Record actual final-review count and repair regressions; do
not reset the sequence on an internal pass or hide rounds through relabeling.

Keep one current task row per stable ID. Use short current status/evidence links
in sections 5–8; put detailed per-round repair narratives and raw logs in linked
reports, with concise history under section 9. Do not let old completion prose
or repeated receipt blocks hide a current remaining gap. Reuse an existing
repository closeout gate when it fits; writing a new general checker is its own
bounded implementation task, not a requirement for every Plan.
