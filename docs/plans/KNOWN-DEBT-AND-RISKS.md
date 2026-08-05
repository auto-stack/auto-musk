# Known Debt & Risks

> 复审（plan-archiver Step 2.5）登记的 workaround / 一致性遗漏 / 已知限制 / 未来增强。
> 每行一项，按严重度分级。新增条目请在对应分级表追加。
>
> **a2r 转译器限制统一登记在 auto-lang `docs/plans/391-a2r-parity-debt-from-musk.md`**；
> 本文件标注交叉引用（D1-D6）。auto-musk 侧的 .at 变通在计划391闭环后逐项去除。

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
| 018 | `wiki.at` TreeNode file 节点 `size/modified = None`：a2r 的 Auto int 模型把 `fs.metadata().len()` 无条件转 `as i32`（u64 截断 + 类型矛盾），无法喂 `Option<u64>`。**a2r 限制，见 auto-lang 计划391 D1**。tree 结构/排序/strip 是功能性面，parity 测试已断言；metadata 差异在此注释记录。 | `auto-src/wiki.at` / [391 D1](../../auto-lang/docs/plans/391-a2r-parity-debt-from-musk.md) |
| 018 | `wiki.at` `load()` 用 `List` + `find_meta` 线性查找替代 `HashMap.get()`。**注**：2026-08-06 探针证实 `HashMap.get(&str键)` 现已可编译（slug 是 `&str` 参数时 `get(slug)` 成立）——此注释的"a2r 借用不可靠"已过时，线性查找变通可去除（计划391 闭环时一并清理，或单独去变通）。 | `auto-src/wiki.at:131,269` |
| 018 | `task_plan.at` C2：`graph.get()` 返回 `Option<&Vec>`，显式标注 `Option<List<str>>` 时 a2r 缺 `&` 借用注入 + owned/borrowed 类型错配（E0308）→ 被迫不标注直接 `is` 匹配。**a2r 限制，见 auto-lang 计划391 D2**。 | `auto-src/task_plan.at:504` / [391 D2](../../auto-lang/docs/plans/391-a2r-parity-debt-from-musk.md) |
| 018 | `task_plan.at` C3：`path.split(".")` 结果 a2r 强制标注 `Vec<String>`（实为 `Vec<&str>`），类型冲突 → 被迫不标注。**a2r 限制，见 auto-lang 计划391 D3**。 | `auto-src/task_plan.at:610` / [391 D3](../../auto-lang/docs/plans/391-a2r-parity-debt-from-musk.md) |
| 018 | `task_plan.at` C1：`impl TryFrom<Node>` trait impl → `static fn from_node`（Auto 无 trait impl 语法；a2r 对 `impl Trait for Type` 误解析顺序反）。**a2r 限制 + 语言设计，见 auto-lang 计划391 D6**。parity 分别调 hw `try_from` / ag `from_node` 比行为。 | `auto-src/task_plan.at:272` / [391 D6](../../auto-lang/docs/plans/391-a2r-parity-debt-from-musk.md) |
| 018 | `app_config` 的 `AAID_URL` env 覆盖在 a2r 产物中缺失：hw `src/app_config.rs` 用 `std::env::var("AAID_URL").ok()` 覆盖 `daemon_url`；a2r parser 无法解析 `env::var(...).ok()` 方法链。**a2r 限制，见 auto-lang 计划391 D4**。`parity_app_config.rs::parity_effective_daemon_url` 固定此分歧行为。 | `src/app_config.rs:153` / [391 D4](../../auto-lang/docs/plans/391-a2r-parity-debt-from-musk.md) |
| 018 | `Result<(), String>`（unit 类型）a2r parser 无法解析（`()` 类型 + `Ok(())` 字面量）→ specs/handoff_store 的写方法用 `Result<bool, String>` 载荷承载成功语义（`Ok(true)`）。**a2r 限制，见 auto-lang 计划391 D5**。跨模块一致约定，行为等价。 | `auto-src/specs.at` / [391 D5](../../auto-lang/docs/plans/391-a2r-parity-debt-from-musk.md) |
| 018 | a2r 输出验证器对 specs.at 报 `unbalanced parentheses (depth: 1)` 警告（编译通过、测试通过，疑为字符串字面量内括号被误判）。非阻断，记为待查。 | `auto-src/specs.a2r.rs` 转译输出 |

## 📋 未来增强

| Plan | 描述 | 参考 |
|---|---|---|
| 018 | **休眠镜像 full parity**：`tools.rs`/`spec_tools`/`orch_tools`/`server_serve`/`server_stream`/`relay_driver` 等 ag 镜像为简化 dormant（description/schema 文本与 hw 有差异 + execute 依赖 extern stub）。计划 §10.6 Phase 4 评估 + §13 C 类已文档化为"设计内的等价镜像，非缺陷"。full parity 需 `.view` 手术 + 元数据对齐，留待后续接线计划。 | `src/auto_generated/{tools,spec_tools,orch_tools,server_serve}.rs` |
| 018 | **HTTP 层测试缺口**（§13 E1）：`/api/run*`、`/api/settings-link`、`/api/chats/.../stream`、`/api/conversations/.../stream`、`/api/files/*`、全部 `/api/forge/*` 无 HTTP 层测试（🔴 手写 handler + forge 独立）。 | — |
