# 目标索引

> 基于 2026-08-14 全量代码扫描提炼的项目目标。

## 核心目标

### 1. AI 编码 Agent（goal-agent）
Rust 后端的 ReAct agent，经 aaid 代理调 LLM，工具在本地执行（读/写/编辑/搜索/命令/spec 读写/编排），path confinement 安全沙箱限制在 workspace 内。

### 2. Spec 知识沉淀（goal-spec-knowledge）
双落点：结构化 ledger（`.autoos/specs.json` 6 区 + 状态机 + relations）+ 文件树知识层（`docs/specs/`，本目录）。Plans 5 态状态机 + merge 沉淀（review_done → 拆解进 ledger + archive）。

### 3. Relay 编排（goal-relay）
多 agent 流水线编排（PipelineEngine + TaskPlan DAG），含 spawn_relay / dispatch / bring_in 编排工具，gate 审批，子会话管理。

### 4. 双前端 Parity（goal-frontend-parity，PLAN-074–079）
Auto 轨 `.at` 源码为单一真源，支持生成 Vue 3 与原生 VM/Iced 桌面端双端平权交付。两阶段验收目标清晰解耦：
- **近期交付目标（大体一致与行为平权）**：全量 108 声明单元与 105 测试场景 100% 覆盖；关键边界 ≤2px，长流累计 ≤4px；消息块（思考、工具门、卡片）、输入（Mention/Tag/IME）、全局壳（侧栏收缩、浮层弹窗、主题切换）和业务页面（文件树、规范、Wiki、白名单）端到端可操作且数据隔离。
- **远期演进目标（最终像素对齐）**：Iced 次像素抗锯齿微调、毛玻璃拟真阴影与原生滚轮阻尼深度优化，作为演进项独立登记，不虚报最终像素完成。

### 5. 安全一致性（goal-security）
工具安全三层：path confinement（workspace root + canonicalize）+ run_command confinement（cwd + cmd 路径校验 + 白名单分级）+ SecurityDenied 结构化错误（driver 短路 ≥3 次强 hint）。
