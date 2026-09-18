# chat Agent 身份标识（Chat Agent Identity）模块规范

> PLAN-071 r3 落地预备（2026-09-17）。用户裁定：auto-forge 的 agent 身份设计
> （名字/职业头像）在 auto-musk 不可简化掉。本规范约束"谁在回答"的可观察身份。

## 身份目录（单一真源）

- 职业目录 = `~/.config/autoos/professions.json`（assistant / advisor /
  architect / planner / tester / coder / reviewer / documenter / gofer +
  super-advisor / super-coder / super-tester），经
  `GET /api/forge/relay/professions` 暴露（id + name）。
- 会话身份 = session.mode 的**生效 role**（`modes/*.at` 的 `role` 字段，
  如 superpowers → "assistant"、basic → "coder"；ModeRegistry 解析含回退）。

## 消息身份契约

- `ChatMessage.profession_id: Option<String>` + `Turn.profession_id`（双存储
  同步），skip-if-none 序列化——旧消息（无字段）wire 形状不变，前端回退
  旧渲染。
- 仅 assistant 落盘点填充；user / system / tool 恒 None。
- 填点（穷举，新增 assistant 落盘点必须遵循）：
  1. **chat 运行主体**（chat_run_owner 持久化）：会话生效 mode → role。
  2. **plan-merge 短路**消息：同会话身份。
  3. **relay 报告回写**（relay_append_report_message_to）：发起会话身份。
- turns 投影（chat_message_to_turns）：Message 主 turn / blocks text turn
  透传身份；ToolCall / ToolResult turn 恒 None。
- ag 镜像（auto_generated 瘦 ChatMessage）侧 chat_message_to_turns 恒 None，
  与镜像"仅承载 parity 所需字段"哲学一致（沿 F-03 后先例，手写侧为生产真源）。

## 前端渲染契约

- 助手消息头部（assistant 臂）：`AgentAvatar`（职业 HSL 色 + 首字母，
  未知 id 哈希取色）+ 职业显示名；无身份回退既有 "🤖 AI" 徽章。
- 职业显示名 = `agentDisplayName(id)` 自 id 推导（"super-coder" →
  "Super Coder"，与目录命名一致）。**不走 AgentConfigs store 查名**：vue
  codegen 仅支持 `store` 单别名绑定，跨 store prop 下传链在 ChatsView 不可用
  （r3 实测 vue-tsc 绑定缺失）。目录名动态显示留后续（KNOWN-DEBT）。
- 流式中的乐观消息（done 前无回填）暂无身份，回填后被持久化消息替换（随
  PLAN-069 直播优先/回填语义）。

## 测试口径

cargo lib：运行主体持久化消息 profession_id = mode role（basic → "coder"）、
turns 主 turn 同步、wire skip-if-none 兼容（parity_chats/conversation 双侧）；
vm-link-probe：.at 全量链接 PASS（体积 WARN 线下）；auto build：vue-tsc
strict + vite build 绿。
