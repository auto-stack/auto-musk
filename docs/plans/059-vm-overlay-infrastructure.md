---
plan_id: PLAN-059
status: executing
feature_name: VM(iced) 悬浮层基础设施与 overlay 组件族落地
author: [zhaopuming]
created_at: 2026-09-03T13:30:00+08:00
updated_at: 2026-09-03T13:30:00+08:00

supersedes_spec_components: []
new_spec_components: []
touched_goals: []

current_step: 0
total_steps: 10
---

# [PLAN-059] VM(iced) 悬浮层基础设施与 overlay 组件族落地

## 变更摘要

PLAN-058 实机验收暴露根因级缺口：**musk VM 轨没有任何悬浮弹窗机制**。所有
本应悬浮的 UI（删除确认 alert-dialog、工程目录 dropdown、设置 dialog、
tooltip/toast）都退化成流内元素嵌入邻近界面。本计划回到 auto-lang 的 widget
实现本身，在 iced renderer 落地悬浮层基础设施，并按家族接通 overlay 组件族，
以 widgets-gallery 双轨对拍为验收门禁。

## 目标

1. iced renderer 悬浮层基础设施：根 Stack 分层 + overlay host + 锚定定位 +
   关闭语义（ESC/遮罩/reka 对齐的 update:open 折算）+ 焦点管理。
2. alert_dialog / dialog 家族模态化（居中卡片 + 遮罩 backdrop + action/cancel
   onclick 派发）。
3. dropdown_menu / popover 弹层（trigger 锚定下方、越界翻转、外点关闭）。
4. tooltip / hover_card（iced 原生 tooltip 或 overlay 实现）。
5. select / combobox 下拉（含键盘导航最低限）。
6. musk 三场景回归：删除确认（替换 PLAN-058 的 VM 内联降级行）、工程目录
   选择 dropdown、设置 dialog 告别流内降级。
7. widgets-gallery overlay 家族页面双轨（vue/vm）对拍门禁。

## 架构方案

### 现状（2026-09-03 勘察实证）

- **schema 矩阵**（`auto-lang/schema/aura.at`）：overlay 类组件 36 个，35 个
  `iced: none/unknown`（唯一例外 sheet=fallback）。扩展到全部悬浮语义家族
  （dropdown/dialog/alert_dialog/popover/hover_card/sheet/drawer/command/
  context_menu/menubar/nav_menu/combobox/tooltip/toast，~100 个元素）无一
  实现；dropdown/modal/dialog/tooltip/toast/select 标注 `fallback`——实机
  表现为**退化为流内容器**（.at 的 absolute 定位类 VM 不消费）。
- **gallery 实证**（examples/widgets-gallery,render:vm）：/alertdialog 页按
  "Show Dialog" 无任何可见反应（trigger 子件 iced:none,open 机制不存在）；
  /dropdownmenu 页按 "Open" 同样无效（页面快照中的 Profile/Billing 等
  item 文本均来自代码展示块,非渲染层）。截图
  `widgets-gallery/src/front/tmp/autoui-screenshot-17884073{70102,75083}.png`。
- **musk 实证**：PLAN-058 T7 实机按 × 后对话框子树整体丢弃
  （autoui_snapshot 无 alertdialog 痕迹）；workspace/settings 面板内联。
- **VM 跨 widget msg 派发缺陷**（PLAN-058 待澄清⑩）：子 widget onclick →
  子 msg → 父 handler 的 emit 链在 VM 静默断路（带参/无参均断）。overlay
  组件（dialog 按钮在弹层内）依赖该链路——**必须先修**，属本计划前置。

### iced 0.14 原生能力（可行性依据）

auto-lang 依赖 iced 0.14（crates/auto-lang/Cargo.toml:164），`iced_widget`
0.14.2 已提供：`stack`（Stack::push/push_under 分层）、`overlay` 模块
（嵌套 overlay,combo_box 内部即用此实现下拉）、原生 `tooltip`、
`combo_box`。悬浮层基础设施完全可行,无需外部 UI 库。

### 架构发现（2026-09-03 T3 勘察，改变实现量级）

**悬浮层基建已存在大半**：`ui/iced/popover.rs`（529 行）是完整的自绘锚定
浮层 widget——placement/at_point/gap/open/on_dismiss/Esc/外点关闭全备，
renderer.rs:3839 已接 `AbstractView::Popover`，aura_view_builder.rs:4972
已有 popover-trigger/content 拆解臂（含自管开合 slot id 与 ondismiss）。
**真正的缺口收窄为**：alert_dialog / dialog / dropdown_menu / tooltip 等
家族没有接上这套机制（builder 无对应臂 → 元素走 default 臂丢弃）。

→ T3/T4 修订：不再需要"根切 Stack"的大改，改为——
① 在 popover.rs 基座加 `ModalWidget`（全屏 backdrop 捕获 + 居中卡片，
复用其 Esc/捕获机制，~150 行）；
② aura_view_builder 加 alert_dialog 家族臂（trigger/content/header/title/
description/footer/action/cancel 拆解，action·cancel onclick 走既有
DynamicMessage 派发——与 popover 臂同构）；
③ dropdown_menu 臂复用 popover（placement bottom-start + 菜单 item 列表）。
T2 的 child_emit 修复已使弹层内按钮的 onclick 派发可用。

```
渲染根: Stack { base: 现有视图树, ...open_overlays }
overlay registry（renderer 侧）:
  { id, kind: modal|anchored, anchor_rect?(trigger 命中区域), content: VNode,
    open: state 绑定, on_dismiss: update:open(false) 语义 }
alert_dialog/dialog  → kind=modal: backdrop（opaque,捕获点击=dismiss）+
                       居中 content 卡片（宽 min(480px, 90vw)）
dropdown/popover     → kind=anchored: trigger rect 下方 4px,右越界左翻,
                       下越界上翻;外点 dismiss
tooltip/hover_card   → hover 进入延迟 ~300ms 显示,anchored 无 dismiss 遮罩
select/combobox      → anchored + 键盘 ↑↓/Enter/ESC
schema 回填:         iced: none → native（随实现逐家族推进）
```

- **事件**：overlay 内容内元素复用现有 handler 派发链——前置修复
  跨 widget msg 派发（PLAN-058 ⑩）：以最小复现（child widget button →
  parent onX）定位 VM emit 断点（怀疑 child_emit.rs/interpreter 作用域）。
- **AutoUI MCP**：aura_snapshot 需包含 overlay 层（open 态时），否则验收
  自动化失明——随基建同步。
- **musk 消费**：PLAN-058 的 VM 内联确认行降级保留为兜底或直接切换
  alert-dialog（取决于本计划落地后的跨 widget 派发修复）；workspace/settings
  的手搓流内面板改为标准 dropdown/dialog。

## 技术栈

- auto-lang（主战场）：crates/auto-lang/src/ui/iced/*（renderer/vnode）、
  schema/aura.at、crates/auto-lang/src/ui/interpreter。
- 验证：widgets-gallery（vue/vm 双轨）、AutoUI MCP snapshot/截图、
  iced_test、musk 实机三场景。
- 仓库与分支：auto-lang 依赖 worktree `auto-lang/.worktrees/auto-musk-dev`
  （AGENTS.md 第三行惯例）；计划文档与验收记录在本仓 docs/plans。

## 需求分析与背景调查

- 用户实证（2026-09-03）：musk VM 版"所有需要弹窗的地方都有问题——本来
  应该悬浮的弹窗变成嵌入到附近的界面里"，点名 alert-dialog/工程目录
  dropdown/设置 dialog 三处；结论"回到 widget 的实现本身去（包括用
  widget-gallery 的展示去验证）"。
- 账本钩子：PLAN-058 待澄清⑩（跨 widget 派发断路）、055-3 子件缺陷族、
  057 行 VM 实机合成输入守卫。
- 相关上游文档：docs/design/autoui/base-styles-and-visual-parity.md
  （§4.5/4.6 双轨对拍规约先例）。

## 测试设计

- **单测**（auto-lang）：overlay registry 挂载/卸载、anchored 定位与翻转、
  modal backdrop dismiss 事件、update:open 折算——iced_test。
- **gallery 门禁**：/alertdialog /dialog /dropdownmenu /popover /tooltip
  /select /combobox 各页 VM 端：按 trigger → MCP snapshot 断言弹层节点存在
  且不在文档流父链下 + 截图人工目验浮空效果；与 vue-ref 对拍。
- **musk 回归**：三场景（删除确认 alert-dialog、workspace dropdown、设置
  dialog）双轨对拍 + PLAN-058 六断言在 alert-dialog 形态下复跑。
- **既有门禁**：auto build --gen-only / cargo tf（auto-lang 全套）/
  vm-safe-lint / musk vitest 基线不劣化。

## 验收标准

1. VM 实机：alert-dialog 以居中模态+遮罩悬浮呈现，ESC/遮罩/取消均可关闭
   且状态复位；action onclick 正确派发（跨 widget 修复生效）。
2. dropdown-menu 以锚定弹层呈现，外点关闭，越界翻转正确。
3. tooltip 悬浮显示；select/combobox 可下拉选择。
4. widgets-gallery overlay 家族页面 vue/vm 截图对拍通过（悬浮语义一致）。
5. musk 三场景回归通过，VM 内联降级行可退役（或登记保留原因）。
6. schema overlay 家族 iced 标注从 none → native 随实现回填。

## 执行步骤（Phase 1 基建 + alert_dialog 纵切；后续家族按同模式铺开）

- [✅ 已完成] worktree 建（基于 master 7ab140c41）；探针工程
  examples/overlay-probe 随 T2 入库。
- [✅ 已完成] **T2 根因实锤+修复**（auto-musk-dev 分支）：派发侧
  `emit_key = "on"+子 msg 名` 与注册侧父声明键**大小写错配**即静默丢派发
  （子 msg `Confirm` + 父绑定 `onconfirm` → lookup("onConfirm") 落空）。
  修复：child_emit.rs 注册/派发两侧键折叠小写匹配 + 2 个单测。全量 lib
  （--features ui-iced）4284 过/173 败 vs 基线 4280/175——零新增失败、
  净修好 3（其余 172 为存量环境批量失败）。**e2e 残项**：VM 实机矩阵复测
  被启动工具链不稳定阻塞（launcher 随机退出/MCP ~60-90s 寿命/探针首编译
  卡 "Waiting for AutoVM server"——探针 build 目录已清需一次性全量编译），
  探针工程就绪（examples/overlay-probe，AUTOUI_MCP_PORT=9277），工具债
  修复后补跑。另登记：VM split 模式必须 src/back/api.at，缺失报
  20×"Expected term, got RBrace" 零位置诊断（解析器诊断债）。
- [ ] **T3 overlay registry + Stack 根**：renderer 根切 Stack（base+layers），
  registry 数据结构与 open 绑定生命周期。验证：iced_test 单测绿；现有
  musk/gallery 无回归（空 overlay 时渲染不变）。
- [ ] **T4 modal kind + alert_dialog 家族**：backdrop/content 居中卡片/
  action·cancel onclick 派发/ESC。验证：gallery /alertdialog 按触发钮出
  浮层,Continue/Cancel onclick 生效;截图双份。
- [ ] **T5 anchored kind + dropdown_menu 家族**：trigger 锚定/翻转/外点
  dismiss。验证：gallery /dropdownmenu 按 Open 出锚定弹层。
- [ ] **T6 tooltip + hover_card**：原生 tooltip 包装或 anchored 变体。
  验证：gallery 对应页。
- [ ] **T7 select/combobox**：anchored + 键盘导航最低限。验证：gallery 页
  下拉选择生效。
- [ ] **T8 schema 回填**：已实现家族 iced 标注 none→native。验证：schema
  校验器通过、S001 INFO 相应减少。
- [ ] **T9 musk 消费回归**：删除确认切 alert-dialog（视跨 widget 修复决定
  保留或退役 PLAN-058 内联行）；workspace/settings 改标准组件。验证：
  musk 实机三场景 + PLAN-058 六断言复跑。
- [ ] **T10 收尾**：账本回写（overlay 缺口族、跨 widget 派发修复）、
  gallery 对拍截图入 attachments、status 推进。

## 复审记录

（/auto-plan:review 填写）

## 待澄清事项

1. **跨仓库计划归属**：主修复面在 auto-lang；本计划文档按先例放本仓
   docs/plans，任务在 auto-lang/.worktrees/auto-musk-dev 执行——是否符合
   你对"回到 widget 实现本身"的预期？还是想立到 auto-lang 自己的 plan 序列？
2. **范围裁剪**：overlay 家族约 8 个家族 100+ 元素，Phase 1 只纵切
   alert_dialog + dropdown_menu + tooltip（musk 三场景所需），其余家族
   （drawer/sheet/command/context_menu/menubar/nav_menu/combobox）建议
   Phase 2 另批——是否同意分期？
3. **modal 库选型**：iced 0.14 原生 Stack+overlay 自研 vs 引入 iced_aw 的
   Modal——倾向原生自研（依赖面小,combo_box 内部即此模式），待复审裁定。
4. **PLAN-058 内联确认行退役时机**：alert-dialog 落地后，VM 内联行是删除
   还是保留为降级兜底？倾向删除（单一实现）。
5. **AutoUI MCP overlay 可见性**：snapshot 是否包含 overlay 层需在 T3 一并
   定义（验收自动化依赖它）。
