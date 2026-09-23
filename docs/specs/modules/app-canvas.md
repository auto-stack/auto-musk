# App Canvas（实况画布）模块规范

> PLAN-087 M1 + PLAN-088 M2 落地（2026-09-23，work pass 待 review）。Design 013
> （AI App Studio 战略）的 M1/M2 里程碑沉淀：一句话生成简单 Auto app →
> workspace 内产物 → chats 视图右侧实况画布（隔离 VM 子进程 + AutoUI MCP 截图
> 流）→ agent 以 canvas_* 工具驱动与断言 → 画布点选↔源码锚点↔层树双向锚定。
> M3（三层生成流）/M4（飞轮）扩展锚点见文末。

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
localStorage 镜像，状态零串味）/`AUTO_VM_WINDOW=480x680`（画布友好固定窗）/
`AUTO_DEBUG_CAPTURE=1`（PLAN-088：debug 捕获面——MCP vtree 通道的叶件
bounds+引导帧 bounds 依赖项；devtools_open 不置位面板不出镜）；
`AUTO_REUSE_BACKEND`/`AUTO_HTTP_BASE` 显式移除（隔离优先）。

## 锚定契约（canvas/anchor.rs，PLAN-088 M2）

- **索引**：看门狗随帧拉 `autoui_vtree`（Atom 文本）→ `AnchorIndex`
  （文档序节点表 + id 反查），解析失败保留旧索引（尽力，无阻断）。
  Atom 消费契约（实测钉死）：Node body 顶层 prop 分隔 `; `、对象值内分隔
  `,`、vnode id=路径哈希大整数——**协议面 id 全字符串形态**（>JS 2^53 数字
  静默截断）。锚点换代时 overlay/picked 保留交集（结构性热重载后失效 id
  自动清，stale pick 204）。
- **坐标契约**：vtree bbox=窗口**逻辑**像素；帧 PNG=逻辑×DPI scale；
  scale=帧宽/AUTO_VM_WINDOW 逻辑宽(480)，musk 单源推导。pick 入参=帧像素；
  载荷携 `bbox`（逻辑）/`bbox_px`（×scale）/`bbox_pct`（像素÷帧尺寸×100，
  前端百分比定位直吃）。**锚定精度以固定窗为前提**（模板池固定窗口径）；
  fit app 的 fit 重排使 bounds 陈旧 = 降级尽力。
- **auto-lang 依赖**（T-02B 最小填充，8fecfcf69+016382eb4）：vtree Atom 输出
  `span:{offset,len}`（源自 vnode.source_span）；probe ForIter 并入 computed
  （view 同步块+wrap_debug 两站）；`AUTO_DEBUG_CAPTURE` env 门控（见上）。
  行号换算在 musk 侧（VM 不持行索引）。
- **命中语义**：deepest-first（含点最小面积，含边界——hit_test.rs 同款）→
  命中无 span 且无 events 时沿 parent 链上溯至最近"实质节点"；返回恒含
  ancestor_chain（root→命中）。
- **源码锚点**：span(字节偏移)→(相对路径,行号) musk 侧换算（候选文件 +
  kind 关键词启发；M2 主路径单文件 app，多文件为启发式）；路径 workspace
  相对化（app_rel 前缀，files API 消费形）。
- **pick API**：`POST /api/canvas/pick` `{x,y}`（帧像素）或
  `{vnode_id:"vnode_N"}` → 200 锚点载荷 / 204 未命中 / 503 无帧或无索引；
  命中即置 picked（status 单源，前端 ≤1s 轮询回流）。status 载荷 =
  M1 五字段 + `frame`（PNG 尺寸）/`picked`/`overlay`/`tree`（扁平文档序表）/
  `pac_head`（轻文本提取 name/title/render/window）。
- **工具**：`canvas_pick{element_id}` → 锚点结构（与点选同构，含
  ancestor_chain；id 失效 Err 指引重取现行 id）；`canvas_overlay{
  element_ids[],clear?}` → 置高亮（琥珀框，status 回流前端渲染）。选中
  （蓝框，pick 置位）与高亮（琥珀，overlay 置位）双框层百分比定位。
- **前端联动**：层树列（扁平表按 depth 缩进，预拼 style——模板插值表达式与
  括号算术 concat 有 codegen 实证坑，规避）→ 点选走 `TreeNodePicked`→
  `canvasPickNode`→`PickBackfill` 即时回填；帧点选 = document 级委托
  （`img[src^="/api/canvas/frame"]`，显示→自然坐标换算）；锚点 chip
  （kind/label/#N/source）→ 源码抽屉（`/api/files/raw/{path}` **无参形态**
  ——显式 workspace 参数与 fetch 拦截器重复 → 400）。
- **已知边界**（登记非门）：层树 1Hz 全量重渲（功能正确，性能味）；
  fit-window app 降级尽力；多文件 span 文件判定启发式；canvas 轨曾现
  ~35s 周期性 VM 退出（未复现，stderr 尾部诊断桩在位，复发可定罪）。

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

## API 五路由（canvas/mod.rs，serve 挂载）

| 路由 | 语义 | 失败面 |
|:---|:---|:---|
| `POST /api/canvas/start?workspace={id}` `{app_path}` | 校验+启动（替换） | 越界 400（报文列全部根）；非 app 目录 400；spawn 失败 500 |
| `GET /api/canvas/frame` | 当前帧 PNG 直出 | 无帧 503 |
| `GET /api/canvas/status` | M1 五字段 + frame/picked/overlay/tree/pac_head（M2） | — |
| `POST /api/canvas/pick` | `{x,y}`（帧像素）或 `{vnode_id}` → 锚点载荷 | 未命中 204；无帧/无索引 503；缺参 400（M2） |
| `POST /api/canvas/stop` | 停止+收割（幂等） | — |

安全边界：`app_path` 经 `tool_safety::resolve_multi` 多根判定 fail-closed
（workspace 根恒第一根 + 白名单），报错文案列全部根；`pac.at` 存在性校验。
agent 工具面（canvas_run）同口径。

## agent 工具七件（canvas/tools.rs，coding 模式白名单）

`canvas_run{app_path}` / `canvas_stop` / `canvas_snapshot`（最新帧落
`{workspace}/.canvas/snap-{seq}.png` + `/api/files` markdown 图 + vtree 摘要
2000 字截断）/ `canvas_act{element_id,action,value?}` / `canvas_state{fields?}` /
`canvas_pick{element_id}`（锚点结构，M2）/ `canvas_overlay{element_ids[],clear?}`
（高亮置位，M2）。注册走 `build_agent_with_context` 白名单过滤
（`modes/coding.at` 收录七名）。

## 生成侧（canvas/templates.rs）

- 内嵌模板池：counter/hello 双模板（084 基线 examples/ui/{002,001} 精简拷贝，
  剔除 .am/.auto 缓存）。coding 模式系统上下文追加生成指导
  （`generation_prompt()`：模板源码 + "选模板→写文件→canvas_run→act/state
  断言"流程 + widget 词汇 + 已知坑节选）。
- 生成校验 = canvas_run 启动即验收（VM 轨对 .at 直接解释，无需 npm 工具链）；
  `auto build -r vue` 为可选非门。
- examples/ui 扩展池缓行（沙箱 read_file 不可达仓外路径）——M3 生成流再评估。

## 前端面板（web 轨验收面）

`canvas_store.at`（1s timer 轮询 status，Http.get_msg 回填；M2 锚定面经
handler 域扁平重建漏斗——r5 处方）+ `canvas_panel.at`（chats 根行第三列
520px：层树列 170px〔pac 头+扁平树+#N 实例〕+ 主区〔状态点/app 名/停止钮/
degraded 红条/帧 img `?t={seq}` 击穿 + 双覆盖框层 + 锚点 chip + 源码抽屉〕）+
`ports/canvas.{web,vm}.at` 门面（web 真调/VM 空值桩）。有非 stopped 会话自动
展开、stop 后收起（cv_open 单源于轮询）。VM 轨：门面桩 = 面板不展开（登记
缺口非回归，whitelist VM 同款）。

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

- M3 三层生成流：bp 注册表/词汇表注入、L1>L2>L3 复用序、`ui.lint` 护栏、
  examples/ui 扩展池（含白名单引导）。
- M4：`blueprint.extract` 飞轮、AppViewport 原生嵌入（路线 B）、VM 内
  OverlayInfo 高亮烘焙评估（M2 以前端覆盖框替代）。
- 多画布会话并发（恒单会话）、画布内嵌用户交互（对用户只读——点击仅作
  选区不作驱动）、行级源码定位（M2 打开文件为止）、fit-window app 锚定
  精化、多文件 span 判定精确化。
