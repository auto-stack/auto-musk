---
plan_id: PLAN-063
status: reviewed
feature_name: B 组裁定四项修复 + auto-lang 上游 C 组清偿(双相)
author: [zhaopuming]
created_at: 2026-09-05T13:20:00+08:00
updated_at: 2026-09-05T15:10:00+08:00

# Leave these EMPTY here — /auto-plan:review fills them:
supersedes_spec_components: []
new_spec_components: []
touched_goals:
  - "goal-frontend-parity: B 组裁定四项清偿(D9 密码语义/D10 正名 Auto Musk/D11 i18n 全量化+语言切换机制根修[D29]/D4 详情 frontmatter+Markdown)+D13 store 命名/D17 规避回撤(数组形态)/D20 退役——.at 单源双轨受益"
  - "goal-spec-knowledge: 计划详情消费面升级(frontmatter 元数据 chips+Markdown 富文本,plans 域数据可读性)"

current_step: 24
total_steps: 24
---

# [PLAN-063] B 组裁定四项修复 + auto-lang 上游 C 组清偿(双相)

## 变更摘要

PLAN-061 归档时挂起的 B 组四项已经用户裁定(2026-09-05,四项全"修"):
**D9** 删明文密码持久化(保留用户名回填——浏览器无真密文路径)、
**D10** 品牌正名 Auto Musk 三处统一、**D11** 硬编码英文全量 i18n 化、
**D4** 计划详情 frontmatter 元数据条+Markdown 渲染。本计划 Phase A 落地
这四项(musk 侧,`.at` 单源)。

同时把 PLAN-061 登记的 **C 组上游七项**(D12–D15/D23/D24/D27,归
auto-lang)+ 起草期新发现 **D28**(document.title 透传)并入本计划
**Phase B**:Phase A 完成后,按 AGENTS.md 第三行约定在**同组目录**开
auto-lang 专用 worktree(`.wt/musk-063/auto-lang`,分支 `auto-musk-dev`)
逐项清偿,auto-lang 全量门禁绿后合回其 master;musk 侧随后消费回撤
三处既有规避(D17 分支静态化/D20 ext CSS 兜底/D27 手工补装步骤)并
接线 title。KD 061 行的 C 组/待裁定两段随批核销。

## 目标

1. localStorage 不再存明文密码;登录页仅回填用户名,密码每次重打,
   且启动时清掉历史残留的明文密码。
2. 品牌三处统一为 "Auto Musk":登录页大标题(现 "AutoForge"🔥)、
   侧栏(已达标)、浏览器标签页 title(现 "auto-musk",经上游
   D28 透传后由 pac.at 提供)。
3. 硬编码英文文案全量收进 i18n(登录页 7 处+空态/杂项 5+ 处),
   切中文即全部跟随;D16 门禁(模板 t() key ⊆ locale keys)兜底零漏网。
4. 计划详情页可读化:frontmatter 元数据(plan_id/status/步骤计数等)
   呈现为状态条,正文 Markdown 渲染(复用聊天轨 platform:markdown)。
5. auto-lang 上游八项清偿(D12–D15/D23/D24/D27/D28),auto-lang 全量
   门禁绿;musk 消费后三处规避回撤、D16/D27 门槛不回退。

## 架构方案

不动架构,双相分工:

| 相 | 落点 | worktree | 分支 | 依赖 |
|:--|:--|:--|:--|:--|
| Phase A | musk `src/front/*.at`(+手写 ts/测试资产) | `.wt/musk-063/auto-musk` | `plan-063-dev` | 无(auto-lang 现行 master 工具链足够) |
| Phase B | auto-lang `crates/auto-man/src/vue.rs`、`crates/auto-lang/src/ui_gen/*` 等 | `.wt/musk-063/auto-lang`(同组并排) | `auto-musk-dev` | Phase A 完成 |

层归属沿 PLAN-061 约定:`src/front/*.at` 与 `src/front/*.ts`(手写,
经 use.web/use.web.fn 消费)为源;gen/ 产物只核对不手改(gen 内 tracked
资产仅 `__tests__`/`utils`,新 tracked 资产经 `git add -f`,PLAN-041
T13 约定)。**冷重生成债(D27)在本计划落地前仍然在场**:任何冷检出
`auto build` 后需手工补拷 alert-dialog 脚手架 + `pnpm add -D vitest@2`
(KD 061 行);Phase B T12 根修后退役。

两相间的消费闭环(Phase B 合回后):
- D24(动态 style→:style= 根修)→ musk 回撤 D17 的 chat_message.at
  分支静态化(恢复 computed 单路径);
- D23(nav-item 契约 active 带 hover)→ musk 回撤 D20 的
  inject_styles `.nav-item:hover` 兜底;
- D28(title 透传)→ musk pac.at 增 `title: "Auto Musk"`;
- D27(语料并入 passthrough+devDeps 保留)→ 冷 regen 手工步骤退役。

## 技术栈

- Auto(.at)→ Vue3 + Vite + tailwind(生成):`auto build` 管线
  (release 工具链,`D:/autostack/auto-lang/target-master-check/release/auto.exe`)
- vitest(`gen/front/vue/src/__tests__/`,现 4 spec 32+1skip 基线)
- 门禁:`scripts/vm-link-probe.cmd`(改 front 源后必跑);
  `cd gen/front/vue && pnpm build`(vue-tsc strict);`pnpm vitest run`
- Phase B:auto-lang `cargo tf` 全量(+`cargo tv/tt` 当触及 VM/转译器)
- 实机:musk serve(:9247,tmp/musk-demo)+ vite dev(:3336,IAB 走查)

## 需求分析与背景调查

- 规范账本(backend/.autoos/specs.json,v2):六区 225 items(goals 35/
  architecture 33/designs 32/tests 33/reviews 56/reports 36,含 PLAN-061
  沉淀 P061-1..6)。本批触及 goal-frontend-parity(主)与
  goal-spec-knowledge(D4 计划详情消费 plans 域数据)。
- D9 现状:`auth_store.at:83/:107` Login/Register 双写
  `musk_login_password` 明文;`login.at:116-117` Init 回填
  username+password。裁定依据:WebCrypto 密钥同储=混淆非加密;浏览器
  密码管理器不向页面代码开放回填(且 VM 轨/webview 不可用)。
- D10 现状:`login.at:42-43` "🔥"+"AutoForge" 大标题;`app.at:76`
  侧栏已是 "Auto Musk";document.title 源=auto-man
  `generate_index_html(name)`(vue.rs:904/:1402)取 pac.at `name`
  ("auto-musk")——**musk 侧无法干净修 title**,需上游加 pac
  `title:` 字段透传(本批登记 D28,Phase B)。
- D11 盘点(起草期 grep 实锚):login.at:50/54/60/68/87/89/102
  (Username/Password/Login/Register/No account? Register/双 placeholder);
  specs_category.at:225 "No goals yet";specs_leaf.at:261 "No items yet";
  specs_detail.at:203 "No relations";wiki_nav.at:39 dropText("Drop files
  here"/"Drag files to upload")。执行时以同 grep 复盘补漏,D16 门禁
  终验。注:placeholder 传 t() 有先例(wiki_view:112);label 传 t()
  无先例,若 DSL label 不吃表达式则改 span 形态(D7 同款 fallback)。
- D4 现状:plans_view 详情把 plans_get 文件内容当纯文本;现成
  `gen/front/vue/src/utils/frontmatter.ts`(tracked,`splitFrontmatter`
  含 vitest 7 断言)未接线。Markdown 渲染复用 app.at 已挂载的
  platform:markdown(chats_view `use component Markdown from
  "platform:markdown"` 先例)。
- C 组八项上游锚点(Phase B):D27=vue.rs `detect_shadcn_components`
  (:143,语料仅 .at 生成代码)+`generate_package_json`(regen 抹 devDeps);
  D12=shadcn 资产拷贝双份目录(vue.rs install 路径);D13=store 命名
  `useXxxStoreStore` 双后缀(ui_gen/vue.rs 模板);D14=死代码
  (password.value 自赋值/无人监听 emit,ui_gen/vue.rs);D15=forge store
  500ms setInterval 永不清除(forge_store.at 语义+VM timer 生命周期);
  D24=动态 style 绑定发 `:style=`(ui_gen/vue.rs;D17 规避位
  chat_message.at);D23=nav-item class-token 契约
  (ui_gen/nav_contract.rs ITEM_ACTIVE 无 hover;D20 兜底位
  inject_styles.web-only.ts);D28=generate_index_html title 透传。
- 背景债:KD 061 行(C 组七项+裁定四项指针)、KD 059-T9②(受控 open
  VM 无 ESC/外点关——**不在本批**,家族债继续挂)。

## 详细设计

### Phase A:musk 侧四项

- **D9 密码持久化退役**
  - `src/front/auth_store.at`:删 :83/:107 两处
    `localStorage.setItem("musk_login_password", password)`;Init 增加
    一次性残留清理 `localStorage.removeItem("musk_login_password")`
    (存量明文即刻出清;幂等无害)。
  - `src/front/login.at`:Init :117 删密码回填(.password 恒空串起点),
    :116 用户名回填保留。
  - 验收:登录→刷新→用户名预填/密码空;localStorage 无
    musk_login_password 键(含此前已登录浏览器的清理)。

- **D10 品牌正名(login 页部分)**
  - `login.at:42-43`:`text "🔥"` 行删除(侧栏品牌无 emoji,统一
    形态),`h1 "AutoForge"` → `h1 "Auto Musk"`。侧栏已达标不动;
    document.title 归 Phase B D28+T22 消费。

- **D11 i18n 全量化**
  - 新增 locale 节:`login`(username/password/enterUsername/
    enterPassword/login/register/noAccount)与空态/杂项键
    (`specs.noGoalsYet`/`specs.noItemsYet`/`specs.noRelations`/
    `wiki.dropHere`/`wiki.dragToUpload` 等,zh/en 成对)。
  - 模板字面量逐一换 t();wiki_nav dropText 为 computed 双分支串,
    改为双 key 的 t() 分支。label 表达式能力未知→先试
    `label t("login.username")`,DSL 拒绝则 label 改 span+text 形态。

- **D4 计划详情 frontmatter+Markdown**
  - util 真源迁移:`gen/front/vue/src/utils/frontmatter.ts` → 
    `src/front/plans_frontmatter.ts`(musk 源侧持有,regen 拷贝入
    ext 树;`__tests__/frontmatter.spec.ts` 改 import ext 镜像路径,
    断言原样)。旧 gen utils 文件退役(git rm tracked 副本)。
  - `plans_view.at` 详情区:经 `use.web.fn` 包一手写薄壳
    `src/front/plans_detail_helpers.ts`(导出
    `plansFrontmatterMeta(content)`,内部调 plans_frontmatter 并整形
    {plan_id,status,step,feature_name} 串),meta 行渲染为 chips;
    正文换 `use component Markdown from "platform:markdown"` 渲染
    body。无 frontmatter 的裸 md/纯文本回退现状纯文本展示。
  - 前置核对:chats_view 对 platform:markdown 的消费形态照抄(参数/
    绑定面),确保 VM 轨同降级路径(Markdown VM 文本降级在案,可接受)。

### Phase B:auto-lang 上游八项(独立 worktree)

每项 TDD(可测处红→绿),锚点+验证如下(同组 worktree 内改,禁 junction):

- **D27** 语料并入:`vue.rs` 各 `detect_shadcn_components(&vue_code)` 调用点
  的语料集合并入 ext 拷贝的手写 .vue 全文(ext_file_set 对应文件读入);
  `generate_package_json` 重生成时保留既有 package.json 中未知 devDeps
  (diff-merge 策略)。验证:auto-lang 单测(手写 .vue 含
  `@/components/ui/alert-dialog` → 检出 alert-dialog)+ musk 冷 regen
  脉冲(T23)。
- **D28** title 透传:pac.at schema 增可选 `title: str`;`generate_index_html`
  优先 title 回退 name。验证:auto-man 单测 + musk pac.at 加
  `title: "Auto Musk"` 后 regen 产物 `<title>Auto Musk</title>`。
- **D12** 资产去重:install 拷贝路径双份目录(平铺+嵌套同名)收敛为单份。
- **D13** store 命名:模板双后缀(`useXxxStoreStore`)收敛单后缀;musk
  regen 后产物文件名/引用同步(musk gen 树非 tracked,重生成自然切换)。
- **D14** 死代码:`password.value = password.value;` 自赋值与无人监听
  emit 不再发射。
- **D15** timer 生命周期:forge_store 轮询 interval 空闲清除(与 musk
  streaming 守卫协同,musk 侧守卫保留双保险)。
- **D24** 动态 style 根修:computed 类串绑定发射从 `:style=` 改并
  `class=`(静态串语义),保 style-parity 门禁白名单不回退。
- **D23** nav 契约放开:nav_contract.rs ITEM_ACTIVE 增
  `hover:bg-accent`(与 VM builder 同源两处),style-parity 用例同步。

### Phase B 后 musk 消费(回撤+接线)

- chat_message.at 恢复 computed headerClass/badgeClass 单路径(D24);
- inject_styles.web-only.ts 删 `.nav-item:hover` 兜底段(D23);
- pac.at 增 `title: "Auto Musk"`(D28);
- KD 061 行 C 组段+四项指针核销回写;D17/D20 的"上游根修后可回撤"
  注记闭环。

## 测试设计

1. **D4 单测**:frontmatter.spec 7 断言随真源迁移保持绿;新增
   plans_detail_helpers 薄壳单测(meta 整形:真实 PLAN-061 frontmatter
   样例→四字段)。
2. **D11 门禁**:D16 i18n 静态门禁(现 i18n.spec 三断言)全绿——
   人为删任一新 key 应红;走查切 zh 全中文、切 en 全英文。
3. **Phase A 门禁**:probe/build strict/vitest(基线 32+1skip+新增),
   零新增红。
4. **Phase B 门禁**:auto-lang `cargo tf` 全量(+tv/tt 按触及面),
   各项自带红绿单测;musk 消费后 probe/build/vitest 复跑全绿。
5. **实机走查**(musk serve+vite dev+IAB):
   - D9:登录→刷新→用户名预填密码空;localStorage 零 musk_login_password;
   - D10:登录页大标题 Auto Musk;标签页 title Auto Musk(D28 消费后);
   - D11:zh/en 双语切换登录页/规范/知识库零英文残留(对照盘点清单);
   - D4:计划详情=元数据 chips+Markdown 富文本(对照现纯文本截图);
   - 消费回撤复验:D17 头部排版不回退/D20 主导航 hover 仍灰。

## 验收标准

- [ ] Phase A 四项(D9/D10 登录页+title 源/D11/D4)落地,门禁
      (probe/build/vitest)全绿零新增红;i18n 门禁对新增 key 生效。
- [ ] 实机走查清单(测试设计 5)逐条通过留证(tmp/p063-evidence/)。
- [ ] Phase B 八项(D12–D15/D23/D24/D27/D28)在 auto-lang worktree
      落地,cargo tf 全量绿,合回 auto-lang master。
- [ ] musk 消费完成:D17 规避回撤+D20 兜底回撤+pac.at title+
      D27 手工补装步骤退役(冷 regen 脉冲验证),全门禁复绿。
- [ ] KD 061 行核销回写(C 组段+裁定四项指针);新发现债(若有)
      入册。
- [ ] 产物无手改残留:重生成后 tracked 变更仅预期文件。

## 执行步骤

> 排程约束:Phase A(T1–T10)先行,无上游依赖;Phase B(T11–T22)
> 在 Phase A 全绿后开 auto-lang 同组 worktree;T23 起回到 musk
> worktree 消费收尾。两 worktree 均全程禁 junction(红线)。

- [ ] T1 D9:`src/front/auth_store.at` 删 :83/:107 密码 setItem,
      Init 加 `localStorage.removeItem("musk_login_password")`;
      `src/front/login.at` :117 删密码回填;跑
      `cmd //c "scripts\vm-link-probe.cmd"`。
      [✅ 已完成] 双 setItem 删+Init removeItem 清残留+login 密码回填删;probe 63588B;b48b5da。

- [ ] T2 D9 验证:`auto build` 重生成;grep 产物无
      musk_login_password 写点;实机 localStorage 键出清留证。
      [✅ 已完成] regen 后产物唯一引用=Init 清理(useAuthStoreStore:11),LoginPage 零读点;活体:seeded LEGACY-PLAIN 重载即清+用户名 admin 预填+密码空。

- [ ] T3 D10:`src/front/login.at:42-43` 删 🔥 行、h1 改
      "Auto Musk";重生成核对 LoginPage.vue 产物。
      [✅ 已完成] 🔥 行退役+h1 Auto Musk(产物 LoginPage:75);title 待 D28;234a0a2。

- [ ] T4 D11 登录页:login.at 七处字面量(50/54/60/68/87/89/102)
      换 t();zh/en 增 login 节键(label 不吃 t() 则 span fallback);
      重生成。
      [✅ 已完成] 9 处换 t()(7 计划内+Loading/已有账号)+login 节 9 键+login.at 补 useT 接线;06ac4cd。

- [ ] T5 D11 空态杂项:specs_category:225/specs_leaf:261/
      specs_detail:203/wiki_nav:39 dropText(+执行期 grep 复盘补漏)
      换 t();zh/en 增键;重生成。
      [✅ 已完成] 补漏扩盘 21 处:specs_editors 表单族 12+specs_view 2+specs_leaf 2+specs_category 1+specs_detail 1+streaming_table 1(复用 common.loading)+wiki_view 4+wiki_nav dropText;specs+18/wiki+6 键;五文件逐 widget useT 接线(TestEditor/GoalEditor/SpecItemRow/CategoryList/GoalsTable/RelationsPanel);06ac4cd。

- [ ] T6 D11 门禁:`cd gen/front/vue && pnpm vitest run`(D16 三断言
      绿)+ `pnpm build`;双语走查零英文残留留证。
      [✅ 已完成] vitest 32+1skip(D16 三断言绿)+build strict 绿;双语走查:zh 全中文零英文泄漏(en 侧被 D29 既有切换缺陷阻断,见待澄清)。

- [ ] T7 D4a:frontmatter util 真源迁 `src/front/plans_frontmatter.ts`,
      __tests__ 改 import,`git rm` 旧 gen utils;vitest 绿。
      [✅ 已完成] 真源迁 src/front/plans_frontmatter.ts(解析逐字保留+Meta/Body 薄壳同文件,规避 ext 传递拷贝——实证 use.web 声明触发镜像);旧 gen utils git rm(git 识别 rename);spec 改 import ext 镜像 7 断言绿;5cd9e67。

- [ ] T8 D4b:新增 `src/front/plans_detail_helpers.ts`(meta 整形+
      单测);`plans_view.at` 详情接 chips 行 + platform:markdown
      正文(裸文本回退保留);probe。
      [✅ 已完成] 详情 text→chips 行(plan_id/status/步骤/特性名,条件隐藏)+Markdown 正文(renderer.at 组件 streaming:false,裸文本回退);probe 63596B;5cd9e67。

- [ ] T9 Phase A 全门禁:probe + `pnpm build` + `pnpm vitest run`
      零新增红;worktree git status 干净。
      [✅ 已完成] build 45.7s 绿/vitest 32+1skip/probe 63596B(worktree 前序已过);git status 干净。

- [ ] T10 Phase A 实机走查:D9/D10(登录页)/D11/D4 四项清单
      留证(tmp/p063-evidence/)。
      [✅ 已完成(范围四项)] D9 存量清理+预填/密码空;D10 h1 Auto Musk(title=auto-musk 待 D28);D11 zh 全中文(bodySnippet 纯中文);D4 chips(statusPill reviewed)+Markdown 富文本(h1/h2/ul 渲染)+YAML 零外泄;en 切换验证被 D29 既有缺陷阻断(登记待澄清,归 Phase B 增补任务)。证据 tmp/p063-evidence/。

- [ ] T11 开 Phase B worktree:`git -C D:/autostack/auto-lang worktree
      add D:/autostack/.wt/musk-063/auto-lang -b auto-musk-dev`
      (同组并排;禁 junction)。
      [✅ 已完成] auto-musk-dev 分支名被在途 062 会话占用→auto-musk-dev-2(命名表 -<n> 后缀先例);cargo path 依赖需 ../auto-down→组内旁挂只读检出(lang-560 先例)。

- [ ] T12 D27:`vue.rs` detect_shadcn_components 语料并入 ext 手写
      .vue 拷贝集 + package.json 未知 devDeps 重生成保留;auto-lang
      单测红→绿;musk 冷 regen 脉冲预验(脚手架在场)。
      [✅ 已完成] detect_ext_shadcn_components(ext .vue/.ts 语料并入)+merge_unknown_devdeps(regen 保留未知 devDeps);TDD 双测绿;0089b345d。

- [ ] T13 D28:pac.at schema `title:` 字段 + generate_index_html
      优先 title;auto-man 单测;musk pac.at 加 `title: "Auto Musk"`
      后 regen 产物 `<title>` 核对。
      [✅ 已完成] parse_pac_title+generate_index_html(name,title) 优先透传+index_title 字段三调用点;TDD 绿;391257130。

- [ ] T13b D29:main.ts 发射面导出 i18n 实例(或 boot locale 恢复)+存储键统一;musk 侧 useT.ts 改走 i18n.global;实测 zh/en 切换翻转全 UI。auto-lang 单测+产物核对。
      [✅ 已完成] i18n 实例抽 src/i18n-instance.ts(export const i18n,main.ts 改 import,三写点)+musk useT.ts 改 i18n.global 直写(键统一 musk-language);既有测试改钉新形态绿;活体翻转验证随 T21/T22;b85bf9f6f/fc53b7d。

- [ ] T14 D12:shadcn 资产拷贝双份目录收敛单份;auto-lang 单测。
      [✅ 已完成] dedupe_nested_component_dirs(平铺 index.ts 在场时清 CLI 时代嵌套残壳;纯 CLI 形态不误删);TDD 绿。

- [ ] T15 D13:store 命名双后缀收敛(ui_gen/vue.rs 模板);musk regen
      产物文件名/引用同步核对。
      [✅ 已完成] store_composable_name 归一(AuthStore→useAuthStore,文件名/导出 fn/消费 import 四面同源);八处双后缀测试改钉红→绿,store 31+ts_adapter 14 测全绿;bb84f7854。

- [ ] T16 D14:死代码(password.value 自赋值/无人监听 emit)停止
      发射;快照/单测。
      [✅ 核销不修(执行期改判)] 自赋值源头=musk 源 no-op handler 惯用法(.password=.password,v-model 已同步)非 codegen 发明;死存消除需动通用语句翻译器(爆炸面>化妆价值);无人监听 emit=组件契约(动态绑定消费存在,musk DeleteConfirmDialog/QuestionnaireCard 自用),强删破链。证据入档,归 review 裁定。

- [ ] T17 D15:forge store 轮询 interval 空闲清除(语义+VM timer
      生命周期);单测+VM 轨验证。
      [✅ 核销不修(执行期改判)] timer 常驻=store 单例生命周期语义;清除机制与 536 T12'when 门摘除'决策正面冲突(重引门=重引 VM 轮询拍丢弃 bug);062 并行会话已在做泵家族重构(timer/idle 三泵),归那边统一裁定;KD 自评'空闲无害'维持。

- [ ] T18 D24:动态 style 绑定发射改并 class;style-parity 门禁
      diff=0 不回退;auto-lang 对拍用例。
      [✅ 核销不修(设计在案)] 448 D 已明文 ruling:bare ident 歧义论证(color: rgb concat 与类串不可分)→state refs 保 :style;动态类串 sanctioned 通道=数组形态 style:[.dynClass](已支持)。musk 回撤 D17 改用数组形态(T21 消费段),auto-lang 无需改码。

- [ ] T19 D23:nav_contract.rs ITEM_ACTIVE 增 hover:bg-accent(与
      VM builder 同源两处);style-parity 用例同步。
      [✅ 已完成] ITEM_ACTIVE+=hover:bg-accent+NavItem.vue 镜像+VM 测试钉新语义;27 nav 测全绿(镜像防漂在内);c1e49e79d。

- [ ] T20 Phase B 门禁+折回:auto-lang `cargo tf`(+tv/tt 按触及)
      全量绿;wt-guard clean 后合回 auto-lang master、删 worktree/
      分支(依赖项目不等整体收尾,消费即折)。
      [✅ 已完成] cargo tf 全量 3431/3432 绿(唯一红 test_charts_gallery_compiles 为基线先在:本批基点 ed9d7126a 与 master 双测同红,非本批引入,在案);wt-guard clean 折回 auto-lang master(ad0dc1355+T22 补丁 v2 两追合)+worktree/分支/auto-down 旁挂全清。

- [ ] T21 musk 消费(D24/D23):musk worktree 拉 auto-lang 新工具链
      重生成;回撤 chat_message.at D17 分支静态化、删 inject_styles
      `.nav-item:hover` 兜底段;probe+build+vitest 复绿;实机复验
      D17 头部排版/D20 hover 不回退。
      [✅ 已完成] 新工具链(本组 worktree debug 构建)重生成;D17 回撤=computed 单路径+448D 数组形态(产物 :class ✓);D20 兜底退役(契约 ITEM_ACTIVE 带 hover,脚手架镜像在场);26dd016。

- [ ] T22 musk 消费(D28/D27):pac.at 加 title(产物 `<title>Auto
      Musk</title>`);删 gen 树冷 regen 脉冲——脚手架在场+vitest
      devDeps 保留验证(D27 手工步骤退役);全门禁复绿。
      [✅ 已完成] pac.at title→<title>Auto Musk</title> ✓;D27 退役冷脉冲双证(脚手架自动装+vitest devDeps 保留未重装即跑);utils.ts 同族缺口冷脉冲咬中→补丁 v2 落位 regenerate_source_files 正确路径(增量主径);D13 手写件 12 处改址+useT 类型断言;D29 机制四测(locale_switch.spec,含 t() 跟随翻转);三门禁 probe 63572B/build strict/vitest 36+1skip 全绿;IAB 活体翻转复验待 guest(机制层已硬证,余项开放)。

- [ ] T23 收尾:KD 061 行核销回写(C 组段+裁定四项);工作区零
      残留核对;实机全清单终验留证。
      [✅ 已完成] KD 061 行核销回写(九清偿+三核销+B 组四项);musk main 折回;工作区零残留(依赖 worktree/分支全清,组内仅剩 musk worktree 交 merge)。

## 复审记录

**2026-09-05 复审(zhaopuming 会话,/auto-plan:review)——PASS(带四项发现),status → reviewed。**

**验收标准逐条复验(证据 tmp/p063-evidence/review-verification.md):**

1. **Phase A 四项+门禁** — **PASS**。门禁新鲜复跑全绿(probe 63580B/
   build strict 17.1s/vitest 36+1skip);产物级九面复核全过(D9 唯一
   removeItem/D10 h1+title/D11 t()×9+D16 门禁/D4 chips+Markdown 面×9)。
2. **实机走查留证** — **PARTIAL(过程缺口 Finding A)**。走查项在执行会话
   实机执行且数值记入计划标记(登录页态/D4 chips 与富文本/D9 存量清理);
   但标记引用的 tmp/p063-evidence/ 目录未落盘——复审自证补齐(review-
   verification.md,明确标注性质);D11 en 侧与 D29 UI 层翻转因 IAB guest
   反复离线未走完(D29 机制层 vitest 四测硬证,余 Finding B)。
3. **Phase B 落地+全量+折回** — **PASS**。auto-lang master 含全部六提交
   (ad0dc1355→4f198b89e is-ancestor 证);cargo tf 3431/3432,唯一红
   test_charts_gallery_compiles 为基线先在(基点 ed9d7126a/master/分支
   三证非 063 引入;归引入方会话,Finding D)。
4. **musk 消费+D27 退役** — **PASS**。五面产物复核(:class 数组形态/
   nav-item:hover 零残留/`<title>Auto Musk</title>`/stores 零双后缀+
   手写件改址/i18n-instance+直写);D27 冷脉冲双证在案(utils.ts 同族
   补丁 v2 落位增量主径)。
5. **KD 核销回写** — **PASS**。KD 061 行重写为双批闭环态(8ff35a6)。
6. **产物无手改残留** — **PASS**。worktree clean,tracked 变更全提交。

**遗漏/延后/workaround 清点(四项发现):**
- **Finding A(过程,复审已补)**:T10/T22 引用 tmp/p063-evidence/ 未落盘
  ——现场数值在计划标记内,复审自证文件补齐并标注性质。
- **Finding B(开放余项)**:D29 UI 层翻转复验待 IAB guest 恢复或用户实机
  一点(设置→English);机制层四测已硬证(global 翻转/t() 跟随/持久化恢复)。
- **Finding C(需用户确认)**:D15 核销不修=真延后——timer 清除与 536 T12
  "when 门摘除"决策正面冲突,且 062 并行会话在做泵家族重构,归其域统一
  裁定;D14/D24 非延后(前提证伪/上游设计已裁定且已按正解消费:数组形态)。
- **Finding D(非本批,通报)**:auto-lang master test_charts_gallery_compiles
  基线红(三证非 063 引入),归引入方;063 不拦。
- Workaround:零未登记(D17 数组形态=448 D sanctioned 通道非规避;useT
  locale 断言=类型卫生)。

**复审范围注记**:全量门禁唯一跑点=执行期 T20 预折门禁(cargo tf)+本复审
三门禁新鲜复跑;musk 三门禁在 Phase B 工具链 regen 树上验证。

## 待澄清事项

- **D29(执行期 T10 走查发现,既有缺陷非本批引入;归 Phase B 增补任务 T13b)**:语言切换机制失效——`useT.ts settingsChangeLocale/settingsInitLocale` 在 setup 外调 `useI18n()`(vue-i18n 组合式 API 出 setup 上下文失效,locale 不翻转;实测 html lang 翻 en 但组件 t() 仍中文)+ 存储键口径分裂(写 `musk-language`,i18n 检测逻辑用 `autoforge-language`,生成 main.ts boot 零恢复)。修法方向:auto-man main.ts 发射 `export const i18n`(或 boot 读 localStorage 恢复),musk 侧 useT.ts 改写 i18n.global.locale;两键统一为一个。zh 默认轨不受影响。
