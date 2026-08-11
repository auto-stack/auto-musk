# 023 — view fn → 独立组件 codegen（auto-lang 转译器改造）

> **状态**：📋 计划（未实施）——独立项目（auto-lang 仓库），auto-musk 侧仅登记等待。
> **2026-08-11 能力复核 + 探针实测**：auto-lang 已合并 `component fn`（P1 独立 SFC + P2 跨文件复用 + P3 computed）到 master，auto.exe 已重装。探针 A-F（`tmp/probe-component-fn/`）实测：合成/props/条件渲染/computed/跨文件复用 ✅；但 **P3 试点（逃生舱原生化）当前不可行**——UserMessage/StreamingTable/RawPreview/ChatMessage 四个纯展示候选分别被 408 P4 的缺陷 5（fn import）/6（动态索引）/7（table 映射）阻塞。详见 §3.3。
> **前置**：Plan 022（前端 Auto 化已完成，逃生舱组件架构稳定）；Plan 018-021（后端全 Auto 化方法论）。
> **仓库**：**auto-lang**（a2r 转译器 + a2vue codegen，构建 `auto.exe`）；auto-musk 为验证方。
> **目标**：让 AutoUI 的 `view fn`（`use { fn }` 逃生舱函数中定义、返回 AuraWidget 的函数）被 a2r/a2vue 转译器**原生合成 AuraWidget 组件**，使 `ChatMessage` 等当前靠逃生舱 `.vue` 文件实现的组件脱离逃生舱、纯 `.at` 表达。

---

## 0. 背景与动机

Plan 022 实现中，ChatsView 的消息渲染（ChatMessage）及其子卡片（ErrandCard/RelayCard/TaskPlanCard/GenericToolCard、MentionInput 等）因以下 codegen 缺口走逃生舱：

- **逃生舱现状**：`src/front/components/ChatMessage.vue` 等 20 个 `.vue` 组件 + `forge_helpers.ts`/`relay_commands.ts`/`questionnaire.ts` 等逃生舱 TS。它们经 `use { component/fn }` 声明触发 ext 复制进生成工程，**当前工作正常**（构建通过、行为对齐）。
- **代价**：
  1. 逃生舱 `.vue` 是手写 Vue，不受 `.at` 单一真源约束——`auto build` 零 drift 目标无法覆盖它们；
  2. `view fn`（返回 AuraWidget 的纯函数，如 `renderMentions`、消息分支渲染）目前要么被 `.at` 丢弃成 null（组件 props fn 调用），要么被 codegen 复制成普通 TS 函数（不参与 Vue 响应式合成）；
  3. codegen 有"复制只看 use 块、不递归逃生舱相对 import"的限制，逃生舱内部依赖需逐个在 use 块声明，维护脆弱。

**核心目标**：auto-lang 的转译器（`api.rs` 合成 AuraWidget 的路径）支持 `view fn`——在 `.at` 中声明、返回 AuraWidget 树、可被 widget/组件调用并渲染的函数，直接生成 Vue 组件（或响应式渲染片段），替代逃生舱。

---

## 1. 现状调研要点（2026-08-10 待办）

> 实施前需在 auto-lang 完成下述探针（本项目仅为登记，不实施）。

| 探针 | 对象 | 问题 |
|---|---|---|
| `view fn` 现状 | `trans/rust.rs` / `ui_gen/vue.rs` | a2r 是否支持返回 widget 树的函数？`view` 保留字对函数名的冲突（Plan 5a 发现 `view` 不能做字段名） |
| `api.rs` 合成 AuraWidget 路径 | `crates/ui_gen/api.rs` | 转译器如何把 `.at` 的 widget 树合成 AuraWidget？`fn` 返回 AuraWidget 是否可走同一路径 |
| 组件 props fn 调用 | `expr_to_vue_bound_value`（Plan 022 已修 `Expr::Call`） | 修复后能否覆盖 `view fn` 作为组件 prop |
| `use { fn }` 的返回消费 | `vue.rs` store/widget 生成 | 逃生舱 fn 返回值被 codegen 如何对待（现为普通 import） |

## 2. 目标与验收标准

1. **原生表达**：`ChatMessage`（含消息 role 分支 + 卡片编排 + mention 高亮）用 `.at` 的 `view fn` 表达，不再依赖逃生舱 `.vue`。
2. **零 drift**：移除逃生舱后重新 `auto build`，生成工程 `vue-tsc && vite build` 全绿；逃生舱清理后 diff 产物稳定。
3. **行为一致**：生成工程 ChatView 的交互（流式渲染、卡片展开、mention、命令路由）与原生 `web/` 对齐（沿用 Plan 022 §7.6 parity 口径）。
4. **无回归**：auto-lang `cargo test -p auto-lang` 全绿（含新增 `view fn` a2vue golden）；auto-musk 全量构建通过。
5. **方法论复用**：`view fn` 能力同时惠及其他逃生舱点（useStreamingDocument 增量 JSON、questionnaire 解析、relay_commands 命令路由）。

## 3. 实施阶段（草案，实施时细化）

| 阶段 | 内容 | 验收 |
|---|---|---|
| **P1** | auto-lang 调研：`view fn` 语法设计（与 `use { fn }` 的关系）、`api.rs` 合成 AuraWidget 的最小改动、a2vue golden 基建扩展 | 探针结论 + 设计文档 |
| **P2** | 转译器：`view fn` 解析 + AuraWidget 合成 + Vue 组件生成（或响应式片段）；单测 + golden | `cargo test -p auto-lang` 绿 |
| **P3** | auto-musk 试点：以最简组件（如 UserMessage/mention 高亮）替换逃生舱验证能力 | 生成工程构建 + 行为 parity |
| **P4** | 全量迁移：ChatMessage 及其子组件、forge_helpers/relay_commands/questionnaire 逃生舱 TS | 逃生舱清零（或登记残留）+ 零 drift |
| **P5** | **跨视图共用组件收敛**（2026-08-10 登记）：三个二级导航 + 三个内容标题栏收敛为共用组件（见 §3.1） | 三视图共用组件，样式零漂移 |
| **P6** | 文档归档：KNOWN-DEBT 更新 + 022 §8 后续项闭环 | 归档 |

## 3.1 后续项登记：跨视图共用组件收敛（2026-08-10）

**背景**：Plan 022 的三个视图二级导航（聊天的 `.sidebar-header`、规范的 `.section-nav-header`、知识库的 `.wiki-nav-header`）和三个内容标题栏（`.chats-header`/`.section-header`/`.wiki-content-header`）目前是**各自独立组件 + CSS 统一**——已在 inject_styles.ts 通过统一规则实现视觉对齐（48px 贴顶全宽 border，headless Chromium 实测三视图 top=0/h=48/border 联通），但**结构层未共用**：

- 聊天：`chats_view.at` 生成的 session 列表 + `.sidebar-header`
- 规范：`specs_view.at` 生成的 section 导航 + `.section-nav-header`
- 知识库：`WikiNav.vue`（逃生舱）+ `.wiki-nav-header`

三处样式各自定义，未来某处微调仍可能再次漂移（当前靠 `inject_styles.ts` 的统一规则 + `!important` 压制）。

**目标**：依赖 P2 的 `view fn`/组件能力，抽象一个共用 `NavSidebar`（header + 列表骨架 + 折叠态）和共用 `ContentHeader`（标题 + 操作区插槽），三个视图以 `.at` 声明复用：

```auto
// 伪代码示意
component NavSidebar {
    msg { ToggleCollapse }
    model { var collapsed bool = false }
    view {
        col {
            // header（48px 贴顶全宽 border）
            row { style: "nav-sidebar-header" ... }
            // 列表骨架由各视图注入
            slot: "list"
        }
    }
}
```

**验收**：
1. 三视图二级导航/内容标题栏改用一个 `.at` 组件（或一个逃生舱 + 三处复用），删除 inject_styles 中针对三个 header 的分散 `!important` 覆盖；
2. 重新 `auto build` 后三视图视觉与现 CSS 对齐版一致（headless Chromium 逐项比对 top/height/border 联通）；
3. 样式单一真源：改一处 header 规则三视图同步生效，无 `!important` 兜底。

**依赖**：需 **auto-lang Plan 408**（`view fn → 独立 Vue 组件合成`，2026-08-10 立项）支持组件插槽/子组件差异（或先以逃生舱 + props 差异实现，再渐进原生化）。**不阻塞** Plan 022 当前功能。

> **2026-08-10 调研更新**：auto-lang 侧已确认——view fn 的**内联展开**已有（374 修复已移植 Vue 路径，`api.rs:406-411` 注册 + vue.rs 测试），但**独立 SFC 合成缺失**（vue.rs 无此路径）。已立项 **Plan 408**（auto-lang `docs/plans/408-view-fn-vue-component-synthesis.md`）专门补此缺口。本计划（含 §3.1 共用组件收敛）依赖 408 完成后推进。

> **2026-08-11 能力复核**（auto-lang `plan-408` 分支实测）：408 的 `component fn`（与 `view fn` 二分的新关键字）已落地 **P1 同文件独立 SFC 合成** + **P2 跨文件 `use { component: X }`（无 from）复用**——有 golden（`test/a2vue/007_component_fn/`）+ e2e（`plan408_tests.rs`）覆盖。能力边界：✅ props（params→defineProps）/ ✅ 条件渲染（`if` 分支，支撑方案 B 差异）；❌ **computed**（硬编码 `Vec::new()`）/ ❌ **msg/emit** / ❌ **slot**。
>
> **§3.1（共用组件收敛）现状判定**：408 文档 §6.3 末句自判"§3.1 需 emit + slot，依赖 Task 2，排期更靠后"。本轮按此不强行推进 §3.1——降级到 props+条件渲染会劣化 WikiNav 逃生舱（含搜索/折叠/DropZone emit 交互），不值得。
>
> **工具链状态**：`plan-408` 分支未合并 auto-lang master，`~/.cargo/bin/auto.exe`（master 构建，md5 = `target/release/auto.exe`）**不含** `component fn`。auto-musk 侧要用该能力，需先把 plan-408 合并 master + `cargo install` 重装（跨仓库操作，待授权）。
>
> **本轮（023 auto-musk 侧）范围**：仅文档收敛（本段落 + 状态行）。实质代码迁移（P3 试点 / §3.1 收敛）待 ① 工具链含 `component fn`、② 408 Task 2（emit/slot）就绪后推进。

### 3.2 component fn 能力探针实测（2026-08-11）

> 前置已就绪：auto-lang 已合并 plan-408（P1+P2+P3，master `c12b407e`），`cargo install` 重装 `auto.exe`（含 `component fn`）。探针工程：`tmp/probe-component-fn/`（隔离，不入主源码）。

**探针矩阵**（4 个场景，逐个 `auto build` + 检查生成 SFC 的 TS 正确性）：

| # | 场景 | 结论 | 证据 |
|---|---|---|---|
| **A** | 基础合成：`component fn Card(title,active)` + widget `<Card/>` + `if` 条件渲染 | ✅ 合成机制 OK；⚠️ 字面量 prop 绑定有缺陷 | `Card.vue` 正确（defineProps + if 分支 style）；App.vue 调用点 `:active="{{ true }}"`（双花括号语法错）、`:title="second"`（字符串字面量未引号被当变量） |
| **B** | callback/event 透传：`component fn NavItem(label,onselect:msg)`，内部 `button { onclick: onselect }` | ⚠️ 父→子 event 透传 OK；❌ 子内部调用 prop 作 handler 不工作 | 父 App.vue：`onselect: .Clicked` → `@select="Clicked"` ✅；但子 NavItem.vue 把 `onselect` 当本地未定义 handler，生成空函数 `ononselect(){// TODO: handler not defined}`，prop 未被当可调用引用 |
| **C** | 跨文件复用：`lib.at` 定义 `component fn SharedCard`，`app.at` `use { component: SharedCard }`（无 from）引用 | ✅✅✅ 完全可用，零 TS 错误 | `SharedCard.vue` 正确合成；App.vue 正确 `import` + `<SharedCard :title="heading"/>`；构建全绿 |
| **D** | computed 块：`component fn Badge(count)` 内 `computed { label => ...; doubled => ... }` | ✅ 基本可用 | Badge.vue 正确 `import { computed }` + `const label = computed(...)`；template 正确 `{{ label }}`；⚠️ `if` 表达式被包多余 IIFE |

**关键结论**：

1. **✅ component fn 核心机制（合成 + props + 条件渲染 + computed + 跨文件复用）可用**——P3 试点（最简逃生舱组件原生化）的路径打通，可挑无交互的纯展示组件先行。

2. **❌ §3.1 共用组件收敛的核心障碍仍在**：共用 `NavSidebar` 需要"内部按钮点击 → 触发外部传入的回调"（子组件内部 `onclick: <prop>`），探针 B 证实此模式**不工作**（prop 未被当 handler，生成空函数）。这等价于缺 emit——§3.1 阻塞于此，与 408 文档 §6.3 自判一致。

3. **⚠️ 3 个 codegen 缺陷**（非阻塞 P3，但需登记/修复）：
   - **字面量 prop 绑定**：bool → `:active="{{ true }}"`（双花括号）；str → `:title="second"`（未引号当变量）。影响所有字面量 prop 透传。
   - **`self.` 前缀错绑**：变量 prop 在某些场景生成 `:title=" self .heading"`，引用不存在的 `self`（跨文件复用场景反而不中招，生成干净的 `:title="heading"`）。
   - **computed `if` 表达式 IIFE 包装**：`computed(() => (() => {...})())`，能跑但多余。

**P3 试点可行性判定**：✅ 可行。挑一个**纯展示、无 click 回调、无 emit**的逃生舱组件（不含 AgentAvatar 的 computed 颜色逻辑——虽然 computed 已支持，但字典+char hash fallback 超当前能力）。候选评估见 §3.3。

**§3.1（P5 共用组件收敛）判定**：❌ 继续阻塞，依赖 auto-lang 补"子组件内部 button onclick 调用 prop 作 handler"（本质是 emit/事件透传，408 Task 2 范畴）。

### 3.3 P3 试点候选评估（2026-08-11）

> 基于 §3.2 探针结论（核心机制可用 + 字面量 prop/self 前缀缺陷 + 子内部回调不工作），扫描 21 个逃生舱 `.vue` 组件，按"纯展示（click/input=0, emit=0）"筛选 P3 首选。

**纯展示候选**（4 个，按推荐度）：

| 组件 | 行数 | computed | 可原生化的关键依赖 | 评估 |
|---|---|---|---|---|
| **ChatMessage.vue** | 50 | 0 | StreamingRenderer + UserMessage（链式逃生舱） | 🥇 最简（computed=0），但链式依赖未原生化——需先原生化 UserMessage/StreamingRenderer，或先做叶子组件 |
| **UserMessage.vue** | 35 | 1 | `v-html`（codegen 已支持 `html:` prop）+ `renderMentions`（逃生舱 TS） | 🥈 叶子组件，v-html 已支持；残留 renderMentions TS（HTML 转义+@高亮）需逃生舱 fn 或内联 |
| **StreamingRenderer.vue** | ? | 3 | markstream-vue 渲染 + 流式增量 | 🥉 依赖 npm 包，流式逻辑复杂 |
| **AgentAvatar.vue** | ? | 7 | professionColors 字典 + char hash + 5 computed | ❌ 408 §6.3 已判定超能力（字典/动态 style 对象） |

**P3 试点实测结论（2026-08-11，探针 E2/F 验证）**：**当前不可行**——四个纯展示候选都被 auto-lang 408 P4 的 codegen 缺陷阻塞，无一首试可用：

| 候选 | 阻塞缺陷 | 探针 |
|---|---|---|
| **UserMessage** | **缺陷 5**：component fn 不支持 `use { fn }`——computed 调 renderMentions 生成悬空标识符（TS2304） | E2 |
| **StreamingTable** | **缺陷 6**：动态索引 `.row[.col]` 生成错位（`<span>{{row}}</span><div>{{col}}</div>`）；**缺陷 7**：原生 `table` 标签被映射成 shadcn Table | F |
| **RawPreview** | 缺陷 5（fn import：rawFileUrl/loadRawFileText）+ onMounted/watch 生命周期 + 正则（超 component fn 范畴） | — |
| **ChatMessage** | 链式依赖（UserMessage + StreamingRenderer 均未原生化，无法单独迁移） | — |

**这些阻塞缺陷已登记到 auto-lang 408 §7 P4**（缺陷 5/6/7），并修订了优先级（§7.6）：缺陷 5（fn import）升为 🔴 高，是 P3 的第一阻塞。

**P3 推进路径（待 auto-lang 408 P4 落地）**：
1. **缺陷 5（fn import）+ 缺陷 1+2（prop 绑定）修复** → UserMessage 可原生化（P3 首试解封）
2. **缺陷 6+7 修复** → StreamingTable 可原生化
3. 叶子就绪后 → ChatMessage 编排组件原生化（消掉链式逃生舱）

**不迁移的组件**（留逃生舱，本轮登记）：
- 含 emit/重交互的（GateCard/MentionInput/QuestionnaireCard/SecretaryMessage/WikiNav 等）——阻塞于 §3.1 同一 emit 缺口（408 P4 缺陷 4）。
- AgentAvatar——阻塞于 computed 字典/动态 style（超当前能力，需 auto-lang 扩展）。

## 4. 风险与降级

| 风险 | 级别 | 降级 |
|---|---|---|
| `view fn` 语法与现有 `use { fn }` 冲突 | 🟡 | 新关键字或后缀，向后兼容 |
| AuraWidget 合成路径复杂度超预期 | 🔴 | 分阶段：先支持"纯渲染 fn"（无状态），再支持"带 store 依赖 fn" |
| 流式/增量 JSON（useStreamingDocument）难以纯原生 | 🟡 | 保留逃生舱 + KNOWN-DEBT（Plan 022 已登记） |
| 迁移引发回归 | 🟡 | 每阶段 golden + 全量构建；P3 先单点验证 |

## 5. 与 auto-musk 的关系

- **不阻塞**：Plan 022 已用逃生舱达成功能 parity，本计划是"逃生舱渐进原生化"的大工程。
- **执行方**：auto-lang 仓库（独立 repo，`D:\autostack\auto-lang`）；auto-musk 仅作为验证与迁移对象。
- **迁移对象**（按优先级）：
  1. ChatMessage 及其子卡片、逃生舱 TS（§3 P3/P4）——功能/交互 parity 的纯原生化；
  2. **三视图二级导航 + 内容标题栏共用组件收敛**（§3.1 P5）——消除三处 `!important` 兜底的 CSS 漂移风险，样式单一真源。
- **状态追踪**：本文件驻留 auto-musk `docs/plans/`，实施在 auto-lang 侧开分支，完成后再回填本文件日志。
