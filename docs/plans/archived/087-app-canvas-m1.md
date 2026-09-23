---
plan_id: PLAN-087
status: archived
feature_name: App Canvas M1——实况画布最小闭环
author: [agent]
created_at: 2026-09-22T17:20:01Z
updated_at: 2026-09-23T03:30:00Z
plan_revision: 1
current_step: 9
total_steps: 9
supersedes_spec_components: []
new_spec_components:
  - docs/specs/modules/app-canvas.md
touched_goals: []
---

# PLAN-087 — App Canvas M1：实况画布最小闭环

## 0. 变更摘要

落地 `docs/designs/013-ai-app-studio-canvas.md` 的 **M1 里程碑**：musk 获得"App Canvas"
能力——一句话生成 002-counter 级简单 Auto app（workspace 内 pac.at+app.at，模板实例化
优先），chats 视图内嵌实况画布面板（隔离 `auto run --render=vm` 子进程 + AutoUI MCP
截图流），agent 以 `canvas_*` 工具驱动界面并做状态断言，形成"生成→实况→驱动→验收"
最小闭环。五块工作：canvas 会话管理器（新造，最重）、API 面、agent 工具、前端面板、
生成侧模板池；其中进程编排经验全部来自 PLAN-080 `scripts/ui-parity/live.mjs` 的
Rust 化复用。实施走 worktree `.wt/musk-087/auto-musk`（分支 `plan-087-dev`，
AGENTS.md 规矩）。

## 1. 目标

一句话 demo 全链路：chat 输入"做一个计数器" → workspace 生成 app → 画布自动展开
实况显示 → agent 点 +1 并断言计数递增、截图进对话 → 改 app.at 后画布热重载刷新 →
停止后零孤儿进程。

### 非目标（后续里程碑）

- M2 双向锚定：画布点选→源码、层树栏、`canvas.pick/overlay`、`source_map.rs` 填充。
- M3 三层生成流：bp 注册表/词汇表注入、L1>L2>L3 复用序、`ui.lint` 护栏。
- M4 飞轮：`blueprint.extract`、AppViewport 原生嵌入（路线 B）。
- 本计划内不做：多画布会话并发（恒单会话）、画布内嵌交互（用户点画布转发到 app——
  M1 画布对用户只读，驱动权在 agent）、VM 桌面轨面板的像素级 parity 验收
  （验收面=web 轨；VM 轨冒烟尽力项，见 §10）。
- 不改 auto-lang 主检出；若 VM 轨冒烟暴露 auto-lang 侧缺陷，登记 KNOWN-DEBT，
  按需另开同组 worktree（AGENTS.md 第三行），不并入本计划范围。

## 2. 架构方案

```
musk serve (axum)
 ├─ canvas/ 模块（新）backend/crates/musk/src/canvas/
 │   ├─ session.rs   spawn auto run --render=vm（cwd=app 目录）
 │   │               env: AUTOUI_MCP_PORT=分配 / AUTO_VM_WINDOW=固定
 │   │               / AUTO_VM_STORAGE_FILE=一次性 reset（080 参数）
 │   │               stdout 抓 "AutoUI MCP: listening on .../mcp"
 │   │               二进制解析序: AUTO_EXE env → 组内兄弟 ../auto-lang
 │   │               → D:/autostack/auto-lang 主检出 → PATH（live.mjs:296 已立 AUTO_EXE）
 │   ├─ mcp_client.rs  JSON-RPC over HTTP: initialize / autoui_screenshot
 │   │               / autoui_action / autoui_state / autoui_snapshot
 │   │               截图 base64→PNG 内存帧缓存（seq 单调）
 │   └─ manager.rs   会话状态机 starting/running/restarting/stopped/degraded；
 │                   单会话（再次 start=替换）；看门狗=帧拉取循环(~1s)+
 │                   连续失败/超时→重启退避；收割 taskkill /T /F（live.mjs:84 同款）
 ├─ server.rs 四路由: POST /api/canvas/start{app_path}（tool_safety::resolve_multi
 │   校验，fail-closed）/ GET /api/canvas/frame（PNG 直出）/ GET /api/canvas/status
 │   / POST /api/canvas/stop
 ├─ tools.rs 五工具: canvas_run / canvas_stop / canvas_snapshot（截图落 workspace+
 │   复用 display_image 的 /api/files 贴图路径，tools.rs:1171-1259 先例）
 │   / canvas_act（包装 autoui_action）/ canvas_state（包装 autoui_state）
 └─ src/front/canvas_panel.at（新）chats 视图右侧可开合面板：img 轮询 frame、
     状态条（running/restarting/stopped）、停止钮；auto build 重生成 Vue
```

生成侧（"一句话"能力）：

- **模板实例化优先**：musk 内嵌最小模板（counter/hello，从 auto-lang
  `examples/ui/002-counter`、`001-helloworld` 精简拷贝，剔除 `.am/`、`.auto/` 缓存），
  auto-lang `examples/ui/001..005` 作扩展池（同一 AUTO_EXE 式解析序，解析不到则降级
  内嵌池）。自由生成为兜底，不作为 M1 验收路径。
- **生成校验 = `auto ui inspect`（结构校验）+ canvas_run 启动即验收**：VM 轨对 .at
  直接解释执行，显示链不需要 npm 工具链；`auto build -r vue` 列为可选非门。
  这让 M1 的"生成成功"判据最轻：能启动、能截图、能驱动。
- **生成指导注入**：coding 模式上下文追加画布生成提示词（widget 词汇摘要 + 已知坑
  清单节选〔视图五坑〕+ pac.at 最小模板 + "先选模板再改造"策略）。

热重载：auto-lang VM 轨既有 `DynamicComponent` mtime 脏标（`ui/hot_reload.rs`），
canvas 只需持续拉帧即得"改完即变"，**零开发量**，AC-04 直接受益。

## 3. 技术栈

- 后端：Rust（tokio/axum/reqwest/serde_json），musk crate 内新 `canvas` 模块。
- 目标 app 运行：auto-lang `auto run --render=vm`（iced AutoVM）+ AutoUI MCP
  （`crates/auto-lang/src/ui/mcp_server.rs`，Streamable HTTP JSON-RPC）。
- 前端：Auto `.at`（真源 `src/front/`）→ auto build 生成 Vue；web 轨为验收面。
- 证据脚本：复用 `scripts/vm-mcp-census.mjs`（孤儿口径）、`vm-hangwatch.mjs`（挂起
  判定参考）。

## 4. 需求分析与背景调查

### 已获授权

- 用户 2026-09-23 对"为 Design 013 M1 立项（顺位 PLAN-087）"明确同意（"OK"）。
  范围=本仓 auto-musk；实施进 worktree `musk-087/plan-087-dev`（AGENTS.md 第一行
  命名）。未约定预算与自动续期上限；未授权改 auto-lang 主检出。
- 战略依据：`docs/designs/013-ai-app-studio-canvas.md`（本日已落盘，用户指示）。

### 背景证据（源码核实）

- **080 进程编排链**：`scripts/ui-parity/live.mjs:73-84`（spawn env 注入、
  `taskkill /T /F` 收割）、`:296`（`AUTO_EXE ?? 'auto'`）、844-874（隔离 env 全家：
  MUSK_CONFIG_DIR/AUTO_REUSE_BACKEND/AUTO_VM_STORAGE_FILE/AUTO_VM_WINDOW/AUTOUI_MCP_PORT
  与 stdout 抓 MCP 端点）。canvas 仅取 run 链，不取 parity 断言。
- **AutoUI MCP 工具面**：`auto-lang crates/auto-lang/src/ui/mcp_server.rs`——
  `autoui_screenshot`（iced 线程截屏 base64，最小化窗前置拒绝 :2220-2230——画布
  常态可见，不触此坑，但看门狗须把它当可恢复错误）、`autoui_action`
  （press/type/toggle/select_option/set_value/submit/scroll/key_press…）、
  `autoui_state`（字段过滤+路径后缀匹配）、`autoui_snapshot`（vtree+bounds+source）。
- **模板基准**：`auto-lang examples/ui/002-counter`＝pac.at 6 行（render:"vue"，
  window:"fit"）+ `src/front/app.at` 49 行；编号系列 001..020 在 `examples/ui/`。
  `auto run` 按 cwd 解析 pac.at；`--render=vm` 可覆写 pac.at 的 render 值。
- **沙箱契约**：`docs/specs/modules/workspace-sandbox.md`——`tool_safety::resolve_multi`
  两段式（绝对/已存在放行；新建归第一根），canvas.start/run 的 app_path 校验直接
  复用，报文列全部根。
- **MCP 采集语义先例**：`docs/specs/modules/ui-parity.md`（snapshot-ok、
  `AUTO_VM_STORAGE_FILE` 每跑重置、action 事件 spy 断言模式）。
- **热重载**：`auto-lang crates/auto-lang/src/ui/hot_reload.rs` + `DynamicComponent`
  mtime 脏标（view() 周期清）。
- **贴图先例**：`backend/crates/musk/src/tools.rs:1171-1259` display_image——图片落
  workspace 经 `/api/files/{ws}/{rel}` 以 markdown 图进对话；canvas_snapshot 同径。
- **M0 阻断**：主检出 musk build 被 auto-lang master 依赖漂移拉入 wgpu-hal 27.0.4
  不兼容阻断（PLAN-086 遗留）；固定位先例=`e120b2a`（组内依赖固定位构建的
  Cargo.lock 登记）。本计划 worktree 内解，不动主检出。
- **进程卫生教训**：`scripts/vm-mcp-census.mjs`（MCP 子进程泄漏普查）、
  080 KNOWN-DEBT（VM silent exit-1 家族）——看门狗与复活为硬性验收（AC-05）。

## 5. 详细设计

### 会话管理器（canvas/manager.rs）

- **单例态**：musk AppState 持 `CanvasManager`（`Arc<RwLock<Option<CanvasSession>>>`）；
  `start(app_path)` 时若已有会话先 `stop()` 再起（替换语义，§10 可否决）。
- **端口分配**：`TcpListener::bind("127.0.0.1:0")` 取临时端口后 drop 再经
  `AUTOUI_MCP_PORT` 注入（TOCTOU 竞态可接受：启动失败即报错重试一次）；避让 musk
  自身 VM 实例默认端口 9247。
- **端点发现**：子进程 stdout 逐行扫描 `AutoUI MCP: listening on`，超时（30s）=
  starting 失败→degraded 报错（不自动重试 spawn，避免坏 app 死循环重启）。
- **看门狗**：帧拉取循环 ~1s；`autoui_screenshot` 连续 3 次失败或单次 >10s →
  restarting：taskkill /T /F 收割→退避（1s/2s/4s，封顶 3 次）→ 重 spawn 同 app_dir；
  超限 → degraded（status 暴露，面板红条，stop/start 可恢复）。最小化窗拒绝文案
  归类为可恢复（下一轮重试），不触发重启。
- **收割**：stop() 与 Drop 双保险 taskkill /T /F + 端口探测确认退场；serve 关停钩子
  全量收割。

### MCP 客户端（canvas/mcp_client.rs）

- reqwest blocking 不用（tokio 生态）——async JSON-RPC：`initialize` 握手一次，
  后续 `tools/call`；`autoui_screenshot` 结果为 text_result(base64 PNG) → 解码入
  帧缓存（`(seq, bytes)`，seq 单调递增供前端判新帧）。
- `canvas_act` 透传 `UiActionType`（press/type/set_value/…）+ 目标选择器（M1 用
  MCP 现有目标语义，vnode id 或文本定位，T-06 落实具体形态）。
- `canvas_state(fields)` 透传字段过滤（counter 断言=`count` 字段）。

### API 与工具

- 路由挂 `server.rs` 既有 /api 族；`start` 请求体 `{app_path}`（workspace 相对或
  绝对），`resolve_multi` 拒绝→400+列根；frame 直出 `image/png`（无帧→503）；
  status 含 `{state, seq, app_path, restarts}`。
- 工具注册进 `tools.rs` 既有 per-mode 白名单机制（coding 模式可用）；`canvas_snapshot`
  = 截图存 `workspace/.canvas/snap-{seq}.png` + vtree 摘要文本 + display_image 式
  markdown 图；`canvas_run(app_path)` 内含路径校验（与 API 同口径）。

### 前端面板（src/front/canvas_panel.at）

- 挂 chats_view 右侧：有会话自动展开、stop 后收起；`img` src=`/api/canvas/frame?
  t={seq}`（status 轮询 ~1s 驱动 seq 变化重拉，避免无脑刷帧）；状态条三态文案；
  停止钮调 stop 后刷新状态。面板组件自身 ~百行级 .at。

### 生成侧

- 内嵌模板常量（counter/hello 两份完整 app 文件树，构建期 include_str! 或随仓库
  资源）；实例化=目录拷贝+pac.at `name/title` 与 widget 名参数化替换。
- 提示词片段进 coding 模式系统上下文：模板清单、"选模板→拷贝→改造→inspect→
  canvas_run"流程、词汇摘要（`schema/aura.at` 高频子集）、坑清单节选。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|:---|:---|:---|:---|:---|:---|
| SD-01 | add | docs/specs/modules/app-canvas.md | before：无 canvas 规范；after：app-canvas 模块规范——会话生命周期（start/替换/stop/复活）、AUTO_EXE 解析序、端口分配、看门狗与退避、进程卫生（census 零残留）、热重载预期（mtime≤5s）、安全边界（resolve_multi fail-closed）、API 四路由与工具五件契约、生成侧模板策略 | M1 为全新模块能力，merge 时需 canonical 契约承载（含后续 M2-M4 扩展锚点） | AC-01..06 |

## 6. 测试设计

- **集成测试（Rust，tests/ 或 canvas 模块内）**：拷 002-counter 副本→start→端点发现
  →screenshot 非空 PNG→`press` +1→`autoui_state` count 递增（对齐 ui-parity.md 的
  action+state 断言模式）→stop→census 口径零孤儿。
- **崩溃复活**：taskkill 子进程→≤15s 帧恢复→退避计数正确。
- **API 测试**：四路由 happy path；越界 app_path→400+列根（AC-06）；无帧→503。
- **热重载**：改 app.at 按钮文案→≤5s 帧变化（前后帧 PNG 对比，人工判读入证据）。
- **前端**：`auto build` 通过（.at→Vue 生成无错）；web 轨手动冒烟（面板展开/帧刷新/
  停止）。VM 轨冒烟尽力项（§10）。
- **端到端**：真实 chat"做一个计数器"全程走查，transcript+截图归档
  `docs/plans/attachments/087/`。

## 7. 验收标准

- **AC-01 一句话生成**：chat 输入"做一个计数器"→ workspace 内生成完整 app 目录
  （pac.at+src/front/app.at），`auto ui inspect` 无致命告警，canvas_run 可启动并出帧。
  验证：产物路径+inspect 输出+首帧截图。
- **AC-02 画布实况**：canvas_run 后 chats 画布面板自动展开，首帧 ≤10s；运行期帧
  持续刷新。验证：web 轨手动冒烟+相隔 ≥2s 两帧截图（seq 不同且内容一致）。
- **AC-03 agent 驱动+断言**：canvas_act 按 +1 后 canvas_state 返回计数递增
  （初值→+1 后值）；canvas_snapshot 截图以图片消息进入对话。验证：transcript 工具
  调用序列+截图。
- **AC-04 热重载**：修改 app.at 按钮文案后 ≤5s 画布帧反映变化。验证：改前/改后帧
  对比截图。
- **AC-05 进程卫生与复活**：stop/会话结束后 auto/MCP 子进程零残留（vm-mcp-census
  口径）；kill 子进程后 ≤15s 帧恢复。验证：census/任务列表前后对比+复活日志。
- **AC-06 安全边界**：workspace 根外 app_path 被 API 与 canvas_run 工具双面拒绝
  （fail-closed，报文列全部根）。验证：越界请求 400 与工具 Err 证据。

## 7.1 实施环境登记

- worktree：`D:/autostack/.wt/musk-087/auto-musk`（分支 `plan-087-dev`，基线=main
  `4a331ff`）；同组兄弟：`D:/autostack/.wt/musk-087/auto-lang`（分支 `auto-musk-dev`
  @ `b4960eb8f`）、`D:/autostack/.wt/musk-087/auto-ai`（分支 `auto-musk-dev` @
  `58bee8d`）——musk 以相对路径 `../../../../auto-{lang,ai}` 引依赖，组内兄弟是
  worktree 构建的硬前提（路径解析序见 AGENTS.md）。
- 主检出在途脏物登记（work 勘察 2026-09-23）：`backend/Cargo.lock` 未提交改动=
  iced 依赖树机械再生（PLAN-086 merge 备注"merge 阶段处置主检出在途脏 lock"所指，
  非手工 WIP）+ `backend/target-smokecheck/`（冒烟构建产物）+ `.zcodeignore`
  untracked。均不进本计划提交面；merge 阶段处置主检出时一并提请处置。
- auto-lang 主检出（`D:/autostack/auto-lang` @ `b4960eb8f`）有未提交 WIP：blueprints/
  目录成批删除（他方计划在途）。兄弟 worktree 自提交态检出，天然隔离，不受影响。

## 8. 执行步骤

- **T-01 M0 解锁与基线构建**（无依赖；全部后续前置）
  建 worktree `.wt/musk-087/auto-musk`（分支 `plan-087-dev`）；以 PLAN-086 固定位
  先例（e120b2a）锁 Cargo.lock 使 `cargo build --release`（backend/ 下跑）通过；
  备选=等 auto-lang master 稳定。产出：构建通过日志+解锁路径决策记录（若两条路都
  阻→升级 §10 阻塞项）。AC：前置。
  [x] 2026-09-23 work：**根因定案非"上游不兼容"而是锁偏斜**——musk 旧锁把
  gpu-allocator 的 windows 边钉 0.57.0（sysinfo 0.33 精确要求使 0.57 常驻锁），
  wgpu-hal 27.0.4 按 manifest 解析 0.58 → 同窗不同版类型劈叉（E0277/E0308
  suballocation.rs dx12 路径）。解法=`cargo update -p gpu-allocator@0.27.0
  --precise 0.27.0` 令其边重解析 0.58（两版共存合法）；lock delta +3541/-156
  （iced 0.14 图形栈全量登记）。兄弟位固定位：auto-lang@8fecfcf69（084 交付基线，
  **master b4960eb8f 的 dx12 编译臂在 musk feature 统一下仍红，且其自身 release
  deps 无 wgpu_hal rlib=默认构建根本不进该臂**）、auto-ai@57eb44a（086 固定位；
  **master 58bee8d 的 RoleConfig.models 字段=PLAN-034 实码与 musk 现码不兼容，
  留上游适配另案，§10-4**）。`cargo build --release` 通过（/tmp/p087-t01-build4.log，
  EXIT=0）。commit 8913acc。
- **T-02 会话管理器骨架**（依赖 T-01；新路径 `backend/crates/musk/src/canvas/`）
  session.rs+manager.rs：spawn/env 注入/端点抓取/AUTO_EXE 解析序/taskkill 收割/
  单会话替换。验证：集成测试 start→stop 零孤儿。→AC-05（部分）。
  [x] 2026-09-23 work：session.rs（spawn/解析序/双流端点抓取/taskkill+端口探测
  收割/Drop 兜底）+manager.rs（单会话替换、状态机、seq 帧缓存）落位；
  `canvas_session_lifecycle_drive_and_census` 全绿（spawn→端点→首帧 PNG 4465B→
  press→state→stop→tasklist census 零净增，7.99s）。commit 见 T-06 批注。
- **T-03 MCP 客户端与帧缓存**（依赖 T-02）
  mcp_client.rs：initialize/screenshot/action/state/snapshot；帧缓存 seq。验证：
  非空 PNG+press→state 计数联动。→AC-03 前置。
  [x] 2026-09-23 work：mcp_client.rs 按 §10-6 契约修正落位（截图=路径读回）；
  lifecycle 臂实测首帧 PNG 4465B→press "+"（ActionResult）→state count=1 断言
  过（vtree id 形态 `#vnode_<n>`→element_id `vnode_<n>`，parse_element_id 契约）。
- **T-04 看门狗与自动复活**（依赖 T-03）
  帧循环/失败判定/退避重启/degraded。验证：kill→≤15s 恢复。→AC-05。
  [x] 2026-09-23 work：帧循环+3 连失败/超时判定+退避复活（1s/2s/4s 封顶 3）+
  最小化护栏归类可恢复+迭代式重 spawn；`canvas_crash_revival_within_15s` 全绿
  （reap_tree_blocking 精确杀本会话 pid→恢复 running+帧增长，restarts=1）。
- **T-05 API 面**（依赖 T-02；改 `server.rs`）
  四路由+resolve_multi 校验。验证：curl 全路由+越界 400。→AC-02/AC-06。
  [x] 2026-09-23 work：canvas_routes() 四路由挂 serve()（dev_seed 后）+ctrl-c
  graceful shutdown 收割钩子；`canvas_api_out_of_root_rejected_with_roots_listed`
  常跑绿（越界 400 报文含 "allowed root" 列根+无帧 503）；happy-path 由
  lifecycle 臂（manager 面）+T-09 serve 冒烟（HTTP 面）覆盖。
- **T-06 agent 工具**（依赖 T-03；改 `tools.rs`）
  五工具注册+snapshot 贴图。验证：工具级冒烟（serve 内 chat 或单测）。→AC-01/03。
  [x] 2026-09-23 work：canvas/tools.rs 五工具（canvas_tool_registry 表）挂
  `build_agent_with_context` 白名单过滤（coding.at 收录五名）；路径校验与 API
  同口径（resolve_multi+validate_app_dir）；snapshot=PNG 落 `.canvas/snap-{seq}.png`
  + `/api/files` markdown 图 + vtree 摘要（2000 字截断）。注册链编译绿+
  lib 全套 467 绿；工具级真调由 lifecycle 臂等价覆盖（同 manager.with_client
  通道：press→state 断言过）。
- **T-07 前端画布面板**（依赖 T-05；新 `src/front/canvas_panel.at`，改
  `chats_view.at` 挂载）
  面板三态+帧轮询+停止钮+auto build。验证：build 通过+web 冒烟。→AC-02。
  [x] 2026-09-23 work：canvas_store.at（1s timer 轮询，Http.get_msg 回填，
  forge_store 同款）+canvas_panel.at（右列 380px：状态点/app 名/停止钮/
  degraded 红条/帧 img）+ports/canvas.{web,vm}.at 门面（VM 桩=面板不展开，
  whitelist VM 缺口同款）+chats_view 根行第三列挂载+i18n 五键。`auto build`
  全绿（vue-tsc+vite）；浏览器冒烟：面板自动展开+实况帧渲染+停止后收起
  （attachments/087/panel-right-column.png）；首轮挂错层（主列内）已修
  （cbbd84d）。
- **T-08 生成侧模板池与提示词**（依赖 T-06）
  内嵌 counter/hello 模板+examples/ui 扩展池解析+提示词注入。验证："做一个计数器"
  生成→inspect→可启动。→AC-01。
  [x] 2026-09-23 work：templates.rs 内嵌双模板（084 基线 examples 精简拷贝）+
  `generation_prompt()` 注入 build_agent_from_mode（mode.name=="coding" 追加，
  单测锚定插值）；扩展池缓行（沙箱 read_file 不可达仓外路径，§10-5）。
  AC-01 验证：模板实例化产物（smoke/ws/counter-app）inspect 臂由 `auto ui
  inspect` CLI 契约核实（告警不致命/解析失败非零退出）+canvas_run 可启动出帧
  （serve 冒烟 running+seq 增长）。**LLM 真聊走查未跑**（隔离 serve 无模型
  配置），证据边界=提示词注入单测+模板可启动，全自然语言臂留用户验收
  （review 可安排真配置环境补跑）。
- **T-09 端到端联调与证据归档**（依赖 T-01..T-08）
  AC-01..06 全走查；证据入 `docs/plans/attachments/087/`。→全部 AC。
  [x] 2026-09-23 work：走查结果（证据在 `docs/plans/attachments/087/`）：
  - AC-01 一句话生成：模板实例化产物可启动出帧（serve 冒烟 running/seq↑）；
    LLM 真聊臂留用户验收（见 T-08 证据边界）。
  - AC-02 画布实况：面板自动展开、首帧 <10s（seq=5@8s 内）、帧持续刷新
    （serve 日志 1s 轮询轨迹+frame?t={seq} 拉帧）；panel-right-column.png。
  - AC-03 驱动+断言：lifecycle 臂 press "+"→state count=1（vtree 定位
    vnode_id）；agent 侧五工具在册（聊天 transcript 臂随 AC-01 留验收）。
  - AC-04 热重载：改按钮文案 5s 内帧 md5 变化（hot-before/after.png，改前
    双帧 md5 全等对照）。
  - AC-05 卫生+复活：census 零孤儿（测试+serve 双口径）；kill→恢复
    restarts=1（revival 臂）；**serve 期天然复活一单**（VM silent exit 家族
    现形→看门狗 1s 判定+1s 退避重 spawn→面板无感，serve2.log）。
  - AC-06 安全边界：越界 400 列根（canvas_api 臂"allowed root"断言）+工具
    面 resolve_multi 同口径（Exec Err）。
  - VM 轨面板冒烟（尽力项）：canvas.vm.at 桩=面板按设计不展开（web-only
    门面），登记 VM 缺口非回归（whitelist VM 同款）；launch-vm 实机未跑。

## 9. 复审记录

- 2026-09-22 draft handoff：`stage: new`，PLAN-087，`plan_revision: 1`。
  `outcome: pass`（授权范围内可开工）。`next: work`（T-01 M0 解锁先行）。
  备注：草案含三处设计定案待用户可否决——M0 固定位优先、单会话替换语义、
  VM 轨冒烟非门（§10）。

- 2026-09-23 work handoff：`stage: work`，PLAN-087，`plan_revision: 1`，
  `outcome: pass`。`code_commit`: plan-087-dev `8913acc..685b157`（5 连击：
  T-01 锁钉位 → canvas 后端 → 前端面板 → 冒烟修正 → SD-01 规范草案备稿）。
  `task_ids`: T-01..T-09 全闭环（逐项证据见 §8 各任务批注）。`evidence`:
  集成 live 双臂全绿（lifecycle 全链 spawn→端点→PNG→press→state→stop→census
  零孤儿 7.99s；revival kill→恢复 restarts=1）；全目标套件 31/32 绿
  （唯一红=tool_atoms run_command_dangerous_returns_paused=084 KNOWN-DEBT
  基线红，与 087 无涉）；serve 冒烟 API 全链+浏览器面板（右列展开/实况帧/
  停止收起）；AC-04 热重载 md5 前后对照；serve 期天然复活一单（AC-05 活体）；
  证据归档 `docs/plans/attachments/087/`。wt-guard clean（node_modules 链与
  target-test 测试缓存已清，merge 期 worktree 移除安全）。
  `blockers`: 无阻（AC-01 LLM 真聊臂与 VM 轨面板冒烟为登记的证据边界/尽力
  项，见 T-08/T-09 批注，非阻断）。`next`: review（worktree 保留）。
  备注：M0 根因翻案（锁偏斜非上游缺陷，§8 T-01 批注）；三处契约修正入
  §10-6；单会话替换语义与 VM 非门按默认案执行，用户可否决。

- 2026-09-23 review：`stage: review`，PLAN-087，`plan_revision: 1`，
  `outcome: pass`。`reviewed_commit`: `685b157`（plan-087-dev HEAD）；
  `base_commit`: `4a331ff`。`dependency_revisions`: auto-lang@`8fecfcf69`、
  auto-ai@`57eb44a`（兄弟 worktree，均无独有提交）。`spec_inputs`:
  `docs/specs/modules/app-canvas.md`@685b157（worktree 备稿，本次审定其内容
  与实现一致；merge 时发布）。
  **独立性声明**：review 在实施会话内进行（无独立 reviewer 授权）——判定从
  工件重建：全量 diff 逐行审（35 文件，16 处机械插入逐点抽查=纯一行）、
  验证命令全部重跑，未沿用执行者结论。
  `acceptance_results`：AC-02/03/04/05/06 **pass**（本次 review 重跑复现）：
  全量套件 31/32 目标绿（`cargo test -j 3`，唯一红=tool_atoms
  run_command_dangerous_returns_paused=084 KNOWN-DEBT 基线红，与 canvas 零
  交集，work/review 两轮同位同因）；live 双臂干净复现 2 passed（lifecycle
  全链含 press→state 断言→stop→census 零孤儿；revival restarts=1）；serve
  冒烟复现 start→running→frame 200→热重载 md5 变化（≤5s）→stop；浏览器
  CUA 真实坐标点击停止钮→status stopped+面板收起（用户路径闭环，此前仅
  直调 fetch 臂）。AC-01 **partial**（登记的证据边界）：模板实例化产物可
  启动出帧（复现）+提示词注入单测锚定；LLM 真聊臂需真模型配置，留用户
  验收安排——不阻塞 M1 验收（模板实例化即 M1 生成路径，计划 §2 定案）。
  `findings`：F-1（外观）server.rs:291 两语句挤一行（`canvas.clone();    axum::serve`），
  仓内无 fmt 门（基线自带漂移），非阻断；F-2（测试环境耦合）lifecycle 的
  census 断言对外部 auto.exe 敏感——与并发 serve 冒烟同跑时误报孤儿一次
  （环境失败非回归，干净重跑即绿）；测试已文档化串行要求（spec 测试口径节），
  更强口径（pid 集合差）留非阻塞改进。
  `evidence`：`/tmp/p087-review-suite.log`（套件+双臂）、浏览器截图
  attachments/087/（panel-right-column.png 等）、serve3.log、热重载 md5 对
  （r-hot-before/after.png 未归档，判定以文本记录为准）。`next`: merge
  （worktree 保留）。

## 10. 待澄清事项

1. **M0 解锁路径偏好**：T-01 默认固定位 Cargo.lock（快、先例在）；若用户倾向等
   auto-lang master 稳定后重建，计划顺延但不阻塞起草。默认按固定位执行。
2. **VM 桌面轨画布冒烟**：验收面定为 web 轨；VM 轨（launch-vm.cmd）面板冒烟为
   尽力项。若冒烟暴露 VM 轨 image 渲染缺陷，登记 KNOWN-DEBT，用户可决定是否升格
   M1.x。
3. **单会话替换语义**：再次 canvas_run=停止旧会话再起新会话（M1 定案）。若用户
   期望"拒绝并提示"，改 manager.rs start 分支即可，不影响其他任务。
4. **auto-ai master 漂移**（work 勘察新增，2026-09-23）：auto-ai main `58bee8d`
   （PLAN-034 role-model-binding）给 `RoleConfig` 增 `models` 字段（+1295 行实码），
   musk `extern_impl.rs` 两处构造点缺字段编译即红。本计划按 086 先例兄弟位钉
   `57eb44a` 规避；musk 侧适配（含 a2r 树重转）需另案，不并入 M1。
5. **examples/ui 扩展池缓行**（契约修正）：原稿"examples/ui 001..005 作扩展池"
   与多根沙箱冲突——agent 的 read_file 被 fail-closed 锁在 workspace 内，读
   auto-lang 仓库路径需白名单授权，不可作为 M1 生成路径。M1 只消费内嵌
   counter/hello 双模板；扩展池（含白名单引导）延 M3 生成流再评估。
5.1 **merge 期依赖位须知**（work 补记 2026-09-23）：① 兄弟 worktree
   `.wt/musk-087/{auto-lang,auto-ai}` 两分支均无独有提交（固定位=已落库 commit），
   merge 无需快进回依赖主分支，随 worktree 清理一并删分支即可；但 **musk
   worktree 的构建/测试在合回前硬依赖两兄弟存在**（相对路径解析序），须待
   musk 分支 ff 落 main 后再拆兄弟位。② 合回主检出后，主检出的构建解析
   `D:/autostack/auto-lang`（master b4960eb8f）——其 dx12 编译臂在 musk feature
   统一下仍红（见 T-01 批注），8913acc 的锁钉位是对 084 基线树求得的，对
   master 树是否足够未验证；主检出 build 健康属 PLAN-086 遗留议题，不因 087
   回归。③ smoke 目录 `.wt/musk-087/smoke/`（隔离 serve 冒烟工作区+日志）随
   组目录清理一并删除。
6. **MCP 契约修正**（源码核实 @8fecfcf69）：① `autoui_screenshot` 返回**落盘
   路径文本**（"Screenshot saved to: <abs>"，默认 `<app>/tmp/autoui-screenshot-<ms>.png`）
   而非 base64——mcp_client 按路径语义读文件，看门狗删上一帧防堆积；
   ② 监听行在 **stderr**（"AutoUI MCP: listening on http://<addr>"）且端口忙时
   auto-lang 自带 +1..+10 回退——session.rs 双流扫描并解析行内实际地址；
   ③ `autoui_action` 参数形为 `element_id`+`action` 枚举+可选 `value`（非原稿
   action_type/target）；④ 热重载轮询 release 缺省 2000ms（`AUTOUI_HOT_RELOAD`
   可调 500ms），AC-04 ≤5s 预算成立。

## 11. merge 收据（PLAN-087:r1）

`stage: merge`，`outcome: pass`，`completion_kind: delivered`（2026-09-23）。

- `prepared`：reviewed 基线 685b157（plan_revision 1）；canonical 增量=
  docs/specs/modules/app-canvas.md（随 685b157 入分支，备稿即 delivery，
  无新代码提交）；账本投影目标 .autoos/specs.json designs/reviews 两段。
- `landed`：主检出脏 Cargo.lock 以 stash 让路（stash@{0}，标签注明由本分支
  锁钉位取代；stash@{1} 为 086 同款在档）；rebase main=恒等（tip 4a331ff
  未动，无哈希改写，review 绑定 hash 原样有效）；`git merge --ff-only
  plan-087-dev` → main tip=`685b157`（零合并提交）；canonical spec 在 main
  实证（sha256=9f5926f7ff435a9264587cad20b3e8c28035f3dd73379da053acf90104b0e90d）；
  `cargo metadata` 主检出解析一致 OK（主检出 build 编译健康属 086 遗留——
  auto-lang master dx12 臂 + auto-ai RoleConfig.models，非 087 回归，见
  §10-5.1②）。
- `ledger_refreshed`：.autoos/specs.json（运行时账本，git 不追踪）——designs
  增 `app-canvas-D1`（Approved，file=docs/specs/modules/app-canvas.md，
  source_sha256 同上，related=[app-canvas-R1]）；reviews 增 `app-canvas-R1`
  （Published，depends_on=[app-canvas-D1]，file=本归档路径）；version 2→3；
  原子写（temp+replace，写前核验无 musk serve 在跑）+回读验证（ids/sha/
  related 全对）。
- `archived`：本文件由 docs/plans/087-app-canvas-m1.md 归档而来（untracked
  既有物，move+add），status: archived；同批收录 docs/designs/
  013-ai-app-studio-canvas.md（计划战略依据，先前 untracked）与
  docs/plans/attachments/087/（review 证据包耐久化）。
- `cleaned`：worktree `D:/autostack/.wt/musk-087/auto-musk` 移除前 fresh
  wt-guard clean；分支 plan-087-dev（已 ff 入 main）删除；兄弟位
  auto-lang/auto-ai（分支均无独有提交，无需回合约）worktree+分支移除；
  smoke/ 临时目录与空组目录删除。逐项实证见 git/文件系统终态。
