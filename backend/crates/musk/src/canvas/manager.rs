//! PLAN-087 T-02/T-04: canvas 会话管理器 —— 单会话状态机 + 看门狗。
//!
//! 单例语义（M1 定案，§10-3 默认）：`start` 时已有会话先停再起（替换）。
//! 状态机 starting → running ⇄ restarting → stopped | degraded：
//! - 看门狗帧循环 ~1s 拉一次 `autoui_screenshot`；连续 3 次失败或单次超时
//!   → restarting（taskkill 收割 → 退避 1s/2s/4s → 重 spawn 同 app_dir），
//!   超限 3 次 → degraded（stop/start 可恢复）。
//! - "Screenshot skipped"（最小化窗护栏）= 可恢复，不计失败不触发重启。
//! - 子进程死亡（try_wait 快检）即刻走复活路径。
//! - 帧缓存 `(seq, PNG bytes)`：seq 单调递增，前端以 `?t={seq}` 判新帧。
//!
//! 线程模型：manager 持共享状态（`std::sync::Mutex`，status/frame 读取来自
//! 同步工具上下文与 async handler 两侧）；看门狗独占子进程与 MCP 客户端，
//! 经 `watch` 停止旗标受控；stop 额外做一次立即 taskkill（不等看门狗拍）。

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::Serialize;

use super::mcp_client::McpClient;
use super::session::SessionHandle;

/// 连续失败阈值（次）。
const FAIL_THRESHOLD: u32 = 3;
/// 重启退避序列（秒），封顶 3 次。
const BACKOFF_SECS: [u64; 3] = [1, 2, 4];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CanvasState {
    Starting,
    Running,
    Restarting,
    Stopped,
    Degraded,
}

#[derive(Debug, Clone, Serialize)]
pub struct CanvasStatus {
    pub state: CanvasState,
    pub seq: u64,
    pub app_path: String,
    pub restarts: u32,
    pub error: String,
}

/// 共享快照（manager 侧读写，看门狗侧更新）。
#[derive(Clone)]
struct Shared {
    state: Arc<Mutex<CanvasState>>,
    seq: Arc<AtomicU64>,
    app_path: Arc<Mutex<String>>,
    restarts: Arc<Mutex<u32>>,
    error: Arc<Mutex<String>>,
    frame: Arc<Mutex<Option<Arc<Vec<u8>>>>>,
    /// 在途子进程 pid（stop 的立即收割用；None = 无在途进程）。
    pid: Arc<Mutex<Option<u32>>>,
    /// 在途会话 MCP 基址（spawn/复活时登记；agent 工具通道据此建短连客户端）。
    mcp_base: Arc<Mutex<String>>,
}

impl Shared {
    fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(CanvasState::Stopped)),
            seq: Arc::new(AtomicU64::new(0)),
            app_path: Arc::new(Mutex::new(String::new())),
            restarts: Arc::new(Mutex::new(0)),
            error: Arc::new(Mutex::new(String::new())),
            frame: Arc::new(Mutex::new(None)),
            pid: Arc::new(Mutex::new(None)),
            mcp_base: Arc::new(Mutex::new(String::new())),
        }
    }

    fn status(&self) -> CanvasStatus {
        CanvasStatus {
            state: *self.state.lock().unwrap(),
            seq: self.seq.load(Ordering::Relaxed),
            app_path: self.app_path.lock().unwrap().clone(),
            restarts: *self.restarts.lock().unwrap(),
            error: self.error.lock().unwrap().clone(),
        }
    }
}

pub struct CanvasManager {
    shared: Shared,
    stop_flag: Arc<AtomicBool>,
    worker: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl CanvasManager {
    pub fn new() -> Self {
        Self {
            shared: Shared::new(),
            stop_flag: Arc::new(AtomicBool::new(false)),
            worker: Mutex::new(None),
        }
    }

    /// 启动（替换语义）：已有会话先停；spawn 成功后看门狗接管。
    pub async fn start(&self, app_dir: PathBuf) -> Result<CanvasStatus, String> {
        self.stop().await;
        {
            let mut st = self.shared.state.lock().unwrap();
            *st = CanvasState::Starting;
        }
        *self.shared.app_path.lock().unwrap() = display_path(&app_dir);
        *self.shared.restarts.lock().unwrap() = 0;
        *self.shared.error.lock().unwrap() = String::new();
        *self.shared.frame.lock().unwrap() = None;
        self.shared.seq.store(0, Ordering::Relaxed);
        self.stop_flag.store(false, Ordering::Relaxed);

        let handle = SessionHandle::spawn(app_dir.clone()).await;
        match handle {
            Ok(mut h) => {
                let pid = h.pid();
                let base = h.mcp_base.clone();
                let client = McpClient::new(base.clone());
                *self.shared.pid.lock().unwrap() = pid;
                *self.shared.mcp_base.lock().unwrap() = base.clone();
                let shared = self.shared.clone();
                let stop = self.stop_flag.clone();
                let worker = tokio::spawn(async move {
                    watchdog(h, client, shared, stop).await;
                });
                *self.worker.lock().unwrap() = Some(worker);
                tracing::info!("canvas session starting: {} (mcp: {base})", app_dir.display());
                Ok(self.shared.status())
            }
            Err(e) => {
                *self.shared.state.lock().unwrap() = CanvasState::Degraded;
                *self.shared.error.lock().unwrap() = e.clone();
                Err(e)
            }
        }
    }

    /// 停止：置停止旗标 + 立即 taskkill + 等看门狗收尾（上限 ~2s，超时不等）。
    pub async fn stop(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        let pid = *self.shared.pid.lock().unwrap();
        if let Some(pid) = pid {
            super::session::reap_tree(Some(pid)).await;
        }
        let worker = self.worker.lock().unwrap().take();
        if let Some(worker) = worker {
            // 看门狗最多再拍一次（截图超时 10s）——但 pid 已死，screenshot
            // 立即 connection refused，收尾亚秒级；2.5s 上限防极端。
            let _ = tokio::time::timeout(Duration::from_millis(2500), worker).await;
        }
        *self.shared.state.lock().unwrap() = CanvasState::Stopped;
        *self.shared.error.lock().unwrap() = String::new();
    }

    /// serve 关停钩子（graceful shutdown 内调用）。
    pub async fn shutdown(&self) {
        self.stop().await;
    }

    pub fn status(&self) -> CanvasStatus {
        self.shared.status()
    }

    /// 当前帧 PNG 字节（无帧 = None → API 503）。
    pub fn frame(&self) -> Option<Arc<Vec<u8>>> {
        self.shared.frame.lock().unwrap().clone()
    }

    pub fn seq(&self) -> u64 {
        self.shared.seq.load(Ordering::Relaxed)
    }

    /// 在途子进程 pid（测试/诊断用；None = 无在途进程）。
    pub fn pid(&self) -> Option<u32> {
        *self.shared.pid.lock().unwrap()
    }

    /// canvas_snapshot 工具用：当前 app_dir（无会话 = None）。
    pub fn app_dir(&self) -> Option<PathBuf> {
        let p = self.shared.app_path.lock().unwrap().clone();
        if p.is_empty() {
            None
        } else {
            Some(PathBuf::from(p))
        }
    }

    /// agent 工具通道：对在途会话建短连客户端执行一次 MCP 调用（服务器端
    /// 每请求独立处理，与看门狗的帧循环互不阻塞）。会话不存在/已停 → Err。
    pub async fn with_client<T>(
        &self,
        f: impl FnOnce(McpClient) -> T,
    ) -> Result<T, String> {
        let base = self.shared.mcp_base.lock().unwrap().clone();
        if base.is_empty() {
            return Err("canvas: no active session (canvas_run first)".to_string());
        }
        {
            let st = *self.shared.state.lock().unwrap();
            if matches!(st, CanvasState::Stopped) {
                return Err("canvas: session is stopped (canvas_run first)".to_string());
            }
        }
        Ok(f(McpClient::new(base)))
    }
}

impl Drop for CanvasManager {
    fn drop(&mut self) {
        // 同步兜底收割（进程退出路径；kill_on_drop + taskkill 双保险）。
        self.stop_flag.store(true, Ordering::Relaxed);
        let pid = *self.shared.pid.lock().unwrap();
        if let Some(pid) = pid {
            super::session::reap_tree_blocking(pid);
        }
    }
}

/// 展示用路径：剥 Windows canonical 化的 `\\?\` 前缀（面板标题/状态字段
/// 显示友好；进程 spawn 仍用原始 PathBuf，不受影响）。
fn display_path(p: &std::path::Path) -> String {
    let s = p.to_string_lossy();
    s.strip_prefix(r"\\?\UNC\")
        .map(|r| format!(r"\\{r}"))
        .unwrap_or_else(|| {
            s.strip_prefix(r"\\?\")
                .map(str::to_string)
                .unwrap_or_else(|| s.to_string())
        })
        .replace('\\', "/")
}

/// 看门狗主体：帧循环 + 失败判定 + 退避复活（迭代式，独占 SessionHandle）。
/// 结构：外层 = 会话生命周期（初始化+帧循环）→ 收割 → 预算内退避重 spawn
/// （封顶 3 次）→ degraded。stop 旗标在任何等待点检查，置位即收割退场。
async fn watchdog(
    mut handle: SessionHandle,
    mut client: McpClient,
    shared: Shared,
    stop: Arc<AtomicBool>,
) {
    let mut fail_streak: u32 = 0;
    let mut prev_frame: Option<PathBuf> = None;

    'session: loop {
        client.initialize().await;
        *shared.state.lock().unwrap() = CanvasState::Starting;

        // ── 帧循环：~1s 一拍 ──
        loop {
            if stop.load(Ordering::Relaxed) {
                handle.reap().await;
                *shared.pid.lock().unwrap() = None;
                return;
            }
            // 子进程死亡快检：即刻走复活路径（不等 3 连失败）。
            if handle.has_exited() {
                tracing::warn!("canvas: child exited (crash path)");
                break;
            }
            let started = std::time::Instant::now();
            let shot = tokio::time::timeout(
                super::mcp_client::CALL_TIMEOUT + Duration::from_secs(2),
                client.screenshot(),
            )
            .await;
            match shot {
                Ok(Ok((path, bytes))) => {
                    // 新帧入缓存；删上一帧文件防 tmp 堆积。
                    shared.seq.fetch_add(1, Ordering::Relaxed);
                    *shared.frame.lock().unwrap() = Some(Arc::new(bytes));
                    if let Some(prev) = prev_frame.replace(path) {
                        let _ = std::fs::remove_file(prev);
                    }
                    fail_streak = 0;
                    *shared.state.lock().unwrap() = CanvasState::Running;
                }
                Ok(Err(text)) if text.contains("Screenshot skipped") => {
                    // 最小化窗护栏：可恢复，不计失败（下一轮重试）。
                }
                Ok(Err(e)) => {
                    fail_streak += 1;
                    tracing::warn!("canvas: screenshot failed ({fail_streak}): {e}");
                    if started.elapsed() > Duration::from_secs(10)
                        || fail_streak >= FAIL_THRESHOLD
                    {
                        break;
                    }
                }
                Err(_) => {
                    // timeout 包裹层超时（>CALL_TIMEOUT+2s）。
                    fail_streak += 1;
                    tracing::warn!(
                        "canvas: screenshot timeout ({fail_streak}) after {:?}",
                        started.elapsed()
                    );
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(1000)).await;
        }

        // ── 复活路径：收割 → 退避 → 重 spawn（封顶 3 次）→ degraded ──
        handle.reap().await;
        *shared.pid.lock().unwrap() = None;
        loop {
            if stop.load(Ordering::Relaxed) {
                return;
            }
            let restarts = *shared.restarts.lock().unwrap();
            if restarts >= BACKOFF_SECS.len() as u32 {
                tracing::error!("canvas: restart budget exhausted → degraded");
                *shared.state.lock().unwrap() = CanvasState::Degraded;
                *shared.error.lock().unwrap() =
                    "canvas: app process keeps dying (restart budget exhausted)".to_string();
                return;
            }
            let backoff = BACKOFF_SECS[restarts as usize];
            *shared.state.lock().unwrap() = CanvasState::Restarting;
            tracing::info!(
                "canvas: restarting in {backoff}s (attempt {}/{})",
                restarts + 1,
                BACKOFF_SECS.len()
            );
            tokio::time::sleep(Duration::from_secs(backoff)).await;
            if stop.load(Ordering::Relaxed) {
                return;
            }
            let app_dir = handle.app_dir.clone();
            match SessionHandle::spawn(app_dir).await {
                Ok(mut h) => {
                    *shared.restarts.lock().unwrap() = restarts + 1;
                    *shared.pid.lock().unwrap() = h.pid();
                    *shared.mcp_base.lock().unwrap() = h.mcp_base.clone();
                    handle = h;
                    client = McpClient::new(handle.mcp_base.clone());
                    fail_streak = 0;
                    prev_frame = None;
                    continue 'session;
                }
                Err(e) => {
                    *shared.restarts.lock().unwrap() = restarts + 1;
                    *shared.error.lock().unwrap() = e;
                    // spawn 失败不回帧循环（旧 client 已失效），回预算判定。
                }
            }
        }
    }
}
