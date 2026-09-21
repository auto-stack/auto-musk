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
