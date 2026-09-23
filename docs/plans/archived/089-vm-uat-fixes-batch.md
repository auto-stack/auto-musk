---
plan_id: PLAN-089
status: archived
feature_name: VM 实机 UAT 缺陷批次修复（rail 高亮/归档 icon/分组时间轴/计划页三修）+ 文件栏 phase
author: [agent]
created_at: 2026-09-23T11:11:00+08:00
updated_at: 2026-09-23T20:05:00+08:00
plan_revision: 3
current_step: 10
total_steps: 10
supersedes_spec_components: []
new_spec_components: [docs/specs/modules/files-browser.md]
touched_goals: []
---

# PLAN-089 — VM 实机 UAT 缺陷批次（2026-09-23 四轮）补录：需求整理 + 交付回执 + 移交清单

## 0. 变更摘要

2026-09-23 用户实机 UAT 连报四轮缺陷/修改需求（会话列表归档 icon、分组时间轴、rail 高亮
与 workspace 图标、计划页三修）。本计划为**补录**：工作已当场交付并合入 main，本文档按
「需求 → 根因 → 修法 → 提交 → 验证」逐条整理，并把本轮新定罪的 VM 坑与移交债集中入册。
全部改动零后端契约变更（后端仅重建带既有代码）、零 spec 语义变更（`new_spec_components: []`）。

落地锚点：**main `88235de`**（vm-ui-polish：会话轨三修）+ **main `2d2a7fe`**（plans-ui：计划页三修）
+ 后端 release 重建（0923，含 084 T-03/T-05 既有代码）。工作流：两个任务型 worktree
（`auto-musk-dev-1/2`）各自 wt-guard → rebase → ff-only 合回，worktree/分支/组目录全清。

## 1. 背景与口径

- 验收面 = VM 桌面版（`launch-vm.cmd` 链：musk.exe serve @17201 + auto.exe run --render vm）。
- 当日运行环境：auto-lang master `b4960eb8f`（release auto.exe 09-23 01:45 构建）、
  auto-ai master 含 PLAN-034、musk main 含 084/085/086/087 全部交付。
- 需求均为用户实机截图报告；根因定位以「源码链路 + MCP 快照/截图 + 像素取证」三重证据
  定谳，不做无证据修复。

## 2. 交付清单（T-01..T-06，全部完成）

### T-01 会话行 hover「归档」icon 不渲染（commit `88235de` ③）

- **需求**：会话列表 hover 时归档 icon 不显示（✎/× 正常，中间空位）。
- **根因**（VM 坑 #24，三重实证）：`aura_view_builder.rs` `convert_text_element_tracked_ctx`
  把带 onclick 的 span 升格为 `View::Button{ label: 拍平文本, content: None }`——icon/Image
  子节点既不进 label 也不进 content 槽，整个静默丢弃。✎/× 是 text 子节点故幸存；UAT 轮
  emoji（文本）能显；rail/头部显式 button 包 icon 正常。
- **修法**：`chats_view.at` 归档两臂 span → 显式 button（icon-only button 渲染臂
  rail/头部已实证；`onclick.stop` 有 report_card 先例）。
- **验证**：源码链路+像素取证（间隙 1632 像素纯背景色=非颜色问题）+同字形头部按钮正常。
  hover 态合成鼠标不可触发（物理鼠标抢占），最终目验移交用户（重启后悬停即见）。

### T-02 新建会话落「更早」分组——分组时间轴整体失效（后端重建）

- **需求**：新建对话 "New chat" 显示在「更早」一栏，应在「今天」。
- **根因**：分组字段 `day_group`（0=今天/1=昨天/2=更早）是 084 T-03 的**后端现算元数据**
  （api.at「后端本地时区现算」——VM Date.now() 负垃圾不可用作时钟）。当时后端 exe 是
  09-22 13:04 旧构建（早于 084 后端提交），响应无 `day_group`，前端 `rebuildSessions`
  `?? 2` 兜底把全部会话塞进「更早」。**不是 VM 时间戳坏了**。
- **修法**：重建后端 release。期间排掉两层阻断：①「wgpu-hal 27.0.4 编译失败」未再复现
  （前挂账的已知阻断随 auto-lang master 前进自然消解，首跑失败系暂时态）；②真阻断=
  auto-ai PLAN-034 给 `RoleConfig` 增 `models` 字段，musk 手工维护 glue
  `auto_generated/extern_impl.rs` 两处初始化缺字段（E0063）——补 `models: None` ×2
  （语义=RoleConfig::empty()，行为零变化；**该 2 行未提交**，正式归属 auto-ai 034
  KNOWN-DEBT 已挂的「musk roles 页未适配 models」适配计划）。
- **验证**：默认工作区 今天2/昨天1/更早14 三分组实证；musk-demo 全 25 条落「更早」为
  正确行为（updated_at 均为 09-18/19 旧会话）。附带收益：T-05 归档路由同车上线，
  「归档 toggle 落空」缺口愈合。

### T-03 展开态 rail 选中高亮与二级导航一致（commit `88235de` ①）

- **需求**：一级 Sidebar 选中高亮应与会话列表一致——文字 primary accent 色，背景带
  accent 色调（非纯灰白）。
- **根因**：展开态 rail 用原生 `sidebar_menu_button`，其 active 类
  `bg-sidebar-accent text-sidebar-accent-foreground` 是 shadcn sidebar 中性灰调色板
  （web `--sidebar-accent: 217°/17.5%` 同灰；VM 映射 sidebar-accent→Secondary）——
  两轨皆灰，属「设计如此」，非渲染 bug。
- **修法**：展开态 rail 手搓化（弃 sidebar_menu_button）：外层容器 `bg-primary/10` 承载
  背景（0921 容器 bg 承载裁定）+ 文字/图标 `text-primary`，与收缩态 rail 结构统一、
  与会话列表选中态同款。
- **验证**：MCP 截图像素级实证（indigo 图标+文字+色调背景，与「你好」选中行一致）。

### T-04 收缩态 workspace 图标靠左不居中（commit `88235de` ②）

- **根因**：按钮内层 `row` 挂 `w-full`，撑满 40px 按钮后图标作为行首子元素必然偏左。
- **修法**：`workspace_selector.at` 收缩态 row 改 `items-center justify-center`（无文本
  兄弟，w-full 无意义）；展开态保持 w-full（text flex-1 依赖）。
- **验证**：MCP 截图实证与设置齿轮对齐居中。

### T-05 计划页缺「已归档」栏目（commit `2d2a7fe` ①）

- **需求**：计划列表没有「已归档」栏目。
- **根因**：后端早已支持（`PlansQuery.include_archived` + `PlanFile.archived` 标志 +
  `archived/` 子目录语义），前端从未消费（plans_view 自述 MVP 态）。
- **修法**：`plans_store.at` 拉 `?include_archived=true` 后分流 `plans`/`archived_plans`
  两表 + `archived_count` 标量（计数必须烤 handler 域——computed 内 `.length` 静默
  返空，VM 坑 #26）；`plans_view.at` 增折叠组头（chats「更早」同构：t() 字面量 key +
  `"" + count` R046 安全拼接 + chevron 双单臂）。归档行点击复用 `SelectPlan` →
  `plans_get` 直读 archived/（plans.rs:655 既有语义）。i18n zh/en 增
  `plans.archivedSection`。
- **验证**：一次性测试计划经 API 归档后，「已归档 1」组头→展开→行点击→详情从
  archived/ 正确渲染，全链 MCP 实证。测试数据已删档。

### T-06 计划详情正文不渲染 + 「编辑」按钮退役（commit `2d2a7fe` ②③）

- **需求**：点击计划右侧不显示内容（应显示 markdown，走 autodown 引擎）；「编辑」按钮
  去掉最好。
- **根因**：`detailMeta/detailBody` computed 调 `plans_frontmatter.ts` 的 use.web fn——
  视图坑 #1（computed 内 use.web.fn 静默返空）：`detailBody=""` → Markdown 组件空源、
  meta chips 全隐。Markdown 组件本体无罪（聊天轨同组件 autodown 渲染已实证）。
- **修法**：`plans_store.at` 增 `plans_front_slice`（handler 域 .at 纯函数：frontmatter
  剥离 + step_label=`cur/total`；**`list.join` 在 VM 链接期 Undefined symbol——已入册
  codegen-quirks #26，用手拼循环**），LoadPlan/UpdatePlan 时拆好拍平进
  `current_body`/`current_step_label` 标量；chips 改读 PlanFile 后端权威字段
  （id/status/feature_name）+ 步进标尺；正文 `Markdown(source: .store.current_body)`。
  头部「编辑」按钮移除（StartEdit/编辑态 handler 域保留，未来换入口可复用）。
- **验证**：MCP 实机全链：详情 H1/粗体/表格/列表 autodown 渲染、chips 出齐
  （PLAN-001|drafting|2/5|ui-smoke-临时）、无编辑钮、归档行详情同链路。

## 2b. Phase 2（0923 晚追加，待执行）：「文件」栏点击无反应 + 分型渲染升级 + 归属架构裁定

### 需求（用户口径）

1. **缺陷**：「文件」一栏点击文件打开没反应。
2. **分型渲染需求**：`.md` 与 `.ad` 文件打开用 **autodown-engine** 展示；其他文件用
   **code_editor 组件**打开。
3. **架构问题**：musk 的「计划」「规范」「知识库」本质上属于 `../jade-edit` 的功能域，
   「文件」属于 `../auto-edit` 的功能域——应当**引用它们的组件（或 bp）**来实现，还是
   **直接调用**它们？已知约束：AutoUI 尚无直接「内嵌」另一个 app 的能力。
   → 分析与推荐见 §6（开放裁定，待用户拍板）。

### 勘察（0923 深夜实机定谳）

- **第一断点（用户症状根因）**：`FileTree`（PLAN-614 移植组件）的行在 **VM 轨没有接
  点击事件**——行渲染为裸 `row`（icon+text），MCP press 报
  `No 'press' handler found`，`onselect: .Pick($event)` 形同虚设 → 点击零反应。
  移植债：组件行可点性只在 web 形态生效，VM 形态未接。
- **第二断点（修好第一层后必然撞上）**：文本加载走视图 handler 的
  `loadFilesFileText(path).await`（**use.web fn**）——VM 侧为 no-op 平台桩，`SetContent`
  永不回填 → 面板空白。修法见 T-07（后端通道）。
- **分型面**（`files_file_kind` 正则）：`.md/.markdown` → markdown ✓；**`.ad` 未收录 →
  "other" → 「无法打开」空态（用户需求缺口实锤）**；代码族 → code（现形态=
  `files_code_markdown` 围栏包装走 autodown 高亮，非 code_editor 组件）。
- **code_editor 原生臂**存在于 VM builder（aura_view_builder:2082 `convert_code_editor`）；
  web 轨映射与 readonly 能力待查（T-09 首步）。
- **附带定案**：文件树本身加载无病（backend 活着时树即出，快照可见 Cargo.toml/
  README.md；早前空树=后端被清场后的空响应级联）。

### 任务（T-07..T-10）

- **[x] T-07 点击链路修复** `[✅ 已完成 2026-09-23 Phase2，musk 0173917 + auto-lang 7183ca386]`
  ①FileTree 行接 VM 点击——勘察定谳：VM builder 容器臂（convert_container_tracked_ctx）
  不消费 events，`row (onclick:)` 在 VM 轨静默丢弃（MCP press 报 No 'press' handler
  found 与勘察一致）；web 轨 row+onclick → div @click 本来就好。修法：行改 **mouse-area
  包裹**（双轨同形：web=div@click 不变，VM=原生 on_click 臂——本组件 chevron 同款先例；
  避开 button 的 shadcn 预设对 web 的视觉回归），state/hover 类挂外臂、内 row 承横排。
  ②文本加载改后端通道——`#[api] files_text(path)` 契约（**JSON `{content,error}` 诚实
  契约**：error 成功恒 ""、失败带消息；绑 response.json() 必炸旧约束绕开；web 生成端
  `!ok` throw 进 catch 臂、VM 非 2xx 包 `{error,status}` 兼容同判），后端
  `/api/files/text/{*path}` 通配符路由复用 raw 的 confinement+20MB 上限+lossy 解码；
  视图 Pick handler 直调（chats_view 视图引 back.api 先例 + mention_input 绑定先例），
  SetContent/SetLoadFailed 回填面不变；ports/files.vm.at+files.web.at+files_web.ts
  no-op 桩全家退役。
  验收：**web 轨浏览器全链实证**——点击 smoke-089.ad → active_path 变化 + 右侧出内容
  （files_text 后端加载成功）；后端 curl 三臂（根文件 / %2F 嵌套 / 404）绿；VM 轨树
  渲染/导航实证（截图），**行点击的 MCP press 受仪器面四层失配阻断（§7 仪器债 #1）**，
  真鼠标验收移交用户（与 T-01 hover 移交同协议）。
- **[x] T-08 `.ad` 支持** `[✅ 已完成]`：`files_file_kind` 增 `.ad` → markdown 臂
  （autodown 引擎原生解析 .ad——markdown 超集实测）。验收：web 打开 .ad 报告出真渲染
  （H1/strong/列表/表格 MCP DOM 快照逐项可见）✓。
- **[x] T-09 非 md 文本 → code_editor 组件** `[✅ 已完成]`：勘察结论——web 轨映射
  完备（vue codegen → CodeMirror 壳组件，auto-man 检测 corpus 自动物化壳+全组
  codemirror 依赖）；**readonly 双轨均无**（View::CodeEditor 无字段）→ auto-lang 补
  `readonly` 穿透（同组 worktree `.wt/musk-089/auto-lang` 分支 musk-089-dev，**已折回
  master `7183ca386`**）：View::CodeEditor 字段+builder bool_prop+核心 CodeEditor
  Config 输入门（handle_key 变更键吞掉/ImeCommit 吞掉/菜单 undo/redo/cut/paste 拒面，
  导航/选择/复制保留）+动态臂+native_projector+vue 臂 `:readonly` 发射+auto-man 壳模板
  `EditorView.editable.of(false)`。musk 侧：code 臂改 `code_editor (readonly: true,
  lang: .store.active_lang, key: "files-viewer")`（§5.4 值差分回写切文件），围栏包装
  files_code_markdown 退役。验收：web 打开 pac.at → 行号+AutoLang 三色高亮+
  `contenteditable=false` ✓；.md/.ad 仍走 autodown ✓。回归锁
  `plan089_mouse_area_press_tests`（auto-lang）+readonly core 测试。
- **[x] T-10 架构裁定与记录** `[✅ 已完成 2026-09-23 晚（用户指示 review+merge 直通，
  裁定移交后续）]`：§6 结论**未经正式拍板**——用户审阅 Phase-2 交付后指示直接进入
  review+merge，裁定按「移交后续」处理并记录在案：A 路线三件（FileTree/autodown/
  code_editor readonly）已在本计划内就绪且被消费，B 路线前置（auto-lang「app 内嵌
  能力」立项）**未启动**；后续若走 B，另立计划承接。本任务按「记录+移交」收口，
  不虚构拍板结论。

### 规范增量（Phase 2 复核定稿，merge 落 docs/specs/modules/files-browser.md）

- **modify** `docs/specs/modules/files-browser.md`：
  1. API 契约表增一行：`GET /api/files/text/{*path}?workspace={id}` →
     `{ content: string, error: string }`——文本正文 JSON 通道（生成绑定固定
     `response.json()`，纯文本响应必炸的约束以此绕开；`error` 成功恒 ""、失败带
     消息；非 2xx 时 VM 桥包 `{error,status}` 同形状兼容、web 端 throw）；沿用
     raw 的 confinement/20MB 上限/lossy 解码。理由：VM 轨文件正文此前为 no-op 桩
     （PLAN-089 T-07 断点）；验收 AC-1。
  2. 查看器分型表：markdown 行扩名增 `.ad`（autodown 引擎原生解析的文档格式，
     markdown 超集）；code 行呈现改「**原生 code_editor 组件（只读，`readonly:
     true`）**」（原「围栏包装经 autodown 管线高亮」退役；高亮/行号/只读由组件
     双轨承担，编辑能力归 auto-edit 域）。理由：T-08/T-09；验收 AC-2/AC-3。
  3. 前端接线节补一句：FileTree 行点击面为 mouse-area 双轨同形（VM 容器臂不消费
     row.onclick——行内点击挂点必须 mouse-area/显式 button）；文本正文经
     `#[api] files_text` 后端通道（视图 handler 直调，ports use.web 桩退役）。
     理由：T-07①；验收 AC-1。
  验收 ID 映射：AC-1→1/3、AC-2→2、AC-3→2。

### 验收清单（Phase 2）

- AC-1：点击文本文件右侧出内容（不再无反应）。**✓ web 全链实证；VM 真鼠标移交（§7 债 #1）**
- AC-2：.md/.ad 走 autodown 真渲染（标题/表格/列表可见）。**✓ web DOM 快照逐项实证**
- AC-3：代码族文件走 code_editor 组件形态（高亮），且只读语义明确。**✓ web 行号+高亮+contenteditable=false**
- AC-4：图片/视频/未知二进制三臂不回归。**✓ 臂代码零改动+三 kind 分派实证+raw/mime 单测绿**（PNG 实点因树坐标滚动战放弃，低价值）
- AC-5：§6 裁定回填本计划（含 auto-lang 议题立项与否）。**待用户拍板（T-10）**

## 2c. Phase 2 交付锚点（2026-09-23 晚）

- musk `plan-089-dev` = main 2d2a7fe + `0173917`（文件栏三修+R001 解阻+Phase1 尾巴收编）。
- auto-lang `musk-089-dev` = master ca880b0e0 + rebase + `7183ca386`（readonly 穿透+
  mouse-area press 链仪器面根修+回归锁），**已折回 master（ff-only，wt-guard 双验
  clean）**；worktree `.wt/musk-089/auto-lang` 留位（musk 构建硬依赖，merge 阶段随
  musk 落地后拆除；折回后分支==master 内容同源）。
- 依赖兄弟位四件套（musk-088 布局同款）：auto-musk + auto-lang(musk-089-dev) +
  auto-down(musk-089-dev) + auto-ai(musk-089-dev)——musk worktree 的 cargo 相对路径
  硬依赖组内兄弟存在。
- `auto build` 门禁**全绿**（Vue project built successfully；R001 解阻后）。pnpm
  node_modules 内自动装齐 codemirror 组依赖（auto-man 依赖检测注入）。
- 宿主链陷阱（本批实测，勿再踩）：`auto run --render vm` 的 VM 宿主 =
  **主检出 rust-workspace/auto-lang target\debug\auto.exe**（按项目名键到主检出，不吃
  worktree 的 release auto-rel.exe）；改 auto-lang 后必须 ①折回 master ②在主检出
  `cargo build -p auto` 重链 ③杀旧 auto.exe（文件锁会让重链静默失败——连续两次
  "Finished" 而 exe mtime 不动的根因）④启动命令的 cd 必须显式包进同一命令段（丢 cd
  = 加载主检出旧 .at，快照 ids 不变假象）。

## 7. 仪器债与 master 漂移登记（Phase 2 新增）

1. **MCP press 对 mouse-area 行（VM 轨）四层失配**：VM 桥 api 调用面已修两层
   （extract_action_from_view MouseArea on_click 臂 + press 校验放行）+ styled_vtree
   对齐根修（builder push(0) 压索引 ↔ vnode_converter/snapshot_builder 下探——探针
   `plan089_mouse_area_press_tests` 三断言锁：wrapper=Container 可见/content 入树/
   path 走到 on_click，进程内全绿）。**残余层**：子 widget 语境（App→FilesView→
   FileTree）下 live styled_vtree/computed 面仍把 wrapper 显为 Row+合并事件（inspect
   kind=Row、collect_view_events 空），press 按到错位节点。真机 iced on_click 链路与
   仪器面无关（web 同源视图层点击全通），VM 真鼠标验收不受阻。归属 auto-lang 仪器面，
   另档排查（复合树探针须复刻 App→子 widget 三级语境）。
2. **auto-lang master 既有测试漂移**（基线对照实锤，非本批）：auto-man
   merged_api_client_crud_fallback_for_uncovered_endpoints（CRUD 兜底生成语义漂移）、
   index_css_values_match_p1_baseline（theme token 金样缺 success/warning/info/error）、
   test_shell_pack_lib_freshness、iced desktop_mcp_switcher_thumbs_injected_and_
   requested——均 master 直接复挂失败。
3. **auto-lang master 严格 codegen 校验**（较 09-23 01:45 二进制新增）阻断 plans_view
   （R001 重复 key 已在本计划内解阻；S001 INFO 族——button/span 的 title prop、
   Markdown 的 source prop——未处理，非阻断）。新校验器对存量 musk 视图的全量影响面
   待普查。
4. **flaky 登记**：auto-man plan609_unresolved_dep_import_guard（并行 fs 重试即绿）。

## 8. §6 裁定现状（Phase 2 末）

推荐维持「近期 A、中期 B」不变。Phase 2 的 code_editor readonly 穿透已为 A 路线
（组件下沉）补齐了查看器语义的最后一块双轨能力；FileTree/autodown/code_editor 三件
现成 musk 侧组件即 §6-A 的下沉候选清单。**待用户拍板后驱动 T-10。**

## 3. 提交与落地锚点

| 提交 | 内容 | 验证 |
|:---|:---|:---|
| `88235de` | vm-ui-polish：T-01/T-03/T-04（app.at、workspace_selector.at、chats_view.at） | 截图像素实证 |
| `2d2a7fe` | plans-ui：T-05/T-06（plans_store.at、plans_view.at、i18n zh/en） | MCP 全链截图实证 |
| 后端 release 重建（未产生提交） | T-02（day_group/归档路由上线） | health+三分组实证 |
| `extern_impl.rs` +2 行（**未提交**） | T-02 附属：RoleConfig.models E0063 解阻 | cargo build 绿 |

纪律记录：两个 worktree（`auto-musk-dev-1/2`）均 wt-guard clean → rebase main →
ff-only 合回 → worktree/分支/组目录删除；验证用 auto.exe 走**私有路径拷贝**
（`.wt/<组>/auto-rel.exe`——共享路径 release exe 当日被兄弟 agent 清场反复误杀）；
测试数据（临时 plan/会话）用后即删。

## 4. 遗留与移交（非本批次范围，均已挂账）

| # | 事项 | 归属/来源 |
|:---|:---|:---|
| 1 | span+onclick 升格链丢非文本子节点的 **auto-lang 根修**（content 槽承载子节点） | auto-lang；musk 侧已按 #24 处方规避（88235de） |
| 2 | ws 弹层 Choose 按压冻结/死亡家族（本次 MCP 驱动切换工作区踩中一次） | PLAN-081 §10-9 数据点挂账，auto-lang |
| 3 | 桥大载荷丢弃 P1（9.3-12KB 丢失） | KNOWN-DEBT 084-D1 |
| 4 | `$$..$$` 数学块语法（autodown 引擎扩展） | KNOWN-DEBT 084-D2 |
| 5 | 通用元素浮空 tooltip 运行时接线 | KNOWN-DEBT 084-D3 |
| 6 | Date.now() VM 负垃圾 native 修复（含 083 F-R1 尾项解锁） | PLAN-083 移交，auto-lang |
| 7 | musk roles 页 models 适配（含本批 extern_impl +2 行收编） | auto-ai PLAN-034 KNOWN-DEBT |
| 8 | 合成鼠标对 iced 的 hover/Click 均不可靠（hover 不触发、click 不派发）——UAT 自动化验证面缺口 | 仪器债；验收靠用户真鼠标或 MCP press |
| 9 | VM computed 内 `.length`/use.web.fn 静默返空根因归因 | auto-lang；musk 侧按 #1/#26 处方规避 |
| 10 | `rust-workspace/Cargo.lock`（本地锁，不入库）陈旧偏斜：wgpu-hal 27.0.4 缺 `windows-core 0.58.0` 边 → VM 宿主重建必炸（E0277 `ResourceCategory/Param` 界限不满足）。**已就地修复**（`cargo update -p wgpu-hal` 归一化本地锁，与 backend 锁对齐；无提交面）。初判「并发撞车」系误归因——并发退场后仍稳定复现，锁归一后即愈 | PLAN-087 gpu-allocator 先例同族（本地锁偏斜 vs manifest 漂移） |

## 5. 验证口径与仪器（本批沉淀）

1. **三重定谳法**：源码链路 + MCP 快照/截图 + 像素取证（间隙直方图判「没画 vs 画了
   不可见」）。
2. **MCP 驱动边界**：press/快照/截图可用；合成 SetCursorPos 悬停不触发 mouse-area、
   mouse_event 合成点击不派发 iced 按钮——按钮驱动一律 MCP press，hover 验证靠用户
   真鼠标。
3. **快照伪影**：快照可整体省略 button 的 onclick/style 行（实际已接线），也可截断
   后半棵树——判「没接线」必须 MCP press 佐证。
4. **数据构造**：plans/chats 均可免登录 API 造数（POST/PUT 直出），用后删档；跨工作区
   验证优先造数而非驱动 ws 选择器（Choose 按压冻结家族）。
5. **实例存活**：共享路径 release auto.exe 会被兄弟 agent 清场误杀——验证实例用
   私有路径拷贝（`auto-rel.exe` 法）。

## 6. 「文件 / 计划 / 规范 / 知识库」归属架构问题（开放裁定，T-10 回填）

### 背景（用户口径）

- musk 的「计划」「规范」「知识库」本质上是 `../jade-edit` 的功能域；「文件」（浏览/
  编辑）本质上是 `../auto-edit` 的功能域。musk 现状 = 两个域的**轻量内嵌复刻**
  （各自 .at 视图 + 本仓 store，PLAN-024/068 起步）。
- 已知约束：**AutoUI 尚无直接「内嵌」另一个 app 的能力**（用户确认）。

### 选项盘点

**A. 组件/bp 复用（近期可行）**
- 现状机制：跨 app 组件引用**无现成机制**——AutoUI 组件来源=本仓 `src/front` +
  ports；bp（auto-lang blueprints）是参考实现/脚手架物，**不是运行时组件库**，
  「引用 bp」实操=物化移植（PLAN-614 FileTree 移植为先例：从 auto-os 组件移植进
  musk，双轨消费）。
- 做法：把两域的**查看器/编辑器纯 UI 组件**（文件树、代码查看/编辑、markdown 文档
  面）下沉为可共享组件（候选落点：auto-lang 组件层或独立组件仓，经 bp 物化对齐
  契约），musk 与 jade-edit/auto-edit 共同消费。
- 成本：组件契约对齐 + 多端消费的升级联动维护。收益：musk 一体化体验不变、无内嵌
  依赖、两域功能一处升级多处受益。

**B. 直接调用（中期路线，依赖能力立项）**
- 桌面壳现有开窗/session 体系只能「另开 app 窗口」，非面板级内嵌——交互与布局
  割裂，不满足 musk 当前一体化形态。
- 前置：auto-lang 立「AutoUI app 内嵌/host 能力」（面板级嵌入另一 app 的路由/视图
  切片）。能力落地后，musk 的 文件/计划/规范/知识库 四栏切换为内嵌
  jade-edit/auto-edit 面板，musk 退役对应复刻视图（store/桥保留为数据通道）。
- 收益：功能归位（编辑能力/域深度不再复刻）、musk 减面。风险：跨 app 版本耦合、
  内嵌协议设计成本。

### 推荐

**近期 A、中期 B**：
1. 本 phase T-07..T-09 以 musk 侧组件先把「能看」修到位（.md/.ad autodown + 
   code_editor）；其中 code_editor 若具备下沉条件则顺手组件化（为 A 铺路）。
2. 在 auto-lang 侧立「AutoUI app 内嵌能力」议题（B 的前置）——能力就绪后按
   文件→编辑器、计划/规范/知识库→jade-edit 的顺序逐栏切换内嵌，复刻视图退役。
3. 裁定待用户拍板；拍板后回填本节并驱动 T-10。

### 与既有资产的衔接

- `../jade-edit`：完整 Auto app（pac.at/src/gen/rust-workspace 全套）。
- `../auto-edit`：编辑器项目（docs/specs/tools；M 轨计划进行中——PLAN-010
  m2-session-restore executing）。
- musk 侧现有可复用底座：FileTree 移植组件（PLAN-614）、autodown 渲染管线
  （PLAN-072/076/082）、code_editor 原生臂（auto-lang 既有）——A 路线的下沉起点。

## 9. 复审记录

- stage: work | plan_id: PLAN-089 | plan_revision: 3 | outcome: pass（Phase 2 范围；
  T-10 待用户裁定，计划保持 executing）| code_commit: musk plan-089-dev `0173917`
  （= main 2d2a7fe + 1 commit）+ auto-lang master `7183ca386`（已折回）|
  task_ids: T-07,T-08,T-09（T-10 open）| evidence: ①web 轨浏览器全链（smoke-089.ad
  点击→files_text→autodown H1/strong/列表/表格 DOM 快照；pac.at→code_editor 行号+
  高亮+contenteditable=false）②后端 curl 三臂（根/%2F 嵌套/404）③files_browser
  单测 7/7 ④auto-lang 范围回归（vnode_converter 23/snapshot 25/code_editor 74/
  新增 mouse_area_press 1+readonly core 1）⑤auto build 门禁绿（R001 解阻）|
  blockers: MCP press 仪器债 §7-1（VM 真鼠标移交）、§6 裁定待用户（T-10）|
  next: 用户真鼠标复验 VM 行点击 + T-10 拍板 → review。

- stage: review | plan_id: PLAN-089 | plan_revision: 3 | outcome: **pass** |
  reviewed_commit: musk plan-089-dev `0173917`（worktree `.wt/musk-089/auto-musk`，
  树净）| base_commit: musk main `2d2a7fe`（注：main 已被 PLAN-088 落地推进，diff
  基以分支基点计；rebase 留 merge 阶段）| dependency_revisions: auto-lang master
  `7183ca386`（本计划折回，含 readonly 穿透+press 链仪器面根修）/ auto-down
  musk-089-dev `3373a5c`（零改动未消费）/ auto-ai musk-089-dev `58bee8d`（零改动，
  RoleConfig.models glue 2 行随 0173917 入库）| spec_inputs:
  docs/specs/modules/files-browser.md（增量已定稿于本计划「规范增量」节：text 端点/
  .ad/只读 code_editor/行点击面三条 modify，AC-1/2/3 映射）|
  acceptance_results: AC-1 **pass**（web 浏览器全链：点击→files_text→内容；
  VM 真鼠标确认随 UAT 移交，§7 债 #1 非阻断）/ AC-2 **pass**（.ad autodown DOM
  快照逐项）/ AC-3 **pass**（行号+三色高亮+contenteditable=false 截图）/
  AC-4 **pass**（三臂代码零改动+kind 分派三例实证+parity_files/mime 单测绿）/
  AC-5 **pass**（T-10 依用户指示「记录+移交」收口，未虚构拍板）|
  findings: F-1 MCP press mouse-area 第四层失配（仪器债，auto-lang 另档，非阻断——
  真机链路无关）；F-2 auto-lang master 既有漂移 4 测试+严格校验器（基线对照实锤，
  非本批，另档）；F-3 冷 target 全量 --tests 的 E0463/E0786（环境伪影：单测/小群组
  全绿+主检出 18/18，非代码回归，挂 087 同族）；F-4 extern_impl RoleConfig.models
  glue 2 行随 0173917 入库（提交说明未点名，plan §3 与本记录补记——行为零变化）|
  evidence: 后端 musk --lib 全量 468/0；worktree parity_files 8+parity_plans 5+
  parity_auth 5 全绿；主检出对照 parity 三组 18/18；auto-lang 范围回归
  vnode_converter 23/snapshot 25/code_editor 74/plan089_mouse_area 1/readonly core 1；
  auto build 门禁绿（Vue project built successfully）；web 全链截图+DOM 快照
  （本计划 §2c/§9 引用）；测试数据 smoke-089.ad 已删档 | next: merge。

## 10. merge 收据（PLAN-089:r3 五检查点）

- **prepared**：复核基线=f1e3266（work 复核绑 0173917，rebase 后恒等映射）；
  canonical Spec diff=files-browser.md 三处 modify（本计划「规范增量」节冻结稿）；
  投影目标=docs/specs/modules/（index.json 已列 files-browser.md，零登记变更）；
  delivery commit=9a76eba（f1e3266 的纯 docs 后代，70cedea→9a76eba）。
- **landed**：main `9a76eba`（ff-only 零 merge commit）；rebase 旧→新映射
  0173917→f1e3266、70cedea→9a76eba，`git range-diff` 两段全 `=`（补丁恒等）；
  落地挡路=主检出 extern_impl models glue 2 行未提交 WIP → stash 让路（stash vs
  landed diff 空=内容恒等）后 drop；主检出 cargo check -p musk 健康门见下。
- **ledger_refreshed**：musk 惯例（PLAN-086 先例）=canonical modules/*.md +
  index.json 登记（files-browser.md 既有条目，零变更）；.autoos/specs.json 结构化
  账本全库零历史（六段 items 恒空=真实沉淀留白惯例），本批不动。
- **archived**：docs/plans/archived/089-vm-uat-fixes-batch.md，status: archived，
  completion_kind: **delivered**（T-01..T-10 全勾；AC-1..5 pass 见 §9 review 行）。
- **cleaned**：见下方补充行（wt/分支/组目录拆除回执）。
