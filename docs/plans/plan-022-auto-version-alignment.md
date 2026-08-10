# Plan 022: auto-musk Auto 版前端对齐原生 Vue 版

> **目标**: 将 `.at` 源码生成的 Vue 前端（Auto 版）与原生手写 Vue 前端（`web/`）在视觉和功能上对齐，聚焦 **chat / specs / wiki** 三个视图。
> **分支**: `main`（auto-musk）+ `plan407/a2vue-icon-text-expr`（auto-lang，已合并 master）
> **状态（2026-08-10）**: 主体完成，持续微调中。

---

## 一、背景

auto-musk 有两个前端：
- **原版**（`web/`）：原生手写 Vue SFC，完整的样式系统（`--af-*` 语义变量 + `useTheme`/`useAccentColor` 动态切换 + 每组件 scoped CSS）
- **Auto 版**（`gen/front/vue/`）：由 `.at` 源码（`src/front/*.at`）通过 auto-lang codegen 生成

Auto 版的样式体系与原版完全不同（shadcn 默认变量 + 空的 `<style>` + `inject_styles.ts` 运行时注入），导致视觉差异显著。本计划系统性对齐两者。

---

## 二、已完成的需求

### A. 转译器增强（auto-lang Plan 407，已合并 master）

| 需求 | 方案 | 提交 |
|---|---|---|
| `.at` text 节点不支持 `t()` i18n 调用 | parser.rs `has_ident_field_primary` peek 加 `LParen` 检测（3 行），让 `text t("nav.chat")` 解析为 `Expr::Call` | auto-lang `plan407` |
| lucide 图标组件不能作子节点 | 探索发现 parser/codegen **本来已支持**（只需 `use { component }` 声明），加 golden case 006 验证 | auto-lang `plan407` |
| golden case 005/006 | 005: text fn call → `{{ t('key') }}`；006: icon child + text 共存 | auto-lang `plan407` |
| auto-musk .at 源码回流 | app.at/chats_view.at 用 `t()` / lucide 替代硬编码，消除 KNOWN-DEBT | `c2ee06b` |

### B. 全局主题与样式对齐

| 需求 | 方案 | 提交 |
|---|---|---|
| primary 色完全不同（原版紫色 vs Auto 版近黑） | inject_styles 覆盖 `:root` / `.dark` 变量为原版 theme.css 值 + `--af-*` 别名层 | `07d15d5` |
| 默认主题模式（原版深色 vs Auto 版永远浅色） | 默认浅色 + useTheme 支持 light/dark/auto 切换 | `496f456` |
| 主题色/外观/语言无法切换 | useTheme + useAccentColor composable（移植原版，5 色板） | `496f456` |
| SettingsMenu 缺失（左下角齿轮按钮） | SettingsMenu.vue 逃生舱组件（accent/theme/language 三分区） | `95854a0` |
| SettingsMenu 弹窗右对齐超出左边界 | `right: 0` → `left: 0` | `5931908` |
| 滚动条样式 | 6px 窄滚动条 + 半透明 thumb（原版样式） | `07d15d5` |
| Noto Sans SC 字体 | @import Google Fonts + body font-family + 标题统一 bold | `8ac53c6` |

### C. 布局与导航

| 需求 | 方案 | 提交 |
|---|---|---|
| 三列背景无层次 | 三列渐变：view-rail secondary 浅灰 → sub-nav card 白 → main 白 | `99bc62c` |
| header 无分隔线 / 高度不一致 | 统一 height:48px + border-bottom + 负 margin 抵消 padding | `99bc62c` `2ea6508` `dc17491` |
| 标题栏 v0.1.0 换行 | col → row（Auto Musk + v0.1.0 水平排列） | `5931908` |
| app-header border 左右空隙 | margin-left/right: -0.75rem 抵消父容器 px-3 | `dc17491` |
| 一级导航无图标 | MessageSquare/Scroll/BookOpen lucide 图标作子节点 | `c80009b` |
| 一级导航无 active 状态 | 条件 style `if current_view == "chats" { "rail-tab active" }` | `058725e` |
| 二级导航 active 不明显 | session-item.active 从灰底 → primary 8% 透明紫底 + 紫色字 | `058725e` |
| 标题字体不统一 | Noto Sans SC 1rem bold 不大写（覆盖分散的 uppercase/0.85rem） | `8ac53c6` |
| WorkspaceSelector 缺失 | WorkspaceSelector.vue 逃生舱（当前工作区指示器 + 切换列表） | `54888d9` |
| api.at 无 workspace 声明 | 补 4 个 #[api] 端点 + 6 个类型 | `fc666df` |

### D. Chat 视图

| 需求 | 方案 | 提交 |
|---|---|---|
| toolbar emoji 图标 | Plus/Trash2 lucide 图标替代 emoji | `e447ed0` |
| 会话删除按钮不在 .at 源里 | chats_view.at 加 DeleteSession msg/handler + span.session-delete-btn | `c50e661` |
| 输入框样式混乱（双层背景/固定宽度） | inject_styles 覆盖：flex 自适应 + backdrop 透明 + textarea block 100% | `2f1b893` `3707994` |
| 输入框白底 + border-top | .chats-input-bar 透明 + 无 border-top + min-height 80px | `3707994` |
| Send 按钮样式不对 | 圆形 50% + linear-gradient 渐变 + lucide Send 图标 | `3707994` |
| 输入框聚焦多余方形框 | textarea :focus box-shadow !important 清零 | `527c110` |
| 消息时间 Invalid Date | `msg.timestamp` → `msg.created_at`（后端字段名） | `450e910` |
| store 类型不匹配（7 个 TS 错误） | forge_store.at 改回 resp.sessions/resp.session（后端有包装层） | `5931908` |
| 会话列表不显示（上轮修复方向搞反） | 同上，恢复 .sessions 包装层 | `5931908` |
| 默认不显示第一个 chat 内容 | LoadSessionList handler 加自动选中第一个逻辑 | `dc17491` |
| ChatMessage view fn 内联失效 | 改为独立逃生舱 .vue 组件（路径 A，零转译器改动） | `3f42164` |
| chat_message.at parse 失败 | 加 ChatMessageHost widget 外壳（parser 不支持顶层 use{}+view fn） | `4ee5cc7` |
| chat 标题显示 ID 而非 title | session_id → t('chat.title')（固定"聊天"） | `0abeb30` |
| info 按钮无弹框 | SessionInfo.vue 逃生舱（Chat ID 复制 + 消息数 + token 消耗） | `b7f63bb` |
| SessionInfo chat id 不显示 | store 加 reactive() 包装（ref 自动解包） | `f6e11ee` |
| 搜索框缺失 | chats-header 加 Search 图标 + input + i18n placeholder | `9761efe` |
| 用户消息不即时显示 | startForgeStream 乐观 push user 消息 | `154d45e` |

### E. Specs 视图

| 需求 | 方案 | 提交 |
|---|---|---|
| 硬编码英文文本 | specs_view.at 8 处 text t() 化 | `5cc23c6` |

### F. Wiki 视图

| 需求 | 方案 | 提交 |
|---|---|---|
| 硬编码英文文本 | wiki_view.at 11 处 text/placeholder t() 化 | `5cc23c6` |
| 二级导航与原版差距大 | WikiNav.vue 逃生舱（双树 Raw+Wiki + 搜索 + 图标 + 折叠） | `858ba5c` |
| wiki_store 不加载 raw_tree | Init 加 LoadRawTree() | `858ba5c` |

### G. i18n

| 需求 | 方案 | 提交 |
|---|---|---|
| i18n 接入（main.ts 已注册但未使用） | app.at/chats_view.at 用 composable useT + t() 表达式 | `a4e4774` |
| vue-i18n `@` 符号冲突（8 console error） | locale 值 `@` → `{'@'}` 转义 | `3144c31` |
| i18n key 覆盖率低（15→34 key） | 扩充 common/specs/wiki/settings/chat.* key | `5cc23c6` `95854a0` |

---

## 三、暂缓 / 待办

### 暂缓（用户决定）

| # | 需求 | 原因 |
|---|---|---|
| specs 示例文档 | musk-demo 的 specs.json 是空骨架（7 个 section 无条目）。需填充种子数据。 | 用户暂缓 |
| 导航视图不完整（3/9） | 原版 9 个导航区（explorer/relay/agents/professions/skills/apis），Auto 版只要 chat/specs/wiki。 | 用户确认 3 个足够 |
| Plan 400（api_gen.rs a2r body 转译） | auto-musk api.at 全是 `return None` 桩函数，无实际需求。 | 后端手写 Rust，不走 .at→Rust 路径 |

### 已知技术债

| # | 需求 | 说明 |
|---|---|---|
| api.ts 类型声明不匹配 | `chats_list_sessions(): Promise<ForgeSessionSummary[]>` 但后端返回 `{sessions:[...]}`。vue-tsc 有 6 个 TS 错误（dev 模式不影响运行，但 `pnpm build` 会失败）。需修正 api.at 的返回类型声明加包装层。 |
| 搜索框输入绑定 | chats-header 搜索框生成的是 `:value`（单向绑定），无 oninput handler。输入不会过滤消息。需加 oninput + computed filteredMessages。 |
| view fn → 独立组件 codegen | auto-lang view fn 设计为"内联展开宏"（Plan 367），ChatMessage 因 `use{component}` 引用跳过内联。当前用逃生舱 .vue 组件绕过。后续可在 auto-lang 做 view fn → 独立组件合成的 codegen 路径。 |
| WikiNav 简化版 | 原版用 TreeView 递归组件（后端树节点有 children 层级），Auto 版用扁平列表。如果后端树有层级需补递归渲染。 |
| WikiNav raw 区功能 | 原版 raw 区有 DropZone 拖拽上传 + 新建文件夹。Auto 版未实现。 |
| 搜索框 Ctrl+Shift+S 快捷键 | 原版有，Auto 版暂未做。 |
| useForgeMode（GSD/Check） | 原版 SettingsMenu 有 GSD/Check 模式切换，Auto 版去掉（后端无对应逻辑）。 |
| AutoOS 设置链接 | 原版 SettingsMenu 有 AutoOS 深链按钮，Auto 版去掉（后端无 /api/settings-link 端点）。 |

---

## 四、架构决策记录

### 1. 样式注入机制
- **原版**：每组件 `<style scoped>` + `--af-*` 语义变量
- **Auto 版**：`inject_styles.ts` 运行时注入全局 CSS（因为 codegen 生成的 `<style>` 为空）
- **主题变量**：inject_styles 开头覆盖 `:root` / `.dark`（对齐原版 theme.css）+ `--af-*` 别名层

### 2. 主题切换
- **useTheme.ts** / **useAccentColor.ts** 作为逃生舱 composable（和 useT.ts 同级）
- codegen 机制：`composable: useTheme from "..."` → `const theme = useTheme()`
- useTheme 的 onMounted(init) 自动执行（读 localStorage + apply）

### 3. 组件策略
- 复杂交互组件（SettingsMenu / SessionInfo / WikiNav / ChatMessage）用**逃生舱 .vue 组件**
- 简单展示用 .at 原生表达（lucide 图标作子节点 / text t() 表达式）
- 逃生舱组件通过 `use { component: X from "src/front/components/X.vue" }` 声明

### 4. .at = single source of truth
- 所有改动优先在 .at 源文件（app.at / chats_view.at / wiki_view.at / forge_store.at 等）
- 重跑 codegen 生成的 .vue 不需要手动修改
- gen/ 下文件被 gitignore，不提交
