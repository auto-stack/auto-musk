---
plan_id: PLAN-097
status: executing
feature_name: VM 计划开发演示与全流程证据
author: [agent]
created_at: 2026-10-02T01:40:31Z
updated_at: 2026-10-02T09:22:00Z
plan_revision: 1
current_step: 1
total_steps: 14
supersedes_spec_components:
  - docs/specs/modules/app-studio-ui.md
  - docs/specs/modules/workspace-ui.md
  - docs/specs/modules/ui-parity.md
new_spec_components:
  - docs/specs/modules/demo-evidence.md
touched_goals:
  - goal-agent
  - goal-relay
  - goal-spec-knowledge
  - goal-frontend-parity
  - goal-security
---

# PLAN-097 — VM 计划开发演示与全流程证据（计划 B）

## 0. 变更摘要

承接 PLAN-096（计划 A）的四阶段控制合同，在真实 auto-musk VM 界面中展示
计划推进、修复、阻塞与交付检查点；补齐当前 VM 隐藏的 Canvas 进度摘要；
提供可重复的 FocusBoard 演示驱动、真实截图和可验证证据索引。

FocusBoard 从仅含工程约定和需求的空仓开始，由 auto-musk 的阶段 Agent 用
三个工程计划完成。每个计划均走 auto-plan:new/work/review/merge 对应产品
闭环，工程代码、测试、Specs、Wiki 内容均由该闭环产出。演示驱动负责启动、
发送需求、操作验证及取证，不负责实现工程或替 Agent 修复。第三轮使用新会话，
证明已有 Specs 和交付工件可以接续开发。

本计划是演示能力与真实验收，不是预制 FocusBoard 的实现。A 控制推进与交付，
B 只消费事实并记录现场；成功必须同时具备工程结果、四阶段交付收据和 VM 截图。

## 1. 目标

### 可交付结果

1. 原生 VM 可见当前计划、阶段、尝试次数、修复次数、阻塞和交付检查点；
   断连、旧版本、未知状态均明确呈现，不把普通 completed 当成 delivered。
2. Canvas 生成/启动/验证进度在 VM 与 Vue 同义，不再受 G-15 的 computed
   web helper 返空影响；显示事实与实际工具/Canvas 状态一致。
3. 一条可重复命令启动隔离环境，依次给 Agent 三个需求，自动等真实交付并
   验证行为，自动保存 Musk 宿主及目标 Canvas 的 PNG、状态、收据和报告。
4. 截图覆盖需求、各计划四阶段、Specs、Wiki、Canvas 点选/源码联动与最终
   界面。读者可以追溯截图所属计划、版本、提交、会话和实际运行来源。

### 范围、依赖与执行位置

| 项目 | 本计划责任 | 位置/约束 |
|---|---|---|
| auto-musk | 前端事实消费、VM 进度、演示脚本、验证和证据 | D:/autostack/.wt/musk-097/auto-musk；plan-097-dev |
| PLAN-096 | 提供 plan_execution、阶段事实、独立评审与交付收据 | 本计划只读；正式接线/真实演示依赖 A 完成 review/merge、定版消费合同 |
| auto-lang/auto-ai/auto-down | 已有编译、VM、模型与 Markdown 能力 | 只读依赖；不得顺带修改，缺口登记后另行规划 |
| FocusBoard | 由 auto-musk Agent 在三个独立产品计划中实现 | 每跑新建 D:/autostack/.demo/musk-097/<run_id>/focusboard；开发 worktree 另置 .wt |

计划文档/进度写主检出 docs/plans/；代码、测试和规范增量准备在专用 worktree。
T-01 可与 A 并行：只探测已有运行、截图和持久化能力。T-02 冻结 A 的已交付
合同后才开始 T-03 及其后接线。A 的 T-01/T-02 完成不等于其控制闭环已交付。
全部 worktree 内禁止 junction/symlink；不得把示例 Git 仓嵌套在平台 worktree 中。

### 非目标

- 不重复实现 A 的授权、重试、评审、合并控制；不引入通用持久化 Runner。
- 不扩展 AutoUI 原语、动态布局引擎、多 Canvas、拖拽设计器或新主导航。
- 不修 G-17 的全部 studio bounds 或动态 :style；沿用已交付布局回退，
  以真实像素/有效节点行为核验，本计划的可见进度和关键操作仍必须通过。
- 不做远程发布/部署，不覆盖共享 Musk/AAID 服务，不修改用户模型配置。
- 不提供预制业务代码、伪造阶段截图、用浏览器截图代替 VM 或把 fixture
  用例当作真实三计划演示。Vue 保留受影响回归；正式展示采用 VM。

## 2. 架构方案

```text
PLAN-096 已交付事实/收据（控制单写者）
  → 现有 Relay 查询 + 事件提示刷新
  → 前端 store：身份校验、单飞轮询、纯投影、明确错误/陈旧
  → VM/Vue 共源 PlanProgress + CanvasProgress

隔离演示驱动（观察与验证）
  → 空工程 + 三份需求（不携带解决方案）
  → Musk 正式 plan 入口 → A 的阶段 Agent 与交付器
  → Musk 宿主 AutoUI MCP 截图 + Canvas 目标截图/交互
  → 工程 Git / Plan / Spec / Wiki / 交付收据核验
  → evidence manifest → 可浏览图文报告 + 机器验收结果
```

UI 与演示驱动不能推进内部阶段、写完成标记或制造评审结果。截图失败不改变
计划执行结果，但演示验收失败；工程未 delivered 不得继续发下一个产品需求。

## 3. 技术栈

- 前端真源 src/front/*.at，共用 store/纯参数 helper；现有 HTTP 异步请求
  通道和轮询计时消息。VM 不依赖 SSE、浏览器 DOM 或 use.web.fn 的计算结果。
- Node.js 内置模块实现演示驱动、JSON-RPC MCP、SHA256、测试与静态报告，
  复用已核实的协议，不依赖临时/未跟踪脚本。
- 现有 RustHTTP + 原生 VM 为正式演示组合；VMHTTP 作为受影响接线回归。
- Git、四个 auto-plan 技能、Specs 派生账本与现有 workspace Wiki API。
- AutoUI autoui_snapshot/action/state/screenshot 采集宿主，Canvas API/
  canvas_* 采集目标；两种来源在索引中独立标识。

## 4. 需求分析与背景调查

### 授权及版本边界

用户要求 VM 示例、两三个工程计划、全流程 Agent 自动完成和截图；本轮明确
“继续规划任务 B”。本轮授权为编写此执行合同，不启动实施 Agent、不开始演示，
不替用户授权远程副作用。后续 work 使用已有范围授权及 A 的实际启动授权合同。
用户未指定费用或时间预算；不虚构额度。演示不自行提高 A 的有界修复上限。

规划时主检出干净，auto-musk main 为
3a9b2e5d273e55c618ac3cd38770650f792ca0a3；PLAN-096 revision 1，executing，
2/14。其专用树已提交 7f4bdbb 的消费 DTO 初稿，仅作为待冻结接口线索。
auto-lang=986e765ac0c1193409c26cbf16c3bb0d63cb6073；
auto-ai=5a50a55844d7aa3523b593f21ba0fb03d18eac48；
auto-down=895f8d0f9355c9f5ec3ce8fca268bdb768395846。
work 必须重拍实际基线和二进制 hash，不能将规划版本视为最终部署版本。

### 权威 Spec 输入（SHA256）

| 路径 | 规划读取 hash |
|---|---|
| docs/specs/00-overview.md | FCF8EB5793E69E5AC9585668598E71257CFFCE091CB9063753932BEDB17266B9 |
| docs/specs/index.json | F5384BF88B35EB55D701ABF14DAFBF45C89281398F454963A5768C57241B77A3 |
| docs/specs/modules/plan-flow.md | 2A904C697EB33A098A9CB4D094D9D06BA3F261A4F27E5639F6CD85FA5792A7C2 |
| docs/specs/modules/app-canvas.md | A2E5BD4048363783212CE16BF2BD9EE679D7E9D532B6A8E65D52707D98DEF47D |
| docs/specs/modules/app-studio-ui.md | 8422A0DC14398EFF16385FADC7569334FDF82B4394295AD835803189946D890C |
| docs/specs/modules/workspace-ui.md | DB74E023FA4B9896C7DBD59FF2F49B54A1BA0E5F1AA08449D994A4B32AC44C38 |
| docs/specs/modules/ui-parity.md | 5194497B7E4F57F534226FB238C306DDBAABE81A955CF6BF8A2668F838BA434E |
| docs/specs/modules/vm-process-stability.md | 93EDE5E330DFC923C78A4AA718E8B2F76D82AD9FFAB280D728667F1F51C08E2F |

overview 旧阶段名称不作为 A 控制状态的枚举来源；A 定版后读取新增
plan-execution-contract.md 并记录 hash。specs-ledger.md 未列入当前 index，
不在 B 修复这项 A 所属知识沉淀问题。

### 已核实实现与缺口

- src/front/canvas_progress.at 经 canvas_helpers.at 的 use.web.fn 计算 rows，
  现行 Spec 登记 VM 返回空（G-15）。ForgeStore 的消息、工具状态和 Canvas
  事实已有来源，优先在 handler 内纯投影后写入可渲染字段。
- relay_store.at 有 LoadRun/OnRelayEvent 和历史查询；Subscribe 使用 SSE。
  ForgeStore 已有 PollStream/异步 ticket 模式。VM 活跃计划需非阻塞单飞
  轮询，不直接照搬同步 LoadRun 高频调用或依赖 SSE。
- A 的 relay/plan_contract.rs 初稿含 contract_version、plan_id/path/revision、
  phase、attempt、repair_count/limit、blocker、reviewed_commit、
  delivery_checkpoints、receipt_ref 和 stage_results；事件 RunPlanEvent
  可提示刷新。实际 HTTP/AutoVM 入口与字段以 A T-10/T-13 定版为准。
- scripts/ui-parity/live.mjs 有真实宿主 MCP、端点发现和截图协议；
  canvas-studio-live.mjs 还依赖 tmp/vm-studio-cycle.mjs，存在旧 093 路径/
  按进程名清理。不能把这条临时依赖直接作为可重复演示入口。
- app-canvas.md 明确：canvas_snapshot 是目标帧；宿主截图另采。
  AUTO_VM_STORAGE_FILE 每 Canvas 会话一次性创建、停止时删除。
  015-notes/db.at 的“JSON persistence”注释不足以证明跨重启落盘实现。
- WikiStore 的实际归属由 workspace.rs 决定，现有 ws_wiki_* 绑定可写
  工作区知识库；Wiki 页面与规范文件不是同一存储，不应直接写 wiki 内部 JSON。
- 旧 launch-vm.cmd 复用固定端口与在用后端，不能证明新源码已部署；本计划
  必须用自有进程/私有端口，记录实际监听端点和工具链/源/二进制指纹。

## 5. 详细设计

### 5.1 消费 A 的事实

T-02 将定版路径、DTO、版本、事件、成功/失败样例固化到
docs/reports/097-vm-demo-baseline.md（新增）。若合同与初稿不同，在本
范围内调整 adapter；缺必要事实返回 A 处理，不靠解析 Agent 叙述补字段。

以 workspace_id/session_id/run_id/plan_id/revision 及请求代次绑定快照。
同一活跃 run 默认 1Hz 单飞异步轮询，事件只提前请求刷新；切换/退订/终态
停止该活跃轮询，迟到响应不得污染新工作区。一次断连保留最后成功快照但标记
陈旧，重试恢复；未知 contract_version 明确不兼容，不能默认为成功。

拟新增 plan_progress_store.at、plan_progress_helpers.at、plan_progress.at；
最终是否并入 RelayStore 在 T-02 决定，避免重复 owner。纯 helper 不访问 store
字段，handler 负责读取事实与赋值。phase/outcome 均来自事实，不从自然语言、
进度绿勾或 PNG 推断。plan/execute/review/document 的用户文案为规划/实施/
复审/沉淀；delivered 仅由 A 已核实交付事实判定，checkpoint 缺件必须可见。

在现有会话/relay 卡增加紧凑摘要与可展开阶段记录，支持选择历史阶段；历史
卡显式标明“已完成阶段记录”，不伪装正在运行。显示 plan、revision、attempt、
修复计数、blocker、reviewed_commit 摘要和 receipt 入口；中英资源齐备。
不另建主栏目，不打断 composer 草稿、审批和 Canvas 状态。

### 5.2 Canvas 进度与 VM 行为

复用 canvas_helpers.at 的事实归类，迁移必要计算到 handler→store 字段。
保留“工具运行/完成/失败”“lint 建议”“未验证”差别，Canvas 帧 seq 更新
只能说最近更新，不能据此声称 hot reload 因果关系。与计划进度分开呈现。
真 VM 上必须有实际生成、启动、运行及验证结果，不能仅以 Gallery fixture 验收。

宿主 action 前重新 snapshot 取当前字符串 vnode id。G-17 零 rect 不用于
计算点击坐标：优先有效节点动作/结构树，再用目标真实帧上的已核实 bounds；
无可用路径明确失败。Canvas 选择、层树、源码抽屉需真实联动；多文件锚点
若启发式不确定，记录实际置信度，不伪称精确来源。

### 5.3 隔离启动与有限 preflight

新增 scripts/demo-focusboard.mjs，命令接口 preflight/run/verify/report；
实现模块 scripts/demo/{contract,vm-session,capture,evidence,focusboard}.mjs
（均新增）。preflight 不启动模型任务，输出明确能力/缺件报告。

T-01 只完成四项探针：宿主截图/动作可达、端点和进程所有权、真实 VM
文件持久化路线、合法构建路线。使用临时小探针，不预制 FocusBoard 业务。
每项保留命令/结果/已选接口；无可行路线就定位阻断，不无限修上游。
T-02 另核验 A 定版的查询与正式 plan 入口，不提前绕过控制器。

每跑唯一 run_id/config/workspace/证据目录，自有 Musk 后端 + 原生 VM，
AUTO_EXE 显式固定；读取启动双流发现实际 MCP 端点，处理端口回退。
复用共享 AAID 只允许只读连接，不修改配置、重启或按名字结束它。
关停只针对本跑 spawn 的进程树，核对 PID/启动身份及所属根；取消保留现场。
遗留进程、端口占用、找不到新二进制均为显式错误，不能换用旧服务继续报成功。

Frontend 生成可在专用树内进行；任何会创建链接的依赖安装放到外部 build
staging（D:/autostack/.demo-build/musk-097/<run_id>），复制生成源和所需
真实文件并核对 hash，不在 worktree 放 node_modules 链接。T-01 固定 staging
命令与相对依赖布局；不把违反 AGENTS 的 pnpm 默认安装当作既成许可。

### 5.4 FocusBoard 三轮需求

Bootstrap 仅创建/提交 README、AGENTS、需求文档与必要空目录；无业务 .at、
预制 Specs、解决方案测试或已完成 Plan。使用平台正式 workspace 注册和 plan
启动入口；AGENTS 约定每产品计划的专用外置 worktree、线性合回及知识职责。
工程 worktree 采用 .wt/focusboard-<run_id>-<NNN>/focusboard，目录无链接；
数据独立于代码树/开发 worktree，路径与访问方式由 T-01 实证后冻结。

| 工程计划 | 给 Agent 的增量需求 | 演示验收 |
|---|---|---|
| P001 基础看板 | 添加/重命名/删除任务；todo/doing/done 三列与状态切换；本地保存；简单 VM 界面；用法 Wiki | CRUD、状态切换；关闭重开仍保留任务及状态 |
| P002 限制并行 | doing 上限 3；达到上限拒绝第 4 项进入；可见计数/解释；补规则 Spec、测试与 Wiki | 第 4 项拒绝且原状态不变；完成/移出后可再进入；P001 不退化 |
| P003 可配置规则 | 新会话；从现有 Specs 继续；上限可设整数 1～5 并保存；降到现有 doing 数以下保留任务、阻止新增；改善该提示文案 | 3→2 超限保留现有；无效输入保留原设定；重启保留设定及任务；回归前两轮 |

P003 不携带前两轮完整聊天、设计推理或手工实现建议；输入为增量需求及
已有工程位置，Agent 应通过实际读取 Specs/Plan/文件建立上下文。证据保存
真实读取事件及对应版本，而不是仅引用 Agent 声称“已读”。

每轮仅提交一次已授权需求后观察；needs_fix 由 A 处理；blocked/needs_replan
或修复封顶停止且留证。不发送“继续”、不改完成标记、不用外部脚本修业务代码。
记录工具/阶段角色实例，确认 review 为独立阶段 Agent。

本地保存必须跨 Canvas stop/start 和合入默认分支后的再次启动。一次性
localStorage 不算跨启动保存；不得改变 Canvas 的隔离/清理合同来凑验收。
若现有 VM 文件能力无法支持样例持久化，保持 AC 阻断并另行规划能力缺口，
不得把需求降为内存保存或偷偷引入新的依赖仓变更。

业务 Specs 为权威；ledger 派生，Wiki 只记用法/设计取舍并链接规范与计划。
要求各产品计划产出 docs/wiki/*.md，review 检查内容与更新；真实交付后驱动
通过既有 workspace Wiki API 发布同一正文，保存源 hash 与回读 hash。
这个确定性发布步骤在报告披露，不由脚本代写知识或建立第二套规范正文。

### 5.5 截图与证据合同

输出位于 D:/autostack/.demo/musk-097/<run_id>/evidence，清理平台
worktree 后仍可读；仓内 docs/reports 保存汇总与可定位的证据索引。
manifest 格式版本 1，包含运行指纹、workspace/session/run/plan 身份、
revision、各阶段/attempt/outcome、reviewed_commit、落地主提交与交付收据。
每个 capture 包含 capture_id、source（host-vm/target-canvas）、UTC 时间、
viewport/DPI、PNG 相对路径及 SHA256、关联状态快照、阶段事实/收据引用，
并区分 live 与 completed-stage-record；工具链、skills、源/二进制 hash 绑定整跑。

截图前后读取身份/阶段状态，拒绝跨工作区、跨计划或重启代次的错误归属；
等可见关键节点及稳定帧，不仅靠固定 sleep。PNG 保留原始画面，不重绘结果、
不拼接假宿主；缩略图属于派生文件。私有配置/凭据不进入分享报告。

每个产品计划至少覆盖：需求及计划正文、规划阶段、实施阶段、复审结论、
沉淀/交付结果、该轮目标 Canvas。优先边运行边取景；短暂阶段若未赶上，
只能拍原生 UI 明确标识的已完成阶段记录并绑定真实收据，报告注明 capture_kind，
不能声称实时现场。无该阶段事实或可见记录则 missing，不用 fixture 补图。
整跑还须有新会话 P003、规范正文、Wiki 页面、Canvas 点选→源码和最终工程图。

verify 检查身份、schema、文件存在/hash、四阶段和所有交付 checkpoint、Git
线性合入及自有 worktree/分支清理、行为断言、知识回读与截图覆盖。缺件、
截图失败、工程失败或 canceled 均非零退出。report 生成按 P001～003 排列的
HTML/Markdown 图文入口及机器结果；不得把未经 review 的证据自标 reviewed。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/modules/app-studio-ui.md | VM 进度 G-15 隐藏 → handler/store 同义可见；工具事实与计划阶段分开；保留 G-17/布局已知边界 | 展示真实 Canvas 能力 | AC-03,04,15 |
| SD-02 | modify | docs/specs/modules/workspace-ui.md | 补计划事实摘要/历史标签/陈旧与不兼容状态；知识导航无新增主栏目 | 避免完成/交付混淆与切换串台 | AC-01,02,05,13,15 |
| SD-03 | modify | docs/specs/modules/ui-parity.md | fixture/runtime 回归与真实工程演示分别取证；新进度组件双端门与缺件规则 | fixture 不能替代全流程实拍 | AC-03,04,06,07,15,17 |
| SD-04 | add | docs/specs/modules/demo-evidence.md | 新增隔离启动、三计划空仓、驱动权限、截图来源/hash/阶段标签、证据保存与失败语义 | 固定可复现演示合同 | AC-06～14,16,17 |

本阶段只提出 delta；work 准备可审文本，review 绑定版本，merge 再按技能
沉淀 canonical/index/derived ledger。A 的 plan-execution-contract 不由 B 重写。

## 6. 测试设计

命令在 B 专用树执行；新增命令/文件为此计划产物，T-01/T-02 冻结实际依赖
解析、staging 和启动命令到 baseline，报告记录退出码和预期。禁止在主检出
运行会改代码/生成物的构建。不得在 worktree 使用链接规避依赖路径。

| 验证 | 命令/方法 | 期望与必要反例 |
|---|---|---|
| V01 探针与冻结 | node scripts/demo-focusboard.mjs preflight --output <本跑目录>（新增）；T-01 先用等价小探针，T-02 补 A 合同检查 | 私有宿主截图可读、真实持久化跨启、构建合法；缺端点/旧二进制/缺 A 合同明确失败 |
| V02 合同/采集测试 | node --test tests/demo/*.test.mjs（新增） | 错 identity/version、迟到轮询、失败截图、hash 漂移、missing 阶段/收据、非 owned PID、wiki hash 错均被拒绝；有效样本通过 |
| V03 前端静态/链接 | auto build --gen-only --strict；scripts/vm-link-probe.cmd；node scripts/ui-parity.mjs check | 使用 T-01 冻结 AUTO_EXE/依赖解析；无新增错误，声明/fixtures 对账；生成源供外部 staging 做 Vue 构建 |
| V04 同源前端行为 | staging 的 Vue 构建/生产 helper 针对性测试 + VM 真机；Gallery 新 PlanProgress/CanvasProgress case | 四阶段、needs_fix、blocked、交付缺项、断连恢复、切换迟到拒收、中英文；VM+RustHTTP 与 VM+VMHTTP 接线均取证，Vue 回归通过 |
| V05 正式三轮演示 | node scripts/demo-focusboard.mjs run --output <新跑目录> | 原生 VM+自有 RustHTTP，真实模型；空仓三个计划四阶段 delivered；无人工 nudge、无脚本业务改写；截图齐备 |
| V06 独立证据核验 | node scripts/demo-focusboard.mjs verify --manifest <path>；node scripts/demo-focusboard.mjs report --manifest <path> | 无依赖运行中进程仍可验 hash/收据/历史工程结果；故意删图、改收据、用 fixture 冒充必失败 |
| V07 受影响后端回归 | cargo test --manifest-path backend/Cargo.toml -p musk --test parity_relay --test parity_relay_store --test parity_relay_api -- --test-threads=1；cargo test --manifest-path backend/Cargo.toml -p musk --lib -- --test-threads=1 | 基线定责、无本改动新增红；若未改后端仍核验前端使用的 query 契约，不宣称 mock 为真实 VMHTTP |
| V08 产物与清理 | cargo build --manifest-path backend/Cargo.toml -p musk --release；固定 hash；owned 进程退出/端口、Git worktree/分支核查 | 实拍绑定当前待审提交，不替换共享部署；外部 evidence 可读，demo worktree 无遗留 |

业务验证在每轮合入后通过目标 VM 动作进行：CRUD/状态、第四项拒绝、释放
额度、上下限及无效输入、降低上限保留现有、跨停止/再次启动保存。通过真实
可观察状态/磁盘数据/界面交叉验证，不能只验文本提示。P003 再跑前两轮行为。
缺 API/几何不使用固定假坐标，T-01 先证明交互路径可达。

## 7. 验收标准

| ID | 可观察结果 | 验证 |
|---|---|---|
| AC-01 | VM 展示 A 的真实阶段/attempt/修复/阻塞/检查点；只有 A delivered 才显示已交付，缺件完成不绿 | V02/V04/V05 |
| AC-02 | VM 单飞异步查询；切换工作区/会话/run 和迟到结果不串台；断连可见陈旧、恢复可见新事实；终态停止轮询 | V02/V04 逆序响应/断连实测 |
| AC-03 | G-15 消除：VM 生成/启动/运行/验证进度实际可见，与 Vue 语义一致；未验证/lint/失败不伪成功 | V03/V04/V05 |
| AC-04 | Canvas 实际运行 FocusBoard；有效节点点选、层树与源码联动截图；置信度真实，未用零 rect 造坐标 | V01/V04/V05 操作与锚点载荷 |
| AC-05 | 保留五栏目及 composer/审批/Canvas 状态；中英标签完整；1024/1280 宽可读、无页面横滚 | V04 宿主快照与 PNG |
| AC-06 | 宿主原生 VM 与目标 Canvas 截图来源区分；PNG 是实际运行截图、可读且原始 hash 匹配，fixture/Web 不算 VM | V02/V05/V06 |
| AC-07 | 每产品计划的需求/计划、四阶段和目标图齐备；live/历史阶段记录标识准确；缺阶段事实/截图明确失败 | V05/V06 覆盖矩阵 |
| AC-08 | 指纹绑定源/二进制/依赖/skills、workspace/session/run/plan/revision/reviewed_commit、收据与截图；错绑/漂移失败 | V01/V02/V06 |
| AC-09 | 全新空工程，无预制业务代码/Specs/解决方案测试；Agent 真正用三个产品计划完成全部增量 | V05 初始 Git 清单与全程工具/提交审计 |
| AC-10 | 每产品计划四技能语义真实落地、review 独立、默认分支线性合入、ledger/归档/自有 worktree 清理齐全，无手工继续或外部修代码 | V05 各轮 A 收据+Git+日志 |
| AC-11 | P001 CRUD/三列/状态跨停启保存；P002 doing=3 拒第四项且释放后允许；P003 整数1～5/无效值拒绝/降限不删任务；最终跨启保存并回归前两轮 | V01/V05 目标 VM 动作+状态/磁盘 |
| AC-12 | P003 新会话无前两轮聊天注入；真实读取已有 Specs/工件后开发，规则在增量后仍正确 | V05 会话输入、工具读取与版本证据 |
| AC-13 | 三轮规范为权威、ledger 派生；Wiki 为 Agent 产出并经复审的用法/取舍，API 发布正文 hash 回读一致，VM 中可见 | V05/V06 源文/回读/栏目截图 |
| AC-14 | 自有进程/私有端点/运行目录隔离；取消/失败只清自己，共享服务与用户配置不变；worktree 无链接，无残余 demo worktree | V01/V02/V05/V08 |
| AC-15 | 受影响 Vue/VM 双端构建/链接/静态门通过；VM 两后端事实接线有真机证据；基线债明确且不豁免本计划关键行为 | V03/V04/V07 |
| AC-16 | 三轮失败/needs_replan/封顶/缺截图停止演示并非零，保留失败现场；不改 A 上限、不覆盖旧跑、不伪最终成功 | V02/V05失败注入 |
| AC-17 | 图文报告和机器 verify 均可复用，外部证据清理后仍可读；每 AC/Spec delta 有版本绑定证据，缺件或变动不会通过 | V06/V08 与独立 review |

## 8. 执行步骤

执行 AC 列中的数字均指 AC-XX。步骤完成需写实跑命令/退出码/工件路径；
缺必要依赖保持未完成。不以修改报告结论代替修复。

| 完成/任务 | 依赖 | 文件/符号与产出 | 验证与预期 | AC |
|---|---|---|---|---|
| [x] T-01 | 无；可与 A 并行 | 新 docs/reports/097-vm-demo-baseline.md；只读 canvas/mcp_client.rs、session.rs、scripts/ui-parity/live.mjs、relay_store.at、既有 VM 文件能力；四项有限探针，记录截图/动作/私有端口/跨启保存/无链接构建路线 | V01 等价探针全部具备命令与决定；缺能力明确定位，不预制业务、不改依赖仓 | 04,06,11,14,15 |
| [ ] T-02 | T-01；A review/merge 已交付 | 冻结 A plan-execution-contract/实际 query 与入口、DTO/event/成功失败样例/hash 到 baseline；新增 scripts/demo/contract.mjs；确定 store owner、确切异步通道与 staging 命令 | V01/V02 正确事实可消费，未知版本/缺件明确拒绝；必要 A 缺口回交，不解析叙述补事实 | 01,02,08,16 |
| [ ] T-03 | T-02 | 新 plan_progress_store.at/plan_progress_helpers.at（或有据并入 RelayStore）；接 relay_store.at/forge_store.at；单飞、身份/代次、陈旧、终态、纯投影 | V02/V04 乱序/切换/断连/修复/交付缺项；新身份不受迟到污染 | 01,02,08 |
| [ ] T-04 | T-03 | 新 plan_progress.at；接 chats_view.at/relay_run_box.at；摘要、展开阶段记录/历史标签/receipt；src/front/i18n/{zh,en}.json 与 lib/i18n.at 既有接线 | V03/V04 原生 VM 可见四阶段与失败，中英/宽度/草稿保留；历史不能标实时 | 01,05,07,15 |
| [ ] T-05 | T-03 | canvas_progress.at/canvas_helpers.at/forge_store.at 与相关 CanvasStore 消息；必要 rows 预计算，消除 G-15，保持工具事实分类 | V03/V04 真 VM tool→store→可见摘要；未验证/失败不绿，seq 不宣称因果 | 03,04,15 |
| [ ] T-06 | T-02 | 新 scripts/demo/vm-session.mjs 与 demo-focusboard.mjs preflight；私有配置/端口、实际端点发现、源/二进制指纹、owned 关停 | V01/V02/V08 端口冲突、旧产物拒绝、非 owned PID 不终止；无 worktree 链接 | 08,14,16 |
| [ ] T-07 | T-04,T-06 | 新 capture.mjs/evidence.mjs；宿主/目标采集、状态稳定/身份二次核验、manifest v1、历史标识、缺件失败、外部保存 | V02 删除图/错 hash/跨身份/截图超时/缺阶段注入全部失败；原图可读 | 06,07,08,16,17 |
| [ ] T-08 | T-02,T-06 | 新 scripts/demo/focusboard.mjs 与 examples/focusboard-demo/{README.md,requests/001.md,002.md,003.md}（仅需求/约定）；空仓 bootstrap、三轮正式入口、新会话、交付等待、工程外置 worktree | V02 bootstrap 白名单与不写业务审计；停止条件正确，无手工 nudge/外部修复 | 09,10,12,14,16 |
| [ ] T-09 | T-07,T-08 | 目标 VM 行为断言/跨启保存/Canvas锚点；既有 Wiki API 同正文发布/回读；Specs/ledger/归档/ff/清理对账 | V02/V05各轮断言、Wiki hash、P003上下文来源；工程未交付不发下一需求 | 04,10,11,12,13 |
| [ ] T-10 | T-03～T-09 | 新 tests/demo/*.test.mjs；生产 helper 针对性测试；tests/ui-parity/cases.json/fixtures 与 Gallery 新组件接入；覆盖失败/乱序/归属/过期/缺件 | V02 全部针对性测试及 V03 静态对账，无镜像实现测试替代行为验证 | 01～10,13,14,16,17 |
| [ ] T-11 | T-10 | 自有 Vue staging + VM 两后端运行；scripts/ui-parity.mjs 受影响 case；受影响后端 query/parity 回归证据 | V03/V04/V07 无新增红，真机原图与事实匹配；已有 G-17 不用伪 bounds 豁免操作 | 01～05,14,15 |
| [ ] T-12 | T-09,T-11 | 新跑真实模型完整 P001～003；正式 VM+RustHTTP；全量 screenshot/log/receipt/行为矩阵；失败保留现场，修复走 A 或本计划 scoped 代码 | V05/V06 三个 delivered +全部行为/截图/知识证据；不得直接改示例来使验收变绿 | 06～14,16,17 |
| [ ] T-13 | T-12 | 新 docs/reports/097-vm-demo-evidence.md、097-vm-demo-spec-delta.md；HTML/Markdown report命令；SD-01～04 可审 before/after 文本及 AC 索引 | V06 离线核 hash/覆盖/收据；外部路径长期可读，区别模型行为与脚本出版步骤；canonical 不提前改 | 07,08,13,17 |
| [ ] T-14 | T-11～T-13 | scoped 修复后重跑受影响门；提交待审实现、固定当前产物 hash；本 Plan 进度/交接更新；owned 进程清理核验 | V02～V08；所有必需 AC 绑定待审提交、worktree 已提交干净，才 execution_done → 独立 review；不自标 reviewed/merged | 01～17 |

## 9. 复审记录

### new 阶段交接（草稿准备完成）

- stage: new
- plan_id: PLAN-097
- plan_revision: 1
- outcome: pass
- next: work；可先执行 T-01。T-02 及后续接线需 PLAN-096 已交付并冻结消费合同。
- changed_tasks: T-01～T-14
- changed_acceptance: AC-01～AC-17
- spec_delta: SD-01～SD-04
- evidence: 规划时读取的 Spec/hash、源代码观察及 A T-01/T-02 已完成事实见 §4；
  此结果只表示草稿可交接，不表示 B 已实施、A 已交付或三轮演示已通过。
- review_authority: 后续独立 auto-plan:review 在 B 专用树复验实现/证据并冻结
  Spec delta；auto-plan:merge 再按 revision-bound 证据沉淀及清理。

### work 阶段记录（T-01 完成；含重启背景）

- stage: work | plan_id: PLAN-097 | plan_revision: 1 | outcome: pass（T-01 四项探针全部可行）
- code_commit(worktree plan-097-dev): 61b35c3（基 7924304；重启时将早前过早开工的
  残留重置丢弃——旧分支上仅存 096 已落地等价补丁，无 097 实现遗留，git cherry
  补丁级等价确认，reset 无损）
- task_ids: T-01
- evidence: 新 docs/reports/097-vm-demo-baseline.md（61b35c3）——①隔离启动+宿主
  MCP 截图/动作 12/12 PASS（私有端口、netstat PID 归属、composer type_text+新建
  会话 press、1920x1200 原图 PNG hash 绑定、自有树清理端口回收；收据
  .demo/musk-097/t01-20261002091414）；②Canvas 文件跨启持久化 12/12 PASS
  （app 目录内 File.write_text/read_text 路线：boot.log 逐行读回
  PERSIST-TOKEN-001，跨 stop/start 存活；越界 app_path 400 fail-closed；frame/
  source/pick 通道可用；收据 .demo/musk-097/t01-20261002085418-persist）；
  ③无链接构建 staging 可行（worktree gen-only 18.8s 无链接→外部 staging 须镜像
  vendor/@autodown file: 依赖→warm store 离线安装 4.3s→vue-tsc+vite 13.5s 出
  dist，worktree 全程 wt-guard clean）。冻结配方与 6 项基线发现详见报告：
  宿主 VM 冷启直达主 shell（无登录表单，080 live.mjs 流程过时）、VM 轨
  ChatsView SendInput/CtxRemove handler 合成失败（canvasStore 未定义，既有债，
  T-05 同域时评估）、宿主 VM 进程偶发无错提前退出（驱动须处理退出事件）、
  worktree 运行模块解析打到主检出（auto-man 按项目名全局解析，T-06 前须实测
  处理）、pnpm 镜像不稳（一律 staging+lockfile+--offline）、VM 运行期 cwd=
  src/front（app 相对路径落 <app>/src/front/data，跨启存活）
- blockers: 无（发现 1/3/4 已在报告定级，均不阻断 T-02）
- next: T-02（A 的 plan-execution-contract 已随 096 交付落地，冻结消费合同/
  DTO/hash 到 baseline；随后 T-03 起接线）

## 10. 待澄清事项

1. **A 的定版消费合同尚未交付。** owner=A 的 T-10/T-13 与 B T-02；可以
   并行做 T-01，不把 7f4bdbb 初稿当正式接口。必要字段缺失先回交 A。
2. **真实跨重启保存路线待探针。** owner=B T-01；冻结现有 VM 文件接口/
   独立数据根/启动配置和验证命令。若必须改上游或 Canvas 隔离合同，保留
   AC-11 阻断并提出额外计划，不在本稿授权范围内静默扩大。
3. **宿主取景与短暂阶段可见性。** owner=B T-01/T-04/T-07；证明真实 MCP
   节点动作与截图可达，实时取景优先，完成记录明确标签，缺事实不能补造。
4. **无链接构建 staging 与现有依赖解析。** owner=B T-01；记录真实复制/
   构建命令和源 hash。若工具写链接进 worktree，改到外部 staging 后再跑。

以上均有本计划内的有界调查任务，当前无须用户重复确认既有示例方向。调查
导致目标/验收/允许仓库范围改变时，按 auto-plan:new 修订并保留原 AC 与证据。
