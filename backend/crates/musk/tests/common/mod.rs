//! common/mod.rs — PLAN-044 T5: VM serve harness(多测试目标共享)。
//!
//! `PARITY_TARGET=vm` 时 parity 套件经本 harness 起 `musk serve`
//! (MUSK_BACKEND=vm 子进程,隔离临时 workspace 经 MUSK_VM_* env 注入),
//! 对真实 HTTP/SSE 打请求——替代 tower oneshot 直打 a2r router 的旧形态
//! (auto-lang 442 §7.2 阻塞点 3 的解法)。
//!
//! 首期(本文件):harness 骨架 + VM serve 冒烟门(#[ignore],手动跑)。
//! parity 套件的 PARITY_TARGET 接线按套件逐个迁移(见计划 T5 回填)。

#![cfg(test)]

use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};

pub struct VmServe {
    pub port: u16,
    child: Child,
}

impl Drop for VmServe {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// 起隔离态 VM serve:临时 config_dir/users/default_root。
pub fn spawn_vm_serve() -> VmServe {
    let port = pick_port();
    let dir = std::env::temp_dir().join(format!(
        "musk-vm-parity-{}-{}",
        std::process::id(),
        port
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");

    let exe = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .map(|d| d.join("musk.exe"))
        .filter(|p| p.is_file())
        .expect("musk.exe next to test binary (cargo test layout)");

    let mut child = Command::new(&exe)
        .arg("serve")
        .arg("--addr")
        .arg(format!("127.0.0.1:{port}"))
        .env("MUSK_BACKEND", "vm")
        .env("MUSK_VM_CONFIG_DIR", &dir)
        .env("MUSK_VM_USERS_PATH", dir.join("users.json"))
        .env("MUSK_VM_DEFAULT_ROOT", &dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn musk serve (vm)");

    // 等路由装配 + 监听就绪(全量语料 compile 数十秒;120s 上限对齐
    // auto-lang serve 冒烟惯例),stderr 逐行看 [HTTP] listening。
    let stderr = child.stderr.take().expect("stderr");
    let started = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let flag = started.clone();
    std::thread::spawn(move || {
        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
            if line.contains("listening") {
                flag.store(true, std::sync::atomic::Ordering::SeqCst);
                break;
            }
        }
    });
    for _ in 0..1200 {
        if started.load(std::sync::atomic::Ordering::SeqCst) {
            break;
        }
        if let Ok(Some(_)) = child.try_wait() {
            panic!("vm serve exited before listening");
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    if !started.load(std::sync::atomic::Ordering::SeqCst) {
        panic!("vm serve did not start listening within 120s");
    }
    VmServe { port, child }
}

impl VmServe {
    pub fn get(&self, path: &str) -> (u16, String) {
        self.req("GET", path)
    }

    /// 任意方法(PLAN-044 T5:parity 需要 DELETE 等)。
    pub fn req(&self, method: &str, path: &str) -> (u16, String) {
        let url = format!("http://127.0.0.1:{}{}", self.port, path);
        let resp = std::panic::catch_unwind(|| {
            ureq::request(method, &url)
                .timeout(std::time::Duration::from_secs(15))
                .call()
        });
        match resp {
            Ok(Ok(r)) => {
                let code = r.status();
                let body = r.into_string().unwrap_or_default();
                (code, body)
            }
            // 4xx/5xx 是 parity 的合法对照面(404/400 等),取状态码+body。
            Ok(Err(ureq::Error::Status(code, r))) => {
                let body = r.into_string().unwrap_or_default();
                (code, body)
            }
            Ok(Err(e)) => panic!("{method} {path} failed: {e}"),
            Err(_) => panic!("{method} {path} panicked"),
        }
    }
}

fn pick_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .expect("bind :0")
        .local_addr()
        .expect("local addr")
        .port()
}

/// T5 冒烟门:VM serve 起服 + health + 无状态数据端点。
/// 手动跑:cargo test -p musk --test vm_serve_harness -- --ignored --nocapture
#[test]
#[ignore = "spawns a full VM serve subprocess (compile ~60s); manual gate"]
fn vm_serve_health_and_data_endpoints() {
    let vm = spawn_vm_serve();
    let (code, body) = vm.get("/api/health");
    assert_eq!(code, 200, "health body: {body}");
    assert!(body.contains("\"ok\""), "health body: {body}");
    let (code, body) = vm.get("/api/forge/relay/runs");
    assert_eq!(code, 200, "relay runs body: {body}");
    assert!(body.contains("\"runs\""), "relay runs body: {body}");
}
