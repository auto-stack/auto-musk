---
name: auto-plan-merge
description: |
  Land a reviewed Plan, update canonical docs/specs knowledge, refresh the
  derived ledger, then archive and clean its worktree with recovery receipts.
  Use for "merge plan", "沉淀计划", or "/auto-plan:merge". Requires revision-bound
  review evidence; distinguishes completed delivery from shelving a Plan.
---

# /auto-plan:merge — Land, consolidate, and archive

Announce: "I'm using /auto-plan:merge to consolidate plan NNN."

Input: a reviewed Plan reference, or a retry of its recorded consolidation.
Outcome: canonical Specs and their ledger view reflect verified delivered work,
the Plan is archived, and temporary worktrees are safely cleaned.

## Authority and compatibility

- **`docs/specs/` is the authoritative source of current project knowledge.**
  Apply the reviewed add/modify/retire delta to these documents.
- The ledger's six sections are derived indexes, relations, and history.
  Current-knowledge items link to canonical Specs; historical review/report
  items link to durable evidence or the archived Plan. Never copy Plan chapters
  into a second independently maintained specification.
- Legacy ledger-only knowledge is not discarded. Reconcile relevant entries
  with code and approved intent, promote enduring content into module Specs
  through a reviewed delta, and preserve unrelated/history entries.
- Current `merge_plan` and `POST /api/plans/{seq}/merge` copy Plan chapters
  and archive immediately. **Do not use these combined operations for this
  workflow.** Use existing non-archiving Spec operations or a validated offline
  projection, followed by explicit archival after verification.
- These are skill instructions and Plan receipts. They do not implement a
  durable Runner, database transaction, or automatic backend retry.

## Gate and recovery baseline

1. Resolve the Plan, main checkout, recorded development branch/worktree,
   dependencies, and current repository rules. Read the review and open issues.
2. For an active Plan, require `reviewed` plus a `pass` for the current
   `plan_revision`, `reviewed_commit`, dependencies, and frozen Spec delta.
   Legacy active reviews without a verifiable baseline need
   [review](../auto-plan-review/SKILL.md); never fill missing evidence with an
   assumed pass. Archived requests use the recovery-only rule in step 5.
3. Ensure required tasks and acceptance criteria are complete. Unrelated open
   checkboxes in examples/history are not delivery blockers; actual unmet
   requirements or unapproved deferrals return to work/review.
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
| `prepared` | Reviewed baseline, canonical Spec diff, projection targets, expected source versions, delivery commit |
| `landed` | Default-branch commit and ancestry proving code and canonical Specs landed |
| `ledger_refreshed` | Target workspace/path, verified item IDs, source references, version/hash |
| `archived` | Archive path and delivered outcome |
| `cleaned` | Guard result and confirmed worktree/branch removal |

Before retrying, reconcile the receipt with disk, Git, and live ledger state.
A timeout or missing receipt is not proof that a side effect did not happen.
Complete only missing operations. Stop automatic retries when no new evidence
or changed condition makes progress possible.

## Prepare canonical Specs in the worktree

Use the Plan's existing worktree; all canonical document edits belong there.
If a legacy Plan's implementation was already landed and its worktree removed,
verify that landing and create a dedicated consolidation worktree under the
repository's naming rules. Never edit canonical Specs directly on main.

- Synchronize/reconcile relevant default-branch changes. Compare the reviewed
  code, dependencies, and Spec targets with the baseline. If they changed in a
  way that affects the review, return for re-review; resolve implementation
  conflicts through work. Preserve unrelated concurrent changes.
- Apply the approved delta to the affected module documents and their existing
  indexes. Preserve unrelated requirements and authored explanations. Record
  Plan provenance and reasons for retired rules.
- Inspect the resulting Spec diff against the frozen delta, code, and acceptance
  evidence. Check links and document structure. Any new requirement or altered
  contract goes back through revision/review.
- Prepare ledger entries from these documents using the current
  `SpecsDocument/SpecItem` schema in `backend/crates/musk/src/specs.rs`.
  Record each canonical target, section, stable item ID, and source hash.
- Commit the prepared changes. A documentation/projection-only descendant of
  `reviewed_commit` may become `delivery_commit` after checking that exact
  delta and confirming implementation/dependencies are unchanged. Otherwise
  the delivery commit needs renewed review.

Do not force-add ignored runtime data or overwrite existing workspace state.
For tracked ledger data, prepare and commit its derived changes in the worktree.
For runtime-only ledger data, prepare the intended projection in the worktree
and publish it to the identified workspace after canonical Specs land.

## Land and refresh the derived view

1. Require a clean committed worktree before landing. Surface unexpected dirty
   changes to the user; never discard or silently include them. Shared Plan
   bookkeeping on main is a separate authorized exception: preserve it and
   commit only the target Plan if needed for a clean landing.
2. Verify the absolute worktree path and run the repository's
   `wt-guard.sh`; it must report clean. For auto-musk the expected group is
   `D:/autostack/.wt/musk-NNN/auto-musk`, branch `plan-NNN-dev`.
   Check ownership rather than inferring it from a name.
3. Land the verified delivery commit on the actual default branch. If default
   advanced beyond the prepared integration base, reconcile in the worktree
   and refresh affected verification before landing.
4. Confirm ancestry and the expected canonical Spec contents on main. Run the
   appropriate integration/smoke checks so main is known-good. Record `landed`.
5. Publish/verify the derived ledger for the correct workspace. A live service
   uses its existing non-archiving operations; inspect their current schema and
   target workspace first. Respect any existing write-approval policy.
   An unavailable service can use an offline read-modify-write of the projection
   at its configured runtime path, with all other writers excluded. If safe
   publication cannot be established, record `blocked`; keep the Plan active.
   Tracked ledger file edits always go through the worktree and Git.

Projection rules:

- Reuse an existing current-knowledge item by canonical target and section.
  If absent, allocate a stable ID once and record it in the receipt. Retries
  reuse that mapping; different Plans updating the same target update the same
  current-knowledge item. Section order is not an identity.
- Current items' `file` points at `docs/specs/...`; content is a derived
  summary/reference. Review/report history preserves the Plan ID, reviewed
  revision/commit, and eventual `docs/plans/archived/...` path.
- Preserve unrelated entries, unknown compatible fields, and existing history.
  Update only affected projections; detect overlapping changes using the
  captured source and ledger versions/hashes.
- Use one writer. A read/check/write sequence is not a lock or transaction.
  For offline updates, validate a temporary complete JSON document and replace
  it atomically under exclusive access; never overwrite a changing live file.
- Maintain schema-required fields and valid statuses. Rebuild derived relations
  and version metadata through existing store logic when available. `related`
  is a computed reverse-link field, so it must not be the sole provenance store.
  Verify links, IDs, content, and schema by reading back the result.
- No Spec impact is valid when review justifies it. Do not create artificial
  knowledge or require the ledger's byte size/item count to grow.

## Archive last, then clean up

Only after landing and ledger verification succeed:

1. On the main checkout, move the shared Plan to `docs/plans/archived/` and
   set `status: archived`. Use `git mv` for a tracked Plan; preserve an
   untracked Plan through the appropriate file move. Do not use a lifecycle
   bypass that archives before the preceding checks.
2. Record `completion_kind: delivered` and the archive checkpoint in the
   receipt. Check that active/archive paths and frontmatter agree, and that
   provenance links resolve. Commit the target Plan's final bookkeeping.
3. Recheck the worktree is clean and all its commits are landed. Immediately
   before removal, verify the resolved absolute path is within its intended
   group and require a fresh `wt-guard.sh` clean result. Remove the worktree
   and its merged branch; remove only the empty group directory. Never create
   junctions/symlinks or bypass the guard.
4. Verify removal and record `cleaned` in the archived Plan; commit that
   receipt update. If cleanup fails, keep the delivered archive, report
   `blocked` with cleanup pending, and retry only that operation.
   Authorized dependency worktrees follow their own review/integration gates
   and should already have been folded/cleaned promptly.

A crash between the file move and status/receipt update is repaired from the
verified delivery evidence, without rerunning completed publication. Do not
claim cross-file or cross-repository atomicity.

## Report

Record/report `stage: merge`, Plan ID/revision, `outcome`, delivery commit,
canonical Spec paths, ledger targets, archive path, and cleanup state.

- `pass`: delivery, derived view, archival, and cleanup are all verified,
  or a repeat request confirms that complete result without duplicate writes.
- `needs_fix` / `needs_replan`: return an active Plan to `executing`,
  invalidate affected evidence, and route to work/new.
- `blocked`: preserve successful checkpoints and identify the exact remaining
  action. Before archival the Plan remains `reviewed` for a publication-only
  blocker; after archival it remains `archived` for recovery/cleanup.

Do not invoke unrelated close-out skills or migrate the entire knowledge base.
The current task's reviewed delta defines the consolidation scope.
