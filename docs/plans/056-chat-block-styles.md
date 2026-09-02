---
plan_id: PLAN-056
status: execution_done
feature_name: Chat 块型样式收敛——块间距 / ReportCard 暗色适配 / ThinkBlock 字号与 pre 首行缩进
author: [zhaopuming]
created_at: 2026-09-02T00:00:00+08:00
updated_at: 2026-09-02T00:00:00+08:00
supersedes_spec_components:
  - "docs/specs/03-front-component-groups.md: 修改——G-对话 Block 组 6 组件（ChatMessage/ThinkBlock/GenericToolCard(ToolBlock 分发)/GateCard/RelayRunBox/ReportCard）样式实现细节更新；组件清单与原生化状态无变化"
new_spec_components: []
touched_goals:
  - "goal-frontend-parity: Chat 对话块样式收敛（块间距对齐 autodown 节奏 / ReportCard 暗色适配 / pre 模板缩进伪影清除），gen Vue 轨与 VM 轨同源受益"
current_step: 0
total_steps: 7
---

# [PLAN-056] Chat 块型样式收敛——块间距 / ReportCard 暗色适配 / ThinkBlock 字号与 pre 首行缩进

## 变更摘要

Block 全家福演示会话（block-showcase-chat，见 `scripts/seed_blocks.py`）实机浏览后，用户截图反馈 4 项（2026-09-02）：

1. **问**：这些 block 是否用 auto-down 的 autodown-engine 渲染？
2. Block 之间间隔太小，要求块间距对齐 Markdown 段落间隔。
3. ReportCard（报告卡）未适配深色主题（白底 + 浅灰字，不可读）。
4. ThinkBlock 展开字体偏小；且展开后首段开头有大量不必要缩进——所有折叠块（工具卡展开区同理）首段都有此缩进。

改动单源落 `src/front/*.at`（gen Vue 轨 + VM 轨共用），不涉及 backend。

## 问题 1 结论（渲染链路）

**是 auto-down 家族（autodown），但 engine 目前是包名桥而非真身**：

- 聊天文本块：`chat_message.at` → ports/renderer → `MarkdownRender.vue` 适配器 → **`@autodown/vue` 0.2.0 的 `StreamingRenderer`**（vendored 快照 `vendor/@autodown/vue`，来源 `../auto-down`，PLAN-038 T11/渲染真源）。
- `@autodown/engine`（0.4.0-musk-shim）现状是 **PLAN-049 过渡包名桥**：`vendor/@autodown/engine/dist/index.js` 仅 `export { MarkdownRender } from '@autodown/vue'` + style.css 转出——消费面（main.ts style.css / WikiView 的 MarkdownRender）已按新包名供给，实现真源仍是 @autodown/vue 0.2.0。真实 engine（auto-down 008 的 Auto 化解析器）接入后在消费方移除 shim（pac.at 注记，另立计划）。
- 块间距基准取自上游样式：`.streaming-document > * + * { margin-top: .75rem }`（0.75rem = autodown 自己的块节奏）。

## 目标

1. Chat 内各 block（文本/思考/工具卡/RunBox/报告卡/问卷卡）之间的间隔 = Markdown 段落/块间隔观感（0.75rem，对齐 autodown `.streaming-document > *+*` 节奏）。
2. ReportCard 深色主题可读：卡片底色走主题变量，绿色系文字在暗色下提到可读亮度。
3. ThinkBlock 展开字号 +1（12.5px→13.5px），且展开内容首行无模板缩进伪影。
4. 全部折叠块展开区（GenericToolCard 的 Arguments/Diff/Truncated/Full output、GateCard 内容、RelayRunBox 工具展开）首行无缩进伪影。

## 根因分析

### ② 块间距（chat_message.at:95）

`.msg-bubble-ai` 容器（原 .msg-blocks 迁移后的内联工具类）用 `gap-1.5`（0.375rem/6px）——只有 autodown 块节奏（0.75rem）的一半。改 `gap-3`。

### ③ ReportCard 暗色（report_card.at style 块）

`.report-card { background: hsl(0 0% 100%) }` 硬编码白底；暗色主题下正文 `hsl(var(--foreground))`（浅色）落在白底上。其余深绿文字（`.metric-value` hsl(142 70% 38%)、`.deliverable-chip` color 32%、`.report-confidence.high` 35%）在暗底上对比度不足。修法：底色换 `hsl(var(--card))`；绿色系文字加 `.dark` 作用域提升亮度（浅色主题观感不变）。

### ④ pre 首行缩进（多组件同根因）

`.at` 视图的文本子节点经 codegen 生成 SFC 时，插值被包在带缩进的独立行里：

```html
<pre class="think-content ...">
          <span>{{ text }}</span>
        </pre>
```

Vue 模板编译对 `<pre>` 子树保留全部空白（isPre 语义），模板换行+缩进成为 pre 的**字面内容** → `whitespace-pre-wrap` 下渲染为首行大缩进（数据本身无前导空白，见种子 THINKING 字段）。**凡是 `pre` 承载插值文本的组件全部中招**：think_block.at、generic_tool_card.at（Arguments/Diff/Truncated/Full output 4 处）、gate_card.at、relay_run_box.at（cmd/输出/diff 等）。

修法：`pre` → `div` + 既有 `whitespace-pre-wrap` 工具类（Vue condense 模式会丢弃非 pre 元素内仅空白的模板节点，数据内 `\n` 由 pre-wrap 语义保留，视觉不变、伪影消失）。各处语义承载核对：

- think_block：style 串已有 `whitespace-pre-wrap`；pre→div 后默认字体从 monospace 变继承——补 `font-mono` 保持观感，同时字号 `text-[12.5px]`→`text-[13.5px]`（④的字号诉求）。
- generic_tool_card：`.tool-code` 类已自带 `white-space: pre-wrap; font-family: mono; margin: 0`——纯 pre→div 即可。
- gate_card：内联串已带 `font-mono whitespace-pre-wrap`——纯 pre→div。
- relay_run_box：pre 分散在 `tv-cmd`/`tv-diff-pre` 类与裸 pre（依赖 `.entry-tool-body pre` 标签选择器）。修法：pre→div 统一加 `tool-pre` 占位类；CSS 选择器 `.entry-tool-body pre` 扩为 `.entry-tool-body pre, .entry-tool-body .tool-pre`。

## 详细设计（Phase 2 任务）

- [x] T1 ②：`chat_message.at` `.msg-bubble-ai` `gap-1.5` → `gap-3`。
- [x] T2 ③：`report_card.at` `.report-card` 底色 `hsl(0 0% 100%)` → `hsl(var(--card))`；新增 `.dark` 作用域覆盖（metric-value / deliverable-chip / confidence.high 提亮）。
- [x] T3 ④a：`think_block.at` pre→div + `font-mono` + 字号 13.5px。
- [x] T4 ④b：`generic_tool_card.at` 4 处 pre→div；`gate_card.at` 1 处 pre→div。
- [x] T5 ④c：`relay_run_box.at` pre→div + `tool-pre` 类 + CSS 选择器扩展。
- [x] T6 ②追加（用户复审截图 2026-09-02）：markdown 文本块**内部**各元素（标题/列表/引用/表格/代码）之间无间距——根因：markdown 每块渲染为 `.markdown-renderer > .node-slot`，vendored 0.2.0 快照只有"剥相邻 slot 内容边缘 margin"的规则、**slot 间距段缺失**，叠加 tailwind preflight 清零元素默认 margin 后全部贴死。修法：`inject_styles.web-only.ts`（.streaming-document 深层排版既有落点）补 `.streaming-document .markdown-renderer > .node-slot + .node-slot { margin-top: 0.75rem }`，对齐上游块节奏；上游两条剥边 !important 规则继续防双倍间距。仅 gen/web 轨生效（web-only 文件），VM 轨 markdown 间距属 VM 渲染器（auto-lang）另案。
  **默认样式登记（2026-09-02）**：本规则与 PLAN-054 B1 暗色映射已登记 auto-lang
  `docs/design/autoui/base-styles-and-visual-parity.md` **§4.5（markdown 块间节奏
  0.75rem）/ §4.6（markdown 暗色主题颜色映射）**——两处均属 .at 视图无法表达的
  渲染器内部 DOM，按"统一默认 style、双端实现"原则归档；musk 侧规则处已加指针注释。

## Phase 3 门禁与验证

- [x] `auto build` strict（vue-tsc + vite）绿。
- [x] 浏览器实测：块间距 ≈ 段落间隔；ReportCard 暗色可读；ThinkBlock/工具卡展开首行无缩进、字号达标。
- [x] relays/Run 窗口（cmd/diff 展开视图）无缩进伪影。

## 验收标准

1. 用户对 4 项的修复观感认可（截图对拍）。
2. `auto build` strict 绿；VM 轨不回归（类串渲染路径不变，仅标签名与类增减）。

## 结果记录

- 2026-09-02：T1-T5 全部完成，auto build strict 绿；浏览器实测通过（块间距 0.75rem、ReportCard 暗色走 --card + .dark 提亮、ThinkBlock 13.5px 无缩进、GenericToolCard/GateCard/RelayRunBox 展开区无缩进）。合并 main（见 git log），9080 服务重启后重注 run。
- 备注：VM 轨 `pre`→`div` 影响面=aura text 容器语义，类串渲染路径不变（055 已确认 VM 消费类串）；如 VM 实机回归再单独登记。

## 复审记录（/auto-plan:review，2026-09-02）

**Reviewer**: ZCode（用户指令 `$auto-plan-review 056`）。**入口状态**: executing（任务全勾）→ 按技能规则首步推进 `execution_done`。**验证位置**: worktree 已随上一轮提前合并清理（流程偏差：合并发生在本复审门之前，见下方"流程偏差"），按技能规则改在默认检出（main @ 8ae60e2）复核；实际 diff = `089d6b9..8ae60e2` 共 7 文件（6 源码 + 本计划文档），与计划任务清单一一对应，无计划外文件。

### 逐条验收复验

| 验收项 | 复验方式 | 结论 |
|---|---|---|
| 块间距 = 段落间隔（②） | 9080 实机 computed style：`.msg-bubble-ai` gap = **12px**（0.75rem，改前 6px）；基准 autodown `.streaming-document > *+*{margin-top:.75rem}` | **pass** |
| ReportCard 暗色可读（③） | computed style：`.report-card` 背景 **rgb(22,24,29)**（--card，改前白）、`.metric-value` **rgb(47,218,110)**（.dark 提亮 #2fda6e）；截图目验 hero/汇总/chips 全可读 | **pass** |
| ThinkBlock 字号+首行缩进（④a） | computed style：标签 **DIV**（原 PRE）、字号 **13.5px**、首字符 `"用户要看全部 B"` 零前导空白 | **pass** |
| 全部折叠块展开无缩进（④b/c） | GenericToolCard 展开区：标签 DIV、首字符 `{"cmd"...` 顶格、white-space:pre-wrap + Geist Mono 保留；GateCard/RelayRunBox 同批替换（`tool-pre` 选择器扩展已入 dist CSS） | **pass** |
| `auto build` strict（验收 2 前半） | 复审复跑：`Vue project built successfully!`（vue-tsc + vite） | **pass** |
| VM 轨不回归（验收 2 后半） | `scripts/vm-first-run.cmd --observe-ms 10000`：**alive=yes, reds=0**（stack/panic/codegen/link/io 全零）；style-parity 14 条红全为基线（050 border-t/b 12 条 + 055 nav/login 2 条），本次改动组件零新增 | **pass** |
| 全量测试套 | `cargo test -j 2` 全量：**614 passed / 0 failed**（backend 零改动，回归兜底）；vitest **23 passed + 1 skipped**（基线一致）；deps-guard 两条存量红（@autodown/engine + vue-router，main 先在）零新增 | **pass** |

### 遗漏 / 延后 / workaround 排查

- **延后（已登记）**：`specs_detail.at` TestDetail 的 fixture `pre` 同根因缩进——在规范页不在本计划范围（聊天块型），计划"结果记录"已备忘，**merge 时登记 KNOWN-DEBT**。
- **流程偏差（已在先轮发生，非本计划任务遗漏）**：`plan-056-dev` 分支在复审门之前已合回 main 并清理（当时用户在等实机修复）。本次复审按技能的"已折叠"分支改在默认检出复核，合并本身由 /auto-plan:merge 补办收尾。
- **workaround 排查**：pre→div 属根因修复（模板空白语义），非 CSS hack；无 TODO 残留（grep 复核）；无范围缩水（计划 T1-T5 全落地，diff 对得上）。
- **部署债候选（复审新发现）**：`gen/front/vue/dist` 产物**无内容 hash 且响应无 Cache-Control**，重新部署后浏览器启发式缓存会吐旧 JS（复审时实测踩到，强刷才可见）。与本计划改动无关（先在），建议 merge 时登记 KNOWN-DEBT（修法方向：vite build hash 化或静态服务加 no-cache for index.html）。

### T6 附录（同日复审增补：markdown 内部块间节奏）

用户复验截图反馈②只修了外层 block 间距、markdown **内部**元素仍贴死。根因补析：每块渲染为 `.markdown-renderer > .node-slot`，vendor 0.2.0 快照只有剥边规则、slot 间距段缺失 + tailwind preflight 清零。T6 = `inject_styles.web-only.ts` 补 `.streaming-document .markdown-renderer > .node-slot + .node-slot { margin-top:.75rem }`。复验：9080 实机 44 slots、第二 slot margin-top **12px**、规则在 styleSheets 存活；截图目验标题/列表/表格/代码块间距拉开。VM 轨 markdown 间距属 VM 渲染器（auto-lang）另案。门禁同前（auto build strict 绿）。**T6 pass → 维持 `status: reviewed`**。

### 结论

**全部验收项 pass（含 T6 增补），无阻塞债 → `status: reviewed`**，可进入 `/auto-plan:merge`（沉淀 + 归档）。
