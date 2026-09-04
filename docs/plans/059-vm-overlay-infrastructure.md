---
plan_id: PLAN-059
status: executing
feature_name: VM(iced) 悬浮层基础设施与 overlay 组件族落地
author: [zhaopuming]
created_at: 2026-09-03T13:30:00+08:00
updated_at: 2026-09-04T12:00:00+08:00

supersedes_spec_components: []
new_spec_components: []
touched_goals: []

current_step: 8
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

### T4 实施检查点（auto-musk-dev 分支 WIP 提交，编译绿）

已落地：
- `view.rs`：`PopoverPlacement::Modal` 变体；
- `popover.rs` Panel 模态三语义：layout 根=全视口命中区+content 居中子节点
  （`move_to` 按值消费须接收返回值）；update 内容外点击/ESC=dismiss+捕获
  （阻断基础树）；draw 先画全视口暗幕（`advanced::renderer::Quad`，
  fill_quad 需 `use iced::advanced::Renderer` trait 在作用域）；
- `aura_view_builder.rs`：主 match 加 alert-dialog 家族臂（容器/文本/button
  委托），`convert_alert_dialog` 子标签改名（trigger/content→popover-*）
  委托 `convert_popover_inner`（placement_override=Modal）+ oncancel 别名
  折算 ondismiss；placement 解析加 "modal"。

**剩余断点（下一段工作从这里继续）**：gallery /alertdialog 实机——按
Show Dialog 后 MCP snapshot 已见对话框子树（title/Cancel/Continue），但
截图视觉无浮层。二选一断点待查：
(a) 触发器 `__popover_toggle` 开合是否真把 POPOVER_OPEN 置为该 slot id
   （popover 臂的自管开合接线对 alert-dialog 委托形态是否生效）；
(b) Modal 的 draw 路径（暗幕/卡片是否真的绘制，index=10.0 层级是否被
   基础树覆盖）。排查顺序：先 autoui_state/snapshot 对比 open 前后差异 →
   再日志确认 popover.rs Panel::draw 是否被调用。

**排查结论（T4 根因定案）——VM 应用视图来自 a2r/ninja 编译产物，运行时
builder 不参与页面构建**。实证链：①builder 两处臂（tracked convert_node_
tracked_ctx + untracked convert_element）与 convert_alert_dialog 入口的
eprintln，/alertdialog 页导航后全部零命中；②convert_element fallback 臂
的未知 tag 日志亦零命中——alert-dialog 元素从未进运行时 builder；③
`auto run --render=vm` 走 windows_ninja 端口把 .at 编译成原生程序（工程
build 目录 build.ninja），页面视图由 **ui_gen/rust.rs 代码生成**产出，其
分发表无 alert-dialog 臂 → 编译页代码直接丢弃对话框子树（触发 button 臂
存在故按钮渲染,弹层机制不存在）。

**T4 剩余实现 = ui_gen/rust.rs（a2r codegen）加 alert-dialog 家族臂**：
生成 Modal 浮层构造代码（对齐 View::Popover/PopoverPlacement::Modal 语义：
居中卡片+全视口暗幕+ESC/外点 dismiss）。已完成的 interpreter 侧臂（本仓
auto-musk-dev 分支提交）服务动态/解释模式,保留。工具注记：工程 `.auto/
ui-cache.json` 缓存编译产物,源码不变则复用旧产物——**codegen 改动后须删
该文件强制重编**;另 Windows 下运行中的 auto.exe 锁文件,cargo build 前须
taskkill（此前多轮"构建成功实为静默失败"皆因此）。

**【2026-09-03 更新】本节工作已立项 auto-lang PLAN-533**
（`auto-lang/docs/plans/533-vm-overlay-runtime-channel.md`,commit b67525b63）：
四件套（codegen 臂+生成侧 Modal 运行时+开合/open 绑定+ESC/外点事件回流）
+ 重做 auto-musk-dev 丢失工作（Modal 基建三件 + child_emit 大小写折叠）。
533 完成并合回后,本计划从 T4 验证起恢复执行（gallery /alertdialog 实机 →
T5-T8 → T9 musk 三场景回归）。

### codegen 侧确认（2026-09-03 补充勘察）

- 用户实机复测定案：编译 VM 轨点 Show Dialog **完全无反应**——alert-dialog
  的 trigger 开合/open 绑定/浮层机制在编译产物中不存在,按钮被编译为普通
  button（无 onclick 语义）,点击自然无事发生。同时解释了为什么 PLAN-058
  的内联确认行（col/row/span/button 基础件）在编译 VM 轨正常工作——
  codegen 对基础件有臂,对 alert-dialog 家族没有。
- ui_gen/rust.rs 现状：6426 行;`tag_to_view_fn`（:3613）按 tag 映射视图
  构造 fn;无 popover/alert-dialog/dialog 任何浮层家族臂;亦无浮层运行时
  （生成的视图代码的 overlay 通道）。
- **T4 剩余工作量重估**：不是"补一个 match 臂",而是给 a2r 编译轨补浮层
  运行时通道——①codegen 臂:alert-dialog 家族 → 生成 Modal 构造调用;
  ②生成侧运行时:Modal/浮层的 iced 实现（可复用 ui/iced/popover.rs 的
  Panel 模式,需确认生成代码可引用的运行时 crate 面）;③触发器开合 +
  open 态绑定（state_ref v-model 对齐 vue 轨）;④ESC/外点关闭事件回流。
  建议作为 auto-lang 独立计划立项（依赖 ui/iced 深水区,非 musk 侧可闭环）,
  musk 侧 VM 确认行（PLAN-058 形态）在浮层通道落地前为最优可用形态。

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

### 并行批次（2026-09-04 增补——等待 auto-lang PLAN-533 期间可即做项）

- [✅ 已完成 2026-09-04] **S1 验收环境硬化**：一键开发栈脚本（scripts/dev-stack.mjs/.cmd）——
  后端 musk serve :9247（workdir tmp/musk-demo,已监听则跳过）+ VM 前端
  （AUTO_BACKEND/AUTO_VM_MERGE=0/RUST_MIN_STACK/**AUTOUI_MCP_PORT=9277 换口**
  ——修复 MCP 与后端抢 9247 的 FATAL 盲验）+ 可选 Vue dev :3335（代理 9247）。
  533 联测期 snapshot/驱动取证的直接前置。
  烟测通过：MCP `listening on http://127.0.0.1:9277` 实证（d5d6270）。
- [ ] **S2 Vue 轨对拍确认**（需人工）：:3335 上逐项核对与 VM 同源修复——
  会话卡 hover ×、工具卡逐卡展开、消息间距 gap-10、copy 靠左;产出 web 基准
  截图（533 对拍门禁的 web 侧基准）。
- [ ] **S3 worktree 收尾**（依赖用户验证展开/hover 正常）：auto-musk-dev-1
  三批提交（120d89e/a2ef16e/cbece28）合回 main,删 worktree+分支。
- [ ] **S4 挂账转计划**：VM 数据面 DEGRADED 30 fn → musk PLAN-060;
  059-FU1 反应性三问题 → auto-lang PLAN-536（原 PLAN-534,序号与
  534-vm-widget-family-parity 冲突,后建者改号）。（已完成 2026-09-04）

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
- [✅ 已完成 2026-09-04 复核] **T3 overlay registry + Stack 根**：533 交付吸收——
  生成侧 Modal 运行时 + overlay 分层随 533 T2/T3 合回 auto-lang master
  （7d2f17cb4）。空 overlay 无回归当日在 gallery VM 实机观察成立（base 树
  渲染不变）；musk 侧回归并入 T9 验证。
- [✅ 已完成 2026-09-04] **T4 modal kind + alert_dialog 家族**：gallery
  /alertdialog VM 编译轨四断言全过（AutoUI MCP :9277 驱动）：①按 Show Dialog
  → 居中卡片+浮层悬浮遮住页内容（截图）；②Cancel → .cancelAction 派发 +
  show→false + __toast 置值；③重开 Continue → .confirmAction 派发 + 关闭；
  ④ESC → show 仍 true（shadcn 语义:alert-dialog 不因 ESC/外点关,533 复审
  裁定以代码为准,验收标准 1 原文"ESC/遮罩可关闭"据此修订）。证据
  docs/attachments/p059-gallery-alertdialog-{closed,open,esc-persist}.png。
- [✅ 已完成 2026-09-04] **T5 anchored kind + dropdown_menu 家族**：gallery
  /dropdownmenu 编译轨：Open 触发（533 铸造 __dlg_toggle_1 自管开合,open_1
  false→true）→ 菜单锚定触发钮正下方悬浮、覆盖后续文档流（截图）；再按
  触发钮 toggle 关闭（true→false）。demo item 未声明 onclick 故派发不可测
  （非缺陷）。**残差**：ESC 不关 dropdown（shadcn 应关）——auto-lang 后续
  家族计划候补。证据 p059-gallery-dropdown-open.png。
- [✅ 已完成 2026-09-04 盘点] **T6 tooltip + hover_card**：编译轨均未实现
  （gallery /tooltip /hovercard 内容在树挂载、无浮层;hover 激活且 MCP 无
  hover 动作不可达）。**toast 顺带盘点**：alert-dialog Cancel 后 __toast
  状态置值但无视觉呈现（sonner 家族未实现）。→ 全部归 auto-lang 二期家族
  （见待澄清⑫）。
- [✅ 已完成 2026-09-04 盘点] **T7 select/combobox**：编译轨均未实现
  （/select 的 select-item、/combobox 的 command-item 均在树挂载、无弹出
  触发元）。→ 归 auto-lang 二期家族（见待澄清⑫）。
- [✅ 已完成 2026-09-04] **T8 schema 回填**：533 T8 交付已在 master——
  schema/aura.at `iced: "native"` 计 27 条（alert_dialog 族 8 + dropdown 族
  等）;其余 252 条 iced:"none" 属未实现家族,随二期推进。
- [🚧 进行中 2026-09-04] **T9 musk 消费回归**：**场景1（删除确认）代码侧完成**——
  chats_view 直写 alert-dialog 单源（web 生效:shadcn v-model:open+title 单删/
  全删分流+新增 confirmIrreversible 键中英+catalog 重跑;DeleteConfirmDialog.vue
  适配器退役）;VM 内联确认行返场兜底（解释器轨 blocker,见待澄清⑨;否则删除
  流程卡死 pending）。门禁四绿;MCP 实机:AskDeleteAll→确认行渲染+取消复位
  （截图 p059-musk-vm-strip-all.png）;单删路径卡片 × 依赖 hover,MCP 无 hover
  动作（057 账本在案）留用户实机。**场景2/3（workspace dropdown、settings）
  未开工**——依赖与场景1 相同的解释器绑定通道,建议 D-GAP 修复后再做。
  顺手修分支既有红两处（4700cdc 引入:getQuestions(.msg)→.rm TS2339;
  UserMessage use.web 指 .at 源模块→use 组件别名 TS2305）。
  提交 auto-musk-dev-1 e3fd1fe+2a216f8。
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
6. **VM 编译轨 alert-dialog 关闭语义**：验收标准 1 原文"ESC/遮罩/取消均可
   关闭"与 shadcn 语义（alert-dialog 仅 cancel/action 关）冲突——533 复审
   已裁定以代码为准（ESC/外点不关）。本计划 T4 按代码语义验证通过。
7. **（T4/T5 执行期发现 2026-09-04）后台 bash 起 VM 窗口 surface 必坏**：
   Git Bash 后台任务直接 spawn `auto run --render=vm` 时 iced 窗口
   "Error Other when presenting surface"——应用逻辑/MCP/渲染循环正常但窗口
   永不上屏,且对此实例请求 MCP 截图会触发 wgpu offscreen texture panic
   整进程崩溃。**PowerShell Start-Process（独立控制台）起窗则正常**。
   dev-stack.mjs 的 spawn detached 需复核此路径（S1 验收环境硬化的工具债
   增补）。
8. **（T6/T7 盘点定案 2026-09-04）编译轨未实现家族清点**：tooltip /
   hover_card / select / combobox / toast(sonner) 视觉在 a2r 编译轨全部
   未实现（schema iced:"none" 252 条的内容在树挂载无浮层）。dropdown 的
   ESC/外点 dismiss 也未接（shadcn 应关）。→ 建议归口 auto-lang 二期家族
   计划（PLAN-536 反应性专项或 534-vm-widget-family-parity,待用户裁定），
   本计划 musk 消费回归（T9）不依赖它们。
9. **（T9 实机 blocker 2026-09-04）解释器轨 alert-dialog open 绑定不渲染
   （musk 上下文）**：`auto run --render=vm` 现走 "vm+vm merged → VM
   interpreter UI"（gallery/musk 同模式）;gallery /alertdialog 模态渲染
   成立,musk chats_view 的 alert-dialog 四形态探针全灭（slot 深嵌/视图
   根部/字面量文案/空 trigger）——状态翻转正常（delete_confirm_open=true）
   但 vtree 无 modal 节点,autoui_check 显示各视图 widget 均走 "unknown
   tag → Column fallback"。疑解释器 fallback 路径的 bindings 解析断链,
   D-GAP 深水区。**处置**:VM 兜底内联行返场（web 不受影响）;根修归
   auto-lang——**已立项为 PLAN-536 题⑥**（2026-09-04 用户裁定追加:
   变更摘要⑥/目标⑥/执行步骤 T8 根修+T9 musk 联测/待澄清③,含本节全部
   探针证据交叉引用）;修复后删兜底行+CSS 抑制,并重启场景2/3。
   另:dev-stack 后台起窗 surface 必坏（presenting surface 错误→窗口不上
   屏,MCP 截图触发 wgpu panic 崩进程）,PowerShell Start-Process 独立
   控制台起窗正常——S1 工具债增补;窗口还会最小化启动（-32000 坐标）,
   需 ShowWindow 还原后再取证。
