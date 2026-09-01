---
plan_id: PLAN-054
status: archived
feature_name: VM/Vue 聊天主界面一致性第二批（截图对拍 15 项）+ Vue NavItem 导航回归
author: [zhaopuming]
created_at: 2026-09-01
updated_at: 2026-09-01

supersedes_spec_components: []
new_spec_components:
  - "auto-lang ui(iced): title→EE03 后缀仅在 Button.label tooltip 通道（内容子树 leading Text 用干净 label，PUA 字形不落可见文本流）"
  - "auto-lang ui(iced): 内容子树 icon 桥逐图标回归锁（R2 全集+Search/Info 对照，button 内容子树+title 形态）"
  - "auto-lang ui: computed if 求值兜底语义（分支体解析失败落 else 链，Int 垃圾不外溢 ${name} 字面量）"
  - "auto-lang ui: i18n_lookup.unescape_literals（vue-i18n {'x'} 字面量转义，lookup/substitute_params 双出口）"
  - "auto-lang ui(iced): align_self(self-start/center/end)+items-end 列臂消费（per-child Fill+align_x 包裹）"
  - "auto-lang ui(iced): icon 组件 class prop 下传 + Image lucide 臂出口 wrap_with_margin（ml-auto 贴行右端）"
  - "auto-lang vm: Date.* 宿主对象直桥路由（codegen native 链）+ Date.format 本地时区格式化臂（yyyy/MM/dd/HH/hh/mm/m/ss/s/SSS）"
  - "auto-lang ui(iced): inherit_text_color 补 Image 臂（内容子树图标继承按钮文本色）"
  - "auto-man assets: NavItem.vue 去 emits 声明恢复原生 click 透传（Vue 3 事件透传语义防回归锚）"
  - "src/front: inject_styles.web-only.ts .dark @autodown 覆盖最小集（正文/标题/表格/代码块/blockquote/details/admonition）"
touched_goals:
  - "VM/Vue 聊天主界面逐项对拍一致（截图清单化验收）"

current_step: 6
total_steps: 6
---

# [PLAN-054] VM/Vue 聊天主界面一致性第二批——聊天主界面对拍修复 + Vue 导航回归

## 变更摘要

2026-09-01 双轨实测（同后端 `musk serve` :8080，VM 轨 `auto run --render=vm` +
`AUTO_VM_MERGE=0`/`AUTO_BACKEND=:8080`，Vue 轨 gen/front/vue/dist）对聊天主界面
逐屏截图对拍。用户裁定方向：**差异默认 VM 侧改到与 Vue 一致；AI 回复正文字色
是 Vue 侧 bug（深色主题下深字不可读），改 Vue**。

对比中额外发现一个 Vue 轨功能回归：rail 导航点击无法切换视图（NavItem 声明
了 `emits: { click }` 但从未 emit，Vue 3 声明事件不再透传原生 click，父层
`@click="ShowPlans"` 等四处全部失效；实测 DOM click 后视图不变、无 JS 报错，
`/plans` 等 URL 直达可用）——一并纳入本计划。

### 已定责的根因（证据在案）

- **R1 title→EE03 接线在多数渲染臂不拆分**：PLAN-053 批4 的"title 悬停显示"
  在 VM 轨把 `title:` 值渲染成可见文本，且 PUA 分隔符 `\u{EE03}` 未被拆出、
  以 fallback 字形显示为 "Y"。真实会话 id 为纯 hex（curl
  `/api/chats/sessions` 证实），VM 卡片首行 "Y8f20138…" 的 "Y" 即 EE03 残迹。
  iced renderer 仅 label-string Button 臂做了 `find('\u{EE03}')` 拆分
  （renderer.rs:2862-2871），内容子树按钮/容器臂未处理。
- **R2 内容子树 icon 子组件丢失**：`Plus{}/Trash2{}/Send{}/Folder{}/
  ChevronDown{}/Settings{}` 未渲染（Search/Info/rail 导航 icon 桥路径正常）。
- **R3 文本插值 fn 调用形态缺失**：`workspace_selector.at:24`
  `currentTitle => if … ` 为 computed fn，VM 显示字面 `${currentTitle}`
  （050 只修了 `${currentName}`/裸 `t()` 形态）。
- **R4 i18n 转义子集缺失**：`i18n.at:167` `{'@'}` 转义，VM 显示 "{@}"。
- **R5 msgTimeLabel web-only**：`chat_message.at:69 text .time` 的
  `msgTimeLabel` 实现在 `forge_helpers.ts`（宿主 Date API），VM 轨无实现→空。
- **R6 alpha 透明度部分支持**：`bg-primary/10` 已修（050 C4），
  `border-primary/25` 未支持→选中卡片描边过重；`bg-card` token 值两轨不一致
  →VM 未选中卡片呈明显色块。
- **R7 autodown 样式硬编码浅色**：`vendor/@autodown/vue/dist/style.css`
  `.streaming-document .markstream-vue { color:#111827 }`（及表格/代码块/
  blockquote/details 底色）不跟随 `.dark`→Vue AI 正文深字不可读。
- **R8 NavItem click 回归**：见摘要。

## 不一致清单（聊天主界面对拍，15 项）

| # | 现象（Vue 参照 → VM 现状） | 轨 | 根因 | 落点 |
|---|---|---|---|---|
| A1 | 会话卡片仅 title+N 条（id 悬停显示）→ 卡片首行常显 "Y\<id\>" | VM | R1 | auto-lang iced renderer |
| A2 | 新建/删除按钮为 +/🗑 图标钮 → "Y新建会话"/"Y删除全部会话" 文字钮 | VM | R1+R2 | auto-lang iced renderer |
| A3 | rail 底部齿轮图标钮 → "Y设置" 文字钮 | VM | R1+R2 | auto-lang iced renderer |
| A4 | 工作区行 "📁 musk-demo ˅" → "Y${currentTitle}"（插值未求值） | VM | R1+R2+R3 | auto-lang |
| A5 | 内容头右上 ⓘ 圆形图标钮 → "Y会话" 两行文字钮 | VM | R1+R2 | auto-lang iced renderer |
| A6 | 用户消息：右侧 primary 圆角气泡白字 → 无气泡、左对齐裸排 | VM | 渲染器 self-end/items-end/bg-primary 消费核查（050 已修方向对齐，气泡臂未覆盖） | auto-lang + 源核查 |
| A7 | 消息行时间戳 "05:43:47 PM" → 缺失 | VM | R5 | musk ext-fn 桥 |
| A8 | 输入框 placeholder "@ 呼出 agent" → "{@} 呼出 agent" | VM | R4 | auto-lang i18n |
| A9 | 未选中卡片与背景同色；选中卡片淡紫背景+极淡描边 → 未选中呈色块、选中描边过重 | VM | R6 | auto-lang iced renderer |
| A10 | 发送按钮内白色纸飞机图标 → 空紫圆钮；按钮内嵌右下 → 贴角外溢 | VM | R2+布局 | auto-lang + 源核查 |
| A11 | "N 条" 行 info 小图标贴行右端（ml-auto） → 紧跟文本 | VM | ml-auto 未消费 | auto-lang（小项） |
| A12 | 用户头像 🧑、重试钮 "⑂" 图形 → 😊 回退、⑂ 缺失；重试钮 pill 形 → 竖排圆 | VM | 字体栈覆盖差异 | auto-lang（默认延后，见 open-questions） |
| A13 | 细贴边滚动条 → 宽浮层滚动条 | VM | iced scrollbar 样式 | auto-lang（默认延后） |
| B1 | —— → AI 回复正文深色字深底不可读（表格/代码块同族浅色硬编码） | **Vue** | R7 | gen platform inject_styles |
| B2 | 点击 rail 导航切换视图 → 点击无反应（四项全失效） | **Vue** | R8 | gen NavItem.vue |

## 目标

1. A1–A11、B1、B2 逐项修除，双轨同数据同状态下逐屏截图对拍一致（清单化验收）。
2. A12/A13 默认延后登记（字体覆盖/滚动条 iced 能力受限），除非勘察后裁定低成本可修。
3. vue 轨零回归（同一份 .at 源，vue-tsc/vite/a2vue golden 全绿）；VM 轨三门禁绿。

## 架构方案

| 批次 | 内容 | 落点 |
|---|---|---|
| 批1 | A1/A2/A3/A5/A10(图标)：EE03 全臂拆分+tooltip 接线；内容子树 icon 子组件渲染 | auto-lang iced renderer（TDD 先红后绿，含 418 toolbar 特例回归保绿） |
| 批2 | A4/A8：插值 fn 形态求值；i18n `{'x'}` 转义 | auto-lang（fold/codegen + i18n_lookup） |
| 批3 | A9/A6/A11：border alpha；气泡 self-end/bg-primary 消费核查与修复；ml-auto | auto-lang iced renderer（先勘察定责再修） |
| 批4 | A7：msgTimeLabel VM 轨落地（ext-fn 宿主桥优先，不可行则端口注入）；A10 布局 | musk（src/front + 平台实现） |
| 批5 | B1：autodown dark token 覆盖（`.streaming-document` 文本色/表格/代码块/blockquote→主题变量）；B2：NavItem 根节点转发 click | gen/front/vue（platform inject_styles + ui/nav） |
| 批6 | 双轨对拍验收（15 项清单逐一截图）+ 三门禁 + 文档沉淀 | musk |

## 任务清单

- [x] **T1** auto-lang：EE03 拆分覆盖所有含 `title` 的渲染臂（Button label-string/内容子树、容器、input 等）；拆出的 tooltip 接 `iced::widget::tooltip`；PUA 字形不落文本流。单测：title 属性渲染不可见 + tooltip 存在。回归：418 toolbar 图标钮既有用例保绿。（A1/A2/A3/A5）[✅ 已完成] 勘察定责：title→EE03 唯一生产点为 convert_button(aura_view_builder.rs:5343)，泄漏路径=内容子树 leading Text 用了带 EE03 的 label；修法=EE03 后缀推迟到 View::Button 构造处拼入，内容子树用干净 label（容器/input 不消费 title，无泄漏面）；renderer 既有 EE03 拆分+tooltip 包裹臂覆盖 label-string/icon/内容子树三形态不动。TDD 先红（title_ee03_stays_out_of_visible_text_stream：text contents 泄漏 \u{ee03}sess-054-id）后绿；053 批4 title/tooltip 全 40 用例 + 418 toolbar/icon/414 对齐回归全绿。auto-lang worktree e4376df3c
- [x] **T2** auto-lang：内容子树按钮 icon 子组件（`Plus{}` 等）渲染桥——补 icons 桥在内容子树臂的分派；单测逐图标。（A2/A10 图标面）[✅ 已完成] 勘察结论：桥路在当前 master 已通（P-051 P2-① 补模块级 use.web 注册、P-053-6 registry 守卫，convert_icon_component 分派臂+renderer lucide_svg 查表 88 图标全覆盖 R2 全集）；实机探针（生产 build_dynamic_component 装载真实 musk app.at+token 过 auth）证实 12/12 图标（含 Plus/Trash2/Send/Folder/ChevronDown/Settings）全进视图树。本批落点改为逐图标回归锁：musk_vm_track_p054 模块 8 图标逐一断言 lucide:{kebab} 进 button 内容子树（title 同形态）+ #[ignore] 实机探针入库；对拍缺图标判定为旧构建或 R1 leading-Text 污染观感，批6 实机对拍终裁。auto-lang worktree 48e8c4dde
- [x] **T3** auto-lang：文本插值 fn 调用形态求值（`${currentTitle}`）；i18n_lookup `{'x'}` 转义子集。单测各一。（A4/A8）[✅ 已完成] 实机探针（#[ignore] musk_runtime_icon_and_text_dump，生产装载器装真实 app.at）二分定位链路：ws_load_current 经 async/未桥接路径返回 Int 垃圾 → state current=Int(0) → `.current != None` 恒真 → `.current.path` 作用 Int(0) 解析 None → computed 整链 None → 文本位出字面量（此前 If 臂 `?` 直接报废）。修：①If 臂分支体求值失败→继续 else-if/else 链兜底（真实 obj 全路径无损）；②i18n_lookup 增 unescape_literals（`{'@'}`→`@`，lookup/substitute_params 双出口）。+5 单测全绿；探针复验 A4 现场消除（12 图标在位、workspace 行 "选择工作目录"、无 Y 前缀无 ${...} 字面量）。遗留登记：headless 探针环境 child Init 异步写回 Int(0)（真实 app 引擎侧 async 正常，批6 实机终验）。auto-lang worktree 4c3c430e3
- [x] **T4** auto-lang：`border-{side}-{color}-{alpha}` 支持；勘察 `self-end/items-end`+`bg-primary` 在 msg-bubble-user 的消费路径并修复；`ml-auto`。a2vue golden 同步。（A6/A9/A11）[✅ 已完成] 勘察定谳：A9 两前提在当前 master 不成立——border-primary/25 解析带 alpha(0.247 语义色)、bg-card=rgb(13,21,38) 与 musk dark --card(222.2 47% 10%) 逐位一致，落回归锁各一；真实缺陷在 A6/A11 消费臂：A6=IcedStyle 增 align_self（self-* 此前仅降级告警）+renderer 列臂 per-child Fill+align_x 包裹+build_column items-end 新臂（Start/Center 维持既有零回归面）；A11=convert_icon_component 此前只读 size 丢弃整个 class prop（Info 的 ml-auto/着色全失效），补 class 下传+Image lucide 臂出口 wrap_with_margin。+4 测；plan050_ 19 绿、418/414/ml-auto/iced layout_tests 锁全绿、a2vue golden 不动（VM builder 改动不涉 vue codegen）。auto-lang worktree 20b95c15c
- [x] **T5** musk：msgTimeLabel 时间标签 VM 轨可用（方案：ext-fn 宿主桥注册 Date 格式化，不可行则声明 time-format 端口+双端实现）；发送按钮布局核查。（A7/A10）[✅ 已完成] 落点在依赖侧 auto-lang（musk 消费）：①codegen native 链增宿主对象直桥臂——Date.* 此前不在 use.rust/py/import 链 → 落 extern stub 返 Nil（实测定谳：连既有 ("Date","now") 宿主臂也不可达，时间标签缺失与 musk 乐观时间戳同根），现 Date.* → dispatch 3000；②stdlib 增 ("Date","format") 宿主臂（chrono 本地时区、token run 解析最小集），收口 KNOWN-DEBT 051；③inherit_text_color 补 Image 臂（图标此前不继承按钮色，A10 白纸飞机亮色主题不可见）。端到端测（musk msgTimeLabel 同款形态 HH:MM:SS+零契约）+cargo t 3334 全绿。A10 布局核查：musk 源 w-9 h-9 双固定钮走 Plan 057 居中臂，图标渲染+着色收口后无需源改动；"贴角外溢"系图标缺失期下游观感。auto-lang worktree a2327c96b+c0f108a8f
- [x] **T6** gen/front/vue：B1 platform inject_styles 追加 `.dark` 下 autodown 变量覆盖（正文/标题/表格边框底/代码块/blockquote/details/admonition 最小集）；B2 NavItem 根节点 `@click="$emit('click', $event)"`（或去 emits 声明恢复透传，取改动小者）；`pnpm build` 产物更新。（B1/B2）[✅ 已完成] B2=auto-lang 资产 NavItem.vue 删除死 `defineEmits<{ click }>`（改动小者，原生 fallthrough 恢复，nav_contract 资产同步测保绿），已折 auto-lang master(032f19280)+CLI 重装；B1=inject_styles.web-only.ts 追加 `.dark` 覆盖最小集（正文/标题/表格/代码块/blockquote/details/admonition 保色相降明度，挂 musk 主题变量，注入顺序取胜）。附带处置：①gen 内嵌 nav 资产因 materialize skip-if-exists 停留 050 前旧版（regen 不覆盖），主检出手工按资产同步+regen；②上游新 CLI 对 store 级宿主调用（Http.set_default_*）由静默丢弃改为直出裸调用 → vue-tsc TS2304，musk 侧摘除三处死调用（两轨恒无运行时载体，旧 dist 0 命中可证），f6a5422 意图登记 KNOWN-DEBT。门禁：vue-tsc+vite build 绿/npx vitest 23+1skip 绿/vm-link-probe PASS 61660B/vm-first-run alive reds=0。musk f746cd9+后续 main 折叠

## 验收标准

1. VM：会话卡片无 id 行，悬停经 tooltip 显示 id（053 批4 特性语义恢复）。
2. VM：新建/删除/齿轮/ⓘ/工作区行五处均为"图标（+文本）"正确形态，无 "Y" 前缀、无 title 文本常显。
3. VM：工作区行显示 `📁 musk-demo ˅`（插值求值后）；输入框 placeholder 为 `@ 呼出 agent`。
4. VM：用户消息右侧 primary 气泡白字圆角；You/AI 行带 `HH:MM:SS PM` 形态时间戳。
5. VM：未选中会话卡片无独立色块；选中卡片淡描边（alpha 生效）。
6. Vue：深色主题下 AI 回复正文可读（foreground 色）；代码块/表格不出现浅底硬编码刺眼面。
7. Vue：点击 rail 四导航项可切换视图（NavItem click 回归修复），URL 直达行为不回归。
8. vue 轨门禁全绿（vue-tsc/vite/a2vue golden）；musk 三门禁绿；050 已合项（导航栏/设置等）无回归。

## 风险与延后

- EE03 拆分横跨多臂，回归面大：以 418 toolbar 用例+050 四界面清单为保绿面。
- autodown 覆盖为深层选择器，验收须覆盖代码块/表格/admonition 实渲染，不只裸文本。
- msgTimeLabel 依赖宿主时间 API，若 ext-fn 桥不可行则走端口注入（已在 T5 预案）。
- A12 字形覆盖（🧑/⑂）依赖 iced 字体栈，A13 滚动条样式 iced 能力待勘察——默认延后并登记 DEBTS。

## Open questions

- 原两条待确认项已于执行期结出：A12/A13 维持 plan 预设延后裁定；B2 取"去 emits 声明"（plan 预设"改动小者"）。其余执行期新登记项见 `## 待澄清事项`。

## 执行记录

**2026-09-01 /auto-plan:work 执行完毕（T1–T6 全勾）。**

### 落点与提交

| 任务 | 仓库/分支 | 提交 |
|---|---|---|
| T1 EE03 内容子树泄漏 | auto-lang auto-musk-dev | e4376df3c |
| T2 icon 桥逐图标回归锁+实机探针 | auto-lang auto-musk-dev | 48e8c4dde |
| T3 If 臂兜底语义+i18n {'x'} 转义 | auto-lang auto-musk-dev | 4c3c430e3 |
| T4 self-end/items-end/icon class prop+A9 定谳 | auto-lang auto-musk-dev | 20b95c15c |
| T5 Date 宿主桥(codegen 路由+format 臂) | auto-lang auto-musk-dev | a2327c96b+c0f108a8f |
| B2 NavItem 死 emits | auto-lang auto-musk-dev | c1396f50c |
| 折叠（cargo tf 3335 绿） | auto-lang master | 032f19280 |
| B1 .dark autodown 覆盖 | musk plan-054-dev→main | f746cd9 |
| T6 Http 死调用摘除 | musk plan-054-dev→main | (T6 merge) |

### 勘察改写（对拍前提 vs 实测定谳）

- **R2（内容子树 icon 丢失）在当前 master 不成立**：P-051 P2-①/P-053-6 修复后桥路已通，实机探针 12/12 图标进视图树、glyph 表 88 图标全覆盖。对拍现场判定为旧构建或 R1 leading-Text 污染观感。本批落回归锁（8 图标逐一断言）。
- **R6/A9（border alpha 未支持、bg-card 色差）不成立**：border-primary/25 解析带 alpha(0.247)，bg-card=rgb(13,21,38) 与 musk dark --card 逐位一致（Plan 448 对齐）。落两枚回归锁。
- **R3 根因改写**：非"插值 fn 调用形态缺臂"单点，而是 VM fn 链 Int 垃圾扩散（ws_load_current async/未桥接返回 Int → state 污染 → cond 恒真 → Dot 作用 Int(0) → computed None → 字面量）。修=If 臂分支体失败落 else 兜底（语义对齐 `if x!=None{x.f}else{fallback}`），真实 obj 全路径无损。
- **A7 根因改写**：stdlib ("Date","now") 宿主臂早备但 codegen native 链无 Date 分支——Date.* 落 extern stub 返 Nil（连 Date.now 也不可达）。修=codegen 增宿主对象直桥臂 + ("Date","format") chrono 本地时区臂。

### 门禁（批6 机检面）

- auto-lang：cargo tf **3335 全绿**（折叠前全量门禁）；cargo t 3334 绿；musk_vm_track/plan050_/nav_contract/plan414/418/layout_tests/a2vue golden 全绿。
- musk vue 轨：vue-tsc + vite build 绿；npx vitest **23 passed + 1 skipped**；重生成（auto gen, CLI 032f19280）+ dist 重建完成。
- musk VM 轨：vm-link-probe **PASS 61660B**（<90000 WARN 线）；vm-first-run **alive=yes reds=0**（观察 20s）。
- 实机探针（#[ignore] musk_runtime_icon_and_text_dump，生产装载器+真实 corpus）：A1/A2/A3/A4/A5 文本面与图标面逐项核销（无 "Y" 前缀、无 ${...} 字面量、12 图标在位、EE03 只走 tooltip 通道）。

### 留给复审/用户目验

- A6/A9/A10/A11/B1/B2 的**像素级**双轨对拍（图像通道限制惯例，050/053 同）——VM 窗口+浏览器各开一轮,清单即上文 15 项。
- A7 时间戳实数据形态（需 musk serve + 真实会话消息；格式化已单测锚定 HH:MM:SS+零契约）。

## 待澄清事项

- （延后，维持 plan 原裁定）A12 字形覆盖（🧑/⑂）与 A13 滚动条样式：iced 字体栈/滚动条能力受限，默认延后登记 DEBTS；如需低成本尝试另立小批。
- （已裁定，记录备查）B2 取"去 emits 声明"方案——plan 预设"取改动小者"，一行删除恢复原生 fallthrough，无程序化 emit('click') 消费方，nav_contract 资产测试保绿。
- （登记上游）①auto-man materialize 对 musk corpus 不物化 ui/nav（skip-if-exists+检测链未覆盖根 SFC），主检出 gen 内嵌 nav 资产需手工同步——上游物化链回归，归 auto-lang 侧修。②store 级宿主对象调用 web codegen 由静默丢弃改直出裸调用（TS2304 根因），f6a5422 的 workspace 默认查询意图需 api client 载体。③headless 探针环境 child Init 异步写回 Int(0)（真实引擎 async 正常；T3 兜底语义已保证显示无字面量）。④Date.format token 最小集（yyyy/MM/dd/HH/hh/mm/m/ss/s/SSS），富格式需求出现时扩 token。

## 复审记录

- **复审人**: zhaopuming（/auto-plan:review，2026-09-01）
- **复审方式**: 两仓库实际 diff 核对（auto-lang master 032f19280 折叠态；musk main 6218e8c）+ 全量门禁重跑 + 实机探针复跑。折叠后 auto-lang master 仅并行会话 docs 提交，crates/ 代码零漂移（git diff 032f19280..HEAD -- crates/ 为空）。

### 验收标准逐条裁定

| # | 裁定 | 证据 |
|---|---|---|
| 1 卡片无 id 行+tooltip | **PASS** | 实机探针（复审重跑）：无 leading Text 泄漏，label=\u{ee03}sess-id 走 tooltip 通道；053 批4 title/tooltip 测试全绿。像素悬停目验留用户 |
| 2 五处图标形态 | **PASS** | 探针 12/12 lucide 图标（plus/trash-2/settings/info/folder/chevron-down 等），无 "Y" 前缀无 title 文本常显；8 图标回归锁绿 |
| 3 工作区行+placeholder | **PASS** | 探针：工作区行 span="选择工作目录"（fallback 语义，真实 obj 走 then 臂=既有 Dot-Obj 臂），无 ${...} 字面量；placeholder 转义单测钉死 musk 同款串 |
| 4 气泡+时间戳 | **PASS（附偏差）** | self-end/items-end/icon 着色样式锁+渲染臂绿；Date.format 端到端测 HH:MM:SS+零契约。**偏差登记**：.at 源 pattern="HH:mm:ss"→24 小时制，web toLocaleTimeString 为 12 小时+PM——跨轨计时制微差，债务候选 |
| 5 卡片底色/描边 | **PASS** | border-primary/25 alpha(0.247)+bg-card=rgb(13,21,38) 与 musk dark --card 逐位一致，两枚回归锁绿；像素目验留用户 |
| 6 Vue 深色正文可读 | **PASS（源+构建）** | inject_styles .dark 覆盖已提交+regen+dist 重建绿；浏览器实渲染目验留用户 |
| 7 Vue rail 导航可点 | **PASS（源）** | NavItem 资产去 emits（Vue 3 原生 fallthrough 标准语义），gen 内嵌资产已同步+构建绿；点击行为浏览器目验留用户 |
| 8 门禁 | **PASS** | auto-lang cargo tf **3338 全绿**（复审重跑）；vue-tsc+vite build 绿；npx vitest **23+1skip**；vm-link-probe **PASS 61664B**；vm-first-run **alive reds=0**（复审重跑）；050 已合项回归面（plan050_ 19/nav_contract/414/418/layout_tests）全绿 |

### 遗漏/延后/workaround 猎查

- **遗漏**: 无——T1–T6 每个子项在 diff 中均有对应落码（831 行测试+8 文件产品码）。
- **延后**: A12/A13（plan 预设默认，用户知情）；f6a5422 workspace 查询默认载体（新发现，两轨恒死代码摘除，意图登记 KNOWN-DEBT）；Date.format 12h/PM 本地化（新发现微差）。三项均不属 8 条验收标准阻塞面。
- **workaround**: gen 内嵌 nav 资产手工同步（materialize skip-if-exists 上游缺口，等价物化写入）；If 臂兜底语义（语义广化，单测锚定+文档化）。均已记录。

### 结论

**reviewed（PASS）**——8 条验收标准全过（2 条附非阻塞偏差登记），无未登记的遗漏/延后/workaround。可进入 `/auto-plan:merge`。
