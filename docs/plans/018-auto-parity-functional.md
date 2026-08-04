# 018 — Auto 化功能一致性计划（Functional Parity）

> **状态**：实施计划。Phase 1（specs 模块）✅ 完成（2026-08-04，7/7 parity 测试通过）。
> **前置**：Plan 014（Auto 后端移植，已归档）+ Plan 015（合并编译清零，已归档）。
> **基线**：v0.3.0（2026-08-04）。
> **仓库**：auto-musk（`backend/crates/musk/`）+ auto-lang（a2r 转译器）。
> **目标**：让 Auto(.at) 版本经 a2r 转译产出的 Rust，在**公共 API 签名**和**运行行为**上与手写 Rust 一致（单测等价），全模块覆盖。接线运行作为后续独立计划。
> **战略定位**：本计划是 **Auto 语言的 dogfooding 工程**——在真实的 Auto/Rust 项目实践中发现 a2r 的不足，逐项改进转译器，推动 Auto 真正成为 Rust 生态的开发语言。a2r 限制不是"既定约束"，而是**要消灭的对象**。

---

## 0. 背景与原则

Plan 014/015 完成了"全后端移植到 Auto + 合并编译 0 错误"，但产物一直作为"可编译镜像"存在（`auto_generated/` 零引用）。本计划推进到**功能等价**：用 parity 测试证明 a2r 产物的行为与手写 Rust 一致。

**功能一致性定义**（用户确认）：
- Auto 生成的 Rust 代码与 Rust 原生代码的**行为一致**（运行结果一致）。
- 公共 API（pub fn/struct/enum）签名一致（允许实现细节不同，如 HashMap vs 线性查找）。
- **不是**逐行级别一致（那是更远期的目标，需 a2r 转译器大幅改进）。

**验证三层标准**（用户确认，逐模块达标须同时满足）：
1. **API 签名一致**：pub struct/enum/fn 与手写版逐一对照。
2. **单测行为等价**：该模块的手写单测在 a2r 产物上全通过（parity 测试）。
3. **覆盖率 ≥80%**：Mutex/async/tokio 部分允许标记"手写边界"。

**策略**（用户确认）：分两步——先"等价镜像"（本计划），后"接线运行"（后续计划）。双管齐下：同时改 a2r 转译器与 .at 源码。

### ⭐ 核心工作范式：逐项闭环（dogfooding 驱动 a2r 进化）

a2r 转译器的改进不是次要任务，而是本计划的**核心驱动力**。每遇到一个 a2r 限制，执行闭环：

```
① 在 auto-musk 发现 a2r 转译失败/产物不正确
    ↓
② 提取最小复现样例（.at → 期望的 Rust）
    ↓
③ 切到 auto-lang 仓库，定位 a2r 源码缺陷，修复
    ↓
④ 在 auto-lang 验证样例通过 + 回归测试
    ↓
⑤ 回到 auto-musk，去掉变通写法，用标准 Rust 写法的 .at
    ↓
⑥ 重新转译 + 跑 parity 测试，确认产物行为一致
```

每个闭环 = 一个 auto-lang PR（a2r 改进）+ 一个 auto-musk commit（去掉变通 + 更新产物）。
**同 session 跨仓库推进**（用户确认）。

---

## 1. a2r 实测发现（2026-08-04 探针验证）

用最小样例实测当前 a2r（auto.exe @ 2026-08-04）的能力边界：

| 限制 | 状态 | 实测结果 |
|---|---|---|
| **a2r-11 可变借用遍历** | ⚠️ 仍存在 | `for mut x in coll` → 报错 "'mut' is not supported"；索引遍历 `self.items[i].field = v` → 产物 `.clone().field = v`（写进 clone，无效）。**变通有效**：构建新集合 + `self.field = new_list` 整体重赋值 |
| **async trait impl** | ✅ 可用 | `spec Tool { fn execute() ~Result<str,E> }` → 正确生成 `#[async_trait] async fn`（Plan 380 P5 成果） |
| **async 泛型闭包 `F: Fn->Fut`** | ❌ 硬墙 | `where F Fn(Req) -> Fut` → 报错 "Expected end of statement, got Fn"；`async fn` 自由函数也不被识别。task_plan_engine::execute 必须留手写边界 |
| **pub enum** | ✅ 可用 | `pub enum Color` → 正确生成 `pub enum Color`（plan 014 遗漏未加 pub） |

**结论**：
- a2r-11 变通写法（plan 014 确立）仍是最优解，无需立即改 a2r。
- async 泛型闭包是无法绕开的硬墙——含此模式的代码（task_plan_engine::execute/run_one）留手写。
- 很多"API 差距"实为 .at 源码遗漏（如 enum 缺 pub、display_title 缺 emoji），修 .at 即可，无需动 a2r。

---

## 2. 阶段总览

```
Phase 0  a2r 转译器改进（auto-lang，治本，按需）
 │
 ├─→ Phase 1  specs 模块打磨 + parity 框架 ✅ 完成
 │
 ├─→ Phase 2  已移植模块对齐（auth/mode/tool_safety/app_config/chats/conversation）
 │
 ├─→ Phase 3  补齐缺失模块（task_plan 系列 + wiki）
 │
 └─→ Phase 4  复杂模块（server/orch_tools/tools/spec_tools）
```

---

## 3. Phase 1 — specs 模块 ✅ 完成（2026-08-04）

**成果**：specs 模块 7/7 parity 测试通过，证明 a2r 产物行为与手写 Rust 完全一致。

### 实施内容

1. **enum pub 可见性**：`specs.at` 的 SectionType/SpecStatus 加 `pub`（plan 014 遗漏）→ 重新转译 + nativeize → 更新 `auto_generated/specs.rs`。
2. **display_title emoji**：`specs.at` 的 display_title 补 emoji（🎯 Goals 等，plan 014 写成了无 emoji 版）。
3. **derive_statuses 诊断**：实证确认**无缺陷**——原"行为差异"是测试预期错误（Goal 须在 InProgress 才能升 Implemented，can_transition(Empty, Implemented) = false）。手写版同此行为。
4. **parity 测试框架**：`tests/parity_specs.rs`（7 测试），对照手写 `specs::X` 与 `auto_generated::specs::X`，覆盖：
   - SectionType 字符串转换（as_str/display_title/from_id）
   - Goals 状态机（合法/非法转换）
   - rebuild_relations 关系图反链
   - derive_statuses 状态推导（InProgress+Done→Implemented + Empty 不强制升级）

### 验收
- `cargo test --test parity_specs` → **7 passed; 0 failed**
- 原有 `cargo test --lib` → **189 passed; 0 failed**（未破坏）

### Parity 测试范式（复用到后续模块）
```rust
use musk::specs as hw;                    // hand-written
use musk::auto_generated::specs as ag;    // a2r-transpiled
// 分别构建 hw/ag 文档（类型不互通），比较行为结果（status.to_str() 等）
```

---

## 4. Phase 0 — a2r 转译器改进（auto-lang，⭐ 核心驱动）

> **战略定位**：本计划的根本目标之一是**通过 dogfooding 驱动 a2r 进化**。a2r 限制不是要绕开的既定约束，而是要消灭的对象。每消灭一个限制，Auto 就更接近"能替代 Rust 的开发语言"。
>
> **节奏**：逐项闭环（§0 核心工作范式）。同 session 跨仓库推进。

### 闭环追踪表

| ID | a2r 限制 | 影响面 | 闭环状态 | auto-lang PR | auto-musk 去变通 |
|---|---|---|---|---|---|
| C1 | a2r-11 可变借用遍历（`for x in coll` 不加 `&`） | 巨大（所有集合遍历场景） | ✅ 完成（2026-08-04） | `e2c94535` | specs.at 去掉 15 处 `.clone()` 变通；7/7 parity 通过 |
| C2 | async 泛型闭包 `F: Fn->Fut` | 大（task_plan_engine::execute） | ⬜ 待启动 | — | execute 留手写边界 |
| C3 | HashMap<K,Vec<V>> 方法调用（a2r-10） | 中（detect_cycle 等） | ⬜ 待启动 | — | task_plan 可用标准写法 |
| C4 | serde 属性透传（default/rename/skip/alias） | 大（全模块向后兼容） | ✅ 确认非 a2r 限制（2026-08-04） | 无需改 a2r | app_config.at 补 10 处 default + Default derive；chats.at 补 ToolCall status/id + 7 处 serde 属性 |
| C5 | enum Default derive + `#[default]` | 中（config 类） | ✅ 已验证可用 | 无需改 a2r | Default derive 透传正常（C4 中验证） |
| C6 | str 所有权推断（`impl Into<String>`） | 大（全模块构造函数） | ⏸️ 推迟到接线阶段 | 无需改 a2r | `&str` 签名行为等价；owned String 传入差异仅在接线运行时出现 |
| C7 | enum 数据载荷（NeedsApproval(String)） | 小（tool_safety） | ⬜ 待启动 | — | CommandTier |

### C1 闭环详情（a2r-11 for 循环借用遍历）✅

**问题**：`for x in self.sections` 生成 `for x in self.sections`（move 集合），迫使 .at 源码写 `.clone()` 变通。

**修复**（auto-lang `rust.rs:8715`）：Named 迭代对 `Ident`/`Dot`（变量/字段访问）自动加 `&`：
```rust
let is_borrowable = matches!(&for_stmt.range, Expr::Ident(_) | Expr::Dot(_, _));
if is_borrowable { sink.body.write(b"&")?; }
```
方法调用（`.clone()`、迭代器方法）不加 `&`（返回临时值/owned 迭代器）。

**去变通**（auto-musk `specs.at`）：15 处 `for x in coll.clone()` → `for x in coll`。

**验证**：
- auto-lang：3 样例（字段/变量加 `&`，Call 不加）+ 2779 passed 零新增回归
- auto-musk：specs 重转译 + 7/7 parity + 220 全量测试通过

> **注**：优先级按"影响面 × 去变通收益"排序。C1（a2r-11）是第一个闭环目标——它影响面最大，且 specs 模块已有变通代码可对照验证。

---

## 5. Phase 2 — 已移植模块对齐（6 个）

顺序：auth → mode → tool_safety → app_config → chats → conversation（由简到繁）。

每模块工作流（复用 Phase 1 范式）：
1. 对照 API 差距清单（详见 §8 附录），逐项修 .at
2. 重新转译 + nativeize + 更新 auto_generated
3. 建 `tests/parity_<module>.rs`，复制手写单测，跑通

### 各模块关键差距（来自探索，待逐项核实）

| 模块 | 主要修复 | B 类（手写边界） |
|---|---|---|
| auth | Hash derive；self by-value；visibility | 4 个 session 方法（Mutex） |
| mode | HashMap→Vec 存储；DEFAULT const | load/parse_mode_at/BUILTIN_MODES（auto_atom） |
| tool_safety | 恢复 NeedsApproval(String) 载荷 | 8 个路径围栏方法（OnceLock/thread_local） |
| app_config | Default derive + serde default；effective_daemon_url env | load/apply（auto_atom/env） |
| chats | ToolCall 补 status/id；usize→u32 | 11 个 ChatStore 方法（Mutex） |
| conversation | ConversationEvent 结构；usize→u32 | ConversationStore 13 方法（Mutex+broadcast） |

---

## 6. Phase 3 — 补齐缺失模块

顺序按可移植率（探索评估）：parser → registry → task_plan → wiki → handoff_store → engine。

| Task | 文件 | 可移植率 | 关键阻塞 |
|---|---|---|---|
| 3.1 | task_plan_parser.rs (137行) | ~95% | 几乎无 |
| 3.2 | task_plan_registry.rs (306行) | ~85% | 借用模式 + 文件 IO |
| 3.3 | task_plan.rs (513行) | ~80% | a2r-10（detect_cycle） |
| 3.4 | wiki.rs (847行) | 高（纯 CRUD） | 无 async/Mutex |
| 3.5 | handoff_store.rs (193行) | ~60% | a2r-10/11（Mutex tuple-key） |
| 3.6 | task_plan_engine.rs (672行) | ~40% | **async 泛型闭包（硬墙）** |

task_plan_engine 的 execute/run_one（async 泛型闭包 `F: Fn->Fut`）**确认是 a2r 硬墙**（§1 实测），留手写边界。

---

## 7. Phase 4 — 复杂模块（server/orch_tools/tools/spec_tools）

- server：52 handler（Plan 380 P1-dyn + async_stream 后可移植，需重新评估）
- orch_tools：补 spawn_task_plan/register_task_plan（8月4日新增）
- tools/spec_tools：9+5 个 `impl Tool`（async trait，~Result 模式可用）

---

## 8. 附录：API 差距清单（6 模块，探索汇总）

### 系统性差距模式（跨模块）
1. `impl Into<String>` 一律退化为 `&str`
2. `usize`→`u32` 系统性收窄（conversation/chats）
3. enum 可见性丢失（specs ✅ 已修）
4. serde 属性普遍丢失（`#[serde(default)]`/rename/skip/alias）
5. `Default` derive 丢失；`const`→`fn` 退化
6. 跨模块类型被重声明而非 `crate::` 引用
7. Mutex/broadcast/OnceLock 相关 API 整块缺失（B 类，手写边界）
8. enum 数据载荷丢失（CommandTier::NeedsApproval(String)）
9. 私有方法/字段被提升为 pub

### 模块逐项（详见探索 agent 输出，存于本计划历史）
- **auth**：AuthStore 缺 sessions 字段 + 4 方法（login/session_user/token_allows/logout）
- **chats**：ChatStore 缺 11 方法；ToolCall 缺 status/id 字段
- **conversation**：ConversationStore 整缺（13 方法）；ConversationEvent 结构不同
- **mode**：ModeRegistry 存储从 HashMap 变 Vec；DEFAULT const 变 fn
- **tool_safety**：CommandTier::NeedsApproval 丢载荷；8 个路径围栏方法缺
- **app_config**：缺 Default derive + serde default + load/apply 生命周期方法

---

## 9. 进度跟踪

| Phase | 状态 | 完成摘要 |
|---|---|---|
| 1 — specs | ✅ | 7/7 parity 测试通过；enum pub + display_title emoji 修复 |
| 0 — a2r 改进 | ⏸️ 按需 | 降级为遇阻时才做 |
| 2 — 6 模块对齐 | ⬜ 待启动 | auth 优先 |
| 3 — 缺失模块 | ⬜ 待启动 | parser 优先（试点） |
| 4 — 复杂模块 | ⬜ 待启动 | 视 Phase 0 成果 |
