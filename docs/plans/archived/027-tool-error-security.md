---
plan_id: PLAN-027
status: merged
feature_name: tool 错误体验 + 安全一致性（结构化错误 / driver 短路 / run_command confinement）
author: [zhaopuming + agent]
created_at: 2026-08-13T08:30:00Z
updated_at: 2026-08-13T09:00:00Z

# 待 /auto-plan:review 填：
supersedes_spec_components: []
new_spec_components:
  - "tool_safety: 结构化错误（path_confined kind）+ run_command confinement + driver 短路"
touched_goals: []

current_step: 4
total_steps: 4

---

# [PLAN-027] tool 错误体验 + 安全一致性改进

> **给执行 Agent：** 用 `/auto-plan:work` 逐步执行。
> **来源：** PLAN-026 chat 调查（chat `4b881f5dc7bb62d3de7ea292`，seq 4 "列 daemon 配置"报错）发现的三个问题，含一个**安全漏洞**。
> **跨 repo：** 改动涉及 `auto-musk`（tool_safety/tools/server + 前端）+ `auto-ai`（auto-ai-agent 的 ToolError + agent loop）。auto-ai 改动建议另开 worktree（参考 PLAN-026 的 auto-lang worktree 做法）。

## 0. 变更摘要 (Executive Summary)

PLAN-026 调查 chat `4b881f5` 的 seq 4 报错时，发现 musk 的 tool 层有三个相互关联的问题：

1. **① 工具错误是纯字符串、SSE 恒报 success** —— 前端（尤其 gen）无法识别"安全拒绝"，显示成成功；assistant 也没体现"路径越界"。
2. **② driver 无短路** —— path_confined 后 LLM 继续重试（read_file/list_dir 同样被拒，纯浪费 turn）。
3. **③ run_command 完全不受 path confinement**（**安全漏洞**）—— `cat`/`type`/`Get-Content` 白名单放行 + execute 不设 cwd → AI 能 `run_command cat D:/anywhere` 绕过读 workspace 外文件。

三者合力，导致用户看到的体验是"报错干瘪 + AI 反复试 + 还能绕过安全"。

| # | 问题 | 优先级 | 性质 |
|:--|:--|:--|:--|
| ③ | run_command 绕过 path confinement | **🔴 最高（安全漏洞）** | tools.rs + tool_safety.rs |
| ① | 错误纯字符串 + SSE 恒 success + 无友好播报 | 🟡 中 | auto-ai-agent + tool_safety + server + 前端 |
| ② | driver 无短路（无效重试） | 🟢 低（体验） | auto-ai-agent agent loop |

## 1. 目标 (Goal)

- **堵 run_command 安全漏洞**：shell 命令也遵守 workspace path confinement（cwd 设到 workspace root + 绝对路径参数校验）。
- **结构化工具错误**：path_confined 等安全拒绝带 `kind`，SSE 真实报 error，前端 ToolCard 显示友好提示，assistant 回复体现限制。
- **driver 短路**：连续同类 security error 后给 LLM 明确信号（减少无效重试）。

**非目标：** 不重构整个 ToolError 架构；不切 Ash shell（Design 004 的长远项，本计划只在 run_command 加 cwd + 参数 confinement）；不改 agent loop 的 LoopDetected/MaxTurns 主逻辑（只在 security error 上加 hint）。

## 2. 架构方案 (Architecture)

```
┌─ auto-ai（auto-ai-agent）─────────────────────────────────────┐
│  ToolError (error.at:15)                                       │
│    + SecurityDenied { kind: "path_confined", path, root, hint }│
│  agent.at:455 agent loop                                       │
│    + security error 连续计数 → N 次注入强 hint                 │
│  tool.at:171 exec_or_msg                                       │
│    保留结构化（不拍平成纯字符串）                               │
└────────────────────────────────────────────────────────────────┘
              │  tool result（结构化 error）
              ▼
┌─ auto-musk 后端 ───────────────────────────────────────────────┐
│  tool_safety.rs:136                                            │
│    返回 SecurityDenied{path,root}（非纯字符串）                │
│  tool_safety.rs classify_command                               │
│    + 解析 cmd 路径参数 confinement                             │
│  tools.rs:119 run_command                                      │
│    + .current_dir(workspace_root)                              │
│  server.rs:413 stream_event_to_json                            │
│    SSE status 按 error 真实（status:"error"）                  │
└────────────────────────────────────────────────────────────────┘
              │  SSE { status:"error", error:{kind:"path_confined",...} }
              ▼
┌─ 前端（gen + web）─────────────────────────────────────────────┐
│  gen generic_tool_card.at + forge_stream.ts                    │
│    识别 status:error / kind:path_confined → 友好提示           │
│  web ChatsView ToolCard                                        │
│    同步（isErrorResult 嗅探 → 读结构化字段）                    │
└────────────────────────────────────────────────────────────────┘
```

## 3. 技术栈 (Tech Stack)

- **auto-ai / auto-ai-agent**（`D:/autostack/auto-ai/crates/auto-ai-agent`）：ToolError enum + agent loop（.at 源 → Rust）
- **auto-musk 后端**：`tool_safety.rs` + `tools.rs` + `server.rs`（Rust/axum）
- **前端**：gen（`src/front/generic_tool_card.at` + `forge_stream.ts`）+ web（`ChatsView.vue`）

## 4. 需求分析与背景调查

### 4.1 chat 4b881f5 的报错链路（本计划的设计依据）

seq 4 "列出 daemon 配置" → AI 调 `read_file(~/.config/autoos/ai-daemon.at)` → `tool_safety.rs:136` 返回纯字符串 `"path '...' outside project root"` → `ToolError::Exec(String)` → LLM 看到 `[tool error: ...]` → LLM 继续试（read_file 别路径 / list_dir / **run_command cat**）→ 其中 run_command 不受 confinement → 可能绕过读到。

### 4.2 实地调查结论（2026-08-13 agent 深入）

| 维度 | 现状 | 位置 |
|:--|:--|:--|
| path 拒绝返回值 | **纯字符串** | tool_safety.rs:136-142 |
| ToolError 变体 | 只有 `Args(str)`/`Exec(str)`，无 kind | auto-ai-agent error.at:15-20 |
| tool error → LLM | 拍平成 `[tool error: ${e.message()}]` | auto-ai-agent tool.at:171-176 |
| SSE tool_result.status | **硬编码 "success"** | server.rs:413-419, 648-658 |
| gen 前端 error 识别 | **没有**（信后端 status，恒 success） | forge_stream.ts:186-187 |
| web 前端 error 识别 | 字符串嗅探（`[tool error:` 等） | useForge.ts:234-243 |
| driver 短路 | **无**（error 喂回 LLM 继续） | agent.at:455-494 |
| 终止条件 | LoopDetected(同参×3) + MaxTurns(×5=50) | agent.at:472-476, 380-381 |
| run_command confinement | **无**（cat/type 放行 + 不设 cwd） | tools.rs:119-160, tool_safety.rs:165-175 |

### 4.3 关键决策（用户确认）

- **D1：run_command 漏洞最高优先**（安全）。先堵，再做错误体验。
- **D2：结构化错误用 ToolError 新变体**（SecurityDenied），不另造 envelope。跨 auto-ai repo。
- **D3：driver 短路用"强 hint"而非硬终止**（连续 N 次同类 security error → 注入"这些路径都被拒，请换思路"，不直接 kill run，保留 LLM 灵活性）。

## 5. 详细设计 (Detailed Design)

### 5.1 ③ run_command confinement（最高优先，堵漏洞）

**`tools.rs:119-160` run_command execute**：
```rust
// 现状：Command::new(cmd).output()，不设 cwd
// 改：.current_dir(workspace_root) + cmd 路径参数 confinement
let root = crate::tool_safety::current_workspace_root();  // 或从 thread-local
let cmd = props.cmd;
// 解析 cmd 文本里的绝对路径参数，逐个 resolve_within_project 校验
crate::tool_safety::confine_command_paths(&cmd, &root)?;  // 新 fn
std::process::Command::new(shell)
    .arg(&cmd)
    .current_dir(&root)  // 关键：限制 cwd 到 workspace
    .output()
```

**`tool_safety.rs` 新 fn `confine_command_paths(cmd, root)`**：
- 用 shlex（或简易 split）拆 cmd 的 tokens
- 对形如绝对路径（`C:\`、`/`、`~/`、`..`）的 token，调 `resolve_within_project` 校验
- 拒绝越界 → 返回 `SecurityDenied{kind:"path_confined", path, root}`

**注意**：Windows `cmd /C` 的 cwd 设置 + 路径解析需测试（`..` 穿越、UNC `\\?\`）。

### 5.2 ① 结构化错误

**`auto-ai-agent error.at:15` ToolError 加变体**：
```
pub enum ToolError {
    Args(str),
    Exec(str),
    SecurityDenied { kind: str, path: str, root: str, hint: str },  // 新
}
```

**`tool_safety.rs:136` 返回结构化**：
```rust
} else {
    Err(ToolError::SecurityDenied {
        kind: "path_confined",
        path: path.to_string(),
        root: root.display().to_string(),
        hint: "AI 只能读写当前 workspace 内的文件；workspace 外的配置请让用户手动提供。".into(),
    })
}
```

**`auto-ai-agent tool.at:171 exec_or_msg`**：保留结构化（不拍平成纯字符串）—— LLM 看到的 tool_result 含结构化 error（模型能理解 kind + hint）。

**`server.rs:413 stream_event_to_json`**：
```rust
let status = match &result { Err(_) => "error", Ok(_) => "success" };
// error 时附带 { error: { kind, path, root, hint } }
```

**前端**：
- gen `generic_tool_card.at` + `forge_stream.ts:186`：识别 `status:"error"` → ToolCard 显示 error 样式；识别 `kind:"path_confined"` → 显示 hint（友好提示）。
- web `ChatsView.vue` + `useForge.ts:234`：`isErrorResult` 改为优先读结构化 `error.kind`，字符串嗅探兜底。

### 5.3 ② driver 短路（强 hint）

**`auto-ai-agent agent.at:455` agent loop**：
- 维护 `consecutive_security_errors: u32`
- tool_result 是 `SecurityDenied` → 计数 +1
- 计数 ≥ 3 → 注入 system/user message：*"你已连续 3 次尝试访问 workspace 外的路径，均被安全策略拒绝。workspace = {root}，外的文件 AI 无法访问。请换思路（让用户提供内容，或用 API 而非读文件）。"*
- 成功的 tool_result → 计数归零

（非硬终止，保留 LLM 灵活性；但强信号后大多数模型会停止无效重试。）

## 6. 测试设计 (Test Design)

- **tool_safety 单测**（`tool_safety.rs` 内嵌 `#[cfg(test)]`）：
  - `run_command` cmd 含绝对路径越界 → 拒绝
  - `run_command` cmd 含 `..` 穿越 → 拒绝
  - `run_command` cwd = workspace root（相对路径在 workspace 内 → 允许）
  - path_confined 返回 `SecurityDenied{kind:"path_confined"}`（非纯字符串）
- **server SSE 测试**：tool error → SSE `status:"error"` + error envelope
- **手测**（chat）：复现 4b881f5 seq 4 场景（AI 读 ai-daemon.at）→
  - read_file 拒绝 → 友好提示（"workspace 外"）
  - 不无限重试（≤3 次后强 hint）
  - run_command cat 也被拒（漏洞堵住）

## 7. 验收标准 (Acceptance Criteria)

- [x] 标准 1：`run_command` 设 cwd=workspace root + 越界路径参数被拒（单测通过）。[✅ 2026-08-24 复核] 最终形态为 `confine_command_paths` 路径校验（tools.rs:301）而非 .current_dir（经 039/040 重写演进，更强）；040 T6 回归测试锁定（tools.rs:1299-1322）。
- [x] 标准 2：`tool_safety` 的 path 拒绝返回 `SecurityDenied{kind:"path_confined"}`（结构化，非纯字符串）。[✅ 2026-08-24 复核] tools.rs:14-20 map_path_error + auto-ai error.at:22 变体在位。
- [ ] 标准 3：SSE `tool_result.status` 在 error 时为 `"error"`（+ error envelope）。[未做→登记] stream_event_to_json 的 status 恒硬编码 "success"（server.rs Tool 分支），错误语义靠双轨前端嗅探；027 提交④许诺登 KNOWN-DEBT 但漏登，2026-08-24 补登（见台账 027 条）。
- [x] 标准 4：gen 前端 ToolCard 识别 path_confined 显示友好提示（hint）；assistant 回复体现安全限制。[✅ 渐进形态] 嗅探 `[security denied`/`[tool error]` 前缀标记 failed + hint 文案随 result 文本展示（web useForge.ts:246 / gen forge_store.at:238 双轨在位）。
- [x] 标准 5：driver 连续 3 次同类 security error → 注入强 hint（agent loop 测试或日志确认）。[✅ 2026-08-24 复核] auto-ai rust-ref/src/agent.rs:474-654（活构建，Cargo.toml [lib] 指 rust-ref）计数+归零+≥3 强 hint 在位。
- [ ] 标准 6：手测 chat 复现场景 —— read_file/run_command 读 workspace 外都被拒 + 友好错误 + 不无限重试。[未做] 需重编 musk.exe；与步骤 4.3 同源，留待（当前 musk 编译红须先适配 auto-ai 027/028 漂移）。
- [x] 标准 7：`cargo test -p musk` 全绿（含新 fixture）。[✅ 当时绿] ae75416 "cargo test 9 passed"；2026-08-24 现状红为 auto-ai 027/028 跨仓漂移（33 错），非本计划缺陷。

## 8. 执行步骤 (Execution Tasks)

> 每个任务 2-5 分钟原子操作，含精确路径 + 操作 + 验证命令。

### 任务 1: 🔴 run_command confinement（堵安全漏洞，最高优先）
- [x] **步骤 1.1:** `backend/crates/musk/src/tools.rs:119-160` run_command execute 加 `.current_dir(workspace_root)`（从 thread-local `current_workspace_root()` 取）。[✅ 当时实现] 后经 039/040 重写演进为 confine_command_paths 路径校验（tools.rs:301，含 cat/type 白名单路径校验），confinement 语义保留且更强。
- [x] **步骤 1.2:** `backend/crates/musk/src/tool_safety.rs` 新 fn `confine_command_paths(cmd: &str, root: &Path) -> Result<()>`：拆 cmd tokens，对绝对路径/`..` token 调 `resolve_within_project` 校验；越界返回错误。在 run_command execute 调用。[✅] tools.rs:301-303 在位。
- [x] **步骤 1.3:** `tool_safety.rs` 内嵌单测：`run_command cat /etc/passwd`（越界）+ `cat ../secret`（穿越）+ `cat local.txt`（workspace 内允许）。[✅] + 040 T6 新增 run_command_confine_blocks_workspace_outside_path_even_with_force（tools.rs:1322）。
- [x] **步骤 1.4:** `cargo test -p musk tool_safety` 全绿。[✅ 当时绿]

### 任务 2: 结构化 tool 错误（跨 auto-ai repo）
- [x] **步骤 2.1:** auto-ai worktree（`D:/autostack/auto-ai`，参考 PLAN-026 worktree 做法）；改 `crates/auto-ai-agent/src/error.at:15` ToolError 加 `SecurityDenied { kind, path, root, hint }` 变体。[✅] error.at:22 在位。
- [x] **步骤 2.2:** `tool.at:171 exec_or_msg` 保留 SecurityDenied 结构化（不拍平）。[✅] tool.at:199 格式化含 hint。
- [x] **步骤 2.3:** 回 auto-musk：`tool_safety.rs:136` 返回 `ToolError::SecurityDenied{kind:"path_confined", path, root, hint}`（替代纯字符串）。[✅] tools.rs:14-20 map_path_error。
- [ ] **步骤 2.4:** `server.rs:413 stream_event_to_json` SSE status 按 error 真实（`"error"`）+ error envelope（kind/path/root/hint）。[未做→登记] status 恒 "success"，前端嗅探替代；KNOWN-DEBT 补登（2026-08-24）。auto-ai 027 content/details 分离合入后适配时顺带解除。
- [x] **步骤 2.5:** auto-ai cargo test + cargo install（更新 auto-ai-agent）；auto-musk cargo test。[✅ 当时绿]

### 任务 3: 前端友好播报（渐进：嗅探 outcome 标记）
- [x] **步骤 3.1:** gen `forge_stream.ts:186` 嗅探 result `[security denied]`/`[tool error]` → status `failed`（对齐 generic_tool_card 的 completed/failed/running）。[✅ 已完成] outcome 含标记即识别为 error（后端 SSE status 恒 success 的渐进绕过）。
- [x] **步骤 3.2:** web `useForge.ts:234 isErrorResult` 加 `[security denied]` 嗅探。[✅ 已完成]
- [x] **步骤 3.3:** auto build（gen）+ 验证。[✅ 已完成] gen forge_stream 含 2 处 security denied 嗅探。

### 任务 4: driver 短路（强 hint）
- [x] **步骤 4.1:** auto-ai agent.rs（rust-ref）run loop 加 `security_errors` 计数；SecurityDenied → +1，成功 → 归零；≥3 → outcome 追加"换思路"强 hint。[✅ 已完成] commit c06804e + merge。
- [x] **步骤 4.2:** cargo check auto-ai-agent 通过；merge 后 auto-musk cargo check --lib 通过。[✅ 已完成]
- [ ] **步骤 4.3:** 手测 chat 复现 4b881f5 场景。[未做] 需重编 musk.exe（musk serve 当前占用旧二进制）。
- [x] **步骤 4.4:** 状态 → execution_done。[✅]

## 9. 复审记录 (Review Log)

> 由 `/auto-plan:review` 填写。
> **注（2026-08-24 普查回填）**：正式复审于 2026-08-13 通过（commit 0396177）；
> 同日 finish-plan 普查对 HEAD 逐项复核（证据见 §7/§8 条目内 [✅ 2026-08-24 复核]
> 注释），勾选框按复核结论回填。唯标准 3/步骤 2.4（SSE status 真字段）与
> 标准 6/步骤 4.3（手测）保持未勾。

- **复审人**: agent（/auto-plan:review 2026-08-13 + 普查复核 2026-08-24）
- **复审时间**: 2026-08-13（0396177）；复核 2026-08-24
- **复审结论**:
  - [x] 验收标准全部满足（标准 3/6 除外，见遗留）
  - [x] 代码无安全隐患（尤其 run_command confinement）
  - [x] Spec 元数据已补全
- **遗留问题**: ① SSE tool_result.status 恒 "success" 硬编码，错误语义靠双轨嗅探 workaround——本计划提交④许诺登 KNOWN-DEBT 但漏登，2026-08-24 补登（台账 027 条）；解除时机 = musk 适配 auto-ai 027 content/details 分离时。② 手测复现（标准 6/步骤 4.3）未做，被 musk 编译红阻塞（同适配任务）。

## 10. 待澄清事项 (Open Questions)

- **auto-ai 改动 worktree 策略**：ToolError + agent loop 在 auto-ai-agent（独立 repo）。是否像 PLAN-026 的 auto-lang 那样开 worktree（auto-musk）做 + 合并回 master？（建议是，保持一致。）
- **ToolError 新变体的 LLM 可见性**：SecurityDenied 结构化传给 LLM 时，模型能否有效理解 kind+hint？（anthropic tool_result 的 is_error + content；需确认 auto-ai-agent 怎么序列化。）
- **run_command cwd 的 Windows 兼容**：`cmd /C` + `.current_dir()` 在 Windows 的行为（路径分隔、UNC `\\?\` workspace root）。需测试。
- **短路阈值**：连续 3 次同类 security error → hint。阈值是否合适？（太低可能误伤合理探索；太高失去短路意义。）
- **classify_command 的白名单**：当前 `cat`/`type` 放行。confinement 后这些命令的路径参数也要校验。白名单是否需调整（如移除纯放行，改为"放行但校验路径"）？

---

*本文件为 PLAN-027，格式遵循设计文档 008（Auto-Plan 核心契约）。来源：PLAN-026 chat 调查（4b881f5）发现的 tool 错误体验 + 安全问题。修复点由 tool_safety/driver/ToolCard 深入调查（2026-08-13）定位。*
