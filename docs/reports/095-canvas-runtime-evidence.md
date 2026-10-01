# PLAN-095 Canvas 运行时消费与稳定性证据报告（T-08）

- 计划：PLAN-095 r2；阶段 work / T-08 双端消费与稳定性收尾
- 日期：2026-10-01
- worktree：musk-095（plan-095-dev）/ auto-lang musk-095（auto-musk-095-dev）
- CLI：auto-musk-095 依赖仓构建（sha256 见 baseline §1 及 T-06 收据 89c22af1…）

## 1. V06 探针（scripts/ui-parity/canvas-runtime-probe.mjs，VM 臂）

fixtures：tests/ui-parity/probes/canvas-runtime/{media-events,coords,focus,
media-soak}（自 auto-lang tmp 定罪 fixtures 移植固化，随仓持久）。
驱动=auto run --render vm + AutoUI MCP；收据 tmp/ui-parity/PLAN-095/
v06-*-receipt.json。

| 探针 | 断言 | 结果 |
|---|---|---|
| media-events | loaded 单次（A：真 PNG 240x160 rev=1）、坏图 error 单次（reason=unsupported image format 可定位）、3s 驻留计数稳定、A→B 换订阅旧图零污染 | PASS |
| coords | __mcp_drag 三点序列（小数/零/负值）typed float 状态断言（mx=-5.50/my=-7.25 vs -5.499/-7.249 ≤0.01）、down/up 协议形态均触发、move_n=3 | PASS |
| focus | ui.focus 原语：composer(.Input)/input(.TiInput) 双目标 __focus_result=ok；缺失目标 miss:.NoSuch 可定位（立即 miss 语义） | PASS |
| media-soak | 5 分钟好/坏图连续翻面（300ms tick）：通知有界（每面恰一次）、mismatches=0（loaded/error 与期望面零错配）、进程存活 | 见收据 |

运行方式：`AUTO_EXE=<cli> node scripts/ui-parity/canvas-runtime-probe.mjs
--plan 095 [--skip-soak]`；缺实机证据非零退出。

## 2. 真实指针/键盘补充证据（auto-lang tmp 驱动，收据已归档）

- t03 overlay-hit：SendInput 真点击穿透被动蓝框（pointer-events:none）命中
  base（hit=1）+ 交互浮层 capture 保留（pop=1）——合成通道不经过命中测试，
  穿透证据只认真指针（docs/reports/095-evidence/t03-overlay-hit-probe.txt）。
- t04 focus：真点击触发 ui.focus 链（handler 帧不回滚标记）+ 双目标
  ok/miss（t04-focus-probe.txt）。
- **环境边界记档**：本会话 SendInput 键盘事件被前台锁系统性拒绝（鼠标位置
  路由不受影响）——「点击后真键盘落字」的 OS 投递半边由 iced 级聚焦结果
  状态（ok/miss）+ VM 级链路单测（ui_focus_vm_tests）+ MCP type_text
  （handler 通道，viatype 落 .text 实证）组合承载；有头环境补拍归 review
  后续。Tab/Escape 不回归归 V04 既有套件。

## 3. Vue 臂合同

- 生成面：image_surface_contract 三轨之 Vue 轨（@load/@error/@wheel 指针
  三相位包装/object-fit 绑定/禁浏览器解码泄漏）1/1 PASS（T-01 重锚后）。
- 打包面：V07 vue-tsc+vite 全量构建 EXIT=0（65 组件，9.89s）。
- 浏览器实机运行（live）：复用 093 live.mjs 基建（canvas-studio 链），
  095 未另起浏览器 farm——**bounded remainder**：Vue 臂 DOM 事件实拍
  （@load/@error 真触发录屏/截图）留独立会话，不阻断 VM 臂合同完整。

## 4. 稳定性（AC-09）

- 媒体 soak（5min 翻面）：见 §1 media-soak 收据。
- 资源有界：媒体票据 registry MAX_ENTRIES 驱逐 + 会话关闭面沿用；通知
  表项随视图存在性生死（卸载即退订）——boundedness 由 T-02 设计 + soak
  计数对账承载。
- PID 纪律：runner 仅 kill 自 spawn 进程树（taskkill /T /F 按 PID）；
  共享服务/093 工作树/昨日遗留 auto 进程未触碰。

## 5. 与 093 的消费对账（§5.8 交付表落实）

| 093 任务 | 交付 | 状态 |
|---|---|---|
| T-03/T-04 媒体帧 | loaded/error+代次过滤（T-02） | ✅ 可接入（事件名 Loaded/LoadFail 形态=探针 fixture） |
| T-05/T-06 叠层+strict | 宿主保持已解除+穿透；strict 门收据（T-06） | ✅ |
| T-06 G-11 消费面 | 子件两形态实例化（T-07） | ✅ 解除——093 补跑 VM 消费面 |
| T-08 composer 聚焦 | ui.focus 原语+消费示例 | ✅ 原语就绪；生产端口接线（platform.vm.at platformFocusComposer）归 093 |
| T-11/T-13 坐标验收 | Float 统一+typed 断言机制（T-05） | ✅ |
