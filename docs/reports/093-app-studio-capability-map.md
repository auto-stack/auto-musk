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

### 8.6 正统修复落账（2026-09-29，auto-musk-dev@c93ed76a0 + musk@5532b88/2e61eda）

用户裁定"不用绕法、正修根因"后，本会话在组内依赖 worktree
（D:/autostack/.wt/musk-093/auto-lang，分支 auto-musk-dev）完成三处
auto-lang 引擎/适配器修复并经 musk 联动验证：

| 缺口 | 根因 | 修复 | 验证 |
|---|---|---|---|
| G-8①RET 下溢 | 守卫写在减法之后（debug 必炸） | engine.rs 守卫前移（checked 语义） | VMHTTP start spawn 路径不再崩 VM |
| G-8②0 参入栈 | resolve_params 声明名回退只切 `.`，`mod#sym` 键永远 miss → 全语料路由 0 参、每请求 VM-RET 守卫 | 抽 param_sig_decl_name 同时切 `.`/`#` + 3 单测 | 守卫 0 触发；auth_login 等拿到真实参数 |
| G-8③catch-all 404 | match_route 段数严格相等，`{*path}` 多段剩余永远 404 | 尾部 catch-all 消费剩余段（axum 语义）+ 5 单测 | /api/files/{ws}/*path 通 → canvasLoadSource 链通 |
| G-10 | 实为 G-9 的读回面（探针层） | 随 G-9 形态修复后 MCP 回读实时 | client-status-poll 断言经 MCP 状态通过 |

musk 侧配套修复：stop_guarded 补 §5.3 清场（帧/锚点/选中/overlay）；
canvas_vm_query_* 字符串数字双形态（409 守卫恢复实效）；
canvasLoadSource 对齐 VM 文件路由（/api/files/{ws}/*path，签名加 ws）。

联动验证：ports 探针 15 断言 × 双后端（VMHTTP/RustHTTP）全绿——
spawn 真目标、真 PNG 帧、锚点树、点选/未命中 204/清选、源码读取、
带代次停止、终态 503、错误面 400；contract 19/19；auto-lang 语料档
cargo tv 162/162；新增定向单测 8/8。

**剩余（下一轮）**：
- G-9 残余：send 位于 if 块内（条件 send）时 park 续体丢失
  （pre-park 写也丢、tick 链停摆）——形态实验矩阵已钉死触发条件，
  复现器即 probe-c fixture；正修在引擎段派发/合成层。探针当前用
  顶层 send 形态（已留哨兵注释），生产 T-04 store 写法受其约束。
- G-7：ext/type 方法体导入解析（wiki shim 撤除依赖此项）。
- V01：`auto build --gen-only --strict` 静默 abort（26m33s exit 1
  无诊断）+ S001 漂移对齐（SCHEMA_DRIFT_GENERATE_AT=1 再生真源）。

### 8.7 G-9 残余侦查（2026-09-29 第二轮，AUTO_DEBUG_G9 探针实证）

触发条件钉死（形态实验矩阵，同引擎同 fixture）：
- 顶层无条件 send + 任意嵌套写回：全链正常（15 断言双后端绿）。
- send 条件化（if 块内 park）：异常。签名：pre-park 写落地（服务端
  t=1,3,4…13 递增实证）；post-resume 写全部丢失（r_start/phase 恒初值）；
  视图模型绑定子节点整体消失（autoui_snapshot 只剩静态 text）；
  tick 链可能停摆（parks=110 自旋态=async 结果丢失后段模式重入臂
  无超时——忙等臂有 30s 上限，段模式没有）。

机理实测（AUTO_DEBUG_G9=1）：rewind/re-fire 机械正确（ip 0x4ee→0x4f4、
0x57c→0x582 各 +6）；两个请求后端都 200；handler 走到 RET
（"resumed to completion"）。即：**续体代码执行了，但其 SET_FIELD
写不落在 state_obj_id 对象上**——视图与 MCP 读的是 state_obj_id
（见到 pre-park 值），续体写的是别处。首要假设：状态对象身份/rc
核算——parked task 栈持 __state stake，完成时 rc_release_task_stack
清账路径与 state 对象存活的交互；次疑：resume 泵与对象替换/回收。
非确定性细节（vm13 两请求 vs vmg10 一请求 vs 本次全流程）提示竞态面。

下一轮最短路径（仪表已就位）：
1. vm_bridge 在 call_handler_for / register_parked / resume 完成 /
   view build 四点打印 state_obj_id + 堆对象指针身份 → 定位"写往
   别处"的分叉点。
2. 审计 rc_release_task_stack 对 parked-完成任务的清账是否波及
   state 对象（stake 表 vs 对象 refcount）。
3. 段模式 re-fire 臂补超时（对齐忙等臂 30s），消灭 parks 自旋态。
4. 修复后：探针恢复自然条件形态回归 15/15 ×2，撤哨兵注释。

本轮收尾状态：探针以可工作形态双模式 15/15（219583b）；引擎诊断
环境门控入库（auto-musk-dev@394739f05）。

### 8.8 G-7 修复 + V01 门通过（2026-09-29 第三轮）

**G-7 正统修复**（auto-musk-dev@c35a55ca8）：根因 = 解析器把 ext 块合并进
`type X`（AST 中 Ext 消失），合并方法随 TypeDecl pass 编码，而 Use 在
后续 pass 才处理 → 方法体 import_scope 恒空 → 裸名 reloc 链接失败。
修复：dep 路径（compile_module_to_bytecode）Use 提到 pass1 之前、pass3
排除防双处理；入口路径（execute_autovm_with_path）Use 分区提出先于
TypeDecl 编码，未合并的独立 Ext 归 other_stmts 按源序。验证：双 fixture
（dep/入口路径）输出预期值；cargo tv 162/162；musk wiki.at 撤除
__wiki_* shim 后自然调用经 VM serve 零链接错误；探针双后端 15/15。
**绕法撤除完成**（ea62d97）。

**V01 门通过**（本轮 CLI，含全部引擎修复）：`auto build --gen-only
--strict` exit 0（39m02s，60 组件，51 S001 Info + S004 均不拦 strict）
→ gen/front/vue `pnpm install` exit 0 → `pnpm build`（vue-tsc + vite）
exit 0（9.74s）。收据：/tmp/v01-trace.log（本机易失；要点入账：59m
墙钟内含 39m 生成 + pnpm 两步）。此前两次"静默 abort"（26m33s exit 1）
未复现——失败运行用的是旧 CLI（gc8f86ef/e2deb4f 代），本轮 CLI 含
G-7/G-8 修复；abort 归因不能精确到单一修复，按"门现绿"记账，观察项
保留（若复现再以 AUTO_BUILD_TRACE=1 定位）。

**S001/S004 漂移分类**（51 组 Info，不拦 strict，非阻塞）：
- Markdown(source:) 为 use.web 组件导入（ports/renderer.at），验证器
  误配 schema autodown 标签——验证器白名单类改进项（auto-lang）。
- button/span 的 title、Check.size 等：stdlib 控件定义无此 prop，
  musk 侧为透传/无效属性——逐条判定后 musk 对齐或 schema 手术式补充
  （破坏性全量再生已否决：diff -219 行丢 Plan 注记/默认值/语义枚举）。

**剩余**：G-9 残余（§8.7 证据包）。V05 最小四模式的 VM 臂两模式已绿
（15×2）；Vue 臂两模式现可回补（V01 已通，gen/front/vue 就绪）——
T-13 全量矩阵范围内执行。

### 8.9 G-9 残余正修完成（2026-09-30 第四轮，auto-musk-dev@957543acb）

内容指纹探针（SET_FIELD 写前对象现值）钉死最后疑点：**续体写全部落在
正确的状态对象上且序列单调**（boot→poll→polls=1→r_start→source→
r_source→done），VM 堆完全健康——问题 100% 在读取面：MCP 快照/视图
同步被 view() 的 view_dirty 门跳过（PLAN-062 语义：视图不脏=快照仍准），
而 park/resume 完成路径只抬 component.dirty、不抬 app 层 view_dirty
（普通事件臂都抬，唯独 __parked_resume_tick 臂遗漏）——脏旗断层。
视图模型绑定子节点消失 = 首帧同步的 vtree（PLAN-633 挂载帧同族：Init
未写时的空值面）被门永久冻结。

修复（一处一行级 + 一处加固）：
- renderer `__parked_resume_tick` 臂：poll_parked_resumes 返回 mutated，
  完成时同步抬 state.app.view_dirty → 同步门重开，快照/视图随写刷新。
- 段模式 http 等待 30s 上限（AutoTask.waiting_http_since 打点 ×15 置位、
  ×15 not-ready 重入出口统一超时）：结果丢失不再无限 rewind+re-park
  自旋，超时回收并走 [VM-HANDLER] failed 错误面。

验证：自然条件形态（生产 store 写法，哨兵撤除）探针 15/15 × 双后端
（VMHTTP/RustHTTP）× 3 连跑确定绿；MCP 读回实时（polls=97/r_source=ok
全程可见）；cargo tv 162/162。G-9 全链闭环，探针即回归哨兵。

**五项上游缺口全部闭环**：G-7（Use 前置，c35a55ca8）、G-8（三根因，
c93ed76a0）、G-9（脏旗断层+等待上限，957543acb）、G-10（随 G-9 消失：
快照刷新后 MCP 读回实时）、V01（门通过，全链 exit 0）。S001/S004 漂移
分类入 §8.8（Info 级非阻塞，判定项后续逐条处理）。

### 8.10 V05 四模式矩阵全绿（2026-09-30，T-03 验证门达成）

| 前端 | 后端 | 断言 | 收据 |
|---|---|---|---|
| VM（iced，自然条件形态） | VMHTTP | 15/15 | ports-vm-receipt.json |
| VM（iced） | RustHTTP | 15/15 | ports-rust-receipt.json |
| Vue（dist+playwright 真面板） | RustHTTP | 8/8 | ports-vue-rust-receipt.json |
| Vue（dist+playwright 真面板） | VMHTTP | 8/8 | ports-vue-vm-receipt.json |

Vue 臂断言链：主壳渲染 → runner 启动真实目标 → 帧 img 真渲染
（naturalWidth>0 = 浏览器解码真实 PNG）→ 树选 UI 点击经 canvasPickNode
回流后端（status.picked 非空）→ 带代次停止 → 面板收起。工程注意（探针
可复用件）：playwright keep-alive 连接需 closeAllConnections 强断
（libuv UV_HANDLE_CLOSING 断言）；当前构建壳不强制登录（080 登录流已
不在默认流上，探针等主壳"会话"渲染即可）。

T-03 完成门：V01 全链 exit 0 ✓ / V07 contract 19 绿 ✓ / V05 最小四
模式 ✓ —— **达成**（canvas_live 真目标生命周期回归按计划归 T-12）。

### 8.11 T-04 双端验证完成（2026-09-30）

- VM 臂（MCP 驱动 musk 前端）：开合投影/帧 URL 双键 ?t=15&gen=1/
  后端清选 3 拍内 picked 清空/收起 12+ 拍不复开（生命周期 running
  不受影响）/expected_generation 重启 gen=2 清场且偏好保持/UnCollapse。
- Vue 臂 9/9 × 双后端：新增 vue-collapse-persists（真实收起钮点击 →
  帧隐藏 → 5s+ 多拍仍隐藏）。
- 补齐缺口：web 门面 canvas.web.at/canvas_web.ts 导出 canvasClearPick
  （vue-tsc TS2305 实证）；面板 Collapse/UnCollapse 消息路由（视图
  onclick 解析到面板命名空间——生成器对未在 on-block 声明的 msg 产
  空桩（"TODO: handler not defined in on-block"），修正为显式转调
  store.Collapse()/store.UnCollapse()）。
- MCP 驱动备忘：store msg 经 autoui_action drag 派发，widget=CanvasStore、
  分隔符为真实 U+001F（可见符号 ␟ 无效）。

### 8.12 T-05 增量记录（2026-09-30）

第一增量（42c8352，VM 臂验证）：canvas_panel 三件套拆分 + studio 开关 +
视口宽通道 + 布局条件。第二增量（527d99c，VM 臂验证）：面板槽三分支
（工作台两列/空工作台/原面板）+ 空工作台与打开已有应用流 + view 闭括号
修复（chats_view 解析失败 20 错——面板槽重排时 view 闭括号缺失，同类
失衡复查建议纳入编辑清单）。

**新发现（待查）**：T-05 改动后的全量生成静默 exit 1（28m04s，无诊断;
AUTO_BUILD_TRACE 未设），退出点紧随 canvas_helpers.at 的 fn-only 警告
——下一文件即拆分后的 canvas_panel.at（3 widget 共声明同一 store +
CanvasPanel 无 msg 块）。前一轮全量（T-04 改动后、拆分前）exit 0 通过，
拆分结构是首要嫌疑。dist 现状 = 上一完整生成的 T-04 版（一致可回滚）。
下一步：AUTO_BUILD_TRACE=1 复现定位 + canvas_panel.vue 生成结构检查;
短期回滚面 = git revert 拆分提交恢复单 widget 形态（studio 布局改走
ChatsView 内联分支）。

### 8.13 测试门禁基线同步 + T-05 Vue 侧阻塞解除（2026-09-30 第五轮）

**门禁基线**：auto-lang worktree rebase 至 master c80887ab7（fix-ui-tier
测试改版；1 冲突 task.rs：PLAN-707 流字段 vs G-9 since 字段——两者都
保留），rebase 后 cargo check ✓、cargo tv 162/162（新档 1.9s）。

**"静默 exit 1"定性翻案**：fix-ui-tier 的 ui_gen 去平方优化随 rebase
生效后，musk 全量生成 39-28 分钟 → **23-31 秒**（62 组件 exit 0）——
此前 28m 处的"静默退出"= 旧平方路径慢到环境终止，非独立缺陷；
AUTO_BUILD_TRACE 逐文件追踪 + RUST_BACKTRACE 均未再捕捉到异常退出。

**Vue 侧两个真实缺口（构建暴露，已修）**：
- viewport_sync 导出名 ≠ use.web 声明名 → ext re-export 链 TS2305
  （pnpm 自 T-05 起持续失败、dist 滞留 T-04 版的真因）。修正：导出
  改名 syncViewport 与声明对齐（plan-037 门面口径：TS 导出名 =
  use.web 名）。
- chats_view 面板槽重排 view 闭括号缺失 → 解析失败 20 错。已修。
- 另：studio 模式跨 store 写入（ForgeStore 调 CanvasStore.EnterStudio）
  运行时 TypeError——模式归位 ChatsView 视图域，CanvasStore 的
  cv_studio/Enter/Exit 退役（71c302b）。

**验证**：Vue 臂 9/10→修复后 studio 断言细节完善中；VM 臂 studio 全
周期绿（进/保持/出）；V01 生成门 23.4s exit 0 + pnpm ✓（新基线复验）。

### 8.14 T-05 完成轮：Vue 侧收口 + §5.2 宽度分层全档（2026-09-30 第六轮）

**T-05 第三增量（c9037c6）**——三处生成/接线缺口收口：
- 工作台槽归位 CanvasStudioSlot（canvas_panel.at 新 widget，CanvasStore
  域）：chats_view 原分支 `if .store.cv_open` 读的是 ForgeStore（字段
  不存在 → undefined 恒走空工作台，studio 双列不渲染——Vue 臂 335px
  取证 + 生成 ChatsView.vue:604 实证）。空工作台输入/启动流随迁
  （model 本地态 path/err——gate_card 先例；canvasStart 在 widget 动作
  域，store 文件不引 web 端口）。
- 入口钮移 ContentHeader 常驻 actions 行：原在 `if .info_open` 信息条内
  （须先点 ℹ 才渲染——"回 527d99c 重做第三步"时该迁移被丢弃，本轮补
  落），样式族对齐行内钮（h-7 w-7）。
- NavSidebar 宽度单源：折叠 w-12 与展开 width_class 双类并存（VM 轨
  style 内联可胜，web 轨级联 240 压 w-12 → studio 强制收起失效，实测
  工作台被挤到 556px）→ computed sidebarClass 按态全串（app 轨
  railClass 同款；widget computed 先例 WorkspaceSelector；`.width_class
  + " ..."` concat 生成器支持实证）。
- 验证：Vue 臂 10/10 × 双后端 ALL PASS（studio=527px > normal=349px）；
  VM 真机 studio 全周期 MCP 驱动绿（进：侧栏 240→48 + 空工作台元素；
  保持 8s+ 快照稳定；出：恢复 240）——脚本 tmp/vm-studio-cycle.mjs。
  VM 快照 label 含 PUA 图标字形前缀（U+EE03+"应用设计"）——精确等值
  匹配恒假，须 includes（探针兼容口径）。

**T-05 第四增量（3de5254）**——§5.2 宽度分层全档：
- 对话右置：内容域包装 col + 可反转内容行（studio → flex-row-reverse，
  VM Plan 412 支持）——聊天列原位单实例不重挂（草稿/焦点/展开态结构
  性保持），对话列 [结构][画布][对话] 右置达成。
- 分层：对话 360(≥1280)/320；结构 220/200；768-1023 结构列受控显隐
  （cv_structure_open 偏好 + CanvasStudioSlot 左缘切换钮列）；<768 页签
  画布/对话/结构（studio_tab 视图态 + 塌缩类 w-0↔flex-1 切换——VM 无
  hidden 类，v-if 重挂被单实例原则排除）。
- 驱动源：ForgeStore.fw_win_w（PollStream 随拍，canvas_store Poll 同款
  读法）+ CanvasStore.cv_win_w 既有——两域各自消费同一 localStorage 键。
- **生成器语义实证（重要）**：`style:` 字面量 → class 属性；内联 if →
  :class 三元；`style: [x]` 数组 → :class join；**字段引用/concat 直形式
  → :style 内联（类名串无效，布局塌）**——本轮三处改数组形式修复。
- 验证：宽度分层取证 4 档全符合 §5.2 预算（tmp/tier-forensic.mjs 收据
  tier-results.json）：1440=[360,968]/img747、1100=[320,668]/img467、
  900=钮列36+img431（钮后 +结构200 → img231）、700=页签3+对话页签
  [588,0]塌缩，全档零横溢出。Vue 臂 10/10 × 双后端（studio img 587）。
  VM 真机全周期 3 轮绿。草稿保持取证 ✓（composer 草稿跨三次模式切换
  原样，tmp/draft-persist.mjs）。

**T-05 遗留（记录不阻断）**：VM 快照 rect 搅动期部分缺 bounds（080 已
知通道缺陷），VM 侧右置几何以渲染器支持（renderer.rs:2421）+ 功能周期
（3 轮）+ Vue 侧几何证据背书；输入焦点在模式切换瞬间重置（唯一 DOM
本地态损失，草稿/展开/审批卡均 store/model 域保持）——如需可后续
autofocus 恢复，非 AC 阻断。

### 8.15 T-06 第一增量：内容包装层 + 点选单源 + 门控守卫（2026-09-30 第七轮）

**落地（601f3a9）**：
- 内容包装层 cv-frame-wrap（canvasContentStyle：fit=aspect-ratio 定比
  盒、100%=帧物理 px 定尺寸+容器 overflow-auto 内滚）——覆盖框百分比
  与点选换算统一坐标系，修 object-contain 留白期选框相对容器错位
  （bbox_pct 以帧物理尺寸为基的语义终于有对应盒）。
- 点选映射单源 canvasMapPhysical（纯 int 乘前除后）：经 use.web.fn
  生成 ext/src/front/canvas_helpers.ts（forge_helpers 同链实证——
  use.web.fn 声明面即可生成 helper 文件模块），canvas_web.ts 点击
  委托消费同一规则；VM 侧后续坐标点选消费同源。
- 委托收敛守卫化：身份（cv-frame-box/-wrap class 命中）+ 版本
  （capture load 打标 dataset.cvLoadedSrc——旧帧迟到加载完成不切回
  旧代次响应）+ 几何（盒内点选/留白出界=显式 {clear:true}）。
- 帧尺寸通道：status.frame（后端 T-02 既有）→ cv_frame_w/h 随拍回填，
  免 web 加载事件；代次清场连带帧尺寸归零。
- **接线回归修复**：installCanvasFrameClicks 自 71c302b 起无人调用
  （移除面板侧两次 setup 调用时"移到 ChatsView"未落）——画布直点
  链路中断两轮未察觉（树选链路独立存活掩盖）；ChatsView setup 承接
  （先于帧 img 存在，capture load 打标不缺拍）。
- **委托命中面实测修正**：留白点击目标是容器而非 img（旧
  closest('img') 早退使清选分支不可达）——按容器命中即处理重构。

**验证**：几何取证 6/6（tmp/geom-forensic.mjs：A 定比包装层/B fit
直点命中/C 留白清选/D 1:1 精确尺寸+内滚/D2 1:1 同链直点/E 回归）。
方法教训：probe-a 活 fixture 帧间内容会动（同物理点 seq5 命中、seq6
204）——点选类断言须锚定当前帧（树选取 picked.bbox 中心）而非固定
坐标；picked 的框字段是 bbox（bbox_px 在 overlay 条目上）。
双臂回归：Vue 臂 ALL PASS × 双后端；VM studio 周期 ALL PASS。

**T-06 剩余**：VM 侧坐标点选消费（同源映射的 VM 臂）、V03 helper 边
界测试（canvas-contract.mjs 新建）、V05 几何子场景（DPI 1/1.5/2、两
种留白、边界点/滚动、框≤2px 精测）、loaded 门控的运行时负向用例。

### 8.16 T-06 第二增量：V03 边界测试 + V05 几何子场景 + G-11 上游缺口（2026-09-30 第八轮）

**落地（本轮）**：
- **V03 canvas-contract.mjs（新，21/21 绿）**：被测对象=生成的 ext 生产
  模块（不复制算法）——canvasMapPhysical 退化/边界点(0,0 含、box_w/
  box_h 严格不含)/负偏移/整除截断扫描（maxErr=0，<2.07px 预算）/
  DPI 无关契约；canvasContentStyle 双模式+退化；帧 URL 双键；picked
  投影 7 字段（forctx.index→"#N"）；overlay 重建+不拦指针+蓝/琥珀分色。
- **V05 几何子场景**：DPI 1/1.5/2 锚定直点三档全命中（映射输入为
  CSS 量，deviceScaleFactor 不进公式——tmp/dpi-gate-forensic.mjs）；
  两种留白——纵向（normal 349×764 盒，geom A）+横向（studio 短窗
  587×584 盒，wrap 412 窄于容器高占满，tmp/letterbox-forensic.mjs
  H/I/J 全绿，换算亚像素 239.42≈240）；版本门控负向用例——篡改
  dataset.cvLoadedSrc 为陈旧值 → 内容盒点击 0 请求（守卫拒绝实证）。
- 锚内个别节点 bbox 缺失实测（根 col 无 bbox、text 子节点有——
  pick_json 对 n.bbox=None 的节点省略 bbox 字段，anchor.rs:210）——
  取证锚定遍历多个树钮。

**G-11（新上游缺口，阻断 VM 消费面）**：**VM 轨 widget→widget 子件
实例化缺面**——musk 真实面板带活会话在 VM 上首次演练（本轮空工作台
启动流 targets/probe-a 拉起 cv_open=true）发现：CanvasStudioSlot/
CanvasPanel 的结构列/画布列子件整体空渲染（列壳样式在、子件内容
零节点），normal/studio 双模式一致；view→widget 正常（壳层全量渲染
对照）。canvas_panel.at 三件套自 PLAN-088 拆分起 VM 轨未带活会话
演练过（T-04 VM 验证走独立 fixture probe-c），缺口潜伏至今。同文件
widget 视图引用兄弟 widget（无 use 行）疑为触发面。修复在 auto-lang
（VM aura builder / codegen 子件解析），本仓边界外。证据快照
tmp/vm-studio-cycle/{wrap-missing,normal-panel}-snapshot.txt。
VM 消费面（同一 .at fn 在 VM 解释器下的样式产出）待 G-11 解除后
补跑；VM 空工作台启动流本身 ✓（type path→启动→cv_open 翻转实证）。

**验证汇总**：V03 21/21；DPI 三档全命中；门控负向 ✓；双留白几何+
直点+清选 ✓；Vue 臂 ALL PASS × 双后端（回归）；VM 功能周期 ALL PASS
（studio/侧栏/启动流）。

### 8.17 T-07 第一增量：结构列页签化 + 折叠树 + 源码面板（2026-09-30 第九轮）

**落地（59eb61a）**：
- canvas_structure.at（新）：CanvasStructureColumn 页签化（结构/源码，
  抽屉退役）；折叠层树（折叠符/节点分行，vid 稳定折叠键，可见行
  store 单源重算）；页签切源码按需加载（state=='' 才拉，失败落
  error 面）。
- canvas_source.at（新）：CanvasSourcePanel 状态机（''/ok/error）+
  行对象渲染（行号列+拾取行高亮+越界提示）；只读红线维持。
- canvas_helpers.at：canvasTreeVisible（前序扁平→可见行，lastAt
  深度祖先链，obj 字符串键）+ canvasSourceLines（char_code_at 10
  切行）+ 投影补 source_line（8 字段）。
- canvas_store.at：折叠键/页签/源码状态机七字段 + ToggleTreeNode/
  SourceTab/SourceLoadFail；代次清场连带折叠键归零。
- canvas_panel.at：结构列迁出（use canvas_structure——跨文件 use 行
  形态）。

**本轮修复的三个跨层缺陷**：
1. **G-12（auto-lang，已修 2327e0bba）**：store composable 发射器
   不注入 range 助手——store 本体动作与 use 导入 fn 内联两路经
   range-for 均产出未定义调用（TS2552）。对齐 PLAN-055 fn 模块先例
   补注入；同仓收编 AUTO_BUILD_TRACE 追踪暂存。
2. **canvasLoadSource 双轨形态缺陷（已修）**：web 侧 raw query 形态
   在 VM serve 不可用（VM serve 路由装配只含 ag 参数路由
   /api/files/{workspace_id}/{*path}，hw files 三路由未挂载，落 ag
   后 200 "null" 兜底体冒充内容实测）——改 workspace 路径段形态
   /api/files/{ws}/{path}（canvas.vm.at 同款，双轨通吃）。
3. **解析契约新证**：①视图体内 let 不支持（R016 垃圾节点——行对象
   改 store 域预构建）；②text 拼接须字符串字面量开头（模型引用
   开头泄漏垃圾节点）；③字段名不得撞元素硬关键字（row.text→body）。

**验证**：V03 37/37（新增折叠/has_kids/同 vid 复用/行切分五态/投影
8 字段）；T-07 取证 4/4 @Rust 后端（页签默认态/折叠跨轮询存活
4→1→4/源码面板 262 行+拾取行 44 高亮+越界与提示面）；Vue 臂
ALL PASS × 双后端。

**G-13（新，musk 后端本仓，T-07 剩余）**：VM serve（vm_backend::
serve，vm_entry.at 路由）**漏挂 hw files_browser 三路由**——/api/
files/* 全部落 ag 参数路由，VM 转译 handler 的 registry 状态桥缺失
→ files 域恒 200 "null"（树/文本/裸读全形态实测；Rust serve 对照
4887 字节真内容）。修复模式 = canvas 域先例（T-03：宿主 HostCall
桥 + vm_entry.at 路由行，vm_backend.rs:289）。VM 臂源码面板/坐标
点选消费与 G-11 同挂此后补。

### 8.18 G-13 修复：canvas 域源码只读通道（2026-09-30 第十轮）

**修复（9819f0b）**：/api/canvas/source?workspace=&path=（双 serve 同
契约）：
- canvas_vm.at：canvas_vm_source 路由 + dispatch（Query Value 编组——
  canvas 域已证通道；Path 元组编组丢段实测 [p]→["ws"，第二段丢失]）。
- extern_impl.rs：canvas_source_host——registry.get(workspace).root +
  read_text_confined（confinement/lossy 同款）→ text/plain 整包响应
  （i64 句柄经 insert_http_response，canvas 同型）。
- vm_backend.rs：注册（**须带 ？ 解包**——漏 ？ 时 enc(Result) 把
  {"Ok": 句柄} 整包当响应体泄漏，200 假成功实测）。
- canvas/mod.rs：hw canvas_routes 补同名路由（Rust serve 同语义直调）。
- canvas.vm.at / canvas_web.ts：源码读取切 query 通道。

**根因链定稿（G-13）**：VM serve（vm_backend::serve → vm_entry.at）路
由装配含 ag 参数路由 /api/files/{workspace_id}/{*path}，**不含 hw
files_browser 三静态路由**（Rust serve 同链有——对照实证）→ /api/
files/* 全落 ag workspace_file，VM 转译 handler 的 registry 状态桥缺
失 → files 域恒 200 "null"（含必然 404 的缺失文件路径——应答者非
files_raw 判别法）。修复面=canvas 域绕行（前缀零冲突）；/api/files
域状态桥归 PLAN-044 follow-through（登记 095 候补）。

**验证**：VM serve curl（真文件 200+487B 真内容、缺失 404 诚实错误
JSON）；T-07 取证 ALL PASS 双后端（VM 后端 262 行+行 44 高亮；Rust
后端网络观测确认新通道 200 真内容）；Vue 臂 ALL PASS × 双后端。
取证方法论：拾取重渲染期的页签点击可能落空（点击后以面板出现为准
重试，≤5 次）；宿主闭包 Result 须 ？ 解包（否则句柄泄漏假 200）。

### 8.19 T-07 第二增量：源码面板行自动定位（2026-09-30 第十一轮）

**落地（06c2a9d）**：installCanvasSourceScroll（canvas_web.ts）——
MutationObserver 监听行渲染（拾取/载入/切页签统一触发面），高亮行
（.cv-source-hl）offsetTop 变化时容器 scrollTop 居中；whitespace-pre
行高恒定 → offsetTop 去重（流式聊天重渲染不扰动）。行容器
cv-source-rows relative（offsetParent 锚）+ 高亮行标记；ChatsView
setup 安装（install 家族幂等）；VM 轨空桩（iced 滚动自管理）。

**验证**：T-07 取证 5/5——新增 E 行定位：拾取行 44/262 → scrollTop=345
= offsetTop 709 − 半容器 364（精确居中）；Vue 臂 ALL PASS × 双后端。

**T-07 剩余（收敛）**：多文件不确定提示（Q-04 confidence 契约——
pick_json 无置信标记，缺数据待后端契约）；G-11（auto-lang
widget→widget，095 T-07）解除后 VM 轨（iced）面板本体渲染与坐标
点选补跑。两项均不阻断 Vue 轨交付面。

### 8.20 T-08 第一增量：元素附件 chip + design_context 冻结/校验/落盘（2026-09-30 第十二轮）

**落地（ee411ef）**：
- **跨 store 依赖首用**：chats_view.at 双 store 挂载（PLAN-048——第二
  store 发独立 facade canvasStore；Vue 轨生成实证，VM 轨待 G-11 后
  观察）。附件 chip 实时读 canvasStore 拾取面（无复制态——A→B 改选
  自然反映），发送时冻结。
- **chip**（composer 上方）：已选 kind·label + 在对话中修改（聚焦现有
  composer 不发送不改文本——platformFocusComposer）+ 移除（后端
  {clear:true} + 投影清双面——仅本地清会被轮询回填实测）。
- **冻结语义**：SendInput 命令解析确认为普通消息后冻结（斜线命令/
  mention/IME 不受上下文影响）；快照字段 = 设计 DTO 全集
  （version/workspace/conversation/generation/frame_seq/app_path/
  vnode/kind/label/source_path/source_line）。
- **队列逐条冻结**：条目对象 {text,ctx}；FlushQueue busy 原样退回
  （不读"当前选择"）；stale 409 整条退回队首 + 错误面提示（不自动
  误发）。
- **后端校验/落盘**（chats_message）：字段面校验（vnode_id/kind 非空
  否则 400——用户文字不落盘）；归属盖章 ownership
  no-canvas/current/stale（stale=canvas 活动代次与快照不符 → 拒收
  不落盘，响应 stale:true 供队列退回）；ChatMessage.design_context
  serde default（旧会话文件默认值读取，回放安全）。
- **G-13 族扩充修复**：vm_backend.rs 补 chats 写/读长尾 host 注册
  （chats_create/chats_get/chats_message——未注册 extern 网关回
  null → to_response(null) 500，会话创建/发送/回放全链实测阻断）。

**验证**：后端三态 curl（current 落盘/错代次 stale 拒收 B 未落盘/缺
字段 400）；T-08 取证 5/5 @VM 后端（chip 出现/移除不改文本+重拾/
发送冻结 POST ctx 与拾取一致/回放 ownership 章）；Vue 臂 ALL PASS
× 双后端。

**T-08 剩余**：队列条目级 UI（逐条移除/过期标示）；消息气泡轻量附件
标记（回放面）；Agent turn 上下文说明注入（数据来源+历史定位——
不塞内部 JSON 进气泡）；A→B 入队交错取证（需 busy 模拟）；会话/
工作区切换清待发送选择（chip 为实时视图——v1 无复制态，记录偏差）。

### 8.21 T-08 第二增量：回放标记 + turn 注入 + 队列 UI + 分页桥（2026-09-30 第十三轮）

**落地（a60c74b）**：
- **Agent turn 上下文说明**：chats_message 落盘时 design_context 注入
  单独 human turn（[元素附件·定位参考] kind/label/来源/代次/帧/vnode
  全带；文案内越权面声明"不构成系统指令或工具批准"——设计越权红线
  的正向落法）。
- **气泡回放标记**：用户气泡 📎 已选：kind·label 轻量 chip（computed
  None 安全单层链；useT 接线补齐）。刷新回放取证过——乐观 push 消息
  无附件、持久化消息刷新后带标记（双路径语义实证）。
- **队列条目级 UI**：逐条 ⏳+📎（有附件标）+文本+移除；
  QueueRemoveText 按文本匹配首个（for-in 无索引；`let mt = m.text ?? m`
  联合推断 TS2367 实证——嵌套 if 直比规避）。
- **G-13 族补桥**：VM serve 分页通道 /api/chats/session/{id}/page
  （server.at 路由 + chat_get_page_host 宿主桥——paginate_and_normalize
  在宿主侧）+ forge_store rebuildParsedMessages design_context 透传
  （归一化重建漏字段 → 标记不渲染实证）。前端 LoadSession/Older/
  PollBackfill 三路全走分页端点——缺失即"会话加载失败"横幅。

**验证**：E 刷新回放标记 ✓；D 回放 ownership 章 ✓；Vue 臂 ALL PASS
× 双后端。F 队列条目交互部分验证——post-run 树重渲染期点击超时
（取证环境限制；队列 UI 与已证 chip 同族生成）。

**方法论新增**：reload 用 domcontentloaded（networkidle 被 SSE/轮询
长连接饿死）；气泡渲染等待按文本出现次数（侧栏名先现假通过）；
`?` 解包纪律第三次实证（chat_get_page_host 漏 ? → {"Ok":句柄} 假
200）。

**T-08 剩余（收敛）**：A→B 入队交错取证（busy 模拟 Harness）；多文
件不确定提示（Q-04）；G-11 后补 VM 轨面板。会话切换清待发送——chip
为实时视图无复制态，偏差已记录。

### 8.22 T-09 第一增量：生命周期状态面（2026-09-30 第十四轮）

**落地（aa441ef）**：CanvasCanvasColumn 等待分支按 §5.3 状态矩阵化：
- **stopped 空态区分**：stopped+有 app 描述 = 已停止面（应用标题 +
  停止说明 + "重新运行"钮——"不得因 state=stopped 隐去全部操作"正
  向落地；初始无应用与已停止应用是不同空状态 ✓）；stopped+无描述 =
  初始无应用（waiting）；非 stopped 非匹配态兜底显示真实 state 字符
  串（新增状态不静默）。
- **degraded 面**：红条升级——简短原因 truncate + 详情展开 toggle
  （cv_error 全文 mono 滚动盒，视图态 cv_error_details_open）+ 红条
  内直接"重新运行"钮。
- **RerunApp**：canvasStart 走既有校验与启动流；失败落 SetCanvasError
  （下一拍轮询以真实状态覆盖——前端不新增重启循环 ✓，沿用后端
  1/2/4s 最多三次策略）。

**验证**：T-09 取证 2/2 @VM 后端（A stop 后已停止面三要素全在/B 重
新运行离开 stopped → running 帧回归）；Vue 臂 ALL PASS × 双后端。

**T-09 剩余**：①degraded 故障注入取证（spawn 失败重试耗尽路径——
坏 pac/坏 app 构造，红条+详情+重运行动作面已就位）；②生成/检查进
度行（工具事件投影——chat 消息 tool_calls → 生成/检查状态行，跨
store facade 读法或后端投影面，增量二）；③原始日志 tail 后端暴露
（session output_tail 未进 status_full）。

### 8.23 T-08 A→B 交错取证：Harness 限制定案（2026-09-30 第十五轮）

**尝试与发现**（tmp/t08-interleave.mjs）：
- A/B 拾取双点可达（root col 与 text 子节点 vnode 可异——text 钮按
  label 子串定位）。
- **流式期 composer 禁输**（MentionInput disabled=store.streaming →
  textarea disabled → focus/type/Enter 全失效——黑盒无法在流式期
  键入）→ 队列入队窗口 = 发送竞态秒级（真实产品 UX 即此），black-box
  无法稳定驱动 A 入队→改选 B→B 入队的交错序列。

**已验证面**（等价覆盖）：逐条冻结语义 = 双证——①单发冻结（t08 取
证 C+D：POST ctx 与拾取一致 + 落盘带章）；②A→B 直接双发各携各的
拾取快照（冻结时点即各自 Send 时刻，本轮 A=root POST ctx=root 实证
+ B=text-child 拾取异 vnode 实证）。队列条目结构 {text,ctx} 入队即
冻（Store.QueueWithCtx 代码面：条目持独立 ctx 对象，拾取变更不触及
已入队条目）+ FlushQueue 读 firstObj.ctx（代码面）。

**剩余**：busy 态 Harness（模拟 streaming 无真实 run——测试基建件，
归 T-11 V03 汇总口径）；队列条目级 UI 交互断言同族（post-run 树重
渲染期点击超时，F 部分验证）。
