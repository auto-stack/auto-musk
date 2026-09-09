# Auto Musk

[English](README.md) | [简体中文](README.zh.md)

Auto Musk 是一个使用 Auto 语言构建、以 **“计划 + 规范”双流程**驱动开发的 Coding Agent。Plan 承载一次变更的分析、实施和验证；Spec 沉淀项目知识，为后续开发提供依据。

作为 auto-forge 的继任者，Auto Musk 将交互式编程与分阶段 Agent 编排结合起来，通过 AutoStack 的配置体系选择角色、技能和背后的模型。这套计划驱动方法已在 [auto-lang](../auto-lang) 中用于 500 多个开发计划，积累了有人参与管理的实际工程经验；这并不意味着这些计划已经可以无人值守地完成。

## 核心特色

- **计划管理变更，规范管理长期知识。** 开发时将上下文集中到 Plan，完成后将经过复审的成果沉淀到项目知识体系。
- **四阶段职责明确。** `new → work → review → merge` 分别负责需求分析、执行实施、复审验证与合入沉淀。
- **以产物交接上下文。** Markdown + YAML frontmatter 的 Plan 保存目标、决策、任务、验收标准、执行进展和复审结果。
- **Agent 可配置、可替换。** 阶段可以选择不同角色及其技能、工具、模型或模型档位和预算；当前内置 Plan 流程的四阶段统一使用 `plan-dev`。
- **完成以验证为依据。** 复审检查实际代码与测试结果，并主动寻找遗漏、未经批准的延后和变通实现。
- **Auto 语言实现。** Auto 源码生成主用 Vue 前端和部分 Rust 后端，同时提供 AutoVM 后端与原生 UI 路线。
- **开发过程可见。** 对话、工具事件、计划进展、确认门、报告、规范和知识库都可以在应用中查看。

## 系统架构

```text
用户 / CLI / Web UI
        |
        v
auto-musk
  对话 + Plans + Spec Ledger + Wiki
  Relay：阶段编排、确认门、交接与报告
        |
        v
auto-ai-agent
  角色 + 技能 + 工具 + Agent 执行循环 + 通用工作流原语
        |
        v
auto-ai-client → auto-ai-daemon（aaid）→ 模型服务商

auto-os-config → 共享角色、技能、模型与应用配置
auto-lang      → Auto 源码编译、代码生成与 VM 运行时
```

| 层次 | 职责 |
|---|---|
| **auto-musk** | 开发流程、项目产物、本地工具、工作区数据、API 与界面 |
| **auto-ai** | Agent 执行与通用编排；daemon 统一负责模型服务商通信、并发与用量统计 |
| **auto-os-config** | AutoStack 应用、角色、技能和模型设置的统一配置界面与服务 |
| **auto-lang** | 语言、编译器、前后端代码生成支持与 VM 运行时 |

开发阶段与 Agent 数量是两个独立维度。Relay 支持每阶段指定角色；当前主用 `plan` flow 按阶段串行运行 `plan-dev`，在实施前由用户确认计划。旧的七角色 Spec 流水线仍保留在代码中供对照。

### 实现形态与目录

默认 HTTP 后端是 Rust/axum，由手写基础设施与 Auto 生成的 Rust 共同组成。主用 Web UI 从 Auto 源码生成 Vue。仓库还保留手写 Web 回退版本和 AutoVM 后端选项，目前尚未完全转为纯 VM 应用。

```text
auto-musk/
├── .agents/skills/               仓库开发使用的四个技能
│   ├── auto-plan-new/
│   ├── auto-plan-work/
│   ├── auto-plan-review/
│   └── auto-plan-merge/
├── skills/                      应用 Agent 技能库
├── src/front/                   Auto UI 源码与平台适配
├── src/back/api.at              前端 API 契约
├── backend/crates/musk/
│   ├── auto-src/                Auto 后端源码
│   └── src/                     Rust 基础设施与 auto_generated/
├── gen/front/vue/               生成的主用 Web 应用
├── web/                         手写 Web 回退与参考实现
├── docs/plans/                  活跃计划
│   └── archived/                已归档计划
├── docs/specs/                  按模块组织的项目知识
├── docs/designs/                架构与设计参考
└── pac.at                       Auto 构建配置
```

运行形态与前端托管逻辑见 [pac.at](pac.at)、[后端入口](backend/crates/musk/src/main.rs) 和 [HTTP 服务](backend/crates/musk/src/server.rs)。

### 计划、规范与运行历史

| 产物 | 用途 | 位置 |
|---|---|---|
| Plan | 一次变更的分析、设计、任务、验收标准与进展 | `docs/plans/NNN-slug.md` |
| 归档 Plan | 保留变更历史 | `docs/plans/archived/` |
| Spec Ledger | 六区派生索引、关系与历史视图 | 工作区 `.autoos/specs.json` |
| 模块规范 | 当前模块行为与设计知识的权威来源 | `docs/specs/` |
| 会话 | 对话与 Relay 活动历史 | 工作区 `.autoos/conversations/` |

`docs/specs/` 是当前项目知识的权威来源，ledger 提供派生索引、关系与历史视图。仓库技能先在 worktree 中应用经过复审的规范增量，合入权威文档，再刷新派生 ledger，验证完成后才归档。沉淀凭据用于核对并继续被中断的操作。

应用现有的 `merge_plan` 操作仍会复制 Plan 章节到 ledger 并立即归档，再由 document 阶段更新模块规范。新版仓库 merge 技能避开了这个组合操作；应用内置流程尚未接入新的沉淀契约。

Relay 的运行状态目前保存在内存中，活动记录写入会话；服务重启后**不会自动恢复正在执行的流程**。Relay run 正常结束，也不等于计划已经验收通过并完成沉淀。

## 快速开始

### 前置条件

- Rust 与 Cargo。
- 构建生成版 Web UI 所需的 Node.js 与 pnpm。
- 从 [auto-lang](../auto-lang) 安装的 `auto` 工具链。
- 与 Cargo 路径依赖匹配的 `auto-ai`、`auto-lang` 兄弟仓库。
- 执行模型任务所需的已配置 `aaid` 服务；可通过 [auto-os-config](../auto-os-config) 管理配置。

```text
<workspace>/
├── auto-musk/
├── auto-ai/
├── auto-lang/
└── auto-os-config/              可选的配置管理应用
```

模型服务商与模型配置位于 `~/.config/autoos/ai-daemon.at`，Auto Musk 运行配置位于 `~/.config/autoos/apps/musk/config.at`。详见 [auto-ai 架构](../auto-ai/ARCHITECTURE.md) 和 [auto-os-config 使用说明](../auto-os-config/README.md)。

### 构建与运行

除特别说明外，以下命令从 Auto Musk 仓库根目录执行。

在独立终端启动模型服务：

```sh
cargo run --manifest-path ../auto-ai/Cargo.toml -p auto-ai-daemon
```

构建并安装 CLI：

```sh
cargo install --path backend/crates/musk
```

生成并构建主用 Web 应用：

```sh
auto build --gen-only
cd gen/front/vue
pnpm install
pnpm build
```

回到仓库根目录，启动应用：

```sh
musk serve
```

打开 [http://127.0.0.1:8080](http://127.0.0.1:8080)。服务同时提供 API，默认托管 `gen/front/vue/dist`。前端开发时，在后端运行的同时进入 `gen/front/vue` 执行 `pnpm dev`；Windows 下也可通过 `start-musk-web.cmd` 启动该前端开发服务。

### CLI 与运行选项

执行 CLI 编程任务前，先进入目标项目目录；该目录决定工具的工作区。

```sh
musk run "阅读项目并总结架构"
musk run --mode coding "实现已经确认的变更"
musk chat
musk chat --mode review
musk modes
musk professions
musk serve --addr 127.0.0.1:8080 --workdir <project-directory>
```

| 选项 | 用途 |
|---|---|
| `--mode` | 选择 `superpowers`、`basic`、`coding`、`review` 等 Agent 模式 |
| `AAID_URL` | 覆盖配置中的 daemon 地址 |
| `MUSK_WEB_DIST` | 覆盖托管的前端目录，也可用于选择 `web/dist` 回退版本 |
| `MUSK_BACKEND=vm` | 选择 AutoVM HTTP 后端路线 |
| `auto run --render=vm` | 通过 Auto 工具链启动原生 VM UI 路线 |

VM 路线仍在持续开发中。前端源码约定见 [Auto UI 开发说明](src/front/README.at-conventions.md)。

## 开发流程使用方式

### 四个仓库技能

[.agents/skills](.agents/skills) 中的四个技能用于能够加载仓库技能的 Coding Agent 宿主。以下示例是**对话指令**，不是 `musk` 的 shell 子命令；具体斜杠命令的呈现方式取决于宿主。

| 阶段 | 对话示例 | 产出 |
|---|---|---|
| [new](.agents/skills/auto-plan-new/SKILL.md) | `/auto-plan:new 为 X 功能创建计划，约束是 Y` | 自包含的计划草稿，交由用户确认 |
| [work](.agents/skills/auto-plan-work/SKILL.md) | `/auto-plan:work 42` 或“执行 Plan 42” | 实现与逐项验证证据；完成后进入 `execution_done` |
| [review](.agents/skills/auto-plan-review/SKILL.md) | `/auto-plan:review 42` 或“复审 Plan 42” | 重新验证的验收结果、问题清单与规范影响元数据 |
| [merge](.agents/skills/auto-plan-merge/SKILL.md) | `/auto-plan:merge 42` 或“沉淀 Plan 42” | 合入通过复审的代码、沉淀知识、归档计划并清理 worktree |

典型操作步骤：

1. 描述需求和约束，让 Agent 创建 Plan。
2. 阅读计划范围、设计和验收标准，确认或修订计划。
3. 在计划专用 worktree 中执行，进展记录在 Plan 中。
4. 复审真实实现；有问题则返回 work 修复，再次 review。
5. 合入复审通过的变更，更新项目知识并归档 Plan。

正常完成路径：

```text
drafting → executing → execution_done → reviewed → archived
                 ↑           |
                 └── 修复 ───┘
```

复审失败应返回修改。应用的归档操作也允许搁置未完成计划，因此不能仅凭归档状态判断交付成功。

Plan 正文包括目标、架构、背景分析、详细设计、测试设计、验收标准、执行任务、复审记录与待澄清事项。Frontmatter 保存 ID、状态、语义版本 `plan_revision`、进度，以及 `supersedes_spec_components`、`new_spec_components`、`touched_goals` 等规范影响字段。稳定的任务与验收 ID 将实施和证据关联起来；计划中的拟议规范增量由 review 对照代码与权威文档核准。

仓库技能允许按需查阅相关代码和规范，并在既有授权范围内做有边界的调整。各阶段按情况使用 `pass`、`needs_fix`、`needs_replan`、`blocked` 交接结果，与 Plan 状态分开。复审失败统一返回 `executing`，通过则记录对应的计划版本和代码提交。已获完整流程授权时，可以继续下一个技能，并限制自动修复次数；这些记录尚不提供后端重启恢复能力。

### 应用内置 Relay 流程

在 Auto Musk 应用对话中，可要求复杂变更按计划流程处理。Agent 可以调用 `spawn_relay(flow_id="plan", ...)`：

```text
plan → [用户确认计划] → execute → review → document
```

这条流程使用相同的 Plan 交接载体与四阶段开发方法，当前每阶段都使用 `plan-dev`。确认门、工具活动和报告显示在会话中。

当前复审失败后，需要通过后续对话继续修复和复审；内置流程尚不会自动循环执行到通过。计划未达到 `reviewed` 时，document 阶段会跳过沉淀。四个仓库技能与应用阶段模板是不同入口，不应假定调用 Relay 就会自动落实全部仓库 worktree 规则。

### 开发 Auto Musk 本身

遵守 [AGENTS.md](AGENTS.md)：

- 仓库改动在专用 worktree 中完成。有 Plan 时，路径为 `D:/autostack/.wt/musk-<NNN>/auto-musk`，分支为 `plan-<NNN>-dev`。
- 共享计划草稿与进度标记保留在主检出；实现改动进入 worktree。
- 依赖项目的 worktree 放在同组目录中并排管理，禁止在 worktree 内创建 junction 或 symlink。
- 实施阶段运行针对性检查，复审或阶段合入时执行要求的全量测试门禁。
- 删除 worktree 前运行 `wt-guard.sh`，结果必须为 clean；按仓库约定合回已提交的变更并完成清理。

[流程设计文档](docs/designs/008-auto-plan.md) 介绍了 Plan/Spec 分工的由来。流程持续演进，具体操作应结合当前技能与实现。

## 展望

下一步是保留“计划 + 规范”的核心分工，让现有流程能够可靠地自动运行。以下为拟议优化方向，尚未全部实现：

1. **运行时对齐技能契约。** 将运行时提示、状态处理及 Auto 与生成实现对齐到新版技能，将确定性校验下沉为共享工具。
2. **应用支持有边界的计划调整。** 将技能中的版本与授权规则接入运行时交接，保留相关证据和用户决策。
3. **构建持久化 Plan Runner。** 保存执行检查点、运行所有者、待确认事项与配置快照，支持重启恢复、暂停、取消和预算限制。
4. **闭合实施与复审循环。** 使用 `pass`、`needs_fix`、`needs_replan`、`blocked` 等结构化结果，将证据绑定到计划与代码版本，停止没有进展的重复尝试。
5. **自动执行可恢复的沉淀。** 在后端落实技能定义的权威规范优先顺序与操作凭据，补齐发布、归档和清理的并发控制与恢复能力。
6. **按能力选择 Agent。** 使用独立评审上下文，通过评估选择角色与模型；在任务依赖和写入范围允许时增加并行执行。
7. **先衡量，再扩大自动化规模。** 用有代表性的历史计划评估人工介入、需求漏检、恢复成功率、耗时与成本，再增加无人值守并发。
