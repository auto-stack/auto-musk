//! vm_serve_harness.rs — PLAN-044 T5: VM serve 冒烟门(harness 见 common)。

mod common;

/// T5 冒烟门:VM serve 起服 + health + 无状态数据端点。
/// 手动跑:cargo test -p musk --test vm_serve_harness -- --ignored --nocapture
#[test]
#[ignore = "spawns a full VM serve subprocess (compile ~60s); manual gate"]
fn vm_serve_health_and_data_endpoints() {
    let vm = common::spawn_vm_serve();
    let (code, body) = vm.get("/api/health");
    assert_eq!(code, 200, "health body: {body}");
    assert!(body.contains("\"ok\""), "health body: {body}");
    let (code, body) = vm.get("/api/forge/relay/runs");
    assert_eq!(code, 200, "relay runs body: {body}");
    assert!(body.contains("\"runs\""), "relay runs body: {body}");
}
