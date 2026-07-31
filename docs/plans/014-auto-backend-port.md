# 014 — auto-musk 后端的 Auto 语言版本（a2r 转译可回 Rust）

> **状态**：阶段 0 PoC 完成（2026-07-31）。阶段 1/2 待启动。
> **目标**：把 auto-musk 的 Rust 后端用 Auto 语言重写一份（`.at`），
> 经 a2r 转译回 Rust 后，实现与现有 Rust 版本一致的能力。
> **前置现实**：a2r 运行时（a2r-std）目前不含 axum/tokio/SSE，故整个后端
> 等价转译**尚不可达**；本计划以「PoC 实测驱动 + 分层推进」务实推进。

---

## 阶段 0 — PoC：实测 a2r 边界 ✅

### 目的

用一个真实、axum-free 的数据模型切片，端到端跑通
`.rs → .at（手写）→ a2r → cargo check`，用事实回答「a2r 现在到底能处理什么」。
结果直接决定阶段 1 的分层比例。

### 选材

`backend/crates/musk/src/specs.rs` 的**数据模型子集**（零 axum/tokio/async）：
- 2 个 scalar enum：`SectionType`（7 变体）+ `SpecStatus`（23 变体）
- 字符串转换：`as_str` / `to_str` / `from_str_lossy`
- `SpecItem` struct（17 字段：String / Vec<String> / Option×6 / u64）
- `SpecItem::new` factory（时间桥接 + struct 字面量构造）
- serde derive（`#[derive(Serialize, Deserialize)]`）

### 工具链（已实测，照搬 auto-lang-creator `tests/verify.sh`）

```
转译：  A2R_CRATE_ROOT=0 D:/autostack/auto-lang/target/debug/auto.exe trans --path X.at rust
产物：  X.a2r.rs（与 .at 并列，不覆盖手写 .rs）
验证：  临时 crate + a2r-std/auto-atom/auto-val path 依赖 + cargo check
基线：  auto-lang master @ d0a96bf7（auto.exe 已编译就绪）
```

### 迭代过程（6 错 → 0 错）

PoC 手写源码：`backend/crates/musk/auto-src/specs.at`
转译产物：`backend/crates/musk/auto-src/specs.a2r.rs`

| 轮次 | cargo 错误数 | 主要问题 | 处置 |
|---|---|---|---|
| v1 | 6 | 见下表 a2r-1~3 | — |
| v2 | 4 | 修 a2r-1（重复定义） | 删手写 `from_id`，用 a2r 自动版 |
| v4 | 5 | 修 a2r-2（str_as_str） | `from_str_lossy` 字符串 `is` 改 Map 查表 |
| v5 | 3 | 修 A4（map.get 返回 Option<&T>） | `is table.get(s) { Some(v) -> v.clone() }` |
| v6 | 2 | 删自测 smoke | a2r-3（str 实参注入）局限在测试代码 |
| v9 | 2 | 加 serde derive | a2r-4：显式 derive 覆盖自动全套 |
| v10 | 4 | enum 也要 serde | A23：显式 derive 须补 Clone, Debug |
| **v12** | **0** ✅ | 全套 derive 补齐 | 完成 |

### a2r 限制发现（实测得到，回灌本计划）

| # | 现象 | 触发 | 规避 |
|---|---|---|---|
| **a2r-1** | scalar enum 的 `from_id` **重复定义**（E0592/E0034）| 手写 `from_id` 与 a2r 自动生成版冲突 | 不要手写 `from_id`/`from_*`；改用 a2r 自动版，或自定义名（如 `parse`） |
| **a2r-2** | 字符串 `is` 匹配生成 unstable `str_as_str`（E0658）| `is s { "x" -> ... }`（s 是 str）| 改用 Map 查表（`table.get(s) { Some(v)->clone() }`）或 `if s == "x"` 链；**勿用深嵌套 if/else（a2r 递归下降解析器会栈溢出）** |
| **a2r-3** | `str` 字面量实参被注入 `.to_string()`（E0308）| 调用形参为 `&str` 的函数时传字面量 | 系统性启发式 bug；库内影响有限（多见于测试代码）。规避：避免 `&str` 形参 + 字面量调用的组合，或调用方先 `let s = "x"` |
| **a2r-4** | 显式 `#[derive(...)]` 覆盖 a2r 自动全套 derive | 给 enum/struct 加任何 derive | 显式写出**全套**需要的 derive（`Clone, Debug, Serialize, Deserialize`），不可只写 serde（A23） |
| **a2r-4b** | 透传 derive 不自动注入对应 `use` | `#[derive(Serialize)]` | 显式 `use.rust serde::{Serialize, Deserialize}` |

> 与技能（auto-lang-creator）规则对应：a2r-1 未被 A 类 23 条覆盖（新发现）；
> a2r-2 ≈ A21；a2r-3 新发现；a2r-4 = A23。

### 阶段 0 结论

✅ **a2r 完全能转译 auto-musk「纯数据模型 + 工具方法」类模块为可编译 Rust。**

转译质量高：struct 字段、类型映射（String/Vec/Option/u64）、factory 构造、
`time::now_sec() as u64` 时间桥接、serde derive 全部正确。a2r 自动为 scalar enum
生成 `Display` + `from_id` + 默认 derive（无显式 derive 时）。

**对 auto-musk 的指导**：specs.rs 的数据模型层（约 ~300 行：两个 enum + 5 个
struct + 字符串转换 + factory）可作为阶段 2 的「首批可移植」目标。

---

## 阶段 1 — 测绘：逐文件标注 a2r 可移植性（待启动）

对 27 个 `.rs` 模块逐一标注三档可移植性，输出分层地图。

### 预判分层（待 PoC 推广后校验）

| 档位 | 判据 | 预判模块 |
|---|---|---|
| 🟢 纯逻辑可移植 | 无 axum/tokio/async/上游 trait | specs.rs 模型层 ✅(已验)、chats.rs 模型、auth.rs 密码逻辑、tool_safety.rs、hello.rs |
| 🟡 需逃生舱 | serde derive / 简单 IO / 上游 trait | mode.rs、app_config.rs（auto_atom）、tool_test.rs |
| 🔴 a2r 不支持 | axum async handler / tokio broadcast / async-trait / SSE | server.rs、main.rs、relay/{driver,api,store}.rs、orch_tools.rs、conversation.rs(broadcast)、tools.rs(async) |

### 阶段 0 实测对预判的修正

- specs.rs 整体（1495 行）含大量纯逻辑（状态机 SectionConfig、derive_statuses、
  rebuild_relations），**比初判更乐观**：状态机部分（match 密集）可能也可移植，
  但 rebuild_relations 用了 `regex::Regex` + `OnceLock` + `HashSet`，需单独验证。
- serde derive 经 a2r-4 规则可控 → 🟡 档中「serde derive」不再是阻塞因素。

---

## 阶段 2 — 分层移植实施（依阶段 1 地图，待定）

依测绘结果分批移植 🟢 模块（每模块一个 `.at` + 一次 a2r + 一次 cargo check 闭环），
🟡 用 `#[rs]` 逃生舱，🔴 模块按用户选定策略处理。

### 目录与产物约定

```
auto-musk/
├── backend/crates/musk/
│   ├── auto-src/            ← 新：手写 .at 源码（a2r 输入）
│   │   ├── specs.at         ✅（阶段 0 PoC，数据模型子集）
│   │   └── ...（按模块）
│   └── src/                 ← 现有手写 Rust（a2r 输出 .a2r.rs 与之并存，不覆盖）
└── docs/plans/014-auto-backend-port.md   ← 本文件
```

### 验证基线（每个移植模块须过）

1. `A2R_CRATE_ROOT=0 auto.exe trans --path <m>.at rust` → 0 转译错误
2. 临时 crate + a2r-std/auto-atom/auto-val（+ serde）path 依赖 → cargo check 0 错误
3. 对照原 `.rs`：公开 API 签名一致（允许 `&'static str`→`String` 等无害差异）

---

## 下一步决策点

1. **specs.rs 全文件推进**？状态机 + rebuild_relations（regex）+ derive_statuses
   是 specs.rs 余下 ~1200 行的核心，需先验证 regex/OnceLock/HashSet 的 a2r 支持。
2. **🔴 模块策略**？server.rs（2206 行，最大）等 axum/tokio 模块是"保留手写 Rust"、
   "用 `#[rs]` 逃生舱"、还是"等 a2r-std 补服务器运行时后再做"？需用户拍板。
3. **🟡 模块**：mode.rs/app_config.rs 用 auto_atom 解析 .at —— 这恰好是 Auto 生态
   原生能力，可能比 Rust 版更自然，值得优先试。
