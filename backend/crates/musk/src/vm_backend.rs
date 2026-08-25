//! vm_backend.rs — PLAN-044 Phase 1: VM 后端桥接（状态闭包桥）。
//!
//! `MUSK_BACKEND=vm` 时 `musk serve` 走本模块而非 axum `server::serve`:
//! 宿主进程内构建一次 `Arc<AppState>`,把数据 extern 注册为捕获 state 的
//! HostCallFn（AppState 永不过 JSON ABI——auto-lang 442 §7.2 阻塞点 1 的
//! 解法),再以 `auto_lang::run_file` 跑 `auto-src/vm_entry.at`（AutoVM HTTP
//! server,axum_adapter 路由仿真 + AUTO_HTTP_PORT 监听）。
//!
//! 路由装配 = vm_entry.at 调 `build_router()`（server.at 🟡 路由）+
//! `relay_routes()`/`wiki_routes()`（分域路由组）;🔴 SSE 路由经 server_stream
//! 模块在 Phase 1 T4 接入。handler 的 `State<AppState>` 形参在 VM 侧绑定为
//! 不透明堆句柄（axum_adapter::app_state_handle）,真实 state 只存在于本模块
//! 的闭包捕获里。

use std::sync::Arc;

use auto_ai_agent::Client;

use crate::server::AppState;

/// PLAN-044 T1:VM 后端入口。阻塞当前线程直至 VM server 退出。
pub fn serve(addr: &str, client: Arc<dyn Client>) -> Result<(), Box<dyn std::error::Error>> {
    let port: String = addr
        .rsplit(':')
        .next()
        .unwrap_or("8080")
        .to_string();
    std::env::set_var("AUTO_HTTP_PORT", &port);

    // AppState 单例：host 闭包捕获的真身（T3 起 extern 实现消费）。
    let state = build_app_state(client);
    init_state(state);

    register_host_calls();

    let entry = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("auto-src")
        .join("vm_entry.at");
    let entry = entry.to_string_lossy().to_string();
    eprintln!("[VM] musk serve (AutoVM backend) on {addr} — entry {entry}");

    // 全量语料 parse+codegen 递归深,32MB 栈（对齐 auto-man run_vm_ui 惯例）。
    let handle = std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .name("musk-vm-server".into())
        .spawn(move || match auto_lang::run_file(&entry) {
            Ok(_) => eprintln!("[VM] run_file returned Ok"),
            Err(e) => eprintln!("[VM] run_file error: {e}"),
        })
        .expect("spawn musk-vm-server thread");
    handle.join().map_err(|_| "vm server thread panicked")?;
    Ok(())
}

/// 与 `server::serve` 同源的 AppState 构建（T1 只需结构就位;数据 extern
/// 在 T2/T3 逐步接线）。抽公共前先保持本地复制,避免动 hw 轨签名。
fn build_app_state(client: Arc<dyn Client>) -> Arc<AppState> {
    let users_path = dirs::home_dir()
        .map(|h| h.join(".config/autoos/users.json"))
        .unwrap_or_else(|| std::path::PathBuf::from("users.json"));
    let config_dir = dirs::home_dir()
        .map(|h| h.join(".config/autoos"))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let default_root =
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let registry = crate::workspace::WorkspaceRegistry::load(
        config_dir.join("workspaces.json"),
        default_root,
    );
    registry.migrate_global_data(&config_dir);
    Arc::new(AppState {
        client,
        auth: Arc::new(crate::auto_generated::auth::AuthStore::new(users_path)),
        registry: Arc::new(registry),
    })
}

/// 全局 state 单例（OnceLock——T3 的 extern_impl 改造与本模块共用同一定义）。
static STATE: std::sync::OnceLock<Arc<AppState>> = std::sync::OnceLock::new();

pub fn init_state(state: Arc<AppState>) {
    let _ = STATE.set(state);
}

pub fn state() -> Option<Arc<AppState>> {
    STATE.get().cloned()
}

/// 数据 extern → HostCallFn 注册表。T1 为空壳（/api/health 纯 Auto 无需
/// extern）;T2 起无状态 extern 直转发,T3 起状态 extern 闭包捕获 STATE。
fn register_host_calls() {
    // PLAN-044 T2/T3 填充。
}
