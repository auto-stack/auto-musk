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

## Ledger refresh (writer regimes)

The spec ledger (`<workspace>/.autoos/specs.json`, workspace-scoped) is a
derived view owned by the application's store. The sanctioned write channel
is a property of the executing session, not of the ledger — determine the
regime first, and a channel requirement from the other regime neither
applies nor blocks (2026-10-12 user ruling; auto-lang PLAN-764 lesson: a
ZCode session deadlocked on `blocked` because the skill demanded auto-ai
in-session tools such a session can never have).

**Regime A — auto-ai / AutoMusk framework sessions** (the session's tool
list carries `read_specs`/`list_specs`/`write_spec`/`update_spec`; PLAN-104
whitelist. Dispatched errand agents get the same workspace-bound
overrides). Three sanctioned channels, all routing into the same workspace
store:

1. **In-session spec tools** — `write_spec` / `update_spec` (chat sessions;
   workspace-bound since PLAN-030's override + PLAN-104's whitelist fix).
   Prefer this channel.
2. **HTTP equivalent** — `POST /api/specs/item` with
   `{"section", "item": {id, title, content, status, file?, tags?}}` and
   `POST /api/specs/transition` for status edges. `file`/`tags` are supported
   since PLAN-104 (use them to record the source hash and commit).
3. **Dispatched errand** — a `dispatch` sub-agent holds the same
   workspace-bound spec tools (PLAN-104); usable when the ledger step is
   delegated.

In this regime never write that file directly with scripts, text
replacement, or a hand-assembled JSON document.

**Regime B — external mature agents** (ZCode, Codex, Claude-style CLI
editors, …) that cannot register auto-ai spec tools in-session. Work down
the ladder:

1. **HTTP first when the store service is up** — channel 2 above is plain
   HTTP any agent can call, and a live store owns the file, so prefer it
   over touching the file directly.
2. **Guarded offline projection** when no store writer is reachable at
   all. This is a sanctioned completion, not `blocked` (auto-lang PLAN-720
   `beae54b07` precedent, tightened):
   - Take a byte-exact backup of `specs.json` before any edit; restore
     from it whenever a guard below fails.
   - Edit programmatically (scripted JSON read-modify-write). Never
     hand-type the document; never delete-and-recreate the file.
   - Surgical scope: touch only the affected sections/items; everything
     else — including top-level fields — must stay byte-identical to the
     backup (a diff proves zero disturbance), and `version` increments by
     exactly one when the document carries it.
   - Validate against the store schema afterwards: six sections with ids
     exactly `goals`/`architecture`/`designs`/`tests`/`reviews`/`reports`
     (no `plans` section — retired in PLAN-024); items complete the same
     fields `SpecItem::new` would set (see
     `backend/crates/musk/src/specs.rs`) and carry `file`/`tags`.
   - Read the result back and verify IDs, content, and links; name the
     channel honestly in the receipt ("guarded offline projection, no
     store writer reachable") — never claim store success.

**Both regimes**:

- **Schema is store-owned**: `SpecsDocument{project, version, sections[6]}`.
  Section ids are exactly `goals` / `architecture` / `designs` / `tests` /
  `reviews` / `reports`. There is no `plans` section (retired in PLAN-024 —
  anything written there is silently dropped on the next load). Item fields
  are completed by `SpecItem::new` inside the tools; Regime B mirrors the
  same field completion per `backend/crates/musk/src/specs.rs`. Do not
  hand-assemble JSON in either regime.
- **Section mapping follows knowledge kind**: goals → `goals`, architecture
  decisions → `architecture`, module/design contracts → `designs`, test
  coverage → `tests`, review conclusions → `reviews`, deposit reports →
  `reports`. Record the source hash and commit in the item's `tags`/`file`;
  the body carries only the derived summary — never copy Plan chapters.
- **Load failure means stop.** If `read_specs`/`list_specs` (or the HTTP GET,
  or a parse attempt in Regime B) reports the ledger unparseable, stop and
  follow the error guidance: restore from a backup or rebuild from
  `docs/specs/`, then retry. An unvalidated hand-edit or
  delete-and-recreate is forbidden — that is how ledger data gets destroyed
  (UAT 2026-09-29, K6: a reviewer's manual JSON surgery erased existing
  entries). The K6 ban targets unvalidated replacement; the Regime B
  offline projection exists only for a ledger that already parses, and its
  guards enforce the same bottom line.

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
| `prepared` | Reviewed baseline, canonical Spec diff, projection targets, expected source versions, delivery commit |
| `landed` | Default-branch tip equal to the delivery commit after a `--ff-only` merge (no merge commit); old→new rebase hash mapping with `git range-diff` equivalence when landing rewrote the branch |
| `ledger_refreshed` | Target workspace/path, verified item IDs, source references, version/hash, and the write channel used (spec tools / HTTP / guarded offline projection) |
| `archived` | Archive path and delivered outcome, **plus the plan/docs artifacts committed to the default branch (untracked docs/ = not archived; PLAN-104 UAT-K13)** |
| `cleaned` | Guard result and confirmed worktree/branch removal, plus build intermediates (cargo `target` trees, `.pdb`) reclaimed under `D:/autostack/plan-artifacts/plan-<NNN>/` while keeping `.exe` outputs |

**Minimum ledger entries per merge (PLAN-104 SD-02, per specs-ledger.md):**
`designs` ≥ 1 (module contracts added/revised by this Plan), `reviews` ≥ 1
(the independent review conclusion — it always exists by merge time),
`reports` ≥ 1 (this deposit itself); `tests` when new test assets landed.
Every entry carries `file` (canonical path) and `tags` (source hash / commit
/ plan id) — all Regime A channels support these fields since PLAN-104, and
a Regime B offline projection writes them identically. A
single-entry refresh with empty `file`/`tags` is under-fill, not a receipt.

**Version-control closing is part of the flow, not an afterthought:** the
demo-workspace UAT of 2026-10-11 (K13) landed code ff-only but left
`docs/plans/**` and `.autoos/**` untracked with no receipts commit — the
plan file was "archived on disk" yet outside version control, and the
linear-history final check failed. Before recording `archived`/`cleaned`:
commit docs/ artifacts to the default branch, verify `git status` clean, and
confirm the expected linear history segments.

Before retrying, reconcile the receipt with disk, Git, and live ledger state.
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
For runtime-only ledger data, draft the intended item contents in the worktree
and publish them to the identified workspace after canonical Specs land —
through the channel your regime sanctions (see "Ledger refresh
(writer regimes)").

## Auto-lang post-review regression gate

For auto-lang (2026-10-14 ruling), merge owns the unified regression gate.
After final code/acceptance/Spec review passes and the delivery baseline is frozen,
run bare cargo t plus only the surface tiers required by AGENTS.md before landing.
Do not run these whole tiers during work, internal review, individual needs_fix
repairs, implementation commits or final-review resubmission. Required targeted
acceptance verification is still work/review-owned. cargo tf remains due-triggered,
main-checkout single-instance batch regression, never a Plan repair gate.

Bind each result to the actual commit, dependencies, verification configuration
and inputs; record pending/passed/failed gates in the Plan. A review pass with
pending merge-owned gates cannot authorize landing or a delivered claim. On a
failure, diagnose it without silently waiving a required gate; code fixes return
to work and affected re-review before rerunning affected gates on the new frozen
baseline. Reuse prior evidence only with an explicit unchanged-input/baseline
justification. Rebase or later edits that affect the tested baseline invalidate
that evidence; check the final delivery identity before landing.

Other repositories retain their own current gate timing and test tiers.

## Land and refresh the derived view

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
5. Publish/verify the derived ledger for the correct workspace — through the
   channel your regime sanctions (see "Ledger refresh (writer regimes)"):
   spec tools in auto-ai sessions; the `/api/specs/item` /
   `/api/specs/transition` endpoints when a service is up (any agent);
   the guarded offline projection for external agents with no reachable
   store writer. Respect any existing write-approval policy. What stays
   forbidden in every regime is an *unvalidated* hand replacement of the
   file — that is how foreign schemas and lost entries happen — which is
   exactly why the offline projection carries backup, scope, schema, and
   read-back guards. Record `blocked` only when the ledger is unparseable
   or the guarded procedure itself fails; keep the Plan active then.
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
  All ledger updates flow through exactly one channel per merge — the
  store's upsert/transition operations, or one guarded offline projection
  when no store writer exists — never a hand-assembled file replacement.
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
   and its merged branch; remove only the empty group directory. Never create
   junctions/symlinks or bypass the guard.
   After worktree removal, also reclaim the Plan's build intermediates: in
   the Plan's artifact directory under `D:/autostack/plan-artifacts/`
   (typically `plan-<NNN>/`; if no such directory exists there is nothing to
   reclaim), delete compilation intermediates — cargo `target` trees and
   `.pdb` debug-symbol files — while keeping final linked outputs and every
   other artifact intact. Rule (2026-10-27 user ruling: keep built `.exe`,
   drop intermediates; the exe↔pdb GUID pairing breakage is accepted):
   - `target` trees: their locations are not fixed (consumer/merge/repair
     checkouts and worktree-group layouts nest them at varying depths, and
     one Plan can hold several — 2026-10 measured 15 GB in a single
     `plan-856/target`), so locate them by search (e.g.
     `find <dir> -type d -iname target -not -path '*/target/*'`) and confirm
     each hit is a cargo build cache (a `CACHEDIR.TAG` file inside) before
     deleting.
   - `.pdb` files: delete wherever they appear under the Plan's directory.
   - Keep `.exe` binaries (runner/evidence executables are the hash-anchored
     measurement and replay anchors) and all logs, receipts, frozen sources,
     and probes.
   Record what was removed (paths/kind) with the `cleaned` evidence.
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
