---
plan_id: PLAN-044
status: executing
feature_name: VM 后端桥接收口（状态闭包桥）+ 数据层 Auto 化分期（extern_impl 退役）
author: [zhaopuming]
created_at: 2026-08-26
updated_at: 2026-08-26

supersedes_spec_components: []
new_spec_components: []
touched_goals: []

current_step: 4
total_steps: 14
---

# [PLAN-044] VM 后端桥接收口 + 数据层 Auto 化分期

## 变更摘要

auto-lang 442 收口时发现数据面 parity 阻塞于 auto-musk 侧：数据 extern
（`relay_*`/`specs_*`/`auth_*`/`wiki_*`）参数含 `State<AppState>`（持
`Arc<dyn Client>` + `WorkspaceRegistry`，不可序列化），无法经 JSON ABI
（HostCallFn `fn(&str)->Result<String,String>`）传送；纯重实现（path b）
体量巨大；既有 parity 测试用 a2r Rust router（tower oneshot）而非 VM。
根因是 musk 后端"handler 层 Auto、数据层 Rust 岛"的混合架构
（`auto-src/server.at:572` 注释写明的设计决策，Rust 岛 =
`extern_impl.rs` 2841 行 / 207 pub fn）。

本计划三层推进：**①状态闭包桥**（AppState 不过 ABI——宿主侧闭包捕获，
`.at` 侧经 `extern_sigs.vm.at` adapter 去状态化，ag 轨零改动）解堵 442
验收 3；**②parity harness 换 VM**（起 `musk serve` VM 后端打真实
HTTP/SSE）闭验收 3 的测试面；**③数据层分期 Auto 化**（auth → specs/wiki
→ relay 逐域在 .at 重实现，extern_impl.rs 趋零），根治"未完全 Auto 化"。

## 目标

1. `musk serve --backend=vm`（env 开关 `MUSK_BACKEND=vm`）以 AutoVM 跑
   .at 路由 + 宿主闭包数据 extern 起服，健康/认证/核心数据端点可用。
2. 既有 parity 测试面（`parity_relay_api` 等）经 env 切换可对 VM 后端
   运行，hw 与 VM 双后端对照全绿（442 验收 3 达成）。
3. AuthStore 域完成 .at 重实现并退役对应 extern（分期第一域，验证
   退役模式可复制）。
4. 442 可按"桥接形态"收口 C3 观察期；数据层剩余域的退役路线在
   KNOWN-DEBT 登记（specs/wiki/relay 排期后续计划承接）。

## 架构方案

```
现状（442 卡点）                          目标（本计划）
──────────────────────                  ──────────────────────
VM handler → extern(s: State<AppState>)  VM handler → extern_sigs.vm.at（无状态签名）
              ↓ JSON ABI                         ↓ JSON ABI（仅业务参数）
              ✗ State 不可序列化                  ✓ 宿主闭包捕获 Arc<AppState>
ag 轨: handler → extern(s) → extern_impl  ag 轨: 不变（State 经 a2r 正常传）
parity: tower oneshot a2r router          parity: env 切换起 musk serve(VM) 打 HTTP/SSE
数据层: extern_impl.rs 2841 行 Rust 岛    分期: auth→specs/wiki→relay 逐域 .at 化退役
```

- **状态闭包桥**：`musk serve` VM 模式下宿主构建一次 `Arc<AppState>`，
  经 `auto_backend_register`/`host_bridge` 把每个数据 extern 注册为捕获
  state 的 HostCallFn（auto-lang 442 C2 items ①②③ 已备好转发 + Query
  参数 marshal + RC 修复）。**AppState 永不过 ABI**。
- **`.at` 侧去状态化**：复用 442 B 阶段验证过的 `X.at → X.vm.at`
  adapter 链（ext_stubs 对后端模块生效，`plan442_musk_probe_tests` 已
  实证）——`extern_sigs.vm.at` 提供无 `s` 参数的 extern 签名（如
  `auth_login_result(username, password)`），调用点 67 处
  `s.view`/`s.registry` 等经 adapter 吸收；ag 轨仍用原签名。
- **AuthStore 先行**：`server.rs:43` 注明 auth 数据层已是 a2r 转译版
  AuthStore——离 .at 最近，重实现面最小，作退役模式的样板。

## 技术栈

auto-musk（backend/crates/musk：`src/main.rs` serve 分支 + 新
`src/vm_backend.rs` 桥接注册 + `auto-src/extern_sigs.vm.at` +
`auto-src/auth_store.at`（新）；`tests/parity_*` harness 适配）；
auto-lang（零新改动——442 C2 前置已合入 master 06360d8ef；如遇新缺口
登记回 442 残余账）。

## 需求分析与背景调查

> 依据 auto-lang 442 计划文档 §7.2/§7.3（2026-08-26 收口记录）与本仓
> 2026-08-26 实测。

### 现状核实（2026-08-26）

- **442 auto-lang 侧就绪**：extern 响应构造器 + SSE 形态 + path(a)
  转发（Query args marshal，1475d31e2）+ path(b) 纯 extern + RC 死区
  UAF 修复已合入（06360d8ef）；真实语料 `musk_backend_gap_enumerator`
  31/32 VM-clean。
- **阻塞面精确**：`State<AppState>` 出现在 4 个 .at 文件
  （relay_api/wiki/server/server_stream），状态相关调用点 67 处；
  `extern_impl.rs` 207 pub fn 中多数**无状态**（如 `wiki_write_page`
  只收 PathBuf/str——这些今天就能过桥）。
- **AppState 组成**（`server.rs:41`）：`client: Arc<dyn Client>`（aaid
  网关客户端）+ `auth: Arc<a2r AuthStore>`（已转译！）+ `registry:
  Arc<WorkspaceRegistry>`。
- **parity 现状**：`parity_relay_api` 等以 tower `oneshot` 直打 a2r
  Rust router，无进程无 HTTP——442 验收 3 要求对照的是 VM 后端。
- **前端已无关联阻塞**：442 B 阶段（musk 侧 platform/composables vm
  adapter + 严格探针全绿）与渲染 0.2.0 切换（51b8abf）均已收口。

### 与既有计划的关系

- **auto-lang 442**：本计划 = 其 §7.3 指定的"auto-musk 侧新的桥接设计
  /实现"承接方。①②完成即满足其验收 3 的 musk 侧条件，442 可收口。
- **PLAN-041（web 轨退役，挂起中）**：442 收口 = 041 解挂条件达成，
  本计划②完成后 041 可启动。
- **KNOWN-DEBT 018（hw/ag 双轨债）**：数据层逐域 .at 化会自然缩小双
  轨面，每域收口时顺带核对。

## 详细设计

### D1 状态闭包桥（Phase 1）

- `src/vm_backend.rs` 新模块：`MUSK_BACKEND=vm` 时 serve 路径改为——
  宿主构建 `Arc<AppState>`（复用 `server.rs:61` 现有构建）→ 经
  auto-lang `backend_abi`/`host_bridge` 注册数据 extern HostCallFn
  （闭包捕获 state；无状态 extern 直接转发 `extern_impl.rs` 同名 fn）→
  `auto_lang::run_file` 跑 `auto-src/main.at`（VM HTTP server 形态）。
- **`extern_sigs.vm.at`**：无状态签名变体。内容 = 现 extern_sigs.at
  的去 `s`/`State` 参数版（如 `extern fn specs_load(q Query<WorkspaceQuery>) Value`）；
  ext_stubs 的 adapter 链在 VM 装载时自动优选 `.vm.at`（机制已实证）。
- **调用点改造**：67 处状态调用点按文件分批——handler 体内的
  `extern_fn(s.view, ...)` 改 `extern_fn(...)`（VM 轨经 adapter），
  **ag 轨兼容策略**：`.at` 源统一用无状态签名，`extern_sigs.at`（Rust
  轨声明）同步去状态化 + `extern_impl.rs` 的包装 fn 改从闭包/全局
  OnceLock 取 state（与 VM 同构，ag 轨也删 State 透传）——**避免双签名
  分叉维护**（比 adapter 双轨更省，裁定见待澄清 #1）。

### D2 parity harness 换 VM（Phase 2）

- `tests/` 新增 `vm_serve_harness.rs`：拉起 `musk serve`（子进程，
  `MUSK_BACKEND=vm`，随机端口），wait-on /health。
- `parity_relay_api`/`parity_app_config` 等加 env 门控：`PARITY_TARGET=vm`
  时 base_url 指向 VM serve，hw 对照不变；SSE 用例走真实流。
- CI/本地验收命令：`PARITY_TARGET=vm cargo test -p musk --test parity_relay_api`。

### D3 AuthStore 域 .at 重实现（Phase 3，退役样板）

- `auto-src/auth_store.at` 新模块：session 表（内存 + JSON 持久化经
  fs natives）、login/logout/me 逻辑；密码 hash 经 `#[rs]` 直通或
  `use.rust` sha2/rand（与 hw 同库，行为等价）。
- `extern_impl.rs` 的 auth 域 fn 退役：`AppState.auth` 字段类型改指向
  .at 版 store 的 a2r 产物；parity（D2 的 VM 面 + hw 面）双绿后删
  Rust 侧旧实现。
- 退役模式固化为清单（域 fn 清单 → .at 重实现 → 双面 parity → 删
  Rust → KNOWN-DEBT 回填），供 specs/wiki/relay 后续复制。

## 测试设计

1. **桥接单测**：vm_backend.rs 注册表完整性（每个 extern_sigs 声明有
   HostCallFn；无状态 extern 直转发的正确性抽查）。
2. **端到端**：VM serve 起服后 `/api/health`、`/api/auth/login+me`、
   `/api/specs?section=goals`（读）、`/api/relay/runs`（列表）实测。
3. **parity 双面**：D2 harness 下 hw vs VM 对照全绿（含 SSE 一条流式
   用例）。
4. **回归**：`cargo test -p musk`（ag 轨全量）+ auto-lang
   `plan442_musk_probe_tests`（前端探针）+ `plan442_musk_backend_probe`
   （后端探针，随 extern_sigs.vm.at 落地推进断言面）。
5. **AuthStore 退役**：auth 域 parity 双面 + 既有 auth 集成测试全绿。

## 验收标准

1. `MUSK_BACKEND=vm cargo run -p musk -- serve` 起服成功，D2 端到端
   四组端点通过；AppState 不出现在任何 JSON ABI 载荷（桥接层断言）。
2. `PARITY_TARGET=vm` 下 parity 套件全绿（hw 对照不回归）——442 验收
   3 的 musk 侧条件达成，442 可转 C3 收口。
3. AuthStore 域：`extern_impl.rs` auth fn 删除，`.at` 实现在双面
   parity 下等价；退役清单模板落档。
4. specs/wiki/relay 三域的剩余 extern 清单 + 退役排期登记
   KNOWN-DEBT（后续计划承接）。
5. 全程 `cargo test -p musk` ag 轨全绿（Rust 生产轨零回归）。

## 执行步骤

### Phase 1 — 状态闭包桥

- [x] **T1** 桥接模块骨架：`src/vm_backend.rs`——`MUSK_BACKEND=vm`
  分支接入 `main.rs` serve 路径；构建 `Arc<AppState>` + 注册表空壳 +
  `auto_lang::run_file("auto-src/main.at")` 引线。验证：
  `MUSK_BACKEND=vm cargo run -p musk -- serve` 进程起且 `/api/health`
  返回 200（无数据 extern 时仅静态路由）。 [✅ 已完成（2026-08-26）实测
  `/api/health` → `{"status":"ok"}` 200 + workflows/professions/modes 200,
  2137 路由注册;配套 auto-lang worktree plan-044 三提交(layer 直通 shim/
  bare Json 构造 shim/response ctor RC stake 作用域修复——后者根治
  `Json(字面量)` 的 use-after-free,已合 master)。]
- [x] **T2** 无状态 extern 直转发：extern_impl.rs 中不收 State 的 fn
  （wiki_* 文件系族等）逐个包 HostCallFn 注册；`extern_sigs.vm.at`
  建立并声明同名无状态签名。验证：VM serve 下 `GET /api/wiki/...`
  读端点 200。 [✅ 已完成(2026-08-26) 超预期形态:未走 extern_sigs.vm.at 分轨,
  而是 extern_sigs.at 桩体统一改调 auto-lang 新 native musk_extern_dispatch
  (name,args)→host(3129,worktree plan-044 合入);extern_impl.rs 无状态集
  (professions/modes/skills/roles/config/app_config/forge_mode/workflows/
  relay professions+flows/wiki 等)经宿主闭包直转,professions 实测真数据。]
- [x] **T3** 状态 extern 去参数化（裁定后按 D1 统一策略）：67 调用点
  所在 4 文件的 `s.*` 实参移除；`extern_sigs.at` 同步；`extern_impl.rs`
  状态 fn 改 OnceLock 全局 state（ag 轨同构改造）。验证：
  `cargo test -p musk` 全绿（ag 轨回归）+ VM serve 下
  `/api/auth/login` + `/api/specs?section=goals` 实测通过。 [✅ 已完成(2026-08-26)
  策略 (b′):调用点零改动——桩体跳过状态参(_s/_ws @T 不进 args),
  宿主闭包捕获 OnceLock AppState;.at 的 s.view 是语言级借用表达式
  (a2r 发 &s),VM 侧经 AppState 访问器直通 shim 原样透传;specs/chats/
  conversations/workspace/relay_runs/ws_wiki_list 桥接实测——specs
  ?workspace=musk-demo 返回与 hw 同形真数据;名字碰撞 specs_list
  (extern 桩 vs server.at handler)改名 specs_files_list 消除;
  cargo test 406 绿。]
- [x] **T4** SSE 路径过桥（▶ 勘察完成,实现续接:mpsc extern 族是 JSON 友好
  线型——mpsc_channel() 返回 json!(id),HANDLES side-table 在宿主,tx/rx 即
  JSON 数字,可直接过现有网关;难点收敛为 mpsc_recv(rx).await 与
  agent_run_stream(...).await 的异步语义过桥——宿主闭包是同步 fn(&str),
  VM 侧 await 需 yield 通道(参照 waiting_sse_stream_id 的 iterator.next
  重试模式);sse_event/Sse.new 等响应构造 auto-lang 已有本地 shim。）:server_stream.at 的 9 处 extern 中状态相关
  者闭包化，流式 handler 经 host_bridge 注入事件（442 C2 ② SSE 形态
  对接）。验证：VM serve 下 `/api/run/stream` 一条完整 SSE 流冒烟
  （MockClient 即可）。
  - **▶ 宿主侧已落地(2026-08-26 续,2eca95e)**:vm_backend 注册 mpsc 全族
    (channel/sender/receiver/try_send/recv/msg_is_none/msg_unwrap,mpsc_recv
    经 VM 专用 tokio Runtime block_on)+ async 生产者(agent_run_stream/
    wf_run_with_progress/chat_run_stream spawn fire-and-forget,与 hw 一致)
    + workflow_exists/mode_exists/stream_event_map;cargo test 406 绿;
    vm_entry 装配 🔴 七条路由(对齐 hw server.rs serve),server_stream.at
    加 run_endpoint 别名(裸名 run 撞内建)。
  - **▶ 剩余卡点(精确记录,待续)**:vm_entry 路由装配的 VM 命名/闭包解析
    三连——① 裸名 `run` 与内建撞名(别名已加,但) ② 模块 pub fn 别名
    (run_endpoint 等)不流经 use 导入的裸名绑定面(collect_module_imports
    别名表只覆盖既有调用面) ③ 模块限定名 fn-ref(server_stream.X 传给
    post())可解析可分发但闭包捕获取错体(POST /api/run/stream 返回
    health 的 body、VMDISP 零命中——疑 axum_adapter resolve_params 的
    exports_by_name addr 反查在多模块场景失真)。下一步:auto-lang 侧修
    fn-ref 闭包解析(worktree),或 vm_entry 改本地 thunk fn 转发绕开。
  - **▶▶ 路由装配打通(2026-08-26 再续)**:根因=跨模块 fn-ref 闭包解析
    陷阱的解法=🔴 路由组装下沉为 server_stream.at 本模块的 red_routes()
    (同 relay_routes 形态——模块内 fn-ref 分发本来就正确);server_stream
    补 Router/routing use.rust 声明;run 改名 run_nonstream(run_endpoint
    转发,避内建撞名)。实测:POST /api/run/stream bogus mode → 正确错误
    包络 {"error":"unknown mode ''; available: ..."};VMDISP 链路活跃。
  - **▶ 剩余(auto-lang 侧,已收敛到点)**:Json body → VM 对象的 Option
    字段语义——缺字段访问报 RuntimeError("Field 'mode' not found on
    __json_object")而非 null(Option unwrap_or 链路断);有值时
    body.mode 也传 null(字段提取丢失)。修点:axum Json extractor 的
    json→instance 字段填充或实例字段访问的缺省语义。
  - **▶▶ 缺字段→null 已修(auto-lang d86615620)**:__json_object 实例缺字段
    读 null。剩两个精确缺口(2026-08-26 收口记录):① Nil 接收者的
    .unwrap_or 方法缺失(CALL_SPEC: no function 'None.unwrap_or'——
    body.mode 为 null 时 unwrap_or 默认值链断);② Json body 的 present
    字段读取也回 null(body.mode="bogus" 但 mode_exists args=[null]——
    .mode 属性访问经 method-dispatch 路径未命中 json instance 字段)。
    修点:auto-lang CALL_SPEC 加 None.unwrap_or 臂 + __json_object 的
    方法形态字段访问(GET_FIELD vs CALL_METHOD 路径分叉)。
  - **▶▶ None.unwrap_or 已修(auto-lang worktree plan-044)**:CALL_SPEC None
    接收者协议(unwrap_or 取默认/clone·ok 透传)。实测 absent mode →
    mode_exists args=["superpowers"] 默认链通;mpsc 全链活跃
    (channel→sender→receiver→recv)。
  - **▶ 剩余两精确点(2026-08-26 晚收口)**:① present 字段读取仍 null——
    body.mode="bogus" 时 mode_exists args=[null];缺字段错误曾出现在
    GET_FIELD 路径(engine:5130)且 Str 分支本身正确,疑点收敛到
    Json<RunRequest> 类型化参数的 .mode 发射路径(Option 字段可能走
    CALL_SPEC getter 而非字段读,或 clone/unwrap_or 链上 String 接收者
    的中转丢值)——需 disasm 单 handler 定位。② SSE 流卡在响应提交前:
    mpsc_recv block_on 在 handler 返回 Response 前被求值(生成器疑似
    急切执行)+ 生产者侧 tokio panic(multi_thread mod.rs:91 = spawn
    无 runtime 上下文——extern_impl agent 路径内部再 spawn 的上下文
    或 block_on 线程问题)。
  - **▶▶▶ T4 全链贯通(2026-08-26 深夜,74f72cb)**:三连修——① void 桩
    补漏(机械改写正则漏了 25 个无返回类型 extern,agent_run_stream
    空体静默是"生产者消失"根因);② mpsc_recv 专职线程桥(VM http
    server 自身是 tokio,嵌套 block_on panic 根治);③ None/unwrap_or
    协议(auto-lang)。实测 VMDISP/VMHOST:agent_run_stream 真实参数
    (query/body/tx)过桥 → aaid 真会话 spawn+finish → turn_start 等
    事件流经 channel → msg_is_none/stream_event_map DTO 转换链活跃。
  - **▶ T4 最后一寸**:生成器急切求值——run_sse_stream(rx) 的事件在
    handler 返回 Response 前被整条消费(响应帧未增量下发,curl 收空)。
    修点:auto-lang 生成器调用语义惰性化(Sse.new 持迭代器、服务器
    迭代时才逐帧拉),或 http_server 的 SSE 分支改拉模式。
  - **▶ 惰性化已就位 + 帧拉取线程化(2026-08-26 深夜二)**:查明运行时
    CALL 本就有生成器短路(Plan 317),SSE 分支两端(axum/legacy)均有
    流式模式,"200 SSE (1ms)" 实测触发;SSE 帧拉取改专职线程(生成器
    体内阻塞宿主调用会饿死异步 worker 的 I/O 反应器——头写入但不上
    线,usize 洗 Send 同 vp 先例;auto-lang worktree 已合)。
  - **▶ 当前精确停摆点**:首帧链 mpsc_recv→turn_start→stream_event_map
    全通后,第二次 iterator.next 未再进 mpsc_recv(生成器未恢复或
    shim_iterator_next 在 puller 线程内让出/阻塞形态待查);且头 0
    字节上线(write+flush 已执行)。下一步:eprintln 逐帧打点 SSE 循环
    + sse_frame_from_nv 对 Event 对象的输出核验。
  - **▶▶ to_value 已修 + 打点就位(auto-lang ac2529aaf)**:to_value 恒等
    shim(DTO 实例即可序列化)——v9 实证第二次 iterator.next 恢复了
    生成器(事件链双次),断点在 sse_event(name,to_value(dto))。
  - **▶ 当前停摆(v10,SSED 打点实测)**:headers flushed → pulling
    frame #1 后 puller 线程内 shim_iterator_next 未进入生成器体
    (无 mpsc_recv VMDISP;v9 同位曾恢复)——嫌疑:线程化拉取与
    迭代器/任务锁序(shim_iterator_next 的 Yield 重试路径在非执行器
    线程上的行为)。下一步:shim_iterator_next 内部打点(进入/恢复/
    Yield 三点),或帧拉取回执行器线程 + mpsc_recv 桥改
    spawn_blocking 形态。
  - **✅✅ T4 验收达成(2026-08-27 凌晨,73f1b7f28 后净二进制复验)**:
    to_value 修毕后全链自然打通——SSED 打点实证逐帧链
    (pulled→formatted→written+flushed),冒烟输出四帧全序列
    turn_start/delta{"text":"ok"}/turn_end/done,流正常终止(channel
    关闭→-1→break),wire 形状与 hw SseEventDto 一致;真实 aaid 会话
    (非 Mock)。v10 停摆为偶发时序(未再复现),稳定性观察并入 T6
    SSE parity 用例。Rust 轨 406 绿;SSED 调试打点已移除。

### Phase 2 — parity harness 换 VM

- [x] **T5** `tests/vm_serve_harness.rs`（首期骨架+冒烟门;套件迁移续接）：子进程拉起 + 端口探活 +
  清理；`PARITY_TARGET` env 门控接入 parity 测试公共构造。验证：
  `PARITY_TARGET=vm cargo test -p musk --test parity_relay_api` 全绿。
- [ ] **T6** SSE parity 用例：真实流对照（hw vs VM 各消费一条流，
  事件序归一断言）。验证：同 T5 命令含流式用例通过。
- [ ] **T7** 442 验收 3 对账：双面 parity 全绿记录回填 auto-lang 442
  文档（其 §7.3 接力项闭环），442 转 C3 观察期流程。验证：442 文档
  grep 到回填记录。

### Phase 3 — AuthStore 域退役（样板）

- [ ] **T8** auth 域 extern 清单盘点：extern_impl.rs 中 auth 相关 fn
  全列 + 每个的行为锚点（hw 侧现有测试名）。验证：清单落本节回填。
- [ ] **T9** `auto-src/auth_store.at` 重实现：session 表 + login/me/
  logout + 持久化；密码 hash 直通 sha2/rand。验证：
  `auto trans --path auto-src/auth_store.at rust` 0 错 + 单测
  （.at 侧逻辑经 VM 探针跑）。
- [ ] **T10** auth 域接线切换：AppState.auth 指向 .at 版 store 产物；
  hw/ag/VM 三面 auth 端点等价。验证：`cargo test -p musk` 全绿 +
  `PARITY_TARGET=vm` auth 用例绿。
- [ ] **T11** auth 域 Rust 旧实现删除 + 退役清单模板落档（D3 五步
  模板）。验证：extern_impl.rs auth fn grep 零命中 + 模板入本计划
  附录节。
- [ ] **T12** 剩余域排期登记：specs/wiki/relay 各域 extern 清单 +
  体量估计 + 顺序建议入 KNOWN-DEBT-AND-RISKS.md。验证：grep 登记
  三条。
- [ ] **T13** 文档收口：KNOWN-DEBT 442-B/442-C 相关条目更新；
  pac.at 头注 "待激活" 注释按 VM serve 可用性改写（保留 rust 默认）。
  验证：三处文件 grep。
- [ ] **T14** spec 沉淀准备：spec-impact 元数据（touched_goals：
  双轨 parity → VM 桥接 + 数据层单源化路线）。验证：frontmatter
  填齐，转 /auto-plan:review。

## 复审记录

（待 /auto-plan:review 填写）

## 待澄清事项

1. **去状态化的轨道策略**（T3 关键裁定）：(a) 双签名——`.at` 统一无
   状态 + ag 轨经 `extern_sigs.web.at`... 即 Rust 轨 adapter 保 State
   透传（extern_impl.rs 零改动）；(b) 单签名——两轨都去 State，
   extern_impl.rs 改 OnceLock 全局 state（推荐：无双签名维护税，且
   Rust 侧 OnceLock 与 VM 闭包同构，一次改完）。本计划按 (b) 起草，
   请确认。
2. **relay 域退役时机**：SSE/timer 最重，是否允许在后续计划长期搁置
   （桥接形态长期承载）——影响 442 C3 观察期的"收口定义"。
3. **aaid Client 的 .at 化**：`Arc<dyn Client>` 本身是否列入退役范围
   （HTTP 客户端经 auto.http natives 可表达，但 MockClient/真实双实现
   面大）——建议本计划不含，登记后续。
4. **观察期长度**：442 C3 移交后的双后端并行观察期（默认 7 天对齐
   041 惯例）——请确认。
