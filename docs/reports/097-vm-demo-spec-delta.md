# 097 — VM 演示证据与规范增量（SD-01～SD-04 可审文本）

计划 PLAN-097 的规范增量准备稿（work 产出，供独立 review 绑定版本；
canonical Spec 的实际沉淀属 merge 阶段——本文不提前改 `docs/specs/**`）。

- 基线：worktree plan-097-dev 终态 861525f（rebase 后 c999fa7，revision 2
  复审绑定 blob e0176663；F-4 更正——本头原为 v1 待审态占位）
- 消费合同冻结：PLAN-096 交付面（baseline §T-02，hash 绑定）
- 真机证据：`docs/reports/097-vm-demo-evidence.md`（T-13 另文）+ run 目录
  manifest（D:/autostack/.demo/musk-097/<run_id>/evidence-manifest.json）
- 修订 v2（2026-10-12 用户裁定）：下表 AC 索引原 v1 的「T-12 运行中」等状态
  为 v1 历史判读；v2 收口判读=三轮 delivered 交付面延期 v0.6
  （KNOWN-DEBT 097-D1/D2），v0.5 证据面以已交付管线+五跑矩阵为准
  （plan §7「v2 验收范围裁定」为权威）

---

## SD-01 modify — docs/specs/modules/app-studio-ui.md

**ID**: SD-01 | **类型**: modify | **关联 AC**: AC-03, AC-04, AC-15

**Before（现行规则）**：
- Canvas 生成/启动/验证进度摘要的行数据经 `canvas_helpers.at` 的
  computed（use.web.fn）计算——登记 G-15：VM 轨该计算恒返空，摘要整块
  隐藏（仅 Vue 可见）。
- Canvas 工具事实（运行/完成/失败/lint/未验证）与计划进度无区分规则。

**After（修订规则）**：
1. **G-15 消除（已实证）**：进度行数据由 `CanvasStore` 在 Poll 拍内
   handler 域预计算（`cp_prog_rows/cp_prog_has`，try 界定），摘要组件
   `CanvasProgressSummary` 自持 store（own-store 读），视图仅做
   for-源列表 + 循环局部成员读 + prop 直渲——零 computed、零模板插值、
   零成员链判言。VM 与 Vue 双轨同义可见。
2. **VM 轨渲染合同（T-11 真机实证的新边界规则）**：
   - 文本位的 store 成员读（模板插值 `${store.x}` 与 `text store.x` 直读）
     被 VM 渲染器丢弃——须走 handler 域预构建列表 + for 局部读；
   - computed 内 `.length` 成员链恒空——布尔化后在 handler 域落账；
   - 单 widget 多个顶层 `on` 块只有一块生效（CanvasProgressSummary 双块
     实证：Init 被丢弃）——每 widget 一个 on 块；
   - 模块 fn 进模板须 `use.web` 形态（plain use 不暴露 Vue 符号；
     VM 轨两者皆通）。
3. **事实分类不变**：工具行（生成/静态建议/Blueprint/启动预览/交互验证/
   画面更新）状态只来自工具 status 字段与实时 canvas 状态，不嗅探文本；
   lint=advisory、未验证=未验证、帧行只写"最近画面已更新"（无因果证据
   不写热更新）。计划进度与 Canvas 工具进度分开呈现。
4. G-17 studio bounds 与动态 :style 的已知边界保留（不豁免）。

**Rationale**：展示真实 Canvas 能力；消解 G-15 的结构性（非 UI）根因，
并把 T-11 真机实证的 VM 渲染边界固化为规范，防止回归。

---

## SD-02 modify — docs/specs/modules/workspace-ui.md

**ID**: SD-02 | **类型**: modify | **关联 AC**: AC-01, AC-02, AC-05, AC-13, AC-15

**Before（现行规则）**：
- 会话/relay 卡展示 run 状态与阶段（RelayRunBox）；计划执行事实
  （attempt/修复/阻塞/交付检查点）无处可见。
- 完成与交付语义未区分；断连/陈旧/合同不兼容无明确呈现。

**After（修订规则）**：
1. **PlanProgressCard（plan_progress.at）**挂在 chat_message 的
   spawn_relay 卡位（RelayRunBox 上方），消费 PlanProgressStore：
   - 相位轨（规划→实施→复审→沉淀→已交付）+ 当前相位高亮；
   - plan_id · r{revision} · phase · attempt · 修复 c/limit 摘要行
     （handler 域预构建标签，VM/Vue 同义）；
   - 交付检查点行（prepared/landed/ledger_refreshed/archived/cleaned
     冻结序，✓/○）；
   - blocker 横幅（blocked/needs_replan 事实可见）；
   - 断连陈旧横幅（最后成功快照+明确陈旧标记）；
   - contract_version 不兼容横幅（未知版本显式呈现，不默认成功）；
   - 可展开阶段记录（stage/attempt/outcome + reviewed_commit + receipt）；
   - 终态显式标「已完成阶段记录」，不伪装正在运行。
2. **不另建主栏目**：不打断 composer 草稿、审批与 Canvas 状态；五栏目
   布局保留；中英 i18n 键齐备（planProgress.*）。
3. 只有 A 控制器 delivered 事实才显示已交付；检查点缺件不绿。

**Rationale**：VM 可见计划推进事实，避免完成/交付混淆与切换串台
（AC-01/02 的 UI 面）。

---

## SD-03 modify — docs/specs/modules/ui-parity.md

**ID**: SD-03 | **类型**: modify | **关联 AC**: AC-03, AC-04, AC-06, AC-07, AC-15, AC-17

**Before（现行规则）**：
- fixture/runtime 回归基于 gallery 双平台挂载与交互断言；
- 组件 case 覆盖以 inventory 自动生成 + 显式 case 混合。

**After（修订规则）**：
1. **fixture.mock 通道**：fixture 可携带 `mock`（路径→确定性 JSON 响应），
   runner 的 mock server 按路径精确回放——消费型组件（store 轮询类）在
   gallery 内可渲染生产合同事实，而非空态挂载。
2. **expect.visible 并入就绪门**：runner 等待 `expect.visible` 全部出现
   才判 snapshot-ok（上限 28×250ms）——消费型组件的事实经 store timer
   到达（~1-2 拍），静态 fixture 首拍即过，行为不变。
3. **新进度组件双端门**：`plan-progress-card`（owner PLAN-097，
   production-contract）与 `canvas-progress-summary`（store-projection）
   为显式 case；断言面覆盖相位轨/检查点✓/历史标签/展开阶段记录
   （reviewed_commit）与工具行/展开 tool 名。声明全分配（catalog
   PASS 115/115）。
4. **--vm-http 臂**：runner 支持 VM 轨切换 VMHTTP（`--server vm`）作
   第二后端取证面；当前该臂受上游 split 启动缺口阻断（缺口登记，
   修后直接复验）。
5. **运行卫生**：vue 模式 gallery 运行会安装 node_modules（junction）——
   运行后必须清理 gallery `gen/` 并过 wt-guard（Plan 529 红线）。

**Rationale**：fixture 不能替代全流程实拍；消费型组件的 parity 需要
确定性事实源；把真机实证的运行卫生规则固化。

---

## SD-04 add — docs/specs/modules/demo-evidence.md

**ID**: SD-04 | **类型**: add | **关联 AC**: AC-06, AC-07, AC-08, AC-09, AC-10, AC-11, AC-12, AC-13, AC-14, AC-16, AC-17

**新增规则（隔离演示合同）**：

1. **隔离启动**：每跑唯一 run_id/目录（`D:/autostack/.demo/musk-097/<run_id>/`）；
   自有 musk serve（私有端口段+MUSK_CONFIG_DIR 隔离）+ 原生 VM
   （AUTO_EXE 显式锁定当日构建——PATH 滞后构建禁令）；AAID 只读复用
   （不修改配置/不重启/不按名结束）；关停只杀自有 spawn 树 + 端口回收
   验收；遗留进程/端口占用/旧二进制 = 显式失败。
2. **空仓三计划**：bootstrap 仅物化 README/AGENTS/requests 三篇（白名单
   审计，任何业务文件=违规）；git init 线性仓；三轮增量需求走正式
   `flow_id=plan`+`authorization=auto` 入口；needs_replan/blocked/修复
   封顶/failed/canceled 即停（零 nudge、零外部修码）；**工程未交付不发
   下一需求**（deliveredGate：completed+delivered+五检查点齐）。
3. **驱动权限**：驱动只观察与验证——发送需求、读事实、采集、断言；
   不推进内部阶段、不写完成标记、不制造评审结果；行为断言经目标 VM
   动作（宿主 MCP vnode 直选，零固定假坐标）+ 磁盘数据交叉验证。
4. **截图与证据**：来源双轨 `host-vm`（autoui_screenshot 原图直存，
   采集前后身份双验）/`target-canvas`（generation 归属核验，跨计划/
   重启代次 409 拒）；capture 记录 capture_id/source/UTC/viewport/png
   相对路径+SHA256/kind(live|completed-stage-record)/stage_ref；live 采集
   必须绑定已登记阶段事实（缺阶段注入拒绝；场景级整屏显式
   allowUnstaged 且落记录）；PNG 原始画面不重绘不拼接。
5. **manifest v1**：运行指纹（musk/auto 二进制 sha256+版本+worktree
   head）、workspace/session/run/plan 身份、revision、各阶段
   stage/attempt/outcome、reviewed_commit、五交付检查点、收据引用、
   落地主提交与 capture 集合；外部保存（run 目录，与 worktree 生命
   周期解耦）；verify 独立核验（删除图/错 hash/schema 缺件=非零）。
6. **知识出版**：每轮 Agent 产出 `docs/wiki/*.md` 为唯一正文来源；驱动
   经既有 workspace Wiki API 发布**同一正文**（create→409 幂等 update），
   源/回读 SHA256 一致判言；报告披露该确定性出版步骤区别于模型行为。
7. **落地对账**：默认分支线性（无 merge commit）、本地分支只剩默认、
   git worktree 注册表除主检出外无项、计划 worktree 目录已清
   （约定外置 + 后端托管 `<plan>/<repo>` 布局）、docs/specs 权威+ledger
   派生+归档非空+Wiki 产出。
8. **P003 上下文**：新 UI 会话（id 不复用）、会话输入==需求原文
   （空白归一后全等——无注入）、run 事件窗内存在 docs/specs|plans|wiki
   的 turn_tool_call 读取证据（声称已读不算）。
9. **失败语义**：任何轮未交付/验收失败/采集缺件/canceled → 演示非零
   退出并保留现场（不覆盖旧跑、不改 A 上限、不伪成功）。

**Rationale**：固定可复现的 VM 全流程演示合同；证据链版本绑定、
缺件显式、机器可验（AC-06～17 的执行面）。

---

## AC ↔ 证据索引（本计划范围内）

| AC | 主证据 | 状态 |
|---|---|---|
| AC-01/02 | ui-parity plan-progress-card（VM snapshot-ok）+ store 单飞/代次 V02 矩阵 + T-12 真实 run 事实 | T-11✅ + T-12 v2 收口（延期 097-D1） |
| AC-03/04 | probe-g15 SMOKE + canvas-progress-summary（VM snapshot-ok）+ T-12 行为断言 | T-11✅ + T-12 v2 收口（延期 097-D1） |
| AC-05 | PlanProgressCard 中英 i18n（27 键）+ V04 快照 | T-11✅ |
| AC-06～08 | capture/evidence 模块 V02 8/8 + T-12 manifest | T-12 v2 收口（延期 097-D1） |
| AC-09～12 | bootstrap 白名单 + waitForRound 停止条件 + deliveredGate + 三轮真实 run | T-12 v2 收口（延期 097-D1） |
| AC-13 | wiki.mjs 发布/回读 hash（V02 5/5）+ T-12 各轮发布记录 | T-12 v2 收口（延期 097-D1） |
| AC-14 | VmSession 端口/归属/指纹 V02 18/18 + T-12 owned 清理 | T-06✅ + T-12 v2 收口（延期 097-D1） |
| AC-15 | V03 三门 + V04 四 case VM + Vue staging 构建 + V07 全绿；VM+VMHTTP 臂=上游缺口登记 | T-11✅（一臂缺口） |
| AC-16 | waitForRound 响亮停 + 干跑失败留证 | T-12 干跑✅ |
| AC-17 | verifyManifest 独立核验 + 本 delta 版本绑定 | T-13/T-14 |
