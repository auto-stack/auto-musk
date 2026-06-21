//! `musk` — auto-musk CLI 入口。
//!
//! 用法:
//!   musk run "<task>"                         用内置 Coder agent 处理任务
//!   musk run --profession coder "<task>"      指定内置 profession
//!   musk run --profession my-profession.at "<task>"  从 .at 文件加载自定义 profession
//!   musk professions                          列出所有内置 profession
//!   musk serve [--addr 127.0.0.1:8080]        启动 HTTP API server(给前端用)
//!
//! agent 经 auto-ai-daemon 调用 LLM,工具在本地执行。详见 backend/README.md。

use std::sync::Arc;

use clap::{Parser, Subcommand};

use auto_ai_agent::{builtin_names, load_builtin, load_profession, Client, Profession};
use auto_ai_client::AiClient;

use musk::tools::{ReadFile, RunCommand, WriteFile};

#[derive(Parser)]
#[command(
    name = "musk",
    version,
    about = "auto-musk — Forge-successor AI coding agent"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Run an agent on a task (one-shot, prints result to stdout).
    Run {
        /// The task description for the agent.
        task: String,

        /// Profession to use: a built-in name (coder, architect, tester,
        /// reviewer, documenter, translator, runner) or a path to a custom
        /// `.at` profession file. Defaults to "coder".
        #[arg(long, default_value = "coder")]
        profession: String,

        /// Disable Superpowers (skill system). The agent gets only base tools
        /// — simpler/faster, like Claude without superpowers. Useful for quick
        /// Q&A or simple edits where brainstorm/plan/execute is overkill.
        #[arg(long)]
        no_skills: bool,
    },

    /// Interactive chat session (multi-turn REPL). The agent persists across
    /// turns, accumulating context. Type a message, Enter to send; `exit` or
    /// `quit` (or Ctrl-D) to leave.
    Chat {
        /// Profession to use (built-in name or .at path). Defaults to "coder".
        #[arg(long, default_value = "coder")]
        profession: String,

        /// Disable Superpowers (skill system) — basic mode (see `musk run --no-skills`).
        #[arg(long)]
        no_skills: bool,
    },

    /// List available built-in professions.
    Professions,

    /// Start the HTTP API server (for the Vue frontend).
    Serve {
        /// Address to listen on.
        #[arg(long, default_value = "127.0.0.1:8080")]
        addr: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.cmd {
        Cmd::Professions => {
            list_professions();
            return;
        }
        Cmd::Run { task, profession, no_skills } => {
            // For `run`, a missing daemon is fatal — there's nothing to do
            // without the LLM. Build the client (blocking daemon discovery)
            // BEFORE entering the tokio runtime to avoid the
            // reqwest::blocking "cannot drop runtime" panic.
            let client: Arc<dyn Client> = match AiClient::new() {
                Ok(c) => Arc::new(c),
                Err(e) => {
                    eprintln!(
                        "musk: cannot reach auto-ai-daemon: {e}\n  \
                         Is `aaid` running? Build & start it from the auto-ai repo:\n    \
                         cd ../auto-ai && cargo run -p auto-ai-daemon\n  \
                         Also ensure ~/.config/autoos/ai-daemon.at is configured."
                    );
                    std::process::exit(1);
                }
            };
            let prof = match resolve_profession(&profession) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("musk: invalid profession '{profession}': {e}");
                    std::process::exit(1);
                }
            };
            let rt = tokio::runtime::Runtime::new().expect("failed to start tokio runtime");
            if let Err(e) = rt.block_on(run_task(&task, prof, client, !no_skills)) {
                eprintln!("musk: {e}");
                std::process::exit(1);
            }
        }
        Cmd::Chat { profession, no_skills } => {
            // Chat needs the daemon (LLM). Build client before the runtime.
            let client: Arc<dyn Client> = match AiClient::new() {
                Ok(c) => Arc::new(c),
                Err(e) => {
                    eprintln!(
                        "musk: cannot reach auto-ai-daemon: {e}\n  \
                         Start it: cd ../auto-ai && cargo run -p auto-ai-daemon"
                    );
                    std::process::exit(1);
                }
            };
            let prof = match resolve_profession(&profession) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("musk: invalid profession '{profession}': {e}");
                    std::process::exit(1);
                }
            };
            let rt = tokio::runtime::Runtime::new().expect("failed to start tokio runtime");
            if let Err(e) = rt.block_on(chat_loop(prof, client, !no_skills)) {
                eprintln!("musk: {e}");
                std::process::exit(1);
            }
        }
        Cmd::Serve { addr } => {
            // For `serve`, start the HTTP server even if the daemon is down —
            // the frontend still loads, and each /api/run surfaces the daemon
            // error so the user can start aaid afterward.
            let client: Arc<dyn Client> = match AiClient::new() {
                Ok(c) => Arc::new(c),
                Err(e) => {
                    eprintln!(
                        "musk: warning — auto-ai-daemon unreachable: {e}\n  \
                         Starting HTTP server anyway; /api/run will error until aaid is up.\n  \
                         Start it: cd ../auto-ai && cargo run -p auto-ai-daemon"
                    );
                    Arc::new(NoDaemonClient)
                }
            };
            let rt = tokio::runtime::Runtime::new().expect("failed to start tokio runtime");
            if let Err(e) = rt.block_on(musk::server::serve(&addr, client)) {
                eprintln!("musk server: {e}");
                std::process::exit(1);
            }
        }
    }
}

/// A stand-in client used by `serve` when the daemon isn't up yet. Every call
/// returns a clear "daemon not running" error so `/api/run` surfaces it to the
/// frontend instead of crashing the server.
struct NoDaemonClient;

#[async_trait::async_trait]
impl Client for NoDaemonClient {
    async fn complete(
        &self,
        _req: &auto_ai_client::CompletionRequest,
    ) -> Result<auto_ai_client::CompletionResponse, auto_ai_client::ClientError> {
        Err(auto_ai_client::ClientError::DaemonUnavailable)
    }
}

/// Resolve a profession spec: a built-in name, or a path to a `.at` file.
fn resolve_profession(spec: &str) -> Result<Arc<dyn Profession>, String> {
    if let Some(p) = load_builtin(spec) {
        return Ok(p);
    }
    let content = std::fs::read_to_string(spec)
        .map_err(|e| format!("not a builtin, and cannot read '{spec}' as a file: {e}"))?;
    load_profession(&content).map_err(|e| format!("parse '{spec}': {e}"))
}

fn list_professions() {
    println!("Built-in professions:");
    for name in builtin_names() {
        if let Some(p) = load_builtin(name) {
            println!(
                "  {name:<14} model={:<10} max_turns={:<4} temp={}",
                p.model(),
                p.max_turns(),
                p.temperature()
            );
            let first_line = p
                .system_prompt()
                .lines()
                .find(|l| !l.trim().is_empty() && !l.starts_with('#'))
                .unwrap_or("")
                .chars()
                .take(70)
                .collect::<String>();
            if !first_line.is_empty() {
                println!("                  {first_line}");
            }
        }
    }
    println!("\nUse --profession <name|file.at> with 'musk run'.");
}

async fn run_task(
    task: &str,
    profession: Arc<dyn Profession>,
    client: Arc<dyn Client>,
    skills_enabled: bool,
) -> Result<(), String> {
    use musk::build_agent;
    let name = profession.name().to_string();
    let model = profession.model().to_string();

    let mut agent = build_agent(profession, client, skills_enabled);
    // build_agent already registers the 3 standard tools.
    let _ = (ReadFile, WriteFile, RunCommand); // imported for discoverability

    println!("musk: running profession '{}' (model={}) on task:\n  {task}\n", name, model);

    let result = agent
        .run(task)
        .await
        .map_err(|e| format!("agent failed: {e}"))?;

    println!(
        "──── result ({} turn{}, {} tool call{}) ────",
        result.turns,
        if result.turns == 1 { "" } else { "s" },
        result.tool_calls.len(),
        if result.tool_calls.len() == 1 { "" } else { "s" }
    );
    println!("{}", result.output);

    if !result.tool_calls.is_empty() {
        println!("\n──── tool calls ────");
        for (i, tc) in result.tool_calls.iter().enumerate() {
            let preview: String = tc.result.chars().take(120).collect();
            let ellipsis = if tc.result.len() > 120 { "…" } else { "" };
            println!("  {}. {} args={} → {}{}", i + 1, tc.tool, tc.args, preview, ellipsis);
        }
    }

    Ok(())
}

/// Interactive multi-turn chat REPL with streaming output. The agent persists
/// across turns, so conversation memory accumulates. Skills (if configured) are
/// available — the model can invoke them via the `skill` tool.
///
/// Uses `Agent::run_stream` so the model's answer prints token-by-token (via
/// `StreamEvent::Delta`), tool calls print inline, and a summary follows each
/// turn.
async fn chat_loop(
    profession: Arc<dyn Profession>,
    client: Arc<dyn Client>,
    skills_enabled: bool,
) -> Result<(), String> {
    use std::io::{self, BufRead, Write};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc as StdArc;

    let name = profession.name().to_string();
    let mut agent = musk::build_agent(profession, client, skills_enabled);

    println!(
        "musk chat — profession '{}' (Ctrl-D or 'exit' to quit)",
        name
    );
    let mut turn = 0u32;
    let stdin = io::stdin();
    loop {
        print!("\nyou> ");
        io::stdout().flush().ok();
        let mut line = String::new();
        let n = stdin.lock().read_line(&mut line).map_err(|e| e.to_string())?;
        if n == 0 {
            break; // EOF (Ctrl-D)
        }
        let input = line.trim();
        if input.is_empty() {
            continue;
        }
        if input == "exit" || input == "quit" {
            break;
        }

        turn += 1;
        println!("\n{} ───", name);

        // The on_event callback: Delta → print token; Tool → print inline;
        // Done → summary; Error → message. Uses atomics for the tool count
        // (the callback is Sync but we want a running tally).
        let tool_count = StdArc::new(AtomicUsize::new(0));
        let tool_count_cb = tool_count.clone();
        let on_event: StdArc<dyn Fn(auto_ai_agent::StreamEvent) + Send + Sync> =
            StdArc::new(move |ev| {
                use auto_ai_agent::StreamEvent;
                match ev {
                    StreamEvent::Delta { text } => {
                        print!("{text}");
                        let _ = io::stdout().flush();
                    }
                    StreamEvent::Tool { tool, result, .. } => {
                        tool_count_cb.fetch_add(1, Ordering::SeqCst);
                        let preview: String = result.chars().take(80).collect();
                        let ellipsis = if result.len() > 80 { "…" } else { "" };
                        println!("\n  [tool] {tool} → {preview}{ellipsis}");
                    }
                    StreamEvent::Done { result } => {
                        let n = result.tool_calls.len();
                        println!(
                            "\n──── turn {turn}, {n} tool call{} ────",
                            if n == 1 { "" } else { "s" }
                        );
                    }
                    StreamEvent::Error { message } => {
                        println!("\n  [error] {message}");
                    }
                }
            });

        match agent.run_stream(input, on_event).await {
            Ok(_) => {}
            Err(e) => {
                // Don't kill the session on one failed turn (e.g. max_turns,
                // loop detected) — print and let the user continue.
                let tc = tool_count.load(Ordering::SeqCst);
                eprintln!(
                    "\n  [agent error after {tc} tool call(s)] {e}\n  (session continues; type another message or 'exit')"
                );
            }
        }
    }
    Ok(())
}
