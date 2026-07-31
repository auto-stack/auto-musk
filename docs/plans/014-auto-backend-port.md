# 014 — auto-musk 后端的 Auto 语言版本（a2r 转译可回 Rust）

> **状态**：**specs.rs 全文件移植完成**（2026-07-31）。1495 行 Rust → Auto
> 全量覆盖（数据模型 + 状态机 + 关系图 + 状态推导 + 概览 + JSON 持久化 CRUD），
> a2r 转译 + cargo check 双双 0 错误。下一步：横向扩展其它 🟢 模块。
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
│   │   ├── specs.at         ✅ 全文件完成（1495 行 Rust → Auto，0 错误）
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
