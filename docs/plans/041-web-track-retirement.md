---
plan_id: PLAN-041
status: executing
feature_name: web 手写轨退役——gen(Auto/vue)轨转正为生产前端
author: [zhaopuming]
created_at: 2026-08-23
updated_at: 2026-08-27

supersedes_spec_components: []
new_spec_components: []
touched_goals: []

current_step: 2
total_steps: 16
---

# [PLAN-041] web 手写轨退役——gen(Auto/vue)轨转正

## 变更摘要

PLAN-037 收口后 `.at` 已是前端单一真源（widget 单轨 + 五域 ports），但**生产仍在跑手写
web/ 轨**（`backend/crates/musk/src/server.rs:67-76` serve `web/dist`；`start-musk-web.cmd`
跑 web dev :8090），且每个功能"双轨实现"（Plan 031-036 全部双写）的维护税持续存在。
本计划分四步退役 web/ 轨：①补齐 gen 轨功能缺口（Specs 组件组 21 件 + URL 路由）；
②编辑器组经 @autodown/editor 融合迁 .at（与 PLAN-038 T14 编辑库路线同轨）；
③生产切换（后端静态服务 + 启动脚本切 gen 产物，带回滚开关与观察期）；
④测试迁移 + web/ 冻结（保留历史参考，停止功能跟进）。

**前置关系（2026-08-23 用户裁定）**：**整个计划（含 Phase 1）等跨平台迁移完成后
启动**——迁移未竟期间 web/ 轨保持现维护节奏（不执行冻结令），本计划先行挂起。
原分相依赖保留供参考（Phase 2 依赖 PLAN-038 编辑库；Phase 3/4 依赖 Phase 1/2）。
**✅ 2026-08-27 解挂启动**：auto-lang 442 C 阶段收口（数据面 parity 达成转 C3
观察期，见 442 §7.4 / PLAN-044），"迁移完成"条件达成，本计划启动执行。

## 目标

1. gen 轨功能对等：Specs 组件组（category 7 + detail 6 + plan 2 + TreeView/
   RelationsPanel/StatusBadge/SpecItemDetail/SpecItemRow/SpecLink）+ 编辑器组
   （GoalEditor/MarkdownEditor/TagInput/TestEditor/AutoDownEditor）全部有 .at 实现，
   Specs/Plans 场景双轨 DOM 对拍无回归。
2. URL 路由能力落地 gen 轨（useViewState 语义：URL 同步/history/popstate）。
3. 生产切换：后端 serve gen 产物、日常脚本启 gen dev server，带回滚开关；观察期
   （默认 7 天）无回滚则收口。
4. vitest 套件迁入 gen 工程（顺手修复 2 个过时品牌断言存量失败）；web/ 标记冻结。

## 架构方案

```
退役前                                退役后
──────────────────────               ──────────────────────
生产: server.rs → web/dist            生产: server.rs → gen/front/vue/dist
日常: start-musk-web.cmd → web :8090  日常: 启动脚本 → gen dev(:3334)
功能: .at + web 双轨实现              功能: .at 单源（codegen 即产物）
缺口: Specs 组件/编辑器仅 web 有      缺口清零（编辑器经 @autodown/editor）
web/: 活跃维护                        web/: 冻结（历史参考 + 对拍基线，只收 bugfix 至观察期结束）
```

- **切换策略二选一**（T11 定夺，默认改路径）：(a) `server.rs` 的 `web_dist` 改指
  `gen/front/vue/dist`（诚实但动后端）；(b) gen build 增发布步骤输出到 `web/dist`
  路径（后端零改动但路径语义撒谎）。默认 (a)，带回滚 env（`MUSK_WEB_DIST` 覆盖）。
- **冻结令协议**：web/ 根 README 加冻结声明 + deps-guard（PLAN-038 T3）白名单注释
  标注 web 域为 frozen；观察期内 web/ 仅收 P0 bugfix，观察期后完全停更。
- **对拍基准反转**：Phase 1/2 的 DOM 对拍以 web/ 现实现为期望快照（最后一次役使
  web/ 作 oracle，快照入库 `scripts/lib-parity/fixtures/`）。

## 技术栈

auto-musk（src/front/*.at 新增 ~21 组件 + specs_view/plans_view 接线；backend
server.rs 一行路径改动；scripts/ 启动脚本与对拍）；../auto-down（@autodown/editor
经 PLAN-038 T14 草稿落地后消费）；auto build（codegen）；vitest（gen 工程）；不动
frontend/（musk-config-remote 独立小应用，与 web/ 无关）。

## 需求分析与背景调查

> 依据 docs/specs/00-overview.md（双轨 parity 架构）与 2026-08-23 实测。

### 现状核实（2026-08-23）

- **生产在 web/**：`server.rs:67-76` 静态服务 `web/dist`；`start-musk-web.cmd` 起
  web dev :8090（proxy /api → :8080）。gen 轨 :3334 为 parity 验证轨。
- **gen 轨组件覆盖**（`gen/front/vue/src/components/` 30 件）：五视图 + 聊天块组 +
  RunBox/Report 等主干全齐（Plan 022/028/031-037 成果）；**缺**：
  - `category/` 7 件（ArchitectureCards/CategoryList/DesignCards/GoalsTable/
    ReportCards/ReviewCards/TestsCards）
  - `detail/` 6 件（Api/Goal/Plan/Report/Review/Test Detail）
  - `plan/` 2 件（PlanMetaBlock/PlanStatusBadge）——注意 Plan 033 已做 Plans UI，
    此 2 件需核对是否已被 .at 版吸收（T1 勘察确认，避免重复移植）
  - 独立 6 件（TreeView/RelationsPanel/StatusBadge/SpecItemDetail/SpecItemRow/
    SpecLink）
  - `editors/` 5 件（GoalEditor/MarkdownEditor/TagInput/TestEditor +
    autodown/core/AutoDownEditor）——PLAN-036 autodown report 的编辑端仍 web 独占
  - URL 路由：web 用 useViewState（7 处引用：URL 同步/history/popstate），gen 用
    裸 ref 切换（Plan 022 已知项）
- **测试在 web/**：`web/src/i18n/__tests__/i18n.spec.ts`（含 2 个过时 AutoForge
  品牌断言存量失败）+ `web/src/utils/__tests__/frontmatter.spec.ts`。
- **双轨税证据**：Plan 031-036 执行步骤均为"T+双轨实现 + codegen + web dist 重建"。

### 与既有计划的关系

- **PLAN-038**（drafting，本日立项）：Phase 3 renderer 真源切 @autodown/vue +
  T14 auto-down 侧计划草稿（含 @autodown/editor 编辑库融合路线）——本计划 Phase 2
  的直接前置。
- **PLAN-039/040**（drafting，另一会话，pi parity）：若涉及前端双轨面，冻结令期间
  新功能一律只落 .at 轨——天然协调。
- **KNOWN-DEBT 022 Phase 5c**（Specs 细化未生成）与 useViewState 项——本计划
  Phase 1 关闭。

## 详细设计

### D1 Specs 组件组迁移（Phase 1）

- 21 件按依赖分层移植：叶子件（StatusBadge/SpecLink/TagInput 等）→ 列表件
  （SpecItemRow/category 卡片组）→ 容器件（SpecItemDetail/detail 组/TreeView/
  RelationsPanel）。每件：web .vue → .at widget，DOM 对拍快照为验收。
- `.at` 落 `src/front/specs_*` 平铺命名（沿用现有 forge_helpers/specs_helpers 惯例），
  `specs_view.at` 接线替换逃生引用。
- Plan 033 的 plan/ 2 件先勘察：若 plans_view.at 已有等价实现则登记吸收、跳过移植。

### D2 URL 路由（Phase 1 末）

- useViewState 语义（URL 同步/history/popstate/快捷键）在 .at 的表达：优先语言层
  routes 能力（PLAN-037 终态提到 widget 多 expose/routes）；不足处经
  `ports/viewstate.web.at` 端口（web adapter 用 history API），VM/Rust 未来提供同名
  adapter——与五域端口机制一致。T8 勘察 auto-lang routes 能力后定夺归属。

### D3 编辑器组迁移（Phase 2，依赖 PLAN-038 编辑库）

- AutoDownEditor → @autodown/editor（Tiptap 内核经其 .at 应用层包装）；Goal/
  Markdown/Tag/Test 编辑器为轻量件，直接手写 .at 移植。
- 阶段门控：若 PLAN-038 编辑库路线延期，备选 = AutoDownEditor 整体先按 ext 逃生舱
  挂 platform/（短期）或 blocker 登记（长期），轻量 4 件照常先行。

### D4 生产切换与回滚（Phase 3）

- `server.rs` web_dist 路径改 `gen/front/vue/dist` + `MUSK_WEB_DIST` env 覆盖
  （回滚开关）；`start-musk-web.cmd` 改启 gen dev（cd gen/front/vue && dev）。
- 切换前 gen 产物完整走一次 `pnpm build` + 冒烟（登录/聊天/Specs/Plans/Wiki 五视图
  + 编辑器开合）；观察期默认 7 天（出 P0 则 env 回滚）。

### D5 测试迁移与冻结（Phase 4）

- 2 个 vitest 套件迁 `gen/front/vue`（package.json 补 vitest devDep；i18n.spec 顺手
  修 'AutoForge'→'Auto Musk' 断言）；web/ README 冻结声明 + KNOWN-DEBT 登记终态。

## 测试设计

1. **DOM 对拍**：21 组件 + 5 编辑器 + 5 视图路由，`scripts/lib-parity/track-switch/`
  下快照对比（web 实现为期望；规范化属性序/自动属性）。
2. **切换冒烟**：Phase 3 后端 serve gen 产物，浏览器五视图 + 报告 + 编辑器实测。
3. **存量不变量**：每 Phase 收口 `auto build` 0 错 + gen `vue-tsc && vite build` 绿
  + `cargo test -p musk` 绿（server.rs 改动后）。
4. **回滚演练**：`MUSK_WEB_DIST` 指回 web/dist 一次，验证回滚路径可用。

## 验收标准

1. gen 轨组件覆盖 = web 轨超集（清单断言：web/src/components 每件在 gen 有对应物
   或登记吸收）；Specs/Plans/编辑器场景对拍零差异（白名单外）。
2. 后端 serve gen 产物、启动脚本启 gen dev；`MUSK_WEB_DIST` 回滚演练通过。
3. vitest 在 gen 工程全绿（含修复的 2 个存量断言）；web/ README 冻结声明 +
   KNOWN-DEBT 登记。
4. 观察期（7 天）无 P0 回滚后计划方可 review。
5. KNOWN-DEBT 022 Phase 5c 与 useViewState 两项标闭。

## 执行步骤

> 前置门：Phase 2 需 PLAN-038 Phase 3 + 编辑库落地；Phase 1 可即刻执行。
> 冻结令自批准日生效（web/ 只收 bugfix）。

### Phase 1 — Specs 组件组 + URL 路由（可先行）

- [x] **T1** 勘察吸收面：`plan/PlanMetaBlock.vue`/`PlanStatusBadge.vue` 对照
  plans_view.at 现有实现，确认被吸收或需移植；产出 21 件最终迁移清单入本节回填。
  验证：清单落档 + grep 无二义引用。
  [✅ 已完成(2026-08-27):最终清单 = 全部 21 件均需移植(gen 零覆盖)——
  category 7 + detail 6 + plan 2(Plan 033 的 plans_view.at 无 PlanMetaBlock/
  PlanStatusBadge 等价物,未吸收)+ 独立 6;SpecItemDetail 随 detail 组。
  编辑器组引用(CategoryList 的 TestEditor/AutoDownEditor 分支)归 Phase 2。]
- [x] **T2** 叶子件移植（StatusBadge/SpecLink/SpecItemRow/CategoryList →
  `src/front/specs_*.at`）+ 对拍快照脚本
  `scripts/lib-parity/track-switch/phase1-leaves.mjs`。验证：对拍 exit 0 +
  `auto build` 0 错。
  [✅ 已完成(2026-08-27):specs_leaf.at 四件(StatusBadge/SpecLink/
  SpecItemRow/CategoryList);对拍脚本落地(vite 双工程子进程 SSR + 归一化
  N1-N6:注释/scoped/事件属性/空白/plain-span text 包裹/size 泄漏/class
  词元排序;web 侧 prop camelCase 映射 + summaryFn 函数标记)——7/7 全等
  exit 0 + auto build(vue-tsc+vite)绿。移植坑入册:computed 不支持对象
  字面量(经 fn 返回)、style 内 transition: all 的 all 关键字、
  transition 需 opacity 形态、icons 端口补 Inbox、事件需 msg 声明。]
- [ ] **T3** category 卡片组 7 件 → .at + specs_view.at 接线 + 对拍。验证：同上。
- [ ] **T4** detail 组 6 件 + SpecItemDetail → .at + 接线 + 对拍。验证：同上。
- [ ] **T5** TreeView + RelationsPanel → .at + 对拍。验证：同上。
- [ ] **T6** 五视图路由对拍（含 popstate/history 行为用例入快照脚本）。
  验证：`node scripts/lib-parity/track-switch/phase1-leaves.mjs` 全量 exit 0。
- [ ] **T7** URL 路由能力勘察：auto-lang widget routes/expose 能力 vs
  `ports/viewstate.web.at` 端口方案，结论回填 D2（语言层 or 端口层）。
  验证：结论含 canary 实测（非纯文档推断）。
- [ ] **T8** URL 路由落地（按 T7 结论）：gen 轨五视图 URL 同步/history/popstate
  达 useViewState 等价。验证：路由对拍 T6 扩展用例绿 + 手动浏览器回退/前进实测。

### Phase 2 — 编辑器组（依赖 PLAN-038 编辑库）

- [ ] **T9** 轻量 4 件移植：GoalEditor/MarkdownEditor/TagInput/TestEditor → .at
  + 对拍。验证：对拍 exit 0 + `auto build` 0 错。
- [ ] **T10** AutoDownEditor 接入：经 @autodown/editor（PLAN-038 T14 落地后）or
  备选路径（D3 门控）落地 gen 轨 + 编辑场景对拍。验证：Specs 编辑器开合/保存
  冒烟 + 对拍绿。

### Phase 3 — 生产切换

- [ ] **T11** 切换实现：`backend/crates/musk/src/server.rs` web_dist 改指
  `gen/front/vue/dist`（保 `MUSK_WEB_DIST` env 覆盖）；`start-musk-web.cmd` 改
  启 gen dev。验证：`cargo test -p musk` 绿 + 后端起服务 serve gen 产物冒烟。
- [ ] **T12** 回滚演练 + 观察期启动：`MUSK_WEB_DIST` 指回 web/dist 验证回滚可用；
  观察期起算登记（本节回填日期）。验证：两个方向各起一次服务冒烟。

### Phase 4 — 测试迁移与冻结

- [ ] **T13** vitest 迁移：2 套件迁 `gen/front/vue`（补 devDep + 修 2 个
  AutoForge 断言）。验证：`cd gen/front/vue && npx vitest run` 全绿。
- [ ] **T14** web/ 冻结：web/README 冻结声明（日期/范围/回滚指针 MUSK_WEB_DIST）+
  KNOWN-DEBT-AND-RISKS.md 登记终态 + 022 Phase 5c/useViewState 两项标闭。
  验证：三处文件 grep 到登记。
- [ ] **T15** 观察期收口（T12 起 7 天，无 P0）：冻结转永久，deps-guard 白名单
  web 域标 frozen。验证：本节回填收口记录 + `/auto-plan:review`。
- [ ] **T16** spec 沉淀准备：整理 spec-impact（touched_goals：双前端 parity →
  单源多后端），转 /auto-plan:review。验证：frontmatter 元数据填齐。

## 复审记录

（待 /auto-plan:review 填写）

## 待澄清事项

1. **切换策略**：server.rs 改路径（默认）vs gen 产物发布到 web/dist——请确认。
2. **观察期长度**：默认 7 天——请确认或调整。
3. **URL 路由归属**：语言层 routes vs ports 端口（T7 canary 定夺，不需预决）。
4. **AutoDownEditor 备选**：若 PLAN-038 编辑库延期，短期 platform/ 逃生舱挂载是否
   可接受（D3 门控）。
5. **frontend/（musk-config-remote）**：独立小应用不受本计划影响——确认无误？
6. **与 039/040 协调**：冻结令期间另一会话的新前端功能只落 .at 轨——是否已知会
   该会话。
