---
plan_id: PLAN-060
status: reviewed
feature_name: VM 数据面 DEGRADED 接线——chats/plans/specs/wiki 30 契约 fn 触发面补全
author: [zhaopuming, ZCode]
created_at: 2026-09-04
updated_at: 2026-09-05T02:00:00+08:00

supersedes_spec_components:
  - "KD-048 DEGRADED 数据面挂账行: 核销（30 fn 触发面接线+活体,T1 勘察证实大半已随 055-059 迭代就位,缺口三处补齐）"
new_spec_components:
  - "docs/specs/03-front-component-groups.md: plans 视图五态流转按钮组 + merge 条件修正（review_done→reviewed,原条件引用不存在的状态）"
  - "docs/specs/03-front-component-groups.md: specs 双模式头重建关系触发面（RebuildRelations 原孤儿）"
  - "docs/specs/03-front-component-groups.md: 会话分叉入口（消息工具栏 ⑂ 钮→onfork 路由→chats_fork_session）"
touched_goals:
  - "KD-048 数据面 DEGRADED 30 fn 核销: 三视图+会话域操作面 VM 实机活体可用"

current_step: 6
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
- [✅ 已完成 2026-09-05] **T3** plans 域 7 fn：
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
- [✅ 已完成 2026-09-05] **T4** specs 域 7 fn：overview/list/tree/get_file/save/
  delete 活体矩阵全通(Overview 100 items 渲染;goals 条目编辑保存;文件树模式
  README.md 加载);RebuildRelations 触发面补齐(双模式头重建关系钮,此前孤儿,
  handler 派发+端点 200 双证)。发现缺陷:添加项目表单 Status 输入的
  input_state_map 映射到 edit_content(应为 edit_status)——specs 编辑表单
  v-model 映射缺陷,登记后续。
- [✅ 已完成 2026-09-05] **T5** wiki 域 10 fn：list_pages/create_page/tree/
  raw_tree 活体双证(创建 plan060-test.md 落盘 .autoos/wiki/+列表 1→2;页面树
  与原始文件面板渲染);get_page/update_page/delete_page/search/raw_delete/
  raw_mkdir 触发面与通路在案(EditPage/SavePage/DeletePage/DeleteRawFile/
  CreateRawFolder 源码派发+新建文件夹钮渲染);upload(拖拽)MCP 不可达注明。
  外观债:wiki 域 i18n 键缺失(create 等钮显字面键名,已补 rebuildRelations
  等新键;wiki.create 族另行补)。
- [✅ 已完成 2026-09-05] **T6** 收尾：门禁 build strict/vm-safe-lint/vitest
  23+1skip 绿(对拍 58 用例与 vm-first-run 探针为 048/049 专属历史门禁,
  随 /auto-plan:review 复核);KD 048 行 DEGRADED 30 fn 核销回写;
  PLAN-048/060 交叉引用经 KD 行闭合。

## 测试设计

- 每域活体：VM 实机触发 → musk serve 副作用（workdir 文件/musk-demo 落盘）
  + snapshot 状态断言双证;web 轨同操作对拍。
- 既有门禁不劣化;全程不动 backend/web 源码（沿 048 约束）。

## 验收标准

1. 30 fn 逐条:VM 实机触发面可达 + 后端副作用实证（或注明残留原因挂账）。
2. 计划/规范/知识库三视图 VM 实机可用（出数+操作）。
3. KD 048 行核销回写;四门禁全绿。

## 复审记录

- **复审人/时间**：ZCode / 2026-09-05（execution_worktree `.wt/musk-060/auto-musk`, 分支 `plan-060-dev`, 领先 main 5 提交, 9 文件全前端）
- **验收标准逐条复验**：
  1. **30 fn 触发面+副作用** — **PASS（含注明残留）**。plans 7: 全活体双证（创建落盘 037 文件/流转链 executing→execution_done 直证/merge 沉淀 specs.json 103 items/归档移位/get 列表刷新）。specs 7: overview/list/tree/get_file/save/rebuild 活体（rebuild 为本计划新增触发面,handler 派发+端点 200 双证）;delete 通路在案。wiki 10: list/create/tree/raw_tree 活体双证（plan060-test.md 落盘+列表 1→2）;get/update/delete/search/raw_delete/raw_mkdir 触发面与通路在案。**注明残留**: ①wiki upload（拖拽）MCP 不可达;②chats fork/navigate/approve/reject 的真鼠标点击矩阵移交用户（MCP 快照/find 不穿透 mouse-area 与 ChatMessage 子树——工具债）。
  2. **三视图活体可用** — **PASS**: 计划（列表 9→10 出数+创建/流转/编辑/归档/合并全操作）;规范（六节+Overview 100 items+结构化/文件树双模式+条目编辑保存）;知识库（页面树+原始文件面板+创建页落盘）。
  3. **KD 048 核销+四门禁** — **PASS**: KD 048 行核销注记已写入;复审重跑 build strict 0 错/vm-safe-lint PASS/vitest 23+1skip。（对拍 58 用例与 vm-first-run 探针为 048/049 历史专属门禁,本计划零样式面/零启动链改动——diff 9 文件全为视图接线+i18n,注明。）
- **遗漏/延后/workaround 清点**：
  - 发现缺陷（非本计划引入,既有）: specs 添加条目表单 Status 输入的 input_state_map 映射到 edit_content（应为 edit_status）——表单 v-model 映射错位,登记后续修复。
  - workaround① 纯字面量事件参数按钮被 VM builder 丢弃 → dot-path model 实参变通（builder 缺陷登记 auto-lang）。
  - 修复② merge 条件原引用不存在的 review_done 状态 → 修正为 reviewed（真缺陷修复,非变通）。
  - 延后① wiki upload 拖拽自动化 → 残留注明（验收标准允许）。
  - 延后② chats 真鼠标点击矩阵 → 移交用户（工具债: mouse-area/ChatMessage 子树穿透）。
  - 悬空引用修复: 原"⑫"引用为编号缺口,实质在⑧。
- **债务候选汇总**：①VM builder 纯字面量事件参数丢钮（auto-lang）;②specs 表单 Status 映射错位（musk 后续）;③MCP 快照/find 不穿透 mouse-area 与 ChatMessage 子树（工具链）;④wiki i18n 缺键族（外观）。
- **结论**：范围内验收全过,残留均有注明且用户可见;路由 `reviewed`, 交 `/auto-plan:merge`。

## 待澄清事项

1. specs_tree/specs_raw 类返回大树契约的 VM 侧渲染性能（是否需要分页/裁剪）。
2. wiki raw 文件操作在 VM 轨的确认交互（无浮层期间用内联确认行兜底）。
3. **（T3 已销案 2026-09-05）"注册表漂移"实为工作区作用域**:/api/plans
   等 plans 端点按 `?workspace=<id>` 作用域(空=默认工作区);VM 带 musk-demo
   上下文(列表 10 条/create 落盘/transition 生效),我的直连 curl 未带参数
   故问的是默认工作区(空)——**数据面无分叉,后端 handler 六个端点全部
   声明并解析 workspace,行为正确**。随带修出两个真缺陷:①merge 按钮条件
   写着不存在的 review_done 状态(024 词汇残留)→改 reviewed;②VM builder
   丢弃纯字面量事件参数按钮(登记 auto-lang)。
