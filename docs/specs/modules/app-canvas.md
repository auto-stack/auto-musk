# App Canvas（实况画布）模块规范

> PLAN-087 M1 落地（2026-09-23，work pass 待 review）。Design 013（AI App Studio
> 战略）的 M1 里程碑沉淀：一句话生成简单 Auto app → workspace 内产物 → chats
> 视图右侧实况画布（隔离 VM 子进程 + AutoUI MCP 截图流）→ agent 以 canvas_*
> 工具驱动与断言。M2（双向锚定）/M3（三层生成流）/M4（飞轮）扩展锚点见文末。

## 会话生命周期（canvas/manager.rs）

- **单会话**：musk AppState 持一个 `CanvasManager`（`Arc`）；`start(app_dir)`
  为替换语义——已有会话先 stop 再起。状态机
  `starting → running ⇄ restarting → stopped | degraded`。
- **端口分配**：`TcpListener::bind("127.0.0.1:0")` 临时取号；AutoUI 端口忙时
  auto-lang 自带 +1..+10 回退，会话以**监听行内实际地址**为准（不信任注入值）。
- **端点发现**：spawn 后双流（stdout+stderr）扫描
  `AutoUI MCP: listening on http://<addr>`（该行在 stderr），30s 超时 = 启动
  失败（degraded 报错，不自动重试 spawn——坏 app 不进死循环）。
- **看门狗**：~1s 帧拉循环；`autoui_screenshot` 连续 3 次失败或单次超时
  （>10s）→ restarting；子进程死亡（try_wait 快检）即刻走复活。退避序列
  1s/2s/4s，封顶 3 次，超限 → degraded（status 暴露，面板红条，stop/start
  可恢复）。最小化窗拒绝（"Screenshot skipped"）归类可恢复，不计失败。
- **收割**：stop() = 停止旗标 + 立即 `taskkill /T /F` + 端口探测确认退场 +
  每会话一次性 storage 文件清理；Drop 同步兜底；serve ctrl-c graceful
  shutdown 钩子全量收割。验收口径 = tasklist census 零孤儿（AC-05）。

## AUTO_EXE 解析序（canvas/session.rs）

`AUTO_EXE` env → 编译期兄弟位 `CARGO_MANIFEST_DIR/../../../../auto-lang/target/
release/auto.exe`（worktree 构建解析组内兄弟、主检出构建解析主检出）→
`D:/autostack/auto-lang/target/release/auto.exe` → PATH。

## Spawn 环境契约

`auto run --render=vm`（cwd=app 目录）+
`AUTOUI_MCP_PORT`（分配的临时口）/`AUTO_VM_STORAGE_FILE`（会话一次性
localStorage 镜像，状态零串味）/`AUTO_VM_WINDOW=480x680`（画布友好固定窗）；
`AUTO_REUSE_BACKEND`/`AUTO_HTTP_BASE` 显式移除（隔离优先）。

## AutoUI MCP 消费契约（canvas/mcp_client.rs）

- JSON-RPC 2.0 over HTTP，单端点 `POST /mcp`，每请求独立处理；
  `initialize`（2024-11-05）尽力握手不作门。
- `autoui_screenshot` 返回**落盘路径文本**（`Screenshot saved to: <abs>`，默认
  `<app>/tmp/autoui-screenshot-<ms>.png`）——消费方读文件取 PNG 字节，并负责
  删上一帧防堆积（baseline 臂写 `tests/` 污染 app 目录，弃用）。
- `autoui_action`：`element_id`（`vnode_<n>`；vtree 显示 `#vnode_<n>`）+
  `action` 枚举（press/type_text/submit/toggle/select_option/set_value/clear/
  scroll/drag/pen/resize_col/key_press/editor_drag）+ 可选 `value`。
- `autoui_state`：`fields` 过滤（全等或 `.field` 后缀），返回
  `State:\n  <name>: <value> (<type>)` 文本。
- `autoui_snapshot`：AURA 文本树（`include_bounds` 开关）。

## API 四路由（canvas/mod.rs，serve 挂载）

| 路由 | 语义 | 失败面 |
|:---|:---|:---|
| `POST /api/canvas/start?workspace={id}` `{app_path}` | 校验+启动（替换） | 越界 400（报文列全部根）；非 app 目录 400；spawn 失败 500 |
| `GET /api/canvas/frame` | 当前帧 PNG 直出 | 无帧 503 |
| `GET /api/canvas/status` | `{state,seq,app_path,restarts,error}` | — |
| `POST /api/canvas/stop` | 停止+收割（幂等） | — |

安全边界：`app_path` 经 `tool_safety::resolve_multi` 多根判定 fail-closed
（workspace 根恒第一根 + 白名单），报错文案列全部根；`pac.at` 存在性校验。
agent 工具面（canvas_run）同口径。

## agent 工具五件（canvas/tools.rs，coding 模式白名单）

`canvas_run{app_path}` / `canvas_stop` / `canvas_snapshot`（最新帧落
`{workspace}/.canvas/snap-{seq}.png` + `/api/files` markdown 图 + vtree 摘要
2000 字截断）/ `canvas_act{element_id,action,value?}` / `canvas_state{fields?}`。
注册走 `build_agent_with_context` 白名单过滤（`modes/coding.at` 收录五名）。

## 生成侧（canvas/templates.rs）

- 内嵌模板池：counter/hello 双模板（084 基线 examples/ui/{002,001} 精简拷贝，
  剔除 .am/.auto 缓存）。coding 模式系统上下文追加生成指导
  （`generation_prompt()`：模板源码 + "选模板→写文件→canvas_run→act/state
  断言"流程 + widget 词汇 + 已知坑节选）。
- 生成校验 = canvas_run 启动即验收（VM 轨对 .at 直接解释，无需 npm 工具链）；
  `auto build -r vue` 为可选非门。
- examples/ui 扩展池缓行（沙箱 read_file 不可达仓外路径）——M3 生成流再评估。

## 前端面板（web 轨验收面）

`canvas_store.at`（1s timer 轮询 status，Http.get_msg 回填）+
`canvas_panel.at`（chats 根行第三列 380px：状态点/app 名/停止钮/degraded
红条/帧 img 以 `?t={seq}` 缓存击穿）+ `ports/canvas.{web,vm}.at` 门面。
有非 stopped 会话自动展开、stop 后收起（cv_open 单源于轮询）。VM 轨：门面
桩 = 面板不展开（登记缺口非回归，whitelist VM 同款）。

## 热重载预期

VM 轨 mtime 脏标热重载（release 缺省 2000ms 轮询，`AUTOUI_HOT_RELOAD=1` 可
500ms）——canvas 零开发量，改 app.at ≤5s 画布帧反映变化（AC-04 实证）。

## 测试口径（tests/canvas_live.rs）

- live 双臂（`#[ignore]`+`#[serial]`，`-- --ignored` 显式跑，需 auto 可执行）：
  lifecycle（spawn→端点→首帧 PNG→press→state 断言→stop→census 零孤儿）、
  revival（manager.pid() 精确 kill→≤15s 恢复→restarts≥1）。
- api 臂（常跑）：越界 400 列根 + 无帧 503。
- 单测：监听行解析/路径校验/模板插值/路由形状。
- **spawn 类测试必串行**（共用全局进程表；并行会互杀他臂会话——首跑实证）。

## 后续里程碑锚点（未实现，扩展位）

- M2 双向锚定：画布点选→源码（`source_map.rs` 填充前置）、层树栏、
  `canvas.pick/overlay`。
- M3 三层生成流：bp 注册表/词汇表注入、L1>L2>L3 复用序、`ui.lint` 护栏、
  examples/ui 扩展池（含白名单引导）。
- M4：`blueprint.extract` 飞轮、AppViewport 原生嵌入（路线 B）。
- 多画布会话并发（M1 恒单会话）、画布内嵌用户交互（M1 对用户只读）。
