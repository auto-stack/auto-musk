---
plan_id: PLAN-056
status: executing
feature_name: Chat 块型样式收敛——块间距 / ReportCard 暗色适配 / ThinkBlock 字号与 pre 首行缩进
author: [zhaopuming]
created_at: 2026-09-02T00:00:00+08:00
updated_at: 2026-09-02T00:00:00+08:00
supersedes_spec_components: []
new_spec_components: []
touched_goals: []
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
