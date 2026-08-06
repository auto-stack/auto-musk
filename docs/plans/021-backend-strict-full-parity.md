# 021 — Auto 后端"严格 100%":残留 hw 端点 + 行为差异 + 测试盲区闭环

> **状态**:🟡 进行中(2026-08-07 启动)。
> **前置**:Plan 020(已归档,业务端点 100% ag handler;Phase G/H 闭合 relay_driver 核心循环 + TaskPlan 执行内核)。
> **仓库**:auto-musk(`backend/crates/musk/`)+ 可能的 auto-ai(Phase C2 的 RoleRegistry)+ 可能的 auto-lang(wiki modified 降级时的 D7 follow-up)。
> **目标**:闭合 Plan 020 审查发现的 4 类"未达严格 100%"残留,使后端在**代码层 ag/hw 等价 + 全端点测试覆盖**两个维度达到 100%。

---

## 0. 为什么需要本计划

Plan 020 把后端业务 HTTP 端点 100% 切到 ag handler,但"Auto 后端严格 100%"审查发现 4 类残留:

1. **`/api/files/{workspace_id}/{*path}`** —— 唯一残留的 hw 业务端点(`server.rs:147,718`),且**完全无 HTTP 测试**。这是 `display_image` 工具(`tools.rs:642`)生成 URL 的依赖端点;其 canonicalize 越界沙箱逻辑的回归无任何测试保护。
2. **2 个用户可见行为差异**:`AAID_URL` env 覆盖在 ag 路径不生效(KNOWN-DEBT 第 27 行);wiki tree 节点 `modified` 恒 None(KNOWN-DEBT 第 28 行)。
3. **HTTP 测试盲区**:`/api/chats/sessions` DELETE + 3 个 specs 端点(经 workspace,`tmp_state` 已隔离,单纯遗漏);5 个写真实 `~/.config/autoos` 的端点(roles/app-config/harness PUT/DELETE,因副作用隔离缺失而未测)。
4. **4 个休眠镜像**(tools/spec_tools/orch_tools/server_serve)的处置决策。

**用户目标口径区分**:
- "用户可见功能无阻断 bug" —— Plan 020 已达成(主流程全通)。
- "代码层 ag/hw 完全等价 + 全端点测试覆盖" —— 本计划达成。

---

## 1. 差距盘点(2026-08-07 调研实测)

| 类别 | 现状 | 风险 |
|---|---|---|
| `/api/files` | hw handler(`server.rs:718`),无 HTTP 测试 | 🔴 沙箱逻辑回归无保护 + 唯一残留 hw 业务端点 |
| AAID_URL env | ag 桩硬编码(`extern_impl.rs:564`),hw 已读 env | 🟡 边缘(仅 env 自定义 daemon URL 时);实际消费路径已走 hw |
| wiki modified | ag 恒 None(`wiki.at:560`) | 🟢 轻微(前端不显示文件修改时间;size 已正确) |
| workspace 端点测试 | 4 个端点(chats DELETE + 3 specs)单纯遗漏 | 🟢 无污染,直接补 |
| config 端点测试 | 5 个端点写真实 `~/.config`,路径硬编码 | 🟡 副作用隔离缺失 |
| 4 休眠镜像 | tools/spec_tools/orch_tools/server_serve 未激活 | 🟢 生产硬编码 hw tools,零功能影响 |

---

## 2. 目标与验收标准

1. **代码层 ag/hw 等价**:serve() 业务端点 100% ag(`/api/files` ag 化,workspace_file 删除);2 个行为差异修复(AAID_URL/wiki modified)。
2. **全端点测试覆盖**:所有 `/api/*` 生产端点有 HTTP 层测试。
3. **休眠镜像决策记录**:4 镜像留待理由写入计划 + KNOWN-DEBT(不实施)。
4. **无回归**:全量测试绿;re-transpile 零 drift。

---

## 3. 实施阶段

### Phase A:`/api/files` ag 化 + HTTP 测试(消除唯一残留 hw 业务端点)

hw `workspace_file`(`server.rs:718-738`)< 30 行:纯文件 I/O = `ws.root.join(path)` + `canonicalize` + 越界 `starts_with` 检查 + `fs::read` + `guess_mime`。**复用 `crate::wiki::guess_mime`(`wiki.rs:392`,pub(crate),wiki raw_file 先例)**。

- **A1**:`extern_impl.rs` 新增 `workspace_file_do(s, workspace_id, path) -> Response`(委托 hw 逻辑);`server.at` 加薄 handler `pub fn workspace_file(s, p) ~Response`;`server.rs:147` 路由切 ag;删 hw `workspace_file` + 私有 `guess_mime_from_path`。
- **A2**:`tests/parity_files.rs`(新建,hw vs ag 双 router):读文本 → 200 + Content-Type;不存在 → 404;越界 `../../etc/passwd` → 403(沙箱锚);子目录 → 200。
- **验收**:parity_files 绿;serve() 业务端点 100% ag;全量测试无回归。

### Phase B:行为差异修复(AAID_URL env + wiki modified)

- **B1(AAID_URL)**:`extern_impl.rs:564` 桩改委托 hw `MuskAppConfig::load().effective_daemon_url()`;`tests/parity_app_config.rs:166-169` 分叉断言改一致(注意 env 竞争串行化)。
- **B2(wiki modified)**:`wiki.at:509` 仿 `file_size` 抽 `file_modified(entry) -> Option<u64>`(显式 `let secs u64 = ...` 抑制 cast);`wiki.at:560` file 节点 `modified: None` → `modified: file_modified(entry)`;`tests/parity_wiki.rs` 改断言。
  - **降级路径**:若 re-transpile 后 `as_secs()` 仍 cast 失败,回退 `modified: None` + KNOWN-DEBT 新增 auto-lang D7 follow-up,不阻塞 021。
- **验收**:parity_app_config 两侧行为一致;parity_wiki file modified 非 None(或降级登记);KNOWN-DEBT 第 27/28 行移除或降级。

### Phase C:HTTP 测试盲区补齐

- **C1(workspace 端点,无基础设施改动)**:`/api/chats/sessions` DELETE(server.rs:1121 漏挂)+ `/api/specs/drift-check|rebuild-relations|related`(逻辑层在 parity_specs.rs 已覆盖,补 HTTP 接线层)。
- **C2(config 端点,AppState 加 config_root)**:
  - `AppState`(`server.rs:58`)加 `pub config_root: PathBuf`;`serve()` 填真实 `~/.config/autoos`,`tmp_state()` 填 temp dir。
  - 路径函数接收 root:`app_config.rs:15 musk_config_path_under(root)`、`server.rs:463 app_harness_dir_under(root, kind)`。
  - handler 改从 `State<AppState>` 取 `config_root`:`app_config_write`/`harness_save`/`harness_delete`。
  - **roles 跨 crate(auto-ai)**:给 `auto_ai_agent::RoleRegistry` 加 `load_from(dir)`(轻量,不破坏现有 API),`role_save_of`/`role_delete_of` 从 `state.config_root` 传入。**用 worktree 实施(auto-musk 前缀)**。
  - 5 端点用 `tmp_state()`(config_root=temp)+ oneshot 测。
- **验收**:HTTP 测试覆盖全部生产端点;全量测试无回归。

### Phase D:休眠镜像记录为"未来待做计划"(不实施)

调研结论:4 镜像激活成本极高(本地 trait Tool 不兼容 / orch_tools 缺 2 工具 / descriptions 缩水 / 需跨 auto-lang 改转译器)、收益为零(生产硬编码 hw tools,无切换路径)。本计划**只记录**:

- 本计划 §"未来待做计划"章节列出 4 镜像 + 留待理由。
- `KNOWN-DEBT-AND-RISKS.md` 第 38 行更新:4 镜像留待未来,理由记录于 Plan 021。
- 不改镜像文件(避免改 tracked 生成物)。

---

## 4. 关键架构决策

1. **`/api/files` 纯 extern 委托**:逻辑 < 30 行无业务逻辑,复用 `wiki::guess_mime`,不强行纯 .at 表达(canonicalize/starts_with/Response 构造是 a2r 边界)。
2. **AAID_URL 改 extern 桩而非 .at**:真正消费路径已走 hw;只需把休眠骨架的桩也对齐 hw,最小改动。
3. **wiki modified 用 store 路径辅助 fn**:391 D1 已验证可行的范式(显式 `let u64` 抑制 cast);不依赖 auto-lang 改动。
4. **config 测试用 AppState 注入**:最干净,与现有 `tmp_state()` 模式一致;roles 跨 crate 加方法不破坏 API。
5. **休眠镜像不激活**:成本/收益比极差,记录留待。

---

## 5. 风险

- 🟡 wiki modified 的 `as_secs()` cast 可能仍失败(表达式位置)→ 有降级路径,不阻塞。
- 🟡 AppState 加 config_root 影响所有 tmp_state 调用点 → 字段加默认值,既有测试不受影响。
- 🟢 roles 跨 crate(RoleRegistry::load_from)→ 加方法不破坏 API,用 worktree 隔离。

---

## 6. 与 KNOWN-DEBT 的关系

本计划闭环时:
- **移除**第 27 行(AAID_URL,B1 修复)。
- **移除或降级**第 28 行(wiki modified,B2 修复则移除 / 降级则改注 D7)。
- **更新**第 38 行(休眠镜像,Phase D 记录)。
- **新增**(若 B2 降级)auto-lang D7 follow-up。

---

## 7. 里程碑

| 里程碑 | 内容 | 验收 |
|---|---|---|
| M1(Phase A) | /api/files ag 化 + HTTP 测试 | parity_files 绿;serve() 业务端点 100% ag |
| M2(Phase B) | AAID_URL + wiki modified 修复 | parity_app_config 一致;parity_wiki modified 非 None(或降级) |
| M3(Phase C1) | workspace 端点测试补齐 | chats DELETE + 3 specs 端点有 HTTP 测试 |
| M4(Phase C2) | config 端点测试(AppState config_root) | 5 config 端点有 HTTP 测试 |
| M5(Phase D) | 休眠镜像记录 | 留待理由写入计划 + KNOWN-DEBT |

---

## 8. 未来待做计划(本计划不实施)

### 休眠镜像激活(tools/spec_tools/orch_tools/server_serve)

**留待理由**(调研 2026-08-07):
- **本地 trait Tool 不兼容**:4 文件都定义本地 `trait Tool`(签名 `fn name(&self) -> String` 等),与 `auto_ai_agent::Tool`(签名 `fn name(&self) -> &str`)不兼容。激活需改 a2r 转译器生成 `impl auto_ai_agent::Tool`,跨 auto-lang。
- **缺工具**:`orch_tools` ag 版仅 3 个,hw 有 5 个(缺 SpawnTaskPlan/RegisterTaskPlan);`tools` ag 版缺 DisplayImage。
- **descriptions 缩水**:ag 版是单行简述,hw 版是面向 LLM 的详尽描述(影响 agent 工具选择质量)。
- **execute 是 stub**:`tools` 的 RunCommand 调空 extern;`spec_tools` 返回空串/"ok";`orch_tools` 返回 "(stub)"。
- **收益为零**:生产 agent 硬编码 hw tools(`lib.rs:176-249`),无"切换到 ag tools"的代码路径。激活只是镜像完整度,无功能收益。

**何时考虑激活**:当 a2r 能生成兼容 `auto_ai_agent::Tool` 的 impl,且项目决定让 agent 工具也走 ag 驱动(而非 hw)时。届时需先评估 description 质量对 agent 行为的影响。

### wiki modified 的 auto-lang D7(若 Phase B2 降级)

若 `as_secs()` 表达式位置 cast 抑制需 auto-lang 改动:让 a2r 对方法链末端返回 u64 的 stdlib 方法(`as_secs`/`as_millis`)在表达式位置也抑制 `as i32` cast(391 D1 只覆盖 store 位置)。属 391 D1 延伸。
