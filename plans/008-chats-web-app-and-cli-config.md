# Plan 008:auto-musk Chats Web 应用 + CLI 配置接通

> **Status**: ✅ **已完成（2026-06-26 核实）**。Level 1（CLI 配置接通）+ Stage 2-6（chats.rs 会话存储 / chat HTTP+SSE 端点 / web/ SPA 骨架 / 聊天区 SSE 流式 / 静态服务）全部落地，6 个阶段各对应一个 commit（见 `e5c8ead`/`50a6c3c`/`a2f8b70`/`97c3116` 等）。
> **Status**: Approved
> **仓库**: `auto-musk`(主)+ `auto-ai`(agent 历史注入小改)
> **前置**: Plan 004(Roles)、auto-musk-config 运行时配置模块(已上线)、统一 Harness 设计(auto-os-config/designs/)
> **影响**: musk CLI 配置生效、musk 独立 web 应用首期(Chats)

---

## 1. 目标(两层)

### Level 1 — CLI 配置接通(轻量,先做)
让 `auto-musk-config` 的运行时配置真正影响 `musk run` / `musk chat`。当前 daemon URL 来自环境变量 `AAID_URL` / 默认值;`~/.config/autoos/apps/musk/config.at` 里写的 `daemon_url` **未被 CLI 读取**(只有 `musk serve` 的 API 读)。

### Level 2 — Chats web 应用(本计划重点,只做 Chats)
为 auto-musk 做一个**独立 web 应用**(`web/`,自带侧栏/导航,非 auto-os-config 插件),首期只实现 **Chats**:持久化多轮会话 + SSE 流式渲染 + 工具调用展示。Flows/Wikis 留后续。

---

## 2. 关键现状(调研结论)

- **CLI**:`musk run`/`musk chat` 已能跑(连 daemon + LLM + 工具),但配置文件未接通。
- **HTTP**:`/api/run` + `/api/run/stream`(SSE)已实现,**单次任务**,每次建新 agent、无记忆。
- **缺口 A**:无持久化多轮会话(无会话存储、无 `/api/chats`、HTTP 每请求都是新 agent)。
- **缺口 B**:`Agent` 无公开的"加载历史"入口(`history()` 只读)。
- **缺口 C**:Flows 后端能跑+流,但无 run 持久化(本期不做)。
- **缺口 D**:无 wiki/docs(本期不做)。
- **无现成 SPA**:`frontend/` 仅 lib-mode 配置包;`gen/` 是 a2vm 产物且 gitignored。

---

## 3. Level 1:CLI 配置接通

1. **`app_config.rs`(新模块)**:把 server.rs 的 `MuskAppConfig` 提为公共模块(`load()` 读 `~/.config/autoos/apps/musk/config.at`),server.rs 复用。
2. **`main.rs`**:`Run`/`Chat` 在 `AiClient::new()` 前,读 app-config 的 `daemon_url`;若与默认不同,`std::env::set_var("AAID_URL", ...)`(`AiClient` 经 `daemon_url()` 读环境变量)。
3. **验证**:改配置 → `musk run "列出当前目录文件"` 正常输出。

---

## 4. Level 2:Chats web 应用

### 4.1 后端:会话持久化 + chat 端点

**`backend/crates/musk/src/chats.rs`(新模块)**:
- `ChatSession { id, name, mode, messages: Vec<ChatMessage>, created_at, updated_at }`
- `ChatMessage { id, role(user/assistant/tool), content, tool_calls?, created_at }`
- `ChatStore`:JSON 持久化到 `~/.config/autoos/chats.json`(镜像 SpecsStore)。CRUD:create/list/get/rename/delete/append_message。

**端点**(server.rs,参考 auto-forge `/api/forge/chats/*`,适配单 agent):
- `POST /api/chats/session` `{mode?}`
- `GET /api/chats/sessions`(summary 列表)
- `GET /api/chats/session/{id}`(完整)
- `PATCH /api/chats/session/{id}` `{name}`
- `DELETE /api/chats/session/{id}` / `DELETE /api/chats/sessions`
- `POST /api/chats/session/{id}/message` `{content}`(存用户消息,触发 turn)
- `GET /api/chats/session/{id}/stream`(SSE,复用现有 StreamEvent→SSE)

**agent 历史注入(缺口 B)**:给 `auto-ai-agent::Agent` 加 `with_history(messages)`,使每个 HTTP chat 请求能在历史上下文中继续。跨 crate 小改,不破坏现有 92 测试。

### 4.2 前端:`web/` 独立 SPA

**目录**:`web/{index.html, src/{main.ts, App.vue, router/, views/, composables/, components/}, vite.config.ts, package.json}`

**布局**(参考 auto-forge ChatsView):
- 左:会话侧栏(新建/列表/重命名/删除,preview + 消息数)
- 右:聊天区(消息流 + 输入框)
- user/assistant 气泡;assistant 流式 markdown;工具调用内联展开;typing dots

**composables**:`useChats.ts`(CRUD + 流式,模块级单例)、`useAuth.ts`(JWT + `/api/` fetch 拦截,复用 musk auth)、路由用 vue-router 或 useViewState。

**流式渲染**:首期 markdown-it + EventSource;auto-forge 的 StreamingRenderer/markstream 较重,后续升级。

### 4.3 静态服务
`musk serve` 用 ServeDir 提供 `web/dist`;dev 模式 Vite 代理 `/api` → :8080。

---

## 5. 实施阶段(每阶段一验证一提交)

| 阶段 | 内容 | 验证 |
|---|---|---|
| 1 | Level 1:CLI 配置接通 | `musk run` 连 daemon 正常 |
| 2 | 后端 chats.rs(模型 + ChatStore + JSON 持久化) | 单测 CRUD + 持久化往返 |
| 3 | 后端 chat 端点 + `Agent::with_history`(auto-ai-agent) | curl:建会话→发消息→SSE 回 |
| 4 | 前端 web/ 骨架(SPA + router + auth + 会话侧栏) | 登录后见空会话列表 |
| 5 | 前端聊天区(消息渲染 + 输入 + SSE + 工具调用) | 浏览器多轮对话 |
| 6 | 静态服务 + Playwright 端到端 | 截图 + 断言 |

---

## 6. 范围与边界
- ✅ 只做 Chats(Flows/Wikis 留后续)
- ✅ 持久化多轮会话(JSON 文件)
- ✅ SSE 流式 + 工具调用展示
- ✅ 复用 musk auth + build_agent_from_mode
- ⏸ 不做:Flows/Wikis UI、relay/spec gate、markstream 高级渲染、会话搜索

## 7. 风险
- agent 历史注入跨 crate,需不破坏 auto-ai-agent 现有测试
- web/ 从零搭(有 gen/front/vue 作依赖参考)
- SSE 同会话多客户端并发首期不处理(假设单用户单会话)
