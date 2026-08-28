# T7 三域视觉验收记录（2026-08-28）

- 登录页/会话壳：截图 screenshots/t5-login-dark.png、t6-chats-view-dark.png（暗色实拍）。
- plans/specs/wiki：IAB 截图管线在本时段降级（stale 帧 → "capture failed for guest"），
  改用 DOM 快照实证 + parity/构建门禁；工作区同时有并行会话在写（共享后端），
  截图含他人活动流，不作验收依据。
- plans：URL /plans 直达 —— rail tab 计划 active + plan 列表（PLAN-018..035）+
  空态「从侧栏选择一个计划」渲染正常（DOM 快照）。
- specs：URL /specs 直达 —— 二级导航 概览/goals/architecture/designs/tests/
  reviews/reports + Overview 正文渲染正常（DOM 快照）。
- wiki：URL /wiki 直达 —— nav 图标钮 + 搜索框 + 原始文件树 + drop-zone
  （"Drag files to upload"）+ 暂无页面空态 + 页面树渲染正常（DOM 快照）。
- 待用户目验清单：三视图亮/暗色各一遍（复审阶段勾选）。
- 环境注记：验收用临时账号 zcode-style-check（注册制,共享 demo 后端）,未改动
  任何工作区数据；用户原登录态在 IAB localStorage 被清除,需重新登录。
