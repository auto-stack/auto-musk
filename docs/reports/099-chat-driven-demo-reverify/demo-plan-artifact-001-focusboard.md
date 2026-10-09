---
plan_id: PLAN-001
status: executing
feature_name: FocusBoard 极简专注看板
author: [agent]
created_at: 2026-10-09T16:23:07
updated_at: 2026-10-09T16:26:59
plan_revision: 1
current_step: 2
total_steps: 4
supersedes_spec_components: []
new_spec_components: [docs/specs/focusboard.md]
touched_goals: []
---

# 0. 变更摘要

新增独立单页应用 `apps/focusboard/index.html`(零依赖、无构建):三列极简专注看板(待办/进行中/已完成)、卡片点击推进、顶部会话内计时器;新增模块规范 `docs/specs/focusboard.md` 与运行说明。

# 1. 目标

- 单页呈现三列看板,列名依次为:待办 / 进行中 / 已完成。
- 卡片唯一交互为点击:点击将卡片向右推进一列,进入"已完成"即标记完成并呈完成态。
- 顶部会话内计时器:页面加载即从 00:00 正向计时,仅存内存,刷新归零(会话语义)。
- 以自包含 HTML 交付,可在宿主 Canvas 中直接渲染。

非目标:数据持久化、多用户协作、卡片增删改、拖拽排序、深色主题(requests/002.md 另立计划,本计划仅以 CSS 变量预留主题位)。

# 2. 架构方案

单文件自包含页面:`index.html` 内联 CSS/JS。状态为内存数组,渲染函数按列全量重绘;无后端、无依赖、无构建步骤,便于 Canvas 直接加载与后续深色主题扩展。

# 3. 技术栈

- HTML5 + CSS3 + 原生 JavaScript(ES2017+)。
- 无框架、无包管理、无网络请求。

# 4. 需求分析与背景调查

- 需求来源:`requests/001.md`(极简专注看板:单页三列(待办/进行中/已完成),卡片可点击标记完成,顶部会话内计时器)与本计划范围一致。
- 授权范围:用户授权按 `AGENTS.md` 计划闭环完成 计划生成 → 执行 → 复审 → 文档,并把界面跑在 Canvas;未设预算/步数上限。
- 背景调查:
  - 仓库为空白 bootstrap 演示仓(`README.md`:"PLAN-099 e2e repo");无 `docs/specs/`、无测试框架、无包管理。
  - `apps/gallery` 为组件级 parity harness(Vue/VM 渲染器),与单页看板不匹配,不复用其构建链。
  - 现有计划:`docs/plans/` 与 `docs/plans/archived/` 均为空,编号取 001。
  - `requests/002.md`(深色主题切换)为后续需求,不纳入本计划。
- 未知约束:无。

# 5. 详细设计

- 布局:顶栏(标题 FocusBoard + 会话计时器 mm:ss)+ 三等宽列。每列头部为列名与计数徽标;卡片为圆角块,含标题。
- 交互状态机:待办 --点击--> 进行中 --点击--> 已完成(终态,再点击无操作)。
- 完成态样式:降低不透明度 + 标题删除线 + "✓" 前缀。
- 计时器:以 `Date.now()` 起点差值每秒重算 mm:ss(≥1 小时显示 h:mm:ss),页面隐��不丢秒。
- 主题:颜色全部经 `:root` CSS 变量,为 requests/002 深色主题预留。
- 数据:内存数组 `cards = [{id, title, col: 'todo'|'doing'|'done'}]`,种子数据 6 条(3/2/1 分布),刷新复位。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/focusboard.md | 无 → 新增 FocusBoard 模块规范:三列布局/点击推进/完成态/会话计时器行为与约束 | 仓库无既有 specs,需固化验收行为 | AC-01, AC-02, AC-03, AC-04 |

# 6. 测试设计

仓库无测试框架,采用检查单式验收:每条 AC 含操作步骤与期望结果,由复审阶段独立复核;另附浏览器控制台冒烟断言(可选)。

# 7. 验收标准

- AC-01: 打开 `apps/focusboard/index.html`,单页呈现三列,列名依次为 待办/进行中/已完成。验证:Canvas 或浏览器打开页面并观察,列数=3 且顺序正确。
- AC-02: 点击"待办"列任一卡片,该卡片移动到"进行中"列并从原列消失,两列计数徽标同步。验证:操作后观察两列。
- AC-03: 点击"进行中"列任一卡片,该卡片进入"已完成"列并呈完成态(删除线 + ✓);再次点击无变化。验证:操作后观察,并再点一次确认终态。
- AC-04: 顶部计时器自加载起从 00:00 正向走秒,格式 mm:ss;刷新页面后归零重计。验证:观察 ≥2 秒,刷新对比读数。

# 8. 执行步骤

- [x] T-01 新建 `apps/focusboard/index.html`:三列布局、种子卡片、点击推进与完成态、会话计时器(CSS 变量主题)。依赖:无。验证:Canvas/浏览器打开,AC-01..AC-04 全部通过。关联:AC-01, AC-02, AC-03, AC-04。[新路径] [✅ 已完成] 证据:文件已写入(9.0KB);node --check 内联脚本语法 PASS;结构断言 PASS(列序 待办→进行中→已完成、NEXT 状态机、role=timer 起点 00:00、6 条种子卡片)。浏览器行为验收转复审阶段复核(本机无 headless 浏览器)。
- [x] T-02 计划状态推进:frontmatter `status: executing`,`current_step` 随任务更新,`9. 复审记录` 留痕。验证:读取计划文件核对字段。关联:AC-01, AC-02, AC-03, AC-04。[✅ 已完成] 证据:status=executing, current_step=2/4,本条及 §9 记录为证。
- T-03 复审:独立复核 AC-01..AC-04 与代码质量(语义化标签、无控制台报错、无内存泄漏句柄),结论写入 `9. 复审记录`。依赖:T-01。验证:复审记录含 pass/changes_requested 结论。关联:AC-01, AC-02, AC-03, AC-04。
- T-04 文档:新增 `docs/specs/focusboard.md`(落实 SD-01)、`apps/focusboard/README.md` 运行说明,归档计划至 `docs/plans/archived/001-focusboard.md`。依赖:T-03。验证:文件存在且与实现一致。关联:AC-01, AC-02, AC-03, AC-04。[新路径]

# 9. 复审记录

- 2026-10-09 /auto-plan:new: stage=new, plan=PLAN-001 rev1, outcome=pass, next=work。任务覆盖全部 AC 与 SD-01;唯一设计裁量:卡片点击语义取"逐列推进,进入已完成即标记完成",已在第 10 节备案。
- 2026-10-09 /auto-plan:work: stage=work, plan_id=PLAN-001, plan_revision=1, outcome=pass, code_commit=N/A(非 git 仓库), task_ids=T-01, T-02, evidence=node --check 内联脚本 PASS + 结构断言 PASS(apps/focusboard/index.html);blockers=无(浏览器侧 AC 观察留待复审);next=review(T-03)。

# 10. 待澄清事项

- 卡片点击语义:需求原文"可点击标记完成";本计划取"逐列推进"解释,使"进行中"列可达且保持单一交互。若期望"任一列点击直达已完成",为单行改动,待用户指示即可调整(不阻塞执行)。
