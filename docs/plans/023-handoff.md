# Plan 023 交接文档 — 剩余逃生舱原生化执行指引

> **给执行 agent**：本文档是 Plan 023（view fn → 独立组件 codegen）剩余工作的执行指引。读完本文 + 023 主文档（`docs/plans/023-view-fn-component-codegen.md`）即可开始。
>
> **生成时间**：2026-08-11。**作者**：上一轮迁移 agent（已完成 8 个逃生舱）。
>
> **更新（2026-08-11 P3 完成轮）**：auto-lang §10.7/§10.4 修复后，**队列 A/B 全部完成（20/21 逃生舱原生化）**，仅剩 StreamingRenderer（队列 C，永久逃生舱 KNOWN-DEBT）。本文件的执行队列已全部落地，仅 §3.1（队列 D）留待下轮。

---

## 0. 一句话现状

Plan 023 的 P2（转译器能力，auto-lang 408 P1-P12）已完成；P3（逃生舱原生化试点）已迁移 **20/21** 个逃生舱。剩余 1 个（StreamingRenderer）为永久逃生舱（流式增量 JSON composable + 动态 component :is + v-bind 展开超 component fn 能力边界）。**§3.1（跨视图共用组件收敛）已解封**（component fn slot P11 已支持 + WikiNav 已原生），留待下轮做视觉 parity 验证。

---

## 1. 工具链前置（每次开工必做）

auto-musk 的 `auto build` 依赖 `~/.cargo/bin/auto.exe`（从 auto-lang master 构建）。auto-lang 有新提交后必须重建：

```bash
cd /d/autostack/auto-lang
git pull  # 确认含最新 408 修复
cargo build --release --bin auto  # ~2-3 分钟
powershell -Command "Copy-Item -Force target/release/auto.exe C:\Users\zhaop\.cargo\bin\auto.exe"
```

**注意**：`cargo install` 常因 auto.exe 被占用失败，用上述 `Copy-Item` 更可靠。若 git 报 `index.lock` 冲突，`powershell -Command "Remove-Item -Force .git/index.lock"`。

---

## 2. 已沉淀的迁移模式（速查）

**标准迁移流程**（每个逃生舱都走这套）：
1. 在 `src/front/` 新建 `<name>.at`，写 `component fn <Name>`
2. `.at` 里的复杂逻辑（字典/正则/宿主API/async）抽到 `forge_helpers.ts` 或对应 ts 的 fn，component fn 用 `use { fn: ... }` 引入
3. 改 `chats_view.at`（或对应 view）的 `use { component: X from "..." }` 声明为 `use { component: X }`（无 from，跨文件引用）
4. 若逃生舱被**其他逃生舱 .vue** import，改那些 .vue 的 import 为 `@/components/X.vue`（指向生成版）
5. `git rm` 删除逃生舱 .vue
6. 逃生舱的 scoped 样式转全局，追加到 `inject_styles.ts`（用 `!important` 覆盖 codegen 注入的 Tailwind class）
7. `auto build` 全绿 → vite transform 验证 → 提交

**8 个已迁移组件的模式参考**（遇到同类问题对照）：
| 模式 | 参考组件 | 关键点 |
|---|---|---|
| 纯展示 + fn | UserMessage, StreamingTable | use{fn} + computed + html: prop |
| 有状态 toggle + computed 互引 | ErrandCard, TaskPlanCard, GenericToolCard | model + on + computed .value（P9/P10 续已修） |
| 编排组件（多子组件） | ChatMessage | use{component} 混合 component fn + 逃生舱 + fn |
| 动态 inline style | AgentAvatar | P12 缺陷 9 修复后，style + class 共存 OK |
| emit 重交互 | ReportCard | msg→defineEmits + handler auto-emit + onclick.stop |
| 字典/映射 | TaskPlanCard | 放 forge_helpers fn（.at 无字典字面量） |

**关键 .at 语法约定**（踩过的坑，必读）：
- **component fn 块顺序**：`use → computed → msg → model → watch → on → view body`（严格，乱序报 "found X"）
- **template text 节点只用简单变量**：`text .x` ✅；`text .x + "y"` ❌（解析失败）；`text .obj.field` 有时失败（下划线字段名）→ 提 computed
- **行内 if 不能带属性 `{}`**：`text if .c { "▲" } else { "▼" } { style: "x" }` ❌ → 提 computed
- **动态 class 用行内 if 表达式**（非拼接）：`class: if .s == "a" { "x a" } else { "x" }` ✅ 生成 `:class`；`class: "x " + .s` ❌ 生成 `:style`（失效）
- **动态 inline style**：`style: "bg: " + .color` ✅（P12 后生成独立 `:style`）；变量 `style: .styleVar` 也 OK
- **可选 props**：.at 的 `name: str = ""`（默认值）解析通过但 defineProps 仍标必填 → 调用方必须传值（可传 `""`）
- **msg variant 不能和 lucide 图标重名**（如 `Download`）→ 改名
- **watch 语法**用 `->` 不是 `=>`：`watch { .path -> { ... } }`

---

## 3. 执行队列 A：§10.7 解封后的逃生舱（async 操作类）

> **前置**：auto-lang 408 §10.7（async handler + on/watch body props 访问）必须先修。修法见 auto-lang `docs/plans/408-*.md` §10.7。
>
> **解封后优先级**（由易到难）：

### A1. RawPreview（最简 async，已试过，回退过）
- **文件**：`src/front/components/RawPreview.vue`（63 行）→ `raw_preview.at`
- **阻塞**：§10.7（async loadRawFileText + on/watch body props 访问）
- **迁移要点**：正则判断文件类型 → `rawFileKind` fn（已写好，回退在 git）；async load + try/catch → `.Init` + `watch`（.at 支持 `.await` 后缀 + try/catch）；MarkdownRender → `use { component: MarkdownRender from "markstream-vue" }`
- **参考**：回退前的 `raw_preview.at` 设计（git 历史或 §3.5 记录）。loadRawFileText 已是 async fn，component fn 调 `.await`
- **特殊**：`rawFileKind` fn 已加到 `raw_upload.ts`（git checkout 回退了，要重新加）

### A2. WorkspaceSelector（async + lifecycle）
- **文件**：`src/front/components/WorkspaceSelector.vue`（208 行）
- **阻塞**：§10.7（async）+ lifecycle（onMounted fetch）
- **迁移要点**：读源码确认 async 调用点 + composable 依赖

### A3. RelayRunBox（async + composable，较复杂）
- **文件**：`src/front/components/RelayRunBox.vue`（189 行）
- **阻塞**：§10.7 + composable（useRelay/useEventRouter/useGateInbox）+ lifecycle
- **迁移要点**：subscribeToRun 日志流（SSE）+ resolveGate 审批交互。composable 用 §10.4 降级（fn 包装）。可能含多个 async 操作

### A4. SettingsMenu（最复杂 async）
- **文件**：`src/front/components/SettingsMenu.vue`（367 行，最大）
- **阻塞**：§10.7 + composable + lifecycle + hostAPI
- **迁移要点**：主题/强调色切换（useTheme/useAccentColor composable）+ 设置面板交互。建议放最后

---

## 4. 执行队列 B：emit 重交互逃生舱（emit 已就绪，P4/P10）

> **前置**：无（emit P4/P10 + model 已就绪）。这些组件的 emit 交互可原生表达，但部分还含 composable/async（标注）。

### B1. QuestionnaireCard（纯 emit，无 composable/async）⭐ 最易
- **文件**：`src/front/components/QuestionnaireCard.vue`（297 行，emit=2）
- **阻塞**：无（emit 已就绪）
- **迁移要点**：问卷渲染 + submit/cancel emit。参考 ReportCard（emit 模式）

### B2. GateCard（emit + lifecycle）
- **文件**：`src/front/components/GateCard.vue`（184 行，emit=3）
- **阻塞**：lifecycle（确认是 onMounted 还是别的）
- **迁移要点**：gate 审批交互（approve/reject/snooze emit）

### B3. MentionDropdown（emit + lifecycle）
- **文件**：`src/front/components/MentionDropdown.vue`（126 行，emit=2）
- **阻塞**：lifecycle
- **注意**：它引用 AgentAvatar（已原生），import 已改为 `@/components/AgentAvatar.vue`

### B4. SecretaryMessage（emit + composable + hostAPI）
- **文件**：`src/front/components/SecretaryMessage.vue`（218 行，emit=5）
- **阻塞**：composable（useGateInbox）+ hostAPI
- **注意**：引用 AgentAvatar（已原生）

### B5. MentionInput（emit + composable + v-html + regex，最复杂的输入组件）
- **文件**：`src/front/components/MentionInput.vue`（249 行，emit=3）
- **阻塞**：composable + v-html backdrop + regex
- **迁移要点**：@mention 输入 + v-html 高亮 backdrop + 键盘导航。v-html 用 `html:` prop。这是输入类最复杂组件，建议谨慎

### B6. WikiNav（emit 最多 + composable + async，知识库导航）
- **文件**：`src/front/components/WikiNav.vue`（268 行，emit=7）
- **阻塞**：composable + async + DropZone 拖拽
- **迁移要点**：双树（Raw + Wiki）+ 搜索 + DropZone 上传 + 折叠。逃生舱里交互最复杂之一。可能需要保留部分逃生舱（DropZone 拖拽上传）

---

## 5. 执行队列 C：流式增量 JSON（StreamingRenderer）

### C1. StreamingRenderer
- **文件**：`src/front/components/StreamingRenderer.vue`（66 行）
- **阻塞**：composable（useStreamingDocument，增量 JSON 解析）
- **特殊**：Plan 022 §4 风险表标记"流式/增量 JSON 难以纯原生"。这是响应式流式渲染，composable 的复杂用法。**可能永久保留逃生舱**（登记 KNOWN-DEBT）。StreamingTable（它的依赖）已原生。

---

## 6. 执行队列 D：§3.1 跨视图共用组件收敛（P5）

> **状态**：能力已就绪（emit P4 + slot P11 + model P4 + 动态 class）。**可立即推进，不依赖 §10.7。**
>
> **目标**（见 023 §3.1）：三视图二级导航 + 内容标题栏收敛为共用 `NavSidebar`（header + slot 列表 + 折叠态）+ `ContentHeader`（标题 + slot 操作区）。

### D1. 设计共用 NavSidebar + ContentHeader（component fn）
- 用 `component fn` + slot（P11）+ model（折叠态）+ emit（toggle）
- 三处 header 结构（见 `inject_styles.ts` 的 `.sidebar-header`/`.section-nav-header`/`.wiki-nav-header` 统一规则）
- 删除 inject_styles 中针对三个 header 的分散 `!important` 覆盖（验收标准）

### D2. 三视图接入
- `chats_view.at` / `specs_view.at` / `wiki_view.at` 各自用 `<NavSidebar>` + slot 注入列表差异
- wiki 的导航在逃生舱 WikiNav 里——要么 WikiNav 先原生化（B6），要么 §3.1 接受 wiki 那份结构性差异

**验收**（023 §3.1）：① 三视图用一个 component fn；② auto build 后视觉对齐；③ 样式单一真源，无 `!important` 兜底。

---

## 7. 收尾：P6 归档

全部逃生舱迁移完成后（或登记残留）：
1. 更新 `docs/plans/KNOWN-DEBT-AND-RISKS.md`（StreamingRenderer 等永久逃生舱）
2. 022 §8 后续项闭环
3. 用 `finish-plan` skill 归档 023

---

## 8. 已迁移清单（20 个，供参考）

| # | 组件 | .at 文件 | 验证能力 | commit |
|---|---|---|---|---|
| 1 | UserMessage | user_message.at | 纯展示 + html:prop + use{fn} | 4b3d6df |
| 2 | ErrandCard | errand_card.at | model toggle + computed 互引 | dff4c64 |
| 3 | TaskPlanCard | task_plan_card.at | 同上 + fn 绕过字典 | 9696041 |
| 4 | GenericToolCard | generic_tool_card.at | 同上 + JSON.stringify→fn | ddc825b |
| 5 | ChatMessage | chat_message.at | 编排（component fn + 逃生舱 + fn 混合） | a6e78d1 |
| 6 | StreamingTable | streaming_table.at | P7 动态索引 + P8 table 标签 | 75374d4 |
| 7 | AgentAvatar | agent_avatar.at | P12 缺陷 9（动态 style + class） | (P12 批) |
| 8 | ReportCard | report_card.at | P4/P10 emit 重交互 | (P12 批) |
| 9 | RawPreview | raw_preview.at | §10.7 async + props 前缀 | d598ced |
| 10 | WorkspaceSelector | workspace_selector.at | async lifecycle + 宿主全局 | ec5465d |
| 11 | RelayRunBox | relay_run_box.at | composable facade refs + async | 86f872f |
| 12 | SettingsMenu | settings_menu.at | 3 composable facade + async | 7fa6790 |
| 13 | QuestionnaireCard | questionnaire_card.at | emit 负载 + 动态键记录 + 受控输入 | c184dc8 |
| 14 | GateCard | gate_card.at | emit 负载 + watch init | 875815a |
| 15 | MentionDropdown | mention_dropdown.at | teleport + emit + 键盘状态上移 | bd93276 |
| 16 | SecretaryMessage | secretary_message.at | 小写 emit 负载 | c8b4163 |
| 17 | MentionInput | mention_input.at | v-html + v-model + keydown 修饰符 | f5c3c7b |
| 18 | WikiNav | wiki_nav.at | DropZone 拖拽 + 上传进度共享 ref | 00e3d12 |
| 19 | SecretaryMessageWrapper | secretary_message_wrapper.at | §10.4 facade refs | 7932e67 |
| 20 | SessionInfo | session_info.at | use store: in component fn | fc8fb0f |

**剩余逃生舱**（1 个，永久 KNOWN-DEBT）：StreamingRenderer（useStreamingDocument 需 Ref 参数 + 动态 component :is + v-bind 展开）。

---

## 9. forge_helpers.ts 已加的 fn（迁移过程产物）

迁移中为绕过 .at 表达力缺口，在 `forge_helpers.ts` 加了这些 fn（component fn 通过 `use { fn }` 引入）：
- `msgTimeLabel(createdAt)` — Date 格式化（ChatMessage）
- `taskPlanStatusLabel(status)` — 状态码→中文映射（TaskPlanCard）
- `toolArgsJson(tc)` — JSON.stringify 包装（GenericToolCard）
- `agentAvatarData(professionId, name, size)` — 颜色字典+hash+initials（AgentAvatar）
- `reportConfidenceClass(confidence)` — toLowerCase+默认值（ReportCard）

**后续迁移按需继续加 fn**（正则/字典/宿主API/async 都放 fn）。

---

## 10. 常见问题

**Q: auto build 报 Parse error "found X" / "Expected node name, got Dot"？**
A: 块顺序错（见 §2 语法约定）或 text 节点用了拼接/行内if带属性。对照已迁移组件修。

**Q: 生成的 SFC 有 TS 错误 `ComputedRef` 不匹配？**
A: computed 互相引用——P9/P10 续已修。若仍中招，检查是否嵌套 if 深层（§7.8 残留，已修）。

**Q: 动态 class 生成成 `:style` 失效？**
A: 用行内 if 表达式（`class: if .x=="a"{...}else{...}`），不要拼接。

**Q: 逃生舱被别的逃生舱 .vue import，怎么处理？**
A: 改那个 .vue 的 import 为 `@/components/X.vue`（参考 UserMessage→ChatMessage、AgentAvatar→MentionDropdown/SecretaryMessage）。

**Q: gen/ 目录清不掉（Device busy）？**
A: 有 vite dev server 在跑。`pkill -f vite` 后 `rm -rf gen`，或 `auto clean`。

---

## 附录：auto-lang 408 缺陷追踪（截至 2026-08-11）

| 缺陷 | 状态 | 解锁的 auto-musk 组件 |
|---|---|---|
| 1-8（P4-P10 续） | ✅ 全修 | 已迁移 8 个 |
| 9（动态 style，P12） | ✅ 修 | AgentAvatar ✅ |
| watch 块（P12 §10.2） | ✅ 修 | RawPreview ✅ |
| 宿主全局（P12 §10.3） | ✅ 修 | SessionInfo/RawPreview ✅ |
| composable facade ref（§10.4） | ✅ 修（正式） | SecretaryMessageWrapper/SessionInfo/RelayRunBox ✅ |
| **§10.7 async handler + props 访问** | ✅ **已修（`1d32f9a3`）** | 队列 A 全部 ✅（RawPreview/WorkspaceSelector/RelayRunBox/SettingsMenu） |

**§10.7 修复后剩余全部原生化**：队列 A 4 个 + 队列 B 6 个 + Wrapper/SessionInfo 2 个 = 12 个新迁移（累计 20/21）。仅 StreamingRenderer 保留为永久逃生舱。

### 本轮新发现的 codegen 缺口（2026-08-11，auto-musk 已绕过，auto-lang 待修）

| # | 缺口 | 现象 | auto-musk 绕过模式 |
|---|---|---|---|
| 1 | convert_condition `.len→.length` 双展开 | view 级 if 的 `.length` → `lengthgth`（`.length` 含子串 `.len`） | view-if 用 `.len`；computed/handler 用 `.length`（ts_adapter 仅方法形式转 `.len`，字段透传） |
| 2 | component fn 不支持 `expose` 块 | MentionDropdown 键盘导航 API（moveUp/currentId/hasItems）无法暴露 | 键盘状态上移父组件（MentionInput 逃生舱管理 index/filtered，dropdown 纯渲染） |
| 3 | handler body 连续多个 `try` 解析失败 | `.Init` 内两个 try → "Expected end of statement, got Try" | helper 内部 try/catch 吞错（不抛异常），.at 免 try |
| 4 | 动态键 v-model 不生成 | `value: .answers[q.id]`（Index 值）不触发 v-model 优化 → 单向 :value + 空 handler | 受控组件：`value:` + `oninput: .X(.key, $event)` + eventInputValue 提取 |
| 5 | msg 多参变体 emit 类型只取首参 | `SelectSingle(str,str)` → `[string]` 但 auto-emit 传 2 参 | msg 变体不声明 payload，on-block 参数驱动 emit 类型（fallback `[any,...]`） |
| 6 | `.at` 无可选 props | `name: str = ""` 解析通过但 defineProps 仍必填 | 调用方传 `""` + helper/computed 兜底 |
| 7 | `link` 标签 → shadcn router-link | 非原生 `<a>`（href/style/text 错乱 + 引 vue-router） | v-html 兜底（rawDownloadHtml helper，含 path 转义） |
| 8 | view 级 if 不支持索引/多参 fn 调用 | `if gateExpanded(.a, .b)` / `if .expanded[sid]` 解析失败 | helper 扁平化（gateWithExpanded → `_expanded` 字段） |
| 9 | `teleport (to:)` 括号形式冲突 | ident+LParen 被当 fn-call primary prop | `teleport { to: "body", ... }` 花括号 props 形式 |
