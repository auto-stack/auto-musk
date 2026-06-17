# 002 — auto-forge AI 能力三成分拆分分析

> **目的**：为"把 auto-forge 拆成 auto-ai-daemon / auto-ai-client / app 三成分"提供事实依据和划分方案。
> **背景**：auto-musk（auto-forge 的 Auto 化）延期。auto-forge 的 LLM 相关能力应下沉为 AutoOS 全局资源，支撑未来多 AI App 生态。
> **依据**：对 auto-forge backend 源码的逐模块分析（带文件路径:行号证据）。
> **决策**：auto-forge 先在 Rust 层完成三成分拆分，再考虑翻译成 Auto（auto-musk）。

---

## 1. 架构愿景

```
┌─────────────────────────────────────────────────────┐
│  auto-ai-daemon（系统服务，独立进程）                │
│  - ApiSource 全生命周期（CRUD/scan/凭证/test）       │
│  - 统一 LLM 调用（chat_turn/SSE 解析/连接池/调度）   │
│  - 接通 ApiSource → 真实请求（当前断裂！）           │
│  - （可选）daemon 级 MCP（资源/调度，非 forge_*）    │
└────────────▲──────────────────────────┬────────────┘
             │ HTTP/SSE                │
┌────────────┴──────────┐  ┌──────────▼──────────────┐
│ auto-ai-client（库）  │  │ auto-ai-client（库）    │
│ chat_turn(req,src,mdl)│  │ （同左，各 app 复用）   │
└────────────▲──────────┘  └──────────▲──────────────┘
             │                        │
┌────────────┴────────────────────────┴──────────────┐
│  app：auto-musk / 其它 AI App                       │
│  - Forge 聊天、Relay 编排、Spec Ledger、工具系统    │
│  - Profession/Soul/AgentConfig（api_source_id 外键）│
│  - app 级 MCP（暴露 app 业务）                      │
└─────────────────────────────────────────────────────┘
```

---

## 2. 最关键发现：ApiSource ↔ ClaudeProvider 当前断裂

**事实**：auto-forge 的"AI 资源管理"和"LLM 调用"是两条互不相连的线。

- `ClaudeProvider`（`backend/src/provider/claude.rs:71-95`）构造时只读 `~/.claude/settings.json` 或环境变量（`claude.rs:35-66`），**不读 `api_sources.json`**。
- 硬编码模型 `claude-3-5-sonnet-20241022`（`claude.rs:14,156`）。
- `AgentConfig.api_source_id`/`model_id`（`relay/config.rs:372-403`）是**写了但没生效**的死代码——`ToolChatRequest`（`turn.rs:182`）根本不含 model 字段。
- 唯一真正读 ApiSource 发请求的是 `do_test_connection`（`relay/api.rs:715-818`，每次临时构造）。

**拆分的地基**：daemon 必须先在内部接通"按 source_id 解析 key/base_url/model → 真实发请求"，否则 client→daemon 无法路由到不同 provider。`do_test_connection`（`api.rs:746-780`）里已有 OpenAI/Local 协议的参考实现，可直接提炼进 daemon 调用路径。

---

## 3. 三成分划分（逐模块，基于代码证据）

### daemon（系统级 AI 资源 + 调用）

| auto-forge 模块 | 位置 | 说明 | 拆分难度 |
|---|---|---|---|
| LLM HTTP 调用（chat_turn/stream/chat） | `provider/claude.rs:137-474` | 唯一真实调用路径 | 中（需接通 ApiSource + 补 OpenAI/Local）|
| SSE 解析 | `provider/sse.rs` | SseParser，独立可移植 | 低 |
| 调用类型 | `provider/types.rs`（43KB） | StreamEvent/ContentBlockDelta 等 | 低（types 部分与 client 共享）|
| ApiSource 数据 + CRUD | `relay/config.rs:17-361` | 数据结构 + 持久化 + scan + 凭证 | 低 |
| api-sources 端点 | `relay/api.rs:611-818, 1613-1617` | 资源 REST | 低 |
| test-connection | `relay/api.rs:715-818` | 含多 provider 协议参考实现 | 低 |
| Provider 枚举 | `relay/agent.rs:82-88` | 当前归属错位（在 agent.rs），应随 daemon | 低 |

### client（调用库）

| 内容 | 说明 |
|---|---|
| HTTP/SSE 客户端 | 调 daemon 的 REST/SSE（非进程内 link——否则等于没拆）|
| `chat_turn(request, source_id, model_id) -> Stream<ToolChatEvent>` | 替换当前 5 处直接调用 |
| ModelConfig DTO | `relay/agent.rs:13-80`，调用参数 |
| ToolChatEvent 桥接 | `provider/types.rs` 的事件类型映射为 SSE 流 |

### app（auto-musk 业务）

| auto-forge 模块 | 位置 | 说明 | 拆分难度 |
|---|---|---|---|
| Forge 聊天循环 + REST | `forge/mod.rs`（含 `:2693` chat_turn 调用点） | 核心 | 中（改走 client）|
| Relay agent 接力（AgentTurn）| `relay/turn.rs:199` chat_turn 调用点 | | 中 |
| Relay pipeline/flow/driver | `relay/pipeline.rs`/`flow.rs`/`driver.rs` | 编排 | 中 |
| TaskPlan 引擎 | `relay/task_plan_engine.rs:524` chat_turn 调用点 | 多 relay 编排 | 中 |
| Errand（轻量 agent）| `forge/errand.rs:160` chat_turn 调用点 | | 低 |
| Profession | `relay/profession.rs`（依赖 forge::SectionType） | app 业务定义 | 低 |
| Soul + souls/*.md | `relay/soul.rs` + `relay/souls/` | 人格，app 自定义 | 低 |
| AgentConfig | `relay/config.rs:372-` | 留 app，api_source_id 是对 daemon 外键 | 低-中 |
| 23 个工具 + ToolRegistry | `forge/tools.rs` | 编码/spec/wiki 工具 | 低 |
| 30 个 MCP（forge_*）| `mcp/mod.rs`（`:332` chat_turn 调用点） | 暴露 app 业务 | 低（forge_send_message 改走 client）|
| runtime（context/session）| `runtime/` | Forge session | 低 |

### 待定/横切

| 模块 | 倾向 | 说明 |
|---|---|---|
| rbac（SQLite+JWT）| 横切/独立 | web 鉴权，与 AI 无关；暂留 app，未来可能独立 auto-auth |

---

## 4. 必须处理的耦合点（拆分硬依赖）

| 耦合点 | 现状 | 拆分处理 |
|---|---|---|
| **5 处直接 `.chat_turn`/`.chat`** | forge/mod.rs:2693、turn.rs:199、errand.rs:160、task_plan_engine.rs:524、mcp/mod.rs:332 | 全部替换为 client 接口 |
| **ApiSource↔Provider 断裂** | ClaudeProvider 读 settings.json 不读 ApiSource | daemon 内接通（按 source_id 解析 key/url/model）|
| **ToolChatRequest 不含 model** | turn.rs:182 构造时无 model 字段 | client 协议带 model/source_id |
| **Provider 枚归属错位** | 在 relay/agent.rs，被 daemon 资源引用 | 随 daemon 迁移 |
| **AppState.ai_provider 单例** | main.rs:20-28 全局共享 | client 句柄替换 |
| **MCP server 持 ai_provider** | mcp/mod.rs:62 | forge_send_message 改走 client |

---

## 5. daemon 最小职责

1. **ApiSource 全生命周期**：CRUD + scan/import + 凭证解析 + test-connection（config.rs + api.rs:715-818 现成）。
2. **统一 LLM 调用**：收编 claude.rs 的 chat_turn/stream，**接通 ApiSource**（按 source_id+model_id 解析 key/url，替代硬编码 settings.json）。
3. **连接池/调度**：warm_up、pool_max_idle_per_host 等系统关注点。
4. 可选：daemon 级 MCP（资源/调度工具）。

## 6. client 形态结论：HTTP/SSE 库

基于代码现状：
- 当前 5 个调用点（forge/relay/errand/task_plan/mcp）各自构造 ToolChatRequest + 消费 ToolChatEvent 流——**正是 HTTP+SSE 客户端形态**。
- ToolChatEvent（TextDelta/ToolUse/ThinkingDelta/Usage/Done）天然映射 SSE 事件流。
- 做成进程内 link = 没拆（app 仍直接持 provider 句柄，绕过 daemon 资源管理）。
- client 核心接口 ≈ `chat_turn(request, source_id, model_id) -> Stream<ToolChatEvent>`，逐点替换现有 5 处调用。

---

## 7. 待决策项（制定拆分计划前需对齐）

1. **实现次序**：先 daemon（地基优先）/ 先 client（接口优先）/ daemon+client 同时。
2. **跨进程边界**：daemon 独立进程 + HTTP/SSE / sidecar 常驻 / 先拆代码层边界后定。
3. **rbac 归属**：留 app / 归 daemon / 独立横切层。
4. **daemon 是否暴露 MCP**：当前 30 个 forge_* 留 app；daemon 级 MCP（资源/调度工具）是否做、何时做。
5. **拆分在哪个仓库进行**：在 auto-forge 原地拆（Rust）/ 新建独立仓库 / 直接在 auto-musk 用 Auto 重写这三成分。

---

## 8. 证据索引

- LLM 调用：`D:\autostack\auto-forge\backend\src\provider\claude.rs`、`sse.rs`、`types.rs`
- ApiSource：`D:\autostack\auto-forge\backend\src\relay\config.rs:17-361`、`relay\api.rs:611-818`
- Profession/Soul/AgentConfig：`relay\profession.rs`、`relay\soul.rs`、`relay\config.rs:372-`
- Forge/Relay 业务：`forge\mod.rs`、`relay\turn.rs`、`relay\pipeline.rs`
- MCP：`mcp\mod.rs`
- 工具：`forge\tools.rs`
- 断裂证据：`claude.rs:35-66`（读 settings.json）、`turn.rs:182`（ToolChatRequest 无 model）
