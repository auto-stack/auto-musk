---
plan_id: PLAN-058
status: execution_done
feature_name: 会话删除两步确认护栏 + Block 全家福会话重灌（+二期：主导航纵排与标题边框）
author: [zhaopuming]
created_at: 2026-09-03T10:00:00+08:00
updated_at: 2026-09-03T12:00:00+08:00

supersedes_spec_components: []
new_spec_components: []
touched_goals: []

current_step: 15
total_steps: 15
---

# [PLAN-058] 会话删除两步确认护栏 + Block 全家福会话重灌

## 变更摘要

用户实机对拍（VM 版 vs vue 版）发现两类问题，本计划一并收敛：

1. **误删护栏**：VM 轨二级导航会话列表 hover 不显示删除 icon，但点击会话卡片
   下半部分会直接删掉会话（零确认），极易误触。要求：hover 显示删除 icon，
   点击 icon 后弹确认，确认后才删除。VM 轨无 CSS/hover/图标，需双轨等价的
   两步删除设计。
2. **数据修复**：Block 全家福测试会话在用户视角已被误删，要求补回一个完整的。

**二期（2026-09-03 用户追加，gen 截图实测）**：

3. **主导航标题下边框**：截图上"Auto Musk v0.1.0"标题栏没有下边框。实测
   边框**存在且已与二级导航标题对齐**（均 y=48 处 1px solid），不可见的真因
   是 `--border`(亮度 18%) 与 rail 的 `bg-secondary`(16%) 仅差 2 个亮度点，
   视觉上等于没有（bg-card 是 10%，所以二级导航那条可见）。修复=提高该线
   对比度（border-white/10），对齐天然成立无需改高度。
4. **主导航底部纵排**：WorkspaceSelector 与 SettingsMenu 齿轮横向并排
   （justify-between）撑宽主导航。改为上下排列：工程目录在上 → 分割线 →
   设置行（齿轮 icon + "设置" 文字，与上方导航项 icon+文字 形态一致）。

## 目标

1. VM 轨会话列表项上不再存在"隐形可点"的删除热区——删除 affordance 必须
   可见（"×" 文本字形，沿 ThinkBlock chevron "▲/▼" 文本字形先例）。
2. 双轨（VM + gen/web）所有删除类操作（单会话删除、全部删除）一律两步：
   点删除钮 → 应用内确认条 → 确认才调 `chats_delete_session` /
   `chats_delete_all_sessions`，取消或误点其他区域不删。
3. gen/web 轨既有 hover 显隐行为保持不变（`inject_styles.web-only.ts`
   `.session-delete-btn` 三规则不动）。
4. Block 全家福会话在主检出工作区（backend/.autoos）重灌完整内容：
   文本块（Markdown 全谱）/ 思考块 / 工具卡×6（含 spawn_relay + report）/
   Run 窗口 / 报告卡 / 问卷 JSON，双轨可打开对拍。

## 架构方案

**不改后端**（`chats_delete_session` / `chats_delete_all_sessions` API 已在，
纯前端交互改造），**不改生成器**（auto-lang 不动）。改动集中在 `.at` 单源：

- **视图态状态机**（`src/front/chats_view.at`）：沿 `var chat_search str = ""`
  （:73）先例加 `var delete_pending_id str = ""`；空串=无待确认，值为会话 id
  或 `"__all__"`（全部删除待确认）。VM 对 str 字段 SET 无障碍（chat_search
  同型已双轨实证）。
- **消息与处理器**：新增 `.AskDelete(id)` / `.AskDeleteAll` / `.CancelDelete`
  / `.ConfirmDelete(id)` / `.ConfirmDeleteAll`；`.DeleteSession` 处理器体
  （:310 `chats_delete_session(id); store.LoadSessionList()`）平移进
  `.ConfirmDelete`。`SelectSession` 处理器顺带清 pending（点别的卡片 = 取消）。
- **删除 affordance 单源字形化**：删除钮 `span` 内的 `Trash2 { size: 12 }`
  是 `use.web` 组件——VM 轨代码生成直接丢弃，span 内容为空 = 隐形；VM 又不
  消费 `absolute` 定位类，span 退化为流内排布，恰好落在卡片下半部 → 用户报
  的"点下半张卡直接删"。改为文本字形 `text "×"`（web 轨视觉从 trash SVG 变
  为字形，登记视觉变更；VM 轨从隐形变可见）。web 轨 hover 显隐仍由
  `inject_styles` 的 `.session-delete-btn` 类钩子承载，CSS 零改动。
- **确认条**：`.delete_pending_id == .s.id` 时在卡片内渲染确认行
  （"确认删除？" + 确认/取消两个 span，`onclick.stop` 防冒泡到卡片
  SelectSession），类串用 tailwind 工具类（沿 PLAN-049 T6 会话壳迁内联先例，
  不新增自定义 CSS 类）；VM 轨无样式但语义完整（in-flow 可读可点）。
- **DeleteAllSessions 同护栏**：头部垃圾桶钮同样改为两步（pending=`__all__`，
  确认条渲染在列表顶部），它比单删更危险且同属"隐形可点"类。
- **数据重灌**：沿 `scripts/seed_blocks.py` 头注两相位法——停 serve 落盘 →
  起 serve 后 `--api` 注入 relay run（RunStore 不落盘）。

## 技术栈

- 源：Auto `.at`（`src/front/chats_view.at`，可能含 `src/front/i18n/`）
- 生成：auto-lang `auto build --gen-only` → `gen/front/vue/`
- 门禁：`vm-safe-lint.mjs`（五模式零新增红）、gen `pnpm build`（vue-tsc+vite）、
  `vitest`（基线 23+1skip）
- 实机：`musk serve`（worktree 隔离 :9093）+ `auto run --render=vm` +
  vite dev（:3342）+ AutoUI MCP snapshot / 截图
- 数据：`scripts/seed_blocks.py`（主检出 backend/.autoos 工作区）

## 需求分析与背景调查

### 根因链（实机现象 → 源侧实证）

| 现象 | 源侧事实 |
|---|---|
| VM hover 无删除 icon | 删除钮内容 `Trash2` 为 `use.web` 组件（web-only），VM 生成物无图标内容；VM 无 CSS，`opacity-60`/类串全部无效 |
| 点卡片下半部直接删 | span 类串 `absolute right-[6px] top-1/2` 是 tailwind 类——VM 不消费，span 流内排布进 msg-count 行（卡片下半部）；空 span 仍带 `onclick.stop: .DeleteSession(.s.id)` 可点 |
| 删除无确认 | `.DeleteSession` 处理器（chats_view.at:310）直接 `chats_delete_session(id)` |
| gen 轨 hover 显隐正常 | `inject_styles.web-only.ts:105-107` 三规则（display:none → hover display:flex → hover opacity）只覆盖 web 轨 |

- `nav_item.at` 的 `NavListItem` 只 import 未实例化（死代码，本计划不动它，
  避免扩散 diff）。
- 头部 `DeleteAllSessions`（chats_view.at:110 附近）同为 `Trash2` web-only
  内容 + 直删，VM 里同属"隐形可点"，一并纳入护栏。

### 数据现状（2026-09-03 勘察）

- 主检出工作区 `backend/.autoos/chats.json` 现存 14 会话，其中
  `block-showcase-chat`（"Block 全家福演示"，3 条消息）**在库**——用户认知中
  "被删"的全家福是 PLAN-057 实机对拍用的 a1 会话，其隔离工作区
  （plan057-demo）随 worktree 清理已不存在。不论删没删错，按用户要求重灌一个
  完整的（seed 脚本固定 id `block-showcase-chat`，重跑即整卡重写为全量 6+ 消息
  版本并刷新 updated_at 置顶）。
- VM 问卷卡渲染受 057 账本 pre-existing 缺陷（`__json_object` 字符串字段读
  污染）阻断，属已知残差——全家福在 VM 轨的问卷卡预期不渲染，不作本计划
  验收项（gen 轨正常）。

### 账本钩子（specs 账本相关行）

- 057 行「VM 实机合成输入守卫」：VM 内自动化点击受限，交互验证以 AutoUI MCP
  snapshot + 用户实机目验结合。
- 055 T16 先例：VM 轨观感偏差登记模式（无 hover 态 → 常显 affordance）。
- 11b6c20 注记：`auto build` 再生成会抹 `gen/front/vue` 的 package.json
  devDeps——vitest 复跑前需会话级补装（vitest@2.x，vite5 兼容）。
- a3491b5 注记：worktree 内构建需 `.worktrees` junction。

## 详细设计

### 删除两步确认状态机（chats_view.at）

```
状态: var delete_pending_id str = ""

. AskDelete(id)        -> { .delete_pending_id = id }
. AskDeleteAll         -> { .delete_pending_id = "__all__" }
. CancelDelete         -> { .delete_pending_id = "" }
. ConfirmDelete(id)    -> { chats_delete_session(id); .delete_pending_id = "";
                            store.LoadSessionList() }
. ConfirmDeleteAll     -> { chats_delete_all_sessions(); .delete_pending_id = "";
                            store.session_id = ""; store.messages = [];
                            store.LoadSessionList() }
. SelectSession(id)    -> { .delete_pending_id = ""; store.SwitchSession(...) }  // 顺带清 pending
```

视图侧（会话卡片内，msg-count 行之后）：

- 删除 span：`onclick.stop` 由 `.DeleteSession(.s.id)` 改 `.AskDelete(.s.id)`；
  内容 `Trash2 { size: 12 }` → `text "×"`；类串保留 `session-delete-btn`（web
  hover 钩子）+ 原工具类，补 `cursor-pointer`。
- 确认条（`if .delete_pending_id == .s.id`）：`row` 内
  `text t("chat.confirmDeleteSession")` + 确认 span（`onclick.stop:
  .ConfirmDelete(.s.id)`，text-destructive）+ 取消 span（`onclick.stop:
  .CancelDelete`）。此时隐藏 "×"（`if .delete_pending_id != .s.id` 包住）。
- 全部删除确认条（`if .delete_pending_id == "__all__"`）：渲染在列表 col 顶部，
  同构（文案 `chat.confirmDeleteAllSessions`）。
- 头部垃圾桶钮 `onclick` 由 `.DeleteAllSessions` 改 `.AskDeleteAll`。

### i18n

新增键（若缺）：`chat.confirmDeleteSession`（确认删除此会话？）、
`chat.confirmDeleteAllSessions`（确认删除全部会话？）、`chat.confirm`（删除）、
`chat.cancel`（取消）。中英两份语言文件同步；如有 catalog 生成脚本
（`scripts/gen-i18n-catalog.mjs`）则重跑。

### 数据重灌（主检出）

```
1. 停主检出 serve（:9247）
2. python scripts/seed_blocks.py D:\autostack\auto-musk\backend      # 落盘
3. 重启 musk serve :9247
4. python scripts/seed_blocks.py D:\autostack\auto-musk\backend --api http://127.0.0.1:9247
   # RunStore 内存注入（脚本自述两相位法）
5. curl /api/chats/sessions 验证 block-showcase-chat 置顶且消息数齐全
```

## 测试设计

- **静态门禁**（worktree 内）：
  - `auto build --gen-only` 退出 0；
  - `node scripts/vm-safe-lint.mjs` 五模式零新增红（新增 `.at` 代码过五模式）；
  - `cd gen/front/vue && pnpm build`（vue-tsc + vite）绿；
  - `npx vitest run` 基线 23+1skip（devDeps 被抹则先 `pnpm add -D vitest@2`）。
- **VM 实机**（worktree serve :9093 + seed + `auto run --render=vm`）：
  - 列表项可见 "×" 字形（AutoUI MCP snapshot 断言删除钮文本节点存在）；
  - 点击卡片非 "×" 区域 → 不触发删除（会话仍在列表）；
  - 点 "×" → 确认条出现 → 取消 → 会话仍在；确认 → 会话消失；
  - 全部删除钮 → 确认条 → 取消不动。
- **gen 实机**（worktree vite :3342，同 :9093 后端）：
  - hover 会话项显示 "×"，移开隐藏；
  - 点 "×" → 确认条 → 取消/确认行为同 VM；
  - 截图证据入 `docs/attachments/`（沿 057 惯例 p058- 前缀）。
- **数据**：主检出 `/api/chats/sessions` 列表含 block-showcase-chat 且
  VM/gen 双轨打开渲染正常（VM 问卷卡残差按账本登记豁免）。

## 验收标准

1. VM 轨会话卡片上不存在隐形可点删除区；删除 affordance（"×"）在 VM 可见。
2. 单会话删除与全部删除在双轨都必须经确认条二次确认；取消/误点他处不删。
3. gen/web 轨 hover 显隐行为与改前一致（inject_styles 零改动）。
4. 四项静态门禁全绿且基线不劣化（vm-safe-lint 零新增、vitest 23+1skip）。
5. 主检出工作区存在内容完整的 Block 全家福会话，双轨可打开对拍。
6. VM 问卷卡不渲染属 057 账本已知残差，不计失败。

## 执行步骤

- [✅ 已完成] worktree 建（a3491b5）；junction 整目录式成环改按条目式
  （.worktrees/auto-{lang,ai} → 依赖根），验证 `ls` 双条目可见。
- [✅ 已完成] 两相位重灌：chats.json 合并（14 会话保留，block-showcase-chat
  整卡重写）+ conversations/run-block-showcase/meta.json 26 turns + report.html；
  serve 重启后 --api 注入 run 200；API 列表含 showcase（u1 + a1(tool_calls:7)
  + a2 问卷共 3 消息=seed 全量设计，原计划"≥6"系起草期误估，据此更正）。
- [✅ 已完成] 状态机落地（worktree 6ec904b）：`delete_pending_id` 视图态 +
  Ask*/Confirm*/CancelDelete 消息组 + 卡片内确认条（两个单臂 if）+ 删除 span
  改 text "×"（Trash2 为 use.web,VM 空内容=隐形可点区根源）+
  SelectSession 清 pending；gen 产物 ChatsView.vue 含 delete_pending_id ×10。
- [✅ 已完成] 全部删除同护栏：头部垃圾桶钮改 `.AskDeleteAll` + `×` 字形 +
  列表顶部 `__all__` 确认条（复用既有键 chat.confirmDeleteAll）。
- [✅ 已完成] i18n 三键（confirmDeleteSession/confirmDelete/cancel）入中英
  chat 区 + catalog 重跑；**顺带修复漂移**：addItem/noDoc/openSystemSettings
  三键只在 i18n.at 不在 json（regen 即丢），补回 json 再生，键集对 main 零丢失。
- [✅ 已完成] 四门禁绿（键名修复后复跑 lint/vitest 同绿）：auto build
  --gen-only exit 0（54 组件）/vm-safe-lint PASS（0 命中+3 既有豁免）/
  pnpm build（vue-tsc+vite）绿/vitest 23+1skip 基线。注：worktree 新 gen 需自主
  检出补拷非生成物 components/ui/；vitest@2 会话级补装（11b6c20 注记在案）。
- [✅ 已完成] VM 实机六断言全过（AutoUI MCP :9273 autoui_action=press 驱动；
  拓扑偏离：直接桥主检出 :9247——本计划零后端改动，省冗余构建，偏离登记
  待澄清⑦）：A1 ×字形可见/A2 确认条/A3 取消恢复+pending 清空/A4 全删确认条/
  A5 全删取消/A6 确认删除 pending 复位。**实机抓错并修复**：确认钮误写
  t("chat.confirm")（json 键名 confirmDelete）致 VM 裸键名渲染，改对齐后
  A6 过。证据 docs/attachments/p058-vm-confirm-strip.png（MCP 实拍）。
- [✅ 已完成] gen 实机（worktree vite :3342 → :9247）：hover 显隐 CSS 实证
  （a11y 树 display:none 时 × 不可见，hover 后 display=flex）→ 点 × 出确认条
  （i18n 正确）→ 取消恢复 → 确认删除后 API 总数 15→14（一次性会话从服务端
  消失）。Block 全家福演示在列表可见。证据
  docs/attachments/p058-gen-confirm-strip.png。偏离注：UI 默认进 musk-demo
  空工作区，经工作区选择器切 backend 后 15 会话全显（待澄清⑤）。
- [✅ 已完成] 状态推进 execution_done；scoped 复验（lint+vitest）绿；残差与
  偏差入待澄清④-⑧；交接 /auto-plan:review（merge 时主检出需重跑
  auto build --gen-only + pnpm install 使 :3334 生效）。
- [✅ 已完成] 标题下边框对比度：border-b 换 border-white/10（实测
  border-border 与 bg-secondary 仅差 2 亮度点不可见）；h-12 不动。gen 实测
  borderBottomColor=rgba(255,255,255,.1) @y=48，与二级导航标题线对齐。
- [✅ 已完成] 底部纵排：WorkspaceSelector 在上 → 分割线（SettingsMenu 根容器
  border-t border-white/10）→ 设置行（齿轮+「设置」全宽 36px 行,.settings-trigger
  样式块同步改,面板上弹锚点不变）。gen 实测：设置钮全宽 167px、分割线居间、
  面板开合正常；VM 快照纵排结构成立。证据 p058-gen-rail-phase2/
  p058-gen-settings-panel.png + p058-vm-phase2-snapshot.txt。
- [✅ 已完成] 重生成+门禁+双轨验证+收尾：auto build 绿（54 组件）/
  vm-safe-lint PASS/vitest 23+1skip/pnpm build 绿；gen 浏览器实测 Block
  全家福全块型渲染（思考块/表格/代码块/工具卡×5/Run 窗口/报告卡/问卷）。
  **执行期抓错**：style{} 块内 CSS 注释致 auto build 静默失败（exit 1 零
  诊断,二分定位），去注释即绿——登记待澄清⑨。
- [✅ 已完成] **三期 T13（用户追加）删除确认迁移 AutoUI alert-dialog**：
  web 轨新增 components/DeleteConfirmDialog.vue 薄适配（ui/alert-dialog
  标准件组合,脚手架取自 auto-man assets 铺入 gen ui/alert-dialog；reka-ui
  已在依赖,补 @vueuse/core）。受控 open 需 state_ref——新增
  delete_confirm_open bool 字段（字面量/表达式致整树退化 div,vue.rs:10863
  实证）。VM 轨 iced:none 实证整树丢弃 + 跨 widget msg 派发不通（带参/无参
  均断,055 子件缺陷族同源）→ 确认行必须内联 chats_view（T3 形态回归）,
  web 侧内联行经 .session-delete-strip{display:none} 抑制（055-T16 CSS
  轨道开关机制）。
- [✅ 已完成] **三期 T14 双轨验证**：浏览器实测模态弹出（role=alertdialog,
  标题正确）/取消关闭/确认删除 API 9→8/全删对话框标题正确且取消无损;
  VM 六断言全过（A1 ×字形/A2 确认行/A3 取消复位/A4 全删确认行/A5 取消/
  A6 确认删除后 LoadSessionList 刷新）。
- [✅ 已完成] **三期 T15 收尾**：门禁四绿复跑（auto build/lint/vitest/
  pnpm build）;状态 execution_done;merge 清单入待澄清⑪。

## 复审记录

（/auto-plan:review 填写）

## 待澄清事项

1. **web 轨删除图标从 trash SVG 统一为 "×" 字形**：单源双轨一致优先，视觉
   略变（登记）；若复审要求 web 保留 SVG，需引入 icons.vm.at 平台分叉，成本
   高于收益，倾向不改。
2. **"弹出 prompt" 的形态**：采用应用内确认条而非原生 `window.confirm`——
   VM 轨无窗体 API，原生 confirm 只覆盖 web 轨，破坏单源；确认条语义等价
   （明确告知 + 二次确认）。
3. Block 全家福会话 id 沿用 `block-showcase-chat`（覆盖重写，置顶）；如需
   保留旧 3 消息版可改 seed 的 CHAT_ID 再跑——倾向覆盖，旧内容无保留价值。
4. **（执行期发现）AutoUI MCP 服务寿命 ~60-90s**：随 `auto run` 启动器进程
   一起消失，VM 本体继续存活；AUTOUI_MCP_PORT=9273 生效但无法长活，长验证需
   启动后立刻批量驱动。auto-lang 侧现象，建议独立立项排查。
5. **（执行期发现）UI 默认工作区是 musk-demo（空）**：用户 15 会话在 backend
   工作区，VM/gen 初启均只显示（或创建）musk-demo 的本地会话——VM 里那张
   "New chat" 幽灵卡（session_id 服务端不存在）即此语义，非 bug；对比时需经
   工作区选择器切 backend。主检出 serve 数据完好：block-showcase-chat 等
   14+1 会话全程未被误删。
6. **（执行期发现）nav_item.at 的 NavListItem 是死代码**（chats_view 仅
   import 未实例化），其 hidden 类变体与本计划无关；清理候选，未动。
7. **（执行偏离）T7 未起 :9093 隔离 serve**：直接桥主检出 :9247——本计划
   零后端改动、契约同源，省一次 musk 全量 debug 构建；如需隔离复现可按原
   计划拓扑重跑。
8. **（执行期发现）OS 级合成输入被 VM 守卫拒绝**（057 账本在案）：鼠标
   click 与键盘 type 均被拦，AutoUI MCP 的 widget 级 autoui_action(press)/
   autoui_type 可用——实机自动化验收应走 MCP 路线。
9. **（二期执行期抓错）style{} 块内 CSS 注释致 auto build 静默失败**：在
   settings_menu.at 的 style 块加 `/* ... */` 注释后 build exit 1 且零诊断
   输出（二分定位确认，去除即绿）。auto-lang 债务：style 块注释应支持或
   报错，不应静默。同批排除项：独立空 div 分割线嫌疑未独立证实（与注释
   同批存在）；nav-item 同形的 icon+span 触发钮无问题。
10. **（三期执行期发现）VM 轨跨 widget msg 派发不通**：端口分轨 widget
   （delete_confirm.vm.at）内联确认行的 onclick → 子 msg → 父 onconfirm/
   oncancel 链路在 VM 实机全程静默（带参/无参均断,MCP ActionResult ok 但
   父 handler 不执行）。与 055"VM 子件上下文三缺陷"同源——**跨 widget
   交互在 VM 必须内联或走 store**,候选升级 auto-lang 账本。 → 已立项 PLAN-059（VM 悬浮层基础设施,含该前置修复与 musk 确认
   alert-dialog 化）。
11. **（三期）merge 后主检出生效清单**：①`auto build --gen-only` 重生成;
    ②`gen/front/vue` 补拷 `src/components/ui/alert-dialog/`（脚手架源:
    auto-lang crates/auto-man/assets/shadcn-ui/alert-dialog）与
    components/DeleteConfirmDialog.vue（随 git 合入,ext 镜像随 build 同步）;
    ③`pnpm add @vueuse/core`（alert-dialog 脚手架依赖,package.json 属生成物
    不入库,每次重生成后需补）;④vitest@2 会话级补装;⑤重启 :3334。

