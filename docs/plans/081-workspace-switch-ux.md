---
plan_id: PLAN-081
status: drafting
feature_name: workspace 选择器显示名修正 + VM 轨切换后列表刷新链
author: [agent]
created_at: 2026-09-21T00:00:00Z
updated_at: 2026-09-21T00:00:00Z
plan_revision: 1
current_step: 0
total_steps: 5
supersedes_spec_components: []
new_spec_components: []
touched_goals: []
---

# PLAN-081 — workspace 选择器显示名修正 + VM 轨切换刷新链

## 0. 变更摘要

两条用户需求（2026-09-21，VM 桌面实机使用中提出）：

1. **显示名**：workspace 选择器只显示**最终目录名**（如 `auto-edit`），
   不再显示完整路径（截断后只剩路径开头，多个工作区无区分度）。
2. **切换刷新**：VM 版切换 workspace 后，会话列表 / 计划列表等不跟随
   刷新（Vue/web 版正常）。需修 VM 轨的刷新触发链。

改动集中在 auto-musk `src/front/`（workspace_selector.at / forge_store.at /
workspace_helpers.at / app.at 接线），后端与 auto-lang 原则上不动（auto-lang
仅在 T-01 定罪"VM 弹层列表不可用"且修因在上游时按条件任务 T-04 开同组
并排 worktree）。

**基线说明（重要）**：用户实跑二进制来自 `D:\autostack\.wt\musk-080\` 组
（launch-vm.cmd：musk-080/auto-musk 源 + musk-080/auto-lang 的 auto.exe，
后端 17201 复用门）。PLAN-080 仍 executing 未合回，其 T-02 已把
WorkspaceSelector 改为 store 单源（`ForgeStore.SetWorkspace`）且触发器显示
完整路径。**本计划 worktree 基于 `plan-080-dev` 顶端堆叠**（否则改动落在
无 SetWorkspace 链的 main 旧态，用户不可见且合并冲突）。

## 1. 目标

- VM 与 web 双轨的 workspace 触发器统一显示最终目录名；完整路径保留在
  悬浮提示（title）作为区分信息。
- VM 轨切换 workspace（列表点选 + "打开文件夹"两条路径）后：会话列表、
  消息面板、计划列表全部切到新 workspace；后续新会话/发消息落在新
  workspace（默认 query 重注入）。
- web/Vue 轨行为零回归（reload 主链保留）。

**非目标**：

- 不改后端 workspace 元数据（`WorkspaceMeta.name` 已是目录名，
  `backend/crates/musk/src/workspace.rs:317-331`；实跑索引
  `~/.config/autoos/workspaces.json` 各条目 name 均为目录名）。
- 不在 auto-lang 实现 VM 版 `location.reload()` 真语义（应用级重启超出
  本需求；用 musk 侧 store 刷新链替代，见 §2）。
- 不处理 PLAN-080 交接的 §10-9 静默崩溃、字体度量族等余项（另行跟踪）。
- Wiki/Whitelist 视图切换刷新仅在 T-01 摸底登记，非本轮必改项。

## 2. 架构方案

### 需求①根因（已定罪，静态证据）

用户实跑的 musk-080 worktree 版 `src/front/workspace_selector.at`（PLAN-080
T-02 引入）：

```text
pathOrName => if .store.workspace_path != "" { .store.workspace_path } else { .store.workspace_name }
wsDisplay => if .pathOrName != "" { .pathOrName } else { "选择工作目录" }
```

触发器**路径优先**（PLAN-080 T-02 时用户裁定"把当前的 workspace 路径列出
来"，落在了触发器上）；`.ws-name` 样式带 truncate，长路径截断后只显示
`\\?\D:\autostack\...` 开头。**本计划修订该裁定**（用户新指令：只显示最终
目录名；完整路径信息移到悬浮提示继续保留区分度）。

修法：`wsDisplay` 改为 **name 优先、path 兜底、占位文案殿后**；`currentTitle`
保持完整路径。弹层列表项本来就是 `text .w.name`（目录名），不动。

### 需求②根因（已定罪，静态证据链）

VM 轨切换 workspace 的链路与 web 的分叉点：

1. `location.reload()` 在 VM 是 **no-op stub**：auto-lang
   `crates/auto-lang/src/vm/codegen.rs:8571` 把 `location.reload` 映射到
   native `auto.dom.reload`，其实现 `crates/auto-lang/src/vm/native.rs:8517
   shim_dom_reload` 为空（注释明言"Browser page reload has no direct
   desktop equivalent; no-op stub"）。web 轨 reload 真实重载 → 全 store
   重挂，所以 web 正常。
2. VM 轨的 `workspace` 查询参数是**进程级默认项**，仅 boot 期注入一次：
   `src/front/ports/platform.vm.at platformRefreshAuth`
   （`Http.set_default_query("workspace", wid)`）。切换后
   localStorage 已更新但默认 query 滞留旧值——即便列表重拉也会打旧
   workspace。（web 轨是 fetch 拦截器每请求读 localStorage，天然正确。）
3. `ForgeStore.SetWorkspace`（worktree 版 forge_store.at:137-148）只回填
   `workspace/workspace_name/workspace_path` + 写 localStorage，**不触发**
   `LoadSessionList`；`PlansStore` 切换后零接线。

### 修法（musk 侧收敛，store 级刷新链）

以 `ForgeStore.SetWorkspace` 为"切换完成"单一事件（它已是 workspace 真值
单源）：

```text
Choose(id)/PickFolder ──> ForgeStore.SetWorkspace(meta)
                            ├─ id 变化：写 localStorage + 回填显示字段
                            ├─ 会话域清场（复用 SwitchSession 的清场模式：
                            │   session_id=""/messages=[]/current_gate/report_data/
                            │   errands/relays/task_plans/streaming=false/
                            │   current_draft/thinking/think_open/tool_open/
                            │   pending_msgs/在途 SSE 先收口）
                            └─ LoadSessionList()（session_id=="" 自动选首个会话）
                          platformRefreshAuth()   # VM 重注入默认 query；web no-op
                          PlansStore.Init()       # 计划列表重拉
                          location.reload()       # web 主链保留；VM no-op 无害
```

- **幂等性**：`SetWorkspace` 在 `meta.id == .workspace` 且真值已回填时只更新
  显示字段、不重拉（boot 链 `app.at Init → ws_resolve_current →
  SetWorkspace` 与 ForgeStore.Init 的 localStorage 引导通常同 id，避免启动
  双拉）。
- Choose 与 PickFolder 两条路径共用一个 helper fn（`ws_after_switch()`，
  落 workspace_helpers.at）防双份拷贝。
- `platformRefreshAuth` 从 `ports/platform.at` 引入 selector（web 实现为
  no-op，双轨安全——同文件已有 login.at 先例）。
- FilesView 已有进视图重拉（`app.at .ShowFiles → FilesStore.LoadTree()`），
  无需改；Wiki/Whitelist 的 VM 侧行为由 T-01 摸底登记。
- PlansStore 在 VM 的加载触发点存疑（PLAN-048 T5 登记"VM 子件 Init 不派发"
  vs 用户观察计划列表有显示），T-01 实测定案后决定 `PlansStore.Init()`
  接在 selector 是否足够，不足则在 app.at 增补派发点。

## 3. 技术栈

- 前端单源 .at（auto-lang 双轨编译：web=Vue/vite，VM=iced）。门禁：
  `auto build`（vue-tsc+vite 全 pipeline）、vitest、VM 实机（launch-vm.cmd
  同款启动：后端 17201 复用 + `auto run --render vm`）。
- VM 取证：080 同款 MCP 快照/截图臂；`AUTO_VM_STORAGE_FILE` 新鲜存储起局。

## 4. 需求分析与背景调查

**授权记录**：用户明示（2026-09-21）"接下来的所有修改需求都用一个新的计划
跟踪（先更新计划，再实施，仍然要用worktree形式做）"——授权建本计划并在
worktree 实施；后续本会话追加的修改需求并入本计划（plan_revision 递增）。
仓库范围：auto-musk 为主；auto-lang 仅当 T-01 定罪修因在上游时按 T-04 开
`D:\autostack\.wt\musk-081\auto-lang`（分支 auto-musk-dev，AGENTS.md 第三行
命名）。

**关键证据**（调查于 2026-09-21，主检出 ce32aa4 + musk-080 worktree 5899fff）：

- 显示根因：worktree workspace_selector.at `pathOrName` 路径优先（上文 §2）；
  主检出 main 版 selector（未含 T-02）仍显示 `.current.name`——差异本身
  证明改动须基于 080 分支。
- 刷新根因三连：shim_dom_reload no-op / 默认 query boot 期一次性 /
  SetWorkspace 无刷新（上文 §2，含行号）。
- 后端 name=目录名：workspace.rs open()/load()（`name: base_id` /
  `name: id`）；`~/.config/autoos/workspaces.json` 实况 8 条全为目录名。
- VM 0920 实机截图（tmp/musk-0920-screenshot.png，原生窗口标题 musk）：
  触发器当时显示 `auto-edit`（PickFolder/解析链回填的 name），证明
  name 显示形态即用户所求。
- 已知关联债：PLAN-080 forge_store.at Init 注释登记"VM 侧 back.api
  workspace_list 在 store Init 返回空 + 本地 JSON 数组索引/迭代求值异常
  （返索引，实测 "0"）"（= 记忆库"VM api 数组债"）。若该债仍在，
  **弹层列表在 VM 可能为空**，用户实际只能走"打开文件夹"切换——T-01
  实机定案，命中则 T-04 修复。

**与 PLAN-080 的关系**：080 保持 executing（T-04/T-05 未闭环）；其 T-02
"触发器显示完整路径（用户裁定）"被本计划修订，080 文档不改（历史记录），
修订依据记于本节。081 基于 plan-080-dev 顶端；收尾顺序：080 先合回 main →
081 rebase main → 081 合回（见 §8 步骤 6）。

## 5. 详细设计

### 显示名（需求①）

`src/front/workspace_selector.at` computed 区：

```text
wsDisplay => if .store.workspace_name != "" { .store.workspace_name }
             else if .store.workspace_path != "" { .store.workspace_path }
             else { "选择工作目录" }
currentTitle => if .hasWs { .store.workspace_path } else { "选择工作目录" }
```

- name 优先（后端恒给目录名）；path 仅作缺 name 的防御兜底；两者皆空保持
  引导文案。`pathOrName` 中间层删除（单层链在 080 已证稳，无需两级中转）。
- 弹层列表项 `text .w.name` 与 Check 判定不动。

### 刷新链（需求②）

1. `src/front/forge_store.at` `.SetWorkspace(meta)`：
   - `meta == None`：维持现状 no-op。
   - id 变化：现回填逻辑 + 会话域清场（字段清单见 §2；在途流先
     `stream_es` 收口再清）+ `.LoadSessionList()`。
   - id 未变：仅回填 name/path 显示字段（幂等，防 boot 双拉）。
2. `src/front/workspace_helpers.at` 新增 `fn ws_after_switch()`：
   `platformRefreshAuth()` + `PlansStore.Init()`（store 交叉调用先例：
   user_message.at:35）。若 T-01 定案 VM 需 App 级派发，则改为在
   app.at 增 `WorkspaceSwitched` 接线。
3. `src/front/workspace_selector.at`：`.Choose` 与 `.PickFolder` 在
   SetWorkspace 之后调 `ws_after_switch()`；`location.reload()` 保留
   （web 主链，VM no-op 无害）。
4. `platformRefreshAuth` 引入：selector 增
   `use.web platformRefreshAuth from "src/front/ports/platform.at"`。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/modules/ui-compositions.md | WorkspaceSelector 浮层契约无触发器显示规则与切换完成语义 → 增：触发器显示最终目录名（name 优先/path 兜底/占位殿后），完整路径在 title 悬浮；切换完成语义 web=页面 reload，VM=store 刷新链（SetWorkspace 幂等回填+会话域清场+LoadSessionList+PlansStore 重拉+默认 query 重注入） | 用户裁定修订（本计划 §4）；双轨行为契约化防再漂移 | AC-01/02/03 |
| SD-02 | modify | docs/specs/modules/vm-data-semantics.md | 未登记 location.reload VM 语义 → 增：VM 轨 `location.reload()` = no-op（auto.dom.reload shim 事实源），需"整页刷新"语义的功能必须显式走 store 级刷新链，禁止依赖 reload | 本次根因；防后续功能重蹈 | AC-02/03 |

无 Spec 影响的说明：后端零改动，workspace-sandbox.md 不动。

## 6. 测试设计

- **静态门**：`auto build`（vue-tsc+vite 全 pipeline）绿；vitest 存量套件
  绿；若动 auto-lang 则 `cargo test -p auto-lang` 相关族。
- **VM 实机**（主验收场，080 同款启动：后端 17201 复用 +
  `cd .wt/musk-081/auto-musk && AUTO_REUSE_BACKEND=1 AUTO_HTTP_PORT=17201
  AUTO_HTTP_BASE=http://127.0.0.1:17201 RUST_MIN_STACK=16777216
  <musk-080 auto-lang>/target/release/auto.exe run --render vm`）：
  - 切换两轮（如 auto-edit → auto-musk → auto-edit），逐轮截图/MCP 快照：
    触发器显示名、会话列表内容、消息面板、计划列表。
  - PickFolder 路径一轮（系统对话框选目录）。
  - 切换后新建会话 + 发一条消息，确认落新 workspace（列表回到 web 版或
    重进视图可见新会话在新 workspace 名下）。
- **web 回归**：浏览器（vite dev 或 dist）同流程一轮——reload 后一切如旧。
- 证据落 `docs/reports/ui-parity/081-*.md`（截图 + 快照 + 命令输出）。

## 7. 验收标准

- **AC-01** 双轨触发器只显示最终目录名（如 `auto-edit`），无路径前缀；
  悬浮 title 显示完整路径。验证：VM 截图 + web 实测。
- **AC-02** VM 弹层点选切换后：触发器、会话列表、消息面板（自动选中首个
  会话）全部切到新 workspace；旧 workspace 会话不再显示。验证：实机两轮
  切换截图对比。
- **AC-03** VM 切换后计划列表刷新为新 workspace 的计划（含详情默认加载
  首篇）。验证：实机截图。
- **AC-04** VM 切换后新建会话/发消息落在新 workspace（默认 query 重注入
  生效）。验证：切换→新建→发消息→会话归属正确。
- **AC-05** VM "打开文件夹"（PickFolder）切换与列表点选同等满足 AC-02/03/04。
- **AC-06** web 轨零回归：reload 主链保留，`auto build` 全 pipeline 绿、
  vitest 绿、浏览器实测切换正常。
- **AC-07**（条件，T-01 命中才生效）VM 弹层列表可用：显示工作区名、可
  搜索过滤、可点选切换。验证：实机操作 + 截图。

## 8. 执行步骤

1. **T-01 实机基线与加载链定案**（bounded investigation，决策产物写回本
   计划 §4/§10）：
   - (a) VM 实机开弹层：列表是否可用（数组债影响面定案 → AC-07 去留）；
   - (b) 计划列表在 VM 的加载触发点（子件 Init 派发与否 → PlansStore 接线
     层级）；
   - (c) 切换后 chats/plans/files/wiki/whitelist 各视图实际不刷新面摸底。
2. **T-02 显示名修正**（§5 显示名；约 workspace_selector.at 单文件）。
3. **T-03 刷新链落地**（§5 刷新链；forge_store.at + workspace_selector.at +
   workspace_helpers.at（+app.at 视 T-01））。
4. **T-04（条件）VM 弹层列表修复**：仅当 T-01 命中数组债。修因在 musk 侧
   （绕行 Http.get/解包姿势）直接修；在上游则开 `.wt/musk-081/auto-lang`
   （分支 auto-musk-dev），集成验证过即按 AGENTS.md 尽快合回 auto-lang
   主分支并清理。
5. **T-05 双轨验证收口**：§6 全量跑 + 证据落 reports + SD-01/02 入册 +
   §9 复审记录。
6. **收尾**：`bash D:/autostack/wt-guard.sh D:/autostack/.wt/musk-081/auto-musk`
   → 与 PLAN-080 协调（080 先 rebase main + ff-only 合回 + 清理，081 随后
   `git rebase main` → wt-guard → main 快进合回 → 删 worktree/分支）。
   hash 证据绑定旧 commit 时按 AGENTS.md 用 `git range-diff` 记录映射。

worktree 建法（步骤 0）：

```bash
cd D:/autostack/.wt && mkdir musk-081
git -C D:/autostack/auto-musk worktree add -b plan-081-dev \
  D:/autostack/.wt/musk-081/auto-musk plan-080-dev
```

## 9. 复审记录

- `stage: new | plan_id: PLAN-081 | plan_revision: 1 | outcome: pass |
  根因双定罪（显示=pathOrName 路径优先；刷新=VM reload no-op + 默认 query
  滞留 + SetWorkspace 无刷新接线）| next: work（T-01 实机基线先行）`
  ——2026-09-21 起草，两需求静态证据链齐备，实施风险集中在 T-01 未定案
  的 VM 加载链细节（已设条件任务兜底）。

## 10. 待澄清事项

- 无阻塞项。执行期待定案点均已内建为 T-01 的调查产出：
  - VM 弹层列表可用性（数组债）→ AC-07 条件生效。
  - PlansStore 在 VM 的加载触发点 → §5 刷新链接线层级二选一。
  - Wiki/Whitelist 刷新面 → 登记或（如一行可及）顺手接，超出则转下轮。
