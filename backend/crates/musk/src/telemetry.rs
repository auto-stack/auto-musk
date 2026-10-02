//! PLAN-098: run telemetry — decorator [`Client`] recording per-request model
//! attribution (actual serving model, usage, latency, stop reason) for chat
//! sessions and relay runs.
//!
//! Design (T-01 frozen, see docs/reports/098-plan-flow-telemetry-evidence.md):
//! - D1/D2: per-run wrapping with a build-time [`TelemetryContext`] — each
//!   instance owns its sink (`Arc<Mutex<Vec<TelemetryRecord>>>`), so concurrent
//!   sessions/runs cannot cross-attribute by construction. Injection points:
//!   `chat_run_owner` (chat) and `MuskAgentFactory::build_agent` (relay).
//!   No task-local: relay drivers have 8 spawn entry points, chat runs under a
//!   `tokio::select!` watchdog — explicit context is spawn-proof.
//! - D3: telemetry is the OUTERMOST decorator (relay plan flow:
//!   `TelemetryClient(PlanRuntimeClient(AiClient))`), so the recorded request
//!   model is the final outgoing value and the recorded response is verbatim.
//! - Side-channel only: sink failures never fail a turn (poisoned locks are
//!   recovered, recording never panics); missing wire facts are recorded as
//!   explicit nulls (the daemon's done frame carries no provider — it stays
//!   `None`; `model` follows `model_meta.id`, PLAN-031's authoritative
//!   actually-served field, and is `None` without it).

use std::sync::{Arc, Mutex};
use std::time::Instant;

use async_trait::async_trait;
use auto_ai_agent::Client;
use auto_ai_client::{ClientError, CompletionRequest, CompletionResponse};

/// Build-time correlation for a telemetry-wrapped client instance.
///
/// Chat runs set the session pair; relay steps set run/step/role. Serialized
/// into every record's `correlation` object (field names align with the
/// turns.jsonl telemetry contract in the plan §2 example).
#[derive(Debug, Clone, PartialEq)]
pub enum TelemetryContext {
    Chat {
        workspace_id: String,
        session_id: String,
    },
    Relay {
        run_id: String,
        step_id: String,
        role: String,
    },
}

impl TelemetryContext {
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Self::Chat {
                workspace_id,
                session_id,
            } => serde_json::json!({
                "kind": "chat",
                "workspace_id": workspace_id,
                "session_id": session_id,
            }),
            Self::Relay {
                run_id,
                step_id,
                role,
            } => serde_json::json!({
                "kind": "relay",
                "run_id": run_id,
                "step_id": step_id,
                "role": role,
            }),
        }
    }
}

/// One LLM request observed through the decorator (in-process shape; the
/// persisted shape is `to_json` / `fold_records`).
#[derive(Debug, Clone, PartialEq)]
pub struct TelemetryRecord {
    /// Epoch milliseconds at request completion.
    pub ts_ms: u64,
    pub elapsed_ms: u64,
    /// The model asked for in the outgoing request (e.g. `tier:mid`).
    pub requested_model: String,
    /// Provider that actually served — the wire carries no provider field
    /// (auto-ai PLAN-031 `ModelMeta`), so this is always `None` on the musk
    /// side today; the aaid log join in collect-telemetry.mjs attributes it
    /// with a confidence mark. Explicit null, never fabricated.
    pub provider: Option<String>,
    /// Actually-served model id (`model_meta.id`); `None` when the daemon
    /// sent no `model_meta` (per plan §5.1: record null, don't guess from
    /// `resp.model` — its provenance predates PLAN-031 and may echo the
    /// request on old daemons).
    pub model: Option<String>,
    pub in_tokens: Option<u64>,
    pub out_tokens: Option<u64>,
    pub stop_reason: Option<String>,
    pub stream: bool,
    /// In-band error from the response body, or the transport error string.
    pub error: Option<String>,
    pub correlation: TelemetryContext,
}

impl TelemetryRecord {
    /// Persisted per-request row (plan §2 field names).
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "ts_ms": self.ts_ms,
            "elapsed_ms": self.elapsed_ms,
            "requested_model": self.requested_model,
            "provider": self.provider,
            "model": self.model,
            "in_tokens": self.in_tokens,
            "out_tokens": self.out_tokens,
            "stop_reason": self.stop_reason,
            "stream": self.stream,
            "error": self.error,
            "correlation": self.correlation.to_json(),
        })
    }

    /// Compact `model_meta` payload for SSE `turn_end`/`done` enrichment
    /// (SD-02: present only when known, omitted otherwise).
    pub fn model_meta_json(&self) -> serde_json::Value {
        serde_json::json!({
            "provider": self.provider,
            "model": self.model,
            "usage": {"in": self.in_tokens, "out": self.out_tokens},
        })
    }
}

fn epoch_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Shared sink handle. `drain` is the fold-point primitive (chat completion
/// arms / relay step end): takes every record observed by this run so far.
#[derive(Clone, Default)]
pub struct TelemetrySink(Arc<Mutex<Vec<TelemetryRecord>>>);

impl TelemetrySink {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(Vec::new())))
    }

    fn push(&self, rec: TelemetryRecord) {
        // Poisoned lock = a panic while some thread held the vec — the data
        // is still consistent enough for telemetry; recover, never fail the
        // turn for a side-channel write.
        let mut v = self.0.lock().unwrap_or_else(|e| e.into_inner());
        v.push(rec);
    }

    /// All records so far (snapshot; does not clear).
    pub fn records(&self) -> Vec<TelemetryRecord> {
        self.0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// Take every record observed so far (fold point).
    pub fn drain(&self) -> Vec<TelemetryRecord> {
        std::mem::take(&mut *self.0.lock().unwrap_or_else(|e| e.into_inner()))
    }

    /// The most recent record, for SSE `turn_end`/`done` enrichment at the
    /// moment the boundary event fires (the just-finished request is last).
    pub fn latest(&self) -> Option<TelemetryRecord> {
        self.0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .last()
            .cloned()
    }
}

/// Decorator: wraps the inner chain, records every completion (both tracks)
/// into the instance sink. Recording never affects the call result.
pub struct TelemetryClient {
    inner: Arc<dyn Client>,
    correlation: TelemetryContext,
    sink: TelemetrySink,
}

impl TelemetryClient {
    pub fn new(inner: Arc<dyn Client>, correlation: TelemetryContext) -> Self {
        Self::with_sink(inner, correlation, TelemetrySink::new())
    }

    /// Wrap with a caller-owned sink — the relay factory shares the sink the
    /// driver created so `run_step` can drain it after the agent completes.
    pub fn with_sink(
        inner: Arc<dyn Client>,
        correlation: TelemetryContext,
        sink: TelemetrySink,
    ) -> Self {
        Self {
            inner,
            correlation,
            sink,
        }
    }

    pub fn sink(&self) -> TelemetrySink {
        self.sink.clone()
    }

    fn record(
        &self,
        req: &CompletionRequest,
        started: Instant,
        resp: Option<&CompletionResponse>,
        err: Option<&ClientError>,
        stream: bool,
    ) {
        let (provider, model, in_tokens, out_tokens, stop_reason, inband_error) = match resp {
            Some(r) => (
                None, // wire carries no provider (T-01 F9) — explicit null
                r.model_meta.as_ref().map(|m| m.id.clone()),
                r.usage.as_ref().map(|u| u.input_tokens as u64),
                r.usage.as_ref().map(|u| u.output_tokens as u64),
                r.stop_reason.clone(),
                r.error.clone(),
            ),
            None => (None, None, None, None, None, None),
        };
        let rec = TelemetryRecord {
            ts_ms: epoch_ms(),
            elapsed_ms: started.elapsed().as_millis() as u64,
            requested_model: req.model.clone(),
            provider,
            model,
            in_tokens,
            out_tokens,
            stop_reason,
            stream,
            error: inband_error.or_else(|| err.map(|e| e.to_string())),
            correlation: self.correlation.clone(),
        };
        self.sink.push(rec);
    }
}

#[async_trait]
impl Client for TelemetryClient {
    async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, ClientError> {
        let started = Instant::now();
        match self.inner.complete(req).await {
            Ok(resp) => {
                self.record(req, started, Some(&resp), None, false);
                Ok(resp)
            }
            Err(e) => {
                self.record(req, started, None, Some(&e), false);
                Err(e)
            }
        }
    }

    async fn complete_stream(
        &self,
        req: &CompletionRequest,
        on_event: Arc<dyn Fn(serde_json::Value) + Send + Sync>,
    ) -> Result<CompletionResponse, ClientError> {
        let started = Instant::now();
        match self.inner.complete_stream(req, on_event).await {
            Ok(resp) => {
                self.record(req, started, Some(&resp), None, true);
                Ok(resp)
            }
            Err(e) => {
                self.record(req, started, None, Some(&e), true);
                Err(e)
            }
        }
    }
}

/// Fold observed records into the persisted turn shape (plan §5.3): one
/// `telemetry` object per assistant turn — a single request flattens to its
/// row; multiple requests (tool loop) aggregate totals plus a per-request
/// `requests[]`. `None` when nothing was recorded (turn without telemetry —
/// the field is omitted at persist time, never written as an empty object).
pub fn fold_records(records: &[TelemetryRecord]) -> Option<serde_json::Value> {
    let first = records.first()?;
    let rows: Vec<serde_json::Value> = records.iter().map(|r| r.to_json()).collect();
    if records.len() == 1 {
        return rows.into_iter().next();
    }
    let sum = |f: fn(&TelemetryRecord) -> Option<u64>| -> Option<u64> {
        // 合计仅在至少一请求携带该字段时给出（全缺失 = null，不伪造 0）。
        let mut vals = records.iter().filter_map(|r| f(r));
        let first = vals.next()?;
        Some(first + vals.sum::<u64>())
    };
    Some(serde_json::json!({
        "provider": first.provider,
        "model": first.model,
        "requested_model": first.requested_model,
        "in_tokens": sum(|r| r.in_tokens),
        "out_tokens": sum(|r| r.out_tokens),
        "elapsed_ms": records.iter().map(|r| r.elapsed_ms).sum::<u64>(),
        "stop_reason": records.last().and_then(|r| r.stop_reason.clone()),
        "stream": records.iter().any(|r| r.stream),
        "error": records.iter().find_map(|r| r.error.clone()),
        "correlation": first.correlation.to_json(),
        "requests": rows,
    }))
}

// ── relay sink registry ────────────────────────────────────────────────────
// The relay factory wraps step clients with telemetry (T-02), while the fold
// point is `RunStore::submit_handoff` (T-03) — a different call chain. A
// run-keyed registry joins the two: one sink per run (steps are sequential
// within a run, concurrent runs get distinct keys → no cross-attribution),
// mirroring the plan_runtime_client keyed-registry pattern (PLAN-096 T-06).

fn run_sinks() -> &'static Mutex<std::collections::HashMap<String, TelemetrySink>> {
    static SINKS: std::sync::OnceLock<Mutex<std::collections::HashMap<String, TelemetrySink>>> =
        std::sync::OnceLock::new();
    SINKS.get_or_init(|| Mutex::new(std::collections::HashMap::new()))
}

/// The telemetry sink for a relay run (created on first use; the factory
/// wraps every step of the run into this same sink).
pub fn relay_sink(run_id: &str) -> TelemetrySink {
    let mut map = run_sinks().lock().unwrap_or_else(|e| e.into_inner());
    map.entry(run_id.to_string())
        .or_insert_with(TelemetrySink::new)
        .clone()
}

/// Drain a run's records at the fold point, keeping only the given step's
/// rows (a previously failed step leaves rows behind — they are dropped here,
/// the failure is already audited by RunFailed; current-step rows fold into
/// the phase boundary). Empty result leaves no stale entry behind.
pub fn relay_drain(run_id: &str, step_id: &str) -> Vec<TelemetryRecord> {
    let mut map = run_sinks().lock().unwrap_or_else(|e| e.into_inner());
    match map.remove(run_id) {
        Some(sink) => sink
            .drain()
            .into_iter()
            .filter(|r| match &r.correlation {
                TelemetryContext::Relay { step_id: s, .. } => s == step_id,
                _ => false,
            })
            .collect(),
        None => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use auto_ai_client::ModelMeta;

    #[derive(Default)]
    struct MockInner {
        resp: Option<CompletionResponse>,
        /// 模拟传输层错误（ClientError 无 Clone——按旗标现构造）。
        fail_transport: bool,
        seen_models: Mutex<Vec<String>>,
    }

    impl MockInner {
        fn ok(model_meta: Option<ModelMeta>, usage: Option<auto_ai_client::Usage>) -> Self {
            Self {
                resp: Some(CompletionResponse {
                    content: "ok".into(),
                    tool_calls: Vec::new(),
                    stop_reason: Some("end_turn".into()),
                    usage,
                    model: "mock".into(),
                    error: None,
                    model_meta,
                }),
                ..Default::default()
            }
        }
    }

    #[async_trait]
    impl Client for MockInner {
        async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, ClientError> {
            self.seen_models.lock().unwrap().push(req.model.clone());
            if self.fail_transport {
                return Err(ClientError::Api("mock transport error".into()));
            }
            Ok(self.resp.clone().unwrap())
        }
    }

    fn chat_ctx(sid: &str) -> TelemetryContext {
        TelemetryContext::Chat {
            workspace_id: "ws".into(),
            session_id: sid.into(),
        }
    }

    fn served_meta(id: &str) -> ModelMeta {
        ModelMeta {
            id: id.into(),
            context_window: 200_000,
            max_output_tokens: None,
        }
    }

    #[tokio::test]
    async fn captures_non_stream_and_stream_tracks() {
        let inner = Arc::new(MockInner::ok(Some(served_meta("glm-5.3")), None));
        let tel = TelemetryClient::new(inner.clone(), chat_ctx("s1"));
        tel.complete(&CompletionRequest::single("tier:mid", "hi")).await.unwrap();
        tel.complete_stream(
            &CompletionRequest::single("tier:mid", "hi2"),
            Arc::new(|_| {}),
        )
        .await
        .unwrap();
        let recs = tel.sink().records();
        assert_eq!(recs.len(), 2);
        assert!(!recs[0].stream, "complete() recorded as non-stream");
        assert!(recs[1].stream, "complete_stream() recorded as stream");
        assert_eq!(recs[0].model.as_deref(), Some("glm-5.3"));
        assert_eq!(recs[0].requested_model, "tier:mid");
    }

    #[tokio::test]
    async fn tier_fallback_records_actual_served_model() {
        // 回退场景：请求 tier:mid，实际服务 glm-5.3-flash（model_meta 真值）。
        let inner = Arc::new(MockInner::ok(Some(served_meta("glm-5.3-flash")), None));
        let tel = TelemetryClient::new(inner, chat_ctx("s1"));
        tel.complete(&CompletionRequest::single("tier:mid", "hi")).await.unwrap();
        let recs = tel.sink().records();
        assert_eq!(recs[0].model.as_deref(), Some("glm-5.3-flash"));
        assert_eq!(recs[0].requested_model, "tier:mid", "requested ≠ served");
    }

    #[tokio::test]
    async fn missing_meta_and_usage_recorded_as_explicit_nulls() {
        // model_meta 缺失 → provider/model 记 null（§5.1；resp.model 不可信
        // —— 其来源早于 PLAN-031，旧 daemon 可能回显请求值）。usage 缺失 → null。
        let inner = Arc::new(MockInner::ok(None, None));
        let tel = TelemetryClient::new(inner, chat_ctx("s1"));
        tel.complete(&CompletionRequest::single("m", "x")).await.unwrap();
        let v = tel.sink().records()[0].to_json();
        assert_eq!(v["provider"], serde_json::Value::Null);
        assert_eq!(v["model"], serde_json::Value::Null);
        assert_eq!(v["in_tokens"], serde_json::Value::Null);
        assert_eq!(v["out_tokens"], serde_json::Value::Null);
        // usage 缺失但流式 input=0 时如实记 0（in 字段存在值可为 0）。
        let inner = Arc::new(MockInner::ok(
            Some(served_meta("m1")),
            Some(auto_ai_client::Usage {
                input_tokens: 0,
                output_tokens: 699,
                ..Default::default()
            }),
        ));
        let tel = TelemetryClient::new(inner, chat_ctx("s2"));
        tel.complete_stream(&CompletionRequest::single("m", "x"), Arc::new(|_| {}))
            .await
            .unwrap();
        let v = tel.sink().records()[0].to_json();
        assert_eq!(v["in_tokens"], 0, "aaid 流式 in=0 如实记录");
        assert_eq!(v["out_tokens"], 699);
    }

    #[tokio::test]
    async fn error_response_recorded_and_passthrough() {
        // 带内错误（resp.error）——记录且原样透传，不改变调用结果。
        let mut m = MockInner::ok(Some(served_meta("m1")), None);
        m.resp.as_mut().unwrap().error = Some("boom".into());
        let tel = TelemetryClient::new(Arc::new(m), chat_ctx("s1"));
        let resp = tel.complete(&CompletionRequest::single("m", "x")).await.unwrap();
        assert_eq!(resp.error.as_deref(), Some("boom"));
        assert_eq!(tel.sink().records()[0].error.as_deref(), Some("boom"));
        // 传输层错误——记录后原样上抛。
        let inner = Arc::new(MockInner {
            fail_transport: true,
            ..Default::default()
        });
        let tel = TelemetryClient::new(inner, chat_ctx("s1"));
        assert!(tel.complete(&CompletionRequest::single("m", "x")).await.is_err());
        let recs = tel.sink().records();
        assert_eq!(recs.len(), 1, "transport error also recorded");
        assert!(recs[0].error.is_some());
    }

    #[tokio::test]
    async fn poisoned_sink_does_not_fail_the_turn() {
        let sink = TelemetrySink::new();
        let inner = Arc::new(MockInner::ok(Some(served_meta("m1")), None));
        let tel = TelemetryClient::with_sink(inner, chat_ctx("s1"), sink.clone());
        // 毒化锁：持锁线程 panic。
        let s2 = sink.clone();
        let poisoner = std::thread::spawn(move || {
            let _g = s2.records();
            let _g2 = s2.0.lock().unwrap();
            panic!("poison the mutex while held");
        });
        assert!(poisoner.join().is_err(), "poisoner must panic holding the lock");
        let resp = tel.complete(&CompletionRequest::single("m", "x")).await;
        assert!(resp.is_ok(), "telemetry side-channel must never fail the turn");
        assert_eq!(tel.sink().records().len(), 1, "recovered lock still records");
    }

    #[tokio::test]
    async fn concurrent_sessions_zero_cross_talk() {
        // 并发反例：两会话交错轮次，各自 sink 只有自己的行（结构保证的实证）。
        let a = TelemetryClient::new(
            Arc::new(MockInner::ok(Some(served_meta("model-a")), None)),
            chat_ctx("session-a"),
        );
        let b = TelemetryClient::new(
            Arc::new(MockInner::ok(Some(served_meta("model-b")), None)),
            chat_ctx("session-b"),
        );
        let (ta, tb) = (a.sink(), b.sink());
        let ha = tokio::spawn(async move {
            for i in 0..5 {
                a.complete(&CompletionRequest::single("m", &format!("a{i}"))).await.unwrap();
            }
        });
        let hb = tokio::spawn(async move {
            for i in 0..5 {
                b.complete(&CompletionRequest::single("m", &format!("b{i}"))).await.unwrap();
            }
        });
        let (ra, rb) = tokio::join!(ha, hb);
        ra.unwrap();
        rb.unwrap();
        for r in ta.records() {
            assert_eq!(r.correlation, chat_ctx("session-a"), "A sink 只记 A");
        }
        for r in tb.records() {
            assert_eq!(r.correlation, chat_ctx("session-b"), "B sink 只记 B");
        }
        assert_eq!(ta.records().len(), 5);
        assert_eq!(tb.records().len(), 5);
    }

    #[test]
    fn fold_single_flattens_and_multi_aggregates() {
        let mk = |model: &str, out: u64, ms: u64| TelemetryRecord {
            ts_ms: 0,
            elapsed_ms: ms,
            requested_model: "tier:mid".into(),
            provider: None,
            model: Some(model.into()),
            in_tokens: None,
            out_tokens: Some(out),
            stop_reason: Some("end_turn".into()),
            stream: true,
            error: None,
            correlation: chat_ctx("s1"),
        };
        // 单请求 → 扁平一行（§2 形状）。
        let one = fold_records(&[mk("glm-5.3", 699, 19_615)]).unwrap();
        assert_eq!(one["model"], "glm-5.3");
        assert_eq!(one["out_tokens"], 699);
        assert!(one.get("requests").is_none(), "单请求不嵌 requests[]");
        // 多请求（工具循环）→ totals + requests[]。
        let multi = fold_records(&[mk("m1", 100, 1_000), mk("m2", 200, 2_000), mk("m3", 300, 4_000)])
            .unwrap();
        assert_eq!(multi["out_tokens"], 600, "轮级 out 合计");
        assert_eq!(multi["elapsed_ms"], 7_000, "轮级耗时合计");
        assert_eq!(multi["requests"].as_array().unwrap().len(), 3);
        assert_eq!(multi["correlation"]["session_id"], "s1");
        assert_eq!(fold_records(&[]), None, "无记录 → None（字段整体省略）");
    }

    #[tokio::test]
    async fn relay_correlation_carries_run_step_role() {
        let inner = Arc::new(MockInner::ok(Some(served_meta("m1")), None));
        let tel = TelemetryClient::new(
            inner,
            TelemetryContext::Relay {
                run_id: "run-1".into(),
                step_id: "execute".into(),
                role: "coder".into(),
            },
        );
        tel.complete(&CompletionRequest::single("m", "x")).await.unwrap();
        let v = tel.sink().records()[0].to_json();
        assert_eq!(v["correlation"]["run_id"], "run-1");
        assert_eq!(v["correlation"]["step_id"], "execute");
        assert_eq!(v["correlation"]["role"], "coder");
    }
}
