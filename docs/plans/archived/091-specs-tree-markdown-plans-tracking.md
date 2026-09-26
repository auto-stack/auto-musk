---
plan_id: PLAN-091
status: archived
completion_kind: delivered
feature_name: 规范树递归+Markdown 与计划页跟踪小改
author: [zcode]
created_at: 2026-09-25T23:04:32+08:00
updated_at: 2026-09-26T11:20:00+08:00
plan_revision: 1
current_step: 6
total_steps: 6
supersedes_spec_components: []
new_spec_components: []
touched_goals: [goal-spec-knowledge]
---

# [PLAN-091] 规范树递归+Markdown 与计划页跟踪小改

## 0. 变更摘要

2026-09-25 功能加强审计裁定的第 1+2 项合并立项，两个互不依赖的小批次：

1. **规范文件树模式升级（根因修复，非新功能发明）**：
   - `GET /api/specs/tree` 数据从 wiki `TreeNode`（name/path/type、叶子缺
     `children`）切到 fs 形 `FilesNode`（id/label/children 恒存在/kind/
     is_leaf）——FileTree 组件（PLAN-614/068）本身已是递归组件，"扁平"观感
     的根因是数据 schema 不匹配（files_browser.rs 注释自证"刻意不用 wiki 的
     TreeNode——schema 与 FileTree 组件不匹配"，spec_tree.rs 未跟进）。
   - 树模式正文从纯文本 `text` 切 Markdown 组件（复用 specs_detail 五处在用
     的 platform:markdown）；正文加载通道从 `specs_get_file`（raw 字节响应）
     切 JSON 包裹的 `specs_text`——`#[api]` 绑定固定 `response.json()`，纯
     文本响应必炸（PLAN-089 T-07 files_text 同款处方与同款缺陷面）。
2. **计划页跟踪小改**：
   - 状态徽标 i18n（065-F1 余项）：`plans_view.at` 三处 raw status 改走
     zh/en 既有 `plans.status*` 键（静态键 if-chain，规避 VM t 动态键 quirk）。
   - 详情同步刷新：切入计划页列表重拉时，若已有选中计划则同步重载其正文，
     消除 plan flow coder 相位后台回写 §8/frontmatter 后正文陈旧。

**非目标**：ledger↔docs/specs 漂移治理（另立项）；merge 入口收敛；§8 勾选
解析展示；run↔计划双向跳转；web 冻结轨（`web/` 不改）；规范树搜索框。

## 1. 目标

- 规范文件树模式呈现嵌套可折叠树（目录 chevron 展开/收起、guides 缩进线、
  文件夹优先排序、目录/文件图标正确），md 文件正文经 Markdown 渲染，正文
  加载失败为独立可见状态。
- 计划页所有状态徽标显示本地化文案（中/英资源随语言切换），未知状态回退
  原始值。
- 切入计划页时选中计划详情正文与磁盘内容一致（外部回写后一进视图即见新
  内容），且不打断进行中的编辑会话。
- 不回归既有门禁（build strict / vitest / ui-parity check / 相关 cargo test）。

成功判据 = §7 五条 AC 全部可复现验证。

## 2. 架构方案

### 后端（hw 逃生舱路由惯例，`backend/crates/musk/src/`）

- `spec_tree.rs`：`spec_tree` handler 改产 fs 形节点——复用
  `crate::files_browser::FilesNode`（pub 结构体，字段 id/label/children/kind/
  icon/is_leaf/badge），目录优先+字母序沿用 `wiki::build_tree` 的排序语义
  自产构建（specs 目录树无忽略清单/深度预算需求，不泛化 files_browser 的
  workspace 根构建器）。路由 `GET /api/specs/tree` 不变，响应体形状
  `Vec<TreeNode>` → `Vec<FilesNode>`。
- 新增 `GET /api/specs/text/{*path}` → `Json<FilesTextResponse>`
  （`{content, error}`，成功 error 恒 ""）：`validate_path_pub` 路径穿越
  守卫 + 根=workspace `docs/specs/`，镜像 files_text 语义。非 2xx 时 VM
  get_json 包错误对象、web 生成端 throw——两臂前端各接（files_view Pick
  同款 try/catch 双臂）。响应类型复用 `FilesTextResponse`（同形状，零新类型；
  评审若偏好独立 `SpecsTextResponse` 别名属等价实现）。
- `GET /api/specs/file/{*path}`（raw）**保留**：浏览器直链/未来图片类消费面，
  不在本计划退役。

### 契约层（`src/back/api.at`）

- `specs_tree()` 返回类型 `[]TreeNode` → `[]FilesNode`（FilesNode 已在
  api.at 定义，PLAN-068 引入）。
- 新增 `specs_text(path str) FilesTextResponse` 声明（`/api/specs/text/{path}`）。
- `TreeNode` 类型暂不删除（wiki 域仍在用）。

### 前端（Auto 单源 `src/front/`，双轨随 codegen）

- `specs_store.at`：`spec_tree` 类型 `List<TreeNode>` → `List<FilesNode>`；
  `LoadSpecFile` 改走 `specs_text` try/catch 双臂（`resp.error ?? ""` 兜
  VM 错误包缺键），新增正文加载中/失败独立标量（`file_loading`/`file_failed`
  形态，对齐 FilesStore active_loading/SetLoadFailed 先例）。
- `specs_view.at` 树模式：正文区 `.md` 路径 → `Markdown(source: …)`
  （ports/renderer.at，specs_detail 同款 props）；非 md → 维持 font-mono
  纯文本兜底；补 loading/failed/未选中三分支（workspace-ui「三个独立状态」
  条款）。FileTree 消费不变（数据形状对齐后递归自然生效）；`default_expanded`
  传顶层目录 id（首屏可见性，执行时按观感可调）。树空态补提示（空 workspace
  docs/specs 为空时现仅空白）。
- `plans_view.at`：三处徽标（列表 `:122`、归档栏 `:169`、详情 chips `:258`）
  if-chain 静态键 `t("plans.statusDrafting")` 等五态 + fallback 原值。
  **禁止动态拼接键**（VM t 动态键 quirk，musk-vm-view-quirks 在案）。
- `plans_store.at`：`PlansLoaded` 回填段尾部，`.current != None` 时追加
  `LoadPlan(.current.seq)`（同步 `plans_get`，与同函数尾部既有
  `LoadPlan(.plans[0].seq)` 同链路同风险面）。编辑安全论证：视图 textarea
  绑定的是视图本地 `edit_content` 拷贝，store 刷新 `current_body` 不打断
  编辑会话；SaveEdit/CancelEdit 后自然显示刷新后正文。

### 归属边界

单仓 musk；无 auto-lang 依赖（不触碰 codegen/VM 引擎）；无跨仓改动。

## 3. 技术栈

Rust axum（hw escape-hatch 路由，spec_tree/plans 同惯例）；Auto .at 单源双轨
（vue codegen + VM/iced）；vitest；cargo test（`backend/` 下跑）。零新依赖。

## 4. 需求分析与背景调查

**授权记录**：2026-09-25 用户在功能加强审计会话中裁定"把 1、2 项合起来立项"。
授权范围 = musk 单仓后端+Auto 前端契约层与视图层；无预算/自动继续限制登记；
worktree 规则按 AGENTS.md（组 `musk-091`，分支 `plan-091-dev`）。

**关键证据（均已核实到行）**：

| # | 证据 | 落点 |
|---|---|---|
| E1 | FilesNode 注释自证 schema 刻意区分："刻意不用 wiki 的 TreeNode（name/type、叶子缺 children，schema 与 FileTree 组件不匹配）" | `backend/crates/musk/src/files_browser.rs:66-78` |
| E2 | spec_tree 复用 `wiki::build_tree` → 产 wiki 形 TreeNode | `backend/crates/musk/src/spec_tree.rs:13,80` |
| E3 | "扁平文件树渲染（递归留作 KNOWN-DEBT）"口径过时——FileTree（PLAN-614 移植）本身递归（guides/has_kids/Toggle）；扁平观感 = E1+E2 产物 | `src/front/specs_view.at:376` / `src/front/filetree.at` |
| E4 | flatten_tree 消费 `{id,label,children,kind,icon,is_leaf,badge}`；头注：叶子缺 children 对 `.len()` 踩 undefined | `src/front/tree_util.at`（node schema 注 + flatten_tree） |
| E5 | `#[api]` 绑定固定 response.json()，纯文本响应必炸；files_text JSON 包裹处方 | `src/back/api.at:203-221`（T-07 注）/ `backend/.../files_browser.rs:94-97,267` |
| E6 | 树模式正文现为 `text .store.active_spec_file_content` 纯文本（font-mono） | `src/front/specs_view.at:409-416` |
| E7 | 三处徽标 raw status；zh.json:203-207 `statusDrafting..statusArchived` 全套在案（en.json:203-207 同）零消费 = 065-F1 余项 | `src/front/plans_view.at:122,169,258` / `src/front/i18n/{zh,en}.json` |
| E8 | PlansLoaded "current 已有值即跳过"——列表刷新不重载正文；ShowPlans 每次切入都 LoadPlans | `src/front/plans_store.at:160` / `src/front/app.at:299-312` |
| E9 | 规范栏目条款："文件模式显示规范树和正文"三独立状态；新增文本必须中英双语 | `docs/specs/modules/workspace-ui.md:17,31` |
| E10 | Markdown 组件先例：specs_detail 五处 `Markdown { source: }`；renderer 端口双轨 facade | `src/front/specs_detail.at:18,281…` / `src/front/ports/renderer.web.at:11` |

**依赖**：无活跃计划冲突。PLAN-089（files_text 处方出处）已归档，本计划只
消费其定型模式。 PLAN-071（计划页列表形态）已交付。

## 5. 详细设计

### T-01 后端 spec_tree schema 切换

- 文件：`backend/crates/musk/src/spec_tree.rs`（handler + tests）。
- 内容：`spec_tree` 自产 `Vec<FilesNode>`（复用 `files_browser::FilesNode`；
  id=相对路径、label=文件名、目录 children 恒非 None、文件 children=[]、
  kind/is_leaf/badge 按 FilesNode 语义）；排序沿用 folders-first + 字母序；
  `docs/specs/` 缺失仍返 `[]`。
- 验证：`cd backend && cargo test -p musk spec_tree`——既有 3 测试改写为
  fs 形状断言（id/label/children 恒存在/kind/is_leaf/排序）。→ AC-01

### T-02 后端 specs_text 端点

- 文件：`backend/crates/musk/src/spec_tree.rs`（路由+handler+tests）、
  `src/back/api.at`（声明）。
- 内容：`GET /api/specs/text/{*path}` → `Json<FilesTextResponse>`；
  `validate_path_pub` 守卫 + 根 `docs/specs/`；读失败→
  `{content:"", error:<msg>}`（200 包裹诚实契约，files_text 同口径）；
  api.at 增 `specs_text` 声明（返回复用 `FilesTextResponse` 类型）。
- 依赖：无（与 T-01 并行）。
- 验证：`cd backend && cargo test -p musk specs_text`——新增 3 测（存在/
  不存在/路径穿越拒绝）。→ AC-02、AC-03

### T-03 前端规范树+正文接线

- 文件：`src/back/api.at`（specs_tree 返回类型）、`src/front/specs_store.at`、
  `src/front/specs_view.at`。
- 内容：store 字段换型；LoadSpecFile 走 specs_text 双臂 try/catch +
  loading/failed 标量；视图正文区 `.md` 判定（active path 后缀）→
  `Markdown(source: .store.active_spec_file_content)`，非 md → font-mono
  纯文本兜底；loading/failed/未选中/空树四分支；FileTree `default_expanded`
  顶层目录 id。
- 依赖：T-01、T-02。
- 验证：`npm run build`（gen vue 生产构建）+ `npx vitest run` 全绿；
  冒烟见 T-06。→ AC-01、AC-02、AC-03

### T-04 计划页状态徽标 i18n

- 文件：`src/front/plans_view.at`（三处）。
- 内容：徽标 span 内 if-chain 五态静态键（drafting/executing/
  execution_done/reviewed/archived → plans.status*）+ else fallback 原值。
  资源已在 zh/en.json（E7），无新增键。
- 依赖：无（与 T-01..03 并行）。
- 验证：构建绿 + 冒烟目验（zh/en 各一轮）。→ AC-04

### T-05 计划详情同步刷新

- 文件：`src/front/plans_store.at`（PlansLoaded）。
- 内容：回填段尾部 `.current != None` → `LoadPlan(.current.seq)`（保留既有
  "无选中自动选首篇"逻辑；两支互斥不双拉）。
- 依赖：无（与 T-01..04 并行）。
- 验证：vitest 回归 + 冒烟（改盘上 plan 文件→切走→切回，正文更新；编辑中
  切走切回，编辑草稿不丢、保存/取消后显示新正文）。→ AC-05

### T-06 双轨门禁+冒烟收口

- 内容：全量门禁——`cd backend && cargo test -p musk`（相关域）/
  `npm run build` / `npx vitest run` / `node scripts/ui-parity.mjs check`
  （workspace-ui.md 验证条款）；VM 轨冒烟：规范树展开/折叠、md 正文渲染、
  计划页徽标中文、切视图正文刷新。VM 若遇已登记引擎基线债阻断，按
  workspace-ui.md 验证条款保留复现命令+错误+受影响范围，不得把 web 结果
  表述为双端通过。
- 依赖：T-01..T-05。
- 验证：门禁全绿 + 冒烟记录入执行步骤勾选注记。→ 全 AC

### 规范增量

| delta_id | 操作 | 目标 | before/after | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/modules/workspace-ui.md`（规范条目） | before："文件模式显示规范树和正文。加载中、加载失败、无内容是三个独立状态。" / after：细化——"规范树为嵌套可折叠树（目录展开/收起）；md 文件正文经 Markdown 渲染，非 md 文件纯文本降级；正文加载失败为独立可见状态（独立于树的加载状态）。" | docs/specs/ 是 canonical 知识层，树模式是其展示门面；schema/渲染契约需要成文 | AC-01/02/03 |
| SD-02 | modify | `docs/specs/modules/workspace-ui.md`（计划条目） | before："内容区展示计划正文、状态和操作。" / after：追加——"状态徽标使用本地化文案（中英资源随语言切换，未知状态回退原始值）；进入栏目时选中计划的详情正文与磁盘内容同步。" | 065-F1 定型的 i18n 条款（workspace-ui:31）在计划页的落地口径 | AC-04/05 |

变更无 Spec 删除/退役；`TreeNode` 契约变化属于内部 API 形状修正，由
SD-01 的行为条款覆盖，不单独立 delta。

## 6. 测试设计

- **后端单测**（`backend/` 下）：spec_tree 既有 3 测改写 fs 形状断言 +
  新增 specs_text 3 测（存在/404/穿越）。锚定：响应形状回归防复发。
- **前端测试**：vitest 既有套件回归（i18n 键存在性门禁如已覆盖 plans.status*
  消费则自然生效；如未覆盖，不新增门禁——静态键 if-chain 由冒烟兜底）。
- **冒烟清单**（T-06）：①树：展开/折叠目录、guides 缩进、folder/file-text
  图标、点 md 文件渲染标题/列表/代码块、点 .at 等非 md 文件纯文本；
  ②失败态：选中被删文件（或改名后旧路径）→ 错误提示非白屏；③计划页：
  zh 徽标"草拟/执行中/…"、en 切换跟进、未知状态回退；④同步刷新：外部
  追加 plan 正文→切视图往返可见；编辑草稿跨往返保留。

## 7. 验收标准

| ID | 行为 | 验证方法与预期 |
|---|---|---|
| AC-01 | 规范文件树渲染嵌套可折叠树 | 冒烟：modules/ 目录可展开出子文件（chevron 翻转、guides 缩进、folder 图标），点击行为正确；`cargo test -p musk spec_tree` 绿（fs 形状断言） |
| AC-02 | md 正文 Markdown 渲染 | 冒烟：选 `docs/specs/modules/plan-flow.md`，标题/表格/代码块经 autodown 渲染（与规范详情页同管线）；非 md 文件纯文本兜底可读 |
| AC-03 | 正文加载失败独立态 | 冒烟：请求不存在文件 → 树不受影响、正文区显示失败说明（非白屏非旧文残留误导）；`cargo test -p musk specs_text` 绿 |
| AC-04 | 状态徽标本地化 | 冒烟：zh 下列表/归档栏/详情 chips 显示"草拟/执行中/执行完成/已复审/已归档"；en 显示 Drafting 等；未知状态字符串原样回退 |
| AC-05 | 详情正文与磁盘同步 | 冒烟：向选中 plan 文件追加一行→切到会话再切回计划页→正文含新行；编辑态往返草稿不丢。代码路径：PlansLoaded 尾部 LoadPlan(current.seq) |

## 8. 执行步骤

- [x] T-01 后端 spec_tree schema 切换（fs 形 FilesNode + 测试改写）
  [✅ 2026-09-26] spec_tree.rs 重写：build_spec_tree 产 FilesNode（复用
  files_browser::FilesNode；folders-first+字母序、dot/manifest 跳过、叶子
  children=[]），响应 `Vec<FilesNode>`。证据：`cd backend && cargo test
  -p musk spec_tree` → 5 passed 0 failed（含新增 spec_text_confined_read_
  contract/spec_text_response_shape_wraps_content）。
  **[review 补正 R-1]**：本任务漏改集成 parity 套件
  `tests/parity_spec_tree.rs`（计划清单只点名单元测试）——nextest 全量
  抓到后随 bcaf16a 更新断言至 FilesNode 形+补 specs_text 集成用例。
- [x] T-02 后端 specs_text 端点 + api.at 声明
  [✅ 2026-09-26] `GET /api/specs/text/{*path}` → `Json<FilesTextResponse>`，
  复用 files_browser 新抽 `read_text_confined`（resolve_confined 升
  pub(crate)+files_text 同步重构单源）；api.at 增声明+specs_tree 返回类型
  切 `[]FilesNode`。证据：同上测试 + `cargo test -p musk files_browser` →
  7 passed 0 failed（重构后回归）。
- [x] T-03 前端规范树+正文接线
  [✅ 2026-09-26] specs_store 换型 `List<FilesNode>`+SelectFile/SetContent/
  SetLoadFailed/ClearFile 三态组+spec_tree_default 顶层目录；specs_view 正文
  四分支（loading/失败/md→Markdown `source`+`streaming:false`/其余 font-mono
  兜底）+FileTree default_expanded+空树 specs.emptyTree（既有键，零新增
  i18n）；目录点击经 specs_helpers 新增 `spec_tree_node_kind` 拦截。
  **执行适配两件**：①specs_view 新 store 消息改限定名 `SpecsStore.*`——
  SelectFile/SetContent/SetLoadFailed 与 FilesStore 撞名，VM multi-store
  消歧（plan-446 A1）在 vm-link-probe 抓到 4 错后限定，复跑 PASS；②
  mention_input.at `.Input` 的 @spec 文件清单消费 specs_tree 旧 TreeNode
  字段（type/path/name）→ vue-tsc 抓到 TS2339，适配 FilesNode
  kind/id/label（push 的 {path,name} 键=@spec token 契约不变；顶层遍历
  口径维持，嵌套文件收录登记 §10）。证据：gen-only 60 组件生成、gen
  SpecsView.vue 含 Markdown/spec_tree_node_kind、 PlansView.vue 三处
  statusDrafting；vue-tsc+vite build 绿。
- [x] T-04 计划页状态徽标 i18n
  [✅ 2026-09-26] plans_view 三处（列表/归档栏/详情 chips）if-chain 五态
  静态键+fallback 原值；资源复用既有 zh/en plans.status* 键（零新增键，
  i18n 门禁套件绿）。证据：gen PlansView.vue:189/231/305。
- [x] T-05 计划详情同步刷新
  [✅ 2026-09-26] PlansLoaded 回填段尾部 `.current != None →
  LoadPlan(.current.seq)`（与无选中自动选首篇互斥；plans_get 同步调用与
  既有链路同面）。证据：vitest 回归绿；AC-05 行为待 review 阶段实机走查
  （本会话未起 VM/web 交互冒烟，见 T-06）。
- [x] T-06 双轨门禁+冒烟收口
  [✅ 2026-09-26] 门禁全绿：`cargo test -p musk spec_tree` 5/5 +
  `files_browser` 7/7（backend/ 下）；gen-only 60 组件零错误；vue-tsc +
  vite build 绿（pnpm install——worktree 冷装必须 pnpm，npm 装 file: 依
  赖缺 lowlight 致 rollup resolve 失败，主检出 .pnpm 布局同证）；vitest
  29+1skip；`node scripts/ui-parity.mjs check` catalog PASS（live 0/4=
  既有缺 live 回执非本计划引入）；vm-link-probe **PASS 96205 bytes**
  （WARN≥90000 趋势告警，非新增红线；FAIL 阈 131072 未触）。API 冒烟
  （隔离 MUSK_CONFIG_DIR + 127.0.0.1:8791 debug serve）：GET
  /api/specs/tree 27 节点/4 目录/叶子 children 恒 []；GET
  /api/specs/text/modules/plan-flow.md → `{content:3209 字符 UTF-8,
  error:""}`；不存在文件→404；`..%2F` 穿越→400；目录→400。
  **交互冒烟（树展开视觉/徽标本地化目验/AC-05 切视图实测）留 review
  阶段**——serve 复现命令已验证可用：worktree 根
  `MUSK_CONFIG_DIR=<worktree>/tmp/p091-smoke-cfg ./backend/target/debug/
  musk.exe serve --addr 127.0.0.1:8791`（默认 workspace root=启动 CWD，
  即 worktree 自带 docs/specs）。

## 9. 复审记录

- 2026-09-25 stage:new（draft handoff，plan_revision 1）：背景调查十项证据
  核实到行（E1-E10）；T-01..T-06 覆盖 AC-01..05 与 SD-01/02；无阻塞裁决
  （待澄清两项均不阻塞 work 起步）。outcome: pass → next: work。
- 2026-09-26 stage:work | plan_id: PLAN-091 | plan_revision: 1 |
  outcome: **pass** | code_commit: plan-091-dev **d358dd2**（基线 e10f9f7）|
  task_ids: T-01..T-06 全勾 | evidence: 六门禁绿（cargo spec_tree 5+files_
  browser 7 / gen-only 60 组件 / vue-tsc+vite / vitest 29+1skip / ui-parity
  catalog / vm-link-probe PASS 96205B）+ API 冒烟四态过（serve 命令已验证）|
  blockers: 无 | next: **review**（交互冒烟项见 T-06 注记：树展开视觉/
  徽标本地化目验/AC-05 切视图实测；SD-01/SD-02 已落 worktree
  docs/specs/modules/workspace-ui.md 随 d358dd2 提交，待 review 复核发布）。
  worktree 组 musk-091（auto-musk plan-091-dev + 只读依赖 worktree
  auto-ai/auto-lang/auto-down 各默认分支，消费即清理原则由 merge 收尾）。
- 2026-09-26 stage:review（自审声明：与 work 同会话，裁定从工件重建——
  提交 diff 通读 + 独立复跑 + IAB 实机走查，不依赖执行期摘要）|
  plan_id: PLAN-091 | plan_revision: 1 | base: e10f9f7 |
  dependency_revisions: auto-ai 79ff93a / auto-lang 729dd4f2f / auto-down
  3373a5c | spec_inputs: workspace-ui.md 增量 cdb4e420（worktree，未发布）
  vs canonical 2e4296e1 |
  **acceptance_results（IAB 实机走查 http://127.0.0.1:8791，隔离
  MUSK_CONFIG_DIR，serve=worktree debug exe + 新构建 dist）**：
  AC-01 **pass**（文件树嵌套渲染：goals/modules/reports/reviews 四目录+
  modules 16 文件，folder/chevron/guides/file-text 图标齐，顶层目录默认
  展开——快照+截图双证）；AC-02 **pass**（点 modules/plan-flow.md → 全量
  Markdown DOM：h1/blockquote/h2/**table**（四相位表）/list/inline code）；
  AC-03 **pass**（API 四态：happy `{content,error:""}` UTF-8 3209 字符/
  404/穿越 400/目录 400 + UI 目录点击回落"从树中选择一个文件"）；
  AC-04 **pass**（API 建走查计划 drafting → 列表徽标"草拟"+详情 chip
  "草拟"+流转按钮 草稿/执行中/待复审/已复审 全本地化）；AC-05 **pass**
  （盘上追加 AC05-SYNC-MARKER-20260926 → 切会话再切回 → 标记行出现在
  正文 DOM——PlansLoaded 尾部 LoadPlan(current.seq) 生效）。
  **findings**：**R-1（P1，已修复）**——nextest 全量套件抓到
  `parity_spec_tree::spec_tree_returns_nested_tree` 仍断言旧 TreeNode 形：
  work 阶段 scoped 验证（`cargo test -p musk spec_tree` 单元过滤）漏掉
  集成 parity 二进制，计划 T-01 文件清单也只点名了单元测试。修复=bcaf16a
  （断言改 FilesNode 形+语义不变+补 spec_text_returns_json_wrapper 集成
  用例）。修复后：nextest spec_tree 过滤 7/7 绿；全量 `--no-fail-fast`
  **714/715 passed**（唯一红=tool_atoms 084-D6 既有基线，KNOWN-DEBT 在案，
  非本计划引入）。
  **证据边界（如实登记）**：AC 验证均在 web 轨（IAB 实机）；VM 轨=链接
  门禁绿（vm-link-probe PASS 96205B @ 最终源）+双轨单源同码，VM 运行时
  目验未执行（时间盒选择，非引擎债阻断）——不将 web 结果表述为双端通过，
  VM 实机走查留 merge 前用户窗口（复现：worktree 根 `MUSK_CONFIG_DIR=<
  wt>/tmp/p091-smoke-cfg ./backend/target/debug/musk.exe serve --addr
  127.0.0.1:8791` + launch-vm 链）。非 md 文件纯文本兜底臂=代码级验证
  （走查未点非 md 文件，行为与改前同形）。
  规范增量复核：SD-01/SD-02 文本与实测行为逐条吻合（含"顶层目录默认
  展开"）；touched_goals=[goal-spec-knowledge] 真实存在（goals/README
  #2）；无 canonical 冲突（主检出 workspace-ui.md 自基线未动）。
  outcome: **pass**（R-1 修复后） | reviewed_commit: plan-091-dev
  **bcaf16a** | next: **merge**。
- 2026-09-26 stage:merge | **PLAN-091:r1** 收据（五 checkpoint 全过）：
  **prepared** — reviewed 基线 bcaf16a；规范增量 SD-01/02 已随 d358dd2 落
  worktree docs/specs/modules/workspace-ui.md（review 复核过文本与实测
  吻合）；交付提交 = d358dd2（主体）+ bcaf16a（R-1）。**landed** —
  worktree 恒等 rebase（main 未移动，哈希零变化，免 range-diff）+
  `git merge --ff-only plan-091-dev` → main tip = **bcaf16a** 零 merge
  commit；主检出已知良好化：nextest spec_tree 7/7 绿（途中排掉两环境
  波折：os error 1455 页面文件瞬态=并行构建峰值挤占、LNK1207 PDB 损坏=
  中断残留，删 musk-*.pdb 重建即愈——非代码问题）+ 主检出 gen-only
  重生成 + pnpm build 绿 13.70s（dist 已部署面）。**ledger_refreshed** —
  workspace=auto-musk 主检出 `.autoos/specs.json`（运行时账本，
  gitignored）：现行知识条目 `workspace-ui-D1`（designs 区，file→
  docs/specs/modules/workspace-ui.md）source_sha256 d8572d6b→
  **02ebe2cd**（离线读-改-写原子落盘，8090 无监听=独占写，回读验证）；
  其余 5 区/条目未触碰。**archived** — 本文件移入 docs/plans/archived/
  并置 status: archived，completion_kind: delivered。**cleaned** —
  wt-guard 复跑 clean 后 `git worktree remove` musk-091/auto-musk +
  `git branch -d plan-091-dev`（was bcaf16a，已含于 main）；三个只读依赖
  worktree 同组拆除（auto-ai 附带临时分支 auto-ai 删除 @79ff93a /
  auto-lang detached @729dd4f2f / auto-down detached @3373a5c——三者主
  检出零改动）；组目录 `.wt/musk-091` rmdir 空 removal ✓。
  **merge 观察项（登记不阻塞）**：主检出构建出现既有 SCHEMA_DRIFT 警告
  （`Markdown(source)` prop 未入 autodown schema 声明面——specs_detail
  自 PLAN-041 起五处同用，且本次 review IAB 走查实证 source 运行时正常
  渲染），非本计划引入，归上游 schema/component 漂移族另录。

## 10. 待澄清事项

1. **specs_text vs 复用 files_text**：✅ 执行已裁定（T-02 落 specs_text，
   根收窄 docs/specs、复用 read_text_confined 单源）。
2. **树默认展开态**：✅ 执行按顶层目录展开落（spec_tree_default），冒烟
   可调不构成契约变更。
3. `specs_get_file`（raw）保留（浏览器直链面）；未来若有图片类规范资产
   消费再评估，本计划不处置。
4. **[执行登记] @spec 文件清单仅顶层遍历**：mention_input.at 的 @spec/
   收录沿既有口径只走 specs_tree 顶层（嵌套 modules/*.md 不入 @ 候选）——
   旧 schema 时代即如此（非本计划回归）。递归收录属行为增强，归后续
   小计划/后续批次，不阻塞本计划 AC。
5. **[执行登记] worktree 冷装必须 pnpm**：gen/front/vue 以 pnpm 布局安装
   （主检出 .pnpm/lowlight@3.3.0 同证）；npm 装 file: 依赖漏 vendor
   engine 的 transitive lowlight → rollup resolve 失败。后续 worktree
   构建直接 `pnpm install`。
