---
name: auto-plan-new
description: |
  Create or revise an implementation Plan grounded in the current module Specs
  and relevant code. Use for "new plan", "新建计划", "/auto-plan:new", or an
  explicit plan revision after needs_replan. Produces a reviewable execution
  contract; does not implement the change.
---

# /auto-plan:new — Create or revise an execution contract

Announce: "I'm using /auto-plan:new to draft or revise the plan."

Input: a requirement, or an existing Plan reference and the reason for revision.
Output: one Plan in `docs/plans/`, with verifiable outcomes and an explicit
handoff to [work](../auto-plan-work/SKILL.md).

## Document-source contract (PLAN-106)

Current knowledge lives in docs/specs/ documents; archived Plans retain review and
delivery history. The application generates indexes automatically. Agents never
write specs.json or manually maintain index.json lists, and merge has no index or
ledger publication prerequisite. Missing/corrupt caches and an offline application
do not block source delivery. Read/list tools may expose a derived six-category
view; retired item-write tools must direct callers to canonical source files.

Legacy per-project ledgers, including `.autoos/specs.json` and
`backend/.autoos/specs.json`, are read-only migration inputs. During ordinary
Plan delivery, never create, append, rewrite, upsert, or sync them, directly or
through a helper/API. Historical Plans, old section quotas, receipt retries, and
offline fallbacks do not authorize these writes; maintain canonical AutoDown
documents in `docs/specs/` and let the application derive its indexes.

## Authority and context

- **`docs/specs/` is authoritative for current agreed project knowledge.**
  Module Specs describe behavior, interfaces, constraints, and design decisions.
  The six-section ledger is a derived index, relation graph, and historical
  view; use it to locate sources, not to override them.
- The Plan is the primary context and proposed change contract. Code and tests
  establish observed behavior. A conflict between code and an agreed requirement
  is a discrepancy to resolve, not permission to rewrite the requirement.
- Start with the Spec overview/index if present, then read only relevant module
  Specs, code, tests, and repository instructions. Record source paths and
  relevant versions or content hashes under background analysis.
- Read other Plans only to identify an existing active change, resolve a
  dependency, or trace a cited decision. Avoid loading unrelated history.
- If Specs are absent or stale, record that explicitly and investigate the code.
  Put the required Spec addition/correction in the proposed delta.

## Locate or allocate the Plan

1. Resolve the main checkout and inspect active Plan metadata for an existing
   matching requirement. Reuse a confirmed match; do not overwrite another
   draft on the strength of a similar title. Preserve progress and review history.
2. For a new Plan, scan both `docs/plans/` and `docs/plans/archived/`,
   taking the maximum valid numeric prefix plus one. Use `NNN-slug.md`.
   The current backend reads a three-digit prefix: if the next number exceeds
   999, report that compatibility blocker rather than silently truncating it.
3. Allocation must have one writer. Use an available exclusive allocation
   mechanism, or serialize Plan creation across the agent and application.
   Recheck IDs across both directories immediately before creation; create
   without overwriting and verify the resulting ID is unique. A max-plus-one
   scan, including the current API, is not by itself concurrency-safe.
4. Plan drafts and progress live on the main checkout, per `AGENTS.md`.
   This skill does not edit implementation files or canonical Specs.

## Draft the contract

Use the existing numbered sections for parser compatibility:

`0. 变更摘要`, `1. 目标`, `2. 架构方案`, `3. 技术栈`,
`4. 需求分析与背景调查`, `5. 详细设计`, `6. 测试设计`,
`7. 验收标准`, `8. 执行步骤`, `9. 复审记录`, `10. 待澄清事项`.

Keep these frontmatter fields; values below illustrate the shape:

```yaml
plan_id: PLAN-042
status: drafting
feature_name: Example change
author: [agent]
created_at: 2026-09-09T00:00:00Z
updated_at: 2026-09-09T00:00:00Z
plan_revision: 1
current_step: 0
total_steps: 3
supersedes_spec_components: []
new_spec_components: []
touched_goals: []
```

- State the goal, non-goals, affected repositories/modules, constraints,
  dependencies, assumptions, and what success looks like.
- Give acceptance criteria stable IDs such as `AC-01`. Each states observable
  behavior and a concrete verification method, including expected results.
- Give executable tasks stable IDs such as `T-01`, dependencies, affected
  files or symbols verified against the repository, the intended outcome,
  linked acceptance IDs, and verification commands with expected results.
  Identify genuinely new paths as new. Give tasks and acceptance criteria an
  `owner_stage` in the Markdown contract: `work`, `review`, or `merge`.
  Future review/closeout operations remain unchecked until actually performed;
  they are not prerequisites for handing completed implementation to review.
  Merge-owned criteria describe closeout effects; do not assign implementation
  correctness to merge to bypass final acceptance review.
- Size tasks by independently verifiable outcomes. The former 2–5 minute rule
  is a heuristic, not a gate. Detail the next executable work; do not invent
  exact implementation mechanics for unresolved research. Use a bounded
  investigation task with a decision artifact when needed.
- Under `5. 详细设计`, include a `### 规范增量` table:
  `delta_id | add/modify/retire | docs/specs/... target | before/after rule |
  rationale | acceptance IDs`. Use stable IDs such as `SD-01`.
  Populate known Spec-impact frontmatter entries provisionally, with
  repository-relative `docs/specs/...` paths; review finalizes them.
  A change with no Spec impact must explain why.
- Under `4. 需求分析与背景调查`, record the authorization already given:
  approved scope/revision, allowed repositories/actions, and any user-specified
  budget or automatic continuation limits. Do not invent approvals or budgets.
  Unknown constraints that affect correctness go to `10. 待澄清事项`.

For auto-lang (PLAN-888 r3, 2026-10-30), draft verification around `cargo t <fixed-group>`
complete affected groups and necessary integration groups. Name existing
group IDs from `cargo t --list`, and mark proposed tests as unexecuted.
Work/review own targeted counterexamples; merge owns the frozen module union
after final review. Do not prescribe stacked whole-repo rounds or three
global passes. The periodic `cargo t all` batch is due-triggered work on main.
Keep original oracles, samples and conditions; module registration is not
daily-admission approval. Other repositories retain their own commands.

## Verification depth and review roles

For compiler/IR contracts, trusted validation, resource lifetimes, publication
transactions, or a rule shared by multiple consumers, read
[verification-contract.md](references/verification-contract.md). Under section
6, state the invariant, representative valid/invalid inputs, verification
entrypoint, observable oracle, applicable paths/states, and coverage boundary.
Choose cases that distinguish a plausible incorrect implementation; a green
test family alone is not the oracle. Ordinary low-impact changes need only
their scoped checks, not a mandatory matrix or a new checker.

Before handoff, prepare the next task's execution entry in sections 5–8:
baseline/source references, dependency-ready task IDs, relevant closed/open work,
and decisions that block it. For high-risk work, use the reference's compact
contract rows to connect data flow and ownership to real producers/consumers,
representative states, observable results, and executable closing conditions.
Reuse existing tables and IDs; do not add a competing checklist or dump the
entire repair history into the entry. These details travel in the Plan so an
executor using another tool/model need not have these local skills installed.
For a simple task, its existing task row suffices; no separate brief is required.

Preflight those rows for contradictions, unowned paths and undefined results.
Mark which path facts are observed and which mechanisms remain hypotheses.
Resolve consequential uncertainty or assign a bounded investigation with a
decision artifact before dependent implementation; let work choose equivalent
local mechanics. Proposed fixtures are not executed evidence: identify the
task that makes them runnable. A required decision is distinct from renewed
approval; inherit authorization for unchanged scope.

For applicable high-risk Plans, give required supplemental cases stable IDs
and bind each to an AC, work owner, actual entrypoint, distinguishing assertion,
and coverage boundary. A fixed corpus does not replace required supplemental
oracles. Validate proposed expected results against the prose contract; resolve
conflicts before implementation rather than letting a fixture weaken a rule.

Resolve material resource-support decisions before dependent implementation:
which inputs are supported, what resource limits are approved, and how resource
rejection differs from malformed syntax or runtime failure. If the implementation
strategy is uncertain, require a bounded whole-path feasibility probe first.
No fixed syntax limit does not promise infinite resources or arbitrary-scale
performance. See the reference for traversal, lifetime, and publication examples.

Record user-assigned implementation, internal-check, and final-review roles in
section 6. When internal and final reviewers differ, internal pass only means
ready for final review; final review precedes merge/archive/cleanup. Do not
prescribe model names globally or create another agent/chat without authority.
Record the intended implementation worktree path and branch in the Markdown
handoff; work confirms their actual repository/path/commit once the tree exists.
There is one authoritative implementation worktree per Plan per repository.
Implementation, internal check, and final review use that same tree in sequence;
independent review means independent context, not a second checkout synchronized
through Git. Cross-repository work retains one sibling tree per repository.
These are Markdown workflow instructions, not new backend fields or statuses.

## Revisions and authorization

`plan_revision` identifies the semantic contract: goals, constraints, design,
tasks, acceptance criteria, and Spec delta. Increment it when those change;
progress ticks, evidence, timestamps, and merge receipts do not increment it.

For legacy Plans, retain their sections and IDs where possible. Establish
revision 1 when first maintaining the contract, recording the baseline and any
actual changes. Missing revision or approval fields are not evidence of approval.

| Change | Handling |
|---|---|
| Path/symbol correction or equivalent implementation within agreed scope | Record reason and evidence, increment revision if contract text changes, inherit existing scope authorization |
| Design no longer works, but goal and acceptance remain unchanged | Revise the affected tasks/design from evidence; retain completed work and existing authorization where it covers the revision |
| Changed goal, acceptance threshold, compatibility promise, repository/action scope, or budget beyond authorization | Present the specific change for user decision before dependent execution |

Never remove acceptance criteria or silently defer work to make a Plan pass.
Mark affected prior verification as stale; preserve it as history.
Keep sections 5–8 as the single current contract, with one authoritative task
row per ID. Preserve superseded wording, findings, and receipts in section 9,
Git revisions, or linked durable reports; do not append competing current task
lists or leave historical "current/pending" prose as live instructions.

For a draft, keep `drafting`. A revision of an in-progress Plan stays
`executing` and records any pending decision explicitly; it is not reset to
a fresh Plan. Return `execution_done` or `reviewed` work to `executing` before a
semantic revision. Do not reopen archived Plans for new requirements.

## Handoff

Check that tasks cover all acceptance criteria and Spec deltas, paths and
commands are grounded, and unresolved assumptions have an owner/next action.

Show the concrete Plan and any decisions still needed. Honor existing
authorization; do not ask again for an already authorized scope. This skill
hands off rather than implementing.

Record a short result under `9. 复审记录` (draft/revision handoff) and report it:

- `stage: new`, Plan ID and revision.
- `outcome: pass` when ready for work within recorded authorization;
  `blocked` when a required decision or prerequisite is missing.
- `next: work` or the precise unblock action, plus changed task/acceptance IDs.

These are skill-level handoff records, not new Plan statuses or a claim that
the current Relay driver parses them. Historical design rationale is in
`docs/designs/008-auto-plan.md`; this contract governs the updated skill.
