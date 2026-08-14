# 目标索引

> 基于 2026-08-14 全量代码扫描提炼的项目目标。

## 核心目标

### 1. AI 编码 Agent（goal-agent）
Rust 后端的 ReAct agent，经 aaid 代理调 LLM，工具在本地执行（读/写/编辑/搜索/命令/spec 读写/编排），path confinement 安全沙箱限制在 workspace 内。

### 2. Spec 知识沉淀（goal-spec-knowledge）
双落点：结构化 ledger（`.autoos/specs.json` 6 区 + 状态机 + relations）+ 文件树知识层（`docs/specs/`，本目录）。Plans 5 态状态机 + merge 沉淀（review_done → 拆解进 ledger + archive）。

### 3. Relay 编排（goal-relay）
多 agent 流水线编排（PipelineEngine + TaskPlan DAG），含 spawn_relay / dispatch / bring_in 编排工具，gate 审批，子会话管理。

### 4. 双前端 Parity（goal-frontend-parity）
原生 `web/`（Vue3 手写 SPA）+ Auto 轨 `.at`（codegen → gen/vue/），共用 NavSidebar/ContentHeader 组件 + authFetch + forge_stream SSE。inject_styles 全局兜底。

### 5. 安全一致性（goal-security）
工具安全三层：path confinement（workspace root + canonicalize）+ run_command confinement（cwd + cmd 路径校验 + 白名单分级）+ SecurityDenied 结构化错误（driver 短路 ≥3 次强 hint）。
