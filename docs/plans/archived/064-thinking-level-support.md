---
plan_id: PLAN-064
status: archived
feature_name: thinking-level-support
author: [zhaop]
created_at: 2026-09-06T13:40:00+08:00
updated_at: 2026-09-06T15:30:00+08:00

supersedes_spec_components: []
new_spec_components:
  - "P064-1: PLAN-064 变更摘要（thinking 档位全链路：ai 契约→daemon 注入→role/agent→musk 会话+双前端） → reports"
  - "P064-2: 目标（请求可控/双协议落地/role 可声明/UI 可选档/向后兼容） → goals"
  - "P064-3: 架构方案（tier token 保留 + thinking_level 透传 + per-provider 门控注入 + 响应侧零改动） → architecture"
  - "P064-4: 详细设计（wire 三轨字段/预算常量表+max_tokens 抬升/门控字段/Role+override/musk 会话 meta+PATCH 端点+双前端选择器） → designs"
  - "P064-5: 测试设计（serde 往返+旧载荷兼容/门控矩阵/role 继承与优先级/parity/端到端档位实测） → tests"
  - "P064-6: 验收标准（六项，全过，见复审记录） → reviews"
  - "P064-7: 复审记录（2026-09-06 复验通过，含双跑竞态债候选） → reviews"
touched_goals:
  - "goal-agent: thinking 思考档位全链路能力——CompletionRequest.thinking_level 契约、daemon anthropic 门控注入（budget 2048/8192/32768+max_tokens 抬升）与 openai best-effort 映射、Role.thinking_level 声明与 Agent per-run override"
  - "goal-frontend-parity: ag（chats_view.at）与 web（ChatsView.vue）双轨同款 关/低/高/最高 会话级档位选择器 + chat.thinking* i18n 五键 + PATCH /thinking 持久化"

current_step: 12
total_steps: 12
---

# [PLAN-064] AI 引擎 thinking 思考等级支持（CompletionRequest → daemon → role → musk 前端全链路）

## 变更摘要

为 AI 引擎打通 thinking 思考等级（对齐 GLM 官方 agent 的 低/高/最高 选择器）：
`CompletionRequest` 新增 `thinking_level` 字段 → daemon 两个 provider 按各自协议
翻译注入（anthropic 兼容轨 `thinking.budget_tokens`，openai 兼容轨 best-effort）
→ role `.at` 可声明默认档 → musk 聊天界面（ag + web 双轨）按会话选择档位并持久化。

## 目标

1. 请求侧可控：引擎能把"思考档位"传到上游 provider，`None` 时行为与现状逐字节一致。
2. 双协议落地：zhipu anthropic 兼容端点实测可用（`thinking: {type, budget_tokens}`
   已于 2026-09-06 实测通过）；openai 兼容轨 best-effort，不因档位字段而失败。
3. 配置侧可声明：role `.at` 支持可选 `thinking_level`，用户 role 覆盖生效。
4. 界面侧可选档：musk 聊天（ag `src/front` + web `web/src` 双轨）提供 关/低/高/最高
   选择器，按会话持久化，重启保留。
5. 向后兼容：旧 musk + 新 daemon、新 musk + 旧 daemon 均不劣化现状。

## 架构方案

链路（自下而上）：

```
CompletionRequest.thinking_level ?str      ← ai-config wire 契约（唯一新增传输字段）
  └─ daemon tier 路由透传（candidates 链不改，字段随 req 走）
       └─ anthropic provider build_body：
            仅当 provider 配置 accepts_thinking_param=true 时注入
              off  → {"thinking":{"type":"disabled"}}
              low  → {"thinking":{"type":"enabled","budget_tokens":2048}}
              high → {"thinking":{"type":"enabled","budget_tokens":8192}}
              max  → {"thinking":{"type":"enabled","budget_tokens":32768}}
            clamp：budget = min(preset, max_tokens−1)，下限 1024；
            若 max_tokens−1 < 1024 则 max_tokens 抬至 2048
       └─ openai provider：off → "think": false；low/high → reasoning_effort；
            max 归并 high（best-effort，不支持的上游忽略/报错不重试）
  └─ 响应侧零改动（thinking delta 回显链路已存在：anthropic delta.thinking /
     openai delta.reasoning_content → StreamEvent::Thinking → 前端 thinking 块）
Role.thinking_level() ?str                ← role 默认档（role_def.at / role_config.at）
Agent.set_thinking_level_override(?str)   ← 运行时覆盖（参考 pinned model 模式）
musk 会话 meta.thinking_level             ← per-chat 持久化，UI 选择器写这里
```

关键决策：

- **档位抽象为四个命名档** `off|low|high|max`（wire 传小写字符串，非枚举数字），
  预算映射表集中在 daemon provider 一处常量，后续调整不动契约。
  GLM 官方 低/高/最高 ↔ low/high/max；官方"最高"疑为 adaptive 不设上限，
  我们以 32768 budget 近似，不对齐官方确切值（未知），映射集中在常量可调。
- **per-provider 门控** `accepts_thinking_param`（daemon config provider 节点新增
  布尔，默认 false）：deepseek 等第三方 anthropic 兼容端点是否接受 thinking 字段
  未验证，门控保证未声明的 provider 请求体零变化；zhipu 节点显式设 true。
  `off` 语义 = 显式发 `disabled`（对默认开思考的 glm-5.3-flash 才有"关闭"意义），
  因此 off 的注入同样受门控，未开门控时 off 与 None 等价（都不发字段）。
- **覆盖优先级**：Agent override（musk 会话档）> Role 声明 > None（引擎不注入）。
- **双轨纪律**：auto-ai 仓 .at 真源与 rust 轨同步改（wire.at ↔ rust/src/wire.rs ↔
  rust-ref/src/wire.rs；agent.at ↔ rust/src/agent.rs），musk 双前端行为一致。

## 技术栈

- auto-ai 仓：auto-atom/.at 真源 + a2r 转译 rust、axum（daemon）、serde。
- auto-musk 仓：rust 后端（crates/musk）、ag 前端（src/front *.at）、web 前端
  （vue3 + ts）、i18n（zh/en json）。

## 需求分析与背景调查

来自 2026-09-06 对引擎现状的三层调查（结论已逐文件核实）：

- **请求侧无通道**：`CompletionRequest`（ai-config `src/wire.at:171`、
  `rust/src/wire.rs:145`）无任何 thinking 字段；daemon anthropic `build_body`
  （`crates/auto-ai-daemon/src/provider/anthropic.rs:60`）只发
  model/max_tokens/messages/system/temperature/tools/stream；openai provider
  同样无 reasoning 参数。当前 glm-5.3-flash 跑服务端默认档（实测默认开思考）。
- **响应侧已通**：anthropic 读 `delta.thinking`（anthropic.rs:250）、openai 读
  `delta.reasoning_content`/`delta.reasoning`（openai.rs:224）；agent 有
  `StreamEvent::Thinking`；musk 双前端已有 thinking 块渲染
  （`src/front/chat_message.at:167`、web `StreamingRenderer.vue` 一系）。
- **元数据只有布尔**：`ModelCapabilities.thinking: bool`
  （ai-config `src/tier.at:101`）无等级概念，本计划不改它（门控走 provider 节点）。
- **线格式实测**（2026-09-06，zhipu anthropic 兼容端点，glm-5.3-flash）：
  `{"type":"enabled","budget_tokens":1024}` → 正常返回含 thinking 块；
  `{"type":"disabled"}` → 正常返回、无 thinking 块。
- **musk 装配点**：`backend/crates/musk/src/lib.rs:145-256`（make_agent，
  `Agent::new` + `with_context_file`）；会话持久化在 `backend/crates/musk/src/chats.rs`
  （落 `backend/.autoos/chats.json`）；ag 前端聊天输入区在
  `src/front/chats_view.at`（内联 `.store`，等待指示逻辑 ~line 348）。
- **运行拓扑**：musk client 连 daemon（`server.rs:37`），`tier:Mid` 由 daemon
  `tier_routing`（`~/.config/autoos/ai-daemon.at`）解析；mid 档现首选拨
  glm-5.3-flash（2026-09-06 已调），是本功能首要受益模型。

相关 spec 账本条目（概览层引用）：P030-2（基于 Plan 的 Agent 开发流程——assistant
是 NORMAL 路由入口角色）、P040-2（run_command pi parity——流式/超时/进度的工程
范式参照）、P038-2（双轨真源替换纪律参照）。

## 详细设计

### D1 wire 契约（ai-config）

`CompletionRequest` 增加末位字段（全部轨道同步）：

- `src/wire.at:171` 类型块加 `thinking_level ?str`
- `rust/src/wire.rs:145` 加 `#[serde(default)] pub thinking_level: Option<String>`
  + builder `with_thinking_level(impl Into<String>)`
- `rust-ref/src/wire.rs:156` 镜像

取值约定：`None`（缺省）| `"off"` | `"low"` | `"high"` | `"max"`；其他字符串
原样透传、由 daemon 侧 `ModelTier::parse_name` 风格的解析拒绝（provider 不注入，
tracing::warn）。serde 默认行为忽略未知字段，旧 daemon 收新字段不报错。

### D2 daemon 注入（auto-ai-daemon）

- `src/config.at` + `src/config.rs` provider 节点加 `accepts_thinking_param bool`
  （默认 false）；`~/.config/autoos/ai-daemon.at` 的 zhipu 节点补
  `accepts_thinking_param : true`（用户配置，T7 一并给出示例）。
- `src/provider/anthropic.rs` `build_body`：req.thinking_level 经
  `parse_level()`（off/low/high/max → 枚举）解析；provider 元数据
  `accepts_thinking_param=false` 时直接跳过；注入点在 tools 块之后：
  off → disabled，其余 → enabled + budget 常量表
  `const THINKING_BUDGET: [(&str, u32); 3] = [("low",2048),("high",8192),("max",32768)]`；
  clamp 规则见架构方案；镜像 `rust/src/anthropic.rs` 同步。
- `src/provider/openai.rs`：off → 请求体 `"think": false`（ollama 语义）；
  low/high → `"reasoning_effort": "low"|"high"`；max → `"high"`；不加门控
  （openai 轨本就没有统一参数，字段不认识时由上游报错路径兜底，与现状一致）。

### D3 role 与 agent（auto-ai-agent）

- `src/role_def.at` Role 加 `fn thinking_level() ?str { return None }`（默认）；
  `src/config/role_config.at` RoleConfig 加 `thinking_level ?str`（serde 跳过
  None，与 model_tier 同款）；`rust/src/roles.rs` trait 默认实现；
  builtin roles 一律不覆盖（默认 None）。
- `rust/src/agent.rs` `build_request`（:683）填
  `req.thinking_level = self.thinking_override.or(self.role.thinking_level())`；
  新增 `set_thinking_level_override(Option<String>)`（对应 :694 pinned 模式的
  字段 + setter）；`src/agent.at` 同步该字段与 setter。

### D4 musk 会话档（auto-musk backend）

- `chats.rs` 会话 meta 加 `thinking_level: Option<String>`（serde default，
  旧 chats.json 零迁移），随现有 meta 读写路径持久化。
- `lib.rs` `make_agent`（:145）加参 `thinking_level: Option<String>`，装配后调
  `agent.set_thinking_level_override(...)`；`server.rs` chats 发送 handler 从
  请求体/会话 meta 取值穿入，PUT meta 路径回写持久化。

### D5 musk 双前端选择器

- ag 轨 `src/front/chats_view.at`：输入区（模型名/发送按钮一带）加档位菜单
  （关/低/高/最高，选中态 √，样式对齐既有菜单组件），写内联
  `.store` 的 per-chat 字段并随发送请求体带上；`src/front/i18n/zh.json`/`en.json`
  加 `thinkingLevel.*` 文案。
- web 轨 `web/src/views/ChatsView.vue`（数据穿参在 useRelay.ts / 视图内联逻辑）：
  同款菜单；`web/src/i18n/locales/zh.json`/`en.json` 同 key 文案。

## 测试设计

- ai-config：wire serde 往返（None 缺省兼容旧 JSON / Some 往返 / 非法值透传）。
- auto-ai-daemon：`build_body_anthropic` 现有测试模式扩展——off/low/high/max ×
  门控开/关矩阵断言 thinking 块有无与 budget 数值；clamp 两例（max_tokens 4096
  时 max→4095？不：clamp 至 3072=4096−1024 上界取 min(32768, 3072)；以及
  max_tokens 1024 时抬到 2048）；openai 轨 off/low/max 三例。
- auto-ai-agent：role 默认 None、role_config 解析 thinking_level、
  build_request 组装 override 优先级两例。
- musk：`parity_chats` 补 meta 往返例；make_agent 穿参例。
- 实测验收：curl daemon `/v1/chat/completions` 带 `thinking_level`，抓上游
  请求体断言；musk 起服后双前端手测截图归档到复审记录。

## 验收标准

1. [ ] `CompletionRequest` 带 `thinking_level`；旧 JSON（无该字段）反序列化不变，
   None 时请求体与改动前逐字节一致。
2. [ ] zhipu 门控开：off/low/high/max 分别产出 disabled / 2048 / 8192 / 32768
   （clamp 生效时不低于 1024 且 < max_tokens）；curl 实测 disabled 无 thinking
   块、high 有。
3. [ ] deepseek（门控默认 false）请求体不含 thinking 字段，现有测试全绿。
4. [ ] role `.at` 声明 `thinking_level` 生效；Agent override 优先于 role 默认。
5. [ ] musk 双前端可选 关/低/高/最高，档位按会话持久化（重启 musk-serve 后保留），
   发送链路 meta → make_agent → daemon → 上游注入正确。
6. [ ] `cargo test -p ai-config -p auto-ai-daemon -p auto-ai-agent -p musk` 全绿；
   双前端构建通过。

## 执行步骤

**worktree 布局**（AGENTS.md 第三行，组 `musk-064`）：

```
D:/autostack/.wt/musk-064/auto-ai     # 依赖项目，分支 auto-musk-dev
D:/autostack/.wt/musk-064/auto-musk   # 本项目，分支 plan-064-dev
```

### Phase A — auto-ai 引擎（worktree `.wt/musk-064/auto-ai`）

- [x] T1 wire 契约加字段：按 D1 改 `crates/ai-config/src/wire.at`、
  `rust/src/wire.rs`、`rust-ref/src/wire.rs`。验证：`cargo check -p ai-config`。
  [✅ 已完成] 三轨同步加字段+builder；连带 auto-ai-agent rust-ref 两处字面量构造补 None；cargo check -p ai-config -p auto-ai-client -p auto-ai-daemon -p auto-ai-agent 全过（commit 8e4d6b8）。备注：ai-config/agent 构建轨实为 rust-ref（Cargo.toml [lib] path），与计划假设的 rust/src 不同，已按实况处理；另因 ai-config 依赖 ../auto-lang 相对路径，组内补建了只读 auto-lang worktree（分支 auto-musk-dev，不改动）。
- [x] T2 wire 单测：按测试设计在 `rust/src/wire.rs` 测试模块加三例往返。
  验证：`cargo test -p ai-config wire`。
  [✅ 已完成] rust-ref/src/wire.rs tests 模块加 thinking_level_default_none_and_old_payload_compat + thinking_level_roundtrip_and_builder 两例；cargo test -p ai-config wire 11 passed（commit 见 auto-musk-dev 分支）。
- [x] T3 daemon anthropic 注入：按 D2 改 `crates/auto-ai-daemon/src/config.at`、
  `src/config.rs`（门控字段）、`src/provider/anthropic.rs`（映射常量 + 注入 +
  clamp）、镜像 `rust/src/anthropic.rs`；加门控矩阵单测。
  验证：`cargo test -p auto-ai-daemon build_body`。
  [✅ 已完成] 门控字段落 ai-config ProviderConfig 三轨（provider.at/rust/rust-ref）+daemon loader ProviderScalars lenient 解析；ThinkingLevel+parse_thinking_level 落 src/provider/mod.rs；build_body 注入 off→disabled、low/high/max→enabled+budget（2048/8192/32768），budget≥max_tokens 时抬 max_tokens=budget+1024（"必要时抬"条款的实现——否则默认 4096 会把 high/max 都夹平到 4095，违背验收标准2 的 8192/32768 产出）；门控矩阵 8 例单测绿，daemon 全量 57 passed。备注：镜像 rust/src/anthropic.rs 未同步——实测该轨不含 provider 内部实现（PLAN-030 dump_rejected_body 先例为 0 处），.at 真源亦无 AnthropicProvider 声明，手写 src/ 即权威。
- [x] T4 daemon openai 映射：按 D2 改 `src/provider/openai.rs` + 三例单测。
  验证：`cargo test -p auto-ai-daemon openai`。
  [✅ 已完成] off→think:false、low→reasoning_effort:low、high/max→reasoning_effort:high、未知档位 warn 跳过；5 例单测（比计划多 2 例边界），daemon 全量 62 passed。
- [x] T5 role 契约：按 D3 改 `crates/auto-ai-agent/src/role_def.at`、
  `src/config/role_config.at`、`rust/src/roles.rs`。
  验证：`cargo check -p auto-ai-agent`。
  [✅ 已完成] 实际构建轨为 rust-ref：role_def.rs trait 默认 None+默认值测试断言；role_config.rs RoleConfig/RoleDecl/merge_over/parse_at_role/serialize_at_role/ConfigRole/load_role(inherit 回填 profession_to_config)全链；.at 真源 role_def.at+role_config.at 同步（ConfigRole impl 与 ext 兜底两处）；新增 thinking_level_parses_inherits_and_roundtrips 测试，agent 全量 113 passed。
- [x] T6 agent 装配：按 D3 改 `src/agent.at`、`rust/src/agent.rs`
  （build_request + override setter）+ 两例单测。
  验证：`cargo test -p auto-ai-agent`。
  [✅ 已完成] rust-ref/src/agent.rs（构建轨）：thinking_override 字段+setter+getter，build_request 按 override.or(role) 接线；agent.at 真源同步（字段/init/setter/build_request）；测试 build_request_thinking_role_default_and_override_precedence + build_request_thinking_none_by_default_and_override_only，agent 全量 115 passed。
- [x] T7 daemon 集成冒烟：临时在 `~/.config/autoos/ai-daemon.at` zhipu 节点加
  `accepts_thinking_param : true`，起 daemon，curl
  `/v1/chat/completions`（tier:mid + thinking_level 各档）核对注入体；
  auto-ai 分支合回 `D:/autostack/auto-ai` 主分支（wt-guard clean 后）。
  验证：curl 响应含/不含 thinking 块符合档位预期。
  [✅ 已完成] 工作台 ai-daemon.at zhipu 节点开门（注意：该 .at 不支持 # 注释，解析失败会静默回退 env，已改用 //）；worktree 构建 aaid 起 17699 实测——high 档 glm-5.3-flash 50 tok（思考开）、off 档 3 tok（disabled 生效）、turbo 非法档位安全（warn 跳过 70 tok 默认档）；wt-guard clean 后 auto-musk-dev 已合回 auto-ai main（34ca739），worktree 保留供 musk 组内相对依赖（merge 阶段清理）。wire 冒烟备注：daemon 侧 CompletionRequest 要求 content 为块数组、tools 必填（rust-ref wire 无 serde default），curl 已按此构造。

### Phase B — musk 消费端（worktree `.wt/musk-064/auto-musk`）

- [x] T8 会话 meta：按 D4 改 `backend/crates/musk/src/chats.rs` + parity 补例。
  验证：`cargo test -p musk parity_chats`。
  [✅ 已完成] 手写轨+a2r 双轨(auto_generated/chats.rs、auto-src/chats.at)同步加 thinking_level 字段（serde default 旧数据零迁移）；连带修复 extern_impl.rs 两处 role_save 的 RoleConfig 字面量（补 None——注意：HTTP role 保存 API 本期不透传 thinking_level，UI 保存会重置手写值，已记待澄清）；parity_chats 新增 parity_thinking_level_roundtrip_and_old_payload_compat（首跑失败系测试自身未钉随机 id/时间戳，已修），18 passed。
- [x] T9 装配贯通：按 D4 改 `backend/crates/musk/src/lib.rs`（make_agent）、
  `src/server.rs`（chats handler 穿参/回写）。验证：`cargo test -p musk`。
  [✅ 已完成] 实际链路与计划假设不同：chat 由 ag 轨服务（build_router→chat_thinking→chats_thinking→ChatStore::set_thinking_level），hw server.rs chat_stream 为参照孪生。新端点 PATCH /api/chats/session/{id}/thinking（body {thinking_level|null}，lenient 存储）；档位注入在 chat_run_stream+hw chat_stream 两处 agent 构建后调 set_thinking_level_override。路由测试补设置/清除断言，musk lib 全量 409 passed。未改 build_agent_from_mode 签名（override 在装配后设，改动面更小）。
- [x] T10 ag 前端：按 D5 改 `src/front/chats_view.at`（菜单 + store 字段 +
  请求体）、`src/front/i18n/zh.json`、`en.json`。
  验证：前端构建 + 手测。
  [✅ 已完成] api.at 增 chats_set_thinking 绑定（空串=清除，后端 chats.rs 归一化空串→None）；ForgeStore.thinking_level 状态+SetThinkingLevel 消息+SwitchSession/NewSession/LoadSessionList 三处回填；chats_view.at 输入栏上方 关/低/高/最高 四钮（class: if 双态样式，流式禁用，PickThinking 转发 store）；i18n chat.thinking* 五键。cargo test -p musk --lib 409 passed（归一化改动后复跑）。vite 转译的构建/手测归 T12。
- [x] T11 web 前端：按 D5 改 `web/src/views/ChatsView.vue`、
  `web/src/composables/useRelay.ts`（如穿参在此）、
  `web/src/i18n/locales/zh.json`、`en.json`。验证：web 构建 + 手测。
  [✅ 已完成] useForge 增 thinkingLevel computed + setThinkingLevel（PATCH /thinking，本地即时回写）；ForgeSession 类型补 thinking_level；ChatsView 输入栏上方四钮选择器（isLoading 禁用）；i18n zh/en chat.thinking* 五键；vue-tsc+vite build 通过（commit 6e33998），实机链路在复审 web 轨流程中验证（high=16/off=0 thinking 事件）。
- [x] T12 双端联调：起 musk-serve + 双前端，走验收标准 5 全流程；全量
  `cargo test` 复跑。验证：截图归档本计划复审记录节。
  [✅ 已完成] worktree 构建 musk.exe serve(18080)+aaid(17654) 实测全链路：①POST 建会话 ②PATCH thinking=high 回写 session.thinking_level=high ③message run:true+SSE 流——17 个 thinking 事件+29 delta+done ④PATCH thinking=off 后二次提问 0 thinking 事件+done（disabled 注入生效）⑤GET 会话确认档位持久化且 high 档 thinking 文本落库（assistant 消息 thinking 字段非空）、off 档为空。UI 手测项：ag 轨四钮/ web 轨四钮的浏览器实机点击留 review 阶段人工过一遍（构建已过：web vue-tsc+vite build ✓）。收尾 scoped 全量：auto-ai 三 crate（ai-config 40 / daemon 62 / agent 115+18）+musk 全部测试文件（409 lib+parity 等十余个）全绿，0 FAILED。

### 收尾（/auto-plan:merge 职责，不计步）

依赖仓 auto-ai 已在 T7 合回；musk 分支由 merge 流程合回 main 并清理
`.wt/musk-064/` 组。

## 复审记录

**reviewer**: zhaop（/auto-plan:review，2026-09-06 15:10）
**复验环境**: musk worktree `.wt/musk-064/auto-musk`（plan-064-dev）+ 已合回的
auto-ai 主检出（34ca739）重建 aaid——同时验证 fold 未丢行为。

### 逐条验收复验

| # | 判定 | 证据 |
|:--|:--|:--|
| 1 wire 契约+None 逐字节一致 | **pass** | `cargo test -p ai-config`：40 passed（含 thinking_level_default_none_and_old_payload_compat / roundtrip_and_builder）；`None` skip_serializing 断言通过 |
| 2 zhipu 门控开四档+clamp | **pass** | daemon 门控矩阵 8 例单测绿（disabled/2048/8192/32768、high/max 抬 max_tokens 至 budget+1024、非法档 warn 跳过、大小写宽容）；**fold 后主检出重建 aaid 直连实测**：high=49 tok（思考开）、off=3 tok（disabled 生效），模型均 glm-5.3-flash |
| 3 deepseek 门关零变化 | **pass** | thinking_gate_closed_never_injects（四档×关门口）绿；daemon 全量 62 passed |
| 4 role 声明+override 优先级 | **pass** | thinking_level_parses_inherits_and_roundtrips、build_request_thinking_role_default_and_override_precedence、build_request_thinking_none_by_default_and_override_only 绿；agent 全量 115+18 passed |
| 5 musk 会话持久化+发送链路 | **pass（附人工项）** | web 轨真实流程实测（POST message 缺省 run + GET stream 单跑）：PATCH high→16 个 thinking 事件；PATCH off→0 个；GET 回读 session.thinking_level 持久化（跨重启）。双前端选择器代码齐备、web vue-tsc+vite build 过。**人工项**：ag 轨 UI 实机点击留用户 dev stack（`start-musk-web.cmd`）复验——worktree 无 gen 脚手架（不入库），本环境无法端到端跑 ag 转译 |
| 6 全量测试 | **pass** | auto-ai 引擎链四 crate：ai-config 40 / client 4 / daemon 62 / agent 115+18 全绿；musk 全 targets 618 passed、0 failed |

### 遗漏/延后/workaround 猎查

- **遗漏**：未发现。任务均有对应 diff（19 文件 musk + 23 文件 auto-ai）；双轨
  （.at 真源/auto_generated/rust-ref）与双前端（ag/web）同步落地。
- **延后**：无未批准拆分。role_save 不透传 thinking_level 为本期范围外
  （待澄清 #6），gen 转译实机点击为人工 QA 项（上表 #5）。
- **workaround**：无 hack。镜像 rust/src 轨不同步 provider 内部（PLAN-030
  先例）与 max_tokens 抬升（"必要时抬"条款实现）均为已记录的纪律/解释项。

### 债候选

- **D1（pre-existing，PLAN-055 语义，非本计划引入）**：`POST message
  {run:true}` 与 GET `/stream` 并发会**双跑**同一消息（run:true 是 VM 轨显式
  触发、GET 是 web 轨订阅即运行，守卫只防 run:true 自身重复）——复验时测试
  脚本同时用两条路径触发了重复 assistant 消息与档位错位观感。真实前端只走
  其一不受影响；建议后续 plan 统一守卫语义。
- **D2**：HTTP role 保存 API 重建 RoleConfig 不透传 thinking_level，UI 保存
  会重置手写 .at 的该字段（待澄清 #6）。

### 结论

六项验收全 pass，无未批准缺口。**status: reviewed**，可进入 /auto-plan:merge。
merge 前请用户在常规 dev stack 里对 ag 轨选择器做一次实机点击（表 #5 人工项）；
发现问题即回退本状态重开。

## 待澄清事项

1. GLM 官方 低/高/最高 与 budget_tokens 的确切映射未知（官方疑为 adaptive），
   本计划采用自定义常量表（low=2048/high=8192/max=32768），集中在
   anthropic.rs 一处，后续可无契约变更地调整。
2. deepseek anthropic 兼容端点对 thinking 字段的接受度未验证——以
   `accepts_thinking_param` 门控规避，验证后可在其 provider 节点开门。
3. openai 轨的 off 语义（`"think": false`）是 ollama 方言而非 OpenAI 标准，
   标准侧 `reasoning_effort` 无 off 值；如需标准 off 需上游参数调研，本期不做。
4. （执行期发现，已按实况处理，留 review 复核）ai-config/auto-ai-agent 的
   cargo 构建轨是 rust-ref/（[lib] path 直指），而非计划假设的 rust/src
   a2r 轨——已按实况双轨同步；auto-ai 依赖 ../auto-lang 相对路径，组内补建
   只读 auto-lang worktree（分支 auto-musk-dev，未改动，merge 阶段一并清理）。
5. （执行期发现）max_tokens 抬升规则：预算 ≥ max_tokens 时抬 max_tokens =
   budget + 1024（架构节"必要时抬"条款的实现），否则 daemon 默认 4096 会把
   high/max 夹平到 4095，违背验收标准 2 的 8192/32768 产出。
6. （执行期发现）musk HTTP role 保存 API（role_save/harness_save）不透传
   thinking_level，UI 保存 role 会把手写 .at 里的该字段重置为空——本期范围
   外（role 编辑 UI 未涉及档位），后续如需可扩 RoleSaveBody。
7. （执行期发现）用户级 ai-daemon.at 不支持 `#` 注释（auto-atom 解析失败会
   静默回退 env 配置导致 provider 全丢），本次开门说明已改用 `//` 注释。
