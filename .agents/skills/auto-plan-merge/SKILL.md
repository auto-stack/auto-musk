---
name: auto-plan-merge
description: |
  Land a reviewed Plan, verify canonical docs/specs knowledge, then archive and clean its worktree with recovery receipts.
  Use for "merge plan", "沉淀计划", or "/auto-plan:merge". Requires revision-bound
  review evidence; distinguishes completed delivery from shelving a Plan.
---

# /auto-plan:merge — Land, consolidate, and archive

Announce: "I'm using /auto-plan:merge to consolidate plan NNN."

Input: a reviewed Plan reference, or a retry of its recorded consolidation.
Outcome: canonical Specs reflect verified delivered work,
the Plan is archived, and temporary worktrees are safely cleaned.

## Evidence handling: plan-artifacts scratch, delete-at-merge (2026-10-31 rulings)

- `D:/autostack/plan-artifacts/plan-<NNN>/` is the SINGLE scratch home for
  everything Plan-related except the Plan file itself: probe sources/outputs,
  logs, gate/ledger dumps, screenshots, repair/review/merge records, receipts/
  handoffs, and any build products or copies worth keeping while the Plan is
  in flight. Lightweight no-plan tasks use their task slug
  (e.g. `plan-artifacts/fix-<slug>/`). Create the directory when work starts;
  an in-flight Plan's scope is protected. Nothing evidence-like lives inside
  any repository checkout: `docs/plans/attachments/` is fully git-ignored in
  auto-lang/auto-musk/auto-down (nothing tracked or pushed, `*.md` included)
  and must not be recreated; the same-day `.wt/<group>/evidence/` convention
  is retired. Worktree-less docs-only work uses the OS temp dir, never the repo.
- The Plan file is the only durable evidence home. At closeout, consolidate
  ALL Plan-related evidence into concise markdown inside the Plan file itself:
  commands, exit codes, key numbers, short code/log excerpts, and local-path
  plus hash for anything kept, under the Plan's verification and review-record
  sections.
- Delete at merge (2026-10-31 user ruling, superseding the 2026-10-30
  latest-five window and its 500 MiB/10 GiB monitoring tiers and daily 03:00
  maintenance): after folding, delete the Plan's whole plan-artifacts scope.
  Build intermediates (cargo `target` trees, `.pdb`, exe copies) are NOT
  reclaimed — they die with the worktree. Only a named unresolved-issue hold
  (owner + reason + closure condition recorded in the Plan and the registry)
  may keep a local scope. Cleanup is self-service at merge closeout plus
  opportunistic orphan sweeping; unknown/unregistered scopes still fail
  closed — verify ownership and status, report, never blind-delete.
- Registry: `D:/autostack/plan-artifacts/.retention/policy.json` stays as the
  index of what exists on disk and why (in-flight scopes, named holds).
  Refresh it at merge closeout: record scope deletions and holds with
  owner/reason/closure condition. Archived status alone never proves issues
  are resolved; a hold is not released just because time passed — check the
  Plan and current debt records. `auto-plan-merge/scripts/artifact_retention.py`
  remains available as a conservative manual sweep/preview tool for legacy
  orphans; its latest-five logic no longer defines policy.
- Rationale: historical Plans rarely need their original evidence again —
  later verification re-runs against current code. Dual scratch homes plus a
  retention window accumulated 67 GB of build intermediates while in-repo
  attachment directories leaked 8916 files into Git history (untracked
  2026-10-31). Enforcement backstop: repo gitignore plus auto-lang's
  pre-commit reject any staging under `docs/plans/attachments/`; the escape
  hatch `AUTO_LANG_ALLOW_EVIDENCE=1` still requires a recorded reason.

## Authority and compatibility

- **`docs/specs/` is the authoritative source of current project knowledge.**
  Apply the reviewed add/modify/retire delta to these documents.
- The ledger's six sections are derived indexes, relations, and history.
Current-knowledge items link to canonical Specs; historical review/report
items link to the archived Plan (closure folds evidence summaries into the
Plan file itself). Never copy Plan chapters
into a second independently maintained specification.
- Legacy ledger-only knowledge is not discarded. Reconcile relevant entries
  with code and approved intent, promote enduring content into module Specs
  through a reviewed delta, and preserve unrelated/history entries.
- Current `merge_plan` and `POST /api/plans/{seq}/merge` copy Plan chapters
  and archive immediately. **Do not use these combined operations for this
  workflow.** Land the reviewed canonical documents, verify source identity,
  then explicitly archive. Combined chapter-copy operations are retired.
- These are skill instructions and Plan receipts. They do not implement a
  durable Runner, database transaction, or automatic backend retry.

## Document-source policy (PLAN-106)

`docs/specs/` is the only current-knowledge authority. Agents maintain the reviewed
canonical documents in the Plan worktree and land them through Git. The application
automatically derives navigation, classifications, relations and history from those
files and archived Plan receipts. No agent publishes an index or writes specs.json.

Cache/index existence, application availability, and legacy ledger parseability do
not gate delivery. A missing or failed index is an application diagnostic and is
rebuildable from sources; do not replace ledger_refreshed with a mandatory
index_refreshed checkpoint. There are no section entry quotas or offline JSON
projection steps. Never copy Plan chapters into a second specification.

Legacy checkpoints remain historical facts. A landed Plan can resume canonical
verification and archive without ledger_refreshed; a ledger_refreshed record alone
never proves landing. Preserve old receipts and verify actual Git ancestry and
canonical source identity. New source changes still need normal review.

Legacy per-project ledgers, including `.autoos/specs.json` and
`backend/.autoos/specs.json`, are read-only migration inputs. During ordinary
Plan delivery, never create, append, rewrite, upsert, or sync them, directly or
through a helper/API. Historical Plans, old section quotas, receipt retries, and
offline fallbacks do not authorize these writes; maintain canonical AutoDown
documents in `docs/specs/` and let the application derive its indexes.

## Gate and recovery baseline

1. Resolve the Plan, main checkout, recorded development branch/worktree,
   dependencies, and current repository rules. Read the review and open issues.
2. For an active Plan, require `reviewed` plus a `pass` for the current
   `plan_revision`, `reviewed_commit`, dependencies, and frozen Spec delta.
   Legacy active reviews without a verifiable baseline need
   [review](../auto-plan-review/SKILL.md); never fill missing evidence with an
   assumed pass. Archived requests use the recovery-only rule in step 5.
   If the Plan assigns internal and final reviewers, require the assigned final
   review, not an internal pass. Never land/archive while waiting for it.
3. Ensure implementation/review tasks and criteria are complete. Only explicitly
   merge-owned operations/gates may remain pending; perform them before claiming
   delivered pass. Unrelated checkboxes in examples/history are not blockers;
   unmet implementation requirements or unapproved deferrals return to work/review.
4. Verify reviewed commits against the real repository. A missing worktree is
   not proof of landing; inspect ancestry and previous receipts.
5. An archived Plan is eligible only for a verified no-op or finishing a
   previously evidenced delivery interrupted during archival/cleanup. Verify
   historical delivery against its recorded commits/receipts, not by requiring
   today's Specs to equal that old snapshot. Never reapply an old delta over
   newer canonical knowledge. A shelved Plan or new feature must not be reopened
   or silently re-merged. If delivery evidence is missing or inconsistent,
   report blocked for reconciliation rather than routing an archive into work.

Record a consolidation receipt under `9. 复审记录`, keyed by
`PLAN-NNN:r<plan_revision>`. Track the following operations with actual
commit/hash/path evidence; do not mark an operation done before verifying it:

| Checkpoint | Evidence |
|---|---|
| `prepared` | Reviewed baseline, canonical Spec diff, source targets, expected source versions, delivery commit |
| `landed` | Default-branch tip equal to the delivery commit after a `--ff-only` merge (no merge commit); old→new rebase hash mapping with `git range-diff` equivalence when landing rewrote the branch |
| `canonical_verified` | Landed canonical paths, source hashes and delivery commit; approved no-impact delta when applicable. This is source verification, not an index publication. |
| `archived` | Archive path and delivered outcome, **plus the plan/docs artifacts committed to the default branch (untracked docs/ = not archived; PLAN-104 UAT-K13)** |
| `cleaned` | Guard result and confirmed worktree/branch removal; build intermediates (cargo `target` trees, `.pdb`, exe copies) NOT reclaimed — deleted with the worktree; **the Plan's whole `plan-artifacts/plan-<NNN>/` scope deleted after its records were folded into the Plan file, registry refreshed, only named unresolved-issue holds kept (2026-10-31 delete-at-merge ruling)** |

**Version-control closing is part of the flow, not an afterthought:** the
demo-workspace UAT of 2026-10-11 (K13) landed code ff-only but left
`docs/plans/**` and `.autoos/**` untracked with no receipts commit — the
plan file was "archived on disk" yet outside version control, and the
linear-history final check failed. Before recording `archived`/`cleaned`:
commit docs/ artifacts to the default branch, verify `git status` clean, and
confirm the expected linear history segments.

Before retrying, reconcile the receipt with disk, Git, and canonical source state.
A timeout or missing receipt is not proof that a side effect did not happen.
Complete only missing operations. Stop automatic retries when no new evidence
or changed condition makes progress possible.

## Prepare canonical Specs in the worktree

Use the Plan's existing worktree, shared by work and the designated review roles;
all canonical document edits belong there. Verify the handoff path/branch and
reviewed baseline before taking write ownership. Do not create a model-specific
merge copy. Retain this tree through final review and verified closeout;
cleanup ends its lifetime.
If a legacy Plan's implementation was already landed and its worktree removed,
verify that landing and create a dedicated consolidation worktree under the
repository's naming rules. Never edit canonical Specs directly on main.

- Synchronize/reconcile relevant default-branch changes. Compare the reviewed
  code, dependencies, and Spec targets with the baseline. If they changed in a
  way that affects the review, return for re-review; resolve implementation
  conflicts through work. Preserve unrelated concurrent changes.
- Guard the delta's canonical target: any spec-delta landing outside
  `docs/specs/` (including empty spec-components justified by a "knowledge
  lives elsewhere" note — module README, ledger-only, etc.) requires a
  recorded, reviewed precedent for that knowledge base in this repo. No
  precedent → stop and get a one-time user confirmation, then record the
  convention (e.g. in `docs/specs/reviews`); an inherited "same-as-NNN" note
  alone is not confirmation. Lesson (auto-edit, 2026-09-22): PLAN-002 silently
  established a README-as-spec convention that five successor plans inherited
  unchecked, leaving `docs/specs/` and the structured sections unbuilt.
- Apply the approved delta to the affected module documents and authored
  navigation links. Leave generated indexes to the application. Preserve
  unrelated requirements and authored explanations. Record Plan provenance and
  reasons for retired rules.
- Inspect the resulting Spec diff against the frozen delta, code, and acceptance
  evidence. Check links and document structure. Any new requirement or altered
  contract goes back through revision/review.
- Commit the prepared changes. A documentation/projection-only descendant of
  `reviewed_commit` may become `delivery_commit` after checking that exact
  delta and confirming implementation/dependencies are unchanged. Otherwise
  the delivery commit needs renewed review.

Do not force-add ignored runtime data. Indexes are application-owned and optional;
no cache/projection draft or publication belongs in the merge worktree.

## Auto-lang post-review fixed-group gate (PLAN-896/898)

Merge owns the frozen affected fixed-group gate after final code/acceptance/Spec
review. Use `cargo t <group> ...` to run complete affected groups plus their
necessary integration groups once. Inside the Plan worktree, inspect and
execute explicit group names:

```text
cargo t --list
cargo t <group> --select-only
cargo t <group> ...
```

Review the generated selection; unmapped concerns must not disappear.
Record the actual selected groups, source/dependency/config/input identities
and execution receipt. A select-only plan is not an execution pass.

Do not run a whole catalog to commit, repair or resubmit review. Reuse valid
single-module/targeted evidence when the relevant mechanism, members, inputs,
dependencies and configuration are unchanged. New HEAD alone is not a reason
to repeat it. Modules aim for <=60s; preparation/cold builds are separate. Do
not demand three global warm rounds, a global P90 or an A/B/C matrix. Preserve
original per-case formal samples, budgets and oracles. Long/conditional/pending
and known-red outcomes remain explicit; module ownership is not approval for
daily admission or silent disabling. Pure docs/assets use scoped content checks.

The periodic `cargo t all` batch is the due-triggered, main-checkout
single-instance gate (PLAN-898). UI/Man owner legs keep their own queues; do
not independently rerun its language coverage. The legacy t/tv/tt/tb/th aliases
are retired and deleted (PLAN-898). Other repositories retain their own commands and gate timing.

Bind each result to the actual commit, dependencies, verification configuration
and inputs; record pending/passed/failed gates in the Plan. A review pass with
pending merge-owned gates cannot authorize landing or a delivered claim. On a
failure, diagnose it without silently waiving a required gate; code fixes return
to work and affected re-review before rerunning affected gates on the new frozen
baseline. Reuse prior evidence only with an explicit unchanged-input/baseline
justification. Rebase or later edits that affect the tested baseline invalidate
that evidence; check the final delivery identity before landing.

## Land and verify canonical sources

Before landing, inspect the Plan-owned diff and the side effects of its helpers,
including ignored/untracked outputs it created. A Plan-owned legacy-ledger write
is `needs_fix`: return to work to remove that write path and repair only the
Plan-owned change before renewed review. Never restore or overwrite another
writer's ledger data. Unrelated pre-existing ledger drift is not a canonical
source-delivery gate; missing or malformed old JSON must not trigger creation,
repair, synchronization, or an offline write-back.

1. Require a clean committed worktree before landing. Surface unexpected dirty
   changes to the user; never discard or silently include them. Shared Plan
   bookkeeping on main is a separate authorized exception: preserve it and
   commit only the target Plan if needed for a clean landing.
2. Verify the absolute worktree path and run the repository's
   `wt-guard.sh`; it must report clean. For auto-musk the expected group is
   `D:/autostack/.wt/musk-NNN/auto-musk`, branch `plan-NNN-dev`.
   Check ownership rather than inferring it from a name.
3. Land the verified delivery commit on the actual default branch with linear
   history. In the worktree, rebase the development branch onto the default
   branch (`git rebase <default>`), resolving conflicts in the worktree; if
   conflicts touch code or Spec targets, refresh the affected verification on
   the rebased state before landing. Development branches are local and
   temporary (the remote carries only the default branch), so rewriting them
   is safe; the default branch history is never rewritten. When the rebase
   changes hashes bound by review evidence or receipts, record the old→new
   mapping and prove patch equivalence with `git range-diff` (a fully equal
   range-diff is the safe-rewrite proof); the rebased tip becomes the
   delivery commit. Then in the main checkout land with
   `git merge --ff-only <dev-branch>`: no merge commit may be created, and a
   refused fast-forward means the rebase was skipped or incomplete — stop and
   reconcile instead of falling back to a plain merge. With `git rerere`
   enabled, replayed commits reuse recorded conflict resolutions.
4. Confirm the default-branch tip equals the landed delivery commit and the
   expected canonical Spec contents are on main. Run the appropriate
   integration/smoke checks so main is known-good. Record `landed`.
5. Verify the reviewed Spec delta is present in the landed commit (or record its
   approved no-impact conclusion), and record canonical paths/source hashes.
   Proceed to archive without any application/index/legacy-ledger write. Optional
   index diagnostics must not change the delivery outcome.

## Archive last, then clean up

Only after landing and canonical-source verification succeed:

1. On the main checkout, move the shared Plan to `docs/plans/archived/` and
   set `status: archived`. Use `git mv` for a tracked Plan; preserve an
   untracked Plan through the appropriate file move. Do not use a lifecycle
   bypass that archives before the preceding checks.
2. Verify the actual archived state before recording the archive checkpoint:
   task IDs and counts agree, delivery/review/archival effects are verified,
   remaining cleanup work is explicitly pending, state and location match,
   links resolve from the real parent, required affected
   README/module references are present, and navigation/history targets identify
   this Plan rather than any existing file. Canonical knowledge entries still
   point to their Specs. Use the repo's reusable gate when available; otherwise
   record direct checks, not an unverified one-off checker. A pre-archive
   simulation is useful but does not replace this actual-state check.
   Record `completion_kind: delivered`, the verified checkpoint and results;
   commit final bookkeeping. If this gate fails, keep the worktree for closeout
   repair, record the pending recovery checkpoint, and do not claim pass or
   clean up. Do not reopen passed implementation for a document-only repair.
3. Recheck the worktree is clean and all its commits are landed. Immediately
   before removal, verify the resolved absolute path is within its intended
   group and require a fresh `wt-guard.sh` clean result. Remove the worktree
   and its merged branch — build intermediates die with it, nothing is
   reclaimed into plan-artifacts — then remove only the empty group directory.
   Never create junctions/symlinks or bypass the guard.
   After worktree removal, fold any remaining evidence records into the Plan
   file, delete the Plan's whole `plan-artifacts/plan-<NNN>/` scope, and
   refresh the registry (final archived Plan/owner, completion time, deletion
   or named unresolved-issue hold with owner + reason + closure condition).
   Unregistered or unknown scopes fail closed: verify and report, never
   blind-delete. This deletes-at-merge (2026-10-31 ruling) and replaces the
   2026-10-30 latest-five window, the daily 03:00 maintenance run, and the
   2026-10-27 instruction to keep every exe and other artifact.

4. Verify removal, complete its owned tasks, recompute final counts, and record
   `cleaned` in the archived Plan. Verify the final task/count/reference state
   before committing that receipt update or reporting merge pass. If cleanup
   fails, leave its tasks pending, keep the delivered archive, report
   `blocked` with cleanup pending, and retry only that operation.
   Authorized dependency worktrees follow their own review/integration gates
   and should already have been folded/cleaned promptly.
5. **Landing is not deployment.** After `cleaned`, verify the
   production-facing artifacts that consume the landed code are current:
   the backend release binary (`cargo build --release`), release binaries of
   dependency repos the Plan touched (e.g. the daemon), and the generated web
   bundle (`auto build` → `gen/front/vue/dist`). A code-only merge leaves all
   three stale while receipts still read delivered. Either rebuild and restart
   the production processes now (check connections first; follow the repo's
   shared-daemon restart protocol), or record an explicit observation item
   naming each stale artifact and its build date in the merge receipt. Lesson
   (auto-musk PLAN-092, 2026-09-27): all three artifacts predated the Plan's
   fixes and the receipt registered only one of them; the defects persisted
   on the running desktop until a later session rebuilt and restarted.
6. **Batch regression due check (auto-lang, fix-test-tiering 2026-09-30).**
   After `cleaned` (or after the artifact check above), read
   `docs/plans/.last-batch-regression.json` on the default branch (missing
   file = due). Due when a plan id divisible by 5 landed after the receipt's
   `last_covered_plan_id`, or the receipt is older than 48h with any merge
   (plan or L0 `fix-*`) after it. When due, hand over to
   [regress](../auto-plan-regress/SKILL.md) to run the batch full gate on the
   main checkout — single instance, never inside plan worktrees. A red batch
   gate routes to a fix-forward plan; it never unmerges.

A crash between the file move and status/receipt update is repaired from the
verified delivery evidence, without rerunning completed publication. Do not
claim cross-file or cross-repository atomicity.

## Report

Record/report `stage: merge`, Plan ID/revision, `outcome`, delivery commit,
canonical Spec paths/source hashes, archive path, and cleanup state.

- `pass`: delivery, canonical sources, archival, and cleanup are all verified,
  or a repeat request confirms that complete result without duplicate writes.
- `needs_fix` / `needs_replan`: return an active Plan to `executing`,
  invalidate affected evidence, and route to work/new.
- `blocked`: preserve successful checkpoints and identify the exact remaining
  action. Before archival the Plan remains `reviewed` for a publication-only
  blocker; after archival it remains `archived` for recovery/cleanup.

Do not invoke unrelated close-out skills or migrate the entire knowledge base.
The current task's reviewed delta defines the consolidation scope.
