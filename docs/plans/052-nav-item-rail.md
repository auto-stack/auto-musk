---
plan_id: MUSK-052
status: executing
feature_name: nav-item-rail
author: [zcode]
created_at: 2026-08-29
---

# [MUSK-052] view rail 换用 auto-lang Plan 482 nav-item 组件

## 变更摘要
auto-lang Plan 482 落地了 nav-item/nav-group 导航组件族（双端 class 契约 +
hover/active/disabled 三态 + icon/desc/badge 槽 + lucide svg 双端渲染，另修复
VM lucide 图标空渲染缺陷）。本计划把 app.at 的 view rail（会话/计划/规范/知识库
四项）从手搓 button+巨型条件样式串迁移到 nav-item，删除逐 text 补色
workaround（8453d6d），并把 viewstate_router.ts 的脆弱结构选择器换成
`.nav-item` 契约锚点。

## 任务
- [ ] T1 app.at rail 四项 → nav-item（icon: message-square/list-todo/scroll/
  book-open 字面量；onclick/active 同现有消息与状态）；确认 use.web 图标组件
  import 是否还有其他消费者，无则删。
- [ ] T2 composables/viewstate_router.ts 选择器 → `.app-rail .nav-item` 锚点。
- [ ] T3 验证：auto build --gen-only + vite build + web 冒烟（rail 四态点击）；
  VM 轨抽查截图（图标/选中/hover）。

## 验收
1. rail 渲染与迁移前等价（图标+文案+选中主色调高亮），样式代码量净减。
2. popstate 后退仍能正确切换 view（选择器更新后）。
3. VM 轨 rail 图标可见（lucide svg 修复随 auto-lang Plan 482）。

上游依赖：auto-lang master（Plan 482 折叠后 target/debug/auto 即含）。
