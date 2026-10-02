//! PLAN-096 T-06: plan-flow runtime client wrapper — the musk-local output
//! budget + truncation registry (§5.5, AC-09).
//!
//! The plan phases get an explicit output ceiling: requests that carry no
//! explicit `max_tokens` are topped up to the effective budget
//! (`MUSK_PLAN_MAX_TOKENS`, default 16384 — deployable smaller/larger; the
//! effective value is recorded, never a cost promise). Existing explicit
//! values pass through untouched. Global daemon / other flows are NOT
//! changed: the wrapper is injected only by the relay factory for runs with
//! plan execution (both tracks — the ag factory delegates to the hw one).
//!
//! Responses whose `stop_reason` is a truncation shape (`max_tokens`,
//! `length`, `max_output_tokens`, `truncated`) are registered per run as
//! incomplete: the controller refuses to route the phase on a truncated
//! response alone and grants ONE bounded continuation per stage (reading the
//! on-disk artifacts, never re-creating the plan) before failing loud with
//! the artifact location. Unknown stop reasons still go through normal
//! stage-artifact verification — absence of an explicit stop reason is not
//! evidence of failure.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use async_trait::async_trait;
use auto_ai_agent::Client;
use auto_ai_client::{ClientError, CompletionRequest, CompletionResponse};

/// Effective output budget for plan phases (Q-03): env override first, then
/// the recorded product default. The value is advisory to the provider —
/// acceptance is verified live in T-12 (L1), never assumed.
pub fn effective_max_tokens() -> usize {
    std::env::var("MUSK_PLAN_MAX_TOKENS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(16384)
}

/// Stop reasons that mean "the response was cut off by the output budget".
fn is_truncation(stop_reason: &str) -> bool {
    matches!(
        stop_reason.trim().to_ascii_lowercase().as_str(),
        "max_tokens" | "length" | "max_output_tokens" | "truncated"
    )
}

#[derive(Debug, Clone, PartialEq)]
pub struct StopRecord {
    pub truncated: bool,
    pub stop_reason: Option<String>,
    pub stream: bool,
}

fn logs() -> &'static Mutex<HashMap<String, Vec<StopRecord>>> {
    static LOGS: OnceLock<Mutex<HashMap<String, Vec<StopRecord>>>> = OnceLock::new();
    LOGS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn record(run_id: &str, rec: StopRecord) {
    logs().lock().unwrap().entry(run_id.to_string()).or_default().push(rec);
}

/// Any truncation-shape response recorded for this run since the last clear.
pub fn truncated_for(run_id: &str) -> bool {
    logs()
        .lock()
        .unwrap()
        .get(run_id)
        .map(|v| v.iter().any(|r| r.truncated))
        .unwrap_or(false)
}

/// The full stop-record log for diagnostics (T-12 evidence).
pub fn records_for(run_id: &str) -> Vec<StopRecord> {
    logs().lock().unwrap().get(run_id).cloned().unwrap_or_default()
}

/// Clear the run's log (phase boundary / after a continuation is granted).
pub fn clear(run_id: &str) {
    logs().lock().unwrap().remove(run_id);
}

/// Wrap an inner client for one plan-flow run: tops up missing `max_tokens`,
/// records stop reasons. Event/tool/usage/model-chain fidelity is preserved
/// by delegation (stream events forwarded verbatim).
pub struct PlanRuntimeClient {
    inner: Arc<dyn Client>,
    run_id: String,
    max_tokens: usize,
}

impl PlanRuntimeClient {
    pub fn new(inner: Arc<dyn Client>, run_id: &str, max_tokens: usize) -> Self {
        Self {
            inner,
            run_id: run_id.to_string(),
            max_tokens,
        }
    }
}

#[async_trait]
impl Client for PlanRuntimeClient {
    async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, ClientError> {
        let mut req = req.clone();
        if req.max_tokens.is_none() {
            req.max_tokens = Some(self.max_tokens);
        }
        let resp = self.inner.complete(&req).await?;
        record(
            &self.run_id,
            StopRecord {
                truncated: resp
                    .stop_reason
                    .as_deref()
                    .map(is_truncation)
                    .unwrap_or(false),
                stop_reason: resp.stop_reason.clone(),
                stream: false,
            },
        );
        Ok(resp)
    }

    async fn complete_stream(
        &self,
        req: &CompletionRequest,
        on_event: Arc<dyn Fn(serde_json::Value) + Send + Sync>,
    ) -> Result<CompletionResponse, ClientError> {
        let mut req = req.clone();
        if req.max_tokens.is_none() {
            req.max_tokens = Some(self.max_tokens);
        }
        let resp = self.inner.complete_stream(&req, on_event).await?;
        record(
            &self.run_id,
            StopRecord {
                truncated: resp
                    .stop_reason
                    .as_deref()
                    .map(is_truncation)
                    .unwrap_or(false),
                stop_reason: resp.stop_reason.clone(),
                stream: true,
            },
        );
        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockInner {
        stop: Option<String>,
        seen_max_tokens: Mutex<Vec<Option<usize>>>,
        stream: bool,
    }

    #[async_trait]
    impl Client for MockInner {
        async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, ClientError> {
            self.seen_max_tokens.lock().unwrap().push(req.max_tokens);
            Ok(CompletionResponse {
                content: "ok".into(),
                tool_calls: Vec::new(),
                stop_reason: self.stop.clone(),
                usage: None,
                model: "mock".into(),
                error: None,
                model_meta: None,
            })
        }
    }

    #[tokio::test]
    async fn tops_up_missing_max_tokens_and_records_stop() {
        let run = "run-budget-1";
        clear(run);
        let inner = Arc::new(MockInner {
            stop: Some("end_turn".into()),
            seen_max_tokens: Mutex::new(Vec::new()),
            stream: false,
        });
        let wrapped = Arc::new(PlanRuntimeClient::new(
            inner.clone() as Arc<dyn Client>,
            run,
            effective_max_tokens(),
        ));
        let mut req = CompletionRequest::single("tier:max", "hi");
        assert!(req.max_tokens.is_none());
        let resp = wrapped.complete(&req).await.unwrap();
        assert_eq!(resp.stop_reason.as_deref(), Some("end_turn"));
        assert_eq!(
            *inner.seen_max_tokens.lock().unwrap(),
            vec![Some(effective_max_tokens())],
            "wrapper must top up the missing budget"
        );
        assert!(!truncated_for(run), "end_turn is not truncation");
        // 显式值不覆盖（Q-03）。
        req.max_tokens = Some(77);
        wrapped.complete(&req).await.unwrap();
        assert_eq!(*inner.seen_max_tokens.lock().unwrap().last().unwrap(), Some(77));
        assert_eq!(records_for(run).len(), 2);
        clear(run);
    }

    #[tokio::test]
    async fn truncation_shapes_are_registered_incomplete() {
        let run = "run-budget-2";
        clear(run);
        for (reason, want) in [
            (Some("max_tokens"), true),
            (Some("length"), true),
            (Some("MAX_TOKENS"), true),
            (Some("max_output_tokens"), true),
            (Some("truncated"), true),
            (Some("end_turn"), false),
            (Some("tool_use"), false),
            (None, false),
        ] {
            clear(run);
            let inner = Arc::new(MockInner {
                stop: reason.map(str::to_string),
                seen_max_tokens: Mutex::new(Vec::new()),
                stream: false,
            });
            let wrapped = Arc::new(PlanRuntimeClient::new(inner, run, 1024));
            wrapped.complete(&CompletionRequest::single("m", "x")).await.unwrap();
            assert_eq!(truncated_for(run), want, "stop_reason {reason:?}");
        }
        clear(run);
    }

    #[tokio::test]
    async fn stream_track_records_too_and_forwards_events() {
        let run = "run-budget-3";
        clear(run);
        // 默认 trait 实现走 complete 回退——事件保真由委托结构保证；这里
        // 验证流式路径同样落 stop 记录（stream=true）。
        let inner = Arc::new(MockInner {
            stop: Some("max_tokens".into()),
            seen_max_tokens: Mutex::new(Vec::new()),
            stream: true,
        });
        let wrapped = Arc::new(PlanRuntimeClient::new(inner, run, 2048));
        let events = Arc::new(Mutex::new(Vec::new()));
        let sink = events.clone();
        let on_event: Arc<dyn Fn(serde_json::Value) + Send + Sync> =
            Arc::new(move |v| sink.lock().unwrap().push(v));
        let resp = wrapped.complete_stream(&CompletionRequest::single("m", "x"), on_event).await.unwrap();
        assert_eq!(resp.stop_reason.as_deref(), Some("max_tokens"));
        assert!(truncated_for(run));
        let recs = records_for(run);
        assert_eq!(recs.len(), 1);
        assert!(recs[0].stream, "stream track marked");
        clear(run);
    }
}
