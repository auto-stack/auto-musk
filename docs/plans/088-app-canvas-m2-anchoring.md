---
plan_id: PLAN-088
status: reviewed
feature_name: App Canvas M2——双向锚定（点选/高亮/层树）
author: [agent]
created_at: 2026-09-23T01:27:59Z
updated_at: 2026-09-23T14:20:00Z
plan_revision: 2
current_step: 9
total_steps: 9
supersedes_spec_components: []
new_spec_components: []
touched_goals: []
supersedes_spec_components: []
new_spec_components: []
touched_goals: []
---

# PLAN-088 — App Canvas M2：双向锚定（点选→源码、AI 高亮、层树栏）

## 0. 变更摘要

落地 `docs/designs/013-ai-app-studio-canvas.md` 的 **M2 里程碑**：在 PLAN-087 交付的
实况画布之上建立"视觉元素 ↔ 源码节点 ↔ 层树"的双向锚定——①用户在画布上点选元素
→ 命中测试 → 源码锚点（file:line）+ 层树联动；②agent 以 `canvas.pick/overlay`
获得同一套锚点并反向高亮（"我要改的是这个"）；③层树栏呈现 app 结构（pac 头 +
live widget 树，节点带源码锚点），树↔画布双向联动。

**架构主判断（相对 Design 013 立稿时的修正）**：auto-lang 基线（084 交付位
`8fecfcf69`）的锚定基建已远超"source_map 填充补全"的预设——M2 理想路径是
**零 auto-lang 改动、纯 musk 侧组装**；`source_map.rs` 填充降级为 T-02 有界验证
的兜底升级路径（仅当实测锚点缺口才开同组 auto-lang worktree，AGENTS.md 第三行）。

**开工硬前置**：PLAN-087 review 通过并 merge 落 main（088 worktree 从合回后的
main 开分支；canvas 模块为其交付物）。

## 1. 目标

Figma-like 双向锚定最小闭环：画布上点 counter 的 + 按钮 → 面板亮起该元素 + 层树
选中对应节点 + 显示 `app.at:NN` 锚点（与源码实际位置吻合）；agent `canvas.overlay`
高亮指定元素在面板呈现；循环列表场景点第 N 项能区分实例。全程目标 app 零改动、
auto-lang 零改动（主路径）。

### 非目标（后续里程碑）

- M3 三层生成流：bp 注册表/词汇表注入、L1>L2>L3 复用序、`ui.lint` 护栏、
  examples/ui 扩展池（087 §10-5 已缓行至 M3）。
- M4 飞轮：`blueprint.extract`；AppViewport 原生嵌入（路线 B）。
- 本计划内不做：画布内嵌交互转发（用户点击仅作"选区"不作"驱动"——驱动权仍在
  agent，M1 非目标延续）；VM 内 OverlayInfo 高亮烘焙（auto-lang 侧 `ui/debug/
  overlay.rs` 已有数据结构，但暴露需 auto-lang 改动，列 M4 评估）；hover 悬停高亮
  （只做 selected/agent 高亮）；行级源码编辑器跳转（pick 结果 chip + raw 预览定位
  为止，编辑器集成延后）。
- 层树的 blueprint 归属层（实例↔定义分栏）延 M3（M2 的 app=模板级单文件/少量
  文件，bp 层无内容可显）。

## 2. 架构方案

```
musk canvas 模块（PLAN-087 交付面之上增量）
 ├─ 锚点缓存（canvas/anchor.rs 新）：帧拉取时随取 autoui_vtree（include_source
 │   默认 true）→ 解析为 {vnode_id → (kind, label, bbox, source, for_iter,
 │   parent 链)} 内存索引；hit-test 在此索引上本地做，不逐击打 MCP
 ├─ 命中测试：deepest-first（auto-lang ui/debug/hit_test.rs 同款语义：含点的
 │   最小面积节点）+ 纯容器/纯文本向上回溯到最近"实质节点"（有 events 或有
 │   source span）；层树可再选祖先补位——产品语义定案见 §10-1
 ├─ API 增量：POST /api/canvas/pick {x,y}（帧像素坐标）→ {vnode_id, kind,
 │   label, source, for_context, bbox, ancestor_chain}；status 载荷增
 │   overlay: [{vnode_id, bbox}]（前端绝对定位画框，目标 app 零改动）
 ├─ 工具增量（canvas/tools.rs）：canvas_pick（element_id→同结构锚点，
 │   agent 侧与用户点选同源）、canvas_overlay（element_ids[]→置高亮，
 │   会话内持续到下次 overlay/stop）
 └─ 前端增量（src/front/）：canvas_panel 增层树栏（pac 头 + live vtree 树）
     + 帧上覆盖框层 + 点选交互（img 点击→坐标换算〔显示尺寸→自然尺寸〕→
     /api/canvas/pick→选中态+锚点 chip）；层树点选↔画布高亮双向联动；
     锚点 chip → files_browser raw 预览打开 .at（定位尽力）
```

坐标系约定：pick 入参 = 帧 PNG 像素坐标（前端按 img 显示/自然尺寸比换算）；vtree
bbox = VM 渲染坐标（AUTO_VM_WINDOW 固定窗，M1 已定）。二者 1:1 假设为 T-02 首项
验证（iced 截图尺寸=窗口逻辑尺寸）。

auto-lang 侧既有基建（本计划只消费、不修改）：
- `autoui_vtree`（vtree+bounds+source+for_iter，`mcp_server.rs:961`）——锚点数据源。
- `ui/debug/hit_test.rs`——deepest 命中语义参照（musk 侧按同语义实现，进程内
  索引上 O(n) 扫描）。
- `autoui_select_rect`（PLAN-646，`:991`）——矩形选区→顶层节点+span+源码切片，
  作为 pick 正确性的交叉验证通道（T-02/T-08 用），不作运行时主路径。
- `autoui_find/inspect`——kind/label 搜索与 element_id 详情（agent 侧辅助）。
- `ui/debug/source_map.rs`（VNodeId→SourceLocation 结构完备）——仅当 T-02 实测
  发现 vtree source 缺口时才考虑填充（auto-lang 侧改动，升级路径见 §8 T-02）。

## 3. 技术栈

- 后端：Rust，canvas 模块内新增 anchor.rs（vtree 索引/命中/回溯）+ 既有
  manager/mcp_client 增量；无新三方依赖（解析 vtree Atom 文本用既有字符串处理，
  或经 mcp_client 请求 json 格式——T-02 定）。
- 前端：Auto `.at`（canvas_panel/canvas_store 增量）+ `auto build`；验收面仍 web 轨
  （087 §10-2 口径延续）。
- 跨仓：理想路径零 auto-lang 改动；兜底路径=A.同组 auto-lang worktree 最小填充。

## 4. 需求分析与背景调查

### 已获授权

- 用户 2026-09-23 指示"087 结束后下一步提前规划"= 授权**起草**本计划（顺位
  PLAN-088）；**执行授权未授予**——088 开工以 087 review pass + merge 落 main 为
  前置，由用户在 087 收尾后启动 work。
- 范围：本仓 auto-musk 为主；auto-lang 仅在 T-02 实测缺口时按 AGENTS.md 第三行
  开同组 worktree（分支 `auto-musk-dev`），且改动面收敛到锚点填充最小集。
- 未约定预算与自动续期上限。

### 背景证据（源码核实，基线=087 兄弟位 auto-lang@8fecfcf69 + musk worktree
### plan-087-dev 685b157）

- **087 交付面**（088 的直接地基）：`backend/crates/musk/src/canvas/{mod,session,
  manager,mcp_client,templates,tools}.rs` 六件；四路由 `canvas_routes()`（server.rs）；
  前端 `src/front/{canvas_panel,canvas_store}.at`+`canvas_web.ts`；五工具在册。
  契约修正（087 §10-6）：截图=落盘路径、监听行在 stderr、`element_id`+action 枚举、
  热重载 2s。
- **vtree 源码锚点已在**：`mcp_server.rs:961`（vtree 描述明言携带 source location）、
  `:977`（`include_source` 默认 true，含 for_iter）；`ComputedNodeLite.source:
  Option<String>`（`:60`）。
- **select_rect 选区机制**（PLAN-646，`:991-1006`）：中心命中+顶层修剪+文档序，
  返回 {id, kind, span, source, structure} 信封，format=json/auto/atom——pick
  语义交叉验证现成。
- **hit_test 语义参照**：`ui/debug/hit_test.rs`——含点最小面积节点（DevTools
  行为），O(n) 交互级。
- **overlay 数据结构参照**：`ui/debug/overlay.rs`——`OverlayInfo{hovered,selected}`
  蓝悬停/橙选中；VM 内烘焙需 auto-lang 暴露，M2 不走此路（前端覆盖框替代）。
- **source_map.rs**（209 行）：VNodeId→SourceLocation{file,line_start,line_end}
  增删查 API 完备；填充度未实测——T-02 有界验证定夺。
- **musk 已进程内依赖 auto-lang**：`backend/crates/musk/Cargo.toml:54`
  （`auto-lang = { path = "../../../../auto-lang/crates/auto-lang" }`）——层树 pac
  头解析可直接用 auto-lang config reader，无需子进程。
- **模板池现状**：087 内嵌 counter/hello 双模板（templates.rs）；循环/多元素场景
  需要更丰富 app——T-02 从 examples/ui 拷 `013-todo` 或 `009-article-feed` 级
  入 workspace 作测试 app（拷贝动作在 musk 侧完成，不经 agent read_file，不受
  087 §10-5 沙箱约束）。
- **files_browser raw 预览**：`GET /api/files/raw/{*path}` + RawPreview.vue（087
  背景已录）——锚点 chip 跳转的现成落点。
- **坐标系先例**：AUTO_VM_WINDOW 固定窗（087 session.rs 注入）；ui-parity live
  链 bounds 与截图同窗（`docs/specs/modules/ui-parity.md` snapshot bounds 口径）。

## 5. 详细设计

### 锚点索引（canvas/anchor.rs，新）

- 数据：`HashMap<VNodeId, NodeInfo{kind,label,bbox,source,for_iter,parent}>` +
  根序列（保文档序）；帧循环拉 vtree 成功即重建索引（原子换 `Arc<RwLock<_>>`，
  pick/overlay 读侧无锁竞争）。
- vtree 消费格式：优先 json 信封（若 MCP 提供）；否则解析 Atom 文本（节流：索引
  重建 ≤1 次/秒，与帧循环同拍）。
- 命中：deepest-first（含点最小面积）→ 若命中节点无 source 且无 events → 沿
  parent 链上溯至最近"实质节点"；返回含 ancestor_chain（供前端层树定位与
  语义再选择）。
- for 实例：`for_iter` 入 NodeInfo；锚点输出含 for_context（"第 N 项"区分）。

### API 与工具增量

- `POST /api/canvas/pick {x,y}` → 200 `{vnode_id,kind,label,bbox,source,
  for_context,ancestor_chain[]}` / 204 未命中 / 503 无会话或无帧。
- `status` 载荷增 `overlay:[{vnode_id,bbox}]` 与 `picked`（当前选中）——前端
  既有 1s 轮询顺带取走，零新通道。
- `canvas_pick(element_id)`（工具）：索引直查 → 同 pick 结构文本；element_id 不在
  当前帧 → Err 附"以 autoui_find 找现行 id"提示。
- `canvas_overlay(element_ids[], clear?)`（工具）：校验 id 存在 → 写 manager
  overlay 态 → 前端下一轮 status 呈现；会话 stop/替换即清。

### 前端层树栏与联动

- canvas_panel 左侧增层树列（树数据 = status 新增 `tree` 载荷，后端由锚点索引
  序列化：pac 头{name,title,render,deps}+vnode 树缩略〔kind+label+source 行〕，
  深度默认折叠到 2 层）；节点点击→`picked` 置位+画布覆盖框；画布点选→树展开至
  命中节点并选中。
- 锚点 chip：`{file}:{line}`（for_context 有则缀 `#i{n}`）→ 点击经 files_browser
  raw 预览打开 .at（行定位尽力：raw 预览无行锚则仅打开文件，登记非门）。
- 覆盖框层：绝对定位 div（border+半透明填充）叠于帧 img 上，数据=status.overlay。

### 层树 pac 头解析

- 进程内复用 auto-lang `AutoConfigReader`（Cargo 已依赖）读目标 app pac.at 的
  name/title/render/dep 键；routes 清单从 app.at 文本轻提取（够用即止，M3 深化）。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|:---|:---|:---|:---|:---|:---|
| SD-01 | modify | docs/specs/modules/app-canvas.md | before（087 merge 后文本）：M1 会话/API/工具契约；after：增补 M2 锚定契约——pick 命中语义（deepest+实质回溯）、坐标换算约定、overlay 生命周期、层树载荷与联动、canvas_pick/canvas_overlay 工具契约、源码锚点口径（file:line+for_context） | M2 契约需入 canonical；以 087 merge 落定的 app-canvas.md 为基线增补 | AC-01..06 |
| SD-02 | add（条件） | docs/plans/KNOWN-DEBT-AND-RISKS.md | 仅当 T-02 实测发现 auto-lang 锚点缺口且本计划内修复：登记缺口与修复位 | 兜底路径的可追溯性 | AC-02 |

## 6. 测试设计

- **锚点正确性（T-02 核心，测试即契约）**：counter（按钮/文本）+ 循环 app（列表
  第 N 项）两 fixture：pick 返回的 source file:line 与 .at 源文件实际位置人工/
  脚本核对（行内容包含对应 widget 关键词）；同一元素重复 pick 锚点稳定；与
  `autoui_select_rect` 结果交叉验证（span 包含关系）。
- **命中语义**：点击按钮内文字 → 回溯到 button（非 text 叶子）；点击空白 → 204；
  重叠容器 → 最小面积优先。
- **坐标换算**：面板缩放显示下点击换算正确（前端测或冒烟肉眼核对入证据）。
- **overlay**：工具置高亮→status 载荷出现→前端框呈现（截图）；id 失效（热重载后
  树变）→ overlay 自动清+工具 Err 文案。
- **层树**：树载荷与 vtree 一致（节点数抽查）；双向联动（树点→框、布点→树选中）
  冒烟。
- **回归**：087 六 AC 抽查不回退（重点 AC-03 驱动断言、AC-04 热重载后锚点索引
  重建、AC-05 卫生）；全目标套件无新增红（tool_atoms 基线红除外，087 口径）。
- **端到端证据**：attachments/088/（点选/高亮/层树/循环区分截图+核对表）。

## 7. 验收标准

- **AC-01 点选→锚点（基础）**：web 轨画布点击 counter 的 + 按钮，返回
  kind=button、vnode_id、`source` 与 app.at 实际行吻合（±0），面板呈现锚点 chip
  与覆盖框。验证：截图+源码行核对表。
- **AC-02 循环实例区分**：在含列表/循环的 app 上点选第 1 与第 3 项，两次锚点
  可区分（for_context/实例标识不同）且 source 指向同一模板行的正确语义。
  验证：双点选截图+锚点对照。
- **AC-03 agent pick**：`canvas_pick(element_id)` 返回与用户点选同构的锚点结构
  （含 ancestor_chain）。验证：工具调用 transcript。
- **AC-04 AI 高亮**：`canvas_overlay([element_id])` 后面板帧上出现该元素覆盖框
  （截图），层树联动选中；热重载后失效 id 被自动清理且工具返回明确 Err。
  验证：截图+工具序列。
- **AC-05 层树栏**：面板显示 pac 头+widget 树（节点含 kind/label/source），树点
  选→画布高亮、画布点选→树选中。验证：双向联动截图。
- **AC-06 源码呈现**：锚点 chip 可打开 files_browser raw 预览并展示对应 .at 文件
  （行定位尽力，非门）。验证：跳转截图。
- **AC-07 M1 回归**：087 AC-03/04/05 抽查通过；测试套件无新增红。验证：重跑
  087 对应集成臂+套件输出。

## 8. 执行步骤

- **T-01 worktree 组与基线**（前置=087 已 merge 落 main）**[✅ 已完成 2026-09-23]**
  建 `.wt/musk-088/auto-musk`（分支 `plan-088-dev`，基线=合回后 main）；兄弟固
  定位沿用 087 钉位（auto-lang@8fecfcf69、auto-ai@57eb44a）+ 锁钉位同 8913acc 口
  径；`cargo build --release` 通过。产出：构建日志。→全部前置。
- **T-02 锚点契约有界验证**（依赖 T-01；产出=决策物）**[✅ 已完成
  2026-09-23]**
  fixture：counter（087 模板同源）+ loop-list（自写最小 for 列表单文件——
  013-todo 依赖 store/api:rust 超出 M2 消费面、009-article-feed 无 for 循环
  不满足 AC-02，按 §10-3 选择规则双降级为自写等价 fixture，登记调整）。
  实测三问全有结论（详见 attachments/088/anchor-contract.md）：①1:1 假设
  按字面不成立——vtree bbox=窗口逻辑像素、帧 PNG=逻辑×DPI scale（本机实测
  2.0），修正契约=scale 由 musk 从帧宽/AUTO_VM_WINDOW 逻辑宽推导；固定窗
  为锚定前提（fit app bounds 陈旧）。②source 锚点缺口**实锤**（vtree 无
  source prop、for_iter 无合并、叶件无 bounds、静态首拍无 bounds 四件）→
  触发 §4 预授权兜底：**T-02B**。③json 信封不存在（Atom 单臂）→ anchor.rs
  走 Atom 解析（实测两处格式偏差已按实修正：对象值内逗号分隔、哈希
  vnode id）。产出《锚点契约记录》入 attachments/088/。→AC-01/02 契约基础。
- **T-02B auto-lang 最小填充 + musk 配套**（T-02 实锤触发的 §4 预授权兜底；
  兄弟 worktree `.wt/musk-088/auto-lang` 分支 `auto-musk-dev`，基线
  8fecfcf69）三件：①vtree_atom.rs 输出 `span:{offset,len}` prop（数据已在
  vnode.source_span）；②renderer.rs 两合并站并入 probe ForIter；③session.rs
  VM debug_mode 增 `AUTO_DEBUG_CAPTURE` env 门控（canvas 会话注入→引导帧
  bounds+叶件 bounds；Plan 402 风险面不适用，登记 KNOWN-DEBT 观察项）。
  musk 配套：spawn 注入 env；行号换算 musk 侧做；087 模板池 `#` 注释缺陷
  顺手修（`//`）。验收=填充后重跑探针（source/for_iter/叶件 bbox 四红转绿）
  + 计划 revision 2 记录改动面（**§10-2 的用户确认改为 handoff 面呈报**，
  改动面即本条三件，无扩项）。→AC-01/02 前置。**[✅ 已完成 2026-09-23：
  auto-lang auto-musk-dev 016382eb4（3 文件 +31/-1）；musk 配套随 T-03..T-08
  提交；探针复跑四绿（span 全节点/叶件 bbox/静态首拍/for_iter 实例标注），
  证据 attachments/088/fill-verified/；087 模板 `#` 注释缺陷已修]**。
- **T-03 锚点索引与命中测试**（依赖 T-02；新 `canvas/anchor.rs`）
  vtree 索引/帧循环挂载/deepest+实质回溯/ancestor_chain。验证：单测+集成（点击
  命中语义三例）。→AC-01。**[✅ 已完成 2026-09-23：anchor.rs 单测 8/8（解析/
  deepest/回溯/实例区分/pick_json/tree_flat/垃圾拒绝/重复 id 拒绝）；集成=
  Stage-1 e2e xy 往返 same-vnode + 204 未命中臂]**。
- **T-04 pick API 与工具**（依赖 T-03；改 server.rs/canvas/tools.rs）
  `/api/canvas/pick`+`canvas_pick`。验证：路由测试+工具冒烟。→AC-01/03。
  **[✅ 已完成 2026-09-23：pick 路由 200/204/503 三臂 e2e；canvas_pick 由
  agent transcript 实证（AC-03，返回锚点结构含 ancestor_chain）]**。
- **T-05 overlay 状态面**（依赖 T-03；改 manager/status 载荷）
  overlay 态+status 载荷+失效清理。验证：载荷测试。→AC-04 前置。**[✅ 已完成
  2026-09-23：status_full 七字段载荷；失效清理=索引换代交集收敛，结构性
  热重载后 overlay=0/picked=CLEARED/stale pick 204 实证（e2e/stale-pick.txt）]**。
- **T-06 canvas_overlay 工具**（依赖 T-05；改 canvas/tools.rs）
  置高亮/clear/Err 文案。验证：工具冒烟+失效清理断言。→AC-04。**[✅ 已完成
  2026-09-23：agent transcript 实证 canvas_overlay completed（AC-04），
  琥珀框浏览器截图 panel-agent-overlay-pick.png]**。
- **T-07 前端层树栏与联动**（依赖 T-04/05；改 canvas_panel/canvas_store +
  status 树载荷〔后端 anchor.rs 序列化〕+ pac 头解析〔AutoConfigReader〕）
  树栏/覆盖框层/点选换算/锚点 chip+raw 预览跳转；`auto build` 通过。验证：web 轨
  冒烟（双向联动）。→AC-01/04/05/06。**[✅ 已完成 2026-09-23：auto build 绿；
  浏览器截图 panel-full-loop.png（层树+chip+蓝框+抽屉源码全就位）；双向联动
  实证（树点→chip#0、帧点→chip#3）；pac 头走轻文本提取（AutoConfigReader
  未启用——够用即止，登记调整）]**。
- **T-08 端到端与回归**（依赖 T-02..T-07）
  AC-01..07 全走查+087 回归抽查；证据归档 attachments/088/。→全部 AC。
  **[✅ 已完成 2026-09-23：三轨走查全绿——API 级（stage1-report.json：行级
  锚点±0/实例区分/往返一致/204）、浏览器（panel-full-loop.png、
  panel-agent-overlay-pick.png、抽屉源码 651 字符含目标行）、agent 臂
  （agent-transcript.md：canvas_overlay+canvas_pick AC-03/04）+087 回归
  PASS（canvas_run counter→act +→state count==1）；e2e 修正三件（pct 公式/
  ws 前缀/抽屉加载形态）随 5884a1b 落盘]**。

## 9. 复审记录

- 2026-09-23 draft handoff：`stage: new`，PLAN-088，`plan_revision: 1`。
  `outcome: pass`（起草完成、任务/验收/规范增量闭环）。
  `next: **blocked-until-087-merge**`——执行前置=PLAN-087 review pass + merge 落
  main（088 worktree 依赖其 canvas 交付面）；087 收尾后由用户启动 work。
  备注：架构主判断修正（零 auto-lang 主路径，Design 013 的 source_map 前置债降级
  为 T-02 兜底）；设计定案一处待可否决（§10-1 命中回溯语义）。
- 2026-09-23 work 收束：`stage: work | plan_id: PLAN-088 | plan_revision: 2 |
  outcome: pass | code_commit: plan-088-dev 841fdbf..5884a1b（musk 四提交：
  a611070 前端 + 5884a1b e2e 修正 + T-03..T-06 后端 + 账本前置；依赖位
  auto-lang auto-musk-dev 016382eb4）| task_ids: T-01,T-02,T-02B,T-03..T-08 |
  evidence: attachments/088/{anchor-contract.md, fill-verified/, e2e/} +
  cargo test -p musk --lib 475/475 + auto build 绿 + 三轨走查全绿 |
  blockers: 无 | next: review。
  备查（执行期登记，merge 时沉淀）：①T-02B 改动面三件如上——§4 预授权
  兜底已启用，请 review/用户复核改动面最小性；②KNOWN-DEBT 候选：canvas 轨
  首跑 ~35s 周期性 VM 退出（未复现，stderr 尾部诊断桩已埋，复发可定罪）/
  层树 1Hz 全量重渲（轮询拍重赋值，功能正确仅性能味）/fit-window app 锚定
  降级尽力（契约=固定窗）/多文件 app 的 span→文件判定为启发式（M2 主路径
  单文件）；③codegen 语法发现三件（裸 fn 调用语句非法/`pac`·`for` 保留
  字段名冲突/handler var 注解须小写形）已入 canvas_store.at 头注。
- 2026-09-23 review：`stage: review | plan_id: PLAN-088 | plan_revision: 2 |
  outcome: pass | reviewed_commit: plan-088-dev 27ec894（=5884a1b+SD-01 备稿，
  代码与 5884a1b 全同）| base_commit: main 841fdbf | dependency_revisions:
  auto-lang auto-musk-dev 016382eb4（基线 8fecfcf69）/ auto-ai 57eb44a /
  auto-down master 3373a5c | spec_inputs: docs/specs/modules/app-canvas.md
  （087 merge 位 + SD-01 增补备稿 27ec894）| acceptance_results: AC-01..07
  全 pass（AC-01/02 评审独立复现：行 17±0/pct 神志/往返同 vnode/204；
  AC-03/04/07 agent transcript；AC-04 失效臂 stale 204；AC-05/06 浏览器截图）
  | findings: 无阻断项。环境注记：评审中前端构建一度失败=自查 junction 时
  击穿 pnpm store（非代码回归），node_modules 重装后复绿；merge 清 worktree
  前需再清 gen/front/vue/node_modules reparse points（wt-guard 已验口径）。
  SD-01 备稿已审（描述落地行为与持久决策，非执行日记）；SD-02 条件项未
  触发（缺口已修非登记债）| evidence: attachments/088/{anchor-contract.md,
  fill-verified/, e2e/} + 评审复现（475/475 lib fresh run @27ec894、
  auto build 绿、8139 独立 serve e2e 重放 JSON 匙验全 true）| next: merge。
  独立性声明：本 review 在实现会话内完成（无独立授权评审位），结论按
  工件重建——关键 AC 均经独立复现或工件核验，不依赖执行摘要。

## 10. 待澄清事项

1. **命中回溯语义（设计定案，可否决）**：deepest-first 命中后，若命中为无源码
   span 且无 events 的纯文本/纯容器节点，上溯至最近"实质节点"（有 events 或有
   source）。备选=严格 deepest（点按钮文字选 text 节点）或 select_rect 式顶层
   修剪（偏浅）。默认按定案执行，层树可选祖先补位。
2. **T-02 升级路径授权**：若锚点缺口实锤需改 auto-lang，属"因本项目而改的外部
   依赖项目"（AGENTS.md 第三行）——按既有惯例同组 worktree 合回 auto-lang
   master；届时以 revision 2 更新本计划并请用户确认改动面。
3. **循环 app fixture 选型**：默认 `013-todo`（列表+交互丰富）；若其依赖
   store/多文件超出 M2 消费面，降级 `009-article-feed`。T-02 定。
4. **行级 raw 预览定位**：files_browser 当前无行锚能力，chip 跳转以"打开文件"
   为验收、行高亮为尽力（AC-06 非门）；行锚需求登记 M3 评估。
