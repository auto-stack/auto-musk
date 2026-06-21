# auto-musk backend (Rust)

auto-musk is the Forge-successor AI coding agent. This is its **Rust backend**
— a CLI agent built on the [auto-ai](../..) three-layer stack
(`auto-ai-daemon` + `auto-ai-client` + `auto-ai-agent`).

> This `rust-impl` branch implements auto-musk in Rust. The `main` branch
> retains the earlier Auto-language work for a future Auto reimplementation.

## Installation

musk depends on `auto-ai` (GitHub) and `auto-lang` (Gitee) via path deps.
The easiest install is via the install script (clones siblings + `cargo install`):

```sh
git clone https://github.com/auto-stack/auto-musk.git
cd auto-musk
git checkout rust-impl
bash install.sh
```

This installs the `musk` binary to `~/.cargo/bin/musk`. You also need the
`aaid` daemon — build it from auto-ai:

```sh
# The install script clones auto-ai to ../auto-ai
cd ../auto-ai
cargo build -p auto-ai-daemon
# Configure: ~/.config/autoos/ai-daemon.at (see crates/ai-config/examples/)
cargo run -p auto-ai-daemon   # leave running in a terminal
```

Then:

```sh
musk run "your task here"     # one-shot
musk chat                      # interactive (streaming, multi-turn)
musk serve                     # HTTP API server (:8080)
musk professions               # list built-in professions
```

## Architecture

```
musk CLI ──► auto-ai-agent (Coder Profession + ReAct loop + tools)
                     │
                     ▼ uses
              auto-ai-client (thin HTTP)
                     │
                     ▼ canonical wire (HTTP)
              auto-ai-daemon (aaid) ──► upstream LLM (Zhipu/Anthropic/OpenAI)
```

- **musk** owns the CLI + local tools (`read_file` / `write_file` / `run_command`).
- **auto-ai-agent** owns the Profession library + ReAct loop.
- **auto-ai-daemon** owns all LLM communication (the single gateway).
- **musk carries no provider knowledge and no LLM code** — it talks to the
  daemon via the canonical wire format.

## Build

```sh
cd backend
cargo build
```

This pulls `auto-ai-agent` and `auto-ai-client` from the sibling `../auto-ai`
repo via path dependencies (no publish needed).

## Run

### 1. Start the daemon

The daemon must be running first. From the auto-ai repo:

```sh
cd ../auto-ai
cargo run -p auto-ai-daemon
```

It needs at least one provider configured. Either:
- set an env var: `ZHIPU_API_KEY=...` (or `ANTHROPIC_API_KEY` / `OPENAI_API_KEY`), or
- create `~/.config/autoos/ai-daemon.at` (see
  `../auto-ai/crates/ai-config/examples/daemon.at` for the format).

### 2. Run musk

```sh
cd backend
cargo run -- run "List the files in the current directory, then read Cargo.toml and summarize it."
```

The Coder agent will loop (Reason → Act via tools → Observe), calling the local
`run_command` / `read_file` tools, and print its final answer plus a summary of
tool calls.

## Tools (built-in)

| Tool | Description |
|---|---|
| `read_file` | Read a file's UTF-8 contents |
| `write_file` | Write text to a file (auto-creates parent dirs) |
| `run_command` | Run a shell command, return combined stdout+stderr |

All execute **locally in the musk process** — only LLM calls go to the daemon.

## Configuration (agent side)

The Coder Profession is built in. To use a custom Profession (own prompt /
model / tools), create a `.at` file (see
`../auto-ai/docs/auto-ai-agent-design.md` §4) and load it via
`auto_ai_agent::load_profession`. *(CLI support for custom profession files
lands in a later task; the MVP hard-codes Coder.)*

## Tests

```sh
cd backend
cargo test    # 7 tool unit tests
```

## Status

This is the **v2 phase 0 + 1 MVP** (see `../plans/001-auto-forge-migration-super-plan.md`
and `../plans/003-musk-mvp-rust-backend.md`). Next phases: HTTP API + frontend
integration, SSE streaming, multi-role workflows.
