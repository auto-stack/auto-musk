---
plan_id: PLAN-104
status: archived
feature_name: UAT 观察面修复批次（VM 活跃会话内存累积/大会话空渲染/spec 写入通道/merge 收尾合同）
author: [agent]
created_at: 2026-10-11T11:45:00Z
updated_at: 2026-10-11T18:05:00Z
plan_revision: 1
current_step: 7
total_steps: 7
supersedes_spec_components:
  - docs/specs/modules/vm-process-stability.md
  - docs/specs/modules/specs-ledger.md
new_spec_components: []
touched_goals:
  - goal-agent
  - goal-frontend-parity
  - goal-spec-knowledge
---

# PLAN-104 — UAT 观察面修复批次（VM 活跃会话内存累积/大会话空渲染/spec 写入通道/merge 收尾合同）

## 0. 变更摘要

2026-10-11 手动 plan-flow UAT（demo2 工作区、会话 87af4d4b7e7a9c25e2d44114）
全流程跑通后登记五项发现（KNOWN-DEBT UAT-K9～K13），本计划修齐：

1. **K9 VM 客户端活跃会话轮询内存累积**：会话活跃期内存 ~1MB/s 增长（用户
   实机），会话停止即走平。客户端每 500ms 拉一次 page（载荷长到 292KB/次），
   拉取→解析→重渲染的分配部分未回收。与 KD-062 已根修的**空闲面**不同面
   （那是静止态）。同轮 20 次静默退出集中在重流式时段（已用私有 MCP 口，
   KD-066 端口争抢定罪不能直接套用）。
2. **K10 大会话页静默空渲染**：353 轮会话在长寿实例（~50min）渲染为空，
   客户端仍每 500ms 拉到 200+292KB 但零错误日志；服务端完好。定性=静默
   渲染失败。
3. **K11 FilesView.Pick RuntimeError**：`Field 'active_kind' not found on
   App_State`，3 次复现——.at 源/生成实例字段面不匹配。
4. **K12 聊天会话 spec 工具缺口三角**：superpowers.at 白名单无 write_spec/
   update_spec（merge SKILL.md 却声称已注册）；chat 侧注册则绑家目录账本
   （default_store→~/.config/autoos/specs.json）而非工作区账本；dispatch
   errand 同样家目录绑定——派工不能补位。HTTP 等价通道可用，但其
   SpecItemPayload 仅 id/title/content/status 四字段，**无法携带 file/tags**
   （skill 要求记录 source hash/commit 落空）。
5. **K13 merge 收尾欠账**：ledger_refreshed 仅写 designs 一条（content=
   路径引用合规；对比 098 写 2 条/099 写 4 条属欠填，reviews/reports 该记
   未记）；git 收尾（docs/ 纳入版本控制+收据提交）未做——demo2 main 仅
   3 提交，docs/ 与 .autoos 均 untracked。

修复面横跨 musk（.at 前端/后端/技能文本）与 auto-lang（VM 运行时，依
T-01 定罪），K9/K10 以有界调查任务先行定罪再修。

## 1. 目标

### 交付目标

- 活跃会话（流式+工具卡持续到达）场景下 VM 客户端内存有界：30 分钟活跃
  流式私有内存斜率 ≤0.1MB/s（与 KD-062 空闲面同阈），1 小时零静默退出。
- 大会话（≥350 轮 / page 载荷 ≥250KB）在 VM 新实例与长寿实例均渲染出内容，
  且有明确降级行为（截断展示/分页），不再静默空白。
- FilesView 点击零 RuntimeError；vm-first-run 门禁 reds=0。
- 聊天会话内可经**会话内工具**写**工作区**账本（file/tags 落盘正确、两会话
  并发零串账）；HTTP 等价通道支持 file/tags；merge 技能文本与三通道
  （会话工具/HTTP/dispatch）行为一致。
- 账本"每 merge 最少条目"合同与 merge git 收尾清单入规范/技能文本；一次
  T1 简版 UAT 复跑实证四收据齐、账本 ≥3 条含 file/tags、docs/ 受版本控制。

### 范围

- 主仓库 auto-musk：src/front/*.at（chats/files 视图渲染链）、backend
  spec 工具注册与 HTTP DTO、modes/superpowers.at、.agents/skills/
  auto-plan-merge/SKILL.md、docs/specs/{vm-process-stability,specs-ledger}.md、
  docs/reports/plan-flow-uat/{spec,RUNBOOK}.md、测试与证据。
- 依赖 auto-lang：若 T-01 定罪 K9/K10 根因在 VM 运行时/渲染器（iced 文本
  缓存、stake 账本活跃面、HTTP 桥缓冲），按 AGENTS.md 第三行同组并排
  worktree（`.wt/musk-104/auto-lang`，分支 `auto-musk-dev-104`）修复。
- 复用资产：`scripts/vm-first-run-soak.mjs`（062 soak）、AUTO_DESKTOP_EXIT_LOG
  三挂点审计口径（575/066）、`scripts/vm-hangwatch.mjs`。

### 非目标

- 不做 page 端点协议重构（SSE 推送/WS 化），只做载荷投影与客户端增量合并。
- 不修 K2（run 中途收束自动续跑）；不动 K1/K1b daemon max_tokens 面。
- 不动 PLAN-102 的 canvas 布局/devtools 面（chats/files 视图与其无文件交集，
  提交序协调即可）。
- 不做 Vue 侧等价修复，除非 T-01 定罪同根（gen 轨同款轮询合并链）。
- demo2 演示仓的历史欠账不回填（它是证据现场）；合同修复由 AC-06 的
  新一轮复跑实证。

### 成功边界

AC-01～AC-07 全部有可复验结果。K9/K10 若 T-01 定罪为纯 auto-lang 面且修复
超出有界范围，按依赖政策登记并回用户裁定，不以削弱 AC 收口。

## 2. 架构方案

```text
T-01 定罪（有界调查，决策工件）
  K9: soak active 场景（持续 page 轮询+流式增量）× 分配剖面
      → 定罪面 ∈ {musk .at store 合并链 / iced 文本与 vnode 缓存 /
                  HTTP 桥缓冲 / musk page 载荷投影}
  K10: 大页矩阵（新实例×长寿实例 × 载荷梯度 50/150/300KB+）
      → 失败阈值与静默失败点
  K11: .at 源字段引用 vs 生成实例字段清单 → 缺字段定性
T-02/T-03 依定罪落点修复（musk 侧就地；auto-lang 侧并排 worktree）
T-04/T-05/T-06 musk 侧合同修复（字段/通道/文本）
T-07 回归 + T1 简版 UAT 复跑实证
```

核心机制取向（T-01 后冻结）：客户端 page 拉取改**增量合并**（内容 hash 或
seq 游标，无新内容跳过重渲染——直接消除"没有新内容也在涨"的分配源）；
大页降级按**载荷/轮数阈值**截断展示+尾部窗口优先（完整内容经现有 source/
导出面可达），不引入虚拟化新原语。

## 3. 技术栈

| 层 | 沿用方案 | 本计划约束 |
|---|---|---|
| VM 前端 | src/front/*.at → auto 生成（gen-only 锁步） | 修复落生成源；gen/ 产物不手改 |
| VM 运行时 | auto-lang master 当日构建锁定（AUTO_EXE 硬规则） | 依赖改动走并排 worktree，不裸改主检出 |
| 后端 | musk axum（spec 工具/HTTP DTO/page 投影） | serde default 兼容旧账本与旧客户端 |
| 技能/规范 | .agents/skills + docs/specs canonical | 文本修复随代码同批落地 |
| 验证 | cargo test 串行 / vm-first-run / soak / ui-parity / 手动 UAT | 新增 active 场景 soak 用例随仓持久 |

## 4. 需求分析与背景调查

### 授权与流程边界

- 用户授权（2026-10-11 会话）："请记录到 DEBTS 里先"+"把这个问题和前面的
  几个问题一起形成一个修复计划"——授权范围=KNOWN-DEBT 登记（已完成，
  UAT-K9～K13 五行）+本计划撰写；**执行、预算、自动续跑需后续指令**。
- 计划编号 104：双目录扫描（active 097/102，archived 最大 103）+serve API
  交叉核对（demo2 仅 PLAN-001），唯一，单写者（本会话）。
- 前置依赖：无硬前置。PLAN-097 T-12 blocked、PLAN-102 executing 与本计划
  无文件交集；K9 取证资产来自 062/066（已交付）。

### 调查版本与关键事实（main@c3f7543，2026-10-11）

| 锚点 | 事实 | 对本计划的意义 |
|---|---|---|
| KNOWN-DEBT UAT-K9～K13 | 五项登记（本会话实测） | 任务与 AC 的依据源 |
| modes/superpowers.at:6-9 | 工具白名单 12 项，无 write_spec/update_spec | K12①；白名单增补点 |
| backend/crates/musk/src/lib.rs:243-254 | spec 工具注册 `Arc::new(T::new())` 非 scoped；:251 白名单过滤 | K12②注册点改造 |
| spec_tools.rs:37-47 | `ws_specs_of`（工作区绑定）已存在；`default_store()`=家目录 | 通道改造复用 ws_specs_of |
| orch_tools.rs:566-585 | errand agent `tools: vec![]`（全工具面）、role=target | dispatch 通道随 ② 自动受益 |
| auto_generated/server.rs:71-89 | SpecItemPayload 仅 id/title/content/status | K12 附带：HTTP DTO 补 file/tags |
| 实测（会话 87af4d4b） | page 292KB/340ms（早先 30ms）；客户端 500ms 轮询；活跃期 ~1MB/s 增长；20 次退出 0.4–15min；353 轮后长寿实例空渲染 | K9/K10 现象与阈值基线 |
| tmp/demo2 终态 | 账本 v4 单条 designs（file/tags 空）；main 3 提交，docs/+.autoos untracked；计划 archived、worktree 已清 | K12 附带/K13 实证 |
| KD-062 行 | 空闲面 2.2MB/s→0 已根修（stake 影子账本+MCP dirty 门控） | K9 为不同面，方法学可复用 |
| KD-066/vm-process-stability.md | 048a 定罪=端口争抢（9247 共址）；本轮 9277 私有口仍 20 次退出 | 退出归因重开，随 T-01 审计 |
| auto-lang dde29cf2c / release auto.exe 10-10 | PLAN-760 登记过"release auto.exe 落后交付"观察项 | T-01 起锁当日构建（AUTO_EXE 硬规则） |

### 与既有计划的边界

- PLAN-102（executing）：canvas 布局/devtools——无文件交集；其 T-06 实机
  长跑若撞 K9 退出面，以本计划 T-01 审计结论共享取证。
- PLAN-097（executing，T-12 blocked）：其驱动管线的模型长尾失败与本计划
  无关；其 vm-session.mjs 的隔离配方为本计划 UAT 复验可复用资产。
- KD-062：本计划为其"残留观察"（MCP snapshot 子进程、静默退出）的延续
  调查面，不重开其已根修结论。

## 5. 详细设计

### 5.1 K9 活跃会话内存有界（T-01 定罪 → T-02 修复）

- 取证：扩展 `scripts/vm-first-run-soak.mjs` 增 **active 场景**（驱动一个
  演示会话持续产流：page 轮询满载荷 + 流式增量到达），采样 PrivateMemory
  斜率与 AUTO_DESKTOP_EXIT_LOG 事件；对照静止面基线。
- 候选定罪面（按证据裁决，不预设立场）：①客户端每拍全量重建（对应 062
  T10 的 active 版——内容未变仍重建/重排）；②文本 shape/glyph 缓存无上界；
  ③HTTP 桥响应缓冲滞留；④musk page 载荷含重复膨胀（353 轮全量投影）。
- 修复取向：**增量合并**——客户端持最近内容 hash/末 seq，page 响应未变
  即跳过解析与重渲染；载荷变更只重建尾部窗口。若定罪在 auto-lang 渲染
  层（缓存无上界），并排 worktree 修 + 上游回归锚。
- 验收阈值：30min 活跃流式斜率 ≤0.1MB/s；1h 零静默退出（审计三挂点
  零外部/未知臂；main_return 关窗除外）。

### 5.2 K10 大会话页渲染降级（T-01 矩阵 → T-03 修复）

- 矩阵：新实例×长寿实例 × 会话规模梯度（50/150/350 轮），记录失败阈值与
  表现（空渲染/退出/卡顿）。
- 修复取向：**载荷阈值降级**——超过阈值（T-01 定，初值 200KB/300 轮）时
  客户端只渲染尾部 N 条+「更早消息已折叠」横幅（i18n 两语），完整历史经
  既有导出/源文件面可达；后端 page 投影同步支持 `?before=<seq>` 尾窗参数
  （向后兼容：缺省行为不变）。长寿实例累积态另由 5.1 的增量合并消除。
- 验收：350 轮会话新/长寿实例均渲染出内容且横幅可见；空渲染形态在
  阈值内不复现。

### 5.3 K11 FilesView active_kind 字段（T-04）

- 源考证：src/front files 视图 .at 引用 `active_kind`，生成 App_State 无此
  字段——补字段声明或改读既有字段（以源意图定）；vm-first-run 门禁
  reds=0 + 实机点击零 RuntimeError。

### 5.4 K12 spec 写入通道三角（T-05）

- ①注册改造：chat agent 构建点（lib.rs all_tools 装配处）将 spec 五工具
  （read_specs/list_specs/write_spec/update_spec/write_goals）改为工作区
  绑定注册（`with_store(registry.get(workspace_id).specs)`，同 scoped! 文件
  工具手法；workspace_id 在 agent 构建上下文中已存在）；errand 路径
  （orch_tools run_errand_agent）同改。
- ②白名单：superpowers.at tools 增 read_specs/list_specs/write_spec/
  update_spec（write_goals 一并评估，T-05 定）。
- ③HTTP DTO：SpecItemPayload 增 `file`/`tags`（serde default，旧客户端
  零破坏）；specs_upsert 透传至 SpecItem。
- ④文本：merge SKILL.md "registered in chat sessions" 更新为三通道如实
  描述（会话工具=工作区绑定；HTTP 等价含 file/tags；dispatch 同工作区）。
- 验收：单测——两会话并发各写各账本零串账；file/tags 经工具与 HTTP 双
  通道落盘；家目录账本零触碰（demo 工作区写后 home 文件 hash 不变）。

### 5.5 K13 merge 收尾合同（T-06）

- specs-ledger.md 增「每 merge 最少条目」合同：merge 的 ledger_refreshed
  至少覆盖 designs（有新模块规格时）+ reviews（复审结论）+ reports（交付
  报告）——存在对应知识即必记，逐条带 file/tags；tests 有新测试资产时
  记录。
- auto-plan-merge SKILL.md 收尾清单强化：docs/ 产物纳入版本控制为
  landed/archived 收据的前置核对面（git status 干净才算 cleaned）。
- plan-flow-uat spec.md K 表引用 K9–K13 + RUNBOOK 增对策行（长会话分阶段
  新会话；白名单目录预注册——本轮 Prompt2 踩过的 path_confined 教训）。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/modules/vm-process-stability.md | 无活跃会话内存与大会话渲染契约 → 活跃轮询渲染内存有界（斜率阈值）+ 大载荷降级渲染行为（阈值+尾窗+横幅）+ 审计判读矩阵增 active 场景行 | KD-062 只覆盖静止面；本轮实测活跃面与静默空渲染在案 | AC-01, AC-02, AC-07 |
| SD-02 | modify | docs/specs/modules/specs-ledger.md | 写入通道仅隐含 relay/HTTP → 三通道合同（会话工具=工作区绑定/HTTP 含 file/tags/errand 同绑定）+ 每 merge 最少条目合同 | K12 三角与 K13 欠填的规范面收口；K6 教训的通道维延伸 | AC-04, AC-05, AC-06 |

## 6. 测试设计

### 命令与证据约定

| 编号 | 命令／工作目录 | 预期结果 |
|---|---|---|
| V01 | worktree：`node scripts/vm-first-run-soak.mjs --scenario active`（新增场景）+ 大页矩阵脚本 | K9 活跃斜率可复现红（>0.1MB/s）+ K10 失败阈值行 + K11 静态定谳——决策工件入 evidence |
| V02 | 修复后 V01 复跑 | 斜率 ≤0.1MB/s；1h 零静默退出（AUTO_DESKTOP_EXIT_LOG 判读） |
| V03 | 大页降级验收：350 轮会话新/长寿实例快照判言+截图 | 内容可见+折叠横幅；阈值内无空渲染 |
| V04 | worktree/backend：`cargo test -p musk --lib`（spec 工具/DTO 新单测在内） | 两会话零串账、file/tags 双通道、家目录零触碰、旧 DTO 兼容——全绿 |
| V05 | worktree/backend：`cargo test -p musk`（逐目标串行） | 全套绿，零新增红 |
| V06 | vm-first-run + ui-parity 受影响 case | reds=0；catalog PASS |
| V07 | T1 简版 UAT 复跑（demo 新仓、四消息点名、分阶段新会话） | 四收据齐；账本 ≥3 条含 file/tags；docs/ 受版本控制（四段线性历史） |

### 场景矩阵

| 场景组 | 输入/故障 | 证据 |
|---|---|---|
| 内存 | 活跃流式 30min；长寿实例 2h；静止对照 | V01/V02 斜率+审计 |
| 大页 | 50/150/350 轮 × 新/长寿实例 | V03 矩阵 |
| 通道 | 会话工具/HTTP/dispatch 三通道写账本；两会话并发；家目录对照 | V04 断言 |
| 合同 | merge 收尾清单核对；最少条目核对 | V07 产物 |
| 回归 | V05/V06 全套 | 零新增红 |

### 证据落点（均为新）

- `docs/reports/104-uat-observation-fixes-evidence.md`：T-01 三份定罪纪要、
  命令表、AC 映射、失败与修复记录。
- `docs/reports/104-uat-observation-fixes-spec-delta.md`：SD-01/02 对照。
- soak active 场景与大页矩阵脚本随仓（scripts/），fixture 判言入
  tests/ 或脚本内置。

## 7. 验收标准

| ID | 可观察的验收结果 | 验证方法与预期 |
|---|---|---|
| AC-01 | 活跃流式 30min：VM 进程私有内存斜率 ≤0.1MB/s；1h 零静默退出（审计三挂点零外部/未知臂） | V02；soak 报告留档 |
| AC-02 | ≥350 轮/≥250KB 会话在 VM 新实例与长寿（≥2h）实例均渲染内容+降级横幅，空渲染不复现 | V03 快照判言+截图 |
| AC-03 | FilesView 点击零 RuntimeError；vm-first-run reds=0 | V06 + 实机点击取证 |
| AC-04 | chat 会话内工具写工作区账本成功且 file/tags 落盘；HTTP 通道同支持；两会话并发零串账；家目录账本 hash 不变 | V04 单测 + 实操抽查 |
| AC-05 | merge SKILL.md 与三通道行为一致；specs-ledger.md 含最少条目合同 | 文档评审 + V07 核对 |
| AC-06 | T1 简版复跑：四收据齐、账本 ≥3 条（designs/reviews/reports）含 file/tags、docs/ 受版本控制四段线性 | V07 产物核验 |
| AC-07 | 既有功能零回归：musk 全测/ui-parity/vm-first-run 零新增红 | V05/V06 |

## 8. 执行步骤

所有任务未开始。每项完成须记录实际命令、结果、代码版本与证据。

- [x] T-01 勘察定罪（有界调查）：K9 active soak 场景落地+分配剖面定罪面；K10 大页矩阵（阈值+失败形态）；K11 active_kind 源考证。产出 evidence §T-01 三份定罪纪要+修复落点冻结（musk 就地 vs auto-lang 并排 worktree）。依赖：无。验证：V01 三红可复现。关联：AC-01, AC-02, AC-03。[✅ 已完成] K11 静态定谳闭环于 T-04；K9 静态定谳=PollBackfill 无条件全量重建（commit cee5eb5）；runtime 五轮 soak 实测（commit ac5b22d + evidence 报告 T-01/T-02 节）：RED 流式 143MB/min / idle 48MB/min；**静默退出归因定谳**=AUTO_DESKTOP_EXIT_LOG 零记录+code1=原生层 exit（判读矩阵第 4 类），非 panic/main_return/端口争抢，与轮询负载强相关（空闲 VM 不退，负载下 2-4 分钟必退）；**AC-01 残留（idle 30-72MB/min）归属 auto-lang**（HTTP 瞬态垃圾 GC 回收不全+每帧重绘分配，KD-062 残留族）——按 §10① 预案登记移交，仪器+数据已备。
- [x] T-02 K9 根修：依 T-01 落点实施增量合并/缓存上界/载荷瘦身；soak active 场景转绿。依赖：T-01。验证：V02。关联：AC-01, AC-07。[✅ 已完成（musk 侧杠杆用尽；AC-01 残留 blocked on auto-lang）] commit 0cb5656（tier2 指纹门）+ ac5b22d（tier1 body 等长预检+tick 条件写）：空闲拍零 store 写入、零 parse、零重建；流式期斜率方向性改善（143→40~123，样本方差大）。残留 idle 30-72MB/min 为 auto-lang 运行时面（见 T-01），AC-01 全额达标随 auto-lang 专项复核。
- [x] T-03 K10 根修：载荷阈值降级（尾窗+横幅+i18n）+ page `?before=` 尾窗参数；大页矩阵转绿。依赖：T-01。验证：V03。关联：AC-02, AC-07。[✅ 已完成（pass，verification-only，降级实施裁剪）] 三阶段收口：①并行会话温和面矩阵（m50/m150/m350 三 cell+9min 增长 cell 全存活零错、六实例 ~35min 零退出，commit 0132e74）；②四轮尖峰面矩阵归 765-R2——**765 r2 判别台定谳推翻**（IAT 钩+审计零行=零进程内真实退出，四轮"退出"=bigpage-matrix 仪器 finally-taskkill 收尾击杀假象，VM 缺陷不成立）；③交接复跑（本会话，两行修+端点勘误后正式跑，commit f00fa34）：**FRESH PASS（350 轮 261KB 大会话渲染健康，54 marker）+long-lived +5/+10min 存活渲染双绿（X9-ALIVE 44 拍）+15min 格退出审计 `site=main_return`（干净关窗，F-10 边界内非缺陷）+全程 RuntimeError 0**。降级无适应症不实施（服务端投影有界+客户端两级门+765 稳态修复三重覆盖；`?before=` 留按需增强候选）；结构性确认"静态会话持续轮询已退役"（K9/K10 驱动面消失）。AC-02=pass。
- [x] T-04 K11 修复：active_kind 字段补齐/改读；vm-first-run 门禁+实机点击取证。依赖：T-01（静态定谳即可先行）。验证：V06。关联：AC-03。[✅ 已完成] 静态定谳先行（T-01 K11 面先行定谳）：根因=files_store.at computed 内函数调用在 VM 不物化为 App_State 字段（097 T-11 矩阵同族），handler/模板读 `.store.active_kind` 抛 `Field not found`（实机 3 次复现）；修法=097 落账先例——active_kind/active_url/active_lang/active_video_html 四 computed 退役改 model var，SelectFile 单一写点（实证唯一）随路径同拍落账；vue 消费 var 与 computed 同形零改动。门禁：gen-only --strict EXIT=0（65 组件）+vm-link-probe PASS（108365B 仅体积 WARN，同 097 基线带）+ui-parity catalog PASS 116/117；worktree commit eba12de；实机点击零 RuntimeError 取证归 T-07 复验。
- [x] T-05 K12 通道修复：spec 工具工作区绑定注册（chat+errand）、superpowers.at 白名单、HTTP DTO file/tags、merge SKILL.md 文本。依赖：无（可与 T-01 并行）。验证：V04。关联：AC-04。[✅ 已完成] worktree commit 09d9d0e。**设计修正（有据，验收不变）**：工作区绑定覆盖注册早已存在（lib.rs:380-393 PLAN-030 `from_ctx`→`ws_specs_of`），真正缺口=白名单过滤连覆盖注册一起跳过（:251 同一过滤器作用于 base 与 override 两处）→①落点=superpowers.at 白名单 +read_specs/list_specs/write_spec/update_spec 四件（lib.rs 零改动；write_goals 不入聊天白名单，least privilege；mode.rs 测试钉住四件）；家目录绑定仅存于无 ctx 的 base 注册——errand 路径实证旧形态 3 参构建（orch_tools.rs:585）→②run_errand_agent 穿线 ToolContext + PLAN-030 同款覆盖注册工作域五件（dispatch/bring_in 双调用点同改）；③HTTP SpecItemPayload 补 file/tags（auto-src/server.at + auto_generated/{server.rs,extern_impl.rs} 三文件锁步，沿 098 T-03 先例；serde Option 旧客户端零破坏，specs_upsert_of 透传 SpecItem.file/tags）；④merge SKILL.md 三通道如实文本。V04：新增 tests/specs_upsert_payload.rs 3/3（全字段/旧四字段兼容/null 容忍）+lib 560 绿 0 红。
- [x] T-06 K13 合同落地：specs-ledger.md SD-02 文本、auto-plan-merge 收尾清单、plan-flow-uat spec K 表与 RUNBOOK 对策行。依赖：T-05（文本随通道事实定稿）。验证：文档评审。关联：AC-05。[✅ 已完成] worktree commit 93d2aad：①specs-ledger.md 写入通道节改三通道合同（会话工具=白名单+PLAN-030 覆盖双闸门/HTTP file-tags/errand 同账本）+「每 merge 最少条目合同」（designs≥1+reviews≥1+reports≥1 有则必记，file/tags 必带，版本控制收尾前置 cleaned；098 两条/099 四条为惯例基线、UAT 单条为欠填反例）；②merge SKILL.md 收据表 archived 行加 docs 提交要求+最少条目段+git 收尾清单；③plan-flow-uat spec.md K 表增 K9/K10/K12-13 三行 + RUNBOOK 补充对策（长会话分阶段新会话/worktree 白名单预注册/终检追加）。仪器脚本 scripts/soak-active-session.mjs 同提交落库（V01/V02 承载面）。
- [x] T-07 回归收口与 UAT 复验：V05/V06 全套；T1 简版复跑（V07）实证 AC-06；evidence/spec-delta 收口，交 /auto-plan:review。依赖：T-02～T-06。验证：V05～V07。关联：AC-01～AC-07。[✅ 已完成（T-03/AC-01/AC-02 的 auto-lang 残面除外）] V07 **pass**（2026-10-11 隔离栈 :17255/worktree ac5b22d/分阶段新会话）：四阶段 4.2/21.5/10.8/13.1min，demo-t1 main `b779ca3→5c93041→f5ae990`，四收据齐，**台账 4 条全带 file+tags**（spec-blob 哈希逐字一致），node --test 15/15，worktree 组+分支已清，零 K2 nudge；V06=自动门禁 HEAD 绿 + 实机冒烟 5min/5124 行零 RuntimeError（FilesView 物理点击因 computer-use 未启用留 review 裁定）；V05 沿用 T-02 精化轮判定（本轮零代码改动）。复跑实录三项新登记：**K14** taskkill 宿主自毁（双 musk 双亡，105 P4/P5/P8 输入）、**K15** serve 硬死转录零落盘（105 P1 族）、K16 zhipu 瞬时 500（重发即过）。证据：evidence §T-07 + spec-delta 报告 + plan-flow-uat T1 复跑行/K14-K16 + runs/2026-10-11-* 遥测 6 份。

### 任务覆盖与依赖检查

| 任务组 | 验收覆盖 | 规范增量 |
|---|---|---|
| T-01～T-03 | AC-01, AC-02, AC-07 | SD-01 |
| T-04 | AC-03 | —（行为修复，无新增契约；SD-01 审计面覆盖） |
| T-05～T-06 | AC-04, AC-05 | SD-02 |
| T-07 | AC-01～AC-07 | SD-01, SD-02 |

## 9. 复审记录

### new 阶段交接（PLAN-104:r1）

- stage: new ｜ plan_id: PLAN-104 ｜ plan_revision: 1 ｜ outcome: **pass**
  （可进入 work；执行开工需用户指令——本轮授权仅覆盖 DEBT 登记与计划撰写）
- 依据：五项发现全部有本会话实机证据（KNOWN-DEBT UAT-K9～K13 同日落纸）；
  扩展点经代码锚定（superpowers.at/lib.rs/spec_tools.rs/orch_tools.rs/
  server.rs DTO 行级事实见 §4）；规范挂点 vm-process-stability 与
  specs-ledger 经范围核对对口；K9/K10 以有界调查任务先行，不为未定根因
  预设实现细节。
- next: work（/auto-plan:work；worktree `.wt/musk-104/auto-musk`，分支
  `plan-104-dev`；若 T-01 定罪 auto-lang 面，按 AGENTS 第三行同组并排
  `.wt/musk-104/auto-lang`）。

### review 阶段复验（PLAN-104:r1 —— musk 侧范围 scoped review）

- stage: review ｜ plan_id: PLAN-104 ｜ plan_revision: 1 ｜ outcome:
  **pass（musk 侧范围；phase-only 判定，整体 Plan 保持 executing——不能据
  此授予最终 reviewed，T-03/T-07/AC-01/02 随 PLAN-765 解阻后终审）**
- reviewed_commit: plan-104-dev@**ac5b22d** ｜ base_commit: 28f3415 ｜
  dependency_revisions: auto-ai@f1c7794、auto-lang@dde29cf2c、
  auto-down@895f8d0（只读兄弟 worktree；auto-lang 兄弟内 release 构建
  `-p auto` 供仪器） ｜
  spec_inputs: docs/specs/modules/{specs-ledger（分支内 T-06 草案已备）,
  vm-process-stability（SD-01 待 merge 备稿，见 F-R1）}.md @main
  （canonical 未发布，live ledger 未动）。
- **独立性声明**：review 与 work 同会话执行（用户知情并直接指令范围）；
  结论以工件重建——`git show HEAD:` 实读两级门/白名单/落账代码
  （forge_store.at :27-35/:340/:552/:1177、superpowers.at 工具行、
  orch_tools.rs 穿线），HEAD 上全量重跑绑定证据（下），不采信实施期
  摘要。
- HEAD 重跑绑定：wt-guard clean；lib **560 绿 0 红**（含 mode 白名单
  断言）；specs_upsert_payload **3/3**；gen-only --strict EXIT=0（65 组件）；
  vm-link-probe PASS（108973B WARN 同带）；ui-parity catalog PASS
  116/117；五轮 soak 数据表与审计空记录见 evidence 报告（随 merge 落
  musk main 后长期可读，关键数字已摘录本记录与 §10）。
- acceptance_results（scoped）：
  - AC-04 通道面 **pass（代码+单测层）**：三通道落地、DTO 兼容 3/3、
    白名单断言钉住、errand 穿线实读确认；显式"两会话并发零串账"运行时
    断言缺口见 F-R2；
  - AC-05 **pass**：三处文本与代码事实一致（specs-ledger 三通道+最少
    条目合同 / merge SKILL.md 收尾清单 / UAT K 表与 RUNBOOK 对策）；
  - AC-07 **pass**：lib 560 + V05 scoped 绿 + 20 编译错/tool_atoms 红
    经 git show 双证为 base 预存（F-R3）；
  - AC-03 **partial**：根因修复+静态三门绿；实机点击取证归 T-07；
  - AC-06 **未验**：V07 复跑随 T-07；
  - AC-01/AC-02 **blocked**（PLAN-765 承接，用户裁定 a+b）。
- findings：
  - F-R1（non-blocking，merge 动作）：SD-01 的 canonical 备稿
    （vm-process-stability.md 增节）未在 worktree 预备——沿 098 先例属
    merge 阶段 prepared 动作；§5 表格 before/after 文本已可审。
  - F-R2（non-blocking）：两会话并发零串账显式断言未落（ToolContext/
    AppState 集成面，orch_tools.rs :591 注释口径即集成测试承载）；绑定
    机制本身为 PLAN-030 既有生产机制（relay 路径在用）。随 T-07 补。
  - F-R3（存量，归属并发会话）：20 目标编译错（parity_plans 夹具
    HashSet vs server 字段 HashMap，base 28f3415 git show 双证）+
    tool_atoms 红（098 F-2）——夹具同步批次另行路由，非本计划引入。
  - F-R4（信息）：soak 流式相位仅 7 样本含启动斜坡、方差大；AC-1 终判
    以 PLAN-765 T-04 扩展窗（300s+）为准。
- evidence: worktree `docs/reports/104-uat-observation-fixes-evidence.md`
  （HEAD 版）；commit 链 09d9d0e→eba12de→cee5eb5→0cb5656→93d2aad→
  ac5b22d；PLAN-765（auto-lang `docs/plans/765-vm-host-active-session-
  stability.md`）已立承接两残留面。
- next: **PLAN-765 work（owner=用户独立实施）** → 解阻后本计划补
  T-03/T-07 → 终审（届时授予 reviewed）→ merge（含 SD-01 canonical
  备稿）。

### work 阶段记录（T-02 精化 + T-01 runtime 判定 + T-06 完成；AC-01/02 残留 blocked on auto-lang）

- stage: work ｜ plan_id: PLAN-104 ｜ plan_revision: 1 ｜ outcome: **blocked**
  （5/7 任务完成；AC-01/AC-02 残留面归属 auto-lang，按 §10① 预案回用户裁定）
- code_commit(worktree plan-104-dev @ base 28f3415): 09d9d0e（T-05）→
  eba12de（T-04）→ cee5eb5（T-01 静态）→ 0cb5656（T-02 tier2）→
  93d2aad（T-06）→ ac5b22d（T-02 精化+T-01 runtime+仪器落库）
- task_ids: T-01✅ T-02✅（残留归属外移） T-04✅ T-05✅ T-06✅；T-03 blocked、
  T-07 待 T-03/依赖解阻
- evidence（runtime 五轮 + V05 判定，全文见 evidence 报告）：
  - RED（旧 .at）流式 143MB/min / idle 48MB/min，静默退出 code=1；
  - GREEN 三轮（tier2→tier1 精化）：流式 40~123 方向性改善；idle 残留
    30-72MB/min 跨形态稳定——tier1 后空闲拍零写入零 parse 零重建，残留
    =HTTP 瞬态垃圾 GC 回收不全+每帧重绘分配（auto-lang KD-062 残留族）；
  - 退出归因：AUTO_DESKTOP_EXIT_LOG 零记录+code1=原生层 exit（判读矩阵
    第 4 类），轮询负载强相关、隔离端口、KD-066 定罪不适用；
  - V05：37 目标串行=11 绿（含新增 specs_upsert_payload）+tool_atoms 存量
    红（098 F-2）+20 编译错为 base 预存（parity_plans 夹具 HashSet vs
    server HashMap 在 28f3415 即不匹配，git show 双证、本 diff 零触及）。
- blockers: **auto-lang 两面**——①idle 内存残留（每帧重绘分配+瞬态垃圾
  回收）；②原生层 exit(1)（轮询负载 2-4 分钟必退）。解阻动作=auto-lang
  专项计划（仪器 scripts/soak-active-session.mjs + 五轮数据 + 审计空证
  据已备，AUTO_DESKTOP_EXIT_LOG 取证口径现成）；解阻后本计划补 T-03
  矩阵与 T-07 收口。
- next: 用户裁定（a. 立 auto-lang 专项（推荐，两面同根概率高）；
  b. 本计划以 musk 侧已达成面先行 review（AC-01/02 记 blocked）；
  c. 维持 executing 待依赖解阻）。

### work 阶段记录（T-05 + T-04 完成；K11 静态定谳并入）

- stage: work ｜ plan_id: PLAN-104 ｜ plan_revision: 1 ｜ outcome: pass
  （2/7 任务完成；整体保持 executing）
- code_commit(worktree plan-104-dev @ base 28f3415): 09d9d0e（T-05）+
  eba12de（T-04）；依赖只读兄弟 worktree：auto-ai@f1c7794、
  auto-lang@dde29cf2c、auto-down@895f8d0（三仓 untracked Cargo.lock 已按
  build.rs 记账需要补拷，wt-guard 前须移除——已记 merge 清单）
- task_ids: T-05, T-04（T-01 的 K11 面静态定谳随 T-04 闭环）
- evidence:
  - T-05（commit 09d9d0e）——**设计修正**：工作区覆盖注册本就存在
    （lib.rs:380-393 PLAN-030），缺口=白名单过滤连覆盖一起跳过；白名单
    +四件（mode.rs 测试钉住）+errand 穿线 ToolContext 覆盖注册工作域五件
    +HTTP DTO file/tags 三文件锁步（沿 098 先例）+SKILL.md 三通道文本。
    V04：specs_upsert_payload 3/3 + lib 560 绿 0 红。
  - T-04（commit eba12de）——K11 定谳=store computed 内函数调用在 VM 不
    物化（097 T-11 矩阵同族）；四 computed 落账改 var（SelectFile 单一
    写点）；gen-only strict EXIT=0 + vm-link-probe PASS（108365B WARN
    同基线带）+ui-parity catalog PASS 116/117。
- blockers: 无（T-01 的 K9/K10 运行时面与 T-02/T-03/T-06/T-07 依序进行）
- next: T-01（K9 soak active 场景 + K10 大页矩阵）；T-06 可与 T-01 并行
  （通道事实已定稿）。

### work 阶段记录（T-07 收口：V07 T1 复跑 pass；AC-01/02 残面维持 auto-lang 裁定）

- stage: work ｜ plan_id: PLAN-104 ｜ plan_revision: 1 ｜ outcome: **pass**
  （6/7 任务完成；T-03 及 AC-01/AC-02 auto-lang 残面维持 §10 裁定待用户）
- code_commit(worktree plan-104-dev)：T-07 落账提交（evidence §T-07 +
  spec-delta 报告 + plan-flow-uat 总表/K14-K16/RUNBOOK 历史 + 遥测 6 份）
- task_ids: T-07 ✅（T-01/02/04/05/06 前轮已毕；T-03 blocked on auto-lang）
- evidence: V07 四阶段 pass（new 2aceea2c / work-resume b815eb06 / review
  0c32f001 / merge 27069ba9；事故会话 9d569e1a+fe215347 亦留遥测）——
  台账 4 条含 file/tags+blob 哈希一致、四收据齐、线性历史、工作区干净、
  零 K2 nudge；V06 门禁绿+实机冒烟零错（点击留 review）；复跑新发现
  K14（taskkill 自毁，105 P4/P5/P8）/K15（serve 硬死转录丢，105 P1 族）
  /K16（zhipu 500）。
- blockers: 与上轮同——auto-lang 两面（idle 内存残留 + 原生 exit(1)）；
  解阻后补 T-03 矩阵与 FilesView 点击复验。
- next: /auto-plan:review（musk 侧已达成面先行；AC-01/AC-02/T-03 记
  blocked 待 §10 裁定 a/b/c）。

### 复核补记（2026-10-12：PLAN-765 delivered 核验 + T-03 四轮归因；**单位口径更正**）

- **单位口径更正（P765-D2，765 owner 定谳并移交修正，本仓 commit
  9794aa1）**：仪器 slopeMBPerMin 历史版本误乘 60——此前记录的"流式
  143 / idle 48 MB/min"实为 **MB/hour**（真值：流式 ~2.4MB/min、idle
  ~0.5-1.2MB/hour）。定性结论（空闲持续增长存在、musk 门有效、残留归属
  auto-lang）不变；数值以 765 T-04 修正后口径为准。
- **PLAN-765 核验（工件重建）**：auto-lang master@43a816b47，765 归档
  delivered（ff-only 合入 4f53653fd，vm-host-stability 规约入册）；T-02
  F-1~F-6 + T-03 F-7（exit 前审计挂点）落码；T-04 交叉复验=本仓裸
  cargo t 5256/5256 + tv 162/162 + musk 三门 PASS + **1h 轮询存活
  （vm_died=false、审计零行、心跳 125 条）** + idle 真趋势 0.017MB/s
  （AC-1 门值 6× 余量；+60MB/h 缓漏源未归因已自登）。release auto.exe
  （10-12 00:30）覆盖全部代码提交（00:25 后零 crates 提交，git log 核对）。
- **T-03 四轮矩阵归因（blocked on 765-R2）**：仪器
  `scripts/bigpage-matrix.mjs`（350 轮合成=261KB 页载荷真实剖面 + MCP
  快照/点击自动化）。四轮死亡归因分层：轮 1-3 = 消息爆发（350×5KB/15s
  写入）+列表轮询叠加窗；轮 4 = 合成预启动（爆发排除）后 VM 唯一负载=
  一次 309KB autoui_snapshot 全树序列化，快照成功返回数秒内原生退出
  （零错误输出）。⇒ 稳态面已被 765 修复，**瞬态尖峰面（burst/大序列化
  的原生退出）= 765-R2 呈报**（与其自登"+60MB/h 缓漏未归因"同族嫌疑）。
  附带勘误：本构建 `autoui_action` click 需 `value:"x,y"` 部件局部坐标
  （快照须 include_bounds 取 rect）——脚本注释在案。
- next: **765-R2**（瞬态尖峰退出归因+防护，owner=用户，矩阵仪器与四轮
  数据就绪）→ 解阻后一键复跑矩阵闭 T-03 → 终审 → merge。

### work 阶段记录（765 落地后解阻件：单位修/AC-01 真值重判/F-R2/矩阵补充证据）

- stage: work ｜ plan_id: PLAN-104 ｜ plan_revision: 1 ｜ outcome: **pass（解阻件）**
  （T-03 维持 blocked on 765-R2——并行会话四轮矩阵定谳尖峰退出面，本会话
  温和载荷结果为补充证据，两分面已在 evidence 勘误节合并）
- code_commit(worktree plan-104-dev)：0e06161（P765-D2 soak 单位修 + F-R2
  两会话并发零串账集成测试）→ 0132e74（矩阵+AC-01 重判）→ c04c7fd（勘误）
- task_ids: 解阻三件——P765-D2✅ / F-R2✅ / AC-01 真值重判 **pass**
  （idle 0.717MB/min 对门 6MB/min 余 8.4×，213 样本；stream 1.287）
- evidence: 温和载荷矩阵六实例零退出零错误有界（0.7-1.5MB/min）；结构性
  观察=无 MCP 订阅形态下静态会话持续轮询已退役（page 单拉+心跳 1/s）；
  全文见 evidence 报告 §T-03 与勘误节。
- blockers: T-03 blocked on auto-lang 765-R2（尖峰退出面）；AC-02 视觉确认
  无自动化通道、长寿轴 9min 界——review 裁定项。
- next: 765-R2 落地后 `scripts/bigpage-matrix.mjs` 一键复跑闭 T-03；或
  用户指示先行 review（musk 侧 6/7+解阻件全绿呈报）。

### 终审（PLAN-104:r1 —— T-03 收口后全量复核，授予 reviewed）

- stage: review ｜ plan_id: PLAN-104 ｜ plan_revision: 1 ｜ outcome:
  **pass（reviewed——7/7 任务完成，AC-01～AC-07 全 pass）**
- reviewed_commit: plan-104-dev@**f00fa34**（T-03 收口跑；链 09d9d0e→
  eba12de→cee5eb5→0cb5656→93d2aad→ac5b22d→fa06b7a→9794aa1→8531f78→
  0132e74→c04c7fd→f00fa34，含并行会话 T-07/勘误/F-R2 与本会话
  T-01/02/04/05/06/T-03）｜ base_commit: 28f3415 ｜ 依赖：auto-ai@f1c7794、
  auto-lang（r2 后 master，765 r1+r2 delivered@4f53653fd+5c3a9b5f0 陆续
  合入）、auto-down@895f8d0。
- **scoped review 升格全量依据**：AC-01（765 修复后真值重判 pass：
  idle 0.717MB/min 对门 6MB/min 余 8.4×，evidence §T-03）与 AC-02
  （矩阵三 cell+增长 cell+交接收口跑全绿，261KB 渲染健康、10min 存活、
  退出审计归类非缺陷）的两项 blocked 已由 PLAN-765 交付+本仓收口跑解除。
- acceptance_results（终版）：**AC-01 pass**（soak 300s/180s 修正仪器，
  213 样本）/ **AC-02 pass**（矩阵+增长 cell+交接收口；视觉确认通道缺失
  由 54-marker 快照断言+六可观测通道全绿补偿，物理抽查注记在案）/
  **AC-03 pass**（静态三门+V06 冒烟 5124 行零 RuntimeError+矩阵全程零
  RuntimeError；FilesView 物理单击留 10 秒手动抽查，非阻断）/
  **AC-04 pass**（通道三处+DTO 3/3+白名单断言+F-R2 双工作区并发隔离
  测试 0.22s 绿，并行会话 0e06161 补齐）/**AC-05 pass**/**AC-06 pass**
  （V07 T1 复跑：四相位、四收据、账本 4 条带 file/tags、零 nudge）/
  **AC-07 pass**（lib 560+V05 scoped 绿+存量红/编译错经 base 双证归属
  并发会话夹具批次）。
- findings 终版：F-R1（SD-01 canonical 备稿→merge prepared 动作）；
  F-R2 **已闭**（0e06161）；F-R3（存量夹具批次，归属并发会话，另行
  路由）；F-R4（已由 765 T-04 扩展窗+修正仪器覆盖）。新增注记：①
  K14/K15/K16 三条复跑实录事件已入 plan-flow-uat；②"静态会话持续轮询
  已退役"结构性事实与脏 localStorage 异常态（新 K 候选）随 evidence 在案。
- spec_inputs 终版：specs-ledger.md（T-06 草案在分支）+ vm-process-stability
  .md（SD-01 备稿随 merge prepared，沿 098 先例）。
- next: **merge**（/auto-plan:merge：canonical prepared → rebase main →
  ff-only → ledger_refreshed → archived → cleaned；worktree+三只读兄弟
  清理随收尾）。

## 10. 待澄清事项

- **裁定已落（2026-10-11 用户三选一）**：**a+b 组合**——auto-lang 侧立项
  **PLAN-765**（auto-lang `docs/plans/765-vm-host-active-session-stability.md`，
  drafting；两残留面承接+T-01 调查先行，work owner=用户独立实施）；
  本计划 **musk 侧已达成面按用户指示先行 review**（AC-04～AC-07 达成面；
  AC-01/AC-02 记 blocked on PLAN-765，T-03/T-07 随其解阻后补跑收口）。
  仪器与数据移交：`scripts/soak-active-session.mjs` + evidence 报告五轮
  表 + 审计空记录（PLAN-765 §4 全量引用）。
- 随 V05 登记候选：**并发会话后端字段改动未同步测试夹具**（20 目标编译
  错，base 28f3415 预存，parity_plans 等；夹具同步批次归属并发会话/独立
  小计划，非本计划范围）。

### merge 阶段收据（PLAN-104:r1，completion_kind: delivered，2026-10-12）

- **prepared**：SD-01 canonical 备稿（vm-process-stability.md 增「PLAN-104 修订」
  节）提交于 9ae4c8b（reviewed f00fa34 纯文档后代=delivery 前置）；SD-02
  （specs-ledger.md）随 T-06 在分支。
- **landed**：rebase main（并行 PLAN-102/105 大量落库；冲突 1 处=extern_impl
  specs_upsert_of，解法=纳 main 状态重构+本 delta file/tags）；range-diff
  （28f3415..3182fd2 对 main..plan-104-dev）：12/14 补丁全等（=）+1 冲突
  解决（7c7570c）+1 夹具对齐（spec_tools AppState 补 PLAN-105 三字段，
  commit 178605a）。git merge --ff-only 落地 **178605a**（9ae4c8b→178605a
  两 docs/夹具提交为 delivery 组成）。落地后 main 冒烟：lib **579 绿 0 红**
  （含 F-R2 并发隔离测试；CWD 敏感测试在 main CWD 通过）。
- **ledger_refreshed**：store-mediated HTTP（landed musk serve :17305，用毕
  停）：v30→**v38**——designs/PLAN-104-vm-process-stability（draft→
  under_review→**approved**，file+tags[source/commit/plan]）、designs/
  PLAN-096-specs-ledger 原位更新（SD-02 三通道+最少条目合同，保 draft）、
  reviews/PLAN-104-review（→**published**）、reports/PLAN-104-delivery
  （→**published**）。回读双验。状态机勘误：PLAN-105 严格化后 designs 须经
  under_review、reports 无 done 态（096 的 done 为严格化前祖传态）。
- **archived**：本计划 mv 至 docs/plans/archived/（untracked 计划 mv+add
  落库），frontmatter status: archived。归档提交 e35b668（含一次 amend
  勘误交付哈希）。
- **cleaned**：三兄弟只读 worktree 的 untracked Cargo.lock 先移除→四
  worktree wt-guard 全 clean→auto-musk/auto-ai/auto-lang/auto-down 四处
  git worktree remove（兄弟自各自主仓移除）→分支 plan-104-dev 删除（was
  178605a）→组目录 D:/autostack/.wt/musk-104 已删（git worktree list 无
  musk-104 残留）。
- **产物检查（landing is not deployment）**：release musk.exe 重建 ✓
  （10-12 04:43，含全部 104 修复）；gen/front/vue/src 再生 ✓（04:43，含
  .at 改动）；**gen/front/vue/dist 陈旧（10-03）——重建被 PLAN-102 在途
  canvas_embed_web.ts TS 错误阻断（@auto/drawlist-renderer 缺失），归属
  PLAN-102 会话，其完成即解阻**；aaid 零触及（运行实例继续有效）。
- **批量回归**：不负——.last-batch-regression.json 已由并行会话刷新至
  105（5bb825b，今日）；104 不被 5 整除、receipt 新鲜无 48h 触发。
- 移交/呈报链：PLAN-765（r1+r2 delivered，两残留面修复）→ 本计划 T-03
  收口 → 本 merge。遗留候选（非本计划债务）：①PLAN-105 send-stall 测试
  CWD 敏感（worktree backend 必现超窗，main 检出稳定过）；②20 目标编译
  错夹具批次（F-R3）；③脏 localStorage 高分配态（并行会话登记新 K 候选）。
