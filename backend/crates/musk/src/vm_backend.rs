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
    // PLAN-044 T5: parity/测试隔离覆盖（未设时与 server::serve 同源）。
    let users_path = std::env::var("MUSK_VM_USERS_PATH")
        .map(std::path::PathBuf::from)
        .ok()
        .or_else(|| {
            dirs::home_dir().map(|h| h.join(".config/autoos/users.json"))
        })
        .unwrap_or_else(|| std::path::PathBuf::from("users.json"));
    let config_dir = std::env::var("MUSK_VM_CONFIG_DIR")
        .map(std::path::PathBuf::from)
        .ok()
        .or_else(|| dirs::home_dir().map(|h| h.join(".config/autoos")))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let default_root = std::env::var("MUSK_VM_DEFAULT_ROOT")
        .map(std::path::PathBuf::from)
        .ok()
        .unwrap_or_else(|| {
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
        });
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

/// 数据 extern → HostCallFn 注册表。T2/T3:经 musk_extern_dispatch(name,args)
/// 网关转发（args 为 JSON 数组,状态参由本侧闭包捕获,不进 ABI）。首期覆盖
/// parity 数据面的核心集;长尾 extern 未注册时网关回退 null（handler 错误
/// 包络兜底）,后续按域补齐。
fn register_host_calls() {
    type SerdeValue = serde_json::Value;

    fn st() -> Result<Arc<AppState>, String> {
        state().ok_or_else(|| "vm_backend: AppState not initialized".to_string())
    }
    fn arg(args: &[SerdeValue], i: usize) -> SerdeValue {
        args.get(i).cloned().unwrap_or(SerdeValue::Null)
    }
    fn wq_server(args: &[SerdeValue]) -> axum::extract::Query<crate::auto_generated::server::WorkspaceQuery> {
        axum::extract::Query(serde_json::from_value(arg(args, 0)).unwrap_or_else(|_| serde_json::from_value(serde_json::json!({})).unwrap()))
    }
    fn wq_relay(args: &[SerdeValue]) -> axum::extract::Query<crate::auto_generated::relay_api::WorkspaceQuery> {
        axum::extract::Query(serde_json::from_value(arg(args, 0)).unwrap_or_else(|_| serde_json::from_value(serde_json::json!({})).unwrap()))
    }
    fn wq_wiki(args: &[SerdeValue]) -> axum::extract::Query<crate::auto_generated::wiki::WorkspaceQuery> {
        axum::extract::Query(serde_json::from_value(arg(args, 0)).unwrap_or_else(|_| serde_json::from_value(serde_json::json!({})).unwrap()))
    }
    fn st_axum(s: &Arc<AppState>) -> axum::extract::State<AppState> {
        axum::extract::State(s.as_ref().clone())
    }
    fn enc(v: impl serde::Serialize) -> Result<String, String> {
        serde_json::to_string(&v).map_err(|e| e.to_string())
    }
    /// json null → Rust None(Option 参数线型)。
    fn opt(v: &SerdeValue) -> Option<SerdeValue> {
        if v.is_null() { None } else { Some(v.clone()) }
    }
    /// VM 后端专用 tokio 运行时(SSE 生产者 spawn + mpsc 阻塞收)。
    fn rt() -> &'static tokio::runtime::Runtime {
        static RT: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();
        RT.get_or_init(|| tokio::runtime::Runtime::new().expect("vm tokio runtime"))
    }

    macro_rules! host {
        ($name:literal, $body:expr) => {{
            let f: auto_lang::vm::host_bridge::HostCallFn = std::sync::Arc::new(move |args_json: &str| {
                let args: Vec<SerdeValue> = serde_json::from_str(args_json).unwrap_or_default();
                let call: fn(&Vec<SerdeValue>) -> Result<String, String> = $body;
                call(&args)
            });
            auto_lang::vm::host_bridge::register_host_call($name, f);
        }};
    }

    use crate::auto_generated::extern_impl as ei;

    // ── 无状态纯数据 ──
    host!("professions_list", |_a| enc(ei::professions_list()));
    host!("modes_all", |_a| enc(ei::modes_all()));
    host!("skills_all", |_a| enc(ei::skills_all()));
    host!("roles_all", |_a| enc(ei::roles_all()));
    host!("config_build", |_a| enc(ei::config_build()));
    host!("app_config_load", |_a| enc(ei::app_config_load()));
    host!("forge_mode_load", |_a| enc(ei::forge_mode_load()));
    host!("workflows_builtin_names", |_a| enc(ei::workflows_builtin_names()));
    host!("relay_professions_list", |_a| enc(ei::relay_professions_list()));
    host!("relay_flows_list", |_a| enc(ei::relay_flows_list()));
    host!("app_config_effective_daemon_url", |_a| enc(ei::app_config_effective_daemon_url::<SerdeValue>(SerdeValue::Null)));

    // ── 状态数据（Query 第一参）──
    host!("specs_load", |a| enc(ei::specs_load(&st_axum(&st()?), wq_server(a))));
    host!("chats_list", |a| enc(ei::chats_list(&st_axum(&st()?), wq_server(a))));
    host!("conversations_list", |a| enc(ei::conversations_list(&st_axum(&st()?), wq_server(a))));
    host!("workspace_list_all", |_a| enc(ei::workspace_list_all(&st_axum(&st()?))));
    host!("relay_runs_list", |a| enc(ei::relay_runs_list(&st_axum(&st()?), wq_relay(a))));
    host!("ws_wiki_list", |a| enc(ei::ws_wiki_list(&st_axum(&st()?), wq_wiki(a))));

    // ── SSE/mpsc 域（T4）── mpsc 句柄即 JSON 数字,HANDLES side-table 在宿主,
    // 直接过网关;async 生产者(agent/chat/wf stream)经 tokio spawn 并发推 tx,
    // VM 侧 mpsc_recv 阻塞收(tokio worker 线程喂消息,无死锁)。
    host!("mpsc_channel", |_a| enc(ei::mpsc_channel()));
    host!("mpsc_sender", |a| enc(ei::mpsc_sender(&arg(a, 0))));
    host!("mpsc_receiver", |a| enc(ei::mpsc_receiver(&arg(a, 0))));
    host!("mpsc_try_send", |a| { ei::mpsc_try_send(&arg(a, 0), arg(a, 1)); enc(()) });
    host!("mpsc_recv", |a| {
        let r = rt().block_on(async { ei::mpsc_recv(&arg(a, 0)).await });
        enc(r)
    });
    host!("msg_is_none", |a| enc(ei::msg_is_none(&opt(&arg(a, 0)))));
    host!("msg_unwrap", |a| enc(ei::msg_unwrap(opt(&arg(a, 0)))));
    host!("workflow_exists", |a| {
        let n: &str = a.first().and_then(|v| v.as_str()).unwrap_or("");
        enc(ei::workflow_exists(n))
    });
    host!("mode_exists", |a| {
        let n: &str = a.first().and_then(|v| v.as_str()).unwrap_or("");
        enc(ei::mode_exists(n))
    });
    host!("stream_event_map", |a| enc(ei::stream_event_map(opt(&arg(a, 0)))));

    // async 流式生产者:fire-and-forget spawn(与 hw 的 spawn 语义一致),
    // 事件经 mpsc 侧表流回,channel 关闭即流终止。
    host!("agent_run_stream", |a| {
        let st = st_axum(&st()?);
        let q = serde_json::from_value::<crate::auto_generated::server_stream::WorkspaceQuery>(arg(a, 0))
            .unwrap_or_else(|_| serde_json::from_value(serde_json::json!({})).unwrap());
        let b = serde_json::from_value::<crate::auto_generated::server_stream::RunRequest>(arg(a, 1))
            .map_err(|e| format!("RunRequest: {e}"))?;
        let tx = arg(a, 2);
        rt().spawn(async move { ei::agent_run_stream(&st, axum::extract::Query(q), axum::Json(b), tx).await });
        enc(())
    });
    host!("wf_run_with_progress", |a| {
        let st = st_axum(&st()?);
        let q = serde_json::from_value::<crate::auto_generated::server_stream::WorkspaceQuery>(arg(a, 0))
            .unwrap_or_else(|_| serde_json::from_value(serde_json::json!({})).unwrap());
        let b = serde_json::from_value::<crate::auto_generated::server_stream::WorkflowRunRequest>(arg(a, 1))
            .map_err(|e| format!("WorkflowRunRequest: {e}"))?;
        let tx = arg(a, 2);
        rt().spawn(async move { ei::wf_run_with_progress(&st, axum::extract::Query(q), axum::Json(b), tx).await });
        enc(())
    });
    host!("chat_run_stream", |a| {
        let st = st_axum(&st()?);
        let q = serde_json::from_value::<crate::auto_generated::server_stream::WorkspaceQuery>(arg(a, 0))
            .unwrap_or_else(|_| serde_json::from_value(serde_json::json!({})).unwrap());
        let id = a.get(2).and_then(|v| v.as_str()).unwrap_or_default().to_string();
        let tx = arg(a, 3);
        rt().spawn(async move { ei::chat_run_stream(&st, axum::extract::Query(q), axum::extract::Path(id), tx).await });
        enc(())
    });

    // ── auth 域 ──
    host!("auth_login_result", |a| {
        let u = arg(a, 0).as_str().unwrap_or_default().to_string();
        let p = arg(a, 1).as_str().unwrap_or_default().to_string();
        let (t, r) = ei::auth_login_result(&st_axum(&st()?), u, p);
        enc(vec![t, r])
    });
}
