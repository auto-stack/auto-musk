# PLAN-076 AutoDown 统一引擎消费矩阵与接口映射报告 (076-engine-map)

> 生成时间：2026-09-19  
> 计划编号：PLAN-076（S3 AutoDown 统一引擎三模式接入与差异关闭）  
> 源码基线：  
> - `auto-musk`: `3a297ac1c448e7bd6b784346069602b0521bbe9c` (main)  
> - `auto-lang`: `d2566829ff8f6066b924354660c47f209eeab4b9` (main)  
> - `auto-down`: `84c989722cd59e811ef9c57098ac4c67c40969e2` (master)  
> Vendor 版本：  
> - `vendor/@autodown/engine`: `0.5.0-musk-vendor` (SHA-256 同步自 auto-down)  
> - `vendor/@autodown/vue`: `0.5.0-musk-alias` (re-export facade)  

---

## 1. 架构现状与核心问题

当前 `auto-musk` 项目内的 Markdown 渲染与编辑通道存在严重分叉与降级：

1. **VM 轨 Markdown 纯文本降级**：  
   `src/front/ports/renderer.vm.at` 中定义了 `widget Markdown(source: str, streaming: bool)`，其实现仅为一个包裹了 `white-space: pre-wrap` 的普通 `div`。标题、列表、代码块高亮、表格、引用及扩展块全部退化为裸字符串，且完全忽略 `streaming` 参数。
2. **`MarkdownRender` 端口不对称**：  
   `src/front/ports/renderer.web.at` 导出了 `MarkdownRender`（委托给 `MarkdownRender.vue` -> `@autodown/engine`），但 `renderer.vm.at` 未提供任何 `MarkdownRender` 实现，导致 `files_view.at` 和 `raw_preview.at` 在 VM 轨处于未对齐/缺少渲染器的状态。
3. **编辑器降级为 HTML `<textarea>` 桩**：  
   `src/front/specs_editors.at` 中的 `AutoDownEditor` 仅为一个 `.autodown-stub` 的原生 `<textarea>`，未接入真实的 `@autodown/editor` 或平台原生 `autodown_editor`。`MarkdownEditor` 亦由裸 `<textarea>` 拼装。
4. **宿主 CSS 侵入式深层补丁**：  
   `src/front/inject_styles.web-only.ts` 包含逾 60 行针对 `.dark .streaming-document` 的 `!important` 覆盖规则（涵盖文本、表格、代码块、引用、admonition、mermaid），以及 `.streaming-document .markdown-renderer > .node-slot + .node-slot` 的 `0.75rem` 垂直节奏补丁。这些属于本应由上游引擎原生支持的样式。

---

## 2. 全量消费矩阵与调用点清单

经过系统扫描，`auto-musk` 源码中共有 **21 处** 直接或间接涉及文档渲染与编辑的调用点：

| 序号 | 消费文件 | 行号 | 导入语句 / 声明方式 | 消费组件 | 传递参数 (Props) | 模式 | 运行平台 | 业务场景与语义说明 |
|---|---|---|---|---|---|---|---|---|
| **1** | `src/front/chat_message.at` | L218 | `use.web component Markdown from "src/front/ports/renderer.at"` | `Markdown` | `source: .block.text`, `streaming: isBlockStreaming(...)` | view-streaming | Both | AI 对话助手文本气泡，随 SSE chunk 动态流式推进 |
| **2** | `src/front/chat_message.at` | L354 | `use.web component Markdown from "src/front/ports/renderer.at"` | `Markdown` | `source: .block.tc.result`, `streaming: false` | view-static | Both | AI 对话助手工具调用执行结果（RESULT 块） |
| **3** | `src/front/generic_tool_card.at` | L123 | `use.web component Markdown from "src/front/ports/renderer.at"` | `Markdown` | `source: .resultText`, `streaming: false` | view-static | Both | 通用工具卡片展开区结果文本渲染 |
| **4** | `src/front/plans_view.at` | L215 | `use.web component Markdown from "src/front/ports/renderer.at"` | `Markdown` | `source: .detailBody`, `streaming: false` | view-static | Both | 计划详情面板正文渲染（位于元数据 chips 之下） |
| **5** | `src/front/relay_run_box.at` | L560 | `use.web component Markdown from "src/front/ports/renderer.at"` | `Markdown` | `source: relayTextBody(.entry.content)`, `streaming: false` | view-static | Both | RunBox 折叠文档块展开正文（`.doc-body`） |
| **6** | `src/front/relay_run_box.at` | L568 | `use.web component Markdown from "src/front/ports/renderer.at"` | `Markdown` | `source: relayTextBody(.entry.content)`, `streaming: false` | view-static | Both | RunBox 普通文本条目渲染（`.entry-md`） |
| **7** | `src/front/relay_run_box.at` | L701 | `use.web component Markdown from "src/front/ports/renderer.at"` | `Markdown` | `source: relayWriteFence(.entry.arguments)`, `streaming: false` | view-static | Both | RunBox `write` 工具文件写入代码围栏预览 |
| **8** | `src/front/relay_run_box.at` | L724 | `use.web component Markdown from "src/front/ports/renderer.at"` | `Markdown` | `source: relayFileFence(...)`, `streaming: false` | view-static | Both | RunBox `file` 工具文件内容代码围栏预览 |
| **9** | `src/front/relay_run_box.at` | L776 | `use.web component Markdown from "src/front/ports/renderer.at"` | `Markdown` | `source: .entry.result`, `streaming: false` | view-static | Both | RunBox `doc` 工具文档型结果预览（字号较普通正文略小） |
| **10** | `src/front/relay_run_box.at` | L840 | `use.web component Markdown from "src/front/ports/renderer.at"` | `Markdown` | `source: relayPhaseDetail(...)`, `streaming: false` | view-static | Both | RunBox 阶段步骤展开详情（`.ph-detail`） |
| **11** | `src/front/report_card.at` | L118 | `use.web component Markdown from "src/front/ports/renderer.at"` | `Markdown` | `source: .summary`, `streaming: false` | view-static | Both | 报告卡片 PPT 内容摘要渲染（`.report-summary`） |
| **12** | `src/front/specs_detail.at` | L281 | `use.web component Markdown from "src/front/ports/renderer.at"` | `Markdown` | `source: .remaining`, `streaming: false` | view-static | Both | `GoalDetail` 目标详情正文 |
| **13** | `src/front/specs_detail.at` | L383 | `use.web component Markdown from "src/front/ports/renderer.at"` | `Markdown` | `source: .processed`, `streaming: false` | view-static | Both | `ReviewDetail` 复审清单与说明 |
| **14** | `src/front/specs_detail.at` | L491 | `use.web component Markdown from "src/front/ports/renderer.at"` | `Markdown` | `source: .remaining`, `streaming: false` | view-static | Both | `TestDetail` 测试用例与步骤详情 |
| **15** | `src/front/specs_detail.at` | L545 | `use.web component Markdown from "src/front/ports/renderer.at"` | `Markdown` | `source: .processed`, `streaming: false` | view-static | Both | `ReportDetail` 报告指标详情 |
| **16** | `src/front/specs_detail.at` | L635 | `use.web component Markdown from "src/front/ports/renderer.at"` | `Markdown` | `source: .contentText`, `streaming: false` | view-static | Both | `SpecItemDetail` 通用规范条目详情 |
| **17** | `src/front/specs_editors.at` | L572 | `use.web component Markdown from "src/front/ports/renderer.at"` | `Markdown` | `source: .contentValue`, `streaming: false` | view-static | Both | `MarkdownEditor` 右侧实时预览视口 |
| **18** | `src/front/files_view.at` | L92 | `use.web component MarkdownRender from "src/front/ports/renderer.at"` | `MarkdownRender` | `content: .store.active_markdown` | view-static | Web-only (VM 缺口) | 文件管理器右侧 Markdown 与代码文件高亮预览面板 |
| **19** | `src/front/raw_preview.at` | L75 | `use.web component MarkdownRender from "src/front/ports/renderer.at"` | `MarkdownRender` | `content: .textContent` | view-static | Web-only (VM 缺口) | 原始文件查看器文本文件高亮预览 |
| **20** | `src/front/wiki_view.at` | L235 | 原生 DSL 关键字 `markdown { ... }` | `markdown` (DSL) | `content: .store.current_page.content` | view-static | Both | 知识库 Wiki 页面正文浏览视口 |
| **21** | `src/front/specs_leaf.at` | L206 | `use auto_down_editor: AutoDownEditor` | `AutoDownEditor` | `content: specStr(...), placeholder: ...` | edit | Both (Stub) | 规范叶子节点条目内联编辑区 |

---

## 3. 上游 AutoDown / AutoUI 平台统一接口标准

根据 `auto-lang` 与 `auto-down` 现行规格，两端统一的原生文档组件接口如下：

### 3.1 渲染组件：`autodown`（别名 `markdown`）

| 属性 / 事件 | 类型 | 默认值 | Vue 端对应实现 | VM / Iced 端对应实现 | 语义说明 |
|---|---|---|---|---|---|
| `content` | `str` | `""` | `:source="content"` | `content: String` | Markdown 源文本 |
| `streaming` | `bool` | `false` | `:streaming="streaming"` | `is_streaming` / 缓存抑制 | 是否处于增量流式阶段 |
| `final` | `bool` | `true` | `:streaming="!final"` (若未传 streaming) | `is_final` | 是否结束流式（截断悬空分词） |
| `placeholder_block_id` | `str \| int` | `None` | `:placeholder-block-id` | `placeholder.0` | 流式幽灵占位块定位 ID |
| `placeholder_height` | `float` | `0.0` | `:placeholder-height` | `placeholder.1` | 幽灵占位块像素高度 |
| `scroll_sync` | `bool` | `true` | `:scroll-sync` | `View::Scrollable` | 是否自备滚动容器与滚动派发 |
| `dark_mode` | `bool` | `false` | `:dark-mode` / `.is-dark` | 主题纪元判断 | 深浅色模式显式标记 |
| `accent` | `str` | `"indigo"` | `:accent` / `data-accent` | 规范强调色映射 | 五色强调色支持 |
| `class` | `str` | `""` | `class="..."` | 内层 padding 解析 | 容器边距与布局样式类 |
| `onscroll` | `event` | — | `@scroll="Handler"` | `(content_h, viewport_h, offset_y)` | 视口滚动事件 |

### 3.2 编辑组件：`autodown_editor`

| 属性 / 事件 | 类型 | 默认值 | Vue 端对应实现 | VM / Iced 端对应实现 | 语义说明 |
|---|---|---|---|---|---|
| `key` / `id` | `str` | `"doc"` | `data-editor-key` | 全局编辑缓存索引键 | 编辑器实例唯一状态键 |
| `content` | `str` | `""` | `:content="content"` | `autodown_editor_sync` | Markdown 正文数据绑定 |
| `placeholder` | `str` | `""` | `:placeholder="..."` | `placeholder: String` | 占位空态引导语 |
| `can_edit` | `bool` | `true` | `:can-edit="can_edit"` | 交互开启/只读视图开关 | 是否可编辑（单实例视图/编辑切换） |
| `dark_mode` | `bool` | `false` | `:dark-mode` / `.is-dark` | 编辑器前景色 palette | 深浅色主题同步 |
| `accent` | `str` | `"indigo"` | `:accent` | 选区与光标色相 | 强调配色盘 |
| `oninput` / `onupdate` | `event` | — | `@update="Handler"` | `input_value: Some(text)` | 内容变更通知（全量字符串） |
| `onsave` | `event` | — | `@save="Handler"` | 快捷键 Ctrl+S 消息派发 | 保存动作触发 |
| `oncancel` | `event` | — | `@cancel="Handler"` | 快捷键 Esc 消息派发 | 取消编辑动作触发 |

---

## 4. 平台差异与缺口定责矩阵

| 缺口 ID | 描述 | 当前状态 | 规约目标 | 修复落点与任务 | 责任归属 |
|---|---|---|---|---|---|
| **GAP-01** | VM 轨 Markdown 渲染为纯文本 | `src/front/ports/renderer.vm.at` 裸 `div` 预排 | 统一转接为原生 `autodown` 控件 | T-02: 改写 `renderer.vm.at` | `auto-musk` |
| **GAP-02** | `renderer.vm.at` 缺失 `MarkdownRender` 端口 | `files_view.at` / `raw_preview.at` 在 VM 无组件 | 补齐 `MarkdownRender` 并内联委托 `autodown` | T-02: 补齐 `renderer.vm.at` 声明 | `auto-musk` |
| **GAP-03** | 规格内联编辑 `AutoDownEditor` 为 textarea stub | `specs_editors.at` 为普通 `<textarea>` | 统一转接为原生 `autodown_editor` | T-02: 升级 `specs_editors.at` | `auto-musk` |
| **GAP-04** | Markdown 块间垂直节奏 12px 缺位 | Musk `inject_styles.web-only.ts` 强补 `0.75rem` | 上游 `@autodown/engine` CSS 内置 slot 间距 | T-03: `auto-down` 样式表收口<br>T-04: 清退 Musk 补丁 | `auto-down` / `auto-musk` |
| **GAP-05** | Markdown 深色模式 50+ 条 `!important` 覆写 | Musk `inject_styles.web-only.ts` 覆盖深色 | 上游 `@autodown/engine` 支持 `.dark` 语义 token | T-03: `auto-down` 深色 token<br>T-04: 清退 Musk 覆写 | `auto-down` / `auto-musk` |
| **GAP-06** | 表格斑马线与边框样式分叉 | Musk 强补 nth-child 底色 | 上游 engine 内置表格标准视觉层 | T-03: `auto-down` 表格收口<br>T-04: 清退 Musk 斑马线规则 | `auto-down` / `auto-musk` |
| **GAP-07** | 增量流式中的尾部悬浮未闭合标签保护 | Vue/VM 在流式末尾遇到未闭合 fence 处理不一 | 两端统一遵循 `final: false` 抑制与平滑渲染 | T-03 / T-05: 引擎流式保护测试 | `auto-down` / `auto-lang` |

---

## 5. T-02 宿主统一转接方案设计

在 T-02 中，我们将通过改造端口适配层彻底终结上述分叉：

```text
[业务调用方] (chat_message, files_view, specs_detail, etc.)
      │
      ▼
[src/front/ports/renderer.at] (单一逻辑入口)
      ├── (Web 目标) ──> renderer.web.at
      │                   ├── Markdown       ──> @autodown/engine (StreamingRenderer)
      │                   └── MarkdownRender ──> @autodown/engine (StreamingRenderer)
      │
      └── (VM 目标)  ──> renderer.vm.at (改造后)
                          ├── Markdown(source, streaming)   ──> autodown { content: source, streaming }
                          └── MarkdownRender(content, final) ──> autodown { content, streaming: !final }
```

同理，对于 `specs_editors.at`：
- 将 `AutoDownEditor(content, placeholder)` 内部的 `<textarea class="autodown-stub">` 替换为带有 `can_edit: true` 与 `onupdate` 消息派发的统一 `autodown_editor`。
- 保证无论是 Web 还是 VM，均不走任何独立的纯文本降级，全量收归 AutoDown 引擎！
