# 020 — 全后端 Auto 化：Relay/Forge + TaskPlan + Wiki + settings_link 接入 Auto 驱动

> **状态**：📋 规划（2026-08-06）。待实施。
> **前置**：Plan 018（已归档，parity 全闭环）+ Plan 019（已归档，6 个 🔴 流式/daemon handler 接线）。
> **仓库**：auto-musk（`backend/crates/musk/`）+ auto-lang（a2r 转译器，构建 `auto.exe`）。
> **目标**：把**剩余未 Auto 化**的后端子系统（Relay/Forge 编排、TaskPlan、Wiki、settings_link）全部移植到 Auto(.at)，经 a2r 生成 Rust 编译运行；serve() 所有业务端点切到 ag handler；Auto 版行为与手写 Rust 原版一致（parity + HTTP 等价测试）。

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
