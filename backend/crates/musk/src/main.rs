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

        /// Agent mode: a built-in name (superpowers, basic, coding, review)
        /// or a path to a custom `.at` mode file. Defaults to "superpowers".
        /// Use 'basic' for quick Q&A (no skills); 'coding' for focused coding;
        /// 'review' for code review (read-only).
        #[arg(long, default_value = "superpowers")]
        mode: String,
    },

    /// Interactive chat session (multi-turn REPL). The agent persists across
    /// turns, accumulating context. Type a message, Enter to send; `exit` or
    /// `quit` (or Ctrl-D) to leave.
    Chat {
        /// Agent mode (see `musk run --mode`). Defaults to "superpowers".
        #[arg(long, default_value = "superpowers")]
        mode: String,
    },

    /// List available built-in professions.
    Professions,

    /// List available agent modes.
    Modes,

    /// Start the HTTP API server (for the Vue frontend).
    Serve {
        /// Address to listen on.
        #[arg(long, default_value = "127.0.0.1:8080")]
        addr: String,
    },
}

fn main() {
    let cli = Cli::parse();

    // Apply the runtime config (daemon_url etc.) from
    // ~/.config/autoos/apps/musk/config.at to the environment, so the CLI
    // (run/chat/serve) honors it just like the config UI. An explicit AAID_URL
    // env var still wins. (Plan 008 Level 1.)
    musk::app_config::apply_app_config();

    match cli.cmd {
        Cmd::Professions => {
            list_professions();
            return;
        }
        Cmd::Modes => {
            list_modes();
            return;
        }
        Cmd::Run { task, mode } => {
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
            let agent_mode = resolve_mode(&mode);
            let rt = tokio::runtime::Runtime::new().expect("failed to start tokio runtime");
            if let Err(e) = rt.block_on(run_task(&task, agent_mode, client)) {
                eprintln!("musk: {e}");
                std::process::exit(1);
            }
        }
        Cmd::Chat { mode } => {
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
            let agent_mode = resolve_mode(&mode);
            let rt = tokio::runtime::Runtime::new().expect("failed to start tokio runtime");
            if let Err(e) = rt.block_on(chat_loop(agent_mode, client)) {
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

/// Resolve a mode spec: built-in name → registry, else error.
fn resolve_mode(spec: &str) -> musk::mode::AgentMode {
    let reg = musk::mode::ModeRegistry::load();
    match reg.get(spec) {
        Some(m) => m.clone(),
        None => {
            eprintln!(
                "musk: unknown mode '{spec}'. Available: {}",
                reg.names().join(", ")
            );
            std::process::exit(1);
        }
    }
}

/// List available modes.
fn list_modes() {
    let reg = musk::mode::ModeRegistry::load();
    println!("Agent modes:");
    for name in reg.names() {
        if let Some(m) = reg.get(&name) {
            println!("  {name:<14} profession={:<10} skills={:<3} tools={}", m.profession, m.skills, m.tools.len());
            if !m.description.is_empty() {
                println!("                  {}", m.description.chars().take(80).collect::<String>());
            }
        }
    }
    println!("\nUse --mode <name> with 'musk run' or 'musk chat'.");
}

async fn run_task(
    task: &str,
    mode: musk::mode::AgentMode,
    client: Arc<dyn Client>,
) -> Result<(), String> {
    use musk::build_agent_from_mode;
    let name = mode.name.clone();
    let prof_name = mode.profession.clone();

    let mut agent = build_agent_from_mode(&mode, client)
        .map_err(|e| format!("build agent: {e}"))?;

    println!("musk: running mode '{}' (profession={}) on task:\n  {task}\n", name, prof_name);

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

/// List available built-in professions.
fn list_professions() {
    use auto_ai_agent::{builtin_names, load_builtin};
    println!("Built-in professions:");
    for name in builtin_names() {
        if let Some(p) = load_builtin(name) {
            println!(
                "  {name:<14} tier={:<5} max_turns={:<4} temp={}",
                p.model_tier().display_name(),
                p.max_turns(),
                p.temperature()
            );
        }
    }
}

/// Interactive multi-turn chat REPL with streaming output. The agent persists
/// across turns, so conversation memory accumulates. Skills (if configured) are
/// available — the model can invoke them via the `skill` tool.
///
/// Uses `Agent::run_stream` so the model's answer prints token-by-token (via
/// `StreamEvent::Delta`), tool calls print inline, and a summary follows each
/// turn.
async fn chat_loop(
    mode: musk::mode::AgentMode,
    client: Arc<dyn Client>,
) -> Result<(), String> {
    use std::io::{self, BufRead, Write};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc as StdArc;

    let name = mode.name.clone();
    let mut agent = musk::build_agent_from_mode(&mode, client)
        .map_err(|e| format!("build agent: {e}"))?;

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
