# 文件浏览（Files Browser）模块规范

> PLAN-068 落地（2026-09-14）。泛化 PLAN-025 的 docs/specs 树浏览器到整个
> workspace 根，提供只读文件浏览与分类型内容查看。

## 定位与能力

一级导航"文件"→ FilesView：左栏 FileTree（workspace 根目录树），右栏分类型
只读查看器（代码高亮 / markdown / 图片 / 视频 / 不能打开空态）。无任何写路径。

## API 契约（hw 逃生舱，files_browser.rs）

| 端点 | 语义 |
|---|---|
| GET /api/files/tree?workspace={id} | `{ tree: FilesNode[], truncated: bool }`——workspace 根目录树 |
| GET /api/files/raw/{*path}?workspace={id} | 原始字节 + MIME（文本/媒体统一端点） |

`FilesNode` = gallery fs 形态（PLAN-614 FileTree 契约）：`{ id: 相对路径,
label, children: **恒存在**（叶子空数组）, kind: "dir"|"file", icon: ""(组件按
kind/扩展名自动映射), is_leaf, badge: "" }`。**不得**复用 wiki `TreeNode`
（name/type、叶子缺 children——FileTree flatten_tree 对缺键取 `.len()` 即崩，
E2E 实证 TypeError）。

## 安全面（服务端强制，前端不作安全边界）

1. **confinement**：`validate_path_pub`（拒 `..`/绝对路径）+ canonicalize 前缀
   双保险（防 symlink 穿出；`\\?\` 前缀两侧一致）。
2. **忽略清单**（目录名，不区分大小写）：`node_modules / target / dist / tmp /
   vendor`；点目录（.git/.autoos/.worktrees 等）由 wiki 树的点文件规则排除。
3. **预算**：节点 ≤ 5000、深度 ≤ 12，超限置 `truncated: true`（不报错）。
4. **读上限**：20 MB，超限 413（前端"文件过大"空态）。
5. **MIME**：扩展 `wiki::guess_mime`——媒体（mp4/webm/webp/avif 等）就绪；
   **`.html/.htm` 强制 `text/plain`**（只读浏览器不得向 app origin 注入可执行
   HTML）；`.svg` 保持 `image/svg+xml`（`<img>` 内安全，直开风险与 wiki raw 同口径）。

## 查看器分型（扩展名表驱动，集中 files 分派处）

| kind | 扩展名 | 呈现 |
|---|---|---|
| code | at/rs/ts/tsx/vue/js/mjs/cjs/json/toml/yaml/yml/css/html/htm/sh/bat/cmd/ps1/py/sql/txt/lock/xml/csv/gitignore | 围栏包装经 autodown 管线高亮（PrismCodeBlock 退役轨口径） |
| markdown | md/markdown | autodown 渲染（聊天同源） |
| image | png/jpg/jpeg/gif/svg/webp/bmp/ico/avif | 原生 `<img>`（raw URL 直喂） |
| video | mp4/m4v/webm/mov | `<video controls>`（html: 兜底；播放未实测登记） |
| other（含无扩展名/目录行选中） | — | "不能打开"空态 |

## 前端接线

- 一级导航"文件"（app.at sidebar + vsSetView 白名单 `files` + i18n nav.files）。
- FileTree 组件族移植自 auto-os widgets-gallery（PLAN-614：filetree.at /
  tree_util.at / tree_icon.at）；纯函数经 `use.web` `@/ext` 导入（mention_helpers
  同款）。**VM 轨缺口**：无 import_aliases 机制 → VM 树不渲染（登记差异，非回归）。
- files_store：树/选中/分派 computed；状态名 `file_*`（避让 api 绑定 `files_tree()`
  撞名——实证）。helper 内联于 store（store 不消费 use.web.fn，nowSec 先例）。
- 文本正文加载走 `ports/files.web.at` `loadFilesFileText`（fetch `.text()`）——
  api.at 生成绑定固定 `response.json()`，对文本响应必炸（E2E 实证 loading 永挂）。
- raw URL **必须显式带 `?workspace=`**（`storage musk_workspace`）：`<img>/<video>`
  不经 fetch 拦截器注入，缺省解析到 serve CWD 工作区（E2E 实证 404）。

## 测试口径

Rust：忽略规则/排序/预算截断/深度截断/MIME/confinement/目录守卫；vitest：D16
i18n 门覆盖 files.* 键；E2E：登录 → 树展开 → 代码/md/图片/未知类型 → 413 →
confinement curl（`--path-as-is` 越界 400）。
