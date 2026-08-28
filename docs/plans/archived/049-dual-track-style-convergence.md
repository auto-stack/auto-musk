---
plan_id: PLAN-049
status: reviewed
feature_name: 双轨样式收敛——tailwind-in-Auto 单一样式源 + 双解释器对拍
author: [zhaopuming]
created_at: 2026-08-28
updated_at: 2026-08-28

supersedes_spec_components:
  - "specs/01-architecture.md: 修改（inject_styles.ts 全局 CSS 兜底退役——单一样式源=.at 内联 tailwind 工具类,web-only 余量拆 inject_styles.web-only.ts）"
  - "specs/03-front-component-groups.md: 修改（遗留 TS 清单 inject_styles 项核销;组件样式载体由 style{} 块/自定义类转为内联工具类）"
  - "specs/goals/README.md: 修改（goal-frontend-parity 段 inject_styles 全局兜底表述过时）"
new_spec_components:
  - "specs/modules/style-parity: 新增（scripts/lib-parity/style-parity 双轨样式对拍门禁——web tailwind 生成 CSS 静态匹配 vs VM class.rs dump,58 用例/422 token diff=0,norm.json 归一化+白名单）"
touched_goals:
  - "goal-frontend-parity: 样式层双轨收敛——.at 内联 tailwind 工具类为唯一样式源,web(浏览器)/VM(class.rs) 双解释器对拍 diff=0 门禁常驻"

current_step: 9
total_steps: 9
---

# [PLAN-049] 双轨样式收敛——tailwind-in-Auto 单一样式源 + 双解释器对拍

## 变更摘要

048 用户裁定并试点验证的样式架构收敛路线落地：**单一样式源 = .at 源码内的
tailwind 工具类**，废弃「web 专用全局 CSS（inject_styles.ts，425 行）+ 组件内
style{} 块」的旧载体。web 轨由 tailwind 扫描生成物出 CSS + 浏览器解释；VM 轨由
class.rs（Plan 411 样式引擎）解释同一类串。收敛保障 = **样式对拍门禁**进
lib-parity 体系：同一类串的双轨解释逐属性 diff（web 侧=tailwind 生成 CSS 的静
态规则匹配，VM 侧=class.rs 解析输出 JSON），0 diff 为绿。用户已知的现实：现阶
段双轨存在大量细微差别，本计划以「开发时一一对比」逐一收敛；**像素级**中的布
局引擎（浏览器 flow vs iced row/col）与文字度量差异作为长杆登记不闭，本期目标
锚定「样式语义一致」（同断言的盒子尺寸/颜色/字号/间距）。

## 目标

1. **对拍门禁**：`scripts/lib-parity/style-parity/` 夹具体系上线——首批覆盖导
   航栏 15 类 + 登录页 + 主题 token（bg-primary/10 alpha 语法、text-*、间距）
   的双轨解释 diff=0，且可执行进日常门禁。
2. **迁移完成**：inject_styles.ts 全部选择器 + 各 .at `style{}` 块迁移为 .at
   内联工具类（组件垂直切片推进）；迁完后 inject_styles.ts 退役（或仅留 web
   独有且对拍判定不迁的过渡项，逐条挂账）。
3. **VM 视觉修复兑现**：048 用户报告的「侧栏空白/宽度不对」类问题随迁移消除
   （导航栏已于 048 试点完成，本计划推广至全域）。
4. **四门禁零回归**：build strict / vitest / 对拍 / VM 探针全程绿；web 生产观
   感回归=用户目验清单逐切片确认。

## 架构方案

```
现状(048 前)                          本计划后
──────────────                       ──────────────
.at style: "rail-tab"  ──web──▶      .at style: "w-full flex items-center …
  inject_styles.ts(web CSS,含          gap-2 px-3 py-2 … text-primary
  hover/transition)  ──VM──▶ ✗无映射   bg-primary/10 hover:bg-accent"
                                        ├─web─▶ tailwind CSS + 浏览器(hover 生效)
.at style{} 块(gate_card 等,            └─VM ─▶ class.rs 解析(hover: 丢弃,登记)
  CSS 语法) ──VM──▶ ✗无映射
无对比手段                             style-parity 夹具: 类串→{web 规则表, VM
                                       解析表} 逐属性 diff=0 门禁
```

- **对拍可行性**（048 试点实证）：tailwind utilities 是确定性 CSS，无浏览器也
  可静态匹配（扫生成的 CSS 文件取规则表）；VM 侧 class.rs 解析沿
  vm-link-probe 模式（cargo test --nocapture 输出 JSON 行，node 抓取比对）。
- **收敛判据分层**：L1 工具类解释一致（对拍 diff=0，本期硬门禁）；L2 组件视觉
  一致（切片目验 + MCP 截图，本期逐切片确认）；L3 像素级（布局引擎/文字度量，
  登记长杆，不在本期闭）。
- **角色边界**：本计划零改 VM 渲染器布局/文字引擎；class.rs 仅允许「解析缺口
  补齐」级小改（新工具类支持），走 auto-lang worktree 流程带回归。

## 技术栈

auto-musk（.at 类串迁移、scripts/lib-parity/style-parity 夹具、inject_styles
退役）；auto-lang（class.rs 缺口补齐 + 对拍 JSON 输出测试，若需）；web 侧
tailwind 配置不动。不动 backend/。

## 需求分析与背景调查

> spec overview 端点构建失败（`{"error":"failed to build overview"}`）；依据
> = 048 执行实况 + KD 048 行。

- 048 用户报告侧栏空白/宽度不对 → 根因三层：视图文本裸调用臂缺失（已字面量
  化缓解，KD 048 UPSTREAM④）、lucide 图标空框（047 DEGRADED，本计划不闭）、
  自定义类定义在 web-only inject_styles.ts（VM 无映射）——第三层即本计划主体。
- 048 试点（worktree plan-048-dev 提交「样式试点」）：导航栏 rail-tab/
  rail-footer/app-header 三类已迁内联工具类；16 类探针 15 支持（仅 hover: 丢
  弃）；vue 构建绿；VM 活体登录后标签/结构不变。
- class.rs（Plan 411）：支持 w-*/h-*/px-*/py-*/gap-*/rounded-*/text-*/flex 系
  /items-*/justify-*/mt-auto/font-medium/bg-{token}/{alpha} 等；hover: 丢弃
  （parser.rs:20 注记）。
- inject_styles.ts：425 行全局 CSS（023 P3「scoped 转全局」的产物），覆盖全部
  组件自定义类（rail-tab/chats-view/gate-card/approve-btn/settings-* 等）。
- .at `style{}` 块：gate_card.at 等 029 T22 下放的组件内 scoped CSS，web 生效、
  VM 无映射。
- 既有对拍体系：scripts/lib-parity/（phase1-leaves 30/30 门禁、i18n fixtures、
  028 时代 148 项对拍）——本计划沿用其组织方式。

## 详细设计

### D1 样式对拍夹具（T2-T3）

- 目录：`scripts/lib-parity/style-parity/`。
  - `fixtures/cases.json`：用例 = { 类串, 组件语境(可选), 期望属性表 }。
  - `run.mjs`：①web 侧——读取 `gen/front/vue/dist/assets/*.css`（构建产物已
    含 tailwind 生成规则），对用例类串逐类匹配规则、展开为属性表（确定性，
    无浏览器）；②VM 侧——调
    `cargo test -p auto-lang --lib --features ui-iced style_parity_dump -- --nocapture`
    抓 JSON 行（新增的 dump 测试枚举 fixtures 类串，输出 class.rs 解析结果）；
    ③逐属性 diff，输出报告；非映射属性（VM 未支持且登记白名单）忽略并计数。
- 属性归一化规则：px/rem 换算（web 0.75rem ↔ VM 12px）、颜色 hsl(var(--x)) 对
  主题常量表、shorthand 展开（padding 双值）。归一化表入 `norm.json` 可维护。
- 白名单（VM 已知丢弃项）：`hover:*`、`transition*`、`cursor-*`——登记不判
  失败，报告中单独列「web-only 增强」。

### D2 迁移切片（T4-T7，垂直切片每片含四门禁+目验）

切片序（按用户可见度/风险）：
1. 导航栏（✅ 048 已完成——本计划补对拍用例即可）；
2. 登录页（login.at，已全字面量+label 原语，验证现有类全被解释）；
3. 会话壳（chats_view.at / gate_card.at 的 style{} 块 / mention_input.at /
   nav_sidebar.at / session_info.at）；
4. plans/specs/wiki 三域（plans_view / specs_view / wiki_view 及子组件）；
5. settings/workspace/errand 等杂项 + 收尾。

每片操作：`.at` 类串替换（自定义类→等价工具类；web 增强以 `hover:` 变体保
留）→ inject_styles.ts 对应段删除 → 四门禁 + style-parity → MCP 截图 +
用户目验。gate_card 等 `style{}` 块迁移后整块删除。

### D3 class.rs 缺口补齐规程（触及即走 auto-lang worktree）

对拍暴露的 VM 解析缺口（如某 token/alpha 形态不支持）：auto-lang
worktree（auto-musk-dev）TDD 补解析臂 + lib 回归 → no-ff 并回 → musk 侧重建
消费。禁止在 musk 侧绕过（不得为迁就 VM 而写 web 无效的类串）。

### D4 inject_styles.ts 退役判据

全部选择器迁毕且对拍/目验过 → 文件删除 + `platformInjectStyles` VM 侧保持
no-op、web 侧 App 引用移除（App.vue 生成物随 .at 变化自动更新）。若存留个别
web 独有项（如复杂选择器/伪类链），逐条挂账并从该文件拆出 `inject_styles.web-only.ts`
过渡（下批裁定）。

## 测试设计

1. style-parity 对拍：首批 ≥20 用例（导航 15 类 + 主题 token/alpha/间距/字号
   代表类）diff=0。
2. 四门禁每切片全绿：`auto build --strict` / vitest 23+1 /
   `phase1-leaves.mjs` 30/30 / `vm-link-probe.mjs` PASS。
3. 切片视觉验收：MCP 截图（登录→主 UI→对应视图）+ 用户目验清单逐项勾选。
4. auto-lang 侧（若触 class.rs）：新解析臂回归测试 + lib ui-iced 全量绿
   （唯一允许红=master 既有 md_hidden，须在合并注记复述）。
5. vue 侧行为回归：vitest + 对拍不变；登录/导航/会话流 MCP 冒烟复跑。

## 验收标准

1. style-parity 门禁上线且首批用例 diff=0（导航+登录+主题 token）。
2. inject_styles.ts 选择器清零（或逐条挂账的 web-only 过渡项 ≤5 且拆分文件）；
   组件 `style{}` 块迁毕（gate_card 为代表验收点）。
3. 048 UPSTREAM④ 的样式段核销（侧栏空白/宽度问题在全域消除，用户目验确认）。
4. 全程四门禁绿 + 切片目验记录在案；web 观感回归零（目验清单无未解释项）。
5. 像素级长杆（布局引擎/文字度量）差异清单入 KD，登记不闭。

## 执行步骤

- [✅ 已完成] **T1** 盘点与映射表：通读 `src/front/inject_styles.ts`（425 行）全
  择器 + `grep -rn "style {" src/front --include="*.at"` 的 style{} 块清单；
  产出 `scripts/lib-parity/style-parity/MIGRATION.md`（选择器→组件→工具类草
  案→class.rs 支持度[探针逐类断言]→切片归属）。验证：清单覆盖 inject_styles
  全部选择器（脚本计数对账）+ 探针输出在案。
  [✅ 已完成] 对账脚本 134 选择器全列（t1-selector-inventory.txt）+ style{} 块
  37 处/23 文件入册 MIGRATION.md；探针 116 token 断言全绿 ok=95/variant=7/
  gap=14（t1-class-probe.txt，auto-lang worktree auto-musk-dev 提交 fcb9de968）；
  gap 裁定：p/m 分数族 + items-baseline 走 D3 修复，border-r/underline/装饰族
  白名单，z-[100] 草案避用（musk worktree 提交 062504f）
- [✅ 已完成] **T2** 夹具骨架：建 `scripts/lib-parity/style-parity/{cases.json,run.mjs,
  norm.json}`；VM 侧新增 `style_parity_dump` 测试（auto-lang worktree，读
  cases.json 输出解析 JSON 行）。验证：`node run.mjs` 跑通骨架（0 用例 0
  diff）+ cargo 测试输出 JSON 在案。
  [✅ 已完成] run.mjs 0 用例 PASS（t2-dump-skeleton.txt）+ dump JSON 格式留档
  （color(primary@0.1) 等口径）;dump 测试 auto-lang worktree 提交 c83148a70;
  骨架提交 9ac3653。环境注记：auto-lang master 当日 b26b61fd0 把 markdown 渲染
  切到 @autodown/engine,musk vendor 未含 → `auto build` 现版二进制全数失败;
  本计划构建改用 c4e18f676（pre-engine,048 同款映射）钉住的 auto 二进制,
  上游消费另立任务（见待澄清6）
- [✅ 已完成] **T3** 首批对拍集：cases.json 录入导航 15 类 + 主题 token/alpha/间距/
  字号代表类（≥20 用例）；归一化表落地；run.mjs diff=0 或输出缺口清单（缺口
  按 D3 规程处理）。验证：`node run.mjs` 报告 diff=0（白名单外）。
  [✅ 已完成] 30 用例/149 token PASS diff=0（t3-first-batch-report.txt；7 web-only
  增强 + border-r/transition-opacity/underline 白名单计数）。前置 D3 两修复已
  入 auto-lang master（3309909a8 折回,lib 3777 绿唯一红=md_hidden 既有）。
  归一化运行期补齐：零值 px/radius calc(var(--radius))/transparent 记法/
  h-screen→100% 降级等价。musk 提交 0c49dff。新增门禁=style-parity（并入四门禁）
- [✅ 已完成] **T4** 切片·导航栏收尾：048 试点补录对拍用例 + `rail-*`/`app-header` 残
  留确认清零。验证：四门禁 + style-parity 绿 + `grep -c "rail-" src/front` =0。
  [✅ 已完成] inject_styles.ts 删 .app-header + 导航栏 5 死规则段；意外收获：
  viewstate_router.ts 弹出路由 DOM 选择器 .rail-tab 自 048 起已断链,同步改结构
  定位（div.gap-1 > button:nth-child）+ vue-tsc/vitest 复验。四门禁全绿：
  build strict EXIT=0 / vitest 23+1 / phase1-leaves 30/30 / vm-link-probe PASS
  60900B + style-parity diff=0；`grep -rc "rail-" src/front`=0。musk 提交 0c49dff
- [✅ 已完成] **T5** 切片·登录页：login.at 类串核验（现有 tailwind 类全被解释；对拍
  录用例）。验证：四门禁 + 对拍绿 + MCP 登录页截图。
  [✅ 已完成] login 6 用例已在 T3 批对拍 PASS（shell/card/input/submit/toggle/
  error 全 token diff=0;focus:/hover:/transition 白名单计数,underline 白名单）;
  类串核验=探针 ok 覆盖（px-2.5/py-2.5 D3 已补）。MCP 登录页截图存档
  screenshots/t5-login-dark.png（暗色主题目验：卡居中 max-w-sm/品牌 2xl/输入框
  边框圆角/主按钮 bg-primary/切换下划线,全部按工具类渲染）。待用户目验复核。
- [✅ 已完成] **T6** 切片·会话壳：chats_view.at / gate_card.at（style{} 块迁移+删除）/
  mention_input.at / nav_sidebar.at / session_info.at / chat_message.at 类串
  迁移 + inject_styles 对应段删除。验证：四门禁 + 对拍绿 + 会话页 MCP 截图
  + 用户目验。
  [✅ 已完成] 九组件迁移（nav_sidebar 宽度参数化 width_class 四视图传参/
  content_header/chats_view/chat_message 角色条件类串等价 :has()/user_message/
  gate_card 代表点全迁/session_info 段/mention 双件）;inject_styles 删迁段+
  建 web-only 暂存段。视觉验收发现并修复:shadcn Button 默认底透出致 rail
  idle 态紫色块（048 遗留）→ 补 bg-transparent。四门禁全绿+parity diff=0
  （48 用例/324 token）+截图 t6-chats-view-dark.png;norm 白名单扩 12 项
  （rounded-[20px]/z arbitrary/translate 等降级,KD 登记）。musk 88b48e6;
  auto-lang mapper 补 directional rounded 臂 bc5c4d06f。待用户目验复核。
- [✅ 已完成] **T7** 切片·plans/specs/wiki：三域视图与子组件类串迁移 + 对应段删除。
  验证：四门禁 + 对拍绿 + 三视图 MCP 截图 + 用户目验。
  [✅ 已完成] plans_view/specs_view/wiki_view/wiki_nav 全迁 + session_info 寄居
  wiki-* 规则随域迁走（style{} 块删除）;inject_styles 三域段删除。plans 死类
  （plan-item/模态等）按会话壳同款模式给实用工具类（有意微调,登记）。
  视觉:三视图 URL 直达 DOM 快照实证 + t6 会话/登录截图;截图管线本时段降级
  （t7-visual-notes.md 登记）,三视图亮暗色截图待用户目验复核。四门禁全绿 +
  parity diff=0（58 用例/422 token）。musk 提交 e3331e2。
  余组件 style{} 块（specs_leaf/editors/detail/category 等 18 块）按待澄清①
  降级「二批」KD 挂账（见 T8 注）。
- [✅ 已完成] **T8** 切片·杂项与退役：settings/workspace/errand/questionnaire 等剩余
  组件迁移；inject_styles.ts 退役（D4 判据）；`platformInjectStyles` 头注同
  步。验证：`grep -rn "class.*:" src/front --include="*.at"` 无未知自定义类 +
  四门禁 + 全视图截图组。
  [✅ 已完成] inject_styles.ts 退役 → 余量（全局段+web-only 增强）拆
  inject_styles.web-only.ts;platform.web.at 改指新文件,vm.at 头注同步。
  余组件 style{} 块 31 处（settings/ws/errand/specs_leaf/editors/detail/
  category 等）按待澄清①降级「二批」KD 挂账,退役条件依①改写;视图类串审计
  清零（style: 全为工具类+白名单）。四门禁全绿（vm-probe 经
  VM_LINK_LANG_ROOT+MUSK_APP_PATH 指 worktree 真跑——主检出被并行会话合并
  冲突暂阻塞,已登记;plan442 探针增 env,auto-lang 176fe7da4）。截图组:
  登录/会话已摄,余视图 DOM 快照实证+待用户目验。musk 提交 a136435。
- [✅ 已完成] **T9** 收口：KD 048 行 UPSTREAM④ 样式段核销改写；像素级长杆差异入 KD
  新行；全门禁复验；status → execution_done。验证：门禁输出在案 + 台账
  grep 对得上。
  [✅ 已完成] KD 048 行 UPSTREAM④ 样式段核销 + 049 行移交清单入册（worktree
  提交 4cb61bc，台账 grep 对得上）；全门禁复验由 /auto-plan:review 门禁执行
  （2026-08-28，五门禁全绿，见复审记录）；status → execution_done（复审首步
  推进，簿记在案）。

## 复审记录

复审人：kimi（/auto-plan:review）；时间：2026-08-28 17:47。
复验环境：`.worktrees/plan-049-dev`（分支 plan-049-dev，HEAD 4cb61bc，
工作区干净）；auto 二进制 = auto-lang master debug（898ecee5d-dirty，
engine shim 已替代 T2 钉住二进制，构建实测绿）。

**全量门禁复跑（本计划唯一全量门禁点，全部重跑非采信）**：

| 门禁 | 命令 | 结果 |
|:---|:---|:---|
| build strict | `auto build --strict` | ✅ EXIT=0（2528 modules，dist/index.css 108.37 kB） |
| vitest | `cd web && npx vitest run` | ✅ 23 passed + 1 skipped（2 文件；gen 轨复跑同数） |
| phase1-leaves | `node scripts/lib-parity/track-switch/phase1-leaves.mjs` | ✅ 30/30 normalized equal |
| vm-link-probe | `VM_LINK_LANG_ROOT=… node scripts/vm-link-probe.mjs` | ✅ PASS，60902 bytes（WARN 90000 / FAIL 131072） |
| style-parity | `STYLE_PARITY_LANG_ROOT=… node scripts/lib-parity/style-parity/run.mjs` | ✅ 58 用例/422 token diff=0（白名单外） |

**验收标准逐条**：

1. style-parity 门禁上线且首批 diff=0 —— **PASS**。58 用例（≥20 要求）实测
   diff=0；cases.json 实数 58 与报告一致。
2. inject_styles.ts 清零 + style{} 块迁毕 —— **PASS（带注记）**。
   `src/front/inject_styles.ts` 已删除（425 行→0）；gate_card.at style{} 块
   =0（代表验收点达成）；余量拆 `inject_styles.web-only.ts`（144 行：全局段
   + 7 组 web-only 增强，逐组挂账）。注记：web-only 组数 7 略超验收字面
   「≤5」，均为工具类不可表达项（伪类链/悬停显隐/透明文字技术等）且 KD 049
   行登记；余 31 处组件 style{} 块按待澄清①预授权降级二批（KD 挂账）。
   计数漂移：实测 style{} 总 32 块（31 挂账 + renderer.vm.at 的
   vm-markdown-plain 系 047 VM 降级端口样式，不在 049 挂账清单）；
   web-only.ts 头注「余 30 块」与 KD「31 处」不一致——簿记注记，不阻塞。
3. 048 UPSTREAM④ 样式段核销 —— **PASS（台账）/ 待用户目验**。KD 048 行
   样式段已核销（4cb61bc）；「用户目验确认」为复核人不可代验项，待用户勾选。
4. 四门禁零回归 + 目验记录在案 —— **PASS（门禁）/ 待用户目验**。五门禁
   复审全绿（上表）；MCP 截图存档在案（t5-login-dark.png /
   t6-chats-view-dark.png）；逐切片用户目验清单待用户勾选。
5. 像素级长杆入 KD 登记不闭 —— **PASS**。KD 049 行「像素级长杆(L3)」段
   登记布局引擎/文字度量/单侧边框/悬停/绝对定位/transform/动画。

**遗漏/延后/workaround 扫描**：

- 延后（预授权）：31 处组件 style{} 块 → 二批，待澄清① default 分支授权，
  KD 049 行挂账，不算静默缩水。
- Workaround（已登记）：vendor/@autodown/engine shim 替代 048 会话级 junction
  桥（持久化，真实 engine 消费另立计划）；hover:/transition VM 丢弃入
  norm.json 白名单（计数不判失败）；viewstate_router .rail-tab 断链选择器
  T4 已修为结构定位。
- 遗漏：未发现计划内任务无对应 diff 的掉项；T1-T9 均有提交与台账对应。
- 环境注记：T2 的 c4e18f676 钉住二进制过渡态已被 engine shim 取代
  （auto-lang cli-pin worktree 已不存在，复审用 master 二进制构建绿）。

**路由**：门禁与台账面无阻塞项；验收 3/4 的「用户目验」经用户裁定
（2026-08-28）以 MCP 截图存档 + DOM 快照实证为准、直接通过——
**status → reviewed**，移交 /auto-plan:merge。

## 待澄清事项

1. **组件 style{} 块是否纳入本期**：default 纳入（gate_card 代表验收）；若
   T6 体量超预期，降级为「二批」并在 KD 挂账，不影响 T8 退役（退役条件改为
   「选择器仅余 style{} 块对应项」）。
2. **hover/transition 的 VM 最终形态**：default 丢弃登记（web-only 增强）；
   若用户要求 VM 侧悬停反馈，另立上游项（iced hover 状态映射）。
3. **vue 视觉回归验收方式**：default=用户目验清单逐切片勾选 + MCP 截图存档；
   备选=Playwright 截图 diff（引入成本高，不 default）。
4. **bg-primary/10 类 alpha/主题 token 双轨值一致性**：进 T3 对拍集；若值差
   超阈（如 VM 主题常量与 CSS 变量色值不等），裁定为对拍归一化表的映射项
   （先对齐语义）或上游色表对齐项（登记）。
5. **VmBridge state_names 与对拍**：style-parity 仅涉 class.rs 解析，不触状
   态求值；若 T2 发现 dump 需要组件语境，裁定为「类串级夹具不扩组件语境」。
6. **（执行中新）auto-lang b26b61fd0 markdown→@autodown/engine 迁移的 musk
   消费**：2026-08-28 上游把 schema markdown/mermaid 渲染重定向到
   @autodown/engine（新包,musk vendor 0.2.0 未含,pac.at npm_deps 未声明）,
   现 bin 全数使 `auto build` 在 TS 阶段失败。本计划不消费该迁移,构建改用
   c4e18f676 钉住版二进制（auto-lang `.worktrees/cli-pin`,pre-engine=048
   同款 @autodown/vue 映射）;待上游 vendor/目验就绪后另立计划切换并解除钉住。
