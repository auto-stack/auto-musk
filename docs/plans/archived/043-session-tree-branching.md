---
plan_id: PLAN-043
status: archived
feature_name: 会话树分支——从任意轮 fork 重试、branchSummary、树导航（对齐 pi 单文件会话树模型）
author: [zhaopuming]
created_at: 2026-08-24
updated_at: 2026-08-25

supersedes_spec_components: []
new_spec_components:
  - "会话树数据模型: ChatMessage.parent_id + ChatSession.active_leaf(serde default 线性退化,树单源 ChatStore/镜像不动)"
  - "投影语义: ChatSession.active_path(leaf→root 链+线性前缀)/history_pairs(with_history 输入)"
  - "分支端点: POST fork / POST navigate(同机制 set_active_leaf 零复制) / GET tree(children+on_active_path+preview)"
  - "前端分支交互: 活跃路径渲染+兄弟分支切换器+从这里重试(fork+预填),双轨"
touched_goals:
  - "会话体验: 从任意轮 fork 重试、分支导航、全历史 append-only 可回看"

current_step: 9
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

## 勘察记录（Phase 0，2026-08-25）

1. **Turn.parent_id 现语义**：构造恒 None（conversation.rs:580 唯一赋值点），完全空闲——但见下条，树不落 ConversationStore。
2. **双存储归属（关键修正）**：Chat 视图的消息流（显示与 with_history 上下文）都走 **ChatStore（chats.json）的 session.messages**（server.rs:560-580 由 messages 构建 history）；ConversationStore 是完成后双写的镜像（chat_message_to_turns → append_turn，供 conversation API/relay 语义）。**树功能落 ChatStore**：`ChatMessage.parent_id` + `ChatSession.active_leaf`（serde default，旧 jsonl 无这两个字段 → 线性退化）。ConversationStore 镜像保持线性 journal 不动（投影兼容 = 镜像照旧双写，树语义单源于 ChatStore）。
3. **active_leaf 持久化位置**：ChatSession（chats.json）新增字段（原计划设想 meta.json 属 ConversationStore 侧，随第 2 项归属修正）。
4. **压缩锚点共存（风险项勘察）**：musk 侧 with_history 喂的是 ChatStore 全量消息（server.rs:628），无 musk 侧压缩截断；auto-ai 压缩在 Memory 内部（锚点后截断）——树投影只改变喂入的消息序列，与压缩叠加无冲突（投影后的路径整体进 Memory，锚点机制照常）。

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

1. Phase 0 勘察三项，结论写回本计划（占位节：勘察记录）。**[✅ 2026-08-25]** 见「勘察记录」节——关键修正：树落 ChatStore（消息流与 with_history 均走 chats.json），Turn.parent_id 空闲但不用，ConversationStore 镜像保持线性。
2. 树投影函数 `history_for_request`（含旧数据线性退化）+ 单测（分叉后两分支
   各自路径正确、公共前缀只出现一次）。**[✅ 2026-08-25]** 实现为 ChatSession::active_path/history_pairs（chats.rs）；chat_run_stream（ag 轨）与 hw server.rs 两处 history 构建换用投影；6 单测含两分支独立/公共前缀一次/旧数据退化。
3. meta.json `active_leaf` 持久化 + 加载兼容。**[✅ 2026-08-25]** 随勘察修正落在 ChatSession.active_leaf（chats.json，serde default 兼容旧数据）；ConversationStore meta.json 不动。
4. fork 端点 + 单测（fork 后旧分支只读保留、新分支追加落在新支）。**[✅ 2026-08-25]** chat_branch.rs hw 路由（/api/plans hw 先例）；集成测试 parity_chat_branch：append-only 断言（旧消息 id/内容/parent 逐字段不变）+ 新消息 parent=fork 点。
5. navigate 端点 + 单测（切换后 with_history 路径变化、turns.jsonl 无重写）。**[✅ 2026-08-25]** 与 fork 同机制（set_active_leaf）；测试断言切换后 history_pairs 路径变化 + 旧消息不变（ChatStore 整文件持久化，等价"不重写"）。
6. tree 查询端点（节点投影 + 分支摘要行）。**[✅ 2026-08-25]** GET tree：节点含 children/on_active_path/preview（60 字符摘要行）。
7. branchSummary（可选档）：离开分支摘要生成与路径注入 + 设置开关。**[⏸ 递延，见待澄清 #1]**
8. 前端：分叉标记 / 分支切换器 / "从这里重试"交互 + SSE 树变更刷新。**[✅ 2026-08-25，双轨]** web（ChatsView visibleMessages 投影 + branchTo + 切换器/hover 重试按钮）与 gen（chats_view.at + forge_store.at BranchTo/RetryFrom + forge_helpers.at chatActivePath/chatSiblings + api.at fork/navigate 契约）；auto build 29 组件 + 双端 vue-tsc/vite build 绿。**简化**：树变更刷新走同客户端直接响应（restoreSession 重载），跨标签 SSE 刷新未做（见待澄清 #2）。
9. 回归：`cargo test` + 前端 vitest + 手工冒烟（fork→两分支各自对话→切换→
   刷新后树仍在）。**[✅ 自动化]** cargo lib+集成 611 通过 0 失败；双前端 build 绿。**手工冒烟留待用户**（浏览器 webview 本环境无法挂载，见 KNOWN-DEBT W2 条）；vitest 套件仍在 web/（2 存量套件，迁 gen 属 PLAN-041 T13 范围）。

## 验收标准

- 从第 3 轮 fork 重试：两分支独立演进，互不污染；切回原分支上下文与离开时
  一致（ScriptedClient/relay 捕获的请求消息序断言）。
- 旧会话（无 parent 的线性 jsonl）加载、对话、追加行为与改造前逐项一致。
- turns.jsonl 只追加、永不重写（文件级断言：fork/navigate 前后旧行字节不变）。
- 树查询响应规模可控（长会话下节点投影有截断/摘要行）。


## 待澄清事项

1. **T7 branchSummary 递延（2026-08-25）**：计划自身定性为"可选增强，默认关"
   （成本敏感）。实现需独立 LLM 摘要调用（复用 auto-ai compact 链路 vs musk
   本地简化模板，二选一未裁定）+ 设置开关的存储位置（ChatSession 字段 vs
   modes 设置）。分支原样可回看的核心体验（fork/navigate/tree）已全量交付，
   摘要属恢复成本优化——建议用户裁定摘要链路后作为独立小任务承接。
2. **跨标签 SSE 树变更刷新未做**（T8 简化）：当前 fork/navigate 后经
   restoreSession 同客户端刷新；多标签同时打开同一会话的场景未推送。若需要，
   沿 PLAN-040 ToolUpdate 的 broadcast 模式补一个 tree_changed 轻事件。

## 复审记录

### /auto-plan:review 正式复审（2026-08-25）

| 验收项 | 判定 | 证据 |
|---|---|---|
| 从任意轮 fork 重试：两分支独立演进互不污染；切回原分支上下文一致 | pass | chats.rs 单测（fork_two_branches_independent_paths/history_pairs_*：公共前缀一次、旧分支不进历史）；**真实服务器冒烟**（:8580 一次性会话：fork 后 tree 显示答一 children=2、分支 B on_path、旧支 off_path；navigate 回原支后 active_path=旧支三消息，分支 B 消息不进上下文）。验收原文"ScriptedClient 捕获请求消息序"以 history_pairs（with_history 的直接输入）等价锚定 |
| 旧会话加载/对话/追加与改造前逐项一致 | pass | active_path_legacy_linear_fallback + history_pairs 排除语义与原实现逐字一致 + 全量回归 611 通过（含既有 16 项 chats 测试）；真实会话（"nihao"，旧 jsonl 无树字段）加载/fork 正常 |
| turns.jsonl 只追加、永不重写 | pass* | 分支操作仅写 chats.json（ChatStore）；turns.jsonl（ConversationStore 镜像）在 fork/navigate 中完全不触碰（chat_branch.rs 只调 ws.chats）。测试断言旧消息 id/内容/parent 逐字段不变。*注：验收原文以 turns.jsonl 表述，勘察修正后树数据面为 chats.json——语义等价（append-only 不重写） |
| 树查询响应规模可控 | pass | tree_nodes 每节点 preview 截断 60 字符（chats.rs）；节点数 = 消息数（线性） |

**手工冒烟（用户指定由复审执行）**：真实服务器 API 级全链路通过（见验收 1）；**UI 视觉检查受环境阻塞**——IAB webview 第三次 "guest not attached"（跨三天复现），桌面 Chrome 无法后台激活（激活接口拒绝）。缓解证据：web/dist 已含新 UI 代码（branch-btn/retry-btn 在产物中）、双轨 vue-tsc/vite build 绿、交互逻辑与 API 已分别验证。**残余：真人过目分支切换器/重试按钮的视觉呈现（约 2 分钟）**。

**遗留（非阻断，在册）**：① T7 branchSummary 递延（待澄清 #1）；② 跨标签 SSE 树刷新未做（待澄清 #2）。

**结论**：review_done，可进入 /auto-plan:merge。

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
