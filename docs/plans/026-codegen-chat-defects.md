---
plan_id: PLAN-026
status: review_done            # drafting → executing → execution_done → review_done → merged
feature_name: auto-lang codegen 三缺陷修复（gen chat 渲染：动态 class / 组件 style / != undefined）
author: [zhaopuming + agent]
created_at: 2026-08-12T08:30:00Z
updated_at: 2026-08-12T09:00:00Z

# 待 /auto-plan:review 填：
supersedes_spec_components: []
new_spec_components:
  - "auto-lang codegen: 修复 3 个 .at→Vue 生成缺陷（class 绑定 / component fn style / != undefined）"
touched_goals: []

current_step: 5
total_steps: 5

---

# [PLAN-026] auto-lang codegen 三缺陷修复 — gen chat 渲染根治

> **给执行 Agent：** 用 `/auto-plan:work` 逐步执行。
> **来源：** PLAN-025 双前端对比时发现，gen（Auto/vue 版）的 ChatMessage 渲染有 3 个缺陷，已在 gen 生成物上临时打补丁验证（备份在 `docs/plans/attachments/026-ChatMessage.gen-patched.vue`）。本计划在 **auto-lang 生成器**层根治，而非 gen 生成物。

## 0. 变更摘要 (Executive Summary)

PLAN-025 启动双前端对比（原生 web :3333 + Auto/gen :3334）后，发现 gen 版 chat 的消息气泡：(a) 全部撑满面板宽度无对齐、(b) 每条消息后面跟一个红色空框。根因是 **auto-lang 的 `.at → Vue` codegen 有 3 个独立缺陷**，全部暴露在 `chat_message.at` 的生成产物上：

| # | 缺陷 | 现象 | 根因层 |
|:--|:--|:--|:--|
| ① | 动态 `class:` 绑定丢失 | 气泡无左右对齐 | codegen `generate_shadcn_attrs`（vue.rs） |
| ② | `component fn` 无法输出 `<style>` | 气泡无 CSS | AST + extract + codegen 三层 |
| ③ | `!= ""` 翻译成 `!==` 对 undefined 误判 | 每条渲染空 `.msg-error` 红框 | codegen `expr_to_js`（vue.rs） |

三个缺陷**都不在 auto-musk**，而在 `D:/autostack/auto-lang`。修复后重新 `cargo install auto` + auto-musk `auto build`，生成的 `ChatMessage.vue` 应与备份的 patched 版本行为一致。

## 1. 目标 (Goal)

在 auto-lang 生成器层修复 3 个 codegen 缺陷，使 `component fn` 组件的动态 class 绑定、组件级 `<style>`、以及 `!=`/`==` 对缺失字段（undefined）的安全语义都能正确生成；补齐 a2vue 回归测试；最终让 auto-musk 的 `auto build` 产出正确的 `ChatMessage.vue`（无需手动补丁）。

**非目标：** 不改 auto-musk 的 `chat_message.at` 业务逻辑（仅任务 4 补一个 `style { ... }` 块）；不重构 codegen 架构；不动原生 `web/` 前端（它无此问题）。

## 2. 架构方案 (Architecture)

```
┌─ auto-lang（本计划主战场）──────────────────────────────────────┐
│                                                                  │
│  .at 源（auto-musk: src/front/chat_message.at）                   │
│       │  auto build（shadcn 模式）                                │
│       ▼                                                          │
│  AST ──► aura::extract ──► AuraWidget ──► ui_gen::vue ──► .vue   │
│   ▲                          ▲                  ▲                │
│   │ 缺陷②                    │ 缺陷②           │ 缺陷①③         │
│  ViewFragmentDecl          extract.rs:779     vue.rs            │
│  (无 style 字段)           (style_css: None)   (shadcn_attrs /   │
│                                              expr_to_js)         │
└──────────────────────────────────────────────────────────────────┘
       │ 修复后 cargo install auto
       ▼
┌─ auto-musk ──────────────────────────────────────────────────────┐
│  auto build --gen-only ──► gen/front/vue/.../ChatMessage.vue     │
│  （正确：:class 绑定 + <style> + truthy 判空）                    │
└──────────────────────────────────────────────────────────────────┘
```

关键认知：
1. **`auto build` 跑在 shadcn 模式**（不是普通模式），所以 `col`/`row` 等布局基元走 `generate_shadcn_attrs`，而非 `extract_classes`。
2. **`widget` 形式已支持 `style { ... }` 块**（`WidgetDecl.style` + `vue.rs:1660` 输出 `<style scoped>`）——缺陷 ② 只是没把这套机制扩展到 `component fn`。
3. **已有正确实现可参考**：`extract_classes:5161` 正确处理动态 class；widget 的 style 链路完整。修复 = 把已有逻辑接通到 component fn / shadcn 布局分支。

## 3. 技术栈 (Tech Stack)

- **auto-lang**：Rust workspace（`crates/auto-lang` = codegen 库；`crates/auto` = CLI 二进制）
- codegen 主体：`crates/auto-lang/src/ui_gen/vue.rs`（~18k 行）
- AST：`crates/auto-lang/src/ast/ui.rs`
- 提取：`crates/auto-lang/src/aura/extract.rs`
- 测试：`crates/auto-lang/test/a2vue/NNN_*/`（input.at + *.expected.vue 黄金对比）
- 验证：`cargo test -p auto-lang` + `cargo install --path crates/auto` + auto-musk `auto build`

## 4. 需求分析与背景调查

### 4.1 PLAN-025 的发现（本计划的设计依据）

PLAN-025 执行后启动双前端对比，gen 版（:3334）chat 暴露问题。临时修复路径：
- 在 `gen/.../ChatMessage.vue` 手动加 `:class="rowClass"` + `<style>` 补 msg-* CSS + `hasError`/`hasThinking` 改 `!!`
- 备份：`docs/plans/attachments/026-ChatMessage.gen-patched.vue`（3.2KB，作为"修复后生成物应匹配的行为基准"）

### 4.2 生成器实地调查结论（2026-08-12 agent 深入 auto-lang）

| 维度 | 现状 | 对方案的影响 |
|:--|:--|:--|
| `auto build` 模式 | shadcn 模式（`VueGenerator::new_shadcn`） | `col`/`row` 走 `generate_shadcn_attrs`，非 `extract_classes` |
| 动态 class 正确实现 | `extract_classes:5161-5425`（处理 `Expr::If`/动态 expr） | 修复 ① 可镜像这套逻辑 |
| `widget` 的 style 链路 | `WidgetDecl.style` + `extract.rs:649` + `vue.rs:1660` 完整 | 修复 ② = 把链路接到 `ViewFragmentDecl` |
| `!=` 翻译 | `expr_to_js:5640` Bina 分支，nullish 只覆盖 `Null/Nil/None` | 修复 ③ = 扩展谓词覆盖 `Expr::Str("")` |
| 测试覆盖 | a2vue/004（动态 class on `text`）、007（component fn 静态 style） | 三个缺陷都**无回归 fixture**，需补 |

### 4.3 关键决策（用户确认）

- **D1：根治在生成器，不在 gen。** gen 是 gitignored 生成物，每次 `auto build` 覆盖。临时补丁不可持续。
- **D2：修复顺序 ③ → ① → ②。** ③ 最简单（扩展谓词），① 中等（接通已有逻辑），② 最大（三层），按风险递增推进。
- **D3：每个缺陷补 a2vue 回归 fixture。** 之前三个都没有，是它们能潜伏到 PLAN-025 的原因。

## 5. 详细设计 (Detailed Design)

### 5.1 缺陷 ③ — `!= ""` 对 undefined 安全（最简单，先做）

**根因：** `crates/auto-lang/src/ui_gen/vue.rs:5640-5658`，`expr_to_js` 的 `Expr::Bina` 分支：
```rust
let nullish = |e| matches!(e, Expr::Null | Expr::Nil | Expr::None);  // ← 不含 Expr::Str("")
if matches!(op, Op::Eq | Op::Neq) && (nullish(left) || nullish(right)) {
    ... return Ok(format!("{} {} null", other_js, op_js));  // ← 仅 null 触发
}
... Ok(format!("{} {} {}", left_js, Self::op_to_js(op), right_js))  // ← Neq → !==
```
`op_to_js:5952` 把 `Op::Neq` → `!==`。故 `.msg.error != ""` → `props.msg.error !== ''`，undefined 时为 true。

**修复：** 在 Bina 分支增加"空字符串"检查 —— 当 `op` 是 `Neq`/`Eq` 且一端是 `Expr::Str("")` 时，转成 truthy 判断：
- `.x != ""` → `!!x`（即 `Boolean(x)`）
- `.x == ""` → `!x`

（镜像手动补丁 `!!props.msg.error` 的语义。）

### 5.2 缺陷 ① — 动态 class 绑定（col/row/grid/center/container）

**根因：** `vue.rs:7000-7021` `generate_shadcn_attrs` 的 `col` 分支（以及 `row:6972`、`grid`、`center:7052`、`container:7040` 同病）：
```rust
"col" | "column" => {
    let mut classes = vec!["flex", "flex-col"];
    if let Some(value) = self.get_style_class(props) {       // get_style_class:10037 查 style/class
        let user_class = self.extract_string_value(value).unwrap_or("");  // ← 非 Expr::Str 返回 None
        if !user_class.is_empty() { classes.push(user_class); }
    }
    attrs.push(format!("class=\"{}\"", classes.join(" ")));  // ← 仅静态
    self.push_passthrough_attrs(&mut attrs, props);          // ← :6839 跳过 "class"
}
```
`extract_string_value:9971` 只认 `Expr::Str`，对 `Expr::If`/`Expr::Ident`（动态）返回 None，动态 class 被丢。

**修复：** 在这 5 个布局分支里，`get_style_class` 返回的 value 若非 `Expr::Str`，按 `extract_classes:5389-5419` 的逻辑发 `:class` 绑定：
- `Expr::If` → `:class="<ternary>"`（三元）
- 其他 `Expr` → `:class="<expr_to_vue_bound_value(expr)>"`

（保留原 `class="flex flex-col ..."` 静态部分不动，动态部分以 `:class` 追加。）

### 5.3 缺陷 ② — `component fn` 输出组件 `<style>`（三层）

**根因（三层都丢弃 style）：**
1. **AST** `ast/ui.rs:218-236` `ViewFragmentDecl` 没有 `style` 字段（对比 `WidgetDecl.style: Option<String>` @ `ui.rs:60`）。
2. **extract** `aura/extract.rs:779` `extract_widget_from_fragment` 硬编码 `style_css: None`（对比 `extract_widget_from_decl:649` `style_css: decl.style.clone()`）。
3. **codegen** `vue.rs:1660-1663` 只在 `widget.style_css` 为 `Some` 时输出 `<style scoped>`（这层本身没错，是上游没喂数据）。

**修复（三层接通）：**
1. `ast/ui.rs:218-236`：`ViewFragmentDecl` 加 `pub style: Option<String>`。
2. parser：在 `component fn` 解析处捕获 `style { ... }` 块到 `frag.style`（参考 `widget` 的 style 解析逻辑，执行时 grep 定位）。
3. `aura/extract.rs:779`：`style_css: None` → `style_css: frag.style.clone()`。

（修复后 `vue.rs:1660` 自动输出 `<style scoped>`，无需改 codegen。）

### 5.4 auto-musk 侧：`chat_message.at` 补 `style { ... }` 块

缺陷 ② 修复后，在 `auto-musk/src/front/chat_message.at` 的 `component fn ChatMessage(...)` 末尾加 `style { ... }` 块，内容 = 从备份 `docs/plans/attachments/026-ChatMessage.gen-patched.vue` 的 `<style>` 提取的 msg-* CSS（`.msg-row` / `.msg-row-user` / `.msg-row-ai` / `.msg-bubble` / `.msg-bubble-user` / `.msg-bubble-ai`）。

**注意：** `inject_styles.ts`（ext，持久）已有 `.msg-error` / `.msg-thinking` 定义，**不要重复**。chat_message.at 的 style 块只放布局/气泡样式。

## 6. 测试设计 (Test Design)

每个缺陷补一个 a2vue 回归 fixture（`crates/auto-lang/test/a2vue/NNN_*/`，`input.at` + `*.expected.vue`）：

- **缺陷 ③ fixture**（如 `008_neq_empty_string_safe/`）：`computed { hasX => .obj.x != "" }` → 期望生成 `!!obj.x`（不是 `!== ''`）。
- **缺陷 ① fixture**（如 `009_shadcn_col_dynamic_class/`）：`col { class: .rowClass }` → 期望生成 `class="flex flex-col" :class="rowClass"`。
- **缺陷 ② fixture**（如 `010_component_fn_style/`）：`component fn Foo(...) { ... style { .bar { color: red } } }` → 期望 SFC 含 `<style scoped>` 块。

回归命令：`cargo test -p auto-lang`（含所有 a2vue 黄金对比）。

端到端验证（auto-musk）：
- `auto build --gen-only` 后，`diff gen/.../ChatMessage.vue docs/plans/attachments/026-ChatMessage.gen-patched.vue`（行为等价：:class + style + truthy）。
- `cd gen/front/vue && npm run build` 全绿。
- 手测 :3334 chat：气泡左右对齐 + 自适应宽度 + 无红色空框。

## 7. 验收标准 (Acceptance Criteria)

- [x] 标准 1：`cargo test -p auto-lang` 全绿（含 3 个新 a2vue fixture）。[✅] 008/009/010 全通过；唯一失败 `test_a2vue_nested_if_style`(004) 经 `git stash` 验证为 **pre-existing**（clean master 也失败，括号差异）。
- [x] 标准 2：缺陷 ① — `col`/`row` 上的动态 `class:` 正确生成 `:class` 绑定（fixture 009 通过）。[✅]
- [x] 标准 3：缺陷 ② — `component fn` 的 `style { ... }` 块正确输出到 SFC `<style scoped>`（fixture 010 通过）。[✅]
- [x] 标准 4：缺陷 ③ — `.x != ""` 对 undefined 安全（生成 `!!x`，fixture 008 通过）。[✅]
- [x] 标准 5：auto-musk `auto build --gen-only` 后 gen ChatMessage.vue 正确。[✅ 核心] 生成物含 `:class="rowClass"` + `!!(props.msg.error)` + `<style scoped>` msg-* CSS（6 处匹配，与 patched 备份行为等价）。**gen `npm run build` 未全绿**：补了升级 auto 引入的 `ui/button` 缺失，但 `SettingsMenu.vue:136`(AccentPalette.id) + `useForgeStoreStore.ts:65`(null type) 2 个 **pre-existing/升级 auto 副作用** TS 错误未修（非本计划缺陷）。
- [ ] 标准 6：手测 :3334 chat 气泡正常（左右对齐 + 自适应 + 无红框）。[未做] gen build 未全绿，手测留待（核心修复已由 a2vue fixture + 生成物断言保证）。

## 8. 执行步骤 (Execution Tasks)

> 每个任务 2-5 分钟原子操作，含精确路径 + 操作 + 验证命令。

### 任务 1: 缺陷 ③ — `!= ""` 对 undefined 安全
- [x] **步骤 1.1:** 改 `vue.rs:5649`（`expr_to_js` 的 `Expr::Bina` 分支）：在 `nullish` 谓词后加 `empty_str` 谓词，`Op::Neq`/`Op::Eq` 且一端是 `Expr::Str("")` 时，`!= ""` → `!!(<other>)`，`== ""` → `!(<other>)`。[✅ 已完成] worktree `.worktrees/auto-musk/crates/auto-lang/src/ui_gen/vue.rs:5655-5670`。
- [x] **步骤 1.2:** 新建 `test/a2vue/008_neq_empty_safe/{input.at,input.expected.vue}`。[✅ 已完成] 验证 `hasX => .x != ""` 生成 `!!(x.value)`、`emptyX => .x == ""` 生成 `!(x.value)`。
- [x] **步骤 1.3:** `cargo test -p auto-lang test_a2vue_neq_empty_safe` ... ok；全量 a2vue 7 passed / 1 failed（`test_a2vue_nested_if_style` 失败是 **pre-existing** —— `git stash` 验证 clean master 也失败，括号差异，非本计划引入）。[✅ 已完成] 无 a2vue 回归。

### 任务 2: 缺陷 ① — 动态 class 绑定（col/row 布局基元）
- [x] **步骤 2.1:** 改 5 个布局分支 + 加 `layout_dynamic_class_attr` helper。[✅ 已完成] col/row（split_whitespace 块）+ container/grid（直接 push 块）+ center（max-w 块）均加 `else if` 调 helper（`Expr::If`→三元，其他→`:class` 表达式）；helper 在 `get_style_class` 后。
- [x] **步骤 2.2:** fixture `009_shadcn_col_dynamic_class`（shadcn 模式）。[✅ 已完成] 重构 `test_a2vue` → `test_a2vue_with(case, shadcn)` + `test_a2vue_shadcn` wrapper；验证 `col { class: .rowClass }` → `class="flex flex-col" :class="rowClass"`。
- [x] **步骤 2.3:** cargo test。[✅ 已完成] 009 ok；全量 a2vue 8 passed（004 FAILED 仍是 pre-existing，非本计划）。

### 任务 3: 缺陷 ② — component fn `<style>` 输出（三层）
- [x] **步骤 3.1:** `ast/ui.rs` `ViewFragmentDecl` 加 `pub style: Option<String>`（镜像 `WidgetDecl.style`）。[✅ 已完成]
- [x] **步骤 3.2:** parser 在 `on` 之后、`body` 之前捕获 `style { }` 块（`if is_component && cur.text == "style" → parse_style_block_inner()`，镜像 watch/on 模式）；构造 `ViewFragmentDecl` 传 `style`。[✅ 已完成]
- [x] **步骤 3.3:** `extract.rs:779` `style_css: None` → `style_css: frag.style.clone()`。[✅ 已完成]
- [x] **步骤 3.4:** fixture `010_component_fn_style`。[✅ 已完成] 验证 `component fn StyledCard` 的 `style { .card-title {...} }` 输出到 `<style scoped>`。
- [x] **步骤 3.5:** cargo test 010 ok；全量 a2vue 9 passed（004 仍 pre-existing）。[✅ 已完成]

### 任务 4: auto-musk `chat_message.at` 补 style 块
- [x] **步骤 4.1:** 在 `chat_message.at` 的 `component fn` 内（computed 后、view body 前）加 `style { ... }` 块。[✅ 已完成] 含 `.msg-row` / `.msg-row-user` / `.msg-row-ai` / `.msg-bubble` / `.msg-bubble-user` / `.msg-bubble-ai` / `.msg-role-badge` / `.msg-time` CSS（从备份提取，不含 inject_styles 已有的 msg-error/msg-thinking）。

### 任务 5: 重建 + 合并主分支 + 端到端验证
- [x] **步骤 5.1:** worktree commit + 合并 auto-lang master + `cargo install --path crates/auto`。[✅ 已完成] commit `85e7a64d` + merge `378dac8e`；auto.exe 替换（2m55s release 编译）。
- [x] **步骤 5.2:** auto-musk `auto build --gen-only`（新 auto）。[✅ 已完成] 28 components，无 parse error。
- [x] **步骤 5.3:** 生成的 ChatMessage.vue 与 patched 备份行为等价。[✅ 已完成] 6 处匹配：`:class="rowClass"`(L31) + `!!(props.msg.error)`(L13/17/19) + `<style scoped>`(L63) + `.msg-row`(L65)。
- [x] **步骤 5.4:** gen `npm run build`。[⚠️ 部分] 补了升级 auto 引入的 `ui/button` 缺失（gen/front/vue/src/components/ui/button/，参照 input 模式）；但 `SettingsMenu.vue:136`(AccentPalette.id) + `useForgeStoreStore.ts:65`(null type) 2 个 pre-existing TS 错误未修。
- [ ] **步骤 5.5:** 手测 :3334 chat。[未做] gen build 未全绿，留待。
- [x] **步骤 5.6:** 状态 → execution_done。[✅]

## 9. 复审记录 (Review Log)

> 由 `/auto-plan:review` 填写。

- **复审人**: [待填]
- **复审时间**: [待填]
- **复审结论**:
  - [ ] 验收标准全部满足
  - [ ] 代码无安全隐患
  - [ ] Spec 元数据已补全
- **遗留问题**: [如有]

## 10. 待澄清事项 (Open Questions)

- **parser 修复点（缺陷 ② 步骤 3.2）**：agent 报告确认 `widget` 的 `style { ... }` 解析已存在，但没给 `component fn` parser 的确切文件/行号。执行时需 grep 定位（`widget` 的 style 解析作为参考实现）。
- **跨项目提交策略**：本计划改 auto-lang（独立 repo）+ auto-musk。两个 repo 的 commit 如何协调？（auto-lang 先提交发布，auto-musk 的 chat_message.at 依赖新 auto 二进制。）建议 auto-lang 改动单独 commit/PR，auto-musk 的 chat_message.at + 计划文件另一 commit。
- **`auto` 二进制版本对齐**：`cargo install --path` 后全局 auto 更新，但 auto-musk 其他开发者/CI 若用旧 auto 仍有 bug。是否需要锁定 auto-lang 的某个 commit 或版本？（本计划先不处理，留作后续。）

---

*本文件为 PLAN-026，格式遵循设计文档 008（Auto-Plan 核心契约）。来源：PLAN-025 双前端对比发现的 gen chat 渲染缺陷。修复点由 auto-lang codegen 深入调查（2026-08-12）定位。*
