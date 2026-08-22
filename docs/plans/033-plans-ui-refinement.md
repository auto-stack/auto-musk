---
plan_id: PLAN-033
status: executing
feature_name: 计划模块 UI/UX 改进（过滤/状态/徽标/归档语义/MetaBlock）
author: [zhaopuming]
created_at: 2026-08-22T10:40:25+08:00
updated_at: 2026-08-22T12:05:00+08:00

supersedes_spec_components: []
new_spec_components: []
touched_goals: []

current_step: 9
total_steps: 12
---

# [PLAN-033] 计划模块 UI/UX 改进：过滤开关、状态重命名、徽标 i18n、归档语义统一、MetaBlock

## 变更摘要

针对 Auto/vue（`web/`，musk-web）"计划"页的五项 UI/UX 与状态模型改进：

1. 左侧列表过滤从"包含归档"checkbox 改为 **Active / All 下拉**，默认 Active，选择持久化。
2. 状态枚举重命名：`merged` → `archived`、`review_done` → `reviewed`（全栈 + 旧文件兼容映射）。
3. 状态徽标接入 i18n（中文模式显示中文）；删除头部冗余的全量状态下拉，状态展示以徽标为唯一来源，状态转移改为"仅列出合法目标"的按钮组。
4. 统一"归档 / 沉淀到 Spec"语义为**单一终态模型**：终态只有 `archived`；reviewed 的唯一出口是"沉淀到 Spec"（沉淀即归档），非 reviewed 计划走"归档"（搁置不沉淀）；两按钮互斥展示，后端同步门禁。
5. 计划正文不再渲染 frontmatter 黑体块；新增 **PlanMetaBlock** 组件：折叠时概要一行，展开后表格显示全部 meta 属性。

## 目标

- 消除状态信息的三处冗余/混排：英文徽标 vs 中文下拉 vs archived 灰标签。
- 让按钮与状态机语义一致：任一时刻"终态动作"按钮至多一个，且名称与结果状态对应。
- 旧数据零迁移成本：磁盘上 `status: merged` / `review_done` 的历史文件读取时自动映射，写入时自愈为新枚举。
- frontmatter 作为元数据被结构化呈现，正文渲染区只渲染正文。

## 架构方案

不改 API 形状（路由、请求/响应 DTO 均不变），只改语义：

- **后端** `backend/crates/musk/src/plans.rs`：`PlanStatus` 枚举变体重命名 + `from_str_lossy` 兼容旧值；`can_transition` 状态机去掉 `reviewed → archived` 手动转移；`archive()` 升级为"置状态 + 移档"并拒绝 reviewed 计划；`merge_plan_stores()` 沉淀后经同一助手进入终态。
- **前端** `web/src/types/plans.ts` 保持为前端单一事实源（枚举、转移表、色调、i18n key 映射），`PlansView.vue` 只做视图编排；新增 `web/src/utils/frontmatter.ts`（纯函数解析）与 `web/src/components/plan/PlanMetaBlock.vue`（展示组件）。
- 跨栈一致性继续靠 `backend/crates/musk/tests/parity_plans.rs` 与约定（无共享常量文件，本计划不引入）。

## 技术栈

Rust（axum hand-written routes）、Vue 3 `<script setup>` + TS、vue-i18n 9（legacy:false）、原生 `<select>`/按钮（无新增 UI 依赖）、Vitest + vue-tsc。**不引入 yaml 依赖**——frontmatter 为扁平键值 + 简单数组，自写 ~40 行解析器并配单测。

## 需求分析与背景调查

 Specs 台账（`docs/specs/`：`00-overview.md`、`01-architecture.md`、`03-front-component-groups.md`、`goals/`、`modules/`、`reviews/`、`index.json`）中与本计划相关的是 `03-front-component-groups.md` 所辖的 web 组件分组——新增 `PlanMetaBlock` 组件、改 `PlansView`/`PlanStatusBadge` 属于该分组的扩展，merge 时需回填。

现状事实（已逐一核对源码）：

- **页面**：`web/src/views/PlansView.vue`（519 行）承载全部 UI。左栏"包含归档"checkbox 在 L10-13（`includeArchived` ref → `loadPlans(includeArchived)`）；头部徽标区 L36-42（plan-id + `PlanStatusBadge` + `archived` 灰标签 + feature_name）；操作区 L43-74 是一个列出**全部 5 个状态**的 `<select>`（禁用项灰显，L45-57）+ 编辑/归档/沉淀按钮。
- **徽标**：`web/src/components/plan/PlanStatusBadge.vue` 标签硬编码英文（且把 `execution_done` 缩成 `exec_done`），不走 i18n——这是"徽标英文、下拉中文"混排的根源。
- **前端类型**：`web/src/types/plans.ts`——`PlanStatus` 联合类型（L3-8）、`canTransition`（L60-70，镜像后端）、`STATUS_TONE`（L73-79）。
- **后端**：`backend/crates/musk/src/plans.rs`——`PlanStatus` 枚举 L26-60（`as_str` snake_case、`from_str_lossy` 未知值静默落 Drafting）、`can_transition` L71-85、HTTP handlers L602-705。
- **归档 vs 沉淀的实际行为（需求 4 的分歧根源）**：
  - `archive()`（L496-512）**只把文件移入 `archived/`，不改状态** → 归档计划顶着过时状态（如 `execution_done`）+ `archived` 布尔标记双轨并存。
  - `merge_plan_stores()`（L531-550）是三合一：门禁 `review_done` → 拆解沉淀进 specs doc → `transition(Merged)` → `archive()`。
  - 即：UI 上两个按钮，但"归档"这个动作在两条路径里语义不同（纯搁置 vs 沉淀的收尾步骤），且状态维度只有 `merged` 一个终态词，`archived` 却是另一个独立布尔——模型与按钮不对应。
- **frontmatter 渲染**：`current.content` 含原始 YAML frontmatter，直接喂给 `markstream-vue`；`key: value` 行 + 结尾 `---` 被解析成 setext H2 标题——这就是"黑体大字号属性块"的来历。仓库无 yaml 解析依赖（`web/package.json` 仅有 marked/markstream-vue/mermaid/lucide/vue-i18n）。
- **i18n**：`web/src/i18n/locales/{zh,en}.json` 已有 `plans.*` 状态标签键（`statusReviewDone`=复审通过、`statusMerged`=已沉淀）；`PlansView.vue` 内仍残留硬编码英文 `alert`/`confirm`（L203、L214）；键集 parity 由 `web/src/i18n/__tests__/i18n.spec.ts` 约束。
- **技能文档**：`.agents/skills/auto-plan-{new,review,merge,work}` 的 SKILL.md 与状态模板仍写 `review_done`/`merged`；用户级 `~/.zcode/skills/{finish-plan,archive-plan}` 也引用旧名（仓库外文件）。
- **历史数据**：`docs/plans/archived/*.md` 的 frontmatter 里存量 `merged`/`review_done`，读取需兼容。
- `gen/front/vue/` 是旧生成版前端，不在本计划范围。

## 详细设计

### D1 状态模型（需求 2 + 4 合并设计）

新枚举（wire 格式）：`drafting | executing | execution_done | reviewed | archived`

```
drafting ──→ executing ──→ execution_done ──→ reviewed ──→ archived
    └──────────────────────────↑__________________│(回退: →executing)
                                                终态唯一入口:
                                                reviewed 只能经"沉淀到 Spec"(merge) 进入 archived
```

`can_transition` 表（前后端镜像）：

| from | allowed |
|---|---|
| drafting | executing, reviewed |
| executing | execution_done, drafting |
| execution_done | reviewed, executing |
| reviewed | executing（回退；**不再允许直接 → archived**） |
| archived | （终态） |

- **`archive()` 新语义**：置 `status: archived`（重写 frontmatter）+ 移入 `archived/`。提取私有助手 `move_to_archived(seq)` = transition(Archived) + rename，供 `archive()` 与 `merge_plan_stores()` 共用。
- **门禁**：`archive()` 遇 `status == reviewed` 返回错误 `"plan {:03} is reviewed; merge it to spec instead of archiving"`——reviewed 的唯一出口是沉淀。
- **`merge_plan_stores()`**：门禁文案改用 `reviewed`；沉淀后 `transition(Archived)` + `move_to_archived`。
- **兼容映射**（`from_str_lossy`）：`"merged" → Archived`、`"review_done" → Reviewed`，未知仍落 Drafting。旧文件读取即映射；任何后续 update/transition 重写 frontmatter 时自愈为新值。**不做一次性磁盘迁移**。
- **需求 4 结论**：采用"单一终态"方案（用户的方案 B 变体）：`archived` 既是状态也是文件位置，二者恒一致；"归档"按钮服务**非 reviewed** 计划的搁置路径（放弃/冻结，不沉淀），"沉淀到 Spec"服务 reviewed 计划的正路（沉淀即归档）。两按钮按状态互斥展示，动作与终态一一对应，分歧消除。

### D2 列表过滤 ComboBox（需求 1）

`PlansView.vue` 左栏：checkbox（L10-13）替换为

```html
<select v-model="filterMode" class="nav-filter" @change="refresh">
  <option value="active">{{ t('plans.filterActive') }}</option>
  <option value="all">{{ t('plans.filterAll') }}</option>
</select>
```

- `filterMode = ref<'active' | 'all'>('active')`，`refresh()` → `loadPlans(filterMode.value === 'all')`（后端 `include_archived` 参数不变）。
- 持久化到 `localStorage['autoforge-plans-filter']`（与语言键同前缀风格），初始化时读取并校验值。
- 删除 i18n 键 `plans.includeArchived`，新增 `filterActive`（进行中 / Active）、`filterAll`（全部 / All）。

### D3 徽标 i18n + 头部去冗余（需求 3）

- `types/plans.ts` 新增导出 `planStatusKey(s: PlanStatus): string`（`execution_done` → `plans.statusExecutionDone`；从 `PlansView.vue` 的 `statusKey` L149-154 迁移）。i18n 键改名：`statusReviewDone` → `statusReviewed`（已复审 / Reviewed）、`statusMerged` → `statusArchived`（已归档 / Archived），其余沿用。
- `PlanStatusBadge.vue`：`label = t(planStatusKey(props.status))`，删除 `exec_done` 缩写 hack；`title` 保留原始枚举值（悬停可看 wire 值）。
- **删除**头部全量状态 `<select>`（L45-57）与 `statusOptions`；`archived` 灰标签（L40）一并删除（徽标已表达"已归档"）。
- 状态转移控件改为**合法目标按钮组**：`v-for s in ALLOWED_TRANSITIONS[current.status]`（新导出的转移表），按钮文案 `t('plans.transitionTo', { status: t(planStatusKey(s)) })`（如"标记为 执行中"）；正向边默认样式、回退边（目标为 drafting/executing 且当前更晚）muted 样式；`archived` 终态不渲染。至多 2 个按钮。
- `onTransition` 改为接收目标状态的普通函数；删除 `Illegal transition` alert 分支（按钮只含合法目标，不可达）；`confirm(\`Archive ${id}?\`)` 改为 `t('plans.archiveConfirm', { id })`。
- **归档/沉淀互斥（需求 4 UI 侧）**：`status === 'reviewed' && !archived` → 只显示"沉淀到 Spec"（primary）；`status !== 'reviewed' && !archived` → 只显示"归档"；`archived` → 两者皆不显示（仅编辑）。

### D4 PlanMetaBlock（需求 5）

- **`web/src/utils/frontmatter.ts`**：`splitFrontmatter(content: string): { meta: Record<string, string | string[]>; body: string } | null`。匹配 `^---\r?\n([\s\S]*?)\r?\n---\r?\n?`；解析扁平 `key: value`（去引号）、行内数组 `[a, b]`、`- item` 块列表；键序保持文件顺序；无 frontmatter 返回 null。
- **`web/src/components/plan/PlanMetaBlock.vue`**：props `meta`。折叠（默认）：一行概要——`feature_name · {created_at} ~ {updated_at} · {current_step}/{total_steps}`（字段缺失则跳过），右侧 chevron 切换；展开：两列表格，键列 monospace（去 snake_case 转空格显示可选，保持原键名亦可），值列字符串原样、数组以 `、` 连接。样式对齐现有 `.plan-item` 密度（0.78rem 级）。
- **`PlansView.vue`**：`const parsed = computed(() => splitFrontmatter(current.value?.content ?? ''))`；`<MarkdownContent :content="parsed?.body ?? current.content" />`；渲染区顶部插 `<PlanMetaBlock v-if="parsed" :meta="parsed.meta" />`。编辑模式不变（textarea 仍编辑含 frontmatter 的全文）。
- 新增 i18n 键：`metaShow`（展开元数据 / Show metadata）、`metaHide`（收起元数据 / Hide metadata）。

### D5 技能与文档同步

`.agents/skills/auto-plan-new/SKILL.md`（frontmatter 模板状态注释行）、`auto-plan-review`（review_done）、`auto-plan-merge`（merged）、`auto-plan-work`（状态推进描述）中的旧状态名改为 `reviewed`/`archived`；`docs/designs/008-auto-plan.md` 中状态机描述同步。**不改** `docs/plans/archived/` 历史文件正文。

## 测试设计

- **后端单测**（`plans.rs` 文内 tests + `tests/parity_plans.rs`）：
  - `from_str_lossy`：`merged→Archived`、`review_done→Reviewed`、未知→Drafting。
  - `can_transition`：`reviewed→archived` 为 false；`archived` 无出边。
  - `archive()`：execution_done 计划归档后 `status==archived && archived==true`；reviewed 计划归档返回 Err 且文件未移动。
  - `merge_plan_stores`：reviewed 计划沉淀后 status==archived 且在 archived/ 目录（沿用现有测试改断言）。
- **前端单测**（Vitest）：
  - `web/src/utils/__tests__/frontmatter.spec.ts`：真实计划 frontmatter（含 `[]` 空数组、`author: [a, b]`、ISO 日期）、块列表、CRLF、无 frontmatter、正文含 `---` 分隔线不误判。
  - `i18n.spec.ts` 既有 parity 自动覆盖新键（增删键两侧同步）。
  - `types/plans.ts` 若有既有测试则同步；无则补 `ALLOWED_TRANSITIONS` 冒烟。
- **类型门禁**：`npx vue-tsc --noEmit`。
- **手动冒烟清单**（执行完成后浏览器走查）：过滤下拉两档+记忆、语言切换后徽标/按钮/确认框全中文或全英文、转移按钮仅显示合法目标、reviewed 只见沉淀按钮、归档后从"全部"可见且无操作按钮、MetaBlock 展开/收起、正文无黑体属性行、编辑保存回显正常。

## 验收标准

1. 左栏过滤为下拉（进行中/全部），默认"进行中"，刷新后记忆上一次选择。
2. 全栈 wire 枚举为 `reviewed`/`archived`；磁盘旧值 `review_done`/`merged` 读取自动映射；`grep -rn "review_done\|\"merged\"" backend/ web/src .agents/skills docs/designs` 无残留（归档计划文件与 gen/ 除外）。
3. 中文模式下徽标、转移按钮、确认/提示文案全部中文（无中英混排）；头部无状态下拉、无重复的 archived 标签；状态展示唯一来源是徽标。
4. reviewed 计划仅显示"沉淀到 Spec"；非 reviewed 未归档计划仅显示"归档"；archived 计划无归档/沉淀按钮；后端拒绝 reviewed 直接归档（错误信息引导走沉淀）。
5. 归档（两条路径）后 `status == archived` 且文件位于 `archived/`，状态与位置恒一致。
6. 正文渲染区不再出现 frontmatter 黑体块；MetaBlock 折叠显示概要行、展开显示全量属性表格。
7. `cargo test -p musk` 全绿；`cd web && npx vue-tsc --noEmit && npx vitest run` 全绿。

## 执行步骤

- [x] **T1** `backend/crates/musk/src/plans.rs`：`PlanStatus` 变体 Rename（`ReviewDone→Reviewed`、`Merged→Archived`），`as_str` 输出 `reviewed`/`archived`，`from_str_lossy` 增加 `merged→Archived`、`review_done→Reviewed` 兼容映射；同步文内所有测试断言。验证：`cargo test -p musk plans`。 [✅ 已完成] 36 测试通过（含 status_roundtrip 改名、legacy_status_strings_map_to_new_enum）
- [x] **T2** `backend/crates/musk/src/plans.rs`：`can_transition` 换 D1 新表（去掉 reviewed→archived）；提取 `move_to_archived(seq)`（直接 set_field 写 archived+rename——不经 can_transition，因两条终态路径均不受手动状态机约束，与 D1 表一致）；`archive()` 加 reviewed 门禁并改走助手；`merge_plan_stores()` 门禁文案与终态调用改新名。验证：`cargo test -p musk plans`。 [✅ 已完成] archive_moves_file_and_sets_status / archive_rejects_reviewed / state_machine 新断言全过
- [x] **T3** `backend/crates/musk/tests/parity_plans.rs`：期望 payload 状态串与转移用例改为新枚举，补一条"archive reviewed 返回 400"用例（另修 plans_archive handler 错误码：非 not-found 一律 400）。验证：`cargo test -p musk --test parity_plans`。 [✅ 已完成] 5 测试通过（含 plans_archive_reviewed_rejected）
- [x] **T4** `web/src/types/plans.ts`：`PlanStatus` 联合类型改名；新增 `ALLOWED_TRANSITIONS`；`canTransition` 基于表实现（保留 from===to 幂等）；`STATUS_TONE` 键同步；新增并导出 `planStatusKey()`。验证：`cd web && npx vue-tsc --noEmit`。 [✅ 已完成] vue-tsc EXIT 0（ALLOWED_TRANSITIONS/planStatusKey/STATUS_TONE 同步改名）
- [x] **T5** `web/src/components/plan/PlanStatusBadge.vue`：标签改 `t(planStatusKey(status))`，删除 `exec_done` hack。验证：`npx vue-tsc --noEmit`。 [✅ 已完成] 徽标走 t(planStatusKey)，删 exec_done hack 与 text-transform
- [x] **T6** `web/src/views/PlansView.vue` + 两个 locales：checkbox 换 `filterMode` 下拉（active/all，默认 active，localStorage 持久化）；删 `includeArchived` 键，增 `filterActive`/`filterAll`。验证：`npx vitest run i18n && npx vue-tsc --noEmit`。 [✅ 已完成] filterMode 下拉+localStorage 持久化，i18n parity 测试通过
- [x] **T7** `web/src/views/PlansView.vue` + locales：删状态 `<select>`/`statusOptions`/archived 灰标签；加合法目标按钮组（`ALLOWED_TRANSITIONS` + `transitionTo` 文案）；归档/沉淀按 D3 规则互斥；`onArchive` 确认框 i18n（增 `archiveConfirm`）；locales 键改名 `statusReviewed`/`statusArchived`。验证：`npx vue-tsc --noEmit && npx vitest run`。 [✅ 已完成] 删全量状态下拉与 archived 灰标签，转移按钮组+互斥归档/沉淀，vue-tsc 0；vitest 2 个失败为存量（主仓同败：brandName 断言过时+缺 DOM 环境），与本计划无关
- [x] **T8** 新建 `web/src/utils/frontmatter.ts` + `web/src/utils/__tests__/frontmatter.spec.ts`（D4 规格的五类用例）。验证：`npx vitest run frontmatter`。 [✅ 已完成] TDD 红→绿：7/7 通过（真实 frontmatter/块列表/引号/CRLF/无围栏/正文 --- 不误判/键序）
- [x] **T9** 新建 `web/src/components/plan/PlanMetaBlock.vue`；`PlansView.vue` 接入 `splitFrontmatter`（MarkdownContent 只喂 body，顶部插 MetaBlock）；locales 增 `metaShow`/`metaHide`。验证：`npx vue-tsc --noEmit && npx vitest run`。 [✅ 已完成] PlanMetaBlock 概要行+展开表格接入，MarkdownContent 只喂 body；vue-tsc 0，vitest 22 过（2 存量失败同前）
- [ ] **T10** 全量回归：`cargo test -p musk` + `cd web && npx vue-tsc --noEmit && npx vitest run`；`grep -rn "review_done" backend/crates/musk/src web/src .agents/skills docs/designs` 确认仅剩兼容映射与注释允许项。
- [ ] **T11** 技能/设计文档同步：`.agents/skills/auto-plan-{new,review,merge,work}/SKILL.md`、`docs/designs/008-auto-plan.md` 中状态名与状态机图改为 `reviewed`/`archived`。验证：`grep -rn "review_done\|status: merged\|status: review_done" .agents/skills docs/designs` 为空。
- [ ] **T12** 手动冒烟（按测试设计清单逐项走查，dev server + 后端），结果记录到本节下方。

## 复审记录

（待 /auto-plan:review 填写）

## 待澄清事项

1. **`execution_done` 是否顺带改名**（如 `executed`，与 reviewed/archived 形容词风格统一）？本计划按原始需求只改两枚举，保持最小变更。
2. **reviewed 计划的强制归档旁路**：后端硬禁 reviewed 直接归档后，若将来需要"废弃已复审计划"（不沉淀），需加 `force` 参数——当前认为不该存在此路径，待确认。
3. **用户级技能文档**（`C:\Users\zhaop\.zcode\skills\{finish-plan,archive-plan}`）中的旧状态名属仓库外文件，本计划不修改；是否由用户自行同步待确认。
4. **MetaBlock 概要行字段**（现定 feature_name + 时间跨度 + 进度）与表格键名是否人性化显示（`feature_name` → `Feature name`），可在走查后微调。
