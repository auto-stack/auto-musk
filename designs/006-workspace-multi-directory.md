# 006 — Workspace 多工作目录设计

> **状态**：设计已确认（brainstorm 完成），待写实现计划。
> **日期**：2026-06-30
> **仓库**：auto-musk（`backend/crates/musk/` + `web/`）
> **当前分支**：`rust-impl`（HEAD `8f8c752`）
> **依赖**：无（独立功能，但触动 specs/chats/wiki/relay 所有 store 的存储路径）

---

## 0. 背景与问题

auto-musk 目前是**单项目 stub 模型**：

- 前端 `useProject.ts` 写死 `projectPath='.'`、`projectName='musk'`，所有方法 no-op。
- 后端无任何 workspace 概念：`AppState` 不记录工作目录；所有数据钉死在 `~/.config/autoos/{specs,chats,wiki,raw,relay}.json`。
- agent 文件操作根目录 = `musk serve` 启动那一刻的 cwd 快照（`tool_safety::PROJECT_ROOT`，`OnceLock`，进程内不可变）。

用户反馈：和 auto-forge 相比没有切换工作目录的能力。auto-forge 的切换 UX 也不好（关闭按钮藏在 Explorer 工具栏，关闭后强制走 welcome→confirm→describe 向导）。

本设计为 auto-musk 设计一套**更方便、支持多 workspace 并行**的工作目录选择/切换机制。

---

## 1. 核心决策（brainstorm 确认）

| 决策点 | 选择 | 理由 |
|---|---|---|
| workspace 含义 | agent 文件操作的根目录 + 该目录下的数据归属 | 贴合"agent 在哪个项目干活"的直觉 |
| 数据隔离 | **按 workspace 完全隔离**（A 看不到 B 的 specs/chats/wiki/runs） | workspace 是独立工作单元 |
| 多 workspace 形态 | 多窗口（类 VS Code，每浏览器标签一个 workspace） | 并行操作不同项目 |
| web 并行实现 | 单进程，按 `?workspace=<id>` 路由，多标签真并行 | web 架构约束下的最优解 |
| **数据存储位置** | **项目内 `{root}/.autoos/`** | 可移植/可 git 跟踪/符合 `.git` 惯例；目录即身份 |
| **workspace 身份** | 可读 id（默认=目录名，重名加序号或用户自定义）+ 全局 `id→path` 映射表 | URL 可读 + 唯一性 |
| 选择器位置 | 导航栏底部（footer 区，设置旁边），B 方案 | 多窗口形态下自然，类 VS Code "打开文件夹" |
| 切换面板 | 经典列表（recent 列表 + "打开文件夹"按钮），A 方案 | 简洁、熟悉 |
| 首次启动 | 默认用启动 cwd，不强制选 | 向后兼容、无摩擦 |

---

## 2. 数据模型

### 2.1 存储布局

```
~/.config/autoos/                      # 只留真正全局的东西
  ├─ users.json                        # 用户认证（全局共享）
  └─ workspaces.json                   # workspace 索引（id → path 映射 + recent）

{项目根}/.autoos/                       # 每个 workspace 独立（完全隔离 + 可移植）
  ├─ specs.json
  ├─ chats.json
  ├─ wiki/        (+ _manifest.json)
  ├─ raw/
  └─ relay/       (各 {run_id}/run.json)
```

### 2.2 全局索引 `workspaces.json`

```jsonc
{
  "workspaces": [
    {
      "id": "auto-musk",            // 可读 id（默认=目录名）
      "path": "D:/autostack/auto-musk",
      "name": "auto-musk",          // 显示名（默认=id，可改）
      "last_opened": 1782801900
    },
    {
      "id": "auto-forge",
      "path": "D:/autostack/auto-forge",
      "name": "auto-forge",
      "last_opened": 1782700000
    }
  ],
  "default_workspace_id": "auto-musk"   // 启动 cwd 对应的 id（兜底）
}
```

**索引只是指针**：真正的数据在项目内的 `.autoos/`。项目移动后 path 失效 → 切换时检测到、提示用户重新定位（数据随目录走，不丢）。

### 2.3 workspace_id 生成规则

打开一个新 workspace（路径 P）时：

1. 取 `P` 的目录名作为候选 id（`D:\autostack\auto-musk` → `auto-musk`）。
2. 若索引中已有该 id 且 path 不同 → 追加序号（`auto-musk-1`、`auto-musk-2`），或提示用户自定义。
3. 若 path 已在索引中 → 复用已有 id（同一目录重复打开，身份不变）。

---

## 3. 后端架构

### 3.1 WorkspaceRegistry

替代当前 `AppState` 里分散的 `specs/chats/wiki/relay` 单例 store。

```rust
pub struct WorkspaceRegistry {
    /// 全局索引（workspaces.json）：id → WorkspaceMeta
    index: RwLock<HashMap<String, WorkspaceMeta>>,
    /// 按规范化根路径缓存已加载的 stores（避免每次请求重新初始化）
    cache: RwLock<HashMap<PathBuf, Arc<WorkspaceStores>>>,
    index_path: PathBuf,   // ~/.config/autoos/workspaces.json
}

pub struct WorkspaceMeta {
    pub id: String,
    pub path: String,        // 项目根（规范化）
    pub name: String,
    pub last_opened: u64,
}

pub struct WorkspaceStores {
    pub root: PathBuf,                    // 项目根 = agent 文件操作根目录
    pub specs: SpecsStore,                // 初始化自 {root}/.autoos/specs.json
    pub chats: ChatStore,                 // {root}/.autoos/chats.json
    pub wiki: WikiStore,                  // {root}/.autoos/{wiki,raw}
    pub relay: RunStore,                  // {root}/.autoos/relay
}

impl WorkspaceRegistry {
    /// 启动时加载：读 workspaces.json；若空则用启动 cwd 建一个 default ws。
    pub fn load(index_path: PathBuf, default_root: PathBuf) -> Self;

    /// 按 workspace_id 取 stores（缓存命中/懒加载 {path}/.autoos/）。
    /// id 不存在时兜底回 default workspace。
    pub fn get(&self, ws_id: &str) -> Arc<WorkspaceStores>;

    /// 打开一个新 workspace（根路径），建/复用索引项 + 初始化 .autoos/。
    /// 返回 WorkspaceMeta（含生成的 id）。
    pub fn open(&self, root_path: &str) -> WorkspaceMeta;

    /// 列出 recent（按 last_opened 倒序）。
    pub fn list(&self) -> Vec<WorkspaceMeta>;

    /// 更新 last_opened（每次切换时调）。
    pub fn touch(&self, ws_id: &str);

    /// 持久化索引到 workspaces.json。
    fn save(&self);
}
```

### 3.2 AppState 改造

```rust
// 旧
pub struct AppState {
    client, auth, specs, chats, wiki, relay,   // 4 个单例 store
}

// 新
pub struct AppState {
    pub client: Arc<dyn Client>,           // 全局
    pub auth: Arc<AuthStore>,              // 全局
    pub registry: Arc<WorkspaceRegistry>,  // 替代 4 个 store
}
```

### 3.3 API 改造

每个业务请求通过查询参数携带 `workspace`：

```
GET  /api/specs?workspace=auto-musk
POST /api/chats/session?workspace=auto-musk   { mode }
GET  /api/forge/relay/runs?workspace=auto-musk
...
```

**所有业务 handler 统一模式**：

```rust
async fn specs_list(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,   // { workspace: Option<String> }
) -> impl IntoResponse {
    let ws = state.registry.get(&q.workspace.unwrap_or_default());
    match ws.specs.load() { ... }      // 操作该 ws 的 store
}
```

`WorkspaceQuery` 可作为通用提取器（一个 `Option<String> workspace` 字段），各 handler 解构 `ws_id` 后 `state.registry.get(&ws_id)`。

**workspace 管理端点**（新）：

| 端点 | 功能 |
|---|---|
| `GET /api/workspace/list` | 返回 recent 列表（`{workspaces: [...]}`） |
| `POST /api/workspace/open` | body `{path}` → 建/复用 ws、初始化 `.autoos/`、返回 `{id, name, path}` |
| `GET /api/workspace/status?workspace=<id>` | 返回 `{id, name, path, root_exists}` |
| `POST /api/workspace/browse?path=<dir>` | 返回子目录列表（前端切换面板的"打开文件夹"浏览用） |
| `PATCH /api/workspace/{id}` | 改 name（用户自定义 id 显示名） |

### 3.4 agent 工具根目录路由

`tool_safety` 从"全局 OnceLock"改成"按 workspace_id 路由"：

```rust
// 旧：进程级单一 root
static PROJECT_ROOT: OnceLock<PathBuf>;
pub fn resolve_within_project(path: &str) -> Result<PathBuf, String>;

// 新：接收 workspace_id，查 registry 得 root
pub fn resolve_within_workspace(
    path: &str,
    ws: &WorkspaceStores,   // 或 ws_id + registry
) -> Result<PathBuf, String>;
```

**绑定传递**：agent 工具执行时如何知道当前 workspace？

- `ChatSession` 加 `workspace_id: Option<String>` 字段。
- `RunMetadata`（relay）加 `workspace_id: Option<String>` 字段。
- chat_stream / relay driver 启动 agent 时，把 session/run 的 `workspace_id` 注入工具上下文（thread-local 或显式传参）。
- 各文件工具（`ReadFile`/`WriteFile`/`EditFile`）的 `execute` 改为读取该上下文得到 root。
- `RunCommand` 加 `.current_dir(&ws.root)`，命令在该 ws 根目录执行。

**测试兼容**：保留 `set_test_root`/`clear_test_root` 的 thread-local 覆盖机制（单测仍用 temp dir 模拟 root）。

### 3.5 serve 命令

```rust
Serve {
    #[arg(long, default_value = "127.0.0.1:8080")]
    addr: String,
    #[arg(long)]
    workdir: Option<String>,   // 新增：可选，指定默认 workspace 根目录
}
```

- 启动时 `WorkspaceRegistry::load()`：读 workspaces.json；若 `--workdir` 给定或索引为空，用启动 cwd（或 workdir）建/复用 default workspace。
- 不传 `--workdir` 时行为完全兼容现状（cwd = default workspace root）。

---

## 4. 前端架构

### 4.1 useProject 改造（stub → 实体）

```ts
// 状态
const _currentWorkspace = ref<WorkspaceMeta | null>(null)   // { id, name, path }
const _recentWorkspaces = ref<WorkspaceMeta[]>([])
const _isLoading = ref(false)

// 派生
const workspaceId = computed(() => _currentWorkspace.value?.id ?? null)
const projectName = computed(() => _currentWorkspace.value?.name ?? null)
const projectPath = computed(() => _currentWorkspace.value?.path ?? null)
const isOpen = computed(() => _currentWorkspace.value !== null)

// 方法（调真实后端）
async function fetchStatus()                 // GET /api/workspace/status?workspace=<url-param>
async function openWorkspace(path: string)   // POST /api/workspace/open → 设当前 ws + 跳 URL
async function loadRecent()                  // GET /api/workspace/list
async function browse(path: string)          // GET /api/workspace/browse?path=
```

**URL 同步**：当前 workspace id 存在 URL 查询参数 `?workspace=<id>`。App 启动时从 URL 读 ws_id → `fetchStatus`；切换时 `openWorkspace` → 更新 URL。

### 4.2 WorkspaceSelector 组件（新）

放在 `App.vue` 导航栏 footer 区（设置菜单旁边），B 方案：

```
┌─────────────────────────┐
│ ⚡ Auto Musk v1          │  brand
├─────────────────────────┤
│ 💬 聊天                  │
│ 🌀 流水线                │  tabs
│ 📜 规范                  │
│ 📚 知识库                │
├─────────────────────────┤
│ 📁 auto-musk ▾   ⚙️     │  footer：workspace 按钮 + 设置
└─────────────────────────┘
```

点击 workspace 按钮 → 弹出切换面板（A 方案：经典列表）：

```
┌─ 切换 Workspace ──── ✕ ─┐
│ 最近打开                  │
│ 📁 auto-musk   D:\...    │  ← 当前（高亮）
│ 📁 auto-forge  D:\...    │
│ 📁 auto-lang   D:\...    │
├──────────────────────────┤
│ 📂 打开其他文件夹...      │  ← 触发浏览/输入
└──────────────────────────┘
```

- 点列表项 → `openWorkspace(path)` → 切换。
- 点"打开其他文件夹" → 路径输入框（带 browse 联想）或浏览面板。
- 面板用 CSS 浮层（绝对定位向上展开），不占布局空间。

### 4.3 各 View 适配

`ChatsView`/`SpecsView`/`WikiView`/`RelayView` 已经从 `useProject` 取 `projectName`/`projectPath`：

- `useSpecs`/`useForge`/`useWiki`/`useRelay` 的所有 fetch 调用，在 URL 里附带 `?workspace=<currentId>`（通过 `authFetch` 包装或各 composable 内部拼接）。
- 切换 workspace 时，各 View 的数据自动重新加载（watch `workspaceId` → reload）。

### 4.4 前端路由

当前前端无 vue-router（用 `useViewState`）。workspace 用查询参数 `?workspace=<id>` 配合 `useViewState`，无需引入路由库：

- URL：`http://localhost:8888/?workspace=auto-musk`（或 `/chats?workspace=auto-musk`）
- `window.history.replaceState` 更新查询参数，不触发刷新。

---

## 5. UX 流程

### 5.1 首次启动

1. `musk serve`（可选 `--workdir`）→ 后端用启动 cwd 建 default workspace（id=目录名）。
2. 浏览器打开 `http://localhost:8888` → 前端无 `?workspace` 参数 → 用 default ws → 进入主界面（当前 cwd 的数据）。
3. 导航栏底部显示 `📁 <目录名> ▾`。

不强制选目录——已有 default，用户想换才换。

### 5.2 切换 workspace

1. 点导航栏底部的 workspace 按钮 → 弹出切换面板（recent 列表）。
2a. 点 recent 项 → `openWorkspace(path)` → 后端建/复用 ws + 前端跳 `?workspace=<id>` → 各 View 重新加载该 ws 数据。
2b. 点"打开其他文件夹" → 路径输入/浏览 → 选定 → 同上。

### 5.3 多标签并行

1. 用户在浏览器开新标签，URL 带 `?workspace=<另一个id>`。
2. 该标签连同一后端，但 `?workspace` 不同 → 操作另一套数据。
3. 两个标签的 agent 操作各自落在不同 root，真并行。

---

## 6. 迁移策略

现有数据在 `~/.config/autoos/{specs,chats,wiki,raw,relay}.*`（全局）。迁移：

- **方案**：首次启动新版本时，检测到旧全局数据 + workspaces.json 不存在 → 把旧数据**移动**到 default workspace 的 `.autoos/`（即启动 cwd 下的 `.autoos/`），并在 workspaces.json 注册 default ws。
- **只读 cwd 兜底**：若启动 cwd 不可写，default workspace 改指向用户的 home 目录下一个可写位置（如 `~/.config/autoos/default-workspace/`），旧数据移动到那里；启动后用户可通过选择器切换到真正想要的项目目录。
- **不强制**：迁移是 best-effort，失败则旧数据保留在原位，用户手动处理。

---

## 7. 范围与不做

### 本设计包含
- WorkspaceRegistry + 项目内 `.autoos/` 存储
- workspace_id（可读 + 映射表）+ `?workspace=` 路由
- 导航栏底部选择器（B）+ 经典列表面板（A）
- agent 工具按 ws_id 路由 root
- 旧数据迁移

### 明确不做（留后续）
- ❌ workspace 级权限/角色（用户认证仍全局）
- ❌ workspace 模板/初始化向导（auto-forge 的 describe 步，暂不做，打开即用）
- ❌ 跨 workspace 搜索/聚合视图
- ❌ workspace 导出/打包
- ❌ 原生文件对话框（rfd）——浏览器内用路径输入 + browse 联想

---

## 8. 风险与权衡

| 风险 | 缓解 |
|---|---|
| 只读目录无法写 `.autoos/` | 检测可写性；启动 cwd 不可写时 default ws 指向 home 下可写位置；用户可随时通过选择器切换到目标项目 |
| 项目移动后 path 失效 | 切换时检测 root_exists，提示重新定位；数据随目录走不丢 |
| 改动面大（所有 store + handler + 工具） | 分阶段实现（见实现计划）；先 store 路径参数化，再 registry，再前端 |
| agent 工具上下文传递复杂 | 用 thread-local 注入（复用现有 `set_test_root` 模式），chat/relay driver 入口设置 |
| 测试改造（大量单测依赖全局 store） | 保留 thread-local 覆盖；测试用 temp dir workspace |

---

## 9. 验收标准

1. `musk serve` 启动后，导航栏底部显示当前 workspace（目录名）。
2. 点 workspace 按钮 → 弹出 recent 列表 + "打开文件夹"。
3. 选另一个目录 → URL 变 `?workspace=<id>`，各 View 数据切换到该 workspace。
4. 两个浏览器标签带不同 `?workspace` → 各自独立、互不干扰（agent 操作不同 root、看到不同数据）。
5. workspace A 的 specs/chats/wiki/runs 在 workspace B 中不可见。
6. 旧全局数据被迁移到 default workspace 的 `.autoos/`。
7. agent 调 read_file/write_file/run_command，操作落在当前 workspace 的 root 内、不能越界。
