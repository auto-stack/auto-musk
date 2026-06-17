# auto-musk 上游依赖：auto-lang 补强任务（交付给 auto-lang）

> 本文件是 auto-musk 移植的两个前置基础的详细任务描述，供复制到 auto-lang 工程的 agent 中执行。
> 两个任务**有依赖顺序**：任务 A 先做，任务 B 后做（B 依赖 A 的 HTTP server 请求处理流程）。

---

# 任务 A：为 AutoVM 实现 `#[api]` 自动路由 + 可用的 HTTP server

## 背景与动机

Auto 语言的 `#[api(method, path)]` 注解当前**只在代码生成阶段被消费**（生成 Rust/Axum 路由或 TypeScript 客户端），AutoVM 运行时完全不认识它。我们正在用 AutoVM 脚本运行模式重写一个全栈应用（auto-musk，移植自 auto-forge），后端需要用 AutoVM 直接以脚本形式提供 HTTP API。因此需要让 `#[api]` 成为 AutoVM 的一等运行时能力：用 AutoVM 跑一个含 `#[api]` 的 `.at` 文件，能自动起 HTTP server 并把这些函数注册为路由。

## 当前状态（已查证）

1. **解析器拒绝 `#[api]`**：`crates/auto-lang/src/parser.rs` 的 `parse_fn_annotations`（约 `parser.rs:6494-6593`）的合法注解白名单里没有 `api`，遇到它落入 `_ =>` 分支返回 `SyntaxError "Unknown annotation 'api'"`（约 `parser.rs:6588-6592`）。`FnAnnotations` 结构体（约 `parser.rs:162-173`）也没有 api 相关字段。

2. **现有 `#[api]` 提取走两条非解析器路径**（仅为代码生成服务）：
   - `crates/auto-lang/src/api/mod.rs:112-178` 的 `ApiExtractor`（AST 启发式）
   - `crates/auto-man/src/api_gen.rs:789-852` 的 `extract_api_lenient`（正则 `r"#\[api\(([^]]*)\]\s*pub\s+fn..."`，正是因为完整解析器会报错才需要 lenient fallback）
   - 提取结果只在 `api_gen.rs:32-92` 的 `generate_api` 里按 backend 分发，**只有 `tauri`/`vue` 分支，没有 `vm` 分支**。

3. **VM 的 http server 是 stub**：
   - `crates/auto-lang/src/vm/ffi/stdlib.rs:1956-1962` `shim_http_server` — 只返回一个计数器 handle。
   - `shim_http_server_get/post/put/delete/static`（`stdlib.rs:1965-2016`）— pop 掉 path 和 handler 后**直接丢弃**，原样返回 server handle。
   - `shim_http_server_listen`（`stdlib.rs:2019-2065`）— **已有 tokio + TcpListener + `tokio::spawn` 每连接的异步骨架**（`stdlib.rs:2024-2057`），但**完全不解析请求、不分发路由**，对所有请求返回硬编码 `"Hello from Auto HTTP Server"`。
   - `shim_http_listen`（`stdlib.rs:3029-3033`）— 明确 stub，只打印 warn。
   - opcode 表：`crates/auto-lang/src/vm/native_catalog.rs:991-1039`（2200-2259）；注册：`stdlib.rs:3593-3643`。

4. **VM 有可用的裸 TCP 和并发能力**：`Net.tcp_bind/accept/write_str/...`（`stdlib.rs:1627-1837`）；Task 系统 `Task.spawn`（`stdlib.rs:3069-3106`）、scheduler（`crates/auto-lang/src/vm/scheduler.rs:489-505` `spawn_dynamic_task`，基于 `tokio::spawn`）。

5. **现成验收样本**：`examples/ui/015-notes/src/back/api.at` 含 5 个标准 `#[api]` CRUD 端点（GET 列表 / GET `:id` / POST / PUT `:id` / DELETE `:id`），配套 `src/back/db.at` 业务层 + `pub type Note`。这是验收基准。

## 目标

让一个含 `#[api]` 的后端 `.at` 文件可以用 AutoVM 直接运行（如 `auto run` 或 `auto xxx.at`）并自动：
1. 解析 `#[api(method, path)]`（不再报错）；
2. 把每个 `#[api]` 函数注册为 HTTP 路由；
3. 起一个 HTTP server 监听端口，把进来的请求按 method+path 分发到对应函数；
4. 自动提取路径参数（`:id`）、解析 JSON body 作为函数参数、把返回值 JSON 序列化为响应。

## 详细范围（任务清单）

1. **解析器层面**
   - 让 `#[api(method = "...", path = "...")]` 成为合法注解（去掉 `Unknown annotation 'api'` 错误）。
   - 在 AST 上保留 method 与 path 信息（可挂在 FnAnnotations 或单独的 attribute 上）。

2. **VM 路由表 + 注册**
   - 模块加载时扫描顶层 `#[api]` 函数，建立路由表 `{(method, path_pattern) -> handler 描述}`。
   - 路径 pattern 需支持 `:param` 占位符，并能匹配实际请求 path、提取参数。

3. **填充 http server native**（替换 stub）
   - `shim_http_server`：创建并存储一个 server 对象（含路由表），返回 handle。
   - `shim_http_server_get/post/put/delete`：真正把 `(path, handler)` 存进路由表（而非丢弃）。
   - `shim_http_server_listen`：复用已有 tokio+spawn 骨架，但实现真正的请求处理：解析 HTTP 请求行/头/body → 按路由表匹配 → 调用 handler → 序列化返回 → 写响应。

4. **【核心难点】VM 函数作为异步 HTTP handler 的回调机制**
   这是本任务的设计核心，需重点论证：
   - VM 是字节码解释器（有自己的 task/栈/ram），HTTP server 跑在 tokio 异步上下文里。请求到达时需要在 tokio 上下文里"重新进入 VM 执行某个函数"，拿到返回值。
   - 需要定义"VM 函数句柄/可调用引用"如何被记录、如何在请求时被调用、如何传入参数（路径参数、body 解析出的实参）、如何取回返回值。
   - 需处理阻抗不匹配：VM 执行可能是阻塞的字节码循环，tokio 是异步的——评估是用 VM 的 Task/scheduler（`scheduler.rs:spawn_dynamic_task`）spawn 一个 VM task 执行 handler，还是在独立 OS 线程跑 VM。
   - 需考虑并发安全：路由表在 listen 期间只读，但多个请求并发调用 handler 时 VM 状态的隔离/共享。

5. **参数与响应的自动序列化**
   - 路径参数 `:id` → 注入为同名函数参数（注意类型，如 `int`）。
   - 非 GET 请求的 body → 按 JSON 解析为函数参数（多个参数 → JSON 对象的字段；单个结构体参数 → 整体）。
   - 返回值 → 自动 JSON 序列化：`[]T`→数组、`?T`→`null` 或对象、结构体→对象、`bool`→布尔。错误/`None` 返回合适的 HTTP 状态码。

6. **启动方式**：明确"含 `#[api]` 的 `.at` 如何被起成 server"——是 `auto run` 自动检测 `#[api]` 后起 server，还是需要一个显式入口（如模块顶层调用 `http.server_listen`，或一个约定的 `main`）。给出推荐的 Auto 层写法，并让 `examples/ui/015-notes` 的 `src/back/api.at` 尽量**不改或最少改动**即可跑起来。

## 验收标准

- `examples/ui/015-notes/src/back/api.at` 的 5 个 `#[api]` 端点，用 AutoVM 起 server 后，`curl` 全部打通：
  - `GET /api/notes` 返回 Note 数组
  - `GET /api/notes/0` 返回单条 / `GET /api/notes/999` 返回 null/404
  - `POST /api/notes`（body `{"title":"x","body":"y"}`）创建并返回新 Note
  - `PUT /api/notes/0` 更新
  - `DELETE /api/notes/0` 删除
- `api.at` 源码尽量零改动（若必须加最小启动入口，请明确记录改了什么、为什么）。
- 提供一个最小的新示例或对 `examples/http_server/` 的补充，演示 AutoVM HTTP server + 路由的标准写法。

## 核心风险

VM 函数句柄作异步 handler 回调（任务清单第 4 项）是真正的设计难点，决定方案是否成立。开始实现前请先产出该机制的设计（数据结构 + 调用时序 + 并发模型），确认可行后再编码。

## 参考资料

- 验收样本：`D:\autostack\auto-lang\examples\ui\015-notes\src\back\api.at`、`db.at`、`pac.at`
- workspace 全栈样本：`D:\autostack\auto-lang\examples\api-example\back\api.at`
- stub 实现：`crates/auto-lang/src/vm/ffi/stdlib.rs:1956-2065`、`3029-3033`
- 路由提取参考：`crates/auto-man/src/api_gen.rs:789-852`（lenient 正则可借鉴 path 解析）
- VM Task/scheduler：`crates/auto-lang/src/vm/scheduler.rs`、`stdlib.rs:3069-3106`
- opcode 表：`crates/auto-lang/src/vm/native_catalog.rs:991-1039`

---

# 任务 B：为 AutoVM 实现 TCP flush + 服务端 SSE 推送

> **依赖任务 A**：本任务的 SSE 分帧需挂在任务 A 实现的 HTTP server 请求处理流程上（handler 内持续向连接写数据）。建议任务 A 完成后再做。

## 背景与动机

我们要移植的应用（auto-musk / auto-forge）有一个核心特性：**流式聊天**——前端发一个聊天请求，后端保持长连接，把 LLM 生成的 token 逐块用 **SSE（Server-Sent Events）** 推送给前端（对应 auto-forge 的 `/api/forge/chats/{sid}/stream`）。我们需要 AutoVM 后端能**作为 SSE 服务端**向前端推流。

**重要澄清（方向区分）**：
- **客户端方向（消费 SSE）已实现**：AutoVM 调用外部 LLM API、读取返回的 `text/event-stream`、解析 delta 事件——这套已经可用（见 `D:\autostack\auto-coder\coder\sse.at` 的增量 SSE 帧解析器；auto-lang 的 `http_stream.*`：`stdlib.rs:2506-2633`）。
- **服务端方向（生产 SSE）是缺口**：AutoVM 作为 server 向客户端持续推送 SSE 帧——当前没有实现。**本任务只补这个方向。**

## 当前状态（已查证）

1. **无 flush 原语**：`shim_net_tcp_stream_write_str`（`crates/auto-lang/src/vm/ffi/stdlib.rs:1813-1828`）用 `stream.write_all`，没有显式 flush，刷出时机不受控。全 `crates/auto-lang/src/vm/` grep `flush` 无相关 native 命中。

2. **write_str 不强制关闭连接**：`write_all` 后连接保持，理论上可多次 `write_str` 再 `close`——所以"长连接多次写"在 TCP 层面可行，缺的是**显式 flush** 和 **SSE 分帧辅助**。

3. **无 chunked transfer / keep-alive / SSE-server 辅助**：grep `sse|text/event-stream|chunked|keep_alive` 在 `examples/`、`tests/`、`test/` 下无服务端 SSE 示例或测试。

4. **可读行**：`shim_net_tcp_stream_read_line`（`stdlib.rs:1795-1809`，`BufReader::read_line`）可用于解析请求头。

5. **客户端流式参考**：`http_stream.get_stream/post_stream/stream_next`（`stdlib.rs:2506-2633`）演示了"逐块读"的模式，但那是消费方向。

## 目标

让 AutoVM 后端能对一个 HTTP 请求保持长连接，并按 SSE 协议（`Content-Type: text/event-stream`）持续推送事件帧，浏览器 `EventSource` 能实时收到。

## 详细范围（任务清单）

1. **新增 `Net.tcp_stream_flush` native**
   - 给 `TcpStream` 包 `BufWriter`（或直接调 `TcpStream::flush`），提供显式 flush 原语，让每次 SSE 帧写入后能立即推给客户端。
   - 注册到 native_catalog 与 stdlib。

2. **服务端 SSE 分帧**
   - 响应头：`Content-Type: text/event-stream`、`Cache-Control: no-cache`、`Connection: keep-alive`（或按需）。
   - 帧格式：`data: <payload>\n\n`（多行 data、`event:`/`id:` 字段按需）；末尾保持连接不 close，直到流结束。
   - 评估是提供高层 native（如 `http.response_sse` / `sse.write_event`），还是仅提供 flush 原语、由 Auto 层手写分帧。**优先提供 flush 原语 + 一个 Auto 层 helper 示例**，保持 native 层最小。

3. **与任务 A 的 HTTP server 集成**
   - 一个 `#[api]` handler 如何返回"流式"响应（而非一次性 JSON）？需定义一种方式让 handler 拿到底层连接、循环写帧 + flush、直到完成。给出推荐的 Auto 层写法（可能是特殊的返回类型、或一个 sse 响应构造器 + 回调）。

4. **并发**：SSE 长连接会占住一个 handler 较长时间，需确认不阻塞其他请求（依赖任务 A 的 per-connection spawn 模型）。

## 验收标准

- 提供一个最小 AutoVM SSE server 示例：起 server，某端点每隔 N 毫秒推一条 `data: ...` 帧。
- 用浏览器 `EventSource`（或 curl）连接，能持续收到推送的帧，连接保持不断。
- 验证 flush 生效：帧是实时到达的，而非缓冲到最后一次性发出。

## 参考资料

- write_str 实现（待加 flush）：`crates/auto-lang/src/vm/ffi/stdlib.rs:1813-1828`
- 裸 TCP 全套：`stdlib.rs:1627-1837`
- 裸 TCP HTTP server 范例：`D:\autostack\auto-lang\examples\http_server\server.at`
- 客户端流式（消费方向，已完成）：`stdlib.rs:2506-2633`
- 客户端 SSE 帧解析器（可借鉴帧格式）：`D:\autostack\auto-coder\coder\sse.at`
- 任务 A 的 server 流程（本任务集成点）：`stdlib.rs:2019-2065`（任务 A 完成后此处为真实实现）

---

## 交付顺序

1. **先做任务 A**（`#[api]` 自动路由 + HTTP server）。核心是 VM 函数句柄作异步 handler 回调机制，先出设计再编码。
2. **再做任务 B**（flush + 服务端 SSE）。基于任务 A 的 server 请求处理流程集成 SSE 推流。

两个任务完成后，auto-musk 的后端 AutoVM 运行路径才具备完整基础（HTTP API + 流式聊天），届时可回到 auto-musk 继续阶段 3（MVP 全栈骨架）。
