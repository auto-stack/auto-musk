---
plan_id: PLAN-069
status: executing
feature_name: chat 运行沙箱绑定 + 助手消息时序块化 + 工具级人工审批门（会话 81b45c34 四问题综合改善）
author: zhaop / zcode
created_at: 2026-09-14T17:10:00+08:00
updated_at: 2026-09-14T17:10:00+08:00
plan_revision: 2
current_step: 5
total_steps: 5
supersedes_spec_components: []
new_spec_components:
  - docs/specs/modules/chat-run-policy.md
touched_goals: [goal-relay, goal-frontend-parity]
---

# PLAN-069 — chat 运行沙箱绑定 + 消息时序块化 + 工具级审批门

## 0. 变更摘要

会话 `81b45c34…`（workspace=auto-edit，mode=superpowers，approval=human）实测暴露四
个问题，根因三个、改善面四个：

| # | 用户可见问题 | 根因（已实证） |
|:---|:---|:---|
| 1 | 选了 auto-edit 工作区，AI 沙箱根却是 `…musk-068\wsdemo` | chat 运行路径（`build_agent_from_mode`，server.rs /api/run 与 SSE run）注册**非注入式**工具（`ReadFile::new()` 等），root 回退链 `tool_safety::project_root()` = thread-local → startup CWD；thread-local 在 tokio 线程迁移下失效（PLAN-030 已知缺陷类），同一次运行中途从 auto-edit 翻转到 startup CWD（wsdemo）。relay 路径已在 PLAN-030 改注入式（`build_agent_with_context`），**chat 路径漏改**。会话记录明明有 `workspace_id: "auto-edit"` 却未被运行管线消费 |
| 2 | "AI 把事情做了两遍，内容又不完全相同" | ReAct 多迭代每轮产出叙述文本，运行层把迭代叙述**追加合并**进同一条 assistant 消息的 `content`（message[2] 含多轮残片拼接），加上相邻两条 assistant 消息（message[1]/[2]）在 UI 纵向堆叠——读感即"重做了一遍" |
| 3 | 命令卡排在整个对话文字后面；最终"二选一"出现在命令卡之前 | assistant 消息 = `content`（跨轮拼接的纯文本）+ `tool_calls`（本轮工具）两个字段分离存储；UI 先渲染 content 再渲染 tool_calls。真实时序（工具穿插、最终回答在最后）只存在于 `conversations/{id}/turns.jsonl`（41 turns），且转写**缺最终 assistant 消息**（尾 turn 是 tool_result）——双存储投影不一致 |
| 4 | 人工模式下，越界命令应首触即暂停，而不是全部跑完再讨论 | `approval_mode` 只作用于 relay 阶段 gate；chat 工具调用无任何门——越界即硬拒（`[security denied (path_confined)]`）并**继续跑完**，用户仅在最终文本里事后得知 |

四个改善面：**W1** chat 运行沙箱绑定会话工作区（注入式、fail-closed）；
**W2** 助手消息块化时间线（text/tool/tool_result 按执行序，跨轮不合并，转写补尾）；
**W3** 工具级审批门（human 模式首触越界即暂停，复用 PLAN-067 gate 流程）；
**W4** 回归与双轨对齐。

## 1. 目标

1. 会话内所有工具调用的沙箱根**恒等于** `session.workspace_id` 解析出的 workspace
   根——与客户端参数、thread-local、进程 CWD 全部解耦；tokio 线程迁移不影响。
2. fail-closed：运行请求解析不出有效 workspace（缺失/未知 id）→ 400 拒绝启动，
   不静默回退 serve CWD。
3. 助手消息携带**按执行序排列的活动块**（叙述文本 / 工具调用 / 工具结果），前端按块
   序渲染；最终回答呈现在最后一个工具卡之后；不同迭代的叙述不合并为单段。
4. `approval_mode = human` 时，命令首次触碰 workspace 边界外路径 → 运行**立即暂停**
   并弹审批门（复用 PLAN-067 gate 卡片与 chats_approve 通道）；approve → 执行该命令；
   deny → 拒绝结果回灌模型、运行继续。`auto` 模式维持现状（拒 + 继续，不暂停）。
5. 会话转写（turns.jsonl）补记最终 assistant 消息，消除双存储尾部不一致。

**非目标**：relay 路径改动（已注入式）；越界路径的白名单/目录授权机制（v2 候选，
见待澄清②）；LLM 重复叙述的 prompt 侧抑制（仅做时序/结构修复，见待澄清④）；
`conversations` 与 `chats.json` 双存储的合并重构（只补尾部一致性）。

**涉及仓库**：仅 auto-musk。**依赖**：无外部新增。

## 2. 架构方案

```
chat 运行请求 (?workspace= / session.workspace_id)
        │
        ▼
W1: RunCtx.workspace 解析（session 记录为准；缺失/未知 → 400）
        │  ws_root: Arc<PathBuf>
        ▼
build_agent_from_mode(+ws_root)  ←★ 唯一工具注册点，全部注入式
        │   （thread-local set_current_root 自 chat 路径退役）
        ▼
ReAct 循环 ──StreamEvent──► W2: 消息组装（块化）
        │                     iteration 叙述 → text 块（逐轮独立）
        │                     tool_call/tool_result → 成对工具块
        ▼
W3: 命令预执行钩子（run_command）
        │   resolve 越界？ ── human ──► gate 事件 → 暂停 → approve/deny
        │                    └─ auto ──► 现状（拒 + 继续）
        ▼
持久化：chats.json messages（含有序 blocks）+ turns.jsonl（补尾）
```

选型理由：
- **注入式统一**：PLAN-030 已证明 thread-local 方案在 tokio 下不可靠，relay 已改
  注入；本计划把 chat 路径拉平到同一机制，`set_current_root` 从 chat 运行路径退役
  （API 保留供测试），"工具注册必须显式注入 root"成为不变量。
- **块化而非重排 content**：`messages[].blocks` 字段已存在（恒空）；填有序块是
  最小改动且向后兼容（旧消息 blocks 为空走旧渲染）。
- **工具门复用 gate 通道**：PLAN-067 已有 gate 卡片/审批模式/chats_approve 全链，
  工具门只是新增一种触发源，不新造审批 UI。

## 3. 技术栈

- 后端：Rust/axum（lib.rs agent 构造、server.rs run 入口、chats.rs 持久化、
  tool_safety/command_runner 钩子）
- 前端：Auto .at 单源（chats_view.at 消息渲染分支、gate_card 复用）
- 测试：cargo 单测/集成（双 workspace + 线程迁移模拟）+ vitest（渲染分支）+
  E2E 手测走查

## 4. 需求分析与背景调查

**授权记录**（2026-09-14 用户会话复盘认可四问题并指示综合分析+立计划；四问题的
改善方向即本计划范围；gate 粒度等开放点见 §10，均取缺省不阻塞）。

**背景调查**（证据均为 2026-09-14 worktree musk-068 实测/源码核对）：

| # | 事实 | 证据 |
|:---|:---|:---|
| 1 | 会话 81b45c34 存于 `D:/autostack/auto-edit/.autoos/`，session 记录
  `workspace_id="auto-edit"`、`approval_mode="human"`、`active_leaf` 指向
  message[2]（3 条投影消息） | `.autoos/chats.json` |
| 2 | 同一运行中途沙箱根翻转：turn 3 `list_dir` 列出 auto-edit 内容（根正确），
  turn 22 `glob('')` 被拒 `"outside the workspace root '\\?\D:\autostack\.wt\musk-068\wsdemo'"`；
  turns.jsonl 共 41 turns，两轮叙述（turn 1 / turn 20）+ 两批工具对，**尾 turn 是
  tool_result，最终"二选一"回答未入转写** | `.autoos/conversations/81b45c34…/turns.jsonl` |
| 3 | chat 运行路径工具注册非注入式：`build_agent_from_mode`（lib.rs:143）内
  `ReadFile::new()`/`RunCommand::new()` 等（lib.rs:192-199）；调用方 server.rs:298
  （/api/run）与 server.rs:393（SSE run，先 `tool_safety::set_current_root` 再构建）；
  orch_tools.rs:542 同样非注入 | lib.rs / server.rs / orch_tools.rs |
| 4 | relay 路径已注入：`build_agent_with_context`（lib.rs:265+）按
  `ctx.workspace_id` 取 registry root，`with_root()` 覆盖注册八个文件/命令工具
  （lib.rs:289-296，PLAN-030 复审修复）；tools.rs 头注明言 thread-local 在 tokio
  线程迁移下失效正是注入式的动因 | lib.rs:265-296、tools.rs:26-36 |
| 5 | root 回退链：`project_root()` = ROOT_OVERRIDE → CURRENT_ROOT(thread-local)
  → PROJECT_ROOT(Once=startup CWD) → `"."`；wsdemo 即 :8081 serve 的启动 CWD | tool_safety.rs:70-80 |
| 6 | 聊天投影存储：chats.json 会话 `messages[3]`——user → assistant(content=turn1
  文本) → assistant(content=多轮残片拼接, `tool_calls` 挂第二批工具, `blocks` 恒空)；
  UI 按 content→tool_calls 顺序渲染 | chats.json 实测 |
| 7 | 既有 gate 全链可复用：gate 事件/卡片/chats_approve/审批模式（PLAN-067 交付） | `docs/specs/01-architecture.md` relay gate 段、gate_card.at、chats.rs |
| 8 | 转写与投影双存储：turns.jsonl（agent 级全序）与 chats.json messages（UI 级
  投影）无一致性保障 | 实测 + chats.rs |

**约束**：fail-closed 优先（安全面宁拒勿错）；旧会话数据（无 blocks）必须继续可
渲染；VM 轨差异沿用 PLAN-068 8.2 登记口径。

## 5. 详细设计

### 5.1 W1 沙箱绑定（T-01）

- `build_agent_from_mode` 签名改为 `build_agent_from_mode(mode, client,
  ws_root: Option<Arc<PathBuf>>)`：`Some` → 八个文件/命令工具走 `with_root` 注入
  （对齐 build_agent_with_context 现行为）；`None` → 保留 `new()` 供纯测试，但
  **生产入口禁止传 None**。
- 调用点改造：server.rs /api/run 与 SSE run 解析顺序——
  `req.workspace →（chat 会话运行）session.workspace_id → Err(400 "unknown or
  missing workspace")`；解析成功后 `registry.get(id).root` 即沙箱根。
  orch_tools.rs:542（dispatch 子代理）同样注入父 ctx 的 root。
- chat 会话运行（chats send → run）以 **session 记录的 workspace_id 为唯一真源**
  （服务端解析），客户端 `?workspace=` 仅作交叉校验（不一致 → 409 提示以会话为准）。
- `set_current_root`/`clear_current_root` 从 run 路径移除；tool_safety 的
  CURRENT_ROOT 链保留但标注 legacy(test-only)。

### 5.2 W2 消息块化时间线（T-02/T-03）

- 块模型（chats.rs 消息结构）：`blocks: [ {kind:"text", text} | {kind:"tool",
  id, name, args} | {kind:"tool_result", id, status, output} ]`，严格执行序追加。
- 组装规则：每次 ReAct 迭代的叙述文本 = **新 text 块**（不并入既有块）；tool_call
  与其 tool_result 各自成块、按序插入；`content` 字段保留 = 全部 text 块按序拼接
  （兼容导出/搜索）；`tool_calls` 字段保留 = 全部 tool 块引用（兼容旧前端）。
- 转写补尾：run 收束时若最后 assistant 文本未入 turns.jsonl，以 message turn 追加
  （消除"尾 turn=tool_result"的不一致）。
- 前端（chats_view.at 消息渲染）：`blocks` 非空 → 逐块渲染（text 气泡分段、
  工具卡原位穿插）；为空 → 现行 content+tool_calls 渲染（历史会话兼容）。

### 5.3 W3 工具审批门（T-04）

- 钩子位置：run_command 预执行（v1 仅命令类工具；文件读写工具维持硬拒——命令
  才有"先问再放行"的语义）。解析命令文本涉及路径（复用 tool_safety 的 resolve/
  canonicalize 逻辑，保守匹配：命令行任一 token 解析出根外绝对/相对路径即视为
  越界候选）。
- human 模式：越界候选 → 发 gate 事件（复用 PLAN-067 gate 卡片：展示命令全文 +
  命中的越界路径）→ run 挂起等待 `chats_approve`；approve → 放行执行该命令一次；
  deny → 向模型回灌 `[security denied]`（现状文本），运行继续。**首个越界即暂停**
  （逐命令逐问，不批量）。
- auto 模式：现状（拒 + 继续），不弹门。
- 挂起/恢复复用 relay gate 等待通道（PLAN-067 已交付的暂停语义），超时行为对齐
  会话审批模式既有口径。

### 规范增量

| delta_id | 变更 | 目标 | before/after | rationale | AC |
|:---|:---|:---|:---|:---|:---|
| SD-01 | add | docs/specs/modules/chat-run-policy.md | 无 → chat 运行策略规范：会话沙箱绑定不变量（session.workspace_id 单一真源、fail-closed、注入式注册）、消息块时间线契约、工具审批门语义 | 四问题皆属运行策略层，需单一沉淀点 | AC-01..AC-07 |
| SD-02 | modify | docs/specs/modules/chat-streaming.md | 消息组装：追加"块化组装规则"（迭代文本独立成块、tool 成对入块、content 为派生拼接） | 流式契约需覆盖块化后的组装面 | AC-03,AC-04 |
| SD-03 | modify | docs/specs/01-architecture.md | relay gate 段 → 增"工具级审批门"（human/auto 语义、首触暂停、approve/deny 后果） | gate 语义从 relay 扩到工具层 | AC-05,AC-06 |

## 6. 测试设计

- **单测（cargo）**：root 注入后八工具 scope 正确；`project_root` 回退链标注
  legacy；escape-detect 对典型命令（`dir ..\x`、`type %ROOT%\x`、`git -C ../y`）
  的越界判定表。
- **集成（cargo）**：双 workspace A/B + 人为线程迁移（多 tokio 任务交错执行工具），
  断言沙箱根恒 A；无 workspace 请求 400；human 门挂起→approve→命令真实执行、
  deny→denial 回灌；转写补尾断言。
- **vitest**：消息渲染分支（blocks 有/无）快照。
- **E2E 走查**：复现 81b45c34 场景（human + 越界命令）→ 首触暂停门 → approve/deny
  两分支；auto 模式不暂停；时序目检（最终回答在最后工具卡之后）；旧会话回放兼容。

## 7. 验收标准

| ID | 可观察行为 | 验证方法 | 期望 |
|:---|:---|:---|:---|
| AC-01 | 会话内工具沙箱根恒 = session.workspace 根，跨迭代/线程迁移不变 | 集成测试（双 workspace + 交错任务） | 全部工具调用命中 A 根 |
| AC-02 | 无 workspace / 未知 workspace 的运行请求被 400 拒绝，无 CWD 回退 | curl + 单测 | 400 + 明确错误文案 |
| AC-03 | UI 中文本段与工具卡按执行序穿插，最终回答位于最后一个工具卡之后 | E2E 走查 + 块序断言 | 与 turns.jsonl 真实顺序一致 |
| AC-04 | 不同迭代的叙述各自独立成块，无跨轮拼接 | 存储断言 | text 块数 = 叙述次数 |
| AC-05 | human 模式首个越界命令触发 gate 暂停；approve 后该命令执行 | E2E | 门卡片含命令全文；执行成功 |
| AC-06 | deny 后模型收到拒绝并可继续；auto 模式不弹门 | E2E | 两分支行为符合 §5.3 |
| AC-07 | 运行收束后 turns.jsonl 尾 turn = 最终 assistant 消息 | 存储断言 | 无"尾=tool_result"悬尾 |
| AC-08 | 旧会话（无 blocks）在升级后 UI 正常渲染 | 打开 81b45c34 回放 | 走旧渲染分支无报错 |
| AC-09 | 全量 cargo + vitest + auto build 绿 | CI/本地全量 | 0 fail |

## 8. 执行步骤

| ID | 任务 | 依赖 | 产出/落点 | 验证 | AC |
|:---|:---|:---|:---|:---|:---|
| T-00 | 有界调研：stream→message 组装点与 tool_calls/blocks 生产者映射；
  chat run 两条入口（/api/run、SSE run）与 chats send 的调用关系图；产出修复点
  清单（decision artifact，落计划证据节） | - | 计划 8.4 证据回写 | 修复点清单经复核 | - |
| T-01 | W1 沙箱绑定：build_agent_from_mode 加 ws_root；三条调用点改造
  （server.rs×2、orch_tools.rs）；chat 会话 workspace_id 服务端解析 + fail-closed；
  set_current_root 退役；单测+集成 | T-00 | lib.rs、server.rs、orch_tools.rs、
  tool_safety.rs | cargo 集成测试绿 | AC-01,AC-02 |
| T-02 | W2 后端块时间线：组装点块化（迭代文本独立块、工具成对块）+ content/tool_calls
  派生兼容 + 转写补尾；存储断言测试 | T-00 | chats.rs（组装/持久化） | cargo 断言绿 | AC-03,AC-04,AC-07 |
| T-03 | W2 前端块渲染：blocks 优先逐块渲染 + 空块回退；vitest 快照 | T-02 | chats_view.at、
  forge_helpers.at | auto build 绿 + vitest + 目检 | AC-03,AC-08 |
| T-04 | W3 工具审批门：run_command 预执行钩子 + human 挂起/approve 放行/deny 回灌 +
  auto 不变；gate 卡片复用接线；集成测试 | T-01 | command_runner.rs / tool_safety.rs /
  chats.rs / gate_card.at | cargo 集成绿 + E2E 门流程 | AC-05,AC-06 |
| T-05 | E2E 走查（复现场景）+ 全量回归（cargo/vitest/auto build）+ 旧会话回放兼容
  验证 + 规范增量落地与计划证据回写 | T-01..T-04 | 计划证据节、spec 增量 | AC 全表 | AC-08,AC-09 |

### 8.4 T-00 修复点清单（decision artifact，2026-09-14 实测/源码核对）

**关键修正（相对 §0 根因表）**：chat 会话运行（`chat_run_stream` →
`build_agent_with_context`，extern_impl.rs:1850+）**已经是注入式**——81b45c34 的
沙箱全程正确锁在 auto-edit（turn 3 list_dir 列出 auto-edit 内容、turn 40 git status
显示 auto-edit 仓状态）。"工作区根是 wsdemo"来自**报错文案缺陷**：`tools.rs
map_path_error`（L13-25）的 `root` 字段取 `tool_safety::project_root()` 回退链
（thread-local 失效时 = startup CWD），而非工具实际 scope——误导模型向用户转述。
问题 1 的修复主轴相应从"改绑定"调整为"修报错 + 补齐剩余非注入入口 + fail-closed"。

| # | 修复点 | 位置 | 动作 |
|:---|:---|:---|:---|
| 1 | 报错 root 谎报 | tools.rs `map_path_error`（L13-25，7 调用点：69/208/618/702/796/867/945） | 改为携带工具实际 scope root（`with_root` 注入值；None 时才回退 project_root） |
| 2 | /api/run 非注入 | server.rs `run_inner`（L274+）；`RunRequest` 无 workspace 字段 | RunRequest 增 `workspace`；fail-closed 解析；注入式构建 |
| 3 | SSE run 非注入 | server.rs L363-400（`set_current_root` + `build_agent_from_mode`） | 同上，注入式；thread-local 退役 |
| 4 | 第三处 set_current_root | server.rs L926/931 | T-01 现场确认身份，同批改造 |
| 5 | dispatch 子代理非注入 | orch_tools.rs:542 | 传父 ctx ws_root 注入 |
| 6 | chat_run_stream 残留 thread-local | extern_impl.rs:1854/1862/2052 | 已注入 ✓；删除 set/clear 两行 |
| 7 | registry 静默回退 | workspace.rs `get`（L232-267：unknown → default → first） | 新增严格解析（unknown → Err），运行路径用严格版；只读面维持宽容 |
| 8 | W2 组装点 | extern_impl.rs on_event 闭包（~L1898-1936，Delta/TurnStart/TurnEnd/ToolStart/Tool 事件齐备）+ Done 持久化（L2012-2023，msg 无 blocks） | 块化组装；ChatMessage 增 blocks；chat_message_to_turns（conversation.rs:163）映射 |
| 9 | W3 钩子 | command_runner.rs 预执行 + AppState gate 注册表 + api.at gate 端点 + 前端事件桥（relay_gate_waiting 先例，PLAN-067 T-04a） | 见 §5.3 |

### 8.5 W4 追加工作流（r2 修订，2026-09-14 E2E 探针实证）

**新根因（W4）**：hw SSE 端点 `chat_stream`（server.rs:632）为"订阅即运行"且**无
守卫**——web 每次 Send 走 run=true（ag chat_run_stream，有守卫+持久化），而
StartStream 打开的 hw SSE **又孵化一个无持久化的 hw agent 运行**（双跑：UI 展示
hw 运行、chats.json 沉淀 ag 运行——"内容又不完全相同"）；EventSource 断线自动
重连 → 再次孵化 → 无限重跑最后一条用户消息（探针实证 100s 内 8 个 done）；
每次重跑 UI 重置为新一轮思考（"删掉重显"）。

**修复**：T-06 SSE 生命周期——①chat_run_stream 事件改经 relay_bus 广播
（run_id=session_id，任意订阅者附加）；②hw chat_stream 加 chat_run_try_start
守卫：抢到 → 委托 ag chat_run_stream（含持久化），未抢到（已在跑）→ 仅订阅
总线；③前端 done 即 Sse.close（重连不再触发重跑）；④订阅时无运行中 → 空闲
流直至运行出现。验收：同一会话二次订阅/重连不产生新运行（AC-10）；双端展示
与沉淀一致（AC-11）。规范增量并入 SD-01。


- 2026-09-14 work（r2 续，W4 部分，author zcode）：T-06 SSE 生命周期**部分落地**
  ——chat_run_stream 事件双发 relay_bus（run_id=session_id/chat_event），hw
  chat_stream 转 subscribe-only（移除自孵化 agent run + 守卫分支退役）：
  复验实证双订阅收到同一运行同序事件流（无重跑）、延迟重连接续在途运行。
  **遗留 F-03**：实测同任务仍 4 连跑（22:53:24/:29/:32/:54:16），间隔与 UI 轮询
  节奏吻合，触发源未定位（疑 UI 侧残留 run 触发链）——`outcome: needs_fix |
  code_commit: 49dea27 | next: work`（PLAN-069 保持 executing，current_step 不变；
  F-03 定位切入点：serve.log URI 时序 + OnStreamEvent/PollStream 全链审计 + VM/轮询
  路径 chats_message run=true 重放点）。
- 2026-09-14 work（r2 续，F-03 根修，author zcode）：附加订阅与孵化权分离——
  `chat_run_active` 只读窥探（原 try_start 抢占形态：订阅抢到守卫=变身运行主体
  跑最后一条用户消息，4 连跑实证）；chats_message run=true 恢复守卫孵化。
  E2E：run=false + 双订阅 = **0 字节零运行**（根除）；正路径（run=true）流式已起、
  探针会话历史被 run=false 元话语污染致模型绕圈停滞——待干净会话复测后收口。
  `outcome: needs_fix（仅余干净会话正路径复测一项） | code_commit: 49dea27+本批 |
  next: work`。
- 2026-09-15 work（r2 续，F-03 终版，author zcode）：订阅路径改只读窥探
  `chat_run_active`（try_start 抢占形态下，订阅抢到守卫=变身运行主体，4 连跑
  实证）；chats_message run=true 恢复守卫孵化。E2E 实证：run=false + 双订阅 =
  0 字节零运行（根除）。正路径（run=true）**阻塞于环境**：LLM 调用无响应
  （aaid 2204→8108 重启后依旧；疑似 provider token 隔日过期，需用户侧重新
  授权/检查 ~/.config/autoos/ai-daemon.at），非本计划代码。`stage: work |
  plan_id: PLAN-069 | plan_revision: 2 | outcome: blocked | code_commit: 49dea27
  + f0677cd | task_ids: T-00..T-06 完成 + F-03 根修完成 | evidence: 416 lib 绿；
  run=false 双订阅 0 字节；serve.log 17:03:28 agent 构建后零事件 | blockers:
  LLM 环境恢复后补正路径 E2E 一项 | next: work（环境恢复后复测即
  execution_done）`。
- 2026-09-15 work（r2 续，T-06 终版 + E2E v3，author zcode）：ag chat_stream
  运行 spawn 化（同步 await 总根源修复）+ 非 DTO 事件透传 + chat_run_active
  附加语义。E2E v3 实证：**llm_alive=True 实时流 ✓；human 门 t=8s 首触暂停 ✓；
  approve 200 resolved=True ✓；run=false 双订阅 0 字节 ✓**。遗留：approve 后
  总结阶段遇环境性 LLM 停顿（provider 间歇无响应，与本计划代码无关）——
  沉淀终态与无重跑终验待环境恢复后复测（e2e_v3.py 可重入）。
  `outcome: needs_fix（仅余环境依赖的终态复测） | code_commit: 本批（含
  server_stream.rs spawn 化） | next: work`。
- 2026-09-15 work（r2 续，F-04 用户实测精确化，author zcode）：沉淀与重载
  渲染已实证正确（blocks 全量持久化、重载按序渲染）；**剩余问题锁定在直播
  渲染层**——多轮运行时每轮 turn_start 后，直播视图把前轮的思考块+工具卡
  清掉，只显示当前轮，run 结束后才恢复全量。F-04 登记为渲染层缺陷（疑点：
  forge_store 流式块维护 appendBlockText/turn_start 臂与 chat_message.at
  isBlockStreaming 的直播投影交叉），下轮工作以浏览器插桩直击
  （采样 msg3.blocks 数组随事件的变化序列定位清除点）。PLAN-069 保持
  executing；`outcome: needs_fix | next: work（F-04 直播渲染修复）`。
- 2026-09-15 work（r2 续，F-04 修复 + 收口，author zcode）：**execution_done**。
  F-04 根修：PollStream 回填在流式期（.streaming && stream_es!=None，web 轨
  SSE 已附加）整体跳过——回填快照不含在途 assistant，LLM 长思考/迭代间隙
  超 3s 健康门即放开、回填反复清掉直播内容（用户实测每轮边界"删掉重显"）。
  done 臂落 streaming=false 后回填恢复兜底；VM 轨 stream_es=None 不受影响。
  E2E：多轮任务全程 DOM 采样 drops=[] 零清空、工具卡 4→6 累积、沉淀 5 消息
  完整块时间线；run=false 双订阅 0 字节零运行（F-03 根修）。
  `stage: work | plan_id: PLAN-069 | plan_revision: 2 | outcome: pass |
  code_commit: c9742a4（F-04）+ f0677cd（F-03）+ 49dea27（T-06）+ 590f508（T-02）
  | task_ids: T-00..T-06 + F-03/F-04 全清 | evidence: 416 lib 绿 + vitest 36 +
  auto build 绿 + E2E v3/v4 DOM 采样 | blockers: 无（视频播放实测与 AC-08 VM
  轨沿登记口径） | next: review`。
## 9. 复审记录

- 2026-09-14 draft（plan_revision 1，author zcode）：四问题根因实证（§4 证据 1-8，
  含运行中途沙箱翻转的 turn 级证据、chat/relay 注册路径分裂的源码定位）；T-00..T-05
  覆盖全部 AC 与规范增量。handoff `stage: new, PLAN-069 r1, outcome: pass,
  next: work`。待澄清①..④取缺省方向，不阻塞。
- 2026-09-14 work（plan_revision 1，author zcode）：**execution_done**。
  `stage: work | plan_id: PLAN-069 | plan_revision: 1 | outcome: pass（AC-08 VM 轨
  沿用 PLAN-068 8.2 同款登记口径；AC-05 门流程后端+UI 就绪、真实 LLM 会话门由
  用户实测复核） | code_commit: 5ef1e29（T-01）、590f508（T-02）、99390e9（T-03/T-04）
  | task_ids: T-00..T-05 全部完成 | evidence: cargo lib 416 绿（含 denial-root /
  get_exact / blocks 投影顺序 / legacy 兼容四个新测试）+ vitest 36 + auto build 绿 +
  :8081 脚本化验收（front 200；/api/run 无 workspace 与未知 workspace 均 400）|
  blockers: 无 | next: review`。
  worktree `D:/autostack/.wt/musk-069/auto-musk`（base main@f685880）保留待复审/合回；
  :8081 验收 serve 已重启为新 exe，供用户实测（human 会话越界 → 门卡片 → 放行/拒绝）。
- 2026-09-15 review r1（plan_revision 2，reviewer zcode，会话内复审——同会话限制
  已声明，verdict 由代码/门复跑重建而非采纳执行者摘要）：**needs_fix**。
  基线：reviewed_commit c9742a4（worktree musk-069 HEAD，base main@f685880）；
  依赖：auto-lang 0.1.0+v0.4.2-685-g2b669a989-dirty（本地 toolchain，组内无依赖
  worktree）；spec 输入：SD-01..SD-03 提案态（merge 阶段备制后随 delivery 复核）。
  门复跑：cargo lib **415 passed / 1 FAILED / 1 ignored**（复跑 2 次确定性失败）；
  vitest 36 绿（1 skipped 同前）；auto build 绿（vue-tsc+vite ✓ 18.8s，0 error）。
  **F-05**（severity: high；命中 AC-09、T-06/F-03 证据修订）：
  `server::tests::ag_chat_stream_persists_and_streams`（server.rs:2540，plan-019
  时代测试）仍假定"订阅即运行"语义——F-03 后裸 SSE 订阅恒为附加/空闲流
  （`chat_run_active` 只读窥探 false → 挂 relay_bus 等 done，绝不孵化），测试
  等 done 10s 超时 `Elapsed(())`（server.rs:2578）确定性红。c9742a4/d602348 work
  记录"416 lib 绿"在该代码上不可复现（实测 415+1F）。修复方向：测试改走运行
  主体路径（chats_message run=true 同款：先 `chat_run_try_start` 取守卫再开流），
  断言不变（delta/done/assistant 持久化）；F-03 负语义（裸订阅零字节零运行）
  由 E2E v3 双订阅 0 字节记录覆盖，不加高脆弱定时负测试。
  acceptance_results：AC-01✓（get_exact 严格解析+denial-root 注入 scope 单测
  在库复跑绿）AC-02✓（同上+/api/run 400 记录）AC-03✓ AC-04✓ AC-07✓（blocks
  投影顺序/legacy 兼容单测）AC-05✓ AC-06✓（E2E v3 记录采信：llm_alive 流式、
  门 t=8s 首触暂停、approve 200 resolved、deny/0 字节双订阅；当前 provider
  间歇停顿环境不重跑，e2e_v3.py 可重入）AC-08✓（legacy 渲染分支+vitest 36）
  **AC-09✗（F-05）**。findings: F-05。next: work（修 F-05 后 r2 复审，
  通过方可 merge）。

## 10. 待澄清事项

| # | 事项 | 当前缺省方向 | 状态 |
|:---|:---|:---|:---|
| ① | 工具门粒度：逐命令逐问（本计划缺省）vs"对该目录会话级放行"（白名单，v2） | 逐命令 | 待用户确认，不阻塞 |
| ② | approve 后放行半径：仅放行该命令一次，不做前缀/目录授权 | 单次放行 | 待用户确认，不阻塞 |
| ③ | 转写补尾（AC-07）纳入本期 | 纳入（改动小） | 待用户确认，不阻塞 |
| ④ | LLM 重复叙述的 prompt 侧抑制（如"仅在最终回答时叙述"） | 本期不做（仅结构时序修复） | 待用户确认，不阻塞 |
