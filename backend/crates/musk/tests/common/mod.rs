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


use std::process::{Child, Command, Stdio};

pub struct VmServe {
    pub port: u16,
    child: Child,
    /// 子进程 stderr 落盘(失败诊断)。
    pub log_path: std::path::PathBuf,
}

impl Drop for VmServe {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// 起隔离态 VM serve:临时 config_dir/users/default_root。
pub fn spawn_vm_serve() -> VmServe {
    spawn_vm_serve_with_env(&[])
}

/// PLAN-044 T6:带额外 env(如 AAID_URL 指向不可达地址 → 确定性错误流)。
pub fn spawn_vm_serve_with_env(extra_env: &[(String, String)]) -> VmServe {
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
        .envs(extra_env.iter().map(|(k, v)| (k, v)))
        .stdout(Stdio::from(std::fs::File::create(dir.join("serve.out")).expect("create serve.out")))
        .stderr(Stdio::from(std::fs::File::create(dir.join("serve.log")).expect("create serve.log")))
        .spawn()
        .expect("spawn musk serve (vm)");

    // 等路由装配 + 监听就绪(全量语料 compile 数十秒;120s 上限对齐
    // auto-lang serve 冒烟惯例)。stderr 已重定向 serve.log,轮询文件 grep
    // "listening"(并保留 serve.log 供失败诊断)。
    let started = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let flag = started.clone();
    let log_path = dir.join("serve.log");
    std::thread::spawn(move || {
        for _ in 0..1200 {
            if let Ok(content) = std::fs::read_to_string(&log_path) {
                if content.contains("listening") {
                    flag.store(true, std::sync::atomic::Ordering::SeqCst);
                    return;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
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
    VmServe { port, child, log_path: dir.join("serve.log") }
}

impl VmServe {
    pub fn get(&self, path: &str) -> (u16, String) {
        self.req("GET", path)
    }

    /// serve.log 尾部(诊断辅助)。
    pub fn log_tail(&self, n: usize) -> String {
        std::fs::read_to_string(&self.log_path)
            .map(|s| s.lines().rev().take(n).collect::<Vec<_>>().join("
"))
            .unwrap_or_default()
    }

    /// 裸 TCP 请求(PLAN-044 T6:SSE 流消费——ureq 对无 content-length 的
    /// 流式响应不稳,裸读 Connection: close 到 EOF)。
    pub fn req_raw(&self, method: &str, path: &str, body: &str) -> (u16, String) {
        use std::io::{Read, Write};
        let mut stream = std::net::TcpStream::connect(("127.0.0.1", self.port))
            .unwrap_or_else(|e| panic!("connect: {e}"));
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(60)))
            .ok();
        let req = format!(
            "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: {len}\r\n\r\n{body}",
            len = body.len()
        );
        stream.write_all(req.as_bytes()).expect("write req");
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).expect("read to eof");
        let raw = String::from_utf8_lossy(&buf).into_owned();
        let code = raw
            .split_whitespace()
            .nth(1)
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(0);
        let body = raw
            .split_once("\r\n\r\n")
            .map(|(_, b)| b.to_string())
            .unwrap_or_default();
        (code, body)
    }

    /// POST + JSON body(PLAN-044 T6:SSE 流消费;读到流关闭)。
    pub fn req_body(&self, method: &str, path: &str, body: &str) -> (u16, String) {
        let url = format!("http://127.0.0.1:{}{}", self.port, path);
        let resp = std::panic::catch_unwind(|| {
            ureq::request(method, &url)
                .timeout(std::time::Duration::from_secs(60))
                .set("Content-Type", "application/json")
                .send_string(body)
        });
        match resp {
            Ok(Ok(r)) => (r.status(), r.into_string().unwrap_or_default()),
            Ok(Err(ureq::Error::Status(code, r))) => {
                (code, r.into_string().unwrap_or_default())
            }
            Ok(Err(e)) => panic!("{method} {path} failed: {e}"),
            Err(_) => panic!("{method} {path} panicked"),
        }
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
