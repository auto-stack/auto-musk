---
plan_id: PLAN-050
status: execution_done
feature_name: VM/Vue 渲染一致性第一批（主导航栏/设置界面/文件夹选择/会话二级导航）
author: [zhaopuming]
created_at: 2026-08-29
updated_at: 2026-08-30

supersedes_spec_components: []
new_spec_components:
  - "src/front/login.at: password 掩码声明 + onenter 提交（VM 源级对齐）"
  - "src/front/settings_menu.at: VM 内嵌展开降级类串（vue scoped CSS 同值镜像）"
  - "auto-lang ui（经 fold）: 内容子树 flex 行/列方向+单侧边框+图标组件注册臂+i18n_lookup 查表+input 稳定 Id"
touched_goals:
  - "VM/Vue 渲染一致性第一批：主导航栏/设置/文件夹选择/会话二级导航四界面 VM 验收"

current_step: 12
total_steps: 12
---

# [PLAN-050] VM/Vue 渲染一致性第一批——主导航栏/设置界面/文件夹选择/会话二级导航

## 变更摘要

2026-08-29 VM/Vue 双轨对比实测（auto-musk-dev-1 批次 + 本会话调查）确认：VM(iced) 轨与
Vue 轨的视觉差距全部卡在 auto-lang iced 渲染器能力缺口，musk 侧单一真源无法绕开
（同一份 .at 类串已在 vue 轨渲染正确）。本计划按用户圈定范围做**第一批一致性**：
主导航栏、设置界面、文件夹选择界面、会话二级导航——其余界面（消息气泡/markdown/
gate 卡等）留待后续批次。

调查已定责的渲染器缺口（证据在案）：

- **C1 内容子树按钮无宽度/对齐**：renderer.rs Button 臂对 content-subtree 按钮不消费
  `w-full/justify-start/items-start/text-left`，iced Button 内容恒居中（"iced 无 flex"
  原文注释，Plan 057 仅双固定尺寸图标钮特例）→ 导航项文字居中、不撑满。
- **C2 单侧边框不渲染**：border-b/r/t/l 仅表格行一个特例（table_row_rule 1px 填充条，
  Plan 411 P2-A④），任意容器不渲染 → rail 标题线/列表头线缺失。
- **C3 slot 子树不渲染**：KD-048 UPSTREAM①（NavSidebar slot(list/actions) 在 VM 缺失）
  → 会话二级列表整体不可见。
- **C4 透明度色不支持**：`bg-primary/10` 形态无解析 → 激活高亮不可见。
- **C5 VM 图标空**：ports 仅 icons.web.at，无 VM 侧图标桥（KD-047，P038 icons_data
  数据已在库 52 枚）→ rail 导航图标空缺。
- **C6 浮层无形态**：设置下拉用 CSS `position:absolute; bottom:100%`（settings_menu.at:91），
  iced 无对应映射 → 下拉不可用/不可见。
- **C7 文本插值臂缺失**：`${currentName}`/`t()` 裸调用恒空（KD-048 UPSTREAM④）→ rail
  底部工作区名裸露。

## 目标

1. 四界面 VM/Vue 逐项对拍一致（清单化验收，见 验收标准）：
   ①主导航栏（标题行 48px+底线、4 导航项左对齐+图标+激活高亮、底部工作区行）；
   ②设置界面（下拉面板展开/主题 accent 切换可用）；
   ③文件夹选择界面（形态以 T2 勘察裁定为准）；
   ④会话二级导航（NavSidebar 头 + NavListItem 卡片列表可见、选中/悬停/删除可达）。
2. vue 轨零回归（同一份 .at 源，门禁全绿）。
3. 渲染器能力以**最小映射集**落地（只实现四界面消费的组合），每项 TDD 先红后绿。

## 架构方案

| 能力 | 落点 | 方案 |
|---|---|---|
| C1 按钮宽度/对齐 | auto-lang `crates/auto-lang/src/ui/iced/renderer.rs` Button 臂 | content-subtree 路径消费 IcedStyle 的 width(Full→Length::Fill)/text_align(Left→align_x Left)/justify-start/items-start；flex-col→Column 包装最小映射 |
| C2 单侧边框 | 同文件 + `crates/auto-lang/src/ui/style/class.rs` | 推广 table_row_rule 的 1px 填充条为通用 border-b/r/t/l 容器包装（border-border 色经 resolve_border_rgb） |
| C3 slot 渲染 | 同文件 slot/children 臂（T5 内 grep 定位缺陷点） | slot(name:) 子树按父作用域展开进宿主元素（先例=NavSidebar 已在 vue 轨正确） |
| C4 透明度色 | class.rs 色解析 | `bg-<token>/<nn>` → Color with alpha（resolve token rgb） |
| C5 VM 图标 | musk `src/front/ports/icons.vm.at`（新增）+ renderer svg 臂 | 经 platform 协议声明（沿 icons.web.at 形状）；渲染读 icons_data.at 的 52 枚路径数据（P038 产物） |
| C6 浮层降级 | musk `src/front/settings_menu.at` + renderer（如需） | 定型"VM 下拉=inline 展开/收起"降级形态（沿 mention_dropdown 的 teleport VM 降级先例），不做 iced 真浮层 |
| C7 插值臂 | renderer/求值臂 | `${ident}` 与 `t(key)` 在 text 节点求值（最小集：rail currentName；`title: t()` 属性位不在范围） |

执行流（AGENTS 规则）：auto-lang 改动在其仓 worktree `.worktrees/auto-musk-dev`
（分支同名，沿 P048 先例 TDD 先红后绿 + lib 全量回归 + no-ff 合并 master）；
musk 侧验收在 `.worktrees/plan-050-dev`；VM 探针读主检出固定路径，合并后主检出
复跑为终态验证。每能力项独立提交，便于逐项回滚。

## 技术栈

auto-lang（iced renderer.rs / class.rs / slot 臂 / icons VM 桥；只动渲染与样式解析，
不动 codegen 契约）+ auto-musk（ports/icons.vm.at、settings_menu.at 降级形态、验收
对拍；web/ 冻结不动、backend/ 不动）。环境：release auto.exe（debug 版 RC canary
阻断主界面渲染——KD-048-b 同族，本计划不含其修复）。

## 需求分析与背景调查

> 来源：2026-08-29 双轨对比实测 + 渲染器源码勘验（证据会话在案），spec ledger
> 脉络：P038（icons_data）→ P044（VM 后端桥）→ P045（VM-clean 源）→ P047（首跑）
> → P048（数据桥/UPSTREAM①④登记）→ P049（双轨样式收敛）。

- VM 跑最新代码已证：autoui_vtree 含 822dbaa 引入的嵌套行结构；`auto run --render=vm`
  每次启动重读主检出 src/front。
- 四界面现状：rail 导航项居中无图标无高亮、标题行无底线；设置下拉不可见/不可用；
  会话二级列表整体缺失（C3）；文件夹选择界面现状未勘察（T2 裁定）。
- web 轨基线：同级类串渲染正确（浏览器实测 sameBox/sameFont/justify 逐项过），
  故对拍目标=VM 侧复现 vue 轨形态，而非改源迁就。
- 已知不在本计划范围：debug RC canary（KD-048-b 族）、VM 静默退出（KD-048-a）、
  消息气泡/markdown/gate 卡一致化、内部滚动双层同步。

## 详细设计

逐能力的设计要点与边界：

- **C1**：IcedStyle 已带 width/height/text_align（label 路径已消费）；本项把消费面
  扩到 content-subtree：①width Full→button.width(Fill)；②text_align Left→内容
  align_x(Left)；③`items-start`→内容 align_y(Top)；④`flex-col`→内容 Column 包装。
  不做通用 flex，仅映射四界面消费的组合；超出组合仍走现行为（居中）。
- **C2**：class.rs 增单侧边框解析（border-b/r/t/l→StyleClass 变体）；renderer 容器
  发射时按边生成 1px 填充条子元素（横边=横向 Space 线，纵边=纵向），颜色统一
  resolve_border_rgb（border-border token）。
- **C3**：先 `grep -n "slot" crates/auto-lang/src/ui/iced/renderer.rs` 与 ui_gen 定位
  slot 在 VM 臂的丢失点（KD-048-①只记现象未记点位），修复=把 slot 子树以父作用域
  求值后并入宿主 children；NavSidebar(list/actions) 作回归样例。
- **C4**：class.rs 色解析增 `/<alpha>` 后缀（百分比→u8 alpha），token rgb 经现有
  resolve 路径。
- **C5**：沿 platform 协议（P028）：icons.at 声明形状，icons.web.at 现实现，新增
  icons.vm.at 返回 icons_data 路径数据；renderer 侧 `component X from ports` 在 VM
  臂按 svg path 直绘（或 Text 富文本兜底，以勘察裁定，倾向 svg 直绘）。
- **C6**：单一真源下不做 VM 分支 → 采用"类串即语义"：VM renderer 把
  `position:absolute` 的容器降级为文档流内展开（宽 100%），配合 settings_menu.at
  既有开合状态（isOpen）实现展开/收起——行为等价下拉、形态为面板内嵌；含
  settings_menu.at 类串微调（worktree plan-050-dev）。
- **C7**：text 节点求值臂增 `${ident}`（读当前 store 作用域 ident）与 `t(key)`
  （i18n VM 读取路径勘察，缺则最小内置 zh/en 查表）。范围钉死 rail `${currentName}`。

## 测试设计

- auto-lang：每能力 `plan050_*` 前缀单测（class.rs 解析断言沿 1730 行先例；
  renderer 断言沿 plan340_tests 邻域先例），`cargo test -p auto-lang plan050_` 绿 +
  `cargo test -p auto-lang --lib` 全量绿（3746 基线）。
- musk：vue 三门禁（`auto build` strict 零 error、`npx vitest run` 23+1、
  scripts/lib-parity 对拍 30/30）+ style-parity 58 用例 + `scripts/vm-link-probe.cmd`
  PASS + `node scripts/vm-first-run.mjs` alive reds=0。
- 四界面验收：`autoui_screenshot`（VM）与浏览器截图并排，按 验收标准 清单逐项勾。

## 验收标准

1. 主导航栏：标题行 48px 带底线；4 导航项左对齐、含图标、激活项有高亮底色；
   底部工作区名显示真实值（非 `${currentName}`）。
2. 设置界面：齿轮点开下拉面板可见（VM 降级形态）、主题 mode/accent 切换即时生效。
3. 文件夹选择界面：按 T2 裁定的形态在 VM 可达可操作（若裁定为浏览器原生能力依赖，
   则以"VM 显式降级形态+登记"作为通过态）。
4. 会话二级导航：列表可见、卡片形态与 vue 一致（左对齐/两行/选中描边）、点击选中
   生效、悬停删除钮可见可点。
5. vue 轨零回归（上述门禁全绿）；vm-link-probe PASS；first-run alive reds=0。
6. KNOWN-DEBT-AND-RISKS.md 增 050 行：四能力落点 + 未覆盖项（其余界面）留档。

## 执行步骤

- [ ] **T1** VM 四界面现状取证：起 release VM（`AUTO_BACKEND=http://127.0.0.1:8081
  AUTOUI_MCP_PORT=9250 auto run --render=vm`），`autoui_screenshot`/`autoui_inspect`
  逐界面截图存 `tmp/plan050-survey/`；产出 `tmp/plan050-survey/gaps.md` 差异表
  （每界面行：现状/依赖能力号 C1-C7）。验证：4 截图 + gaps.md 存在且含 4 行。
  [✅ 已完成] tmp/plan050-survey/{01-rail,02-settings-open,03-folder-picker}.png +
  04-session-nav.vtree.txt + gaps.md（4 界面行全，依赖能力号逐项标注）。
- [ ] **T2** 文件夹选择界面形态裁定：grep musk 源定位该界面（WorkspaceSelector/raw
  路径输入/原生 input），对照 vue 轨形态写裁定行入 gaps.md（若依赖浏览器原生能力
  → 记 VM 显式降级形态为本计划通过态）。验证：gaps.md 该行含裁定与依据。
  [✅ 已完成] 界面身份=WorkspaceSelector（web 原生逃生舱，Plan 407）；裁定=VM 通过
  态为内嵌展开列表降级（数据同源 vue 轨，选后写 musk_workspace + 显示真实名），
  不做 iced 原生文件夹对话框；依据已写入 gaps.md T2 节。
- [ ] **T3** C1 按钮宽度/对齐（auto-lang worktree）：renderer.rs Button content-subtree
  臂消费 width(Full)/text_align(Left)/items-start/flex-col；新增 plan050_* 单测先红
  后绿。验证：`cargo test -p auto-lang plan050_` 绿。
  [✅ 已完成] auto-musk-dev@7fed34b：plan050_content_align 最小映射（justify-content
  优先、text-left 兜底、items-start→垂直），Button 臂 Fill 容器承载；w-full→Fill 为
  既有路径。注意：单测需 `cargo test -p auto-lang --features ui-iced plan050_`（ui-iced
  门控），4 绿。
- [ ] **T4** C2 单侧边框 + 容器对齐最小集：class.rs 解析 border-b/r/t/l 与
  items-*/justify-*；renderer 容器发射 1px 填充条与 align_x/align_y；单测同上。
  验证：`cargo test -p auto-lang plan050_` 绿。
  [✅ 已完成] auto-musk-dev@26220179：BorderBottom/Top/Left/Right 变体+解析+IcedStyle
  旗标+apply_side_borders 1px 填充条（推广 table_row_rule），挂钩 build_row/column/
  container 三发射点；容器 items-*/justify-* 消费经查 iced 侧已有（renderer 1471-1489/
  1664-1672），未重复实现。plan050_ 5 绿。
- [ ] **T5** C3 slot 子树渲染：grep 定位 slot 丢失点→修复→NavSidebar(list/actions)
  冒烟单测（slot 内 button 可见性断言）。验证：`cargo test -p auto-lang plan050_` 绿。
  [✅ 已完成] 由上游 Plan 476 承接（用户携 009 需求说明立项）：SlotFills 父作用域
  捕获 + outlet 渲染臂 + 五容器拼接,slot 单测 16 绿（c45a5e237 合入 master）。
  auto-musk-dev worktree 合并 master（8bc51ed4,renderer tests 锚点冲突保留双侧）,
  worktree release 构建 + VM 实测：会话二级导航完整恢复（真实数据卡片列表,
  tmp/plan050-survey/06-rail-slot-batch.png）。无降级补丁需拆除（NavListItem
  组件形态本就在位）。
- [ ] **T6** C4 透明度色：class.rs `bg-<token>/<nn>` 解析 + 渲染 alpha；单测。
  验证：`cargo test -p auto-lang plan050_` 绿。
  [✅ 已完成] auto-musk-dev@2411c5398：解析+alpha 上游已有（待澄清 5,Plan 409 在库），
  实机高亮已现（待澄清 8）；本项新增 plan050_bg_alpha_survives_to_iced_style 钉住
  from_style/convert_color 渲染侧 alpha 不拍平（10%→25/255、50%→127/255）。
  `cargo test -p auto-lang --features ui-iced plan050_` 6 绿。
- [ ] **T7** C5 VM 图标桥：musk 新增 `src/front/ports/icons.vm.at` + auto-lang VM 臂
  svg 直绘（数据取 icons_data.at）；单测。验证：同上 + rail 图标在 VM 截图可见。
  [✅ 已完成（代码+单测；VM 实证随 T10）] auto-musk-dev@32cb5e78e：勘察裁定走既有
  lucide:/resvg currentColor 直绘管线（Plan 408/442 在库），icons.vm.at 数据桥无需
  另建。①builder 图标组件臂：use.web component 声明名→View::Image{lucide:kebab,
  size→固定像素}（此前 unknown fallback→Empty）；②renderer lucide 补 27 枚
  glyph（路径数据取 musk icons_data.at 0.460.0 单一真源）。先红后绿；
  plan050_ 10 绿，`--lib` 3905 绿。
- [ ] **T8** C6 设置下拉 VM 降级形态：renderer absolute 容器降级文档流展开 + musk
  settings_menu.at 类串微调（worktree plan-050-dev）；单测 + 实机点开验证。
  验证：VM 截图下拉面板可见且切换生效。
  [✅ 已完成（代码+单测；实机点开随 T10/T11）] 勘察：absolute 在 iced adapter
  即 "store but will be ignored"（在库），降级天然成立，renderer 零改动。
  ①auto-lang e16b7ce29：plan050_absolute_utilities_degrade_to_inline_flow 钉
  契约（定位类解析不炸+外观类存活）。②musk plan-050-dev@35b859b：settings_menu.at
  类串即语义微调——面板/分区/标题/模式钮/色板/主题行补工具类（镜像 scoped CSS
  值，vue 浮层不变）；VM 丢弃定位类=内联展开；色板内联 background VM 不解析呈
  中性圆（降级登记，切换仍可点）。
- [ ] **T9** C7 文本插值最小集：`${ident}`/`t(key)` text 臂求值（rail currentName
  打通）；单测。验证：VM rail 显示 musk-demo。
  [✅ 已完成（代码+单测；VM 实证随 T10 release 重装）] auto-musk-dev@ff7eb1261：
  根因三处断链全修——①engine decode_tagged_nv 补 is_null 臂（null nv 曾落兜底
  Int(0)，`.x != None` 恒真而 `.x.field` 落空=rail 裸串根因）；②AWAIT_FUTURE
  改 pop_nv/push_nv 保标签（曾把 null 拍平 Int(-2147483647)）+ future body
  结果改 nv 解码；③t()/i18n.t() 提取臂（曾产畸形元素树文本恒空）+ $t() 模板
  查表 + i18n_lookup（front 根 i18n/{lang}.json 平铺装载，AUTO_LOCALE 默认
  zh）。corpus plan050_stub_nil 全链回归 + plan449 探针 border-r 翻 ok。
  plan050_ 9 绿；`--lib` 3904 绿。
- [ ] **T10** auto-lang 收口：`cargo test -p auto-lang --lib` 全量绿 + no-ff 合回
  master + 主检出 release 重装（`cargo build --release -p auto`）+
  `scripts/vm-link-probe.cmd` PASS + first-run alive reds=0。验证：命令全过。
- [ ] **T11** musk 验收对拍：四界面 VM/浏览器截图并排核对（验收标准 1-4 逐项），
  必要 .at 微调在 `.worktrees/plan-050-dev`；对拍材料存 `docs/plans/050-review/`。
  验证：清单 4/4 勾完。
- [ ] **T12** 全量门禁 + 收尾：vue 三门禁 + style-parity + 探针 + first-run 全绿；
  KNOWN-DEBT 增 050 行；worktree 折叠（auto-musk-dev 与 plan-050-dev 合回各自
  master/main 并清理）。验证：门禁输出贴计划复审记录。

## Phase 2 复活清单（2026-08-30 用户实测，052 nav-item 化后的新回归+未覆盖面）

1. 标题行 `v0.1.0` 字体上漂（未纵向居中）——items-baseline 的 VM 降级
   （ItemsStart）缺字号配对的纵向校正；方向：行高固定 + center_y 或 leading 补偿。
   [✅ 已修] auto-lang 318d011a8：降级目标 ItemsStart→ItemsCenter（行内纵向
   居中近似，配对测试随断言更新 _to_center）。
2. nav-item（052 组件化）文字+图标重新居中——nav-item 的 VM 臂未走 C1 的
   content-subtree flex 行修复（或 NavItem 组件路径绕过 button 修复面）；
   方向：nav-item VM 臂消费 justify-start/items-center。
   [✅ 已修（双修互补）] ①并行会话 master b5aaf7f54（6d51cf092）
   plan414_content_alignment：高度臂让位显式对齐类（text-left/justify-*），
   含实机截图 rail 四项左对齐取证；②本批 nav-item 三 preset 补 justify-start
   （NavItem.vue 资产同步；web 侧 justify-start=flex 默认值零视觉变化，
   nav_contract_matches_scaffold_assets 复绿）。
3. rail 底部工具栏不贴底——`.at` 的 `mt-auto` 无 iced 映射（margin 系静默跳过）；
   方向：mt-auto → 列内弹性占位（Fill spacer）或 col space-between 映射。
   [✅ 已修] auto-lang 318d011a8：Column 发射时对 mt-auto 子项前置 Fill 弹性
   占位条。实现细节：master 通用 mt-<size> 解析已产 MarginTop(SizeValue::Auto)
   （前会话专用 MarginTopAuto 变体被通用路径抢先成死代码，已删去重），
   plan050_mt_auto_spacer 决策函数 + plan050_mt_auto_child_gets_fill_spacer
   全链回归钉（mt-auto 命中/mt-4 与空类不命中）。
4. 选择工作目录面板未做降级类串——workspace_selector.at 的 ws-panel 系仍是裸
   CSS 名（T8 只覆盖了 settings_menu.at）；方向：复制 T8 做法补工具类。
   [✅ 已修] 98006de（2026-08-30，已进 main）。
5. settings 触发钮无齿轮图标且挤扁——`Settings { size: 16 }` 子组件形态的图标
   未渲染 + 按钮无尺寸类坍缩；对照 widgets-gallery 查 `button(variant="icon",
   icon="xxx")` 的 VM 支持面与 icon 子组件臂的命中条件（Settings 在 46 名单内）。
   [✅ 已修] 98006de（2026-08-30，已进 main；触发钮尺寸/居中工具类）。

### Phase 2 收口记录（2026-08-30）

- #1/#3 落点全在 auto-lang（前会话已在 `.worktrees/plan-050-p2` 实现三项修复
  但未合回）；本批收口：并入最新 master（两处冲突正交并存——mt-auto 占位 ×
  Plan 490 wrap_layout_onclick；plan414 × plan050_mt_auto_spacer）、mt-auto 与
  master 通用路径去重、NavItem.vue 资产同步，合入 master **318d011a8** +
  release 重装完成。
- 验证：`cargo test -p auto-lang --features ui-iced plan050_` **13 绿**；全量
  `--lib` 差分**零新增**（master 基线 8-9 败 dock/notif/settings/stage3 flaky
  族持平，双仓对跑实证）；`vm-link-probe` **PASS 61419B**（历史 61162B）；
  `vm-first-run`（release）**alive reds=0**。
- 残留：#1/#3 的 rail 实机像素目验留用户（本环境图像通道限制，项目惯例）；
  #2 已有并行会话实机截图取证。

## 复审记录

- 复审人：ZCode（/auto-plan:review），2026-08-30。执行 worktree 已折叠，对照默认检出复核。
- 判定：
  1. 主导航栏 ✅ pass——t11-1-rail.png：底线/左对齐+图标/激活高亮/真实工作区文案
  2. 设置界面 ⚠️ partial——面板可见 ✓（t11-2-settings.png，C6 落地）；mode/accent
     切换即时生效未独立复验（按钮可达，VisualStore 标准链路，051 已深验同族）→
     唯一待复验项，随下轮实机一次点击补证
  3. 文件夹选择 ✅ pass——t11-3-ws.png：T2 裁定的内嵌展开降级形态
  4. 会话二级导航 ✅ pass——卡片两行式/选中描边/左对齐（悬停删除钮：051 P2 已
     以 hidden 落地 VM 降级，见 1eb2e38）
  5. vue 零回归 ✅ pass——六门禁全绿（build strict/vitest 23+1/leaves 30/30/
     style-parity 12=基线/probe 61162B/first-run reds=0，master 6f7189bce）
  6. KNOWN-DEBT 050 行 ✅ 终态入册
- 遗漏/延后/工作量扫描：浏览器侧对拍截图延后（登记）；gate 卡裸串族延后（登记，
  后续批次）；无未登记 workaround。
- 结论：仅剩 2 的一次点击级复验，补证后翻 reviewed；其余全 pass。


## 待澄清事项

1. 文件夹选择界面若为浏览器原生能力（input/directory picker），VM 侧等价物
   （iced 文件对话框）可能超本计划体量——T2 裁定，必要时降级形态通过。
2. C1/C4 的"最小映射集"边界：仅覆盖四界面消费的组合；新组合出现时按需追加，
   不承诺通用 flex/色板。
3. icons VM 桥的渲染方式（svg 直绘 vs Text 兜底）由 T7 勘察定，倾向 svg 直绘。
4. **T5 执行受阻（2026-08-29）**：C3 不是"缺陷修复"而是"特性缺建"——VM 前端管线
   （aura_view_builder → iced renderer）完全没有 slot 替换机制（`"slot"` 处理仅在
   aura/types+schema、trans/rust、ui_gen/vue 存在）；render_child_widget 调用点直接
   `return`，调用位 slot 填充从未传入。正确实现=跨作用域内容模板求值（父作用域求值
   填充子树 + 子视图槽位拼接），体量为独立特性批次。**已裁定走 (a)**：需求说明已出
   → `docs/designs/009-vm-slot-substitution-requirement.md`（携往 auto-lang 立项）。
   界面④在本计划的通过态：等上游批次，或 T11 以 .at 内联 NavSidebar 壳兜底（届时
   视上游进度裁定，兜底补丁在该批次合入后拆除）。
5. **T6 发现（2026-08-29）**：`bg-primary/10` 解析+alpha 上游已支持（Plan 409 §10
   回归测试 test_semantic_color_with_alpha_is_dark_aware 在库，alpha 25 主题色断言）。
   实测激活高亮仍不显示——根因移至 `style: if` 条件求值（current_view 字符串等值）
   或 build_button_style 应用层，需另查。本项不再新增解析代码。
6. **T10 前置条件**：auto-lang 主检出有未提交改动（renderer.rs/ui_gen/vue.rs），
   折叠 auto-musk-dev 分支前需用户先提交/stash，否则 merge 会被本地改动拒绝。
7. 自动生成的 NavListItem.vue 等产物按 gitignored gen/ 处理（T3 前批已核 gen/ 除
   少数强制跟踪的 spec 文件外均 ignore），产物同步走文件复制。
8. **C1 残余（2026-08-29 实测）**：T3 合入 Plan 476 后，VM 激活钮底色已撑满但文字
   仍居中——plan050_content_align 包装未生效于该路径，疑点=Button 臂 iced_style
   附着或 content-subtree 判定；T7-T9 续跑时先以运行期日志定位此项（计划内 T3 的
   收尾细查,非新能力）。
9. **T9 根因记录（2026-08-29）**：rail 裸 `${currentName}` 根因不在求值臂而在 VM
    三处断链——①decode_tagged_nv 无 is_null 臂（fn return None 落兜底 Int(0)，
    `.x != None` 恒真而 `.x.field` 落空）；②AWAIT_FUTURE pop_i32/push_i32 拍平
    nanobox 标签；③t()/i18n.t() 文本提取产畸形元素树（文本恒空）。全部已在
    auto-musk-dev@ff7eb1261 修复（详见 T9 标记）。i18n 查表约定：front 根
    i18n/{lang}.json 平铺装载，AUTO_LOCALE 默认 zh；运行期切语言不在本批。
10. **登录串扰+掩码（2026-08-29 用户实测回归）**：①password 明文——login.at 缺
   `password: true` 声明（convert_input 双臂本就消费），plan-050-dev@4e184b9 补上
   即掩码生效；②password 击键追加进 user 框+光标双闪——iced text_input 无显式 Id
   共享内部编辑器 State，auto-lang@1a8516b5b 以 placeholder+width+password 派生
   稳定 Id 逐框区分（视图消息归属/派发回写两探针均正确，错位在 widget 层）。
   待用户在最新实例实测确认。②的 Id 派生补丁经二分验证**不充分**：掩码已修
   （***），但双焦点仍在（password 击键追加进 user 框，两框同时带焦点环）。
   对照实验排除输入管线回归——converter（双输入）在 8bc51ed4b（T3+476）与
   T9/Id 各构建上行为一致、联动正常（用户实测+快照双重确认），串扰为 musk
   登录页特有：`if !authenticated` 条件下子 widget 双 text_input 双焦点、键盘
   事件双投递（各框各自追加）。定性：iced widget 焦点层缺陷，KD-048 同族
   上游债，需独立批次定位（下一样点：最小双输入+条件子 widget 复现例）。
   **已裁定走上游批次**：需求说明已出 →
   `docs/designs/011-vm-text-input-double-focus-requirement.md`（携往 auto-lang
   立项，含最小复现样点/根因方向/验收清单/伴生的 MCP autoui_type id→action
   对位错位顺修项）。③另发现 MCP autoui_type 的 UiNode path→id 对位
   错位（password id 派发 UsernameChanged）——agent 工具层债，非四界面范围，
   不修留档（auto-musk-dev 诊断记录在案）。
11. **T7 偏差记录（2026-08-29）**：icons.vm.at 未建——渲染走既有 lucide:/resvg
    管线（Plan 408/442），glyph 数据以 musk icons_data.at 为源镜像进 renderer
    bundle（0.460.0 同源对拍）。未来 VM 侧若需运行期取 icon 数据再立项桥接。
