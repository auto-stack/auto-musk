//! Relay orchestration engine — the auto-forge differentiation core.
//!
//! Ported from auto-forge `backend/src/relay/` (~12k lines across 21 files),
//! flattened to auto-musk's app-layer model. Key design decision (verified
//! against auto-ai-agent): the `Profession` trait in auto-ai-agent is purely
//! static config (no `handoff_to`/`dispatchable_to`/`owned_sections`/`ForgePhase`),
//! and its `relay.rs` only defines a 101-line `RelayTarget` trait with runtime
//! handoff explicitly marked as a v2 concern. So we build the orchestration
//! layer entirely here in the musk app, not in auto-ai-agent.
//!
//! This module is the P2b.1 foundation: profession metadata (P2a) + the
//! pure state machine (PipelineEngine) + run store + REST/SSE endpoints.
//! A full background driver + AgentTurn ReAct loop arrives in P2b.2; for now
//! `advance` drives a step synchronously via `build_agent_from_mode`.

pub mod api;
pub mod budget;
pub mod driver;
pub mod flow;
pub mod handoff;
pub mod pipeline;
pub mod profession;
pub mod store;

pub use budget::{BudgetAction, BudgetTracker, TokenBudget};
pub use flow::{ExitRouting, FlowSpec, FlowStep, GateType};
pub use handoff::HandoffDocument;
pub use pipeline::{AdvanceResult, GateDecision, PipelineEngine, PipelineStatus, RelayMode};
pub use profession::{ForgePhase, Profession, ProfessionRegistry};
pub use store::{RunEvent, RunState, RunStore, RunSummary};
