---
plan_id: PLAN-061
status: archived
feature_name: Auto/vue 轨实测走查缺陷修复批次
author: [zhaopuming]
created_at: 2026-09-04T00:00:00+08:00
updated_at: 2026-09-05T11:15:00+08:00

# Leave these EMPTY here — /auto-plan:review fills them:
supersedes_spec_components: []
new_spec_components: []
touched_goals:
  - "goal-frontend-parity: Auto/vue 生产轨实测缺陷批次清偿——A 组 14 项全落地(D1 结构化视图/D2 登录容错/D3 密码语义/D5-D8/D16 i18n 门禁/D17-D22 交互与排版/D25 叶子渲染),全部修在 .at 单源,双轨同步受益"
  - "goal-spec-knowledge: specs 结构化视图消费面修复(D1 大小写归一,92→103 items 前端可见),ledger 双落点的 UI 消费端恢复"

current_step: 19
total_steps: 19
---

# [PLAN-061] Auto/vue 轨实测走查缺陷修复批次

## 变更摘要

2026-09-04 对 gen/front/vue(Auto/vue 生产轨,dev :3334)做了整轨实测走查
(登录 → 会话/计划/规范/知识库/设置逐页 + 源码层归属核对 + 性能探针实测),
登记 15 项发现;同日用户实机复审补充 6 项 UI 缺陷(D17–D22,含截图)。
本批计划把**确证的功能/交互缺陷(A 组,13 项)**落成原子修复任务(全部改
`src/front/*.at` 源、ext 手写层或 vitest 测试,经 `auto run` 重生成生效);
**产品/安全取舍项(B 组,5 项)**待用户裁定后并入;**生成器上游项
(C 组,6 项)**登记归 auto-lang,不在本批修。

用户将继续复审并补充更多待修点,补充项按 D 编号顺延并入 A/B 组后再开工。

## 目标

1. 规范(specs)结构化视图能显示后端真实数据(实测 92 条被大小写 bug 吞掉)。
2. 登录失败有错误提示、按钮不再卡死;密码框按密码语义显示;敏感信息不落盘。
3. 知识库/规范界面文案与 i18n key 对齐,中英文混杂收口一批。
4. 建立"模板 t() key ⊆ locale keys"回归门禁,防止 i18n 缺 key 再漏网。
5. 修复 HTML 规范违例(button 嵌 button)与 localStorage 对象直存。
6. 聊天区消息头/工具栏排版修正(时间戳行内居中、copy 移左);会话列表与
   主导航 hover 语义统一为中性灰;设置弹层支持点外关闭;新建会话后自动
   聚焦输入框。

## 架构方案

不动架构。全部修复落在既有分层内,核心约束是**层归属**:

| 层 | 路径 | 是否受重生成覆盖 | 本批动作 |
|:---|:---|:---|:---|
| Auto 源 | `src/front/*.at`、`src/front/i18n/*.json` | 源(手写) | **改这里** |
| 生成产物 | `gen/front/vue/src/components/**`、`stores/**`、`locales/**`、`ext/src/front/specs_helpers.ts` 等 | 每次 `auto run` 重生成覆盖 | 只核对,不手改 |
| ext 手写层 | `gen/front/vue/src/ext/src/front/**`(web-only 留守,KNOWN-DEBT 029 D 组)、`src/__tests__/**`、`src/front/ports/**` | 不覆盖 | 测试/桥接/CSS 兜底可改 |

注意:`ext/src/front/specs_helpers.ts` 头注标明 "Auto-generated from .at fn
module by AutoUI (Plan 028 M1)",虽位于 ext 树但**受重生成覆盖**,根修必须在
`src/front/specs_helpers.at`。

两个已证实的生成器行为约束(修复方案均据此设计,上游根修见 C 组):

- **动态 style 绑定缺陷**:元素绑 computed 类串时,codegen 产出 `:style="…"`
  (Tailwind 类串进 style 属性 = 非法 CSS 被浏览器整条丢弃);静态 style 串
  才会正确并入 `class="…"`。实证:`components/ChatMessage.vue:65-66`
  `:style="headerClass"`/`:style="badgeClass"`(失效)vs `:107` 静态串正常。
  .at 侧规避 = 把分支性类串落成常量串、按条件渲染不同元素。
- **shadcn Button variant 泄漏**:`Button.vue` 经 `cn()`(tailwind-merge)
  合并 variant 与传入类串,**传入串未覆盖的 hover:*/focus:* 组会保留默认
  variant 值**(default = `bg-primary … hover:bg-primary/90`)。"选中态"
  类串若不带 hover: 覆盖,hover 时就漏出实心 primary。

## 技术栈

- Auto(.at)→ Vue3 + Vite + tailwind(生成):`auto run` 管线(AutoUI)
- vitest(组件内 `gen/front/vue/src/__tests__/`,现 2 个 spec)
- 门禁:`scripts/vm-link-probe.cmd`(改任何 front 源后必跑,见
  `src/front/README.at-conventions.md`);`cd gen/front/vue && pnpm build`
  (vue-tsc strict);`pnpm vitest run`
- 后端联调:`backend/crates/musk`(admin/admin,musk-demo workspace)

## 需求分析与背景调查

- spec 概览(specs v1,92 items):goals 9 / architecture 7 / designs 8 /
  tests 8 / reviews 36 / reports 24。规范后端数据完备,前端结构化视图
  因 D1 全部显示为空,是本批最高价值修复。
- 实测方法与证据:2026-09-04 会话内走查(登录页/四视图/设置菜单截图与
  DOM 快照在案),MutationObserver/fetch/longtask/rAF 探针实测,流式轮询
  分桶采样。性能结论:**Vue 轨无重渲染风暴**(空闲 37.6s 零变更零请求;
  PollStream 拉取期每秒 DOM 变更为 0,回复到达一次性批量渲染 9 mutation)。
  自动化 locator 点击超时为 IAB 后台停帧(rAF 计数恒 0)所致,非应用债,
  不立项。
- 用户实机复审(2026-09-04,四张截图):消息头时间戳排版、AI 工具栏位置、
  选中会话 hover 实心 indigo、主导航无 hover、设置弹层点外不关、新建会话
  不聚焦 → D17–D22,根因均已定位到源码行级(见登记册)。
- 与 KNOWN-DEBT 059-FU1 的边界:5498 次 Init 重入是 VM(iced)轨观察,
  根修已归 auto-lang PLAN-536;本计划实测确认 Vue 轨不受此病影响,
  不重复立项。
- 生成 DSL 已知约束(执行时规避,见 R1/R3 与 KNOWN-DEBT 029):store 内
  try 块内嵌 if-return 不支持(029 缺口③);DSL 属性透传对原生
  input 无 `password` 语义(login.at 的 `password: true` 即由此失效);
  裸 window 全局禁止,DOM 副作用一律走 `ports/platform.at` 逃生舱。

## 详细设计(缺陷登记册)

### A 组:确证功能/交互缺陷(本批执行,见执行步骤 T1–T18)

- **D1 规范结构化视图恒空**
  - 证据:后端 `GET /api/specs?workspace=musk-demo` 返回 92 条(goals 9),
    UI 各节均显示 "No goals yet / No items yet";整页刷新不复现好转
    (排除时序)。
  - 根因:`src/front/specs_helpers.at:328` `specSectionItems` 以
    `section.section_type == section_type` 全等比较,后端返回
    `"Goals"`(首字母大写),前端传 `'goals'`(小写),永不匹配。
  - 修法:比较前两侧归一化小写(或在 .at 内建 section id 映射)。
  - 层:`src/front/specs_helpers.at` → 产物 `ext/src/front/specs_helpers.ts`。

- **D2 登录失败静默卡死**
  - 证据:实测(后端不可达语义等价路径)按钮卡 "Loading..." 永不复位;
    store 的 `error` ref 全程无赋值点,LoginPage 的错误横幅为死代码。
  - 根因:`src/front/auth_store.at` Login/Register 无 try/catch/finally;
    `lib/api.ts` 的 auth_login 对 !ok 抛错,异常吞掉后 loading 停留 true。
  - 修法:await 包 try/catch,catch 中 `SetError(用户可读文案)`;
    loading 复位放 finally 语义位。规避 DSL"try 内嵌 if return 不支持"
    (029 缺口③):catch 内不做 return,只赋值。
  - 层:`src/front/auth_store.at` → 产物 `stores/useAuthStoreStore.ts`。

- **D3 密码框明文显示**
  - 证据:登录页输入密码可见(截图在案);产物
    `components/LoginPage.vue` 模板为 `:password="true"`(对原生 input
    无效绑定),无 `type="password"`。
  - 根因:`src/front/login.at:64` 以 `password: true` 透传属性,DSL/
    codegen 无该语义。
  - 修法:改为产出 `type="password"`(优先 DSL 可表达的 attr 名;若
    codegen input 映射不支持 `type` 透传,则该任务记 blocker 转
    auto-lang,本批先落 D2/D8)。
  - 层:`src/front/login.at` → 产物 `components/LoginPage.vue`。

- **D5 知识库按钮显示原始 i18n key**
  - 证据:页面右上角渲染 "wiki.edit" / "wiki.delete" 字面量。
  - 根因:`src/front/wiki_view.at:222/227` 用 `t("wiki.edit")` /
    `t("wiki.delete")`;`src/front/i18n/{zh,en}.json` 的 wiki 节只有
    `editPage`/`deletePage`,无 `edit`/`delete`。
  - 修法:在 `src/front/i18n/zh.json` 与 `en.json` 的 wiki 节补
    `"edit"`/`"delete"` 两 key(中文 "编辑"/"删除",英文 "Edit"/"Delete");
    不改模板 key(语义更通用)。
  - 层:`src/front/i18n/*.json`(源)→ 产物 `locales/*.json`。

- **D6 计划列表项文字两侧被裁切**
  - 证据:计划侧栏列宽 ~210px,"PLAN-018 — SSE取證 [ reviewed" 左右均被
    硬裁,无省略号。
  - 根因:`src/front/plans_view.at` 列表项容器无 min-w/ellipsis 类串。
  - 修法:列表项文本容器补 truncate 形态类串(`min-w-0 truncate`,以
    DSL 类串可表达为准)。
  - 层:`src/front/plans_view.at` → 产物 `components/PlansView.vue`。

- **D7 WikiNav button 嵌 button(HTML 规范违例)**
  - 证据:vite 每次编译告警 "<button> cannot be child of <button>";
    浏览器解析器会提前闭合外层 button,内层删除钮交互/布局不可靠。
  - 根因:`src/front/wiki_nav.at` 树节点行外层为 button,行内删除钮
    仍是 button(产物 `components/WikiNav.vue`)。
  - 修法:内层删除钮改非 button 形态(span + 点击 + stop propagation,
    以 DSL 可表达为准;若 DSL 强制 button 则记 blocker 转 auto-lang)。
  - 层:`src/front/wiki_nav.at` → 产物 `components/WikiNav.vue`。

- **D8 musk_user 落盘成 "[object Object]"**
  - 证据:`stores/useAuthStoreStore.ts` 中
    `localStorage.setItem('musk_user', user.value)`(对象直存),
    Init 读回为字符串。
  - 根因:`src/front/auth_store.at:65/84` setItem 未 JSON 化。
  - 修法:写侧 `JSON.stringify`,读侧 parse + 守卫(parse 失败弃用)。
  - 层:`src/front/auth_store.at` → 产物 `stores/useAuthStoreStore.ts`。

- **D16 i18n 缺 key 无回归门禁**
  - 证据:`wiki.edit`/`wiki.delete` 漏网(D5)未被现有 2 个测试抓住。
  - 修法:`gen/front/vue/src/__tests__/i18n.spec.ts`(或新 spec)加静态
    断言:扫描生成产物 `components/**/*.vue` 模板中的
    `t('<section>.<key>')` 字面量,集合 ⊆ `locales/{zh,en}.json` keys。
    纯 ext/测试层,不依赖重生成。
  - 层:`gen/front/vue/src/__tests__/`(不覆盖,可直接改)。

- **D17 消息头时间戳悬空、与角色名无间距**(用户截图 1)
  - 证据:消息头渲染为 "🤖 AI07:54:18 PM"——时间与名字无间距且
    时间戳纵向偏高(基线对齐而非居中);用户与 AI 消息同病。
  - 根因:**codegen 动态 style 绑定缺陷**(见架构方案)。
    `chat_message.at` 头行 `row { style: .headerClass }`(
    `headerClass` = "flex items-center gap-2 px-1 [justify-end]")是
    computed 串,产物 `ChatMessage.vue:65` 发成
    `:style="headerClass"`,整条类串被浏览器丢弃,行只剩静态
    `class="flex flex-row"`(无 items-center/gap-2);`badgeClass`
    同病(`:style` 于 `:66`)。静态 style 串的元素(如工具栏行
    `:107`)则正常并入 class=。
  - 修法:.at 层规避——头行按 `isUser` 分支拆成两个 row,各用**静态**
    style 串(用户行含 justify-end);badge 颜色同样分支静态化。
    修好后 items-center(时间戳行内居中)+ gap-2(与名字间距)自然
    恢复。上游根修登记 C 组 D24。
  - 层:`src/front/chat_message.at:88-95` → 产物 `components/ChatMessage.vue`。

- **D18 AI 消息工具栏 copy 靠右、侵入下方消息区**(用户截图 1)
  - 证据:AI 消息底部 copy 按钮行 `justify-end`,图标贴右缘,视觉上
    挤进下一条消息区域;用户裁定移左。
  - 根因:`src/front/chat_message.at:157` 工具栏行静态串
    `"flex items-center justify-end gap-1 mt-[2px] px-[3px]"`
    (.at 注释"对齐 web 原版 message-toolbar:copy 靠右"——设计裁定更新)。
  - 修法:`justify-end` → `justify-start`(静态串,产出可靠);单源
    修改,VM 轨同步生效。
  - 层:`src/front/chat_message.at:157`。

- **D19 选中会话项 hover 变实心 indigo、文字不可读**(用户截图 2/3)
  - 证据:选中项("Block 全家福演示")hover 时整块变实心 primary 底色;
    非选中项 hover 为正常灰色(bg-accent)。
  - 根因:**shadcn Button variant 泄漏**(见架构方案)。会话项是
    `<Button>`,default variant 自带 `hover:bg-primary/90`;选中分支
    类串(`chats_view.at:186`)只有静态 `bg-primary/10 border-primary/25
    text-primary`,**无 hover:* 覆盖**,tailwind-merge 保留 variant 的
    `hover:bg-primary/90` → hover 实心 indigo。非选中分支自带
    `hover:bg-accent` 把它顶掉,故正常。
  - 修法:选中分支类串补 `hover:bg-accent`(与用户要求"和普通项一样
    的灰色高亮"一致)。同批扫描其余"active 分支无 hover: 的 Button":
    `nav_item.at:28`(NavListItem 计划列表选中态)、
    `specs_view.at:301`(规范节导航选中态)同病同修。
  - 层:`src/front/chats_view.at`、`src/front/nav_item.at`、
    `src/front/specs_view.at` → 产物 `ChatsView.vue` 等。

- **D20 主导航(active 项)无 hover 高亮**(用户反馈 4)
  - 证据:`components/ui/nav/NavItem.vue` 契约注释明示 "Active items
    never carry hover classes (build-time either/or) … mirrors the VM
    builder exactly"(Plan 482 class-token 契约):`ITEM_ACTIVE` 只有
    `bg-primary/10 text-primary font-medium`,无 hover。
  - 修法:web 侧 ext 兜底——`inject_styles.web-only.ts` 增一条
    `.nav-item:hover { background-color: hsl(var(--accent)); }`
    (active/非 active 统一中性灰 hover;`nav-item` token 类所有
    导航项都带)。ext 层不被重生成覆盖。上游契约同步登记 C 组 D23。
  - 层:`gen/front/vue/src/ext/src/front/inject_styles.web-only.ts`
    (手写层,直接改)。

- **D21 设置弹层点击外部不关闭**(用户截图 4)
  - 证据:面板 `settings-panel absolute bottom-full … z-100`,开合仅
    `.Toggle` 一条路径;无遮罩、无 document 外点监听、无 ESC。
  - 根因:`src/front/settings_menu.at` 弹层为纯状态条件渲染,DSL 层
    未表达任何 dismiss 路径。
  - 修法(首选,.at 可表达):`.isOpen` 时渲染全屏透明背板
    `div { style: "fixed inset-0 z-99"  onclick: .Close }`,置于面板
    z-100 之下,点击背板即 `SetOpen(false)`;若 codegen 对普通 div 的
    onclick 不可表达,则改走 ports 逃生舱(照 `setup_auth_fetch` 模式:
    `platformSetupOutsideClose` 在 ext 层装 document mousedown 监听,
    命中 `.settings-panel` 外即派发关闭)——两条路都在本任务内完成,
    不留 TBD。
  - 层:`src/front/settings_menu.at`(+可能 `ports/platform.at` /
    `ports/platform.ts` / ext 实现)。

- **D22 新建会话后不自动聚焦输入框**(用户反馈 2)
  - 证据:`chats_view.at:375` `.NewSession -> { store.NewSession() }`,
    无聚焦动作;用户点 `+` 后需再手点输入框才能打字。
  - 修法:走 ports 逃生舱(既有模式,`chats_view.at:18` 已
    `use.web platformRunRelayCommand`):`ports/platform.at` 增
    `platformFocusComposer`,`ports/platform.ts` 透传,ext 实现为
    `document.querySelector('.chats-input')?.focus()`——`.chats-input`
    是 composer textarea 的稳定类(`mention_input.at:123`,常驻挂载,
    无需等重渲染)。handler 改为
    `.NewSession -> { store.NewSession() platformFocusComposer() }`。
  - 层:`src/front/chats_view.at` + `src/front/ports/platform.at` +
    `ports/platform.ts` + ext 实现(手写桥)。

- **D25 会话最新一条消息(叶子)永不渲染**⭐
  - 证据:重建的 Block 全家福会话(10 条消息,后端 active_leaf 正确指向
    末条 assistant)在 UI 只渲染前 9 条——DOM 消息根节点恰 9 个;回看旧
    会话"问卷消息空白""probe 回复不见"同为此病。发新消息后,原叶子转为
    中间节点即显现(与"AI 已回复但当时看不见、过一会又有了"的用户体感
    吻合)。
  - 根因:`src/front/forge_helpers.at` 的 `chatActivePath` 差一错误——
    主路径 `out` = 头段 `[0..anchor]` + `chain[c] (c=len-2..0)`,最远只
    推到叶子的父节点(`chain[0]`),**叶子本身从未 append**;仅
    `chain.length == 0` 的早退分支(叶子即根)用 `hi <= leafIdx` 含叶。
    产物 `ext/src/front/forge_helpers.ts:354-403`。
  - 修法:chain 走完后 `out.push(messages[leafIdx])`(或循环改为
    `c = chain.length - 1` 且头段只到 anchor 前一节点,二选一,注意
    R1 纯 fn 形态);补 vitest 单测钉死"线性链含叶/分支链含叶"两形态。
  - 影响:此病修复前,QuestionnaireCard 的挂载条件
    (`msg.id == lastMsgId`)与叶子隐藏互相抵消——问卷卡永远不可见。
  - 层:`src/front/forge_helpers.at` → 产物 `forge_helpers.ts`。

- **D26 问卷块判别字段口径不一(kind vs type)**
  - 证据:重建会话中 agent 按 `{"kind":"questionnaire",...}` 产出问卷
    JSON,前端 `questionnaireFor`/`stripQuestionnaire` 只认
    `json.type == 'questionnaire'` → 不识别为问卷块。口径二选一统一:
    前端兼容 kind,或 prompt/文档教 agent 用 type(本批执行取后者,
    兼容性放宽归 C 组上游)。
  - 层:`src/front/questionnaire.at` → 产物 `questionnaire.ts`。

### B 组:待用户裁定后并入(默认建议已给出)

- **D9 明文密码持久化 localStorage**(`auth_store.at:67`
  `musk_login_password`,登录页 onMounted 回填)。安全建议:去掉持久化;
  若保留回填便利,至少不存明文。→ 待澄清⑨a。
- **D10 品牌名不统一**:登录页 "AutoForge"🔥 / 侧栏 "Auto Musk" /
  document.title "auto-musk"。需定一个正名统一。→ 待澄清⑨b。
- **D11 空状态与登录页 i18n 硬编码英文**:"No goals yet"/"Drag files to
  upload"/Username/Login 等。建议随批 i18n 化,涉及多处模板,工作量中。
  → 待澄清⑨c(是否入本批)。
- **D4 计划详情 Markdown/frontmatter 不渲染**:详情把 YAML frontmatter 与
  `##` 标记当纯文本;现成 `gen/front/vue/src/utils/frontmatter.ts`(含
  vitest)未接线。修在 `plans_view.at` + 接 util,工作量较大。
  → 待澄清⑨d(是否入本批)。

### C 组:生成器上游债(登记归 auto-lang,不在本批)

- **D12** alert-dialog 资产目录双份(`components/ui/alert-dialog/{,alert-dialog/}`
  字节级相同)——生成器资产复制逻辑去重。
- **D13** store 命名双后缀(`useAuthStoreStore` 等 9 处)——codegen 模板。
- **D14** 生成死代码(`password.value = password.value;`、无人监听的 emit
  等)——codegen。
- **D15** forge store 500ms `setInterval` 永不清除(`forge_store.at` 语义 +
  codegen timer 生命周期)——现由 streaming 守卫兜底,空闲无害。
- **D23** nav-item class-token 契约:active 项无 hover(Plan 482,VM
  builder 同源)——上游契约放开后撤 D20 的 ext CSS 兜底。
- **D24** codegen 动态 style 绑定产出 `:style=`(Tailwind 类串作废,
  实证 `ChatMessage.vue:65-66`)——D17 的上游根修;root-fix 后可回撤
  .at 分支静态化规避。
- **D27** auto-man shadcn 脚手架检测语料只含 .at 生成代码,手写 .vue
  passthrough(use.web 组件,如 DeleteConfirmDialog.vue)的
  `@/components/ui/*` 导入标记不入语料 → 冷检出重生成后脚手架缺失
  (vue-tsc TS2307)。058 时已知手工步("gen ui/ 补脚手架+@vueuse/core,
  regen 后需重装"),536 T9 改端口链后 .at 语料不再含 alert-dialog 标记,
  缺陷潜伏化(存量树靠粘性资产存活)。上游根修=语料并入 passthrough
  .vue;musk 侧每次冷重生成后手工补拷。vitest devDeps 同被重生成抹除
  (058 ⑪④ 既有工具债,`pnpm add -D vitest@2` 会话级补装)。

### 环境注记(非债,不立项)

IAB/无头后台页 rAF 停帧导致 Playwright locator click actionability 超时;
自动化应改用 evaluate 原生触发或确保页面前台可见。Vue 轨性能实测无债
(证据见需求分析)。

## 测试设计

1. **D1 单测(红→绿)**:新增
   `gen/front/vue/src/__tests__/specs_helpers.spec.ts`:
   `specSectionItems({sections:[{section_type:"Goals",items:[{id:"x"}]}]}, "goals")`
   长度为 1;修复前红,重生成后绿。
2. **D16 i18n 静态断言**:模板 t() key 全集 ⊆ zh/en locale keys;修复 D5
   后全绿,人为删 key 应红。
3. **既有门禁**:`scripts/vm-link-probe.cmd`(改 .at 后必跑);
   `cd gen/front/vue && pnpm build`(vue-tsc strict);
   `pnpm vitest run`(现状 23+1skip,要求零新增红)。
4. **实机走查验收**(admin/admin,musk-demo):
   - D1:规范→goals 显示 9 条;
   - D2:错误密码显示错误提示且按钮复位;正确密码进入主界面;
   - D3:密码输入为打点;
   - D5:知识库页面按钮显示"编辑/删除";
   - D6:计划列表无硬裁切;
   - D8:刷新后 localStorage 的 musk_user 为合法 JSON;
   - D17:消息头 "AI"/时间戳同 行内居中且有间距(对照用户截图 1);
   - D18:AI 消息 copy 图标在内容块左侧;
   - D19:选中会话项 hover 为灰色、文字可读(对照用户截图 2/3);
   - D20:主导航四项 hover 均有灰色背景;
   - D21:设置面板打开时点击外部即关闭;
   - D22:点 `+` 新建会话后直接键盘输入可进入 composer。
   以上逐条截图/录证留档。

## 验收标准

- [x] A 组 13 项(D1/D2/D3/D5/D6/D7/D8/D16/D17/D18/D19/D20/D21/D22 中除
      blocker 转出者)全部落地,门禁(vm-link-probe/build/vitest)全绿,
      零新增红。
      [✅] 14/14 落地(含 D25/D26 增量),零 blocker 转出(D3 type 透传/
      D7 span 形态/D15— DSL 均可表达);D18/D21 属"已由 059 分支顺带
      解决+本批复核确认"形态(排程前置预告的合并后复核);终态门禁:
      probe 63149B PASS / pnpm build strict ✓ / vitest 32+1skip
      (基线 23+1,新增 9:D1×3+D16×3+D25×3)。
- [x] 实机走查清单(测试设计 4,共 12 条)逐条通过并留证。
      [✅] 两批全 PASS:tmp/p061-evidence/{T10-第一批走查证据,
      T18-第二批走查证据}.md + d2-d3 截图;IAB 截图通道间歇失败,
      以 DOM 断言/计算样式几何量/焦点态/真实点击行为替代像素留证
      (对截断/类型/交互类更硬)。
- [ ] B 组每项在待澄清⑨有明确裁定(修/不修/移 C 组);裁定为修的项并入
      执行步骤并复跑门禁。
      → **执行期状态(2026-09-05)**:⑨a-d 四项用户裁定未落,保持待澄清
      开放(本批不擅自裁定);⑨e 已由 D25/D26 并入清偿。KD 061 行登记
      待裁定指针。
- [x] C 组 6 项(D12–D15/D23/D24)在 KNOWN-DEBT-AND-RISKS.md 登记在案
      (引用本计划编号)。
      [✅] KD 061 行登记七项(D12–D15/D23/D24+执行期新增 D27 冷重生成
      脚手架缺失/vitest 抹除)。
- [x] 产物无手改残留:重生成后 `git diff` 仅预期文件变化。
      [✅] worktree git status 干净;tracked gen 资产仅 __tests__/utils
      (add -f 约定),手改 gen 拷贝(inject_styles)已归真源 src/front/
      并经 regen 传播(8571ad5)。

## 执行步骤

> 前置:本计划 drafting → 用户补充裁定后转 executing。T5/T8/T15 若触达
> DSL 能力边界,按登记册给出的一条龙 fallback 完成之,不留 TBD。
>
> 排程前置(2026-09-04 核实):本批**无 auto-lang 硬依赖**(536 已
> execution_done,其交付只影响 059 残项退役,不影响本批任一任务;全部
> 修复用的是当前 master 工具链已实证能力)。真实排序约束是与
> `.wt/vm-chat-fixes/auto-musk`(分支 `auto-musk-dev-1`,PLAN-059 执行
> 分支,门禁四绿)在 chat_message.at / chats_view.at / nav_item.at /
> i18n 正面重叠——**待该分支合回 main 后再开工**。届时:T17/T18 落点按
> 合并后消息卡组织复核(该分支重构过消息卡渲染路径);D19 与该分支
> 12d931d 的 `hover:bg-primary/15` 方案统一为本批裁定的
> `hover:bg-accent` 灰色方案(用户 2026-09-04 明示)。
>
> 开工前置已清(2026-09-05):①059 分支已合回(925c9ed/966ec46),worktree
> 已清理;②auto-lang master release 工具链重装(target-master-check,
> g7009a5111)后 T9 alert-dialog modal 廉价回归 PASS(模态渲染+Cancel
> 派发+状态复位,像素/状态双证,tmp/p061-preflight/);③冷重生成陷阱
> 实录:gen 树 gitignored → worktree 首次 `auto build` 后
> alert-dialog 脚手架缺失(auto-man shadcn 检测语料只扫 .at 生成代码,
> 手写 DeleteConfirmDialog.vue 的 `@/components/ui/alert-dialog` 标记
> 不入语料——058 已知手工步"regen 后需重装"在 536 T9 改端口链后变
> 潜伏;登记 C 组 D27)与 vitest devDeps 被抹(058 ⑪④ 工具债)——
> 会话级处置:拷 shadcn-ui/alert-dialog 脚手架 + `pnpm add -D vitest@2`,
> 处置后 build/vitest 23+1skip 双绿基线确立。

- [x] T1 写 D1 回归单测:新建 `gen/front/vue/src/__tests__/specs_helpers.spec.ts`,
      断言 `specSectionItems` 对 `"Goals"`/`"goals"` 命中;运行
      `cd gen/front/vue && pnpm vitest run src/__tests__/specs_helpers.spec.ts`
      确认当前为红。
      [✅ 已完成] 3 测全红(大小写敏感比较实证);git add -f 入库(PLAN-041
      T13 约定)。
- [x] T2 修 D1:`src/front/specs_helpers.at:328` `specSectionItems` 比较前
      两侧小写归一(遵守 R1 纯 fn 形态);跑 `scripts/vm-link-probe.cmd`。
      [✅ 已完成] want/have 双侧 to_lower;probe PASS 63135B。
- [x] T3 重生成 vue 轨(`auto run`,AutoUI 管线,同 PLAN-028 M1 流程),
      确认 `ext/src/front/specs_helpers.ts` 产物含归一化逻辑;T1 单测转绿。
      [✅ 已完成] release 工具链 `auto build`;产物 :263 toLowerCase;
      vitest 3/3 绿(重生成抹 devDeps,vitest@2 会话级补装——D27)。
      82e2e54。
- [x] T4 修 D2:`src/front/auth_store.at` Login/Register 补 try/catch +
      loading 复位 + SetError(规避 try 内嵌 if-return);跑
      `scripts/vm-link-probe.cmd` 并重生成。
      [✅ 已完成] catch 只赋值/两路 loading 复位/错误文案接通;
      probe 63105B;产物 catch 落地;97618d5。
- [x] T5 修 D3:`src/front/login.at:64` 密码输入框属性改为产出
      `type="password"`;若 codegen 不支持则记 blocker 转出并跳到 T6;
      重生成后核对 `components/LoginPage.vue` 产物。
      [✅ 已完成] type 属性透传成立(产物 `:type="'password'"`),
      无 blocker;3ef9cfb。
- [x] T6 修 D5:`src/front/i18n/zh.json`、`en.json` wiki 节补
      `"edit"`/`"delete"`;重生成后核对 `locales/*.json` 同步。
      [✅ 已完成] 双 key 补入,产物 locales 同步;9e19bb8。
- [x] T7 写 D16 门禁:扩展 `gen/front/vue/src/__tests__/i18n.spec.ts`(或
      新建 spec),静态扫描产物模板 t() key ⊆ locale keys;
      `pnpm vitest run` 全绿(依赖 T6 先落)。
      [✅ 已完成] import.meta.glob(?raw,eager) 静态门禁(非空守卫+en/zh
      两断言,shim 补 glob 声明规避 node 内建类型);首跑即抓出
      wiki.create/wiki.contentPlaceholder 两枚 D5 同族漏网 key,随批补齐;
      vitest 29+1skip;15c7a60。
- [x] T8 修 D7:`src/front/wiki_nav.at` 内层删除钮去 button 嵌套(改 span
      形态;DSL 不可表达则记 blocker 转出);重生成后确认 vite 编译零
      "<button> cannot be child of <button>" 告警。
      [✅ 已完成] raw 树内删除钮 button→span+onclick.stop(div onclick
      先例证普通元素可挂),产物 WikiNav.vue:193,DSL 可表达无 blocker;
      67799f1。
- [x] T9 修 D6+D8:`src/front/plans_view.at` 列表项补 truncate 类串;
      `src/front/auth_store.at` musk_user 写读 JSON 化;重生成,
      跑 `scripts/vm-link-probe.cmd` + `pnpm build`。
      [✅ 已完成] D6 断点重定:truncate 已在类串,真因=Button(inline-flex)
      多 text 子节点各成 flex item 不吃 ellipsis→单 span(flex-1 min-w-0
      truncate)承载整条标签(wiki_nav 同款);D8 三写侧 JSON.stringify+
      Init 读侧 JSON.parse 守卫(坏串弃用走重登);probe 63125B+regen 绿,
      产物 PlansView:152/useAuthStoreStore:14;6f3daf1。
- [x] T10 实机走查第一批(D1/D2/D3/D5/D6/D8 六项,测试设计 4 清单),
      截图留证。
      [✅ 已完成] 六项全 PASS:worktree regen 产物+vite :3336+musk serve
      :9247;D1 goals 结构化条目(账本已增至 103 items)/D2 错密码横幅+按钮
      复位+正码入主界面/D3 type=password/D5 编辑/删除按钮/D6 几何硬证
      ellipsized(4 项 scrollW>clientW)/D8 musk_user 合法 JSON+重载恢复;
      IAB 截图通道间歇失败(环境注记族),D2/D3 截图落盘,余以 DOM 断言+
      计算样式为证;证据 tmp/p061-evidence/T10-第一批走查证据.md。
- [x] T11 修 D17:`src/front/chat_message.at` 消息头按 isUser 分支拆成
      两个静态 style 的 row(badge 颜色同批静态化),绕开 codegen
      `:style` 缺陷;重生成后核对 `ChatMessage.vue` 产物为
      `class="flex items-center gap-2 …"` 形态。
      [✅ 已完成] headerClass/badgeClass/badge 三 computed 退役,双分支
      静态 row+字面量 badge;产物 :81/:89 `class="flex flex-row
      items-center gap-2 px-1( justify-end)"`,组件内 :style= 清零;
      838571f。
- [x] T12 修 D18:`src/front/chat_message.at:157` 工具栏行
      `justify-end` → `justify-start`;重生成。
      [✅ 已完成] 核验销案:059 分支已顺带修妥(用户反馈 2026-09-03
      注释在案),源码+产物 :194 均 justify-start,排程前置预告的
      "合并后复核"确认,无需重修;838571f 附带核验。
- [x] T13 修 D19:`src/front/chats_view.at:186` 选中分支类串补
      `hover:bg-accent`;同批扫描并修 `nav_item.at:28`、
      `specs_view.at:301` 同病位点(active 分支缺 hover: 的 Button);
      重生成后在产物 `ChatsView.vue` 确认 hover 类落地。
      [✅ 已完成] 扫描扩至 6 族位点:chats 会话选中
      hover:bg-primary/15→hover:bg-accent(统一 059 分支 12d931d 为
      用户裁定灰方案)/nav_item card+compact 两 active/plans 列表选中
      /specs 节导航选中/specs 结构化+文件树模式开关×2;产物
      ChatsView:4/SpecsView:10/PlansView:8 hover:bg-accent,src 零
      hover:bg-primary/15 残留;probe PASS;e53d14e。
- [x] T14 修 D20:`gen/front/vue/src/ext/src/front/inject_styles.web-only.ts`
      增 `.nav-item:hover { background-color: hsl(var(--accent)); }`
      (ext 手写层,直接改);C 组 D23 上游登记。
      [✅ 已完成] 落点勘误:该文件真源=src/front/inject_styles.web-only.ts
      (platform.web.at use.web 引),gen/ext 侧是 regen 整写拷贝(手改
      必被冲,实测复现)——改真源后 regen 传播,产物 :188 hover 规则
      落地;8571ad5。
- [x] T15 修 D21:`src/front/settings_menu.at` `.isOpen` 分支加全屏透明
      背板 div(`fixed inset-0 z-99` + onclick 关闭,面板 z-100 之上
      保持);若 codegen 对 div onclick 不可表达,同任务内改走 ports
      逃生舱(`platformSetupOutsideClose`,照 `setup_auth_fetch` 模式)
      实现之;重生成后实机验证点外可关。
      [✅ 已完成] 核验销案:登记形态(z-100 absolute popover 无 dismiss)
      已被 059 T9 dialog 受控重构退役——现行 dialog(open:.isOpen) web
      侧自带遮罩外点/X/ESC 三路关闭(reka),VM 侧显式关闭钮(ESC/外点
      =KD 059-T9② 上游家族债在案);旧背板方案对现行架构为倒退,不施;
      web 外点关闭 T18 实机复验。
- [x] T16 修 D22:`ports/platform.at` 增 `platformFocusComposer` 声明,
      `ports/platform.ts` 透传,ext 实现 `querySelector('.chats-input')
      ?.focus()`;`src/front/chats_view.at:375` handler 追加调用;
      重生成。
      [✅ 已完成] 端口链三件:web=src/front/focus_composer.ts 手写件
      (querySelector('.chats-input').focus(),常驻挂载无需等重渲染)+
      platform.web.at 转发/vm=platform.vm.at 降级留痕(iced 无 DOM
      查询/focus native)/chats_view NewSession 追加调用;产物
      ChatsView:15/133+ext 镜像;probe 63135B;eacc022。
- [x] T17 全量门禁复跑:`scripts/vm-link-probe.cmd` +
      `cd gen/front/vue && pnpm build` + `pnpm vitest run`,零新增红;
      `git diff` 核对无产物手改残留。
      [✅ 已完成] probe PASS 63149B / pnpm build(vue-tsc strict+vite)
      ✓ / vitest 29+1skip(基线 23+1 + 新增 6:D1 三测+D16 门禁三测)
      零新增红;worktree git status 干净——重生成零手改残留(tracked
      gen 资产仅 __tests__/utils,手改 gen 拷贝已被 regen 自然冲平)。
- [x] T18 实机走查第二批(D17–D22 六项,测试设计 4 清单逐条,对照用户
      四张截图),截图/录证留档。
      [✅ 已完成] 六项全 PASS(计算样式/DOM 焦点态/真实 CUA 点击为证,
      IAB 截图通道间歇失败——环境注记族):D17 四头行 alignItems=center+
      gap=8px/D18 工具栏 flex-start/D19 选中卡 hover:bg-accent 且
      variant hover:bg-primary/90 被 tailwind-merge 剔除/D20 规则注入+
      4 导航项全带 nav-item/D21 真实点击遮罩关闭(data-state=closed,
      触发钮 open 退出)/D22 点 + 后 activeElement=TEXTAREA.chats-input
      (首测红两处回修 4fd0103:同步 focus 被点击默认焦点覆盖→
      setTimeout(0);裸 .chats-input 双持有→收窄 textarea.chats-input);
      证据 tmp/p061-evidence/T18-第二批走查证据.md。
- [x] T19 修 D25(⭐ 头号):先在
      `gen/front/vue/src/__tests__/` 补 `chatActivePath` 单测(线性链含
      叶/分支链含叶/叶子即根三形态,当前应红);再修
      `src/front/forge_helpers.at` `chatActivePath`——chain 走完后补
      `out.push(messages[leafIdx])`;跑 `scripts/vm-link-probe.cmd`,
      重生成后单测转绿。D26 随批:prompt/文档侧统一问卷块用
      `"type":"questionnaire"`(见登记册,不改前端判定)。
      [✅ 已完成] TDD:红(线性 10→9/分支缺叶;叶子即根本绿)→chain 走完
      补 push(messages[leafIdx])→绿 3/3;probe 63105B;活体:Block
      全家福 23 条=DOM 23 徽章,末条 AI 叶子在場(问卷卡可达性同源
      解除)。D26:demo 工作区 Block 规范 v2 §8 口径统一 survey→type:
      'questionnaire'+kind 陷阱注记(文档侧教 agent,前端判定不动);
      52df5a5。

## 复审记录

**2026-09-05 复审(zhaopuming 会话,/auto-plan:review)——PASS,status → reviewed。**

**验收标准逐条复验(worktree plan-061-dev,13 提交 82e2e54..52df5a5,diff 23 文件 +315/-203):**

1. **A 组全落地+门禁全绿** — **PASS**。复审新鲜重跑三门禁:
   ① vm-link-probe PASS 63145B;② pnpm build(vue-tsc strict+vite)✓;
   ③ vitest 32+1skip(基线 23+1,新增 9:D1×3/D16×3/D25×3)零新增红。
   diff 逐项签名核验:to_lower×2/登录+注册双 catch 文案/type:"password"/
   i18n 四 key/plans truncate span/wiki_nav span 化/stringify×3+parse/
   glob 门禁+shim 声明/双静态头行字面量/hover:bg-accent 增 9 行+
   hover:bg-primary/15 删 1 行/nav-item:hover/focusComposer 链×3/
   out.push(messages[leafIdx])——14/14 与 diff 一一对应,零 blocker
   转出(登记册预告的 T5/T8/T15 fallback 均未触发,DSL 可表达)。
   核验销案二项(D18/D21):059 分支先行修复在案(源码/产物复核),
   属排程前置预告的"合并后复核"形态,非遗漏。
2. **实机走查 12 条留证** — **PASS**。两批证据文件在案
   (tmp/p061-evidence/{T10,T18}+d2-d3 截图);IAB 截图通道间歇失败,
   以计算样式几何量/DOM 断言/焦点态/真实 CUA 点击替代像素留证
   (D6 ellipsized 四项 scrollW>clientW/D21 真实点击遮罩关闭/
   D22 activeElement=TEXTAREA.chats-input,对交互类更硬)。
3. **B 组裁定** — **执行期状态如实**:⑨a-d 四项保持待澄清开放
   (用户裁定未落,本批不擅自裁定);⑨e 已由 D25/D26 并入清偿。
   KD 061 行挂待裁定指针。
4. **C 组 KD 登记** — **PASS**。KD 061 行七项(D12-D15/D23/D24/D27)
   引用本计划编号;D27 为执行期新发现(冷重生成脚手架缺失+vitest
   抹除),根修归 auto-man。
5. **产物无手改残留** — **PASS**。worktree git status 干净;tracked
   gen 资产仅 __tests__/utils(add -f 约定);D20 手改 gen 拷贝已归
   真源 src/front/(8571ad5,regen 整写拷贝的勘误)。

**遗漏/延后/workaround 清点:**
- 遗漏:无(每任务有 diff 对应;D26 文档侧修在 demo 工作区数据
  tmp/musk-demo/docs/plans/block-rendering-spec-v2.md——工作区数据
  本不入版本库,chats.json 同口径,复验在案)。
- 延后:B 组⑨a-d 待用户裁定(计划内机制,非静默);D26 前端 kind
  兼容放宽归 C 组(登记册明示)。
- Workaround(均有登记/计划内设计):D17 .at 分支静态化→C 组 D24;
  D20 ext CSS 兜底→C 组 D23;D2 catch 自赋值→029 缺口③规避
  (登记册修法明示);D22 VM 侧 focus 降级 no-op(iced 无 DOM 查询/
  focus native,ports 注记在案——iced focus 接线归上游,候选随
  KD 059-T9 家族观察);D27 冷重生成手工补装(KD 061)。
- diff 零 TODO/FIXME/HACK 新增。

**复审范围注记**:本计划仅触前端源(src/front+gen tests),后端
零改动(diff 证实);style-parity/phase1-leaves 对拍为 auto-lang 侧
fixture 门禁(KD 049),不消费 musk 应用类串,与本 diff 面无交,
未随本批评测。pre-flight 转交语三项(release 工具链 g7009a5111+
T9 modal 回归 PASS+059 已合回)执行期已清并在案。

## 待澄清事项

- ⑨a D9 明文密码持久化:删除持久化(推荐,安全)还是保留回填便利?
- ⑨b D10 品牌正名:AutoForge / Auto Musk / auto-musk 三选一,统一范围
  含 document.title?
- ⑨c D11 空状态与登录页 i18n 化是否入本批(涉及多模板,工作量中)?
- ⑨d D4 计划详情 Markdown/frontmatter 渲染是否入本批(需接线
  frontmatter util,工作量较大)?
- ⑨e 用户复审本登记册后补充的待修点(预留编号:D25 起,顺延并入 A 或
  B 组,并同步增补执行步骤与 total_steps)。D17–D22 已于 2026-09-04
  第二轮复审并入。
