---
plan_id: PLAN-043
status: drafting
feature_name: 会话树分支——从任意轮 fork 重试、branchSummary、树导航（对齐 pi 单文件会话树模型）
author: [zhaopuming]
created_at: 2026-08-24
updated_at: 2026-08-24

supersedes_spec_components: []
new_spec_components: []
touched_goals: []

current_step: 0
total_steps: 9
---

# [PLAN-043] 会话树分支：fork、branchSummary 与树导航

## 变更摘要

musk 的会话目前是**纯线性**的：一次回答不满意只能继续追加消息（"重新生成"
语义上等于再追加一轮），旧路径与新尝试混在同一条时间线里，无法对比、无法
回退到某轮重试。而数据结构其实已有基础：`Conversation` 带
`parent_id`/`parent_turn_id`（`conversation.rs:18-20`，当前用于 errand/子任务
的父子 conversation），`Turn` 也带 `parent_id`（`conversation.rs:148`），
存储是 append-only 的 `{dir}/{conv-id}/turns.jsonl + meta.json`
（`conversation.rs:468-469、644`）。

pi 的会话模型是**单文件树**：每条 entry 带 `parentId`，分支不建新文件，上下文
投影按"活跃 leaf → root"路径重建，离开分支时生成 branchSummary 摘要，全历史
永远可回看。本计划把这套模型移植到 musk 的 ConversationStore：从任意 assistant
轮 fork 出新分支、在分支间导航、（可选）离开分支时摘要——Web 端 Chats 视图
获得"从这里重试"能力。

**前置**：无硬前置；建议在 PLAN-042 之后做（其 run_command 等工具行为先稳定）。
**与 auto-ai 的关系**：纯 musk 侧实现（ConversationStore 在 musk 仓）；Agent
只需继续用现有 `with_history` 按树路径喂消息。压缩锚点（auto-ai PLAN-028/031
的 summary）在树路径上的共存见 §风险。

## pi 参考实现索引

pi 仓库本地克隆 `D:\github\pi`（main @ a1f955e9f）：

| 关注点 | pi 位置 | 移植要点 |
|---|---|---|
| 单文件树模型：SessionEntryBase { id, parentId }，分支 = 挂新子节点，不建新文件 | `packages/coding-agent/src/core/session-manager.ts`（SessionEntry 定义与静态工厂） | musk 对应：Turn.parent_id 已存在，补树语义与活跃 leaf 概念 |
| 上下文投影：从活跃 leaf 回溯 root 构建消息序列（遇 compaction 锚点从锚点起） | `packages/agent/src/harness/session/context.ts`（buildSessionContext） | musk 对应：`with_history` 改为按 leaf→root 路径投影 |
| 离开分支时生成 branchSummary（LLM 摘要被离开的分支，供回来时快速恢复） | `packages/agent/src/harness/compaction/branch-summarization.ts` | musk 默认关、手动开；先做"分支保留原样可回看"，摘要为增强 |
| fork 语义（fork = 在指定位置前分叉，原历史不动） | `packages/coding-agent/src/modes/rpc/rpc-types.ts` 的 session/fork、get_tree、navigateTree 命令面 | API 设计蓝本：POST /fork、GET /tree、POST /navigate |
| 树内导航的原地切换（不复制会话、不建新文件） | `packages/coding-agent/src/core/agent-session-runtime.ts`（会话/分支切换的 runtime 重建） | musk 对应：切 active_leaf 后下一请求按新路径 with_history |
| 入口类型清单（message/model_change/compaction/branchSummary/custom…） | `packages/coding-agent/src/core/session-manager.ts` 的 SessionEntry 类型 | 参考哪些变更要留痕；musk 现有 TurnKind 已覆盖大部分 |
| 树可视化与导航 UI | `packages/coding-agent/src/modes/interactive/components/`（session/tree 选择器） | Web 端对应：时间线分叉标记 + 分支切换器 |

## 方案

### Phase 0：勘察（半步，先行决断）

1. **Turn.parent_id 的现语义**：grep 其赋值点，确认当前是否恒为 None（线性
   append）还是已有用途。若空闲 → 直接采用 turn 级树（pi 同构）；若已被
   errand 复用 → 评估改用独立字段 `branch_parent_id` 或复用语义兼容。
2. **ChatStore（chats.json）与 ConversationStore 的双存储关系**：确认 Chats
   视图读写哪一个；树功能落在实际供 Chat 消息流的存储上，另一侧做投影兼容。
3. **活跃分支的持久化位置**：`meta.json` 增加 `active_leaf: TurnId`。

### Phase 1：树投影与 fork

- **投影**：`history_for_request(conv) = 沿 active_leaf → root 收集 Turn，
  逆序还原为消息序列`。旧数据（parent 恒 None）自然退化为现行线性读取——
  无迁移成本。
- **fork from turn N**：`POST /api/chats/session/{id}/fork { turn_id }`——
  不复制任何数据；创建一个新的"分支标记"（首条新 Turn 以 N 为
  parent），active_leaf 切到新分支末尾（初始 = N）。前端时间线上 N 之后出现
  分叉。
- **navigate**：`POST .../navigate { turn_id }`——active_leaf 切到指定分支
  末尾；下一请求按该路径重建记忆。历史 turn 一律不可变（append-only 保持，
  树由 parent 指针表达——与 pi 一致）。

### Phase 2：branchSummary（可选增强，默认关）

- 离开一个"有实质进展"的分支（该分支 turn 数 ≥ 阈值）且开启设置时，对被
  离开分支发一次独立摘要请求（复用 auto-ai 的 compact 摘要链路或 musk 本地
  简化模板），摘要存为该分支末尾的 summary Turn；
- 回到该分支或从它再 fork 时，摘要随路径进上下文，降低恢复成本。
- 成本敏感：默认关闭，`modes`/设置里开。

### Phase 3：API 与前端

- `GET /api/chats/session/{id}/tree`：Turn 树（节点 = turn_id/kind/摘要行/
  分支名）；
- 前端 Chats 视图：消息时间线分叉缩进标记、分支切换器（下拉或侧栏）、
  assistant 轮 hover 出现"从这里重试"（= fork + 预填输入框）；
- SSE：树结构变更（fork/navigate）推送轻量事件刷新 UI。

## 任务分解（9 步）

1. Phase 0 勘察三项，结论写回本计划（占位节：勘察记录）。
2. 树投影函数 `history_for_request`（含旧数据线性退化）+ 单测（分叉后两分支
   各自路径正确、公共前缀只出现一次）。
3. meta.json `active_leaf` 持久化 + 加载兼容。
4. fork 端点 + 单测（fork 后旧分支只读保留、新分支追加落在新支）。
5. navigate 端点 + 单测（切换后 with_history 路径变化、turns.jsonl 无重写）。
6. tree 查询端点（节点投影 + 分支摘要行）。
7. branchSummary（可选档）：离开分支摘要生成与路径注入 + 设置开关。
8. 前端：分叉标记 / 分支切换器 / "从这里重试"交互 + SSE 树变更刷新。
9. 回归：`cargo test` + 前端 vitest + 手工冒烟（fork→两分支各自对话→切换→
   刷新后树仍在）。

## 验收标准

- 从第 3 轮 fork 重试：两分支独立演进，互不污染；切回原分支上下文与离开时
  一致（ScriptedClient/relay 捕获的请求消息序断言）。
- 旧会话（无 parent 的线性 jsonl）加载、对话、追加行为与改造前逐项一致。
- turns.jsonl 只追加、永不重写（文件级断言：fork/navigate 前后旧行字节不变）。
- 树查询响应规模可控（长会话下节点投影有截断/摘要行）。

## 风险

- **与压缩锚点共存**：auto-ai 压缩后 Memory 头部是摘要锚点；树投影发生在
  musk 侧 with_history（喂完整路径），两者叠加可能出现"摘要 + 又投影了被摘
  要掉的历史"。缓解：勘察项 4——确认 with_history 喂的是 ConversationStore
  全量还是 Memory 快照，若 musk 侧已按"压缩后截断"喂历史，则树投影只作用
  于保留尾部（与 pi 的"上下文不越过 compaction"一致）。
- ChatStore/ConversationStore 双存储的一致性：Phase 0 勘察定归属，避免两边
  各维护一套树。
- branchSummary 的 token 成本：默认关 + 阈值门槛。
- 前端复杂度：分叉渲染先做最小版（缩进 + 标记），对比视图后续再加。
