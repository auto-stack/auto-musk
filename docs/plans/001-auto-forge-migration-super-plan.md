# 001 — auto-musk 总计划（Super Plan）

> **状态**：高层总纲。每个阶段的详细实施计划由对应的 `00x-*.md` 单独建立。
> **维护**：阶段开始前用 `writing-plans` skill 生成详细计划；阶段完成后在此文件勾选进度。
> **进度**（2026-08-04 核对）：**阶段 0-6 全部 ✅**(核心功能 100% 落地);Thinking 链路 ✅(`017`)。原 P2b.3 checkpoint 回滚与 P3b MCP 层经评估降级为按需 Backlog(理由见 `009` §11)。

---

## ⚠️ 架构演进 v2（2026-06-18，重大修订）— Rust 后端

**auto-musk 后端正式改为 Rust**,废弃原 v1 的 AutoVM 脚本后端路线。原因:

1. **auto-ai 仓库已完成三层 Rust AI 栈**(2026-06-17 ~ 06-18):
   - `ai-config` — canonical wire 类型 + 统一 ProviderConfig + auto-atom `.at` 解析 + model 校验
   - `auto-ai-client` — 精简的 daemon HTTP 客户端(canonical wire 格式)
   - `auto-ai-daemon`(`aaid`) — **唯一 LLM 出口**:所有 provider 通信 + canonical↔provider 转换 + 并发池 + usage
   - `auto-ai-agent` — Profession 库(从 Forge souls 移植)+ ReAct 循环 + Workflow 引擎
   - 详见 auto-ai 的 `ARCHITECTURE.md`

2. auto-musk 作为 **Forge 继任者**,用 Rust 后端能**直接复用** `auto-ai-agent` 这套已验证的能力,而不是在 AutoVM 里重写一遍 agent 层。Forge 进入维护模式。

3. 这绕开了 v1 的所有阻塞项(AutoVM `#[api]` panic、服务端 SSE、Plan 325/316 等)——那些是 auto-lang/AutoVM 层的问题,与 agent 应用无关。

### 新技术栈

| 层 | 技术 | 来源 |
|---|---|---|
| **前端** | Vue3 + TypeScript + shadcn-vue(a2vue 生成) | 保留 v1 已完成的前端骨架(`src/front/`、`gen/front/vue/`) |
| **后端** | **Rust**(axum HTTP server) | 新写,依赖 `auto-ai-agent` + `auto-ai-client` |
| **AI 能力** | 经 `auto-ai-daemon`(HTTP) | 复用 auto-ai,不自己实现 |
| **配置** | `.at`(auto-atom) | 复用 `ai-config` |

### auto-musk 的定位

auto-musk = **Forge 继任者**,一个完整的 AI 编码助手应用:
- 前端:Forge 的 9-tab UI(聊天/Explorer/Specs/Relay/...)
- 后端(Rust):业务逻辑 + 调 `auto-ai-agent` 的 Profession/Agent/Workflow + 暴露 HTTP API 给前端
- 不含 LLM 通信(归 daemon)、不含 provider 知识(归 daemon)、不含 canonical 类型定义(归 ai-config)

### 对 v1 阶段的影响

| v1 阶段 | 命运 |
|---|---|
| 阶段 1(VM `#[api]` + http server) | **废弃**(后端不再用 AutoVM) |
| 阶段 2(VM flush + 服务端 SSE) | **废弃**(Rust 用 axum + 已有 SSE 能力) |
| 阶段 0(骨架锚定) | **已完成**(前端骨架在),后端骨架待建(Rust workspace) |
| 阶段 3(MVP 全栈) | **重写**为 Rust 后端 + 前端打通 |
| 阶段 4(RBAC + App Shell) | 保留(前端已部分完成),后端用 Rust 实现 |
| 阶段 5(Forge 聊天流式) | 保留,后端走 `auto-ai-agent` + axum SSE |
| 阶段 6(Spec Ledger) | 保留,用 Rust 重写(纯数据,与 LLM 无关) |
| 阶段 7(Relay 引擎) | **大幅简化** —— 直接用 `auto-ai-agent` 的 Workflow 引擎,不重写 |
| 阶段 8(MCP + 配置) | 保留;ApiSource 已归 daemon,Profession 配置走 `ai-config` |

---

## v2 阶段总览(Rust 后端)

```
阶段 0  Rust 后端骨架 + 依赖 auto-ai-agent【auto-musk】
   │     (新建 Cargo workspace, axum server 骨架, 依赖 auto-ai-agent via git)
   │
   ├─→ 阶段 1  MVP:单 agent + 基础工具 + CLI【首期,详见 003-*.md】
   │            (Coder profession + 读/写/执行工具 + ReAct,CLI 接任务,
   │             经 daemon 完成。证明 auto-ai-agent 栈端到端可用)
   │
   ├─→ 阶段 2  HTTP API + 前后端打通
   │            (axum 暴露 /api/agent/run,前端 chats 页面调它)
   │
   ├─→ 阶段 3  Forge 聊天流式(SSE)
   │            (axum SSE 端点,前端流式渲染 token)
   │
   ├─→ 阶段 4  多角色 Workflow(architect→coder→tester→reviewer)
   │            (直接用 auto-ai-agent 的 Workflow 引擎,前端 relay 页面)
   │
   ├─→ 阶段 5  Spec Ledger(数据持久化)
   │
   └─→ 阶段 6  RBAC + App Shell 收尾 + MCP 评估
```

### 依赖关系
- 阶段 0 是地基(后端骨架 + 依赖接线)。
- 阶段 1(MVP)依赖阶段 0,**是首期重点**,产出可运行的 CLI agent。
- 阶段 2+ 依赖阶段 1。
- **不再依赖 auto-lang 的任何补强**(Rust 后端绕开了 AutoVM 所有阻塞)。

### auto-musk ↔ auto-ai 依赖
auto-musk 后端 Cargo.toml:
```toml
[dependencies]
auto-ai-agent = { git = "git@github.com:auto-stack/auto-ai.git", branch = "main" }
# 或本地开发用 path = "../auto-ai/crates/auto-ai-agent"
```
(开发期用 path 依赖便于联调;发布前切 git 依赖。)

---

## v2 进度跟踪

| 阶段 | 状态 | 计划文件 | 完成摘要 |
|------|------|----------|----------|
| 0 — Rust 后端骨架 | ✅ | (本文件 §阶段0) | `backend/Cargo.toml` + `crates/musk` 骨架就位（2026-06-26）。 |
| 1 — MVP 单 agent + 基础工具 | ✅ | `003-*.md`(首期，已归档) | MVP CLI + 9 工具 + 流式 chat（`003`/`004`/`005`/`006`/`008` 全部落地）。 |
| 2 — HTTP API + 前后端打通 | ✅ | `008`/`009`(已归档/活跃) | chats Web 应用 + CLI 配置 + 工具卡流式全部接通；剩余 parity（MCP）见 `009`。 |
| 3 — Forge 聊天流式 SSE | ✅ | `005`/`008`(已归档) | `main.rs`/`chats.rs` SSE 端点已落地（CLI 终端流式见 `005`，Web 见 `008`）。 |
| 4 — 多角色 Workflow | ✅ | `012`/`013`(已归档) | Relay Box（`012`）+ Conversation/Turn 数据模型（`013`）均已落地。 |
| 5 — Spec Ledger | ✅ | `009` §P0/P1a/P1b | 7 类 spec section + per-section 状态机 + 派生层 + 审批队列全部落地。 |
| 6 — RBAC + App Shell + MCP | ✅ | `010`/`011`(已归档) | UI 移植（`010`）+ 多目录 workspace（`011`）已完成；MCP 层降级为按需 Backlog（见 `009` §11）。 |
| + — Thinking 推理链路 | ✅ | `017`(已归档) | LLM reasoning 流式透传（auto-ai `993d4dc` + auto-musk `42e3a96`）。 |

---

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# 以下为 v1 历史(AutoVM 方向,已废弃,保留作参考)
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## ⚠️ 架构演进通告 v1(2026-06-17,已被 v2 取代)

> **auto-musk(Auto 化)整体延期**,原因:发现 auto-forge 的 LLM 相关能力应下沉为 AutoOS 全局资源。
> **v2(2026-06-18)更新**:这些全局能力现在已在 auto-ai 仓库以 **Rust** 实现(auto-ai-daemon/client/agent/ai-config),
> 因此 auto-musk 不再需要 AutoVM 后端,改为 Rust 后端直接复用。见本文件顶部 v2 章节。

**新的三成分架构**(v1 设想,v2 已在 auto-ai 用 Rust 落地):

| 成分 | 职责 | v2 落地 |
|---|---|---|
| **auto-ai-daemon** | 系统级 AI 资源管理 + 统一 LLM 调用 | ✅ auto-ai `auto-ai-daemon`(Rust) |
| **auto-ai-client** | AI 调用客户端库 | ✅ auto-ai `auto-ai-client`(Rust,精简) |
| **app**(auto-musk) | 应用业务 | ⬜ auto-musk(Rust 后端,v2) |

详见 `designs/002-auto-forge-ai-capability-split.md`(v1 拆分分析,仍可作为"哪些能力归哪层"的参考)。

---

## v1 原始内容(已废弃,保留备查)

### v1 目标
用 **Auto 语言**重写 auto-forge,后端用 AutoVM 脚本运行,前端用 a2vue 转 Vue3。
**v2 取消**:后端改 Rust,agent 层复用 auto-ai-agent,不重写。

### v1 大步骤总览(已废弃)
- 阶段 0 工程骨架 ✅(前端部分完成)
- 阶段 1 补强 auto-lang:VM `#[api]` ❌废弃(Rust 后端不需要)
- 阶段 2 补强 auto-lang:TCP flush + 服务端 SSE ❌废弃(axum 自带)
- 阶段 3 MVP 全栈骨架 → 重写为 Rust(v2 阶段 0+1)
- 阶段 4 RBAC + App Shell → 保留,后端 Rust
- 阶段 5 Forge 聊天 → 保留,走 auto-ai-agent
- 阶段 6 Spec Ledger → 保留,Rust 重写
- 阶段 7 Relay 引擎 → 大幅简化(用 auto-ai-agent Workflow)
- 阶段 8 MCP + 配置 → 保留,ApiSource 归 daemon

### v1 参考基线(保留)
| 项目 | 角色 | commit | 日期 |
|------|------|--------|------|
| **auto-forge** | 移植目标 | `03087fb` | 2026-06-16 |
| **auto-lang** | (v2 不再依赖其补强) | `c05482a` | 2026-06-16 |
| **auto-ai** | AI 三层栈(v2 依赖) | `main`(3b4976f+) | 2026-06-18 |
