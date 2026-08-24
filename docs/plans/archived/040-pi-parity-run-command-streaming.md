---
plan_id: PLAN-040
status: archived
feature_name: run_command 对齐 pi bash——tokio 流式输出、超时、进程树终止、尾部截断+临时文件、ToolUpdate SSE 实时进度与 CommandRunner 接缝
author: [zhaopuming]
created_at: 2026-08-23
updated_at: 2026-08-24


supersedes_spec_components:
  - "docs/specs/01-architecture.md: tools.rs 行——RunCommand 重写为流式/超时/进程树杀/临时文件/pi 退出码语义(PLAN-040)"
  - "docs/specs/01-architecture.md: relay/ 行——RunEvent 16→17 变体(+ToolUpdate,SSE-only 易态不落历史)"
  - "docs/specs/01-architecture.md: tool_context.rs 行——ToolContext 增 progress 字段(ProgressSink)"
  - "docs/specs/00-overview.md: §目标 1 Agent 运行——run_command 补齐 pi parity(与 PLAN-039 文件工具 parity 并列)"
new_spec_components:
  - "docs/specs/01-architecture.md: 新增 command_runner.rs 行——CommandRunner trait(Ash 后座契约)+ LocalRunner(tokio 流式/超时杀树)+ ExecOptions/ExecOutcome"
  - "docs/specs/01-architecture.md: 新增 output_accumulator.rs 行——有界内存流式累积器(滚动尾部/UTF-8 流式解码/超限临时文件转储/快照)"
  - "RunEvent::ToolUpdate + ProgressSink: 工具实时进度通道(100ms 节流,chat= session_id/relay=run_id,SSE-only)"
touched_goals:
  - "goal-agent: run_command 对齐 pi bash(流式输出/超时/可控终止/退出码错误化),Agent 运行目标的核心工具能力补全"

current_step: 10
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
   [✅ 已完成] (复审补记:初次标记因全角括号替换未命中而丢失,工作与证据在
   commit c764b6a)command_runner.rs:ExecOptions(on_data/timeout/env)+ExecOutcome
   (combined/exit_code/timed_out)+trait;LocalRunner 双读任务流式拉取+合并通道,
   wait 与 drain **并发 join**(串行在大输出下死锁——管道写满阻塞子进程,实测修复),
   超时 kill_process_tree(Win taskkill /F /T;Unix process_group(0)+killpg,libc 仅
   unix 依赖);tokio features 补 process/time/io-util。8/8 测试绿(含 T7 补充的
   Windows 树杀实测)。
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

## 复审记录

- **复审人**:zhaopuming(经 /auto-plan:review,worktree `plan-040-run-command-streaming` @ 669fb88)
- **时间**:2026-08-24
- **方法**:worktree 内重跑全部验证命令(不信任已勾选项)+ diff 逐文件核对(main..HEAD,20 文件 +1539/-56)

### 验收标准逐项判定

| # | 标准 | 判定 | 证据 |
|---|---|---|---|
| 1 | 10MB 输出:上下文 ≤50KB 尾部 + 完整输出临时文件;前端流式(节流 ≤100ms) | **过**(E2E 人工观察待用户) | `cargo test -- --ignored` 重跑:上下文 <60KB(50KB 尾部+尾注)、临时文件精确 10MB(tools.rs `run_command_smoke_10mb_output`);节流常量 `THROTTLE=100ms`(tools.rs:359);链路组件级全绿:ProgressSink 发布(tool_context 2 测)→ 桥接(extern_impl bus→mpsc)→ SSE 透传(server_stream tool_update 直通;relay 端点 api.rs:341 对 BusEvent 原样序列化)→ 前端(useForge 替换式 partial + ChatsView shell-partial 折叠,vue-tsc 0 错 + build 过) |
| 2 | 超时命令指定秒数被杀,进程树无孤儿 | **过** | `windows_timeout_kills_process_tree_no_orphans` 重跑(5.44s):start /b 孙 ping + 主 ping,3s `taskkill /T /F`,tasklist CSV 前后无增量;Unix 侧 process_group(0)+killpg 代码审查(Windows host 无法运行验证,libc 仅 cfg(unix) 依赖) |
| 3 | 非零退出码错误回喂,agent 循环不中断 | **过**(验收措辞的 ScriptedClient 在 musk 不存在,以下述替代) | 工具层:`run_command_nonzero_exit_is_error_with_output` 断言 Err(Exec) 含 "Command exited with code";循环回喂:auto-ai `tool.at:171` exec_or_msg 把 Err→`[tool error: …]` 字符串回喂(源码确认,非实测) |
| 4 | PAUSED + force 行为与重写前逐项一致 | **过** | T6 四测试重跑:非白名单 Ok(PAUSED) 且未执行 / force 真执行 / force 不豁免 confine / 审批闭环;tools.rs run_command 组 11/11 |

### 回归验证(重跑)

- 全量 `cargo test`:31 个 test target 全部 ok,无失败
- 组件分组重跑:accumulator 13 / command_runner 8 / tool_context 2 / run_command 11(另 1 ignored 冒烟单独跑)
- 前端 `vue-tsc -b` 0 错误;`npm run build` 成功(6.59s)
- `[exit: N]` 旧标记:grep 全仓零引用(前端 isErrorResult 判定不依赖旧格式)

### 与计划的偏差(已核实为合理)

1. `ExecOptions.on_data`:计划 `Box<dyn Fn>` → 实现 `Arc<dyn Fn>`——两个读任务共享同一回调,Box 无法 clone,语义等价。
2. `RunEvent::ToolUpdate` 实际字段是计划签名的超集(+`timestamp`/`tool_name`,全部 `serde default`,wire 兼容);`tool_call_id` 允许空,前端按 tool_name+running 匹配。
3. T9(可选)落地为最小版:仅注入 `MUSK_SESSION_ID`;pi 的 SESSION_FILE/PROVIDER/MODEL 未注入(等前端显示会话上下文时再加)。
4. LocalRunner 的 wait 与 drain 必须**并发 join**——计划未预见:串行等待在大输出下死锁(管道写满→子进程阻塞→永不退出),实施中实测发现并修复,是本计划最重要的实现细节沉淀。

### Debt 候选(不阻塞 merge)

- **DEBT-040-1**:前端流式进度 E2E(起 musk serve + 真实 LLM 会话跑长命令,人工观察 tool_update 渲染)未执行——组件级全绿,端到端待用户冒烟。
- **DEBT-040-2**:Windows `taskkill /T` 覆盖多数场景;若未来发现漏网进程(跨控制台分离),按计划风险节评估 Job Object 兜底。
- **DEBT-040-3**:Unix 侧 killpg 逻辑无运行验证(仅代码审查;开发 host 为 Windows)。
- **DEBT-040-4**:`01-architecture.md` 的 "RunEvent(16)" 计数与 tool_context 行已过时——由 /auto-plan:merge 按 supersedes 元数据更新。

### 二次复审补记(2026-08-24,用户显式重跑 /auto-plan:review)

- 状态确认:reviewed @ aa6469a 后工作树干净、无代码漂移,验收判定与元数据有效。
- 复审修复一处账面缺陷:任务 3 的 `[✅ 已完成]` 标记在执行期因全角括号替换未命中
  静默丢失(10/9)——工作与证据本就在(commit c764b6a,8/8 测试),已补记,10/10。

### 结论

4/4 验收全过,无阻塞 debt → `status: reviewed`,可执行 /auto-plan:merge。
