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

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use serde::Serialize;
use serde_json::{json, Value};

use super::anchor::AnchorIndex;
use super::mcp_client::McpClient;
use super::session::SessionHandle;

/// 连续失败阈值（次）。
const FAIL_THRESHOLD: u32 = 3;
/// 重启退避序列（秒），封顶 3 次。
const BACKOFF_SECS: [u64; 3] = [1, 2, 4];
/// 会话注入 AUTO_VM_WINDOW 的逻辑宽（T-02 契约：scale = 帧宽/此值）。
const WINDOW_LOGICAL_W: f32 = 480.0;

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
    /// 会话 app 相对 workspace 根的路径前缀（源码锚点 workspace 化用）。
    app_rel: Arc<Mutex<String>>,
    /// 锚点索引（PLAN-088 T-03）：帧循环随取 vtree 原子换新；pick/overlay
    /// 读侧短临界，无锁竞争面。None = 尚无可用 vtree。
    anchor: Arc<RwLock<Option<Arc<AnchorIndex>>>>,
    /// 当前选中（pick 置位；层树联动与 chip 渲染源）。
    picked: Arc<Mutex<Option<Value>>>,
    /// agent 高亮 vnode 集合（canvas_overlay 置位；索引换代保留交集）。
    overlay: Arc<Mutex<Vec<u64>>>,
    /// pac 头轻提取（start 时读一次；层树载荷头段）。
    pac: Arc<Mutex<Value>>,
    /// 会话 app 目录原始 PathBuf（span→file:line 解析用；app_path 为展示形）。
    app_dir_raw: Arc<Mutex<Option<PathBuf>>>,
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
            app_rel: Arc::new(Mutex::new(String::new())),
            anchor: Arc::new(RwLock::new(None)),
            picked: Arc::new(Mutex::new(None)),
            overlay: Arc::new(Mutex::new(Vec::new())),
            pac: Arc::new(Mutex::new(Value::Null)),
            app_dir_raw: Arc::new(Mutex::new(None)),
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

    /// 锚点换代：换新索引并把 overlay/picked 收敛到仍存在的 vnode
    ///（热重载后失效 id 自动清理，AC-04 失效臂）。
    fn swap_anchor(&self, next: AnchorIndex, seq: u64) {
        let next = Arc::new(next);
        let alive = |v: u64| next.get(v).is_some();
        {
            let mut ov = self.overlay.lock().unwrap();
            ov.retain(|&v| alive(v));
        }
        {
            let mut pk = self.picked.lock().unwrap();
            if let Some(p) = pk.as_ref() {
                let vid = p
                    .get("vnode_id")
                    .and_then(|v| v.as_str())
                    .and_then(super::tools::parse_vnode_id);
                if matches!(vid, Some(v) if !alive(v)) {
                    *pk = None;
                }
            }
        }
        *self.anchor.write().unwrap() = Some(next);
        let _ = seq; // 索引自带 seq 字段（构建时戳）；此处仅语义对齐调用点。
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
    /// `ws_root` = workspace 根（源码锚点路径前缀：resolve_source 产出
    /// app 相对路径，files API 消费 workspace 相对路径——T-08 实证）。
    pub async fn start(&self, app_dir: PathBuf, ws_root: &Path) -> Result<CanvasStatus, String> {
        self.stop().await;
        {
            let mut st = self.shared.state.lock().unwrap();
            *st = CanvasState::Starting;
        }
        *self.shared.app_path.lock().unwrap() = display_path(&app_dir);
        *self.shared.restarts.lock().unwrap() = 0;
        *self.shared.error.lock().unwrap() = String::new();
        *self.shared.frame.lock().unwrap() = None;
        *self.shared.anchor.write().unwrap() = None;
        *self.shared.picked.lock().unwrap() = None;
        self.shared.overlay.lock().unwrap().clear();
        *self.shared.app_dir_raw.lock().unwrap() = Some(app_dir.clone());
        // app 相对 ws 根前缀（"loop-app" 形态；越界解析已由上游沙箱保证）。
        let app_rel = app_dir
            .strip_prefix(ws_root)
            .unwrap_or(app_dir.file_name().map(|n| Path::new(n)).unwrap_or(app_dir.as_path()))
            .to_string_lossy()
            .replace('\\', "/");
        *self.shared.app_rel.lock().unwrap() = app_rel;
        *self.shared.pac.lock().unwrap() = read_pac_head(&app_dir);
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

    // ── PLAN-088：锚定面（pick/overlay/status_full） ────────────────────────

    /// 当前帧 PNG 像素尺寸（IHDR 直读，无解码；无帧 = None）。
    pub fn frame_px_size(&self) -> Option<(u32, u32)> {
        let f = self.shared.frame.lock().unwrap().clone()?;
        png_size(&f)
    }

    /// 坐标 pick（帧像素入参；T-02 契约：逻辑 = 像素 ÷ scale，scale =
    /// 帧宽/480）。命中 → 实质回溯 → 置 picked 并返回锚点；未命中 → None。
    pub fn pick_at(&self, px: f64, py: f64) -> Option<Value> {
        let (frame_w, frame_h) = self.frame_px_size()?;
        let scale = frame_w as f32 / WINDOW_LOGICAL_W;
        let (lx, ly) = (px as f32 / scale, py as f32 / scale);
        let idx = self.shared.anchor.read().unwrap().clone()?;
        let hit = idx.hit_test(lx, ly)?;
        let vnode = idx.substantive_anchor(hit);
        let app_dir = self.shared.app_dir_raw.lock().unwrap().clone();
        let app_rel = self.shared.app_rel.lock().unwrap().clone();
        let resolver = move |off: usize, len: usize| {
            app_dir
                .as_deref()
                .and_then(|d| AnchorIndex::resolve_source(d, (off, len), ""))
                .map(|(rel, line)| (format!("{app_rel}/{rel}"), line))
        };
        let payload = idx.pick_json(vnode, scale, frame_w, frame_h, &resolver)?;
        *self.shared.picked.lock().unwrap() = Some(payload.clone());
        Some(payload)
    }

    /// vnode 直选（层树联动 / canvas_pick 工具共用）：索引直查，坐标臂同构。
    pub fn pick_vnode_str(&self, vnode: &str) -> Option<Value> {
        let id = super::tools::parse_vnode_id(vnode)?;
        self.pick_vnode(id)
    }

    /// vnode 直选（u64 内核形态；协议面经 pick_vnode_str 的字符串形态）。
    pub fn pick_vnode(&self, vnode: u64) -> Option<Value> {
        let (frame_w, frame_h) = self.frame_px_size()?;
        let scale = frame_w as f32 / WINDOW_LOGICAL_W;
        let idx = self.shared.anchor.read().unwrap().clone()?;
        if idx.get(vnode).is_none() {
            return None;
        }
        let vnode = idx.substantive_anchor(vnode);
        let app_dir = self.shared.app_dir_raw.lock().unwrap().clone();
        let app_rel = self.shared.app_rel.lock().unwrap().clone();
        let resolver = move |off: usize, len: usize| {
            app_dir
                .as_deref()
                .and_then(|d| AnchorIndex::resolve_source(d, (off, len), ""))
                .map(|(rel, line)| (format!("{app_rel}/{rel}"), line))
        };
        let payload = idx.pick_json(vnode, scale, frame_w, frame_h, &resolver)?;
        *self.shared.picked.lock().unwrap() = Some(payload.clone());
        Some(payload)
    }

    /// overlay 置位/清除（canvas_overlay 工具）。id 不在当前索引 = Err
    ///（文案指引找现行 id）；置换语义：clear=true 清空，否则整表替换。
    pub fn set_overlay(&self, ids: Vec<u64>, clear: bool) -> Result<(), String> {
        if clear {
            self.shared.overlay.lock().unwrap().clear();
            return Ok(());
        }
        let idx = self.shared.anchor.read().unwrap().clone();
        let Some(idx) = idx else {
            return Err("canvas_overlay: no anchor index yet (session starting or app has no vtree)".to_string());
        };
        for v in &ids {
            if idx.get(*v).is_none() {
                return Err(format!(
                    "canvas_overlay: vnode_{v} not in current frame (hot reload may have changed ids — re-read via canvas_snapshot / autoui_find)"
                ));
            }
        }
        *self.shared.overlay.lock().unwrap() = ids;
        Ok(())
    }

    /// status 全载荷（M2）：M1 基础字段 + picked + overlay + tree + pac。
    pub fn status_full(&self) -> Value {
        let st = self.shared.status();
        let (frame_w, frame_h) = self.frame_px_size().unwrap_or((0, 0));
        let scale = if frame_w > 0 { frame_w as f32 / WINDOW_LOGICAL_W } else { 1.0 };
        let anchor = self.shared.anchor.read().unwrap().clone();
        let overlay: Vec<Value> = match &anchor {
            Some(idx) => {
                let (fw, fh) = (frame_w.max(1) as f32, frame_h.max(1) as f32);
                self.shared
                    .overlay
                    .lock()
                    .unwrap()
                    .iter()
                    .filter_map(|v| {
                        let n = idx.get(*v)?;
                        let r = n.bbox?;
                        let (px, py, pw, ph) = (r.x * scale, r.y * scale, r.w * scale, r.h * scale);
                        Some(json!({
                            "vnode_id": format!("vnode_{v}"),
                            "bbox_px": { "x": px, "y": py, "w": pw, "h": ph },
                            "bbox_pct": {
                                "x": px / fw * 100.0, "y": py / fh * 100.0,
                                "w": pw / fw * 100.0, "h": ph / fh * 100.0,
                            },
                        }))
                    })
                    .collect()
            }
            None => Vec::new(),
        };
        let app_dir = self.shared.app_dir_raw.lock().unwrap().clone();
        let app_rel = self.shared.app_rel.lock().unwrap().clone();
        let resolver = move |off: usize, len: usize| {
            app_dir
                .as_deref()
                .and_then(|d| AnchorIndex::resolve_source(d, (off, len), ""))
                .map(|(rel, line)| (format!("{app_rel}/{rel}"), line))
        };
        let tree = anchor.map(|a| a.tree_flat_json(&resolver)).unwrap_or(Value::Null);
        let frame = self
            .frame_px_size()
            .map(|(w, h)| json!({ "w": w, "h": h }))
            .unwrap_or(Value::Null);
        json!({
            "state": st.state,
            "seq": st.seq,
            "app_path": st.app_path,
            "restarts": st.restarts,
            "error": st.error,
            "frame": frame,
            "picked": *self.shared.picked.lock().unwrap(),
            "overlay": overlay,
            "tree": tree,
            "pac_head": *self.shared.pac.lock().unwrap(),
        })
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

/// 帧 PNG 尺寸（IHDR 直读：宽 = 16..20、高 = 20..24 大端；签名 8 + 块头 8）。
fn png_size(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 24 || &bytes[12..16] != b"IHDR" {
        return None;
    }
    Some((
        u32::from_be_bytes(bytes[16..20].try_into().ok()?),
        u32::from_be_bytes(bytes[20..24].try_into().ok()?),
    ))
}

/// pac 头轻提取（name/title/render/window；够用即止，M3 深化——AutoConfigReader
/// 进程内复用按计划仍为选项，此处零依赖文本面即可覆盖 M2 层树头段）。
fn read_pac_head(app_dir: &std::path::Path) -> Value {
    let mut o = serde_json::Map::new();
    if let Ok(text) = std::fs::read_to_string(app_dir.join("pac.at")) {
        for line in text.lines() {
            let line = line.trim();
            let Some((k, v)) = line.split_once(':') else { continue };
            let k = k.trim();
            if !matches!(k, "name" | "title" | "title_zh" | "render" | "window") {
                continue;
            }
            let v = v.trim().trim_matches(',').trim();
            let v = v.trim_matches('"');
            o.insert(k.to_string(), json!(v));
        }
    }
    Value::Object(o)
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
                tracing::warn!(
                    "canvas: child exited (crash path)\n--- child output tail ---\n{}",
                    handle.output_tail(30)
                );
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
                    // 锚点索引随帧重建（PLAN-088 T-03）：尽力而为，失败保留
                    // 旧索引（陈旧无害，换代时 overlay/picked 交集收敛）。
                    let seq = shared.seq.load(Ordering::Relaxed);
                    match client.vtree().await {
                        Ok(atom) => match AnchorIndex::parse(&atom, seq) {
                            Ok(idx) => shared.swap_anchor(idx, seq),
                            Err(e) => tracing::debug!("canvas: vtree parse failed: {e}"),
                        },
                        Err(e) => tracing::debug!("canvas: vtree fetch failed: {e}"),
                    }
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
