# 020 — 全后端 Auto 化：Relay/Forge + TaskPlan + Wiki + settings_link 接入 Auto 驱动

> **状态**：✅ 完成（2026-08-06 启动；Phase A/B/C/D/E/F 全闭环）。
> **前置**：Plan 018（已归档，parity 全闭环）+ Plan 019（已归档，6 个 🔴 流式/daemon handler 接线）。
> **仓库**：auto-musk（`backend/crates/musk/`）+ auto-lang（a2r 转译器，构建 `auto.exe`）。
> **目标**：把**剩余未 Auto 化**的后端子系统（Relay/Forge 编排、TaskPlan、Wiki、settings_link）全部移植到 Auto(.at)，经 a2r 生成 Rust 编译运行；serve() 所有业务端点切到 ag handler；Auto 版行为与手写 Rust 原版一致（parity + HTTP 等价测试）。

---

## 0. 实施日志（2026-08-06）

| 阶段 | 内容 | 提交 | 验收 |
|---|---|---|---|
| **Phase A1** | `feature_dev.at` 全移植（纯逻辑 + drive loop，PipelineEngine/Agent 直驱，3 个 extern） | `2630980` | parity_feature_dev 9 项绿（含 run()/run_with_emit 端到端行为等价） |
| **Phase A2** | `relay_store.at` 补全（RunMetadata 11 字段 + skip_serializing_if 修 wire 分歧）；relay_driver 既有 parity 保持 | `ba4f1ac` | parity_relay_store 7 项绿 |
| **Phase B1** | `task_plan_engine.at` 全移植（类型 + 迭代 DFS topological_order + execute 全 Auto 化，executor 用 pub spec TaskPlanExecutor trait） | `685e49a` | parity_task_plan_engine 6 项绿（serial/parallel/failure/input_from 端到端） |
| **Phase B2** | `task_plan_parser.at` + `task_plan_registry.at` 移植；**修 task_plan.at 既有 parity 漏洞**（require_string_prop 缺失/类型错应报错而非返回 ""） | `eaf5f81` | parity_task_plan_parser_registry 9 项绿；既有 parity_task_plan 17 项保持 |
| **Phase C** | `wiki.at` WRITE 路径补全（create/update/delete/save_manifest）；嵌套臂内 guard 死锁 → cache_insert 独立锁 helper | `d0849fc` | parity_wiki_write 5 项绿；既有 parity_wiki 11 项保持 |
| **Phase D1** | `relay_api.at` 全移植：13 个 relay handler（list/start/get/delete/title/advance/handoff/gate/rerun/events-SSE/professions/souls/flows）+ 6 个 task_plan handler（list/get/create/delete/start/events-SSE）+ 2 router | ⏳ 本次 | parity_relay_api 4 项绿（hw vs ag 双 router 对照：状态码 + wire 形状；含 SSE content-type、400/404 文本 body、multipart 无） |
| **Phase D2** | `wiki.at` HTTP 层补全：12 个 handler（tree/raw_tree/pages/page CRUD/search/upload/file/delete/mkdir）+ `wiki_routes()` router；tree 构建走 .at 内 build_tree + strip_md_extensions | ⏳ 本次 | parity_wiki_http 3 项绿（wiki CRUD + raw 文件系统 + multipart upload 逐键等价；modified mtime 分歧已在 parity_wiki 文档化） |
| **Phase E** | `settings_link` Auto 化：`server_stream.at` 加 settings_link handler（~Response + 错误包络）；extern `settings_link_do` 封装 reqwest::blocking（spawn_blocking + Client.post + json 解析） | ⏳ 本次 | parity_settings_link 1 项绿（shape 契约：200↔running+url / 500↔error+message） |
| **Phase F** | serve() 接线：relay/task_plan/wiki 合并 + settings_link 路由全切 ag handler；hw settings_link 删除；`wf_run`/`wf_run_with_progress`/`workflow_exists` extern 从委托 hw feature_dev 改调 ag 版（引擎全链路 Auto）；生产 router 组合测试同步 | ⏳ 本次 | 全量 403 测试绿（24 个测试二进制 0 失败）；4 个改动模块 re-transpile 零 drift |

**累计**：403 测试全绿（lib 228 + 集成 175）；6 个 .at 模块改动 re-transpile 零 drift。

**Phase D/E/F 新习得 a2r 约束**（在既有清单基础上追加）：
- `~Response` 需 `use.rust axum::response::Response` 才会发射具体 `Response` 类型；缺省时 a2r 发射 `impl Response`（axum 不接受，编译失败）。
- a2r 对 Json body 的 Option 字段 `is body.field` match 不自动 clone（跨 Deref move E0507）——需显式 `.clone()`。
- `text_response` 参数：a2r 对字符串字面拼接注入 `.as_str()`，对 fn 调用返回 String 直接传值 → extern 用 `impl Into<String>` 兼容两类调用点。
- a2r `use.rust` 不支持 `as` 别名；`std::path::Path` 与 `axum::extract::Path` 重名时需在数据层放弃 `std::path::Path` 导入（该类型未被使用）。
- spec trait 无 supertrait 语法（无法声明 Send+Sync）→ `Arc<dyn TaskPlanExecutor>` 非 Send，不能直接 tokio::spawn；用独立线程 + current-thread runtime block_on（future 不跨线程）。
- hw relay/api.rs 的 400/404 是**纯文本 body**（`(StatusCode, String)`），ag 需 `text_response` 而非 `err_response`（JSON）。
- `relay_store.at` StartRunRequest.steps 缺 `#[serde(default)]`（hw 有）→ 缺省请求体 422；补 default 对齐。
- 服务真实环境有用户 config.at 的 daemon_url 优先于 `AAID_URL` → settings_link 测试用 shape 契约而非精确 URL。

**既有 a2r 约束清单**（Phase A/B/C 习得，保持）：
- Auto 关键字避让：`spec`/`dep`/`task`/`var` 不能作变量/参数名（字段名可）。
- `is` 臂的 wildcard 绑定（`other ->`）只能用于表达式臂；block 臂内调用绑定值失败。
- 嵌套在 is 臂内的 `is guard.get` 不发射 `drop(guard)` → 同 Mutex 二次 lock 死锁（抽独立锁作用域 helper）。
- 循环变量直接作"嵌套调用实参的字面量字段"→ 类型推断错位（预绑定 local）。
- 局部 fn 的 `str` 参数生成 `&str`；跨模块调用需 `.view`（`&x`），extern `@T`/`@str` 参数**不要**再写 `.view`（会双引用）。
- `~Result<T, str>` 返回 → 自动 async fn；对 `~Result` 局部 fn 调用自动加 `.await`（勿手写双 await）；spec trait 方法调用需手写 `.await`。
- a2r 类型表把 auto-ai-agent 枚举建模为 tuple 变体，运行时 rust-ref 是 struct 变体 → nativeize 模式重写（(3b) AdvanceResult）。

**残留 hw（本计划范围外，同 Plan 019 理由）**：静态文件服务 / CORS / TcpListener / `axum::serve` 外壳、`workspace_file`（纯文件 I/O 服务端点）。serve() 业务端点 100% 由 ag handler 服务。

---

## 0. 为什么需要本计划

Plan 018 + 019 完成后，Auto 驱动的是 **Musk 主应用 API**（auth/specs/chats/conversations/workspace/config/app-config + run/workflow 非流式与全部流式 handler，约 43 个端点）。但盘点发现**后端并未 100% Auto 化**：

- **Relay/Forge 编排**（`/api/forge/relay/*`、`task-plans/*`、`events`，~14 端点）——手写 `relay/api.rs` 585 行，ag 镜像只有 DTO 骨架。
- **Wiki**（`/api/forge/wiki/*`、`/api/forge/raw/*`，4 端点）——手写 `wiki.rs` 847 行，ag 镜像 429 行、未接线。
- **settings_link**（`/api/settings-link`）——手写（reqwest::blocking），.at 未移植。
- **Relay/TaskPlan 引擎层**——`feature_dev.rs`(409) / `task_plan_engine.rs`(672) / `task_plan_parser.rs`(137) / `task_plan_registry.rs`(306) **无 .at 镜像**；`relay_driver.rs`(297) / `relay_store.rs`(1098) 的 ag 镜像仅 38/157 行骨架。

这些是后端的一部分；`serve()` 目前由 ag 主 router + 手写 relay/wiki/settings_link/文件服务拼装而成。**本计划把它们全部纳入 Auto 驱动**，实现用户目标："整个 auto-musk 后端 Auto 化，a2r 生成的 Rust 工程能编译运行，行为与 Rust 原版一致"。

---

## 1. 差距盘点（2026-08-06 实测）

| 子系统 | hw 模块（行数） | .at 镜像 | ag 产物（行数） | 现状 |
|---|---|---|---|---|
| Relay HTTP 层 | `relay/api.rs`（585） | `relay_api.at` | 38 | **只移植了 5 个 DTO**；handler（async extractor + SSE）标注为"a2r blind spots"未移植 |
| Relay workflow 引擎 | `relay/feature_dev.rs`（409） | ❌ 无 | — | 从零移植。**019 的 `/api/run`+`/api/workflow/run` 目前经 extern 委托 hw feature_dev** |
| Relay driver | `relay/driver.rs`（297） | `relay_driver.at` | 38 | 骨架（dormant） |
| Relay store | `relay/store.rs`（1098） | `relay_store.at` | 157 | 部分（RunStore 数据层） |
| Relay profession/flows | `relay/profession.rs`（494）+ `flows.rs`（59） | `relay_profession.at`/`relay_flows.at` | — | parity_relay 3 项已绿（flows/professions/build_agent） |
| TaskPlan | `task_plan.rs`（513） | `task_plan.at` | 569 | parity_task_plan 17 项已绿 ✅ |
| TaskPlan 引擎 | `task_plan_engine.rs`（672） | ❌ 无 | — | 从零移植 |
| TaskPlan parser/registry | `task_plan_parser.rs`（137）+ `task_plan_registry.rs`（306） | ❌ 无 | — | 从零移植 |
| Wiki | `wiki.rs`（847） | `wiki.at` | 429 | 部分（parity_wiki 11 项已绿）；`/api/forge/wiki/*` 未接线 |
| settings_link | `server.rs:197`（~58） | ❌ 无 | — | reqwest::blocking；extern 封装方案（见 §3.5） |

**总计**：约 **5700 行** hw Rust 待 Auto 化（relay 4804 + wiki 847 + settings_link 58）。

**明确保留 hw（不在本计划范围，理由同 Plan 019）**：
- 静态文件服务 / CORS / TcpListener / `axum::serve` 外壳（a2r 不可表达 axum 外壳）。
- `workspace_file`（`/api/files/{workspace_id}/{*path}`）——纯文件 I/O 服务端点，非业务逻辑。

---

## 2. 目标与验收标准

**用户目标**（本计划验收金标准）：
1. **整个后端 Auto 化**：所有业务 HTTP 端点由 ag handler 服务（serve() 只剩外壳 + 文件服务 hw）。
2. **a2r 产物可编译运行**：每个移植模块经 `auto.exe trans` + `nativeize.pl` 生成 Rust，零手修、编译通过。
3. **行为与 Rust 原版一致**：parity 测试（行为等价）+ HTTP 层等价测试（状态码/wire 形状）全绿。

**最终 serve() 形态**：
```
ag build_router（37 路由）
+ ag server_stream（6 handler）
+ ag relay（relay_routes + task_plan_routes + wiki_routes 全部转译版）
+ ag settings_link
+ hw：静态文件 / CORS / TcpListener / workspace_file（唯一残留外壳）
```

---

## 3. 实施阶段（每阶段独立验收，参照 018 范式）

### Phase A：Relay 引擎层 .at 移植（feature_dev + driver + store + profession + flows）

- 从零移植 `feature_dev.at`（workflow 引擎：`require_builtin`/`run`/`run_stream`/`drive`/PipelineEngine 驱动）——对齐 hw 409 行。
- 补全 `relay_driver.at`（MuskAgentFactory / build_agent）、`relay_store.at`（RunStore 数据层，对齐 1098 行）、`relay_profession.at`、`relay_flows.at`。
- **验收**：parity_relay 扩展（feature_dev 的 run/run_stream 行为等价 + store CRUD + driver build_agent）。hw `feature_dev` 单测在 ag 产物上通过。

### Phase B：TaskPlan 引擎层补 .at（engine + parser + registry）

- 从零移植 `task_plan_engine.at`（672 行）+ `task_plan_parser.at`（137）+ `task_plan_registry.at`（306）。
- `task_plan.at` 已有 parity 17 项 ✅，保持。
- **验收**：parity_task_plan 扩展覆盖 engine/parser/registry；hw 单测在 ag 产物上通过。

### Phase C：Wiki 补全

- `wiki.at` 对齐 hw `wiki.rs` 847 行（tree/search/raw 数据层）。
- **验收**：parity_wiki 扩展；hw wiki 单测在 ag 产物上通过。

### Phase D：Relay/Wiki HTTP 层 .at 移植（api handlers + routes）

- 移植 `relay_api.at` 的 handler（list_runs/start_run/advance/submit_handoff/resolve_gate/rerun/run_events SSE/list_task_plans 等）——**Plan 019 已证明 async extractor + SSE handler 可移植**（server_stream.at 先例）。
- 移植 `relay_routes`/`task_plan_routes`/`wiki_routes` 到 .at（router 组合与 server.at 的 build_router 同模式）。
- **验收**：HTTP 层等价测试覆盖 `/api/forge/relay/*` + `task-plans/*` + `/api/forge/wiki/*` + `/api/forge/raw/*`（状态码 + wire 形状，参照 019 契约测试金标准）。

### Phase E：settings_link Auto 化

- `.at` 移植 handler 骨架（`~Response` + 错误包络模式，019 先例）+ extern `settings_link_do()` 封装 `reqwest::blocking`（spawn_blocking + Client.post + json 解析，返回 Value）。
- **验收**：HTTP 等价测试（running / error 路径）+ 契约测试。

### Phase F：serve() 接线 + 全量验收

- serve() 的 relay/wiki/task_plan merge 与 settings_link 路由切到 ag handler。
- **接线迁移**：019 中 `wf_run`/`agent_run`/`wf_run_with_progress` extern 从委托 hw `feature_dev` 改为调用 ag 版（引擎 Auto 化后全链路 Auto）。
- **验收**：serve() 只剩外壳 hw；全量 lib + parity + HTTP 等价测试全绿；`auto.exe trans` 产物零 drift。

---

## 4. 关键架构决策

1. **沿用 018 范式**：`.at` 为唯一真源 → `auto.exe trans` → `nativeize.pl` → `auto_generated/*.rs`；parity 测试证明行为等价；接线经 extern 委托真实 store/engine。
2. **HTTP 层移植参照 019**：async extractor + SSE handler 已由 server_stream.at 证明可移植；DTO + `~Response` + 错误包络为既有模式。
3. **引擎层从零移植**：feature_dev / task_plan_engine / parser / registry 无 .at，需按 018 的 `impl TryFrom`→`static fn`、`.view` 借用标记等已学约束编写。
4. **settings_link 用 extern 封装 reqwest**（不要求 a2r 支持 reqwest::blocking）。
5. **外壳保留 hw**：静态文件/CORS/TcpListener/workspace_file。

---

## 5. 风险

- 🔴 **规模大**（~5700 行 hw → Auto，接近或超过 Plan 018 体量）→ 按 Phase 逐块闭环，每 Phase 独立验收。
- 🔴 **feature_dev 从零移植**（019 的 run/workflow_run 依赖它）→ 先移植 + parity，再切换 extern 委托；切换前 hw 仍是真源，可回退。
- 🟡 **a2r 新转译限制**（engine 复杂逻辑：PipelineEngine advance / 泛型驱动）→ 逐个处理，必要时开 auto-lang follow-up（018 已闭环 391/392/393 同类）。
- 🟡 **run_events SSE handler**（`Sse<impl Stream>`）→ 019 已证明可移植（server_stream.at）。
- 🟢 **parity 覆盖不足的边界**（store 持久化、并发）→ 每 Phase 的 parity 扩展 + HTTP 等价测试兜底。

---

## 6. 与 KNOWN-DEBT 的关系

本计划认领以下已登记条目：
- 🟢 **休眠镜像 full parity**（018）：`tools`/`spec_tools`/`orch_tools`/`server_serve`/`relay_driver` 等 ag 镜像为简化 dormant → **relay_driver 等纳入本计划补全**；tools/spec_tools/orch_tools/server_serve 评估后决定补全或保持休眠（非 HTTP 端点面）。
- 📋 **HTTP 层测试缺口**（018 §13 E1）：`/api/forge/*` 无 HTTP 层测试 → Phase D 验收补齐。
- 完成时移除/更新对应 KNOWN-DEBT 条目。

---

## 7. 里程碑

| 里程碑 | 内容 | 验收 |
|---|---|---|
| M1（Phase A） | Relay 引擎层 .at 全移植 | parity_relay 扩展全绿 |
| M2（Phase B+C） | TaskPlan + Wiki 引擎补全 | parity_task_plan/wiki 扩展全绿 |
| M3（Phase D） | Relay/Wiki/TaskPlan HTTP 层移植 | HTTP 等价测试全绿 |
| M4（Phase E） | settings_link Auto 化 | 契约测试全绿 |
| M5（Phase F） | serve() 全接线 | serve() 只剩外壳 hw；全量测试全绿；产物零 drift |
