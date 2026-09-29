# PLAN-093 T-01 能力探针报告（capability map）

- plan_id: PLAN-093 / plan_revision: 1 / 任务: T-01
- 生成时间: 2026-09-29
- 作者: work 执行会话（auto-plan:work）

## 1. 版本与实际命令

| 项 | 值 |
|---|---|
| musk worktree | `D:/autostack/.wt/musk-093/auto-musk`，分支 `plan-093-dev`，base `f8f99f3dc066b557b4c5b003dd2e0d3faa8bb428` |
| musk 探针新增 | 未改任何既有源码；新增 `tests/ui-parity/probes/{probe-a-media,probe-b-pointer}`、`scripts/ui-parity/canvas-studio-probe.mjs`、`scripts/ui-parity/gen-probe-png.mjs`、`tests/ui-parity/probes/fixture-frame.png`（240×160 确定性 PNG，蓝标记块 (150,90,40,24)） |
| auto CLI（探针实际使用） | `target/debug/deps/auto.exe`（2026-09-29 由 auto-lang 检出源构建）。主 `target/debug/auto.exe` 被 2 个在跑 auto 进程锁定（os error 5，与 PLAN-094 部署观察项同源），未强杀 |
| auto CLI（仓库既有） | `auto 0.1.0+v0.4.2-2183-gc8f86ef92`（build 2026-09-28） |
| auto-lang 检出 | `e2deb4f879c27cf4e34597363f9866fe4bcf7ae2`（master 工作区；PLAN-705 已并入，检出处干净仅有 .tmp-* 杂物）——探针前执行了增量 `cargo build --bin auto` |
| auto-ai / auto-down | `5a50a55844d7aa3523b593f21ba0fb03d18eac48` / `3373a5cc6e3a00336613133db51906fb0940777d`（与计划 §4 调查版一致，未动） |
| 探针执行方式 | `node scripts/ui-parity/canvas-studio-probe.mjs [--probe a|b]`——本地 Node 帧服务器 + `auto run --render=vm` + AutoUI MCP（snapshot/autoui_state/autoui_action drag 合成） |
| worktree 构建环境 | 宿主 shell 带 `RUSTC_WRAPPER=sccache`——**worktree 全新 target 下 sccache 使 test profile 编译损坏**（"only metadata stub for rlib `test`"、"crate X required in rlib format" 全家桶；full clean 后复现，去 sccache 后消失）。本计划所有 cargo 命令一律前置 `RUSTC_WRAPPER=` |
| 依赖解析 | musk 后端 path deps（`../../../../auto-ai|auto-lang/...`）从组目录解析 → 已建同组依赖 worktree `D:/autostack/.wt/musk-093/auto-ai`（auto-musk-dev@main 5a50a55）、`D:/autostack/.wt/musk-093/auto-lang`（auto-musk-dev@master ec5adb7af；合回时按 AGENTS.md 尽快折叠清理） |
| 收据 | `tmp/ui-parity/PLAN-093/probe-a-receipt.json`、`probe-b-receipt.json`（含模型状态原文、快照、命令环境） |

## 2. 探针结果

### 探针 A — 真实 PNG 媒体运输链（PASS）

结论：**VM 臂可用 .at 代码走通"HTTP 二进制帧 → 落盘 → 媒体管线 ticket → image_surface 渲染"全程，逐字节忠实，非阻塞**。

实测链路（`probe-a-media/src/front/app.at`）：

1. `Http.request("GET", url)` → `.send()`——句柄族非阻塞 HTTP（PLAN-705 固定 async executor）。handler 在 `HttpRequest` 等待期 **park**（`[VM-PARKED] handler_App_Tick parked … — UI stays interactive`），完成后 resume，每拍 17–19ms 往返，UI 不冻结（PLAN-702 段派发实证）。
2. `res.status_code()` == 200；`res.body_to_file(path)` 宿主侧直写盘，返回字节数 = 428 == fixture PNG 实际大小（**字节忠实**）。
3. 每帧写独立路径 `frame-<seq>.png` 后 `image.open_session(path)`（单文件 viewer 会话）→ 下拍 `image.current_uri(sid)` 取得 `/api/__auto/media/{id}/{rev}` ticket URI → `image_surface(src=ticket)` 消费（渐进上屏）。
4. 落盘文件复核：49 个 frame PNG 均以 PNG magic 开头、428 字节。

关键实证（反面）：

- 裸 `Http.get(url)` 被编译期改写为 `auto.http.get_json + json.to_value`（auto-lang codegen PLAN-080 F-2③），二进制体走 JSON 解析直接失败——**帧运输必须用 `Http.request().send()` 句柄族**。
- `res.body_bytes()` 返回列表**跨 park/CALL_SPEC 丢 RC 生命周期**（auto-lang plan-022 D4 已知限制，本次复现：`Invalid list ID: 4000249`，上游 shim 内现成 debug 打印佐证该路径正被上游调查）——**必须用 `body_to_file`，字节不过 VM 堆**。
- `image.queue(path)` 返回**不透明句柄 int**，.at 面 `ImageTicket{uri}` 结构体字段读全空（`uri: 0 (int)`）——票 URI 无 .at 访问器。可用替代：`thumb()`（但只产方形 rendition，变形）与 `open_session(file)+current_uri()`（原始 rendition，**本计划选定**）。
- `open_session` 条目由 detached worker 异步补齐，**当拍 `current_uri` 恒空**，需下一拍重试（探针用 Tick 顶部有界重试）。
- 会话关闭语义：未引用资产 30s 宽限 + 解码 LRU（`queue_media_thumbnail` 文档口径），滚动关闭两代前的会话安全。

### 探针 B — 指针逻辑坐标 / 叠层 / 窗口 KV（PASS，附三项上游缺陷）

结论：**mouse-area coords 事件管道真实可达（onclick/onmousemove 经真实派发管道触发 handler）；窗口宽高 KV 可读；CSS-absolute 叠层在 VM 臂不可用**。

1. 结构：`mouse-area(coords:"240x160", onmousemove:.Move, onclick:.Click)` **内包 image**（非 absolute 兄弟）→ 快照呈现为带 `onclick/onmousemove` 的可寻址 container 节点。
2. AutoUI MCP `autoui_action drag` 合成（value = `App␟Move␟Move␟Click␟155,95;170,100;189,113`，分隔符 `U+0001F`，字段是 **handler 消息名**而非 DOM 事件名，widget 名 = `App`）→ `moves=3, clicks=1`——消息经 `__mcp_drag → on_with_input_for → call_handler_for("App",…)` 真实派发链命中 handler（`handler_App_Move`）。
3. `localStorage.getItem("vm.window_inner_width"/"vm.window_inner_height")` 返回实窗 `1024x768`（renderer 双漏斗发布，Plan 046-B 实证）。注意 Init 当拍常取不到（首次发布前），须随 tick 轮询。
4. **缺陷 B-1（叠层 hoist 丢树）**：内容 col 内任一 `position:absolute`+`z-N` 子节点（div/row 均复现）→ **宿主子树整体从渲染视图消失**（base+content 全弃；auto-lang `fold_floats`/Overlay hoist 路径，schema `overlay` 标 iced:"fallback" 与此吻合）。→ T-06 覆盖框 VM 方案不得依赖 CSS-absolute 叠层。
5. **缺陷 B-2（合成坐标位型错读）**：drag/pen 合成通道以 `d`（Double）编码坐标，`.at` float 形参收到位型错读 int（155.001 → 824633721，189.113 → 1649267442）。真实 PointerArea 路径发 `f`（Float）编码（Plan 499 §1.5），生产行为不受影响；仅 MCP 合成验收通道受限（T-11 确定性测试需绕行或修上游编码）。
6. 补充：元素实参位用括号表达式（`text ("a"+.b) {…}`）解析失败（error[19]/[20] Expected term）——文案一律预拼进 model 字段（canvas_panel 既有口径）。

## 3. 定案：VM 帧显示与点选链（供 T-03/T-04/T-06 遵守）

```text
[poll tick] Http.request("GET","{base}/api/canvas/frame?t=<seq>&gen=<gid>")   ← 相对 URL 不展开（RequestBuilder 族
   → res.body_to_file(<tmpdir>/frame-<seq>.png)            无 resolve_http_base_url；须 .at 侧拼绝对基址
   → image.open_session(<该路径>) → 下拍 current_uri → ticket URI
   → image_surface(src=ticket, fit=contain/one-to-one, width/height=内容矩形)
   → 版本门：onload 事件 VM 臂未接线（见 §4 G-1）→ 以 seq/代次 + 「等待新画面」
     候态表述，拿不到已渲染确证前不得宣称"已同步"
[hit] mouse-area(coords="<逻辑宽>x<逻辑高>") 内包 image，共享几何 helper 从
      容器尺寸+帧尺寸算内容矩形；点选坐标 = 事件逻辑坐标（origin=命中区左上）
```

- 帧文件滚动清理：每 seq 新路径（registry 以 path 为 fingerprint，同路径去重旧帧）；旧文件用 `file.delete` 滚删。
- 后端帧下载鉴权：`Http.request().send()` 走 Plan 446 E4 默认头/默认 query 注入（JWT+workspace），与 `#[api]`/`get_msg` 同源。

## 4. 上游能力缺口与最小依赖合同（全部已实证，非推测）

| ID | 缺口 | 实证位置 | 对 AC 的影响 | 最小依赖任务（auto-lang） |
|---|---|---|---|---|
| G-1 | `image_surface` onload/onerror 事件 VM 臂未消费（Plan 547 Task 25 "input adapter" 未实现；`AbstractView::ImageSurface` paint 臂显式 `let _ = (on_error, on_loaded, …)` 丢弃） | renderer.rs:6394-6400；`ImageSurfaceEvent` 全仓无消费者 | AC-04/AC-14「已展示当前版本」缺直接确证；以候态表述降级可交付 | 接线 ImageSurface 事件 → VM 消息（照 PointerArea 管道），预计小改 |
| G-2 | CSS-absolute+z 叠层 hoist 丢宿主子树 | 探针 B（div/row 两型复现，快照收据） | AC-06 VM 覆盖框（Agent 琥珀/选中蓝）无显示层落点 | 修 Overlay hoist；或本计划 T-06 走备选（覆盖框由共享几何生成的兄弟层/后端合成），定案推迟到 T-06 |
| G-3 | `dom.focus_first` VM 臂为 no-op stub；无 autofocus prop；无 .at 可达的置焦原语 | native.rs:8938（`shim_dom_focus_first` 空实现） | AC-10「修改按钮聚焦 composer」VM 臂 | 最小原语：按 widget id 置焦 text_input（iced request_focus 接线） |
| G-4 | `image.queue` 返回不透明句柄、URI 无 .at 访问器；`thumb()` 仅方形 rendition | stdlib.rs:4296 + 探针 A state 收据 | 非阻塞（open_session 替代已可用） | 可选小改：`queue_media_uri(path)` 仿 thumb 返回 URI 串 |
| G-5 | `body_bytes` 返回列表跨 park 丢 RC（plan-022 D4 已知）+ shim 内遗留 debug 打印 | stdlib.rs:6284-6302 + 探针 A 复现 | 非阻塞（body_to_file 已替代） | 可选：修 RC 生命周期 + 清 debug 打印 |
| G-6 | MCP drag/pen 合成用 `d`（Double）编码，.at float 形参位型错读 | renderer.rs:17005 + 探针 B state 收据 | T-11 确定性坐标断言需绕行 | 合成通道改 `f` 编码或 handler 双型分发 |

依赖任务边界：以上均为 auto-lang 侧独立任务（各自 worktree、各自验收），**不阻塞 T-02/T-04/T-05/T-07 的开工**（均已给出仓内替代）；G-1/G-2 阻断的仅是 VM 臂的"已展示确证"与"显示层覆盖框"，在依赖任务落地前 T-13 不得以降级表述冒充 AC-04/AC-06 通过。

## 5. 后端再生链与 V01 状态

- 实际再生命令（沿 PLAN-014/024/086 口径）：`auto trans --path backend/crates/musk/auto-src/<mod>.at rust` → `auto-src/nativeize.pl` 清理 → 按 KNOWN-DEBT 086 手修落位 `src/auto_generated/<mod>.rs`；extern 签名 sidecar 由 `node auto-src/gen_extern_sigs.js` 生成。
- **V01（`auto build --gen-only --strict`）当前在 worktree base 上即失败（与本计划改动无关的预存阻塞）**：
  - 两个 auto 二进制（gc8f86ef 与 e2deb4f 构建）同样失败；stderr 末尾恒为 5 条 S001 schema drift Info（`span.title`、`button.title`、`Check.size`、`Markdown.source` ×2 等，均为 musk 既有 widget 对当前 `schema/aura.at@e2deb4f` 的漂移），随后进程无消息退出（exit 1/127，`--lenient` 同样失败；失败点在 "Scanning targets" 之后、`Generating Vue project` 前后漂移）。
  - S001 属 Severity::Info（validators.rs:1033），CLI 的静默 abort 点未定位（需上游排查）。
  - **不影响 T-01 完成门**：VM 链已由探针实证（`auto run --render=vm` 运行时路径双探针绿，且这正是 T-05/T-13 的运行形态）。
  - 解除动作（按序）：①上游定位 `auto build` 静默 abort；②schema 漂移对齐（`SCHEMA_DRIFT_GENERATE_AT=1` 在 auto-lang 依赖 worktree 再生 aura.at，或 musk 侧 widget prop 对齐——须逐条判断 prop 合法性，不得静默删改既有行为）；③恢复后重跑 `auto build --gen-only --strict` + `gen/front/vue pnpm build` 补 V01 收据。T-05 起每个触前端生成的任务开工前须复测该门。

## 6. 其余 T-01 勘察结论（源码级，供下游引用）

- **VMHTTP 桥二进制响应通道（T-03 定案）**：VM HTTP 服务器 `ApiBody::Text(Vec<u8>)` 按原始字节回包；宿主侧 `auto_lang::vm::ffi::stdlib::insert_http_response(status, headers, body) -> u64`（thread-local 注册表，owner 单线程语义下安全）→ .at handler 返回该 i64 句柄即按字节出包（`lookup_http_response` 编组，Plan 442 C2）。canvas_routes 为手写 Rust Router，vm_entry.at 未合并 → VMHTTP 模式下 `/api/canvas/*` 现在 404；T-03 以 `.at` 声明 canvas 路由 + 宿主调用桥接同一 CanvasManager。
- **Canvas 后端现状（T-02 改面）**：`CanvasManager`（单例、watchdog 1/2/4s×3、帧缓存 `(seq, PNG)`、锚点索引、picked/overlay）；路由 start/frame/status/pick/stop 无代次/归属概念；pick 未命中 204 但不清 picked；`ToolContext` 已有 workspace/parent_conversation_id 可挂归属。
- **既有 web 前端债（T-04/T-06 改面）**：`canvas_store.at` 非 stopped 每拍强制 `cv_open=true`（轮询复活用户收起）；`canvas_web.ts` 以整 img 盒换算点选（contain 留白出界→后端 204 兜底）、document 级全局点击委托；`ports/canvas.vm.at` 全空桩。
- **auto-lang 能力面（已核对 @e2deb4f）**：`imagesurface` iced:full（fit contain/width/one-to-one/free + zoom/pan，像素仅出自后台 decode LRU，UI 线程零解码）；`mouse-area` iced:full（onclick/ondblclick/onmousemove≤30Hz 限频+0.5px 量化，`coords` 逻辑幅面）；`image` iced:partial（同步 http 抓取 3s 超时、无鉴权头——禁用于帧链）；相对 URL 基址展开仅 `get_json` 族有，RequestBuilder 族无。

## 7. T-02 接口表（已实现，2026-09-29，commit 见 worktree log）

Canvas 预览身份契约落地（`backend/crates/musk/src/canvas/{manager,mod,tools}.rs`）：

| 端点/入口 | 增量 | 兼容语义 |
|---|---|---|
| `POST /api/canvas/start` | body 增 `conversation_id?`、`expected_generation?`；响应增 `generation_id/owner_workspace_id/owner_conversation_id`（state/app_path 原样保留） | 无 expected = 旧无条件替换；expected 与当前不一致且当前≠0 → **409** `{error, generation_id}` 不替换新目标 |
| `GET /api/canvas/frame` | query 增 `generation?`、`seq?` | 无参 = 现行 200/503；带 generation 不一致或 seq 为旧值（有帧）→ **409**（不伪造历史帧）；无帧 503 保持 |
| `GET /api/canvas/status` | 载荷增 `generation_id/owner_workspace_id/owner_conversation_id`，`frame` 增 `valid` | 原字段全部保留 |
| `POST /api/canvas/pick` | body 增 `clear?`（与 x/y/vnode_id 互斥，违者 400）、`expected_generation?`；**未命中（204）与显式 clear 均清 picked**；clear → 200 `{cleared:true}` | 旧 x/y/vnode_id 语义不变 |
| `POST /api/canvas/stop` | query 增 `generation?` | 无参 = 旧无条件停止；不一致 → **409** 不杀新目标；响应增 `generation_id` |
| 工具面 | `canvas_run` 登记归属（workspace+conversation）；`canvas_stop/pick/act/state/snapshot/overlay` 按 `ensure_session_owner` 守卫（归属他人 → Exec 错误拒绝）；未绑定预览（无归属）保持任意会话可用 | 成功文案不变 |

Manager 内部：`begin_session`（spawn 前纯状态：冲突判定+归属登记+清场，测试直驱）、
`publish_frame`/`publish_anchor`（帧/锚点唯一写点，seq 先自增）、watchdog 复活保留
代次但清 frame/anchor/picked/overlay；代次仅显式 start 递增。

测试：`backend/crates/musk/tests/canvas_studio_contract.rs`（19 例，无 spawn，
oneshot 直打路由）——代次递增/冲突、归属停止守卫、未绑定语义、帧 seq/代次
冲突不伪造历史、未命中清选、换代丢陈旧选、status 身份字段、路由 409/清选臂。


## 8. T-03 接缝修复与实机验证记录（2026-09-29，work 会话续）

### 8.1 阻塞②解除：VM serve 符号解析（musk 语料适配 Plan 545）

- 根因：auto-lang Plan 545 移除隐式平铺导入——依赖模块导出仅注册
  `mod#sym`/`mod.sym` 限定名；裸名 reloc 不再绑定依赖导出。语料
  （仅 vm_entry.at 有裸 use）全部建立在旧语义上，首个跨模块调用即
  `Undefined symbol: auth_header_token in module server`。
- 适配（全在 musk 仓，46b1173）：242 个跨模块调用名静态盘点
  （analyze_cross_module.js）；228 个指向 extern_sigs，其余为路由装配
  （vm_entry→5 路由模块）与两处歧义签名裁定（drive_run→relay_driver、
  build_agent_with_context→lib）。17 模块加具名条目导入
  （`use X: a, b`，stdlib 先例 http_stream.at）；vm_entry 裸 use 升级条目
  形态。歧义名 8 个中仅 2 个有真实跨模块调用点，逐一按签名/await 形态
  裁定，其余无人调用不导入（避免 import_scope 遮蔽本地定义——
  resolve_call_symbol 首查 import_scope）。
- 实证：serve 启动 3601 路由、`/api/professions` 200。

### 8.2 G-7（新登记）：ext/type 方法体解析不到模块导入

- 实证：deps/auto.exe v0.4.2-2221 双 fixture——条目导入后顶层 fn 裸调用
  解析 ✓；同文件 ext 块方法体内同一 fn 裸调用 → 裸 reloc link 失败；
  限定调用 `mod.fn()` 在方法体内 → "Undefined variable"。dep 模块路径同症。
- 影响：VM 加载集内仅 wiki（3 名 6 调用点），顶层 shim（`__wiki_*`）规避。
  transpile-only 模块还有 25 个方法体调用点（auth/chats/orch_tools/
  relay_driver/spec_tools/task_plan_engine/task_plan_registry/tools），
  VM 轨不加载、暂不处理。
- 解除动作（依赖任务）：auto-lang codegen 让 ext 方法体调用解析接
  import_scope/known_module_prefixes，或 musk 语料全量 shim 化（后者的
  a2r 轨影响需先评估）。

### 8.3 VMHTTP 桥三处修复与五路由实证（8869f6a）

- 桥内嵌套 runtime：canvas_*_host 宿主闭包在 tokio worker 内执行，
  `Runtime::block_on` 直接 panic（"Cannot start a runtime from within a
  runtime"，vmret5 实证）→ 专职线程桥（vm_backend mpsc_recv 同款）。
- 3 提取器 handler：`(s,q,body)` 三提取器（async 与否皆）经 axum_adapter
  派发触发引擎 RET 帧错位（engine.rs:9004 下溢；chat_get 3 参同症、
  auth_login 2 参正常）→ 五 handler 去 State 提取器（宿主闭包本就用
  STATE 单例自建 state，wire 契约不变）。
- session.rs:155/165：`split_off(160)` 把"最后 160 行"误写成下标 160，
  目标子进程早退（行数<160）即 panic → `len.saturating_sub(160)`。
- 实证（curl 收据 /tmp/vmret9.log，本机易失，要点入本节）：status 200
  （generation_id/owner_workspace_id/owner_conversation_id/frame.valid
  全）·frame 503（无目标）·frame?generation=999 → 409·stop 200
  `{"state":"stopped","generation_id":0}`·pick clear 200
  `{"cleared":true}`·start 不存在路径 400（路径越界文案列根）。0 panic。

### 8.4 G-8/G-9/G-10（新登记）：VM 轨引擎/续体缺陷（阻塞剩余验证）

- G-8 .at 路由 handler 帧核算错位：axum_adapter 派发的 handler task
  每请求触发 `[VM-RET] underflow guard: bp=1, n_args=1`（engine.rs:9010
  守卫夹值保命）；start 的 spawn 长宿主调用路径则在下溢守卫之前的
  减法处 panic（debug 构建）→ VM server 线程死亡（vmret11 实证）。
  宿主侧 insert_http_response 直出的 200（curl 可见）掩盖 handler 任务
  自身帧已破。解除动作（依赖任务）：auto-lang 引擎/适配器修帧核算与
  长宿主调用下的 RET 路径；守卫前移到减法之前是最小止血。
- G-9 VM 客户端 POST park 续体丢失：`Http.request(..).body(..).send()`
  与 `Http.post(url,body)` 两形态，park 后 resume 到 completion 但调用方
  状态写入丢失、后续 tick 停摆（vm11/12 实证）；GET 无 body 链
  （probe-a 49 帧、probe-c status 轮询）可用。影响 ports/canvas.vm.at 的
  canvasStart/canvasPickNode/canvasClearPick/canvasStop（全 POST）与
  T-04 store 的全部 POST 调用形态。解除动作（依赖任务）：修 POST+body
  的 park/resume 续体恢复；此前 T-04 的 POST 面需以轮询/委托形态绕行
  （如 status 驱动 + runner 侧驱动），不得以降级表述冒充通过。
- G-10 MCP autoui_state 回读滞后：park/resume 后的 tick 模型写入不进
  MCP 快照（rust 模式后端日志证明状态机实际推进 source 步、快照恒
  初值）。探针证据通道受限；产品 UI 不受影响（view 重建消费真模型）。
  解除动作（依赖任务）：autoui_state 快照刷新时机对齐 park/resume 写回。

### 8.5 对下游任务的约束

- T-04：store 的 POST 面在 G-9 解除前不可用；轮询链（GET status/frame）
  与 files/raw 可用。VM 臂"已同步/已展示"表述继续受 G-1（image 事件）
  与本节 G-8 双重约束。
- T-06：VM 臂覆盖框/点选依赖 G-2 与 G-8；web 臂不受影响。
- T-13：V05 四模式矩阵在 G-8/9/10 + V01（静默 abort，45min 构建
  26m33s exit 1 无诊断，/tmp/v01-full.log 要点已录）解除前无法全绿；
  不得以部分证据冒充 AC-05/AC-16 通过。
