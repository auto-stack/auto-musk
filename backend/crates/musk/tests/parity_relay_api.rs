//! parity_relay_api.rs — Plan 020 Phase D: relay_api.at HTTP 层等价测试。
//!
//! hw `relay::api::relay_routes/task_plan_routes` vs ag
//! `auto_generated::relay_api::relay_routes/task_plan_routes` 双 router 对照:
//! 同一请求序列跑两边,比较**状态码 + wire 形状**。
//!
//! - stateless 端点(professions / souls / flows / 空列表)逐键等价断言。
//! - stateful 端点(run lifecycle)含时间戳/run_id,按 key 结构 + 语义值断言
//!   (run_id 用响应里取出的值,后续请求参数化)。
//! - advance/gate-resolve 会 tokio::spawn 后台 driver —— 响应是立即 snapshot,
//!   只断言状态码 + run_id;后台异步效果不跨边比较(同一 extern → 同一 driver)。
//! - 错误路径(404 run / 400 decision / 400 handoff)精确比对状态码 + 文本 body。

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use serde_json::{json, Value};
use tower::ServiceExt;

use auto_ai_agent::Client;
use auto_ai_client::{ClientError, CompletionRequest, CompletionResponse};
use musk::server::AppState;

struct MockClient;
#[async_trait::async_trait]
impl Client for MockClient {
    async fn complete(&self, _req: &CompletionRequest) -> Result<CompletionResponse, ClientError> {
        Err(ClientError::DaemonUnavailable)
    }
}

fn tmp_state() -> AppState {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "musk-parity-relay-api-{}-{}",
        std::process::id(),
        n
    ));
    let _ = std::fs::create_dir_all(&dir);
    let registry = musk::workspace::WorkspaceRegistry::load(dir.join("workspaces.json"), dir.clone());
    AppState {
        client: Arc::new(MockClient) as Arc<dyn Client>,
        auth: Arc::new(musk::auto_generated::auth::AuthStore::new(dir.join("users.json"))),
        registry: Arc::new(registry),
    }
}

fn hw_app(state: AppState) -> Router {
    musk::relay::api::relay_routes()
        .merge(musk::relay::api::task_plan_routes())
        .with_state(state)
}

fn ag_app(state: AppState) -> Router {
    musk::auto_generated::relay_api::relay_routes()
        .merge(musk::auto_generated::relay_api::task_plan_routes())
        .with_state(state)
}

/// Send a JSON request; return (status, parsed body). Empty body → Value::Null。
async fn send(app: &Router, method: &str, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    let resp = match body {
        Some(b) => {
            builder = builder.header("content-type", "application/json");
            app.clone()
                .oneshot(builder.body(Body::from(b.to_string())).unwrap())
                .await
                .unwrap()
        }
        None => app
            .clone()
            .oneshot(builder.body(Body::empty()).unwrap())
            .await
            .unwrap(),
    };
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
    // 非 JSON body(hw relay 的 404/400 是纯文本)→ 回退 Value::String。
    let json = if bytes.is_empty() {
        Value::Null
    } else {
        match serde_json::from_slice::<Value>(&bytes) {
            Ok(v) => v,
            Err(_) => Value::String(String::from_utf8_lossy(&bytes).into_owned()),
        }
    };
    (status, json)
}

/// 断言 run state 的 wire 形状(hw/ag 共同结构),忽略 run_id/timestamp。
fn assert_run_state_shape(status: &str, body: &Value) {
    assert_eq!(body["run_id"].as_str().map(|s| !s.is_empty()), Some(true), "run_id present");
    assert_eq!(body["status"], status);
    assert!(body["steps"].is_array(), "steps array");
    assert!(body["total_steps"].as_u64().is_some(), "total_steps");
    assert!(body["cumulative_tokens"].as_u64().is_some(), "cumulative_tokens");
    // title 字段 skip_serializing_if None —— 未设标题时不出现。
    assert!(body.get("title").is_none() || body["title"].is_string());
}

// ── Relay run lifecycle ─────────────────────────────────────────────────────

#[tokio::test]
async fn relay_run_lifecycle_hw_vs_ag() {
    let hw = hw_app(tmp_state());
    let ag = ag_app(tmp_state());

    // list(空)→ 两边 {runs: []} 逐键等价。
    let (s_hw, b_hw) = send(&hw, "GET", "/api/forge/relay/runs", None).await;
    let (s_ag, b_ag) = send(&ag, "GET", "/api/forge/relay/runs", None).await;
    assert_eq!((s_hw, &b_hw), (s_ag, &b_ag), "empty runs list parity");
    assert_eq!(b_hw["runs"], json!([]));

    // start → {run_id, state};两边状态都是 idle,结构一致。
    let (s_hw, b_hw) = send(&hw, "POST", "/api/forge/relay/runs", Some(json!({"flow_id": "simple"}))).await;
    let (s_ag, b_ag) = send(&ag, "POST", "/api/forge/relay/runs", Some(json!({"flow_id": "simple"}))).await;
    assert_eq!(s_hw, StatusCode::OK);
    assert_eq!(s_ag, StatusCode::OK);
    let run_hw = b_hw["run_id"].as_str().unwrap().to_string();
    let run_ag = b_ag["run_id"].as_str().unwrap().to_string();
    assert_eq!(b_hw["state"]["status"], "idle");
    assert_eq!(b_ag["state"]["status"], "idle");
    assert_eq!(b_hw["state"]["steps"][0]["role_id"], b_ag["state"]["steps"][0]["role_id"]);

    // start 后的列表非空,两边各含自己的 run。
    let (_, b_hw) = send(&hw, "GET", "/api/forge/relay/runs", None).await;
    let (_, b_ag) = send(&ag, "GET", "/api/forge/relay/runs", None).await;
    assert_eq!(b_hw["runs"][0]["run_id"], run_hw);
    assert_eq!(b_ag["runs"][0]["run_id"], run_ag);
    assert_eq!(b_hw["runs"][0]["status"], b_ag["runs"][0]["status"]);

    // get → 200 + state 结构。
    let (s_hw, b_hw) = send(&hw, "GET", &format!("/api/forge/relay/runs/{run_hw}"), None).await;
    let (s_ag, b_ag) = send(&ag, "GET", &format!("/api/forge/relay/runs/{run_ag}"), None).await;
    assert_eq!(s_hw, StatusCode::OK);
    assert_eq!(s_ag, StatusCode::OK);
    assert_run_state_shape("idle", &b_hw);
    assert_run_state_shape("idle", &b_ag);

    // get 缺失 run → 404,纯文本 body 两边一致。
    let (s_hw, b_hw) = send(&hw, "GET", "/api/forge/relay/runs/nope", None).await;
    let (s_ag, b_ag) = send(&ag, "GET", "/api/forge/relay/runs/nope", None).await;
    assert_eq!((s_hw, &b_hw), (s_ag, &b_ag), "404 get parity");
    assert_eq!(s_hw, StatusCode::NOT_FOUND);
    assert_eq!(b_hw, json!("run 'nope' not found"));

    // title → 200 + title 出现在 state 里。
    let (s_hw, b_hw) = send(&hw, "PATCH", &format!("/api/forge/relay/runs/{run_hw}/title"), Some(json!({"title": "My Run"}))).await;
    let (s_ag, b_ag) = send(&ag, "PATCH", &format!("/api/forge/relay/runs/{run_ag}/title"), Some(json!({"title": "My Run"}))).await;
    assert_eq!(s_hw, StatusCode::OK);
    assert_eq!(s_ag, StatusCode::OK);
    assert_eq!(b_hw["title"], "My Run");
    assert_eq!(b_ag["title"], "My Run");
    // title 后 list 的 summary 带 title。
    let (_, b_hw) = send(&hw, "GET", "/api/forge/relay/runs", None).await;
    let (_, b_ag) = send(&ag, "GET", "/api/forge/relay/runs", None).await;
    assert_eq!(b_hw["runs"][0]["title"], "My Run");
    assert_eq!(b_ag["runs"][0]["title"], "My Run");

    // advance → 200(后台 driver 异步推进;只断言状态码 + run_id)。
    let (s_hw, b_hw) = send(&hw, "POST", &format!("/api/forge/relay/runs/{run_hw}/advance"), None).await;
    let (s_ag, b_ag) = send(&ag, "POST", &format!("/api/forge/relay/runs/{run_ag}/advance"), None).await;
    assert_eq!(s_hw, StatusCode::OK);
    assert_eq!(s_ag, StatusCode::OK);
    assert_eq!(b_hw["run_id"], run_hw);
    assert_eq!(b_ag["run_id"], run_ag);

    // submit_handoff(合法 handoff)→ 200;step_history 记录 + 路由到下一步。
    let mut h = musk::relay::HandoffDocument::new("advisor", "coder");
    h.summary = "done".into();
    let handoff_json = serde_json::to_value(&h).unwrap();
    let (s_hw, b_hw) = send(&hw, "POST", &format!("/api/forge/relay/runs/{run_hw}/handoff"), Some(json!({"handoff": handoff_json.clone()}))).await;
    let (s_ag, b_ag) = send(&ag, "POST", &format!("/api/forge/relay/runs/{run_ag}/handoff"), Some(json!({"handoff": handoff_json}))).await;
    assert_eq!(s_hw, StatusCode::OK);
    assert_eq!(s_ag, StatusCode::OK);
    assert!(b_hw["step_history"].is_array());
    assert_eq!(b_ag["step_history"].as_array().map(|a| a.len()), b_hw["step_history"].as_array().map(|a| a.len()));
    assert_eq!(b_ag["status"], b_hw["status"], "post-handoff status parity");

    // handoff 非法 JSON → 400(invalid handoff 文本,两边一致)。
    let (s_hw, b_hw) = send(&hw, "POST", &format!("/api/forge/relay/runs/{run_hw}/handoff"), Some(json!({"handoff": {"from": "x"}}))).await;
    let (s_ag, b_ag) = send(&ag, "POST", &format!("/api/forge/relay/runs/{run_ag}/handoff"), Some(json!({"handoff": {"from": "x"}}))).await;
    assert_eq!((s_hw, &b_hw), (s_ag, &b_ag), "400 handoff parity");
    assert_eq!(s_hw, StatusCode::BAD_REQUEST);
    assert!(b_hw.as_str().unwrap_or("").contains("invalid handoff"));

    // gate:未知 decision → 400 精确文本。
    let (s_hw, b_hw) = send(&hw, "POST", &format!("/api/forge/relay/runs/{run_hw}/gate"), Some(json!({"decision": "bogus"}))).await;
    let (s_ag, b_ag) = send(&ag, "POST", &format!("/api/forge/relay/runs/{run_ag}/gate"), Some(json!({"decision": "bogus"}))).await;
    assert_eq!((s_hw, &b_hw), (s_ag, &b_ag), "400 gate parity");
    assert_eq!(s_hw, StatusCode::BAD_REQUEST);
    assert_eq!(b_hw, json!("unknown gate decision 'bogus' (want approve|reject|edit)"));

    // gate:合法 decision 但 run 不在 gate → 200(引擎 resolve 处理,行为同 hw)。
    let (s_hw, _) = send(&hw, "POST", &format!("/api/forge/relay/runs/{run_hw}/gate"), Some(json!({"decision": "approve"}))).await;
    let (s_ag, _) = send(&ag, "POST", &format!("/api/forge/relay/runs/{run_ag}/gate"), Some(json!({"decision": "approve"}))).await;
    assert_eq!(s_hw, StatusCode::OK);
    assert_eq!(s_ag, StatusCode::OK);

    // rerun → 200。
    let (s_hw, b_hw) = send(&hw, "POST", &format!("/api/forge/relay/runs/{run_hw}/rerun"), None).await;
    let (s_ag, b_ag) = send(&ag, "POST", &format!("/api/forge/relay/runs/{run_ag}/rerun"), None).await;
    assert_eq!(s_hw, StatusCode::OK);
    assert_eq!(s_ag, StatusCode::OK);
    assert_eq!(b_hw["run_id"], run_hw);
    assert_eq!(b_ag["run_id"], run_ag);

    // delete → 200 {status: deleted, id}(id 是各自的 run_id)。
    let (s_hw, b_hw) = send(&hw, "DELETE", &format!("/api/forge/relay/runs/{run_hw}"), None).await;
    let (s_ag, b_ag) = send(&ag, "DELETE", &format!("/api/forge/relay/runs/{run_ag}"), None).await;
    assert_eq!(s_hw, s_ag, "delete status parity");
    assert_eq!(s_hw, StatusCode::OK);
    assert_eq!(b_hw["status"], "deleted");
    assert_eq!(b_ag["status"], "deleted");
    assert_eq!(b_hw["id"], run_hw);
    assert_eq!(b_ag["id"], run_ag);

    // delete 已删 → 404(文本 body 含各自 run_id,只比状态码 + 前缀)。
    let (s_hw, b_hw) = send(&hw, "DELETE", &format!("/api/forge/relay/runs/{run_hw}"), None).await;
    let (s_ag, b_ag) = send(&ag, "DELETE", &format!("/api/forge/relay/runs/{run_ag}"), None).await;
    assert_eq!(s_hw, s_ag, "double-delete status parity");
    assert_eq!(s_hw, StatusCode::NOT_FOUND);
    assert!(b_hw.as_str().unwrap_or("").starts_with("run '"));
    assert!(b_ag.as_str().unwrap_or("").starts_with("run '"));
    assert!(b_hw.as_str().unwrap_or("").ends_with("' not found"));
    assert!(b_ag.as_str().unwrap_or("").ends_with("' not found"));
}

// ── Stateless endpoints ─────────────────────────────────────────────────────

#[tokio::test]
async fn relay_professions_souls_flows_hw_vs_ag() {
    let hw = hw_app(tmp_state());
    let ag = ag_app(tmp_state());

    // professions → {professions: [...]} 逐键等价。
    let (s_hw, b_hw) = send(&hw, "GET", "/api/forge/relay/professions", None).await;
    let (s_ag, b_ag) = send(&ag, "GET", "/api/forge/relay/professions", None).await;
    assert_eq!((s_hw, &b_hw), (s_ag, &b_ag), "professions parity");
    assert!(!b_hw["professions"].as_array().unwrap().is_empty());

    // souls → {souls: []} 逐键等价。
    let (s_hw, b_hw) = send(&hw, "GET", "/api/forge/relay/souls", None).await;
    let (s_ag, b_ag) = send(&ag, "GET", "/api/forge/relay/souls", None).await;
    assert_eq!((s_hw, &b_hw), (s_ag, &b_ag), "souls parity");
    assert_eq!(b_hw, json!({"souls": []}));

    // flows → {flows: [...]}(builtin_flows 确定性)逐键等价。
    let (s_hw, b_hw) = send(&hw, "GET", "/api/forge/relay/flows", None).await;
    let (s_ag, b_ag) = send(&ag, "GET", "/api/forge/relay/flows", None).await;
    assert_eq!((s_hw, &b_hw), (s_ag, &b_ag), "flows parity");
    let flows = b_hw["flows"].as_array().unwrap();
    assert!(!flows.is_empty());
    assert!(flows[0]["id"].is_string());
    assert!(flows[0]["steps"].is_array());
}

// ── SSE endpoints ───────────────────────────────────────────────────────────

#[tokio::test]
async fn relay_sse_endpoints_return_event_stream() {
    let hw = hw_app(tmp_state());
    let ag = ag_app(tmp_state());
    for (app, name) in [(&hw, "hw"), (&ag, "ag")] {
        // run_events:订阅即 200 + text/event-stream(不消费 body,避免 keep-alive 阻塞)。
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/forge/relay/runs/whatever/events")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK, "{name} run_events 200");
        assert_eq!(
            resp.headers()["content-type"].to_str().unwrap(),
            "text/event-stream",
            "{name} run_events content-type"
        );
        // task_plan_events 同样。
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/forge/relay/task_plans/whatever/events")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK, "{name} task_plan_events 200");
        assert_eq!(
            resp.headers()["content-type"].to_str().unwrap(),
            "text/event-stream",
            "{name} task_plan_events content-type"
        );
    }
}

// ── TaskPlan handlers ───────────────────────────────────────────────────────

#[tokio::test]
async fn task_plan_crud_and_start_hw_vs_ag() {
    let hw = hw_app(tmp_state());
    let ag = ag_app(tmp_state());

    // 自定义用户 plan atom(内建 deferred-decompose 由 Registry::new 预载)。
    let custom_atom = r#"task_plan(id: "my-plan", version: 1) {
    title: "My Plan"
    default_mode: "gsd"
    phase(name: "build") {
        mode: "serial"
        run(name: "code", flow_id: "simple") {
            input: "write code"
        }
    }
}"#;

    // 初始列表含内建 deferred-decompose(Registry::new 预载),两边等价。
    let (s_hw, b_hw) = send(&hw, "GET", "/api/forge/relay/task_plans", None).await;
    let (s_ag, b_ag) = send(&ag, "GET", "/api/forge/relay/task_plans", None).await;
    assert_eq!((s_hw, &b_hw), (s_ag, &b_ag), "initial task_plans list parity");
    let initial = b_hw["task_plans"].as_array().unwrap();
    assert!(initial.iter().any(|s| s["id"] == "deferred-decompose" && s["source"] == "Builtin"));

    // register 用户 plan → {task_plan_registered, id, phase_count, run_count}。
    let (s_hw, b_hw) = send(&hw, "POST", "/api/forge/relay/task_plans", Some(json!({"atom": custom_atom}))).await;
    let (s_ag, b_ag) = send(&ag, "POST", "/api/forge/relay/task_plans", Some(json!({"atom": custom_atom}))).await;
    assert_eq!((s_hw, &b_hw), (s_ag, &b_ag), "register parity");
    assert_eq!(b_hw, json!({"task_plan_registered": true, "id": "my-plan", "phase_count": 1, "run_count": 1}));

    // register 非法 atom → 400(两边一致)。
    let (s_hw, b_hw) = send(&hw, "POST", "/api/forge/relay/task_plans", Some(json!({"atom": "not an atom"}))).await;
    let (s_ag, b_ag) = send(&ag, "POST", "/api/forge/relay/task_plans", Some(json!({"atom": "not an atom"}))).await;
    assert_eq!((s_hw, &b_hw), (s_ag, &b_ag), "register-bad parity");
    assert_eq!(s_hw, StatusCode::BAD_REQUEST);

    // list 现在含内建 + 用户计划。
    let (_, b_hw) = send(&hw, "GET", "/api/forge/relay/task_plans", None).await;
    let (_, b_ag) = send(&ag, "GET", "/api/forge/relay/task_plans", None).await;
    assert_eq!(b_hw["task_plans"].as_array().map(|a| a.len()), b_ag["task_plans"].as_array().map(|a| a.len()));
    assert!(b_hw["task_plans"].as_array().unwrap().iter().any(|s| s["id"] == "my-plan"));

    // get → 200 + plan 详情(id/title/phases)。
    let (s_hw, b_hw) = send(&hw, "GET", "/api/forge/relay/task_plans/my-plan", None).await;
    let (s_ag, b_ag) = send(&ag, "GET", "/api/forge/relay/task_plans/my-plan", None).await;
    assert_eq!(s_hw, StatusCode::OK);
    assert_eq!(s_ag, StatusCode::OK);
    assert_eq!(b_hw["id"], "my-plan");
    assert_eq!(b_ag["id"], "my-plan");
    assert_eq!(b_ag["phases"], b_hw["phases"], "plan detail wire parity");

    // get 缺失 → 404 精确文本。
    let (s_hw, b_hw) = send(&hw, "GET", "/api/forge/relay/task_plans/missing", None).await;
    let (s_ag, b_ag) = send(&ag, "GET", "/api/forge/relay/task_plans/missing", None).await;
    assert_eq!((s_hw, &b_hw), (s_ag, &b_ag), "task_plan 404 parity");
    assert_eq!(b_hw, json!("task_plan 'missing' not found"));

    // start → {instance_id, task_plan_id, status: started}(后台线程跑,立即返回)。
    let (s_hw, b_hw) = send(&hw, "POST", "/api/forge/relay/task_plans/my-plan/runs", Some(json!({"initial_input": "build a thing"}))).await;
    let (s_ag, b_ag) = send(&ag, "POST", "/api/forge/relay/task_plans/my-plan/runs", Some(json!({"initial_input": "build a thing"}))).await;
    assert_eq!(s_hw, StatusCode::OK);
    assert_eq!(s_ag, StatusCode::OK);
    assert!(b_hw["instance_id"].as_str().unwrap().starts_with("my-plan-"));
    assert!(b_ag["instance_id"].as_str().unwrap().starts_with("my-plan-"));
    assert_eq!(b_ag["task_plan_id"], "my-plan");
    assert_eq!(b_ag["status"], "started");

    // start 缺失 plan → 404。
    let (s_hw, b_hw) = send(&hw, "POST", "/api/forge/relay/task_plans/missing/runs", Some(json!({"initial_input": "x"}))).await;
    let (s_ag, b_ag) = send(&ag, "POST", "/api/forge/relay/task_plans/missing/runs", Some(json!({"initial_input": "x"}))).await;
    assert_eq!((s_hw, &b_hw), (s_ag, &b_ag), "task_plan start 404 parity");
    assert_eq!(b_hw, json!("task_plan 'missing' not found"));

    // delete(用户计划)→ 200 {deleted: id}。
    let (s_hw, b_hw) = send(&hw, "DELETE", "/api/forge/relay/task_plans/my-plan", None).await;
    let (s_ag, b_ag) = send(&ag, "DELETE", "/api/forge/relay/task_plans/my-plan", None).await;
    assert_eq!((s_hw, &b_hw), (s_ag, &b_ag), "task_plan delete parity");
    assert_eq!(b_hw, json!({"deleted": "my-plan"}));

    // delete 已删 → 400。
    let (s_hw, b_hw) = send(&hw, "DELETE", "/api/forge/relay/task_plans/my-plan", None).await;
    let (s_ag, b_ag) = send(&ag, "DELETE", "/api/forge/relay/task_plans/my-plan", None).await;
    assert_eq!((s_hw, &b_hw), (s_ag, &b_ag), "task_plan double-delete 400 parity");
    assert_eq!(b_hw, json!("cannot remove 'my-plan' (not found or built-in)"));
}


// ═══════════════════════════════════════════════════════════════════════════
// PLAN-044 T5: PARITY_TARGET=vm —— VM serve(AutoVM 后端)对照 hw。
// 无状态端点集逐键等价 + 错误路径状态码;套件其余用例(有状态 lifecycle)
// 迁移属后续(需跨进程存储对齐,见计划 T5 回填)。
// 跑法:PARITY_TARGET=vm cargo test -p musk --test parity_relay_api -- --nocapture
// ═══════════════════════════════════════════════════════════════════════════

mod common;

fn vm_target_enabled() -> bool {
    std::env::var("PARITY_TARGET").as_deref() == Ok("vm")
}

#[test]
fn relay_stateless_vm_vs_hw() {
    if !vm_target_enabled() {
        eprintln!("relay_stateless_vm_vs_hw: SKIPPED — set PARITY_TARGET=vm to run");
        return;
    }
    // hw 侧:进程内 router(与既有 parity 同源 tmp_state)。
    let rt = tokio::runtime::Runtime::new().expect("hw tokio rt");
    let expected: Vec<(&str, &str, u16, Value)> = rt.block_on(async {
        let hw = hw_app(tmp_state());
        let mut out = Vec::new();
        for (m, u) in [
            ("GET", "/api/forge/relay/professions"),
            ("GET", "/api/forge/relay/souls"),
            ("GET", "/api/forge/relay/flows"),
            ("GET", "/api/forge/relay/runs"),
            ("GET", "/api/forge/relay/task_plans"),
            ("DELETE", "/api/forge/relay/runs/nonexistent-run"),
        ] {
            let (s, b) = send(&hw, m, u, None).await;
            out.push((m, u, s.as_u16(), b));
        }
        out
    });

    // VM 侧:子进程 serve(隔离临时态)。与既有 parity 惯例一致:解析为
    // Value 语义比较(VM 序列化带空格,语义等价即 parity)。
    let vm = common::spawn_vm_serve();
    for (m, u, want_code, want_body) in expected {
        let (code, body) = vm.req(m, u);
        assert_eq!(code, want_code, "{m} {u}: hw={want_code} vm={code} body={body}");
        // DELETE 404 错误包络含 run_id 文本,只比状态码;GET 逐键语义比较。
        if m == "GET" {
            let vm_body: Value =
                serde_json::from_str(&body).unwrap_or(Value::Null);
            assert_eq!(
                vm_body, want_body,
                "{m} {u}: hw={want_body} vm={body}"
            );
        }
    }
    eprintln!("relay_stateless_vm_vs_hw: 5 endpoint(s) VM≡hw");
}
