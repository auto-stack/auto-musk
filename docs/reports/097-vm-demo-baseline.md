# PLAN-097 T-01 — VM 演示基线报告（097-vm-demo-baseline）

- plan: PLAN-097（VM 计划开发演示与全流程证据，计划 B）
- task: T-01（四项有限探针 + 本报告）
- 日期: 2026-10-02
- 代码基线: auto-musk worktree `plan-097-dev` @ `7924304`（= main 最新，096 已交付落地）
- musk.exe: worktree 自建 release，`26e82d82b310b76263a613cbaa4a84eb708790ff6aba201a407df8a9347d2642`（77,243,904 B）
- auto 工具链: `D:/autostack/auto-lang/target/debug/auto.exe`，`auto 0.1.0+v0.4.2-2573-g982e80004-dirty`，`d63b3ddc722d606167516bf2c6afe3709c5b32353ba03a090e7c3b3d8070b19b`（tracked 源净，dirty 仅为 .tmp-vm-* 未跟踪调试脚本；构建于 2026-10-02 13:06，落后 HEAD be92a13ac 数个提交）
- node: v25.2.1；pnpm: 11.6.0
- 探针件（临时，不入 scripts/demo）: worktree `tmp/plan097-t01/{probe-launch,probe-canvas-persist,diag-canvas,dump-host-ui}.mjs` + `probe-app/`
- 证据目录（仓外，清理 worktree 后仍可读）: `D:/autostack/.demo/musk-097/t01-*`、`D:/autostack/.demo-build/musk-097/t01/*`

## 结论一览

| 探针 | 结论 | 收据 |
|---|---|---|
| ① 宿主截图/动作可达 | **可行**。私有端口起 musk serve + `auto run --render=vm`（复用门命中），MCP snapshot/action/screenshot 全通；截图 1920x1200 PNG 原图 hash 绑定 | `.demo/musk-097/t01-20261002091414/probe-launch.json`（12/12 PASS） |
| ② 端点与进程所有权 | **可行**。起前端口空闲检查、netstat PID 归属 == spawn PID、收尾只杀自有进程树、端口回收全部可验证 | 同上（owned-cleanup PASS） |
| ③ 真实 VM 文件跨启持久化 | **可行（路线 A：app 目录内普通文件）**。Canvas app 写 `data/state.txt`+`data/boot.log`，跨 canvas stop/start 存活且读回；越界 app_path 400；frame/source 通道可用 | `.demo/musk-097/t01-20261002085418-persist/probe-persist.json`（12/12 PASS） |
| ④ 无链接构建 staging | **可行**。worktree gen-only 无链接 → 外部 staging（须镜像 `vendor/@autodown`）→ warm store 离线安装 → vue-tsc+vite 构建出 dist；worktree 全程 wt-guard clean | `.demo-build/musk-097/t01/pnpm-*.log` + dist/index.html |

## 冻结命令与配方（T-06/T-07/T-11 直接消费）

### 隔离启动（探针①②实证）

```text
# 后端（自有进程 + 隔离配置 + AAID 死端口保证确定性）
backend/target/release/musk.exe serve --workdir <run>/ws
  env: MUSK_SERVE_PORT=<私有端口> MUSK_CONFIG_DIR=<run>/home
       AAID_URL=http://127.0.0.1:9   # 探针用死端口;正式演示换真实 AAID
       AUTO_EXE=<auto 绝对路径>      # canvas spawn 由此解析子 VM 的工具链
# 宿主 VM（复用门:17781 等端口已有活监听即直连,不 spawn 生成的后端）
auto run --render vm --port <front> --back-port <back>
  env: AUTO_REUSE_BACKEND=1 AUTO_HTTP_PORT=<back> AUTO_HTTP_BASE=http://127.0.0.1:<back>
       AUTOUI_MCP_PORT=<mcp> AUTO_VM_WINDOW=1280x800
       AUTO_VM_STORAGE_FILE=<run>/storage.json   # 指一次性文件保证登录态从零
       RUST_MIN_STACK=16777216
```

- MCP 端点发现：等待子进程 stderr 行 `AutoUI MCP: listening on http://127.0.0.1:<port>`（+`/mcp`）；端口回退行 `port N busy — fell back` 需显式记录（本探针未触发）。
- `/api/health` 在当前 build 返回 404（路由缺位）；探针以 `<500` 为就绪判据。**T-06 需定健康检查口径**（TCP 连通 or 实际就绪路由）。
- 复用门语义（auto-lang rust_ui.rs:3729）：`AUTO_REUSE_BACKEND=1` 且端口已有活监听 → 复用；否则**先 kill 占端口进程再 spawn 生成的后端**（kill 有 musk.exe/aaid.exe 保护名单，PLAN-082）。故"先起自有 serve 再起 VM"是必须顺序。
- 进程清理：只 taskkill `/T /F` 自己 spawn 的 Windows PID；**Git Bash `$!` 是 MSYS pid，不能用于 taskkill**（本探针 bash 中转实验曾因此泄漏进程，已清理；Node spawn 的 p.pid 是正确的）。
- 归属核验：`netstat -ano` LISTENING 行的 PID == spawn 的 serve pid（back 端口）/ VM pid（MCP 端口）。

### Canvas 目标采集与控制（探针③实证）

- 控制面走 musk HTTP API（canvas/mod.rs）：`POST /api/canvas/start?workspace=<id>` `{app_path}`（返回 generation_id）、`GET /api/canvas/status`（state/seq/frame/tree/overlay/pac_head/restarts，**不含 MCP 端点与 pid**）、`GET /api/canvas/frame?generation=`（PNG 直出，409/503 语义齐备）、`POST /api/canvas/pick`（x/y 或 vnode_id 或 clear）、`POST /api/canvas/stop?generation=`、`GET /api/canvas/source?workspace=&path=`（只读源码）。
- demo 驱动采集目标 Canvas 用这套 API 即可，**不需要直连 canvas VM 的 MCP**（其端口为临时分配、不对外暴露）。
- 身份绑定：generation_id 贯穿 start/frame/pick/stop 的 409 冲突语义，直接对齐计划 §5.5 的"跨身份拒绝"。
- 越界 fail-closed 实证：`app_path: "../../evil-app"` → 400，报文列出全部允许根。

### 跨启持久化路线（AC-11 依赖，冻结）

- **路线 A（已实证，推荐）**：app 在自身目录内用 `file.create_dir/write_text/read_text/append_text` 读写普通文件。canvas stop/start 只删 `%TEMP%/musk-canvas-<muskpid>-<port>.json`（Storage KV 一次性镜像，app 未用 KV 时根本不创建）与旧截图帧，**不触碰 app 目录**。
- **注意：VM 运行期进程 cwd = `<app>/src/front`**（rust_ui.rs 已知行为）——app 相对路径 `data/state.txt` 实际落在 `<app>/src/front/data/`，仍在 app 目录内、跨启存活。FocusBoard 的 P001"本地保存"按此实现并在需求文档写明。
- 断言口径：boot.log 逐行内容断言（首行 `boot state=created`，其后每行 `boot read=<token>`），**不能用行数断言**——单会话内管理器可能复活重启一次（本探针实测发生），多出 read 行属正常。
- KV 路线（Storage.get/set）在 canvas 下被 `AUTO_VM_STORAGE_FILE` 钉到一次性临时文件、stop 即删（session.rs:76-80/222/233）——**不可用作跨启保存**，不改后端合同。

### 无链接构建 staging（探针④实证，AC-14/15 依赖）

```text
# 1) worktree 内生成（无 npm 步骤,无链接）
auto build --gen-only --strict        # 实测 18.8s exit 0,65 components
# 2) 外部 staging 复制（必须镜像仓布局——package.json 有 file: 相对依赖!）
#    package.json: "@autodown/vue":"file:../../../vendor/@autodown/vue"、"@autodown/engine":同形
#    → staging 布局: <stage>/gen/front/vue + <stage>/vendor/@autodown/{vue,engine}
cp -r gen/front/vue/. <stage>/gen/front/vue/
cp -r vendor/@autodown/{vue,engine} <stage>/vendor/@autodown/
cp <主检出同产物>/pnpm-lock.yaml <stage>/gen/front/vue/   # 离线解析需要 lockfile
# 3) staging 内离线安装+构建（junction 只落 staging）
pnpm install --offline    # 实测 4.3s exit 0（warm store）;公网镜像当日故障(华为云 UND_ERR_DESTROYED),离线路线是稳定路径
pnpm build                # vue-tsc && vite build,实测 13.5s exit 0 → dist/
# 4) 复验:worktree wt-guard 全程 clean
```

- 生成源自包含性：vite alias 仅 `@`→`./src`，pnpm-workspace `packages: []`；唯二外部引用是上述 vendor `file:` 依赖，复制 vendor 后闭环。
- 一次性删除 worktree 内旧构建产物安全（git clean -fdx）：本 T-01 起跑前已执行，guard clean。

### 驱动协议要点（探针①实证，喂给 T-07 capture/evidence）

- snapshot 解析：`<indent><kind> #vnode_N "label" @rect(x,y,w,h)` + `style:/placeholder:` 行；**vnode id 每帧重渲染会变，一切 action 前必须重拍快照**；搅动期部分快照无样式/无 rect，缺靶按可重试处理；稳定帧门 = 连拍两次一致。
- 文本注意：快照 label 对 CJK 长文按渲染宽度裁剪——等待锚用短中文词（如「新建会话」）或拉丁 marker。
- 截图：`autoui_screenshot` 落 PNG 于 VM 进程 cwd 相对 `tmp/`，响应文本 `Screenshot saved to: <path>`；采后搬移进 evidence 目录并记 SHA256（本次 1920x1200, 93,500B, `ad9ac778da2f5100…cca8`）。

## 基线发现（债/风险，非本计划引入；逐条已定位）

1. **VM 轨 ChatsView 两个 handler 合成失败**：`handler synthesis failed: ChatsView..SendInput / CtxRemove: Undefined variable: canvasStore` → `[CODEGEN] dropping poisoned export`（宿主 VM 日志实证；`use store: CanvasStore` 于 PLAN-093 T-08 进入 chats_view.at）。影响：VM 客户端聊天发送/上下文移除按钮失效。演示影响有限（三份需求按计划走正式 plan 入口/HTTP，不依赖 UI 发送），但 **AC-05 的"操作验证"如需 UI 发送则会撞上**。T-05 做 canvas/进度投影时与该症状同域，届时评估是否顺带 scoped 修复或登记 KNOWN-DEBT。
2. **无 auth 中间件 → 客户端冷启直达主 shell（无登录表单）**：PLAN-080 live.mjs 的"等登录表单再登录"流程对当前代码已过时（本探针首轮 300s 等表单超时即此因）。UI 操作演示从 shell 起步。
3. **宿主 VM 进程偶发提前退出**：一次实测 `[X9] vm interpreter returned ok=true` 于等待期（无 panic、无错误行），复跑即愈。非确定性环境噪声——T-06/T-07 驱动须处理 VM 退出事件并显式重试/报错，不可无限等待。
4. **worktree 运行的模块解析打到了主检出**：宿主 VM 日志 `store facade: use store: AuthStore resolved to D:\autostack\auto-musk\src\front\auth_store.at`（主检出路径）；auto-man 的 AmConfig/AutoCache/端口簿按项目名 `auto-musk` 全局解析，不区分 worktree。当前 worktree==main 内容故无实害，但**正式演示若在内容不同的树内跑，store/模块可能读到主检出文件**——T-06 冻结启动命令前必须实测并处理（候选：每跑独立 AmConfig/缓存根,或验证实际读取路径并记录）。
5. pnpm 公网镜像不稳（华为云 UND_ERR_DESTROYED）：demo 的 Vue 臂一律走 staging+lockfile+`--offline`。
6. `use auto.fs/file` 裸名即用；`File.*` 大写别名亦可用（041-auto-edit 先例）；`Storage.*` 在 canvas 下会话级（见上）。

## 与计划条款的对账

- §5.3「复用共享 AAID 只允许只读连接」：探针用死端口失败臂（确定性），正式演示（T-12）才接真实模型。
- §5.5「截图保留原始画面」：本次 PNG 为 `autoui_screenshot` 原图直存，无重绘/拼接；hash 已记录。
- §4「work 必须重拍实际基线和二进制 hash」：即本报告指纹节；musk.exe 为 worktree 自建 release（7924304）。
- 探针产物落仓外 `D:/autostack/.demo{,-build}/musk-097/t01*`，worktree 无残留（tmp/ 探针件为临时调试用，不入提交）。
