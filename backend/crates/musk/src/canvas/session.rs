//! PLAN-087 T-02: canvas 会话 spawn/收割 —— 隔离 `auto run --render=vm` 子进程。
//!
//! 编排参数复用 PLAN-080 ui-parity live 链：`AUTOUI_MCP_PORT`（AutoUI MCP 监听
//! 口）、`AUTO_VM_STORAGE_FILE`（每会话一次性 localStorage 镜像，状态零串味）、
//! `AUTO_VM_WINDOW`（画布友好固定窗）。端点发现 = 子进程 stderr 抓
//! `AutoUI MCP: listening on http://<addr>`（mcp_server.rs stderr 打印；端口
//! 忙时 auto-lang 自带 +1..+10 回退并打印实际地址，故必须解析行内地址而非
//! 信任注入值）。收割 = `taskkill /T /F`（live.mjs 同款）+ 端口探测确认退场。

use std::path::{Path, PathBuf};
use std::time::Duration;

/// 端点发现超时：VM 冷启动（首次 iced/wgpu 初始化）在慢盘上可到十几秒。
pub const ENDPOINT_TIMEOUT: Duration = Duration::from_secs(30);

/// auto 可执行解析序（AUTO_EXE 式，live.mjs:296 同源）：
/// `AUTO_EXE` env → 编译期兄弟位 `../auto-lang/target/release/auto.exe`
/// （worktree 构建解析组内兄弟，主检出构建解析主检出）→ `D:/autostack/auto-lang`
/// 主检出 → PATH 上的 `auto`。
pub fn resolve_auto_exe() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("AUTO_EXE") {
        if !p.trim().is_empty() {
            let path = PathBuf::from(p);
            if path.exists() {
                return Some(path);
            }
            // env 显式指定但不存在的路径不静默跳过——调用方报错更可诊断。
            return Some(path);
        }
    }
    // 编译期兄弟位：backend/crates/musk → 上溯 4 级 = 仓库父目录 → auto-lang。
    // worktree 构建时即组内兄弟（AGENTS.md 解析序），主检出构建时即主检出。
    let sibling = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../auto-lang/target/release/auto.exe");
    if sibling.exists() {
        return Some(sibling);
    }
    let main_checkout = PathBuf::from("D:/autostack/auto-lang/target/release/auto.exe");
    if main_checkout.exists() {
        return Some(main_checkout);
    }
    None
}

/// 一个已就绪的画布会话进程句柄。
pub struct SessionHandle {
    pub app_dir: PathBuf,
    /// AutoUI MCP 基址（`http://127.0.0.1:<port>`，取自监听行实际地址）。
    pub mcp_base: String,
    pub mcp_port: u16,
    child: tokio::process::Child,
    /// 每会话一次性 storage 镜像路径（Drop 清理）。
    storage_file: PathBuf,
}

impl SessionHandle {
    /// Spawn 隔离 VM 实例并等待 MCP 端点就绪。失败时进程已被收割。
    pub async fn spawn(app_dir: PathBuf) -> Result<Self, String> {
        let exe = resolve_auto_exe().ok_or_else(|| {
            "auto executable not found: set AUTO_EXE or build auto-lang (target/release/auto.exe)".to_string()
        })?;
        let port = alloc_port().await?;
        let storage_file = std::env::temp_dir().join(format!(
            "musk-canvas-{}-{}.json",
            std::process::id(),
            port
        ));
        let mut cmd = tokio::process::Command::new(&exe);
        cmd.args(["run", "--render=vm"])
            .current_dir(&app_dir)
            .env("AUTOUI_MCP_PORT", port.to_string())
            .env("AUTO_VM_STORAGE_FILE", &storage_file)
            .env("AUTO_VM_WINDOW", "480x680")
            // PLAN-088 T-02B：debug 捕获面（devtools_open 不置位，面板不出镜）
            // ——MCP vtree 通道的叶件 bounds + 引导帧 bounds 依赖项（auto-lang
            // session.rs 同名门控）。M2 画布无真鼠点击/hover，402 回归面不适用。
            .env("AUTO_DEBUG_CAPTURE", "1")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);
        // 不继承任何可能让 VM 复用既有后端/窗口的环境（隔离优先）。
        cmd.env_remove("AUTO_REUSE_BACKEND");
        cmd.env_remove("AUTO_HTTP_BASE");

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("failed to spawn {}: {e}", exe.display()))?;
        let pid = child.id();
        let mut stdout = child.stdout.take().ok_or("stdout not piped")?;
        let mut stderr = child.stderr.take().ok_or("stderr not piped")?;

        // 双流扫描监听行（stderr 为主，stdout 兜底）；行经 channel 汇聚。
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
        tokio::spawn(async move {
            use tokio::io::AsyncBufReadExt;
            let mut lines = tokio::io::BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if tx.send(format!("E{line}")).is_err() {
                    break;
                }
            }
        });
        let (tx2, mut rx2) = tokio::sync::mpsc::unbounded_channel::<String>();
        tokio::spawn(async move {
            use tokio::io::AsyncBufReadExt;
            let mut lines = tokio::io::BufReader::new(stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if tx2.send(format!("O{line}")).is_err() {
                    break;
                }
            }
        });

        let deadline = tokio::time::Instant::now() + ENDPOINT_TIMEOUT;
        let mut mcp_base: Option<String> = None;
        loop {
            if tokio::time::Instant::now() >= deadline {
                let _ = reap_tree(pid).await;
                return Err(format!(
                    "canvas: MCP endpoint not announced within {:?} (app: {})",
                    ENDPOINT_TIMEOUT,
                    app_dir.display()
                ));
            }
            // 子进程先行退出 = 坏 app（解析失败等），把退出码带给调用方。
            if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
                let _ = reap_tree(pid).await;
                return Err(format!(
                    "canvas: auto run exited early (status: {status}) for {}",
                    app_dir.display()
                ));
            }
            tokio::select! {
                line = rx.recv() => {
                    if let Some(line) = line {
                        if let Some(addr) = parse_listening_line(&line) {
                            mcp_base = Some(format!("http://{addr}"));
                        }
                    }
                }
                line = rx2.recv() => {
                    if let Some(line) = line {
                        if let Some(addr) = parse_listening_line(&line) {
                            mcp_base = Some(format!("http://{addr}"));
                        }
                    }
                }
                _ = tokio::time::sleep(Duration::from_millis(100)) => {}
            }
            if let Some(base) = mcp_base {
                let mcp_port = base.rsplit(':').next().and_then(|p| p.parse().ok()).unwrap_or(port);
                return Ok(Self {
                    app_dir,
                    mcp_base: base,
                    mcp_port,
                    child,
                    storage_file,
                });
            }
        }
    }

    pub fn pid(&mut self) -> Option<u32> {
        self.child.id()
    }

    /// 子进程是否已退出（看门狗每拍快检，死亡即刻走复活而非等截图失败）。
    pub fn has_exited(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(Some(_)))
    }

    /// 收割进程树并确认端口退场。幂等（taskkill 对已死 pid 报错即忽略）。
    pub async fn reap(&mut self) {
        let pid = self.child.id();
        let port = self.mcp_port;
        let storage = self.storage_file.clone();
        let _ = self.child.kill().await; // kill_on_drop 兜底
        reap_tree(pid).await;
        // 端口探测确认退场：最多 ~2s， refused = 已无人监听。
        for _ in 0..10 {
            if std::net::TcpStream::connect_timeout(
                &std::net::SocketAddr::from(([127, 0, 0, 1], port)),
                Duration::from_millis(200),
            )
            .is_err()
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
        let _ = std::fs::remove_file(storage);
    }
}

impl Drop for SessionHandle {
    fn drop(&mut self) {
        // 同步兜底（异步 reap 不可达的路径）：kill_on_drop 只杀直接子进程，
        // 树内孙进程靠 taskkill /T。Drop 阻塞可接受（进程退出路径）。
        if let Some(pid) = self.child.id() {
            reap_tree_blocking(pid);
        }
        let _ = std::fs::remove_file(&self.storage_file);
    }
}

/// `taskkill /T /F` 收割进程树（live.mjs:84 同款）。async 形。
pub async fn reap_tree(pid: Option<u32>) {
    let Some(pid) = pid else { return };
    reap_tree_blocking(pid);
}

/// `taskkill /T /F` 同步形——Drop 与 async 上下文共用。
pub fn reap_tree_blocking(pid: u32) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let _ = std::process::Command::new("taskkill")
            .args(["/T", "/F", "/PID", &pid.to_string()])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
    }
    #[cfg(not(windows))]
    {
        let _ = std::process::Command::new("kill")
            .args(["-9", &pid.to_string()])
            .output();
    }
}

/// 临时端口分配：bind :0 取号即 drop（TOCTOU 竞态可接受：spawn 失败即报错，
/// 看门狗按退避重试）。AutoUI 默认 9247 在Registered 口径之外（临时端口段）。
async fn alloc_port() -> Result<u16, String> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| format!("canvas: cannot allocate ephemeral port: {e}"))?;
    listener.local_addr().map(|a| a.port()).map_err(|e| e.to_string())
}

/// 解析监听行 → `host:port`。匹配 `AutoUI MCP: listening on http://<addr>`
/// （stderr 打印，mcp_server.rs）；行首带流标记（E/O 前缀）不影响查找。
pub fn parse_listening_line(line: &str) -> Option<String> {
    const MARKER: &str = "AutoUI MCP: listening on http://";
    let idx = line.find(MARKER)?;
    let addr = line[idx + MARKER.len()..].trim();
    if addr.is_empty() {
        return None;
    }
    Some(addr.to_string())
}

/// workspace 沙箱判定后的 app 目录校验（ pac.at 存在性）——API 与 canvas_run
/// 工具共用的最后一步（resolve_multi 已保证落根内，这里校验它确实是个 app）。
pub fn validate_app_dir(dir: &Path) -> Result<(), String> {
    if !dir.is_dir() {
        return Err(format!("canvas: '{}' is not a directory", dir.display()));
    }
    if !dir.join("pac.at").exists() {
        return Err(format!(
            "canvas: '{}' has no pac.at (not an auto app directory)",
            dir.display()
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_listening_line_extracts_addr() {
        assert_eq!(
            parse_listening_line("AutoUI MCP: listening on http://127.0.0.1:9247"),
            Some("127.0.0.1:9247".to_string())
        );
        // 端口忙回退行不匹配监听行标记。
        assert_eq!(
            parse_listening_line("AutoUI MCP: port 9247 busy — fell back to 9248"),
            None
        );
        // stderr 流标记前缀可穿透。
        assert_eq!(
            parse_listening_line("E[boot] AutoUI MCP: listening on http://127.0.0.1:53123"),
            Some("127.0.0.1:53123".to_string())
        );
        assert_eq!(parse_listening_line("nothing here"), None);
    }

    #[test]
    fn resolve_auto_exe_prefers_env() {
        // 不设 AUTO_EXE 时至少有一个解析臂命中（CI/开发机上有 auto-lang 构建），
        // 或整体 None——两种都合法，只验证不 panic 且 env 优先。
        // （env 置换在并行测试下有竞态，此处只测纯查找路径的存在性语义。）
        let resolved = resolve_auto_exe();
        if let Ok(p) = std::env::var("AUTO_EXE") {
            if !p.trim().is_empty() {
                assert_eq!(resolved, Some(PathBuf::from(&p)));
            }
        }
    }

    #[tokio::test]
    async fn validate_app_dir_requires_pac() {
        let tmp = std::env::temp_dir().join(format!("musk-canvas-test-{}", std::process::id()));
        let app = tmp.join("no-pac");
        let _ = std::fs::create_dir_all(&app);
        assert!(validate_app_dir(&app).is_err());
        std::fs::write(app.join("pac.at"), "pac {}").unwrap();
        assert!(validate_app_dir(&app).is_ok());
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
