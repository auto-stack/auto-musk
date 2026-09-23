# UI Lint 规则规范（ui-lint）

> PLAN-090 M3 落地（2026-09-23）。
> AutoUI 代码生成护栏：在 Coding 模式生成/修改 `.at` 页面时，提供原生的、毫秒级的
> 静态特征扫描与避坑指导。

## 定位与设计原则

1. **Advisory（建议性）而非 Blocking（阻断性）**：
   - `ui_lint` 的报告结果仅供 Agent 自查和即时自纠，不构成构建或运行的硬性红线。
   - 即使报告中存在 warning 或 advice，Agent 在判断符合业务特例时仍可继续后续验证链。
2. **轻量与零依赖**：
   - musk 后端原生实现（`canvas/ui_lint.rs`），纯行级/括号深度特征匹配，无重型 AST/Parser 依赖。
   - 运行耗时通常 < 5ms，可在 Agent 代码写入文件后即时触发。
3. **红绿样例强锚定**：
   - 每条规则必须配对至少一个典型触发样例（红）和一个有效替代样例（绿），并在单测中作为回归用例（`test_ui_lint_rules`）。

## 规则注册表（v1，8 条）

| 规则 ID | 级别 (Severity) | 检出意图 | 坑源引证 | 处方要点 (Prescription) |
|:---|:---|:---|:---|:---|
| **L001** | `warning` | `span` 元素绑定 `onclick` 事件 | PLAN-088/089 #24 | 行内可点击元素一律改用 `button`（AutoUI 中 span 升格 button 会丢失非文本复杂子节点）。 |
| **L002** | `warning` | 在 `computed` 块内调用 `use.web` 声明的函数 | 视图坑 #1 | computed 块在 VM/Vue 响应式中仅用于计算纯数据。调用 web 扩展或外部副作用函数应移至 handler 域。 |
| **L003** | `advice` | `t(...)` 国际化函数传入非字面量键 | 视图坑 | VM 渲染引擎在动态键求值时查表失明。国际化键名必须为静态字符串字面量（如 `t("app.title")`）。 |
| **L004** | `warning` | `handler` 或 `msg` 命名包含下划线 | Codegen 命名契约 | 事件处理器和消息命名必须统一使用小驼峰（camelCase，如 `handleClick`、`incrementCount`）。 |
| **L005** | `warning` | `.length` 属性出现在 `computed` 块或视图表达式中 | PLAN-089 #26 | AutoUI 编译器目前无法稳定求值直接 `.length`。应在 handler 域或 model 中维护拍平的数字标量（如 `count: int`）。 |
| **L006** | `warning` | 调用 `list.join(...)` 方法 | PLAN-089 #26 | VM 链接期可能触发 Undefined symbol 异常。应使用循环或在 handler 中预拼装字符串。 |
| **L007** | `advice` | 使用 `style { ... }` 块或内嵌 CSS 声明文本 | Codegen #19 | 推荐使用 Tailwind 实用类串 `style: "..."`，避免手写原始 CSS 声明块。 |
| **L008** | `warning` | 单行括号深度过深或开闭括号严重不对称 | 084-D5 编译爆炸 | 避免在一行内书写复杂内联嵌套表达式，应拆分至中间变量或 handler 处理。 |

## 详细规则规约

### L001: 行内可点击元素避免使用 `span`
- **触发条件**：单行或多行跨行检测到 `<span ... onclick: ...>` 或 `span { ... onclick: ... }`。
- **机制原理**：AutoUI 在处理 `span` 时若附加点击事件，跨端代码生成将其升格为交互元素（`<button>`），若 `span` 内部包裹了图标、复杂格式化文本或组件插槽，会被降级为纯文本，导致 UI 损毁。
- **正例**：
  ```auto
  button {
      class: "p-1 text-blue-500",
      onclick: on_click_item,
      span { text: "Click me" }
  }
  ```

### L002: computed 块禁止调用 web 外部扩展函数
- **触发条件**：解析出以 `use.web` 导入的模块（如 `use.web "./api.ts" as api`），且在 `computed { ... }` 块内发现了该模块函数的调用（如 `api.fetch(...)`）。
- **机制原理**：`computed` 是纯函数派生状态。触发 I/O 或异步交互会导致渲染死循环或状态串味。
- **正例**：在 `handler` 处理消息时发起调用，并将结果存储在 `state` 中；`computed` 仅读取 `state`。

### L003: 国际化函数 `t(...)` 必须使用静态字面量键
- **触发条件**：`t(...)` 调用入参为变量或拼接表达式（非双引号字面量）。
- **机制原理**：静态分析与 VM 词典查表仅支持静态键。动态键在 VM 模式下导致静默查表失败。
- **正例**：`t("nav.home")`

### L004: handler 与 msg 采用 camelCase 驼峰命名
- **触发条件**：`handler handle_click`、`handler on_submit` 或 `msg Click_Button`。
- **机制原理**：Auto 语言代码生成至前端 Vue 3 / TypeScript 绑定层时，下划线命名与框架生成模板冲突。
- **正例**：`handler handleClick`、`msg ClickButton`

### L005: 视图/computed 避免内联求值 `.length`
- **触发条件**：在 `computed` 块内或 view 模板表达式中出现 `.length`。
- **机制原理**：Auto 容器类型转换与 VM 属性访问对 `.length` 的动态派发存在已知降级。
- **正例**：在数据更新时的 handler 里显式维护 `itemCount: int`，或者在 Model 中增加计数标量。

### L006: 避免调用 `list.join(...)`
- **触发条件**：源码中出现 `.join(` 调用。
- **机制原理**：标准库列表方法的链接符号可能在特定编译 target（如 VM 纯解释轨）未导出。
- **正例**：通过循环手动拼接字符串，或使用格式化插值。

### L007: 推荐 Tailwind 样式串，避免原始 CSS 样式块
- **触发条件**：出现 `style {` 或 `style: "...: ...;"` 样式声明。
- **机制原理**：AutoUI 原生深度集成 Tailwind CSS。原始样式块不利于响应式与主题变量统一管理。
- **正例**：`class: "flex flex-col gap-2 p-4 bg-white rounded-lg shadow"`

### L008: 括号深度过深防爆预警
- **触发条件**：单行中 `(`, `[`, `{` 嵌套层数超过阈值（如深度 ≥ 6），或单行开闭括号差值过大。
- **机制原理**：AutoUI 递归下降解析器对极深嵌套的复杂内联表达式可能遭遇递归栈压力或 codegen 歧义。
- **正例**：拆分成局部变量赋值后再组合。

## 输出格式契约

`ui_lint` 工具及后端接口提供标准化的纯文本报表输出（`format_lint_report`）：

```text
UI Lint Report: <path>
Found 2 advisory finding(s):

[L001 WARN] Line 12: span with onclick detected
  Excerpt: span { class: "btn", onclick: handleClick, text: "Save" }
  Fix: Replace clickable span with button to avoid dropping non-text children

[L004 WARN] Line 25: handler name uses snake_case
  Excerpt: handler handle_save(msg: SaveMsg)
  Fix: Use camelCase for handler and msg names (e.g. handleSave)
```

若检查通过，输出：
```text
UI Lint: <path> passed with 0 findings.
```

## Agent 生成流程集成

在 `templates.rs`（Generation Prompt v2）中约定：
1. **生成即自检**：Agent 在写入 `.at` 文件后，必须立即调用 `ui_lint(path)` 进行快检。
2. **就地自纠**：若报告中存在 findings，Agent 应参考 Fix 处方就地修正，再次复验。
3. **结合 Blueprint 契约**：对于 Blueprint 场景，`ui_lint` 作为快速语法护栏，随后衔接 `bp_check` 行为硬性验证。
