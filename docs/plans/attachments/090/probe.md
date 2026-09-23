# PLAN-090 T-01 勘察决策记录（Bounded Probe）

勘察时间：2026-09-23
勘察环境：`D:/autostack/auto-lang` (target/release/auto.exe 存在), `D:/autostack/.wt/musk-090/auto-musk`

---

## 1. blueprints 活体与降级实测 (Item ①)

- **主检出状态勘察**：
  经实机勘察，`D:/autostack/auto-lang/blueprints` 目录完好存在，包含 7 种 kind、18 个 blueprint 包。
- **CLI 命令探测**：
  - `auto bp list`：输出 7 类别、18 个包（32 行文本，格式为 `# <kind>\n  <kind>/<name>`），执行耗时 <100ms。
  - `auto bp list --format at`：输出包含 AST 数据表（含 gotchas、references 源码、spec_text 等，全量约 30KB+）。
  - `auto bp show <kind>/<name>`：透传 frontmatter（palette/extension_points/variants/dataSource）、Intent、guidance、variants 列表和 gotchas。
  - `auto bp check <file> [--spec <kind>/<name>]`：
    - 正确实现：输出 `✓ loading state present`、`✓ error state present`、`✓ used widgets within palette`，3 passed, 0 failed，退出码 0。
    - 不合规实现（缺少 loading/error 等）：输出 `✗ loading state present ... 1 passed, 2 failed`，退出码 1。
- **降级实测**：
  当设置 `AUTO_BLUEPRINTS_ROOT="D:/autostack/nonexistent"` 时，`auto bp list` 退出码为 0，标准输出为：
  `(no blueprints found under blueprints/)`。
  因此，bp 通道探测到此标志或子进程错误时，应进入 loud-degraded 状态，返回降级说明与恢复指引。

---

## 2. 校验面与输出形态取舍 (Item ② / §5.6)

- **`auto ui inspect <file>` 探测**：
  - 速度快（~100ms），输出组件结构、SFC preview 以及 `[S001 INFO]` 等检验建议。
  - 退出码始终为 0，目前为诊断与审查用途，非严格式机器门控。
- **`auto build -d <dir> --gen-only --strict` 探测**：
  - 会执行 Vue 项目脚手架生成，写盘 `gen/front/vue`，对工作区有写副作用，且执行时间较长（1-2s）。
- **取舍结论**：
  保持 §5.6 既定策略，不引入 `app_check` 工具；验收流水线保持精简高效的三道门：
  1. `ui_lint`（musk 原生轻量 advisory 快门，即时捕获已知坑并给处方）
  2. `bp_check`（auto-lang 静态行为契约硬门）
  3. `canvas_run`（实况启动与端点连通验收门）

---

## 3. L1 bind vs L2 copy 在 VM 轨与沙箱中的可行性 (Item ③)

- **L1 bind 机制**：
  `auto bp add <KEY> --bind` 生成 `use bps.<kind>.<name>.reference.<variant>: Widget`，依赖 `pac.at` 中声明：
  ```
  dep bps {
      path: "<path to blueprints>"
  }
  ```
  在独立 workspace app 中，外部相对路径因工作区位置各异，容易发生路径越界或脱节。
- **L2 copy 机制**：
  `auto bp add <KEY> --reference <variant>`（或直接拷贝参考实现 .at 文件）生成自包含代码，无外部 `bps` 依赖包诉求，零越界风险，在 Vue 和 VM 模式下即开即用，且更易于 agent 根据用户需求进行局部定制。
- **复用序 v2 判定**：
  L1>L2>L3 复用序细化为：
  - **L2 copy/adapt 优先**（推荐主力）：命中 blueprint 时拷贝参考实现并在此基础上修改，最健壮、最自包含。
  - **L1 bind 进阶**：用于严格组件化、已配置 bps 依赖的场景。
  - **L3 自由生成**：兜底无对应 blueprint 时的全新 UI 搭建。

---

## 4. examples README 与目录解析面 (Item ④)

- **README 表格结构**：
  `D:/autostack/auto-lang/examples/ui/README.md` 的 `## 示例总览` 节包含标准 Markdown 表格：
  `| 编号 | 名称 | 一句话 | 端口 | 桌面 | 状态 |`
  正则匹配 `^\|\s*(\d{3})\s*\|\s*([^\|]+?)\s*\|\s*([^\|]+?)\s*\|` 即可提取 34 个示例的编号、名称和说明。
- **文件结构**：
  `examples/ui/<dir>/` 下各示例均以 `pac.at` 为根，界面核心为 `src/front/app.at`。
- **工具设计**：
  - `app_examples_list`：解析 README 表格或目录列表，输出编号、名称、一句话简述。
  - `app_example_read`：接受 `{name, file?}`，默认返回 `pac.at` 与 `src/front/app.at` 内容，带 32KB 截断保护。
