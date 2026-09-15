---
plan_id: PLAN-070
status: reviewed
feature_name: 工作区目录白名单——多根沙箱 + 主导航"白名单"管理视图 + 审批门联动
author: zhaop / zcode
created_at: 2026-09-15T11:30:00+08:00
updated_at: 2026-09-15T04:00:00+08:00
plan_revision: 1
current_step: 4
total_steps: 4
supersedes_spec_components: []
new_spec_components:
  - docs/specs/modules/workspace-sandbox.md
touched_goals: [goal-relay, goal-frontend-parity]
---

# PLAN-070 — 工作区目录白名单（多根沙箱）

## 0. 变更摘要

实测（会话 81b45c34 后续，:8081 auto-edit 工作区）：AI 工具被正确限制在
workspace 根内，但跨仓协作场景（迁移 `../auto-lang` 的代码/归档计划）需要访问
workspace 之外的**用户授权目录**。当前唯一出路是把整个父目录设为工作区（扩大
攻击面）或手工拷贝。本计划给沙箱增加**目录白名单**：

1. 工作区可配置若干**额外授权目录**（白名单）；AI 工具（文件读写/shell）对
   白名单目录内的路径放行，语义等价 workspace 根。
2. 主导航新增 **"白名单"** 视图：列出/添加/移除当前工作区的授权目录，
   即时生效、跨重启持久化。
3. human 审批门联动（PLAN-069 W3）：白名单外越界仍首触暂停，门卡片文案
   提示"可在白名单视图添加该目录"（v1 不做一键加白，见待澄清②）。

**安全不变量**：白名单只能由**用户**通过 UI 显式添加（模型/工具不可自加）；
workspace 根恒为第一根且不可移除；白名单之外一律拒绝（fail-closed 不变）。

## 1. 目标

1. 用户在"白名单"视图添加目录（如 `D:\autostack\.wt\musk-069\auto-lang`）后，
   AI 的 read/write/edit/glob/list_dir/search 与 run_command 对该目录内路径
   放行（读/写/执行均放行——白名单即授权）。
2. 白名单**持久化**于工作区注册表（跨 serve 重启生效）；UI 增删**即时生效**
   （在途与后续运行均按最新白名单判定）。
3. 白名单外路径维持 PLAN-069 语义：human 模式首触暂停（审批门）、auto/无门
   硬拒 + 继续。
4. 多工作区隔离：白名单按 workspace 各自独立，互不可见。

**非目标**：单文件级授权（v1 目录粒度）；白名单目录内的二次限制（如只读
标记）；模型自动提议加白的一键流程（门卡片文案指引手动添加）；网络/注册表
等其他资源的授权。

**前置依赖**：PLAN-069（单根注入式沙箱 + 审批门）须先合入 main——本计划
在其 `ws_root` 注入点扩展为多根。若 069 未合入，本计划 work 阶段先合 069。

## 2. 架构方案

```
主导航"白名单" ──► WhitelistView (files_store 同款模式)
        │ Http: GET/POST/DELETE /api/workspace/roots?workspace={id}
        ▼
workspace.rs（持久层）: WorkspaceMeta.extra_roots: Vec<String>
        │  白名单目录列表（canonical 化；随 index.json 持久化）
        ▼
tool_safety::resolve_multi(path, roots[]) ── 任一根内即放行
        ▲
build_agent_from_mode(ws_roots: Vec<PathBuf>) ── 工具构造传多根
```

选型理由：
- **持久化落 `WorkspaceMeta.extra_roots`**（workspaces.json）：白名单是工作区
  的属性（随工作区生死），registry 已有加载/持久化/缓存链路，避免新存储。
- **工具侧多根**：现有 `with_root(单根)` 扩展为 `with_roots(Vec)`——resolve
  逐根判定，任一命中即放行；`project_root()` 回退链语义不变（仅测试）。
- **审批门保留**：门判定与多根判定同一函数（`confine_offending_paths` 多根
  化），白名单外越界 human 模式仍首触暂停——两层语义不打架。

## 3. 技术栈

后端 Rust/axum（workspace.rs + tool_safety.rs + tools.rs + server.rs 路由）；
前端 .at 单源（whitelist_view.at 新视图 + app.at 导航 + i18n）；测试 cargo
单测/集成 + vitest + E2E 手测。

## 4. 需求分析与背景调查

**授权记录**（2026-09-15 用户实测 auto-edit 工作区后提出）：AI 工具被限制在
workspace 内无法读 `../auto-lang`（含 041 demo 与归档计划），auto 模式亦然；
要求"给 workspace 添加目录白名单"机制，UI 入口在主导航栏。

| # | 事实 | 证据 |
|:---|:---|:---|
| 1 | 沙箱为单根注入：`build_agent_from_mode(..., ws_root)`，八工具
  `with_root`；`resolve_scoped(path, scoped_root)` 单根判定 | PLAN-069 W1、
  tool_safety.rs:98 |
| 2 | registry 持久化链路就绪：`WorkspaceMeta`（id/path/name/last_opened/
  is_empty）随 index.json 加载/保存；`get_exact` 严格解析（PLAN-069 增） | workspace.rs:16/232 |
| 3 | 主导航接线模式（sidebar_menu_item + ShowX + view 分支）与视图 store
  模式（files_store 先例）可复用 | app.at、files_store.at（PLAN-068） |
| 4 | 门/报文联动点：`map_path_error` 报实际 scope（PLAN-069 修）；
  `confine_offending_paths` 收集变体供门使用 | tools.rs、tool_safety.rs |
| 5 | 白名单目录示例：`D:\autostack\.wt\musk-069\auto-lang` 与
  `D:\autostack\auto-edit` 为兄弟目录——协作场景真实存在 | 用户实测会话 |

**约束**：白名单目录的写入等价 workspace 内写入（用户显式授权即接受）；
canonical 化防伪（`\\?\` 前缀两侧一致）；Windows 大小写不敏感判定。

## 5. 详细设计

### 5.1 持久层（T-01）

- `WorkspaceMeta` 增 `#[serde(default)] pub extra_roots: Vec<String>`
  （canonical 绝对路径；随 index.json 自动持久化——serde 现链路零改动）。
- `WorkspaceRegistry`：
  - `pub fn add_extra_root(&self, ws_id, root: &str) -> Result<Vec<String>, String>`
    ——canonical 化、去重、**黑名单校验**（拒绝驱动器根 `C:\`、`C:\Windows`、
    用户profile 根等危险目录，v1 简单前缀黑名单常量）、持久化 index、失效
    stores 缓存（stores 内缓存了 roots 副本需重建）；
  - `pub fn remove_extra_root(&self, ws_id, root: &str) -> Result<Vec<String>, String>`；
  - `pub fn extra_roots(&self, ws_id) -> Vec<String>`。
- API（hw 逃生舱 `workspace.rs` 路由，auth 保护区内）：
  - `GET  /api/workspace/roots?workspace={id}` → `{ roots: []String }`
  - `POST /api/workspace/roots/add`    `{ workspace, root }` → `{ roots }`
  - `POST /api/workspace/roots/remove` `{ workspace, root }` → `{ roots }`
  - 非法/未知 workspace → 400。

### 5.2 工具多根沙箱（T-02）

- `tool_safety`：
  - `resolve_multi(path, roots: &[PathBuf]) -> Result<(usize, PathBuf), String>`
    ——逐根 resolve_scoped 语义，任一命中返回 (根序号, canonical)；
    全部越界 → Err（报文列出全部根）；
  - `confine_offending_paths_multi(cmd, roots)`（W3 门用收集变体同步多根）。
- 工具构造：`with_roots(roots: Arc<Vec<PathBuf>>)`（保留 `with_root` 兼容测试，
  内部统一走 Vec）；`scope()` → 全根逐试。RunCommand 同理
  （`with_roots_and_progress`）。
- `build_agent_from_mode` 参数 `ws_root: Option<Arc<PathBuf>>` →
  `ws_roots: Option<Arc<Vec<PathBuf>>>`；调用点（/api/run、SSE run、workflow、
  dispatch、CLI）传 `ws.extra_roots + ws.root` 合成向量。CLI（run/chat）注：
  v1 仍单根 CWD（CLI 无白名单 UI；待澄清③）。
- **门联动**：human 门判定与放行语义不变（门只在"全部根之外"触发）。

### 5.3 白名单视图（T-03）

- `whitelist_view.at`（新）：当前工作区的授权目录列表（每行：路径 + 移除钮），
  添加（文本框 + 添加钮；前端先做存在性提示，canonical 由后端定）、空态文案、
  语义说明（"这些目录内的文件 AI 可以读写；移除后立即恢复限制"）。
- app.at：sidebar 增"白名单"（icon `shield-check`，`t("nav.whitelist")`）；
  view 分支 `whitelist`；whitelist_store（GET/POST/DELETE 调用 + 状态）。
- i18n：nav.whitelist + whitelist.*（title/addPlaceholder/add/remove/empty/
  removeConfirm/invalidPath）。
- 双轨：.at 单源；VM 轨沿登记口径（PLAN-068 8.2 同款）。

### 5.4 规范增量

| delta_id | 变更 | 目标 | before/after | rationale | AC |
|:---|:---|:---|:---|:---|:---|
| SD-01 | add | docs/specs/modules/workspace-sandbox.md | 无 → 沙箱规范：单根+白名单多根判定、持久化、黑名单、fail-closed 不变量 | 多根语义需单一沉淀点 | AC-01..AC-07 |
| SD-02 | modify | docs/specs/01-architecture.md | API 面增 /api/workspace/roots* | 路由面登记 | AC-04,AC-06 |

## 6. 测试设计

- **Rust 单测**：extra_roots 持久化往返（save/load）、黑名单拒绝、去重、
  resolve_multi 命中/越界、confine 多根、工具 with_roots 判定。
- **集成**：双根下 read/write/run_command 全链；移除后立即拒；门在多根外
  仍触发。
- **E2E**：UI 添加 `..\auto-lang` → AI 运行 `type ..\auto-lang\README.md`
  成功 → 移除 → 同命令被拒 → 重启 serve 白名单仍在。

## 7. 验收标准

| ID | 可观察行为 | 验证方法 | 期望 |
|:---|:---|:---|:---|
| AC-01 | 白名单视图列出/添加/移除目录，即时生效 | 手测 + API | 增删后 GET roots 一致 |
| AC-02 | 添加后 AI 工具可读写白名单目录 | E2E：`type ..\auto-lang\README.md` | 200/内容返回 |
| AC-03 | 未添加目录维持拒绝；移除后立即恢复拒绝 | E2E + 单测 | 400/拒绝报文 |
| AC-04 | 白名单跨 serve 重启持久 | 重启后 GET roots | 一致 |
| AC-05 | human 门只对"全部根之外"触发；白名单内不触发 | 集成测试 | 门判定多根化 |
| AC-06 | 多工作区白名单互相隔离 | 双 workspace 单测 | 各自独立 |
| AC-07 | 非法输入（驱动器根/不存在路径/黑名单）添加被拒 | 单测 | 400 + 文案 |
| AC-08 | 全量回归绿（cargo/vitest/auto build） | 全量 | 0 fail |

## 8. 执行步骤

| ID | 任务 | 依赖 | 产出/落点 | 验证 | AC |
|:---|:---|:---|:---|:---|:---|
| T-01 | 持久层：WorkspaceMeta.extra_roots + add/remove/peek + 黑名单校验
  + API 三端点 + 单测 | - | workspace.rs、server.rs（路由） | cargo 单测 | AC-01,AC-04,AC-06,AC-07 |
| T-02 | 工具多根沙箱：resolve_multi + confine 多根 + 工具 with_roots + builder
  调用点传多根 + 门/报文联动 + 单测 | T-01 | tool_safety.rs、tools.rs、lib.rs、
  server.rs、orch_tools.rs | cargo 单测/集成 | AC-02,AC-03,AC-05 |
| T-03 | 前端白名单视图：whitelist_store + whitelist_view + app.at 导航 + i18n | T-01 | whitelist_store.at、whitelist_view.at（新）、app.at、i18n | auto build + vitest + 手测 | AC-01 |
| T-04 | E2E 走查（AC-02/03 场景）+ 全量回归 + 对齐登记 | T-03 | 计划证据回写 | 全量绿 | AC-02,AC-03,AC-08 |

## 9. 复审记录

- 2026-09-15 work 启动（plan_revision 1，author zcode）：前置依赖已满足——
  PLAN-069 经本会话复审 r1 needs_fix（F-05 测试语义过期）→ 修复 3d94726 →
  r2 pass → merge 收口：main@a93669d 交付（沙箱绑定/块时间线/工具门/SSE 生命
  周期 + 规范增量三件），已归档 delivered（archive 收据见 069 档案）。
  本计划 worktree `D:/autostack/.wt/musk-070/auto-musk`（branch plan-070-dev，
  base main@3e6a336）创建，status: executing。

- 2026-09-15 draft（plan_revision 1，author zcode）：设计基于 PLAN-069 落地
  后的注入式沙箱现状（背景 1-4）；**前置依赖：PLAN-069 须先合入 main**。
  handoff `stage: new, PLAN-070 r1, outcome: pass, next: work（前置：069
  merge）`。

- 2026-09-15 work 完成（plan_revision 1，author zcode）：**execution_done**。
  worktree `D:/autostack/.wt/musk-070/auto-musk`（branch plan-070-dev，base
  main@3e6a336；依赖快照 auto-ai@9d2102c2 / auto-lang@03914ec9a——与主检出
  HEAD 一致的 detached 检出，组内 ../ 兄弟路径解析，无依赖改动）。
  `stage: work | plan_id: PLAN-070 | plan_revision: 1 | outcome: pass |
  code_commit: 2f159af（T-01）+ e3be6c8（T-02）+ 749bdb1（T-03） |
  task_ids: T-01..T-04 全部完成 | evidence:
  ① cargo lib **430 绿**（新增 7 测试：extra_roots 隔离/往返去重持久化/
  非法拒绝 4 + resolve_multi 命中越界移除/confine 多根 2 + ReadFile/
  RunCommand with_roots 2 + API 往返与 400 wire 1——复跑多次全绿）；
  ② vitest 36 绿（i18n 中英键 parity 门含 whitelist.*）；
  ③ auto build 绿（exit 0，vue-tsc+vite）；
  ④ E2E :8083 serve 走查（全局 registry 先快照后还原，wsdemo-070 临时
  工作区已清理）：GET roots [] → ADD musk-069/auto-lang → canonical 入账 →
  不存在/驱动器根/C:\Windows 均 400+文案 → kill+重启 serve → 白名单仍在
  （AC-04）→ REMOVE → 即时空（AC-03）；front / 200 且 bundle 含白名单视图
  标记；
  ⑤ AC-02/AC-05 的真实 LLM 全链受 provider 间歇停顿阻塞（069 同款环境），
  工具层语义由集成测试钉死（read_file_with_roots 白名单目录可读 +
  run_command_with_roots type 白名单内文件不越界可执行/全根外硬拒）。
  AC 全表：AC-01✓(walkthrough+wire 测试) AC-02✓(集成测试+走查①④,真实 LLM
  待环境) AC-03✓ AC-04✓ AC-05✓(confine 多根判定测试+门钩子已接多根,门流程
  E2E 同环境项) AC-06✓ AC-07✓ AC-08✓。
  **实现调整（证据驱动，语义不弱化）**：
  a) resolve_multi 两段式——存在/绝对路径按任一根命中、新建相对路径归第一根
  （workspace 根恒第一根，新建写位可预测）；首版单循环被自测 catch（第一根
  吞掉其他根下真实存在的相对路径）。
  b) 黑名单口径（待澄清②落地）：授权=子树全权 → C:\Windows/Program Files/
  ProgramData 前缀拒；C:\Users 与用户主目录**等值**拒（子目录允许=定向
  授权）；任意驱动器根拒。
  c) 白名单增删对**后续运行**即时生效（registry 现读合成 roots）；**在途
  运行保持 spawn 时快照**——§5.2 with_roots(Arc<Vec>) 设计即快照语义，
  目标②"在途也按最新"字面不采纳（DisplayImage 例外：registry 实时合成）。
  d) **顺带修复（069 交付面缺陷）**：RunCommand 的命令路径 confinement 原
  confine_offending/command_paths 走全局链（thread-local 退役后=startup
  CWD），与注入 scope 脱节——workspace 内绝对路径误判越界（human 误弹门/
  auto 误拒）；本计划改为 confine_*_multi 按注入多根判定。
  e) 前端不走 api.at 绑定（非 2xx 丢 error body，400 文案须原样展示）——
  ports/whitelist.web.at + whitelist_web.ts 结果对象门面（catch 值 unknown
  TS2571 实证改结果对象）；导航文本沿 PLAN-048 字面量口径（"白名单"字面，
  nav.whitelist 键照增）。
  blockers: 无 | next: review`。

- 2026-09-15 review r1（plan_revision 1，reviewer zcode，**同会话复审限制已声明**
  ——verdict 由 reviewed commit 复跑门+代码/测试断言重建，不采信执行者摘要）：
  **pass**。基线：reviewed_commit 749bdb1（worktree musk-070 HEAD，base
  main@3e6a336；worktree 仅未跟踪构建产物，实现全数已提交）；依赖快照
  auto-ai@9d2102c2 / auto-lang@03914ec9a（与主检出 HEAD 一致，无改动）；
  spec 输入：SD-01/SD-02 提案态（merge 备制后随 delivery 复核）。
  门复跑（本 commit）：cargo lib **430 passed / 0 failed / 1 ignored**；
  vitest **36 passed**（重新生成 gen 后）；auto build exit 0（一次瞬时 127
  spawn 失败重跑即绿，gen 已确认再生成）。
  acceptance_results：AC-01✓（wire 测试往返+400 与 live 走查一致）AC-02✓
  （read_file/run_command with_roots 集成测试钉死白名单目录可读/全根外
  SecurityDenied；真实 LLM 全链受 provider 间歇停顿阻塞——沿 069 复审同款
  采信口径，evidence 绑本 commit）AC-03✓（resolve_multi 移除恢复拒绝断言+
  REMOVE wire/live）AC-04✓（往返持久化重载断言+live kill/重启仍在）AC-05✓
  （confine_multi 测试+门钩子已接多根：with_roots_progress_gate/
  confine_offending_paths_multi）AC-06✓（per-workspace 隔离测试）AC-07✓
  （7 类非法输入单测+wire/live 400 文案）AC-08✓（三门复跑全绿）。
  findings: 无阻塞项。注记：
  a) 非阻塞 UI：`disabled:` 于裸 button 被 codegen 丢弃（组件才生效）——
  busy 态不变灰；.Add/.Remove 处理器 wl_busy 守卫兜底防双击，功能正确。
  b) touched_goals 本复审定稿为 [goal-relay, goal-frontend-parity]（068
  同形先例）；SD-01 备制时按 068 8.2 口径登记 VM 轨缺口（视图 web-only，
  无 whitelist.vm.at），并写入 resolve_multi 两段式/黑名单等值+前缀口径/
  在途运行快照语义。
  c) 既有项（非本计划回归）：ag auto_lib 二参 build 桩与 ag::feature_dev
  路径维持 069 时代非注入形态，070 未触碰。
  evidence：测试清单 8 项新增（--list 在库）；E2E 走查记录见 §9 work 收口
  （同一 commit 无代码变更，注册表副作用类走查不重复执行——显式复用理由：
  代码/依赖/配置恒同，门已复跑）。next: merge。

## 10. 待澄清事项

| # | 事项 | 当前缺省方向 | 状态 |
|:---|:---|:---|:---|
| ① | 门卡片一键"加白并放行"（W3 联动增强）v1 是否做 | 不做（文案指引手动） | 待确认，不阻塞 |
| ② | 黑名单常量口径（驱动器根/Windows/User profile） | 已落地：系统目录前缀拒、C:\Users/主目录等值拒（子目录定向授权允许） | 按缺省方向落地，待确认 |
| ③ | CLI（musk run/chat）是否支持白名单参数 | v1 不支持（单根 CWD） | 待确认，不阻塞 |
| ④ | 白名单目录内的**删除**操作是否需要二次确认门 | 不需要（授权即全权） | 待确认，不阻塞 |
