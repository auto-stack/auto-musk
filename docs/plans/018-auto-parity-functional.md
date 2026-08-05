# 018 — Auto 化功能一致性计划（Functional Parity）

> **状态**：实施计划。Phase 1（specs 模块）✅ 完成（2026-08-04，7/7 parity 测试通过）。
> **前置**：Plan 014（Auto 后端移植，已归档）+ Plan 015（合并编译清零，已归档）。
> **基线**：v0.3.0（2026-08-04）。
> **仓库**：auto-musk（`backend/crates/musk/`）+ auto-lang（a2r 转译器）。
> **目标**：让 Auto(.at) 版本经 a2r 转译产出的 Rust，在**公共 API 签名**和**运行行为**上与手写 Rust 一致（单测等价），全模块覆盖。接线运行作为后续独立计划。
> **战略定位**：本计划是 **Auto 语言的 dogfooding 工程**——在真实的 Auto/Rust 项目实践中发现 a2r 的不足，逐项改进转译器，推动 Auto 真正成为 Rust 生态的开发语言。a2r 限制不是"既定约束"，而是**要消灭的对象**。
> **Phase 3 进度**：wiki 试点 ✅（§10）+ task_plan 3.3 ✅（§10.5）+ handoff_store 3.5 ✅（§10.6，2026-08-05）。

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
| **a2r-10 HashMap<K,Vec<V>>（C3）** | ✅ 已闭环（2026-08-05） | `HashMap<String, Vec<String>>` 的 `insert`/`get` 已正确发射（task_plan detect_cycle 实证）。剩余调用点细节：`.get()` 返回 `Option<&Vec>` 需 `is` 直接匹配不可标注类型；split 结果 a2r 强制标 `Vec<String>`（实为 `Vec<&str>`）→ 无标注 `let parts = ...` 变通 |
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
| C3 | HashMap<K,Vec<V>> 方法调用（a2r-10） | 中（detect_cycle 等） | ✅ 完成（2026-08-05，随 task_plan 3.3） | 无需改 a2r（insert/get 已可用） | task_plan.at detect_cycle 用 HashMap 标准写法；parity 17/17 通过 |
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
| 3.3 | task_plan.rs (513行) | **✅ 已闭环（2026-08-05）** | 阻塞已清除：a2r-10 HashMap<K,Vec<V>> insert/get 可用 + auto_val 上游类型可用 + a2r 修 #[default] 透传/InvalidType seed |
| 3.4 | wiki.rs (847行) | **✅ 数据层+读路径已闭环（试点）** | axum/async 手写边界 |
| 3.5 | handoff_store.rs (193行) | **✅ 已闭环（2026-08-05，7/7 parity）** | tuple-key 改判为字符串 key 变通（见 §10.6） |
| 3.6 | task_plan_engine.rs (672行) | ~40% | **async 泛型闭包（硬墙，2026-08-05 复测确认）** |

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


## 10.5 Phase 3.3 — task_plan.rs 移植 ✅ 闭环（2026-08-05）

### 移植范围
`auto-src/task_plan.at`，对齐 hw `relay/task_plan.rs`：
- **数据模型**：TaskPlan / Phase / RunRef / TaskMode / PhaseMode（serde wire 完全对齐）
- **builder**：new / add_phase / with_mode / depends_on / add_run / with_input /
  with_input_from / with_context / with_mode_override
- **validate / detect_cycle / dfs**：重复 phase/run 名、未知依赖、环（含自环）、
  handoff path 语法（split 三段 + handoff/output 第三段）
- **Atom 解析**：`impl TryFrom<Node>` → `static fn from_node`（a2r 无法表达 trait
  impl，行为等价）

### 阻塞清除实录（重要：计划的 a2r-10 阻塞已不存在）
计划 §6 预估 task_plan ~80% 可移植、阻塞为 a2r-10（detect_cycle HashMap）。实测：
- **a2r-10 HashMap<K,Vec<V>> 已可用**：`graph.insert(...)` / `is graph.get(...)` 正确
  发射（无需 specs.at 的 List 线性变通）。
- **auto_val/auto_atom 上游类型已可用**：`use.rust auto_val::{Kid, Node, Value}` /
  `use.rust auto_atom::{AtomError, AtomResult}` 均可转译（mode.at 注释里的旧结论
  "a2r 无法 import 上游 crate 类型" 已过时）。`node.kids_iter()` / `Kid.Node(child)`
  匹配 / `AtomError::ValidationError(..)` 构造 / `Result<_, AtomError>` 返回全部可行。

### 联动 a2r 修复（auto-lang，2 commit）
1. **C11 prescan fn_mut_params（`dfa7bd05`，并行 session 已并入）**：prescan 注册
   `mut p T` 参数标志，覆盖"函数声明在调用者之后"场景（task_plan 的 dfs 在文件底部）。
   修前调用点发 `visited.clone()`（E0308），修后 `&mut visited`。
2. **`&T` 参数去 clone（`dfa7bd05` 同批）**：`needs_ref_borrow` 加入 `needs_clone`
   排除条件——修前 `dfs(..., &graph.clone(), ...)`（多余 clone），修后 `&graph`。
3. **scalar enum `#[default]` 变体透传（`75ff834b`）**：变体级 attrs 此前被静默
   丢弃 → `#[derive(Default)]` 无 `#[default]` 变体 → E0665。现在按 `item.attrs`
   发射 `#[default]`（TaskMode/PhaseMode 用 `TaskMode.default()`）。
4. **AtomError::InvalidType struct-variant seed（`75ff834b`）**：`InvalidType{expected,
   found}` 注册到 `seed_known_struct_enum_variants` → .at 构造发射 struct 语法
   `AtomError::InvalidType { expected, found }` 而非 tuple（E0599）。

### 已知简化（文档化）
- **TryFrom<Node> trait impl → static fn from_node**：a2r 无 trait impl 语法。
  parity 分别调 hw `TaskPlan::try_from(node)` 与 ag `TaskPlan::from_node(node)`
  比较行为（含错误字符串逐字一致）。
- **`graph.get()` 返回 `Option<&Vec>`**：.at 用 `is graph.get(node)` 直接匹配，
  不可标注 `Option<List<str>>`（类型不匹配 E0308）。
- **`split` 结果类型**：a2r 对 `let parts List<str> = path.split(".")` 强制标
  `Vec<String>`（实为 `Vec<&str>`）→ 无标注 `let parts = ...`（Rust 推断 `Vec<&str>`，
  `parts[i]` 与字面量比较直接成立）。
- **str 参数与 owned String 区分**：`&str` 参数（dfs 的 node）直接传 `contains(node)`；
  owned String 字段赋值用 `.to_string()`（`Some(input.to_string())`）。

### 验收
- `tests/parity_task_plan.rs` **17/17 通过**：TaskMode/PhaseMode wire + default +
  TaskPlan/RunRef builder wire + validate 全错误路径（重复/未知依赖/环/自环/路径
  语法，错误字符串与 hw 逐字一致）+ from_node 全字段/默认值/错误语义。
- **re-transpile 零 drift**：`trans --path task_plan.at rust` 后 diff auto_generated
  = 0（仅约定 `use super::extern_impl::*;` 前缀）。
- 全量：207 lib + 全部 parity 套件 + tool_atoms 23 + 集成测试，14 套件全绿。

### 3.5/3.6 实况（2026-08-05 更新）
- **handoff_store 3.5 ✅ 已闭环**（见 §10.6，7/7 parity）。
- **task_plan_engine 3.6 ⛔ 维持手写边界**：async 泛型闭包 `F: Fn->Fut` 复测
  仍为 a2r 解析错误（`Expected end of statement, got Fn`，§1 硬墙确认）。核心
  execute/run_one 依赖注入 executor 无法表达；可移植部分（new/validate）还依赖
  uuidish（rand）/broadcast（SSE）/get_builtin_flow——均为手写边界，整体移植
  价值低，维持计划原判断（C2）。


## 10.6 Phase 3.5 — handoff_store.rs 移植 ✅ 闭环（2026-08-05）

### 移植范围
`auto-src/handoff_store.at`，对齐 hw `relay/handoff_store.rs`：
- **数据模型**：HandoffStore（data_dir + Mutex cache）
- **save / load**：fs create_dir_all + to_string_pretty + write / read_to_string
  + from_str + cache 读写
- **resolve_path**：split('.') 分段校验（≥5 段 + 第 4 段 "handoff"）+ to_value
  + Value 嵌套 get
- **save_from_run**：use.rust 引用真实 `crate::relay::store::RunStore`
  （`@RunStore` → `&RunStore`，`last_handoff` 跨模块调用）

### 阻塞改判（计划 §6 预估 ~60%，阻塞 a2r-10/11 Mutex tuple-key）
- **tuple-key 实测确认缺陷**：`HashMap<(String,String,String), _>` 的
  `insert(key)` 误加 `.to_string()`（tuple 无 Display，E0277）+ `get(key)` 缺 `&`。
  → **字符串拼接 key 变通**（`"tp/phase/run"`）。cache 是私有字段，行为等价，
  计划 §0 允许实现细节不同（hw tuple vs ag string key）。
- **fs / serde_json / PathBuf join 均可用**（wiki 先例 + 本模块实证）；
  `impl Into<PathBuf>` → PathBuf 参数（C6 已知退化）。

### 死锁修复（a2r match 不释放 guard）
- **现象**：`load()` 首次 `self.cache.lock()` 的 guard 在 `is {... None -> {} }`
  匹配后**仍持锁**（a2r 的 match 生成 `None => {},` 不像 hw 的 if-let 靠 NLL
  释放），随后 `guard2 = self.cache.lock()` 二次加锁 → **Mutex 死锁**
  （parity_load_after_reload_from_disk 挂起 >60s）。
- **修复**：cache 读写抽成独立辅助 fn `cache_get`/`cache_put`（`@Mutex<HashMap<..>>`
  引用参数），guard 在 fn 退出时作用域结束自动 drop，锁释放。
- **验证**：7/7 parity 通过（含跨实例磁盘重载）。

### 联动 a2r 修复（auto-lang 工作树，待并行 session 合并）
- **`&self.field` 传 `@T` 参数误加 self-dot `.clone()`**：`arg()` 对
  `self.dot` 表达式无条件加 `.clone()` → `&self.cache.clone()`（Mutex 无
  Clone，E0599）。修复：call-site 在 `needs_ref_borrow`（`&` 已注入）且参数是
  Dot 表达式时直接发射 expr 跳过 clone。golden 316 无回归。

### 已知简化（文档化）
- `save` 返回 `Result<bool, String>`（hw: `Result<(), String>` — a2r 无法表达
  unit 类型，bool 载荷承载，与 task_plan/specs 同约定）。
- 错误信息：`failed to write handoff` 无 io 详情（nativeize 把 fs::write 桥接
  成 `.is_ok()` bool，丢失错误对象）；create_dir_all/serialize 错误仍带详情。
- 多级 PathBuf join 链（`.join().join().join()`）被 a2r 拆成无效果语句 → 分步
  绑定中间变量。

### 验收
- `tests/parity_handoff_store.rs` **7/7 通过**：save/load + 跨实例磁盘重载 +
  resolve_path 全字段（summary/token_usage.cumulative）/缺失字段/非 handoff 段/
  短路径，与 hw wire 逐字一致。
- **re-transpile 零 drift**；a2r golden 316 无回归；全量 15 套件全绿。

### Phase 4 评估（2026-08-05，记录待后续）
- server 已在 §11 ④ 接线闭环。tools/spec_tools/orch_tools 的 ag 镜像是
  **简化 dormant**：description 文本（hw "Read the full UTF-8..." vs ag
  "Read file contents"）与 schema 详细度均不同，execute 依赖 extern stub
  （§13 C2 "(stub)…"）。三文件 re-transpile 均有 drift（旧产物带 `&`，当前
  a2r 遵循 `.view` 约定）——需 .view 手术 + 元数据对齐才是 full parity。
  因 dormant 不接线（§13 B3 文档化"设计内的等价镜像，非缺陷"），收益低，
  留待后续接线计划处理。


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
| 2 — 已移植模块 | ✅ 8/8 有 parity 测试 | specs 12 ✅ / app_config **6** ✅(2026-08-05 修正,env 竞态两测合并后为 6) / chats 17 ✅ / auth 8 ✅ / tool_safety 7 ✅ / conversation 10 ✅ / mode 4 ✅ / **wiki 11 ✅** / relay 5 ✅ |
| 3 — 缺失模块 | ✅ wiki 试点 ✅（§10）+ **task_plan 3.3 ✅（2026-08-05，17/17 parity）** + **handoff_store 3.5 ✅（2026-08-05，7/7 parity）** | parser/registry 改判为边界（探索实测）；engine 3.6 为 async 硬墙维持手写边界 |
| 4 — 复杂模块 | 🔶 评估完成（2026-08-05） | server 已接线闭环（§11 ④）；tools/spec_tools/orch_tools 为简化 dormant 镜像（description/schema 与 hw 有文本差异 + execute 依赖 stub），§13 C2/B3 已文档化；full parity 需 .view 手术 + 元数据对齐，待后续接线计划 |

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
| ② | **specs/wiki/chats 数据层接线**：extern_impl 对应 stub 换真实委托（走 `s.registry` 的 workspace stores）；ag router 的 specs/workspace/chats 路由接入 | 各端点真实 CRUD 返回，与手写版一致。**✅ specs/chats/workspace 已完成(2026-08-05, 22 stub 委托 + 23 路由接入)** |
| ③ | **extern_impl 剩余 stub 全部真实委托**（config/modes/skills/roles/app-config/harness/conversations/relay/drive/agent/ctx） | 全部端点有真实行为；fake 常量清零。**✅ 已闭环(2026-08-05):config 页 6 路由 + conversations 3 路由 + app-config 2 路由 + harness 2 路由 + agent/ctx 簇 17 stub + relay_driver/factory 委托;drive/relay 编排簇 9 stub 为硬墙(见 C1 ③ relay 段)** |
| ④ | **auto_generated::server 整体接入**：ag build_router（38 路由）作为主 router；7 个 🔴 流式/daemon handler + wiki + 静态文件路由与手写 router 合并 | 全服务端由转译 handler 驱动；原有 45 路由功能不丢。**✅ 已闭环(2026-08-05):serve() 改以 ag build_router() 为主 router(pub + 补 specs_delete 路由),🔴 路由(run/run_stream/workflow_run/workflow_run_stream/settings_link/chat_stream/conversation_stream)+ files + relay/task_plan/wiki 合并;hw health/workflows 死代码移除;组合冒烟测试 production_router_composition_serves_core_endpoints 通过,305 项测试全绿** |

### 手写边界（接线阶段保持手写）
- **数据 store**：auth/specs/chats/wiki 等 store 本体保持手写（ag/hw store 类型不相同的镜像；
  换 store 需 a2r 支持类型同一性 —— 后续独立 dogfooding 目标）。
- **7 个 🔴 handler**（run/run_stream/chat_stream/conversation_stream/workflow_run/
  workflow_run_stream/settings_link）：async 泛型闭包/reqwest/SSE 硬墙。
- **serve() 外壳**（静态文件/CORS/TcpListener/axum::serve）。

### 已完成的接线（滚动更新）
- 2026-08-05：本路线图写入计划；开始 ①。
- 2026-08-05：① auth 接线闭环（serve() 接 ag auth 3 路由，手写 handlers 删除）。
- 2026-08-05：② specs/chats 数据层接线闭环 —— 17 个 extern stub 真实委托 +
  ag specs 8 路由 / chats 10 路由接入 serve()（详见 §12 C1 ② 接线）。
- 2026-08-05：② workspace 路由闭环 —— 5 个 extern stub 委托 + 5 路由接入。
  ② 全部完成（22 stub 委托 + 23 路由，详见 §12 C1）。
- 2026-08-05：③ config 页 + conversations 接线 —— professions/config/modes/
  skills/roles 6 路由 + conversations 3 路由接入（详见 §12 C1 ③ 接线）。

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
  **✅ 已闭环(2026-08-05, 750aeb1)**:server.at 补 44 处 `.view` 标记 + 15 个 handler body 统一
  `let resp = extern(); return Json(resp)`(extern 负责 Value 包装)+ DTO 扩容(RoleSaveBody 12 字段 /
  AppConfigSaveBody+HarnessSelection / AppHarnessSaveBody)+ extern_sigs.at 8 处过期返回类型 → Value。
  **验证:re-transpile(trans --path server.at rust -o + nativeize)后 diff auto_generated/server.rs = 0**,
  hand-edit drift 清零,生成产物完全可由 server.at 复现。304 项测试全绿。
- **结论**:① 是唯一已闭环的接线切片。②③④ 需先决策:
  要么"先对齐 ag store API"(② 的大工作),要么"server.at .view 手术 + DTO parity + extern_impl
  委托 + router 合并"(③④ 的大工作)。两者都是多 session 级别,建议作为下一个独立计划里程碑。

### ② 重新评估:workspace stores 41 处级联是否必需(2026-08-05)✅ 非必需
- **结论:级联非必需。** C2 extern_impl 委托路径(与 auth 委托同模式)已验证可行,
  换 store 类型(41 处级联)作为最后手段,不推荐。
- **PoC 证据**:`specs_load` 从泛型 fake stub 改为走 `s.0.registry` 的真实
  workspace stores(hw SpecsStore):
  `pub fn specs_load(s: &State<AppState>, q: Query<auto_generated::server::WorkspaceQuery>) -> Value`
  → seed 一个 spec item 后,委托返回真实 doc(与 hw specs_list 的 `Json(doc)` 同构)。
  测试 `server::tests::specs_extern_delegation_returns_real_doc` 通过。
- **为什么可行**:
  - extern_impl 是手写 Rust,签名可自由定为具体 `&State<AppState>`;调用点仍由
    extern_sigs.at 的 `@T` 驱动 `&s`,无需动 server.at 的调用形态。
  - ag/hw store 类型不相同的镜像(即便 C1 对齐了写方法签名),swap 仍会波及
    6 模块 41 处;委托路径完全不碰 store 类型。
- **顺带确认**:ag server 有自己的 `WorkspaceQuery` mirror(与 hw 不同),委托
  函数必须用 ag 类型做参数(取 `q.workspace` 手动解析,不用 hw 的 id_or_default)。
- **② 剩余工作(委托路径)**:其余 ~17 个 specs/chats extern stub 逐个改具体签名
  + hw store → ag DTO/Value 转换 + ag router 的 specs/chats 路由接入 serve()。
  每 stub 一个 PoC 同款改动,可增量验收。
- **② 完整接线(2026-08-05)✅**:17 个 specs/chats extern stub 全部真实委托
  (走 `s.0.registry` 的 hw stores),ag server 的 specs 8 路由 + chats 10 路由
  接入 serve()(stream 端点保持手写)。详见 §12。
- **C1 的意义**:specs store `&mut doc` 对齐 + chats CRUD 11/11,让委托可选的
  "swap 到底层 ag store"路径也接近就绪(非必需,但留作未来 dogfooding)。

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
- **chats 缺 11 个 CRUD 方法 ✅(2026-08-05)**:ag ChatStore 11 个方法全部移植
  (含 approve_spec_change,见 §12 C1)。
- **workspace stores 41 处级联 ✅(重新评估完成,2026-08-05)**:**非必需**。
  C2 extern_impl 委托路径已 PoC 验证(specs_load 走 s.0.registry 真实 hw store),
  store swap 作为最后手段不推荐 —— 走"ag handler 委托 + 状态码模型"接线(见 §11 ② 重新评估)。

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

### C1 chats CRUD 移植(2026-08-05)✅ 11/11
- **ag ChatStore 补 11 个 CRUD 方法**:create/list/get/rename/delete/delete_all/
  append_message/queue_spec_change/reject_spec_change/reject_all_spec_changes/
  approve_spec_change,基于 load_map/save_map + `HashMap<str, ChatSession>` 的
  values() 线性查找 + insert 重建 map 模式(不碰 HashMap.remove/contains_key ——
  a2r-10 对 String key 缺 `&`)。
- **list 手动稳定插入排序**(降序 by updated_at;a2r List 无闭包 sort_by)。
- **approve_spec_change(第 11 个)**:chats.at 去掉 mirror SpecChange/SpecStatus,
  直接引用真实 `crate::auto_generated::specs` 类型(ChatSession 与 hw 一样携带
  `Vec<crate::specs::SpecChange>`)。spec 应用走 ag specs store 的
  transition_item/upsert_item(刚对齐的 `&mut doc` 签名)——跨模块调用用显式标记:
  - `.mut` → `&mut doc`(a2r 的 fn_mut_params 只注册当前转译单元,对
    crate::auto_generated::specs 的方法不注入 &mut)
  - `.view` → `&field`(&str 参数;未知 callee 不自动 borrow owned String)
  - **a2r 缺口**:`mut p T` 参数发射 `p: &mut T` 缺 `mut` 绑定,`&mut p` 重借
    E0596 → apply 辅助按值收 doc + `var d` 本地可变绑定,返回更新后的 doc。
- **顺带发现 3 个 a2r codegen 缺口(均有 .at 变通 + 文档)**:
  1. `.append(...)` 无条件重映射为 `.push_str(...)`(String 方法表,对 struct
     receiver 误伤)→ ag ChatSession 方法命名 `push_message`。
  2. `HashMap::insert` 返回 `Option<V>`,a2r 在 if/else 分支尾漏 `;` → Option 泄
     漏成尾类型 → `map_insert` 辅助(绑定结果到 let,返回 void)。
  3. `fix_result_none_unit` 把字面 `Ok(None)` 无条件改 `Ok(())`(破坏
     `Result<Option<T>, _>` 返回)→ not-found 分支写 `return Ok(target)`。
- parity:新增 8 个 store CRUD 对齐测试(create+get / list 结构+排序 / rename
  持久化 / delete+delete_all / append_message 自动命名+持久化 / queue+reject /
  reject_all / approve 状态迁移+upsert+错误路径);parity_chats 17/17 通过。
- 残差:ag id 为 extern new_id stub 产 16 位 hex(hw new_id(12) 为 24 位)——extern
  stub 既有差异,非本移植引入;parity 只比结构不比 id。

### C1 ② specs/chats 数据层接线(2026-08-05)✅
- **17 个 extern stub 真实委托**(走 `s.0.registry` 的 hw workspace stores,与
  auth 委托同模式):
  - specs 8 个:load(先前的 PoC)/ overview(load+rebuild+derive+overview)/
    drift / rebuild+save / related / upsert(ag 请求体 `{section,item{id,title,
    content,status}}` → hw SpecItem 转换)/ transition / delete。
  - chats 10 个:create(含 conversation 双写)/ list / get / rename(含双写)/
    delete(含双写)/ delete_all / message(append+双写 turns,返回
    `{"session","queued"}`)/ approve(经 hw approve_spec_change 应用进 specs doc)/
    reject / reject_all。
- **wire 形状与 hw 一致**:chats 响应改回 `{"session": {...}}` / `{"sessions": [...]}`
  (ag DTO 原为简化版 {id,name,mode});specs 的 DriftResult/RelatedInfo/TransitionOk/
  Deleted 本就与 hw 同构。
- **serve() 接入**:ag specs 8 路由 + chats 10 路由替换 hw handlers;`chat_stream`
  (7 个 🔴 之一)保持手写。**hw 的 18 个 specs/chats handlers + 5 个请求 DTO 删除**
  (死代码,465 行)。
- **手改 auto_generated/server.rs(文档化 drift)**:18 个 handler 加 `pub` +
  chats handler 返回形态改 wire 兼容。根因:当前 a2r 重转译 server.at 有
  s vs s.view drift(见 §11 ②③④),不能靠 re-transpile,只能手改生成产物;
  server.at 仍是规范源(待 .view 手术后可重新转译)。
- **验收**:集成测试 `server::tests::specs_chats_endpoints_run_on_transpiled_handlers`
  覆盖 specs upsert/list/transition/overview + chats create/message/list/get/
  rename/delete 全链路真实 CRUD,与 hw 行为一致。全套 201 lib + parity 通过。

### C1 ② workspace 路由接线(2026-08-05)✅
- **5 个 extern stub 真实委托**(走 `s.0.registry` 真实 registry):
  list(完整 metas,含 last_opened/is_empty)/ open(canonicalize+touch)/ status
  (live is_empty 重查 + root_exists)/ browse(目录遍历,隐藏 dotfiles)/ initialize
  (写 .autoos/initialized 标记)。
- **wire 形状与 hw 一致**:`{"workspaces": [...]}` / `{"workspace": meta}` /
  `{"workspace", "root_exists"}` / `{"entries", "parent"}` / `{"status",
  "workspace"}`。ag DTO 原为简化版(id/name/empty 等),handlers 手改返回 Value。
- **serve() 接入**:workspace 5 路由替换 hw handlers;删除 hw 死 handlers 5 个 +
  DTO 2 个(95 行)。
- **验收**:集成测试 `server::tests::workspace_endpoints_run_on_transpiled_handlers`
  覆盖 list/open/status/browse/initialize 全链路,与 hw 行为一致。
- 至此 ②(specs/chats/workspace)闭环:22 个 extern stub 委托 + 23 路由接入。

### C1 ③ config 页 + conversations 接线(2026-08-05)🔧 部分
- **config 页 6 路由委托**:
  - professions/config/modes/skills(读真实 ModeRegistry/SkillRegistry/builtin
    professions,返回 hw wire 形状)
  - roles(roles_all/role_get 读 RoleRegistry;role_save_of/role_delete_of 读写,
    ag RoleSaveBody 扩为 hw 全字段 description/inherit/allowed_tiers/skills/
    token_budget/max_turns/tools/soul)
- **conversations 3 路由委托**:list/get/rename/delete 走 workspace
  ConversationStore(chat_create 双写保证 conversation 与 session 联动)。
- serve() 接入 9 路由(/api/professions|config|modes|skills|roles|roles/{name} +
  /api/conversations|{id}|{id}/title);删除 hw 死 handlers 12 个 + DTO 2 个
  (308 行)。stream 端点保持手写。
- **验收**:集成测试 config_endpoints_run_on_transpiled_handlers +
  conversations_endpoints_run_on_transpiled_handlers;204 lib + parity 全过。
- **③ 剩余**:app-config + harness(需 MuskAppConfig 全字段/HarnessSelection
  序列化 + app_harness_dir 扫描)和 relay/drive/agent/ctx(🔴 流式 handler 背书
  stub,涉及 relay 编排引擎/agent 构建)——挂起,待后续切片。

### C1 ③ relay/agent/ctx 委托 + parity(2026-08-05)✅
- **范围结论**:relay/drive/agent/ctx extern 只支撑编译但休眠的镜像模块
  (server_serve/server_stream/auto_lib/relay_driver/orch_tools),活引擎是手写
  src/relay/*。auto_lib 是混合镜像(直接 use hw 工具/orch 类型)。委托零运行时风险。
- **lib.rs**:resolve_role/find_context_file 改 pub(crate),抽出 find_ctx_upward。
- **driver.rs**:MuskAgentFactory pub(供 extern factory_build_agent + parity 构造)。
- **auto_lib .view/.mut 手术**:register_shared/skill_tool 需 &mut → 调用点补
  `.mut`(agent_register_shared(agent.mut,...))+ var agent;其余 extern 调用补
  `.view`。re-transpile 后与旧产物差异仅为 &mut + 等价 for 借用(文档化)。
- **extern_impl 委托 17 stub**:agent 簇(register_shared/skill_tool/
  with_context_file/with_history/build_agent_with_context/mode_tools_contains)+
  role 解析簇(registry_resolve/load_builtin_role/read_at_file/resolve_role)+
  context 簇(find_context_file/find_ctx_upward/current_dir/ctx_is_some/
  ctx_unwrap)+ relay_driver 路径(handoff_render/factory_build_agent)+
  drive_clear_root。
- **relay_driver.at**:MuskAgentFactory + build_agent 加 pub(parity 访问)。
- **验收**:tests/parity_relay.rs 5 项 —— parity_builtin_flows_match(FlowSpec
  wire 相等)、parity_profession_registry_matches_hw_semantics(hw 数据 seed 喂 ag
  registry + wire round-trip + get/can_handoff/needs_approval/register)、
  parity_build_agent_from_mode_registers_same_tools(ag auto_lib vs hw,工具集
  相等,覆盖全部 agent 簇委托)、parity_factory_build_agent_matches_hw_factory、
  parity_relay_driver_factory_build_agent(带/不带 handoff)。310 项测试全绿。
- **硬墙边界(未委托,文档化)**:drive/relay 编排簇 9 stub —— relay_advance/
  publish/step_context/submit_error + drive_set_root/accumulated/finalize_output/
  submit_handoff/handle_stream_event + agent_run_stream_with_sink。原因:转译
  drive_loop/run_step 的 extern 只收 (ws_id, run_id) 字符串(无 state 注入),
  且 DriveStreamSink::on_event(i32) 丢 Delta 文本 —— 忠实委托需 server_serve.at
  事件管道重构(sink 签名改 Value + state 透传),非"委托"能解决。orch_*/serve_*/
  stream_* 簇同理(死镜像 + 手写边界)。

### C1 ③ app-config + harness 接线(2026-08-05)✅
- **app-config 2 路由委托**:load(读 MuskAppConfig → {stored, effective})/
  write(ag AppConfigSaveBody 扩为 hw 全字段 daemon_url/default_mode/context_file/
  serve_addr/auto_start_daemon/harness{roles,skills,modes} → 写 config.at)。
- **harness 2 路由委托**:list(读 MuskAppConfig harness 选中项 + os_available +
  app_custom 两级)/ save+delete(roles kind 写/删 app 级 .at 角色 + soul)。
  server.rs 的 app_harness_dir/scan_app_* 辅助改 pub(crate) 供 extern 复用。
- serve() 接入 4 路由(/api/app-config + /api/app-harness/{kind}|/{kind}/{name});
  删除 hw 死 handlers 5 个 + DTO 3 个(234 行)。
- **验收**:集成测试 app_config_endpoints_run_on_transpiled_handlers 覆盖
  app-config + harness list 读端点(写端点会碰用户真实配置,不测);
  205 lib + parity 全过。
- **③ 剩余**:relay/drive/agent/ctx(🔴 流式 handler 背书 stub)——挂起。
  **✅ 已闭环(2026-08-05,见上"C1 ③ relay/agent/ctx 委托 + parity"):agent/ctx
  簇 17 stub + relay_driver/factory 委托 + parity_relay 5 项;drive/relay 编排
  簇 9 stub 为硬墙(需 server_serve.at 事件管道重构),文档化为下个边界。**

---

## 13. 复审问题清单(2026-08-05)🔧 修复中

全量复审(审计 agent + 人工验证)发现的问题,按优先级记录;修复进度见每项 ✅。

### A. Live 路径真 bug(WORKAROUND 产生错误行为)

| # | 问题 | 位置 | 影响 | 状态 |
|---|---|---|---|---|
| A1 | `path_inner` 返回空串,`chat_delete`/`conversation_delete` 的响应 id 永远 `""` | extern_impl.rs:962 | `DELETE /api/chats/session/{id}` 与 `/api/conversations/{id}` wire 错误(hw 返回真实 id)。测试只断言 status 未断言 id,漏网 | ✅ 修复 |
| A2 | `random_hex` 用单个 u64 零填充 → auth salt(16)/session token(32)只有 64-bit 熵 | extern_impl.rs:53 | **安全回归**:生产 auth 端点(ag AuthStore)token/salt 可预测;hw 用 fill_bytes 全随机 | ✅ 修复 |
| A3 | 错误语义全量回归:28 个委托函数出错返回 `Value::Null`/空串,handler 一律 `Json<Value>` → **200+null/空 body**(hw 4xx/5xx+错误 DTO) | extern_impl 各委托 + server.at handlers | chat_get 不存在→200 null(hw 404);specs_transition 失败→200 `{"new_status":""}`;role_save 失败→200 null。§11 C3 状态码模型只修了 auth 切片 | ✅ 修复(to_response 模型) |
| A4 | `chats_delete_all` 吞失败:delete_all 失败也返回 200 `{"status":"deleted_all"}`(hw 500) | extern_impl.rs:673 | 写失败被静默 | ✅ 修复 |

### B. 文档/计划出入(遗漏)

| # | 计划声称 | 实际 | 状态 |
|---|---|---|---|
| B1 | parity_app_config 7/7(§9 进度表,行 289) | 6(env 竞态两测合并后未更新计数;§9 历史记录行 307 为合并前快照,保留) | ✅ 修正 |
| B2 | lib 测试计数 189/199/201/204/205(各处验收记录) | 这些是各提交时点的历史计数(准确);当前总数 311 | ✅ 已核实为历史记录,无需改 |
| B3 | ③ 验收"fake 常量清零"标 ✅ | **未达成**:extern_impl 151 个函数中 68 个仍为 fake stub — 2 个 live(A1 path_inner、workflows_builtin_names 硬编码)、56 个 dormant-only、10 个零调用 | 🔶 记录;live 2 个已修(path_inner 修复;workflows_builtin_names 硬编码当前值与 hw 一致,留作待办) |

### C. 休眠镜像潜在雷(若接线会 panic/垃圾,当前不可达)

| # | 问题 | 位置 | 状态 |
|---|---|---|---|
| C1 | `serve_init_state` `unimplemented!()`(dormant server_serve 的 serve()) | extern_impl.rs:927 | 🔶 文档化(休眠) |
| C2 | tools.rs 9 个 stub 返回 `"(stub)…"` 且绕过路径安全/审批(dormant auto_generated::tools) | extern_impl.rs:935-947 | 🔶 文档化(休眠) |
| C3 | server_stream/server_serve 假管道:mpsc 永不投递、relay_advance→Null、advance_kind→"completed" | extern_impl.rs:824-855,949-956 | 🔶 文档化(休眠,与 drive/relay 硬墙同源) |
| C4 | TreeNode file 节点 size/modified=None(dormant auto_generated::wiki) | auto_generated/wiki.rs:371,381 | 🔶 已在 §10 文档化 |
| C5 | `new_id(12)` 16 vs 24 hex(dormant ag chats) | extern_impl.rs:52 | ✅ 随 A2 修复 |

### D. 已解决/非问题(复审确认,无需处理)

- chats SpecChange 镜像已移除,直接引用真实 `crate::auto_generated::specs` 类型,SpecStatus 23 变体全量共享。
- `workflows_builtin_names` 硬编码 `["feature-dev"]` 与 hw 当前一致(仅未来新增 workflow 时漂移)。
- 其余 dormant 镜像(orch_tools/server_stream/server_serve/relay_*)为 014/015 设计内的"等价镜像",非缺陷。

### E. 测试覆盖缺口

| # | 缺口 | 状态 |
|---|---|---|
| E1 | HTTP 层无测试:`/api/run*`、`/api/settings-link`、`/api/chats/session/{id}/stream`、`/api/conversations/{id}/stream`、`/api/files/*`、全部 `/api/forge/*` | 🔶 记录(🔴 手写 handler + forge 独立,待补) |
| E2 | 错误路径无断言:无任何 404/500 断言(仅 auth 401) | ✅ A3 修复后补 delete id + 404 断言 |
| E3 | `production_router_composition` 对不存在的 specs item 断言"200 空 id"——固化 A1 bug 行为 | ✅ 修正为 404 断言 |

### 修复实施(2026-08-05, 与 §13 同 session)

- **A1/A2/A4 + A3 extern 层**:extern_impl.rs — `path_inner` 改 `&Path<String> → p.0.clone()`;`random_hex`/`new_id` 改 `fill_bytes` 全随机(hw 同语义);`specs_drift/related_of/transition_of/delete_of` 与 `chats_delete/conversations_delete/chats_delete_all` 改返回 `Value`(错误 → `Value::Null`,成功返回完整 wire 形状 DTO);新增 `to_response(v, msg, code)` helper(`Value::Null` = 错误 → `err_response`;否则 `ok_response`)。
- **A3 handler 层**:server.at 30 个 handler 改 `~Json<Value>` → `~Response`,body 改 `return to_response(extern(...), "msg", code)`(错误码:404 not-found / 400 校验写 / 500 加载 IO)。re-transpile 后 diff 仅预期变化。
- **E2/E3 测试**:`delegated_endpoints_return_http_errors`(chat get/delete 不存在 → 404,workspace status 不存在 → 404);delete 集成测试补 `id` 断言(chat + conversation);`production_router_composition` 的 specs delete 断言改 404。
- **验收**:311 项测试全绿(207 lib + parity 各套件 + 新错误语义测试)。
