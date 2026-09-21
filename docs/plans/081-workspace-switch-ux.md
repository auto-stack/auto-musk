---
plan_id: PLAN-081
status: executing
feature_name: workspace 选择器显示名修正 + VM 轨切换后列表刷新链
author: [agent]
created_at: 2026-09-21T00:00:00Z
updated_at: 2026-09-21T18:30:00Z
plan_revision: 6
current_step: 11
total_steps: 11
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
- **需求③（r3 并入，2026-09-21 用户截图报）**：收缩态 rail 三修——
  (a) 顶部 logo 只显示一只鹿；(b) 选中导航项背景改为比底色稍亮（bg-accent
  随 accent 预设过亮）；(c) 底部 workspace/设置图标对齐（实测：文件夹居中、
  设置齿轮左偏 8px——用户感知"文件夹歪右"实为齿轮偏左的对比）。
- **需求④（r5 并入，2026-09-21 用户截图报）**：工具卡标题栏恢复单行且
  内容正确——现显示 name="0"、参数段 "0"/:0:0/"0" 七连垃圾且把 header
  顶高；应为 [tool 名 + 主要参数（如 read_file 的路径）+ 右侧状态]。
- **需求⑤（r5 并入，同报）**：思考卡标题栏右侧补上下箭头 icon
  （展开/收起指示，现完全缺失）。
- **需求⑥（r6 并入，2026-09-21 用户截图报）**：三种 block（思考/工具/
  文本）之间无间距贴在一起——设置正常 gap（用户问：Vue 版有而 VM 无
  效？答：同一份 Auto 类串 gap-3，web 由真 CSS 生效，VM 渲染器把 for
  块包一层列、gap 只看直接子级故恒不生效）。
- **需求⑦（r6 并入，同报）**：工具卡标题栏参数段（a) 左对齐、紧跟
  tool 名留 gap（VM 弹性盒把它撑到行中）；(b) 颜色调暗（黑→灰、白→
  浅灰）。

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

### 刷新链（需求②）——实施形（r2 修订）

1. `src/front/forge_store.at` `.SetWorkspace(meta)`：✅ 已实施
   - `meta == None`：维持现状 no-op。
   - id 变化：回填 + 会话域清场（session_id/messages/active_leaf/
     session_status/session_phase/streaming/current_draft/thinking/
     think_open/tool_open/current_gate/report_data/errands/relays/
     task_plans/pending_msgs + `poll_window=[]` + `stream_es=Sse.close`)
     + `.LoadSessionList()`（空 session_id 自动选首会话）。
   - id 未变：仅回填 name/path 显示字段（幂等，防 boot 双拉）。
2. ~~`ws_after_switch()` helper~~ **r2 改为直接接线**（fn 内调 store 语义
   未证实，避免不确定面；仅两处调用点，重复可接受）：
   `src/front/workspace_selector.at` `.Choose`/`.PickFolder` 顺序敏感链：
   `platformRefreshAuth()`（**先于** SetWorkspace——chats_list_sessions
   无参全靠默认 query，后刷则内部重拉打旧 workspace）→
   `ForgeStore.SetWorkspace(meta)` → `PlansStore.Reload()`（新 msg：
   重置 current 防旧 workspace 详情串台 + LoadPlans）→
   `location.reload()`（web 主链，VM no-op 无害）。
3. `src/front/plans_store.at`：增 `Reload` msg + `plans_loaded` 字段；
   `plans_view.at` `.Init` 加 `plans_loaded` 防双拉守卫（FilesView/
   file_loaded 先例）；`app.at` `.ShowPlans` 增 `PlansStore.Init()`
   代派（进视图 freshness；web 轨由视图守卫防双拉）。
4. `login.at`/`app.at` boot 链：`platformRefreshAuth()` 提到
   `SetWorkspace` 之前（ws_resolve_current 内部已回写 localStorage，
   先注入默认 query 再回填，SetWorkspace 内部重拉才带对 workspace）。

### r5：工具卡 "0" 中毒与单行 header（需求④）+ 思考卡 chevron（需求⑤）

**根因（实机定罪，含一次反转）**：重载消息（API blocks 路径）的工具卡
name="0"、参数段 "0"/:0:0/"0" 七连（getToolSummary 七分支逐一命中），
status 幸存（本地合成默认值）。[BLKDBG] 临时打印证明：API 原生值层数据
完好、handler 上下文字段读全程真值；而**渲染/计算上下文对"存储可达嵌套
对象"（b.tool 及其字段）的读产出 "0"，字符串读全上下文可靠**——同代码
同会话两实例对照（实例 1 vtree 真值 vs 实例 2 boot 18s 像素全 "0"）定罪。
r4 现场验证幸存是因当时可见卡片走 legacy tool_calls 路径；T-03 刷新链
使 blocks 路径成为 VM 常态后被命中。

**修法（musk 侧两层拍平）**：
1. `normalizeToolBlocks`（forge_store.at）：入 store 前整树 JSON 往返
   （异常回退原值）；tool 块重建为**纯字符串字段**（tool_name/tool_id/
   tool_status/tool_result/tool_gate_id/tool_pending_cmd/
   tool_escape_paths_text/tool_args_json=stringify(arguments) +
   summary=handler 上下文现算成串）。store 文件不消费跨文件 fn 导入
   （nowSec 先例），摘要内联为 `summaryTextOf`/`segJoin`（改名防 VM
   扁平命名空间与 forge_helpers 版撞名）。
2. `messageBlocks`（forge_helpers.at）：拍平块只读字符串字段，
   arguments 由 args_json 本地 JSON.parse 重建；live(SSE) 本地字面量块
   原样透传；legacy 路径不动。模板 `text .block.summary` 单节点。
3. 布局：header 弃 `for seg` 循环——**VM row 内 for 子树被包装为列**
   （七段竖排顶高的布局根因）；单串双轨一行，逐段条件 class 链
   （auto-lang 债①同族）退役，段级配色双轨退役（用户已确认单行诉求）。
4. chevron（需求⑤）：span 内内联 if/else 被 VM 丢弃（2026-09-03 定案）
   ——改工具卡已证形态（直挂 if/else text 节点，ml-auto 靠右）；
   顺删从未接线的 `chev` computed。

### r6：块间 gap（需求⑥）+ 参数段左对齐/调暗（需求⑦）

1. **块间距**：msg-bubble-ai 撤 `gap-3`（VM 的 for 包列使 gap 恒不生效；
   撤除防 web 与 margin 双倍），text/think/tool-block-slot 三个块 wrapper
   加 `mb-[12px]`（margin 在 VM 已证生效）——双轨等价 12px；末块对
   toolbar 的 trailing 12px 两轨一致，可接受。
2. **参数段**：name/summary 弃 `min-w-0 truncate`（VM 弹性盒撑宽+盒内
   居中=参数漂到行中的根因）改 `shrink-0` 紧跟（flex gap-2≈8px）；
   summary 撤 `font-mono`（VM 类串解析阻断嫌疑、VM 默认字体已等宽，
   web mono 由 scoped .tool-seg 提供）——撤后 `text-muted-foreground`
   恢复解析（像素实证 #94a3b7，暗于 name 的 foreground 白）。
   web 侧长参数截断由 scoped .tool-name/.tool-seg 兜底（工具名恒短）。

### 规范增量（r6 追加）

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/modules/ui-compositions.md | WorkspaceSelector 浮层契约无触发器显示规则与切换完成语义 → 增：触发器显示最终目录名（name 优先/path 兜底/占位殿后），完整路径在 title 悬浮；切换完成语义 web=页面 reload，VM=store 刷新链（SetWorkspace 幂等回填+会话域清场+LoadSessionList+PlansStore 重拉+默认 query 重注入） | 用户裁定修订（本计划 §4）；双轨行为契约化防再漂移 | AC-01/02/03 |
| SD-02 | modify | docs/specs/modules/vm-data-semantics.md | 未登记 location.reload VM 语义 → 增：VM 轨 `location.reload()` = no-op（auto.dom.reload shim 事实源），需"整页刷新"语义的功能必须显式走 store 级刷新链，禁止依赖 reload | 本次根因；防后续功能重蹈 | AC-02/03 |
| SD-03 | modify | docs/specs/modules/vm-data-semantics.md | 增：VM 渲染/计算上下文对"存储可达嵌套对象"的字段读不可靠（产出 "0"），字符串读可靠——跨 store 边界的后端 JSON 载荷必须在 ingest（handler 上下文）拍平为字符串字段/现算串，禁止模板与 computed 直读嵌套对象；另增 VM row 内 for 子树包装为列，行内多段必须预拼接为单串 | r5 工具卡 "0" 定罪事实源；防后续功能重蹈 | AC-11/12 |
| SD-04 | modify | docs/specs/modules/ui-compositions.md | 增：消息块间距契约=块 wrapper `mb-[12px]`（不依赖容器 gap——VM 的 for 包列使 gap 不生效）；工具卡 header 排布契约=name/summary `shrink-0` 紧跟（禁 min-w-0/truncate 弹性盒——VM 撑宽居中），参数色 text-muted-foreground（VM 类串对 font-mono 等存在解析阻断面，颜色类与阻断类不同串） | r6 定罪事实源 | AC-13/14 |

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
- **AC-08**（r3）收缩态顶部只显示一只鹿 logo（dark 形态）；web 亮色主题
  仍自动切换为 light 鹿。验证：VM 截图像素扫描（图标簇唯一）+ web CSS
  机制保留（`!important` 翻回）。
- **AC-09**（r3）收缩态选中导航项背景=底色 +foreground 10%（暗轨约
  +25 亮度），不再随 accent 预设爆亮。验证：VM 截图像素采样对比底色。
- **AC-10**（r3）收缩态底部 workspace 文件夹与设置齿轮两图标水平中心
  对齐导航列中心（±2px）。验证：VM 截图像素中心测量。
- **AC-11**（r5）VM 重载消息的工具卡标题栏单行且内容真实：tool 名 +
  主要参数（如 `run_command ls -la` / `read_file README.md`）+ 右侧
  状态 + chevron；全树无 `content: "0"` 垃圾节点。验证：vtree 文本 +
  实机截图。
- **AC-12**（r5）思考卡标题栏右侧有上下箭头（▼/▲ 随展开态），与工具卡
  同款已证形态。验证：实机截图 + vtree。
- **AC-13**（r6）三种 block（文本/思考/工具）之间有 12px 间距，双轨一致。
  验证：实机截图 + vtree margin 节点。
- **AC-14**（r6）工具卡参数段紧跟 tool 名（gap≈8-10px）左对齐，颜色
  暗于 tool 名（muted-foreground #94a3b7 vs foreground 白）。验证：实机
  截图像素取样。

## 8. 执行步骤

1. [x] **T-01 实机基线与加载链定案** [✅ 已完成（2026-09-21，MCP 9251 臂，
   7 实例轮换）：
   - (a) 弹层列表**可用**——items+Choose 派发全通（数组债未命中当前
     080-tip 二进制）→ T-04 取消、AC-07 不适用；
   - (b) **修正定案**：VM 子视图挂载 Init **会派发**（boot 进 plans 视图
     plans_loaded=true 实证；旧基线"plans 恒空"实为 backend 真值 0 篇的
     不可区分歧义）。用户所报"计划列表不跟切"根因=切换无重拉 + 默认
     query 滞留（重进视图重拉也打旧 query）双因，均入 T-03 修复面；
   - (c) 切换后 session_id/session_list 滞留旧 workspace 实锤（基线：
     backend→auto-edit 后 session_list 仍 22 条=backend 真值，API 对账
     backend=22/auto-edit=15）；files 由 .ShowFiles 进视图重拉既有；
     wiki/whitelist 与 plans 同族（重进视图重拉+query 滞留），本轮仅
     修 plans（点名项），wiki/whitelist 登记 §10。
   - 附带新数据点（§10-9）：**ws 弹层 Choose 按压与冻结/死亡家族强
     相关**（新码 3/4 实例按压后 MCP 失联、旧码同位 1 例复现；080 的
     40+ 导航按压零复现口径之外的更窄触发面）。]
2. [x] **T-02 显示名修正** [✅ 已完成（nameOrPath name 优先/path 兜底/
   占位殿后；currentTitle=完整路径。验证：auto build 绿 + VM vtree 触发
   钮可见文本="backend"目录名、label 组合内 path 仅存 tooltip 位）。]
3. [x] **T-03 刷新链落地** [✅ 已完成（实施形见 §5 r2——SetWorkspace
   幂等清场+重拉、selector 顺序敏感链 refreshAuth→SetWorkspace→
   PlansStore.Reload、plans_store Reload/plans_loaded、plans_view 守卫、
   app.at .ShowPlans 代派+boot refreshAuth 前置、login.at 同序。验证：
   auto build 全 pipeline 绿（vue-tsc+vite 11.05s）、vitest 23+1skip、
   生成 Vue 产物逐点核对（useForgeStore/usePlansStore/App.vue/
   WorkspaceSelector.vue）。VM 实机：AC-02 全证（切换后 session_list
   22→15=auto-edit 真值、session_id=9063dfd 即 auto-edit 首会话
   "@plan/001 实施这个计划"、messages 4 条挂载）；AC-03 store 级证
   （plans_loaded=true、plans=[1]=PLAN-005、current 自动选中）；AC-04
   机制证（无参列表 15 条获取本身=默认 query 已重注入实证；NewSession
   显式传 .workspace）。视觉级（截图）因 §10-9 冻结未采全，转 §10。）]
4. [x] **T-04（条件）VM 弹层列表修复** [✅ 已取消——T-01(a) 定案列表可用，
   数组债未命中，无需修复。]
5. [x] **T-05 双轨验证收口** [✅ 静态门绿+VM 状态级实证（见 §9 首条）；
   视觉级余项（AC-01/03 截图补采、AC-05 PickFolder、AC-06 浏览器 E2E）
   受 §10-9 冻结家族制约转 §10；证据收据
   docs/reports/ui-parity/081-workspace-switch-ux.md。]
6. [x] **T-06 收缩态 rail 三修（r3）** [✅ 已完成（musk 0f28c31）：
   - ③a 双鹿=light 鹿 img 加 `hidden` 类（VM 不渲染；web 亮色由
     `display:block !important` 翻回）——像素证 [24-103]→[44-83] 单鹿；
   - ③c 齿轮左偏 8px 双根因（用户感知"文件夹歪右"实为齿轮偏左对比）：
     wrapper `border-t` 边框容器几何左移（A/B 像素证 56→64）+ title 经
     EE03 进 label 参与布局（复合因素）——分隔线改独立 bg-border h-px
     元素（border 式分隔线同样触发位移）、收缩态不挂 title；文件夹/
     齿轮/导航图标中心全 64=rail 中心；
   - ③b 选中背景改 wrapper col+bg-accent（web 两主题正常）；**VM 侧
     当前 master 二进制 rail 子树 bg 全不上色**（按钮/容器×语义/alpha/
     hex 矩阵排除+生成产物核对；080 二进制用户实机曾画 teal=auto-lang
     回归面）——AC-09 VM 视觉暂缺，转 §10 上游项。
   - 附带 auto-lang 债三件（§10）：icon 节点条件 class 毒化全局解析
     （鹿 hidden 失效复现/回退即愈）；if 分支子节点 style 疑构建期
     丢弃；rail 子树 bg 不渲染。多轮 A/B 期间一轮构建静默失败（warning
     无 success 行）→ 截图对旧码，教训=每轮 build 必 grep success 行。]
7. [x] **T-07 工具卡/思考卡不渲染修复（r4，用户报"工具调用组件又坏了"）**
   [✅ 已完成（musk 344dcfb）：根因=if 分支内 col 挂 `class:` 属性时整棵
   子树不进渲染（快照/vtree 节点齐全而像素空白——auto-lang master 渲染
   面，与 rail bg 不上色同族）。chat_message.at 三处卡片容器（think-block
   col、通用 tool-card 三态 col、spawn_relay 兜底 col）class→style。
   验证：实机像素 completed 绿字 rgb(34,197,94)+裁图视觉确认 💭 chip/
   双 🔧 卡（run_command ls -la、read_file README.md）/边框圆角全渲染；
   web 轨 class/style 等价。]
8. [x] **T-08 工具卡 "0" 中毒根修 + header 单行化（r5 需求④）**
   [✅ 已完成（musk 0b70043）：定罪与实施形见 §5 r5——normalizeToolBlocks
   JSON 往返 + tool 块拍平纯字符串字段（summary 于 handler 上下文现算，
   内联 summaryTextOf/segJoin）；messageBlocks 改字符串消费 + args_json
   本地 parse；live 块透传；legacy 不动；header 弃 for seg 循环改单串
   （for 子树在 VM row 内被包装为列=顶高布局根因）。验证：auto build 绿
   + vitest 23+1skip + 生成产物三处核对；VM 实机 vtree 全树零
   content:"0"、按钮 row 五兄弟 [🔧 run_command ls -la completed ▼]/
   [🔧 read_file README.md completed ▼] 单行、像素截图
   tmp/p081-r5-chats-fixed.png；ToolToggleKey 展开链通（tkey #tool:tc-1
   稳定）ARGUMENTS/RESULT 正文渲染；[BLKDBG] 取证打印已撤，净码复验臂
   同口径全绿。]
9. [x] **T-09 思考卡 chevron 补齐（r5 需求⑤）**
   [✅ 已完成（同 0b70043）：span 内内联 if/else（VM 丢弃，2026-09-03
   定案）改工具卡已证形态直挂 if/else text 节点 ml-auto；顺删未接线
   chev computed。验证：实机截图 💭 已思考 · 301 tokens ▼ 箭头在位；
   vtree label 含 ▼ 子节点。]
10. [x] **T-10 块间 gap（r6 需求⑥）**
    [✅ 已完成（musk 4945f61）：msg-bubble-ai 撤 gap-3 + 三块 wrapper
    mb-[12px]（双轨等价）。验证：vtree margin b:12 节点 8 处；实机截图
    块间分隔可见。]
11. [x] **T-11 参数段左对齐+调暗（r6 需求⑦）**
    [✅ 已完成（同 4945f61）：name/summary 弃 min-w-0 truncate 改
    shrink-0 紧跟；summary 撤 font-mono 后 text-muted-foreground 恢复
    解析。验证：像素取样 name=(247,249,251)/param=(148,163,183)=#94a3b7、
    参数紧跟 name（gap≈10px 逻辑）；spawn_relay 兜底卡 name 同步处理。]
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

- `stage: work | plan_id: PLAN-081 | plan_revision: 4 | outcome: pass（代码
  完成，T-01..T-04 闭环；T-05 静态门绿+VM 状态级实证，视觉级余项因外部
  制约转 §10）| code_commit: musk plan-081-dev @ 9565b48（clean，wt-guard
  clean）| task_ids: T-01 定案（弹层可用/子件 Init 实派发/切换滞留实锤/
  §10-9 Choose 相关新数据点）、T-02 显示名、T-03 刷新链（实施形 r2：
  Reload msg+顺序敏感链+boot 前置）、T-04 取消 | evidence: auto build 全
  pipeline 绿（080 worktree auto.exe）；vitest 23+1skip；生成 Vue 产物
  逐点核对；VM 实机（MCP 9251，后端 17201 复用）AC-01 vtree 可见文本=
  目录名、AC-02 session_list 22→15+session_id 9063dfd+messages 挂载、
  AC-03 store 级 plans=[1]+current 选中、AC-04 机制级（无参列表重注入
  实证+显式参数）；基线对账 API：backend 22 会话/0 计划、auto-edit 15
  会话/1 计划（PLAN-005）| blockers: 无代码阻塞；视觉级验证（AC-01/03
  截图、AC-05 PickFolder、AC-06 浏览器 E2E）受 §10-9 冻结家族与原生
  对话框制约 | next: review（SD-01/02 于 merge 阶段入册 docs/specs）`
  ——2026-09-21 work 收口。修订 r2：§5 刷新链改直接接线（fn 调 store
  语义未证实）；PlansStore.Init→Reload（current 串台防护）；boot 链
  refreshAuth 前置（app.at/login.at）；T-04 取消（数组债未命中）。

- `stage: work | plan_id: PLAN-081 | plan_revision: 4 | outcome: pass（需求③
  三修完成：③a/③c 实机像素双绿、③b web 绿+VM 受上游 bg 渲染回归所限）
  | code_commit: musk plan-081-dev @ 0f28c31（clean，wt-guard clean）|
  task_ids: T-06 三修+收尾 | evidence: auto build 全 pipeline 绿；VM one-shot
  臂（MCP 9251）像素测量——鹿 [44-83] 单只/齿轮文件夹导航中心全 64；A/B
  排除矩阵（placement/class-vs-style/title 单因/alpha/hex/col-vs-div/
  if 分支）在案 | blockers: ③b VM 视觉=auto-lang master rail 子树 bg
  渲染回归（§10 转上游）；§10-9 冻结家族继续干扰（每实例 1-2 分钟内
  死亡高频）| next: review（或用户实机目验 ③a/③c）`
——2026-09-21 r3（需求③收缩态三修）。附带 auto-lang 债三件+一轮静默
构建失败教训入 §10。

- `stage: work | plan_id: PLAN-081 | plan_revision: 5 | outcome: pass
  （需求④⑤完成：工具卡数据中毒根修+header 单行化、思考卡 chevron 补齐，
  实机像素/vtree 双证）| code_commit: musk plan-081-dev @ 4595fa2
  （0b70043 代码 + 收据 docs；clean）| task_ids: T-08/T-09 | evidence:
  auto build 全 pipeline 绿；vitest 23+1skip；VM 实机（MCP 9251，后端
  17201 复用）AC-11/12 全证——vtree 全树零 content:"0"、按钮 row 五兄弟
  单行、像素截图 tmp/p081-r5-chats-fixed.png、展开链通；定罪反转一次：
  初判 API 边界毁数（round-trip 修），对照臂证明 handler 读真值而渲染
  像素 "0" → 二次定罪"渲染/计算上下文嵌套对象读不可靠"→ 拍平字符串
  字段根修 | blockers: 无 | next: review（或用户实机目验；SD-03 于
  merge 阶段入册）`
  ——2026-09-21 r5（需求④⑤）。新债登记：VM 渲染上下文嵌套读 "0"（债⑤
  +拍平处方）、row 内 for 子树包装为列、`.messages=` 全量重绑画面滞留
  嫌疑（PLAN-536 族）、VM 文本 flex 计量差异，均入 §10。

- `stage: work | plan_id: PLAN-081 | plan_revision: 6 | outcome: pass
  （需求⑥⑦完成：块间 12px gap 双轨等价、参数段左对齐+调暗，像素实证）
  | code_commit: musk plan-081-dev @ 4945f61（clean）| task_ids: T-10/
  T-11 | evidence: auto build 绿（gen/front/vue node_modules d3 错配
  复发——junction 清理后下次构建必坏，删目录重装即愈，入册）；vitest
  23+1skip；VM 实机 vtree margin b:12×8 节点；像素取样 name=(247,249,
  251)/param=(148,163,183)=#94a3b7、参数紧跟 name；**新定罪：summary
  串内 font-mono 阻断同串颜色类解析（撤除即愈，r5 参数呈白即此）** |
  blockers: 无 | next: review（或用户实机目验）`
  ——2026-09-21 r6（需求⑥⑦）。类串部分失效面（font-mono 阻断色类）与
  弹性盒居中面入 §10。

## 10. 待澄清事项

- 无阻塞项。T-01 定案后余留（非本轮范围，按需立后续）：
  - **§10-9 新样本（r6 后，用户目验窗被"自动关闭"）**：18:24:50 启动
    （agent Bash 血缘：Bash→cmd→start cmd→auto.exe），18:25:53 末次
    X9-ALIVE 心跳、18:26:16 日志终止——**寿命 ~90s**；无 panic/无
    returned、WER（Application 1000/1001/1002）无 auto.exe 崩溃记录=
    外部终止再实证。**新对照实验已布置**：`launch-vm-081.cmd`（主检出
    根，双击启动，指向 081 worktree r6 + 日志 tmp/p081-vm-user.log）
    ——用户双击启动若存活 >3min 而 agent 血缘实例恒 ~90s 亡，则杀手=
    agent 会话进程清理（job 对象连带）；若同样亡，§10-9 与启动方式
    无关的口径维持。**18:37 追记：同血缘实例 5（PID 21816，18:34:50 起）
    存活 3min+ 仍在跑（心跳正常）——杀手非固定 90s 计时器，呈间歇/
    事件触发；双击对照实验仍开放。**
  - **080 移交债②实锤复现（r7 观察，用户报，081 不承接）**：助手消息
    正文在 VM autodown 渲染丢正文——「目录内容」markdown 表格只画边框/
    分隔线、单元格文本全空；「README 一句话总结」带粗体+行内代码的 CJK
    段落正文整段不进逻辑树（r5 vtree 同证）。定界：musk 侧无罪——content
    字符串经拍平链完整（后端 API 对账原文 516 字节含表格+段落），Markdown
    端口直通原生 autodown（renderer.vm.at），标题/hr 均渲染=引擎收到全文。
    = KNOWN-DEBT（Plan 080）②「表格单元格文字空白」的同源实锤 + 新变体
    （粗体 CJK 段落正文缺失，疑 F-UAT-2 段落修复未覆盖此形态）。接收
    计划待建中，修因在 auto-lang autodown 引擎（上游）。
  - **§10-9 新数据点（转 080 交接清单①）**：ws 弹层 Choose 按压与冻结/
    死亡强相关（新码 3/4 实例、旧码 1 例同位复现；进程活/MCP 死或整体
    亡）。080 的"触发条件不在导航切换层"口径需补此窄触发面。r3 轮更甚：
    每实例 1-2 分钟内死亡高频化（同会话多实例）。
  - **视觉级验证余项（review/用户目验承接）**：AC-01/03 切换域截图、
    AC-05 PickFolder 实机（与 Choose 同链，风险低）、AC-06 web 浏览器 E2E。
  - **VM workspace 选择不跨启动持久**：Choose 写 localStorage（VM 会话
    KV，进程级不落盘）→ 重启回退 registry 默认（backend）；叠加
    ws_resolve_current 列表回退链被数组迭代债打断（080 登记"VM 回填
    链暂缓"）。用户未报，登记观察。
  - **Wiki/Whitelist 同族刷新**：与 plans 同根因（进视图重拉+query
    滞留）；本轮只修点名项，同款一行接线可后续批量收。
  - **auto-lang 债四件（r3/r4 实测，转上游登记）**：① `icon` 节点挂条件
    class（`class: if..else..`）毒化全局解析——他处 img 的 hidden 失效
    （双鹿回归，回退即愈，像素复验）；② if 分支子节点 style 疑 VM 构建
    期丢弃（快照打印原始串≠消费面，多组"在树里但不生效"观测）；③
    master 二进制 rail 子树 bg 渲染回归（按钮/容器×语义/alpha/hex 全
    不画；080 二进制实机可画 teal 高亮）——PLAN-081 ③b VM 视觉受此
    所限，web 侧 bg-accent 正常；④ **if 分支内 col 挂 class: 属性整棵
    子树不渲染**（工具卡/思考卡全空白，class→style 即愈=r4 修复）——
    与 ②③ 同族"树在漆不出"渲染面回归。
  - **auto-lang 债⑤（r5 实测定罪，渲染/求值面）**：**渲染/计算上下文对
    "存储可达嵌套对象"（API 值树经 store 根态中转的 obj 字段链）的字段
    读产出 "0"**，字符串读全上下文可靠、native stringify/parse 任意上
    下文可靠、handler 上下文读可靠（BLKDBG pre/post 实证）。处方：ingest
    （handler 上下文）把后端 JSON 载荷拍平为纯字符串字段 + 现算串；他店
    （errands/relays/task_plans/specs 等）同症按同方处理。附带布局债：
    **row 内 for 子树被包装为列**（多段竖排；亦使容器 gap 对块恒失效，
    r6 改 wrapper margin），行内多段必须预拼接单串；文本节点 min-w-0 在
    VM flex 内膨胀（参数段居中分布 vs web 紧跟），r6 以 shrink-0+hug
    收口。
  - **VM 类串解析部分失效面（r6 新数据点）**：文本类串内含 `font-mono`
    时，同串的颜色类（text-muted-foreground）不生效（撤除即愈——r5
    参数段呈白色即此）；`min-w-0 truncate` 同串时疑似同族。与债①②④
    （类串解析/丢弃面）同族，具体阻断边界待上游定界。**行内紧凑排布
    处方：shrink-0+hug，禁弹性盒；颜色类串避免混入 font-family 类。**
  - **`.messages = <全量重绑>` 画面滞留嫌疑（r5，未定罪）**：实例 2
    boot 18s 时轮询已多次换入真值而像素仍显示旧 "0" 态——与 PLAN-536
    "跨帧 SET_FIELD 重绑定不可见"族同疑（push 可见、赋值可疑）。r5 拍平
    后 boot 首绘即真值，此嫌疑对现网无感；若后续再现"换数据不换画面"
    按 536 族并案。
  - **§10-9 首份带捕获证据的死亡样本（r4，p081-vm-live.log）**：X9 心跳
    每 30s 至 1789976704 止、无 X9-PANIC/无 returned/无 main_err → 四分
    法=外部终止类**首次有日志实证**；`[POLL] streaming=-2147483647` 垃圾
    读数（PLAN-536 根态重绑不可见债的实锤表现）；死点=工具轮对话后
    ~120s（与上一样本"切换后精确 120.03s"同拍=deadman 过期节拍）——
    触发面新增"带工具调用的对话轮"；强相关但未定因果，X9 桩续采。
  - **工程教训**：auto build 偶发静默失败（只打 warning 无 success 行）
    ——每轮构建必须 grep "successfully" 再起实例，否则截图对旧码误导
    定罪（r3 轮实际发生一次）。r5 补充：wt-guard 清 junction 后
    `gen/front/vue/node_modules` 可残留坏 pnpm 状态（d3-contour@4 配
    d3-array@2 的 "blur2 is not exported"）——删除该 node_modules 重装
    即愈，非源码问题。**r6 确认其复现规律：每次 junction 清理后，下一轮
    build 必先删 gen/front/vue/node_modules 再 build，否则 d3 错配。**

