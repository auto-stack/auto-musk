# PLAN-093 证据报告 — AutoUI 应用设计工作台与实时 Canvas 双端 UX

- 计划：PLAN-093（revision 1）｜ 证据基线：plan-093-dev@<见 git log>（worktree）
- 依赖：auto-lang sibling auto-musk-dev@fbf55f913（G-12/G-16 修复，待随最终 merge 折回
  auto-lang master——master 有用户 WIP，折回时机已确认放最终 merge）；auto-ai@5a50a55；
  auto-down@895f8d0。宿主 DPI 1.0（Windows 10 x64，VM 窗口 1280x800 / 目标 480x680 逻辑）。
- 复验入口：`node scripts/ui-parity/canvas-studio-live.mjs --plan 093`（V05 四模式）、
  `node scripts/ui-parity/canvas-contract.mjs`（V03）、`node scripts/ui-parity.mjs check`（V06）、
  `cargo test -p musk --test canvas_studio_contract`（V02 面）、
  `cargo test -p musk --test canvas_live -- --ignored --test-threads=1`（V07，逐目标串行跑法见 §8.27）。
- 收据：`tmp/ui-parity/PLAN-093/`（canvas-studio-live-receipt.json、ports-vue-*-receipt.json、
  studio-live-*.log、vm-cycle-receipt-093.txt、musk-canvas-live-pid-receipts.jsonl、
  probe/curl/几何取证各轮落点）。tmp 快照类为过程件；本报告与计划 §9 引用为持久索引。

## AC 结果矩阵

| AC | 结果 | 证据（可复验入口） |
|---|---|---|
| AC-01 入口/进出/空态/打开应用 | ✅ | VM cycle：studio 进出（侧栏 240→48→240）、空工作台（提示+路径输入+启动）、打开已有应用（targets/probe-a → running seq:1，坏路径错误面 T-05）；Vue 臂 studio 切换 587>349px；收起后轮询不复开（T-04 collapse-persists，5s+ 多拍） |
| AC-02 三栏/窄窗/滚动/composer 可达 | ✅ | §5.2 宽度分层 4 档取证（tier-results.json：1440=[360,968]/1100=[320,668]/900=钮列+结构受控/700=页签塌缩）+ Vue 臂；VM：studio 布局渲染（树行 184/画布列/对话列节点在）；VM studio 精确 rect 见 G-17 记档（布局渲染不受影响） |
| AC-03 收起≠停止；停止收割+保留工作台 | ✅ | T-04（收起 12+ 拍不复开、生命周期 running 不变）；T-12：stop → owned-PID confirmed-dead（PID 收据 18 笔）+ 已停止工作台（标题+重运行钮） |
| AC-04 状态真实性（无伪造绿勾） | ✅ | T-09 进度投影（V03：只投影有证据行/状态只来自 status 字段+实时状态/代次章失配不冒充）+ degraded 故障注入（spawn 重试耗尽→degraded tail 750B→UI 红条）+ lint advisory 行（非硬门） |
| AC-05 双端四模式真实帧/选/源码/停 | ✅ | **V05 FOUR-MODE ALL PASS**（canvas-studio-live-receipt.json）：vue-{rust,vm}（17 断言×2）、vm-{rust,vm}（VM cycle ALL PASS×2，坐标点选端到端命中+源码面板行渲染）；VM 无桩（canvas.vm.at T-03 起真实通道） |
| AC-06 适应/100% 点选对准+留白清选 | ✅* | web：V03 几何 21 例（映射边界/截断扫描/双模式样式）+ DPI 1/1.5/2 锚定直点全命中 + 双留白（§8.16）；VM：坐标链端到端（drag (240,340)→picked 大命中钮 bbox 完整）。*VM 侧 wrap 自适应 :style 受「字段引用」codegen 限制 → 静态等比回退尺寸（360x510=0.75）承载，自适应精确面归 web 轨（§8.30/§8.31 记档） |
| AC-07 树 kind/label/循环/折叠/大 ID | ✅ | V03 65/65（大 ID >2^53 全链保真 4 例/折叠保持）；VM 结构列树行 184（G-16 修复后）；源码面板行高亮+定位（G-13 通道，262 行实证） |
| AC-08 清选/未命中/旧响应不复活 | ✅ | 后端契约 21/21（未命中清选/204 无 null 锚点/代次丢陈旧选/路由 409·clear 臂）；web 委托 204→ClearPick + chip ⨯ 双面清（Vue 臂）；VM chip ⨯ 快照断言待 label 形态对齐（清选链已由契约+Vue 臂承载，记档） |
| AC-09 源码只读/行定位/来源置信 | ✅ | G-13 canvas 域只读通道（双 serve 同契约）；行自动定位（44/262→居中）；Q-04 置信度 exact/uncertain 全链（pac.at 排除实证；uncertain 提示条+ctx source_confidence+Agent 注记）；非法行号/读失败/无 source 状态机 |
| AC-10 chip/冻结/队列/IME 不变 | ✅ | T-08：chip 双 store 挂载、SendInput 冻结（命令/mention/IME 不受影响三态 curl+取证 5/5）、队列条目 {text,ctx} 逐条冻结（busy 退回/stale 409 队首错误面）、A→B 等价覆盖双证（§8.23）、回放标记+turn 注入+分页桥（§8.21） |
| AC-11 代次/归属/上下文校验 | ✅ | 契约：代次冲突 409/归属停止拒止/未绑定语义/frame seq 不伪造历史/design_context stale 拒收+当前代次带章+缺字段拒收（21/21）；A→B 乱序（等价覆盖双证 §8.23） |
| AC-12 失败恢复/重启耗尽/stop 不伪造 | ✅ | T-12：重启预算耗尽→degraded（error 含 budget+tail 750B）→显式 stop 可用（V07 4/4 实机 26.81s，PID 起止收据）；首启失败直落 degraded（§8.23b）；stop 失败不伪造（契约） |
| AC-13 主题/中英/键盘/状态非颜色 | ✅* | 主题：全程沿 pac.at 语义 token（未新增主题面——构造性统一）；中英：新增 22+4 键双语言入册（i18n zh/en）；键盘：Escape 收敛（源码页签/错误详情，不触 stop——T-10 增一 Vue 臂 ALL PASS）；状态点+文本双编码。*Tab 序巡检与双主题实拍列 review 复验项（记档） |
| AC-14 单飞/不重下/稳定 | ✅ | poll 单飞+有界 tick 释放（T-04）；seq 双键零重写（帧 URL 身份+seq）；稳定性：8s 快照一致（cycle）+ 5min 媒体 soak（095 T-08 499 周期，运行时层）+ T-12 真实生命周期回归 |
| AC-15 会话/审批/mention/流式/M3 沙箱保持 | ✅ | V02 全套绿（lib 505+33 目标；串行跑法）；V07 4×ignored（M3 全链 lint→bp_check→canvas_run→act/state/snapshot→stop）；审批门/队列/附件回放/旧 JSON（parity 族+chat_page 8/8） |
| AC-16 复验入口/证据绑定/delta 边界 | ✅ | V06 catalog PASS（114 declarations）；本报告+spec-delta 报告（093-app-studio-spec-delta.md）绑定 revision 与提交；ledger 为派生（merge 阶段刷新） |

✅* = 通过且带记档限制（限制不降门槛：限制面由另一轨承载或列 review 复验项）。

## 场景矩阵落点（§6 → 收据）

- 入口/布局五尺寸：tier-results.json（Vue）+ VM 1280x800 窗口实证；VM 多尺寸=窗口参数（同布局树）。
- 几何/DPI：geom/letterbox 取证 + V03（web/VM 同一换算 helper 单源）；VM 坐标链端到端（本轮）。
- 选择/时序/附件：契约 21/21 + Vue 臂 + A→B 双证。
- 状态/生成：T-09 投影 + degraded 注入 + T-12 耗尽。
- 稳定性：8s 快照一致 + 095 soak（运行时）+ T-12 PID 收据。

## 已知限制（记档，均不降门槛）

1. G-15：进度摘要 VM 隐藏（computed use.web.fn VM 返空，坑①家族）——VM 轨摘要待
   store 域预计算形态；web 轨全量 ✓。
2. G-17：studio 模式 VM 快照 rect 全零（布局渲染不受影响——树行/wrap 节点在）；
   坐标/几何面按 normal 模式验证（同一组件链）。
3. wrap 自适应 :style（字段引用）VM 不产出——静态等比回退承载；web 轨精确自适应。
4. VM chip ⨯ 快照断言待对 label 形态（清选链已由契约+Vue 臂承载）。
5. auto-lang sibling 修复（G-12/G-16）折回 master 放最终 merge（用户 WIP 在主检出）。
