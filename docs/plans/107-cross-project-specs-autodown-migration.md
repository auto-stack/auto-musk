---
plan_id: PLAN-107
status: reviewed
feature_name: 跨项目旧 Spec 账本格式兼容、AutoDown 文档迁移与去重回滚
author: [agent]
created_at: 2026-10-30T13:49:01+08:00
updated_at: 2026-10-30T13:49:01+08:00
plan_revision: 3
current_step: 11
total_steps: 12
supersedes_spec_components:
  - docs/specs/modules/spec-document-store.md
  - docs/specs/modules/README.md
  - docs/specs/history/legacy-ledger/README.md
new_spec_components:
  - docs/specs/modules/spec-batch-migration.md
touched_goals:
  - goal-spec-knowledge
  - goal-security
---

# PLAN-107 — 跨项目 specs.json 拆分为 AutoDown 文档

## 0. 变更摘要

承接 PLAN-106 的文档单源合同，补齐旧账本结构兼容，迁移盘点范围内尚未处理的数据，并完成跨来源、跨项目及子模块副本的可追溯对账和安全回滚。AutoDown 直接使用 UTF-8 Markdown，无须把 .md 批量改名为 .ad；应用自动索引，不新增 agent 手填 JSON 或索引发布门。

本计划已于2026-10-31按用户要求进入迁移执行；实施在独立工作树进行，先冻结来源/归属，再生成和复审文档。原起草记录保留在§9。

## 1. 目标

- 支持观测到的 section 对象数组及 section→items 字典格式，保留顶层、区块、条目、未知字段及原始字节备份；合法输入不静默漏项，非法输入明确拒绝。
- 所有待迁移记录都有明确结果：关联已有当前规范、经内容核对提炼新规范、保全历史或显式待裁定；每条原记录都能还原，不能只导出正文而丢状态/依赖/扩展数据。
- 将原 12 个尚未迁移主项目及新增 auto-web/auto-ui/auto-man 三仓与 auto-musk/backend 旧账本迁至各项目根 docs/specs；auto-musk 根账本的 37 条作为已完成迁移对照，验证后跳过，不重复导入。
- 对 auto-os/apps 六份同项目副本建立只读别名与对应关系，仅迁移权威独立仓一次；防止不同仓同 ID 或不同来源同 ID 互相覆盖。
- 提供可审阅 dry-run、逐项 apply、幂等重入、中断恢复与回滚；落地主分支后的撤销走可追溯的反向提交，不破坏后续用户编辑。
- 迁移切换后，范围内仓库的活跃操作规约不再要求 merge 写 specs.json；当前规范与历史分开，自动索引不作为交付门。

非目标：迁移演示/临时/构建缓存目录；把其他在途计划的工作树当新项目导入；修改各项目业务/编译器实现；同步所有子模块版本、推送远程或部署桌面；删除原 JSON；自动重新批准旧状态；承诺跨仓原子提交或无限规模输入。

## 2. 架构方案

`显式项目/来源登记 → 原字节快照与仓库身份 → 格式适配（保留来源位置）→ 逐条对账及文档变更预览 → worktree 条件写入 → AutoDown/自动索引核验 → 逐仓复审、ff-only 落地 → 汇总交付收据`。

源对象必须区分 source_project、source_ledger_relative_path、source_sha256、source_schema、section_key/原位置、item_position、legacy_id。目标项目根与历史引用解析根分开绑定；不以当前进程工作目录猜测归属。

每项目有独立 journal、备份、before/after 指纹及完成状态，批次只聚合这些事实。局部失败不伪装全量成功；已完成项目可保留、继续其余项目或按收据安全撤销。索引/应用不可用不妨碍源数据备份及 Git 交付，实际阅读验证仍须完成后才可声称相应 AC 通过。

## 3. 技术栈与修改范围

工具继续位于 auto-musk：backend/crates/musk/src/spec_migration.rs、main.rs；复用 spec_documents.rs 的路径保护/文件锁/CAS，spec_index.rs 的派生索引。现有 spec_documents_contract.rs 扩展，拟新增跨格式/跨来源/批次真实 CLI 集成 target spec_migration_batch_contract.rs（新增路径）；如需要批次操作入口，优先扩展现有 spec-migrate，不造第二套不兼容工具。

仅 auto-musk 修改迁移实现/测试。批次入口为 scripts/spec-migration-batch.py，调用同一 Rust spec-migrate 引擎，不另实现导入格式。新增 main.rs 参数 --source-key；新日志/批次 state 均为工具生成的本地恢复材料。其余允许仓库：auto-blog、auto-charts、auto-database、auto-down、auto-edit、auto-lang、auto-launcher、auto-os、auto-os-config、auto-paint、auto-reader、auto-term、auto-web、auto-ui、auto-man；仅迁移文档、对账报告、必要的活跃知识流程规约与其上游规范源，不顺带改业务代码。

主工作树：D:/autostack/.wt/musk-107/auto-musk，分支 plan-107-dev；其他目标在同组 D:/autostack/.wt/musk-107/<repo>，对应本仓计划的 sibling 分支 auto-musk-dev。work 必须检查每仓 AGENTS/当前 owner/在途计划，禁止接管他人分支和 WIP；已完成该仓的复审/消费验证后尽快合回并清理，不悬挂到整批结束。不得在 apps 子模块检出内写文件或切分支，不建立 junction/symlink。当前用户的工作树约定优先，同时保留各仓自己的复审与文档验证门。

共享计划只写 auto-musk 主检出 docs/plans；其他仓必要的归属指针写各自工作树中的迁移报告，不分配重复的计划正文。

## 4. 需求分析与背景调查

### 已获授权

用户明确要求据盘点结果新建一个 Plan：补旧格式兼容、将各项目未迁移 specs.json 拆为 AutoDown 文档、包含 auto-musk/backend、对子模块副本去重、完整对账和回滚。原起草授权只含计划；2026-10-31用户另行要求完成迁移并扩展三仓，见§9 r2执行授权；未指定执行者、最终复审者、模型、预算或自动修复轮数。后续用户启动 work/goal 时依该合同执行，不重复询问合同已列明的例行仓库范围。

### 数据与版本快照

盘点证据：D:/autostack/plan-artifacts/specs-inventory-20261030/inventory.json（本地证据，含各源路径/SHA256/分类/失效引用）；起草时重新核验下列 13 份根账本 SHA256 均仍与清单一致。Git HEAD 仅为调查基线，T-01 必须执行前重读，不能锁死其他在途计划的正常进展。

| 项目 | 根账本记录 | 观测结构 | 起草时 Git HEAD |
|---|---:|---|---|
| auto-blog | 10 | array-of-sections | `f992419e0e25558313519036c7c5b5fe0708965e` |
| auto-charts | 5 | array-of-sections | `77b6f82aa43aef807ee7123c7a2eaed37f47f715` |
| auto-database | 4 | array-of-sections | `bb3a0b30ded5b3e5db4dada60c9ec033e980b5dd` |
| auto-down | 251 | map-of-item-arrays | `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4` |
| auto-edit | 30 | array-of-sections | `acf71a0541e0f41741ce83083430c79cbc94bc06` |
| auto-lang | 1485 | array-of-sections | `c139675c63fb825981cbe7f717a6aedf9eae9592` |
| auto-launcher | 10 | array-of-sections | `df207283536ff943e2e574dc886c4cb8594cce13` |
| auto-musk | 37 | array-of-sections | `8983771762e13fc20a8edb8db1e7352be4e7e0fa` |
| auto-os | 209 | array-of-sections | `85cee77b4c83c779ff9407177ea239af55939eaa` |
| auto-os-config | 38 | array-of-sections | `905e99335c37f259c69607faf44132d31752f969` |
| auto-paint | 4 | array-of-sections | `de667c33c20077827b48f342a715d8df49e000b5` |
| auto-reader | 7 | array-of-sections | `fa6185ebd1e35229ae496ef1e07afa2b157c7d74` |
| auto-term | 160 | map-of-item-arrays | `7a0d0e8b718fb260641f0d09ada95a7346771cd3` |

根账本共 2,250 条，其中 auto-musk 根 37 条在 PLAN-106 已迁移。另有 auto-musk/backend/.autoos/specs.json 245 条，原 ID 与根 37 条无交集，244 条 file 从 auto-musk 项目根解析可找到，另 1 条 G1/test 缺引用，须保全并明确标记，不能造为现行需求。待处理基线合计 2,458 条（2,250−37+245）；这是已观测基数，不是允许丢弃或新增数据的固定配额。后续源变动必须重新冻结并解释差额。

六份 auto-os/apps/{auto-blog,auto-database,auto-launcher,auto-os-config,auto-reader,auto-term} 是真实子模块，.gitmodules 已核对；其 JSON 解析值与对应顶层独立仓一致，个别原字节因排版不同。工作树存在不同版本账本，只作在途写者/漂移调查，不按独立新项目导入。

### 已观察到的代码和规约缺口

- spec_migration::dry_run 目前只接受 legacy.sections.as_array()；auto-down（251）、auto-term（160）实际是 section→item 数组字典；auto-os 当前为 section 对象数组，不应误判为字典。
- 当前导入器每条均输出历史 Markdown，并有 envelope 字段保全；不会自动提炼当前知识。apply/rollback 支持单 root/source、源指纹、目标条件和自有文件哈希，不具备批次跨来源登记/完整撤销报告。
- auto-os 有跨分类重复 ID（例如 P016-1 同时用于 reports/architecture/designs/tests，正文各不相同）；旧 ID 不能直接当唯一文件名或全局身份。
- auto-lang 有 720 条路径指向 docs/plans/archive（单数），auto-os/auto-os-config 同样使用 archive；现有链接推断只识别 archived 与 PLAN-NNN 前缀，必须支持实际 archive/archived、Pnnn 与已给定源路径，不搬迁历史目录。
- auto-lang 有 98 条 file 直连路径失效、63 条无 file；auto-os 有 67 条失效、44 条无 file。这些是旧路径核对结果，不等于正文丢失；须逐项保全并验证映射，不凭“文件存在”认定当前要求一致。
- auto-lang/docs/specs/README.md 与 docs/design/autoplan-spec-ledger.md 仍描述兼容 upsert，README 要求先调整设计源；auto-lang/AGENTS.md、auto-os/AGENTS.md 亦有旧 ledger 口径。需要在其工作树内同步最小活跃条款，保持其他测试/评审规则。
- auto-blog/edit/lang/launcher/os/reader 等有在途计划；迁移不得承接其实现任务或改写共享计划状态。T-01 单独确认旧写者的切换窗口；不能只传一个 stopped 参数就当停写证据。

已读当前规范：auto-musk/docs/specs/00-overview.md、modules/spec-document-store.md、modules/specs-ledger.md、modules/README.md；并查看 auto-edit 的总览、auto-lang 的操作规约和 auto-os 的计划/子模块规则。源 SHA256：spec-document-store.md `9c2d03cfa8436781028d3b403ed0d2671f9aabc8f77bbfc10abf6e6d36d28103`；spec_migration.rs `2c511a79f2d44ca1f474861d5d6945eff1fc09f7c810c5798cf1e4c7046b2fbe`；spec_documents_contract.rs `2dfd73975c7b1d7e1249bf3193e1da4d57959ffd30846b07ef42619e36d67299`。

## 5. 详细设计

### 范围、身份与来源定位

- T-01 输出源登记：规范仓库身份、实际默认分支、执行工作树、source 的只读绝对路径与项目内相对路径、目标根、引用解析根、原字节 SHA256、结构和数量、活跃写者及停写/隔离证据。备份/journal 都放本地 artifact 目录，不能放进共享旧 JSON 或提交原始证据。
- root/source/解析根均显式限定在本计划已登记项目。auto-musk/backend 是另一 source，destination/reference_root 为同组 auto-musk 工作树根；不建立 backend/docs/specs、不把 source 所在目录误当项目根。原 JSON 保留不写。
- 六子模块按仓库归属与已确认的记录集等价判定 alias；同时保留每份原字节哈希/备份及 alias→权威来源映射。格式差异不重复导入；如果记录集已分歧，列出新增/修改/缺失逐项处理，不丢弃副本独有内容。不同项目内容恰巧相同不能跨项目去重。
- 子模块目录和其 gitlink 保持不动；本计划交付顶层独立仓中的文档，报告记录镜像仍停留旧 pin 的事实及权威新提交。更新发行 pin/推送/部署另属发行工作，不能把“去重完成”描述成每份旧检出已自动更新。
- 稳定迁移身份包含项目/来源、source 完整哈希、section 键或位置、item 原位置；legacy_id 只作来源属性。同 ID 不同正文全部保留；重试同一 manifest 不生成第二套文件。哈希前缀缩写碰撞须拒绝/无损区分，不能覆盖另一来源。

### 输入格式与完整性

- 支持 sections 为 [{id, items, ...unknown}] 与 {sectionKey: [items...]}，保留原排序/位置语义与所有未知顶层/区块字段；如 T-01 发现登记源包含等价的 map-of-section-objects，可作为同合同的无损适配，须有真实样本/fixture，否则明确不支持，不能猜测丢字段。
- 缺 sections、错误的 items 类型、非对象条目、无法解析 JSON 等应明确失败，原源不变；空区块/空正文/缺 title/id/file/未知状态是可保全数据，不以默认 approved 补值。section/item 错形不能静默跳过并返回成功。
- 原字节备份与源哈希必须对应同一次读快照；字段保全可在 Markdown fenced JSON 中保存结构化原值，不能成为第二个独立可写事实源。围栏长度/元数据转义必须容纳原正文中的代码块及特殊标题，AutoDown 可正确读取。
- archive/archived、旧项目前缀、相对路径及正文锚点按显式 source profile 解析；只读取白名单项目内受限来源。绝对/遍历/链接逃逸不得用于自动读取或写入，原引用字符串仍完整保全并诊断。自动链接不能凭短 ID 猜另一个项目的 Plan。

### 文档拆分与对账

1. 历史保全：每条来源记录有不可混淆的历史位置 docs/specs/history/legacy-ledger/<source-key>/...md 与 envelope。原文、状态、依赖、未知字段可逐项回读还原；历史带 historical，不参与现行要求晋升。
2. 现行知识：与已存在 canonical Spec/归档 Plan 做内容核对。相同内容链接既有源；账本独有且可核实的持久要求提炼到该项目 docs/specs/{goals,modules,...}，按主题组织而不机械复制计划章节。缺少 docs/specs 的项目先形成最小概览及主题文档；先前知识目录保留来源指针，不平移整个文档库。
3. 冲突/断链/过时：完整保全后标明理由、候选规范及待裁定项；不能默认为有效，也不能因全部被导入历史而声称现行知识核对完成。每个条目都有已审阅的 disposition；需要用户决定的新业务语义才停相应项目，其他项目可继续。
4. 每项目维护可提交的 docs/specs/history/legacy-ledger/README.md 对账报告；auto-musk 在既有报告追加 backend 来源，不重导根 37 条。报告包含 source/section/原位置/旧 ID→保全文件、canonical/归档链接、状态/理由、相关提交及总数；大项目可拆分报告分片，README 仅导航，不生成手填产品索引。

### dry-run / apply / 恢复与回滚

- dry-run 可创建显式指定的本地 manifest/备份，但不改源/目标业务文件。预览给出全部预计写集合、existing target before hash、source 指纹及冲突；目标文件不允许任意绝对路径，只在允许的 docs/specs 范围。
- apply 前验证来源登记、停写证据、source/manifest/备份/目标指纹，独占每项目迁移，保留文件 CAS。源或目标漂移返回具体冲突；其他工具/进程不得凭进程内锁绕过。重复执行和部分完成按产物哈希恢复，完整成功才标记该来源完成。
- 自动导入工具仅撤销自身创建且 after hash 未变的文件；pre-existing 文件不删除。后续用户编辑、journal/备份损坏、目标指向其他项目时拒绝相应撤销并报告已撤/未撤位置。重复 rollback 幂等，撤销后的首次索引读取反映真实剩余集合。
- 经审阅的 canonical 提炼、既有文档/规约修改和对账报告属于完整迁移写集合，须记录 before/after 或对应 Git 补丁/提交。未合入时可按所有权与指纹撤销；已合入后通过新的反向 Git 提交撤销本计划补丁，不 reset/强推、不整文件恢复覆盖后续修改。出现后续用户编辑需显式解决冲突，不能承诺自动完整回退。
- 批次 journal 记录逐来源 prepared/applied/verified/landed/rollback 状态及每仓提交，不宣称跨仓原子性。部分失败后“从磁盘/提交事实重建并续跑/安全撤销”必须可演练，不能只靠内存成功计数。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/modules/spec-document-store.md | 单来源数组迁移扩至明确旧格式/多来源；维持路径/CAS/备份与历史边界 | 兼容真实旧数据 | AC-01/02/04 |
| SD-02 | add | docs/specs/modules/spec-batch-migration.md | 新增来源登记、批次状态、子模块 alias、内容核对与分阶段回滚合同 | 跨项目迁移可恢复 | AC-02…07 |
| SD-03 | modify | docs/specs/modules/README.md | 新增迁移模块文档导航，不增加 JSON 索引 | 可定位实现合同 | AC-07 |
| SD-04 | modify | docs/specs/history/legacy-ledger/README.md | 保持根37来源不变，补 backend245 对账/原始保全与提炼位置 | 补遗漏来源 | AC-03/05 |
| SD-05 | add/modify | 各目标仓 docs/specs/history/legacy-ledger/README.md；必要的 docs/specs 模块/概览 | 每来源完整去向；当前知识核对后提炼、历史隔离；具体主题路径由 T-01 冻结后 review 定案 | 避免导出即当有效规范 | AC-03/05/07 |
| SD-06 | modify | auto-lang/docs/specs/README.md、其 docs/design/autoplan-spec-ledger.md 和 AGENTS.md；auto-os/AGENTS.md；其他登记仓命中的活跃源规则 | 移除本次迁移范围的旧 JSON 写入/手填索引义务；保留原有业务/测试/复审纪律 | 避免迁移后再生旧记录 | AC-06/07 |

SD-05/06 的跨仓路径必须带 repo 标识登记，不能误填为 auto-musk 本仓路径。T-01 可细化同范围文件/主题，新增仓库或业务要求必须另获授权；不得仅修改下游 README 而保留其上游活跃设计规则矛盾。

## 6. 测试设计

执行者/内部检查者/最终复审者未指定；本计划不创建 agent 或新聊天。按 work→最终 review→merge 职责执行，优先独立上下文；同实施会话复审须披露限制，不代替用户另行指定的最终复审者。

以下是待实现并执行的验收案例，不是已通过证据。T-05 建立真实 CLI fixture；现有 spec_documents_contract 作为回归基线，新 target spec_migration_batch_contract 覆盖多来源。所有测试必须使用本计划构建、可追溯的可执行程序，而非陈旧全局安装。

| Case | AC / owner_stage | 实际入口和输入 | 可区分错误实现的断言 | 边界 |
|---|---|---|---|---|
| V-01 | AC-01/work | spec-migrate dry-run/apply，数组与字典真实快照脱敏/最小样本，未知字段/空字段 | 原始条目与导入回读 JSON 值逐一相等、区块/envelope 完整；错形非零且源/目标无变更 | 已登记结构；非任意 JSON 猜测 |
| V-02 | AC-02/work | 同项目 root+backend、同 ID 跨 section/跨来源/跨项目；默认 cwd 故意置其他仓 | backend 写入项目根 docs/specs；各内容均可回读；目标身份不碰撞，无跨仓串用 | 注册根与受限引用 |
| V-03 | AC-02/work | 同仓子模块字节不同但解析值相等；副本另加一条真实差异；不同仓同内容控制 | 等价 alias 只产生一份导入且两源备份可查；分歧项不会遗漏；不同仓不会误合并 | 六副本去重，不更新发行 pin |
| V-04 | AC-03/05/work | 有效当前知识、相同 canonical、旧 Plan 摘录、正文冲突、缺源、重复 ID | 对账覆盖100%；真实内容比对给出可审阅理由；缺源/冲突留原值且不晋升为 approved | 语义结论需人/agent核对，计数不能替代 |
| V-05 | AC-04/work | 两进程 apply、源/目标在 dry-run 后改变、同 manifest 重试 | 互斥/冲突明确且不覆盖；完整重试不新增重复文档；原 JSON hash 不变 | 迁移工具间协作，外部旧进程须停写/隔离 |
| V-06 | AC-04/work | apply 第 k 项失败、备份/manifest 不完整、另项目已成功后中断 | 完成位置/残留真实可查；重新启动按事实续跑，不把半批标全成功 | 逐项目恢复，无跨仓事务承诺 |
| V-07 | AC-04/work | apply→rollback→再读索引；已编辑产物、pre-existing 目标、重复 rollback | 仅删除自有未改文件；修改保护与残留清单成立；重复撤销可重入；重读集合正确 | 工具级撤销 |
| V-08 | AC-04/08/work | 临时真 Git 多仓 commit+ff，随后用户改同文件，再撤销迁移 | 新反向提交保留无关改动；重叠修改明确冲突不覆盖；记录部分撤销仓库状态 | 已落地反向提交，不改默认分支历史 |
| V-09 | AC-05/work | 真数据各项目 dry-run/apply/重试与根37 skip | 冻结来源总数与逐条结果对齐；副本不重复算；245 backend逐条有去向；所有原字节hash不变 | 数量漂移重新冻结，不硬编码2458过关 |
| V-10 | AC-06/07/work | 已生成 Markdown→新 index API→真实 AutoDown Web/VM 打开代表文档；检查 archive/archived 链接、长围栏/中文/元数据 | 可读且元数据/正文完整；历史不伪装当前；有效源链接可用；旧规范规则不再要求写JSON | 两端读验证；编辑核心沿用PLAN106，不重写 |

建议定向命令（在 auto-musk 工作树、依赖根按环境显式配置）：python scripts/cargo-worktree.py test --manifest-path backend/Cargo.toml -p musk --test spec_documents_contract --test spec_migration_batch_contract。T-05 加入真实 CLI subprocess，不以纯 helper 总数代替实际命令；受影响其他 test targets 依据调用面补充。各数据仓采用文档/引用/实际索引读取与自身要求的受影响检查，不因迁移复制一套 auto-lang 编译器全量门到每仓。合回前适用门保持各仓规约，不重复执行未受影响的完整测试。

资源合同：必须支持冻结的最大真实账本（当前 auto-lang 约2.45MB/1485条）与本批全量，覆盖文件名/围栏/编码、明确 I/O/磁盘失败。不得新增会排除当前合法记录的任意深度/长度上限；不承诺无限输入或固定性能数字。

## 7. 验收标准

- [x] AC-01（owner_stage: work）两种实测结构均可完整导入；未知字段、空正文/缺字段、排序/身份可追溯；错形拒绝而非漏项成功；V-01通过。
- [x] AC-02（owner_stage: work）来源/root/解析根隔离、重复ID处理和六子模块 alias 去重均正确；分歧副本独有数据不丢、不同项目不误去重；V-02/03通过。
- [x] AC-03（owner_stage: work）冻结的全部待迁移条目都有保全及已审阅去向；auto-musk/backend 独立覆盖、根37不重复；现行知识按主题核对提炼，历史与待裁定不伪作新要求；V-04/09通过。
- [x] AC-04（owner_stage: work）真实 CLI dry-run/apply/重试/中断恢复/rollback及落地后的反向提交方案可演练，冲突和用户后续编辑受保护，原JSON/备份指纹可核对；V-05…08通过。
- [x] AC-05（owner_stage: work）每项目有可提交的完整对账报告，包含源指纹、逐项去向/理由、数量和失效/冲突清单，链接与原始字段回读可验证，原始证据留本地；V-04/09通过。
- [x] AC-06（owner_stage: work）各仓必要活跃流程规约及其上游设计源已一致切换；旧写者停写/隔离有实际证据，源漂移正确拒绝，无agent手填索引或JSON发布门；V-05/10及规则核对通过。
- [x] AC-07（owner_stage: work）各仓文档通过实际自动索引/阅读核验，Web/VM可读代表性迁移文档；SD-01…06与仓库规范一致，所有缺失/坏源都有诚实诊断，不因应用/缓存不可用放弃保全；V-10通过。
- [x] AC-08（owner_stage: review）最终复审核对全部来源/去重/完整对账/语义提炼/回滚路径，抽取正常与反例验证，绑定每仓 reviewed_commit、来源快照与规范增量；必须先通过才能合回相应仓库。
- [ ] AC-09（owner_stage: merge）所有目标仓都按已评审版本线性合回、最终报告落Git；保留部分失败/反向提交事实且无假全量成功；技能/规约实际来源、原JSON不变、生产旧产物观察与工作树/分支清理均有收据，Plan最终真实归档。

## 8. 执行步骤

| ID / owner_stage | 依赖 | 文件/实际入口与预期结果 | AC / 验证 |
|---|---|---|---|
| T-01 / work | 无 | 重新盘点13根+backend+6副本、各仓AGENTS/活跃计划/旧写者；冻结源与全部写集合/主题路径、工作树所有权、归档目录/别名解析策略、需调整的活跃规约清单。先做一份小源及最大源 whole-path dry-run可行性调查，未决路径在后续写入前闭合 | AC-02/03/06；调查结论与baseline记录在§9，未知源码与机制不得假装已验证 |
| T-02 / work | T-01 | spec_migration.rs：格式适配、结构验证、全字段/原位置/围栏保全，维持原工具兼容与无损备份 | AC-01；V-01 |
| T-03 / work | T-01,T-02 | spec_migration.rs/main.rs（必要时新模块）：来源登记/批次选择、backend根映射、alias比对、重复ID/旧路径/归档目录支持；具体CLI参数由T-01定案并写用法 | AC-02；V-02/03 |
| T-04 / work | T-02,T-03 | manifest/journal演进、真实写集合指纹/迁移互斥、逐项恢复/回滚与跨仓提交撤销说明；兼容PLAN106已存在manifest或明确只读旧版本恢复策略 | AC-04；V-05…08 |
| T-05 / work | T-02…T-04 | 扩展spec_documents_contract.rs；新增spec_migration_batch_contract.rs，使CLI/多进程/真Git/反例可执行，公开可复现命令与artifact身份 | AC-01/02/04；定向目标非零用例且通过 |
| T-06 / work | T-01,T-05 | 原八小项目auto-blog/charts/database/edit/launcher/os-config/paint/reader及新增auto-web/auto-ui/auto-man，当前基线123条：各自worktree内dry-run、内容对账、apply、再读/重试、报告与canonical提炼，核对镜像alias不重复写 | AC-03/05；V-04/09，逐仓证据与代码/数据commit |
| T-07 / work | T-01,T-05 | 大项目auto-lang/down/os/term，当前基线2119条：分批执行同样闭环，处理缺源/跨分类同ID/archive路径与当前规范冲突；不以批次数或全量历史导出替代逐条去向核对 | AC-03/05；V-04/09，最大真实源可执行 |
| T-08 / work | T-01,T-05 | auto-musk工作树消费backend245条，reference/destination为项目根；核验已有根37映射后skip，新增旧源对账/保全/提炼，不生成backend/docs/specs | AC-03/05；V-02/09 |
| T-09 / work | T-01,T-06…T-08 | 各仓活跃知识流程规则最小一致性调整；auto-lang先改design再对应README/AGENTS，auto-os旧账本规则同步，关闭已观测旧写入口/流程；新增本仓spec-batch-migration模块并更新导航 | AC-06/07；SD-01…06内容与停写/隔离证据 |
| T-10 / work | T-05…T-09 | 真数据汇总对账、全部源不变/alias/报告链接/字段回读、Web/VM代表性阅读和首次索引、必要CLI回滚演练；按仓/来源列出verified/待裁定，不放过未决核心要求 | AC-01…07；V-01…10证据齐全，准备review基线 |
| T-11 / review | 相应仓work任务及其验证完成 | 同一仓实施树中暂停写入，独立逐仓/最终总体验收和规范增量复核，绑定源/commit/依赖；允许已完成仓先复审后交merge，其余任务继续，未完成仓不能提前计pass | AC-08；最终pass或明确findings回work |
| T-12 / merge | 相应仓T-11 pass；最终完成需全批pass | guard/rebase/range-diff/ff-only、canonical与迁移对账核验、逐仓提交/alias旧pin观察、归档Plan+提交收据、工作树及编译中间物清理/保全exe与生产产物观察 | AC-09；真实归档状态/12任务计数/清理一致；不得写旧JSON |

- [x] T-01（owner_stage: work）源、仓库、写者和范围基线冻结。
- [x] T-02（owner_stage: work）旧格式及完整性适配。
- [x] T-03（owner_stage: work）跨来源与子模块去重。
- [x] T-04（owner_stage: work）恢复及两阶段回滚。
- [x] T-05（owner_stage: work）真实CLI风险用例。
- [x] T-06（owner_stage: work）原八个小项目及三份拆分账本迁移。
- [x] T-07（owner_stage: work）四个大项目迁移。
- [x] T-08（owner_stage: work）backend旧源迁移及根账本skip。
- [x] T-09（owner_stage: work）规约切换和规范增量。
- [x] T-10（owner_stage: work）真实阅读及全批对账验证。
- [x] T-11（owner_stage: review）逐仓及最终复审。
- [ ] T-12（owner_stage: merge）合回、真实归档与清理。

执行入口：T-01正在执行；2026-10-31重新盘点冻结17个来源、6份子模块别名，尚未apply。先继承本文的明确数据范围，重读各仓baseline/规约/在途写者并冻结实际来源，确认停写窗口后才apply；T-02…05建立兼容与安全oracle后才批量落数据。小项目pilot只改变实施顺序，不能削减大项目和backend范围。所有跨仓实现/数据/规范更改在该仓同一专用树中接受复审，已通过的仓尽快合回清理，整批仍保持executing直到全部work任务完成。

## 9. 复审记录

### PLAN-107:r1 — new / 起草交接

- stage: new | outcome: pass | next: work；status=drafting，current_step=0/12；AC-01…09、T-01…12均未执行，不把本文的fixture/命令当通过证据。
- 起草只写主检出本Plan；未创建实施工作树、未修改迁移器/其他仓规范、未运行apply/rollback、未停止服务。范围来自用户本轮明确要求；未继承已完成PLAN106的goal自动执行授权来启动新计划。
- 起草时核对盘点的13根账本hash一致，补实测archive/archived差异、auto-os跨分类重复ID、backend根映射、六副本等价和活跃旧写规约。数据/代码调查基线见§4；执行前T-01重新冻结，不能把旧盘点数量当不变配额。
- Plan取号在.git/auto-plan-new.alloc.lock的Windows排他文件锁下扫描active+archived最大ID+1，以exclusive create写入，写后复查本ID唯一；起草未观察到本机17201/8080应用写者监听。该锁不是声称现有REST分配器自动共享跨进程锁；如写后同号碰撞，删除本次自有新草稿并重新取号，不覆盖别人的Plan。


### PLAN-107:r2 — 执行授权与拆分后重冻结（2026-10-31）

- 用户要求现在优先完成迁移；随后明确授权将auto-web及auto-man/auto-ui三份拆分账本一并纳入。授权执行、复核与按计划交付；不包含远程push/改发行pin/生产强制重启。未创建新goal或其他agent。
- 用户明确允许帮助提交主检出六技能既有WIP，原样审查后提交3b291c91d9c1668396b192665245494fc7d2a28d；未夹入本计划或__pycache__。实施工作树D:/autostack/.wt/musk-107/auto-musk，plan-107-dev，从该提交建立，未触碰其他仓原有WIP。
- auto-lang PLAN890实际已archived r7/12-of-12，新auto-web仓已存在；UI014仍drafting，不以全部未来重构完成为全局门。按现有已提交职责核对canonical知识，不消费他方WIP。
- 重冻结：auto-lang1499（较旧基线+14），auto-web/auto-ui/auto-man各5（+15）；待迁移2487，auto-musk根37为skip对照。共16项目17来源；6份子模块解析值与顶层源仍等价。逐源原字节备份、hash、HEAD、分支、位置在D:/autostack/plan-artifacts/plan-107/source-registry.json。源隔离为本计划只读输入快照，应用前及合回前重查原账本hash；外部后续写入不得静默丢弃。
- 当前其他仓只读调查发现auto-lang/auto-os/auto-ui有他方文档WIP，保留原样；只在从已提交版本建立的本计划树修改，触及同文件时先对账。T-01未完成、其他work/AC仍未勾选。


### PLAN-107:r2 — T-01…05 闭合与真实小项目pilot（2026-10-31）

- stage: work | outcome: in_progress | next: T-06/T-07/T-08；基线3b291c9，16项目17来源（其中根37 skip），2487待迁移，六alias等价。各仓唯一树均已建立于musk-107组，分支auto-musk-dev（本仓plan-107-dev），初始tracked links为0；source-registry.json绑定每仓实际execution_head。既有非本计划WIP全保留。
- T-01决策：原源只读、输入为字节快照隔离；每项目target/reference root是其本计划树。规范/历史归属按已提交资料，源/副本前后hash一致才允许推进。批次只写生成历史文件及本计划报告/最小知识规约，完整写集合由batch-state.json的write_set按execution_head捕获；codecore之外不改其他仓业务。
- T-02/03：Rust适配数组、map-of-item-arrays及有fixture的map-of-section-objects；拒绝错形，保全原顺序/envelope/未知字段、动态JSON围栏、大整数；v2完整source-key+完整sha命名空间，archive与archived/Pnnn及受限项目前缀引用；v1manifest继续恢复。
- T-04/05：同一个Rust引擎由musk spec-migrate和Python批次壳消费。原子exact/no-clobber恢复材料、backup与manifest digest检查、按文件CAS、部分完成重建；Python记录prepared/verified/partial-or-conflict/import-rolled-back，按显式--only隔离其他项目漂移。rollback只撤自身未改产物；已落地通过反向Git提交演练保留后续修改，重叠修改真实冲突。
- 验证：现有spec_documents_contract 11 passed；新spec_migration_batch_contract 7 passed（含实际CLI、错形/重复ID/两种map/metadata/源目标及备份损坏/超64位JSON整数）；Python真实临时Git worktree批次4 passed，包含别名去重、重复apply/rollback、独立项目继续、部分冲突/恢复、已提交数据反向撤销/后续修改保护。
- 首轮Python4失败为Windows canonicalize的\\?\前缀与普通绝对路径比较，修正physical identity归一化后4通过；原失败日志保留，不计产品CLI行为red。依赖基线check成功（2m15s），未增加外仓代码兼容修复；现有编译告警与测试通过分列。
- auto-charts真源5条pilot：prepare→apply→重复apply→rollback→恢复apply全部成功，原JSONhash未变；不是仅helper证明。全量prepare开始前，冻结registry未变。日志 D:/autostack/plan-artifacts/plan-107-{core-final-tests,python-batch-tests2,pilot,prepare-all}.log；执行数字是已运行事实，后续批次尚未算完成。

## 10. 待澄清事项

当前无阻止计划进入work的用户决策。T-01拥有明确边界的技术调查：旧写者切换窗口、各源字段/归档规则、主题拆分位置、子模块漂移/别名和CLI批次参数、旧manifest兼容路径。未授权新增仓库、业务语义、远程发布/子模块发行pin调整或生产服务强制重启；发现确需这些变更时仅暂停相关依赖动作并呈现具体决策，其他已授权工作可继续。

### PLAN-107:r3 — 原生阅读最小兼容修复授权（2026-10-31）

- 用户明确允许本次原生读取迁移JSON代码块所需的最小auto-ui兼容修复；范围为auto-ui-runtime iced renderer初始化顺序及针对性回归，不扩大业务功能。r2的数据迁移/归属合同不变，最终review以r3为准。
- 实际反例：新VM进程、MCP先探测创建编辑核心，再从真实文件树打开auto-down的0-0.md；首个fence同步早于DocEditor字体回调安装，code_editor/core/mod.rs:509 panic，继发registry锁poison。API17来源与Web真实AutoDown均已pass；原生暂不标pass。修复把两条iced装配路径的DocEditor构造提前到sync之前，保持字体源不变；fresh-process回归与真实阅读复验待完成。

### PLAN-107:r3 — 全量迁移与对账冻结前证据

- T-06/07/08：2487条新导入（小八仓108、三拆分仓15、大四仓2119、backend245）+16 envelope；根37保持skip，总源条目2524。17来源全部verified；6副本解析等价，仅alias，所有原JSON及快照/副本sha未变。
- T-09：2487条逐条保全与去向报告；5050条新对账导航链接全部存在。两份旧web合同归位auto-ui/auto-man的docs/specs/modules/web-integration.md，旧路径转引；auto-down和auto-os-config补最小知识入口；auto-lang上游设计、README、AGENTS及auto-os AGENTS退役JSON/upsert和强制手填索引义务。当前知识与历史断言分开，不重认证历史测试数字。
- 风险验证：Rust现有11+新增7通过；最终Python6通过，增加真实双apply进程项目锁与分歧alias拒绝丢独有字段。全部真实源逐记录及envelope未知字段/顺序回读通过。Web真实AutoDown3项及API17来源2524身份通过；原生字体初始化问题仍在修复，T-10/AC-06未标完成。
- 其他仓代码仅auto-ui最小兼容修复（已明确授权）；auto-os主检出组合体检有既有auto-charts指针差异+d5f5535，27条manifest无缺pac.at。本计划不改其gitlink/他方WIP，不将该基线红伪作绿。
- 当前六技能2026-10-31已更新证据规则为合并时删除scratch，覆盖旧latest-five；本计划将摘要、源指纹与实际提交/回滚映射归入本文件后按当前规则清理。保全的原值文档与只读旧JSON及Git补丁共同提供长期迁移/撤销依据，不依赖临时journal。

### PLAN-107:r3 — execution_done / 最终复审基线

- 16仓迁移/规约及最小auto-ui修复全部提交，专用树git status为空，实际分支/HEAD已核对。实施写入暂停进入final review；无另行指定最终复审者，当前同实施上下文，不能声称独立agent。
- 原生真实AutoDown core探测已通过backend旧源0-1.md：中文标题、完整hash、Original record代码块存在，spec_dirty=false，磁盘字节与manifest相同。Web使用auto-down map源0-0.md；API覆盖17来源2524身份。原生此处验证默认项目的真实阅读，不将跨工作区切换作为已验证承诺。
- auto-ui新进程integration test spec_document_first_fence 1 passed (0.79s)，修复后auto-ui-tool构建32.53s；泛型与VM两处均先DocEditor构造再sync。早期lib test构建被既有autodown_engine.rs 18处旧测试符号缺失阻断，未声称该全lib通过；改用独立集成target避免无关旧测试编译。初版集成断言要求序列化围栏/末尾换行字节不变，超出既有AutoDown语义；改为中文与JSON语义保持（真实磁盘源仍逐字节不变）。原失败日志摘要保留于此。

| 仓库 | diff基线 | reviewed_commit |
|---|---|---|
| auto-blog | `04cf7b3a81766e54379adb8fe2ab51118d2617b7` | `4b9273b9149c62634295029e73d0dbdc9287776f` |
| auto-charts | `77b6f82aa43aef807ee7123c7a2eaed37f47f715` | `8f7ce7023924ef49bfdc543cd037b685a6d9940b` |
| auto-database | `bb3a0b30ded5b3e5db4dada60c9ec033e980b5dd` | `f159b417a37ae3792c40177616efe2286ccaaa99` |
| auto-down | `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4` | `dea5bd2626eceedd7d661bb92398e10ee23fbfd2` |
| auto-edit | `acf71a0541e0f41741ce83083430c79cbc94bc06` | `b21a509f6746ced41fecd3ac700f0c09bfae1f9f` |
| auto-lang | `98270a4f5995a115732841433a2328976b3fd585` | `08eaeada142e9e97ed1e3488443f5b483fe232e9` |
| auto-launcher | `df207283536ff943e2e574dc886c4cb8594cce13` | `8c6e3f2357c2398870b79813e8458445b0605253` |
| auto-musk | `3b291c91d9c1668396b192665245494fc7d2a28d` | `522e010c03dad3b60755128aff80fb0c6b4206ed` |
| auto-os | `734a6691edd1f72f3fe7f56c967aca3d8a84b1d7` | `dab860d6dc5661da871385e07e3bc7c2527d03a2` |
| auto-os-config | `905e99335c37f259c69607faf44132d31752f969` | `e294e4ee432ded5fdc6f3d7d8dee397da54d1d8f` |
| auto-paint | `de667c33c20077827b48f342a715d8df49e000b5` | `7e0161705e9526eebcd812cc71ec8c392eb522e3` |
| auto-reader | `44967204bb4cdd0267e8ef38fee8281c3822d96a` | `248c7f890580f7515b7e9e309cb5eb2edd0e1d23` |
| auto-term | `7a0d0e8b718fb260641f0d09ada95a7346771cd3` | `80227e18f1be565a72c32f92e9e2ee2027609035` |
| auto-web | `f29c7d1439e2dd202860dfd3ba300da07354f486` | `332717fe7da4f0a6f3a654d15017dd7759561816` |
| auto-ui | `7a53d13d5e6da6eedaa9b1e9de0a3fdcaca089c4` | `598347af8a922fb5a1afa3e4152770366f75a00e` |
| auto-man | `c4e5aca3d9adaba0f37ad2c035956426458e27f1` | `be740b6882eaeb0bd63db82e339246e8ffce9850` |

#### 来源冻结与对账（最终）

| 来源 | 新导入/skip条数 | 原字节SHA256 |
|---|---|---|
| auto-blog-root | 10 | `c8373155bdeac61a65804915e452ffc262e3e09af6fee12c23246505cebcf98a` |
| auto-charts-root | 5 | `24efedb3429b86a82071f9a3e8cd7a00b16cc8d5a39fe1c5a0037871ed4553f4` |
| auto-database-root | 4 | `b694d0fc5d22f368a6b5509bca33e4cbb68417f254afe54a67f8881a7a28f39e` |
| auto-down-root | 251 | `86f54c9ec932ac17e6723d448880934be3c466232ddb39cc27c75715a6ce02b7` |
| auto-edit-root | 30 | `c2eba5a8629464efca035319c2ce841be74b50681aed1265c708fbf242445941` |
| auto-lang-root | 1499 | `fa01d1dc2195d9cd4c5021d04ac2177ac96af9c2d39da8d045a04249a9143291` |
| auto-launcher-root | 10 | `c3ef747630a6354b5376c4761ee65158eef8cc8679ebad7f435e5a3532b4efbd` |
| auto-musk-root | 37 skip | `4d73a7aadbf4e3a2c83942d024f0921e5099763b94af4172e9430463f9561367` |
| auto-musk-backend | 245 | `9cf74525a18f6985d384bc5121db7479c7455e4ecb9ed740b7dc5843fa75261d` |
| auto-os-root | 209 | `dcd90f8b3eda6b89563c7e73bfba5120adf8d2edd0297a456a79ee7cf83a832c` |
| auto-os-config-root | 38 | `2120fe0b62ccecf3cf344a0b1e98124cdf0c50513fbb4cd0b2f6395726e21097` |
| auto-paint-root | 4 | `09ea3b783ffb59d77ac0ce42833145c5bbca8d5932ec81e883be14caa1c382b0` |
| auto-reader-root | 7 | `cf2215c35b27b2c5722047ecf5e88438484db8ac870afa1d665794a565bac24b` |
| auto-term-root | 160 | `9b90581b8fa12378fdc3ce1e122f0f394c700052057383371c9ec7f2dc61e0ec` |
| auto-web-root | 5 | `5d8488c4d7572e601b0f0977c5305612fc4d63526e8a2b96b5df449a4f7a6b0f` |
| auto-ui-root | 5 | `2cc490a1eacfe9682d5fe143c31a9682d6ef995b20fb52c178b7b3b13cb82807` |
| auto-man-root | 5 | `fd1cb266de2266924287cda0b1330758dc9f9a6bb8a57145a2d810b6bdc53090` |

六alias原字节指纹（各自与登记快照一致、解析值与归属源相同）：
- `D:\autostack\auto-os\apps\auto-blog\.autoos\specs.json` → `auto-blog-root` SHA256 `83962267700e82085381409b19a8776f99eb079044f36e60f95e96f5309f4a95`。
- `D:\autostack\auto-os\apps\auto-database\.autoos\specs.json` → `auto-database-root` SHA256 `b694d0fc5d22f368a6b5509bca33e4cbb68417f254afe54a67f8881a7a28f39e`。
- `D:\autostack\auto-os\apps\auto-launcher\.autoos\specs.json` → `auto-launcher-root` SHA256 `c3ef747630a6354b5376c4761ee65158eef8cc8679ebad7f435e5a3532b4efbd`。
- `D:\autostack\auto-os\apps\auto-os-config\.autoos\specs.json` → `auto-os-config-root` SHA256 `6edf576c18482528a2e1c24b66862a81f32cb46faace8707b3f0dae73a9f2cda`。
- `D:\autostack\auto-os\apps\auto-reader\.autoos\specs.json` → `auto-reader-root` SHA256 `cf2215c35b27b2c5722047ecf5e88438484db8ac870afa1d665794a565bac24b`。
- `D:\autostack\auto-os\apps\auto-term\.autoos\specs.json` → `auto-term-root` SHA256 `2aedec205b8d6260b525b7e3ecffdabaee7787b26d8566f410bd4215dee4371c`。

证据冻结摘要：Rust两个target 11+7 pass；Python真实Git/CLI 6 pass；17来源全字段+envelope顺序100%回读；5050对账链接零断链；Web3项、原生真实core/只读源、API17来源通过。原始本地log将在合并整理摘要后按当前证据政策清理，不是永久附件依赖。

### PLAN-107:r3 — final review / pass（同实施上下文）

- stage: review | review_scope: final | outcome: pass | reviewer_identity: 当前会话Codex，未创建独立agent，复审独立性受限 | plan_revision:3。逐仓复核实际worktree/branch/committed HEAD、Git差异、spec delta、原始运行记录与文件；16树无未提交实现。reviewed_commit以以上16仓表为准，auto-musk最终为`522e010c03dad3b60755128aff80fb0c6b4206ed`，auto-ui为`598347af8a922fb5a1afa3e4152770366f75a00e`。
- AC-01→T-02/03/05→Rust map/array/未知字段/错形/大整数7及全源envelope回读pass；AC-02→T-01/03/05→16仓17源、6alias、命名空间/重复ID/分歧保护pass；AC-03→T-06…09→2487条报告与主题提炼pass；AC-04→T-04/05→Rust恢复+Python6真实CLI/Git/双进程pass；AC-05→T-09→5050链接、全记录/完整元数据回读pass；AC-06→T-01/09/10→源快照隔离与原hash不变、活跃规约/六技能实际源核对pass；AC-07→T-10→API17/Web3/原生真实core及源字节未变pass；AC-08本次冻结/反例/差异复核pass。AC-09/T-12保留merge-owned pending。
- 逐条去向计数：current-source-linked440，historical-plan-linked1937，historical-source-linked44，external-historical-plan37，external-owner-linked23，external-historical-receipt3，empty-test-marker1，historical-unresolved-source2，总2487。两条缺源为auto-launcher reviews P001-9/P001-10，原值保全且明确来源未认证，不伪造复审通过，不阻塞已存在当前规范交付；此为本计划允许的诚实历史诊断。UI015与T-788/AGEX/空G1例外按报告明确归属，历史Plan不自动成为当前需求。
- 复审另选CLI反例：foreign-root-journal返回1/“different root/source”，另选重算正确digest但包含../../../../的target返回1/“Spec path must be relative to docs/specs”，均无逃逸写入。不是仅信测试banner。
- SD-01…06逐仓文档delta与实现一致；r3授权增量为auto-ui/docs/specs/modules/native-document-initialization.md及两条renderer装配初始化、独立回归target，无新增业务承诺。active规则扫描仅命中只读历史JSON/禁止写入声明；旧设计§1历史事实已显式注明被§5覆盖。
- 待merge gate：每仓fresh guard→rebase真实默认分支→等价range-diff→ff-only→默认分支规范/原JSON指纹核验；本计划数据/doc门使用全字段和链接检查。语言仓仅文档，按当前规则不运行语言编译全组；UI受影响gate采用已通过新进程集成及真实VM，代码/依赖/输入不变可复用；root采用最终Rust18/Python6与真实API/Web证据，不重复无关全库旧红。auto-ui全lib旧测试编译红18、auto-os既有组合pin差异保持显式观察，不改他方内容。
- Spec delta冻结：下表为reviewed docs/specs Git tree与含规约变更的binary diff SHA256；可按上述base/reviewed提交重新生成。

| 仓库 | docs/specs Git tree | 规范/规约delta SHA256 |
|---|---|---|
| `auto-blog` | `0f36b39efcadc7c5978c5d586a05113d79af32d3` | `05c2fe9259aaf67fb1e500efa5fc86ef2513b9d815f244929621a8d55a595dd7` |
| `auto-charts` | `76c426c76db965325495c2453e210444ce5d85b0` | `ce726e0a9d95922985d54e580c116e0b5d7f1236202c87daf76dda34897bfec3` |
| `auto-database` | `dddd41a9c6c558df0ab77635b63b8345b4358177` | `a2eefaa7ae17b06a0d33861752f3fa427c1d6e0134c93ef26bbee6cda9b2c9b7` |
| `auto-down` | `d98782fcb84f78db22f52df1300aa92e7278edcb` | `6a92df0e3a718a508ce47ad6a149c387abbe2e36c0593938687a03bb440d3e06` |
| `auto-edit` | `964d60e8f3457c29128b95fbae7c0cffedde8485` | `6a07d712971c8dbf365021730fd1cf464872f2041d5e005b1d4cd68d4275ef3a` |
| `auto-lang` | `afd0fc8b929f4b18b2ebed91c25edf48108f9da7` | `cf3954c1e9ffef902944fddf7b5976e42ccc4c91eff72a6e715561f0e8c797e8` |
| `auto-launcher` | `53490b5cd6c71fd87ef6b5c591d75c8160137f2a` | `2efdde7d2122fd167963e077af491bcf1bdc8508fa98b9689938305d9a06636c` |
| `auto-musk` | `f543b15f25f63524daaacda83281e9ceeead6496` | `d4463ea73d0aa7803f5f9768ed85e6c7162efc45e730cfd0daf44b7756b0bb6f` |
| `auto-os` | `3ac44c489fe3df8a7a2a74b968e10f088e137a97` | `c56e93ac1b1484cf53d130ec02c8e075285d894e391752b5117884cf84a96309` |
| `auto-os-config` | `437caa4cae55fb7951a9f8741019e888fc5a03da` | `6ec933c73aee6d79a66a480719b2f0d12710dccd9eced7a030be69dd1f002133` |
| `auto-paint` | `3271cb22df89a74293a71bfb63101332ca639137` | `f3c892a736ae5f3ebbe695e515aafe85b76204615e459107f4b47e8aa7bc1d89` |
| `auto-reader` | `3c5371b5bdf6b94ece7642735fde560360e72cc5` | `77d2317ad595dc82f8fc1b6df182538c7c0eda0e8b8c1a332d7df50a1ddf6a5f` |
| `auto-term` | `1f9cbcf0be66b8124f02974c0de91b649f920a56` | `8662fa7ff044ac8f51a4f613f51defcb9f1ba3645d11d27d74dbabd88a94406b` |
| `auto-web` | `83ff1020e5309f04d60967f0c040b4944f27450a` | `4dc7ebb8c8433d6c95892d2b326c9aca44d5ae145bca85811d51fe4d3e5a6bf7` |
| `auto-ui` | `4bbe130a356f89eddd5adb28558de6ad6ec2c057` | `cf69adcd8fb5ee5d65b5c5f37d73d81c2ab096a7aafaea7bad3546f0e02e1714` |
| `auto-man` | `ea0638fd43e06c964cd0789c6622249a8a32c75d` | `0b7613871317c7f8acb1964a8d277be32ca8f6aea9218da423fce4928520eeaa` |

next: /auto-plan-merge；实施写入保持冻结，若rebase内容变化则返回受影响复验。

### PLAN-107:r3 — merge prepared / 并行默认分支推进

- 当前reviewed基线有效；实际默认分支较起点推进：auto-down b1805ed（证据附件退役），auto-musk ded1220（当前技能政策及附件退役），auto-reader e4dbb51（其他Plan文档），auto-lang 15d978c59（Plan890落地代码/规范及政策）。不覆盖这些并行结果。先rebase并重新核验新落地引用/规约；auto-lang代码推进会影响原生/后端依赖，需在当前依赖上补受影响集成验证，不能仅靠range-diff称依赖未变。
- prepared：各仓Spec及报告均在committed review baseline；原JSON只读；不发布任何中央JSON/索引，不用后端自动归档。最终range-diff与新交付SHA记录待完成。
