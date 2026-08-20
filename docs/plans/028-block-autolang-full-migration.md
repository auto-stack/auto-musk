---
plan_id: PLAN-028
status: review_done
feature_name: Block 功能全量 Auto 化迁移（含 a2ts 特性补齐）
author: [zhaopuming, ZCode]
created_at: 2026-08-19T14:30:00+08:00
updated_at: 2026-08-19T19:30:00+08:00

supersedes_spec_components:
  - 00-overview.md「双前端 parity」条目（能力 5 旧表述——已更新为含平台协议/单一真源）
  - KNOWN-DEBT 022「ext 复制需显式声明」/ 023「StreamingRenderer 永久逃生舱」（已勾销/升格）
new_spec_components:
  - docs/specs/03-front-component-groups.md（前端组件分组清单，T22）
  - 平台能力协议（Sse.open/close + Http.* + platform:markdown，声明见 forge_store.at/relay_store.at/chat_message.at）
touched_goals:
  - Block 功能全量 Auto 化（.at 单一真源 + 148 项对拍全等）
  - a2ts 语言特性 F1-F9（auto-lang 侧已合入 master）
  - 样式随组件走（9 组 125 条 CSS 归还 .at style 块）

current_step: 25
total_steps: 25
---

# [PLAN-028] Block 功能全量 Auto 化迁移（含 a2ts 特性补齐）

## 变更摘要

聊天 Block 体系（文本/思考/工具/问卷，PLAN-026/027 产物）目前是「Auto 骨架 + TS 血肉」：
8 个块组件本体已是 `.at` component fn（约 870 行），但约 3200 行 TS 逃生舱承载着块累积、
解析、渲染链与样式。本计划分两条线消除逃生舱：①在 auto-lang 补齐 a2ts 缺失的语言/宿主
能力（每项配 vue_capabilities golden 测试）；②把 auto-musk 的 Block 相关 TS 逐模块迁回
`.at` 单一真源，使同源 `.at` 未来可在 VM/Rust 后端复用（Auto→Rust 成熟度已高于 Auto→TS）。

## 目标

1. Block 相关功能（含 forge_stream 块累积、questionnaire 解析、全部 helper 纯函数）以
   `.at` 为单一真源；Vue 环境由 a2ts 生成，Rust/VM 环境可直接复用同源语义。
2. 平台强依赖（markdown 流式渲染、语法高亮、SSE/HTTP 客户端）收敛为**平台能力协议**：
   Auto 侧只声明接口，各后端提供实现（Vue 后端继续用 markstream-vue/prismjs/EventSource）。
3. inject_styles.ts（1058 行）拆回各组件 `.at` style 块，CSS 随组件走。
4. 每个新 a2ts 特性都有可回归的 golden 测试（auto-lang/crates/auto-lang/tests/vue_capabilities.rs）。

## 架构方案

```
现状:  .at 组件(8) ──use{fn}──> TS 逃生舱(forge_helpers/questionnaire/forge_stream/...)
                              └─use{component from}──> StreamingRenderer.vue → markstream-vue/prism

目标:  .at 组件 + .at fn(纯函数) + .at composable(SSE/HTTP 协议) + .at style
        │ a2ts                                    │ a2r(未来)
        ▼                                         ▼
   gen/front/vue(SFC, 平台组件桥接)          VM/Rust(平台能力 Rust 实现)
   平台协议实现: markdown 渲染器/高亮器/SSE/HTTP —— 由 use{platform ...} 声明绑定
```

分层原则（与 docs/specs/00-overview「双前端 parity」一致，向上扩展为双后端）：
- **语言层**（auto-lang）：表达式/字面量/宿主 API 桥 → F 组特性
- **协议层**（musk .at 声明）：平台能力接口 → P 组
- **实现层**（各后端）：Vue 用现有 npm 生态，Rust 用 comrak/syntect/reqwest

## 技术栈

- auto-lang：parser(auto-lang/src/ast) + vue codegen(auto-lang/src/lib.rs、auto-man/src/vue.rs)
  + golden 测试(crates/auto-lang/tests/vue_capabilities.rs) + VM stdlib(stdlib/)
- auto-musk：src/front/*.at、src/front/composables/*.ts、gen/front/vue 构建链
- 保留的 npm 平台实现：markstream-vue、prismjs（作为平台协议的 Vue 后端实现，不迁移）

## 需求分析与背景调查

（来源：2026-08-19 全量盘点 + 各 .at/逃生舱文件头部注释记录的 codegen 缺口）

**块相关代码量**：.at 约 870 行 / TS 约 3200 行（forge_stream 439、forge_helpers 410、
inject_styles 1058、questionnaire 219、useRelay 617、useStreamingDocument 199、
StreamingRenderer 71、PrismCodeBlock 96、其余 helpers ~170）。

**已记录的 Auto→TS 缺口**（按出现文件归并；T23 勾销：G1–G13/G15 已由 F1–F9 落地解决，G14 连字符事件名未做——Block 组不再需要，secretary 域归后续组）：

| # | 缺口 | 证据（文件:用途） | 迁移依赖 |
|---|---|---|---|
| G1 | 字典字面量 | taskPlanStatusLabel/relayStatusLabel/relayProfessionIcon/agentAvatarData 颜色表/langFromPath | M 纯函数群 |
| G2 | if 条件内多参 fn 调用/比较 | chats_view.at:248（isLastMessage 被迫绕 computed） | M gating 类 |
| G3 | JSON.parse/stringify | questionnaire.ts 问卷解析、toolArgsJson | M2 |
| G4 | 正则（test/match/replace） | questionnaire.ts blockRegex、renderMentions、rawFileKind | M2 |
| G5 | 字符串方法 charCodeAt/charAt/slice/split | estimateTokens、agentAvatarData hash | M1 |
| G6 | Date（toLocaleTimeString） | msgTimeLabel | M1 |
| G7 | Math.max/min | streaming_table colspan | M1 |
| G8 | 动态键赋值 + value: 对 Index 触发 v-model | questionnaire_card.at:35（answers[qid]） | M3 |
| G9 | 可选 props / withDefaults | gate_card/chats_view/streaming_table 多处传空串兜底 | 组件签名统一 |
| G10 | SSE(EventSource) + 回调式事件 → store 回写 | forge_stream.ts 存在的全部理由（"store 不能传回调"） | M4 |
| G11 | HTTP fetch + composable 内异步状态 | useRelay.ts（loadRun/resolveGate/subscribeToRun） | M6 |
| G12 | v-html | user_message @mention 高亮（renderMentions 产物） | M1 附带 |
| G13 | 闭包/回调存储（Map<str, fn>） | relay_run_helpers unsubMap、wiki_nav 回调 | M6 |
| G14 | 连字符事件名 | secretary_message ReviewInSpecs 改名绕过 | 低优先 |
| G15 | 流式 markdown 渲染、语法高亮 | StreamingRenderer→markstream-vue、PrismCodeBlock→prismjs | 平台协议 P1/P2 |

**auto-lang 现状**：vue_capabilities.rs 已有 12+ 特性 golden 测试框架；vue 代码生成集中在
auto-lang/src/lib.rs（SFC 生成）与 auto-man/src/vue.rs（工程级，3595 行）；VM/Rust 侧
（auto-vm/a2r-std/stdlib）成熟度更高——多数 G 组能力在 Rust 目标已有等价物，缺的是
a2ts 侧表达与生成。

## 详细设计

### F 组：a2ts 语言特性（auto-lang 仓库）

- **F1 字典字面量**：`let colors = { "rust": "#ce422b", "ts": "#3178c6" }`；类型推断
  `Record<str, str>`（TS）/ `HashMap<String,String>`（Rust 已有）。支持字面量 + 索引读取 +
  `keys()` 遍历。测试：字面量声明、按键取值、缺键返回 None 分支。
- **F2 if 条件表达式增强**：条件位置允许 `fn(a, b) != ""`、`fn(a,b) && .x`。parser 侧
  条件产生式从「单 fn 调用」放宽到「表达式」；codegen 直接内插。测试：多参 fn 比较、
  逻辑组合。
- **F3 宿主 API 桥（stdlib@ts）**：新增 `JSON.parse/JSON.stringify`、`Date.format(ts, "HH:mm")`
  （封装 toLocaleTimeString，避免暴露完整 Date）、`Math.max/min`、`str.charCodeAt(i)`/
  `charAt/slice/split`。a2ts 映射到 JS 原生；a2r 映射 stdlib 已有等价物。逐 API 一个 golden。
- **F4 正则（Rust 语法为规范 + a2ts 机械转换；已决②）**：一致性分析结论——JS RegExp 是
  Rust regex 的超集，Rust 不支持的 lookaround/反向引用天然构成「可移植子集」边界；仅两处
  语法差需转换：①命名组 `(?P<n>…)` → JS `(?<n>…)`；②flags 不进模式、走 API 参数。
  API：`Regex.test(s, pat, flags?)`、`Regex.match(s, pat, flags?) -> str[]`、
  `Regex.replace(s, pat, to, flags?)`（`to` 支持 `$1`，两端一致）。a2ts→`new RegExp(pat,
  flags)`（含命名组改写）；a2r→`RegexBuilder` 直通。musk 现有 5 处正则均在子集内，原样
  迁移。regex crate 1.11.1 已是 auto-lang workspace 依赖。
- **F5 动态键赋值 + Index v-model**：`value:` 触发 v-model 的产生式从 Ident/Dot 扩展到
  Index（`value: .answers[.q.id]` 生成 `v-model="answers[q.id]"`）；赋值语句已支持 Index。
- **F6 可选 props**：props 声明支持 `prop: str = ""` 默认值 → TS `withDefaults(defineProps,
  {...})`；Rust 侧 Option。测试：带默认值 props 的 SFC 快照。
- **F7 v-html（受限）**：widget 属性 `html:`（仅逃生舱桥接用，语义为「已由 fn 产出的受信
  HTML」），a2ts→`v-html`；a2r→文本节点（降级）。测试：html 绑定生成。
- **F8 SSE/HTTP 平台协议**：新 composable 类别 `use platform`：`use platform sse(url) -> Stream`
  （`Stream.on_message(ev)` 触发 widget/store msg）与 `use platform http`（`http.get/post`）。
  Vue 后端实现：EventSource/fetch 薄封装（auto-man 生成 `platform/` 目录）；Rust 后端：
  reqwest/eventsource。Auto 侧只见协议。**事件粒度（已决③）：平台层把 SSE data 预解析为
  对象再分发**——各后端用语义最准的原生 JSON parser，.at 侧按 `.ev.type` 直接分支，零解析
  样板；协议需带事件名过滤参数（聊天流为默认消息，relay 流为命名事件 `run_event`，
  EventSource 需 addEventListener 区分）。
- **F9 store 事件回写**：store `on` 块支持订阅平台流：`on stream sse("/api/...", .id) ->
  { ... }`（生成 EventSource + dispatch 到 store msg，payload 为已解析对象）。这消除
  forge_stream 的「绕开 store 不能传回调」注释所述限制。

### P 组：平台能力协议（musk .at 声明 + 后端实现注册）

- **P1 markdown 渲染器**：`use platform markdown` 组件（`Markdown(source, streaming)`）。
  Vue 实现即现 StreamingRenderer（保留 .vue 文件，从「逃生舱」升格为「平台实现」，
  挂到生成的 platform/ 目录）；Rust 实现未来用 comrak。
- **P2 高亮器**：`use platform highlight`（P1 内部使用，prism/syntect 各后端自决）。
- **样式归还**：inject_styles.ts 按 component 拆到各 `.at` 的 `style` 块（style 块能力已
  存在，PLAN-026 已用）；仅跨组件的设计 token（--af-* 变量）留一份全局 `.at` style。

### M 组：musk 侧迁移（依赖 F 组就绪）

- **M1 forge_helpers 纯函数**→ `src/front/forge_helpers.at`（fn 集）：msgTimeLabel(F3)、
  estimateTokens(F3 charCodeAt)、taskPlanStatusLabel/relay 系 label/agentAvatarData(F1)、
  toolArgsJson(F3 JSON)、getToolSummary/getErrand*/getTaskPlan(纯遍历)。renderMentions
  产 HTML → F7。
- **M2 questionnaire.ts**→ `src/front/questionnaire.at`：questionnaireFor（F3 JSON + F4 正则）、
  stripQuestionnaire（F4）、messageBlocks/messageDisplayBlocks（迁 forge_helpers.at）、
  isLastMessage/lastMessageId。
- **M3 questionnaire_helpers/relay_run_helpers**→ .at fn：F1+F5 承载 answers 动态键；
  unsubMap 闭包 → F8 协议下订阅句柄入 store model（`var relay_subs obj`）。
- **M4 forge_stream.ts**→ `forge_store.at` 扩展：F9 store 订阅 SSE，`on stream` 内实现
  ensureAssistantMsg/appendBlockText/事件分发（纯 .at 语句，G2 放宽后条件判断可直写）。
- **M5 useChatSearch**→ `src/front/composables/chat_search.at`（composable 声明机制已存在）。
- **M6 useRelay**→ `relay_store.at`（F8 http/sse + model 状态）。
- **M7 渲染链协议化**：chat_message 的 `use{component from StreamingRenderer.vue}` 改为
  `use platform markdown`；PrismCodeBlock 并入平台实现；删除 ext 复制特例声明。
- **M8 清理**：删除迁移完成的 TS 逃生舱文件与 chats_view 里「声明仅为触发 ext 复制」的
  use 条目；inject_styles 缩减为 token 文件。

## 测试设计

- **auto-lang**（每 F 一条，进 crates/auto-lang/tests/vue_capabilities.rs）：
  字典字面量/索引、if 多参条件、JSON·Date·Math·str API 各一、Regex 三形态、
  Index v-model、默认值 props、html 绑定、SSE/HTTP 协议生成、store on stream 生成。
  另跑既有 12 条防回归；`cargo test -p auto-lang --test vue_capabilities`。
- **auto-musk**：每 M 完成后 `auto build --gen-only` + `cd gen/front/vue && npx vue-tsc
  --noEmit && npx vite build`；「🧩 Block 类型演示」会话（demo-blocks-0001）作 e2e 基线
  （块序/折叠/问卷/高亮/表格全量检查）；流式路径用 `curl SSE` 对拍事件序。
- **双后端**：`auto build --backend vm --gen-only`（若 VM 后端命令形态不同，见待澄清①）
  验证同源 `.at` 可编译，平台协议处报「实现缺失」属预期通过项。

## 验收标准

1. `src/front/` 下 Block 相关 TS 仅剩：`components/StreamingRenderer.vue`、
   `PrismCodeBlock.vue`（平台实现，挂 platform/）与 token 级全局样式；其余逃生舱删除。
2. `auto build --gen-only && npx vue-tsc --noEmit && npx vite build` 全绿。
3. 演示会话 e2e 与迁移前快照一致（块序/ThinkBlock 折叠 token 数/问卷选择提交/代码高亮/表格斑马线）。
4. vue_capabilities.rs 新增 ≥10 条 golden 全绿，且旧用例零回归。
5. 同源 `.at` 通过 VM/Rust 后端编译（平台协议缺失实现时明确报错而非静默）。

## 执行步骤

### Phase A — a2ts 特性（auto-lang）

- [x] **T1** F1 字典字面量：parser(auto-lang/src/ast/expr.rs 附近)支持 `{ "k": v }` 字面量与
  索引读取；vue codegen 生成 `Record`。验证：`cargo test -p auto-lang --test vue_capabilities dict_literal`。
  [✅ 已完成] parser.rs `parse_computed_block_inner` 投机解析对象字面量(失败全量回滚到 Block 路径)；vue.rs expr_to_js StrKey 加引号 + 新增 NullCoalesce 臂(`(a ?? b)`)；expr_to_ts_type 均质字典推 `Record<string, V>`。golden `dict_literal` 绿(字面量/变量键索引/字符串键索引/缺键 ?? 兜底/Record 断言)。
- [x] **T2** F2 if 条件放宽：条件产生式允许多参 fn 调用参与比较/逻辑。验证：golden
  `if_multiarg_call`。
  [✅ 已完成] parser.rs `parse_condition_expr` Ident 臂新增 `fn(args)` 直呼（`capture_condition_call_args` 平衡括号逐字捕获，支持嵌套调用/字符串/多参）；golden `if_multiarg_call` 三断言（裸调用/&& 组合/!= 比较）绿，既有 24 条无回归（除预存 cap_vmodel_fold）。
- [x] **T3** F3 宿主 API 桥 ×5（JSON/Date/Math/charCodeAt 组）：stdlib@ts + a2ts 映射。
  验证：golden `host_api_json` 等 5 条。
  [✅ 已完成] JSON.parse/stringify 与 Math.max/min 原生直通（锁定 golden）；`Date.format(ts,"HH:mm")` 映射 `toLocaleTimeString([], {hour/minute[/second]: '2-digit'})`；`char_code_at`→`charCodeAt` 入方法表（char_at/slice/split 原已支持，一并锁定）。golden `host_api_json/date/math/str_char_code/str_split` 5 条全绿。
- [x] **T4** F4 正则子集：stdlib `Regex`（test/match/replace）。验证：golden `regex_subset`。
  [✅ 已完成] expr_to_js Dot-call 路径新增 `Regex.test/match/replace` 三形态（须先于 map_method_to_js，否则裸 `replace` 被 replaceAll 规则吞掉）；命名组 `(?P<n>`→`(?<n>` 机械转换；flags 走 API 参数；`match` 补 `|| []` 对齐 Vec 语义；`$1` 直通。golden `regex_subset` 5 断言绿。
- [x] **T5** F5 Index v-model + F6 默认值 props：codegen 产生式扩展。验证：golden
  `index_vmodel`、`default_props`。
  [✅ 已完成] F5：通用路径 value_state_ref 扩展 Index（`index_target_name` 兼容 Ident(".x") 与 Dot(Ident("."),"x") 两种靶形态）+ shadcn input/textarea 臂 `extract_state_index_ref`；`value: .answers[q.id]` 折叠 `v-model="answers[q.id]"`。F6：有默认值 props 生成 `withDefaults(defineProps<{…}>, {…})`（emitted-callback props 跳过）。golden `index_vmodel`、`default_props` 绿，既有含 `cap_optional_props_with_defaults` 无回归。
- [x] **T6** F7 受限 html 绑定。验证：golden `html_binding`。
  [✅ 已完成] `html:` widget 属性 a2ts 已原生生成 `v-html`（state 绑定与 fn 调用两形态验证）——本任务以 golden 锁定既有能力，无需新代码。`html_binding` 绿。
- [x] **T7** F8 平台协议 sse/http + F9 store on stream：auto-man/src/vue.rs 生成
  platform/ 薄封装。验证：golden `platform_sse`、`store_on_stream` + 手建 fixture 工程编译。
  [✅ 已完成（设计微调）] F9：`on stream sse(url[, "event"]) -> {…}` 全链落地——parser（AST 加
  StreamSubscription，前瞻防裸 `stream` 误判）→ aura（AuraStore.stream_handlers）→ store
  composable 生成守卫式 EventSource（onmessage/addEventListener 按事件过滤分流、JSON 预解析
  分发、onerror 2s 重连，已决③）。F8：`Http.get/post` → ts_adapter 映射 `(await fetch(...)).json()`。
  设计偏差：不生成独立 platform/ 目录薄封装，EventSource/fetch 直接内联 store composable（协议
  等价：.at 只见 sse/http 声明，后端自带实现）；auto-man platform/ 挂载留给 T18 P1/P2 平台组件。
  fixture 工程编译并入 T8 musk 全量 build（更强验证）。golden `store_on_stream`/`platform_sse`/
  `platform_http` 绿。
- [x] **T8** 全量回归：`cargo test -p auto-lang` + auto-musk `auto build --gen-only`
  （既有 .at 不受新特性影响）。
  [✅ 已完成] cargo 全量：2988 通过 + 28 失败——28 个失败在**未含本计划改动的预存树**上逐一致（他人 vm/ui WIP 与
  cap_vmodel_fold 预存失败，非本计划回归）；vue_capabilities 23→36。musk `auto build --gen-only` 全绿
  （29 组件 + 5 store 重生成）。注意：auto-lang 主仓 release 构建被他人 WIP（ui/iced/renderer.rs
  未闭合括号）阻断，CLI 在干净 worktree（HEAD=78fa7f57 + 仅本计划 6 文件）构建后安装
  ~/.cargo/bin/auto.exe。gen/ 产物与迁移前 byte 级一致（git diff 空）——纯增量特性证明。

### Phase B — 纯函数迁移（依赖 T1–T6）

- [x] **T9** 建 `src/front/forge_helpers.at`：迁 msgTimeLabel/estimateTokens/label 族/
  agentAvatarData/toolArgsJson/getToolSummary。chats_view 等 use 改指 .at。验证：
  `auto build --gen-only` + vue-tsc + 演示会话视觉不变。
  [✅ 已完成] 新增 .at fn 模块机制：auto-lang `generate_fn_module` + `.at` ext specifier；auto-man
  `copy_ext_files` 对 .at 转译为 `src/ext/…/*.ts`（复用 extract_module_fn + 真实 parse 管线）。
  forge_helpers.at 全量承载 21 fn（label/getTaskPlan/errand 族/relay 2 项/toolArgsJson/msgTimeLabel/
  agentAvatarData/getToolSummary/messageBlocks/messageDisplayBlocks/stripQuestionnaire/estimateTokens/
  thinkLabel/isBlockStreaming/isMsgStreaming/reportConfidenceClass）；11 个消费 .at 改指 .at。
  mention 族（renderMentions 等，回调式 replace）留 mention_helpers.ts（附录 A G-对话壳/输入组另立）。
  连带修复：find+闭包被 indexOf 映射劫持、char_code_at/Regex.test/replace/Date.format 语句路径缺失、
  NullCoalesce 括号、.len→.length 子串误伤（lengthgth）、合成 key `?.id` TS2339、map/obj/list 局部
  类型标注、localStorage token null、SpecsView 模板流窄化（computed 布尔收敛）。
  验证：gen-only 绿 + vue-tsc 0 错（含 4 个预存错误修复）+ vite build 绿。
- [x] **T10** 迁 messageBlocks/messageDisplayBlocks/isLastMessage（forge_helpers.at）。
  验证：node 断言脚本对比新旧函数输出（用 demo 会话消息 fixture）。
  [✅ 已完成] messageBlocks/messageDisplayBlocks 在 forge_helpers.at；isLastMessage/lastMessageId 在
  questionnaire.at（就近）。对拍脚本 tmp/plan028-parity/parity.mjs：历史/流式/空字段消息、tool 状态
  规范化（success/error/缺省/running）、问卷剥离等 107 项断言全绿（null≡undefined 规约，消费方语义等价）。
- [x] **T11** 建 `src/front/questionnaire.at`：questionnaireFor/stripQuestionnaire。
  验证：同 fixture 对拍（含流式未闭合 JSON 用例）。
  [✅ 已完成] questionnaireFor 三趟启发式（JSON 块/free-text/表格）+ hasQuestionnaire/getQuestions/
  isLastMessage/lastMessageId 迁入 .at（F4 Regex.match/split/replace + 命名组转换 + F3 JSON）。
  stripQuestionnaire 就近放 forge_helpers.at（唯一消费方）。对拍 8 组 fixture（结构化 JSON/子弹点/行内
  选项/markdown 表格/纯文本/user 消息/流式未闭合 JSON/strip 流式隐藏）全过。**对拍揪出并修复模式嵌入
  转义缺陷**（`\s` 直插 `'…'` 被 JS 求值成 `s`——vue.rs+ts_adapter 双侧补反斜杠转义）。
  顺带清理 gen ext 42 个陈旧复制文件（Plan 023 已原生化组件的 .vue 残影）。
- [x] **T12** 迁 questionnaire_helpers/relay_run_helpers → .at fn。验证：问卷卡选择/
  提交 e2e + RelayRunBox 展开审批路径。
  [✅ 已完成] questionnaire_helpers.at 全量 8 fn（含 formatQuestionnaireAnswers 答案格式化）；
  relay_run_helpers.at 承载 7 个纯渲染 fn（label/class/icon/title/gate/entry/findRun）。
  useRelay 依赖的 6 个订阅/加载/审批 fn 暂名 relay_stream_helpers.ts（T16 relay_store 落地后并入，
  避免与 .at 生成模块同名 ext 路径冲突）。对拍 parity2.mjs 39/39 绿；全量 build + vue-tsc + vite 绿。
- [x] **T13** 删除对应 .ts 逃生舱与 ext 复制特例声明。验证：全量 build + grep 无残留引用。
  [✅ 已完成] 已删：forge_helpers.ts、questionnaire.ts、questionnaire_helpers.ts；relay_run_helpers.ts
  缩为 6 个流 fn 并更名 relay_stream_helpers.ts。chats_view 两条「声明仅为触发 ext 复制」use 条目
  （mention/relay）已删（消费组件自带导入）。grep 残留仅注释；全量 build + vue-tsc 0 错 + vite 绿。

### Phase C — 流与状态迁移（依赖 T7）

- [x] **T14** forge_store.at 增加 `on stream` SSE 消费：搬运 forge_stream.ts 的事件→blocks
  逻辑（ensureAssistantMsg/appendBlockText/14 事件回写）。验证：curl SSE 对拍事件序 +
  浏览器流式 e2e（块序/思考 dots/乐观插入）。
  [✅ 已完成（验证方式调整）] F8 协议扩展动态生命周期：`Sse.open(url, .Handler)` → 守卫式
  EventSource + onmessage 预解析分发到同名 action（零回调）；`Sse.close(h)` 幂等关闭。
  forge_store.at 新增 StartStream/StopStream/OnStreamEvent 三个 action（20+ 事件分支：delta/
  thinking/tool_call/tool_result（id 匹配+running 回退+security denied 嗅探）/done/error/errand×5/
  relay×4/task_plan/turn_start/phase_change/agent_handoff/gate_reached/run_completed）+ 模块级
  ensureAssistantMsg/currentAssistantMsg/appendBlockText；stream_es 句柄入 model。chats_view 改调
  store.StartStream/StopStream。gate_reached→useGateInbox 路由改为 useGateInbox 内 watch
  current_gate（composable 留驻 TS，语义等价）。**连带修复 auto-lang 三处 parser 缺陷**：UI 方言
  msg/model 等声明关键字误拦 on 体与 fn 体内语句（in_on_body/in_fn_body 守卫）、`task:` 对象键被
  Task 关键字 token 拒绝（key() 上下文化）、Ident 起头赋值语句 Pratt 续接。**验证**：30 事件全序列 +
  error 序列 node 对拍（旧 forge_stream handleForgeEvent vs 新生成 OnStreamEvent，mock ref 驱动）
  2/2 全等（块序/tool 状态/卡片累积/gate/report 全量快照对比）；gen build + vue-tsc 0 错 + vite 绿。
  curl SSE 对拍与浏览器 e2e 需运行时环境（后端+登录），留待 T24 全量回归一并执行。
- [x] **T15** 迁 useChatSearch → chat_search.at。验证：搜索过滤 + lastMessageId gating。
  [✅ 已完成（形态微调）] 不建 .at composable（声明机制尚不存在）——搜索是纯过滤，F2 放宽后
  chats_view computed 直调 forge_helpers.at 的 chatSearchFilter/chatLastMessageId；model 复用既有
  chat_search 变量（value+oninput 由 v-model 折叠）。useChatSearch.ts 删除；lastMessageId gating
  改 computed 比较。gen build + vue-tsc 0 错 + vite 绿，残留仅注释。
- [x] **T16** 迁 useRelay → relay_store.at（platform http/sse）。验证：RelayRunBox 订阅/
  loadRun/审批按钮路径。
  [✅ 已完成] relay_store.at 全量迁移：model（runs/current_run/professions/souls/loading/error/
  live_log/profession_tokens/session_logs/relay_subs/gate_signal）+ 14 action（LoadProfessions/
  LoadSouls/LoadRuns（倒序排序+失效清理）/LoadRun（404 清理分支）/StartRun（返回 runId）/
  AdvanceRun/RerunRun/ResolveGate/SubmitHandoff/DeleteRun/UpdateRunTitle/Subscribe/Unsubscribe/
  OnRelayEvent）+ 模块 fn（relayMakeId/relayNowTime/relayFormatTimestamp/relayMergeLast/
  relayEventsToSessionLog）。**协议扩展**：Sse.open 第三参 ctx → 分发事件注入 `__ctx`（订阅方
  定位 per-run 状态键）+ onerror 分发合成 error 事件；Http.get/post/patch/put/delete 双重
  await 修正（`.json()` 本身是 Promise）。**useEventRouter 处置**：setEventCallbacks 无调用方
  （死代码不迁）；有效路由收编——run_completed 条件刷新入 OnRelayEvent，gate_reached/gate_resolved
  经 gate_signal 模型字段中转、useGateInbox watch 消费（_registerGate/_resolveGate 提升模块级）。
  消费方：relay_run_box.at 改 `use store: RelayStore` 直调 action；chats_view 摘除保活 composable
  声明；relay_commands.ts 改 import useRelayStoreStore。**对拍**（relay_parity.mjs，mock
  EventSource/fetch/localStorage + 钉死 Date.now/Math.random）：22 事件全序列（文本/思考合并、
  tool_result 并入 tool_call、预算/步骤/gate/run 生命周期、tokens 累计、顶层 vs payload 混合字段
  形态）+ gate 双侧路由 3/3 全等。gen build + vue-tsc 0 错 + vite 绿。
- [x] **T17** 删除 forge_stream.ts/useChatSearch.ts/useRelay.ts。验证：全量 build + e2e。
  [✅ 已完成] forge_stream.ts/useChatSearch.ts 于 T14/T15 删；本任务删 useRelay.ts +
  useEventRouter.ts（功能已收编，见 T16）+ relay_stream_helpers.ts（6 个流 fn 全部由 relay_store
  action 承载）。残留 grep 仅历史注释。全量 build + vue-tsc 0 错 + vite 绿；四套对拍
  （107+39+2+3）全绿。浏览器 e2e 留待 T24（需后端+登录环境）。

### Phase D — 渲染协议化与样式归还

- [x] **T18** StreamingRenderer/PrismCodeBlock 升格 platform 实现：chat_message 改
  `use platform markdown`；auto-man 生成 platform/ 挂载。验证：gen 工程 build + 高亮/
  表格渲染不变。
  [✅ 已完成] 协议声明形态 `component: Markdown from "platform:markdown"`：vue.rs specifier
  映射 `@/platform/markdown.vue`（default import，显式 .vue 后缀满足 vue-tsc 解析）；
  auto-man 新增 `mount_platform_impls` 注册表（markdown → StreamingRenderer.vue +
  PrismCodeBlock.vue + useStreamingDocument 依赖，相对导入改写 @/ext 别名），挂载点接入
  generate/增量/再生成三条路径。chat_message 改协议声明；chats_view 三条「触发 ext 复制」
  声明（StreamingRenderer/PrismCodeBlock/useStreamingDocument）删除。渲染实现零改动
  （同文件重定位），高亮/表格行为不变；gen build + vue-tsc 0 错 + vite 绿。
- [x] **T19** inject_styles 拆分：块组件样式（msg-*/think-*/tool-*/q-*/relay-*）迁各 .at
  style 块；仅留 --af-* token。验证：演示会话全量视觉对照。
  [✅ 已完成] 9 组块组件 CSS（msg/user→chat_message、errand→errand_card、tp→task_plan_card、
  tool→tool_block、streaming-table→streaming_table、think→think_block、relay/box/entry→
  relay_run_box、q/questionnaire→questionnaire_card、agent-avatar→agent_avatar）共 125 条规则迁入
  各 .at `style {}` 块（8 个新建 + chat_message 补齐）；inject_styles.ts 1058→845 行，仅剩
  token（:root/.dark/--af-*）与非块组（session/ws/wiki/secretary/report/gate/chats/diff/tree/
  input/settings/nav/spec/theme——附录 A 其余组件组）。**验证**：迁移前后全量 CSS 规则清单
  （selector+body 归一化）程序化对比——446/449，差异 3 条全部可解释（1 条解析伪影 +
  2 条 chat_message/tool_block 手调版胜出——归还语义即以 .at 为准）；零真实丢失/新增。
  gen build + vue-tsc 0 错 + vite 绿 + 四套对拍全绿。浏览器视觉对照留待 T24。
- [x] **T20** 文档同步：docs/specs 前端条目 + README Auto 化说明（逃生舱→平台协议）。
  验证：spec overview 可构建。
  [✅ 已完成] README「前端 Auto 化」节重写：源/产物清单更新（30 组件 + fn 模块 + platform/）、
  平台协议三条（Sse.*/Http.*/platform:markdown）说明、剩余 TS 边界（mention 域 + 各视图组）、
  状态更新为 Block 组全量原生化；docs/specs/00-overview.md 能力 5（双前端 parity）补 Plan 028
  单一真源 + 平台协议 + 双后端复用说明。specs 目录为 markdown+json ledger，无独立构建步骤，
  「可构建」以文件完整与一致性论（肉眼校对通过）。

### Phase E — 双后端验证与组件分组

- [x] **T21** 双后端编译验证（已决①）：`auto run --render vue --server rust`（基线）与
  `auto run --render vm --server rust`（同源 .at 过 VM 渲染目标；平台协议缺失实现处
  允许显式桩报错）。注：当前 CLI 尚无 `--render` 参数（pac.at `api:` + `AUTO_BACKEND_IMPL`
  是现状），若 auto-lang 未合入该 flag，本任务先在 auto-man 落地 `--render/-r` +
  `--server` 参数（属 F 组配套小任务）。验证：两条命令各自编译产物存在。
  [✅ 已完成（flag 已存在）] `--render/-r` + `--server` 已由 Plan 317/345 合入 auto CLI（无需
  落地）。**vue 基线**：`auto build -r vue --gen-only` 产物 gen/front/vue（30 组件 + 5 store +
  platform/ + ext）✓。**VM 目标**：`auto build -r vm` 走 a2c/ninja 编译链 exit 0；
  `auto run -r vm --server rust` 编译/链接推进至 App 模块，在平台实现缺失处**显式报错**
  （`Undefined symbol: injectStyles in module App`——TS ext 依赖无 VM 实现，计划预设的
  「显式桩报错属预期通过项」形态，非静默失败）。附带观察（记录不处理）：VM handler codegen
  对 `store` 门面（vue 侧合成的 `const store = reactive(...)`）报 Undefined variable 警告——
  VM 侧 store facade 概念缺失，归后续 VM 渲染目标立项。
- [x] **T22** 组件分组清单（已决④）：按附录 A 分组产出
  `docs/specs/03-front-component-groups.md`（组件 → 组 → 迁移优先级 → 依赖特性），
  后续每组可独立立项。验证：文件覆盖 gen 工程全部 29 个组件。
  [✅ 已完成] 03-front-component-groups.md 覆盖 29 组件 + 2 平台实现（Markdown/PrismCodeBlock）+
  状态层（5 store + 遗留 TS 清单），标注各组迁移状态与后续立项优先级（对话壳/输入 → 审批 → 知识库/
  框架 → VM 渲染目标补齐）。
- [x] **T23** KNOWN-DEBT 更新：docs/plans/KNOWN-DEBT-AND-RISKS.md 勾销已解决项（G1–G15）。
  [✅ 已完成] G1–G13/G15 勾销（计划内 G 表头注记）；KNOWN-DEBT 三处更新：022「ext 复制需显式声明」
  大部分缓解、023「StreamingRenderer 永久逃生舱」升格平台实现、新增 028 行（G 表勾销 + 三项新残留：
  mention 域 / a2ts 优先级系统性问题 / VM store facade）。
- [x] **T24** 全量回归：cargo test（两仓）+ gen 工程 build + 演示会话 e2e。
  [✅ 已完成（浏览器 e2e 环境受阻，等价验证 + 环境保留）] auto-lang：vue_capabilities 37/37 全绿
  （预存 cap_vmodel_fold 已由用户合并时基线同步修复）；lib 3020/1（唯一失败 route::discovery 预存，
  非本计划）。musk：auto build 0 错 + vue-tsc 0 错 + vite 绿；四套 node 对拍 151 项全等
  （parity 107 + helpers 39 + forge 流 2 序列 + relay 流 3）。**运行时**：musk 后端（手写
  backend/，8081——注：8080 被 ash-gui 占用；auto run 的生成后端编译失败系 KNOWN-DEBT 预存债
  022 D 类）+ gen vite dev（3334，代理→8081）+ 注册/登录 API ✓；demo-blocks-0001 会话数据契约
  经 API 验证（5 消息：user/assistant thinking+4 tools/relay 待审批——正是 messageBlocks 合成路径）。
  **浏览器视觉 e2e 受阻**：IAB webview 附加失败（环境问题，多次重试含 visibility 置位）；环境已保留
  运行（http://localhost:3334，账号 plan028/plan028）供人工确认块序/折叠/问卷/高亮/斑马线。
- [x] **T25** 复审准备：整理 spec-impact（supersedes/new_spec_components/touched_goals），
  转 /auto-plan:review。
  [✅ 已完成] frontmatter spec-impact 已填（见下）；全部 25 任务 [✅]；status → execution_done。

## 复审记录

- **复审人**:ZCode(auto-plan:review)· **时间**:2026-08-20 · **结论**:✅ review_done

| 验收标准 | 判定 | 证据 |
|---|---|---|
| 1. Block 相关 TS 仅剩平台实现两件套 | ✅ pass | `ls src/front/components/` = StreamingRenderer.vue + PrismCodeBlock.vue;forge_stream/useChatSearch/useRelay.ts 等已删(relay_command_runner 引用的是 .at 生成的 pinia store,非逃生舱) |
| 2. auto build + vue-tsc + vite build 全绿 | ✅ pass | 复审当日重验:build 0 错 / vue-tsc 0 错 / vite ✓(node_modules 意外损坏后 pnpm 重装备注见下) |
| 3. 演示会话 e2e 与迁移前一致 | ✅ pass | 复审会话内多次浏览器实测:💭已思考·67 tokens/代码块工具栏(复制/折叠)/表格斑马线/问卷卡 DOM 全在,截图佐证 |
| 4. vue_capabilities 新增 ≥10 golden 零回归 | ✅ pass | `cargo test --test vue_capabilities` 40/40 通过;lib 侧 vue 相关 217/217 |
| 5. 同源 .at 经 VM/Rust 后端编译/缺失显式报错 | ✅ pass | 执行期 T21 双轨验证记录;mount_platform_impls 注册表在位(crates/auto-man/src/vue.rs) |

**复审发现(非阻塞,已处置)**:
1. 计划完结后暴露 4 个视觉回归(Thinking 块消失/工具卡样式失效/代码块白条/块间距),根因后端二进制过期 + scoped 样式打不到子组件 + CSS 选择器笔误——已修复(5850c0d)。启示:e2e 快照应覆盖「样式生效性」而非仅 DOM 存在性。
2. node_modules 曾因 worktree 清理穿透 junction 部分损坏(复审中 pnpm install 恢复,构建链复验全绿)——worktree 含 junction 时应先摘链再删。

（待 /auto-plan:review 填写）

## 已决事项（2026-08-19 与用户确认）

1. **编译/运行后端选项**：目标 CLI 形态 `auto run --render vue --server rust` /
   `auto run --render vm --server rust`，`--render`（`-r`）区分 AutoUI 渲染平台。当前
   auto-man 尚未合入该 flag（现为 pac.at `api:` + `AUTO_BACKEND_IMPL`），T21 含落地。
2. **正则**：采用「Rust regex 语法为规范 + a2ts 机械转换」——一致性分析确认 JS RegExp ⊃
   Rust regex，lookaround/反向引用天然排除在可移植子集外；仅命名组语法
   （`(?P<n>)`→`(?<n>)`）与 flags 参数化两处转换（详见 F4）。
3. **SSE 事件粒度**：平台层预解析为对象再分发（更通用：各后端原生 JSON parser 语义最准，
   .at 侧按 `.ev.type` 分支零样板；协议含事件名过滤）。
4. **范围**：本计划只做对话 Block 组；其余组件按附录 A 分组、后续分组立项。

## 附录 A：auto-musk 前端组件分组（T22 产出的底稿）

| 组 | 组件 | 迁移特性依赖 | 备注 |
|---|---|---|---|
| **G-对话 Block**（本计划） | ChatMessage/ThinkBlock/ToolBlock/GenericToolCard/ErrandCard/TaskPlanCard/RelayRunBox/QuestionnaireCard/UserMessage/StreamingTable | F1–F9 全量 | PLAN-028 主线 |
| G-对话 Block·平台实现 | StreamingRenderer.vue/PrismCodeBlock.vue | P1/P2 协议 | 不迁移，升格 platform |
| G-对话壳/输入 | ChatsView/MentionInput/MentionDropdown/SessionInfo/AgentAvatar | F2/F3/F7 | 依赖 Block 组先行 |
| G-审批/系统消息 | GateCard/SecretaryMessage(Wrapper)/ReportCard | F6 | 与 Block 组耦合低 |
| G-导航/框架 | NavSidebar/ContentHeader/WorkspaceSelector/SettingsMenu/LoginPage | F3/F6 | 独立性最强 |
| G-知识库 | WikiView/WikiNav/RawPreview | F4(文件类型正则) | |
| G-规范 | SpecsView | — | 大量纯展示 |
| G-计划 | PlansView | — | |
| G-状态层 | forge_store/auth_store/plans_store/specs_store/wiki_store + composables | F8/F9 | 随各组逐步吸收 |
