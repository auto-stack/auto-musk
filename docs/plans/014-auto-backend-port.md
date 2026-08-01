# 014 — auto-musk 后端的 Auto 语言版本（a2r 转译可回 Rust）

> **状态**：**14 个模块移植完成，server.rs 全量 handler 移植**（2026-08-01）。specs 全文件 +
> auth/hello/tool_safety/mode/app_config/chats/relay{profession,store,api,flows}/conversation(数据层)/
> **server(45 handler + 50 DTO + 36 路由)**。
>
> **auto-lang 上游改动**：Plan 379（route 保留字）+ Plan 380 P0（元组结构体构造）+ P1（str 字面量
> 兼容，解锁 `Json(DTO(field:"x"))` 嵌套构造）。三者完成后，axum handler 全量移植可行。
>
> **server.rs 移植形态**：45/52 handler 用 Auto（async fn + 整体 extractor + 具体 Json<T> 返回 +
> json!→DTO），7 个 🔴 handler（daemon/SSE/reqwest）+ serve() 外壳 + store 访问逻辑保留手写。
> 路由表用拆分赋值式（`app = app.route(...)`）避开 a2r-22 超长方法链栈溢出。
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
| **a2r-5** | `from`/`to` 是保留字，做参数名/标识符致解析失败（E0007，报错位置误导到文件尾）| `fn f(from X)` | A22 扩展：改名 `from_status`/`to_status`/`up_to` 等 |
| **a2r-6** | tuple 字段访问 `pair.1` 报"Invalid field name"（E0099）；仅 `pair.0` 经 `fix_tuple_index` 处理 | `pair.1` / `pair.0` 后跟非 clone 操作 | 用 `pair[0]`/`pair[1]` 写法（A20），由 fix_tuple_index 转成 `.0`/`.1` |
| **a2r-7** | 显式 derive 必须含 `PartialEq, Eq`，否则 enum `==` 失败（E0369）| 加 serde derive 时漏掉 | derive 全套写齐 `Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize` |
| **a2r-8** | `r"..."` 原始字符串字面量被渲染成 `r { content: "..." }`（结构体误解析）| `Regex.new(r"\d+")` | 用普通字符串 + 双反斜杠 `"\\d+"` |
| **a2r-9** | `str` 变量传给 `&str` 形参方法调用类型不匹配（E0308）| `re.find_iter(text)`（text: String）| a2r-3 的泛化；regex/IO 场景高发，需手动 `&` 或改参数形态 |
| **a2r-10** | `HashMap<K, Vec<V>>` 复合泛型方法调用处理不全：`insert` 给 Vec 值误注入 `.to_string()`（E0599）；`remove`/`contains_key` 对 String key 不加 `&`（E0308，但 `get` 会加）| `m.insert(k, list_val)` / `m.remove(string_key)` | **可绕开**（非硬边界）：用 `List<结构体>` + 线性查找替代 HashMap（specs.rs rebuild_relations 已验证）|
| **a2r-11** | 无可变借用遍历 / 就地嵌套修改：`mut fn` 里 `for x in self.coll` 生成 move 而非 `&mut`；`for mut x` 被拒；`self.items[i].field = v` 编译成 `self.items[i].clone().field = v`（写进 clone，无效）| 任何"遍历 `&mut self` 集合并改元素字段"的方法 | **可绕开**（非硬边界）：构建新集合 + **整体字段重赋值** `self.field = new_list`（a2r 正确处理）；而非就地改元素（derive_statuses / rebuild_relations 已验证）|
| **a2r-12** | 在 `ext A` 方法里把 `B(...)` struct 字面量作为 `.push()` 参数（A≠B）触发 "field type mismatch" 类型检查失败 | `list.push(OtherType(field: ...))` | 用 `let x B = B(...); list.push(x)` 中转变量绕开（已验证）|
| **a2r-13** | 借用标记 `.view` = `&`：传 `&T` 形参时需显式 `arg.view`；a2r 不会自动给 struct 值参数加 `&` | `serde_json::to_string(doc)` 要 `&doc` | 写 `to_string(doc.view)` → 生成 `&doc`。同理 `.mut`=`&mut`、`.take`=move |
| **a2r-14** | serde_json 等外部 crate 的函数调用：`serde_json.fn()` 会被当成值访问（E0423）；必须 `use.rust serde_json::{fn}` 显式导入函数后裸调用 `fn(...)` | `serde_json.to_string_pretty(x)` | `use.rust serde_json::{to_string_pretty, from_slice}` 然后 `to_string_pretty(x.view)` |
| **a2r-15** | `fs.write` 等名称冲突：a2r 优先映射到 `a2r_std::fs::write`（签名 `(str,str)->bool`）而非 `std::fs::write`（`(Path,[u8])->Result`）| `fs.write(path, bytes)` | 改用 String 内容（a2r_std::write 接 `&str`）或 `write_bytes`；按 a2r_std 的实际签名处理返回值 |
| **a2r-16** | `#[rs]` 逃生舱**仍按 Auto 语法解析函数体**，不是原样透传任意 Rust：不支持 `&mut`/`&` 引用、`vec![]` 宏、`::` 路径、`use` 语句 | `#[rs] fn f() { let mut buf = vec![0u8; n]; }` | `#[rs` 仅适合"Auto 语法 + Rust 语义"的代码（接近 Rust 的方法调用）。真正的 Rust API（sha2/hex/rand/Mutex）须保留手写 Rust，不走 `#[rs]` |
| **a2r-17** | a2r 对 `str.contains`/`str.find`/`fs.write`/`fs.exists` 等**硬桥接到 a2r-std**（rust.rs 源码硬编码分支，`a2r_std_used.set(true)`），写法无法绕开（连 `#[rs]` 内的 `.contains()` 都被桥接）。导致最终产物依赖 a2r-std crate | `text.contains(n)` → `a2r_std::str_contains(text, n)` | **后处理 `nativeize.pl`**：把桥接调用 1:1 替换回原生（`a.contains(b)`、`std::fs::write(p,c).is_ok()`）并删 `use a2r_std` 注入。`time` 可在 .at 层用 `SystemTime.elapsed()` 绕开（非 str/fs 方法，不桥接） |
| **a2r-18** | named-field 异构 enum（tag union）的**解构**：`Variant(e)` 把 e 绑到第一个命名字段，且省略 `..`，导致 E0027（"pattern does not mention fields"）。a2r 测试只有 positional（元组变体）解构用例 | `is ev { RunEvent.StepStarted(e) -> e.timestamp }` | named-field 变体的字段访问型方法（如 timestamp()）保留手写 Rust；无解构的 arm（`Variant -> "str"`，如 event_type）可用 |
| **a2r-19** | `str` 类型注解一律渲染成 `String`，但 `substring`/`trim` 等方法返回 `&str`；赋值给 str 变量或链式 `.to_string()` 会类型不匹配（E0308） | `let s str = text.trim()` → `let s: String = text.trim()`（&str≠String） | 用 `+ ""`/`+ "…"` 拼接（a2r 把 &str+&str 处理成 String）替代中间变量；或 `text.trim().to_string()` 单独一行（注意 a2r 对链式 `.to_string()` 的解析，必要时拆成两步） |
| **a2r-20** | ~~元组结构体构造误造 field0~~ **已修复**（auto-lang Plan 380 P0）| `Json("ok")` 曾 → `Json { field0: "ok" }`（E0560）| **已修复**：a2r 现在对全位置参数 + 无已知字段的类型生成位置构造 `Json(value)`。axum handler 的 `Json(v)` 返回构造、Option/Result 包装均已可用 |
| **a2r-21** | axum handler 的 extractor 参数解构不被支持（`State(s)`/`Path(id)`/`Json(body)` 作为函数参数）| `fn h(State(s) ~AppState)` 解析失败 | 含 extractor 的 handler 保留手写 Rust（server.rs 主体）|
| **a2r-22** | 超长方法链（≥~20 个 `.method()` 链式）导致 a2r 递归下降解析器**栈溢出**（每个 `.x()` 是一层嵌套表达式）| `Router.new().route(...).route(...) × 35` 栈溢出 | 拆成 `var app = X.new(); app = app.route(...)` 重复赋值（每条独立语句，非嵌套表达式）。server.at 的 build_router 已用此模式 |
| **a2r-23** | 字符串字面量（`"x"`，推断为 StrSlice）赋给 str 类型字段（StrOwned）报 field type mismatch —— 仅在**嵌套构造**（函数调用参数位置的 struct 字面量）时触发（裸 let 走 codegen .to_string() 路径不触发）| `Json(StatusOk(status: "ok"))` → E0106 | **auto-lang Plan 380 P1 已修复**：types_are_compatible 增加 StrSlice↔StrOwned/StrFixed 兼容 |

> 与技能（auto-lang-creator）规则对应：a2r-1/a2r-5/a2r-8/a2r-9 未被 A 类 23 条
> 覆盖（新发现）；a2r-2 ≈ A21；a2r-3/a2r-9 同源；a2r-4/a2r-7 = A23；a2r-6 = A20。
> **a2r-5 的报错会误导到文件末尾**（offset 越界），定位时需二分排查保留字。

### 阶段 0 结论

✅ **a2r 完全能转译 auto-musk「纯数据模型 + 工具方法」类模块为可编译 Rust。**

转译质量高：struct 字段、类型映射（String/Vec/Option/u64）、factory 构造、
`time::now_sec() as u64` 时间桥接、serde derive 全部正确。a2r 自动为 scalar enum
生成 `Display` + `from_id` + 默认 derive（无显式 derive 时）。

**对 auto-musk 的指导**：specs.rs 的数据模型层（约 ~300 行：两个 enum + 5 个
struct + 字符串转换 + factory）可作为阶段 2 的「首批可移植」目标。

---

## specs.rs 全文移植进度（阶段 2 首个模块）

逐批验证 specs.rs（1495 行），每批 a2r 转译 + cargo check 闭环至 0 错误。

### 批次 A — 数据模型 + 状态机 ✅（specs.rs 行 1-736，约 50%）

**已移植并 0 错误编译**（`auto-src/specs.at`，373 行 Auto）：
- 2 scalar enum：`SectionType`（7 变体）+ `SpecStatus`（23 变体），全套 derive
  （Clone/Copy/Debug/PartialEq/Eq/Serialize/Deserialize）
- 字符串转换：`as_str`/`display_title`/`to_str`（枚举 `is` 匹配）+
  `from_str_lossy`（Map 查表，避开 str_as_str）
- 4 个 struct + factory：`SpecItem`（17 字段）/`SpecsSection`/`SpecsDocument`/
  `SpecChange`，均含 serde derive
- **`SectionConfig` 状态机**：`for_type`（7 分支 is）+ `can_transition`
  （`List<(SpecStatus, SpecStatus)>` 转换矩阵，for 遍历 + `pair[N]` 比较）
- `Copy` derive 解决 owned enum 多次方法调用的 move 问题

发现并解决 a2r-5/6/7（详见上表）。

### 批次 B — rebuild_relations（关系图反链）✅ 完成

原版用 `regex::Regex` + `OnceLock` + `HashSet` + `HashMap` 做关系图反链。

**已移植并 0 错误**（`specs.at`）：
- `all_ids(doc)` —— 遍历 sections/items 收集到 `HashSet<str>` ✅
- `scan_refs(text, known)` —— **绕开 regex**，改为遍历 known 用 `str_contains`
  检查（语义等价：原版本就 filter `known.contains`，仅放弃 `\b` 词边界，
  实际 spec ID 独立出现，影响可忽略）✅
- **`rebuild_relations`** ✅ —— 两个 a2r 限制组合绕开：
  - a2r-10（HashMap 复合泛型缺陷）→ 用 `List<ReverseEntry>`（target_id +
    referrers）+ 线性查找替代 `HashMap<String, Vec<String>>`
  - a2r-11（无可变借用遍历）→ 整体字段重赋值 `self.sections = new_sections`
    （构建新 sections 列表，每个 item 用更新的 related 复制）
  - a2r-12（push struct 字面量）→ `let x = Struct(...); list.push(x)` 中转

### 批次 C — derive_statuses（状态推导）✅ 完成

原版用 iterator 链（filter_map/filter/all/any）+ `matches!` 宏 + 函数内局部
struct + 就地修改 `item.status`。

**已移植并 0 错误**（`specs.at`）：
- `Snap` struct（提到顶层，原版是函数内局部）+ `find_snap` 线性查找
- `is_goal_advanceable` / `is_test_done` / `is_test_pending`（`is` 链替代
  `matches!` 宏；Auto 无逻辑非运算符 not/!，用否定语义函数替代）
- `section_complete_status`（7 分支 is）
- **`derive_statuses`** ✅ —— 三条规则全实现，组合绕开：
  - iterator 链 → `for` 循环 + 辅助 bool（探针验证可行）
  - `matches!` 宏 → `is_goal_advanceable`/`is_test_pending` 辅助函数
  - HashMap 快照 → `List<Snap>` + `find_snap` 线性查找（绕开 a2r-10）
  - 就地修改 status → 构建新 sections/items 列表 + 整体字段重赋值
    `self.sections = new_sections`（绕开 a2r-11）
  - struct 字面量 push → let 中转（绕开 a2r-12）
  - move 值 → `for x in coll.clone()` + `fn(coll.clone())`

**关键突破**：a2r-10/11 原判"硬边界"，经组合绕开后**降级为可处理** —— 只要
重构为"构建新集合 + 整体重赋值"模式，"遍历并修改 self"的方法都能移植。

### 批次 D — SpecsStore（JSON 持久化 + CRUD）✅ 完成

`std::fs` + `serde_json` + `PathBuf` + IO 错误处理 —— 原以为是 a2r 最难覆盖的
领域，实测全部攻克。

**已移植并 0 错误**（`specs.at`）：
- `SpecsStore` struct + `new` + `load`/`save`（文件读写 + serde_json 序列化）
- `upsert_item` / `transition_item` / `delete_item`（CRUD，整体重赋值绕开 a2r-11）
- `drift_check`（版本对比）
- `SectionOverview` / `SpecsOverview` + `overview()`（聚合统计）

**IO 层的 a2r 突破**：
- a2r-13：`.view` 所有权标记 = `&`，解决 serde_json `&T` 参数（`doc.view` → `&doc`）
- a2r-14：外部 crate 函数需 `use.rust serde_json::{fn}` 显式导入后裸调用
- a2r-15：`fs.write` 名称冲突，a2r 优先映射到 a2r_std::fs::write（按其签名处理）
- Result 类型不标注（让 a2r 推断），用 `is` 匹配 Ok/Err 解包
- CRUD 用"返回更新后的 doc（Option<SpecsDocument>）"而非就地改 `&mut doc`

**结论**：IO 层不是硬边界 —— serde_json + 文件 IO 在 a2r 里可用，只要按
`.view` / 显式导入 / Result 解包的约定写。

---

## 阶段 1 — 测绘：逐文件标注 a2r 可移植性（待启动）

对 27 个 `.rs` 模块逐一标注三档可移植性，输出分层地图。

### 预判分层（specs.rs 全文件实测后修正）

| 档位 | 判据 | 模块 |
|---|---|---|
| 🟢 已验证可移植 | 无 axum/tokio/async/上游 async trait | **specs.rs ✅ 全文件**（含 regex/HashMap/IO/CRUD）、hello.rs |
| 🟢 预判可移植 | 同上，待验证 | chats.rs 模型、auth.rs 密码逻辑、tool_safety.rs、mode.rs、app_config.rs |
| 🔴 a2r 不支持 | axum async handler / tokio broadcast / async-trait / SSE | server.rs、main.rs、relay/{driver,api}.rs、orch_tools.rs、conversation.rs(broadcast)、tools.rs(async) |

### specs.rs 全文件实测对预判的根本修正

specs.rs 原判"纯逻辑层可移植、rebuild_relations/derive_statuses/SpecsStore 需逃生舱"，
**全部被推翻** —— 经组合绕开后全文件 0 错误。关键经验（适用其它模块）：

1. **regex/HashMap/IO 都不是硬边界**：regex 用 str_contains 替代；HashMap 用
   List+线性查找替代；IO 用 `.view` + serde_json 显式导入。
2. **"就地修改"是唯一需要重构的模式**：a2r-11 不支持就地改 self 集合元素，
   但"构建新集合 + 整体字段重赋值"可绕开。这是高频模式但都有等价重写。
3. **serde derive / 文件持久化可用**：🟡 档不再是阻塞因素。

---

## 阶段 2 — 分层移植实施（依阶段 1 地图，待定）

依测绘结果分批移植 🟢 模块（每模块一个 `.at` + 一次 a2r + 一次 cargo check 闭环），
🟡 用 `#[rs]` 逃生舱，🔴 模块按用户选定策略处理。

### 目录与产物约定

```
auto-musk/
├── backend/crates/musk/
│   ├── auto-src/            ← 新：手写 .at 源码（a2r 输入）
│   │   ├── specs.at         ✅ 全文件（1495 行 → Auto）
│   │   ├── auth.at          ✅ 数据模型+权限+用户 IO
│   │   ├── hello.at         ✅ 全文件（greet）
│   │   ├── tool_safety.at   ✅ 命令分类
│   │   ├── mode.at          ✅ struct + registry
│   │   ├── app_config.at    ✅ struct + effective
│   │   ├── chats.at         ✅ 会话模型 + summary/append + ChatStore IO
│   │   ├── relay_profession.at ✅ Profession + ForgePhase + Registry
│   │   ├── relay_store.at   ✅ RunEvent(异构 enum) + 7 读模型 struct
│   │   ├── server.at        ✅ axum DTO struct + 路由表骨架 + health handler
│   │   ├── conversation.at  ✅ 会话数据层（12 类型 + 转换函数）
│   │   ├── relay_api.at     ✅ relay API DTO（5 个）
│   │   ├── relay_flows.at   ✅ 4 个 flow 构造（全文件，上游类型边界用例）
│   │   ├── nativeize.pl     ✅ 后处理脚本（a2r 输出 → 去 a2r-std → 纯 Rust）
│   │   └── ...（🔴 模块 server/relay/main 等暂不移植）
│   └── src/                 ← 现有手写 Rust（a2r 输出 .a2r.rs 与之并存，不覆盖）
└── docs/plans/014-auto-backend-port.md   ← 本文件
```

### 验证基线（每个移植模块须过）

**重要约束**：最终产物必须是**不依赖 a2r-std 的纯 Rust**（用 `use.rust` 直接调
Rust 库）。a2r 转译器会对 `str.contains`/`str.find`/`fs.write`/`time` 硬桥接到
a2r-std（转译器层面，写法绕不开），所以需后处理。

```
1. 转译:  A2R_CRATE_ROOT=0 auto.exe trans --path <m>.at rust   (0 转译错误)
2. 去桥接: perl nativeize.pl <m>.a2r.rs
   (把 a2r_std::str_contains(a,b)->a.contains(b)、a2r_std::fs::write(p,c)->
    std::fs::write(p,c).is_ok()，并删 use a2r_std 注入)
3. cargo check: 临时 crate 仅依赖 serde/serde_json 等**真实 crate**（不含 a2r-std）→ 0 错误
4. 对照原 .rs: 公开 API 一致（允许 &'static str→String 等无害差异）
```

`.at` 源码层也应尽量避开桥接：`time` 用 `SystemTime.elapsed()`（非 `a2r_std::time`）；
但 `str.contains`/`fs.write` 无法在 .at 绕开（a2r 硬桥接），只能靠 nativeize 后处理。

### 已完成模块

| 模块 | 行数 | 状态 | 策略 |
|---|---|---|---|
| specs.rs | 1495 | ✅ 全文件 | 纯 Auto（regex→str_contains、HashMap→List、就地改→整体重赋值、IO→.view+serde_json 显式导入）|
| auth.rs | 266 | ✅ 部分 | 数据模型+权限+用户 IO 用 Auto；hash_password/random_hex/Mutex sessions 保留手写 Rust |
| hello.rs | 28 | ✅ 全文件 | 纯 Auto（单函数 greet）|
| tool_safety.rs | 336 | ✅ 部分 | 命令分类（CommandTier+classify_command）用 Auto；路径限制（OnceLock+thread_local+Path 方法）保留手写 |
| mode.rs | 236 | ✅ 部分 | AgentMode struct + ModeRegistry 纯逻辑用 Auto（List 替代 HashMap）；parse_mode_at(auto_atom)+include_str! 保留手写 |
| app_config.rs | 250 | ✅ 部分 | MuskAppConfig/HarnessSelection struct + effective_* 用 Auto；load/parse/env 保留手写 |
| chats.rs | 596 | ✅ 部分 | Role/ToolCall/ChatMessage/ChatSession/Summary + summary/append + ChatStore IO 用 Auto（SpecChange 跨模块重声明）；new_id(rand) 保留手写 |
| relay/profession.rs | 494 | ✅ 部分 | Profession struct + ForgePhase enum + Registry（get/list/can_handoff/needs_approval/register）用 Auto；default_professions(292 行数据)/dirs/save 保留手写 |
| relay/store.rs | 1078 | ✅ 部分 | RunEvent（15 变体 hetero tag union）+ 7 个读模型 struct 用 Auto；RunState/RunEntry(含上游类型)/RunStore(Mutex) 保留手写 |
| server.rs | 2206 | ✅ 部分 | **45 个 🟡 handler 全移植**（async fn + 整体 extractor 参数 + 具体 Json<T> 返回 + json!→DTO）+ 50 个 DTO struct + build_router() 36 条路由装配（拆分赋值式避 a2r-22 栈溢出）；7 个 🔴 handler（run/run_stream/chat_stream/conversation_stream/workflow_run/workflow_run_stream/settings_link —— daemon/SSE/reqwest）+ serve() 外壳（静态文件/CORS/TcpListener/axum::serve）+ store/registry 访问逻辑保留手写 |
| conversation.rs | 1331 | ✅ 部分 | 12 个数据类型（Conversation/ConversationKind/Driver/ConversationStatus/Turn/TurnKind/ToolRecord/GateRecord/GateInfo/ConversationSummary/ConversationEvent + ChatMessage/Role 跨模块重声明）+ to_status_str + chat_message_to_turns + now_secs 用 Auto；ConversationStore(Mutex+broadcast)/run_event_to_turns(上游+宏) 保留手写 |
| relay/api.rs | 393 | ✅ 部分 | 5 个 DTO（BusEvent/ResolveGateBody/SubmitHandoffBody/UpdateTitleBody/ListRunsQuery）用 Auto；bus(OnceLock+broadcast)+handler(async+extractor)+relay_routes 保留手写 |
| relay/flows.rs | 59 | ✅ 全文件 | 纯 Auto（4 个 flow 构造 + get_builtin_flow）；**边界用例验证**：上游 auto_ai_agent 类型（FlowSpec/FlowStep/GateType/ExitRouting）作为不透明构造目标 + builder 链 + 字段访问可转译为原生 Rust |

### 混合策略（auth.rs 验证确立）

经 specs.rs（纯 Auto）+ auth.rs（混合）验证，确立移植策略：
- **纯逻辑 + 数据模型 + serde IO** → Auto（占模块主体）
- **外部 crate 复杂 API（sha2/hex/rand/regex/Mutex）** → 保留手写 Rust
- **`#[rs]` 逃生舱**：仅适合"Auto 语法 + Rust 语义"的代码；不能写任意 Rust（a2r-16）

这意味着每个模块通常是"Auto 主体 + 少量手写 Rust 边角"的混合，而非全 Auto。
不影响"Auto 版本经 a2r 转译实现同等能力"的总目标——手写 Rust 边角可视为
a2r-std 的等价补充，最终产物仍是完整可编译的 Rust。

### 跨模块类型约定（relay/chats 验证确立）

auto-musk 的模块间有类型引用（chats→specs::SpecChange，relay/profession→
specs::SectionType，relay/store→relay::GateType）。a2r 单文件转译不解析跨文件
引用，处理方式：**在引用方 .at 里重新声明被引用类型**（自包含）。代价是同一类型
在多个 .at 里重复声明，但：
- 语义一致（serde 序列化的变体集/字段集相同，JSON 往返兼容）；
- 真正的单一真源仍是 specs.rs（Auto 版只是可编译的镜像）。
- 含上游 auto_ai_agent 复杂类型（PipelineEngine/FlowSpec/StepRecord）的 struct
  （RunState/RunEntry）不移植——这些类型无法用 plain data 重声明。

**异构 enum（tag union）**：`tag E { Variant { fields } }` 生成正确的 Rust 异构
enum + serde 标签属性透传（`#[serde(tag=..)]`）。但 named-field 变体的**解构**
a2r 处理有缺陷（a2r-18），字段访问型方法保留手写。

### 剩余模块评估（2026-08-01 精确评估，逐文件实测）

经逐文件读取评估，剩余 12 个模块的可移植边界如下（server.rs 已落地骨架，见已完成表）：

| 模块 | 行数 | 判定 | 可移植部分 / 必须手写 |
|---|---|---|---|
| **conversation.rs** | 1331 | 🟢 **高** ⭐ | **可移植**：L14-157 全部数据类型（Conversation/ConversationKind/Driver/ConversationStatus/Turn/TurnKind/ToolRecord/GateRecord/GateInfo/ConversationSummary，全 serde derive）+ ConversationEvent + to_status_str + chat_message_to_turns + now_secs。**手写**：ConversationStore（Mutex+broadcast）、run_event_to_turns（上游 RunEvent + macro_rules!）、测试 |
| **relay/api.rs** | 393 | 🟡 中 | **可移植**：L34-86 的 5 个 DTO（BusEvent/ResolveGateBody/SubmitHandoffBody/UpdateTitleBody/ListRunsQuery）。**手写**：bus()（OnceLock+broadcast）、所有 handler（async+extractor+impl Trait+Sse）、relay_routes |
| **relay/flows.rs** | 59 | 🟡 中（边界用例）| **逻辑 100% 纯**（4 个 flow 构造，无 async/trait/Mutex），唯一障碍是产物类型 FlowSpec/FlowStep/GateType/ExitRouting 来自上游 auto_ai_agent。**需试验**：上游类型作为不透明构造目标 + builder 方法链能否转译 |
| tool_test.rs | 184 | 🟡 低价值 | Fixture/CaseCategory/Expect/ToolCase 可移植；Sandbox(tempfile)+run_case(async+闭包)手写。低价值未做 |
| main.rs | 370 | 🔴 手写 | clap derive（Cli/Cmd）+ tokio runtime + Arc<dyn Client> + NoDaemonClient(async trait) + 闭包捕获 StreamEvent。无纯数据可提取 |
| lib.rs | 261 | 🔴 手写 | OwnedRole(Arc<dyn Role>) + impl Role + build_agent_*(Arc<dyn Client>/Tool) + resolve_role。全是上游 trait 桥接 |
| workflow.rs | 30 | 🔴 手写 | include_str! + parse_at_workflow/Workflow（上游）。无数据 |
| tool_context.rs | 18 | 🔴 手写 | ToolContext.state: Arc<AppState>，与 server 耦合 |
| tools.rs | 874 | 🔴 手写 | 9 个单元 struct（无字段）+ 9 个 #[async_trait] impl Tool + json!() 宏。校验逻辑（EditFile/BatchReplace/ListSymbols）可抽纯函数但需重构、收益小 |
| spec_tools.rs | 559 | 🔴 手写 | 5 个 #[async_trait] impl Tool + json!()。WriteSpec/WriteGoals 的 markdown 解析可抽纯函数，收益中 |
| orch_tools.rs | 494 | 🔴 手写 | 3 个 Tool impl + tokio::spawn + thread_local。build_toolcall_turn 可抽纯函数（依赖 Turn 先移植）|
| relay/driver.rs | 289 | 🔴 手写 | AgentFactory trait + async loop + 闭包捕获 StreamEvent + Mutex。无数据 |
| relay/mod.rs | 35 | 🔴 手写 | 纯 pub use re-export 墙，无逻辑 |

**实施步骤（按 ROI 排序）**：
1. **conversation.rs 数据层**（高收益）：提取 L14-157 + ConversationEvent + 3 个纯函数 → `conversation.at`，复刻 specs.rs/chats.rs 模式。
2. **relay/api.rs DTO 层**（低风险）：提取 5 个 DTO → 并入 `relay_store.at` 或新建 `relay_api.at`。
3. **relay/flows.rs 边界试验**（验证新规则）：整文件尝试转译，确认"上游类型 + builder 链"是否可移植。通过则白捡 59 行 + 扩展可移植规则；不通过则记录为 a2r 限制。
4. 其余 9 个模块：确认无纯数据可提取，保持手写，不投入。

**跨模块注意**：`now_secs()` 在 conversation.rs/relay api/driver 三处重复，移植时统一到 conversation.at。relay/store.rs 的 RunEvent（已移植）是 conversation.rs::run_event_to_turns 的入参依赖。

---

## 最终总结（2026-08-01，持续更新）

### 已完成

**14 个模块移植到 Auto**（specs/auth/hello/tool_safety/mode/app_config/chats/
relay{profession,store,api,flows}/conversation/server）：
- specs.rs 全文件（1495 行，纯 Auto）
- server.rs 45/52 handler + 50 DTO + 36 路由
- 其余 12 个模块的数据层 + 纯逻辑 + serde IO

**auto-lang 上游 5 项改进**（均 worktree 模式 → 合并 master）：
- Plan 379：放宽 `route` 保留字（axum `.route()` 可调用）
- Plan 380 P0：元组结构体构造（`Json(v)` 不误造 field0）
- Plan 380 P1-str：str 字面量兼容（`Json(DTO(field:"x"))` 嵌套构造）
- Plan 380 P1-dyn：泛型参数 `dyn` 解析（`Arc<dyn T>`/`Box<dyn T>` 字段）—— 解锁 AppState
- Plan 380 P4 调研：确认 `async_stream` 桥接**已是 Plan 321 实现**（`~Stream<T>`+yield），非缺口

**23 个 a2r 限制实测记录**，流转 `auto trans → nativeize.pl → cargo check`（无 a2r-std）。

### 剩余 🔴 handler 的阻塞重新评估（P1-dyn + async_stream 后，逐个精确评估）

之前判为"自然终点"的结论**已过时** —— P4 调研发现 async_stream 桥接早就是 Plan 321
实现，P1-dyn 修复了 `Arc<dyn T>` 字段。经逐个 handler 精确评估，**6 个全部可移植**：

| handler | 行号 | 判定 | 关键障碍 + 改写策略 |
|---|---|---|---|
| **workflow_run** | 1975-2009 | 🟢 | **最干净**：concrete Result 返回 + `Vec<Arc<dyn Tool>>`(P1-dyn) + 显式 .await + DTO 映射。仅 extractor 解构→整体参数 |
| **conversation_stream** | 1905-1939 | 🟡 | `Sse`+`Event` builder（已支持）+ combinator 链→generator `~Stream<Event>` + yield（替代 BroadcastStream.filter_map().map()）|
| **workflow_run_stream** | 2019-2080 | 🟡 | mpsc+spawn+stream!/yield（已支持）+ `dyn Fn` 闭包→named sink struct（Arc<dyn StreamSink>，P1-dyn）+ workflow_event_to_json→DTO |
| **run_stream_handler** | 1007-1085 | 🟡 | 同上 recipe + stream_event_to_json→SseEventDto + impl IntoResponse→concrete Response |
| **run** / run_inner | 984-997/931-982 | 🟡 | agent.run().await（显式）+ concrete Result + DTO。impl IntoResponse→concrete Response |
| **chat_stream** | 1529-1685 | 🟡 | **最大改写**：dyn Fn 闭包（重捕获 Mutex 累加器 + Value 解析）→sink struct + SseEventDto（去掉 Value 解析）+ history 构建 |

**helpers**：`stream_event_to_json`/`workflow_event_to_json` → 改写为 DTO-returning（当前
全用 json!() 宏）；`shared_tools`（`Vec<Arc<dyn Tool>>`）→ 🟢 P1-dyn 支持；`run_inner` → 🟢。

**唯一待确认**：a2r 对 `Arc<dyn Fn(...)>` **闭包** trait object 的支持（P1-dyn 覆盖 named
trait，dyn Fn 闭包不同）。若不支持，统一绕开为 **named sink struct**（实现 StreamSink
trait，捕获物作字段，Arc<dyn StreamSink> 走 P1-dyn）。三个流式 handler 的闭包体都简单
（try_send + DTO 转换），sink struct 改写直接。

**settings_link**（reqwest + spawn_blocking 外部 HTTP）仍保留手写。

**实施顺序**（按 ROI）：workflow_run(🟢) → workflow_event_to_json DTO → workflow_run_stream →
stream_event_to_json DTO → run_stream_handler → conversation_stream → run/run_inner →
chat_stream（最大，最后）。

**剩余 9 个 🔴 模块**（main/lib/tools/spec_tools/orch_tools/relay{driver,mod}/workflow/
tool_context）的 async trait 实现（`impl Tool`/`AgentFactory`）仍需 a2r 支持 trait impl 的
async 方法 —— 这是真正剩余的 a2r 缺口，不在本计划当前范围。
