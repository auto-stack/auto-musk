# PLAN-081 验证收据（work 阶段）

日期：2026-09-21 · 代码：musk plan-081-dev @ 9565b48（基 plan-080-dev @ 5899fff）
仪器：后端 17201 复用（musk-080 worktree musk.exe serve）；VM =
`auto.exe run --render vm`（musk-080 worktree auto-lang 二进制），MCP 固定
9251（`AUTOUI_MCP_PORT=9251 AUTOUI_ACCEPTANCE=1`）；启动脚本
`.wt/musk-081/auto-musk/tmp/p081-launch-vm.cmd`（launch-vm.cmd 同款）。
原始探针产物：`tmp/p081-*.json`（主检出 tmp/，未入库）。

## 静态门

| 门 | 结果 |
|---|---|
| `auto build`（080 worktree auto.exe；vue-tsc+vite 全 pipeline） | ✅ 绿（vite 11.05s，"Vue project built successfully!"） |
| `npx vitest run`（web/，npm ci 后） | ✅ 23 passed + 1 skipped（与基线一致） |
| 生成 Vue 产物核对 | ✅ WorkspaceSelector.vue（nameOrPath/wsDisplay/两条链+plansStore.Reload）、useForgeStore.ts（switched+清场+LoadSessionList）、usePlansStore.ts（plans_loaded/Reload）、App.vue（ShowPlans→plansStore.Init()） |
| `wt-guard.sh` | ✅ clean（auto build 的 pnpm node_modules 737 个 junction 已按守卫指引 link-only 清除，清单 tmp/p081-junction-clean.txt） |

## VM 实机（MCP autoui_state/vtree/action）

API 真值对账：backend=22 会话/0 计划；auto-edit=15 会话/1 计划（PLAN-005）；
auto-musk=2 计划。

- **AC-01 触发器显示目录名** ✅：vtree 触发钮可见 text 节点 content=
  `"backend"`（展开 rail）；按钮 label 组合（`可见文本+U+EE03+title`）内
  完整路径仅存 tooltip 位。修改前同位显示 `\\?\D:\autostack\...` 截断。
- **AC-02 切换后会话域刷新** ✅：弹层点选 backend→auto-edit 后 state：
  `workspace: auto-edit`、`session_list: 15 vmref`（=auto-edit 真值，切换前
  22=backend 真值）、`session_id: 9063dfd498088b370bc969c1`（API 确认=
  auto-edit 首会话"@plan/001 实施这个计划"，切换前 95c167e=backend 首
  会话）、`messages: 4 vmref`（会话详情挂载）。
- **AC-03 切换后计划刷新** ✅（store 级）：`plans_loaded: true`、
  `plans: [1 vmref]`、`current: <vmref>`（自动选中首篇=PLAN-005）。
  视图渲染截图未采（见"制约"）。
- **AC-04 后续请求落新 workspace** ✅（机制级）：`chats_list_sessions()`
  无 workspace 参数，15 条结果获取本身即"默认 query 已重注入"实证；
  NewSession 另有显式 `.workspace` 参数（SetWorkspace 已更新）。
- **AC-05 PickFolder**：与 Choose 同链同序（代码级等价），实机按压需
  原生系统对话框，未自动化——转用户目验/review。
- **AC-06 web 零回归**：reload 调用保留于链尾（web 主链不变）；build/
  vitest/产物核对绿；浏览器 E2E 未跑（转 review）。

## 基线（修复前，同仪器）

- Choose(auto-edit) 后：`workspace` 字段切而 `session_id` 恒 95c167e、
  `session_list` 恒 22 条（API 对账=backend 真值）→ 刷新链断裂实锤。
- plans 视图（旧码）`plans: []`：backend 真值 0 篇的歧义读数；新码实机
  boot 进 plans 视图 `plans_loaded=true` → **VM 子视图挂载 Init 实际派发**
  （修正 PLAN-048 T5 口径在本场景的适用面）。用户所报"计划不跟切"根因
  =切换无重拉 + 默认 query 滞留（重进视图重拉也打旧 query）。

## 制约与移交

- **§10-9 新数据点**：ws 弹层 Choose 按压与冻结/死亡家族强相关——本轮
  7 实例中新码 3/4 按压后 MCP 失联（进程活/MCP 死），旧码（080-tip 原样）
  同位复现 1 例 → 非本次引入；080"40+ 导航按压零复现"之外的更窄触发面，
  已转 080 交接清单①。视觉级截图（AC-01/03 收尾）因此未采全。
- VM workspace 选择不跨启动持久（localStorage=进程级会话 KV + resolve
  回退链被数组迭代债打断 → 每次启动回 registry 默认 backend）：080 已
  登记债的复合表现，本轮未动，计划 §10 观察项。
- SD-01/SD-02 spec 增量按流程于 merge 阶段入册。

## r5（需求④⑤：工具卡 "0" 中毒/单行 header + 思考卡 chevron）——2026-09-21

代码：musk plan-081-dev @ 0b70043（基 344dcfb）。仪器同上（MCP 9251 +
后端 17201 复用；auto-lang master release auto.exe）。

### 定罪（工具卡 "0"）

- 症状：重载消息的工具卡 name="0"、参数段 "0"/:0:0/"0" 七连、header 被顶
  高；status（本地合成默认）幸存。getToolSummary 七分支逐一命中
  （path/limit/offset/pattern/query/slug/sectionId/skill_name 全读出 "0"）。
- 取证（[BLKDBG] 临时打印，已撤）：normalizeToolBlocks（handler 上下文）
  内，API 原值与 JSON 往返后的 tool 载荷读数**全程真值**——原生值层数据
  完好、handler 上下文读可靠。
- 反转证据：同代码同会话两实例对照——实例 1 vtree 真值 vs 实例 2 像素全
  "0"（boot 后 18s）→ **渲染/计算上下文对"存储可达嵌套对象"的字段读产出
  "0"，字符串读全上下文可靠**。
- r4 现场验证为何幸存：其时可见卡片走 legacy tool_calls 合成路径；本轮
  T-03 刷新链使 API blocks 路径成为 VM 常态后被命中。

### 修法（musk 侧拍平，两层）

1. normalizeToolBlocks：入 store 前整树 JSON 往返；tool 块重建为纯字符串
   字段（tool_name/tool_id/tool_status/tool_result/tool_gate_id/
   tool_pending_cmd/tool_escape_paths_text/tool_args_json + summary 于
   handler 上下文由内联 summaryTextOf 现算成串——store 文件不消费跨文件
   fn 导入，沿 nowSec 内联先例，改名防 VM 扁平命名空间撞名）。
2. messageBlocks：只读字符串字段 + args_json 本地 JSON.parse 重建
   arguments；live(SSE) 本地字面量块原样透传；legacy 路径不动。

### 布局与 chevron

- header 弃 `for seg in .block.summary`（VM row 内 for 子树被包装为列=
  七段竖排顶高的布局根因），改单 text 节点 `text .block.summary`；
  逐段条件 class 链（auto-lang 债①同族）随之退役，段级配色双轨退役。
- 思考卡 chevron：span 内内联 if/else 被 VM 丢弃（2026-09-03 定案）→
  改工具卡已证形态（直挂 if/else text 节点，ml-auto 靠右）。

### 验证

- 静态：auto build 全 pipeline 绿（grep "Vue project built successfully!"，
  首轮因 gen/front/vue node_modules 残留 d3 版本错配失败，删除重装即愈）
  ；vitest 23+1skip；生成产物三处核对（ChatMessage.vue 单 summary span/
  chevron 对/useForgeStore.ts round-trip+拍平）。
- VM 实机：vtree 全树 `content: "0"` 计数=0；按钮 row 五兄弟单行
  [🔧 run_command ls -la completed ▼] / [🔧 read_file README.md completed
  ▼] / [💭 已思考 · 301 tokens ▼]；像素截图 tmp/p081-r5-chats-fixed.png
  同证；ToolToggleKey 展开链通（tkey #tool:tc-1 稳定），ARGUMENTS/
  RESULT 正文（total 146 = ls -la 输出）在树。
- 打印移除后净码复验一臂：vtree 同口径全绿。

### 移交/登记（r5 新增，见 plan §10）

- VM 渲染/计算上下文嵌套对象读产出 "0"（渲染面债⑤）——处方：ingest
  拍平字符串；errands/relays/task_plans 等他店 ingest 若再现同症按同方
  处理。
- `.messages = <全量重绑>` 赋值后画面滞留旧态的嫌疑（实例 2 boot 18s
  时轮询已换真值而像素仍旧）——与 PLAN-536 重绑定不可见族同疑，未定罪。
- VM row 内文本节点 min-w-0/flex 计量与 web 差异：工具卡参数段在 VM
  呈居中分布（web 紧跟 name），观感可接受，登记不动。
- §10-9 家族延续：r5 轮 3 实例均 ~2min 内死亡（其中 1 例用户交互后
  手动关闭）；截图通道对最小化窗口失败（window size zero）。
