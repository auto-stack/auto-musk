# 004 — 工具集补全(B,优先级最高)

> **状态**:设计 + 实施计划。执行时用 `executing-plans` 技能逐 task 推进。

> **✅ 已完成（2026-06-26 核实）**：list_dir / list_symbols / glob / batch_replace 全部实现并注册。tools.rs 现 9 个工具。
> **澄清**：计划标题"~10 个工具"曾疑缺 `skill`——核实确认 `skill` 是独立 mode 机制（`mode.rs:22 skills: bool`，basic/coding mode 切换），**不在 tools.rs 工具位**，故 9 个工具即完整，无缺失。
> **仓库**:auto-musk(`backend/crates/musk/src/tools.rs`)。
> **优先级**:1️⃣ 最高 —— 工具是 agent 的"手",executing-plans 的验证阶段依赖它们。

## 目标

从 auto-forge 的 23 个工具里,**挑选对三步曲(brainstorm/write-plan/execute)最关键的**补全,不照搬全部。当前 musk 有 6 个工具(read/write/edit/search/run/skill),补到 ~10 个,覆盖"探索 + 编辑 + 验证"完整闭环。

## 现状(已实现,不重复)

| 工具 | 状态 | 用途 |
|---|---|---|
| `read_file` | ✅ | 读文件(offset/limit) |
| `write_file` | ✅ | 写文件(覆盖) |
| `edit_file` | ✅ | 唯一匹配替换 |
| `run_command` | ✅ | 执行 shell |
| `search` | ✅ | rg/grep 内容搜索 |
| `skill` | ✅ | 加载技能 |

## 要补的工具(按价值排序)

### 1. `list_dir` — 目录列表(探索)
- **为什么**:exploring 阶段(`brainstorming`)高频需求 —— 先看有哪些文件再看内容。当前只能 `run_command ls`,但那是原始 shell 输出,`list_dir` 给结构化的 `{name, is_dir, size}` 列表。
- **实现**:`std::fs::read_dir`,返回 JSON 数组。参数:`path`(默认当前目录)。
- **参考**:auto-forge 无直接对应,但 Forge 的 `bring_in`/exploration 逻辑里隐含。这是 musk 自己的小工具。

### 2. `list_symbols` — 文件符号大纲(探索)
- **为什么**:看一个 Rust/TS 文件的结构(pub fn/struct/enum/mod),不用读全文。探索大文件时关键。
- **实现**:轻量正则扫描 —— Rust 的 `pub fn/struct/enum/impl/mod`、TS 的 `function/export/const/interface/class`。**不引入 tree-sitter**(太重),正则够用。
- **参数**:`path`。
- **参考**:auto-forge `tools.rs:361` 的 `list_symbols`(它还解析 Vue defineProps/defineEmits,我们 MVP 先做 Rust/TS)。

### 3. `glob` — 文件名模式匹配(探索)
- **为什么**:找"所有 .rs 文件""所有 test_*" —— 探索项目结构。比 `search`(内容)和 `list_dir`(单层)更准。
- **实现**:用 `glob` crate(轻量)。参数:`pattern`(如 `**/*.rs`)、`path`(默认当前目录)。
- **参考**:无 Forge 对应,是 musk 补的。

### 4. `batch_replace` — 多处批量替换(编辑,executing-plans 高频)
- **为什么**:executing-plans 一个 task 常要改多处(改 import + 改定义 + 改调用),逐个 `edit_file` 轮次多。批量一次完成。
- **实现**:`{path, replacements: [{old, new}, ...]}`,逐个验证唯一性后替换。
- **参考**:auto-forge `tools.rs:1078`。

## 不补的(明确排除,后续)

- `read_specs`/`write_specs`/`update_spec` 等 —— 依赖 Spec Ledger HTTP 端点,等 specs 页打通后再说。
- `spawn_relay`/`spawn_task_plan`/`dispatch` —— 这些是 Forge 的子 agent 调度,依赖 Pipeline 引擎,不在 Skill 路线上。
- `query_wiki` 等 —— 无 wiki 系统。
- 文件读取缓存 —— 性能优化,非功能。

## 文件结构

```
backend/crates/musk/src/tools.rs   ← 加 ListDir / ListSymbols / Glob / BatchReplace
```
每个工具一个 `pub struct` + `impl Tool` + 2-3 个单元测试,与现有工具同风格。

## Tasks(执行清单)

### Task 1: `list_dir`
- Files: `backend/crates/musk/src/tools.rs`
- [x] 实现 `ListDir`(返回 `[{name, is_dir, size}]`)
- [x] 测试:列 tmp 目录、不存在报错
- [x] 注册进 `build_agent`(`lib.rs`)
- [x] commit

### Task 2: `list_symbols`
- [x] 实现(Rust + TS 正则扫描,返回符号行列表)
- [x] 测试:扫 tools.rs 自己(应含 `pub struct EditFile`)
- [x] 注册进 `build_agent`
- [x] commit

### Task 3: `glob`(加 `glob` crate 依赖)
- [x] `Cargo.toml` + `glob = "0.3"`
- [x] 实现 `Glob`(pattern + path)
- [x] 测试:找 `**/*.rs`
- [x] 注册进 `build_agent`
- [x] commit

### Task 4: `batch_replace`
- [x] 实现(逐个唯一性校验 + 替换)
- [x] 测试:批量替换、其中一个不唯一时报错回滚
- [x] 注册进 `build_agent`
- [x] commit

### Task 5: 验证 + 提交
- [x] `musk chat` 真实 LLM:让模型探索一个项目,确认它用了新工具
- [x] 全测试绿
- [x] push rust-impl

## 验收
- 工具数从 6 → 10,覆盖"探索(list_dir/list_symbols/glob)+ 编辑(edit_file/batch_replace)+ 验证(run_command/search)+ 技能(skill)"闭环。
- `musk chat` 里模型能用 `list_symbols` 看结构、用 `glob` 找文件、用 `batch_replace` 批量改。
