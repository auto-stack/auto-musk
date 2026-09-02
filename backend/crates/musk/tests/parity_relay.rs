//! parity_relay.rs — Plan 018 §11 ③ (relay/agent/ctx extern 委托) 的镜像 parity。
//!
//! The transpiled relay/agent mirrors (auto_generated::relay_flows /
//! relay_profession / auto_lib / relay_driver) are compiled-but-dormant; their
//! extern stubs now delegate to the same hand-written logic as the live hw
//! paths (src/lib.rs, src/relay/driver.rs). These tests exercise the mirrors
//! directly and assert they behave like hw:
//! - relay_flows: transpiled flow builders produce byte-equal FlowSpec data.
//! - relay_profession: transpiled registry logic agrees with hw semantics.
//! - auto_lib / relay_driver: transpiled agent factories (via delegated
//!   externs build_agent_with_context / handoff_render / agent_with_history /
//!   agent_register_shared / ...) build agents equivalent to hw.

use std::sync::Arc;

use auto_ai_agent::{Agent, AgentFactory, Client, HandoffDocument};
use auto_ai_client::{ClientError, CompletionRequest, CompletionResponse};
use musk::auto_generated::auto_lib as ag_lib;
use musk::auto_generated::extern_impl as ag_extern;
use musk::auto_generated::relay_driver as ag_driver;
use musk::auto_generated::relay_flows as ag_flows;
use musk::auto_generated::relay_profession as ag_prof;
use musk::mode::AgentMode;
use musk::relay::driver as hw_driver;
use musk::relay::flows as hw_flows;
use musk::relay::profession as hw_prof;
use musk::server::AppState;

/// Deterministic mock client — agent construction never calls it.
struct MockClient;

#[async_trait::async_trait]
impl Client for MockClient {
    async fn complete(&self, _req: &CompletionRequest) -> Result<CompletionResponse, ClientError> {
        Err(ClientError::DaemonUnavailable)
    }
}

/// Registered tool names as an ordered set (HashMap iteration order in
/// ToolRegistry::names() is not stable across instances).
fn tool_set(agent: &Agent) -> std::collections::BTreeSet<String> {
    agent.tools().names().into_iter().collect()
}

fn test_state() -> AppState {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "musk-parity-relay-{}-{}",
        std::process::id(),
        n
    ));
    let _ = std::fs::create_dir_all(&dir);
    let registry =
        musk::workspace::WorkspaceRegistry::load(dir.join("workspaces.json"), dir.clone());
    AppState {
        client: Arc::new(MockClient) as Arc<dyn Client>,
        auth: Arc::new(musk::auto_generated::auth::AuthStore::new(dir.join("users.json"))),
        registry: Arc::new(registry),
        chat_runs: Arc::new(std::sync::Mutex::new(std::collections::HashSet::new())),
    }
}

/// hw Profession ground truth → ag Profession via wire round-trip. This also
/// asserts the two structs serialize identically (SectionType / ForgePhase
/// wire parity).
fn hw_defaults_as_ag() -> Vec<ag_prof::Profession> {
    let hw_defaults: Vec<hw_prof::Profession> = hw_prof::default_professions();
    hw_defaults
        .iter()
        .map(|p| serde_json::from_value(serde_json::to_value(p).unwrap()).unwrap())
        .collect()
}

#[test]
fn parity_builtin_flows_match() {
    let hw_list = hw_flows::builtin_flows();
    let ag_list = ag_flows::builtin_flows();
    assert_eq!(ag_list.len(), hw_list.len(), "same number of builtin flows");

    for (ag, hw) in ag_list.iter().zip(hw_list.iter()) {
        assert_eq!(
            serde_json::to_value(ag).unwrap(),
            serde_json::to_value(hw).unwrap(),
            "flow wire mismatch (same auto_ai_agent FlowSpec type, so this is the \
             transpiled builder producing identical data)"
        );
    }

    for flow in &hw_list {
        assert!(
            ag_flows::get_builtin_flow(&flow.id).is_some(),
            "ag resolves builtin flow '{}'",
            flow.id
        );
        assert!(
            hw_flows::get_builtin_flow(&flow.id).is_some(),
            "hw resolves builtin flow '{}'",
            flow.id
        );
    }
    assert_eq!(
        ag_flows::get_builtin_flow("no-such-flow").is_some(),
        hw_flows::get_builtin_flow("no-such-flow").is_some(),
        "unknown flow resolves the same way on both sides"
    );
}

#[test]
fn parity_profession_registry_matches_hw_semantics() {
    let hw_defaults: Vec<hw_prof::Profession> = hw_prof::default_professions();
    let ag_defaults = hw_defaults_as_ag();
    assert!(!ag_defaults.is_empty(), "hw default professions seed is non-empty");

    let ag_reg = ag_prof::ProfessionRegistry { professions: ag_defaults.clone() };

    // get() returns the same wire data as the hw ground-truth entry.
    for p in &hw_defaults {
        let ag_got = ag_reg.get(&p.id).unwrap();
        assert_eq!(
            serde_json::to_value(ag_got).unwrap(),
            serde_json::to_value(p).unwrap(),
            "get('{}') wire mismatch",
            p.id
        );
    }
    assert_eq!(ag_reg.list().len(), hw_defaults.len());

    // can_handoff / needs_approval agree with the reference semantics over hw data.
    let ids: Vec<&str> = hw_defaults.iter().map(|p| p.id.as_str()).collect();
    for from in &ids {
        for to in &ids {
            let hw_from = hw_defaults.iter().find(|p| p.id == *from);
            let expect_handoff = hw_from.map_or(false, |p| p.handoff_to.iter().any(|h| h == *to));
            assert_eq!(
                ag_reg.can_handoff(from, to),
                expect_handoff,
                "can_handoff({from},{to})"
            );
            let expect_approval =
                hw_from.map_or(false, |p| p.approval_gates.iter().any(|g| g == *to));
            assert_eq!(
                ag_reg.needs_approval(from, to),
                expect_approval,
                "needs_approval({from},{to})"
            );
        }
    }

    // register(): upsert replaces same-id, appends new.
    let mut ag_reg = ag_prof::ProfessionRegistry { professions: vec![] };
    let sample = ag_defaults[0].clone();
    ag_reg.register(sample.clone());
    ag_reg.register(sample.clone());
    assert_eq!(ag_reg.list().len(), 1, "re-register replaces same id");
    ag_reg.register(ag_defaults[1].clone());
    assert_eq!(ag_reg.list().len(), 2, "new id appends");
    assert_eq!(ag_reg.get(&sample.id).unwrap().id, sample.id);
}

/// ag auto_lib::build_agent_from_mode (delegated externs) registers the same
/// tool set as hw lib::build_agent_from_mode.
#[test]
fn parity_build_agent_from_mode_registers_same_tools() {
    let mode = AgentMode {
        name: "parity-test".into(),
        description: String::new(),
        role: "gofer".into(),
        skills: false,
        tools: vec![],
        workflow: None,
        context_file: String::new(),
        extra_system_prompt: String::new(),
    };

    let ag_agent = ag_lib::build_agent_from_mode(mode.clone(), Arc::new(MockClient)).unwrap();
    let hw_agent = musk::build_agent_from_mode(&mode, Arc::new(MockClient)).unwrap();

    assert_eq!(
        tool_set(&ag_agent),
        tool_set(&hw_agent),
        "registered tool sets must match"
    );
    assert!(tool_set(&hw_agent).contains("read_file"), "hw has base toolset");
    assert!(tool_set(&ag_agent).contains("run_command"), "ag has base toolset");
}

/// ag extern factory_build_agent delegates to the hw MuskAgentFactory with the
/// same role and no prior handoff → equivalent agent.
#[tokio::test]
async fn parity_factory_build_agent_matches_hw_factory() {
    let state = Arc::new(test_state());
    let ag_agent = ag_extern::factory_build_agent(&state, "ws-parity", "run-1", "coder").await;
    let hw_factory = hw_driver::MuskAgentFactory {
        state: state.clone(),
        workspace_id: "ws-parity".into(),
        run_id: "run-1".into(),
    };
    let hw_agent = hw_factory.build_agent("coder", None).unwrap();

    assert_eq!(tool_set(&ag_agent), tool_set(&hw_agent), "factory-built agents match");
    assert!(
        tool_set(&ag_agent).contains("spawn_relay"),
        "PLAN-030: spawn_relay is registered again — it is the entry point into \
         the plan-driven dev flow (flow_id=\"plan\")"
    );
}

/// The transpiled relay_driver::MuskAgentFactory mirror (exercising the
/// delegated build_agent_with_context / handoff_render / agent_with_history
/// externs) builds agents equivalent to the hw factory, with and without a
/// prior handoff.
#[tokio::test]
async fn parity_relay_driver_factory_build_agent() {
    let state = Arc::new(test_state());
    let handoff = HandoffDocument::new("architect", "coder");
    let handoff_json = serde_json::to_string(&handoff).unwrap();

    let ag_factory = ag_driver::MuskAgentFactory {
        state: state.clone(),
        workspace_id: "ws-parity".into(),
        run_id: "run-1".into(),
    };
    let hw_factory = hw_driver::MuskAgentFactory {
        state: state.clone(),
        workspace_id: "ws-parity".into(),
        run_id: "run-1".into(),
    };

    // No prior handoff.
    let ag_agent = ag_factory.build_agent("coder", None).unwrap();
    let hw_agent = hw_factory.build_agent("coder", None).unwrap();
    assert_eq!(tool_set(&ag_agent), tool_set(&hw_agent), "no-handoff build matches");

    // With a prior handoff (ag passes serialized JSON, hw passes the doc).
    let ag_agent = ag_factory.build_agent("coder", Some(handoff_json)).unwrap();
    let hw_agent = hw_factory.build_agent("coder", Some(&handoff)).unwrap();
    assert_eq!(tool_set(&ag_agent), tool_set(&hw_agent), "handoff build matches");
}
