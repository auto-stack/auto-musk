//! Built-in workflow definitions + loading.
//!
//! Workflows are `.at` files parsed by `auto-ai-agent::parse_at_workflow`.
//! The built-in ones are embedded via `include_str!`; custom ones can be
//! loaded from a path at request time.

use auto_ai_agent::{parse_at_workflow, Workflow};

/// Built-in workflow: architect → coder → tester → reviewer.
pub const FEATURE_DEV_AT: &str = include_str!("../workflows/feature-dev.at");

/// All built-in workflow names.
pub fn builtin_names() -> &'static [&'static str] {
    &["feature-dev"]
}

/// Load a built-in workflow by name, else parse a `.at` file at `path`.
pub fn load(spec: &str) -> Result<Workflow, String> {
    let content: &str = match spec {
        "feature-dev" => FEATURE_DEV_AT,
        _ => {
            return parse_at_workflow(
                &std::fs::read_to_string(spec)
                    .map_err(|e| format!("not a builtin, cannot read '{spec}': {e}"))?,
            )
            .map_err(|e| format!("parse '{spec}': {e}"))
        }
    };
    parse_at_workflow(content).map_err(|e| format!("parse builtin '{spec}': {e}"))
}
