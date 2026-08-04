//! Relay orchestration engine — multi-agent pipeline for musk.
//!
//! As of Plan 008, the generic orchestration primitives (HandoffDocument,
//! BudgetTracker, FlowSpec, PipelineEngine) have been moved to
//! `auto_ai_agent::orchestration`. This module re-exports them for backward
//! compatibility + provides musk-specific layers:
//!
//! - `profession.rs` — musk's Profession metadata (handoff_to, dispatchable_to,
//!   approval_gates, ForgePhase) + ProfessionRegistry
//! - `store.rs` — run persistence + SSE event bus (app-specific)
//! - `api.rs` — HTTP/SSE endpoints (app-specific)
//! - `driver.rs` — musk's orchestration loop (builds agents with musk context)

// Re-export the generic orchestration types from auto-ai-agent (Plan 008).
pub use auto_ai_agent::orchestration::{
    AdvanceResult, BudgetAction, BudgetStrategy, BudgetTracker, ContextPointers,
    Decision, ExitRouting, FlowSpec, FlowStep, GateDecision, GateType,
    HandoffDocument, PendingGate, PipelineEngine, PipelineMode, PipelineStatus,
    StepRecord, TokenBudget, TokenUsage, WorkProduct, Question,
};

// Musk-specific modules.
pub mod api;
pub mod driver;
pub mod flows;
pub use flows::{builtin_flows, get_builtin_flow};
pub mod profession;
pub mod store;
pub mod task_plan;
pub mod task_plan_parser;
pub mod handoff_store;
pub mod task_plan_registry;
pub mod task_plan_engine;

// Musk-specific re-exports.
pub use profession::{ForgePhase, Profession, ProfessionRegistry};
pub use store::{RunEvent, RunState, RunStore, RunSummary};

// Compatibility alias: musk code uses `RelayMode` → now `PipelineMode`.
pub type RelayMode = PipelineMode;
