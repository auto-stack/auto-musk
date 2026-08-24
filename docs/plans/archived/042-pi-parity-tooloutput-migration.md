---
plan_id: PLAN-042
status: archived
feature_name: ToolOutput 迁移——修复主线编译断、填真 details（edit diff / read 截断 / run_command 全量输出路径）、事件链透传与前端渲染、PLAN-040 账面清理
author: [zhaopuming]
created_at: 2026-08-24
updated_at: 2026-08-24

supersedes_spec_components: []
new_spec_components:
  - "edit_diff: 替换组行区间(replaced_groups) + generate_edit_diff(行号 diff/unified patch/first_changed_line,语义对齐 pi)"
  - "工具 details 载荷约定: edit {diff,patch,first_changed_line} / read {truncation} / run_command {truncation,full_output_path}"
  - "事件链 details: RunEvent.TurnToolResult.details + ToolRecord.details(serde default 兼容旧回放) + SSE 透传"
  - "前端工具卡 details 区: Diff/Truncated 徽标/Full output(双轨)"
touched_goals:
  - "工具结果双通道呈现（content 给模型 + details 给 UI：diff/截断/全量输出）"
  - "ToolOutput 迁移与主线绿色基线恢复"

current_step: 10
total_steps: 10
---

# [PLAN-042] ToolOutput 迁移与绿色基线恢复

## 变更摘要

**紧急**：auto-ai PLAN-027 已把 `Tool::execute` 签名改为
`Result<ToolOutput, ToolError>`（`auto-ai/crates/auto-ai-agent/rust-ref/src/tool.rs:72`），
而 musk 的全部工具实现仍返回 `Result<String, _>`。实测 `cargo check -p musk`
失败（33 个错误，`E0053: expected ToolOutput, found String`）——**musk 主线当前
不可编译**。这是 027 计划中明示的挂起任务（其任务 5"musk 侧迁移"），因两仓合入
时序（musk 039/040 先合、auto-ai 027 后合）而暴露。

迁移范围（`impl Tool for` 实测清点）：

| 轨道 | 文件 | 处数 |
|---|---|---|
| 手写轨（必修） | tools.rs | 9 |
| | plan_tools.rs | 6 |
| | orch_tools.rs | 5 |
| | spec_tools.rs | 5 |
| | report_tools.rs | 1 |
| auto_generated 镜像轨 | tools.rs / spec_tools.rs / orch_tools.rs | 8 / 5 / 3 |

本计划分两层：①机械迁移恢复编译（`ToolOutput::text()` 包裹）；②在 PLAN-039
预留的三个挂接点（`tools.rs:100` read 截断、`tools.rs:591` edit diff、
`tool_truncate.rs:26` TruncationResult 文档）填**真 details**，打通
StreamEvent::Tool → relay → SSE → 前端的 details 消费链，让 027 的
content/details 分离真正产出 UI 价值（edit diff 渲染、截断徽标、审批 diff）。
顺带完成 PLAN-040 复审遗留的三件账面清理。

**优先级：P0（阻塞一切 musk 开发）**。

## pi 参考实现索引

pi 仓库本地克隆 `D:\github\pi`（main @ a1f955e9f），路径前缀
`packages/coding-agent/src/core/tools/`：

| 关注点 | pi 位置 | 移植要点 |
|---|---|---|
| details 形状三范例 | `edit.ts:83-90`（`EditToolDetails { diff, patch, firstChangedLine }`）、`bash.ts:53-56`（`{ truncation, fullOutputPath }`）、`read.ts:34-36`（`{ truncation }`） | musk 侧对应字段直接对齐这三组 |
| diff 生成：带行号展示 diff + 统一 patch，上下文 4 行 | `edit-diff.ts:376`（`generateDiffString`，返回 diff + firstChangedLine）、`:365`（`generateUnifiedPatch`） | musk 不引 diff 库：`edit_diff.rs` 已知替换区间，直接构造 hunk（±4 行上下文），见 §方案 2 |
| details 只进事件流、不进 LLM | `packages/agent/src/agent-loop.ts` 的 convertToLlm 边界；auto-ai 侧已有断言（`auto-ai/crates/auto-ai-agent/tests/mvp_harness.rs:643-674` 请求体零泄漏） | musk 侧无需重复实现，靠 auto-ai 保证；musk 只做消费 |
| 工具事件携带 details | `packages/agent/src/types.ts` 的 `tool_execution_end`（result 整体含 details） | musk 对应：relay 桥接把 `StreamEvent::Tool` 的 details 字段带进 `RunEvent::ToolResult` |
| 前端消费 details（按工具名分发渲染） | `edit.ts:414`（renderResult 读 `details.diff`） | 前端 useForge/ChatsView 按 tool_name 解析 details |
| `firstChangedLine` 语义（新文件行号，编辑器跳转用） | `edit.ts:89` | Web 前端可用于"跳到改动行"链接 |

## 方案

### 1. 机械迁移（恢复编译）

- 返回 `Ok(s)` → `Ok(ToolOutput::text(s))`；返回点为表达式的直接
  `ToolOutput::text(...)` 包裹。42 处 impl 一次切完，不留兼容层。
- auto_generated 镜像轨的 16 处：先确认其是否参与编译（039 复审称镜像模块
  休眠，但 `impl Tool for` 计数非零——以 `cargo check` 错误清单为准：报错则
  迁移，不报错则在本计划中仅登记、随 PLAN-041（web 轨退役）一并处置）。

### 2. edit_file 真 details（新增 diff 生成）

`edit_diff.rs` 已精确掌握每个替换的行区间，无需 LCS/diff 库：

```rust
pub struct EditDetails { pub diff: String, pub patch: String, pub first_changed_line: Option<usize> }
pub fn generate_edit_diff(original: &str, new: &str, spans: &[ReplacedSpan]) -> EditDetails;
```

- `diff`：被替换行 ±4 行上下文的行号标注 diff（对齐 pi `generateDiffString`
  的 `+行号 /-行号 / 行号` 格式与 firstChangedLine 语义）；
- `patch`：unified diff 文本（`--- a/path / +++ b/path` + @@ hunk）；
- 多处编辑合并为多个 hunk。

### 3. read / run_command 真 details

- read_file：`{ truncation: { total_lines, output_lines, truncated_by } }`
  （`tools.rs:100` 挂接点）；
- run_command：`{ truncation, full_output_path }`（PLAN-040 的
  `output_accumulator` 快照已有这两块，接 `tool_truncate.rs:26` 文档位）。

### 4. 事件链透传与持久化

- `relay/driver.rs` 桥接：`StreamEvent::Tool(name, content, details)` 的
  details 映射进 `RunEvent::ToolResult`（新增 `details: Option<Value>` 字段，
  serde default 兼容旧前端）；
- `conversation.rs` 的 ToolResult Turn 增加 details 持久化（刷新后前端回放
  仍有 diff）；
- SSE `SseEventDto` 原样 JSON 下发（沿 PLAN-040 ToolUpdate 的透传模式）。

### 5. 前端渲染

- ChatsView 工具结果块：edit 显示折叠 diff（details.diff）；read/run 显示
  截断徽标（N of M lines）；run_command 附 Full output 链接；
- spec 审批队列（approve/reject UI）消费 details.diff 展示将写入的变更
  （spec_tools 的 write_spec/update_spec 若产出 diff 则直接复用）。

### 6. 账面清理（PLAN-040 复审遗留）

1. `docs/plans/archived/040-*.md` frontmatter `current_step: 5` → `10`；
2. 删除 `auto_generated/extern_sigs.rs:574` 的 `batch_replace_do` 死签名；
3. DEBT-040-1/2/3（前端 E2E 冒烟、Windows Job Object 兜底、Unix killpg 实
   测）登记进 `docs/plans/KNOWN-DEBT-AND-RISKS.md`。

## 任务分解（10 步）

1. 手写轨 26 处 impl 机械迁移，`cargo check` 恢复零错误（仅 `text()` 包裹，
   行为零变化）。**[✅ 2026-08-24 完成，ad5b2a4]** 26 签名 + 31 返回点 +
   测试适配（execute 绑定后取 `.content`）；验证：lib 392 + 28 集成目标
   232 + bin 全部 0 失败；musk serve :8580 已用新二进制重启。
2. auto_generated 轨 16 处处置（迁移或登记休眠），全仓 `cargo check` 绿。
   **[✅ 处置=登记休眠，2026-08-24]** 镜像轨 impl 未出现在错误清单
   （不参与编译，039 复审的休眠结论成立）；唯一报错的 extern_impl
   3 处为 StreamEvent::Tool 匹配缺 `details`，已绑 `details: _` 修复；
   server.rs hw 轨已顺带把 details 透传进 SSE（任务 7 的 hw 侧先行片）。
3. `edit_diff.rs` 增加 `generate_edit_diff`（diff/patch/first_changed_line）
   + 单测（CRLF 文件行号、多 hunk、上下文截断）。**[✅ 2026-08-24，511202c]**
   AppliedEdits 扩展 `replaced_groups`（替换组新旧行区间，分组逻辑与模糊
   路径共用）；generate_edit_diff 5 单测（单编辑/多组行号换算/远距省略号/
   删除/无尾换行）。
4. edit_file 返回真 details；tools.rs:591 挂接点注释移除。**[✅ 511202c]**
   details={diff, patch, first_changed_line}；锚定测试
   edit_file_details_carries_diff_and_patch。
5. read_file details（tools.rs:100 挂接点）。**[✅ 511202c]**
   details={truncation:{total_lines, output_lines, truncated_by}}（截断与
   user_limit 续读两种情形都带）；锚定测试 read_file_details_on_truncation
   （注：total_lines 为 split 口径，含末尾换行产生的空行）。
6. run_command details（truncation + full_output_path）。**[✅ 511202c]**
   含 last_line_partial；仅截断时携带（错误路径走 ToolError 文本，无 details）。
7. relay 桥接 + RunEvent::ToolResult.details + conversation 持久化 + SSE 透传。
   **[✅ 511202c]** RunEvent.TurnToolResult / ToolRecord 增 details 字段
   （serde default 兼容旧回放）；driver + extern_impl ag 轨 + conversation
   持久化透传；SSE 经 serde 自动下发；ag 镜像轨（relay_store/conversation
   产物 + .at 源）手工同步（a2r 转译有已知漂移，镜像轨按惯例手工维护）；
   SSE details 序列化锚定测试（server.rs contract_stream_event_tool_pairing）。
8. 前端渲染（diff 折叠块 / 截断徽标 / Full output 链接）。**[✅ 2026-08-24，
   0e431d6]** 双轨：web（ChatsView 泛型卡 + useForge 留存 + 类型）与 gen
   （generic_tool_card.at 计算属性守卫 + forge_helpers.at toolTruncBadge +
   forge_store.at 留存）；auto build 30 组件 + 双端 vue-tsc/vite build 绿。
9. 账面清理三件。**[✅ 2026-08-24]** ① 040 frontmatter current_step 5→10；
   ② extern_sigs.rs:574 `batch_replace_do` 死签名已删（无其他引用，check 绿）；
   ③ DEBT-040-1/2/3 已登 KNOWN-DEBT 🟢 区。
10. 回归：`cargo test` 全绿 + 手工冒烟（编辑看 diff、读大文件看徽标、刷新后
    details 仍在）。**[✅ 自动化部分 2026-08-24]** cargo lib 399 + 集成全绿
    （603 通过 0 失败）；双前端 build 绿；:8580 已起新二进制。**手工冒烟
    留待用户**（真实 LLM 会话编辑/读大文件/刷新回放）。

## 验收标准

- `cargo check && cargo test -p musk` 全绿（当前 33 错误清零）。
- edit/read/run_command 的 ToolResult SSE 事件携带 details；LLM 请求体不含
  details（信任 auto-ai 链路保证，musk 侧以事件捕获测试锚定）。
- 会话刷新回放后 diff 徽标仍渲染（conversation 持久化生效）。
- 审批 UI 能展示 spec 写入的 diff（若 spec 工具接了 details）。

## 复审记录

### /auto-plan:review 正式复审（2026-08-25）

| 验收项 | 判定 | 证据 |
|---|---|---|
| `cargo check && cargo test -p musk` 全绿（33 错清零） | pass | lib 400 + 28 集成目标全绿（本复审重跑，fc4b005 含两处复审补丁）；复审中又捕获一例新的跨仓漂移——auto-ai 031（压缩二期）给 `CompletionResponse` 加 `model_meta`，server.rs MockClient 补 `model_meta: None` 即恢复（1 处，非本计划缺陷） |
| edit/read/run_command SSE 携带 details；LLM 请求体不含 | pass | SSE 锚定 `contract_stream_event_tool_pairing`（details 透传 + None→null）；三工具 details 内容锚定（edit_file_details…/read_file_details…）；LLM 侧零泄漏信任 auto-ai mvp_harness 既有断言（042 §pi 索引约定） |
| 刷新回放后 diff 仍渲染（conversation 持久化） | pass | 复审补锚定 `run_event_tool_result_details_persisted_into_turn`（映射 + serde 往返不丢，fc4b005） |
| 审批 UI 展示 spec 写入 diff | n/a | 计划原文括号条款（"若 spec 工具接了 details"）——spec_tools 未接 details，未启用该分支 |

**遗留（非阻断，均已在册）**：① T10 手工冒烟（真实 LLM 会话编辑/读大文件/刷新回放）留待用户；② SSE `tool_result.status` 真字段维持 027 债务（事件无 error 标记）；③ ag 镜像轨为手工同步而非 retranspile（a2r 已知漂移，随 041 轨道处置收尾）。

**结论**：review_done，可进入 /auto-plan:merge。

## 风险

- RunEvent 加字段：serde default 兜底旧事件回放；前端对未知工具名的 details
  静默忽略。
- diff 行号在 CRLF/BOM 文件上的正确性：单测覆盖（edit_diff.rs 已有 CRLF
  fixtures 可复用）。
- auto_generated 轨若需迁移，涉及 .at 源同步（retranspile）——与 PLAN-041
  的轨道处置决策协调，必要时只迁 rust-ref 侧保证编译、镜像轨随 041 收尾。
