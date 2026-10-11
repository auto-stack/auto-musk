//! PLAN-087 T-02: canvas 会话 spawn/收割 —— 隔离 `auto run --render=vm` 子进程。
//!
//! 编排参数复用 PLAN-080 ui-parity live 链：`AUTOUI_MCP_PORT`（AutoUI MCP 监听
//! 口）、`AUTO_VM_STORAGE_FILE`（每会话一次性 localStorage 镜像，状态零串味）、
//! `AUTO_VM_WINDOW`（画布友好固定窗）。端点发现 = 子进程 stderr 抓
//! `AutoUI MCP: listening on http://<addr>`（mcp_server.rs stderr 打印；端口
//! 忙时 auto-lang 自带 +1..+10 回退并打印实际地址，故必须解析行内地址而非
//! 信任注入值）。收割 = `taskkill /T /F`（live.mjs 同款）+ 端口探测确认退场。

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
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
    /// 子进程输出环形缓冲（崩溃诊断；T-08 canvas 轨 ~35s 周期性 child
    /// exited 定罪用——stderr 此前被丢弃，死因不可见）。
    recent_output: Arc<Mutex<Vec<String>>>,
    /// PLAN-102 T-02：embedded 呈现的 bridge 宿主子进程（`auto rqhost
    /// --bridge-ws`——去窗 rq 宿主 + WS 桥，D-1）。None = windowed 旧轨
    ///（可见原生窗 spawn——保留为 legacy/窗口模式呈现源）。
    bridge: Option<BridgeChild>,
}

/// PLAN-102 T-02：bridge 宿主子进程与其数据面端点。
pub struct BridgeChild {
    pub child: tokio::process::Child,
    /// 会话专用 well-known 管道名（app 侧 AUTO_RQHOST_PIPE 同值）。
    pub pipe: String,
    /// WS 数据面端口（回环 + token）。
    pub ws_port: u16,
    /// 静态 token（回环 WS 准入；status 携给前端直连 bridge）。
    pub token: String,
    /// PLAN-102 T-06：宿主采纳的目标 app 名（bridge stderr `adopted \`NAME\``
    /// 解析——remote-frame/镜像订阅按 Hello app_name 精确匹配，URL 缺
    /// app 参数将永不命中；槽在采纳后异步填充）。
    pub app_name: Arc<Mutex<Option<String>>>,
}

impl BridgeChild {
    /// 前端直连 URL（drawlist/remote-frame 消费面）。
    pub fn ws_url(&self) -> String {
        format!("ws://127.0.0.1:{}/?token={}", self.ws_port, self.token)
    }
}

impl SessionHandle {
    /// PLAN-102 T-06：bridge 采纳名槽（status URL 织参源）。
    pub fn bridge_app(&self) -> Option<Arc<Mutex<Option<String>>>> {
        self.bridge.as_ref().map(|b| b.app_name.clone())
    }

    /// 子进程输出尾部（崩溃诊断用；最多 `n` 行）。
    pub fn output_tail(&self, n: usize) -> String {
        let buf = self.recent_output.lock().unwrap();
        let start = buf.len().saturating_sub(n);
        buf[start..].join("\n")
    }
}

impl SessionHandle {
    /// Spawn 隔离 VM 实例并等待 MCP 端点就绪。失败时进程已被收割。
    /// PLAN-102 T-02 起产品路径 = [`Self::spawn_embedded`]（bridge 呈现）；
    /// 本形态（可见原生窗）保留为窗口呈现源/legacy 通道——rq 重连 seam
    /// 落地后由呈现切换路由按需启用。
    #[allow(dead_code)]
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
        // PLAN-097 演示 UX：画布原生窗确定性停靠位——serve 级
        // `MUSK_CANVAS_WINDOW_POS=x,y`（演示驱动按宿主窗右侧计算）透传
        // auto-lang `AUTO_VM_POSITION`，画布窗不遮宿主。原生窗 = 帧渲染
        // 源，Musk 面板镜像才是"内嵌"呈现面；Windows 上 Level::AlwaysOn
        // Bottom 非粘性（激活即抬升，t12 解阻批次实证），位置停靠为可行解。
        // 缺省 = OS 选择（独立运行形态不变）。
        if let Ok(pos) = std::env::var("MUSK_CANVAS_WINDOW_POS") {
            let pos = pos.trim();
            if !pos.is_empty() {
                cmd.env("AUTO_VM_POSITION", pos);
            }
        }
        // 不继承任何可能让 VM 复用既有后端/窗口的环境（隔离优先）。
        cmd.env_remove("AUTO_REUSE_BACKEND");
        cmd.env_remove("AUTO_HTTP_BASE");

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("failed to spawn {}: {e}", exe.display()))?;
        let pid = child.id();
        let mut stdout = child.stdout.take().ok_or("stdout not piped")?;
        let mut stderr = child.stderr.take().ok_or("stderr not piped")?;

        // 双流汇聚任务：环形缓冲（崩溃诊断，T-08 定罪 canvas 轨 ~35s 周期性
        // child exited 用——此前就绪后 stderr 被丢弃，死因不可见）+ 监听行
        // 解析（发现即写 found）。主循环轮询 found。子进程死亡（管道 EOF）
        // 时任务自然退场；缓冲随 recent Arc 存活供崩溃路径读取。
        let recent: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let found: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        {
            let recent = recent.clone();
            let found = found.clone();
            tokio::spawn(async move {
                use tokio::io::AsyncBufReadExt;
                let mut es = tokio::io::BufReader::new(stderr).lines();
                let mut os = tokio::io::BufReader::new(stdout).lines();
                loop {
                    tokio::select! {
                        line = es.next_line() => {
                            match line {
                                Ok(Some(l)) => {
                                    if let Some(addr) = parse_listening_line(&format!("E{l}")) {
                                        *found.lock().unwrap() = Some(format!("http://{addr}"));
                                    }
                                    let mut buf = recent.lock().unwrap();
                                    if buf.len() >= 200 { buf.remove(0); }
                                    buf.push(l);
                                }
                                Ok(None) | Err(_) => break,
                            }
                        }
                        line = os.next_line() => {
                            match line {
                                Ok(Some(l)) => {
                                    if let Some(addr) = parse_listening_line(&format!("O{l}")) {
                                        *found.lock().unwrap() = Some(format!("http://{addr}"));
                                    }
                                    let mut buf = recent.lock().unwrap();
                                    if buf.len() >= 200 { buf.remove(0); }
                                    buf.push(l);
                                }
                                Ok(None) | Err(_) => break,
                            }
                        }
                    }
                }
            });
        }
        let deadline = tokio::time::Instant::now() + ENDPOINT_TIMEOUT;
        loop {
            if tokio::time::Instant::now() >= deadline {
                let _ = reap_tree(pid).await;
                let tail = {
                    let buf = recent.lock().unwrap();
                    let start = buf.len().saturating_sub(160);
                    buf[start..].join("\n")
                };
                return Err(format!(
                    "canvas: MCP endpoint not announced within {:?} (app: {})\n--- child output tail ---\n{tail}",
                    ENDPOINT_TIMEOUT,
                    app_dir.display()
                ));
            }
            // 子进程先行退出 = 坏 app（解析失败等），把退出码带给调用方。
            if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
                let _ = reap_tree(pid).await;
                let tail = {
                    let buf = recent.lock().unwrap();
                    let start = buf.len().saturating_sub(160);
                    buf[start..].join("\n")
                };
                return Err(format!(
                    "canvas: auto run exited early (status: {status}) for {}\n--- child output tail ---\n{tail}",
                    app_dir.display()
                ));
            }
            if let Some(base) = found.lock().unwrap().clone() {
                let mcp_port = base.rsplit(':').next().and_then(|p| p.parse().ok()).unwrap_or(port);
                return Ok(Self {
                    app_dir,
                    mcp_base: base,
                    mcp_port,
                    child,
                    storage_file,
                    recent_output: recent,
                    bridge: None,
                });
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    pub fn pid(&mut self) -> Option<u32> {
        self.child.id()
    }

    /// PLAN-102 T-02：embedded 呈现 spawn——bridge 宿主（`auto rqhost
    /// --bridge-ws`，D-1 去窗 rq 宿主 + WS 桥）+ rq remote app（headless
    /// iced，DisplayList v2 帧源；MCP 共位 D-2——vtree/state/act 面经
    /// AUTOUI_MCP_PORT 上线）。数据面（帧/输入）= 前端直连 bridge WS；
    /// 控制面（status/pick/收割）= 既有 MCP/HTTP 路由。失败时两子进程
    /// 均已收割。
    pub async fn spawn_embedded(app_dir: PathBuf) -> Result<(Self, BridgeEndpoint), String> {
        let exe = resolve_auto_exe().ok_or_else(|| {
            "auto executable not found: set AUTO_EXE or build auto-lang (target/release/auto.exe)".to_string()
        })?;
        let port = alloc_port().await?;
        let ws_port = alloc_port().await?;
        // 会话专用 well-known 管道 + 回环 WS 静态 token（pid+纳秒——不引
        // rand 依赖的充分熵；威胁模型 = 本机 stray 进程，同 remote WS 律）。
        let pipe = format!("musk-canvas-{}-{}", std::process::id(), port);
        let token = format!("musk-canvas-{:x}-{:x}", std::process::id(), std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0));
        // ── bridge 宿主子进程（stdout/stderr 不参与监听行发现——诊断弃置；
        //    其就绪判据 = WS 端口可连）。
        let mut bridge_cmd = tokio::process::Command::new(&exe);
        bridge_cmd
            .args(["rqhost", "--pipe", &pipe, "--bridge-ws", &ws_port.to_string(), "--bridge-token", &token])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            // PLAN-102 T-06：stderr 尾随——`adopted \`NAME\`` 解析出订阅名
            //（Hello app_name 精确匹配的 URL 织参源）。
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);
        let mut bridge_child = bridge_cmd
            .spawn()
            .map_err(|e| format!("failed to spawn bridge host: {e}"))?;
        let bridge_app: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        if let Some(berr) = bridge_child.stderr.take() {
            let sink = bridge_app.clone();
            tokio::spawn(async move {
                use tokio::io::{AsyncBufReadExt, BufReader};
                let mut lines = BufReader::new(berr).lines();
                while let Ok(Some(l)) = lines.next_line().await {
                    if let Some(n) = l.split("adopted `").nth(1).and_then(|r| r.split('`').next()) {
                        let mut g = sink.lock().unwrap();
                        if g.is_none() {
                            *g = Some(n.to_string());
                        }
                    }
                }
            });
        }
        // WS 端口就绪等待（~10s；超时 = 收割两子进程后报错）。
        let ws_ready = 'ws: {
            let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
            while tokio::time::Instant::now() < deadline {
                if bridge_child.try_wait().map_err(|e| e.to_string())?.is_some() {
                    break 'ws Err("bridge host exited during startup".to_string());
                }
                if tokio::net::TcpStream::connect(("127.0.0.1", ws_port)).await.is_ok() {
                    break 'ws Ok(());
                }
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
            break 'ws Err(format!("bridge ws not listening within 10s on port {ws_port}"));
        };
        if let Err(e) = ws_ready {
            let _ = bridge_child.kill().await;
            if let Some(pid) = bridge_child.id() {
                reap_tree(Some(pid)).await;
            }
            return Err(e);
        }
        // ── app 子进程（rq remote 臂）：MCP 监听行就绪发现复用既有机制。
        let storage_file = std::env::temp_dir().join(format!(
            "musk-canvas-{}-{}.json",
            std::process::id(),
            port
        ));
        let mut cmd = tokio::process::Command::new(&exe);
        cmd.args(["run", "--render=vm", "-q"])
            .current_dir(&app_dir)
            .env("AUTOUI_MCP_PORT", port.to_string())
            .env("AUTO_VM_STORAGE_FILE", &storage_file)
            // 逻辑表面尺寸（rq 客户端 Hello 凭据——帧/输入坐标系值域）。
            .env("AUTO_VM_WINDOW", "480x680")
            // PLAN-088 debug 捕获面（MCP vtree 通道依赖）。
            .env("AUTO_DEBUG_CAPTURE", "1")
            // PLAN-102 呈现臂：bridge 宿主采纳 + remote 渲染（headless）。
            .env("AUTO_RQHOST_WELLKNOWN", &pipe)
            .env("AUTO_RQHOST_PIPE", &pipe)
            .env("AUTO_VM_RENDER", "remote")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);
        // 不继承任何可能让 VM 复用既有后端/窗口的环境（隔离优先）。
        // 例外：musk 自身被显式赋予 AUTO_REUSE_BACKEND=1 时透传（探针/
        // 合同测试的 dummy 后端约定——canvas-runtime-probe 同款；产线
        // serve 不设 = 零变化）。
        match std::env::var("AUTO_REUSE_BACKEND") {
            Ok(v) if v == "1" => {
                cmd.env("AUTO_REUSE_BACKEND", "1");
                if let Ok(p) = std::env::var("AUTO_HTTP_PORT") {
                    cmd.env("AUTO_HTTP_PORT", p);
                }
            }
            _ => {
                cmd.env_remove("AUTO_REUSE_BACKEND");
                cmd.env_remove("AUTO_HTTP_BASE");
            }
        }

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                // bridge 宿主已在跑——同步收割（同进程树；kill_on_drop 兜底）。
                if let Some(bpid) = bridge_child.id() {
                    reap_tree_blocking(bpid);
                }
                let _ = bridge_child.kill();
                return Err(format!("failed to spawn {}: {e}", exe.display()));
            }
        };
        let app_pid = child.id();
        let mut stdout = child.stdout.take().ok_or("stdout not piped")?;
        let mut stderr = child.stderr.take().ok_or("stderr not piped")?;
        let recent: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let found: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        {
            let recent = recent.clone();
            let found = found.clone();
            tokio::spawn(async move {
                use tokio::io::AsyncBufReadExt;
                let mut es = tokio::io::BufReader::new(stderr).lines();
                let mut os = tokio::io::BufReader::new(stdout).lines();
                loop {
                    tokio::select! {
                        line = es.next_line() => {
                            match line {
                                Ok(Some(l)) => {
                                    if let Some(addr) = parse_listening_line(&format!("E{l}")) {
                                        *found.lock().unwrap() = Some(format!("http://{addr}"));
                                    }
                                    let mut buf = recent.lock().unwrap();
                                    if buf.len() >= 200 { buf.remove(0); }
                                    buf.push(l);
                                }
                                Ok(None) | Err(_) => break,
                            }
                        }
                        line = os.next_line() => {
                            match line {
                                Ok(Some(l)) => {
                                    if let Some(addr) = parse_listening_line(&format!("O{l}")) {
                                        *found.lock().unwrap() = Some(format!("http://{addr}"));
                                    }
                                    let mut buf = recent.lock().unwrap();
                                    if buf.len() >= 200 { buf.remove(0); }
                                    buf.push(l);
                                }
                                Ok(None) | Err(_) => break,
                            }
                        }
                    }
                }
            });
        }
        let deadline = tokio::time::Instant::now() + ENDPOINT_TIMEOUT;
        loop {
            if tokio::time::Instant::now() >= deadline {
                let _ = reap_tree(app_pid).await;
                let _ = bridge_child.kill().await;
                if let Some(bpid) = bridge_child.id() {
                    reap_tree(Some(bpid)).await;
                }
                let tail = {
                    let buf = recent.lock().unwrap();
                    let start = buf.len().saturating_sub(160);
                    buf[start..].join("\n")
                };
                return Err(format!(
                    "canvas: MCP endpoint not announced within {:?} (app: {})\n--- child output tail ---\n{tail}",
                    ENDPOINT_TIMEOUT,
                    app_dir.display()
                ));
            }
            if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
                let _ = reap_tree(app_pid).await;
                let _ = bridge_child.kill().await;
                if let Some(bpid) = bridge_child.id() {
                    reap_tree(Some(bpid)).await;
                }
                let tail = {
                    let buf = recent.lock().unwrap();
                    let start = buf.len().saturating_sub(160);
                    buf[start..].join("\n")
                };
                return Err(format!(
                    "canvas: auto run exited early (status: {status}) for {}\n--- child output tail ---\n{tail}",
                    app_dir.display()
                ));
            }
            if let Some(base) = found.lock().unwrap().clone() {
                let mcp_port = base.rsplit(':').next().and_then(|p| p.parse().ok()).unwrap_or(port);
                let bridge = BridgeChild { child: bridge_child, pipe: pipe.clone(), ws_port, token: token.clone(), app_name: bridge_app };
                let endpoint = BridgeEndpoint { pipe, ws_port, token };
                return Ok((Self {
                    app_dir,
                    mcp_base: base,
                    mcp_port,
                    child,
                    storage_file,
                    recent_output: recent,
                    bridge: Some(bridge),
                }, endpoint));
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// bridge 数据面端点（embedded 会话；windowed 旧轨 None）。
    pub fn bridge(&self) -> Option<&BridgeChild> {
        self.bridge.as_ref()
    }

    /// 子进程是否已退出（看门狗每拍快检，死亡即刻走复活而非等截图失败）。
    pub fn has_exited(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(Some(_)))
    }

    /// 收割进程树并确认端口退场。幂等（taskkill 对已死 pid 报错即忽略）。
    /// PLAN-102 T-02：embedded 会话连带收割 bridge 宿主子进程。
    pub async fn reap(&mut self) {
        let pid = self.child.id();
        let port = self.mcp_port;
        let storage = self.storage_file.clone();
        let bridge_pid = self.bridge.as_mut().and_then(|b| b.child.id());
        let _ = self.child.kill().await; // kill_on_drop 兜底
        if let Some(mut bridge) = self.bridge.take() {
            let _ = bridge.child.kill().await; // kill_on_drop 兜底
        }
        reap_tree(pid).await;
        if let Some(bpid) = bridge_pid {
            reap_tree(Some(bpid)).await;
        }
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
        if let Some(bridge) = self.bridge.as_mut() {
            if let Some(bpid) = bridge.child.id() {
                reap_tree_blocking(bpid);
            }
            let _ = bridge.child.kill();
        }
        let _ = std::fs::remove_file(&self.storage_file);
    }
}

/// PLAN-102 T-02：embedded 会话数据面端点（spawn_embedded 返回——
/// manager 登记 status / 前端直连 bridge WS 消费）。
#[derive(Debug, Clone)]
pub struct BridgeEndpoint {
    pub pipe: String,
    pub ws_port: u16,
    pub token: String,
}

impl BridgeEndpoint {
    pub fn ws_url(&self) -> String {
        format!("ws://127.0.0.1:{}/?token={}", self.ws_port, self.token)
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
