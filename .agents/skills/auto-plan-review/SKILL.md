---
name: auto-plan-review
description: |
  Independently verify a Plan against agreed acceptance criteria and actual
  code, bind evidence to revisions, and validate its proposed Spec delta.
  Use for "review plan", "复审计划", "/auto-plan:review", or work completion.
  Returns pass, needs_fix, needs_replan, or blocked; never merges code.
---

# /auto-plan:review — Verify the implementation and Spec delta

Announce: "I'm using /auto-plan:review to verify plan NNN."

Input: a Plan reference, optionally a named phase for an approved incremental
landing. Output: reproducible evidence and an unambiguous next action.

## Document-source contract (PLAN-106)

Current knowledge lives in docs/specs/ documents; archived Plans retain review and
delivery history. The application generates indexes automatically. Agents never
write specs.json or manually maintain index.json lists, and merge has no index or
ledger publication prerequisite. Missing/corrupt caches and an offline application
do not block source delivery. Read/list tools may expose a derived six-category
view; retired item-write tools must direct callers to canonical source files.


## Evidence retention (2026-10-30 user ruling; supersedes permanent exe retention)

- Keep durable Plan/review summaries (commands, exit codes, source/dependency identity,
  counts, limitations and evidence hashes) in version control. Raw evidence stays local.
- Raw evidence, including exe replay anchors, expires 14 days after final review and
  landing. Existing unclassified history gets one 14-day adoption window, not a new
  window every run. Hashes are identity records, not a promise of permanent replay.
- In-flight Plans and necessary evidence for unresolved issues are protected. Register
  a hold with exact scope/paths, reason, owner and next review date (within 14 days).
  Review dates trigger reassessment, never automatic deletion of an unresolved hold.
  Unknown ownership, frozen dependency sources and recovery material fail closed.
- Keep only necessary final executable roles; do not archive every build/test executable
  or entire worktree/target/node_modules trees. Promote enduring regressions/fixtures
  and unique dependency sources into their owning source repository before retiring copies.
- Default budgets: 500 MiB unique data per Plan and 10 GiB globally under
  D:/autostack/plan-artifacts. Check before producing more artifacts and at cleanup.
  Do not grow an over-budget scope without a named, reasoned, time-bounded exception;
  do not evict protected evidence to force a green budget. Report legacy excess honestly.
- Registry: D:/autostack/plan-artifacts/.retention/policy.json. Register owner/status,
  retention_started_at/expires_at, explicitly classified expire_files (size, mtime_ns and SHA-256) and holds. Refresh
  status/holds from actual Plans and active consumers before each maintenance run.
  Classify expendable logs/screenshots/probes explicitly; never expire arbitrary sources
  or SDK trees by extension/mtime. Preserve the first failure while an issue is open.
- Use auto-plan-merge/scripts/artifact_retention.py (relative to the installed skills
  root): preview first; --apply --deduplicate reclaims closed-scope Cargo caches/PDB/
  node_modules and expired listed files, and deduplicates full-SHA256-equal exe files.
  It skips active processes, unknown/held scopes and reparse points. Duplicate paths
  share read-only NTFS hardlinks, never symlinks/junctions; retained binaries are immutable.
  Rebuild into a fresh path; never overwrite an archived executable in place.
- Run maintenance at merge closeout and the daily retention follow-up. Record the
  receipt, file-data/storage-byte savings and any hold/budget/error. Exit 2 means budget excess,
  not test failure or permission to erase evidence. No full Rust gate is needed.

## Entry, authority, and independence

- Read the Plan from the main checkout. Read the implementation and diff in
  its existing worktree, plus relevant `docs/specs/`, tests, and repository
  rules. Module Specs are authoritative project knowledge; ledger entries are
  derived navigation/history, not competing requirements.
- Compare the approved intent, actual behavior, and proposed change. Trust code
  as evidence of what exists; do not treat a mismatch as permission to weaken
  the agreed requirement.
- Normal entry is `execution_done`. If `executing` claims all tasks done,
  verify that claim before treating it as ready. For high-risk or repaired work,
  reconcile the readiness record and finding closure table with the actual
  contract, code, and assertions; an internal pass or test total is not the gate.
  Known unfinished work-owned criteria or required oracles return to work,
  even if the receipt says execution_done. Record the concrete gap; do not
  spend another full final-review cycle merely to reconfirm admitted unfinished
  implementation. Explicit investigation or targeted review requests can still
  examine partial work, but cannot issue final pass. A re-review is appropriate
  on explicit request or when merge detects stale evidence. Do not reopen an
  archived Plan.
- Prefer an independent review session/context when available and authorized.
  A separate role/model is optional; do not claim independence merely from a
  role name. If reviewing in the implementation session, state that limitation
  and reconstruct the verdict from artifacts rather than the executor's summary.
  Do not create another task or agent without the applicable authorization.
- Identify the recorded worktree via `git worktree list --porcelain`; verify
  its resolved path, repository, branch, and handoff commit. Internal and final
  reviewers enter the same Plan tree used by work, with independent contexts
  where assigned. Do not create a reviewer-specific checkout or synchronize
  between model-specific trees. Set tool working directories explicitly.
- During review, implementation writes are paused. Inspect/test committed code
  and record findings; route repairs to work in the same tree. Read the current
  shared Plan on main, not a stale copy in the implementation tree.
  If an active, unlanded Plan's tree is missing, resolve ownership/recovery
  before issuing pass. For an already-landed historical review, verify commit
  ancestry and record the inspected checkout and historical baseline;
  absence alone does not prove landing. Code changes go back to work.

## Establish the review baseline

Resolve `review_scope: internal | final | phase` from the request and the Plan's
assigned roles; record it in the receipt. With no split-role workflow, the
ordinary independent acceptance review is final. With a designated final
reviewer, an internal pass leaves `execution_done` and routes to that reviewer;
it cannot grant `reviewed` or authorize merge. An internal non-pass returns the
affected work to `executing`. Phase passes keep the Plan `executing`.
Use a context independent of implementation for the assigned final review when
available and authorized; do not relabel an implementation-session self-check
as the separate final reviewer. Record actual reviewer/context identity when
exposed, and limitations otherwise; never fabricate a model or session ID.

1. Record Plan revision, full Git commit SHA, diff base, dependency revisions,
   and relevant Spec source versions/hashes. Migrate a legacy Plan's missing
   revision as described in [new](../auto-plan-new/SKILL.md); do not fabricate
   old approvals or evidence.
   For legacy tasks/criteria without stable IDs, assign IDs while preserving
   their meaning and progress; record that normalization in the review baseline.
2. Require reviewed implementation changes to be committed. Inventory dirty
   changes and route them for resolution; do not issue a pass tied only to HEAD
   while uncommitted implementation is being tested.
3. Check authorization for the actual scope and any plan revisions. Preserve
   acceptance IDs and prior review records.
4. Recheck HEAD, implementation cleanliness, dependencies, and verification
   configuration before recording the verdict. Unexpected drift makes affected
   evidence stale; reconcile it instead of passing the originally named commit
   against a changed tree. Record the worktree path and branch with the receipt.

Progress markers, timestamps, and this review record do not change
`plan_revision`. Changes to the semantic contract or reviewed implementation
invalidate the affected review. A previous pass never covers future edits
automatically.

## Verify coverage and behavior

Maintain an acceptance-ID evidence map with `pass / partial / fail`, command or
inspection method, result, and the revision/input covered. The initial review
checks all work-owned criteria; a repair re-review updates affected entries and
retains applicable evidence for the rest. It does not re-execute every AC merely
because this submission is called a final review.

### Repair re-review test scope

- Start from the prior verdict, current repair diff and closure rows. Reproduce
  the affected findings and accepted controls; independently inspect their
  shared invariant and choose representative sibling/failure cases where the
  changed mechanism warrants them. Record the `check → finding/invariant` scope.
- Do not require the executor to run global tests to become reviewable, and do
  not run them again yourself after each repair. This includes full standalone
  probe/workspace suites, every test family, and aggregate all/acceptance/closeout
  scripts. A module or integration expansion needs a concrete impact reason;
  a new HEAD, a high-risk label, or a request to re-review is insufficient.
- Reuse identified evidence when the relevant implementation, dependencies,
  verification configuration and inputs are unchanged. Re-run only checks
  invalidated by the repair or new evidence. Do not replay unrelated GC/scale,
  full performance matrices or already-closed compatibility families by default.
- If an in-contract blocker remains, finish the necessary targeted diagnosis
  and return its reproducible finding package. Do not spend a global regression
  run on code already known to need repair. Keep untouched AC evidence and the
  future unified gate separate from the affected failure.

### Final unified regression

Only after all work-owned features/findings are complete and the code is frozen
does the repository's designated final-review or merge owner run the unified
regression gate. Reuse a valid run on that same baseline; the next role does not
repeat it just because ownership changes. If it fails, diagnose the failure,
repair with targeted checks, re-review affected criteria, then rerun the gates
invalidated by that change on the new frozen baseline.

For auto-lang (PLAN-888 r3, 2026-10-30), finish final code, acceptance and Spec
review first. Review uses finding-specific replays and affected `cargo tm`
modules; it does not stack legacy t/tv/tt/tb/taa or repeat the full catalog.
Inspect the explicit module IDs or changed-path selection, complete leaves and
actual consumer/integration closure, source/dependency/config/input identities,
and original atomic outcomes. Final pass records the frozen affected module
union as pending merge-owned work, not a bare cargo t prerequisite. Reviewed
does not permit landing with an unmet gate. Valid evidence for unchanged inputs
may be reused; exclusions/pending and known reds remain explicit, not green.
`cargo tf --batch` is due-triggered on main as a single instance. Other
repositories retain their own gate commands and timing. Documentation-only
changes use content/link/format checks unless repository rules require more.

### Behavioral evidence

- For a multi-repository Plan, identify each repository's own checks and gate
  owner. Run its affected repair checks now and leave its unified gate to the
  assigned final stage; do not apply one repository's tier by analogy to another.
- Exercise user-visible/API behavior where required; source inspection alone
  does not demonstrate a runtime acceptance criterion.
- Verify test assertions and negative cases, not just exit codes. Separate
  baseline/environment failures from regressions, but never turn an unverified
  required criterion into a pass.

For the Plan's high-risk invariants, inspect the complete declared path/state
matrix and independently choose representative counterexamples, not only the
executor's regression inputs. Read
[verification-contract.md](../auto-plan-new/references/verification-contract.md)
when applicable. Cover the identified risk areas in the first review rather
than stopping at the first defect; mark unexamined areas pending with reasons.
New scope is a contract change, not an undisclosed acceptance requirement.
For document contracts, check rule consistency and dependency feasibility,
not only field or example counts.

On re-review, distinguish old-example closure, full-invariant closure, and
new repair regressions. Replay affected prior inputs and accepted controls
against the new baseline, with hashes when input identity matters. A changed
parser/traversal/executor needs representative combination or failure-exit
checks for its new paths; earlier helper evidence does not cover a replacement.
Keep counterexample input, expected behavior, command and actual result durable
and directly reusable by work. Describe the violated existing rule and coverage
boundary, so a new probe does not silently become a new scale promise. After
two failed final reviews, verify the diagnosis and coverage/design adjustment
from work before treating the next handoff as ready.

Check raw invocation/results against claimed case IDs and totals. For a
validator used as acceptance evidence, test meaningful valid and invalid
controls, actual mutation, and failure propagation; an ALL-PASS banner cannot
prove its own coverage. Required future merge checks remain recorded pending,
not claimed pass on the strength of a mock or a receipt template.

Map `AC IDs → task IDs → code/artifacts → evidence`. Look for missing
sub-items, configuration/call-site gaps, unapproved scope reductions,
deferrals, or workarounds. Findings need stable IDs, affected acceptance/tasks,
severity, evidence, and a concrete correction or investigation.
Give each repair finding its minimal replay/affected-check scope; leave the
Plan-wide regression pending instead of attaching it to every finding.
Use the shared reference's reproducible finding package: violated existing rule,
bound input/setup and command/entrypoint, expected versus observed values,
established cause versus hypothesis, affected sibling paths and legitimate
controls, and the closing oracle with its coverage boundary. Do not supply an
unverified implementation prescription as a confirmed root cause. A durable
replay and concise package serve work better than a longer narrative or more
test totals.

Before treating a failed probe as a product defect, verify that its input
reaches the claimed production path/state and that its expected result follows
the agreed contract. Keep an incorrect or environment-bound probe's raw record,
correct the probe, and classify it separately; do not weaken a valid assertion
or add a new product promise to make the probe pass.

Recording an unmet requirement as debt does not make it complete. Distinguish
nonblocking improvements outside the agreed scope from failures within it.
An in-contract defect cannot be downgraded merely because it is pre-existing,
rare, or described as nearly unreachable. Findings identify whether they are
an original requirement gap, contract clarification/change, new regression,
input drift, or closeout defect; this determines the repair and review scope.

## Review the knowledge delta

Read `### 规范增量` under the Plan's detailed design, or add it for a legacy
Plan using the verified implementation. For each add/modify/retire operation:

- Verify the repository-relative `docs/specs/...` target and the before/after
  rule, rationale, acceptance IDs, and current target version.
- Ensure the proposed text describes current behavior and enduring decisions,
  not abandoned implementation plans or an execution diary.
- Resolve conflicts with current canonical Specs through evidence and approved
  intent; concurrent changes require reconciliation, never blind Plan priority.
- Finalize `supersedes_spec_components`, `new_spec_components`, and
  `touched_goals`. Use exact `docs/specs/...` paths and real goal IDs.
  Empty impact requires a written explanation.
- Review any prepared Spec diff in the worktree. The application automatically derives its index
  from canonical documents and archived review/history artifacts; agents do not publish projections.

Do not publish canonical Specs or modify the live ledger during review.
If correcting the delta changes the semantic contract, increment its revision
and verify the resulting contract before issuing the final verdict.

## Record and route

Append to `9. 复审记录`:

`stage: review | review_scope | reviewer_identity/limitations | plan_id |
plan_revision | outcome | worktree_path | branch | reviewed_commit | base_commit |
dependency_revisions |
spec_inputs | acceptance_results | pending_closeout_gates | findings | evidence |
next`

Keep evidence paths resolvable after worktree removal: record durable repository
artifacts or concise command/result excerpts in the Plan, not only temporary logs.
The evidence package includes a frozen copy/hash of the reviewed Spec delta.

| Outcome | Plan state | Next action |
|---|---|---|
| final `pass` | `reviewed` | merge; required implementation/review criteria and delta passed, explicit merge-owned gates remain for closeout |
| internal `pass` | `execution_done` | designated final review; no merge/archive/cleanup |
| `needs_fix` | `executing` | work with the concrete findings and affected task IDs |
| `needs_replan` | `executing` | new to revise the invalid design or task contract |
| `blocked` | `executing` | Resolve the named prerequisite/decision, then resume the appropriate stage |

For any non-pass, reopen affected task checkboxes and their dependent evidence;
recompute `current_step` from the completed task count. Preserve unaffected
completed tasks and all historical findings. Never leave a failed review in
`execution_done` while asking work to repair it.

A phase-only review records its phase/commit verdict while the overall Plan
stays `executing`; it cannot grant final `reviewed` status. Phase landing
still requires its affected regression gate; for auto-lang this is the frozen
`cargo tm` module/consumer union after phase acceptance review, before landing,
not a new `cargo tf --batch` run or legacy-tier stack during each repair.

Do not merge or fix implementation within review. If the whole workflow is
already authorized, hand control to the appropriate skill and honor the repair
limit from [work](../auto-plan-work/SKILL.md); otherwise report the next action.
Any semantic fix changes the reviewed baseline: return to work, commit it,
and verify affected criteria on the resulting revision before a new final pass.
Do not cover code added after pass with a comment-only/Spec-only exception.
Outcome records are a skill contract, not new backend statuses or automatic
Relay routing.
