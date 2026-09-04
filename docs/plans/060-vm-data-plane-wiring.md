---
plan_id: PLAN-060
status: executing
feature_name: VM 数据面 DEGRADED 接线——chats/plans/specs/wiki 30 契约 fn 触发面补全
author: [zhaopuming, ZCode]
created_at: 2026-09-04
updated_at: 2026-09-05T01:30:00+08:00

supersedes_spec_components: []
new_spec_components: []
touched_goals: []

current_step: 1
total_steps: 6
---

# [PLAN-060] VM 数据面 DEGRADED 接线

## 变更摘要

PLAN-048 定型 api_over_http 通路时挂账的 **DEGRADED 数据面 30 fn**（代码通路
已通、触发面/接线未通）：VM 轨的 计划/规范/知识库 三个视图基本为空壳,会话域
缺删除/分叉/导航等操作面。本计划逐域接线并活体验证,核销 KD 048 行挂账。

## 范围（048 KD DEGRADED 清单,共 30 fn）

- **chats 域（6）**：chats_approve / chats_reject_all / chats_fork_session /
  chats_navigate_session / chats_delete_session / chats_delete_all_sessions
  （forge+chats_view 域）
- **plans 域（7）**：plans_list / plans_get / plans_create / plans_update /
  plans_transition / plans_archive / plans_merge
- **specs 域（7）**：specs_list / specs_overview / specs_save_item /
  specs_delete_item / specs_rebuild_relations / specs_tree / specs_get_file
- **wiki 域（10）**：wiki_list_pages / wiki_get_page / wiki_create_page /
  wiki_update_page / wiki_delete_page / wiki_search / wiki_tree /
  wiki_raw_tree / wiki_raw_delete_file / wiki_raw_mkdir

（MVP 7 fn 中 chats_create_session/chats_get_session/chats_send_message/
auth_register/auth_me 通路已通;auth_login/chats_list_sessions 已活体验证——
不在本计划范围。）

## 目标

1. VM 轨 计划/规范/知识库 三视图活体可用（列表出数、增删改触发后端落盘、
   MCP snapshot + musk serve 副作用双证）。
2. 会话域 6 操作在 VM 实机可达（删除两步确认/分叉/导航/审批门）。
3. KD 048 行 DEGRADED 清单核销（30/30 或逐条注明残留原因）。

## 前置与风险

- **不依赖 auto-lang PLAN-533**（浮层通道）——本计划全部为数据面接线,悬浮
  组件缺位不影响列表/表单类触发面。
- 已知风险：KD 059-FU1（timer 驱动状态写入不重渲染）——本计划接线以
  handler 驱动为主（已证可重渲染）,避免 timer 依赖;若遇必须 timer 的场景
  挂账等 auto-lang 侧根修。
- 跨 widget 限制：子件 model 共享根态（055 族）——涉状态的子组件沿
  think_open/tool_open 键列表上提先例,不做子件本地可变态。

## 执行步骤

- [✅ 已完成 2026-09-05] **T1** 勘察（源码盘点+worktree VM 活体勘察,MCP :9278,
  登录 review060 后逐视图 snapshot/截图）。**结论:048/060 立项时的"空壳"清单
  大半已过时**——四域 store 层 30 fn 调用全在,三视图在当前二进制活体出数:
  计划列表 9 条(musk-demo)/规范六节+Overview 100 items/知识库页面树+原始
  文件面板。**真实缺口收敛为**:
  ① plans_transition:视图 TransitionTo 孤儿(无状态流转按钮);
  ② specs_rebuild_relations + specs_list(LoadDocument handler):无触发点;
  ③ wiki:源码全接线+页面树/raw 面板活体渲染,逐触发器活体矩阵待跑(T5);
  ④ chats fork:navigate 走分叉切换器(已有)但**创建分叉无触发面**
     (chats_fork_session 无 UI 入口,切换器只切既有分支);
  ⑤ chats approve/reject:审批门触发在案(GateCard onapprove/onreject),
     活体依赖门数据。
  T2-T5 由此收敛为"补缺口触发面 + 30 fn 活体矩阵双证"。
- [🚧 代码完成 2026-09-05] **T2** chats 域 6 fn：删除两步确认已交付（059,
  alert-dialog 单源）;分叉入口新增（ChatMessage 工具栏 ⑂ 钮 → onfork 路由 →
  ForkFrom → store.BranchTo(mid,"fork") → chats_fork_session,c13c250 后续
  提交）;navigate（分叉切换器）/approve/reject（GateCard）既有接线确认。
  **活体点击矩阵移交用户**——MCP 快照/find 不穿透 mouse-area 与
  ChatMessage 子树（工具债,登记 KD）:点 ⑂ → 列表出新分叉会话 → 切换器
  navigate → 审批门（需门数据）。
- [🚧 受阻 2026-09-05] **T3** plans 域 7 fn：
  **已完成部分**:plans_transition 触发面补齐(五态按钮组 drafting/executing/
  execution_done/reviewed/review_done;TransitionTo 此前孤儿)——纯字面量事件
  参数的按钮被 VM builder 丢弃(gen vue 正常),dot-path model 实参变通后渲染
  (builder 缺陷登记 auto-lang)。活体:plans_list ✓(列表 9→10 出数)、
  plans_create ✓(双证:状态 creating=false+plans 10 条刷新;磁盘文件
  musk-demo/docs/plans/037-plan060-review-test.md 落盘)、plans_get ✓
  (SelectPlan→current)、plans_transition ✓(executing 迁移落盘)。
  **受阻**:transition→review_done 后续(merge 按钮条件=review_done)被数据面
  归属问题挡住——见待澄清③。plans_update/merge/archive 的活体矩阵随之待
  数据面归属裁定后补跑(触发面本身已全部在案)。
- [ ] **T4** specs 域 7 fn：SpecsView 树/总览/编辑/删除/重建关系 + 活体。
- [ ] **T5** wiki 域 10 fn：WikiView 列表/页/编辑/删除/搜索/raw 目录操作 + 活体。
- [ ] **T6** 收尾：musk 四门禁（build strict/vitest/对拍/探针）+ KD 048 核销
  回写 + PLAN-048/060 交叉引用闭合。

## 测试设计

- 每域活体：VM 实机触发 → musk serve 副作用（workdir 文件/musk-demo 落盘）
  + snapshot 状态断言双证;web 轨同操作对拍。
- 既有门禁不劣化;全程不动 backend/web 源码（沿 048 约束）。

## 验收标准

1. 30 fn 逐条:VM 实机触发面可达 + 后端副作用实证（或注明残留原因挂账）。
2. 计划/规范/知识库三视图 VM 实机可用（出数+操作）。
3. KD 048 行核销回写;四门禁全绿。

## 待澄清事项

1. specs_tree/specs_raw 类返回大树契约的 VM 侧渲染性能（是否需要分页/裁剪）。
2. wiki raw 文件操作在 VM 轨的确认交互（无浮层期间用内联确认行兜底）。
3. **（T3 受阻 2026-09-05）plans 数据面归属与注册表漂移**:VM(merged 模式)
   的 plans_create 把文件写到 **serve 数据目录**
   （main/tmp/musk-demo/docs/plans/037-plan060-review-test.md,worktree 内
   无副本）,但 **:9247 serve 的 /api/plans 注册表返回 []**、
   transition/archive 对新计划报 "plan 037 not found"——同目录文件存在而
   注册表不认。三问:①api_over_http 通路在 merged 模式下 plans 读写到底
   落 in-process 还是 :9247（两者注册表明显是两套）;②serve 的
   PlansStore 注册表生命周期(疑似启动时扫描后不 rescans,与磁盘漂移);
   ③review060 登录态下 VM 列表见过 10 条而 HTTP 侧恒 []——用户作用域还是
   注册表分叉。裁定前 T3 的 transition/update/merge/archive 活体矩阵冻结
   (触发面 UI 已全部在案);裁定后 30 分钟内可补完全矩阵。
