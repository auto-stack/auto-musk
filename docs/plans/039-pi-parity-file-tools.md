---
plan_id: PLAN-039
status: executing
feature_name: 文件工具对齐 pi——edit 吸收 batch_replace（CRLF/BOM/模糊匹配/多重编辑/自愈报错）、read 分页截断、共享截断模块与 String::truncate panic 修复
author: [zhaopuming]
created_at: 2026-08-23
updated_at: 2026-08-23

supersedes_spec_components: []
new_spec_components: []
touched_goals: []

current_step: 1
total_steps: 12
---

# [PLAN-039] 文件工具对齐 pi：edit / read / 截断体系

## 变更摘要

对照 pi-mono（本地克隆 `D:\github\pi`，main @ a1f955e9f）的工具层实现，musk 的
`edit_file`/`read_file`/`search` 存在四类工程缺口：

1. **`edit_file` 纯精确匹配**（`backend/crates/musk/src/tools.rs:238`）：无 CRLF/BOM
   处理、无 Unicode 模糊回退、单次单替换。musk 在 Windows 上开发，模型输出的
   `old_string` 几乎永远是 LF、不含 BOM、可能带智能引号——CRLF/带 BOM 文件必然
   匹配失败。
2. **`read_file` 全量返回**（`tools.rs:40`）：无截断无分页，大文件直接灌爆上下文。
3. **`search` 的 `result.truncate(8192)` 有 panic 风险**（`tools.rs:370`）：Rust
   `String::truncate` 在切割点非 UTF-8 字符边界时 panic；中文内容下 8192 字节边界
   落在多字节字符中间的概率极高，**这是会让整个 musk 进程崩溃的真 bug**。
4. **工具报错不自愈**：not found / 歧义 / 空串共用粗糙报错，模型缺少"下一步怎么改"
   的指引。

本计划：抽共享截断模块（修 panic）→ `read_file` 分页 → 重写 `edit_file` 吸收
`batch_replace`（单工具多编辑）。工具签名若 auto-ai PLAN-027（content/details
分离）先落地则直接采用 `ToolOutput`，否则基于现有 `String` 返回并留两个标注挂接点。

**不改变**：path confinement（`tool_safety::resolve_scoped` 注入式 root）、白名单
审批流、`.at` 双轨边界（新代码落在手写轨 `tools.rs`，与现状一致；`.at` 化不在
本计划范围）。

## pi 参考实现索引

pi 仓库路径前缀 `D:\github\pi\packages\coding-agent\src\core\tools\`：

| 关注点 | pi 位置 | 移植要点 |
|---|---|---|
| 模糊匹配规范化表：NFKC、去行尾空白、智能引号→ASCII、Unicode 破折号→`-`、特殊空格→空格 | `edit-diff.ts:34`（`normalizeForFuzzyMatch`，含完整 Unicode 码点表） | 码点表逐字照抄进 Rust `char::map`；Rust 侧 NFKC 用 `unicode-normalization` crate |
| 精确匹配优先、失败后规范化空间重试 | `edit-diff.ts:207`（`fuzzyFindText`） | 直接翻译 |
| 模糊匹配的回写保护：只把被修改触达的行从规范化空间写回，未触达行保留原始字节 | `edit-diff.ts:132`（`applyReplacementsPreservingUnchangedLines`） | **核心正确性机制**，防止规范化污染全文件；Rust 按行分组应用 |
| CRLF/BOM 往返：匹配前 `splitBom`+`normalizeToLF`，写回恢复 | `edit-diff.ts:11-25`（`detectLineEnding` 等）+ `edit.ts:363-370`（execute 中使用） | `splitBom` 参照 `edit.ts:8` 引用的 `splitBom` 实现 |
| 多重编辑 `edits[]`：全部对原始文件匹配（非增量）、重叠检测、倒序单趟应用 | `edit-diff.ts:300`（`applyEditsToNormalizedContent`，重叠检测在 :341-350） | 吸收 `batch_replace`；比 musk 现在的顺序 `replacen` 更安全（见 §任务 5） |
| 五类自愈报错：not found / 出现 N 次（提示补上下文）/ 空 oldText / 无变化（提示特殊字符）/ 重叠 | `edit-diff.ts:253-289`（错误构造函数） | 文案照抄语义 |
| 模型怪癖垫片：edits 发成 JSON 字符串或单对象时归一化（pi 点名 Opus 4.6、GLM-5.1） | `edit.ts:116`（`prepareEditArguments`） | 在 musk 工具 execute 入口做同等归一 |
| 工具用法守则注入系统提示（"oldText 最小但唯一"、"邻近合并"） | `edit.ts:56-64`（`editToolSystemPromptContribution`） | musk 无 promptSnippet 通道——并入工具 description（见 §任务 8） |
| read 双限制截断（2000 行/50KB 先到为准、永不切半行） | `truncate.ts:78`（`truncateHead`） | Rust 版共享模块 |
| read 截断后的可执行续读指令 `[Showing lines 1-50 of 100. Use offset=51 to continue.]` | `read.ts:302-321` | 模型自愈的关键设计 |
| 单行超限的逃生通道（给出精确 bash sed 命令） | `read.ts:297-301` | musk 对应 `run_command` 提示 |
| 截断元数据结构 `TruncationResult`（总量/显示量/截断原因） | `truncate.ts:15` | 若 PLAN-027 落地，放 `ToolOutput.details` |
| 无头工具工厂（无 UI 依赖版本，结构更贴近 Rust 移植） | `D:\github\pi\packages\agent\src\harness\tools\edit.ts`、`read.ts`、`edit-diff.ts` | 移植时优先对照这版（去掉 TUI 渲染） |

## 方案

### 1. 共享截断模块 `tool_truncate.rs`（新文件）

```rust
pub struct TruncationResult { pub content: String, pub truncated: bool,
    pub truncated_by: TruncatedBy /* Lines | Bytes */, pub total_lines: usize, pub output_lines: usize }
pub fn truncate_head(&str, max_lines: usize, max_bytes: usize) -> TruncationResult; // read 用
pub fn truncate_tail(&str, max_lines: usize, max_bytes: usize) -> TruncationResult; // run/search 用
pub fn truncate_line(&str, max_chars: usize) -> String; // grep 行截断，500 chars
```

字节计数用 `str::len()`（UTF-8 字节数）；**所有切割点必须落在字符边界**
（`floor_char_boundary` 语义，std 未稳定则手写：`while !s.is_char_boundary(i) { i -= 1 }`）。
`search` 改用本模块，panic 随之消除（任务 1 兼修 bug）。

### 2. `read_file` 分页

参数加 `offset`（1 起始行号）/`limit`；默认截断 2000 行 / 50KB；截断/用户 limit
提前停时附 `Use offset=N to continue.` 尾注；offset 越界报错带总行数；单行超限
给 `run_command` sed 逃生提示。返回细节（截断元数据）按 PLAN-027 落地情况放
details 或尾注。

### 3. `edit_file` 重写（吸收 `batch_replace`）

- 参数改 `edits: [{old_string, new_string}]`，入口垫片兼容 JSON 字符串/单对象形态；
- 流程：read → `split_bom` → `detect_line_ending` → `normalize_to_lf` → 逐 edit
  `fuzzy_find`（精确优先）→ 歧义计数（规范化空间）→ 重叠检测 → 行级应用
  （未触达行保留原始字节）→ 恢复 BOM/CRLF → 写回；
- 删除 `batch_replace` 工具（其原子性语义被 `edits[]` + 前置校验全覆盖）；
- per-path 写互斥：`DashMap<PathBuf, Arc<Mutex<()>>>`（当前 ReAct 循环串行执行
  工具，此为防御性加固，成本低）。

### 4. 保留 musk 自有优势

path confinement 与注入式 root 原样保留在所有新代码路径中；`list_symbols`
（pi 没有）不动。

## 任务分解（12 步）

1. `tool_truncate.rs`：三函数 + 字符边界安全 + 单测（多字节中文、恰好整行、首行超限）。[✅ 已完成] 16/16 单测通过（head/tail/line 三函数，多字节边界/整行临界/首行超限全覆盖）；commit `feat(plan-039): T1`
2. `search` 改用共享模块，删除裸 `truncate`（**修 panic，可单独提前合入**）。
3. `read_file` offset/limit + 截断 + 续读尾注 + 越界报错；单测。
4. `tool_truncate` 接入 `run_command` 输出上限（临时措施，完整重写在 PLAN-040）。
5. `edit_diff.rs`（新文件）：规范化表 + `fuzzy_find` + 行级保留应用 + 重叠检测；
   单测用例对照 pi 行为（CRLF、BOM、智能引号、NBSP、行尾空白、重复、重叠、
   模糊命中但原文混排 CRLF）。
6. `edit_file` 重写接 `edit_diff.rs` + 入口垫片 + 五类报错；单测。
7. 删除 `batch_replace`；grep `skills/`、`docs/` 中对其引用并更新（TDD/executing-plans 技能若点名该工具则改写为 `edit_file` 多编辑用法）。
8. 工具 description 重写：并入用法守则（最小唯一 oldText/邻近合并/多编辑单调用）。
9. per-path 写互斥加固 + 单测（并发两写同文件）。
10. 若 PLAN-027 已落地：返回 `ToolOutput`，diff/truncation 放 details；否则在
    返回字符串中保留简短确认 + 标注 `// PLAN-027 挂接点`。
11. `tool_test.rs` 扩充：对拍用例表（与任务 5 同源 fixtures）。
12. 回归：`cargo test` + 手工冒烟（CRLF 文件编辑、大文件读取分页）。

## 验收标准

- pi 行为对拍表全绿：上述 8 类边界用例与 pi `edit-diff.ts` 语义一致（模糊命中时
  未触达行字节不变，用二进制对比断言）。
- `read_file` 读 10MB 文件返回 ≤50KB 且尾注可指导续读。
- 中文内容 search 截断不再 panic（回归测试覆盖多字节边界）。
- `batch_replace` 移除后全部测试与技能引用更新完毕。

## 风险

- `unicode-normalization` 为新依赖（NFKC）：纯 Rust 无 lifecycle 脚本，加入前过
  依赖审查惯例。
- 删 `batch_replace` 是行为变更：旧会话 replay 不受影响（工具运行时注册），但需
  确认 relay/plan 工具无交叉引用（任务 7 的 grep 覆盖）。
- 模糊匹配扩大"模型以为会改但实际匹配到别处"的表面积：pi 的缓解是歧义（>1 次
  即失败）+ 行级保留，我们照搬，不额外放宽。
