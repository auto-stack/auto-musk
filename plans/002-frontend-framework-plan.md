# 002 — auto-musk 前端工程框架计划（优先前端，后端占位）

> **状态**：前端框架计划（基于 015-notes 唯一权威全栈样本）
> **背景**：后端卡在 auto-lang 的 `#[api]` VM server panic（见 `auto-lang-fix-312-server-panic.md`）。本计划**优先推进前端**，后端先用 015-notes 式代码占位，待 auto-lang 修复后前后端打通测试。
> **创建日期**：2026-06-16
> **权威样本**：`examples/ui/015-notes`（用户明确：016+ 示例未完善，仅认 015-notes 为全栈参考）

---

## 1. 目标与策略

**目标**：用 a2vue 搭建 auto-musk 的前端工程框架，移植 auto-forge 的前端骨架（App Shell + 多视图路由 + 导航），形成可见的产品雏形。后端先用占位代码，不阻塞前端。

**核心策略——分层落地**：a2vue 生成的 Vue 是**标准 `<script setup lang="ts">` + 原生 HTML + tailwind**（015-notes `App.vue` 实证），无魔法。因此采用：

| 层 | 用什么 | 适用 |
|---|---|---|
| **widget DSL（.at）** | a2vue 生成 | 布局、路由、表单、CRUD 类视图、导航 shell——a2vue 擅长的 |
| **手写 Vue 组件（.vue）** | 直接写 | SSE 流式渲染、富文本(Tiptap)、图表(mermaid)、全局状态(composable)——a2vue 无对应表达 |

依据：生成器按 source artifact 精确写文件（015-notes `.auto/ui-cache.json` 的 `artifacts` 表：每个 `.at` → 一个 `output_path`）。**手写的、不在 artifacts 表里的 `.vue` 不会被覆盖**，可安全混用。

## 2. 调研结论：a2vue 能力边界（对照 auto-forge 前端需求）

| auto-forge 前端需求 | a2vue 能力 | 落地策略 |
|---|---|---|
| 多视图路由（9-tab） | ✅ `routes { "/" -> use xxx }` + `outlet` | widget DSL |
| 导航 rail / 响应式布局 | ✅ 内置 `row/col/aside/header/nav-link/icon/drawer` | widget DSL |
| 登录表单 | ✅ input/button/on | widget DSL |
| API 调用（CRUD） | ✅ `use back.api: fn` → `await fn()` + 自动生成 api.ts | widget DSL |
| 组件复用 | ✅ `components/*.at` + 父调子 `Child(prop:val)` | widget DSL |
| computed | ✅ `computed { x => ... }` | widget DSL |
| **登录态（JWT + authFetch）** | ⚠️ 生成的 api.ts 不带 auth 头 | 手写 api wrapper |
| **SSE 流式 token 渲染** | ❌ widget 无流式概念（`.Init`→onMounted 仅一次） | **手写 Vue 组件** |
| Tiptap 富文本 | ❌ | 手写 Vue 组件 |
| mermaid / marked / markdown | ❌ | 手写 Vue 组件 |
| Composables 模块级单例 store | ❌（只有 widget 内 model） | 手写 composable |
| vue-i18n | ❌ | 生成后接 |

**结论**：a2vue 覆盖 auto-forge 前端的"骨架与表单类视图"（约 60-70%），复杂交互（流式/富文本/图表/全局状态）走手写 Vue 混用。

## 3. 工程目录结构

参照 015-notes 的 workspace 结构（前端 `scene: "ui"` + `api` 指向后端）：

```
auto-musk/
├── pac.at                      # scene: "ui", render: "vue", api: "./src/back/api.at"
├── src/
│   ├── front/                  # 前端 Auto 源（widget DSL）
│   │   ├── app.at              # 根 widget App：routes + 导航 shell + outlet
│   │   ├── pages/              # 【路由页面】必须放此子目录（输出到 @/pages/）
│   │   │   ├── login.at        # 登录页
│   │   │   ├── chats.at        # 聊天页（外壳用 DSL，流式区嵌手写组件）
│   │   │   ├── explorer.at     # 项目浏览器页
│   │   │   ├── specs.at        # Spec Ledger 页（占位）
│   │   │   └── relay.at        # Relay 页（占位）
│   │   ├── nav_rail.at         # 可复用子组件（直接下 → 输出到 @/components/）
│   │   └── ...                 # 其他可复用 widget（直接下，非 pages/ 子目录）
│   └── back/                   # 后端 Auto 源（占位，待 auto-lang 修复后激活）
│       ├── api.at              # #[api] 端点（仿 015-notes，占位）
│       └── db.at               # 业务层（占位）
├── gen/front/vue/              # a2vue 生成产物（完整 Vue3+TS+Vite 工程）
│   └── src/
│       ├── App.vue, main.ts     # ← 生成器管辖（被 .at 映射，重新生成会覆盖）
│       ├── components/          # widget 生成的组件 + 【手写 .vue 可混放】
│       ├── views/               # widget 生成的页面 + 【手写页面可混放】
│       ├── composables/         # 【手写】全局状态（useAuth/useForge/useSpecs...）
│       ├── lib/
│       │   ├── api.ts           # ← 生成器管辖（从 #[api] 生成）
│       │   ├── sse.ts           # 【手写】EventSource 封装
│       │   └── auth.ts          # 【手写】api wrapper（在 api.ts 上加 auth 头）
│       └── components/custom/   # 【手写】StreamingRenderer.vue 等（SSE/Tiptap/mermaid）
└── plans/                      # 计划文档
```

**关键约定**（基于 Step 1 验证结论）：
- `src/front/*.at` → 生成到 `gen/front/vue/src/`（widget 管辖，会被覆盖）。
- **手写文件直接放进生成的 `gen/front/vue/src/`**——与生成文件自然共存，`auto gen` 不清理、不覆盖（Step 1 已验证）。
- **唯一约束**：手写文件避开"生成器管辖路径"（即 ui-cache.json artifacts 表里的 output_path，如 App.vue、components/EditorPanel.vue、lib/api.ts）。放在 `composables/`、`lib/`（非 api.ts）、`components/custom/` 等专属子目录最安全。
- 手写文件应纳入 git 跟踪（gen/ 目录的 gitignore 策略需区分：生成文件可忽略，手写文件必须跟踪——建议用 git 跟踪整个 gen/ 或仅跟踪手写子目录，Step 2 时定）。
- 后端 `src/back/` 先占位，`pac.at` 的 `api` 指向它但不依赖它运行（前端可独立 `auto gen` 生成 + dev，Step 2 验证）。

## 4. 前端 MVP 范围（本计划交付）

对应总计划阶段 4 的前端部分（RBAC 最小版 + App Shell + 只读视图），但**纯前端先行**：

1. **App Shell**（`app.at`）：routes + 左侧导航 rail（仿 auto-forge 9-tab）+ outlet + 响应式布局。参照 component-gallery 的 app.at 布局模式（DSL 写法）。
2. **登录视图**（`login.at`）：表单 + 提交（前端校验，真实鉴权待后端）。
3. **一个只读样板视图**（`explorer.at`）：项目/文件列表（mock 数据，展示视图模式）。
4. **占位视图**：chats/specs/relay 等先放占位页（"Coming soon"），后续阶段填充。
5. **手写 api wrapper + auth composable**（`frontend-custom/`）：为后端打通预留接口形状。

**本 MVP 不含**：SSE 流式、Tiptap、真实后端对接（这些在后续阶段 + 后端修复后做）。

## 5. 风险点验证结论（Step 1 已实测，2026-06-17）

### ✅ 风险 1（已解除）：手写 Vue 组件可与生成工程混用

**实验**（在 015-notes 生成工程内放置 3 个探针，再 `auto gen`）：
- `src/components/HandwrittenProbe.vue`（与生成组件同目录）
- `src/composables/useAuth.ts`（全新目录）
- `src/lib/sse.ts`（与生成的 api.ts 同目录）

**结果**：
1. **`auto gen` 不清理手写文件**——3 个探针在不同位置全部存活，内容完整。
2. **生成器精确覆盖管辖 artifact**——`App.vue` 被重新生成（mtime 更新），但源未变的 `api.ts` 因增量缓存跳过。
3. **手写组件可被正常 import + 构建**——把 3 个探针接入 App.vue 后：`vue-tsc --noEmit` 零类型错误；`vite build` 成功（34 模块转换，产出 dist）。

**结论**：分层落地策略成立。手写 Vue 组件（SSE/Tiptap/composable 等）可直接放入生成工程的 `src/`，与 widget 生成的组件混用，`auto gen` 不破坏它们。

**对目录结构的影响**（修正 §3）：`frontend-custom/` 这个外置目录**不需要**——手写文件直接放进生成的 `gen/front/vue/src/` 即可。约定手写文件放在生成器不管辖的子目录（如 `src/composables/`、`src/lib/` 的非 api.ts 文件、`src/components/` 下的手写 .vue），与生成文件自然共存。**唯一注意**：被 widget 源（.at）映射到的 output_path（如 App.vue、components/EditorPanel.vue）会被覆盖，手写文件避开这些路径即可（ui-cache.json 的 artifacts 表可查哪些路径是生成器管辖的）。

### ✅ 风险 2（已解除）：纯前端 dev 完全独立于后端
后端 `#[api]` server 当前 panic 跑不起来，但前端完全不受影响。Step 2 实测（2026-06-17）：
- `auto gen` 正常生成前端工程，**不因后端缺失报错**
- `pnpm install` + `vue-tsc --noEmit`（零错误）+ `vite build`（39 模块成功）全部通过
- 后端 `#[api]` 只在前端 `use back.api: fn` 时生成 `lib/api.ts` 客户端；当前骨架前端未引用后端，故无 api.ts，前端自洽
- **结论**：前端开发可完全独立推进，后端修复后接入即可。

### ✅ 风险 3（已解除）：路由 + outlet 生成质量良好
Step 2 实测：app.at 的 `routes { "/" -> use xxx }` + view 里的 `outlet` 生成质量优秀：
- 生成标准 `src/router/index.ts`（vue-router + 懒加载 `() => import('@/pages/xxx.vue')` + hash history）
- `outlet` → `<router-view/>`，App.vue 成正确路由容器
- `main.ts` 自动 `app.use(router)`
- build 通过，产出路由懒加载 chunk（explorer.js/login.js）

### ⚠️ 关键目录约定（Step 2 踩坑得出，务必遵守）
a2vue 对前端源码目录有**特殊约定**，放错位置会导致路由路径与组件路径错配（build 失败）：

| 源位置（src/front/） | 输出位置（gen/.../src/） | 用途 | 扫描方式 |
|---|---|---|---|
| `app.at`（直接下） | `App.vue` | 根 widget（含 routes/outlet） | 固定入口 |
| `*.at`（直接下，非 app/pac/types/mod） | `components/<Widget>.vue` | 可复用子组件 | 单层扫描（**不递归**） |
| `pages/*.at`（**必须 pages/ 子目录**） | `pages/<name>.vue` | 路由页面 | **递归扫描** |

- **路由页面必须放 `src/front/pages/`**——`routes {}` 的 import 路径硬编码为 `@/pages/<name>.vue`（`vue.rs:751`），只有 pages/ 目录的 .at 会输出到 `@/pages/`。
- **普通子组件放 `src/front/` 直接下**——输出到 `@/components/`。
- **不要用 `src/front/views/` 这类自定义子目录**——不会被扫描（单层 read_dir，`vue.rs:834`）。
- 证据：曾误把页面放 `src/front/` 直接下 → 生成到 `components/LoginPage.vue`，但路由指向 `@/pages/login.vue`，build 失败（TS2307 + ENOENT）。改为 `pages/` 后路径一致，build 通过。

## 6. 实施步骤

### Step 1 — 验证手写组件接入 ✅ 完成（2026-06-17）
- 在 015-notes 生成工程放 3 个手写探针 → `auto gen` → 全部存活 ✅
- 接入 App.vue → `vue-tsc` 零错误 + `vite build` 成功 ✅
- **产出**：结论见 §5 风险 1；目录策略简化为"手写文件直接放生成工程 src/，避开 artifact 路径"。

### Step 2 — 搭建 auto-musk workspace 骨架 ✅ 完成（2026-06-17）
- 建立 `pac.at`（scene:ui, render:vue, api:rust）+ `src/front/app.at`（routes + outlet）+ `src/front/pages/{login,explorer}.at` + `src/back/{api,db}.at`（占位）
- `auto gen` 成功，生成完整 Vue 工程（含 router/index.ts、pages/、App.vue with router-view）
- `pnpm install` + `vue-tsc`（零错误）+ `vite build`（39 模块成功）全部通过
- **踩坑修正**：页面必须放 `src/front/pages/`（非直接下、非 views/），见 §5 目录约定
- **产出**：auto-musk 可运行的前端骨架；风险 2/3 解除；目录约定明确

### Step 2 — 搭建 auto-musk workspace 骨架
- 建 `pac.at`（scene:ui, render:vue, api 指向 src/back/api.at）
- 建 `src/front/app.at`（最小 App：一个 route + outlet）
- 建 `src/back/api.at` + `db.at`（015-notes 式占位，一个 hello 端点）
- 验证 `auto gen` 生成前端工程、`auto run`（或 `npm run dev`）能起 Vite

### Step 3 — App Shell + 导航 + 路由 ✅ 完成（2026-06-17）
- `app.at`：5 路由（/ login，/explorer /chats /specs /relay 主界面页）
- `src/front/nav_rail.at`：可复用 NavRail 子组件（输出到 components/），各主界面页 `use nav_rail: NavRail` 引入
- `pages/`：explorer（含 NavRail + mock 列表）、chats/specs/relay（占位）、login（全屏，无 rail）
- **生成验证**：`Routes: 5` + NavRail.vue 子组件正确输出到 components/
- **构建验证**：`vue-tsc` 零错误 + `vite build` 51 模块成功
- **运行验证**：`npm run dev` → Vite dev server HTTP 200（localhost:3001，3000 被占自动切）
- **DSL 风格定调**：沿用 015-notes 已验证风格（`row/col/link/text/button` + `(prop: val) {}` 语法）。`link (to: "/x")` → `<router-link>`（含 active-class），生成质量好。未依赖 component-gallery 的未验证内置组件（nav-link/icon/theme-toggle）。
- **App Shell 形态**：login 独立全屏；主界面页各自带 NavRail（a2vue 的 routes 是扁平的，无嵌套布局继承，故 rail 放可复用子组件由各页引入，而非全局包裹）。
- **已知小坑**：vite dev 默认监听 IPv6 `[::1]`，curl 127.0.0.1 连不上，需用 `localhost`。

### Step 4 — 登录视图 + explorer 样板视图（部分已在 Step 2/3 完成）
- ✅ `login.at`：表单 + 提交（前端校验，真实鉴权待后端）
- ✅ `explorer.at`：NavRail + mock 列表骨架
- ⬜ 手写 `composables/useAuth.ts` + `lib/auth.ts`（api wrapper 加 auth 头）——待后端修复后做（当前无后端可对接，手写 wrapper 无意义）
- 手写 `frontend-custom/lib/api.ts` wrapper + `composables/useAuth.ts`

### Step 5 — 接入占位后端（待 auto-lang 修复后）
- 后端 `#[api]` server 修复后，激活 `src/back/`
- 前端 explorer 从 mock 切到真实 `#[api]` 调用
- 验证前后端打通（vite proxy /api → 后端）

## 7. 与总计划的对应

- 本计划 = 总计划阶段 0（骨架）+ 阶段 4（RBAC+Shell）的前端部分，**前置并独立于后端修复**。
- 后端修复后，本计划的 Step 5 衔接总计划阶段 3（MVP 全栈）。
- SSE 流式（总计划阶段 5）依赖本计划的 `StreamingRenderer.vue` 手写组件框架，但实际实现延后。

## 8. 不做的事（YAGNI）

- 不在本计划做 SSE/Tiptap/mermaid 的实际实现（仅预留手写组件位置）
- 不做 vue-i18n、主题切换（auto-forge 有，但非 MVP 前端骨架必需）
- 不做 Composables 全套 store（仅 useAuth 一个，其余后续）
- 不碰后端逻辑（占位即可）
