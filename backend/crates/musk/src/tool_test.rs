//! Tool atom testing framework — test each tool's `execute()` across a matrix
//! of normal/boundary/error cases, in an isolated sandbox that auto-cleans.
//!
//! (Design 003 — Tool Atom Testing Framework.)
//!
//! ## How it works
//! Each case runs in a unique `tempfile::TempDir` (the sandbox). Side-effect
//! tools (write_file/edit_file/…) operate there via `chdir`. The sandbox is
//! dropped at the end → state auto-restored, no manual cleanup.
//!
//! Side-effect cases use `#[serial]` (the `serial_test` crate) so the global
//! CWD doesn't collide across parallel tests.

use std::path::Path;

use serde_json::Value as JsonValue;

/// A fixture file/dir to set up inside the sandbox before calling the tool.
#[derive(Clone, Debug)]
pub enum Fixture {
    /// Create a file with the given content (relative to sandbox root).
    File { path: &'static str, content: &'static str },
    /// Create an empty directory.
    Dir { path: &'static str },
}

impl Fixture {
    pub fn file(path: &'static str, content: &'static str) -> Self {
        Fixture::File { path, content }
    }
    pub fn dir(path: &'static str) -> Self {
        Fixture::Dir { path }
    }
}

/// Case category — used for coverage reporting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaseCategory {
    Normal,
    Boundary,
    Error,
}

/// What a case expects from the tool call.
#[derive(Clone, Debug)]
pub enum Expect {
    /// Ok, and the result string contains `substr`.
    OkContains(&'static str),
    /// Ok, and the result equals `exact` exactly.
    OkExact(&'static str),
    /// Ok, and additionally a file in the sandbox has the expected content
    /// (for verifying side effects: the file was modified correctly).
    OkFileEquals { path: &'static str, content: &'static str },
    /// Error of any kind.
    Err,
    /// Error whose message contains `substr`.
    ErrContains(&'static str),
}

/// One declarative tool test case: given fixtures → call tool → assert.
#[derive(Clone, Debug)]
pub struct ToolCase {
    pub name: &'static str,
    pub category: CaseCategory,
    pub fixtures: Vec<Fixture>,
    /// (tool_name, args)
    pub call: (&'static str, JsonValue),
    pub expect: Expect,
}

/// An isolated sandbox: a temp dir + a tool registry. chdir into the dir on
/// creation; restored on drop.
pub struct Sandbox {
    _dir: tempfile::TempDir, // kept alive to auto-delete on drop
    _prev_cwd: std::path::PathBuf,
}

impl Sandbox {
    /// Create a unique temp dir, chdir into it, apply fixtures.
    pub fn new(fixtures: &[Fixture]) -> std::io::Result<Self> {
        let dir = tempfile::tempdir()?;
        let prev_cwd = std::env::current_dir()?;
        std::env::set_current_dir(dir.path())?;
        let root = dir.path();
        for fx in fixtures {
            match fx {
                Fixture::File { path, content } => {
                    let full = root.join(path);
                    if let Some(parent) = full.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::write(full, content)?;
                }
                Fixture::Dir { path } => {
                    std::fs::create_dir_all(root.join(path))?;
                }
            }
        }
        Ok(Self { _dir: dir, _prev_cwd: prev_cwd })
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        // Restore CWD; TempDir auto-deletes on its own drop.
        let _ = std::env::set_current_dir(&self._prev_cwd);
    }
}

/// Run a single case: set up sandbox, call the tool, assert the expectation.
/// `tool` is a closure that produces a fresh tool instance (tools are cheap
/// unit structs). This keeps the framework decoupled from concrete tool types.
pub async fn run_case<F>(case: &ToolCase, make_tool: F)
where
    F: Fn(&str) -> Option<Box<dyn auto_ai_agent::Tool>>,
{
    let _sb = match Sandbox::new(&case.fixtures) {
        Ok(s) => s,
        Err(e) => panic!("[{}] sandbox setup failed: {e}", case.name),
    };

    let (tool_name, args) = &case.call;
    let tool = make_tool(tool_name).unwrap_or_else(|| {
        panic!("[{}] unknown tool '{tool_name}'", case.name)
    });

    let result = tool.execute(args).await;

    match &case.expect {
        Expect::OkContains(sub) => {
            let out = result.unwrap_or_else(|e| panic!("[{}] expected Ok, got Err: {e}", case.name));
            assert!(
                out.contains(sub),
                "[{}] expected result containing {:?}, got: {:?}",
                case.name, sub, out
            );
        }
        Expect::OkExact(exact) => {
            let out = result.unwrap_or_else(|e| panic!("[{}] expected Ok, got Err: {e}", case.name));
            assert_eq!(out, *exact, "[{}] result mismatch", case.name);
        }
        Expect::OkFileEquals { path, content } => {
            let out = result.unwrap_or_else(|e| panic!("[{}] expected Ok, got Err: {e}", case.name));
            let _ = out; // result itself isn't asserted here, only the file
            let actual = std::fs::read_to_string(path).unwrap_or_else(|e| {
                panic!("[{}] expected file {:?} to exist after call: {e}", case.name, path)
            });
            assert_eq!(
                actual, *content,
                "[{}] file {:?} content mismatch",
                case.name, path
            );
        }
        Expect::Err => {
            assert!(result.is_err(), "[{}] expected Err, got Ok: {:?}", case.name, result.unwrap());
        }
        Expect::ErrContains(sub) => {
            let err = result.unwrap_err();
            let msg = err.to_string();
            assert!(
                msg.contains(sub),
                "[{}] expected error containing {:?}, got: {:?}",
                case.name, sub, msg
            );
        }
    }
}

/// Run all cases, collecting pass/fail. Panics on first failure (so `cargo
/// test` reports it). For a gentler aggregate report, see `run_cases_report`.
pub async fn run_cases<F>(cases: &[ToolCase], make_tool: F)
where
    F: Fn(&str) -> Option<Box<dyn auto_ai_agent::Tool>>,
{
    for c in cases {
        run_case(c, &make_tool).await;
    }
}
