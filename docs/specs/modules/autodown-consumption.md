# AutoDown Engine Consumption Spec (统一文档引擎消费契约)

> 规约编号：SPEC-FRONT-AUTODOWN-01  
> 规范等级：Canonical Module Spec  
> 建立依据：PLAN-076 (AutoDown 统一引擎三模式接入与差异关闭)  
> 维护责任：Front Architecture / UI Parity  

---

## 1. 架构目标与定位

AutoDown Unified Document Engine (`@autodown/engine` / auto-lang native `autodown` & `autodown_editor`) 是 auto-musk 全仓唯一的文档解析、渲染与编辑引擎。

本规约强制关闭任何临时降级与纯文本 fallback：
1. **唯一引擎原则**：全仓禁止引入第三种 markdown 解析/渲染器，禁止裸 `div`/`pre-wrap` 文本降级。
2. **双轨平权**：Web/Vue 轨与 Native/VM 轨均基于统一 AST 与组件模型，具备完全一致的视觉与交互语义。
3. **三模式闭合**：全面覆盖静态展示 (`view`)、流式输出 (`stream`) 与交互编辑 (`edit`) 三种标准模式。

---

## 2. 消费端口与组件映射规范

Musk 宿主前端代码仅通过统一端口抽象与原生 AST 组件消费 AutoDown 引擎能力：

```
                    ┌────────────────────────┐
                    │     Musk Consumer      │
                    │ (Chat, Specs, Wiki...) │
                    └───────────┬────────────┘
                                │
        ┌───────────────────────┴───────────────────────┐
        ▼                                               ▼
[ ports/renderer.web.at ]                     [ ports/renderer.vm.at ]
        │                                               │
        ├─ Markdown (platform:markdown)                 ├─ Markdown (native autodown)
        ├─ MarkdownRender (local adapter)               ├─ MarkdownRender (native autodown)
        └─ AutoDownEditor (@autodown/engine)            └─ AutoDownEditor (autodown_editor)
        │                                               │
        ▼                                               ▼
@autodown/engine (Vue 3)                       ui::autodown_render (VM/Iced)
```

### 2.1 组件消费矩阵

| 宿主端口 / 组件 | 模式 (Mode) | Web / Vue 映射 | Native / VM 映射 | 语义与契约 |
|---|---|---|---|---|
| `Markdown` (`ports/renderer.at`) | `view` / `stream` | `platform:markdown` → `@autodown/engine` `StreamingRenderer` | 原生 `autodown { content, streaming }` | 气泡与正文流式/静态渲染，支持代码块复制、表格、Callout、Task list |
| `MarkdownRender` (`ports/renderer.at`) | `view` (静态) | `src/front/components/MarkdownRender.vue` | 原生 `widget MarkdownRender` (转接 `autodown { streaming: false }`) | 文件只读预览 (`files_view`) 与草稿只读快照 (`raw_preview`) |
| `AutoDownEditor` (`specs_editors.at`) | `edit` (编辑) | 原生 `autodown_editor` (生成 `<EngineEditor />`) | 原生 `autodown_editor { content, oninput }` | 交互式 Markdown 编辑，支持块选择、双向编辑、快捷菜单与自动保存 |

---

## 3. 三模式标准契约

### 3.1 模式一：静态视图 (View Mode)
- **输入**：`source: str`，`streaming: false`。
- **渲染特征**：
  - 标题降序节奏：H1 (1.58rem) ~ H6 (0.95rem)，首行外边距折叠，块间间距标准 12px (`0.75rem`)。
  - 代码块：顶栏显示语言标识与复制按钮，语法高亮完整映射 Design 22 调色盘。
  - 任务列表：支持选中/未选状态复选框。
  - 表格：斑马线支持，对齐方式对齐。

### 3.2 模式二：流式渲染 (Streaming Mode)
- **输入**：`source: str`，`streaming: true`。
- **保护机制**：
  - 未闭合语法容错：未闭合的 ```` ``` ```` 代码围栏、未闭合的表格 `|`、未闭合的 Callout 不破坏页面布局。
  - 打字机增量：尾部增量节点平滑上屏，禁止整段重绘闪烁。
  - 滚动同步：支持 `scrollSync: false` 保持自然标题外边距，或 `scrollSync: true` 用于编辑端对齐。

### 3.3 模式三：交互编辑 (Edit Mode)
- **输入与事件**：
  - 属性：`content: str`，`placeholder: str`，`dark_mode: bool`。
  - 事件：`oninput: (val) => ...` (输入实时响应)，`onsave: (val) => ...` (保存提交)。
- **双端交互**：
  - Web 端：提供 SlashMenu、BubbleMenu、代码块多语言切换。
  - VM 端：底层基于 `ui::autodown_editor` 原生事件循环分发与文本光标计算。

---

## 4. 主题与样式继承规范

### 4.1 深色模式 (Dark Mode) 自动适配
- **单源标准**：深色模式视觉设计严格遵循 auto-lang Design 22 §7 (Zinc 基线色)。
- **环境传播机制**：
  - 当宿主环境为深色（`html.dark`、外层包含 `.dark` 或 `[data-theme='dark']`）时，引擎根节点自动挂载 `.is-dark` class。
  - `--ad-fg: #fafafa`，`--ad-surface: #09090b`，`--ad-border: #3f3f46` 自动生效。
  - 宿主严禁在全局或局部层增加针对 `.dark .streaming-document ...` 的 `!important` 覆写。

### 4.2 块间节奏 (Slot Rhythm)
- 单个 Markdown 渲染器内兄弟块节点间距标准为 `margin-top: 0.75rem` (12px)。
- 间距由 `@autodown/engine` 原生 `:deep(.markdown-renderer > .node-slot + .node-slot)` 提供，宿主已清退 `inject_styles.web-only.ts` 临时规则。

---

## 5. 版本与供应链管理

- **源仓库**：`D:/autostack/auto-down` (`master` 分支)。
- **消费落点**：`auto-musk/vendor/@autodown/engine`。
- **发布审计**：
  - 同步 vendor 必须包含全量 dist 产物及 `.dist-stamp`。
  - 必须记录对应 `auto-down` master commit hash 与验证记录。
  - 禁止创建任何指向 vendor 的 symlink/junction。
