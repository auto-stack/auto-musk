# Known Debt & Risks

> 复审（plan-archiver Step 2.5）登记的 workaround / 一致性遗漏 / 已知限制 / 未来增强。
> 每行一项，按严重度分级。新增条目请在对应分级表追加。

严重度图例：
- 🔴 **高风险** — 特定（非平凡）条件下可能引发 UB / 数据损坏
- 🟡 **一致性遗漏** — 功能正确，但代码未达计划自身的一致性目标
- 🟢 **已知限制** — 设计决策（非 bug），值得记录
- 📋 **未来增强** — 留待后续计划的优化 / 清理

---

## 🟡 一致性遗漏

_无。_（2026-08-06 复审清理后，此前登记的两项已全部解决：
`save_from_run` 补齐 `self.save` 持久化 + parity 测试；`auto-src/extern_impl.rs`
过时副本删除。）

## 🟢 已知限制

| Plan | 描述 | 参考 |
|---|---|---|
| 018 | `wiki.at` 三处设计内简化（已在 §10 文档化）：(1) `load()` 用 `List` + `find_meta` 线性查找替代 `HashMap.get()`（a2r 借用规则对 `&str` 键不可靠）；(2) TreeNode file 节点 `size/modified = None`（a2r Auto int 模型把 `.len()` 无条件转 `as i32`，无法喂 `Option<u64>`）；(3) `WikiSource` 不 derive(Default)（a2r 丢弃 `#[default]`）。 | `auto-src/wiki.at` |
| 018 | `task_plan.at` C1-C3 变通（文档化）：(1) `impl TryFrom<Node>` trait impl → `static fn from_node`（a2r 无 trait impl 语法）；(2) `graph.get()` 返回 `Option<&Vec>` 不标注类型直接匹配；(3) `split` 结果不标注（a2r 强制 `Vec<String>`，实为 `Vec<&str>`）。 | `auto-src/task_plan.at` |
| 018 | `app_config` 的 `AAID_URL` env 覆盖在 a2r 产物中缺失（§9 B 类手写边界）：hw `src/app_config.rs` 用 `std::env::var("AAID_URL").ok()` 覆盖 `daemon_url`；ag 版 `extern_impl.rs` 硬编码常量。a2r 无法表达 `env::var(...).ok()`。`parity_app_config.rs::parity_effective_daemon_url` 固定此分歧行为。 | `src/app_config.rs:153` / `src/auto_generated/extern_impl.rs` |
| 018 | `Result<(), String>`（unit 类型）a2r 不可表达 → specs/handoff_store 的写方法用 `Result<bool, String>` 载荷承载成功语义（`Ok(true)`）。跨模块一致约定。 | `auto-src/specs.at` / `auto-src/handoff_store.at` |
| 018 | a2r 输出验证器对 specs.at 报 `unbalanced parentheses (depth: 1)` 警告（编译通过、测试通过，疑为字符串字面量内括号被误判）。非阻断，记为待查。 | `auto-src/specs.a2r.rs` 转译输出 |

## 📋 未来增强

| Plan | 描述 | 参考 |
|---|---|---|
| 018 | **休眠镜像 full parity**：`tools.rs`/`spec_tools`/`orch_tools`/`server_serve`/`server_stream`/`relay_driver` 等 ag 镜像为简化 dormant（description/schema 文本与 hw 有差异 + execute 依赖 extern stub）。计划 §10.6 Phase 4 评估 + §13 C 类已文档化为"设计内的等价镜像，非缺陷"。full parity 需 `.view` 手术 + 元数据对齐，留待后续接线计划。 | `src/auto_generated/{tools,spec_tools,orch_tools,server_serve}.rs` |
| 018 | **HTTP 层测试缺口**（§13 E1）：`/api/run*`、`/api/settings-link`、`/api/chats/.../stream`、`/api/conversations/.../stream`、`/api/files/*`、全部 `/api/forge/*` 无 HTTP 层测试（🔴 手写 handler + forge 独立）。 | — |
