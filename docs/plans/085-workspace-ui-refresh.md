---
plan_id: "085"
status: execution_done
feature_name: "五栏目 UI/UX 整理与双端布局收敛"
author: Codex
created_at: "2026-09-22"
updated_at: "2026-09-23"
plan_revision: 3
current_step: 5
total_steps: 5
supersedes_spec_components: []
new_spec_components: [workspace-ui]
touched_goals: []
---

# PLAN-085 五栏目 UI/UX 整理

## 0. 变更摘要

最初基于 main 639cf4e 调研；执行中 PLAN-084 已以 dc15729 交付并归档。本计划整理会话、计划、规范、知识、文件的公共布局、信息层级和空状态，Auto 源为实现真源，不直接修改生成 Vue。085 的改动重放到 084 终态之上，保留其会话分组、归档、头部与 composer 交互。

## 1. 目标

让内容优先、操作可发现、状态可理解；保留五栏目及已有能力。紫色作为少量强调，侧栏去掉重卡片感，统一导航尺寸与阅读节奏。支持已有明暗模式。

## 2. 架构方案

继续使用 App + NavSidebar + ContentHeader + 五个视图，布局和状态文案均在 src/front/*.at。采用原生 row/col/button/text/icon，不增加组件依赖、渲染器或后端数据模型。共享 token 仍来自 pac.at，消除 Web 注入的重复主题覆盖。

## 3. 技术栈

AutoUI VM / Auto 生成 Vue；AutoDown 保持内容渲染边界；现有 ui-parity Gallery 与真机 MCP。auto-lang 只读依赖，按 AUTO_LANG_ROOT -> 同组兄弟 -> 主检出解析，无链接。

## 4. 需求分析与背景调查

用户于本任务授权分析并美化五个栏目，要求采用四技能及专用 worktree，允许本范围实现、验证和本地合回。无发布/推送要求。main 的 backend/Cargo.lock、.zcodeignore、target-smokecheck 及其他计划/设计草稿保持原样，不纳入提交。PLAN-084 已交付到 main dc15729，085 必须保留其终态。

截图与代码：会话标题过大且 shrink-0，可挤占工具区；NavSidebar 200/220/240px 和文件288px不一致；rail bg-secondary 对比过强；会话满屏横向分散，输入框960px；SpecsView 空 overview 永远称加载中，概览无文件入口；FilesView 缺少标题；Wiki/Plans 空态缺主要操作。

AutoUI 限制：VM 无 CSS cascade，纯类名通过共享 style 解析；动态条件保留既有安全写法；循环单根，事件入参 dot-path；消息70%最大宽度维持已有契约；Markdown 排版交给 AutoDown。本计划不承诺修复数学引擎或全部既有 VM 数据桥缺陷。

参考 https://learn.chatgpt.com/docs/features 的侧栏/内容面板分工；此处视觉选择为项目设计判断，不复刻产品。

## 5. 详细设计

- 导航：64px 图标轨，240px 二级栏，48px 顶栏；标题16px semibold，操作28px；文件对齐公共侧栏。
- 会话：列表减少描边，选中柔和背景；消息区域和输入区采用960px阅读容器、16px边距；用户气泡柔和底色，AI 去重复上下分割线；保留消息70%上限及展开工具/思考/审批。
- 计划：保留列表和编辑/状态操作；空态说明计划用途与新建入口；列表加载失败显式错误和重试；状态按钮使用本地化名称，底层状态值不变。
- 规范：概览可进入模块文件和索引视图，明确 docs/specs 权威、ledger 为索引历史；加载与空内容分开；文件模式具备标题/返回入口。
- 知识：空态提供新建页面并说明资料整理用途，搜索和树保持；文件：标准顶栏、路径标题、加载提示。
- 主题：品牌主色仍由 pac.at / AutoUI 共享；rail 使用低对比底色。调查实证表明当前 `theme.colors` 是跨明暗模式的单份局部覆盖，无法表达 Web 已有的双模式 surface token，因此保留 Web 明暗 surface 映射，待 AutoUI 支持分模式主题声明后再收敛。

| Delta | 类型 | 目标 | 变化 | 验收 |
|---|---|---|---|---|
| SD-01 | add | docs/specs/modules/workspace-ui.md | 记录五栏布局、信息层级、状态及双端约束 | AC-01–04 |
| SD-02 | modify | docs/specs/modules/ui-default-styles.md | 明确 pac.at 共享品牌色、Web 暂存分模式 surface token 的边界 | AC-01 |

## 6. 测试设计

运行 auto 生成/构建和 ui-parity check；对修改的生产组件运行 Gallery VM/Vue smoke，采集快照/截图实际检查空态、布局及操作；运行现有 live 回归按环境可用性留真实结果。失败保留 executing，不以静态通过代替运行验收。保留生成物作为本地证据，不手工修改产物。验证窄窗口约1024px及1280px；若 VM 功能受限，记录复现与剩余项，不宣称双端验收完成。

## 7. 验收标准

- AC-01：五栏公共导航/顶栏一致，长标题不遮挡操作；明暗主题来自同源；新增文本中英文齐全。
- AC-02：会话阅读区域/输入框对齐，消息与工具/思考操作保持可用；已有70%气泡约束保留。
- AC-03：计划/知识空态具备可用的新建入口；规范概览不永久假加载，概览可进入文件模式；文件显示标题和路径。
- AC-04：Auto 生成与相关检查无新增失败，双端运行证据可复查；产物均来自 Auto 真源。

## 8. 执行步骤

- [x] T-01 完成来源分析与详细方案，记录084边界。（AC-01–04）证据：五视图、共享组件、AutoUI 主题与 ui-parity 脚本已核查；084 交付后基线更新为 dc15729。
- [x] T-02 公共导航/标题/主题与会话视觉整理。（AC-01/02，依赖T-01）证据：`ee3df8b`、`6dc49b3`；统一 64/240/48px 层级、960px 会话阅读面与输入区，修正浅色用户气泡对比度。
- [x] T-03 计划/规范/知识/文件状态和布局整理。（AC-03，依赖T-01）证据：`ee3df8b`、`6dc49b3`；五栏目空态、主要入口、加载/失败状态与文件标题完成。
- [x] T-04 生成、双端运行与交互验证，处理实际发现。（AC-01–04，依赖T-02/03）证据：Auto 生成、Vue production build、ui-parity catalog 均通过；五栏目 1024/1280 明暗截图无水平溢出。VM live 在真实后端隔离工作区停于 `phase 1/2: VM arm` 超过 90 秒无首帧，人工终止；与 PLAN-084 已登记 VM container/启动基线缺陷同域，作为评审已知阻断，不表述为双端通过。
- [x] T-05 准备模块规范与持久证据，交接 review/merge。（SD-01/02，依赖T-04）证据：新增 `docs/specs/modules/workspace-ui.md`，更新默认样式边界与 spec 索引；评审、合回、归档由后续阶段负责。

## 9. 复审记录

stage: new | plan_id: 085 | plan_revision: 1 | outcome: pass | changed: AC-01–04,T-01–05,SD-01/02 | next: work

stage: work | plan_id: 085 | plan_revision: 1 | outcome: executing | code_base: 639cf4e（待重放 dc15729） | task_ids: T-01 | evidence: Auto/Vue/VM 源码、现有 Specs、用户截图和 ui-parity 运行链完成调查 | blockers: none | next: T-02/T-03

stage: new | plan_id: 085 | plan_revision: 2 | outcome: pass | changed: theme design,SD-02 | evidence: Web 实拍删除双模式 surface 覆盖后，暗色背景与前景 token 分属不同模式，规范侧栏文本对比失效；当前 pac.at theme.colors 无 light/dark 分表 | next: work

stage: new | plan_id: 085 | plan_revision: 3 | outcome: pass | changed: T-05 phase boundary | reason: review/merge 不能作为 work 阶段完成条件，改为准备规范与证据后交接 | next: work

stage: work | plan_id: 085 | plan_revision: 3 | outcome: execution_done | code_base: dc15729 | code_commit: 6dc49b397a3ed3bdd0548ce39d0374b936f52c32 | task_ids: T-02,T-03,T-04,T-05 | evidence: Auto gen PASS; Vue production build PASS; ui-parity 108 declarations/106 cases PASS; Web five views dark/light + 1024/1280 overflow PASS; VM live current-revision attempt stalled at VM arm >90s and was terminated | blockers: VM runtime evidence blocked by registered baseline engine/startup defect; no Web or source blocker | next: review

## 10. 待澄清事项

PLAN-084 已交付并归档；085 需在 rebase 后重新验证重叠的 app/chats/header/composer 文件。主检出已有其他未提交改动，合回前重新核查，不纳入本计划。
