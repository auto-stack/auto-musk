---
plan_id: PLAN-090
status: archived
completion_kind: delivered
feature_name: App Canvas M3 —— 三层生成流（bp/词汇表注入 + L1>L2>L3 复用序 + ui.lint 护栏 + examples 扩展池）
author: [agent]
created_at: 2026-09-23T13:42:38+08:00
updated_at: 2026-09-23T23:55:00+08:00
plan_revision: 2
current_step: 8
total_steps: 8
supersedes_spec_components: []
new_spec_components:
  - docs/specs/modules/ui-lint.md
touched_goals: []
---

# PLAN-090 — App Canvas M3：三层生成流（Design 013 第三个里程碑）

## 0. 变更摘要

M1（PLAN-087，实况画布最小闭环）与 M2（PLAN-088，双向锚定）已交付归档。本计划落
Design 013 §9 的 **M3 三层生成流**：把 AutoUI 的 widget/blueprint/app 三层架构变成
agent 生成 app 时的**可用词汇与护栏**——

1. **bp 库注入**：blueprint 目录摘要进 coding 模式系统上下文，`bp_list/bp_show/
   bp_check` 三工具接通 auto-lang 既有 `auto bp` CLI（spec 查看 + 静态验收门）。
2. **词汇表注入**：stdlib widget 分类名单 + 核心层统计进系统上下文（替换 M1 的
   手写"safe subset"），运行时探测 + 缓存 + 内嵌兜底降级链。
3. **L1>L2>L3 复用序**：生成指导 v2 重写——bind 现成 blueprint 优先、拷参考实现
   次之、自由生成兜底；编辑流嵌入"层归属三问"纪律（Design 013 §6）。
4. **ui.lint 护栏**：musk 侧规则注册表 v1（6-8 条静态可检坑规则，来自视图坑/
   codegen 坑实锤清单），`ui_lint` 工具 advisory 快门 + 每规则红绿样例锚定。
5. **examples/ui 扩展池**：后端只读工具 `app_examples_list/read`（兄弟位解析，
   与 AUTO_EXE 同源），解开 M1 登记的"沙箱 read_file 不可达仓外路径"缓行项。
6. **验收循环定型**：bp_show 选型 → 写 .at → ui_lint → bp_check → canvas_run
   实况 → act/state 断言，**≤3 轮**就地修复（N 默认值写进指导）。

零前端面板改动（M2 交付形态维持）、零 auto-lang 主路径依赖（全部经子进程/只读消费；
若 T-01 判定需 auto-lang 改动则走兄弟 worktree 规则单列条件任务）。

## 1. 目标

**做什么**：为 coding 模式的 agent 配齐"三层生成流"的输入（词汇表/bp 目录/demo 池）、
护栏（ui_lint + bp_check）、与流程纪律（复用序 + 层归属三问 + N 轮循环），并以一条
live e2e 生成流验收全链。

**非目标**（明确排除）：

- 前端画布面板/层树 UI 改动（M2 已交付形态维持；M3 是生成侧/agent 工具面）。
- `blueprint.extract` 飞轮与 variant promotion review（M4）。
- AppViewport 原生嵌入/路线 B（M4）；多画布会话；用户画布交互驱动。
- musk 自身前端 .at 的 lint 合规改造（ui_lint 服务"生成的目标 app"，不反身执法）。
- 通用 web 预览/多语言代码编辑器（Design 013 §10-5 范围纪律）。

**成功样貌**：在 musk coding 模式对话里说"做一个笔记列表 app"，agent 先报 bp 选型
（note-list 命中）、按 spec+参考实现生成 workspace app、ui_lint/bp_check 双门过、
canvas_run 起实况帧、act/state 断言通过、≤3 轮内收束——全程截图与结构化证据回流
对话。

## 2. 架构方案

```
                    coding 模式 agent
   系统上下文（注入段 ≤8KB，惰性缓存）        工具面（新增 6 件，白名单过滤）
 ┌──────────────────────────────┐   ┌─────────────────────────────────────┐
 │ bp 目录摘要（kind/name/intent │   │ bp_list / bp_show / bp_check        │
 │  +palette 简表，18 包 7 kind）│→→│  （subprocess: auto bp …）           │
 │ 词汇表摘要（stdlib 7 分类 53  │   │ ui_lint{path}（musk 原生规则注册表） │
 │  widget 名单 + tier 统计）    │   │ app_examples_list / app_example_read│
 │ 生成指导 v2（复用序/三问/循环）│   │  （后端只读，兄弟位解析）            │
 └──────────────────────────────┘   └─────────────────────────────────────┘
        惰性探测 + 缓存 + 降级链：runtime 子进程（auto bp list --format at /
        schema 摘要）→ 不可达 = 内嵌 M1 安全子集兜底 + degraded 标记
```

- **数据源全部经 auto-lang 既有 CLI/检出消费**：`auto bp list --format at`
  （cmd_bp.rs emit_catalog_at，kind/name/spec_text/gotchas/references 全量表）、
  `auto bp show`（spec.md 原文+variants+gotchas）、`auto bp check`（loading/error
  态 + palette 硬门，非零退出=失败信号）。AUTO_EXE 解析序复用
  `canvas/session.rs`（env → 编译期兄弟位 → 主检出 → PATH）。
- **两级注入**：目录/词汇只进**摘要**（系统上下文有预算），详情按需 `bp_show` 取
  ——避免 18 包全文（~100KB 级）撑爆系统提示。
- **ui_lint = musk 原生 advisory**：轻量文本级规则（不做完整语法解析），产出
  "发现 + 处方文案"；硬门仍是 bp_check（auto-lang 权威静态门）与 canvas_run
  （实况启动）——ui_lint 是第一道快门，不是真理裁决。
- **examples 池走后端只读**，不放开 agent 沙箱（read_file 白名单不动）；指导中
  同时告知用户可经 UI 白名单把 auto-lang/examples 加为 extra root（备选路径）。

## 3. 技术栈

- musk backend（Rust/axum/tokio）：新 `canvas/bp_tools.rs`、`canvas/ui_lint.rs`、
  `canvas/examples_pool.rs`（或并入既有文件，T 期定）；lib.rs 注入点扩容。
- auto-lang 既有面（**只消费不改**）：`auto bp list/show/check`、`schema/aura.at`
  （468 元素注册表）、`stdlib/aura/widgets/`（7 分类 53 件）、`examples/ui/`
  （34 demo）、`docs/components/core.md`（docs_gen 产物，参考）、
  `validators.rs` R001-R009（`auto build --strict` 内，T-01 评估可达性）。
- 测试：单测（红绿样例/解析/降级链）+ live 双臂（`#[ignore]+#[serial]`，沿
  `tests/canvas_live.rs` 形态）。

## 4. 需求分析与背景调查

### 已授权范围

- 用户指示：PLAN-088（M2）已完成，继续下一阶段计划规划（auto-plan:new，
  2026-09-23）。本计划起草属既定 Design 013 M0-M4 分期的第三里程碑，无超出
  战略设计的新范围。
- 执行授权：按 plan flow 惯例，评审后由用户启动 work；未授权自动续跑预算。

### 基线勘察（2026-09-23，主检出 main 0c702d0）

- **M1/M2 已交付面**（docs/specs/modules/app-canvas.md，main 2999cf5+0c702d0）：
  canvas 七工具（run/stop/snapshot/act/state/pick/overlay）、API 五路由、
  锚定契约、生成侧双模板（counter/hello）+ GENERATION_PROMPT v1（手写 safe
  subset 词汇）、coding.at 白名单收录七名。注入点 lib.rs:194-204（coding 模式
  追加 generation_prompt()）；工具注册 lib.rs:371-377（白名单过滤）。
- **auto-lang bp 体系**（Explore 勘察 2026-09-23）：`auto bp list --format at`
  产静态表（已提交产物 examples/bps-gallery/src/front/registry.at）；bp 目录
  18 包/7 kind（form 4/navigation 3/data-display 5/layout 3/feedback 2/
  editor 1/dashboard 1）；spec frontmatter=palette/extension_points/variants/
  dataSource/props/actions/acceptance；`auto bp check` 硬门=loading 态+error 态
  +palette 合法集；L1 bind/L2 copy/L3 generate 三档契约（docs/specs/blueprint/
  contract.md）与 agent-generation-workflow 设计（spec 即 skill 文件、check+build
  双门 ≤N 轮、L3 是兜底非默认）。
- **词汇表资产**：schema/aura.at 6023 行 468 元素（tier：builtin_widget 114/
  native_html 48/unclassified 77/web_component 225/package_origin 4）；
  stdlib 7 分类 53 widget；`auto docs gen` CLI 在（core.md 2315 行产物）。
- **坑知识双形态**：人读 autoui-skill（C1-C9/P1-P5）+ 机器执行 validators.rs
  R001-R009（auto build --strict 内）；musk 侧实锤坑清单（视图坑 + codegen
  quirks 记忆册）为 ui_lint v1 规则源。
- **examples 池**：34 个编号 demo（001-helloworld..041-auto-edit，024-040 空洞
  有去向注记）；002-counter=pac.at 6 行+app.at 49 行。

### 环境与依赖项

- **auto-lang 主检出 `blueprints/` 处于已删除未暂存态**（他方 WIP，PLAN-088 merge
  收据已呈报未触碰；git HEAD 完好，stash 无对应项）。M3 运行时主数据源依赖该
  目录 → 工具必须 loud-degraded；开发/e2e 以指向已知良好 blueprints 的构建验证
  （见 §10-1）。
- musk 主检出另有他方 WIP：extern_impl.rs models:None ×2（auto-ai 034 解阻，
  PLAN-089 在档）；PLAN-089（VM UAT 批次）executing 中，触 files/plans 视图族，
  与本计划 canvas/生成侧面**无文件冲突**。
- 主检出 musk build 已可构建（PLAN-089 T-02 0923 后端 release 重建实证，086 的
  wgpu-hal 阻断未复现）。

## 5. 详细设计

### 5.1 bp 通道三工具（canvas/bp_tools.rs）

| 工具 | 参数 | 行为 | 失败面 |
|:---|:---|:---|:---|
| `bp_list` | — | 子进程 `auto bp list --format at` → 解析为结构化清单（kind/name/intent 一行/palette/variants 数）；输出精简表格 | auto 不可达/blueprints 空 → **degraded 报文**（说明缺失与恢复指引），非空成功 |
| `bp_show` | `{kind,name}` | 子进程 `auto bp show kind/name` 原文透传（spec+variants+gotchas，本就是 agent skill 接口形态）+ 截断护栏（>16KB 头部截断提示用 read 面自查） | 同上；未知名 → 报错并列出可用名 |
| `bp_check` | `{path,spec?}` | path 经 `resolve_within_sandbox`（与 canvas_run 同口径）；子进程 `auto bp check <abs> --spec kind/name`（无 spec 时免 spec 门仅跑行为门）；退出码+输出透传 | 沙箱越界 Err；auto 失败输出原样回喂（修复信号） |

- 子进程统一走 AUTO_EXE 解析序（复用 session.rs 既有函数，抽公用）；cwd=workspace
  根；超时（30s）防挂。
- 目录摘要（系统上下文用）与 bp_list 同源：一次性拉 `--format at` 全量表 →
  提取 per-bp 摘要行 → 进程内缓存（AppState/OnceCell，TTL 或随启动一次）。

### 5.2 词汇表注入（canvas/vocabulary.rs 新建）

- 摘要形态：stdlib 7 分类 53 widget 名单（分类:逗号名单）+ builtin_widget tier
  计数 + "safe subset 强调段"（M1 手写名单保留为兜底与速查）。
- 生成方式：优先运行时从 auto-lang 检出读 `schema/aura.at` + `stdlib/aura/`
  目录清单（兄弟位解析，与 AUTO_EXE 同源序）；检出不可达 → **内嵌快照兜底**
  （build 时手工物化一份名单常量，随本仓版本演进，标注数据源版本）。
- 注入预算：词汇表+bp 目录+指导合计追加段 ≤8KB（单测断言字符数上限）。

### 5.3 ui_lint 规则注册表 v1（canvas/ui_lint.rs 新建）

规则=（id、意图、模式、处方文案、severity）。advisory 语义：报告≠失败，指引
agent 修。v1 候选 8 条（工作期按红绿样例可裁并）：

| id | 坑源 | 检出意图 | 处方要点 |
|:---|:---|:---|:---|
| L001 | 088/089 #24 | `span` 挂 onclick（升格 button 丢非文本子节点） | 行内可点元素一律显式 `button` |
| L002 | 视图坑 #1 | computed 块内调用 use.web 声明的 fn | 移 handler 域，computed 只读 state |
| L003 | 视图坑 | `t(` 非字面量键（动态键 VM 查表失明） | 键字面量化 |
| L004 | codegen | handler/msg 名含下划线 | 改驼峰 |
| L005 | 089 #26 | `.length` 出现在 computed/视图表达式（静默返空） | handler 域算好拍平标量 |
| L006 | 089 #26 | `list.join(` 链接期 Undefined symbol | 手拼循环 |
| L007 | codegen #19 | `style {` 块/含 CSS 声明串的 style 值 | tailwind 类串 `style: "…"` |
| L008 | 084-D5 前置 | 行括号深度/计数异常（爆炸前置告警） | 拆分表达式 |

- 实现：行级/轻量块跟踪扫描 .at 文本，零新依赖；误报可能 → severity 分
  warn/advice，报告头部声明 advisory 性质。
- `ui_lint{path}` 工具：path 沙箱解析 → 全规则跑 → 结构化发现列表（id/行号/
  摘录/处方）。生成指导 v2 要求写完 .at 后自跑。

### 5.4 examples 扩展池（canvas/examples_pool.rs 新建）

- `app_examples_list`：读 auto-lang 位 `examples/ui/README.md` 总览（解析编号/
  名称/一句话）+ 目录 census；不可达 → degraded。
- `app_example_read{name,file?}`：默认返回该 demo 的 pac.at+src/front/app.at
  （file 可指定其他源文件）；只读、大小截断护栏（>32KB 提示）。
- 解析序与 AUTO_EXE 同源（env 覆盖 → 编译期兄弟位 → 主检出），**后端直读不经
  agent 沙箱**；只读面不引入写路径。

### 5.5 生成指导 v2（templates.rs GENERATION_PROMPT 重写）

结构（保持英文输出、模板插值机制沿用）：

1. **Reuse ladder**（L1>L2>L3）：能 bp_check 过的 bp 组装优先（bp_show 读 spec）
  → 次选参考实现拷贝改（bp add --reference 语义=整文件拷入再改）→ 兜底模板/
  自由生成；每级说明何时降级。
2. **层归属三问**（编辑纪律，Design §6）：一次性微调（改实例 prop/slot）？模式
   级改动（升 bp 定义）？新能力（新 bp/widget）？——回答后再动手。
3. **Vocabulary**：§5.2 摘要 + safe subset。
4. **Blueprint catalog**：§5.1 摘要表。
5. **Examples pool**：app_examples 用法一句。
6. **Verification loop**：写完自跑 ui_lint（advisory 先修）→ bp 消费时 bp_check
   （硬门）→ canvas_run（实况启动即验收）→ snapshot/act/state 断言 → **≤3 轮**
   就地修复后必须汇报残留。
7. 保留 M1 已知坑节选（实证有效部分）。

- coding.at 白名单扩容：+bp_list/bp_show/bp_check/ui_lint/app_examples_list/
  app_example_read（共 13 canvas 系工具）；非 coding 模式零变化（注入点同 M1
  门控）。

### 5.6 条件任务：`app_check`（T-01 实测判定）

T-01 实测判定：`auto ui inspect` 诊断输出 exit code 恒 0 且无严格结构化机读门控，`auto build --gen-only --strict` 触发 Vue 项目生成写盘副作用（耗时 1-2s）。因此 `app_check` 缓行并登记，验收门维持 `ui_lint`（musk 原生轻量 advisory）+ `bp_check`（auto-lang 静态行为硬门）+ `canvas_run`（实况启动验收门）三道。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|:---|:---|:---|:---|:---|:---|
| SD-01 | modify | docs/specs/modules/app-canvas.md | before：M3 仅"后续里程碑锚点"占位（bp 注册表/词汇表注入、L1>L2>L3 复用序、ui.lint 护栏、examples 扩展池）；after：新增「三层生成流（M3）」节——bp 三工具契约与降级链、两级注入与预算、examples 池只读语义、生成指导 v2 要点（复用序/三问/N=3 轮）、工具登记表 7→13；锚点表移除 M3 行 | M3 落地行为的规范沉淀，与 M1/M2 节同构 | AC-01..AC-07 |
| SD-02 | add | docs/specs/modules/ui-lint.md | before：无（坑知识散在归档计划/记忆）；after：ui_lint 规则注册表契约——规则 id/意图/模式/处方/severity 字段、advisory 语义（报告≠失败）、红绿样例锚定要求、新增规则流程（坑源引证+双样例） | 规则集将跨里程碑增长，需独立可引用的规范册 | AC-03 |

无其他 Spec 影响：前端零改动（workspace-ui.md 不动）；ui-parity.md 不动（消费面
是 musk backend 而非测试基建）。

## 6. 测试设计

- **单测（常跑）**：bp 目录摘要解析（fixture：miniature --format at 样本）；
  注入段尺寸 ≤8KB 断言；降级链（探测失败→内嵌兜底+degraded 标记）；ui_lint
  每规则红绿样例（fixture .at 文件，一红一绿）；vocabulary 摘要生成；指导文本
  关键词锚定（reuse ladder/three questions/≤3 rounds）。
- **live 臂（`#[ignore]+#[serial]`，AUTO_EXE 可执行 + blueprints 可达环境）**：
  bp_list 18 包/7 kind 全量；bp_show 全文往返；bp_check 好/坏样例退出码；
  examples list ≥30/read 002-counter 往返；degraded 实测（AUTO_EXE 指向剥离
  blueprints 的构建或 AUTO_BLUEPRINTS_ROOT 指空目录）。
- **e2e 生成流（live，验收主链）**：预置坑样例（span+onclick+icon 子节点）→
  ui_lint 抓 L001 → 修复 → 以 note-list（或 login）bp spec 生成 workspace app
  → ui_lint 零红 + bp_check 过 → canvas_run 首帧 → canvas_act press +
  canvas_state 断言 → stop → census 零孤儿（沿 canvas_live.rs 口径）。
- **回归**：`cargo test -p musk` 全绿；`auto build` 双轨绿（musk 自身前端无改
  动，build 应零 diff）；既有 canvas_live 双臂不回归。

## 7. 验收标准

| ID | 判据 | 验证方法 |
|:---|:---|:---|
| AC-01 | coding 模式系统上下文含 bp 目录摘要（≥18 包 7 kind 全列）+ 词汇表摘要（stdlib 分类名单），追加段 ≤8KB；非 coding 模式注入零变化 | 单测断言 + 手动 inspection（构建 agent 后 dump 系统提示尾部） |
| AC-02 | bp_list/bp_show/bp_check 三工具往返正确；bp_check 坏样例（缺 loading 态）失败、好样例过；blueprints 不可达 → 显式 degraded 报文（非空成功、非 panic） | live 测试 + 单测降级链 |
| AC-03 | ui_lint v1 规则红绿样例判定全对（每规则一红一绿 fixture）；对干净样例（002-counter 源）零红 | 单测 fixture 集 |
| AC-04 | e2e 生成流闭环：bp 选型 → 生成 → ui_lint（含预置坑检出→修复）→ bp_check → canvas_run 起帧 → act/state 断言，全程 ≤3 修复轮 | live e2e 测试（§6） |
| AC-05 | app_examples_list 列 ≥30 demo；app_example_read 返回 002-counter 完整双文件；auto-lang 位不可达 → degraded | live 测试 |
| AC-06 | 安全面不回退：bp_check/app 文件路径经沙箱解析越界即拒；examples/bp 为后端只读面（无写路径）；既有 canvas 七工具与测试全回归 | 单测（越界样例）+ 全量回归 |
| AC-07 | coding.at 白名单收录新六件；指导 v2 含复用序/三问/N 轮关键词（单测锚定）；SD-01/SD-02 落稿与实现一致 | 单测 + review 比对 |

## 8. 执行步骤

| # | 任务 | 产出/文件 | 依赖 | 验证 | AC |
|:---|:---|:---|:---|:---|:---|
| T-01 | [x] [✅ 已完成] 勘察决策件（bounded probe，决策记录回填 §5）：①blueprints 活体（`--format at` 输出尺寸/解析面/工作树被删态 degraded 实测）②`auto build --gen-only --strict` 与 `auto ui inspect` 校验面与输出形态（§5.6 取舍）③L1 bind 工件在 VM 轨 workspace app 可运行性（pac.at dep 解析；不可行→复用序 v2 调整为 L2 先行+L1 登记 KNOWN-DEBT）④examples README 解析面 | 勘察记录（attachments/090/probe.md）+ §5 回填 + plan_revision 2 | — | 勘察记录实证落地，四项决策明确 | 全局 |
| T-02 | [x] [✅ 已完成] bp 通道三工具 bp_list/bp_show/bp_check + 目录摘要缓存 | canvas/bp_tools.rs、lib.rs 注册、coding.at | T-01① | 单测+集成测试全绿（bp_tools_test 3 项/单测 3 项，覆盖 live 往返/好坏样例/degraded，commit d7fce32） | AC-02 |
| T-03 | [x] [✅ 已完成] 词汇表注入（运行时探测+内嵌兜底+摘要生成）+ 注入段组装（目录+词汇+指导，预算断言） | canvas/vocabulary.rs、templates.rs 接线、lib.rs | T-02（摘要同源） | 单测全绿（7分类53widget快照与探测/追加段4.5KB≤8KB预算/关键词断言全过，commit 44bd88e） | AC-01 |
| T-04 | [x] [✅ 已完成] ui_lint 规则注册表 v1 + 工具 + 红绿样例集 | canvas/ui_lint.rs、tests fixtures、lib.rs、coding.at | — | 单测红绿全对（L001-L008 规则 8 组红绿样例+干净 002-counter 零红共 9 单测全绿，commit d90466d） | AC-03 |
| T-05 | [x] [✅ 已完成] examples 池两工具 app_examples_list/read（兄弟位解析+只读+截断） | canvas/examples_pool.rs、lib.rs、coding.at | T-01④ | live 往返/degraded（examples_pool_test 验证 ≥30 demo/002-counter 双文件/路径穿越拦截/degraded 均过，commit 1a638f3） | AC-05 |
| T-06 | [x] [✅ 已完成] 生成指导 v2 重写（复用序/三问/循环 N=3/坑节选保留）+ coding.at 白名单六件 + （条件）app_check | templates.rs、coding.at、（bp_tools.rs） | T-02..T-05 | 单测全绿（关键词断言/追加段4.5KB≤8KB预算/coding.at 13件全量断言通过，commit fea5e5b） | AC-07（§5.6→AC-02 面） |
| T-07 | [x] [✅ 已完成] e2e 生成流 live 测试 + 全量回归（cargo test/auto build 双轨/既有 canvas_live） | tests/canvas_live.rs 增臂或新 tests 文件 | T-02..T-06 | live 全绿（canvas_generation_flow_m3_e2e 15.53s 通过）+ 回归零红（496 单元测试 + bp/examples 集成测试全绿，commit 1394cf3） | AC-04/AC-06 |
| T-08 | [x] [✅ 已完成] 规范增量落稿（app-canvas.md M3 节 + ui-lint.md 新册）与账本衔接 | docs/specs/modules/{app-canvas.md,ui-lint.md,README.md} | T-07 | spec 比对与 wt-guard 零污染验证通过（commit b868394） | AC-07/SD-01/SD-02 |

执行纪律：musk 改动在 `.wt/musk-090/auto-musk`（分支 `plan-090-dev`）；若 T-01
判定需 auto-lang 改动 → 同组并排 `.wt/musk-090/auto-lang`（分支 `auto-musk-dev`），
集成验证通过即按 AGENTS.md 折回，不等整体收尾。

## 9. 复审记录

- 2026-09-23 new 起草：`stage: new | plan_id: PLAN-090 | plan_revision: 1 |
  outcome: pass`（起草完成：任务/验收/规范增量闭环，路径与命令均经主检出勘察
  落地；T-01 为有界勘察门，其结论可能以 plan_revision 2 回填复用序细节）。
  `next: work`（由用户启动；建议先过 §10-1 blueprints 环境确认）。
- 2026-09-23 work 实施完成：`stage: work | plan_id: PLAN-090 | plan_revision: 2 | outcome: pass | next: review`（T-01 至 T-08 全量交付：bp 工具三件套、词汇表 53 件注入、ui_lint 8 条规则注册表、examples 池只读通道、生成指导 v2 重构与 13 件白名单扩容、m3 e2e 生成流 live 闭环测试、app-canvas.md 与 ui-lint.md 规范沉淀；496 lib 单测 + bp/examples/live e2e 测全绿；wt-guard 洁净；待复审）
- 2026-09-23 review 复审通过：`stage: review | plan_id: PLAN-090 | plan_revision: 2 | outcome: pass | reviewed_commit: b8683949be9f8379ee2166e99eb692b20a89ec6b | base_commit: 4f8bf458f9f7c845c6681f0ac27d135981175a78 | dependency_revisions: auto-ai=58bee8d, auto-lang=39863eb | spec_inputs: docs/specs/modules/app-canvas.md, docs/specs/modules/ui-lint.md, docs/specs/modules/README.md | acceptance_results: AC-01..AC-07 all pass | findings: none | evidence: tests/canvas_live.rs (canvas_generation_flow_m3_e2e pass 3.54s), bp_tools_test (3 pass), examples_pool_test (2 pass), 35 canvas unit tests pass, 496 lib tests pass, wt-guard clean | next: merge`
- 2026-09-23 merge 合并与沉淀完成：
  `stage: merge | plan_id: PLAN-090 | plan_revision: 2 | outcome: pass`
  - **prepared**: reviewed_commit=b8683949be9f8379ee2166e99eb692b20a89ec6b; base_commit=4f8bf458f9f7c845c6681f0ac27d135981175a78; canonical specs=docs/specs/modules/app-canvas.md, docs/specs/modules/ui-lint.md, docs/specs/modules/README.md; delivery_commit=7f1389146df5d43ca6498305886657c9197c394c
  - **landed**: main 快进合回（git merge --ff-only plan-090-dev -> 7f13891）；range-diff 7/7 补丁逐条全等等价（d7fce32=ba60d9c, 44bd88e=8a95e5b, d90466d=afe075f, 1a638f3=01d4880, fea5e5b=f1b3777, 1394cf3=d4f4e75, b868394=7f13891）；线性历史无 merge commit
  - **ledger_refreshed**: docs/specs/index.json（spec_files 登记 modules/app-canvas.md 与 modules/ui-lint.md，updated_at=2026-09-23T23:54:00+08:00，commit 9325ac5）；.autoos/specs.json 离线原子刷新（app-canvas-D1 M3 增量与 sha256 dba0cc96...，新增 ui-lint-D1 sha256 bf6dea0f...，新增 app-canvas-R3 交付评审记录）
  - **archived**: docs/plans/archived/090-app-canvas-m3-generation-flow.md；completion_kind=delivered
  - **cleaned**: wt-guard clean；主 worktree D:/autostack/.wt/musk-090/auto-musk 已移除；plan-090-dev 本地开发分支已删除；兄弟 worktree auto-ai 与 auto-lang 干净折回并移除；组目录 D:/autostack/.wt/musk-090 已清理

## 10. 待澄清事项

1. **auto-lang 主检出 blueprints/ 已删除未暂存（他方 WIP）**：M3 运行时主数据源。
   开发不受阻（worktree/e2e 用已知良好位）；主检出恢复前生产体验为 degraded。
   需用户/所有者确认恢复时点——不阻塞 work 启动，但 e2e 验收环境需指定
   AUTO_EXE/AUTO_BLUEPRINTS_ROOT 指向完好 blueprints 的构建。
2. **N 轮上限默认 3**（Design 16 指标未定数值，本计划定 3 写进指导）——如需他值
   请在 work 启动前示下，缺省按 3 执行。
3. **注入预算 8KB** 为经验上限（≈2K token）——可调，缺省按 8KB。
4. **L1 bind 若 VM 轨不可用**（T-01③判定）：默认降级处置=L2 copy 先行 + KNOWN-DEBT
   登记 L1 待 auto-lang 支持；若用户要求 M3 内根修 auto-lang（跨仓任务），请明示
   扩权。
