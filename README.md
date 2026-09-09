# Auto Musk

[English](README.md) | [简体中文](README.zh.md)

Auto Musk is an Auto-language coding agent built around a **Plan + Spec development workflow**. A Plan carries one change from analysis through implementation and verification; Specs preserve the project's accumulated knowledge for future work.

The successor to auto-forge, Auto Musk combines interactive coding with staged agent orchestration. Roles, skills, and backing models are configurable through the AutoStack ecosystem. The Plan-based method has been used for more than 500 development plans in [auto-lang](../auto-lang); this is practical experience with a human-supervised workflow, rather than a claim of unattended completion.

## What makes it different

- **Plans for change, Specs for continuity.** Keep the working context together during development, then deposit reviewed outcomes into organized project knowledge.
- **Four explicit stages.** `new → work → review → merge` separates analysis, execution, verification, and knowledge consolidation.
- **Artifact-based handoffs.** A Markdown Plan with YAML frontmatter carries goals, decisions, tasks, acceptance criteria, progress, and review results across stages.
- **Configurable agents.** A stage can select a role with its own skills, tools, model or model tier, and budget. The current built-in Plan flow uses the same `plan-dev` role for all four phases.
- **Verification before completion.** The review discipline checks actual code and test results, including omitted requirements and unapproved deferrals.
- **Auto-language implementation.** Auto sources generate the primary Vue frontend and parts of the Rust backend; an AutoVM backend and native UI path are also available.
- **Visible development activity.** Chat, tool events, Plan progress, approval gates, reports, Specs, and Wiki are accessible from the application.

## Architecture

```text
User / CLI / Web UI
        |
        v
auto-musk
  Chat + Plans + Spec Ledger + Wiki
  Relay: phase orchestration, gates, handoffs, reports
        |
        v
auto-ai-agent
  Roles + skills + tools + agent loop + generic workflow primitives
        |
        v
auto-ai-client → auto-ai-daemon (aaid) → Model providers

auto-os-config → Shared role, skill, model, and application configuration
auto-lang      → Auto source compilation, code generation, and VM runtime
```

| Layer | Responsibility |
|---|---|
| **auto-musk** | Development workflow, project artifacts, local tools, workspace data, API, and UI |
| **auto-ai** | Agent execution and generic orchestration; the daemon owns provider communication, concurrency, and usage tracking |
| **auto-os-config** | Shared configuration editor and service for AutoStack applications, roles, skills, and model settings |
| **auto-lang** | Language, compiler, generated frontend/backend support, and VM runtime |

Agent count and development stages are separate choices. Relay can select different roles per stage; the canonical `plan` flow currently runs serially as `plan-dev`, with a human confirmation gate before execution. The older seven-role Spec pipelines remain in code for comparison.

### Implementation and repository layout

The default HTTP backend is Rust/axum, combining handwritten infrastructure with Rust generated from Auto. The primary web UI is generated from Auto into Vue. The repository retains a handwritten web fallback and an AutoVM backend option; it is not yet an entirely VM-native application.

```text
auto-musk/
├── .agents/skills/               Four repository development skills
│   ├── auto-plan-new/
│   ├── auto-plan-work/
│   ├── auto-plan-review/
│   └── auto-plan-merge/
├── skills/                      Application agent skills
├── src/front/                   Auto UI sources and platform adapters
├── src/back/api.at              Frontend API contract
├── backend/crates/musk/
│   ├── auto-src/                Auto backend sources
│   └── src/                     Rust infrastructure and auto_generated/
├── gen/front/vue/               Generated primary web application
├── web/                         Handwritten web fallback/reference
├── docs/plans/                  Active Plans
│   └── archived/                Archived Plans
├── docs/specs/                  Module-oriented project knowledge
├── docs/designs/                Architecture and design references
└── pac.at                       Auto build configuration
```

See [pac.at](pac.at), the [backend entry point](backend/crates/musk/src/main.rs), and the [HTTP server](backend/crates/musk/src/server.rs) for runtime selection and frontend hosting.

### Plans, Specs, and run history

| Artifact | Purpose | Location |
|---|---|---|
| Plan | One change's analysis, design, tasks, acceptance criteria, and progress | `docs/plans/NNN-slug.md` |
| Archived Plan | Preserved change history | `docs/plans/archived/` |
| Spec Ledger | Derived indexes, relations, and history across six sections | Workspace `.autoos/specs.json` |
| Module Specs | Authoritative current module behavior and design knowledge | `docs/specs/` |
| Conversations | Chat and Relay activity history | Workspace `.autoos/conversations/` |

`docs/specs/` is the authoritative source of current project knowledge. The ledger provides derived indexes, relations, and history. The repository skills apply a reviewed Spec delta in the worktree, land the canonical documents, refresh the derived ledger, and archive only after verifying those operations. Consolidation receipts support checking and completing interrupted work.

The application's existing `merge_plan` operation still copies Plan chapters to the ledger and archives immediately, with module updates performed afterward by the document phase. The updated repository merge skill avoids that combined operation; the application flow has not yet adopted the new consolidation contract.

Relay runs currently live in memory. Their activity is recorded in conversations, but restarting the service does **not** resume active runs. A completed Relay run is also distinct from an accepted and consolidated Plan.

## Getting started

### Prerequisites

- Rust and Cargo.
- Node.js and pnpm for the generated web UI.
- An installed `auto` toolchain from [auto-lang](../auto-lang).
- Sibling checkouts of `auto-ai` and `auto-lang`, matching the Cargo path dependencies.
- A configured `aaid` daemon for model-backed tasks. [auto-os-config](../auto-os-config) provides a configuration UI.

```text
<workspace>/
├── auto-musk/
├── auto-ai/
├── auto-lang/
└── auto-os-config/              Optional settings application
```

Configure the daemon's providers and models in `~/.config/autoos/ai-daemon.at`. Auto Musk runtime settings live in `~/.config/autoos/apps/musk/config.at`. See [auto-ai architecture](../auto-ai/ARCHITECTURE.md) and [auto-os-config setup](../auto-os-config/README.md).

### Build and run

Commands below start in the Auto Musk repository root unless noted.

Start the model daemon in a separate terminal:

```sh
cargo run --manifest-path ../auto-ai/Cargo.toml -p auto-ai-daemon
```

Build and install the CLI:

```sh
cargo install --path backend/crates/musk
```

Generate and build the primary web application:

```sh
auto build --gen-only
cd gen/front/vue
pnpm install
pnpm build
```

Back at the repository root, start the application:

```sh
musk serve
```

Open [http://127.0.0.1:8080](http://127.0.0.1:8080). The server hosts the API and, by default, `gen/front/vue/dist`. For frontend development, run `pnpm dev` in `gen/front/vue` alongside the backend. On Windows, `start-musk-web.cmd` starts that frontend dev server.

### CLI and runtime options

Run CLI coding tasks from the intended project directory; it establishes the tool workspace.

```sh
musk run "Read the project and summarize its architecture"
musk run --mode coding "Implement the agreed change"
musk chat
musk chat --mode review
musk modes
musk professions
musk serve --addr 127.0.0.1:8080 --workdir <project-directory>
```

| Option | Purpose |
|---|---|
| `--mode` | Select an agent mode, such as `superpowers`, `basic`, `coding`, or `review` |
| `AAID_URL` | Override the configured daemon URL |
| `MUSK_WEB_DIST` | Override the hosted frontend directory, including the `web/dist` fallback |
| `MUSK_BACKEND=vm` | Select the AutoVM HTTP backend path |
| `auto run --render=vm` | Launch the native VM UI path using the Auto toolchain |

VM paths are under active development. For frontend source conventions, see [the Auto UI notes](src/front/README.at-conventions.md).

## Using the development workflow

### Four repository skills

The skills in [.agents/skills](.agents/skills) are intended for a coding-agent host that loads repository skills. The examples below are **conversation instructions**, not `musk` shell subcommands; exact slash-command presentation depends on the host.

| Stage | Example instruction | Output |
|---|---|---|
| [new](.agents/skills/auto-plan-new/SKILL.md) | `/auto-plan:new Add feature X with constraints Y` | A self-contained draft Plan for confirmation |
| [work](.agents/skills/auto-plan-work/SKILL.md) | `/auto-plan:work 42` | Implementation and per-task evidence; `execution_done` when complete |
| [review](.agents/skills/auto-plan-review/SKILL.md) | `/auto-plan:review 42` | Reproduced acceptance checks, findings, and Spec-impact metadata |
| [merge](.agents/skills/auto-plan-merge/SKILL.md) | `/auto-plan:merge 42` | Reviewed code landed, knowledge deposited, Plan archived, and worktree cleaned up |

Typical use:

1. Describe the requirement and constraints; ask the agent to create a Plan.
2. Read the proposed scope, design, and acceptance criteria; confirm or revise it.
3. Execute the Plan in its dedicated worktree. Progress remains recorded in the Plan.
4. Review the actual implementation. Address findings through work and repeat review as needed.
5. Merge the reviewed change, consolidate the project knowledge, and archive the Plan.

The normal completion path is:

```text
drafting → executing → execution_done → reviewed → archived
                 ↑           |
                 └── fixes ──┘
```

Review failure returns the work for correction. Archiving can also mean shelving an unfinished Plan through the application's archive action, so archive status alone does not prove successful delivery.

A Plan contains goals, architecture, background analysis, detailed design, test design, acceptance criteria, execution tasks, review records, and open questions. Frontmatter carries its ID, status, semantic `plan_revision`, progress, and Spec-impact fields: `supersedes_spec_components`, `new_spec_components`, and `touched_goals`. Stable task and acceptance IDs connect work to evidence. The Plan includes a proposed Spec delta that review validates against code and canonical documents.

The repository skills allow relevant code/Spec lookup and bounded adjustments within existing authorization. Handoffs use `pass`, `needs_fix`, `needs_replan`, or `blocked` as appropriate to the stage, separate from Plan status. Failed review returns to `executing`; successful review records its Plan revision and code commit. An already authorized end-to-end workflow can continue through the next skill, with bounded repair attempts. These records do not provide backend restart recovery.

### Built-in Relay flow

Within Auto Musk's application chat, ask for a complex change to follow the Plan flow. The agent can invoke `spawn_relay(flow_id="plan", ...)`:

```text
plan → [human confirms the Plan] → execute → review → document
```

This flow uses the same Plan artifact and four-stage discipline. Each phase currently uses `plan-dev`. The application displays the gate, tool activity, and reports in the conversation.

Today, failed review requires a follow-up to repair and re-review; the built-in flow does not automatically repeat work until review passes. The document phase skips consolidation when the Plan is not reviewed. The four repository skills and the application phase templates are separate entry points, so do not assume invoking a Relay flow enforces every repository worktree rule automatically.

### Developing Auto Musk itself

Follow [AGENTS.md](AGENTS.md):

- Make repository changes in a dedicated worktree. Plan work uses `D:/autostack/.wt/musk-<NNN>/auto-musk` and branch `plan-<NNN>-dev`.
- Keep shared Plan drafts and progress markers on the main checkout; implementation belongs in the worktree.
- Put dependency worktrees alongside the project in the same group. Never create junctions or symlinks inside a worktree.
- Run scoped checks during implementation and the required full-suite gate during review or phase landing.
- Before removing a worktree, run `wt-guard.sh` and require a clean result. Merge committed work and complete cleanup according to the repository rules.

The [workflow design](docs/designs/008-auto-plan.md) explains the Plan/Spec split. The current skills and implementation should be consulted for operational details as the design evolves.

## Outlook

The next step is to make the existing workflow reliably automated while preserving the Plan/Spec split. The following are proposed improvements, not completed features:

1. **Align the runtime with the skill contracts.** Bring runtime prompts, state handling, and Auto/generated implementations into line with the updated skills. Move deterministic validation into shared tools.
2. **Support bounded adaptation in the application.** Carry the skills' revision and authorization rules into runtime handoffs, preserving relevant evidence and user decisions.
3. **Build a durable Plan Runner.** Persist execution checkpoints, run ownership, pending gates, and configuration snapshots; support restart recovery, pause, cancellation, and budget limits.
4. **Close the work/review loop.** Use structured outcomes such as `pass`, `needs_fix`, `needs_replan`, and `blocked`; bind evidence to Plan and code revisions and stop repeated attempts without progress.
5. **Automate recoverable consolidation.** Implement the skills' canonical-Spec-first ordering and receipts in the backend, with concurrency control and recovery for publication, archival, and cleanup.
6. **Route agents by capability.** Use independent review contexts and evaluated role/model choices; add parallel work where task dependencies and write ownership permit it.
7. **Measure before scaling.** Evaluate on representative historical Plans, tracking human intervention, missed requirements, recovery success, time, and cost before increasing unattended concurrency.
