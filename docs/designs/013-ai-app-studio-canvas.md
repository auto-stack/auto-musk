# 013 — AI App Studio：实况画布、双向锚定与三层生成（战略设计）

> 日期：2026-09-23；状态：提案（战略设计，未实现）；定位：musk 差异化方向的总纲。
> 用户方向：musk 不做通用 coding agent，做"用 AI 生成 Auto App"的工坊；Auto 的强项是
> UI 与 OS，故 UI 是主战场——要一栏可实时预览 app 的 Canvas，且 AI 能直接操作界面
> （类 Figma Canvas）；生成能力必须结合 AutoUI 的 widget / blueprint / app 三层架构。
> 设计目录沿用 `docs/designs/`（Design 012 裁定，不另建平行 `docs/design/`）。

## 1. 结论与定位

**把 musk 做成"长在三层架构上的 AI 工作台"，不是"带预览的 coding agent"。**

与 v0 / bolt / lovable 一类的本质差异（护城河三支柱）：

1. **画布是实况，不是投影**。Canvas 跑的是隔离 VM 里的真实 app 进程（有状态、有数据、
   可交互），不是生成结果的静态截图。因此：点选反馈真实命中、AI 操作是真实驱动、
   `autoui_state` 能回答"为什么渲染成这样"（列表为空是数据没加载还是样式问题——
   截图永远回答不了）。
2. **编辑有层语义，不是文件重写**。三层架构让 AI 的每次编辑都有工程归属：改实例
   prop / 升 blueprint 定义 / 新增模式。直接解决 LLM UI 生成"一把梭重生成、用户改过的
   东西被冲掉"的行业通病。
3. **验收闭环有状态可见**。改 → 热重载 → 结构化 snapshot + 求值态 dump + 视觉断言，
   agent 自己当验收员。musk 团队积累的视图坑/codegen 坑知识库前移为生成时护栏
   （`ui.lint`），别的 agent 没有这份 Auto 专属资产。

叙事一句话：**别的 agent"写代码给你看"，musk"在画布上和你一起捏 app"**。

## 2. 依据与既有资产盘点（源码调查）

地基比预期好，大部分组件已存在并被验证，缺的是组装：

- **PLAN-080 隔离实况链**（已交付）：`scripts/ui-parity/live.mjs`——真进程
  `musk.exe serve`（`MUSK_SERVE_PORT`+`MUSK_CONFIG_DIR` 隔离）→ `auto run --render=vm`
  （`AUTO_REUSE_BACKEND=1`/`AUTO_HTTP_BASE`/`AUTO_VM_STORAGE_FILE`/`AUTO_VM_WINDOW`/
  `AUTOUI_MCP_PORT`）→ 子进程 stdout 抓取 `AutoUI MCP: listening on .../mcp`。
  反向用于任意目标 app，即实况画布的进程编排骨架。
- **AutoUI MCP**（auto-lang `crates/auto-lang/src/ui/mcp_server.rs`）：17 个工具——
  `autoui_snapshot`（带 bounds/`source: "app.at:42"`/`for_context`）、`autoui_action`
  （press/type/toggle/select_option/set_value/submit/scroll/drag/key_press…）、
  `autoui_screenshot`（iced 线程截屏，base64 PNG；最小化窗护栏）、`autoui_state`、
  `autoui_vtree`、`autoui_find/check/wait/inspect/select_rect/fixture` 等。采集语义已
  沉淀为 musk `docs/specs/modules/ui-parity.md` 契约。
- **源码锚定体系（已建约 80%）**：`AuraNodeId(u32)`（源级视图树）；`DebugIdMap`
  （View 子索引路径→AuraNodeId）；`VNode.path` 稳定逻辑路径（for 循环展开后仍稳定，
  `aura_view_builder.rs` 有稳定性断言）；`AppState.line_to_aura_ids` 双向源行↔控件映射
  （本为 DevTools 点选高亮而建）；`ComputedNodeLite` 进 MCP 快照；`VNodeId→SourceLocation`
  的 `ui/debug/source_map.rs` 结构在、填充不全（M2 前置债）。
- **三层架构契约**（auto-lang）：
  - widget=词汇表：`schema/aura.at`（6014 行机器可读元素注册表：tag/category/tier/
    backends/props+类型+描述）、`stdlib/aura/widgets/`（form/layout/data/…分类）、
    package 组件（`ComponentRegistry`，`auto-os/widgets-gallery/src/front/components/`
    为实例）、`docs_gen.rs` 可生成 Markdown 组件参考 + kitchen-sink demo。
  - blueprint=模式库：`blueprints/<kind>/<name>/{spec.md, reference/<variant>.at,
    gotchas.md}`（TOML frontmatter：palette/extension_points/variants/dataSource/props/
    actions/acceptance）；`BlueprintRegistry`；`auto bp list|show|add|bind|check`；
    `auto bp list --format at` 产出静态注册表（bps-gallery 消费）。
  - app=编排：`pac.at` 清单 + `src/front/app.at`（`routes{}`→页面、`outlet`、store）。
  - **消费三档已为 agent 留位**：`docs/specs/blueprint/contract.md` L1 bind（零拷贝）/
    L2 copy（`auto bp add --reference`）/ L3 agent 生成（variant promotion review）；
    `docs/design/blueprints/agent-generation-workflow.md` 明确"agent 读 spec→写 .at→
    check/build 循环 ≤N 轮"。`crates/autoui-skill/`（SKILL.md + known-pitfalls.md +
    generator-contracts.md）是现成的 agent 技能雏形。
- **热重载**：VM 轨 `ui/hot_reload.rs`（notify 监听）+ `DynamicComponent` mtime 脏标
  （改 `.at` 即时生效）；Vue 轨 `auto watch`（<1s，Vite HMR）。"agent 改→画布即时变"
  的底座已在。
- **demo 素材库**：`auto-lang/examples/ui/001-helloworld .. 020-music-player` 编号系列
  （002-counter = pac.at 6 行 + 49 行 app.at），天然是 M1"一句话生成"的 few-shot/
  模板池与验收基准。
- **musk 侧挂点**：agent 工具注册表 `backend/crates/musk/src/tools.rs`（read/write/edit/
  search/run_command/display_image…）；多根沙箱 `tool_safety.rs`；HTTP 面 `server.rs`
  （axum + SSE + 静态托管）；前端真源 `src/front/*.at`（app.at 视图族：chats/plans/
  specs/wiki/files/whitelist）；085 五栏 workspace 刷新在稿（画布栏应在其布局中占席）。
- **auto-lang 预留原语**（musk 尚未消费）：`View::Canvas`（状态驱动、nodes/edges/on_hit，
  Plan 563/661）、`WindowThumbnail`、`WorkspacePreview`、`ImageSurface`，及 auto-os
  ui-gallery 已用的 `AppViewport` 嵌入——路线 B 的素材。

## 3. 产品形态：三栏工作台

```
┌────────────┬────────────────────────┬──────────────┐
│  层树       │       Canvas（实况）     │   Chat/Plan  │
│ app        │  ┌──────────────────┐  │              │
│ ├ routes   │  │  真跑在隔离 VM 里  │  │  agent 对话   │
│ ├ bp 实例   │  │  的目标 app       │  │  计划/收据    │
│ └ widgets  │  │ （可点选/可高亮/   │  │              │
│            │  │  AI 可直接操作）   │  │              │
└────────────┴──┴──────────────────┴──┴──────────────┘
```

- Canvas 定位为**实况**（run mode）：目标 app 以子进程跑在隔离 AutoVM 实例中，永不降级
  为静态截图。未来可加结构化"设计态"（不启进程的树编辑），但实况是底线与差异点。
- Chat/Plan 沿用现有 chat 收束与 plan flow；画布产生的编辑走既有 worktree/沙箱纪律。
- 层树（app→blueprint 实例→widget）是三层架构的可视化，点选即定位源码。
- musk 自身前端即 `.at`：画布栏用 Auto 写，吃自己狗粮；与 musk 自身的 VM 桌面轨
  （launch-vm.cmd）分实例分端口共存。

## 4. 支柱一：实况画布

**路线 A（先行）：子进程 + 截图流 + bounds 覆盖层。**
musk backend 新增 *canvas 会话管理器*：为每个工作会话 spawn 隔离
`auto run --render=vm --path <app_dir>`（独立 `AUTOUI_MCP_PORT`、独立 home/storage，
编排参数复用 080 链）；musk 内置 MCP 客户端（JSON-RPC over HTTP）周期拉取
`autoui_screenshot`；前端呈现 = 截图 + 覆盖层。**目标 app 零改动**。
会话管理器统一收口：端口分配（避让 musk 自身实例）、看门狗（挂起检测，承
`vm-hangwatch` 经验）、进程收割（kill-tree，承 `vm-mcp-census` 孤儿进程教训）、
掉线自动复活。

**路线 B（后置升级）：原生嵌入。** auto-os ui-gallery 已用 `AppViewport` 内嵌 demo app；
让 musk 自身 VM 窗口直接内嵌目标实例，像素级实时、零截图延迟；代价是 MCP 控制需
打通嵌入实例。作为体验升级项，不阻塞主线。

画布三向交互语义：
| 方向 | 语义 | 通道 |
|:---|:---|:---|
| 用户→AI | 点选元素=对话上下文（"改这个按钮"） | 前端坐标→后端以最近 snapshot bounds 命中测试→源码锚点 |
| AI→用户 | 高亮="我要改的是这个" | 锚点→bounds→前端覆盖框 + 层树联动 |
| AI→app | 真实驱动（click/type/scroll/断言） | `autoui_action`/`autoui_keyboard` 全套 |

## 5. 支柱二：双向锚定（体验脊柱）

视觉元素 ↔ 源码节点 ↔ 架构层的稳定双向映射，穿透 for 循环（"表格第 3 行的删除按钮"
→ `VNode.path` 稳定逻辑路径覆盖此场景）。三方向各自的现状与缺口：

- 用户→AI（点选）：`ComputedNodeLite.source`、`line_to_aura_ids` 已备；
  缺口=`source_map.rs` 的 `VNodeId→SourceLocation` 填充不全（M2 前置债）。
- AI→用户（高亮）：bounds 体系齐备；MCP 缺 `autoui_highlight` 类工具
  （路线 A 下可用覆盖层绕过，不必改 auto-lang）。
- AI→app（操作/断言）：`autoui_action`+`autoui_check` 直接可用。

## 6. 支柱三：三层架构 × AI 生成（层归属决策）

三层对 AI 的意义：

- **widget=词汇表**：`schema/aura.at` + stdlib widgets + `docs_gen` 参考，整体注入
  agent palette。agent 不手写已知 widget。
- **blueprint=模式库**：`auto bp list --format at` 注册表 + `auto bp show` spec 是
  为 agent 消费设计的输入格式（palette/extension_points/variants/acceptance）。
- **app=编排**：pac.at + routes + stores + 数据接线。

**生成流遵守 L1 > L2 > L3 复用优先序**：能 bind 现成 blueprint 就不拷贝，能拷参考
实现就不从零生成；L3 生成时以 extension_points（`// ── EDIT: fields ──` 标记）为
编辑区域，改局部不动全身。

**编辑流核心=层归属三问**，agent 每次编辑前必须回答：
1. 一次性微调？（改实例 prop / slot）
2. 模式级改动？（升 blueprint 定义层，走 contract 的 variant promotion review）
3. 新能力？（新 blueprint / 新 widget）

**`blueprint.extract` 飞轮**：agent 生成了好页面 → 主动提议沉淀为
`blueprints/<kind>/<name>`（spec.md + reference + gotchas 从生成物反向生成）。
用户用得越多，模式库越富，后续生成越偏 L1/L2 复用——护城河随使用增厚，v0 无对应物。

## 7. Agent 工具面（挂 musk `tools.rs`，canvas 命名空间统一）

```
app.tree          # 三层结构（app/routes/bp 实例/widget 清单）
canvas.run/stop   # 会话管理器（隔离 VM + MCP 生命周期）
canvas.snapshot   # 结构化 vtree + 截图 → View::Image 进对话
canvas.aura       # aura_snapshot_builder 的 .at+求值态 dump（状态可见性）
canvas.act        # 包装 autoui_action/keyboard/type
canvas.pick       # 命中测试 → 源码锚点
canvas.overlay    # 高亮给用户看
bp.list/show/check/extract
ui.lint           # 已知坑编译成预检规则（视图五坑+codegen 坑清单 → 生成时护栏）
```

安全边界：`canvas.run` 的目标 app 必须位于工作区根内（沿用 `tool_safety.rs` 多根沙箱
fail-closed 语义）；agent 对画布的驱动不触发 human gate（目标是 app 本身，非宿主系统），
但对 app 后端/网络的访问沿用既有 gate 策略。

## 8. 验收闭环

```
意图 → 层归属决策 → patch（层内精确编辑）
  → VM mtime 热重载（改 .at 即时生效）
  → canvas.snapshot + canvas.aura + ui.lint
  → agent 自检（视觉断言 + 状态断言，≤N 轮就地再改）
```

PLAN-080 live parity 臂由"测试基建"升格为 **agent 自己的眼睛和手**——对既有投资的关键复用。

## 9. 分期路线（M0→M4）

- **M0 前置解锁**：主检出 musk build 仍被 wgpu-hal 27.0.4 依赖漂移阻断（PLAN-086
  遗留）。P0 动工前：等 auto-lang master 稳定后重建，或先以组内固定位构建跑通。
- **M1 实况画布最小闭环**（首个用户可见里程碑）：一句话 → 生成 002-counter 级简单
  app（workspace 内 pac.at+app.at）→ canvas 面板实况展示 → agent 以 `canvas.act`
  点按 +1 并用 `canvas.state` 断言 → 截图进对话汇报；再改 app.at，VM mtime 热重载，
  画布自动更新。生成侧 M1 允许"模板实例化"（examples/ui 001..005 模板池）而非自由生成。
- **M2 双向锚定**：画布点选→源码定位、层树栏、`canvas.pick/overlay`。前置：补
  `source_map.rs` 填充。
- **M3 三层生成流**：词汇表/bp 库注入、L1>L2>L3 策略、check/build ≤N 轮循环、
  `ui.lint` 护栏上线。
- **M4 飞轮与升级**：`bp.extract` + variant promotion review；路线 B（AppViewport
  原生嵌入）评估。

## 10. 风险与已知债

1. **VM 进程稳定性升档**：画布把 VM 从"测试跑几分钟"变为"用户全程挂着"，silent
   exit-1 家族与 MCP 孤儿进程（080 KNOWN-DEBT 挂账）必须由会话管理器的看门狗+
   自动复活兜住，这是 M1 的硬性验收项。
2. **wgpu-hal 依赖漂移**阻断主检出构建（M0）。
3. **MCP 采集语义坑**：截图最小化窗护栏、press 语义差异等——quirks 清单随 M1/M2
   迭代入 `ui.lint` 与会话管理器护栏。
4. **锚定穿透力**：for 上下文、条件渲染下的路径稳定性需在 M2 用真实 app（列表/
   表格类）压测，不只 counter 级 demo。
5. **范围纪律**：画布只承载 Auto app；不做通用 web 预览，不做多语言代码编辑器——
   那是通用 coding agent 的战场，不是 musk 的。
