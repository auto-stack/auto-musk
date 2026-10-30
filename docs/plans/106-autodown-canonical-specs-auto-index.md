---
plan_id: PLAN-106
status: reviewed
feature_name: AutoDown 权威 Spec 文档维护、应用自动索引与旧账本迁移
author: [agent]
created_at: 2026-10-30T00:00:00+08:00
updated_at: 2026-10-30T00:00:00+08:00
plan_revision: 5
current_step: 9
total_steps: 11
supersedes_spec_components:
  - docs/specs/00-overview.md
  - docs/specs/01-architecture.md
  - docs/specs/goals/README.md
  - docs/specs/modules/specs-ledger.md
  - docs/specs/modules/autodown-consumption.md
  - docs/specs/modules/plan-flow.md
  - docs/specs/modules/plan-execution-contract.md
  - docs/specs/modules/workspace-ui.md
  - docs/specs/modules/README.md
new_spec_components:
  - docs/specs/modules/spec-document-store.md
touched_goals:
  - goal-spec-knowledge
  - goal-frontend-parity
  - goal-security
---

# PLAN-106 — AutoDown 直接维护权威 Spec，索引由应用自动生成

## 0. 变更摘要

将 Spec 从「权威文件 + agent 手工维护中央账本」收敛为「docs/specs 文档单源 + 应用可重建索引」。AutoDown 编辑器直接读取和保存权威文档；结构化导航、关系与历史从文档元数据、正文及归档 Plan 生成。merge 不再写入、校验或等待 `.autoos/specs.json`，应用不在线、缓存不可写或旧账本损坏均不得阻断已通过验证的交付归档。

同时调整后端受控交付、旧 API/工具、产品内相位提示及 auto-plan 六项技能；对旧账本独有的知识、历史、状态和扩展字段提供有清单、可恢复、幂等的迁移。本文只创建执行合同，不实现产品、不改技能、不迁移当前数据。

## 1. 目标

- 单一权威：当前知识在 `docs/specs/` 文档中维护，历史有明确类型与出处；不存在另一份独立可写的中央 Spec 数据。
- 编辑闭环：Web 与 VM 均通过既有 AutoDown 引擎编辑实际文件，保存、重载、冲突提示和工作区隔离可验证。
- 自动索引：首次读取、保存后、外部文件/Git 更新后，应用自动得到相应索引；删除缓存后能完全重建。
- 并发边界：不同文件可独立保存；同文件过期保存显式冲突，不静默覆盖。索引延迟不参与交付成功判定。
- 流程一致：CLI agent、产品内 relay、旧受控交付恢复路径都不再要求 agent 写 JSON。
- 数据延续：旧账本每条记录与所有承载内容都有可核对的去向，不因已有 docs/specs 而直接丢弃账本。

非目标：改造 AutoDown 引擎语法、实时多人协同 CRDT、通用数据库平台、重写其他项目的业务代码、改变 Plan 的既有复审和 ff-only 门禁。不批量改写历史归档收据来伪造新流程。默认延续 `.md`，不要求既有 Spec 转换为 `.ad`。

## 2. 架构方案

`AutoDown / agent 文件工具 → docs/specs 文档（Git/worktree）→ 应用扫描及解析 → 内存/本地派生缓存 → 树、分类、关系与检索`。

`docs/plans/archived/` 中的复审/交付收据是历史索引的另一输入；不能把归档内容解释为当前模块要求。缓存可以使用 JSON，但由应用独占生成，agent 无写入职责、无最低条目配额，缓存不进入 Git 交付合同。

1. 文档元数据保存确需持久维护的信息：稳定标识、知识类型、状态、标签与依赖；正文由 AutoDown 维护。无元数据的现有 Markdown 仍可读、可编辑、可索引，不要求全库补头。稳定标识优先取显式 ID，否则按工作区内规范路径生成；重命名保留显式 ID。重复 ID 报诊断，不隐式合并或覆盖。
2. 保存使用当前文件内容哈希作为前置条件；条件比较与落盘组成同一受保护操作，不能仅靠先检查再 atomic_write。保护粒度为文件，跨进程同文件写入也不能绕过。文件不存在的创建有独立前置条件。返回保存后的哈希。
3. 索引不是写入事实源。扫描内容哈希和文档集合，识别新增、修改、删除、重命名；启动/首次访问必须能补回服务停止期间的变更。监听可作优化，不能成为唯一发现机制。读取响应标明来源指纹和新鲜度，失败时不把旧缓存伪装为当前内容。
4. 自动构建按工作区隔离，发布以完整 generation 为单位，禁止旧任务覆盖新任务；跨应用进程也须成立。构建失败保留可读原文并显示诊断；缓存缺失、损坏或不可写可回退内存重建，不要求 agent 修文件。
5. 结构化界面可以保留为文档投影，编辑必须定位源文档；无法无歧义回写的旧 item 更新接口应明确退役并提供定位信息，不得成功返回而只改旧 JSON。

## 3. 技术栈

沿用 Rust/axum、workspace registry、现有 Auto `.at` 单源及 Web/VM 编译管线、AutoDown `autodown_editor`、现有文件树与受限路径读取。新增文档持久化/索引/迁移模块按既有项目结构组织。修改后端真实源及必要的 `auto-src` 源/桥接，再按仓库生成流程更新产物，不只修 `auto_generated` 或 `gen`。

实施只在 `D:/autostack/.wt/musk-106/auto-musk/`，分支 `plan-106-dev`；Plan 共享状态继续在主检出 `docs/plans/`。不创建 worktree 内链接。依赖源默认只读；如查明必须修改外部仓库，先记录具体缺口及新增范围，再确定依赖任务，不能把未授权外部改造悄悄并入本计划。

## 4. 需求分析与背景调查

### 已获授权

用户 2026-10-30 明确要求新建 Plan：AutoDown 维护 docs/specs 权威文档、应用自动生成索引、merge 不写 specs.json、同步调整技能和数据迁移。起草轮仅授权创建计划。随后用户明确授权使用 /auto-plan-goal 自动完成执行、独立复审、有界修复、合并及收尾；该授权由所有后续阶段继承。未指定预算、执行者或最终复审者。

### 实测基线与来源

主检出 `D:/autostack/auto-musk`，分支 `main`，HEAD `25a8aace96547a380ce0f1f34324f584664e614f`；起草前 git status clean。active 目录无数字计划，已归档最高 PLAN-105，未发现同范围在途计划。

| 来源 | 已观察到的事实 / 约束 |
|---|---|
| docs/specs/00-overview.md、01-architecture.md、goals/README.md | 仍描述双落点，架构摘要还存在章节复制式 merge 旧描述，需一并纠正 |
| docs/specs/modules/specs-ledger.md | 权威在 docs/specs，但仍强制刷新六区账本、配额与双轨写入；明确无并发事务 |
| backend/crates/musk/src/specs.rs | SpecsStore 仅持路径；load / upsert / save 分离；save 原子替换不能提供整段事务 |
| src/front/specs_view.at、specs_store.at、specs_editors.at、specs_leaf.at | 文件树读原文；条目编辑仍走 specs_save_item；已有 AutoDown 编辑器可复用 |
| src/back/api.at、backend/crates/musk/src/spec_tree.rs | GET tree/text/file 已存在；缺少权威文档的带版本保存接口 |
| backend/crates/musk/src/auto_generated/extern_impl.rs、spec_tools.rs、workspace.rs | HTTP/agent 工具仍通向旧 SpecsStore，workspace 初始化仍装配 JSON store；生成桥接须追溯真实来源 |
| backend/crates/musk/src/plan_delivery.rs、plans.rs、plan_merge.rs、plan_tools.rs | refresh 写 ledger，archive 要求 ledger_refreshed；旧 merge 路径还会复制章节到 JSON |
| docs/specs/modules/plan-flow.md、plan-execution-contract.md | relay 相位输入、批准绑定、skills_hashes、受控交付与恢复必须同步兼容 |
| docs/specs/index.json | 另有已跟踪 spec_files/goals 索引；须明确生成/退役策略，不能留下第二份手填清单 |
| .agents/skills/auto-plan-{new,work,review,merge,goal,regress}/SKILL.md | 仓库技能为实际维护源；已确认用户目录中的 auto-plan-merge、auto-plan-goal 是指向本仓库的 Junction，禁止在 worktree 重建链接；需盘点其余分发副本和产品内加载路径 |
| .autoos/specs.json | 当前可解析，version=39：designs 18、tests 1、reviews 9、reports 9，goals/architecture 0，共 37 条；此为迁移调查样本，执行前重新取数与哈希，不能硬编码为全局数据规模 |

关键 SHA256（2026-10-30 读取）：
- specs-ledger.md：`1706344E14E60CD0E810C9DFA6D7E57EA56F12B6B3987C7610BFDA7FF0DC151E`
- autodown-consumption.md：`5695CBF90B753432069F32288BB65CDAE406F7182E58886BC223A1256771B338`
- specs.rs：`C124C8449CBF16FE2A856A35910F13D22651A09645ECBAC2413DBEF47CD914FD`
- .agents/skills/auto-plan-merge/SKILL.md：`75A401121CCDAC13D7BF00860C1DBB6EC7BE7A72F1B5D13532D0689F53F18E24`

## 5. 详细设计

### 文档编辑、索引和兼容合同

- 支持已有 UTF-8 Markdown（`.md`/`.markdown`）；`.ad` 复用已支持的 AutoDown 能力并在 T-01 确认读写往返。未知格式只读并明确提示。保存不得丢掉元数据、未知兼容字段或未修改正文；读取失败、写入失败、保存冲突、未保存离开均有独立可见结果。
- Git 仓库中编辑必须针对专用工作树。选择主检出时可浏览，保存拒绝并提示切换专用工作树；前端不能通过通用文件接口绕过。应用不自动提交、合并或改主分支。非 Git 工作区按选定 root 维护文档。外部 agent 仍直接通过各自工作树的正常文件工具编辑，不经过中央账本。
- 写路径局限于所选 workspace 的 docs/specs，阻止 `..`、绝对路径、工作区串用和经链接越界；创建时不存在的目标也必须验证父目录。不在本计划引入 worktree 内 junction/symlink。
- 普通文件保存成功表示源文件已持久化；索引失败单独显示“索引待更新”，不撤销已完成保存。源文件更新失败不能返回成功。
- 六类展示保留为分类能力，状态/依赖从文档元数据读出，反向关系计算产生。缺字段不凭空制造“已批准”；历史条目不覆盖当前契约。
- `docs/specs/index.json` 不再由 agent 追加列表：T-01 确认其真实消费者后，迁移消费者至自动索引；若保留兼容快照，必须应用自动生成且可删除重建，不作为另一权威或 merge 门禁。
- 旧 read/list 能力兼容映射新文档投影；write/update/status/delete 等写接口及工具逐项给出映射或明确退役错误。具备源定位及 expected hash 时才能写权威文档，禁止自动拼接目标或修改 JSON 后报成功。spec 工具模式白名单、override、errand 注册与提示需同步核验。

### 交付流程与在途恢复

- 交付仍验证 reviewed baseline、规范增量、ff-only landing、归档/清理及既有生产产物观察职责；只移除 agent 账本发布和最低条目配额。
- 新的归档前置条件以已合入的 canonical Spec/批准无 Spec 影响结论及交付证据为准。索引状态可观测，但不成为新的同步 `index_refreshed` 硬门。
- 对存在 prepared/landed/ledger_refreshed 的旧收据提供版本化解读：保留历史事实，已 landed 的计划可从 canonical 检查继续归档，无需补写 JSON；只有 ledger_refreshed 而缺真实 landing 证据不得获得放行。
- 技能更新可能影响已冻结 skills_hashes。T-06 必须给出显式兼容/重新绑定策略：区分未开始、reviewed 待 merge、landed 待 archive、已 archive 待 cleanup；不把过时 ledger 义务继续施加给恢复，也不静默豁免其他语义漂移。
- 当前计划自身首次收尾适用已审查的新流程，记录策略版本及迁移报告，不要求为了退出旧账本再写一次旧账本。

### 现有数据迁移

迁移工具由 T-05 新增；默认 dry-run，按显式 workspace/root 操作，不在启动时静默把旧账本转成现行规范。各类数据逐条给出 `旧 section/id + 源哈希 → 类型/目标路径/锚点/处理结果`。

| 原数据 | 迁移规则 |
|---|---|
| 能关联到现有权威文档，摘要与正文一致 | 关联既有文档；仅缺失且无冲突的持久元数据可补入；不覆盖正文 |
| ledger 独有的当前知识、目标、测试契约 | 在 docs/specs 下生成可评审的文档变更，保留原始出处和状态；与现有规范冲突时列入待裁定清单，不自动认定旧账本更权威 |
| reviews/reports 与其他历史条目 | 优先关联归档 Plan；独有历史落新增 docs/specs/history/legacy-ledger/（历史命名空间），不伪装现行设计、不伪造评审通过 |
| 未知字段、状态、断链、重 ID、无来源条目 | 在迁移文档或逐条保全记录中完整保留；给出诊断/待裁定分类，不能静默过滤。保留结构化原值时可用 Markdown 内 fenced JSON，非新的中央可写账本 |
| 旧账本不可解析 | 原字节保全并报告失败/人工恢复路径；现有 docs/specs 仍可读取和生成索引，merge 不因旧文件损坏阻塞 |

实施前保存字节级备份和 SHA256；备份、原始证据留本地，简明迁移报告可提交。apply 必须校验 dry-run 时的账本及目标文件指纹，抢占变化时报冲突；重复运行复用映射不重复生成。中断恢复以逐项完成记录和目标哈希核对，回滚只撤销迁移自有且未被后续修改的文件，否则明确拒绝覆盖后续编辑。

迁移产物进专用工作树并走正常评审/合回；运行中旧写入口在切换边界明确禁写，避免备份以后旧进程又追加数据。未更新客户端/旧二进制需显式停写或隔离，不承诺新进程能拦截任意旧进程直接写文件。原 JSON 不删除，不再参与正常读写；迁移完成后可只读保留为恢复材料。其他已注册工作区先盘点、各自 dry-run，未经明确选择不批量迁移。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/modules/specs-ledger.md | 退休中央写入、双轨离线投影和条目配额；定义只读兼容及可重建派生索引 | 去除共享写热点 | AC-02/04/05/06 |
| SD-02 | add | docs/specs/modules/spec-document-store.md | 新增源格式、元数据、文件 CAS/路径约束、索引新鲜度、迁移与恢复合同 | 文件单源可验证 | AC-01/02/03/06/07 |
| SD-03 | modify | docs/specs/modules/autodown-consumption.md | AutoDown 编辑实际 Spec 文件，双端保存/错误/冲突一致 | 编辑落到权威源 | AC-01/03 |
| SD-04 | modify | docs/specs/modules/plan-flow.md; docs/specs/modules/plan-execution-contract.md | 去除 ledger_refreshed 归档硬门；兼容旧收据和冻结技能版本 | 产品流程与技能一致 | AC-04/05/08 |
| SD-05 | modify | docs/specs/00-overview.md; docs/specs/01-architecture.md; docs/specs/goals/README.md; docs/specs/modules/workspace-ui.md; docs/specs/modules/README.md | 替换双写/章节复制叙述，说明单源、自动索引和历史边界；进度卡改读四检查点 | 消除相互矛盾的导航规则 | AC-02/05/08 |

## 6. 测试设计

执行者、内部检查者和最终复审者尚未指定；按既有独立复审职责执行，本计划不创建新 agent 或聊天。实施完成不代表最终复审通过。

以下为待实现的案例，非已运行证据。T-08 将新入口接到现有集成测试框架；代表性状态必须经过真实 HTTP/工具/文件及交付消费者，不能仅测纯解析 helper。

| Case | AC / owner_stage | 输入与真实入口 | 可区分错误实现的断言 | 覆盖边界 |
|---|---|---|---|---|
| V-01 | AC-01/03, work | Web/VM 的 SpecsView → AutoDown → 新文档保存 API → 同路径重读；含元数据/中文/空正文 | docs/specs 文件真实变化；元数据保留；旧 JSON 字节不变；重启读到新文档 | 两前端与实际 HTTP；格式支持以 T-01 定案为准 |
| V-02 | AC-03, work | 两客户端、两服务进程基于同 hash 保存同文件，再并行保存不同文件 | 同文件恰一成功、另一明确冲突且草稿在；不同文件都成功，目标内容正确 | 条件比较到落盘的竞态；不承诺实时协同 |
| V-03 | AC-02, work | 关闭服务后 Git/文件工具增改删重命名；启动/读取索引；删/损坏缓存再读 | 集合与内容指纹匹配实际源；删除项消失；ID 规则成立；旧任务不覆盖新代 | 首次访问、保存失效、离线修改、跨进程发布 |
| V-04 | AC-03/07, work | 遍历/绝对路径、另一个 workspace、主检出、链接越界目标和不存在文件父目录 | 返回明确错误、目标字节不变；合法 worktree 与非 Git root 对照可保存 | 真实受限写路径，不能仅字符串匹配 |
| V-05 | AC-04/08, work | plan_delivery_contract 临时 Git 仓库，应用停机/无缓存/旧 JSON 损坏/缓存只读；旧收据恢复 | landing/归档可完成且无 ledger 写；没有 landing 的伪造旧 checkpoint 仍拒绝 | prepare/land/archive/cleanup，新旧 checkpoint 组合 |
| V-06 | AC-06, work | 迁移 dry-run/apply/再次 apply：已有权威、ledger-only、历史、未知字段、断链、重 ID、冲突 | 每条都有唯一可追溯结果；有冲突不覆盖；原字节/hash 可核对；重试不重复 | 新建真实临时文件 fixtures；当前 37 条另做 dry-run 对账 |
| V-07 | AC-06, work | 迁移前后目标被改、apply 中断、恢复、回滚、回滚时用户已编辑、旧账本损坏 | 变化被拒绝或逐项安全恢复；回滚不覆盖用户编辑；损坏原文保全 | 指纹检查、部分应用状态与首次后续读取闭环 |
| V-08 | AC-05/07/08, work | 旧 HTTP item/transition/delete、spec 工具、errand、模式白名单、relay document 相位 | 读投影正确；写映射/明确退役；无任何成功但只写旧 JSON；实际技能加载源已更新 | 不以 rg 无匹配代替运行验证 |

定向验证基线命令（工作树中执行）：`cargo test --manifest-path backend/Cargo.toml -p musk --test parity_spec_tree`、`--test plan_delivery_contract`、`--test plan_flow_contract`、`--test parity_plans`、`--test plan105_specs_transition_honesty`。测试随 API 退役更新其行为预期，不能删掉错误/状态保护来换绿。T-08 新增有意义的 document/index/migration 集成测试并记录准确 test target 与非零用例数。前端沿用 `auto build` 及现有双端验证入口；T-01 固定实际命令、依赖与工具版本。完整回归依仓库规则，不引入无关 auto-lang 全量门。

## 7. 验收标准

- [x] AC-01（owner_stage: work）Web/VM AutoDown 编辑并保存 docs/specs 的真实文件，刷新/重启不丢内容；格式、元数据与错误交互满足 §5，V-01 通过。
- [x] AC-02（owner_stage: work）应用自动发现文档与历史变化；缓存删损/不可写时能从源读取及重建，过期和构建失败诚实可见；V-03 通过，无 agent 手填索引清单。
- [x] AC-03（owner_stage: work）同文件并发无静默覆盖，不同文件独立成功；越界与主检出写入拒绝，合法 worktree 可写，冲突保留草稿；V-02/V-04 通过。
- [x] AC-04（owner_stage: work）新旧受控交付路径在服务停止、无/坏账本、只读缓存下均不要求 ledger 写入；归档仍验证真实 canonical landing，V-05 通过。
- [x] AC-05（owner_stage: work）仓库六项技能、产品内提示/入口及实际分发加载源一致取消 agent JSON 义务和条目配额；旧收据/skills_hashes 策略可运行，V-08 通过。
- [x] AC-06（owner_stage: work）迁移清单覆盖所有旧条目及扩展字段，dry-run 不写源，apply 幂等、冲突不覆盖、部分失败可恢复且回滚不毁后续编辑；V-06/V-07 通过；当前工作区真实数据 dry-run 有逐条去向报告。
- [x] AC-07（owner_stage: work）旧查询、写 API/工具/分类视图无双事实源、无假成功；工作区隔离；V-04/V-08 通过，兼容及退役清单完整。
- [x] AC-08（owner_stage: review）独立复审核实 SD-01…05、AC-01…07 与真实消费者路径，新旧恢复及迁移风险已覆盖，绑定 reviewed_commit/依赖/冻结规范增量；不得以内部检查替代最终复审。
- [ ] AC-09（owner_stage: merge）按新合同线性合回并归档；真实迁移变更须单独可审阅后合入，账本独有内容未丢失；记录归档路径、技能实际生效来源、生产产物状态、工作树清理和缓存非阻断状态，不新增 ledger 刷新义务。

## 8. 执行步骤

所有任务初始未执行；total_steps=11。下一步为 T-01，依赖就绪，无用户必须先回答的正确性问题。

| Task / owner_stage | 依赖 | 路径/符号与工作结果 | 关联 AC / 关闭验证 |
|---|---|---|---|
| T-01 / work | 无 | 确认本计划唯一 worktree/分支和新基线；有限调查 spec_tree/specs_store/spec_tools、生成源、docs/specs/index.json 消费者、技能加载/分发、旧收据与当前数据；在 §9 固定源格式/元数据最小字段、API、索引失效策略、支持边界和双端构建命令。机制待选而行为已冻结，不得扩大目标 | AC-01…07；形成有实际路径的决策表/调用图，未决技术事实须在依赖任务前闭合 |
| T-02 / work | T-01 | backend/crates/musk/src/spec_tree.rs + 新增 spec_documents.rs（拟定名）、src/back/api.at 及真实路由源；实现受限的按文件条件保存、哈希返回、主检出保护和持久元数据读取 | AC-01/03/07；V-01/02/04 的后端入口先通过 |
| T-03 / work | T-01,T-02 | specs.rs/workspace.rs 与新增 spec_index.rs（拟定名）；统一扫描/派生关系和历史/失效/完整代发布；明确替换 docs/specs/index.json 消费；缓存成为可丢弃应用数据 | AC-02/07；V-03，删除所有缓存后的首次读取仍正确 |
| T-04 / work | T-02,T-03 | src/front/specs_view.at、specs_store.at、specs_editors.at、specs_leaf.at 与必要类型/端口；默认文档导航、源文档 AutoDown 编辑、冲突草稿/未保存提示、结构化投影跳转源 | AC-01/03/07；Web/VM V-01/02/04，auto build 与双端实际保存重载 |
| T-05 / work | T-02,T-03 | 新增 spec_migration.rs 及 CLI/HTTP 入口（T-01 定案）；legacy SpecsDocument 只作输入，完成 dry-run/apply/resume/rollback 与源停写切换；在本计划工作树生成当前工作区可评审的迁移文档/映射及简明报告 | AC-06/07；V-06/07；每条输入均有处理记录，冲突不猜测解决，无未授权其他工作区批量写入 |
| T-06 / work | T-01,T-03 | plan_delivery.rs、plans.rs、plan_merge.rs、plan_tools.rs、relay/plan_contract.rs、plan_control.rs、plan_flow.rs、spec_tools.rs、plan_progress_helpers.at、scripts/demo 消费合同、真实 auto-src/桥接；去掉 ledger 门、阻断旧双写、兼容旧 checkpoint/冻结技能绑定，完成旧工具映射或退役 | AC-04/05/07；V-05/08，旧 JSON 不可用仍可交付且证据不足仍拒绝 |
| T-07 / work | T-03,T-05,T-06 | .agents/skills/auto-plan-{new,work,review,merge,goal,regress}/SKILL.md 及相关 references、产品内分发内容；同步 SD-01…05 的 reviewed 草案及索引策略；验证用户目录链接指向的实际源和其他副本，不在 worktree 建链接 | AC-05/08；内容交叉核对与 V-08 实际加载，旧规则仅在清楚标识的历史说明保留 |
| T-08 / work | T-04…T-07 | backend/crates/musk/tests 中扩展上述真实 test targets，新增 document/index/migration 集成测试；双端验证、当前 37 条重新盘点后的 dry-run 对账与迁移变更检查；收集 V-01…08 证据和来源指纹 | AC-01…07；定向测试非零且通过，失败输入的拒绝与合法对照同时成立，未执行项目不可标 pass |
| T-09 / review | T-08 | 在同一实施 worktree 独立审代码/规范增量/技能及迁移产物；复验风险案例、兼容入口和实际数据去向，绑定 delivery 前基线 | AC-08；结论 pass 或按 finding 回 work，未来记录不得提前勾选 |
| T-10 / merge | T-09 pass | 应用已审查规范增量与已核对迁移变更，守卫/rebase/range-diff/ff-only，确认 main 良好；更新共享 Plan 并提交归档收据；索引不作为交付门 | AC-09；delivery_commit 与 canonical/迁移报告在主分支，原 JSON 无新增业务写入 |
| T-11 / merge | T-10 | 核验归档状态、实际技能来源、工作树/分支/中间产物清理，执行既有生产产物新鲜度观察/更新职责；未解决迁移冲突保留明确状态，不伪称全量完成 | AC-09；最终 Git/路径/收据一致，无残留 ledger 配额门 |

任务进度：
- [x] T-01（owner_stage: work）基线与有限调查决策闭合。
- [x] T-02（owner_stage: work）权威文档保存。
- [x] T-03（owner_stage: work）自动索引与重建。
- [x] T-04（owner_stage: work）双端 AutoDown 编辑闭环。
- [x] T-05（owner_stage: work）安全迁移及实际数据对账。
- [x] T-06（owner_stage: work）交付/旧 API/工具与恢复适配。
- [x] T-07（owner_stage: work）技能、提示、规范及分发生效核对。
- [x] T-08（owner_stage: work）风险验证及实施证据齐备。
- [x] T-09（owner_stage: review）独立最终复审。
- [ ] T-10（owner_stage: merge）线性合回与归档。
- [ ] T-11（owner_stage: merge）清理与生产产物观察。

上表为每个 Task 的唯一设计合同；checklist 仅记录执行状态，不建立另一份任务定义。代码/技能/迁移文档在 worktree，只有 Plan 本身共享记账在主检出；不因本计划修改 merge 技能而绕过尚未完成的 review。

## 9. 复审记录

### PLAN-106:r1 — 起草交接（2026-10-30）

- stage: new
- outcome: pass
- next: work / T-01
- 本轮只读调查并创建此草稿；未执行实现、测试、迁移、技能更新、合并或部署。
- 完成已知入口与规范矛盾定位；合同覆盖 AC-01…09、SD-01…05、T-01…11、V-01…08。
- 用户授权的目标已明确；技术策略由 T-01 在行为约束内定案，不新增审批门。status 保持 drafting，后续执行按实际入口的授权与状态规则启动。
- 证据限于 §4 的来源/哈希/当前账本聚合；未来 fixtures 和验证命令不是通过证据。

### PLAN-106:r1 — goal 启动与 work 入口（2026-10-30）

- stage: goal | round: 1 | from: new | to: work | status: executing | outcome: pass | next: work
- 授权：用户明确请求全链自动完成；不重复询问已授权的执行/复审/合并。
- 主检出代码干净，仅本 Plan 草稿未跟踪；未发现本工作区 relay 驱动本计划。无关 os-v050 worktree 为已有 prunable 残留，不接管。
- worktree: D:/autostack/.wt/musk-106/auto-musk；branch: plan-106-dev；base_commit: 25a8aace96547a380ce0f1f34324f584664e614f。已核验实际 root/branch。
- 当前责任：work / T-01；review/merge 尚未执行。独立复审按技能要求采用独立上下文、同一工作树、顺序移交，不并发写入。
### PLAN-106:r2 — 依赖兼容范围修订（2026-10-30）

- stage: new | outcome: pass | next: work；目标/AC/SD/任务 ID 不变，仅扩大已明确批准的依赖修复范围。
- 用户明确批准“允许必要的依赖兼容修复”。auto-ai `ai-config/rust-ref/src/loader.rs` 的 5 个 as_uint 调用不能编译；当前 auto-val 对应公开 API 为 to_uint，支持 Int/Uint、返回 u32。
- T-01/T-08 增含该兼容修复的 scoped 测试与消费集成；T-09 复核；合回依赖后尽快清理，不等待整体收尾。只读 auto-lang sibling 供相对路径解析，基线 28f6170bed8d8a1d3ebafc5219326a988450bb0b（默认分支 master）；auto-ai 基线 2a4d606f3cba16530d07a9bf5287529d3eabe63c（main）。
- 新 worktree：D:/autostack/.wt/musk-106/auto-ai，branch auto-musk-dev；D:/autostack/.wt/musk-106/auto-lang 为只读 detached 基线，不修改其源。
- 初次 cargo check exit=101，失败在依赖、尚未到 musk。日志 D:/autostack/plan-artifacts/plan-106-check.log；后续验证不得把本次失败计作产品通过。
### PLAN-106:r3 — T-01 决策、消费者补齐与阶段性验证（2026-10-30）

- stage: work | outcome: executing | next: T-02/T-04/T-08；并非 work pass。
- r3 为同一目标的路径/策略定案：增加实际进度卡及 demo 合同消费者（旧 ledger_refreshed 会导致假缺件），SD-05 同步 workspace-ui；AC 门槛不变。技能冻结值保留审计，document 相位仅覆盖旧账本义务，不豁免提交/依赖/批准检查。
- T-01 闭合：支持 md/markdown/ad；可选 YAML 元数据；按文件 OS 锁+expected_hash；Git 主检出只读；首次/每次读取扫描+双快照，选择纯内存索引（无共享缓存发布竞争）。AutoDown 编辑正文，原始 frontmatter 与正文分离后再原字节拼回，防止富文本序列化改坏未知元数据。未知格式只读。
- 真实宿主桥接 `src/auto_generated/extern_impl.rs` 本身是手写 host glue（无另一生成源）；新文件接口在 hw spec_tree 路由，前端 API 在 src/back/api.at。旧 combined merge 退役；refresh 为只验证 canonical 的兼容入口。
- T-03 证据：spec_documents_contract 中 automatic_index_offline_changes_identity_history_and_diagnostics 通过，覆盖离线增改删名、显式 ID、重复 ID/历史边界及坏账本不影响指纹；无持久缓存依赖。文档接口按显式 workspace get_exact，未知 ID 不回落另一工作区。
- 定向结果：doc/store 7 + HTTP spec tree 7 通过；parity_plans 5 通过；修正原有测试夹具 chat_runs 已由 HashSet 变为 HashMap 的基线漂移后，plan_delivery_contract 9、plan_flow_contract 19、退役状态入口 1 通过。测试与实现仍在变动，最终交接会绑定提交并标明重用范围。
- demo 三组定向 Node 测试 38 通过（当前四检查点 + 兼容读取旧 ledger_refreshed）；依赖 ai-config 新增真实配置解析验证 1 通过，消费 cargo check 通过。
- Web auto build 已通过一次；默认 pnpm 在生成目录制造了 junction，已将整份生成工程原子移动到 D:/autostack/plan-artifacts/plan-106/verified-web-build，未递归删除链接/目标；worktree reparse points=0。后续构建设置 npm_config_node_linker=hoisted，避免再次产生链接。构建进程栈设置 RUST_MIN_STACK=67108864，AUTO_LANG_ROOT/AUTO_UI_ROOT 显式指主依赖，未修改 auto-lang 源。
- 当前真实账本 dry-run：37 条 + envelope，source SHA256 4d73a7aadbf4e3a2c83942d024f0921e5099763b94af4172e9430463f9561367；导入到本计划工作树，重复 apply 成功，原 JSON 哈希未变。所有 18 个 designs 摘要已读取，均引用现有 canonical 或已归档计划，旧配额条款明确退休；迁移报告将逐项列实际现行规范去向，不将历史导入等同现行批准。
- 本次日志本地：D:/autostack/plan-artifacts/plan-106-{contract-tests3,contract-tests4,demo-tests2,dependency-test,front-build4}.log；完整日志/原始 manifest/备份不进 Git。当前 work-owned 尚有 UI 双端真实交互、失败/恢复补充断言和最终基线核验，未交最终复审。

### PLAN-106:r3 — 必要依赖修复的复核与提前收尾（2026-10-30）

- dependency: auto-ai；scope: ai-config unsigned Value accessor compatibility；reviewed_commit/delivery_commit: ad5cb7b4589da78930fc7b31ea896fec36a841db。
- 已从提交 diff、opt_models 调用及 auto-val::Value::to_uint 定义重建判定：5 个旧 as_uint 改当前公开 to_uint，维持 u32 到字段的取值路径。新增真实配置解析测试验证 context/max-output 与三种 cost 值；loader::tests 11 passed、0 failed；musk 集成消费已通过（同补丁、提交前后源字节相同）。
- 复核在实施会话内完成，有此独立性限制，未冒称另一 agent 的结论；本计划没有指定独立外部最终评审者。auto-musk 整体最终复审尚未开始。
- auto-ai worktree clean → wt-guard clean → rebase main（无重写）→ wt-guard clean → main ff-only 落 ad5cb7b → fresh guard clean → worktree/auto-musk-dev 删除，主检出 tracked diff=0；既有 untracked 日志原样保留。
- 只读 auto-lang detached sibling 28f6170be 未改源码，确认干净且 guard clean 后已移除。后续显式 AUTO_LANG_ROOT=D:/autostack/auto-lang；auto-ai 依赖消费其已验证 main，无悬挂依赖工作树。
### PLAN-106:r4 — 原生 AutoDown 装配探针与依赖范围补齐（2026-10-30）

- stage: new | outcome: pass | next: work；AC/目标/SD 不变，补入用户已批准的必要依赖兼容修复。
- Web 真实链已通过 4 断言（保存+元数据字节保留/重载/409 保草稿/未保存切文件拦截）；双真实后端进程验证 same-file=[409,200]、independent=[200,200]、两进程索引指纹相同且不创建 specs.json。
- VM 实机已选中文档、编辑、保存并回读源，元数据仍在；但 autoui_editor_state 显式报告 requires autodown + code-editor features。不能把该降级运行声称原生 AutoDown 验收通过。
- 原因已定位：auto-ui/crates/auto-ui-tools/Cargo.toml 依赖 auto-ui-runtime 只用默认 ui-iced（含 code-editor，不含 autodown），实际 View::AutodownEditor 降级；现有完整核心在 auto-down，非本计划新增引擎。
- T-01/T-04/T-08/T-09 的依赖范围新增 auto-ui：仅为工具运行闭包启用既有 runtime autodown feature，并验证完整核心/真实 UI。worktree D:/autostack/.wt/musk-106/auto-ui，branch auto-musk-dev，base c11015c1ef3707a232939afe331dbd5941d6291b，default master。auto-down 只读消费现有核心，暂未修改。
- auto-ui 主检出已有设计/计划共享状态改动，无代码 WIP；原样保留，不纳入此依赖修复。功能 check 探针进行中；如查明需要新的引擎语义或 SDK 发布承诺，须重新界定，不默认扩大。
### PLAN-106:r5 — 原生正文保护回归与执行入口修订（2026-10-30）

- stage: new | outcome: pass | next: work；目标、AC 阈值、仓库授权不变。规范增量 SD-05 补既有 modules/README 导航和模块职责描述，避免当前索引说明仍称 ledger/章节合并。
- T-04/T-08/T-09 补 auto-ui-runtime/src/ui/autodown_editor/core.rs 普通光标移动的正文保护，属于已批准依赖兼容范围：新启用真实核心后，零位移拖选残留块内 selection anchor，End 再输入会把后缀误当选区替换。直接公开核心接口回放：普通打字控制通过，End 反例实测 Na!，期望 Native baseline 106!。
- 永久回归入口（新增）auto-ui/crates/auto-ui-runtime/tests/spec_document_motion.rs：真实 render_frame → MousePressed/Dragged/Released → KeyPressed → emit_document；覆盖 End 与 Home/Left/Right 的普通移动正文保全，另有 Shift+End 的有意选区控制，不能以降低正文断言换绿。关联 AC-01/AC-03，owner_stage: work。
- 最初 whole-lib test 编译被无关旧 autodown_engine 测试自由函数名及静态 include 兄弟路径阻断；未计为行为 red，也未保留无关修补。改用真实公共 API 集成入口；该入口已执行到上述因果明确的 red，日志 native-motion-public-red2.log。
- 修复仅在普通水平 motion 分支清块内 selection；不改解析语法、保留 shift 选区。待针对性 green、真实原生按键→保存→重载/冲突闭环及复核后才能完成 T-04/T-08。
- 原生编辑器显式稳定 key 防止 dirty 提示插入时换编辑实例；VM 非 2xx 的 error 字段与 Web catch 统一转为保草稿的可读失败提示。公开测试脚本 scripts/spec-document-{ui,processes,vm}.mjs 为真正执行入口，不能用 API 直写代替编辑壳。
### PLAN-106:r5 — 原生依赖修复完成与消费验证（2026-10-30）

- dependency review_scope: phase；reviewed_commit/delivery_commit: auto-ui 916d9190c4da88dcc34d95cfc74fe06587b8d627。仅工具默认启用现有 AutoDown、普通 motion 清残留块内选区、新公开集成回归（3 文件）。复核在实施会话内，有独立性限制，不冒称另一 agent。
- 反例 public-red2：普通打字 1 pass，End 1 fail（Na! vs Native baseline 106!）；green：4 passed，包含 Home/Left/Right 的保全与 Shift+End 有意替换控制。未改解析语法，未削弱完整正文断言。
- 真 VM AutoDown 核心通过 autoui_editor_state 可读（不再是 feature 缺失降级），真实 MousePressed/Dragged/Released + KeyPressed 触发 SpecDraft；保存原元数据、切文件重载、409 保草稿/保较新源全部通过，ui-vm-receipt.json 五项成功。T-04 完成；Web 四断言仍有效，其后仅稳定 key 与 VM 返回错误的可读文案变化，最终生成检查待闭合。
- native-ui-tool.exe 保全于 D:/autostack/plan-artifacts/plan-106/，SHA256 37315B6913D2CC25B1DC2A616620A24F1314682F55BAAAC75275E4084E4D23AF。日志 native-motion-green.log / ui-vm-final.log / native-tool-build2.log。
- 消费通过后：auto-ui clean → guard clean → rebase master（无重写）→ guard clean → ff-only 落 916d919 → 停止自有原生测试进程 → fresh guard clean → worktree/auto-musk-dev 删除。auto-ui 主检出原有 docs WIP 原样保留，crates tracked diff 为空。
- auto-lang 在执行期间从 28f6170be 前进到 b0a6c869da6d92307a0afd7a8ce0216eaaa4dd33；Git diff 仅 docs 与旧 ledger，无 crates 变化，相关代码/配置输入不变，既有代码证据可复用。auto-down 只读基线 c3464d0a0d36e4f8a7988b0c0c26448fc52361b4；auto-ai 已合回 ad5cb7b。
- 后端复验遇共享目标目录中 rmeta/rlib 依赖形式不匹配（E0463 / rlib unavailable），不是用例通过证据。为隔离构建缓存，六个指定目标在新 backend-target 中重新验证；不把该环境失败冒充产品行为 red。
- 增补正常交付合法控制：已在 reviewed_commit 中提交的 canonical 文档无需制造 dirty delta；经批准无 Spec 影响可正常交付；缺 reviewed_commit 仍拒绝，landed/prepared 的 rebase 映射仍须匹配，不能以旧 ledger 检查点替代。

### PLAN-106:r5 — work 完成 / goal round 1

- stage: work | outcome: pass | next: review | status: execution_done | current_step: 8/11；AC-01…07、T-01…08 已完成，review/merge 所有项仍待相应 owner 验证。
- committed worktree: D:/autostack/.wt/musk-106/auto-musk，plan-106-dev，implementation_commit: b03a703bbf2ceb8f1019255470e9661fed6ca20e；base_commit: 25a8aace96547a380ce0f1f34324f584664e614f。迁移独立提交 2bed232，保全 37 条原记录与 envelope，README 逐条对账，原 JSON SHA256 不变；未迁移其他工作区。
- V-01/02/04：真实 Web 六项、VM 六项通过，含原生核心按键、完整正文/元数据保全、源重载、冲突保草稿、未保存页面切换保护；Web 结构化条目编辑实际打开对应 canonical 源。两个真实 HTTP 服务进程：同文件 200/409、不同文件 200/200，索引完整指纹相同，无 specs.json。
- V-03/06/07/08：spec_documents_contract 11 passed（含 worker），覆盖 OS 跨进程锁、合法 worktree/主检出保护、链接父目录拒绝、离线变更/重命名/历史身份、CRLF/BOM、真实工具退役、完整迁移/恢复/回滚/备份拒错。parity_spec_tree 7、parity_plans 5、transition_honesty 1、plan_flow_contract 19 passed。
- V-05：plan_delivery_contract 最终 12 passed：已提交 canonical delta、批准无 Spec 影响、缺 reviewed_commit 拒绝、实际 rebase/ff/归档、坏账本不影响交付、无 landing 的旧 checkpoint 不放行。演示消费者 Node 38 passed。首次失败的 fixture 缩进/Windows 路径及无增量错误文案已修复，保留失败日志，未降低行为断言。
- 最终完整 auto build 在专用外部构建副本 full-web-source 执行，退出 0，vue-tsc+vite 成功；与 worktree 对应 source 相同。gen-only 对多 widget 文件/扩展 helper 的生成不完整，失败 TS 日志不作通过证据；用完整构建重新生成，没有交付手改 Vue。首轮新 source helper 使用 file 参数触发既有 file 命名空间分派，改为 s 后完整构建与真实源跳转通过。节点依赖链接只存在外部构建目录，实施 worktree reparse count=0。
- 六技能在隔离 serve 的实际 skills 目录均与仓库源字节一致；Codex 用户技能为指向主仓源的既有 Junction。产品全局六技能待已复审合入后同步并核验（merge-owned）。文档/迁移报告 authored 相对链接全通过。
- dependencies: auto-ai ad5cb7b4589da78930fc7b31ea896fec36a841db；auto-ui 916d9190c4da88dcc34d95cfc74fe06587b8d627（两项必要修复已合回/清理）；auto-down c3464d0a0d36e4f8a7988b0c0c26448fc52361b4；auto-lang 5243568ae65dcae79104a71172d574bdcd8369f0，较上次仅 docs/Plan 变更，crates/配置输入未漂移。
- evidence（local-only；首个 aggregate 中两目标曾失败，分别以 documents-final/delivery-final 的全部通过闭合，其余四目标输入未变）：
  - D:\autostack\plan-artifacts\plan-106-backend-isolated-tests.log SHA256 2674ee1131cfbde66022de8a730bacb3e57d40d1486dacbac7410bb44a98e54a
  - D:\autostack\plan-artifacts\plan-106-documents-final.log SHA256 c391c964a1b1cc9ce1582a8cd6b9b0f6119cce92908b2de4f60b560b295368a9
  - D:\autostack\plan-artifacts\plan-106-delivery-final.log SHA256 4fdf9520d505b074ad4ec5acaa89de2bca244f9dca997585e63711c7ab2d0451
  - D:\autostack\plan-artifacts\plan-106-demo-final.log SHA256 9c3ad537a15beb09c0f71f69767cb54349def95974be54ef7d16eac26781fc90
  - D:\autostack\plan-artifacts\plan-106-ui-web-final2.log SHA256 f58c616ac09fb0365715f12e2d23fd1533588d222c0115d3fafcc3ea7f873c48
  - D:\autostack\plan-artifacts\plan-106-ui-vm-final2.log SHA256 dc17da6b4b9aa77d23206cc5f6c227e2654fe1f87ed1605c940bc89c0e4012f1
  - D:\autostack\plan-artifacts\plan-106-http-process-final2.log SHA256 8346b46f4fd7c39951aa675ce1db236ef53f35416f89da6b82fd4b654de4e255
  - D:\autostack\plan-artifacts\plan-106-web-full-final2.log SHA256 7b15c9b8da1ef52673fc9e2144bf4e9a0d1749f5d3a71e28e6126ea32de1f97c
- merge-owned 尚待：AC-08 最终复审；AC-09 规范落地主分支、共享 Plan 归档提交、技能生效、产物观察及 guard/清理。无额外索引或 JSON 交付门。


### PLAN-106:r5 — final review round 2 / needs_fix

- stage: review | review_scope: final | outcome: needs_fix | next: work；reviewed candidate b03a703bbf2ceb8f1019255470e9661fed6ca20e，base 25a8aac，依赖沿用 work 收据。复审者为本实施会话，未创建其他 agent；按 review 技能从提交/实际调用路径重建结论，独立性有限。
- F106-R1（P1，AC-03/07，T-02/T-08）：旧 tree/text/file 用 registry.get 回退默认工作区；raw file 仍直接 fs::read。新 document/index 虽正确，旧消费者可越过工作区选择/链接隔离。对现有约束的实现缺口；修复范围为旧读取路径精确 workspace、链接父目录/树节点保护。closing oracle：真实旧三入口 unknown workspace 404，链接源拒绝/不枚举，合法源控制继续通过。
- F106-R2（P1，AC-04，T-06/T-08）：archive 在已归档分支先返回，并可补写 delivered 收据，未核验 prepared/landed/canonical，违反实际 landing 门。需将验证放到重入前，并正确读取已归档合同。closing oracle：已经 archived 但无 landing 也拒绝，合法归档重复调用仍幂等。
- 其余 AC-01/02/03/05/06 与 SD-01…05 审查无新增问题；真实双端六项、38 Node、迁移/链接/authored 文档对账证据保持。仅上述路径受影响，不附加全局回归门、不扩大目标/授权/合同。repair cycle: 1/3。


### PLAN-106:r5 — work repair / goal round 3

- stage: work | outcome: pass | next: review | repair cycle: 1/3；implementation_commit: 75be939d6a682c569561f5b22cb867776765e2b5，干净 worktree；AC-04/07、T-02/06/08 重新完成，current_step 8/11。
- F106-R1 已闭合：真实旧服务反例 unknown workspace 的 tree/text/file 均 200；修复后新旧五类入口 unknown workspace 404。旧 raw file 使用受限路径解析，树忽略链接；真实 Windows junction 案例要求 text 按既有协议 400、file/document 403，均不得泄露外部正文，tree 不枚举链接。合法读取对照通过。
- F106-R2 已闭合：归档重入前先核验 prepared/landed/真实提交和 canonical 源；归档合同从实际 archived 路径读取。新增 archived-without-landing 反例拒绝、合法 archive 再入幂等通过。
- affected checks：parity_spec_tree 9 passed；plan_delivery_contract 13 passed。日志 D:\autostack\plan-artifacts\plan-106-review-repair1-final2.log，SHA256 870971232f9b43f85dcd28428f6e767c3531dd867a8a03306bf48c15a5c8784c，exit 0。原先 linked-text 测试误要求 403，实际既有 read_text_confined 契约是 400；改为固定旧协议，仍严格断言拒绝及无内容泄露，未降低隔离要求。
- 未改 editor/index/migration/skills 或依赖源码，既有双端各六项、迁移11、relay19、Node38、parity_plans5、transition1 的证据输入未受影响，按复审技能复用，不重复全局测试。真实迁移逐项 fenced JSON 与原 ledger 的 37 条记录 JSON value 全等，原源指纹不变。


### PLAN-106:r5 — final re-review / goal round 4

- stage: review | review_scope: final | outcome: pass | next: merge | reviewed_commit: 75be939d6a682c569561f5b22cb867776765e2b5 | base_commit: 25a8aace96547a380ce0f1f34324f584664e614f | branch: plan-106-dev | worktree_path: D:/autostack/.wt/musk-106/auto-musk。复审身份/限制：同一实施会话，未委派独立 agent；没有指定另一最终复审者。采用已提交基线、逐项合同/实际代码核对及独立挑选的边界回放，按技能允许的同会话复审记录，不声称独立会话。
- F106-R1/R2 closure: pass。复审冻结提交后重新运行 legacy_read_routes 两项（2 passed）、archived_status_alone 一项（1 passed），exit 0；合法源/真实 Windows 链接/未知 workspace/合法 archive 重入和伪造状态均区分。修复范围只含 2 源文件与对应测试，T-09/AC-08 完成。
- acceptance map：AC-01→T-02/04/08→AutoDown Web6/VM6；AC-02→T-03/08→index 离线/身份/诊断与两 HTTP 进程指纹；AC-03→T-02/04/08→文件锁线程/双进程、HTTP409、主检出/链接拒写与双端草稿；AC-04→T-06/08→delivery13 与伪归档反例；AC-05→T-06/07→relay19、实际 serve 六技能字节核验；AC-06→T-05/08→migration 正/反例、37 原记录逐条全等与备份；AC-07→T-02/03/06/08→HTTP9、旧工具/写入口410/errand 默认绑定代码、读取投影。全部 pass。
- SD-01…05、10 个 canonical 目标及历史迁移独立提交均核对：源文档单一权威、内存自动索引无发布竞态、原 YAML 前缀保全、当前规范与历史隔离、四检查点与旧冻结技能策略一致，删除手填 index.json 消费者要求。已改 authored 文档相对链接无断链，无未批准兼容/阈值变更。
- frozen Spec delta SHA256（git diff base..reviewed -- docs/specs）：ed44c05f8b33bca0453e04939c7be69863dee2ec10abac6bf1195fae8b647476；逐路径已提交 blob SHA256 存 D:\autostack\plan-artifacts\plan-106\review-source-facts.json，SHA256 80ca2c8ed21de300f00e8c4374aef471da377a000419e84e443fd8cf4b5b69b2。区分 Git LF 规范化后的 blob 与工作树原 CRLF 字节，landing 比对相同 Git blob；并非交付索引/缓存。
- dependencies 与上一工作收据一致：auto-lang 5243568ae65dcae79104a71172d574bdcd8369f0；auto-ai ad5cb7b4589da78930fc7b31ea896fec36a841db；auto-ui 916d9190c4da88dcc34d95cfc74fe06587b8d627；auto-down c3464d0a0d36e4f8a7988b0c0c26448fc52361b4。复审末 HEAD 与 worktree 清洁状态均未漂移。
- evidence reuse：未改动的前端、索引、迁移、技能及依赖输入保持不变，保留已执行证据，不将所有权变化视为重复全局测试的理由。目标后端合计 58 项（9+13+19+11+5+1，含 worker），Node38、两端各6、跨真实 HTTP 进程与依赖11/4项通过；修复后两个受影响目标完整重跑并通过。
- pending_closeout_gates：仅 AC-09/T-10/T-11，main ff-only、canonical 源身份/真实归档、全局技能同步、产物观察、guard/清理；当前仓 auto-musk 不适用 auto-lang 的全量 cargo tier/batch due 门。


### PLAN-106:r5 — merge prepared / goal round 5

- stage: merge | outcome: in_progress | checkpoint: prepared | policy: document-source-v1。承接用户整链授权和当前 r5 最终 pass；reviewed/delivery candidate 75be939d6a682c569561f5b22cb867776765e2b5，Spec delta/49 canonical+历史源 blob 指纹见 final review，源码/依赖未变。
- 主检出只有本共享 Plan 未跟踪，按 AGENTS/merge 技能的共享 Plan 例外先提交本计划收据，再在专用 worktree rebase main，并用 range-diff 核验三项开发补丁逐条等价；不把此 bookkeeping 当产品变更。
- AC-09/T-10/T-11 尚待实际落地/归档/清理，不预勾选。缓存/应用/旧 JSON 无发布动作或门禁。

## 10. 待澄清事项

无阻止计划进入后续 work 的用户决策。T-01 拥有以下技术核实：真实生成源与双端写接口能力、文档元数据往返、技能分发副本、index.json 消费者、索引扫描与失效机制、旧收据/skills_hashes 兼容细节。若必须改外部依赖或出现超出 §1 的目标变更，先记录事实并修订合同；不得以“继续用旧 JSON 写入”替代本计划验收标准。
