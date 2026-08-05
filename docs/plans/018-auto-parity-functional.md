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
| **a2r-11 可变借用遍历** | 🔶 部分（已闭环） | `for mut x in coll` → 仍报错 "'mut' is not supported"；但索引就地改已修好——`self.items[i].field = v` → 产物不再写 clone（a2r-11 完整版，见 §12）。剩余缺口：for 循环遍历集合时的可变借用迭代、方法参数 `&mut` 发射（a2r-11 基础切片漏了 ext 方法分支，C1 store 对齐时补齐） |
| **async trait impl** | ✅ 可用 | `spec Tool { fn execute() ~Result<str,E> }` → 正确生成 `#[async_trait] async fn`（Plan 380 P5 成果） |
| **async 泛型闭包 `F: Fn->Fut`** | ❌ 硬墙 | `where F Fn(Req) -> Fut` → 报错 "Expected end of statement, got Fn"；`async fn` 自由函数也不被识别。task_plan_engine::execute 必须留手写边界 |
| **pub enum** | ✅ 可用 | `pub enum Color` → 正确生成 `pub enum Color`（plan 014 遗漏未加 pub） |

**结论**：
- a2r-11 已大幅推进：索引元素就地改 + 方法 `&mut` 参数已落地（§12）；剩余 `for mut x` 遍历缺口仍有变通（构建新集合 + 整体重赋值）。
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
| C7 | enum 数据载荷（NeedsApproval(String)） | 小（tool_safety） | ✅ 完成（随 C7b） | — | tag 定义 + 构造均可用 |
| C7b | tag union 命名字段构造丢值 | 中（所有带载荷 enum） | ✅ 完成（2026-08-04） | auto-lang rust.rs:5818 | 加 Arg::Pair 处理；tool_safety 恢复 tag + 去掉 classify_reason 变通 |
| C8 | `const` 关键字不支持 | 中（mode DEFAULT/BUILTIN_MODES） | ✅ 完成（2026-08-04） | auto-lang `e01f0f84` | mode.at 用 `pub const DEFAULT str = "superpowers"` 替代 `static fn default_name()` 变通 |

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
| 3.1 | task_plan_parser.rs (137行) | **已改判 ~5%（边界）** | auto_atom/auto_val 上游 crate API |
| 3.2 | task_plan_registry.rs (306行) | **已改判 ~30%（边界）** | include_str! + auto_atom + 文件 IO |
| 3.3 | task_plan.rs (513行) | ~80% | a2r-10（detect_cycle） |
| 3.4 | wiki.rs (847行) | **✅ 数据层+读路径已闭环（试点）** | axum/async 手写边界 |
| 3.5 | handoff_store.rs (193行) | ~60% | a2r-10/11（Mutex tuple-key） |
| 3.6 | task_plan_engine.rs (672行) | ~40% | **async 泛型闭包（硬墙）** |

task_plan_engine 的 execute/run_one（async 泛型闭包 `F: Fn->Fut`）**确认是 a2r 硬墙**（§1 实测），留手写边界。

**Phase 3 实况修订（2026-08-04）**：探索 agent 实测后发现计划中 3.1/3.2 的可移植率估算是错的 ——
task_plan_parser 是 `auto_atom::AtomParser`/`auto_val::Node` 上游 crate API 的薄封装，registry 依赖
`include_str!` + `parse_task_plan` + 文件 IO，二者都是**手写边界**（a2r 无法 import 上游 crate 类型）。
试点改为 **wiki 数据层 + 纯 Mutex CRUD + fs + serde_json**（与 Phase 2 同构），闭环内容见 §10。

---

## 7. Phase 4 — 复杂模块（server/orch_tools/tools/spec_tools）

- server：52 handler（Plan 380 P1-dyn + async_stream 后可移植，需重新评估）
- orch_tools：补 spawn_task_plan/register_task_plan（8月4日新增）
- tools/spec_tools：9+5 个 `impl Tool`（async trait，~Result 模式可用）

---

## 10. Phase 3 试点 — wiki 数据层 ✅ 闭环（2026-08-04）

### 移植范围
`auto-src/wiki.at`（数据模型 + WikiStore 读路径 + tree builders），对齐 Phase 2 模式：
- **数据模型**：WikiSource / WikiPage / WikiPageMeta / WikiManifest / TreeNode
- **WikiStore 读路径**：new / load / list_pages / get_page / search（5 个纯 Mutex CRUD + fs + serde_json）
- **tree builders**：parse_manifest / walk_md_files / find_meta / comes_first / insert_sorted / build_tree / strip_md_extensions

### 手写边界（未移植，文档化）
- **写路径**（create_page / update_page / delete_page / save_manifest）：因 a2r **parser 状态 bug** 推迟
  —— 某些 ext-block 方法组合在 ext 闭合处报 "Expected term, got RBrace" 或 "field type mismatch"
  （E0106 无行号）。之前把写路径方法移出 ext 块后 tmp 副本可转译，但**随后用同一二进制重试时字节
  相同的文件也失败**（旧观察过期），最终确认是 `types_are_compatible` 容器类型缺陷（见 C9）+ 组合
  触发。写路径留作后续闭环。
- **axum 路由/handler**（wiki_routes / wiki_tree / Multipart）+ API DTO + guess_mime：async 手写边界。
- **TreeNode file 节点 metadata**（size/modified）：ag 版设为 `None`（见下"已知简化"）。

### C9 闭环 — a2r `types_are_compatible` 容器类型缺陷（auto-lang）
`check_field_type` 的 `types_are_compatible` **没有容器类型分支**：
1. `GenericInstance` 无匹配分支 → 结构相同（Display 相同的 `Option<List<TreeNode>>`）被判不兼容；
   触发 `strip_md_extensions` 的 `children: children`（变量显式注解与字段同为 GenericInstance）。
2. `Option<T>`（.at 写法 → GenericInstance）与 `Some(x)`/`?T`（推断 → Type::Option）是别名但无等价
   规则；触发 `build_tree` 的 `children: Some(children)`。

修复（`crates/auto-lang/src/infer/mod.rs`）：新增 GenericInstance↔GenericInstance 逐元素比较 +
GenericInstance(Option/Result/List) ↔ Type::Option/Result/List 别名等价 + List/Map/Slice/Reference/
Option/Result/Tuple 逐元素 + 缺失原语自等（Byte/USize/U64/I64）。

### C9 连带 codegen 修复（`crates/auto-lang/src/trans/rust.rs`，均为新路径首踩）
1. **`expr_map_value_is_string` 无法穿透容器/包装**：`HashMap<str, WikiPage>` 解析为 GenericInstance、
   `self.pages.lock().unwrap()` 是 MutexGuard 包装 —— 回退 `true` 给结构体 insert 值误加 `.to_string()`
   （`page.to_string()` → E0308 非 Display）。新增 `map_value_ty` 穿透 Map/HashMap/BTreeMap 泛型与
   Mutex/RwLock/Arc 等包装；未知回退改为 `false`（String 值本就无需 `.to_string()`）。
2. **`needs_as_str` 对 `join`/PathBuf 误加 `.as_str()`**：`fs.read_to_string(self.wiki_dir.join(...))`
   产出 `join(...).as_str()`（PathBuf 无 as_str，E0599）。新增 `join` 方法守卫 + PathBuf/Path 类型本地
   变量守卫（`let page_path PathBuf = ...` 需显式注解）。
3. **`arg_is_str_slice` 用 `local_var_types` 判定**：该表把所有 str 变量（含本地 String）记为 StrSlice，
   导致同模块 fn 调用传本地 str 变量到 &str 参数时漏加 `.as_str()`（E0308 String vs &str）。改为按
   `current_fn_str_params`（真正的 &str 参数）判定 —— 与 `is_str_slice_var` 一致。

### 已知简化（文档化）
- **TreeNode file 节点 size/modified = None**：手写版从 `fs::Metadata` 丰富，但 a2r 的 Auto int 模型把
  `.len()` 无条件转 `as i32`（无法喂 `Option<u64>`），修 codegen 需触 Auto int 模型深层，留作后续。
  tree 结构/排序/strip 是功能性面，parity 测试已断言；metadata 差异在 parity_wiki.rs 注释记录。
- **load() 用 List + find_meta 线性查找**替代 HashMap.get()：a2r 的 `.get()` 借用规则对 `&str` 键
  不可靠（Call 参数不加 `&`；Ident 又可能多加 `&&`）。manifest 小，线性可接受。
- **WikiSource 不 derive(Default)**：a2r 丢弃 `#[default]` 且 enum 上 derive(Default) 必须带它
  （E0665）。load() 显式 `WikiSource.Custom` 兜底，行为一致；wire 格式（snake_case）完全对齐。

### 验收
`tests/parity_wiki.rs` 11/11 通过：WikiSource wire + 数据模型 wire + WikiStore 读路径
（load/list_pages/get_page/search + _manifest 优先）+ tree builders（walk/build/strip + 排序）。


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
| 0 — a2r 改进 | 🔶 C1 ✅ + C4/C5 ✅(无需改) + C6 推迟 + C7b ✅ + C8 ✅ + **C9 ✅** | C1 for 借用遍历(`e2c94535`)；C4 serde 属性是 .at 遗漏；C7b tag 构造丢值(`94418cda`)；C8 const 支持(`e01f0f84`)；C9 types_are_compatible 容器类型 + 连带 codegen 修复（见 §10） |
| 2 — 已移植模块 | ✅ 8/8 有 parity 测试 | specs 7 ✅ / app_config 7 ✅ / chats 9 ✅ / auth 8 ✅ / tool_safety 7 ✅ / conversation 10 ✅ / mode 4 ✅ / **wiki 11 ✅** |
| 3 — 缺失模块 | 🔶 wiki 试点 ✅（§10） | parser/registry 改判为边界（探索实测）；task_plan 3.3 待续 |
| 4 — 复杂模块 | ⬜ 待启动 | 视 Phase 0 成果 |

### Phase 2 各模块详情

| 模块 | 状态 | 完成内容 | 手写边界（依赖外部 crate/a2r 限制） |
|---|---|---|---|
| specs | ✅ | enum pub + display_title + 去 15 处 clone；7/7 parity 测试 | — |
| app_config | ✅ | 10 处 serde default + Default derive；7/7 parity 测试 | load/parse_from_at/to_at_source/apply_to_env（auto_atom/auto_val/env） |
| chats | ✅ | ToolCall status/id + 7 处 serde 属性；9/9 parity 测试 | new_id（rand）；ChatStore 11 方法（文件 IO） |
| auth | ✅ | Hash derive + sessions 字段 + 4 个 Mutex 方法 + 修正 derive；8/8 parity 测试 | hash_password/random_hex（sha2/hex/rand） |
| mode | ✅ | **C8 闭环**：`static fn default_name()` 变通 → `pub const DEFAULT str`（转译 `&str`，对齐手写 `&'static str`）；4/4 parity 测试 | BUILTIN_MODES（include_str!）；load/parse_mode_at（auto_atom/dirs） |
| tool_safety | ✅ | C7b 载荷恢复 + classify_reason 去变通；**理由文案对齐手写版**（⚠️ emoji/引号/完整句）；7/7 parity 测试 | 8 路径围栏（OnceLock/thread_local） |
| conversation | ✅ | ConversationEvent 结构对齐 + serde 属性补齐 + chat_message_to_turns 主 turn 条件对齐 + pub；10/10 parity 测试 | ConversationStore（Mutex+broadcast+jsonl）；run_event_to_turns（上游 RunEvent）；now_secs 的 UNIX_EPOCH（a2r 无法表达，转译版返回≈0，已文档化） |

### 2026-08-04 parity 测试补充记录

- **新增 5 个 parity 测试文件**（`tests/parity_auth.rs` 8 + `parity_tool_safety.rs` 7 + `parity_app_config.rs` 7 + `parity_chats.rs` 9 + `parity_conversation.rs` 10 = 41 测试）。至此 app_config/chats/auth/tool_safety/conversation 达到 §0 三层标准的第 2 层（单测行为等价）——此前仅完成第 1 层（API 对齐）。
- **conversation.at 三处对齐**（C4 类，均无需改 a2r）：
  1. `ConversationEvent` 重构为 `{ conversation_id, turn: Option<Turn>, status: Option<String> }`（原为过期结构 `{ kind, conversation_id, turn_id }`）+ 去掉 Serialize/Deserialize derive（对齐手写版仅 Clone, Debug）。
  2. 补 26 处 serde 属性（`default`/`skip_serializing_if`/`rename`），对齐手写版 Conversation/Turn/ToolRecord/GateRecord/ConversationSummary 的线格式。
  3. `chat_message_to_turns` 主 turn 条件对齐为 `!content.is_empty() || tool_calls.is_empty()`——Auto 无 `||`/`or` 关键字，改用两个嵌套 if 累积布尔量（实测 a2r 可转译），并加 `pub`。
- **C8 闭环（2026-08-04）**：a2r 新增 const 关键字支持（`e01f0f84`）——ext 关联 const + 顶层 pub const，`str` 类型 const → `&str`。mode.at 的 `static fn default_name()` 变通改为 `pub const DEFAULT str = "superpowers"`（转译产物 `pub const DEFAULT: &str`，与手写版 `&'static str` 语义等价），4/4 parity 测试通过。
- **顺带修复两个既有回归（auto-lang `cf0b2e25`，均非 C8 引入）**：
  - 大 .at 文件栈溢出：specs.at(~1100 行) 在 Windows 1MB 主线程栈上溢出（f288f80d 起可复现）。build.rs 默认栈 4MB → 64MB（虚拟预留）。
  - `rust_return_type_name` 的 impl 前缀启发式（Plan 380 P2）误伤本地 struct：`-> Foo` 错误产出 `-> impl Foo`、`Option<Foo>` 错误产出 `Option<impl Foo>`（E0404 不可编译）。修复为本地声明类型不加 impl；真实 trait（如 axum IntoResponse）不受影响。10 个 golden 重生成。
- **新发现（已记录，非本次闭环）**：
  - `app_config` effective_daemon_url 的 `AAID_URL` env 覆盖在 a2r 产物中缺失——实测 a2r 无法表达 `env::var(...).ok()`（`Expected Asn, but found .`），确认是 B 类手写边界而非 .at 遗漏。已在 `parity_app_config.rs::parity_effective_daemon_url`（含文档化分歧断言；2026-08-04 合并原 `documented_divergence_env_override_skipped_in_ag` 以消除并行 env 竞态 flake）固定当前行为。
  - `auto_generated::chats` 的 `SpecChange` 是自包含镜像，其 `SpecStatus` 仅含 Empty/Draft 两变体（真实版 23 变体）。parity 测试以 `status: None` 规避该收窄。
  - `conversation` 转译版 `now_secs` 用 `SystemTime::now().elapsed()`（≈0），手写版用 `duration_since(UNIX_EPOCH)`。UNIX_EPOCH 常量 a2r 无法表达，转译版该函数为私有，暂无实际影响——文档化待 C8 后评估。
- **Phase 3 wiki 试点闭环（2026-08-04，详见 §10）**：
  - 探索实测改判：task_plan_parser/registry 是上游 crate API 边界（auto_atom/auto_val/include_str!），
    可移植率估算大幅修正；试点改为 wiki 数据层 + 读路径 + tree builders。
  - `auto-src/wiki.at` 新增 + `tests/parity_wiki.rs` 11/11 通过（wire 格式 + WikiStore 读路径 +
    tree 结构/排序/strip）。
  - **C9 闭环（auto-lang）**：`types_are_compatible` 缺容器类型分支（GenericInstance 判不兼容 +
    `Option<T>`/`Some(x)` 别名不等价）；连带修 3 处 codegen（insert 值 `.to_string()` 误加 / `join`+PathBuf
    `.as_str()` 误加 / 本地 str 变量漏加 `.as_str()`）。
  - 文档化简化：TreeNode file 节点 metadata 省略（`.len() as i32` Auto int 模型限制）；load() 用
    List+find_meta 线性查找（HashMap.get() 借用不可靠）；WikiSource 不 derive(Default)（E0665）。
  - 写路径（create_page/update_page/delete_page/save_manifest）因 a2r parser 状态 bug 推迟，留作后续闭环。
  - 顺带：重生 3 个 impl-prefix 遗留陈旧 golden（`rand_custom`×2 + `log_custom`，上一轮修复漏网）。

---

## 11. 接线运行（2026-08-05 启动）— 让 Auto 版真正跑起来

> 计划 018 §0 声明"接线运行作为后续独立计划"。本阶段把它落地。
> **前置评估（2026-08-05 实测）**：`auto_generated/` 26 个模块全量编译、但**运行时零引用**
> （`src/` 非生成代码无任何 `auto_generated::*` 调用；`src/main.rs:161` 跑手写 `server::serve`）。
> extern_impl 150 个辅助函数中 ~100 个是 fake stub（`auth_login_role`→"admin"、`specs_load`→`Null`），
> 若直接接入会返回假数据。**架构事实**：ag server handler 引用的是**手写 AppState/store**，
> extern_impl 就是设计好的桥 —— 接线正解是"extern_impl 真实委托 + ag router 接入"，而非换 store
> 类型（ag/hw store 是类型不相同的镜像，换 store 会引发 DTO 类型级联，代价大且非架构本意）。

### 路线图（4 步，每步可独立验收）

| # | 内容 | 验收 |
|---|---|---|
| ① | **auth 接线**：extern_impl 的 auth 7 个 stub 换真实委托（走 `s.auth`）；ag router 的 `/api/auth/*` 3 路由接入 serve() | `musk serve` 后 login/me/logout 由转译 handler 服务，行为与手写版一致（curl 验证） |
| ② | **specs/wiki/chats 数据层接线**：extern_impl 对应 stub 换真实委托（走 `s.registry` 的 workspace stores）；ag router 的 specs/workspace/chats 路由接入 | 各端点真实 CRUD 返回，与手写版一致 |
| ③ | **extern_impl 剩余 stub 全部真实委托**（config/modes/skills/roles/app-config/harness/conversations/relay/drive/agent/ctx） | 全部端点有真实行为；fake 常量清零 |
| ④ | **auto_generated::server 整体接入**：ag build_router（36 路由）作为主 router；7 个 🔴 流式/daemon handler + wiki + 静态文件路由与手写 router 合并 | 全服务端由转译 handler 驱动；原有 45 路由功能不丢 |

### 手写边界（接线阶段保持手写）
- **数据 store**：auth/specs/chats/wiki 等 store 本体保持手写（ag/hw store 类型不相同的镜像；
  换 store 需 a2r 支持类型同一性 —— 后续独立 dogfooding 目标）。
- **7 个 🔴 handler**（run/run_stream/chat_stream/conversation_stream/workflow_run/
  workflow_run_stream/settings_link）：async 泛型闭包/reqwest/SSE 硬墙。
- **serve() 外壳**（静态文件/CORS/TcpListener/axum::serve）。

### 已完成的接线（滚动更新）
- 2026-08-05：本路线图写入计划；开始 ①。

### ②③④ 实测阻塞(2026-08-05,已停止推进,待决策)
- **② workspace stores 级联**:specs/chats/wiki 在 `WorkspaceStores`(`src/workspace.rs:38`),
  被 6 模块 41 处引用(chats→specs、spec_tools→specs、conversation→chats)。swap store 类型
  会波及整个模块图,不是 auth 那样的局限改动。
- **② ag store API 与手写版不一致(parity 缺口)**:`ag::SpecsStore::load()` 返回 `SpecsDocument`
  (无 Result)、`save(&self, doc)` 按值 —— 手写版返回 `std::io::Result<SpecsDocument>`、
  `save(&self, &doc)`。手写 specs handler `match ws.specs.load() { Ok/Err }` 依赖 Result。
  parity_specs 7 个测试只覆盖数据模型/状态机,**未覆盖 store IO 签名**。
- **③④ a2r codegen drift(s vs s.view)**:当前 a2r 重转译 server.at 产出 `auth_login_role(s, ...)`
  (无 `&`),已提交产物是 `&s`;新输出与 extern_impl `&T` 签名不编译。根因:plan 014 文档化的
  `.view` 借用标记约定(a2r-13)未被 server.at 遵守 —— server.at 有 0 处 `.view`、32 处裸 `s,`
  extern 调用。旧 a2r 会自动加 `&`,当前 a2r 遵循约定不再自动加。
  **修法**:server.at 的 extern 调用补 `s.view`(~90 处)+ 修 LoginResponse DTO + extern_impl 委托
  + router 合并 —— 大手术,且影响当前可用 server。
- **结论**:① 是唯一已闭环的接线切片。②③④ 需先决策:
  要么"先对齐 ag store API"(② 的大工作),要么"server.at .view 手术 + DTO parity + extern_impl
  委托 + router 合并"(③④ 的大工作)。两者都是多 session 级别,建议作为下一个独立计划里程碑。

### B 阶段完成状态(2026-08-05)
- **specs ✅ 对齐闭环**:ag SpecsStore load/save/drift_check → Result 签名 + project_name 修复
  (hw 用文件 stem 兜底,ag 硬编码 "project" —— 真 parity 缺口,已修)+ parity_specs 2 个
  store IO 测试(往返/drift_check/corrupt→Err)。
- **chats ⬜**:ag ChatStore 缺 11 个 CRUD 方法(create/list/get/append_message 手写),
  "对齐"= 先移植方法,非签名微调 —— 单独工作项。
- **wiki 🔶**:读路径已对齐;写路径(create/update/delete/save_manifest)因 a2r parser
  bug 推迟(见 §10)。
- **a2r 类型系统边界(记录)**:`Result<(), str>`(unit 类型)不可表达 → save 用
  Result<str,str> 形状;io::Error 不可表达 → load 错误为 String;io::ErrorKind 不可
  区分 → load 兜底对所有读错误生效(与 hw NotFound 语义一致,其它为残差);
  `&mut doc` 就地改(a2r-11) → upsert/transition/delete 已对齐 `&mut doc`
  (见 §12 C1 store 写方法对齐;残差仅 unit 载荷 → bool)。

### C — 接线运行新 Phase(2026-08-05 起,置于 A/B 之后)
C 不再作为独立计划,并入本计划作为新 Phase。内容即本 §11 路线图的 ②③④:
C1 数据层接线(specs/wiki/chats 读路径 swap,chats 需先移植 CRUD)
C2 extern_impl stub → 真实委托(auth 7 个已可做,基于 A 的 .view 修复)
C3 ag server build_router 接入(main.rs,含 DTO parity 修复)

### C 阶段完成状态(2026-08-05)
- **C2-auth ✅(首个垂直切片)**:extern_impl auth 7 个 stub → 真实委托(走 AppState.auth,
  即 ① 的 ag AuthStore);修 ag auth_login 双重登录缺陷(split 版两次 login 产生两个
  session → role 查 session#1、token 是 session#2 失配 → 合并为 auth_login_result 返回
  (token, role));LoginResponse DTO → {token, user: UserInfo}(对齐手写 wire);
  auth_me → 返回 UserInfo 形状(对齐 hw);三个 handler pub。
- **测试**:`ag_auth_handlers_produce_real_behavior` —— 转译 handler 栈经真实委托产生
  真实行为(login 真实 token → me 真实用户 → logout 失效)。
- **C3 边界(文档化)**:ag server **无 HTTP 状态码模型**(设计:错误用 DTO、状态码在外壳层
  处理)。ag handler 无法表达 401/500 → 生产接线 auth_login 的 401 会回归为 200+空数据。
  行为保真的生产接线(C3)待 a2r/server.at 支持 handler 错误状态码 —— 新的 dogfooding 目标。
- **C1 现状**:auth 已接线(①);specs/wiki/chats 受 workspace stores 41 处级联 + a2r-11
  写方法 + chats 缺 11 个 CRUD 方法阻塞(见 §11 ②③④ 实测阻塞)。

---

## 12. Phase C 阻塞修复（2026-08-05 起）

> 目标:清除 C1/C3 的阻塞,让"Auto 版跑起来"继续推进。

### C3 — ag server 无 HTTP 状态码模型 ⛔→🔧
- **现象**:ag handler 一律 `~Json<T>`,无法表达 401/500(设计注释:"错误也用 DTO,
  状态码在外壳层处理",但外壳层从未实现)。
- **修复方案(实测可行,auto-musk 单侧,无需 a2r)**:
  1. extern_impl 加 `ok_response<T: Serialize>(v) -> Response` +
     `err_response(msg: &str, code: u16) -> Response`(构建 `(StatusCode, Json(ApiError))`)。
  2. server.at handler 返回 `~Response`,成功 `ok_response(LoginResponse(...))`,
     失败 `err_response("invalid credentials", 401u)`。a2r 已验证可表达 `~Response`。
  3. 修完 auth_login/auth_me(401 语义保真)后,把 ag auth 路由接入 serve()
     (替换手写 auth handler)→ 行为保真的生产接线成立。

### C1 — 数据层 swap 三阻塞
- **a2r-11 就地修改 ⛔→🔧(auto-lang worktree)**:
  ag store 写方法(upsert/transition/delete)被迫函数式返回新 doc,hw 用 `&mut doc`
  就地改。修复:a2r 支持 `&mut` 参数 + 集合元素就地修改(重大特性,worktree 开发)。
- **chats 缺 11 个 CRUD 方法 ⛔→🔧(auto-musk)**:
  chats.at 移植 ChatStore 的 create/list/get/append_message/summary 等(仿 specs.at)。
- **workspace stores 41 处级联 🔶(重新评估)**:
  C2 extern_impl 委托路径若成立,store swap 可能非必需 —— 级联作为最后手段,
  优先走"ag handler 委托 + 状态码模型"的完整接线。

### C3 完成状态(2026-08-05)✅
- extern_impl `ok_response`/`err_response` helper 落地(auth 401 保真)。
- server.at auth_login/auth_me 返 `~Response`,失败 401。
- serve() 接入 ag auth 路由,**生产 auth 端点完全转译**(handler+DTO+store),
  手写 auth handlers 删除。
- 测试:真实 token + wire 形状 + bad-credentials→401 + logout→401。
- **C3 主要阻塞已清除**(auth 切片);其余端点(specs/workspace/chats)同模式扩展。

### C1 a2r-11 进度(2026-08-05, worktree `c11-inplace-mut`)
- ✅ **基础切片**(worktree commit `19123312`):`mut p T` 参数 → `&mut T`;
  调用点传 `&mut arg`(不再 `.clone()`);简单字段就地赋值直接发射。
  golden 301 通过 0 失败。
- ⬜ **剩余**:索引元素就地改(`doc.items[i].field = v` —— `.get()` 转换在赋值
  LHS 误加 `.clone()`,深 codegen 路径)+ `*doc = new_doc` 整体重赋值 —— 后续
  切片的 dogfooding 目标。
- worktree 分支 `c11-inplace-mut`,待完整后合并 master。

### C1 a2r-11 完整(2026-08-05, 已合并 master `9e9e0748`)✅
- **五项能力全落地**(worktree `c11-inplace-mut` → merge):
  1. `mut p T` 参数 → `p: &mut T`
  2. 调用点传 `&mut arg`
  3. 简单字段就地改 `doc.field = v`
  4. **索引元素就地改** `doc.items[i].field = v`(assign_lhs_depth 标记跳过 LHS `.clone()`;
     顺带修复 3 个 matrix golden 的 no-op bug —— 旧产物写 clone)
  5. **整体重赋值** `*doc = new_doc`(deref-assign)
- golden 301 通过 0 失败。
- **下一步(C1 继续)**:用 a2r-11 对齐 ag store 写方法签名
  (`upsert_item(mut doc Doc, ...)` → `&mut doc`),解除 store API 分歧。

### C1 store 写方法对齐(2026-08-05, 已合并 master `46337958`)✅
- **specs.at 写方法 `&mut doc` 化**(worktree `c1-store-mut-param` → merge):
  upsert_item/transition_item/delete_item 从"by-value doc + Option<SpecsDocument>
  返回新 doc"改为 `mut doc SpecsDocument`(`&mut SpecsDocument`)就地改,
  签名与 hw 同构:
  - hw: `upsert(&self, doc: &mut Doc, ...) -> Result<(), String>`
  - ag: `upsert(&self, mut doc: &mut Doc, ...) -> Result<bool, String>`
  - delete 两边都是 `Result<bool, String>`(removed 语义完全同构)。
  - **残差**:a2r 无法表达 unit 类型(`Result<(), str>` unit parse error,且
    `Ok(())` 无法解析/`Ok(null)` 会卡死 transpiler)—— upsert/transition 用
    bool 载荷承载成功语义,文档化。
- **顺带发现并修复 2 个 auto-lang 缺口**:
  1. **方法参数 `&mut` 发射缺失**:a2r-11 基础切片只在"自由函数/static"分支补了
     `ParamMode::Mut → &mut T`;**ext-block 方法参数分支漏了**(发射 `mut p: T`
     by-value)。→ 方法分支补齐,新增回归测试 `test_a2r_method_mut_param_emits_mut_ref`。
  2. **fix_residual_error_box 嵌套括号 Bug**:旧 regex 只匹配一层括号,字符串
     concat 生成的嵌套 `format!(...)` 残留在 `Err(Box::new(...))` 里,破坏
     `Result<_, String>`(E0308 Box<String> vs String)。→ 改为平衡括号扫描,
     新增回归测试 `test_a2r_err_concat_no_box_residual`。
- 就地修改用**索引 LHS 赋值**(`doc.sections[idx].items[idx] = x` 跳过 LHS
  `.clone()`,写真实元素);append 走"克隆列表 + 整体字段重赋值"(索引上的
  `.push()` 会落在 `.clone()` 临时值,no-op)。
- parity:新增 3 个写方法对齐测试(upsert 替换不重复 / transition 校验+Done
  设置 completed_at / delete true→false),错误字符串与 hw 逐字一致
  (`section 'x' not found` / `item 'x' not found in 'y'`)。
- a2r golden 303 通过 0 失败;musk 全套测试通过(199 lib + 各 parity 套件)。
