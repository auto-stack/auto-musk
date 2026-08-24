//! 执行后端接缝(PLAN-040 T3,对齐 pi `BashOperations`):[`CommandRunner`]
//! trait + 本地 tokio 实现 [`LocalRunner`]。
//!
//! 工具层(run_command)只做参数校验、安全分级与输出格式化;进程生命周期
//! 全部在这里:**流式读** stdout/stderr(on_data 回调,chunk 粒度)、
//! **超时**到点**杀整个进程树**(Windows `taskkill /T /F`;Unix
//! `process_group(0)` + `killpg`,pi `shell.ts killProcessTree` 同源)、
//! 合并输出返回。
//!
//! # Ash 后座(PLAN-040 T8 占位)
//!
//! 未来 Ash 逐命令沙箱就绪后实现本 trait 换掉 `LocalRunner`,**工具层零
//! 改动**。替换实现的契约:
//!
//! - **安全不在这一层**:白名单分类 / PAUSED+force 审批流 /
//!   `confine_command_paths` 都在 run_command 工具层,runner 收到的命令
//!   已过审(Ash 接管安全时,这些检查同步上收到 Ash 策略,工具层保持
//!   现状直至切换日)。
//! - **流式语义**:`on_data` 以 chunk 粒度回调合并输出(stdout+stderr
//!   交错),调用方在回调里做有界累积与节流进度,不做全量缓冲。
//! - **超时杀树**:到点终止命令的**整个**进程组/沙箱实例,输出保留,
//!   `timed_out = true`。
//! - **退出码透明**:runner 原样上报退出码;非零码的"错误化"语义在
//!   工具层(pi:exitCode !== 0 → error result)。
//! - **cwd / env**:以给定 cwd 与叠加 env 执行,不得逸出到别处落盘。

use std::collections::HashMap;
use std::path::Path;

use async_trait::async_trait;
use auto_ai_agent::ToolError;

/// 执行选项(pi `BashOperations.exec` 的 opts)。
#[derive(Default)]
pub struct ExecOptions {
    /// 流式输出回调(stdout/stderr 合并流,chunk 粒度;累积/节流由调用方做)。
    pub on_data: Option<std::sync::Arc<dyn Fn(Vec<u8>) + Send + Sync>>,
    /// 超时(None = 无默认超时,pi 语义);到点杀进程树,输出保留。
    pub timeout: Option<std::time::Duration>,
    /// 额外环境变量(在继承环境上叠加)。
    pub env: HashMap<String, String>,
}

/// 执行结果:合并输出 + 退出码 + 超时标记(非零退出码的"错误化"由工具层做)。
#[derive(Debug, Clone)]
pub struct ExecOutcome {
    /// stdout+stderr 合并字节流(读取顺序交错,近似终端观感)。
    pub combined: Vec<u8>,
    /// 进程退出码(被信号杀死/平台无码时 None)。
    pub exit_code: Option<i32>,
    /// 是否超时被杀(此时输出保留,由工具层追加超时状态文案)。
    pub timed_out: bool,
}

/// 可注入执行后端(接缝:LocalRunner ↔ 未来 Ash 沙箱)。
#[async_trait]
pub trait CommandRunner: Send + Sync {
    async fn exec(&self, cmd: &str, cwd: &Path, opts: ExecOptions) -> Result<ExecOutcome, ToolError>;
}

/// 本地执行后端:`tokio::process` 流式版。Windows `cmd /C`、Unix `sh -c`
/// (沿用重写前的 shell 选择);Unix 侧 `process_group(0)` + `kill_on_drop`。
pub struct LocalRunner;

/// 杀整个进程树(pi `shell.ts killProcessTree`):Windows `taskkill /F /T`;
/// Unix 对进程组 SIGKILL(spawn 时 process_group(0),pgid == pid)。
/// 尽力而为:进程已退出时忽略失败。
pub fn kill_process_tree(pid: u32) {
    if cfg!(windows) {
        use std::process::Stdio;
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/T", "/PID", &pid.to_string()])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    } else {
        // spawn 时 process_group(0) 已把子进程放进独立进程组(pgid == pid),
        // 对负 pid 即对整组 SIGKILL;组杀失败回落单杀(进程可能已退出)。
        #[cfg(unix)]
        unsafe {
            if libc::kill(-(pid as i32), libc::SIGKILL) == -1 {
                let _ = libc::kill(pid as i32, libc::SIGKILL);
            }
        }
        #[cfg(not(unix))]
        let _ = pid;
    }
}

/// 读一个输出流直到 EOF(流被 take 后可能为 None):每个 chunk 喂 on_data
/// 并转发给合并通道。
async fn read_stream<R: tokio::io::AsyncRead + Unpin>(
    reader: Option<R>,
    tx: tokio::sync::mpsc::Sender<Vec<u8>>,
    on_data: Option<std::sync::Arc<dyn Fn(Vec<u8>) + Send + Sync>>,
) {
    use tokio::io::AsyncReadExt;
    let Some(mut reader) = reader else {
        return;
    };
    let mut buf = vec![0u8; 8 * 1024];
    loop {
        match reader.read(&mut buf).await {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                let chunk = buf[..n].to_vec();
                if let Some(f) = &on_data {
                    f(chunk.clone());
                }
                if tx.send(chunk).await.is_err() {
                    break; // 接收端已放弃(超时路径)
                }
            }
        }
    }
}

#[async_trait]
impl CommandRunner for LocalRunner {
    async fn exec(&self, cmd: &str, cwd: &Path, opts: ExecOptions) -> Result<ExecOutcome, ToolError> {
        use std::process::Stdio;
        let mut command = if cfg!(windows) {
            let mut c = tokio::process::Command::new("cmd");
            c.arg("/C").arg(cmd);
            c
        } else {
            let mut c = tokio::process::Command::new("sh");
            c.arg("-c").arg(cmd);
            c
        };
        command
            .current_dir(cwd)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        // 独立进程组:超时时整组收割,不留孤儿(Unix;Windows 走 taskkill /T)。
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        for (k, v) in &opts.env {
            command.env(k, v);
        }

        let mut child = command
            .spawn()
            .map_err(|e| ToolError::Exec(format!("spawn '{cmd}': {e}")))?;
        let pid = child.id();
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        // 两个读取任务并发拉流;合并通道近似终端的交错顺序。原始 tx 在
        // 此 drop——读任务退出(管道 EOF)后 channel 关闭,rx 排空返回
        // None,wait_and_drain 的 drain 循环才能结束(否则死锁)。
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<u8>>(64);
        let h_out = tokio::spawn(read_stream(stdout, tx.clone(), opts.on_data.clone()));
        let h_err = tokio::spawn(read_stream(stderr, tx.clone(), opts.on_data));
        drop(tx);

        // 等待与排空必须**并发**:wait 先行会在大输出下死锁(管道+channel
        // 写满 → 子进程 write 阻塞 → 永不退出)。join 后读任务的 tx 已
        // 全部 drop,drain 收到 None 才返回。
        let mut combined: Vec<u8> = Vec::new();
        let drain = async {
            while let Some(chunk) = rx.recv().await {
                combined.extend_from_slice(&chunk);
            }
        };
        let wait_and_drain = async {
            let (status, ()) = futures::join!(child.wait(), drain);
            status
        };
        let exit_status = match opts.timeout {
            Some(d) => match tokio::time::timeout(d, wait_and_drain).await {
                Ok(s) => s,
                Err(_elapsed) => {
                    if let Some(pid) = pid {
                        kill_process_tree(pid);
                    }
                    // 回收主进程句柄(树已杀,几乎立即返回),再尽力收
                    // 已在管道里的输出——读任务的 tx 已随 wait_and_drain
                    // 被 cancel drop,rx 排空即止。
                    let status = child.wait().await;
                    while let Ok(chunk) = rx.try_recv() {
                        combined.extend_from_slice(&chunk);
                    }
                    // 等读任务收尾,防 kill_on_drop 兜底误杀无关句柄。
                    let _ = h_out.await;
                    let _ = h_err.await;
                    return Ok(ExecOutcome {
                        combined,
                        exit_code: status.as_ref().ok().and_then(|s| s.code()),
                        timed_out: true,
                    });
                }
            },
            None => wait_and_drain.await,
        };
        let _ = h_out.await;
        let _ = h_err.await;
        match exit_status {
            Ok(status) => Ok(ExecOutcome {
                combined,
                exit_code: status.code(),
                timed_out: false,
            }),
            Err(e) => Err(ToolError::Exec(format!("wait '{cmd}': {e}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    fn tmp_dir() -> std::path::PathBuf {
        std::env::temp_dir()
    }

    /// 平台 shell 差异:测试命令按平台给。
    fn shell_cmd(win: &str, unix: &str) -> String {
        if cfg!(windows) { win.to_string() } else { unix.to_string() }
    }

    /// 基本执行:输出流过 on_data、合并输出包含内容、退出码 0。
    #[tokio::test]
    async fn echo_streams_output_with_zero_exit() {
        let chunks = std::sync::Arc::new(std::sync::Mutex::new(Vec::<Vec<u8>>::new()));
        let seen = chunks.clone();
        let opts = ExecOptions {
            on_data: Some(std::sync::Arc::new(move |c| seen.lock().unwrap().push(c))),
            ..Default::default()
        };
        let out = LocalRunner
            .exec(&shell_cmd("echo hello-runner", "echo hello-runner"), &tmp_dir(), opts)
            .await
            .expect("exec ok");
        assert_eq!(out.exit_code, Some(0));
        assert!(!out.timed_out);
        let text = String::from_utf8_lossy(&out.combined).to_string();
        assert!(text.contains("hello-runner"), "combined: {text}");
        assert!(
            !chunks.lock().unwrap().is_empty(),
            "on_data got streaming chunks"
        );
    }

    /// 非零退出码如实上报(错误化在工具层,runner 不做语义)。
    #[tokio::test]
    async fn nonzero_exit_code_reported() {
        let out = LocalRunner
            .exec(&shell_cmd("cmd /C exit 3", "sh -c 'exit 3'"), &tmp_dir(), ExecOptions::default())
            .await
            .expect("exec ok");
        assert_eq!(out.exit_code, Some(3));
        assert!(!out.timed_out);
    }

    /// 超时:到点杀进程,timed_out 置位,及时返回(远小于命令自身的 30s)。
    #[tokio::test]
    async fn timeout_kills_long_running_command() {
        let started = Instant::now();
        let opts = ExecOptions {
            timeout: Some(std::time::Duration::from_secs(1)),
            ..Default::default()
        };
        let out = LocalRunner
            .exec(
                &shell_cmd("ping -n 30 127.0.0.1 > nul", "sleep 30"),
                &tmp_dir(),
                opts,
            )
            .await
            .expect("exec ok");
        assert!(out.timed_out, "must be flagged timed out");
        assert!(started.elapsed() < std::time::Duration::from_secs(10), "killed promptly");
    }

    /// 超时前已产出的输出保留(pi:输出保留 + 状态追加"timed out")。
    #[tokio::test]
    async fn timeout_preserves_output_produced_so_far() {
        let opts = ExecOptions {
            timeout: Some(std::time::Duration::from_secs(2)),
            ..Default::default()
        };
        // Windows:先 echo 再长 ping;Unix:echo 后 sleep。
        let out = LocalRunner
            .exec(
                &shell_cmd(
                    "echo early-output & ping -n 30 127.0.0.1 > nul",
                    "echo early-output; sleep 30",
                ),
                &tmp_dir(),
                opts,
            )
            .await
            .expect("exec ok");
        assert!(out.timed_out);
        let text = String::from_utf8_lossy(&out.combined);
        assert!(text.contains("early-output"), "pre-timeout output kept: {text}");
    }

    /// 大输出(数百 KB)流过不卡死、不丢 on_data(管道并发背压验证)。
    #[tokio::test]
    async fn large_output_streams_through() {
        let total = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let tally = total.clone();
        let opts = ExecOptions {
            on_data: Some(std::sync::Arc::new(move |c| {
                total.fetch_add(c.len(), std::sync::atomic::Ordering::Relaxed);
            })),
            ..Default::default()
        };
        // 20000 行 × ~42B ≈ 840KB。
        let out = LocalRunner
            .exec(
                &shell_cmd(
                    "for /L %i in (1,1,20000) do @echo 0123456789012345678901234567890123456789",
                    "for i in $(seq 1 20000); do echo 0123456789012345678901234567890123456789; done",
                ),
                &tmp_dir(),
                opts,
            )
            .await
            .expect("exec ok");
        assert_eq!(out.exit_code, Some(0));
        assert!(out.combined.len() > 500_000, "combined len: {}", out.combined.len());
        let streamed = tally.load(std::sync::atomic::Ordering::Relaxed);
        assert!(streamed > 500_000, "on_data saw: {streamed}");
    }

    /// spawn 失败(可执行不存在)→ 不 panic;命令层错误(shell 找不到命令的
    /// 非零退出)或 spawn 层 Err 都接受。
    #[tokio::test]
    async fn spawn_failure_is_exec_error() {
        let runner = LocalRunner;
        let res = runner
            .exec("this-command-does-not-exist-xyz --version", &tmp_dir(), ExecOptions::default())
            .await;
        match res {
            Ok(out) => assert_ne!(out.exit_code, Some(0), "missing command cannot succeed"),
            Err(e) => {
                let msg = match e {
                    ToolError::Exec(m) => m,
                    other => format!("{other:?}"),
                };
                assert!(!msg.is_empty());
            }
        }
    }

    /// 进程树杀除函数本身的可执行性(孤儿实测在 T7):
    /// 对不存在 PID 调用不 panic(尽力而为语义)。
    #[test]
    fn kill_process_tree_tolerates_missing_pid() {
        // PID 0xFFF0C0DE 大概率不存在;taskkill 失败被忽略。
        kill_process_tree(0xFFF0_C0DE & 0xFFFF_FFFF);
    }

    /// PLAN-040 T7: Windows 进程树终止实测——`start /b` 分离孙进程 ping +
    /// 主进程 ping,超时 taskkill /T /F 后**无孤儿**(tasklist 口径验证,
    /// 等价任务管理器人工核对;Job Object 兜底方案见计划风险节)。
    #[cfg(windows)]
    #[tokio::test]
    async fn windows_timeout_kills_process_tree_no_orphans() {
        fn count_ping() -> usize {
            let out = std::process::Command::new("tasklist")
                .args(["/FI", "IMAGENAME eq PING.EXE", "/FO", "CSV", "/NH"])
                .output()
                .expect("tasklist runs");
            let text = String::from_utf8_lossy(&out.stdout);
            text.lines().filter(|l| l.to_uppercase().contains("PING.EXE")).count()
        }
        let before = count_ping();
        let opts = ExecOptions {
            timeout: Some(std::time::Duration::from_secs(3)),
            ..Default::default()
        };
        // start /b 在 cmd 内启动分离孙进程 ping;第二个 ping 保持主命令存活
        // 直到超时——树:cmd →(start /b)ping 孙 + ping 子。
        let started = std::time::Instant::now();
        let out = LocalRunner
            .exec("start /b ping -n 60 127.0.0.1 & ping -n 60 127.0.0.1", &tmp_dir(), opts)
            .await
            .expect("exec ok");
        assert!(out.timed_out, "tree runs until timeout");
        assert!(
            started.elapsed() < std::time::Duration::from_secs(15),
            "killed promptly (elapsed {:?})",
            started.elapsed()
        );
        // 给 taskkill 的异步收割留收尾时间,再验证无新增 ping 孤儿。
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let after = count_ping();
        assert!(
            after <= before,
            "process-tree kill left orphans: before {before}, after {after}"
        );
    }
}
