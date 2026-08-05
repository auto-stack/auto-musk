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

| Plan | 描述 | 参考 |
|---|---|---|
| 018 | `auto_generated::handoff_store::save_from_run` **只 load 不 save**：hw 版（`src/relay/handoff_store.rs:126-139`）先 `last_handoff` 再 `self.save(...)` 落盘 + `tracing::warn!` 日志；ag 版（`auto-src/handoff_store.at:146-152`）只 `last_handoff` 直接返回，**从未持久化**。函数名误导。计划 §10.6 的"已知简化"小节未记录此行为差异，且 7/7 parity 测试未覆盖 `save_from_run`。**当前安全**：`auto_generated::handoff_store` 零 live 引用（活路径用 `crate::relay::handoff_store::HandoffStore`，见 `src/workspace.rs:45,418`），属休眠镜像；若未来接线 ag 版 handoff store 会导致 handoff 不落盘。 | `auto-src/handoff_store.at:146` / `src/auto_generated/handoff_store.rs:134` |
| 018 | `auto-src/extern_impl.rs` **规范源严重漂移**：停在 plan 014（`ed6d9e8`），仍是 fake stub（`new_id(_nbytes)` 忽略参数、`path_inner` 返回空串、`specs_drift`→`Null` 等）；生效产物 `src/auto_generated/extern_impl.rs` 已修复全部 A 类 bug（`eaf2c62`）。两文件 `diff` 大量不同。**根因**：extern_impl 是计划 §11 明确的"手写桥接层"（非 a2r 产物），规范源角色模糊。**当前安全**：编译取 `auto_generated/` 版，`auto-src/extern_impl.rs` 不参与编译；但作为"规范源"会误导后续维护者。 | `auto-src/extern_impl.rs` vs `src/auto_generated/extern_impl.rs` |

## 🟢 已知限制

| Plan | 描述 | 参考 |
|---|---|---|
| 018 | `specs.at::rebuild_relations` 仍用 `List<ReverseEntry>` 线性查找变通（a2r-10 复合泛型 HashMap 限制 + a2r-11 整体重赋值）。**该变通已可消除**：`task_plan.at::detect_cycle`（行 182-184）实证 `HashMap<String, Vec<String>>` 的 `insert`/`get` 已可用（§4 C3 闭环）。specs 未跟进去变通属历史遗留。行为等价（parity_specs 12/12 通过），去变通为纯优化。 | `auto-src/specs.at:420-488` |
| 018 | `wiki.at` 三处设计内简化（已在 §10 文档化）：(1) `load()` 用 `List` + `find_meta` 线性查找替代 `HashMap.get()`（a2r 借用规则对 `&str` 键不可靠）；(2) TreeNode file 节点 `size/modified = None`（a2r Auto int 模型把 `.len()` 无条件转 `as i32`，无法喂 `Option<u64>`）；(3) `WikiSource` 不 derive(Default)（a2r 丢弃 `#[default]`）。 | `auto-src/wiki.at:131,268,390` |
| 018 | `task_plan.at` C1-C3 变通（文档化）：(1) `impl TryFrom<Node>` trait impl → `static fn from_node`（a2r 无 trait impl 语法）；(2) `graph.get()` 返回 `Option<&Vec>` 不标注类型直接匹配；(3) `split` 结果不标注（a2r 强制 `Vec<String>`，实为 `Vec<&str>`）。 | `auto-src/task_plan.at:267,504,610` |
| 018 | `conversation.at::now_secs` 用 `SystemTime.now().elapsed()`（返回自系统启动≈0）而非 hw 的 `duration_since(UNIX_EPOCH)`。计划 §9 称"a2r 无法表达 UNIX_EPOCH"，但 `wiki.at:91` 已成功转译 `SystemTime.UNIX_EPOCH` 点号访问形式 —— 该绝对表述已过时，conversation.at 的写法更像历史残留。**当前无影响**：该函数私有且未被调用。 | `auto-src/conversation.at:16-21` |
| 018 | `app_config` 的 `AAID_URL` env 覆盖在 a2r 产物中缺失（§9 B 类手写边界）：hw `src/app_config.rs:150-156` 用 `std::env::var("AAID_URL").ok()` 覆盖 `daemon_url`；ag 版 `extern_impl.rs:513` 硬编码常量。a2r 无法表达 `env::var(...).ok()`（`Expected Asn, but found .`）。`parity_app_config.rs::parity_effective_daemon_url` 固定此分歧行为。 | `src/app_config.rs:153` / `src/auto_generated/extern_impl.rs:513` |
| 018 | `Result<(), String>`（unit 类型）a2r 不可表达 → specs/handoff_store 的写方法用 `Result<bool, String>` 载荷承载成功语义（`Ok(true)`）。跨模块一致约定。 | `auto-src/specs.at` / `auto-src/handoff_store.at:57` |

## 📋 未来增强

| Plan | 描述 | 参考 |
|---|---|---|
| 018 | **休眠镜像 full parity**：`tools.rs`/`spec_tools`/`orch_tools`/`server_serve`/`server_stream`/`relay_driver` 等 ag 镜像为简化 dormant（description/schema 文本与 hw 有差异 + execute 依赖 extern stub）。计划 §10.6 Phase 4 评估 + §13 C 类已文档化为"设计内的等价镜像，非缺陷"。full parity 需 `.view` 手术 + 元数据对齐，留待后续接线计划。 | `src/auto_generated/{tools,spec_tools,orch_tools,server_serve}.rs` |
| 018 | **文档数字出入**（非阻塞）：§11 与 commit 750aeb1 声称"补 44 处 `.view`"，实际当前 `server.at` 共 42 处 `.view`，750aeb1 本身仅新增 9 处。不影响 re-transpile drift=0 的实质结论。 | `auto-src/server.at` |
| 018 | **HTTP 层测试缺口**（§13 E1）：`/api/run*`、`/api/settings-link`、`/api/chats/.../stream`、`/api/conversations/.../stream`、`/api/files/*`、全部 `/api/forge/*` 无 HTTP 层测试（🔴 手写 handler + forge 独立）。 | — |
