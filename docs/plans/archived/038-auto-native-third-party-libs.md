---
plan_id: PLAN-038
status: archived
feature_name: 第三方库 Auto 版替换（i18n/icons + 渲染真源切 auto-down + 高亮方案对比）——VM/Rust 目标即插即用
author: [zhaopuming]
created_at: 2026-08-23
updated_at: 2026-08-23

supersedes_spec_components:
  - "00-overview.md: 关键能力5（双前端 parity）——.at 轨渲染真源（markdown）经 ports/renderer 切 @autodown/vue（vendored 快照）；新增 src/front/lib 纯 Auto 库层（auto-i18n/auto-icons）与 deps-guard 依赖边界守卫"
  - "03-front-component-groups.md: G-对话 Block·平台实现表——Markdown（platform:markdown）实现由 markstream-vue+useStreamingDocument 改为 @autodown/vue StreamingRenderer 再导出（保留 PrismCodeBlock setCustomComponents 注册）"
new_spec_components: []
touched_goals:
  - "goal-frontend-parity: 第三方库面 .at 单源化（i18n 156/156、icons 数据层 52/52 对拍全等）；渲染真源切 @autodown/vue（render-switch 5/5 白名单外零差异）；依赖白名单守卫（deps-guard）；svg 渲染层降级登记 KNOWN-DEBT（解除条件 auto-lang 442 A4）；高亮/mermaid 跨轨决策落档（(a) syntect 原生+prism 保留/不复刻）"

current_step: 16
total_steps: 18
---

# [PLAN-038] 第三方库 Auto 版替换——为 VM/Rust 目标预置纯 Auto 能力

## 变更摘要

2026-08-23 全量普查 auto-musk 的 vue/ts/js 第三方依赖（双轨 grep + node_modules 实证），
确认 `.at` 单一真源经 PLAN-037 五域端口（platform/composables/icons/renderer/upload）实际
消费的第三方库共 5 个：vue-i18n、lucide-vue-next、markstream-vue、mermaid、prismjs；
另确认 `marked` 为零使用遗留声明、gen 轨 package.json 存在大量未使用模板依赖。

结合 `../auto-down` 与 `../auto-lang` 的现状勘察（2026-08-23 实测，见需求分析），处理
策略分三类：

1. **自建纯 Auto 库**（本计划直接交付）：auto-i18n、auto-icons——小体量、纯逻辑/纯
   数据，`src/front/lib/` 下 .at 实现 + 对 npm 原库对拍全等。
2. **渲染真源切换到 auto-down**（本计划做消费侧切换 + 协同登记）：markstream-vue 的
   消灭不由 musk 从零复刻，而是把 ports/renderer 域切到 **@autodown/vue**（我方可控、
   已部分 Auto 化、与 musk 现有 StreamingRenderer 逃生舱同源）；其内部对 markstream-vue
   的进一步 Auto 化替代在 auto-down 仓推进（本计划起草其计划草稿并登记依赖），编辑库
   （@autodown/editor）随后一并融合（待澄清 #5）。
3. **高亮方案对比决策**（本计划做对比 + 决策落地第一步）：prismjs 的 Rust 替代已存在
   ——auto-lang 内置 `code_editor` 的 **syntect**（two-face 全量语法集）高亮（examples/
   041-auto-edit 实证）；对比 prismjs/lowlight/syntect 三方案一致性后定夺，不再默认
   "复刻 prismjs"。

mermaid 不复刻（体量失衡），走平台端口 + VM 轨降级。附带清理死依赖、用守卫脚本固化
依赖边界。**本计划不改动 musk 现有运行路径行为**（渲染切换有对拍保障）；每 Phase 收口
`auto build` + web vitest 存量全绿（不变量）。

## 目标

1. **依赖普查固化为可执行资产**：第三方白名单守卫脚本（deps-guard）+ 死依赖清理
   （web 轨 `marked`、gen 轨 codemirror 系/reka-ui 等未用声明）。
2. **auto-i18n**：vue-i18n 语义子集（嵌套键查找 + `{name}` 具名插值 + 缺键回退）纯 .at
   实现，81 键 × zh/en 全量对拍全等。
3. **auto-icons**：lucide 图标集（ports 37 符号 ∪ web 轨差集）SVG path 数据 .at 化 +
   `Icon` widget 渲染层，renderToString 对拍全等。
4. **渲染真源切 @autodown/vue**：ports/renderer 域从 markstream-vue 直依赖切到
   @autodown/vue（workspace/file 接入），musk 真实内容渲染 DOM 对拍无回归；auto-down
   侧「渲染库 Auto 化 + markstream-vue 消灭」计划草稿落地并作为依赖项登记。
5. **高亮方案对比与决策**：prismjs vs lowlight（@autodown/vue 内置）vs syntect
   （auto-lang code_editor）在 musk 实际语言集（11 种）上的 token/scopes 一致性实测，
   结论三选一登记（VM 轨 syntect 原生 / .at 复刻 / 降级），并落地决策的第一步。
6. **mermaid 决策**：不复刻，VM 轨降级渲染路径明确化。

## 架构方案

```
现状（PLAN-037 落定）                本计划后
─────────────────────               ──────────────────────
调用方 .at（纯 Auto 符号）           调用方不变
  └─ use pac.ports.<域>               └─ use pac.ports.<域>
       ├─ ports/composables.web.at         ├─ composables: vue-i18n 旁新增 src/front/lib/i18n.at（纯 .at，对拍绿后可直绑）
       │    └─ use.web from "vue-i18n"     ├─ icons: lucide 旁新增 src/front/lib/{icons_data,icon}.at
       ├─ ports/icons.web.at               ├─ renderer: markstream-vue → @autodown/vue（真源升级；
       │    └─ use.web from "lucide-vue-next"  │   其内部 Auto 化在 auto-down 仓推进，musk 跟进版本）
       └─ ports/renderer.web.at            └─ 高亮: prismjs（vue 轨现状）→ 对比决策后定
            └─ use.web from "markstream-vue"
```

- **自建库的归属**：先落 `auto-musk/src/front/lib/`（无 musk 专属耦合、模块自足），
  未来可整体提取为 auto-lang 生态包；ports 的 npm 绑定本计划仅 renderer 域切换，
  i18n/icons 的直绑替换属跨平台迁移完成后的接入步骤（待澄清 #7）。
- **对拍（differential testing）为验收手段**：每项一个 `scripts/lib-parity/*.mjs`，
  同输入喂 npm 原库（node 直跑）与 .at 编译产物 / 切换前后双版本，断言输出全等；
  fixtures 全部来自 musk 真实内容 + 构造边界，落 `scripts/lib-parity/fixtures/` 入库。
- **auto-down 协同边界**：本计划只做 musk 消费侧（依赖接入、端口切换、对拍）+
  起草 auto-down 侧计划草稿；markstream-vue 在 auto-down 内部的消灭（解析层/渲染层
  Auto 化、编辑库融合）由该草稿承接，musk 以版本升级方式跟进。
- **能力风险前置**：icons 渲染层依赖 .at UI 的 svg 元素支持——Phase 2 首任务 canary
  实测，不支持则数据层照常交付 + 渲染降级登记 KNOWN-DEBT。

## 技术栈

auto-musk（src/front/lib/ 新库 + scripts/lib-parity/ 对拍 + scripts/gen-*.mjs 数据生成器
+ ports/renderer 切换）、../auto-down（@autodown/vue 接入 + 侧计划草稿）、../auto-lang
（只读：syntect 高亮对比所需 two-face/syntect 版本对齐；041-auto-edit 实证参考）、
auto build（.at → gen/front/vue 编译）、node（对拍脚本）、web/node_modules（npm 原库
对拍基准：vue-i18n / lucide-vue-next / prismjs / @vue/server-renderer）。

## 需求分析与背景调查

> 依据 docs/specs/00-overview.md（双轨 parity、Block 组件组全量原生化、平台协议声明）
> 与 2026-08-23 实测普查（grep 双轨 import + package.json + node_modules + 两个兄弟仓勘察）。

### 第三方依赖普查结论（auto-musk 侧实测）

**`.at` 单源经五域端口消费的第三方：**

| 库 | 版本 | 用途 | 使用面 | 处置（本计划） |
|---|---|---|---|---|
| vue-i18n | ^9.14 | t() 翻译 + locale 切换（useT 桥 + i18n/index.ts） | 14 import + ports/composables | 自建 auto-i18n（81 键×2 语言，仅具名插值 `{count}`，量小） |
| lucide-vue-next | ^0.460/^0.312 | 图标组件 | 30 import + ports/icons（37 符号） | 自建 auto-icons（~44 图标，纯 SVG path 数据） |
| markstream-vue | 0.0.14-beta.8 | 流式 markdown 渲染 | 7 文件，chat/wiki/plans/report 渲染核心 | **切换 @autodown/vue**（见下）；其传递依赖 stream-markdown-parser/markstream-core/@chenglou/pretext/@floating-ui/dom 随之只经 auto-down 间接存在 |
| prismjs | ^1.29 | code_block 语法高亮（PrismCodeBlock 逃生舱，11 语言） | gen 轨 2 import | **对比决策**（prismjs/lowlight/syntect 三方案） |
| mermaid | ^11.15 | 图表 → SVG | 2 文件 | 不复刻——平台端口 + VM 轨降级（auto-down 侧同样依赖 mermaid，决策一并适用，待澄清 #4） |

**确认排除：** `marked`（零使用死声明）；gen 轨 package.json 的 vue-codemirror+
@codemirror/*（唯一引用者 CodeEditor.vue 是死文件）、reka-ui/vaul-vue/vue-sonner/
vee-validate/@vee-validate/zod/zod/embla-carousel/@vueuse/core（零 import 模板遗留）；
`vue` 本体（跨平台运行时职责）；`vitest`（测试基建）；tailwind-merge+clsx（gen 轨
cn()，VM 轨样式模型未定，待澄清 #6）。

### ../auto-down 勘察（2026-08-23 实测）

pnpm workspace 三包格局（autodown/packages/）：

- **@autodown/core 0.2.0**——IAL 解析器/类型/fixtures。**Auto 化已落地**：
  `auto/ial.at` 为单源，经 `auto trans --path auto/ial.at ts`（a2ts 通道）转译发布，
  带文档化的后修补清单（`int?`/`List<int?>` 优先级、`RegExp(p,f)` 直通 JS、内置
  parseInt/isNaN 直通）。这是".at 库源 → TS 发包"的既有先例，渲染库 Auto 化可循此路径。
- **@autodown/vue 0.1.1**——"streaming and static document rendering for AutoDown"：
  StreamingRenderer.vue / useStreamingDocument.ts / StreamingTable.vue + katex/lowlight/
  mermaid/details 容器变换。**内部仍依赖 markstream-vue ^0.0.14-beta.8**（MarkdownRender/
  enableKatex/enableMermaid），高亮走 lowlight（hljs 系）。与 musk 的 StreamingRenderer
  逃生舱（src/front/components/StreamingRenderer.vue，"从 web/ 整体移植"）同源同构，
  上游多了 codeBlockProps/placeholder/katex/details 能力——切换成本天然低。
- **@autodown/editor 0.2.0**——Tiptap WYSIWYG 编辑器；已带 .at 应用层
  （editor/src/auto/src/front/*.at：auto_down_editor/code_block_menu/bubble_menu 等
  一组 widget）。编辑库融合属后续（待澄清 #5）。

### ../auto-lang 041-auto-edit 勘察（2026-08-23 实测）

- 编辑器全 .at（src/front/app.at 475 行）：消费内置 UI 元素 `code_editor (key, lang,
  style)` + `code_editor_*` API 族（cursor/selection/text/undo/redo），`lang:"auto"` 实证
  自带高亮。
- 高亮实现源：`crates/auto-lang/src/ui/code_editor/core/highlight.rs`——**syntect**
  （two-face 全量 Sublime 语法集 + 自带 AutoLang 语法定义 + 主题合成），进程级
  SyntaxSystem 单例；`lang_to_extension` 已映射 rust/python/js/ts/json/toml/yaml/md/
  html/css/c/cpp/go/java 等，musk 11 语言（rust/typescript/javascript/json/bash/
  python/markdown/yaml/toml/sql/java/c）基本全覆盖（bash=shell-script、sql 需实测确认，
  对比任务覆盖）。Rust/VM 侧高亮原语已存在，缺的是只读渲染消费面 + 与 prismjs 的一致性结论。

### 与既有架构的关系

- PLAN-037（execution_done）：五域端口 + use.web + `.web.at` 适配器 + 编译期目标门控
  已就绪——本计划的切换与新增库都在端口架构内进行，VM/Rust 接入时复用同一机制。
- KNOWN-DEBT 029 D 组"永久保留"清单：useT（vue-i18n 桥）与 StreamingRenderer/
  PrismCodeBlock（markstream/prism 依赖）正是本计划的处理对象——完成后 D 组可缩容。
- pac.at `npm_deps: ["markstream-vue@0.0.14-beta.8"]` 需随 renderer 切换调整为
  @autodown/vue 接入声明。

## 详细设计

### D1 目录与命名（auto-musk 侧新增）

```
src/front/lib/
  i18n.at / i18n_catalog.at        # catalog 为生成物（T6 脚本从 i18n/*.json 生成）
  icons_data.at / icon.at           # 数据层 / 渲染层
scripts/
  gen-i18n-catalog.mjs  gen-icons.at-data.mjs
  lib-parity/
    deps-guard.mjs  i18n.mjs  icons.mjs  render-switch.mjs  highlight-compare.mjs
    fixtures/                        # fixtures 入库（真实内容采样）
scripts/highlight-rs/                # syntect 侧对比输出器（cargo 小工程）
../auto-down/plans/008-*.md          # auto-down 侧计划草稿（T14 产出，编号以现场为准）
```

### D2 auto-i18n（语义子集 = musk 实际使用面）

- `fn i18nT(locale String, key String, params Value) -> String`：嵌套键点分查找 →
  `{name}` 具名插值 → 缺键返回 key 本身（vue-i18n 默认行为，fixtures 含缺键用例锁定）。
- locale 状态（切换/持久化 localStorage）是宿主职责：vue 轨 = 现有 useT 桥，VM 轨 =
  架构迁移侧信号机制——库只做纯函数，保证两端同一 t 语义。

### D3 auto-icons（数据层与渲染层分离）

- 数据层 `icons_data.at`：每图标 = 元素数组（tag/attrs 规范化；lucide 统一
  stroke="currentColor"/stroke-width=2/viewBox 24 提到渲染层默认值），生成器从
  `web/node_modules/lucide-vue-next` dist 提取，加"生成物勿手改"头注。
- 渲染层 `icon.at`：`widget Icon(name String, size Num, stroke_width Num)` → svg 元素树。
  **前置 canary**：单图标经 auto build 产物核验 .at UI 对 svg 元素/属性的支持度；
  不支持 → 数据层照常交付 + 渲染层降级登记 KNOWN-DEBT（待 auto-lang svg 节点能力）。
- 对拍：@vue/server-renderer（web/node_modules 已有）双端 renderToString，规范化
  （属性序/自闭合形式）后逐图标全等。

### D4 渲染真源切换 @autodown/vue（markstream-vue 消灭的 musk 侧步骤）

- **接入方式**（T11 现场定夺，默认 file: 链接）：auto-down 为 pnpm workspace 且
  @autodown/vue `main` 指 dist——需先在 auto-down 侧 `pnpm build` 出 dist；musk web/
  package.json 以 `file:../auto-down/autodown/packages/vue` 依赖（npm 支持），pac.at
  npm_deps 对应调整。若 auto-down 侧愿意 npm 发包则改版本号直依赖（草稿里列为建议）。
- **切换面**：ports/renderer.web.at 的 `use.web component MarkdownRender from
  "markstream-vue"` → `"@autodown/vue"`（其 StreamingRenderer 是同名能力超集）；
  musk 的 StreamingRenderer 逃生舱（src/front/components/StreamingRenderer.vue）对齐
  上游形态（codeBlockProps/details/katex 按需启用，PrismCodeBlock 的
  setCustomComponents 注册在切换后保留，行为不变）。
- **对拍**：切换前后对同 fixtures（musk 真实 wiki/chat/report 内容 + 流式前缀）DOM
  快照 diff；差异白名单（上游新增能力的可容忍差异，如 details 折叠属性）显式登记。
- **协同**：markstream-vue 在 @autodown/vue 内部的消灭（解析层下沉/替换、渲染层循
  core 的 a2ts 模式 .at 化、mermaid/katex 可选化）由 T14 的 auto-down 侧计划草稿承接；
  本计划不实现、只登记依赖并跟进版本。

### D5 高亮方案对比与决策（prismjs / lowlight / syntect）

- **三方现状**：musk vue 轨 = prismjs（PrismCodeBlock，11 语言手动 import）；
  @autodown/vue = lowlight（hljs common 集）；auto-lang Rust = syntect（two-face 全量，
  code_editor 内置，041 实证）。
- **对比脚本**：`scripts/highlight-rs/`（cargo 小工程，对齐 auto-lang 的 syntect/
  two-face 版本）对 fixtures 代码块输出每语言 scope 序列 JSON；node 侧 prismjs 输出
  token 类型序列、lowlight 输出 hast 类名序列；经 scope→token 近似映射表对齐后 diff。
- **决策三选一**（T16 落记录）：
  (a) **VM 轨 syntect 原生**：登记 auto-lang 侧"只读高亮渲染原语"需求（code_editor
  只读模式或 highlight-only API），vue 轨保留 prismjs（双轨差异实测可容忍则接受，
  否则 vue 轨同步换装）；——当前推荐，待对比数据支撑。
  (b) **.at tokenizer 复刻**：prism 文法数据化 + .at 实现（受 .at 正则能力门控，
  a2ts 的 RegExp 直通先例只解决 vue 轨、不解决 VM 轨）；对比证明 (a) 不可行才走。
  (c) **降级无高亮**：VM 轨首版 `<pre>`，高亮后置。
- mermaid 降级与 D4 切换联动：@autodown/vue 默认 enableMermaid——musk 切换后 web 轨
  行为不变（mermaid 经其内部走 npm 包），VM 轨降级路径在 auto-down 侧草稿中登记。

### D6 deps-guard（依赖边界固化）

白名单 = 普查结论表 + vue + 测试基建（vitest/@vue/server-renderer/@types/*）+
@autodown/vue（file/workspace 接入后）。扫描面：web/src 与 gen/front/vue/src 的非相对
import、src/front 全部 `use.web ... from` 目标。新增第三方依赖 → 脚本非零退出并打印
清单（CI 可挂）。

## 测试设计

1. **对拍套件**：`node scripts/lib-parity/{i18n,icons}.mjs` exit 0 = 全等；
   `render-switch.mjs` = 切换前后 DOM 快照 diff（白名单外零差异）；
   `highlight-compare.mjs` = 三方案一致性矩阵报告（产出决策数据，不设全等断言）。
2. **存量不变量**：每 Phase 收口 `auto build`（0 错）+ `cd web && npx vitest run`
   （存量 2 套件绿）+ `cd web && npm run build`（Phase 0 清理与 renderer 切换后验证
   vue-tsc 零引用断裂）。
3. **deps-guard**：`node scripts/lib-parity/deps-guard.mjs` exit 0。

## 验收标准

1. vue-i18n / lucide 有纯 .at 实现或明确降级登记，`src/front/lib/` 全部模块零
   `use.web`（grep 断言）；i18n/icons 对拍全绿。
2. ports/renderer 域消费 @autodown/vue，musk 真实内容渲染对拍无回归（白名单外零
   差异）；pac.at npm_deps 同步更新；auto-down 侧计划草稿存在且覆盖"渲染库 Auto 化 +
   markstream-vue 消灭 + 编辑库融合"路线。
3. 高亮三方案对比报告落复审记录，决策三选一有数据支撑并登记后续动作（auto-lang
   原语需求 / 复刻子计划 / 降级）。
4. `marked` 从 web/package.json 移除；gen 轨未用依赖从生成模板清除，重建后 grep
   零命中。
5. deps-guard 落地且当前 exit 0；此后新增第三方依赖会被显式拦截。
6. web 原生轨与 gen 轨行为零回归（auto build + vitest + npm run build 全绿 +
   render-switch 对拍）。
7. mermaid 不复刻决策与库归属决策经用户确认并记录于复审记录。

## 执行步骤

> 粒度约定：每任务 2-5 分钟原子操作（文件路径 + 操作 + 验证命令）；Phase 收口统一跑
> `auto build` + `cd web && npx vitest run`。

### Phase 0 — 普查固化 + 死依赖清理

- [x] **T1** 移除 web/package.json 的 `marked` 与 `@types/marked` 声明（grep 实证
  web/src、gen 源零引用）；验证：`cd web && npm install && npm run build` 绿 +
  `grep -c marked web/package.json` 返回 0。 [✅ 已完成] grep web/src gen src/front 零引用；package.json 两处声明移除；npm install + npm run build 绿（built in 6.57s）；grep -c marked web/package.json = 0（lockfile 剩 4 处为 mermaid 传递依赖，非直依赖）
- [x] **T2** 定位 gen/front/vue/package.json 的生成模板（`grep -rl "vue-sonner" --include="*.json" --include="*.ts" --include="*.at" . ../auto-lang` 找模板源），
  从模板删除未用依赖：vue-codemirror、@codemirror/*（7 项）、reka-ui、vaul-vue、
  vue-sonner、vee-validate、@vee-validate/zod、zod、embla-carousel-vue、@vueuse/core；
  验证：`auto build --gen-only` 后 `grep -E "codemirror|reka-ui|vue-sonner" gen/front/vue/package.json`
  零命中，`cd gen/front/vue && pnpm install && pnpm build` 绿。 [✅ 已完成（转责,用户裁定选项 (ii)）] 模板源实测在 `../auto-lang/crates/auto-man/src/vue.rs`（共享全生态+需重编译 CLI,见待澄清 #9 三选项）→ 裁定选项 (ii)：auto-lang 侧"按使用裁剪"条件化依赖,已登记为 auto-lang 442 **Phase 0 P0-1**（不 gated 可先行,含特性→依赖映射、双向验收——musk grep 零命中 + widgets-gallery 不回归）；musk 侧 grep 零命中验收随 442 P0-1 落地复核（deps-guard TRANSITIONAL 区届时清零）
- [x] **T3** 新建 `scripts/lib-parity/deps-guard.mjs`（白名单内置于脚本头部注释块），
  扫描 web/src + gen/front/vue/src 非相对 import 与 src/front 全部 `use.web ... from`
  目标，超白名单即 exit 1 并打印；验证：`node scripts/lib-parity/deps-guard.mjs` exit 0。 [✅ 已完成] 实测 exit 0；codemirror 系 10 包以 TRANSITIONAL 过渡区放行并单列打印（挂 T2 阻塞/待澄清 #9，落定后清零）

### Phase 1 — auto-i18n（纯逻辑首发）

- [x] **T4** 新建 `scripts/lib-parity/i18n-fixtures.mjs`：node 直跑 vue-i18n
  （createI18n + legacy:false），对 src/front/i18n/{zh,en}.json 全部 81 键生成期望输出
  （无参 / `{count}` 插值 / 缺键回退三类用例）→ `scripts/lib-parity/fixtures/i18n-expected.json`；
  验证：node 脚本运行成功且 fixtures 键数 = 81×2×3。 [✅ 已完成] 156 例生成（72 键×2 语言 plain=144 + interp=2 + missing=10）；实测叶子键 72/语言非普查口径 81，以实际文件为准；语义锁定含 {'@'} 字面量转义与缺键返回 key 本身（vue-i18n 实测行为，Composer.t 经 locale.value 切换调用）
- [x] **T5** 新建 `src/front/lib/i18n.at`：`fn i18nT(locale String, key String, params Value) -> String`
  （嵌套点分查找 + 具名插值 + 缺键回退 key），引用 `i18n_catalog.at`；验证：
  `auto build` 0 错且 gen 产物含 `lib/i18n` 模块。 [✅ 已完成] auto build exit 0；gen/front/vue/src/ext/src/front/lib/i18n.ts 落地（经 composables.web.at re-export 入编译图——lib 无人引用则不编译）。实现偏差两条已实证并处理：(1) 原生 `use <stem>: <fn>` 对 fn 模块不产 import（组件专用）→ i18n.at 做成自足单文件（内嵌 @gen 标记区块目录，零 use.web，满足验收 1 字面 grep）；(2) `.to_lower` 属性引用不做 JS 映射（仅方法调用映射）→ 字符串叶子判定改用原生 `.split` 属性
- [x] **T6** 新建 `scripts/gen-i18n-catalog.mjs`：读 `src/front/i18n/*.json` → 生成
  `src/front/lib/i18n_catalog.at`（.at 值字面量 + "生成物勿手改"头注）；验证：连续
  运行两次输出 diff 为空（幂等）+ `auto build` 0 错。 [✅ 已完成] 幂等实测 diff 空；D1 布局偏差：因 T5 (1) 生成器改为写 i18n.at 的 @gen:i18n-catalog-begin/end 区块（独立 i18n_catalog.at 不再存在），区块外为手写区——理由：跨文件引用仅 use.web 通道可用会破坏验收 1「lib 零 use.web」
- [x] **T7** 新建 `scripts/lib-parity/i18n.mjs`：import gen 编译的 i18n 产物，跑全部
  fixtures 断言全等；验证：`node scripts/lib-parity/i18n.mjs` exit 0。 [✅ 已完成] 156/156 全等（node 25 原生 strip-types 直跑 gen .ts）；键数口径修正：普查 81 键与两份目录均不符——.at 真源 src/front/i18n 72 键/9 节（auto-i18n 范围），web 轨副本 web/src/i18n/locales 357 键/21 节（vue-i18n 继续服务，直绑替换属后续）

### Phase 2 — auto-icons（数据层 + 渲染层）

- [x] **T8** 新建 `scripts/gen-icons.at-data.mjs`：从 `web/node_modules/lucide-vue-next`
  dist 提取图标集（`src/front/ports/icons.web.at` 的 37 符号 ∪ web/src 直接 import
  差集，脚本内固化清单并 grep 核对）→ `src/front/lib/icons_data.at`；验证：node 脚本
  生成 + 清单内每个符号在 icons_data.at 中 grep 命中 + `auto build` 0 错。 [✅ 已完成] 52 图标（ports 实测 38 符号 ∪ web 多行 import 全量 50 符号，并集 52；初扫 grep 漏多行 import 被脚本自证清单核对机制拦下——File/Image/FileCode/PanelLeft/Clipboard/Search/Info/Copy/CopyCheck/Link2/FolderInput/FolderPlus 补入）；别名映射 Loader2→loader-circle、HelpCircle→circle-help、UploadCloud→upload、FileIcon→file 等改名史显式登记；52/52 grep 命中；auto build exit 0；幂等 ✓；经 ports/icons.web.at `use.web icons_data` re-export 入编译图（gen ext/src/front/lib/icons_data.ts）
- [x] **T9** 新建 `src/front/lib/icon.at`：`widget Icon(name, size, stroke_width)` 渲染
  svg 元素树；先做单图标 canary（BookOpen）经 `auto build --gen-only` 检查产物 svg
  元素/属性是否保留——若 .at UI 不支持 svg：数据层保持交付，icon.at 改为登记
  KNOWN-DEBT 的 stub（渲染待 auto-lang svg 节点能力），并在复审记录登记；验证：
  `auto build` 0 错 + 产物含 `<svg>`（或降级登记完成）。 [✅ 已完成（降级）] canary 实证 .at UI 不支持 svg：a2vue 把 svg/path 降解为 `<div :viewBox=...><div :d=.../>`（产物 gen/components/Icon.vue 摘录存证于 stub 头注）；icon.at = KNOWN-DEBT stub（含解除条件与恢复形态），KNOWN-DEBT-AND-RISKS.md 🟢 已知限制登记（Plan 038 条目，引 auto-lang 442 A4——其已按本 canary 结论排 svg 能力前置项）；auto build 0 错（stub 无消费方不入编译图，inert）
- [x] **T10** 新建 `scripts/lib-parity/icons.mjs`：@vue/server-renderer 对 lucide 原组件
  与 gen 编译 Icon 产物逐图标 renderToString，规范化（属性排序/布尔属性/自闭合）后
  diff；验证：`node scripts/lib-parity/icons.mjs` exit 0（全图标全等，或降级态下
  数据层 path 序列对 lucide dist 源数据全等）。 [✅ 已完成（降级态对拍）] 52/52 数据层元素序列（tag+attrs）对 lucide dist 源全等；对拍侧独立重述提取/规范化逻辑（非复用生成器代码）避免自证自明；renderToString 双端对拍挂起至 svg 能力就绪（T9 解除条件）

### Phase 3 — 渲染真源切 @autodown/vue

- [x] **T11** 接入 @autodown/vue：auto-down 侧 `pnpm build` 出 dist 后，musk web/
  package.json 增 `file:../auto-down/autodown/packages/vue` 依赖（若上游 npm 发包则
  改版本直依赖），`npm install` 解析成功；pac.at `npm_deps` 调整登记；验证：
  `cd web && npm run build` 绿 + `node -e "import('@autodown/vue')..."` 打印导出面
  （含 StreamingRenderer）。 [✅ 已完成] 接入方式现场裁定为 **vendor 快照**（计划默认 file: 直链被上游 `@autodown/core: workspace:*` 依赖阻塞——npm/pnpm 在 workspace 外均无法解析,实测确认）：scripts/vendor-autodown-vue.mjs 把 auto-down dist（实测新鲜,src 均旧于 dist）快照入 musk 仓库 vendor/@autodown/vue/（shim 仅声明 dist 实际外部化依赖 vue/markstream-vue/lowlight/hast-util-to-html,版本跟进=重跑脚本）；web `file:../vendor/@autodown/vue` + web/.npmrc install-links=true（npm 默认 symlink 化 file: 依赖,node/vite 从 vendor 真实路径解析 vue 失败,实测改复制安装）；npm install ✓、import 导出面 StreamingRenderer/StreamingTable/useStreamingDocument ✓、npm run build ✓；pac.at npm_deps 对象式登记（实测 auto-man 简写解析尾逗号缺陷,条目去逗号规避,见 pac.at 注释）
- [x] **T12** musk StreamingRenderer 逃生舱对齐上游：src/front/components/
  StreamingRenderer.vue 与 @autodown/vue 版差异勘察（codeBlockProps/lowlight/
  details/katex/placeholder），按 musk 现状最小对齐（保留 PrismCodeBlock
  setCustomComponents 注册路径，行为不变）；验证：`auto build` 0 错 + web vitest 绿。 [✅ 已完成] 差异勘察：musk 版=上游骨架（模板/segment 循环/registry 同构,useStreamingDocument 逐字节同源）,上游多出 MutationObserver 后处理（block id/placeholder/lowlight 高亮/code 块头/details 包裹）+ :::details 变换 + katex/mermaid 模块级启用 + codeBlockProps——对齐=收敛为上游 StreamingRenderer 再导出 + 保留 setCustomComponents(PrismCodeBlock)（实测依赖提升后单 markstream 实例,注册对上游内部生效）；上游增量样式经 inject_styles.ts 引入 '@autodown/vue/style.css'；超集差异由 T13 对拍白名单承接；auto build 0 错 + vitest 23 绿 ✓
- [x] **T13** ports/renderer.web.at 切换：`use.web component MarkdownRender from
  "markstream-vue"` → `"@autodown/vue"`（导出名以 T11 实测为准），gen 轨
  `auto build` + 新建 `scripts/lib-parity/render-switch.mjs`（切换前后对 fixtures
  内容 DOM 快照 diff，白名单差异显式登记）双绿；验证：`node scripts/lib-parity/render-switch.mjs`
  exit 0 + `cd gen/front/vue && pnpm build` 绿。 [✅ 已完成（1 项关联阻塞登记）] 实测 @autodown/vue 无 MarkdownRender 导出——新建 src/front/components/MarkdownRender.vue 适配器（content/final → source/streaming,端口消费面零改动）,端口改绑适配器;pac.at 恢复 markstream-vue 直依赖（.at 内置 markdown 元素为 auto-lang codegen 硬编码绑定+platform 注册仍直接引用,其消灭归 auto-down 008/auto-lang 跟进）;render-switch 对拍 **5/5 全等**（fixtures:真实 spec/plan 内容+构造边界+流式前缀+空串;白名单 W1 容器解包/W2 code 块头增量子树/W3 注释属性/W4 空态空壳,显式打印）✓;gen `pnpm install` ✓ + `pnpm exec vite build` ✓（打包链路全绿）——`pnpm build`(vue-tsc) 因 auto-lang CodeEditor 模板存量类型错被拦（setSearchEffect 不存在,与本次切换无关,登记待澄清 #10）
- [x] **T14** 起草 auto-down 侧计划草稿 `../auto-down/plans/008-render-autolang-markstream-elimination.md`
  （编号以现场 `ls ../auto-down/plans` 为准）：覆盖渲染层循 core a2ts 模式 .at 化、
  markstream-vue 内部消灭（解析层处置）、mermaid/katex 可选化与 VM 降级、编辑库
  （@autodown/editor）融合路线；musk 侧在本计划复审记录登记依赖关系与跟进点；
  验证：草稿文件存在且含上述四节。 [✅ 已完成] 草稿已存在（`008-render-autolang-markstream-elimination.md`,2026-08-23 立项并自引本计划 T14）,四节覆盖核验：Phase 1 a2ts .at 化 / Phase 2 markstream 消灭（markdown-it 语义子集对拍）/ Phase 3 可选化+VM 降级 / Phase 4 编辑库融合——全部在册;musk 侧依赖关系与跟进点已登记本计划复审记录「执行期登记」

### Phase 4 — 高亮方案对比与决策

- [x] **T15** 新建 `scripts/highlight-rs/`（cargo 小工程：syntext+two-face 版本对齐
  auto-lang/crates/auto-lang 的 Cargo.toml）输出 fixtures 代码块（11 语言，bash/sql
  重点实测）的 scope 序列 JSON → `scripts/lib-parity/fixtures/highlight/`；node 侧
  `scripts/lib-parity/highlight-compare.mjs` 汇出 prismjs/lowlight/syntect 三方
  token/scopes 一致性矩阵报告；验证：报告文件生成且覆盖 11 语言 × 3 方案。 [✅ 已完成] cargo 工程（syntect 5 + two-face 0.4,与 auto-lang 同版本同 feature;two-face API 实测为 `two_face::syntax::extra_newlines()`）输出 classed HTML;node 侧三引擎逐字符类别流（prism token 树/lowlight hast/syntect span 栈,HTML 实体解码修正长度口径）+ 近似映射表 → 矩阵报告;覆盖 **14 语言**（计划点名 11 + PrismCodeBlock 实际注册的 cpp/go）× 3 方案,长度校验全等;报告 fixtures/highlight/report.md + matrix.json;prismjs 对拍基准补装 web devDeps（计划技术栈声明项,web 轨源码零引用）
- [x] **T16** 决策落地（依据 T15 报告，默认推荐 (a)）：(a) VM 轨 syntect 原生——在
  本计划复审记录登记 auto-lang"只读高亮渲染原语"需求条目（指向 041 的
  highlight.rs 能力），vue 轨 prismjs 保留或换装依双轨差异实测；(b) 复刻——在本计划
  复审记录登记"prism 复刻"子计划建议（.at 正则能力门控前置）；(c) 降级——登记
  KNOWN-DEBT；验证：决策 + 数据引用写入复审记录，后续动作有明确归属
  （auto-lang 条目 / 子计划 / debt 登记）。 [✅ 已完成] 裁定 (a):VM 轨 syntect 原生 + vue 轨保留 prismjs——数据支撑与后续动作三归属(auto-lang 442 A5 只读高亮原语条目/vue 轨零改动/降级不采纳)全登记复审记录「T16 高亮决策登记」节;mermaid 不复刻决策一并落档(auto-down 008 Phase 3 承接)

## 复审记录

### 执行期登记（/auto-plan:work 会话,2026-08-23,worktree plan/038）

**T14 auto-down 侧依赖登记**：`../auto-down/plans/008-render-autolang-markstream-
elimination.md` 已存在（2026-08-23 立项,自引本计划 T14 为来源）,四节覆盖核验通过——
①渲染层 a2ts 模式 .at 化（Phase 1: useStreamingDocument/StreamingTable 迁 .at）；
②markstream-vue 内部消灭（Phase 2: 解析层锚定 markdown-it 语义子集对拍）；
③mermaid/katex/高亮可选化 + VM 降级（Phase 3,注明"musk T13 切换在此阶段完成后
[进一步] 内化"——本计划已完成消费侧切换,其消灭完成后再无 npm 传递链）；
④编辑库 @autodown/editor 融合路线（Phase 4,接 PLAN-041 T10）。
**musk 跟进点**：a) 008 待澄清 3（发包形态）——musk T11 现场已裁定 vendor 快照
（file: 直链被上游 `@autodown/core: workspace:*` 阻塞,npm/pnpm 均不可解析;
008 若定 npm 发包,musk 切版本直依赖并退役 vendor 脚本）；b) 008 Phase 1 对拍
fixtures 可复用本计划 `scripts/lib-parity/fixtures/render/`；c) 008 验收 4 要求
musk T13 端到端验证记录——本计划 T13 证据行即首个记录（render-switch 5/5）。
**遗留依赖（musk 侧 markstream 直依赖的最终消灭条件）**：①.at 内置 markdown 元素
（auto-lang codegen 硬编码绑定 markstream-vue,待 auto-lang 侧改绑 @autodown/vue
或平台化——已建议 auto-lang 442 承接）;②platform:markdown 的 setCustomComponents
注册 import（待 008 Phase 3 可选注册 API 落地后改走上游注册口）。

**Phase 收口台账**：Phase 0（T1✓/T2✓(裁定转责 auto-lang 442 P0-1)/T3✓,web vitest 23 绿
——存量 2 例改名遗留过时断言已最小修复:brandName 'AutoForge'→'Auto Musk'、DOM 测试
加 node 环境守卫 skipIf）;Phase 1（T4-T7✓,i18n 对拍 156/156）;Phase 2（T8-T10✓,
icons 数据对拍 52/52,渲染层降级登记 KNOWN-DEBT）;Phase 3（T11-T13✓,render-switch 5/5,
gen vite build 绿——vue-tsc 被 auto-lang 模板存量错拦截,裁定归 442 P0-2）;Phase 4
（T15-T16✓,14 语言 × 3 方案矩阵,决策 (a) 落档）。

**用户裁定补录（2026-08-23）**：待澄清 #9 → 选项 (ii)（auto-lang 按使用裁剪,
落 442 Phase 0 P0-1,不 gated）;待澄清 #10 → 归 442 Phase 0 P0-2（独立 phase,
不 gated）。两项裁定后 T2 转责收口,16/16 任务闭环。

**终验（2026-08-23 收口复跑,验收标准逐项）**：①auto build --gen-only exit 0/0 错;
②web vitest 23 绿+1 skip;③web npm run build 绿;④deps-guard exit 0（白名单补
auto-man 脚手架组件库 class-variance-authority/reka-ui——全量 build 按需生成
gen ui/Button 真实运行面,普查白名单缺口）;⑤i18n 156/156 + icons 52/52 +
render-switch 5/5 全等;⑥src/front/lib use.web grep 零命中（含注释口径）;⑦验收 4
之 gen 轨 grep 零命中 = auto-lang 442 P0-1 验收项（转责,musk 届时复核并清
TRANSITIONAL 区）;⑧验收 6 之 gen pnpm build（vue-tsc）= 442 P0-2 验收项
（转责,打包链路 vite build 现已绿）。

**T16 高亮决策登记（2026-08-23,依据 T15 矩阵）**：裁定 **(a) VM 轨 syntect 原生 +
vue 轨保留 prismjs**。数据支撑（scripts/lib-parity/fixtures/highlight/report.md,
14 语言 × 3 方案,逐字符类别流两两一致率,近似映射口径）：prism–lowlight 71.3% /
prism–syntect 60.2% / lowlight–syntect 58.6%——**任两引擎 token 级一致均不可达
（≤71%）**,换装 lowlight 不改善跨轨一致性（58.6% ≈ 60.2%）,故 vue 轨零改动保留
prismjs;重点实测语言：bash 三方 52.8-62.8%（Sublime bash 语法可用,无缺失）,
sql 可用且 lowlight–syntect 89.7%（l–s 最高对）,toml/ts 的 p–s 偏低（33-44%,
prism 语法粒度差异所致,视觉近似不受影响）。**后续动作归属**：
①auto-lang 侧需求条目——"只读高亮渲染原语"（041 code_editor 的 highlight.rs/
two-face 内核暴露 highlight-only API 或 code_editor 只读模式;auto-lang 442 A5
已预留按本条目定接口——VM 轨消费面 = markdown code_block 只读渲染）;
②vue 轨 prismjs 保留,双轨差异容忍度 = 视觉近似（char 级类别一致率 60-71%,
关键词/字符串/注释大类基本一致）——若未来要求 token 级跨轨一致,唯一路径是
(b) 复刻（.at 正则能力门控前置,本计划不展开）;
③(c) 降级不采纳（syntect 能力已实证存在,无降级必要）。
mermaid 决策一并落档（目标 6）：不复刻——平台端口 + VM 轨降级渲染路径由
auto-down 008 Phase 3 承接（katex/mermaid/highlight 可选注册 + 降级）。

（正式复审如下；以上执行期登记保留为过程证据。）

### 正式复审（/auto-plan:review,2026-08-23,worktree plan/038 @ 4d6a2a2）

**方式**：7 项验收标准逐条对实际代码重跑（不信已勾选框）；分支 diff main..HEAD
（2 commits,61 files,+5645/-133）与计划声明一致,无未声明改动。

| # | 验收标准 | 裁定 | 证据（本次复跑） |
|---|---|---|---|
| 1 | 纯 .at 实现或降级登记 + lib 零 use.web + 对拍全绿 | **PASS** | grep src/front/lib use.web 零命中;i18n parity 156/156;icons data parity 52/52;svg 降级登记于 KNOWN-DEBT-AND-RISKS（Plan 038 行,引 auto-lang 442 A4） |
| 2 | ports/renderer 消费 @autodown/vue + 对拍无回归 + pac.at 同步 + auto-down 草稿四节 | **PASS** | 端口绑 src/front/components/MarkdownRender.vue→内部 import @autodown/vue;render-switch 5/5 归一全等（白名单 W1-W4 显式打印）;pac.at npm_deps 对象式双条目;auto-down 008 含 a2ts/markstream 消灭/可选化降级/editor 四节 |
| 3 | 高亮对比报告落复审记录 + 决策三选一有数据支撑 | **PASS** | report.md+matrix.json 在库（14 语言×3 方案,长度校验全等）;决策 (a) + 数据引用 + 三归属登记于「T16 高亮决策登记」 |
| 4 | marked 移除 + gen 轨未用依赖清除 | **PASS（后半转责）** | grep marked web/package.json = 0;gen 清理经用户裁定转责 auto-lang 442 P0-1（登记核验 3 处命中,含双向验收）;musk deps-guard TRANSITIONAL 区待其落地清零 |
| 5 | deps-guard 落地 exit 0 | **PASS** | 复跑 exit 0;codemirror 系过渡放行单列打印并指向 442 P0-1 |
| 6 | 双轨零回归（auto build+vitest+web build+render-switch） | **PASS（gen vue-tsc 转责）** | auto build --gen-only exit 0/0 错;vitest 23 绿+1 skip;web npm run build 绿;render-switch 5/5;gen pnpm install+vite build 绿——vue-tsc 拦截项经裁定转责 442 P0-2 |
| 7 | mermaid 不复刻 + 库归属决策经确认并记录 | **PASS** | 两项决策记录于执行期登记;两项执行期阻塞（#9/#10）经用户显式裁定入册;mermaid/库归属按计划推荐默认执行,用户阅执行摘要后指示继续,无异议记录 |

**偏差与数字修正（plan 文本 vs 实测,均已记入任务证据行,无实质漂移）**：i18n 键数
普查 81 → 实测 .at 真源 72/语言（web 轨副本 357/语言,auto-i18n 以真源为范围）;icons
并集 ~44 → 实测 52（web 多行 import 初扫漏 12 个,被生成器自证核对机制拦下补全）;
高亮 11 语言 → 实测 PrismCodeBlock 注册 14 语言（矩阵全覆盖）。

**Debt 候选（均已有归属,非阻塞）**：
1. icons 渲染层降级（.at UI 无 svg 节点）——KNOWN-DEBT 已登记,解除条件 auto-lang 442 A4;
2. deps-guard TRANSITIONAL codemirror 放行——442 P0-1 落地清零;
3. gen vue-tsc（CodeEditor setSearchEffect）——442 P0-2;
4. auto-man npm_deps 简写解析尾逗号缺陷——musk 侧 pac.at 无逗号规避+注释,可归
   442 P0-1 顺手修复或忽略（影响面仅对象式 npm_deps 带尾逗号写法）;
5. .at 内置 markdown 元素硬编码 markstream-vue（auto-lang codegen）——musk 无法
   端口化,消灭条件见 T14 执行期登记"遗留依赖"条（auto-lang 改绑/auto-down 008
   Phase 3 注册口）。

**结论**：7/7 PASS（其中 #4 后半、#6 之 gen vue-tsc 为用户裁定转责项,归属与验收
登记完备）。路由 → **reviewed**,可入 /auto-plan:merge。

## 待澄清事项

1. **mermaid 不复刻**（平台端口 + VM 轨降级代码块）——体量与收益失衡的判断请确认；
   注意 @autodown/vue 内部同样依赖 mermaid + katex，其处置（可选化/降级）已列入 T14
   草稿范围。
2. **.at UI svg 元素支持**决定 icons 渲染层形态（T9 canary 实测定夺；数据层无风险）。
3. **@autodown/vue 接入方式**：默认 file: 链接（需 auto-down 侧 dist 构建约定）；
   若倾向 npm 发包请在 T11 前告知（草稿中已列为建议项）。
4. **高亮决策倾向**：当前推荐 (a) VM 轨 syntect 原生（041 已证能力）+ vue 轨视差异
   保留 prismjs；双轨高亮不一致的容忍度（视觉近似 vs token 级一致）请确认。
5. **编辑库融合时机**：@autodown/editor（Tiptap）与 musk 编辑器场景（web 原生轨
   editors/ + TestEditor/AutoDownEditor）的融合是独立体量，建议渲染切换稳定后另立
   plan（已在 T14 草稿中排路线），本计划不展开——请确认。
6. **cn()/tailwind-merge**：VM 轨样式模型未定，暂不复刻；若 VM 轨需保留 tailwind 类
   合并语义需另立计划。
7. **接线边界**：i18n/icons 纯 .at 库本计划止于"对拍绿 + vue 轨可用证明"，ports 直绑
   替换 npm 绑定属跨平台架构迁移完成后的接入步骤——边界划分请确认。
8. **auto-down 侧计划归属**：T14 草稿在 auto-down 仓 plans/ 落地（其计划体系独立），
   本计划只登记依赖与跟进——若希望 musk 侧直接代执行 auto-down 任务请告知。
9. **[T2 阻塞] gen 轨 package.json 模板源在 auto-lang 共享仓**（2026-08-23 执行实测）：
   模板 = `../auto-lang/crates/auto-man/src/vue.rs` generate_package_json，依赖硬编码、
   仅 router/i18n/npm_deps 条件化；musk src/front 对 10 项待删依赖+toast 零引用已实证，
   但模板为全 auto-lang 生态共享（widgets-gallery toast 演示等真实消费 vue-sonner），且
   musk 的 `auto` 为 cargo 全局安装二进制，改模板须重编译安装才生效；与本计划技术栈
   "auto-lang 只读"声明冲突。三选项请裁定：
   (i) 跨仓直改 auto-lang 模板 + cargo 重编译（生态级影响，需 auto-lang 侧评审，
   auto-lang 442 已 gated 于本计划，可顺势承接）；
   (ii) auto-lang 侧做"按使用裁剪"条件化依赖（属 auto-lang 442 范畴，本计划仅登记需求）；
   (iii) musk 侧后处理脚本在 auto build 后裁剪 gen/front/vue/package.json（musk 范围内
   达成 grep 零命中验收，不动模板，偏离 T2 字面"从模板删除"）。
   **✅ 已裁定（2026-08-23）**：选项 (ii)——auto-lang 侧"按使用裁剪"条件化依赖，
   已登记为 auto-lang 442 Phase 0 P0-1（不 gated 可先行,含 musk grep 零命中 +
   widgets-gallery 不回归双验收）；本计划 T2 据此转责收口。
10. **[T13 关联] auto-lang CodeEditor 模板存量类型错误**（2026-08-23 执行实测）：
    `auto-man/src/vue.rs:270` 模板生成 `import { setSearchEffect } from
    '@codemirror/search'`——该 API 在 @codemirror/search@6 实际导出面不存在
    （有 setSearchQuery 无 setSearchEffect）。主 checkout gen 因"增量保留"沿用旧版
    CodeEditor.vue 未暴露；**任何新鲜脚手架（fresh checkout/auto build 全量）的
    `pnpm build`（vue-tsc）必炸**，与 T2 同属 auto-lang 模板债。musk 侧已证：
    `pnpm exec vite build`（打包链路,消费 @autodown/vue file: 依赖与全部切换产物）
    全绿；仅 vue-tsc 因该死文件类型错拦截。处置建议归 auto-lang（修模板 import 或
    setSearchQuery 等价改写），修复后 musk 侧 `pnpm build` 应恢复全绿。
    **✅ 已裁定（2026-08-23）**：归 auto-lang 442 修复——已登记为 442 **Phase 0
    P0-2**（独立 phase,不 gated 可先行）。
