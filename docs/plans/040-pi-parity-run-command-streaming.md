---
plan_id: PLAN-040
status: execution_done
feature_name: run_command 对齐 pi bash——tokio 流式输出、超时、进程树终止、尾部截断+临时文件、ToolUpdate SSE 实时进度与 CommandRunner 接缝
author: [zhaopuming]
created_at: 2026-08-23
updated_at: 2026-08-24

supersedes_spec_components: []
new_spec_components: []
touched_goals: []

current_step: 5
total_steps: 10
---

# [PLAN-040] run_command 重写：流式、超时、可控终止

## 变更摘要

现状 `run_command`（`backend/crates/musk/src/tools.rs:141`）用
`std::process::Command::output()` 阻塞等待完成：**无超时、无流式、无取消、
输出无上限全量缓冲**。跑一次 `cargo build` 就是数分钟黑盒 + 数 MB 输出全量进
上下文。非零退出码只追加 `[exit: N]` 标记（`Ok` 返回），模型可能忽视。

pi 的 bash 工具（510 行 + 222 行输出累积器 + 276 行截断模块）把这件事工程化为：
流式累积 + 有界内存 + 尾部截断 + 超限落临时文件 + 100ms 节流进度推送 + 进程树
级终止。本计划按 pi 模式重写，并利用 musk 已有的 SSE 基础设施把实时输出推给
Web 前端。白名单审批流（PAUSED + `force`）与 `confine_command_paths` 原样保留。

**前置**：PLAN-039 的共享截断模块 `tool_truncate.rs`（`truncate_tail`）。
**依赖协调**：auto-ai PLAN-026 的取消语义落地后，run_command 的终止与
Agent 级 CancellationToken 打通（本计划先用工具本地超时，不强依赖）。

## pi 参考实现索引

pi 仓库路径前缀 `D:\github\pi\packages\coding-agent\src\core\tools\`：

| 关注点 | pi 位置 | 移植要点 |
|---|---|---|
| 流式输出经 `onData` 回调进 `OutputAccumulator` | `bash.ts:393-397`（`handleData`） | tokio 版：`BufReader::lines()` 两个任务读 stdout/stderr |
| 有界内存累积器：滚动尾部缓冲（2×maxBytes）+ UTF-8 边界安全裁剪 + 行边界跟踪 | `output-accumulator.ts:35`（`OutputAccumulator` 类，裁剪在 `trimTail`:179 的字节边界扫描） | Rust：`String` 尾部缓冲 + `is_char_boundary` 回退 |
| 超限自动落临时文件，路径随结果给模型 | `output-accumulator.ts:205-221`（`shouldUseTempFile`/`ensureTempFile`）+ `bash.ts:413-424`（尾注构造 `Full output: {path}`） | `std::env::temp_dir()` + 随机后缀 |
| **尾部截断**（保末尾——错误与最终结果在末尾），read 才截头 | `truncate.ts:168`（`truncateTail`）；头/尾选择的设计注释在文件头 | PLAN-039 的 `truncate_tail` |
| 截断尾注带行号区间与总量：`[Showing lines 51-5000 of 5000. Full output: …]` | `bash.ts:409-427`（`formatOutput`） | 文案照抄语义 |
| 100ms 节流的进度更新（`onUpdate` 流式部分结果） | `bash.ts:206`（`BASH_UPDATE_THROTTLE_MS`）+ `bash.ts:374-387`（节流调度） | musk 对应 ToolUpdate SSE，节流同值起步 |
| 超时 → 杀整个进程树（detached + tracked PID） | `bash.ts:115-127`（`killProcessTree` 调用点）；实现在 `D:\github\pi\packages\coding-agent\src\utils\shell.ts` | Windows：`taskkill /PID {pid} /T /F`；Unix：`process_group(0)` + `killpg`（tokio::process::Command 支持） |
| 非零退出码 = 错误结果（输出保留 + 状态追加） | `bash.ts:456-458`（`exitCode !== 0` → throw） | musk 改为 `Err(ToolError::Exec(output + "Command exited with code N"))`，`exec_or_msg` 会转字符串回喂，循环不断 |
| 可注入执行后端 `BashOperations`（工具层只做格式化，执行解耦） | `bash.ts:62-80`（接口）+ `:88`（本地实现工厂） | musk 对应 `CommandRunner` trait——Ash 后座的接缝（见任务 8） |
| timeout 参数语义（可选、无默认超时、上限校验） | `bash.ts:28-39`（`resolveTimeoutMs`）+ schema `:41-44` | 照搬：可选 timeout 秒参数 |
| spawn 前环境注入（session/model 元数据） | `bash.ts:164-190`（`resolveSpawnContext`，PI_* 变量） | 可选：注入 MUSK_SESSION_ID 等；价值待前端显示会话上下文时体现，标记可选任务 |

## 方案

### 1. `command_runner.rs`（新文件）：执行后端 trait

```rust
pub trait CommandRunner: Send + Sync {
    async fn exec(&self, cmd: &str, cwd: &Path, opts: ExecOptions)
        -> Result<ExecOutcome, ToolError>;
}
pub struct ExecOptions { pub on_data: Option<Box<dyn Fn(Vec<u8>) + Send + Sync>>, pub timeout: Option<Duration>, pub env: HashMap<String,String> }
```

本地实现 `LocalRunner`：`tokio::process::Command`，Windows `cmd /C`、Unix `sh -c`
（沿用现状）；Unix 侧 `process_group(0)` + `kill_on_drop(true)`。

进度回调经 musk 已有的进程级 broadcast 总线发出（`RunEvent` 新增
`ToolUpdate { run_id, tool_call_id, partial }` 变体，100ms 节流），SSE 端点复用
现有 `/api/chats/session/{id}/stream`——**不需要动 auto-ai 的 StreamEvent**
（工具在 musk 进程内，经 ToolContext 拿 sender；auto-ai 侧事件协议升级由
PLAN-026 独立推进，两轨将来在 relay 桥接层合流）。

### 2. 输出处理 `output_accumulator.rs`（新文件）

照 pi 语义：滚动尾部缓冲（100KB）+ 临时文件转储（超限时）+ `TruncationResult`；
结束快照经 PLAN-039 的 `truncate_tail`（2000 行/50KB）。

### 3. `run_command` 工具重写

- 参数：`cmd`、`force`（保留）、新增可选 `timeout`（秒）；
- 白名单分类与 `confine_command_paths` 不动，置于执行最前；
- 流程：分类 → confine → `CommandRunner::exec`（on_data 进累积器 + 节流推
  ToolUpdate）→ 快照截断 → 尾注（行区间 + Full output 路径）→ 退出码非零转
  `ToolError::Exec`；
- 超时：到点杀进程树，输出保留 + `Command timed out after Ns` 状态追加。

## 任务分解（10 步）

1. `output_accumulator.rs` + 单测（流式分块、多字节边界、超限转临时文件、行计数）。
   [✅ 已完成] TDD 红→绿:`backend/crates/musk/src/output_accumulator.rs`(滚动尾部+流式 UTF-8 解码+临时文件转储+pi snapshot 语义),13/13 测试过;与 pi 分野:临时文件失败优雅降级不 throw。
2. `RunEvent::ToolUpdate` 变体 + relay driver 桥接 + SSE 透传；前端 ToolCall 块
   渲染 partial（增量追加 + 折叠）。
   [✅ 已完成] store.rs+auto_generated/relay_store.rs(wire parity 手补)ToolUpdate 变体
   +event_type/timestamp 分支;conversation.rs run_event_to_turns 空分支(partial 不
   落历史);chat SSE:extern_impl 桥接任务(bus→mpsc,run 结束 abort)+ server_stream
   chat_sse_stream tool_update 原样透传(SseEventDto 枚举外);relay SSE 经 BusEvent 自动
   透传;前端 useForge tool_update 增量追加(按 name+running 匹配)+ ChatsView shell 卡
   partial 折叠渲染(flex 贴底);useRelay if 链未知类型忽略。ProgressSink 单测 2/2 绿。
3. `CommandRunner` trait + `LocalRunner`（tokio 流式读、进程组、超时杀树）+ 单测
   （sleep 超时、子进程孤儿清理）。
4. `run_command` 重写接 runner + 累积器 + 截断尾注 + 退出码语义。
   [✅ 已完成] tools.rs RunCommand 重写:timeout 参数(pi resolveTimeoutMs 校验:
   >0 有限/上限 i32::MAX ms)→ LocalRunner.exec(on_data:累积器 append+100ms 节流
   ProgressSink 快照推送)→ finish → snapshot(true) → pi formatOutput 三态尾注
   (lastLinePartial/lines/bytes + Full output 路径,临时文件失败退化说明文本)→
   超时/非零退出码 = ToolError::Exec(appendStatus:输出保留+状态追加);PAUSED 与
   confine 逻辑零改动;lib.rs 装配换 with_root_and_progress;前端 partial 改快照
   替换式。新测试 4 + 更新旧封顶测试(临时文件真实存在+全量字节),tools:: 60/60、
   全量 31 target 全绿;[exit: 旧标记 grep 零引用。
5. `tool_context.rs` 扩展：ToolContext 挂 broadcast sender（工具侧推 ToolUpdate）。
   [✅ 已完成] ToolContext 新增 progress: Option<ProgressSink>(进程级 broadcast 总线
   sender + run_id;send() 推 RunEvent::ToolUpdate);4 个构造点全接:relay/driver.rs
   (run_id)、hw server.rs、ag extern_impl(session_id)、ag relay_driver.rs(run_id)。
   (作为 T2 桥接的前置依赖与 T2 合并落地;单测见 tool_context.rs 2/2 绿)
6. 白名单/force/confine 回归测试（确保重写未削弱安全层）。
   [✅ 已完成] tools.rs 新增 4 测试:非白名单 whoami → Ok("⏸ PAUSED")且**未执行**
   (若误执行会返回用户名);force=true 真执行;force **不豁免** confine(白名单
   type/cat + C:\Windows 绝对路径仍拒);PAUSED→force 审批闭环(Ok→Ok)。
   10/10 run_command 测试绿。
7. Windows 进程树终止实测（`taskkill /T /F`，cmd → 子进程链）。
   [✅ 已完成] command_runner.rs windows_timeout_kills_process_tree_no_orphans:
   cmd→(start /b)孙 ping+主 ping,3s 超时 taskkill /T /F,tasklist CSV 口径验证
   杀树前后 ping 数无增(任务管理器等价),5.4s 通过。Job Object 兜底未需要。
8. `CommandRunner` 的 Ash 后端占位文档（未来 Ash 逐命令沙箱就绪后换实现，工具层
   零改动）。
   [✅ 已完成] command_runner.rs 模块文档"Ash 后座"节:替换契约五条——安全不在
   runner 层(分类/PAUSE/confine 留工具层,切换日上收 Ash 策略)、流式 on_data chunk
   语义、超时杀树、退出码透明(错误化在工具层)、cwd/env 不逸出。
9. （可选）MUSK_* 环境变量注入。
   [✅ 已完成·轻量] progress 通道存在时注入 MUSK_SESSION_ID(pi PI_* 对应,chat 场景
   =session_id/relay=run_id);无订阅(测试/CLI)不注入。测试:注入/不注入双向断言。
10. 回归：`cargo test` + 手工冒烟（长输出命令、超时命令、非零退出、PAUSED 流程）。
   [✅ 已完成] 全量 cargo test 31 target 全绿(含本计划新增:accumulator 13+runner 8+
   tool_context 2+tools 组 12);10MB 验收冒烟(cargo test -- --ignored):上下文 60KB
   内含尾注+临时文件精确 10MB,0.19s;超时/非零退出/PAUSED 由常驻测试覆盖;前端
   vue-tsc 0 错误 + vite build 成功。

## 验收标准

- `run_command` 跑 10MB 输出命令：上下文收到 ≤50KB 尾部 + 完整输出在临时文件，
  前端全程可见流式进度（ToolUpdate 节流 ≤100ms 间隔）。
- 超时命令在指定秒数被杀，进程树无孤儿（Windows 任务管理器验证）。
- 非零退出码以错误回喂（ScriptedClient 断言 ToolResult 含 exit code 文本），
  agent 循环不中断。
- 白名单 PAUSED + force 审批流行为与重写前逐项一致。

## 风险

- Windows 进程树终止是历史难点（cmd /C 的孙进程）：`taskkill /T` 覆盖多数场景，
  Job Object 为兜底方案（若实测有漏网进程，登记 KNOWN-DEBT 再评估）。
- ToolUpdate 事件量：100ms 节流 + 前端折叠渲染是 pi 验证过的组合，但 musk 的
  SSE 经 axum broadcast，需确认背压策略（丢弃旧 partial 可接受——partial 本就
  是易态）。
- 退出码语义从 `Ok+[exit: N]` 改为 `Err`：改变模型可见行为，属有意对齐 pi
  （错误更显眼、自愈更快）；技能文档若依赖旧标记格式需同步 grep 更新。
