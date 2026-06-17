# 001 — auto-musk 前端 Widget 设计参考

> **目的**：指导 auto-musk 前端用 Auto widget（a2vue）落地时，**从源头避免 auto-forge 前端的重复模式**。
> **依据**：auto-forge 前端技术债评估（重复模式 R1-R5，详见文末附录）。auto-forge 是各模块 AI 独立实现的产物，存在 ~1100 行可消除重复；auto-musk 移植时复用其"数据契约 + UX 范式"，但**不搬 Vue 代码**，而是设计统一的 widget。
> **原则**：auto-forge 现存的重复恰好是一份免费的领域分析——它标出了"同一类东西的多个变体"和"同一 CRUD 的多个实例"，我们把这些模式直接固化成少量通用 widget。

---

## 1. 设计总则

| 原则 | 说明 |
|---|---|
| **一类实体 = 一个通用 widget** | 凡是 auto-forge 里出现 N 个近亲的（7 类 Spec 卡片、4 个 CRUD 配置实体），auto-musk 只设计 1 个参数化 widget，用 `section`/`entity` 等 prop 区分 |
| **复用契约，不复用源码** | 数据模型/状态机/UX 范式参照 auto-forge，但实现用 Auto widget DSL 重写 |
| **DSL 能表达用 DSL，不能则手写** | 布局/卡片/列表/表单用 widget；SSE 流式/Tiptap/图表手写 Vue 混用（见 plans/002 §2） |
| **避坑 a2vue 缺陷** | 见 plans/002 §2.2（outlet 必需、input 无 v-model、无 DSL 路由跳转等） |

---

## 2. 核心 Widget 设计（按优先级）

### 2.1 `SpecSectionWidget` —— 消灭 R4（7 类 Spec 卡片重复）

**背景**：auto-forge 有 6 个近亲卡片组件（Architecture/Design/Plan/Report/Review/Tests Cards，各 ~60 行）+ GoalsTable，都做"列表卡片 + 状态徽章 + 点击进详情 + 展开/编辑"，唯一差异是 section-type 和 summaryFn。

**设计**：1 个通用 widget，prop 化差异点。

```
widget SpecSectionWidget (section: str) {
    model {
        items List = []
        expanded_id str = ""
        editing_id str = ""
    }
    view {
        col {
            // 标题栏 + 新增按钮
            row { text .section_title { } button "+" { onclick: .CreateNew } }
            // 卡片列表（统一布局，status badge 统一渲染）
            for item in .items {
                card (onclick: .Select(item.id)) {
                    card-header {
                        text item.title { }
                        StatusBadge (status: item.status) { }   // 复用 §2.5
                    }
                    card-content { text .summary_of(item) { } }
                }
            }
            if .items.len() == 0 { EmptyState (text: "No items yet") { } }
        }
    }
    on {
        .Select(id) -> { .expanded_id = id }   // 触发详情面板
        .CreateNew -> { .editing_id = "new" }
    }
}
```

**消灭**：auto-forge 的 6 个 `*Cards.vue`（~360 行）→ 1 个 widget + 7 行调用（按 section-type 实例化）。

**注意**：summary 函数（每类 spec 提取不同摘要）放在 widget 的 computed 或外部 helper，按 section 分派。

---

### 2.2 `CrudEntityWidget` —— 消灭 R1 + R2（CRUD composable + 配置视图重复，~750 行，最大债）

**背景**：auto-forge 有 4 个近亲 composable（useProfessions/Skills/AgentConfigs/ApiSources，各写一遍 load/create/update/delete/reset 样板 ~400 行）+ 3 个克隆配置视图（Skills/Professions/AgentsConfig，各自 editing/handleSave/edit-overlay 壳 ~360 行）。

**设计**：1 个通用 CRUD widget，封装"列表 + 编辑面板 + 增删改"全套；数据层用统一的后端契约。

```
widget CrudEntityWidget (entity: str, fields: List) {
    model {
        items List = []
        editing bool = false
        editing_id str = ""
        is_new bool = false
        draft Map = {}     // 当前编辑的实体副本
        loading bool = false
        error str = ""
    }
    view {
        col {
            // 工具栏
            row { text .entity_title button "New" { onclick: .StartCreate } }
            if .loading { text "Loading..." }
            // 列表网格
            row { for item in .items { card (onclick: .StartEdit(item.id)) { ... } } }
            // 编辑面板（overlay，按 fields 动态渲染表单）
            if .editing {
                EditOverlay (onClose: .CancelEdit) {
                    for f in .fields { FieldEditor (field: f, value: .draft[f.key]) { } }
                    row { button "Save" { onclick: .Save } button "Cancel" { onclick: .CancelEdit } }
                }
            }
        }
    }
    on {
        .Init -> { .items = list_entities(.entity) }     // use back.api: list_entities
        .StartCreate -> { .editing = true; .is_new = true; .draft = {} }
        .StartEdit(id) -> { .editing = true; .is_new = false; .draft = get_entity(.entity, id) }
        .Save -> {
            if .is_new { create_entity(.entity, .draft) } else { update_entity(.entity, .editing_id, .draft) }
            .editing = false; .Init
        }
        .Delete(id) -> { delete_entity(.entity, id); .Init }
    }
}
```

**消灭**：auto-forge 的 4 个 CRUD composable + 3 个配置视图（~750 行）→ 1 个 widget + 各配置实体的 field 定义（~20 行/实体）。

**后端契约要求**：所有配置实体走统一的 RESTful 约定（`GET /api/config/{entity}` / `POST` / `PUT /:id` / `DELETE /:id`），这样前端 1 个 `list_entities(entity)` 通用客户端即可。auto-forge 后端已有 `/api/forge/config/{api-sources,professions,skills,agents}`，移植时统一成这个约定。

**⚠️ a2vue 限制**：表单的 input 当前生成 `:value` 非 `v-model`（plans/002 §2.2 缺陷 2），编辑面板的表单输入需手写 Vue 或等 a2vue 修复。这是 CrudEntityWidget 落地的主要阻塞点。

---

### 2.3 `MasterDetailLayout` —— 消灭 R3（主从侧栏重复，~100 行）

**背景**：auto-forge 的 WikiView + ApiSourcesView 各自重写了"可折叠侧栏 + 详情区"主从布局，各自维护 `sidebarCollapsed` ref + localStorage 持久化。

**设计**：1 个布局 widget，slot 化两侧。

```
widget MasterDetailLayout (sidebar_title: str) {
    model {
        collapsed bool = false
    }
    view {
        row {
            if !.collapsed {
                col (style: "w-64 border-r") {
                    row { text .sidebar_title button "<" { onclick: .Collapse } }
                    slot sidebar
                }
            }
            col (style: "flex-1") {
                if .collapsed { button ">" { onclick: .Expand } }
                slot detail
            }
        }
    }
    on {
        .Init -> { .collapsed = load_local("sidebar_collapsed") }
        .Collapse -> { .collapsed = true; save_local("sidebar_collapsed", true) }
        .Expand -> { .collapsed = false; save_local("sidebar_collapsed", false) }
    }
}
```

**消灭**：WikiView/ApiSourcesView/SpecsView 的侧栏布局重复。各视图只填 `sidebar`/`detail` 两个 slot。

**待确认**：a2vue 是否支持 `slot` 语义（widget 的具名插槽）。若不支持，退化为 props 传入两个子 widget。

---

### 2.4 `EmptyState` / `LoadingState` / `ErrorState` —— 消灭 R5（状态态 UI 重复）

**背景**：auto-forge 各视图各自实现空态/加载态文案（无统一组件，~60 行散落）。

**设计**：3 个极简通用 widget。

```
widget EmptyState (icon: str, text: str) {
    view { col (style: "py-10 items-center text-muted-foreground") {
        icon (name: .icon) {}; text .text {}
    } }
}
widget LoadingState (text: str) {
    view { col (style: "py-10 items-center") { icon (name: "loader-2") {}; text .text {} } }
}
widget ErrorState (text: str) {
    view { col (style: "py-10 items-center text-red-500") { icon (name: "alert-circle") {}; text .text {} } }
}
```

Step 4 的 Explorer 空态已用 card 手写，应替换为 `EmptyState`。

---

### 2.5 `StatusBadge` —— 沿用（auto-forge 已有健康抽象）

**背景**：auto-forge 的 `StatusBadge.vue`（6 处复用）+ `StatusTransition.vue`（含 `SECTION_STATUSES`/`TRANSITIONS`）已是健康抽象。

**设计**：直接移植语义。auto-forge 的 22 种 Status + 状态机（`StatusTransition.vue:42-79`）是后端 `SectionConfig::for_type` 的镜像——**移植时以后端契约为准**（更可靠），前端 widget 按 status 字符串映射颜色/文案。

```
widget StatusBadge (status: str) {
    view {
        badge (variant: .variant_of(.status)) { text .label_of(.status) }
    }
}
```

`variant_of`/`label_of` 按 22 种 status 的映射表（draft=灰/review=黄/approved=绿/...）。

---

## 3. 移植阶段映射（widget → auto-forge 对应模块）

| auto-musk widget | 消灭的重复 | 对应 auto-forge 文件 | 移植阶段（总计划）|
|---|---|---|---|
| SpecSectionWidget | R4 | category/*Cards.vue × 7 | 阶段 6（Spec Ledger）|
| CrudEntityWidget | R1+R2 | composables/use{Prof,Skills,Agents,ApiSources} + 3 配置视图 | 阶段 8（配置体系）|
| MasterDetailLayout | R3 | WikiView/ApiSourcesView 布局 | 阶段 6/8 |
| EmptyState/Loading/Error | R5 | 各视图状态态 | 贯穿 |
| StatusBadge | （沿用）| StatusBadge/StatusTransition | 阶段 6 |

> **关键**：这些通用 widget 在对应阶段（6/8）才需要，不是现在做。本设计参考是"提前规划，避免届时各视图又各写各的"。

---

## 4. 不做的事（YAGNI）

- **不抽 GoalsTable 的树形视图**：它是唯一的树形特例（父子 goal），抽通用化收益低，单独保留。
- **不为 7 类 Spec 的详情组件抽通用**：auto-forge 的 `*Detail.vue` 各自解析不同 markdown 结构（PlanDetail 解析阶段/ReportDetail 解析 metrics），是合理的领域差异，非技术债。
- **不在 auto-forge 回头重构**：auto-forge 在迭代中，回归风险不值得（见评估结论）。所有统一在 auto-musk 落地。

---

## 附录：auto-forge 前端重复模式（R1-R5，本设计依据）

| # | 重复模式 | 量级 | auto-musk 对应 widget |
|---|---|---|---|
| R1 | 4 个 CRUD composable 样板（load/create/update/delete/reset）| ~400 行 | CrudEntityWidget |
| R2 | 3 个配置视图的编辑态壳克隆（editing/handleSave/edit-overlay）| ~360 行 | CrudEntityWidget |
| R3 | 主从侧栏布局 + 折叠持久化（WikiView/ApiSourcesView）| ~100 行 | MasterDetailLayout |
| R4 | 6 个 Spec 卡片组件模板壳 | ~250 行 | SpecSectionWidget |
| R5 | 空态/加载态 UI 散落 | ~60 行 | EmptyState/Loading/Error |
| ~~R6~~ | 6 处裸 fetch 漏 auth（bug）| — | 不适用（已在 auto-forge plan 023 修） |

（R1-R5 评估证据见 auto-forge 前端技术债分析会话记录；R6 已写入 `auto-forge/docs/plans/023-fix-bare-fetch-auth-bugs.md`。）
