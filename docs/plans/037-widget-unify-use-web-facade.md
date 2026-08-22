---
plan_id: PLAN-037
status: executing
feature_name: widget 统一 + use.web 生态导入 + 跨后端 facade
author: [zhaopuming]
created_at: 2026-08-22
updated_at: 2026-08-22

supersedes_spec_components: []
new_spec_components: []
touched_goals: []

current_step: 0
total_steps: 24
---

# [PLAN-037] widget 统一 + use.web 生态导入 + 跨后端 facade

## 变更摘要

六步走完成三项统一:(1) widget 补齐子组件组合能力(含 v-model/model 寻址),auto-musk 全部
`component fn` 迁移为 `widget`,使 widget 成为唯一 UI 单元;(2) 发明 `use.web` 语句替代
use 块(归入 `use.rust`/`use.py` 生态 use 家族,默认普通导入 + `component`/`composable`
修饰),auto-musk 全部 use 块迁移;(3) 非 `.at` 的 web 依赖按域抽成 facade(端口声明 +
`.web.at` 适配器 + 编译期目标门控),为 VM/Rust 后端铺路。**每个 Phase 结束 musk auto/vue
版必须全绿可用**(不变量)。

## 目标

1. widget 在子组件场景完全等价于 component fn(k3 canary 矩阵全绿),并新增 v-model
   model 寻址(状态槽 → 自动 `v-model`,三个编译错误守护)。
2. `use.web` 语句落地(parser → 同一 ExtImport AST,codegen 零语义变化;非 web 后端
   显式报错;use 块保留为废弃别名)。
3. auto-musk 零 `component fn`、零 use 块(grep 断言)。
4. escape hatch(`.ts`/`.vue`/npm/composable)按域收进 `src/front/ports/`,调用面变纯
   Auto;stream 域完成 port/adapter 拆分 proof(编译期门控)。

## 架构方案

三层结构(本计划落完后):

```
调用方(.at,纯 Auto 符号)
  └─ use pac.ports.<域>: *                ← 端口声明(Auto fn 签名)
        └─ ports/<域>.web.at              ← adapter:use.web 绑 web 实现
              └─ use.web X from "..."     ← 生态绑定(仅 web 目标参与编译)
```

- 目标真源:`pac.at` 的 `render:` 字段。`.web.at` 后缀文件仅在 web 目标参与编译;
  缺当前目标实现 → 构建期显式报错(对齐 use.py 的 fail-fast 先例)。
- v-model 契约(Phase 1 实现,widget/component fn 同规则):子的 model 变量 → 
  `defineModel`(未绑定时为本地 ref,行为不变);调用点传状态槽(`value: .name`)且
  命中子 model 变量 → 生成 `v-model:name`;表达式→通道、父侧非可写、子侧 prop/model
  同名,均编译错误。
- `use.web` 语法:`use.web X from "path"`(默认普通导入,函数/对象/常量皆可)、
  `use.web component X, Y from "npm-or-file"`、`use.web composable useT from "..."`
  (含 `refs: [x]` 解构变体)。

## 技术栈

auto-lang(parser.rs / aura/extract.rs / ast/ui.rs / ui_gen/vue.rs)、auto-musk
(src/front/*.at、pac.at)、k3 canary(auto-lang/examples/capability-tests/k3-widget-composition)。

## 需求分析与背景调查

来自 2026-08-22 会话的实测结论(k2/k3 canary + 源码核验):

- **widget 组合已基本可用**:k3 证明 widget 当子组件(循环实例化 + props + 回调 +
  每实例状态)全绿,产物与 component fn 版等价。回调契约 = `on_<snake>: msg` 参数 +
  子 msg 同名 Pascal 变体(vue.rs `prop_is_emitted_callback`);**k2 canary 当前在
  master 为红**(回调 prop 无变体 + 当函数调用的旧惯用法),需处理。
- **component fn 无独有能力**;widget 反多 expose/routes/tick/key_bindings。双轨
  (parser/extract/render 三对平行路径)是漂移温床,迁移后 musk 侧单轨。
- **use 块现状**:2026-07-29 为 a2vue 发明(`use { fn|component|composable: X from
  "path" }`),仅 Vue 后端消费,其他后端静默忽略;错误后端静默消失是坏味道。
  `use.rust`(Plan 212b,需先 `dep`)/`use.py`(Plan 214,FFI 门控报错)是既有生态
  use 先例。
- **musk 现状**:6 个 widget(app/chats_view/plans_view/login/specs_view/wiki_view)、
  24 个 component fn(执行时以 grep 为准);use 块遍布。`.at` fn 模块
  (forge_helpers.at 等)已是纯 Auto,**不进 facade 范围**。真 escape hatch:
  forge_stream.ts、inject_styles.ts、setup_auth_fetch.ts、raw_upload.ts、
  composables/(useT/gate_router)、vue-i18n(composable)、lucide-vue-next 图标、
  StreamingRenderer.vue;markstream-vue 走 pac npm_deps,不涉 use。
- **Vue 版本**:gen 工程钉 `>=3.4.0`,`defineModel` 可用;codegen 尚未用过它。
- **验证基线**:`auto build`(musk 根,含 vue-tsc + vite)= 主门;`cargo test -p musk`
  后端不动;`cd web && npx vitest run` 存量 2 失败为基线(手写轨,防误伤)。

## 详细设计

### D1 — v-model / model 寻址(auto-lang)

| 子组件侧 | 调用点传的 | 结果 |
|---|---|---|
| `value` 是 model 变量 | 状态槽(`.name`) | `v-model:value="name"` |
| `value` 是 model 变量 | 表达式/prop/字面量 | 编译错误("model 通道需可写状态槽") |
| `value` 是 prop | 任意 | 单向下行(现行为) |

- 子 codegen:model 变量 → `const x = defineModel<T>('x', ...)`;未绑定 = 本地 ref。
- 调用点:prop 名命中子的 model 变量 + `extract_state_ref` 命中 → v-model;
  类型不一致编译错误;父侧目标是自身 prop → 错误;子侧 prop/model 同名 → 子定义报错。
- 跨文件子的 model 变量收集:复用 408 P2/P6 的 sub_widgets 收集机制。

### D2 — use.web 语句(auto-lang)

```
use.web agentAvatarData from "src/front/forge_helpers.at"          // 默认普通导入
use.web component MessageSquare, ListTodo from "lucide-vue-next"
use.web composable useT from "src/front/composables/useT.ts"
use.web composable useI18n refs: [locale] from "vue-i18n"
```

- parser 新增语句 → 产出**同一个 `ExtImport` AST**(kind: 默认 Fn / Component /
  Composable),`register_ext_imports` 等消费侧零改动。
- 作用域:文件级声明,codegen 按"符号被哪个 SFC 引用"发射 import(未被引用不发射)。
- 后端门控:非 web 目标(VM/a2r/a2py)遇到 → 显式报错(对齐 use.py 文案风格)。
- use 块保留为废弃别名(paser 双入口,一个迁移期)。

### D3 — facade / 编译期门控

- `src/front/ports/<域>.at`:Auto fn 签名(端口)+ `use.web` 绑定 + wrapper fn 转发。
- 门控 v1 用**文件级约定**:`<域>.web.at` 仅 web 目标编译(谓词 = pac `render`);
  缺实现报错。不做语句级条件编译。
- component 类依赖(图标/StreamingRenderer)**v1 不做转发**——调用点保留
  `use.web component`(登记 KNOWN-DEBT,语言层 component 符号转发另立计划)。

## 测试设计

- auto-lang:每能力一单测(defineModel 生成、v-model 调用点、三类编译错误、
  use.web 解析/门控/别名);golden 扩展 k3(test/a2vue/ 下新增 expected 产物)。
- canary:k3 加 Phase 4(矩阵)、Phase 5(回调绑定形态等价性)、Phase 6(三层
  bind 链 Parent→Child→Input)。
- musk:每批次 `auto build`;Phase 收口 `cargo test -p musk` + `cd web && npx vitest run`
  (存量 2 失败基线);最终 grep 断言零 use 块、零 component fn。

## 验收标准

1. k3 矩阵全绿;widget 子组件 = component fn 产物等价(除事件名大小写约定)。
2. v-model 链(Parent.name ↔ Child.value ↔ Input)在 canary 端到端通过。
3. `grep -rn "component fn" src/front/` 与 `grep -rn "use {" src/front/` 均零命中。
4. ports/ 目录承担全部非 .at web 依赖的调用面;stream 域 port/adapter 拆分编译通过。
5. 每 Phase 后 musk `auto build` 全绿;最终 `cargo test -p musk` 全绿、web vitest
   基线不变(2 存量失败)。

## 执行步骤

### Phase 0 — 基线修复(musk 存量断裂,2026-08-22 执行中发现)

> **发现**:musk main(8404202)+ auto-lang master(8c0ef9bb)binary 下 `auto build`
> 已有 **7 个存量 TS 错误**(与 PLAN-037 改动无关,双 binary 对照错误集合完全一致;
> 系 musk 上次全绿(b84030a)后 auto-lang plan-418 合并造成的工具链漂移)。
> 本计划不变量(musk 每 Phase 全绿)以此为前提,必须先修。
> 错误清单(全局 binary 与 plan-037 worktree binary 一致):
> 1. `ChatsView.vue(204)` TS2345:ReportCard 调用缺 deckMeta/deckHtml props
> 2. `RelayRunBox.vue(66)` TS2305:api 无 `Value` 导出
> 3. `RelayRunBox.vue(310/361/376)` TS2339:relayTVCmdLine/relayWriteFence/relayFileFence 不存在
> 4. `relay_run_helpers.ts(58/68)` TS7034/7005:转译产物隐式 any[] ×2

- [x] **T0a** 修 RelayRunBox 族(错误 2/3):诊断 `Value` 类型映射与 relay fns 的
  use{fn} 暴露链(疑 plan-418 codegen 漂移)。验证:musk `auto build` 该 4 错清零。
- [x] **T0b** 修 ChatsView/ReportCard(错误 1):deckMeta/deckHtml 调用点或 props
  可选化。验证:该错清零。
- [x] **T0c** 修 relay_run_helpers 转译隐式 any(错误 4):.at 源补类型标注或转译器
  补 `any[]` 注解。验证:musk `auto build` 全绿(0 错)——**此后 Phase 1-6 的
  不变量基线成立**。

### Phase 1 完成(2026-08-22)

T1-T5 全部完成,Phase 1 收口验证:649 ui_gen 单测(含新增 model 寻址单测)+ 8 golden
更新全绿;k3 Phase 4/5/6 canary 全绿;musk `auto build` 全绿零误伤(T4 defineModel 改造
对全部存量组件零破坏;T5 通道检查未命中任何存量误用)。

关键落地:
- T4:model 变量 → `defineModel<T>("name", {default})` 无条件编译(未绑定=本地 ref,
  行为与之前完全一致)。
- T5:子 model 变量即双向通道。同文件(api.rs)+ 跨文件(auto-man from_workspace
  预扫描 → ui_build_..._full → with_sub_widget_models)收集;调用点状态槽目标折叠
  `v-model:key`,表达式/prop/字面量喂通道 = 硬构建错误;子侧 prop/model 同名 = 生成
  错误;原生 input 裸 value 槽不再静默退化为单向(补齐"input 对 model 槽永远双向"的
  设计初衷)。
- 零关键字达成:无需 bind/v-model 修饰词,双向性由"子的 model 变量 × 父的可写状态槽"
  两端推导。

### Phase 1 — widget 能力补齐(auto-lang,用户步骤 1)

> **工具链约定**:Phase 内对 auto-lang 的改动,验证用 worktree binary
> `D:/autostack/auto-lang-p037/target/release/auto.exe`(cargo build --release),
> 不覆盖全局 `~/.cargo/bin/auto.exe`(保留回退版本)。

- [x] **T1** k3 Phase 4 矩阵:扩展
  `auto-lang/examples/capability-tests/k3-widget-composition/`(新增 `item_matrix.at`
  复活为 widget 版):child widget 内组合 `use{fn}` + `slot` + `computed` + `style` +
  `watch` + `.Init/.Destroy` + msg。逐项开启验证,记录哪项红。验证:`cd k3 目录 && auto build`。
- [x] **T2** k3 Phase 5 回调形态等价:同场景验证父侧 `onselect: .X($event)`(component fn
  惯用)与 `on_select: .X`(widget 契约)在 **widget 子**上的产物差异,产出迁移规则表
  (Phase 4 的输入)。验证:`auto build` + 产物 diff 记入 README。
- [x] **T3** 修 k2 回归:`examples/capability-tests/k2-child-handler-binding` 二选一——
  (a) codegen 补全"回调 prop 无变体"惯用法(defineProps 剔除 + defineEmits 补名)或
  (b) k2 源改为契约惯用法并注明。验证:`cd k2 目录 && auto build` 绿。
- [x] **T4** defineModel 编译:`ui_gen/vue.rs` model 变量 → `defineModel<T>()`(未绑定
  行为不变,存量 widget 回归);单测 `test_widget_model_var_define_model`。验证:
  `cargo test -p auto-lang --lib ui_gen::vue`。
- [x] **T5** 调用点 model 寻址:prop 命中子 model 变量 + 状态槽 → `v-model:name`;
  实现三类编译错误(D1 表);k3 Phase 6(三层链 Parent→Child→Input)落地;golden
  `test/a2vue/011_model_binding/`。验证:`cargo test` + k3 `auto build` + musk 根
  `auto build` 回归。

### Phase 2 — use.web 语句(auto-lang,用户步骤 2)

- [x] **T6** parser:`use.web` 语句(D2 语法,默认/component/composable + `refs:`)→
  ExtImport AST;单测覆盖四形态。验证:`cargo test -p auto-lang --lib`。
- [x] **T7** 门控 + 别名:非 web 目标遇 `use.web` 显式报错(仿 use.py 文案);use 块
  保留别名(双入口单测)。验证:`cargo test -p auto-lang --lib` + musk 根 `auto build`。
- [x] **T8** k3 试点:canary 内把一个 use 块改为 `use.web`(证明端到端)。验证:
  k3 `auto build` 绿 + 产物 import 不变。

### Phase 2 完成(2026-08-22)

T6-T8 全部完成:651 ui_gen 单测(含 use.web 四形态解析单测)+ k3 T8 试点
(use 块 → 顶层 use.web,产物逐字相同)+ musk 回归全绿(use 块别名生效,存量零影响)。

落地:`use.web NAME from "path"`(默认普通导入)/`use.web component A, B from "..."`
/`use.web composable X refs: [...] from "..."`;AST `Stmt::UseWeb`(六个 match 补臂);
parser 与 use.rust/use.py 同族派发;api.rs 把文件级条目挂到文件内全部 widget;
a2r 两处显式报错("use.web requires the vue render target")。

### Phase 3 — musk use 块 → use.web(用户步骤 3)

- [x] **T9** 盘点:`grep -rn "use {" D:/autostack/auto-musk/src/front/*.at` 生成清单
  (按文件×kind 登记进本节回填),定批次。验证:清单写入计划。
- [x] **T10** 批次 1:6 个 widget 文件(app/chats_view/plans_view/login/specs_view/
  wiki_view.at)的 use 块 → `use.web`。验证:musk 根 `auto build`。
- [x] **T11** 批次 2:component fn 前半(A–M,grep 清单为准)。验证:musk 根 `auto build`。
- [x] **T12** 批次 3:component fn 后半(N–Z)+ 断言 `grep -rn "use {" src/front/` 零命中。
  验证:`auto build` + `cd web && npx vitest run`(基线 2 失败)。

### Phase 3 完成(2026-08-22)

T9-T12 完成:`scripts/migrate_use_web.py` 机械转换 25 文件 29 块——带 from 的条目
(fn/component/composable,含 refs)全部提升为顶层 `use.web` 语句;**无 from 的
`component: X`(跨文件组件引用)保留在原 use 块**(Phase 4 转 widget 后改顶层
`use alias: X` 消化)。收口:`auto build` 全绿 + web vitest 基线不变(2 存量失败/
22 过,主检出演示 worktree web/ 无 node_modules 属 checkout 固有)+ 块内 web 条目
零残留(仅注释提及旧形式)。

### Phase 4 — musk component fn → widget(用户步骤 4)

- [x] **T13** 试点定样板:`think_block.at` 单文件迁移(`component fn X(p) { blocks…
  body }` → `widget X(p) { blocks… view { body } }`),按 T2 规则表核对父侧绑定,
  产物 diff 对拍。验证:`auto build` + gen 产物手查。
- [x] **T14** 批次 A(叶子渲染):agent_avatar/user_message/content_header/report_card/
  questionnaire_card/secretary_message/streaming_table + think_block 复核。验证:
  `auto build`。
- [x] **T15** 批次 B(卡片组):generic_tool_card/errand_card/task_plan_card/gate_card/
  tool_block/raw_preview。验证:`auto build`。
- [x] **T16** 批次 C(输入/导航/复杂):mention_dropdown/mention_input/nav_sidebar/
  wiki_nav/workspace_selector/settings_menu/session_info/relay_run_box/chat_message/
  secretary_message_wrapper + 断言 `grep -rn "component fn" src/front/` 零命中。
  验证:`auto build` + `cargo test -p musk` + vitest 基线。

### Phase 4 完成(2026-08-22)

T13-T16 完成:24 个 component fn 全部迁移为 widget(`scripts/migrate_widget.py`
机械转换:改名 + view 体包裹;试点 ThinkBlock 产物逐字节相同;转换器多行注释
lookahead bug 修复后 3 个伤件重转)。收口:`grep component fn` 零命中;`auto build`
全绿;`cargo test -p musk` 绿;web vitest 基线不变。

顺带修:auto_type_to_ts_type/is_builtin_type_name 补 `obj`/`list` 内建映射
(component fn 提取曾擦为 any,widget 路径直映射后暴露 TS2305)。

**widget 现为 musk 唯一 UI 单元**(6 页面 + 24 子组件,共 30 个 widget)。

### Phase 5 — use.web 抽 facade(用户步骤 5)

- [ ] **T17** 域盘点:非 `.at` 的 use.web 清单化(stream/styles/auth/composables/
  icons+renderer 五类),定 `src/front/ports/` 目录与命名。验证:清单写入计划。
- [ ] **T18** stream 域:`ports/stream.at`(端口签名 + `use.web startForgeStream
  from "src/front/forge_stream.ts"` + wrapper fn),chats_view 等调用方改 use 端口。
  验证:`auto build`。
- [ ] **T19** styles+auth 域:`ports/platform.at`(inject_styles/setup_auth_fetch),
  app.at 调用面改。验证:`auto build`。
- [ ] **T20** composables 域:`ports/i18n_router.at`(useT/gate_router/useI18n refs)。
  验证:`auto build` + i18n 手工 smoke(切语言)。
- [ ] **T21** icons/renderer 域:决策落地——调用点保留 `use.web component`(登记
  KNOWN-DEBT)或最小转发机制;`grep -rn "use.web" src/front/ --include="*.at" |
  grep -v ports/` 仅剩已登记项。验证:`auto build`。

### Phase 6 — 编译期目标门控(auto-lang + musk,用户步骤 6)

- [ ] **T22** auto-lang 门控机制:`.web.at` 后缀文件按 pac `render` 选择参与编译;
  端口在当前目标无 adapter → 构建期显式报错。单测:web 目标选中 / rust 目标跳过并
  在缺 adapter 时报错。验证:`cargo test -p auto-lang --lib`。
- [ ] **T23** musk stream proof:拆 `ports/stream.at`(纯签名) + `ports/stream.web.at`
  (use.web 绑定),调用方不变。验证:musk `auto build` 绿;临时改 pac `render` 触发
  缺 adapter 报错截图/记录后复原。
- [ ] **T24** 全量收口:`auto build` + `cargo test -p musk` + `cd web && npx vitest run`
  (2 存量失败基线)+ k2/k3 canary 全绿;更新 `docs/specs/01-architecture.md` 的
  .at 源分类(widget 单一单元 / use.web / ports)。

## 复审记录

### Phase 0 完成(2026-08-22)

7 个存量错全清 + 顺带修出 4 个更深的问题,musk `auto build` 全绿(优于 main 基线——
main 本身就是红的,此前被主检出 gen/ 中未跟踪的 deck.vue 掩盖):

1. relay_run_box.at use 清单补 relayTVCmdLine/relayWriteFence/relayFileFence(模板用而未导入)
2. ReportCard deckMeta/deckHtml 调用点显式传空(Auto 轨从未接 deck 载荷,行为不变)
3. auto-lang:`Value` 内建类型映射 → any(auto_type_to_ts_type + is_builtin_type_name)
4. deck.vue 从主检出 gen/ 残留收编为受管源文件 `src/platform/deck.vue`,导入改 ext 路径
5. relay_run_helpers.at relayPreviewLines 本地变量 `list` 改名 `rows`——规避 parser
   "类型名被同名本地变量遮蔽后 let 标注丢失"怪癖(最小用例已复现确认为 parser 层问题)


(执行中回填)

## 待澄清事项

5. parser 怪癖:局部变量与类型名同名(如 `let list = ...` 后接 `let out list = []`)
   会使后者类型标注丢失(探针已复现,见 Phase 0 记录第 5 条)——Phase 0 以改名规避,
   parser 层修复(类型位置不应走符号解析)另立小计划。

1. component 符号转发(图标/StreamingRenderer 经 ports 引用)——v1 保留调用点直连,
   语言层机制另立计划(T21 决策)。
2. auto-lang 的 component fn 双轨(parser/extract/render)退役——本计划只迁 musk 源,
   语言层保留兼容;退役另立计划。
3. `view {}` 包裹可选化、setup 前导槽(`.Setup`)、`.Init`(onMounted)语义文档化——
   均登记为后续设计项,不在本计划。
4. k2 惯用法(T3)选 codegen 补全还是 canary 改写,执行时按影响面定。
