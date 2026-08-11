# `/auto-plan` 设计文档

> **版本**：v1.0
> **状态**：定稿
> **最后更新**：2026-08-11


## 一、概述

### 1.1 背景

在长期使用 Superpowers 技能框架进行 AI 辅助开发的过程中，我们积累了约 400 个计划文件（Plan），成功完成了一个完整的编译器项目。这种 **Plan-driven（计划驱动）** 模式的核心优势在于：

- **上下文局部性**：所有信息（需求分析、架构决策、详细设计、执行步骤、验收标准）集中在一个文件中，Agent 执行时无需频繁查找分散的文档。
- **低错误率**：执行 Agent 只读一个 Plan 文件，减少了 Attention 机制的信息丢失。

然而，这种模式也存在明显的短板：Plan 文件归档后虽然可以作为工程“知识”来源，但由于它是基于单次需求的独立文件，**并未形成有组织的知识库**，事后查找和引用不便。

另一方面，**Spec-driven（规格驱动）** 模式（如 spec-kit、OpenSpec）通过维护结构化的规格文档实现了知识的长期沉淀，但在开发过程中需要频繁更新分散的多个文档，增加了上下文切换成本和信息丢失的风险。

### 1.2 核心思想：Checkpoint-Spec Sync（检查点-规格同步模式）

`/auto-plan` 的核心设计理念是 **“开发期用 Plan，归档期同步 Spec”**——即“延迟规格物化”（Deferred Spec Materialization）：

- **开发阶段**：以 Plan 文件为唯一事实源（Single Source of Truth），所有信息集中在一个文件中，Agent 执行时只依赖当前 Plan，保持 Superpowers 模式的高效与顺畅。
- **归档/合并阶段**：通过专门的 `merge` 技能，将已完成的 Plan 文件中的信息**自动拆解并同步**到结构化的 Spec 知识库中，实现知识的长期沉淀与检索。

这种模式不是 Plan-driven 与 Spec-driven 的折中，而是**进化**——它同时拥有了两者的优点：

| 维度 | Plan-driven | Spec-driven | **`/auto-plan`** |
|:---|:---|:---|:---|
| 执行效率 | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ |
| 知识沉淀 | ⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| 上下文切换成本 | 低 | 高 | **低** |
| Token 消耗 | 低 | 高 | **低** |
| 引用便捷性 | 中（依赖文件名） | 高（结构化检索） | **高（3位数字ID）** |

### 1.3 设计目标

1. **保持开发流程的顺畅**：Plan 文件作为唯一执行上下文，Agent 不需要在多个文档间跳转。
2. **实现知识的系统化沉淀**：归档时自动将 Plan 信息同步到结构化的 Spec 知识库。
3. **降低 Token 消耗**：开发阶段只加载当前 Plan，避免加载全量 Spec。
4. **极简引用**：通过 3 位数字 ID（如 `Plan 42`）实现便捷的人机交互。
5. **支持渐进式采纳**：可兼容已有项目的 Plan 文件体系，逐步引入 Spec 知识库。


## 二、设计原则

### 2.1 借鉴 Superpowers

Superpowers 是一套面向 AI 编程代理的“技能型开发方法学”，其核心理念是“**先思考、再计划、后编码、再审查**”。它不是让 AI 更聪明，而是让 AI 更守规矩——通过一组可组合的 Skills 和强制流程门禁，把“随口一句需求”变成可验证、可回滚、可 Review 的开发节奏。

`/auto-plan` 继承了 Superpowers 的以下核心原则：

- **Plan 是执行手册**：Plan 文件包含精确的文件路径、完整操作和验证命令。
- **先设计后编码**：未经充分设计和复审，不进入编码阶段。
- **技能驱动**：通过可组合的 Skills 定义工作流程。
- **状态门禁**：通过 `drafting → executing → review_done → merged` 状态流转确保质量。

### 2.2 借鉴 spec-kit / OpenSpec

spec-kit 和 OpenSpec 是规格驱动开发（SDD）框架，强调在编码前先定义规范。其核心价值在于：

- **先达成一致再构建**：人与 AI 先就“做什么”达成共识。
- **保持组织性**：每个变更都有自己的提案，包含 proposal、specs、design、tasks。
- **流动迭代**：文档可以随时更新，没有僵化的阶段门槛。

`/auto-plan` 借鉴了其**知识组织方式**（Spec 的结构化存储），但将“规范先行”调整为“**Plan 先行，归档时沉淀为 Spec**”，以兼顾开发效率与知识管理。


## 三、目录结构设计

### 3.1 整体结构

```
<project-root>/
├── docs/
│   ├── plans/                              # Plan 文件目录（开发期唯一事实源）
│   │   ├── 001-project-init.md             # 3位序号 + 英文短词
│   │   ├── 002-feature-auth.md
│   │   ├── 042-new-parser.md               # 示例：Plan 42
│   │   ├── 043-bugfix-memory-leak.md
│   │   └── archived/                       # 已合并的 Plan 文件
│   │       ├── 001-project-init.md
│   │       └── ...
│   │
│   └── specs/                              # Spec 知识库（归档后的结构化知识）
│       ├── index.json                      # Spec 索引（供 new-plan 快速检索）
│       ├── 00-overview.md                  # 项目概览、总体目标
│       ├── 01-architecture.md              # 全局架构设计
│       ├── goals/                          # 需求与目标
│       │   ├── README.md
│       │   └── goal-xxx.md
│       ├── modules/                        # 按模块/领域组织（树状结构）
│       │   ├── parser/
│       │   │   ├── spec.md                 # 模块规格说明
│       │   │   ├── design.md               # 模块详细设计
│       │   │   └── tests.md                # 模块测试策略
│       │   ├── codegen/
│       │   └── optimizer/
│       └── reviews/                        # 复审报告
│           └── review-042.md
│
├── .auto-plan/
│   ├── skills/                             # `/auto-plan` 技能定义
│   │   ├── new.md
│   │   ├── work.md
│   │   ├── review.md
│   │   └── merge.md
│   └── templates/                          # 文档模板
│       ├── plan-template.md
│       └── spec-template.md
```

### 3.2 设计说明

**Plan 与 Spec 分开放置**：
- `docs/plans/` —— 按数字序号平铺，每个文件是独立的、自包含的执行手册。
- `docs/specs/` —— 按模块/领域树状分层，每个文件是结构化的知识单元。

**为什么用 3 位数字序号**：
- **引用极简**：用户输入 “执行 Plan 42” 即可定位 `042-*.md`。
- **防漏号机制**：`/auto-plan:new` 通过读取目录最大序号 + 1 自动分配，杜绝冲突。
- **容量充足**：3 位数字支持 000-999，共 1000 个 Plan，对绝大多数项目绰绰有余。

**Spec 按模块分层存放**：采用树状结构组织 Spec，符合开发人员对代码结构的认知，便于按模块查找和阅读。

**Spec 索引文件**（`index.json`）：维护所有模块的轻量级索引，供 `new-plan` 技能快速了解项目全貌，避免加载全量 Spec。


## 四、Plan 文件格式设计

### 4.1 文件命名

```
<3位数字序号>-<英文短词>.md
```

示例：
- `001-project-init.md`
- `042-new-parser.md`
- `099-fix-memory-leak.md`

**引用方式**：用户在对话中输入 “Plan 42” 或 “执行 Plan 42”，Agent 自动定位到 `042-*.md` 文件。

### 4.2 文件结构

Plan 文件采用 **Markdown + YAML Frontmatter** 格式。YAML 元数据区块用于技能间传递状态和归档指引，正文包含完整的开发信息。

```markdown
---
# ============ 元数据区块 (Auto-Plan 核心契约) ============

# 基础信息
plan_id: PLAN-042                            # 3位数字序号，与文件名前缀一致
status: drafting                             # drafting → executing → review_done → merged
feature_name: 支持新语法解析
author: [创建者/Agent]
created_at: 2026-08-11T10:00:00Z
updated_at: 2026-08-11T16:30:00Z

# ============ Spec 合并指引 (/auto-plan:merge 时使用) ============
# 声明此 Plan 将影响哪些 Spec 模块
# 格式: "模块路径: 变更类型 (新增/修改/废弃)"
supersedes_spec_components:
  - "specs/modules/parser/spec.md: 修改"
  - "specs/modules/parser/design.md: 修改"
new_spec_components:
  - "specs/modules/optimizer/spec.md: 新增"
touched_goals:
  - "goal-001: 支持新语法"
  - "goal-003: 性能提升20%"

# ============ 执行进度追踪 (供 /auto-plan:work 更新) ============
current_step: 3
total_steps: 8

---

# [PLAN-042] 支持新语法解析 - 实施计划

> **给执行 Agent 的指令：** 必须使用 `/auto-plan:work` 技能逐步执行此计划。
> **用户引用方式：** 在对话中输入 "执行 Plan 42" 即可定位本文件。

## 0. 变更摘要 (Executive Summary)
[用 2-3 句话向 Reviewer 说明：这个 Plan 做了什么变更，为什么要做]

## 1. 目标 (Goal)
[一句话描述此计划要构建什么]

## 2. 架构方案 (Architecture)
[2-3 句话描述技术方案和实现思路，必要时附上简图]

## 3. 技术栈 (Tech Stack)
[列出此功能涉及的关键技术/库/版本]

## 4. 需求分析与背景调查
[详细的需求分析、背景信息、以及与现有架构的关系]
> **设计约束：** 起草本 Plan 时，已通过 `/auto-plan:new` 拉取 `specs/index.json` 骨架，确保未偏离现有模块边界。

## 5. 详细设计 (Detailed Design)
[具体的接口设计、数据结构设计、算法描述等]

### 5.1 接口变更
[如新增/修改的 API 签名]

### 5.2 数据模型
[数据库表结构变更、数据模型定义]

## 6. 测试设计 (Test Design)
[测试策略、关键测试用例描述]
- **单元测试**: ...
- **集成测试**: ...

## 7. 验收标准 (Acceptance Criteria)
> 复审时 (`/auto-plan:review`) 逐项勾选。
- [ ] 标准 1: 新语法能被正确解析为 AST。
- [ ] 标准 2: 性能测试不低于 X req/s。
- [ ] 标准 3: 现有回归测试全部通过。

## 8. 执行步骤 (Execution Tasks)
> **粒度要求：** 每个任务应是 2-5 分钟可完成的原子操作。
> **格式要求：** 必须包含精确的文件路径、操作描述、验证命令。

### 任务 1: 搭建测试骨架
- [ ] **步骤 1.1:** 编写测试文件 `tests/test_new_parser.py`（包含新语法的正向/负向用例）。
- [ ] **步骤 1.2:** 运行测试 `pytest tests/test_new_parser.py`，预期 **失败 (Red)**。

### 任务 2: 实现核心解析逻辑
- [ ] **步骤 2.1:** 修改 `src/parser/grammar.py`，新增新语法产生式。
- [ ] **步骤 2.2:** 修改 `src/parser/ast_builder.py`，处理新节点类型。
- [ ] **步骤 2.3:** 再次运行测试，预期 **通过 (Green)**。

### 任务 3: 集成与文档
- [ ] **步骤 3.1:** 更新 CLI 入口，暴露新参数。
- [ ] **步骤 3.2:** 运行全量回归测试套件。
- [ ] **步骤 3.3:** 提交代码 (Commit)。

## 9. 复审记录 (Review Log)
> 由 `/auto-plan:review` 技能在复审时自动填写，人工确认。

- **复审人**: [Agent/开发者]
- **复审时间**: [时间]
- **复审结论**: 
  - [ ] 验收标准全部满足
  - [ ] 代码无安全隐患
  - [ ] Spec 元数据已补全
- **遗留问题**: [如有，写在这里]

## 10. 待澄清事项 (Open Questions)
[执行过程中遇到的模糊点，由 `/auto-plan:work` 技能在此追加]
- (无)
```

### 4.3 关键字段说明

| 字段 | 用途 | 维护者 |
|:---|:---|:---|
| `plan_id` | 唯一标识，与文件名前缀一致 | `new` 技能自动生成 |
| `status` | 状态管理，贯穿完整生命周期 | 各技能按阶段更新 |
| `supersedes_spec_components` | 声明修改了哪些现有 Spec | `review` 技能补全 |
| `new_spec_components` | 声明需要新建哪些 Spec | `review` 技能补全 |
| `touched_goals` | 关联到 Spec 中的 Goals | `review` 技能补全 |
| `current_step` / `total_steps` | 执行进度追踪 | `work` 技能实时更新 |


## 五、Spec 知识库格式设计

### 5.1 文件命名与组织

Spec 知识库按**模块（领域）** 分层存放：

```
specs/
├── index.json              # 轻量级索引
├── 00-overview.md          # 项目概览
├── 01-architecture.md      # 全局架构
├── goals/
│   ├── README.md           # 目标索引
│   └── goal-xxx.md         # 具体目标
├── modules/
│   ├── parser/
│   │   ├── spec.md         # 模块规格
│   │   ├── design.md       # 模块设计
│   │   └── tests.md        # 测试策略
│   └── ...
└── reviews/
    └── review-042.md       # 复审报告（关联 Plan 42）
```

### 5.2 Spec 文档模板

#### `spec.md`（模块规格说明）

```markdown
# [模块名称] 规格说明

## 概述
[模块的职责和功能描述]

## 需求 (Requirements)
### 需求: [需求名称]
系统 **必须 (SHALL)** [描述具体行为]

#### 场景: [场景名称]
- **给定 (GIVEN)** [初始状态]
- **当 (WHEN)** [执行操作]
- **那么 (THEN)** [预期结果]

## 接口 (Interfaces)
[对外暴露的 API/接口定义]

## 依赖 (Dependencies)
[依赖的其他模块或外部服务]
```

#### `design.md`（模块详细设计）

```markdown
# [模块名称] 详细设计

## 设计决策
[关键设计决策及其理由]

## 数据结构
[核心数据结构和数据模型]

## 算法/流程
[关键算法或业务流程描述]

## 变更历史
| 日期 | 变更说明 | 关联 Plan |
|:---|:---|:---|
| 2026-08-11 | 新增 XX 功能 | PLAN-042 |
```

#### `tests.md`（模块测试策略）

```markdown
# [模块名称] 测试策略

## 测试范围
[测试覆盖的范围]

## 测试用例
### 用例: [用例名称]
- **前置条件**: ...
- **操作步骤**: ...
- **预期结果**: ...

## 测试数据
[测试数据说明]
```

### 5.3 Spec 索引文件 (`index.json`)

```json
{
  "version": "1.0",
  "updated_at": "2026-08-11T16:30:00Z",
  "modules": [
    {
      "path": "modules/parser",
      "name": "Parser",
      "description": "语法解析器",
      "key_interfaces": ["parse()", "validate()"],
      "dependencies": ["tokenizer"]
    },
    {
      "path": "modules/codegen",
      "name": "CodeGen",
      "description": "代码生成器",
      "key_interfaces": ["generate()"],
      "dependencies": ["parser"]
    }
  ],
  "goals": [
    {
      "id": "goal-001",
      "name": "支持新语法",
      "status": "in_progress",
      "related_modules": ["parser"],
      "related_plans": ["PLAN-042"]
    }
  ]
}
```


## 六、技能（Skills）设计

### 6.1 技能总览

| 技能命令 | 职责 | 触发时机 |
|:---|:---|:---|
| **`/auto-plan:new`** | 创建新 Plan 文件（自动分配序号） | 收到新需求时 |
| **`/auto-plan:work`** | 执行 Plan 文件 | 开始开发时 |
| **`/auto-plan:review`** | 复审 Plan 执行结果，补全元数据 | 开发完成时 |
| **`/auto-plan:merge`** | **将 Plan 合并进 Spec 知识库** | 复审通过后 |

### 6.2 `/auto-plan:new` —— 创建新计划

**输入**：用户需求描述

**流程**：

1. **自动分配序号**：
   - 执行 `ls -1 docs/plans/ | grep -E '^[0-9]{3}-.*\.md$' | sort -n` 获取所有已存在的 Plan 文件前缀。
   - 取最大的数字编号 `N`，新编号为 `N+1`（如不足 3 位则补零，例如 `042`）。
   - **防漏号机制**：完全由 AI 动态计算，杜绝人工硬编码带来的冲突。
2. **读取 Spec 骨架**：加载 `specs/index.json`，获取项目模块概览和已有 Goals。
3. **需求澄清**（可选）：如需求模糊，进行头脑风暴式澄清。
4. **生成 Plan 文件**：
   - 创建 `docs/plans/<3位序号>-<英文短词>.md`
   - 填充 YAML Frontmatter（`plan_id: PLAN-042`, `status: drafting`, `feature_name` 等）
   - 在 `## 需求分析与背景调查` 中写入从 `index.json` 获取的模块摘要
   - **注意**：此时 `supersedes_spec_components` 和 `new_spec_components` 暂不填写（由 review 阶段补全）
5. **输出**：生成完整的 Plan 模板，等待用户确认或补充。

**约束**：
- **绝不**读取任何旧 Plan 文件内容，避免上下文污染。
- **只读** `specs/index.json`（骨架），不读全量 Spec。
- 生成 Plan 后，将 `status` 设为 `drafting`。

### 6.3 `/auto-plan:work` —— 执行计划

**输入**：Plan ID 或文件名（如 “Plan 42” 或 “042-new-parser.md”）。如未指定，自动选择最新 `drafting`/`executing` 状态的 Plan。

**流程**：

1. **定位目标 Plan**：
   - 如输入为数字（如 “42”），匹配 `docs/plans/042-*.md`。
   - 如输入为文件名，直接加载。
   - 如未输入，选择最新的 `drafting` 或 `executing` 状态文件。
2. **加载目标 Plan**：读取指定的 Plan 文件作为**唯一上下文**。
3. **更新状态**：将 `status` 从 `drafting` 更新为 `executing`。
4. **逐步执行**：
   - 按顺序执行 `## 执行步骤` 中的每个任务。
   - 每完成一步，在原文件的任务后追加 `[✅ 已完成] {简短说明}`。
   - 每完成一步，更新 `current_step` 计数。
5. **遇到模糊点**：**禁止**发散去查 Spec，直接在 Plan 末尾的 `## 待澄清事项` 章节追加问题，等待用户响应。
6. **全部完成**：将 `status` 更新为 `execution_done`。

**约束**：
- **只读**目标 Plan 文件，不读取任何其他文档。
- 每个任务完成后**必须**更新 Plan 文件中的状态标记。
- 遵循 TDD 原则：先写测试，再实现。

### 6.4 `/auto-plan:review` —— 复审计划

**输入**：Plan ID 或文件名

**流程**：

1. **加载目标 Plan**：读取 Plan 文件及当前代码状态。
2. **执行复审检查**：
   - 对照 `## 验收标准` 逐项检查是否全部满足。
   - 获取当前 Git Diff，检查代码变更是否符合 Plan 中的设计。
   - 运行测试套件，确认所有测试通过。
3. **补全元数据（关键步骤）**：复审通过后，AI 分析本次变更影响的 Spec 模块，**自动补全** YAML 中的：
   - `supersedes_spec_components`：哪些现有 Spec 被修改
   - `new_spec_components`：哪些新 Spec 需要创建
   - `touched_goals`：关联到哪些 Goals
4. **记录复审结果**：在 `## 复审记录` 中写入结论。
5. **更新状态**：复审通过后将 `status` 更新为 `review_done`；不通过则退回修改。

**约束**：
- 复审不通过时，**阻止**进入合并阶段。
- 补全的元数据必须**精确**，为后续 `merge` 提供准确指引。

### 6.5 `/auto-plan:merge` —— 合并到 Spec（杀手锏）

**输入**：Plan ID 或文件名（状态必须为 `review_done`）

**流程**：

1. **读取 Plan 元数据**：提取 `supersedes_spec_components` 和 `new_spec_components`。
2. **提取内容**：
   - 从 Plan 正文中抓取与 Spec 相关的信息片段（架构决策、接口定义、测试用例等）。
   - 按目标 Spec 文件分类组织。
3. **执行 Spec 同步**：
   - **修改已有 Spec**：读取目标 Spec 文件，生成增量更新（diff），合并到现有内容中。
   - **新建 Spec**：使用 Spec 模板创建新文件，填充从 Plan 提取的内容。
4. **更新索引**：更新 `specs/index.json`，反映模块变更及关联的 Plan ID。
5. **移动 Plan**：将 Plan 文件从 `docs/plans/` 移动到 `docs/plans/archived/`。
6. **更新 Plan 状态**：将 `status` 更新为 `merged`。

**冲突处理策略**：
- **Plan 优先**：由于 Plan 已经过复审，其内容被视为“已批准”的变更，采用强覆盖策略。
- **保护用户自定义内容**：使用标记（如 `<!-- AUTO-PLAN:START -->` 和 `<!-- AUTO-PLAN:END -->`）保护 Spec 中的人工编辑内容，避免被自动覆盖。
- **生成变更日志**：在 Spec 文件的 `## 变更历史` 中记录本次变更及关联的 Plan ID。


## 七、工作流程

### 7.1 完整生命周期

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          /auto-plan 完整工作流                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  用户提出需求                                                                │
│       │                                                                     │
│       ▼                                                                     │
│  ┌─────────────────┐                                                        │
│  │ /auto-plan:new  │ ◄── ① 自动计算序号（如 042）                          │
│  │  创建 Plan      │     ② 读取 specs/index.json（骨架）                   │
│  └─────────────────┘     ③ 生成 docs/plans/042-<name>.md                  │
│       │                     status: drafting                                │
│       ▼                                                                     │
│  ┌─────────────────┐                                                        │
│  │  人工补充/确认   │     开发者填充 Plan 细节                               │
│  └─────────────────┘                                                        │
│       │                                                                     │
│       ▼                                                                     │
│  ┌─────────────────┐                                                        │
│  │ /auto-plan:work │ ◄── ① 输入 "执行 Plan 42"                            │
│  │  执行 Plan      │     ② 只读 042-*.md 作为唯一上下文                    │
│  └─────────────────┘     ③ 逐步执行，更新 current_step                    │
│       │                     status: executing → execution_done              │
│       ▼                                                                     │
│  ┌─────────────────┐                                                        │
│  │ /auto-plan:review│ ◄── ① 对照验收标准检查                               │
│  │  复审           │     ② 补全 Spec 元数据（supersedes_spec_components）│
│  └─────────────────┘     status: review_done                               │
│       │                                                                     │
│       ▼                                                                     │
│  ┌─────────────────┐                                                        │
│  │ /auto-plan:merge│ ◄── ① 读取 YAML 元数据                               │
│  │  合并到 Spec    │     ② 更新 specs/（修改/新建）                       │
│  └─────────────────┘     ③ 更新 index.json                                │
│       │                    ④ 移动 Plan 到 archived/                       │
│       │                    ⑤ status: merged                                │
│       ▼                                                                     │
│  ┌─────────────────┐                                                        │
│  │  ✅ 完成        │     Plan 已执行，Spec 已同步，知识已沉淀               │
│  └─────────────────┘                                                        │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 7.2 状态流转

```
drafting ──(/auto-plan:work)──▶ executing ──(全部完成)──▶ execution_done
     │                              │
     │                              ▼
     └──(/auto-plan:review)──▶ review_done ──(/auto-plan:merge)──▶ merged
                                    │
                                    ▼
                              (不通过)──▶ 返回修改
```

### 7.3 关键流程说明

**开发阶段（Plan 为核心）**：
- 所有信息集中在 Plan 文件中。
- Agent 执行时只依赖当前 Plan，不加载 Spec。
- 保持 Superpowers 模式的高效与顺畅。
- 用户引用极简：“执行 Plan 42”、“复审 Plan 42”。

**合并阶段（同步到 Spec）**：
- 只有 `merge` 技能会触碰 Spec 知识库。
- 将 Plan 的“过程式叙事”转化为 Spec 的“结构化知识”。
- 不影响主开发流程，是“后台任务”。
- 完成合并后，Plan 进入 `merged` 状态并移至 `archived/` 目录。


## 八、防漏号机制详解

### 8.1 问题定义

在纯人工或半自动环境下，使用数字序号面临的核心风险是：**重复编号**（两个 Plan 都叫 `042`）或**跳号**（忘记 `042` 直接用了 `043`）。

### 8.2 解决方案

`/auto-plan:new` 技能内置**强制动态分配算法**：

```
1. 执行 Shell 命令：
   ls -1 docs/plans/ | grep -E '^[0-9]{3}-.*\.md$' | sed 's/-.*//' | sort -n
   
2. 解析输出，得到所有已用编号的列表，例如：["001", "002", "005", "006"]

3. 找到最大编号 "006"，新编号 = 6 + 1 = 7，格式化为 "007"

4. 如目录为空，从 "001" 开始

5. 生成文件名：docs/plans/007-<user_provided_name>.md
   plan_id 同步设为：PLAN-007
```

**此机制保证**：
- **绝不冲突**：基于文件系统事实，而非记忆。
- **绝不遗漏**：AI 执行命令是确定性的。
- **完全自动化**：用户无需手动指定编号。


## 九、参考与借鉴

### 9.1 Superpowers

| 借鉴点 | 在 `/auto-plan` 中的应用 |
|:---|:---|
| Plan 是执行手册 | Plan 文件包含精确任务、路径、验证命令 |
| 2-5 分钟任务粒度 | 执行步骤遵循此粒度 |
| 先设计后编码 | `new` → `work` 流程强制先有设计 |
| 技能驱动 | 四个核心技能覆盖完整生命周期 |
| 状态门禁 | `drafting` → `executing` → `review_done` → `merged` |

### 9.2 spec-kit / OpenSpec

| 借鉴点 | 在 `/auto-plan` 中的应用 |
|:---|:---|
| 规范驱动开发理念 | Spec 知识库作为长期真相源 |
| 规范与变更分离 | `specs/`（当前真实）与 `plans/`（变更提案）分离 |
| 结构化 Spec 文档 | `spec.md`、`design.md`、`tests.md` 分类 |
| 变更归档机制 | `merge` 技能将变更合并回 Spec |
| 标记保护用户内容 | 使用标记保护 Spec 中的人工编辑 |


## 十、未来演进

### 10.1 格式升级：AutoDown

当前采用 **Markdown + YAML Frontmatter** 格式，未来可升级到自研的 **AutoDown** 格式（Markdown 超集）。

**升级策略**：
- 标准 Markdown + YAML 文件天然是合法的 AutoDown 文件（向下兼容）。
- 只需替换底层解析引擎，上层 Skills 的 Prompt 无需改动。
- 迁移成本为零。

### 10.2 工具化（Harness）演进

当前采用 **Skills（技能）** 实现，未来可考虑将确定性逻辑改写成底层工具（Harness）。

**转正门槛**：
- 连续 50 个真实任务中，0 次因格式解析失败而返工。
- `merge` 的冲突合并逻辑形成稳定的工程公约。

**哪些适合工具化**：
- `merge` 中的“查找 Spec 文件位置并插入片段”——确定性逻辑，省 Token。
- Plan 文件格式校验脚本——纯确定性校验。

**哪些应保留在 Skills 中**：
- `new` 中的“需求分解与设计”——推理密集型，LLM 擅长。
- `review` 中的“判断验收标准是否满足”——需要理解和判断。


## 十一、总结

`/auto-plan` 是一种 **“检查点-规格同步”（Checkpoint-Spec Sync）** 开发模式，它：

1. **开发阶段以 Plan 为核心**：保持 Superpowers 的高效执行力和低 Token 消耗。
2. **合并阶段同步到 Spec**：实现 spec-kit/OpenSpec 的知识沉淀能力。
3. **通过四个原子技能**（`new`、`work`、`review`、`merge`）覆盖完整生命周期。
4. **Plan 文件是唯一事实源**，Spec 是 Plan 的“物化视图”。
5. **采用 3 位数字 ID**（如 `Plan 42`）实现极简引用，并内置防漏号机制。
6. **采用 Skills 优先策略**，待流程稳定后再考虑工具化。

**核心口诀**：**“执行时聚集成一点（Plan），合并时发散成网（Spec）”**。

---

*设计文档结束*