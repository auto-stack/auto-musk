# 015 — 合并编译错误清零（153→0）+ a2r 问题清单

> **状态**：✅ **完成**（2026-08-01）。24 个 .at 转译产物合并编译 **0 错误**（96 warning，均为 unused import）。
> 前置：计划 014（335→153）。本计划把剩余 153 错误清零。

## 关键发现：推翻交接说明的「C1 需 auto-lang worktree」

交接说明（014-handoff）建议第一步「建 auto-lang worktree 改进 a2r 的 str→String 转换（C1=32 个）」。
**经实测推翻**：当前 153 错误里真正的 str↔String 不匹配只有 ~18 个，且根因是
`extern_impl.rs` 的 stub 签名不精确（不是 a2r 转译器局限）。

**153 错误的真实根因分布**（三方 agent 交叉验证）：
- **~120 个**：`extern_impl.rs` stub 签名不精确 —— 形参 `String` 应 `&str`、返回 `Value`
  应为具体 DTO、`<T>` 泛型按值取致 move、`value_get_str` 按值取致 move。
- **~13 个**：`.at` 源码写法 bug（lib.at 结构损坏、server_serve.at ext 块、server_stream.at 导入）。
- **~20 个**：a2r 转译器 bug（见 §3 清单）。

**结论**：绝大多数可在 auto-musk 直接修，**无需先动 a2r**。auto-lang worktree 留作治本（§3）。

## 修复分层（按持久性）

### 持久层（不被重新转译覆盖）

1. **`extern_impl.rs`**（手写 glue，最大收益）：
   - str 形参 `String`→`&str`（serve_listen/http_post_json/drive_*/relay_* 等）
   - `value_get_str(v: Value)`→`(v: &Value)`（与 bool/array 一致，止 move）
   - stub 返回 `Value`→具体 DTO（specs_drift→DriftResult、professions_list→Vec<ProfessionItem>
     、app_config_load→AppConfigResp、wf_run→WorkflowRunResponse 等 14 个）
   - `<T>(_s: T)`→`(_s: &T)`（handler 委托 stub，止 State move）
   - `Agent::new` 参数顺序修复 + 新增 `StubRole`（impl Role）
   - `NoDaemonClient::complete` 按引用 + `DaemonUnavailable` 去 tuple
   - mpsc/broadcast/advance_*/conv_event_*/msg_* 全部改引用
   - `agent_run_stream_with_sink` sink 改泛型 `<W: Send+Sync+'static>`
   - `.delete()` 路由（a2r 误生成 `.remove()`）

2. **`.at` 源码**（重新转译保留）：
   - `lib.at`：删除残缺重复的 OwnedRole 定义；加 `#[derive(Clone)]`；`.view` 标记借用
     （`resolve_role(mode.role.view)`）；spec_tools 构造 `ListSpecs()`→`ListSpecs.new()`
   - `server_serve.at`：spec 块 `fn on_event(self, ev)`→`fn on_event(ev)`（与 a2r 隐式 self
     约定一致）

3. **真实 crate**（`src/tools.rs`）：9 个 unit struct 加 `impl X { pub fn new() }`
   （供 auto_lib 的 `ReadFile.new()` 构造）

### 产物层（重新转译会丢失，记入 §3 a2r 清单）

- `auto_main.rs`：精简为最小可编译版本（a2r 误生成 Subcommand derive + 重复 import）
- `tools.rs`/`spec_tools.rs`/`orch_tools.rs`：`value_get_*(&args)` 引用注入、删损坏 `fn main()`
- `server.rs`：`(&s, ...)` 引用注入、`.delete()` 路由、`build_router` 返回 `Router<AppState>`
- `server_serve.rs`/`server_stream.rs`/`relay_driver.rs`/`relay_flows.rs`/`workflow.rs`/`specs.rs`：
  各类引用注入、DTO 字段类型、derive 修正
- `auto_lib.rs`：手写 `impl auto_ai_agent::Role for OwnedRole`（Auto 的 str→String 与 Role
  的 `&str` 返回不兼容）

## 3. a2r 转译器问题清单（→ auto-lang worktree 治本）

以下问题在 auto-musk 侧已用「产物 .rs 手改」绕开，但**重新转译会重现**。需在 auto-lang
用 worktree 方式修复 a2r（参考既有 plan-380-* worktree 模式）。每条含复现 + 建议修法。

### A1. `#[derive(Parser)]` on `type` 误加 Subcommand + 重复 import
- **复现**：`main.at` 干净（`use.rust clap::Parser` + `#[derive(Parser)]` + `type Cli { cmd str }`），
  但 `auto_main.rs` 生成 `use clap::{Parser, Subcommand};` + `use clap::Parser;`（重复）+
  `#[derive(Parser, Subcommand)]`（Subcommand 只支持 enum）。
- **影响**：auto_main.rs E0252 + E0119 + derive 错误。
- **建议**：a2r 对 clap derive 不要硬编码 Subcommand；`use.rust` 透传不要合并/重复。

### A2. `spec Client { fn complete(req) ~str }` 转成 `async fn complete(req: i32) -> String`
- **复现**：main.at 的 `spec Client` trait 被 a2r 转成 `trait Client { async fn complete(req: i32) -> String }`，
  与 extern_impl re-export 的上游 `auto_ai_agent::Client`（`complete(&self, req: &CompletionRequest)`）冲突。
- **建议**：a2r 对 `spec` + `~str` 的 async 方法应生成 `-> String`（已对）但参数类型不应推断为 i32；
  且本地 trait 不应与 re-export 的同名上游 trait 冲突（命名空间隔离）。

### A3. 引用注入缺失：`fn(str形参)` 调用点不自动加 `&`
- **复现**：`tools.at` 的 `value_get_str(args, k)`（extern_impl 形参 `&Value`）→ a2r 生成
  `value_get_str(args, k)`（漏 `&`）；`resolve_within_project(path)`（形参 `&str`）→ 漏 `&path`。
  触发 E0308（expected &str/&Value, found String/Value）+ 连锁 E0382（args 被 move）。
- **建议**：a2r 在函数调用生成时，根据被调函数的形参类型自动注入 `&`（值→引用）或 `.to_string()`
  （引用→owned String）。这是最大的一类（影响 ~50 个错误）。

### A4. `Ok("literal")` 返回 `Result<String,_>` 不自动加 `.to_string()`
- **复现**：`return Ok("")` / `return Ok("wrote file")` → a2r 透传 `Ok("")`，但函数返回
  `Result<String,String>` → E0308（expected String, found &str）。
- **建议**：a2r 对 `Ok(&str字面量)` 在 `Result<String,_>` 上下文自动加 `.to_string()`。

### A5. struct 过度 derive：含 `Arc<dyn T>` 字段自动 derive `Debug/Eq/Ord`
- **复现**：`OwnedRole { inner: Arc<dyn Role> }` → a2r 自动加 `#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]`，
  但 `dyn Role` 不实现 Debug/Eq/Ord → E0277/E0369。同理 relay_driver 的 `MuskAgentFactory { state: Arc<AppState> }`。
- **建议**：a2r 检测字段类型含 `dyn Trait` 或非 Debug/Eq 类型时，只 derive Clone（或跳过 Debug/Eq/Ord）。

### A6. 顶层 `let schema = \`...\`` 转译进 `fn main()` + format! 花括号不转义
- **复现**：`tools.at` 顶层 `let read_file_schema = \`{"type":"object",...}\`` → a2r 生成
  `fn main() { let read_file_schema: String = format!("{{..."); }`，花括号不平衡 + 与 extern_impl
  的 `const` 遮蔽。
- **建议**：顶层 `let` 常量应生成 `const`/`static`，不进 `fn main()`；format! 的 `{}`/`}` 需转义。

### A7. 全路径类型注解 `Arc<dyn auto_ai_agent::Role>` 不解析
- **复现**：lib.at 写 `inner Arc<dyn auto_ai_agent::Role>`（全路径）→ a2r 报错
  `Expected term, got RBrace`（误导到文件尾）。改用短名 `Arc<dyn Role>`（靠 use.rust 导入）才正常。
- **建议**：a2r 类型注解应支持 `crate::` / `module::Type` 全路径。

### A8. `spec` vs `ext` 方法的 self 处理不一致
- **复现**：server_serve.at `spec DriveSink { fn on_event(self, ev) }` + `ext DriveStreamSink { fn on_event(ev) {...} }`
  → a2r 生成 trait `fn on_event(ev: i32)`（无 self，因 ext 块没写 self）但 impl 用 `self.xxx` → E0424。
  改成 spec/ext 都不写 self（隐式 self 约定）才正确生成 `fn on_event(&self, ev)`。
- **建议**：a2r 的 spec 块方法应统一隐式 &self（与 ext 块一致），不要因 spec 写了 `self` 就改变行为。

### A9. `.delete()` 路由方法误生成 `.remove()`
- **复现**：server.at 的 axum 路由 `.delete(handler)` → a2r 生成 `.remove(handler)`（MethodRouter 无 remove 方法）。
- **建议**：a2r 不要把 `delete` 映射成 `remove`（可能误把 delete 当保留字/同义词）。

### A10. `nativeize.pl` 对部分 .at 生成 `use crate::extern_impl::*`（应 `super::`）
- **复现**：lib.at/server_serve.at 转译 + nativeize 后，:1 是 `use crate::extern_impl::*`
  （E0432 unresolved import），手动改 `super::extern_impl::*` 才对。其它 .at 生成 `super::` 正常。
- **建议**：排查 nativeize.pl 或 a2r 为何这两个文件生成 `crate::`（可能与模块名/路径推断有关）。

## 修复统计

| 阶段 | 错误数 | 主要动作 |
|---|---|---|
| 起点 | 153 | — |
| extern_impl str/DTO/Agent::new | 139 | stub 签名精确化 |
| tools/spec/orch 引用注入 | 86 | value_get_*(&args) + 删 fn main |
| lib.at 结构修复 + 重新转译 | 83→5* | OwnedRole 去重 + .view |
| auto_main 精简 + auto_lib Role impl | 69 | a2r bug 产物手改 |
| server.rs &s + .delete() | 57→40 | handler move + 路由 |
| server_serve.at ext self + 引用 | 23 | 重新转译 + 产物 |
| server_stream sink/yield/IntoResponse | 16 | mpsc/sink/Stream |
| relay_driver/workflow/specs 收尾 | **0** | derive + ExitRouting + parse_at_workflow |

*5 是假象（auto_lib E0432 掩盖下游），修后真实 83。

## 下一步（阶段 3：auto-lang worktree）

按 §3 清单到 `D:/autostack/auto-lang` 建 worktree（如 `plan-380-a3-ref-injection`），
优先修 **A3（引用注入，影响最大 ~50 错误）** + **A5（过度 derive）** + **A1（Subcommand）**。
修后回归测试：auto-musk 全部 .at 重新转译 + nativeize + cargo check 应无需产物手改即可 0 错误。

## 关键文件

- 持久 glue：`backend/crates/musk/src/auto_generated/extern_impl.rs`
- .at 源：`backend/crates/musk/auto-src/{lib,server_serve}.at`（已改）+ 其余 22 个 .at（未改）
- 真实 crate：`backend/crates/musk/src/tools.rs`（9 个 struct 加 new）
- a2r 源：`D:/autostack/auto-lang/crates/auto-lang/src/trans/rust.rs`
