---
plan_id: PLAN-101
status: archived
feature_name: VM 轨 composer 发送与 workspace 当前项标记修复
author: [agent]
created_at: 2026-10-09T09:55:00Z
updated_at: 2026-10-10T02:05:00Z
plan_revision: 2
current_step: 5
total_steps: 5
supersedes_spec_components:
  - docs/specs/modules/app-studio-ui.md
  - docs/specs/modules/workspace-ui.md
  - docs/specs/modules/ui-parity.md
new_spec_components: []
touched_goals:
  - goal-agent
  - goal-frontend-parity
---

# PLAN-101 — VM 轨 composer 发送与 workspace 当前项标记修复

## 0. 变更摘要

2026-10-09 用户实机启动原生 VM 栈（auto-lang `v0.4.2-2730` 当日构建 +
musk RustHTTP 后端 + 共享 aaid）撞出两个 VM 轨可用性缺陷：

1. **composer 发送完全无反应**——输入"你好"按 Enter 或发送钮，消息不出现在
   对话区。VM 日志实证：`handler synthesis failed: ChatsView..SendInput:
   Undefined variable: canvasStore` / `ChatsView..CtxRemove` 同款，两 handler
   被 poison-drop，`[VM-EMIT] MentionInput.send -> ChatsView.SendInput failed:
   handler not found`。根因：`chats_view.at` handler 体内引用模块级第二
   store 裸别名 `canvasStore`（`use store: CanvasStore`），VM 轨 handler 合成
   不绑定该别名。PLAN-097 T-01 基线登记过此债（"VM 轨 ChatsView
   SendInput/CtxRemove handler 合成失败，既有债"），当时演示驱动走 API 未
   撞用户输入侧，挂账至今。
2. **workspace 选择器当前项无标记**——打开弹层后当前 workspace（tmp）行
   既无 ✓ 也无高亮。MCP 快照实证：所有行均无 `ws-item-check` 节点；
   `autoui_inspect` 证实 tmp 行 `w.id="tmp"` 与 `.store.workspace="tmp"`
   值相等而条件块不渲染。根因：`workspace_selector.at:209` 模板条件
   `if .hasWs && .store.workspace == w.id` 属 VM 轨模板条件渲染断点族
   （097 T-05 根因矩阵："computed 内/模板跨 store 读在 VM 恒空"）。

本计划在专用 worktree 修复两缺陷：发送链把第二 store 读挪出 handler 作用域
（模块级 helper / 限定式跨 widget emit——均为已实证通道），workspace 选择器
当前项改经 handler 域预构建行数据 + 模板循环局部读（097 T-11 同款模式）。

## 1. 目标

### 可交付结果

1. 原生 VM 界面 composer 输入文本后 Enter/发送钮，消息作为 user 消息出现在
   对话区且后端收到请求；冷启 codegen 日志无 SendInput/CtxRemove 合成 WARN。
2. workspace 选择器弹层中当前 workspace 行渲染 ✓ + 背景高亮；切换 workspace
   后再次打开，新当前项标记正确、旧项标记消失。
3. Web/Vue 轨两行为不回退（发送语义含拾取 ctx 快照等价、选择器 ✓ 标记如常）。
4. 受影响静态门/回归全绿，证据绑定本计划提交。

### 范围、依赖与执行位置

| 项目 | 本计划责任 | 位置/约束 |
|---|---|---|
| auto-musk | 前端 .at 修复 + 真机验证 + 受影响门 | worktree `D:/autostack/.wt/musk-101/auto-musk`，分支 `plan-101-dev` |
| auto-lang / auto-ai | 引擎/后端 | 只读依赖，不得顺带修改；引擎缺口登记回交 |
| 执行位置 | 代码/产物在 worktree；计划文档与进度写主检出 | 跨仓依赖回落 `D:/autostack/<repo>` 主检出（前端改动无需 cargo 构建） |

worktree 内禁止 junction/symlink；Vue 依赖安装一律外部 staging
（097 T-01 冻结配方）。真机验证复用当日栈模式：musk serve（RustHTTP）+
`auto run --render vm --back-port`（AUTO_REUSE_BACKEND 复用门）+ AutoUI MCP。

### 非目标

- 不修拾取 chip 的 VM 可见性（`chats_view.at:805` 模板跨 store 条件同族
  缺陷，且 VM 帧点选安装为空桩——chip 在 VM 无可达触发路径），登记为既有边界。
- 不碰 G-15/G-17、VMHTTP split 接线、Relay/计划流（099/100 范围）。
- 不改后端、不动 aaid/共享服务配置。

## 2. 架构方案

```text
缺陷① 发送链（handler 域摘除第二 store 裸别名）
  SendInput: ctx 快照构造 → 模块级 helper（模块作用域解析 canvasStore
    别名——与模板同域；handler→模块 fn 调用 = auto-lang 733 后实证通道）
  CtxRemove: canvasStore.ClearPick() → 限定式 CanvasStore.ClearPick()
    （跨 widget emit 形态同 WorkspaceSelector.Choose 内 PlansStore.Reload()
    ——VM 轨工作实证）
  兜底：若模块级 helper 读第二 store 在 VM 仍返空/合成失败 → 改走
    ports 轨道分流（vm 变体返 None，web 不变）+ 登记上游缺口；VM 侧
    不因兜底损失现有能力（chip 本就不渲染，ctx 无从产生）

缺陷② 当前项标记（handler 域预构建 + 循环局部读）
  workspace_helpers.at 新增 ws_tag_current(recent, current_id)——行数据
    打 is_current 布尔（纯 fn，VM/web 双轨同语义）
  WorkspaceSelector Init/UpdateQuery/Choose/PickFolder 四处赋值点统一过
    tag；模板 if w.is_current（循环局部读=实证通道）+ 条件 style 高亮
```

## 3. 技术栈

- Auto 前端真源 `src/front/*.at`（单源双轨）；验证用 AutoUI MCP
  （snapshot/inspect/action/keyboard）+ 既有 scripts/ui-parity 门。
- `auto build --gen-only --strict` + `scripts/vm-link-probe.cmd` +
  `node scripts/ui-parity.mjs check` 静态三门（沿 097 V03 口径）。
- Vue 外部 staging 构建（vue-tsc + vite，097 T-01 冻结配方，worktree 无链接）。

## 4. 需求分析与背景调查

### 授权

用户 2026-10-09 明确指示："把上面两个问题合成一个修复计划
（/auto-plan:new），然后实施修复吧。"——new 草稿与 work 实施均为已授权
范围；未指定费用/时间预算，不虚构额度。review/merge 按技能流程后续执行。

### 权威 Spec 输入（SHA256）

| 路径 | 规划读取 hash |
|---|---|
| docs/specs/00-overview.md | FCF8EB5793E69E5AC9585668598E71257CFFCE091CB9063753932BEDB17266B9 |
| docs/specs/index.json | 3D1149673FE0F65476607F9831C1B4DAC7FB18668D0A3252821ADD8E754BF18E4 |
| docs/specs/modules/workspace-ui.md | DB74E023FA4B9896C7DBD59FF2F49B54A1BA0E5F1AA08449D994A4B32AC44C38 |
| docs/specs/modules/app-studio-ui.md | 8422A0DC14398EFF16385FADC7569334FDF82B4394295AD835803189946D890C |
| docs/specs/modules/ui-parity.md | 5194497B7E4F57F534226FB238C306DDDBAABE81A955CF6BF8A2668F838BA434E |
| docs/specs/modules/vm-data-semantics.md | BCF486CE4EB9E40CBC266F9290E1B0FF299AA8DCCC502D348C583E07FD28054B |

### 代码基线（main @ cb7d8dc8feb05cbc756f5207c770b567a05eea47）

| 文件 | SHA256 |
|---|---|
| src/front/chats_view.at | 2AADA8BA46AC4E71DA311D17ED60CDE446D617F6A83D53C32B8D3C107E3E38E8 |
| src/front/workspace_selector.at | 37B552B90EEC578272E26FD7F52951C4429E39DE456EA778C8968F379667990A |
| src/front/workspace_helpers.at | 8D281FACE9F39E33E2B1D8E9EB0ACE8F8774E4FF8F0330E64C82F2F79CEEC176 |
| src/front/canvas_store.at | BDAE09C24FAA4C0970CEA4BE92DA32F55AF0E316B14C2CC243D88D5D4400C5EB |
| src/front/ports/canvas.vm.at | AF2952AA96F1C7567FB37660D2AD433E5D101A8C3887210ECF770304598F4E93 |

### 已核实实现与缺口

- 缺陷①实证（主检出当日栈 tmp/dev-stack-vm.log:3641-3643,3894-4013）：
  合成 WARN + poison-drop + emit 无 handler；Web/Vue 轨无此缺陷（同源码
  web 合成路径正常）。097 worktree（.wt/musk-097，@dfc1e63）同样未修。
- 已实证可用通道（097 证据）：handler 域调用模块级 fn（733 修复后
  StatusBackfill 链）；跨 widget 限定式 emit（WorkspaceSelector.Choose →
  PlansStore.Reload() VM 工作）；模板循环局部读（`.w.name` 渲染）与
  属性位条件（onclick/style）；handler 域 store 读（.store.*）。
- 断点族（097 T-05 根因矩阵 + 今日复测）：handler 体内第二 store 裸别名
  （本次①）、模板条件内 computed+store 读（本次②）、模板跨 store 文本位读
  （chip，非目标）。
- CtxRemove 语义：`canvasClearPick().await`（后端清，ports 两轨齐备）+
  本地投影清。改限定式 emit 后语义不变（CanvasStore.ClearPick handler
  本体不动，canvas_store.at:327）。
- SendInput ctx 快照（PLAN-093 Q-04 DTO 14 字段）为 web 轨在用的真实
  能力；helper 提取是纯搬移，web 行为必须逐字段等价。

## 5. 详细设计

### 5.1 缺陷①：composer 发送链

1. `chats_view.at` 新增模块级 fn `pick_ctx_snapshot(workspace str, session str) Value`：
   模块作用域读 `canvasStore.cv_picked_*`（别名解析与模板同域），构造
   PLAN-093 Q-04 DTO；`cv_picked_id == ""` 返 None。SendInput handler 内
   `var ctx = pick_ctx_snapshot(.store.workspace, .store.session_id)` 替换
   原内联块（含 `if canvasStore.cv_picked_id != ""` 守卫，helper 内承担）。
2. `.CtxRemove`：`canvasStore.ClearPick()` → `CanvasStore.ClearPick()`
   （限定 emit；`.await` 仍属前句 `canvasClearPick()`）。
3. 验收实证：冷启日志零 `handler synthesis failed`（ChatsView 域）；
   MCP type_text + Enter 后快照见 user 消息节点，serve 日志见
   POST 发送请求；ctx=None 路径与 Queue/Send 分支不变。
4. 兜底（T-02 内判定，不改变本计划目标）：helper 方案若在 VM 合成或
   求值失败 → ports 分流（`ports/pick_ctx.web.at` 读 canvasStore 返回
   DTO；`ports/pick_ctx.vm.at` 返 None + 既有能力无损），并在 §9 记录
   上游缺口登记；不得为此修改 auto-lang。

### 5.2 缺陷②：workspace 选择器当前项

1. `workspace_helpers.at` 新增 `ws_tag_current(recent []Value, current_id str) []Value`：
   逐行置 `is_current: (w.id == current_id)`，返回新列表（纯 fn）。
2. `workspace_selector.at` 赋值点统一过 tag：
   - `.Init`：`.recent = ws_tag_current(ws_load_recent().await, .store.workspace)`，
     `.shown = .recent`；
   - `.UpdateQuery`：`.shown = ws_tag_current(ws_filter_recent(.recent, .query), .store.workspace)`；
     注：`.recent` 已带 is_current，过滤天然保留；此步为防御性重标，口径统一；
   - `.Choose(id)` / `.PickFolder` 成功链内：`.recent = ws_tag_current(.recent, 新 id)`
     （VM 下 location.reload 为 no-op，弹层可能保持开启，必须即时重标）。
3. 模板：`if .hasWs && .store.workspace == w.id` 块改行数据 mark 文本位
   （`span { style: "ws-item-check" text .w.mark }`，非当前行空串视觉等价
   不渲染；text 成员读=实证通道）。~~行 style 条件高亮~~（r2 撤：动态
   style 字段引用 VM 不产出——app-studio-ui 已知限制族，AC-04 修订登记）。
4. 已核实 `hasWs` computed 在触发器位（title 属性）工作正常，保留不动。

### 5.3 验证与回归设计

- 真机（worktree 起 VM + 复用 :19247 或自有端口）：composer 发送闭环；
  选择器打开/切换/重开三态快照；冷启合成日志零 WARN。
- 静态三门 + Vue staging 构建（097 配方）；ui-parity catalog 对账
  （新增 fn 若入声明清单则补 case，否则保持计数）。
- 受影响回归：workspace 切换链（SetWorkspace/Reload/RefreshAuth 顺序）
  与发送链（Queue/Send/relay 命令分支）逐项人工核 + 真机冒烟。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/modules/app-studio-ui.md | before：composer 交互契约未记 VM 发送断点（SendInput 债务仅散见 097 基线）；after：composer 发送在 VM 轨必须可用——SendInput/CtxRemove handler 不得引用第二 store 裸别名（VM 合成断点），拾取 ctx 快照经模块级 helper/端口链构造；拾取 chip VM 可见性登记为引擎模板条件族既有边界（非本模块承诺） | 发送是核心交互，双端同义 | AC-01,02,03,06 |
| SD-02 | modify | docs/specs/modules/workspace-ui.md | before：未约束选择器当前项标记的 VM 可达性；after：workspace 选择器当前项必须以可见标记呈现（✓；背景高亮受 VM 动态样式引擎限制，恢复后补），VM 轨经 handler 域预构建行数据（mark 预拼）+ 模板文本位成员读实现，禁止模板条件成员访问/直读 store | 当前项不可见导致切换无锚点（用户实报） | AC-04,05 |
| SD-03 | modify | docs/specs/modules/ui-parity.md | before：live 冒烟未列 composer 发送与 ws 当前项标记断言；after：两行为列入 VM 端 live 冒烟断言组件（composer 发送出现 user 消息节点；选择器当前行含 check 节点），fixture 回归覆盖 is_current 投影 | 防回漂，双端门语义一致 | AC-01,04,07 |

## 6. 测试设计

| 验证 | 命令/方法 | 期望与必要反例 |
|---|---|---|
| V01 真机发送闭环 | worktree 起 VM（MCP）：composer type_text「你好」→ Enter → 快照含 user 消息 + serve 日志 POST；再测发送钮 | user 消息节点出现；后端 200；冷启日志零 `handler synthesis failed`（ChatsView 域）；改前基线（emit failed）留证对比 |
| V02 选择器三态 | MCP：开弹层快照（tmp 行 ✓+高亮）→ press demo 行 → 重开弹层（demo ✓、tmp 无）→ 搜索过滤后标记仍在 | is_current 重标正确；必要反例：非当前行无 check 节点（改前全行无 check 留证） |
| V03 静态三门 | `auto build --gen-only --strict`；`scripts/vm-link-probe.cmd`；`node scripts/ui-parity.mjs check` | exit 0 / PASS / catalog 对账（计数变化须归因本改动） |
| V04 web 不回退 | 外部 staging Vue 构建（vue-tsc+vite）；ws_tag_current 纯 fn 行为断言（node 直测生成产物或等价 fixture） | 构建零错；tag 逻辑真值表（命中/未命中/空列表/重复 id）全对；SendInput web 语义逐字段等价（helper 提取纯搬移，code review 绑定） |
| V05 受影响回归 | 发送链分支核对（relay 命令/Send/Queue/ctx）；workspace 切换链核对（RefreshAuth→SetWorkspace→Reload 顺序）；ui-parity 受影响 case 复跑 | 无行为漂移；09x/09y 在途计划范围不重叠（099=流渲染/徽章/守卫/canvas 内嵌；100=relay stage/RunBox），无冲突文件 |

## 7. 验收标准

| ID | 可观察结果 | 验证 |
|---|---|---|
| AC-01 | VM composer 输入文本 Enter/发送钮后，消息作为 user 消息出现在对话区，后端收到发送请求 | V01 真机 |
| AC-02 | VM 冷启 codegen 日志 ChatsView 域零 `handler synthesis failed`；SendInput/CtxRemove 在导出表（emit 不再报 handler not found） | V01 日志 + emit 实证 |
| AC-03 | CtxRemove 按压后拾取态清除语义不变（后端 clear + 本地投影清，经 CanvasStore 限定 emit） | V01 + 代码核对（canvas_store.at ClearPick 本体不动） |
| AC-04 | workspace 选择器弹层：当前 workspace 行渲染 ✓ 标记，其余行无（r2 修订：背景高亮因 VM 动态 style 字段引用引擎已知限制不产出——实机取证，登记 §10 不上验收） | V02 快照+截图 |
| AC-05 | 切换 workspace 后不重载再次打开弹层，新当前项标记正确、旧项消失（含搜索过滤态） | V02 |
| AC-06 | Web/Vue 轨：选择器 ✓ 标记行为不变；composer 发送含拾取 ctx 快照语义逐字段等价 | V04 staging 构建 + 真值表 + review 绑定 |
| AC-07 | V03 三门全绿；catalog 计数变化全部归因本计划 | V03 |
| AC-08 | 无新增用户可见文本（✓/高亮均复用既有资源与 token），中英资源不动 | V03 静态门 + diff 核对 |

## 8. 执行步骤

| 完成/任务 | 依赖 | 文件/符号与产出 | 验证与预期 | AC |
|---|---|---|---|---|
| [x] T-01 | 无 | worktree `D:/autostack/.wt/musk-101/auto-musk`（plan-101-dev）；主检出基线 cb7d8dc；worktree 起 VM 复现两缺陷并留证（快照/log/截图，对比主检出今日已采证据） | 两缺陷在 worktree 构建下复现（emit failed / 全行无 check），证据落 tmp/ | 01,04 反例 |
| [x] T-02 | T-01 | chats_view.at：`pick_ctx_snapshot` helper + SendInput 改接；CtxRemove 限定 emit | V01：发送闭环真机 PASS；合成零 WARN；兜底判定记录（helper 成功或 ports 分流） | 01,02,03 |
| [x] T-03 | T-01 | workspace_helpers.at：`ws_tag_current`；workspace_selector.at：四赋值点 + 模板 `w.is_current` + 高亮 style | V02 三态真机 PASS | 04,05 |
| [x] T-04 | T-02,T-03 | V03 三门 + V04 staging 构建 + 真值表 + V05 受影响核对 | 全绿；计数归因 | 06,07,08 |
| [x] T-05 | T-04 | worktree 提交（plan-101-dev）；本 Plan 进度/交接记录更新；owned 进程清理核验 | wt-guard clean；真机残留清理 | 全部 |

## 9. 复审记录

### new 阶段交接（草稿准备完成）

- stage: new
- plan_id: PLAN-101
- plan_revision: 1
- outcome: pass
- next: work（用户已授权实施："然后实施修复吧"）
- changed_tasks: T-01～T-05
- changed_acceptance: AC-01～AC-08
- spec_delta: SD-01～SD-03
- evidence: §4 代码/Spec hash、今日实机取证（VM 日志合成 WARN 行、MCP 快照
  全行无 check、inspect 值相等不渲染）；097 T-01 基线债登记与 T-05/T-11
  实证通道。此结果只表示草稿可交接，不表示缺陷已修复。
- review_authority: 独立 auto-plan:review 在 worktree 复验实现/证据并冻结
  Spec delta；auto-plan:merge 沉淀与清理。

### work 阶段记录（T-01～T-05 完成；plan_revision 1→2）

- stage: work | plan_id: PLAN-101 | plan_revision: 2 | outcome: pass
- code_commit(worktree plan-101-dev): 5b2fa9f（基 cb7d8dc）
- task_ids: T-01, T-02, T-03, T-04, T-05
- evidence:
  - **T-01**：worktree 起 VM（AUTO_EXE 锁当日构建 auto v0.4.2-2730）复现两
    缺陷——合成 WARN（ChatsView..SendInput/CtxRemove: Undefined variable:
    canvasStore）+ type/Enter 后 `[VM-EMIT] MentionInput.send -> ChatsView.
    SendInput failed: handler not found`；store facade 日志确认模块解析落
    worktree 路径。
  - **T-02（含 §5.1 兜底触发）**：主设计"模块级 helper 读第二 store 别名"
    实机证伪——`import stmt: Undefined variable: canvasStore` poison-drop
    且 VM UI init 失败；按兜底改端口 states 通道：`canvasPickSnapshot`
    （ports/canvas.vm.at 无态直查 GET /api/canvas/status + canvas_web.ts
    双轨同实现，DTO 逐字段对齐原内联块含 forctx→loop_context 映射）。
    CtxRemove 改限定式 `CanvasStore.ClearPick()`（VM 冷启合成零 WARN，
    形态同 PlansStore.Reload 先例）。真机：发送钮闭环 POST
    /api/chats/session/<sid>/message 200 + user 消息可见 + composer 清空 +
    `handler_ChatsView_SendInput parked (wait HttpRequest) resumed to
    completion`（含快照 HTTP）；切 workspace 后无会话发送 404 为既有
    空会话行为（非回归，会话激活后 200）。
  - **T-03（三次实机迭代）**：① `w.is_current = ...` 静态名 SET_FIELD 在
    VM handler 续体 RuntimeError（Init resume FAILED）→ JSON 克隆+动态键
    `item["is_current"]`（gate_init_edited_specs 同款）通过；② 模板
    `if w.is_current` 成员访问条件实测不渲染（Choose 字面 id 重标后仍无
    ✓）→ 判定 VM 模板 if 条件成员访问为断点，✓ 改行数据 mark + text 成员
    读（text .w.name 实证通道）；③ 动态 `style: .w.row_style` 不产出
    （app-studio-ui 已知限制族）→ 背景高亮撤出（AC-04 r2 修订），按钮
    样式恢复静态串。时序竞争：Init 先于 app.at 解析链 resume（store 仍空）
    → app.at SetWorkspace 后补发 `WorkspaceSelector.WsResolved(id)` 定向
    重标。真机三态：冷启首开 tmp 行 ✓ / Choose demo-1 后 ✓ 迁移且弹层
    保持开启 / store workspace 正确切换（快照+截图
    src/front/tmp/autoui-screenshot-1791542247106.png）。
  - **T-04**：gen-only strict exit 0（65 组件）；vm-link-probe PASS
    （107777B WARN 体积趋势同基线）；ui-parity catalog PASS 114/114；
    Vue 外部 staging 构建 PASS（vue-tsc+vite 17.3s 出 dist，.demo-build/
    musk-101/vue-build，worktree 无链接 wt-guard clean）。
  - **T-05**：提交 5b2fa9f；wt-guard clean；真机验证 VM 实例**保留运行**
    供用户试用（非残留，用户可见窗口； owned 清理随 merge/用户指示）。
- spec_delta: SD-01～SD-03 落地文本在 review 冻结（SD-02 高亮句随 AC-04
  r2 修订——"✓ + 背景高亮"改"✓ 标记，高亮登记引擎限制"）。
- blockers: 无（三项引擎限制已取证并绕开/登记 §10，均不动 auto-lang）。
- next: review（独立 auto-plan:review 复验 5b2fa9f）

## 10. 待澄清事项

1. ~~模块级 helper 读第二 store 在 VM 的可达性~~（T-02 已判定）：模块级
   fn 读 `use store:` 别名在 VM 同为 poison-drop（`import stmt:
   Undefined variable: canvasStore` + VM UI init 失败实证）；已按 §5.1
   兜底走 ports states 通道（能力无损），**登记为 auto-lang 上游缺口**：
   handler/模块 fn 域的第二 store 别名解析（owner 视角可复核是否并入
   733 族修复）。
2. **拾取 chip VM 可见性**（owner=后续计划）：`chats_view.at:805` 模板跨
   store 条件同族断点，VM 帧点选安装为空桩、无可达触发路径，本计划登记
   不修；若上游引擎修复模板条件族，chip 可见性应自然恢复，届时复验。
3. ~~ui-parity catalog 计数~~（T-04 实跑归因）：ws_tag_current/
   canvasPickSnapshot 不入声明清单；移除 Check 未动计数；114/114 PASS。
4. **本轮新增实机取证的引擎断点（均绕开/登记，未动 auto-lang）**：
   ① handler 续体内静态名字段写（`item.x = v`）RuntimeError；动态键
   （`item["x"] = v`）为已证通道（JSON 克隆后写入安全）；
   ② 模板 `if` 条件的循环变量成员访问恒不渲染（裸 binding/模型布尔
   可用——`.collapsed`/`.studio_mode` 实证；成员访问断——`w.is_current`
   字面重标后仍不渲染实证）；
   ③ 动态 style 字段引用（`style: .w.x`）不产出（与 app-studio-ui
   "VM 轨动态 :style 字段引用不产出"同族）；静态串与裸 binding 的
   if-else 全静态串可用。
   候选收口：auto-lang VM 渲染/合成三断点（①②③）一次修复后，本计划
   的 mark/row_style 预拼可回退为直读条件。

### review 阶段记录（独立复验 5b2fa9f）

- stage: review | plan_id: PLAN-101 | plan_revision: 2 | outcome: pass
- reviewed_commit: 5b2fa9f（worktree plan-101-dev，status clean）
- base_commit: cb7d8dc
- dependency_revisions: auto-lang release v0.4.2-2747（D:/autostack/auto-lang
  target/release，launch-vm 同款引擎）；aaid 127.0.0.1:17654 共享实例
- spec_inputs: §4 六份 Spec hash 未变（主检出 docs/specs 现值核对：目标三文件
  无 composer/选择器标记契约在案，SD after 规则无冲突）
- 独立性声明：复审在新会话上下文重建证据，未沿用 work 阶段摘要；真机栈为
  复审自建（worktree 源 + release 引擎 + 复用 :17201 RustHTTP 后端 +
  AutoUI MCP :9377，AUTOUI_ACCEPTANCE=1 验收通道）
- acceptance_results:
  - AC-01 **pass**（真机） autoui_type「你好」→ 发送钮 press：handler
    `.MentionInput.send你好`、state_changes `text: "你好" -> ""`（composer
    清空）；autoui_exists 仅剩对话区 `Text "你好"` 节点；serve 日志
    `POST /api/chats/session/0b5c16c7…/message?workspace=tmp` 200；VM 日志
    `handler_ChatsView_SendInput parked (event SendInput, wait
    HttpRequest(375))` → `resumed to completion`（ctx 快照端口链在跑）。
    Enter 腿：合成焦点不可达（type=handler 臂/textarea press 非法/desktop
    臂排队无焦点投递），`onkeydown.enter.exact.prevent: .send(this.text)`
    与发送钮 onclick 同一 `.send` emit 链（按钮腿已实证到 handler+POST），
    快照原文在案——记为同链等价证据 + harness 限制（见 F-03），与 work 阶段
    V01 实采按钮腿证据口径一致。
  - AC-02 **pass** 两次冷启（含/不含 AUTOUI_ACCEPTANCE）日志零
    `handler synthesis failed`、零 poison；vm-link-probe 以
    MUSK_APP_PATH 指 worktree 源：零 poison、`PASS — musk frontend links
    on VM target`（106147B，WARN 带内）。
  - AC-03 **pass** diff 核对：canvas_store.at 未动（ClearPick 本体原样）；
    CtxRemove 已改限定式 `CanvasStore.ClearPick()`；V01 走 ctx=None 路径
    发送闭环。
  - AC-04 **pass**（真机）弹层快照：tmp 行 `text "✓"`，demo/v083-ws/
    auto-edit/backend 行均无 check 节点。
  - AC-05 **pass**（真机）Choose(demo-1)：`workspace: "tmp" -> "demo-1"`，
    弹层未关，✓ 迁至 demo 行；过滤 "de"：demo 行 HAS ✓、musk-demo 无；
    Choose(tmp) 回切：✓ 回迁 tmp 行。
  - AC-06 **pass**（复用+补证）staging 构建收据同 commit 复用（T-04，
    产物已按 T-05 清理）；复审补 gen-only --strict 65 组件零错（web
    codegen 面）；canvas_web.ts 与 canvas.vm.at 快照 DTO 逐字段对齐
    （review 逐行读 diff 核对，含 forctx 缺省 → index -1 三字段空）。
  - AC-07 **pass** 三门复审实跑全绿：gen-only strict exit 0（65 组件）；
    vm-link-probe PASS（主检出源 108065B / worktree 源 106147B）；ui-parity
    catalog PASS 114/114（live 0/4 为离线 skip 留痕，非红）。
  - AC-08 **pass** diff 七文件无 i18n/资源改动；✓ 复用既有 token。
- findings（均不阻断）:
  - F-01(info) vm-link-probe 缺省链接**主检出**语料（auto-lang
    plan442_musk_probe_tests.rs locate_musk_app: MUSK_APP_PATH 优先，否则
    `../../../auto-musk`）——输出中的
    `dropping poisoned export handler_ChatsView_SendInput/CtxRemove` 为
    未修主检出的已知缺陷签名，非本计划回归；指 worktree 源即零 poison。
  - F-02(minor,引擎边界) canvasPickSnapshot 退化态语义差：picked 存在但
    vnode_id 空时返空 vnode_id DTO（原内联守卫返 None）。实际后端 picked
    蕴含 vnode_id 非空，不可达；登记不修。
  - F-03(harness) 合成 Enter 需原生焦点，MCP 三臂均不可达（见 AC-01）；
    非应用缺陷，录入 ui-parity live 冒烟配方时以发送钮为注入路径。
- evidence: 本记录内联摘录为耐久证据（快照/日志原件在复审临时栈，已随
  owned 进程清理；关键行文如上）。Spec delta SD-01～SD-03 复审通过：
  after 文本与实机行为一致（SD-02 已含 r2 高亮登记修订），目标文件与
  现行 Spec 无冲突，live-required 契约语义与 ui-parity.md PLAN-080 契约
  一致（离线 skip 留痕）。冻结：SD 文本以本 Plan §5.5 表为准随 merge 落地。
- next: merge（auto-plan:merge 落地 + 归档 + 清 worktree）

### merge 收据（PLAN-101:r2）

- stage: merge | plan_id: PLAN-101 | plan_revision: 2 | outcome: pass
- prepared: reviewed_commit 5b2fa9f（worktree clean）；delivery 前备=SD-01～03
  落 docs/specs 三文件（projection-only 后代，实现/依赖零变化核对）
- landed: rebase main（ea35e52）后 ff-only；旧→新映射 5b2fa9f→b8810b8、
  e200db1→13509e9，`git range-diff ea35e52..5b2fa9f ea35e52..plan-101-dev`
  逐条等价（`1: 5b2fa9f = 1: b8810b8`）；main tip=delivery **13509e9**，
  无 merge commit；三份 Spec 增量段在主检出现值确认
- ledger_refreshed: workspace `auto-musk`（store-mediated POST
  /api/specs/item @17201），version 18→21，读回校验三条目在案：
  designs/PLAN-101-vm-composer-send、reviews/PLAN-101-review、
  reports/PLAN-101-delivery（均 Draft，source 引用 delivery 13509e9）
- archived: docs/plans/archived/101-vm-composer-send-and-ws-current-mark.md
  status: archived；completion_kind: delivered
- 工件时点观察（landing ≠ deployment）：①VM UI 轨源码即时编译，主检出
  重启即生效（无需构建）；②gen/front/vue/dist 为 merge 前构建（web 轨
  stale，按需 `auto build`+pnpm build 重建）；③backend release musk.exe
  缺位——本计划未触后端，launch-vm.cmd 已具 release→debug 回退
- cleaned: （清理后补记）
