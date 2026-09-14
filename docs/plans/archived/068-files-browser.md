---
plan_id: PLAN-068
status: archived
feature_name: 文件浏览大模块——一级导航"文件" + workspace FileTree + 分类型内容查看器（代码只读高亮 / markdown / 媒体）
author: zhaop / zcode
created_at: 2026-09-14T14:20:00+08:00
updated_at: 2026-09-14T16:40:00+08:00
plan_revision: 1
current_step: 6
total_steps: 6
supersedes_spec_components: []
new_spec_components:
  - docs/specs/modules/files-browser.md
touched_goals: [goal-frontend-parity]
---

# PLAN-068 — 文件浏览大模块

## 0. 变更摘要

为 musk 补上缺失的文件浏览能力：左侧一级导航新增 **"文件"** 入口；点击后内容区切换为
FilesView——左栏用 **FileTree** 展示当前 workspace 根目录树，右栏为分类型内容查看器：

| 文件类型 | 打开方式 |
|:---|:---|
| 文本/代码（.at/.rs/.ts/.vue/.json/…） | 只读高亮代码查看器（v1 只读；编辑器见待澄清①） |
| markdown（.md/.markdown） | autodown 渲染（与聊天同源管线） |
| 图片（png/jpg/gif/svg/webp/…） | 浏览器原生 `<img>` |
| 视频（mp4/webm） | 浏览器原生 `<video controls>` |
| 其他（二进制/未知） | "不能打开"空态 |

核心复用：FileTree 组件族从 auto-os widgets-gallery（PLAN-614）移植；后端文件 API 从
PLAN-025 的 `spec_tree.rs`（docs/specs 树浏览器）泛化到 workspace 根目录，新增忽略规则
与容量上限。

## 1. 目标

1. 一级导航可见"文件"入口（i18n 中英），点击进入 FilesView，再点其他视图正常切回。
2. FileTree 展示 workspace 根目录真实结构：目录优先、按名排序、展开/收起、目录/文件按
   扩展名配图标；忽略清单内的路径不出现。
3. 点击文件按类型打开查看器，全部只读，不提供任何写路径。
4. 安全面：路径逃逸（`..`、绝对路径、符号链接穿出）一律拒绝；忽略规则服务端强制。
5. 双轨对齐：.at 单源，vue 轨与 VM 轨同一份 FilesView/FileTree 可用（PLAN-067 T-06
   对齐登记口径）。

**非目标**：文件编辑/保存/重命名/删除；搜索；懒加载子树（v1 全量树 + 服务端上限）；
视频拖动进度条（Range 请求，v1 只要求能播）；自定义图片/视频预览面板（原生标签足够，
见待澄清④）；hidden 文件显示开关。

**涉及仓库**：仅 auto-musk（组件从 auto-os/widgets-gallery **复制移植**，不改对方仓）。

## 2. 架构方案

```
一级导航 app.at ── ShowFiles ──► current_view == "files" ──► FilesView
                                                                │
              ┌─────────────────────────────────────────────────┘
              ├── 左栏: FileTree (filetree.at + tree_util.at + tree_icon.at 移植)
              │           │ 点击叶子/文件
              │           ▼
              │   forge_store FilesState: files_tree / files_selected / files_cache
              │           │ Http.*
              └── 右栏: ViewerPanel ┐
                    ├─ 代码:   CodeViewer(只读高亮, prismjs 同源)
                    ├─ md:     autodown 渲染(聊天管线同源)
                    ├─ 图片:   <img src="/api/files/raw/{*path}">
                    ├─ 视频:   <video src=同上 controls>
                    └─ 其他:   "不能打开"空态

后端(hw 逃生舱, PLAN-025 spec_tree 先例):
  GET /api/files/tree            → Vec<TreeNode>（workspace 根, 忽略规则+上限）
  GET /api/files/raw/{*path}     → 原始字节 + MIME（confinement 校验）
```

选型理由：
- **hw 逃生舱路由**（`files_routes()` 挂 server.rs，同 `spec_tree_routes`/`plans_routes`）：
  文件系统遍历是基础设施逻辑，带忽略规则/上限/安全校验，不适合走 api.at 生成轨；
  api.at 以注释登记契约（PLAN-024/025 同款口径）。
- **组件移植而非跨仓引用**：auto-musk 的 .at 组件为平铺 `use`（`use mention_dropdown:
  MentionDropdown`），与 widgets-gallery 的 package 形态不同源；以复制 + 头注保留
  来源与 PLAN-614 署名（vendor-autodown-vue 脚本同理念，v1 手动复制）。
- **只读查看器自研轻量**：widgets-gallery 的 `code_block.at` 是 VM 轨骨架（view 仅占位
  div），vue 轨现成能力是 prismjs（gen 依赖已有）+ 聊天代码块渲染；v1 查看器 =
  prismjs 高亮 + 行号 + 复制钮，不做编辑。

## 3. 技术栈

- 后端：Rust/axum（hw 路由，复用 `spec_tree.rs` 的 `build_tree`/`validate_path_pub`/
  `guess_mime` 泛化）
- 前端：Auto .at 单源（auto build → Vue SFC / VM 双轨）；FileTree 用 mouse-area/icon/
  class 字面量（gallery 原样，双轨已验证）；markdown 管线与聊天同源（@autodown/vue）
- i18n：vue-i18n，`src/front/i18n/{zh,en}.json` 增 `nav.files` + `files.*` 键
- 测试：Rust 单测（忽略规则/confinement/上限）+ vitest（tree_util 纯函数 + viewer
  分型）+ 双轨手测脚本

## 4. 需求分析与背景调查

**授权记录**（2026-09-14 用户需求原话综合）：一级导航加"文件"；FileTree 浏览 workspace
目录；文本代码文件用 codeeditor（**或其只读模式**）显示（**或参考 auto-edit**）；
markdown/autodown 文件用 autodown 组件打开；图片/视频用浏览器自带播放器（**或设计
image/video 预览面板**）；其他类型显示"不能打开"。三处括号 = 用户已预留的实现弹性，
本计划取"只读模式 / 原生标签"侧并记录理由如下。

**背景调查**（证据路径）：

| 事实 | 证据 |
|:---|:---|
| FileTree 已存在于 auto-os widgets-gallery（PLAN-614）：`filetree.at`(81 行, fs 形态
  node schema `{id,label,children,kind,icon,is_leaf,badge}`, 内持展开/选中态) +
  `tree_util.at`(216 行, flatten_tree/toggle_id/has_id 纯函数, VM 轨 while+索引纪律) +
  `tree_icon.at`(调色板分支发射, vue 轨 icon 只认字面量名的绕法) + 单文件单 widget
  纪律（auto run 增量路径预存 bug 规避） | `D:/autostack/auto-os/widgets-gallery/src/front/components/` |
| **CodeEditor 组件不存在**：全 AutoStack 仓内容级搜索无果（auto-musk src/front、web/、
  frontend/、auto-edit（仅空 docs 目录）、auto-ui、auto-os、auto-down 等）；gallery 有
  `code_block.at` 但为 VM 轨骨架（view 仅占位 div），非可用编辑器 | 本计划 T-00 调查记录 |
| specs 树先例：`GET /api/specs/tree` + `/api/specs/file/{*path}`（PLAN-025，hw 逃生
  舱），`build_tree`（目录优先/字母序）、`validate_path_pub`（拒 `..`/绝对路径）、
  `guess_mime`（.md→text/markdown）可直接泛化；`spec_file` 已是原始字节 + MIME 响应 | `backend/crates/musk/src/spec_tree.rs:39-90` |
| Auto 轨已有树渲染消费先例：SpecsView tree 模式渲染 `store.spec_tree` 节点 | `src/front/specs_view.at:70,89,151-153,318` |
| 一级导航接线：sidebar_menu_item + `onclick: .ShowX` + `current_view == "x"` v-if 分支
  ；i18n text 均已 t() 化 | `src/front/app.at:100-135` |
| vue 轨 markdown/代码渲染依赖就绪：@autodown/vue + @autodown/engine（vendor）、prismjs
  ；聊天流内 markdown 已对齐（PLAN-067） | `gen/front/vue/package.json`、`src/front/chat_message.at` |
| workspace 根语义 = registry 中 workspace 的 root（同 `specs_dir_for` 的取法），数据
  落 `.autoos/`（须入忽略清单） | `spec_tree.rs:39-43`、`docs/specs/00-overview.md` |
| 构建宿主链：`auto build`（.at→SFC+Rust）→ vue-tsc+vite → `musk serve` 托管
  gen/front/vue/dist；PATH auto.exe 已含 popover 臂修复（2026-09-14，master@44e220b0c） | 本会话实测 |

**约束**：只读；workspace confinement；忽略规则服务端强制（前端不作为安全边界）；
VM 轨 `for-in` 对参数列表零次迭代的遍历纪律随移植组件继承。

## 5. 详细设计

### 5.1 后端 API（T-01）

新文件 `backend/crates/musk/src/files_browser.rs`（hw 路由 `files_routes()`，挂载点同
`spec_tree_routes`；`spec_tree.rs` 中 `build_tree`/`validate_path_pub`/`guess_mime`
提为 `pub(crate)` 复用或移入共享模块——以最小 diff 为准，执行期定）：

- `GET /api/files/tree?workspace={id}` → `Json<Vec<TreeNode>>`
  - 根 = workspace root（复用 `specs_dir_for` 的 registry 取法）。
  - 忽略清单（目录名级，常量）：`.git, .autoos, .worktrees, node_modules, target,
    dist, tmp, vendor, .pnpm`；文件名级：`.DS_Store`。
  - 上限：总节点 ≤ 5000、深度 ≤ 12，超限节点截断并在响应附 `truncated: true` 标记
    （TreeNode 增量字段，旧消费方不受影响）。
  - 目录优先 + 字母序（`build_tree` 现行为）。
- `GET /api/files/raw/{*path}?workspace={id}` → 原始字节 + `CONTENT_TYPE: guess_mime`
  - `validate_path_pub` 拒绝逃逸；最终路径 canonicalize 后必须仍以 workspace root 为
    前缀（双保险，防 symlink 穿出）。
  - 大小上限 20 MB：超限返回 413（前端提示"文件过大，无法预览"）。
  - 404 语义与 spec_file 一致（不存在/读失败）。
- 单测：忽略规则命中/不误伤、confinement（`../`、绝对路径、根外符号链接）、截断上限、
  MIME 猜测、空 workspace 空树。
- api.at 注释登记契约（hw 口径，PLAN-024/025 同款）。

### 5.2 FileTree 组件移植（T-02）

- 复制 → `src/front/filetree.at`、`src/front/filetree_util.at`、`src/front/tree_icon.at`
  （单文件单 widget 纪律保留；头注保留 PLAN-614 来源标注 + 移植说明）。
- 适配点（预期最小）：`use` 语句改 auto-musk 平铺形态（`use filetree_util: flatten_tree,
  toggle_id`；`use tree_icon: TreeIcon`）；样式 class 字面量原样保留（Tailwind JIT 可
  扫描性由 tree_util 现有 pads 阶梯设计保证）。
- 图标映射：fs 形态 `kind/icon` 自动映射已在组件内（目录/文件按扩展名），执行期对照
  gallery 同版本，不扩调色板。
- 不改 gallery 仓；如发现移植中组件 bug，登记 DEBTS 候选回馈对方仓，不在本计划内修。

### 5.3 状态与数据流（T-03）

`forge_store.at` 增 FilesState（与 spec_tree 消费同款模式）：

- 字段：`files_tree List`、`files_truncated bool`、`files_selected str`、
  `files_cache obj`（path → {kind, content?, raw_url?}，LRU 上限 20 条即可）。
- 动作：`LoadFilesTree`（进视图时拉取；失败置空树 + 错误态）、`SelectFile(path)`
  （写 selected + 按 ext 分型取内容：文本/markdown 走 fetch JSON/text，媒体只记
  raw_url 惰性加载）。
- 文本读取：v1 复用 `/api/files/raw`（text/* 按 UTF-8 文本解），不另设 text 端点。

### 5.4 FilesView 与查看器（T-04/T-05）

- `src/front/files_view.at`：左右布局（左 280px 树栏 + 右 flex-1 内容栏，`overflow`
  各自独立）；顶部窄条显示选中相对路径 + truncated 警示。
- 类型分派（扩展名表驱动，集中一处便于增量）：
  - 代码/文本：`.rs .at .ts .tsx .vue .js .mjs .cjs .json .toml .yaml .yml .css .html
    .sh .py .sql .txt .lock .gitignore` 等 → CodeViewer（prismjs 高亮，语言按扩展名
    映射；行号 + Copy 钮对齐聊天代码块视觉）。
  - markdown：`.md .markdown` → autodown 渲染（管线与 chat_message 现行路径同源，
    执行期对接点以 chat 实现为准）。
  - 图片：`png jpg jpeg gif svg webp ico bmp` → `<img>`（object-contain，深色底）。
  - 视频：`mp4 webm mov`（浏览器可解码面）→ `<video controls>`。
  - 其余 → "不能打开"空态（图标 + `files.cannotOpen` 文案 + 检测到的 MIME 提示）。
- app.at：sidebar_menu 增"文件"项（icon `folder`，`text t("nav.files")`，
  `onclick: .ShowFiles`，`active: .current_view == "files"`）；视图分支增 FilesView；
  msg 增 `ShowFiles`。
- i18n：`nav.files`（文件/Files）、`files.title`、`files.cannotOpen`、`files.tooLarge`、
  `files.emptyTree`、`files.loadFailed`、`files.truncated`。

### 5.6 双轨对齐（T-06）

- .at 单源使 VM 轨自动获得 FilesView；移植组件继承 gallery 的 VM 纪律（while+索引
  遍历、字面量 class、TreeIcon 调色板）。按 PLAN-067 T-06 口径做一轮 vue/VM 差异
  登记文档（已知差异：媒体标签 VM 轨能力待实测，差异登记而非阻塞）。

### 规范增量

| delta_id | 变更 | 目标 | before/after | rationale | AC |
|:---|:---|:---|:---|:---|:---|
| SD-01 | add | docs/specs/modules/files-browser.md | 无 → 文件浏览模块规范：API 契约（tree/raw）、忽略规则与上限、confinement 规则、查看器分型矩阵 | 新模块需沉淀安全面与契约 | AC-01..AC-08 |
| SD-02 | modify | docs/specs/03-front-component-groups.md | G-导航/框架（或新增 G-文件浏览组）→ 登记 FileTree/CodeViewer/ViewerPanel 组件组及 gallery 移植来源 | 组件分组清单完整性 | AC-02,AC-03 |
| SD-03 | modify | docs/specs/01-architecture.md | API surface 段 → 增 /api/files/*（hw 逃生舱口径） | 路由面登记（执行期核对现有段落形态后落笔） | AC-07 |

## 6. 测试设计

- **Rust 单测**（`files_browser.rs` 内 `mod tests`，spec_tree.rs 测试同款 make_dir/
  write 脚手架）：忽略规则、目录优先排序、节点/深度截断、confinement（含 symlink 穿出
  用例，Unix 语义为主，Windows 行为登记）、MIME、20MB 上限、空树。
- **vitest**：`filetree_util` 纯函数（flatten/展开切换/边界：深树、空 children、
  badge）；viewer 扩展名分派表（参数化）。
- **手测脚本**（验收走查单）：登录 → 文件 → 展开到 `src/front` → 打开 `app.at`（代码
  高亮）→ 打开 `README.md`（markdown 渲染）→ 打开 `docs/` 下任一图片 → 打开 `.ico`
  之外未知类型（不能打开）→ 切回会话再切回（状态保持）→ VM 轨 `auto run --render=vm`
  同一轮走查。
- **回归**：全量 `cargo test`（backend）+ `auto build` + vitest 全量。

## 7. 验收标准

| ID | 可观察行为 | 验证方法 | 期望 |
|:---|:---|:---|:---|
| AC-01 | 一级导航出现"文件"（中英随 locale），点击进入 FilesView，再点"会话"等正常切回 | 手测 + i18n 文件 diff | 双 locale 均有键；视图互切无残影 |
| AC-02 | FileTree 展示 workspace 根目录：目录优先字母序，展开/收起/选中态正确，忽略清单路径不可见 | 手测对照 `ls` + Rust 单测 | 树内容与磁盘一致（除忽略项）；单测绿 |
| AC-03 | 点击代码文件（如 `src/front/app.at`）右栏只读高亮显示，含行号与复制钮 | 手测 | 内容与磁盘一致；无编辑入口 |
| AC-04 | 点击 `.md`（如 `README.md`）右栏 markdown 渲染（标题/代码块/表格） | 手测 | 与聊天内 markdown 视觉同源 |
| AC-05 | 点击图片/视频文件，右栏原生预览可显示/播放 | 手测（仓库内放一张图/一段视频或用现有资产） | img/video 正常加载 |
| AC-06 | 点击未知/二进制类型（如 `.exe`、无扩展名），右栏显示"不能打开" | 手测 | 空态文案出现，无报错白屏 |
| AC-07 | 构造 `?path=../auto-musk/pac.at` 或绝对路径请求被拒；忽略目录（如 node_modules）不出现在树 | curl + 单测 | 400/404，树中无该目录 |
| AC-08 | vue 轨与 VM 轨同一 .at 均可进入文件视图并完成 AC-02..AC-06 主路径 | VM 轨手测走查单 | 差异登记文档产出，主路径双轨可用 |
| AC-09 | 超大文件（>20MB）返回 413，前端提示"文件过大" | 单测 + 手测（构造大文件） | 413 + 前端空态提示 |

## 8. 执行步骤

| ID | 任务 | 依赖 | 产出/落点 | 验证 | AC |
|:---|:---|:---|:---|:---|:---|
| T-01 | 后端 files API：`files_browser.rs`（tree + raw，忽略规则/上限/confinement/单测；api.at 注释登记；server.rs 挂载） | - | `backend/crates/musk/src/files_browser.rs`（新）、`server.rs`、`src/back/api.at` | `cargo test -p musk files_browser` 绿 + curl 冒烟 | AC-02,AC-07,AC-09 |
| T-02 | FileTree 组件移植：filetree/filetree_util/tree_icon 三文件复制适配 | - | `src/front/filetree.at`、`filetree_util.at`、`tree_icon.at`（新） | `auto build` 绿；gallery 行为对照 | AC-02 |
| T-03 | forge_store FilesState + LoadFilesTree/SelectFile + i18n 键 | T-01 | `src/front/forge_store.at`、`src/front/i18n/{zh,en}.json` | `auto build` 绿；vitest 状态单测 | AC-01,AC-02 |
| T-04 | FilesView 布局 + app.at 导航接线 + viewer 扩展名分派表 | T-02,T-03 | `src/front/files_view.at`（新）、`src/front/app.at` | `auto build` 绿；手测 AC-01/02 | AC-01,AC-02 |
| T-05 | 查看器实现：CodeViewer（prismjs）、markdown 管线对接、img/video、不能打开/过大空态 | T-04 | `src/front/files_view.at`（或拆 `files_viewer.at`） | vitest 分派表 + 手测 AC-03..06,09 | AC-03..AC-06,AC-09 |
| T-06 | 双轨对齐登记 + 全量回归（cargo/vitest/auto build）+ 验收走查单执行 | T-05 | `docs/plans/068-*.md` 进度回写；对齐登记文档 | 全量绿 + 走查单全过 | AC-08 |

### 8.1 执行勾选与证据

worktree `D:/autostack/.wt/musk-068/auto-musk`（branch `plan-068-dev`，base
main@f685880）。依赖兄弟（只读 detached）：auto-ai@9d2102c、auto-lang@2c92f3755。

- [x] T-01 后端 files API —— commit `d4cac2d`。`files_browser.rs`（tree/raw +
  忽略清单 + 预算 + canonicalize confinement + 20MB 上限 + 扩展 MIME 表），
  `lib.rs`/`server.rs` 挂载，api.at 契约。验证：`cargo test -p musk --lib
  files_browser` 6/6 绿；curl 冒烟（树结构/`..` 与绝对路径与编码绕过 400/
  真实文件 200 text/markdown）。
- [x] T-02 FileTree 移植 —— commit `82c2bfe`。`filetree.at`/`tree_util.at`/
  `tree_icon.at`；纯函数经 `use.web` `@/ext` 导入（mention_helpers 同款）。
  验证：auto build 绿。
- [x] T-03 状态层 —— commit `82c2bfe`（`files_store.at`）。验证：auto build
  绿；运行时分派正确（走查）。
- [x] T-04 FilesView + 导航接线 —— commit `82c2bfe`（`files_view.at`、
  `app.at`、`viewstate_router.ts`、i18n zh/en）。验证：auto build 绿 +
  浏览器走查（AC-01/02）。
- [x] T-05 查看器 —— commit `82c2bfe` + 修复批 `73822ba`。验证：走查
  AC-03..06、AC-09。
- [x] T-05/T-06 复审 r1 重开项已清（r2：F-01 三件规范增量落地 d2fde81 + F-02
  loadFailed 键补齐，D16 门绿）。原证据保留：8.3 走查、cargo lib 418 绿、auto build
  绿（修复版 auto，无需手工补丁）。
- [x] T-06 双轨对齐登记 + 回归（原勾选保留，重开项见上） —— 走查单结果见 8.3；全量回归见 9；对齐
  登记见 8.2。

**E2E 修复批**（commit `73822ba`，走查实证三问题）：
① 后端树节点从 wiki `TreeNode` 改为 gallery fs schema `FilesNode`
  （`id/label/children 恒在/kind/icon/is_leaf/badge`）——wiki schema 叶子缺
  `children`，FileTree `flatten_tree` 取 `.len()` 踩 undefined（TypeError 实证）；
② 文本正文加载改视图侧 `ports/files.web.at`（`.text()` + encodeURIComponent）
  ——api.at 生成绑定固定 `response.json()`，对文本响应必炸（loading 永挂实证）；
  store 不消费 use.web.fn，故 `SelectFile` 只置状态、`SetContent/SetLoadFailed`
  回填；移除不可用的 `files_raw` 绑定；
③ raw URL 显式携带 workspace（`storage musk_workspace`）——`<img>/<video>`
  不经 fetch 拦截器注入，缺省解析到 serve CWD 工作区致 404 实证。

**与计划的偏差**（等价实现，不改动契约）：
- T-03 原设计 forge_store 增 FilesState → 落为独立 `files_store.at`（specs_store
  分视图 store 惯例）；
- T-05 代码高亮不引 prismjs 组件（`PrismCodeBlock.vue` 已退役），代码内容围栏
  包装后经 autodown 管线渲染（聊天同源，plan §5.4 即此意）；
- 计划 §6 的 vitest helper 单测取消：helper 按仓库纪律内联进 store（不导出），
  分派逻辑由 Rust 忽略规则测试 + 走查覆盖；
- 后端新增 `FilesNode` 契约类型（原计划复用 TreeNode——schema 不匹配，见①）。

### 8.2 双轨对齐登记（T-06）

| 面 | vue 轨 | VM 轨 | 依据 |
|:---|:---|:---|:---|
| 导航入口 / 视图切换 | ✅ | ✅（骨架） | app.at 单源；ShowFiles 双轨驱动 LoadTree |
| FileTree 树渲染/交互 | ✅ E2E | ❌ 空树 | flatten_tree/toggle_id 经 use.web（vue 专属）；VM 无 import_aliases 机制——与 mention_helpers 等既有 use.web 面一致，非新退步；待 VM 侧函数别名能力后启用（DEBTS 候选） |
| 代码/markdown 查看器 | ✅ E2E | ❌ | MarkdownRender/loadFilesFileText 为 use.web；VM 待平台能力 |
| 图片预览 | ✅ E2E | ❌ | img 标签 web 侧 |
| 视频内嵌 | 结构就绪 | ❌ | html: 兜底为 web 逃生舱；播放未实测（无资产，ffmpeg 缺席）——登记 |
| 不能打开/过大空态 | ✅ E2E | 部分 | 静态文案臂 |

### 8.3 验收走查单结果（2026-09-14，worktree serve :8081，workspace=musk-demo）

| AC | 结果 | 证据 |
|:---|:---|:---|
| AC-01 | ✅ | 一级导航"文件"出现（i18n zh/en），视图互切正常 |
| AC-02 | ✅ | 树=磁盘实况（docs/notes/notes-app/plans/f1.txt/…），忽略项不可见；展开 13→21→34 行 |
| AC-03 | ✅ | tmp-analyze.js → javascript 高亮代码块 |
| AC-04 | ✅ | 018-sse.md → 4 标题 2 段落真渲染（streaming-document） |
| AC-05 | ✅ 图片 | 临时 1×1 png naturalWidth>0，URL 带 workspace；⚠ 视频播放未实测（无资产），MIME video/mp4 单测覆盖 |
| AC-06 | ✅ | 目录行/未知类型 → "不能打开该文件类型" |
| AC-07 | ✅ | `../`、绝对路径、`%2e%2e%2f` 编码绕过均 400；忽略目录不可见；symlink 由 canonicalize 单测覆盖 |
| AC-08 | ⚠ 部分 | vue 轨全过；VM 轨骨架可用、树/查看器受 use.web 限制——8.2 登记（既有平台缺口，非本计划回归） |
| AC-09 | ✅ | 21MB 文件 413 实测 |

## 9. 复审记录

- 2026-09-14 draft（plan_revision 1，author zcode）：背景调查完成（FileTree 位于
  widgets-gallery / CodeEditor 不存在 / spec_tree 先例可泛化，证据见 §4）；任务覆盖
  全部 AC；handoff `stage: new, PLAN-068 r1, outcome: pass, next: work`。待澄清①..④
  均已取缺省方向（只读查看器 / Range 不做 / 忽略清单常量 / 扩展名表），不阻塞开工，
  用户可在 work 前改判。
- 2026-09-14 work（plan_revision 1，author zcode）：**execution_done**。
  `stage: work | plan_id: PLAN-068 | plan_revision: 1 | outcome: pass（含显式偏差，
  见 blockers） | code_commit: 73822ba（T-01 d4cac2d、T-02..05 82c2bfe、修复批
  73822ba） | task_ids: T-01..T-06 全部 [x]（证据见 8.1） | evidence: cargo
  files_browser 6/6 + 全量 cargo test 绿（exit 0）+ vitest 36 + auto build 全绿 +
  浏览器走查 8.3（AC-01..07、09 ✅） | blockers: AC-08 VM 轨树/查看器受 use.web
  平台限制（8.2 登记，属既有平台缺口非本计划回归）——建议复审裁定的两个选项：
  (a) 接受登记为 DEBTS 候选并按 8.2 修订 AC-08 文字（升 plan_revision）；
  (b) 要求本计划内实现 VM 侧函数别名（需另立调研，超出当前授权范围）。
  另：视频播放未实测（无资产/无 ffmpeg），video/mp4 MIME 单测覆盖，登记于 8.3。
  | next: review`。worktree 保留待复审/合回；:8081 验收 serve（worktree exe）仍在
  运行供复验，merge 阶段随组清理。

- 2026-09-14 review r1（plan_revision 1，author zcode——**实现同会话复审，独立受限，
  判定由工件重构**）：baseline = plan-068-dev@73822ba（base f685880；deps auto-ai@9d2102c、
  auto-lang@2c92f3755 detached）。复跑：cargo lib 418 绿；auto build（修复版 PATH
  auto v551）绿——不再需要手工 8px 补丁；vitest **2 失败**（D16 i18n 门）。
  `stage: review | plan_id: PLAN-068 | plan_revision: 1 | outcome: needs_fix |
  reviewed_commit: 73822ba | base_commit: f685880 | acceptance: AC-01..07,09 复验过
  （8.3 走查+脚本化），AC-08 沿登记 | findings: F-01 规范增量三项未落地（SD-01
  files-browser.md / SD-02 组件组行 / SD-03 架构 API 面）——T-05"规范增量落地"勾选
  失实；F-02 files.loadFailed i18n 键缺失（73822ba 增 usage 未增键，D16 门红，
  修复批后未复跑 vitest——验证缺口） | evidence: vitest i18n.spec 输出（键缺失清单）、
  docs/specs/modules/ 目录清单（无 files-browser.md） | next: work（重开 T-05；
  修复后二轮复审）`。

- 2026-09-14 review r2（plan_revision 1，author zcode，同会话复审限制同 r1）：
  F-01/F-02 修复复验——**vitest 5 文件 36 测试全绿（D16 i18n 门过）**、auto build 绿、
  cargo files_browser 6/6。规范增量三件落地核验：docs/specs/modules/files-browser.md
  （描述即当前实现：FilesNode 契约/忽略清单/预算/confinement/MIME .html→text/plain/
  查看器矩阵/五条实证注记——无执行日记体）、03-front-component-groups.md 增 G-文件
  浏览组、01-architecture.md API 面增 /api/files/* 行。`stage: review |
  plan_id: PLAN-068 | plan_revision: 1 | outcome: pass | reviewed_commit: d2fde81 |
  base_commit: f685880 | dependency_revisions: auto-ai@9d2102c、auto-lang@2c92f3755
  （detached siblings） | spec_inputs: docs/specs/modules/files-browser.md（新增，
  本记录即冻结稿）+03/01 两处增行 | acceptance_results: AC-01..07,09 pass（8.3 走查
  +脚本化+本轮复验），AC-08 partial 沿 8.2 登记口径（VM 缺口既有，非回归——复审
  接受该登记） | findings: r1 F-01/F-02 已清 | next: merge`（随后 plan-068-dev 与
  main 合并冲突面：server.rs/lib.rs 各一处相邻路由与 mod 声明，PLAN-069 已在
  plan-069-dev 含同类行，合并顺序无关、冲突平凡）。

- 2026-09-14 merge 收据（PLAN-068:r1，author zcode）：`prepared`=ad15903 前置
  （worktree 内 docs/specs 三件 + index 挂载 delivery commit）；`landed`=main FF 至
  ad15903（含 d4cac2d/82c2bfe/73822ba/d2fde81 全部交付；main 冒烟：files_browser
  6/6、vitest 36、auto build 绿、gen/App.vue 含 FilesView×2）；`ledger_refreshed`=
  docs/specs/index.json 挂载 modules/files-browser.md（tracked view；runtime
  specs.json 为 plan-chapter 投影无模块先例，维持现状不造条目）；`archived`=
  docs/plans/archived/068-files-browser.md（status: archived，c2684c7）；
  `cleaned`=wt-guard 三 worktree 全 clean，musk-068 组（auto-musk worktree +
  plan-068-dev 分支 + auto-ai/auto-lang detached 兄弟 + 组目录）全数移除，
  `git worktree list` 0 项。

## 10. 待澄清事项

| # | 事项 | 当前缺省方向 | 状态 |
|:---|:---|:---|:---|
| ① | 用户预期复用"CodeEditor 组件"，但该组件全仓不存在（auto-edit 仓仅有空 docs）。
  v1 是否接受**只读高亮查看器**（prismjs），编辑器等 auto-edit 成熟或另立计划？ | 接受只读（用户原话已给"只读模式"弹性） | 待用户确认，不阻塞 |
| ② | "autodown 文件"口径：按 `.md/.markdown` 由 autodown 渲染理解；`.at`（Auto 源码）
  归代码查看器而非 markdown。是否正确？ | 是 | 待用户确认，不阻塞 |
| ③ | 忽略清单为服务端常量（.git/.autoos/.worktrees/node_modules/target/dist/tmp/
  vendor/.pnpm/.DS_Store），v1 不做 UI 开关。是否需要可配置？ | 常量即可 | 待用户确认，不阻塞 |
| ④ | 图片/视频 v1 用原生标签，不做自定义预览面板；视频不做 Range/进度拖动。后续如需
  预览面板（缩放/暗色审片/外链）另立增量。 | 原生标签 | 待用户确认，不阻塞 |
