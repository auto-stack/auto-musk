# 计划 014 交接说明：合并编译验证（335→153 错误）

## Status: SUPERSEDED

> 🗄️ **已归档**（2026-08-03）。本交接为 335→153 的中间记录，已被 `015-merge-compile-zero.md`（153→0）及 commit `f065ff1`（Plan 384 阶段二，全量自动转译 0 产物手改）超越。保留作错误分类（§7）的历史参考。

> 日期：2026-08-01
> 仓库：auto-musk（main 分支，已提交）+ auto-lang（master，已合并所有 P0~P7/G1~G4）
> 计划文档：`docs/plans/014-auto-backend-port.md`（§7 是错误分析）

## 当前状态

24 个 `.at` 文件覆盖 auto-musk 全后端（~15000 行 Rust），全部单独转译 0 错误。
合并编译在真实 musk workspace（`src/auto_generated/`）从 335 个错误降到 **153 个**。

## 流水线

```
cd backend/crates/musk/auto-src
# 1. 转译全部 .at
for f in *.at; do A2R_CRATE_ROOT=0 D:/autostack/auto-lang/target/debug/auto.exe trans --path "$f" rust; done
# 2. nativeize（去 a2r-std + 注入 extern_impl glob import）
for f in *.a2r.rs; do perl nativeize.pl "$f"; done
# 3. 复制到 workspace
for f in *.a2r.rs; do base=$(basename "$f" .a2r.rs); cp "$f" "../src/auto_generated/${base}.rs"; done
mv ../src/auto_generated/lib.rs ../src/auto_generated/auto_lib.rs
mv ../src/auto_generated/main.rs ../src/auto_generated/auto_main.rs
# 4. 编译验证
cd ../../..  # backend/
cargo check --color never 2>&1 | grep -cE "^error"
```

## 剩余 153 个错误分 7 类（详见计划 014 §7）

### C1: String vs &str 类型不匹配（32 个 E0308）— 需 auto-lang worktree

**根因**：a2r 对 str 参数/返回值统一生成 `&str` 或 `String`，但调用方/被调方期望不一致。
a2r 的 str→String 转换是启发式的（有时加 `.to_string()`，有时不加）。

17 个 `expected String, found &str` + 15 个 `expected &str, found String`。

分布：server_serve.rs（11）、auto_lib.rs（5）、orch_tools.rs（2）、server_stream.rs（1）、auto_main.rs（1）、其它（2）。

**方案**：auto-lang worktree 改进 a2r 的 str 参数传递——对函数调用参数统一加
`.to_string()`（当形参是 `String`）；对返回值统一加 `.to_string()`。位置：
`crates/auto-lang/src/trans/rust.rs` 的函数调用生成逻辑（搜索 `.to_string()` 注入点）。

### C2: moved value（20 个 E0382）— 需 auto-lang worktree

**根因**：handler 参数 `s: State<AppState>` 被多个 extern 函数调用消费（move），
a2r 不自动加 `.clone()`。

分布：server.rs（handler 里 `s` 参数重复使用）。

**方案**：auto-lang worktree 改进 a2r——对函数参数被多次使用时自动加 `.clone()`。
或 auto-musk 侧在 .at 手动加 clone（每个重复使用 `s` 的地方加 `s.clone()`）。

### C3 剩余: mpsc Value 类型 + Agent::new + Role trait（~30 个）

**mpsc（9 个）**：a2r 把 `mpsc_sender/receiver` 返回值推断为 i32。server_stream.rs
里 `wf_run_with_progress(s, q, body, Arc::new(sink))` 的 `sink` 字段 `tx` 类型不匹配。
**方案**：extern_impl 里 `mpsc_sender/receiver` 返回类型精确化 + .at 改写。

**Agent::new（4 个 E0277）**：`Agent::new(Arc<dyn Client>, "")` 但实际签名是
`Agent::new(role: impl Role, client: Arc<dyn Client>)`。extern_impl 的 stub 签名错。
**方案**：extern_impl 改 Agent::new 调用（需先构造 Role stub）。

**Role trait 冲突（2 个）**：auto_lib.rs 的 `OwnedRole` derive 了 `Debug/Eq` 等，
但 `Arc<dyn Role>` 字段不支持这些 trait。
**方案**：auto_lib.at 的 OwnedRole 去掉 `#[derive]`（或显式只 derive Clone）。

**AppConfigResp（2 个）**：`app_config_load()` 返回 Value 但调用方期望 AppConfigResp。
**方案**：extern_impl 改返回类型或 .at 内联构造。

### C4 剩余: Subcommand 仅 enum（1 个 error:）

**根因**：auto_main.at 的 `#[derive(Parser, Subcommand)]` 用在 `type Cli` 上，
但 clap 的 Subcommand 只支持 enum。已在 .at 里去掉 Subcommand，但 a2r 转译后
在 src/auto_generated/auto_main.rs 仍残留旧版。

**方案**：重新转译 main.at 并复制到 auto_generated/auto_main.rs。

### C7 剩余: sse 模块路径（5 个 E0433）

**根因**：server_stream.rs 里 `sse::keep_alive()` 被改为 `sse.keep_alive()`，
但 a2r 转译仍可能生成 `sse::` 路径。

**方案**：确保 .at 里用 `sse.keep_alive()`（方法调用）而非 `sse::keep_alive()`。

### E0599（10 个）: 方法不存在

- `Cli.clone()`（1 个）：Cli 没实现 Clone。
- `MethodRouter.remove()`（3 个）：axum 0.8 的 `.route()` 链里 `.delete()` 被误生成。
- 其它（6 个）：各种 stub 类型缺方法。

### E0271（4 个）: Stream Item 类型不匹配

async_stream 生成的 `Stream<Item = Event>` 但 Sse 期望 `Stream<Item = Result<Event, _>>`。

## auto-lang 已完成的改进（Plan 380，全部合并 master）

| 项 | 内容 | commit |
|---|---|---|
| P0 | 元组结构体构造 `Json(v)` | `bd4c475e` |
| P1-str | str 字面量兼容 | (合并) |
| P1-dyn | `Arc<dyn T>` 字段 | `7336edee` |
| P5 | async trait GenericInstance 比对 | `e1125f60` |
| P5b | User 类型通用兜底 | `c487a8d4` |
| P6 | async trait impl 方法 async fn 一致性 | `33e205b1` |
| P7 | impl 块 `#[async_trait]` | `c03858e0` |
| G1 | `#{}` comptime a2r 转译 | `5e4802f3` |
| G2 | `pub use.rust` pub 前缀 | `8f0fdfa3` |
| G3 | 多 derive 透传 | (已工作) |
| G4 | tuple 数组 | (已工作) |

## 下一步（按优先级）

1. **C1（auto-lang worktree）**：a2r str→String 转换改进。影响 32 个错误。
   位置：`rust.rs` 搜索 `.to_string()` 注入逻辑 + str 参数传递。
2. **C2（auto-lang worktree 或 .at 手动）**：clone 注入。影响 20 个错误。
3. **C3 剩余（auto-musk）**：extern_impl 签名精确化 + .at 改写。
4. **重新转译 + 复制 + cargo check** 验证。

## 关键文件

- auto-musk `.at` 源码：`backend/crates/musk/auto-src/*.at`（24 个）
- auto-musk 转译产物：`backend/crates/musk/src/auto_generated/*.rs`（25 个，含 extern_impl.rs）
- auto-musk 计划：`docs/plans/014-auto-backend-port.md`（§7 = 错误分析）
- auto-lang 计划：`docs/plans/380-a2r-rust-interop-completeness.md`
- nativeize 脚本：`backend/crates/musk/auto-src/nativeize.pl`
- extern_impl glue layer：`backend/crates/musk/src/auto_generated/extern_impl.rs`
