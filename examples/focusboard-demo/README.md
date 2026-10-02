# FocusBoard 演示仓（空仓 bootstrap 约定）

本目录是 FocusBoard 三轮产品计划的**需求与约定**来源；由演示驱动
（scripts/demo-focusboard.mjs，T-08）在每跑新建的空仓中物化为初始提交。
本文件与 requests/ 只携带需求与工程约定，**不携带任何解决方案**：无业务
.at、无预制 Specs、无解决方案测试、无已完成的计划。

## 工程约定（对三轮 Agent 生效）

- 仓库布局：`src/front/app.at` 为 VM 界面入口；数据文件放在 app 目录内
  （VM 运行期 cwd = src/front，app 相对路径实际落在
  `<app>/src/front/data/`，仍在 app 目录内、跨 Canvas stop/start 存活
  ——PLAN-097 T-01 实证；P001 的"本地保存"按此实现）。
- 每个产品计划走平台的 `/auto-plan:new → work → review → merge` 四技能
  闭环；工程 worktree 外置（演示驱动按 `.wt/focusboard-<run_id>-<NNN>/`
  约定创建），默认分支线性合回（ff-only）。
- 业务 Specs 为权威（`docs/specs/`），ledger 派生；Wiki 只记用法/设计
  取舍并链接规范与计划（`docs/wiki/*.md`）。
- 界面跑在 Musk 原生 VM Canvas 上（可被演示驱动截图/点选/断言）。
- 交付检查点由平台受控交付器核验；演示驱动只观察、发送需求与取证，
  不推进内部阶段、不写完成标记、不代修业务代码。

## 三轮需求

| 轮次 | 需求文件 | 摘要 |
|---|---|---|
| P001 | requests/001.md | 基础看板：CRUD、三列、状态切换、本地保存、VM 界面、用法 Wiki |
| P002 | requests/002.md | 限制并行：doing 上限 3、拒绝第 4 项、可见计数/解释、规则 Spec、测试、Wiki |
| P003 | requests/003.md | 可配置规则：新会话续接 Specs；上限 1~5 可设可存；降限保留任务并阻止新增；文案改善 |
