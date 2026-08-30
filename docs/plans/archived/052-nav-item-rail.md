---
plan_id: MUSK-052
status: archived
feature_name: nav-item-rail
author: [zcode]
created_at: 2026-08-29
updated_at: 2026-08-31

supersedes_spec_components: []
new_spec_components:
  - "src/front/app.at: view rail 四项迁 auto-lang Plan 482 nav-item 契约组件（icon/label/active/onclick），删 3.2KB 手搓条件样式串与 8453d6d 逐 text 补色 workaround"
  - "src/front/composables/viewstate_router.ts: popstate 桥选择器由脆弱结构定位改锚 .app-rail .nav-item 契约类"
touched_goals:
  - "goal-frontend-parity: rail 契约组件化 + 选择器锚点稳定（web/VM 双端经 auto-lang 482 class 契约）"
---

# [MUSK-052] view rail 换用 auto-lang Plan 482 nav-item 组件

## 变更摘要
auto-lang Plan 482 落地了 nav-item/nav-group 导航组件族（双端 class 契约 +
hover/active/disabled 三态 + icon/desc/badge 槽 + lucide svg 双端渲染，另修复
VM lucide 图标空渲染缺陷）。本计划把 app.at 的 view rail（会话/计划/规范/知识库
四项）从手搓 button+巨型条件样式串迁移到 nav-item，删除逐 text 补色
workaround（8453d6d），并把 viewstate_router.ts 的脆弱结构选择器换成
`.nav-item` 契约锚点。

## 执行步骤
- [x] T1 app.at rail 四项 → nav-item；use.web 图标 import（仅 rail 消费）已删。
  [✅] rail 3184B 手搓串 → 776B nav-item；-45/+12 行。
- [x] T2 viewstate_router.ts 选择器 → `.app-rail .nav-item:nth-of-type(n)`。
  [✅] NavItem onclick 态渲染 <button class="nav-item">，锚点稳定。
- [x] T3 验证：auto build --gen-only + vite build ✓（3.20s 绿）；生成产物
  NavItem/:active/:icon-comp/@click 核对 ✓。web 全栈冒烟与 VM 轨截图归
  PLAN-050 parity 线复跑（rail 组件本体已在 auto-lang 015/018 双端实测）。
  [✅] 注：worktree 全新工程需 vendor shim dist 与 ui/ 脚手架拷自主 checkout
  （gen 为构建产物，--gen-only 不物化）。

## 验收标准
1. rail 渲染与迁移前等价（图标+文案+选中主色调高亮），样式代码量净减。
2. popstate 后退仍能正确切换 view（选择器更新后）。
3. VM 轨 rail 图标可见（lucide svg 修复随 auto-lang Plan 482）。

上游依赖：auto-lang master（Plan 482 折叠后 target/debug/auto 即含）。

## 复审记录

- 复审人：ZCode（/auto-plan:review），2026-08-31。052 无独立 worktree——执行随
  PLAN-050 批次折叠，迁移提交 c405c40 已在 main，对照主检出 main 0e2ea2e 复核。
- 逐验收裁定：
  1. rail 渲染等价+代码量净减 —— **pass**：c405c40 app.at -45/+12（手搓 3.2KB
     if 样式串 → 四 nav-item 声明，8453d6d 补色 workaround 与 rail 专属图标
     import 删除）；VM 快照 rail 四项（会话/计划/规范/知识库）均带 [Image] 图标
     +文案（tmp/plan050-review/snap-initial.txt）；active 经 nav-item active prop
     （app.at:77-80），高亮底色渲染属 482 双端 class 契约（其单测域）。
  2. popstate 后退切换 —— **pass（静态契约+产物核验）**：viewstate_router.ts:97
     锚 `.app-rail .nav-item:nth-of-type(n)`；gen NavItem.vue onclick 态（无 to）
     root=button、class 串以 `nav-item` 开头 → 选择器↔生成 DOM 匹配实证；diff
     仅换锚点（脆弱结构定位 `div.gap-1 > button:nth-child` 退役），路由行为链路
     沿用 plan-041 T7/T8 既有实现。
  3. VM 轨 rail 图标可见 —— **pass**：快照四 [Image]（lucide svg，随 482 修复+
     050 C5 图标桥）+ first-run alive reds=0 + vm-link-probe PASS 61217B。
- 门禁（2026-08-31 复跑）：auto build strict ✓（vue-tsc+vite 5.72s，gen NavItem
  随产）/ vitest 23+1 skip ✓ / phase1-leaves 30/30 ✓ / style-parity 12 红=基线
  差分恒等 ✓。
- 遗漏/延后/workaround 扫描：T3 注记的"web 全栈冒烟与 VM 截图归 PLAN-050 parity
  线复跑"已由 050 T11/Phase 2 与本次快证覆盖，无悬空；无未登记 workaround。
- 文档规范化：复审时将 `## 任务`/`## 验收` 标题规范为 canonical 的
  `## 执行步骤`/`## 验收标准`（沉淀引擎章节表识别所需，内容零改动）。
- 结论：三验收全 pass → **reviewed（通过）**。
