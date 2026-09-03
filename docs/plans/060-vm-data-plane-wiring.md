---
plan_id: PLAN-060
status: drafting
feature_name: VM 数据面 DEGRADED 接线——chats/plans/specs/wiki 30 契约 fn 触发面补全
author: [zhaopuming, ZCode]
created_at: 2026-09-04
updated_at: 2026-09-04T00:10:00+08:00

supersedes_spec_components: []
new_spec_components: []
touched_goals: []

current_step: 0
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

- [ ] **T1** 勘察：三视图（PlansView/SpecsView/WikiView）+ 会话操作的现有
  store/视图消费点、缺哪层接线（Init 面/触发 handler/结果回填）,产出接线清单。
- [ ] **T2** chats 域 6 fn：删除两步确认已有内联行（PLAN-058）,补 fork/navigate/
  approve/reject 触发面 + 活体验证。
- [ ] **T3** plans 域 7 fn：PlansView 列表/创建/状态机/归档/合并接线 + 活体。
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
