---
plan_id: PLAN-082
status: archived
feature_name: VM Rich 文本吞噬根修（080 移交债②接收计划：表格空文本+段落缺失）
author: [agent]
created_at: 2026-09-21T00:00:00Z
updated_at: 2026-09-21T20:55:00Z
plan_revision: 2
current_step: 5
total_steps: 7
supersedes_spec_components: []
new_spec_components: [docs/specs/modules/autodown-consumption.md]
touched_goals: []
---

# PLAN-082 — VM Rich 文本吞噬根修（080 移交债②接收计划）

## 0. 变更摘要

修复 VM（iced）轨 markdown 富文本内容整体不可见的根因：auto-lang
`iced/renderer.rs` 的 `convert_view_messages`（VM 动态视图渲染主链必经桥）
缺 `View::Rich` 显式臂，兜底 `_ => AbstractView::Empty` 把每个 Rich 节点
静默摊平为空。凡 `render_inlines` 产出 Rich 的内容——markdown 段落、表格
单元格、引用正文、列表项正文——在 VM 上整体消失。修法=补 Rich 直通臂+
修复随 Rich 形态改版而失效的两个既有单测+新增转换链回归钉，真机像素验收
后销 KNOWN-DEBT Plan 080 行②与 PLAN-081 §10 登记。

- 主改动仓：**auto-lang**（一个函数臂 + 测试）；musk 仓仅文档销项。
- 类型：缺陷根修（非新功能）；Rich 基础设施（View 变体、into_iced 臂、
  vnode/snapshot 映射）080 已备齐，本计划接通最后一个断点。

## 1. 目标

### 目标

1. VM 轨 markdown 富文本全形态可见：段落（含粗体/斜体/行内码/链接混排、
   CJK）、表格单元格文本、引用正文、列表项正文。
2. 关闭 KNOWN-DEBT-AND-RISKS.md Plan 080 行②「表格单元格空文本观察项」
   与 PLAN-081 §10 登记的「autodown 表格/粗体段落正文渲染债」（081 r6
   用户实机实锤：「目录内容」表格只画行边框单元格全空；「README 一句话
   总结」粗体+行内码 CJK 段落整段缺失）。
3. 防复发：View 枚举新增变体时，VM 动态转换链（convert_view_messages）
   漏臂=静默 Empty 的缺陷模式获得回归钉与规范约束。

### 非目标

- §10-9 VM 实例间歇死亡家族（080 移交①，独立接收路径）。
- §10-10 字体度量族（text-* line-height 映射等，080 移交③）。
- MCP 操作面对 MaxWidthPct/Rich 子树的 Find 不可见（§10-11 伪影家族，
  独立专项；本计划只保渲染面与 vtree 文本面）。
- iced Rich 的排版增强（跨 span 折行之外的任何排版调优）。
- web/Vue 轨（不经过 View 枚举，无此缺陷，零改动）。

### 成功样态

用户在 VM 实机（launch-vm.cmd 双击链）打开含表格与富文本段落的助手消息，
表格单元格文本、粗体/行内码 CJK 段落全部可见；`cargo test -p auto-lang
--features autodown --lib` 该模块零红且差分不劣于基线。

## 2. 架构方案

### 渲染链与断点定位（调研实证）

生产链（VM 动态视图，每帧）：

```
musk .at autodown widget (renderer.vm.at)
  → aura_view_builder.rs:2097 "autodown"|"markdown" 臂
  → render_document_streamed_with (autodown_render.rs:209, StreamCache 按 path 键)
  → View 树（段落/表格 cell = View::Rich；标题/hr/marker = View::Text）
  → convert_view_messages (iced/renderer.rs:7309)   ← ★ 断点：无 Rich 臂
  → render_dynamic_view (renderer.rs:24677)
  → into_iced Rich 臂 (renderer.rs:3601)            ← 生产链永远零命中
```

`convert_view_messages` 兜底臂 `_ => AbstractView::Empty`（函数尾，
renderer.rs ≈7801）把 Rich 替换为 Empty。`AbstractView` 是
`use crate::ui::view::View as AbstractView`（renderer.rs:8）的别名，
故 Rich 在此被吞是编译期事实，与运行态无关。

### 证据链（2026-09-21 调研，全部可复核）

| # | 证据 | 位置/命令 |
|---|---|---|
| E1 | convert_view_messages 全函数（7309–7802）零 `Rich` 匹配，兜底臂 `_ => AbstractView::Empty` | `awk 'NR>=7309 && NR<=7904 && /Rich/'` 零命中 |
| E2 | 生产主链与 face 链均经该函数 | renderer.rs:21319（face）、21650 附近（主路径，`view_with_debug_gated` → 同函数） |
| E3 | 080 期「into_iced Rich 臂零命中实证」的真正机制：Rich 在上游 convert 层已被吞 | 77729f830 提交辞 + E1/E2 |
| E4 | 离线探针：render_dynamic_view 直构路径（绕过 convert 层）Rich 布局正常（行高 20.8px 非零），但 3 块文档仅 Heading 有 text 候选——直构活、生产死 | `cargo test -p auto-lang --features "autodown iced-layout-tests" --lib plan080_uat_fuat2 -- --nocapture`（plan080_uat_fuat2_rich_paragraph_geometry） |
| E5 | master 上 autodown_render 两个单测红：期望 Row/Text 旧形态，实际 Rich（文本在 spans 中，测试断言未跟上 F-UAT-2 改版） | `cargo test -p auto-lang --features autodown --lib -- renders_heading_paragraph_inline_marks renders_table_headers_and_rows` |
| E6 | 症状分布与 Rich 覆盖面精确重合：标题（Text）可见、hr/表格边框（非文本 widget）可见、段落/表格 cell（Rich）不可见；vtree 同证缺失（vtree 从 iced 树导出，反映被吞后的树）；vnode/snapshot 面有 Rich 拼接映射（vnode_converter.rs:238、snapshot_builder.rs:81）故 MCP 快照面与 vtree 面曾互相矛盾 | musk KNOWN-DEBT 080 行② / PLAN-081 §10 / 081 r5-r6 实机记录 |
| E7 | musk 侧无罪定界（081 r6 已做）：content 拍平链完整（后端对账 516 字节原文），标题/hr 渲染=引擎收到全文 | PLAN-081 归档 §10 |

### 方案

1. **T-01 主修**：`convert_view_messages` 补 `View::Rich` 直通臂——
   `AbstractView::Rich { spans, style } => AbstractView::Rich { spans, style }`
   （RichSpanView 仅 content+style，无消息载荷需转换；与 Text 臂同款的
   最小直通）。顺带清理 into_iced Rich 臂中 `IcedStyle::from_style` /
   `effective_font_size` 的重复调用（renderer.rs:3612/3619，行为无变化）。
2. **T-02 测试修复与回归钉**：
   - 修 E5 两个红测试：断言更新为 Rich 形态（内容断言从 `text_of` 改为
     spans 拼接/逐 span 内容，样式断言随 span_class 现状）。
   - 新增转换链回归钉：`render_document`（CJK+粗体+行内码+表格样例）→
     `convert_view_messages` → 断言 Rich 节点存活且 spans 文本逐字保留
     （E1 缺臂回归一旦复发即红）。
   - 布局探针升级：plan080_uat_fuat2_rich_paragraph_geometry 从"文档化
     无断言"升为断言 text 候选覆盖三块（防 into_iced 臂再断）。
3. **T-03 构建与实机验收**：auto-lang worktree（`.wt/musk-082/auto-lang`，
   分支 `auto-musk-dev`，AGENTS.md 第三行规则）release 构建 → musk VM
   实机（launch-vm.cmd 标准序，主检出 musk 无代码改动）→ 像素+vtree
   双证验收矩阵（§6 测试设计）。
4. **T-04 收尾销项**：auto-lang KNOWN-DEBT §10-11 关联观察补记根因、
   musk KNOWN-DEBT Plan 080 行②销项、PLAN-081 §10 移交注记、规范增量
   落册（SD-01）。

### 风险与二阶防线

- **iced Rich 首次在生产链激活**，实机可能暴露二阶问题（Bold/Mono 字体
  覆盖 `..Font::DEFAULT` 的 CJK fallback、max-w 容器内折行、行高）。
  防线：T-03 验收矩阵逐形态覆盖；若个别形态异常，在 into_iced Rich 臂内
  就地修（不回退 convert 层直通）；若 Rich 整体不可用（低概率，探针 E4
  已证布局正常），显式降级臂 Rich→逐 span Text Row 保可见性并在 §10 登记
  （「散架但可见」优于「不可见」，且与 vue 轨差异转独立债）。
- **§10-9 死亡家族干扰验收**：实例可能中途死（外部终止类）。验收用
  one-shot 脚本模式（wait-MCP→find→screenshot 每步 retry），死亡即重启
  重跑，不误判为渲染回归。

## 3. 技术栈

- auto-lang：Rust / iced 0.14（`text::Rich`/`Span`）/ 既有
  autodown-core 解析层（零改动）。
- musk：零代码改动；文档（KNOWN-DEBT、plans）。
- 验收仪器：launch-vm.cmd 标准启动链、MCP `autoui_vtree`/`autoui_screenshot`、
  PowerShell System.Drawing 像素采样（阈值避开底色 ±12 容差教训）、
  iced-layout-tests 探针。

## 4. 需求分析与背景调查

### 授权记录

- 授权来源：用户 2026-09-21 指示「计划081留下了一个debt，是markdown的
  表格内容等内容不显示的问题。请调研并立项解决」。
- 授权范围：调研（已完，证据链 E1–E7）+ 本计划立项；执行需走
  /auto-plan:work（含 auto-lang worktree 建立与构建验收）。
- 关联裁定：PLAN-080 收口时用户裁定「A 立即修 / B 先收口 081」中选 B
  （081 归档已交付），本计划即 B 之后的接收计划；080 归档辞与 081 §10
  均明示「接收计划待建」。
- 预算：无用户指定预算；自动延续按 auto-plan 默认。

### 背景调查（关键事实）

1. **缺陷史（080 UAT 三阶段摇摆）**：7f2d94d74 引入 Rich（当日）→
   77729f830 回退 Row（「into_iced Rich 臂零命中」=本根因首证，但当时
   定位错层）→ 264992a8c 恢复 Rich（用户真机「一度解决了」，遗留表格
   cell 空白观察项）→ adcdb02ae 移除色回退层。**convert_view_messages
   自 Rich 引入起从未有过 Rich 臂**（`git log -S "Rich"` 全历史仅 5 提交，
   均不涉及该函数）。
2. **「一度解决了」与 E1 的表面矛盾**进 §10 待澄清（不影响立案：静态
   证据链闭环；候选解释=验证面混叠或过渡构建，T-03 真机验收顺带澄清）。
3. **musk 定界**（081 r6）：`/api/chats/session/{id}` 原文完整；
   sessions 列表=`/api/chats/sessions`、详情=单数（取错回退 SPA HTML
   的坑已在案）；Markdown 端口 renderer.vm.at 直通原生 autodown。
4. **测试门失效旁证**：E5 两红测试在 master 挂红未修=080 落地时
   `cargo test` 门未覆盖该模块（080 收口记录「cargo tf 3689/3689 全绿」
   指带 feature 组合的门，与 `--features autodown --lib` 全量口径存在
   缺口）；T-02 修复时用当前红作失败基线，修复后转绿。
5. **代码位置核对**（2026-09-21 master 实测）：本计划 §2/§6 引用的
   行号与符号均按 auto-lang master（57f43bb91）核对在案。

## 5. 详细设计

### T-01 convert_view_messages Rich 直通臂

位置：`crates/auto-lang/src/ui/iced/renderer.rs:7309` 函数体内，
AnchorSlot 臂（≈7479）与兜底臂之间任一显式位置，与既有显式臂注释风格
一致（「必须显式臂——否则掉进 `_ => Empty`」注释族，Grid/Popover/
MouseArea/video 同款教训）：

```rust
// F-UAT-2/PLAN-082: Rich 段落必须显式直通——此前漏臂被兜底 `_ => Empty`
// 摊平，markdown 段落/表格 cell/引用正文在 VM 动态路径整体不可见
// （080 期「into_iced Rich 臂零命中」真因）。RichSpanView 无消息载荷，
// content+style 原样透传。
AbstractView::Rich { spans, style } => AbstractView::Rich { spans, style },
```

顺带清理（同提交，行为无变化）：renderer.rs:3612–3620 into_iced Rich 臂
内 `let is = IcedStyle::from_style(st)` 与 `effective_font_size(&is)` 的
重复求值合并为一次。

### T-02 测试修复与回归钉

全部落 `crates/auto-lang/src/ui/autodown_render.rs` tests 模块与
`iced/renderer.rs` tests 模块（`#[cfg(feature = "iced-layout-tests")]`）：

1. `renders_heading_paragraph_inline_marks`（autodown_render.rs:971）：
   段落断言从 `View::Row { children: spans }` 改为 `View::Rich { spans }`，
   逐 span 内容/样式断言保留原语义（「粗」span font-bold、「码」span
   font-mono+bg-muted 等按 span_class 现状）。
2. `renders_table_headers_and_rows`（:1225）：`text_of` 对 Table
   headers/rows 的取文本改为 spans 拼接取值（复用新 helper）。
3. 新增 `convert_view_messages_preserves_rich`（renderer.rs tests）：
   `render_document`("中文 **粗体** 与 `码`。\n\n| 名 | 值 |\n| --- | --- |\n|
   甲 | 乙 |\n") → convert_view_messages → 递归收集断言：段落 Rich 存活、
   spans 拼接=「中文 粗体 与 码。」、表格 cell Rich 文本=甲/乙。
4. plan080_uat_fuat2_rich_paragraph_geometry（renderer.rs:34670）升断言：
   text 候选须覆盖 Heading+两段落（拉丁+CJK），防 into_iced 臂再断。

### T-03 构建与实机验收

1. worktree：`git worktree add D:/autostack/.wt/musk-082/auto-lang -b
   auto-musk-dev D:/autostack/auto-lang`（基于 master；musk 本仓零代码
   改动不开 worktree）。
2. 构建：worktree 内 release 构建 auto.exe；**铁律**（记忆在案）：每轮
   build 后 grep `successfully` 再起实例；构建产物 fresh 目录
   gen/front/vue/node_modules 会装 pnpm junction，收尾前
   `Get-ChildItem -Recurse -Attributes ReparsePoint` 枚举 +
   `[IO.Directory]::Delete($path,$false)` 清理后过 wt-guard。
3. 实机验收：主检出 launch-vm.cmd 标准序（musk 主检出无改动直接复用）；
   MCP 端口从启动日志 grep `AutoUI MCP: listening` 取真实口（9247 被
   auto-edit 占用时回退 9248+）；验收矩阵见 §6；截图落盘跟随 cwd 用完
   即清；窗口须可见（最小化截图失败在案）。

### T-04 收尾销项

- auto-lang：rebase master → `--ff-only` 合回 → worktree/分支/组目录
  清理（AGENTS.md 收尾流程；range-diff 若有 rebase 映射）；
  auto-lang KNOWN-DEBT §10-11 关联观察补根因注记。
- musk：KNOWN-DEBT-AND-RISKS.md Plan 080 行②加销项戳（指向本计划）；
  081 归档 §10 移交注记回填；ledger 刷新（merge 阶段技能承担）。

### 规范增量

| delta_id | add/modify/retire | target | before/after | rationale | acceptance |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/modules/autodown-consumption.md | before：双轨平权为原则性表述，未约定 VM 动态转换链的新变体接入点。after：新增「VM 动态视图转换链四站点契约」——View 枚举每新增变体必须同步四处显式臂：`convert_view_messages`（动态渲染）、`into_iced`（直构渲染）、`vnode_converter`（MCP VNode 面）、`snapshot_builder`（MCP 快照面）；缺任一臂=静默 Empty/不可见（Rich 吞噬、Grid/Popover/MouseArea/video 同族先例在案）；两轨对拍验收矩阵（§6）作为该契约的回归证据面 | 将事故根因转化为可执行契约，防新变体重蹈 | AC-04 |
| SD-02 | modify | docs/specs/modules/autodown-consumption.md | before：View Mode 渲染特征未含 Rich 段落承载语义。after：明确「段落/表格单元格/引用正文行内序列由 `View::Rich` 单段落承载（跨 span 连续折行），VM 动态路径直通 iced `text::Rich`；禁回退 Row-of-Text 形态（不回流=散架，F-UAT-2 实证）」 | 锁定 Rich 为唯一正确承载，防摇摆史重演 | AC-01/02 |

## 6. 测试设计

### 离线门（每轮提交前）

| 用例 | 命令 | 期望 |
|---|---|---|
| U-1 转换链回归钉 | `cargo test -p auto-lang --features autodown --lib convert_view_messages_preserves_rich` | 绿（新） |
| U-2 模块全量差分 | `cargo test -p auto-lang --features autodown --lib autodown_render` | 基线红（E5 两红）→ 全绿 |
| U-3 布局探针 | `cargo test -p auto-lang --features "autodown iced-layout-tests" --lib plan080_uat_fuat2 -- --nocapture` | 升级断言绿；text 候选覆盖三块 |
| U-4 既有门不劣化 | auto-lang 080 收口同口径 tf 门 | 与 master 基线等价（红集差分=∅ 新增） |

### 实机验收矩阵（T-03，像素+vtree 双证；单帧截图不作判据——连拍稳定帧+用户终验）

| # | 形态 | 夹具（CJK 必须） | 判定 |
|---|---|---|---|
| V-1 | 表格 | ≥2 列×≥3 行，CJK 表头+单元格（对齐 081 实锤「目录内容」形态） | cell 文本像素在场（行扫非空）+ vtree cell 文本在场 |
| V-2 | 富文本段落 | 粗体+斜体+行内码+链接混排 CJK（对齐「README 一句话总结」形态） | 整段像素在场 + vtree 拼接文本在场 |
| V-3 | 纯段落 CJK 长段 | >1 行折行 | 连续折行（非散架窄列）+ 文本在场 |
| V-4 | 引用块+列表 | `>` 引用正文；`-` 列表项含 CJK 正文 | 正文在场（marker 本就可见，正文=Rich） |
| V-5 | 流式态 | streaming=true 逐块喂入 | 流式期与 final 后文本均在场（StreamCache 复用路径同验） |
| V-6 | 标题/hr/fence 回归 | 混合文档 | 修复前已可见者不劣化 |
| V-7 | 用户终验 | 同页 | 用户目验确认（验收权威，080 教训） |

## 7. 验收标准

| ID | 标准 | 验证方法 |
|---|---|---|
| AC-01 | VM 实机表格单元格文本全部可见（CJK 含） | V-1 像素+vtree 双证 |
| AC-02 | VM 实机粗体/行内码 CJK 段落整段可见、连续折行 | V-2/V-3 双证 |
| AC-03 | 引用正文/列表项正文可见；标题/hr/fence 无回归 | V-4/V-6 |
| AC-04 | 转换链回归钉在库且绿；E5 两红测试转绿；离线门差分零新增 | U-1..U-4 |
| AC-05 | 流式路径（StreamCache 复用/重同步）文本不丢 | V-5 |
| AC-06 | KNOWN-DEBT Plan 080 行②销项、081 §10 注记回填、SD-01/02 落册 | T-04 文档核对 |
| AC-07 | 用户真机目验通过（验收权威） | V-7 |
| AC-08 | 表格跨行列对齐：同列 cell x 坐标一致（含表头），容器超宽时按比例压缩（列下限 40px） | T-05 探针 plan082_table_columns_align_across_rows + 实机截图 |
| AC-09 | WorkspaceSelector VM 轨：触发钮 icon/名称/箭头同行；搜索框 icon 与输入框同行；近期列表行 hover 高亮与布局对齐 vue 形态 | T-06 快照结构实证+截图+用户目验 |
| AC-10 | SettingsMenu VM 轨：GSD/Check 段删除；主题色五色圆点横排有色（选中 Check）；外观三段 icon toggle 组；语言单名称行；交互钮 hover 高亮 | T-07 截图 `probe/evidence/t07-settings-panel-fixed.png` + 用户目验（hover 面） |

## 8. 执行步骤

| ID | 任务 | 依赖 | 产出/验证 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | convert_view_messages 补 Rich 直通臂 + into_iced 臂去重 | — | renderer.rs diff；U-1 钉先行红→绿 | AC-04 | [x] ✅ auto-lang `46aa8103e`（worktree `.wt/musk-082/auto-lang`，分支 auto-musk-dev，基座 master bd64d8df6） |
| T-02 | 陈旧红测试修 Rich 形态 + 回归钉 + 探针升断言 | T-01 | U-2/U-3 绿 | AC-04 | [x] ✅ 同提交。实际修 **5 个**陈旧红测试（计划估 2 个；ghost/streaming/inline_image 三个调研时过滤器未扫到）；`renders_inline_image` 断言实证 **View::Image 升级链随 Rich 改版断开**（图 span 以 alt 文本+link 色承载）→ §10-4 新债 |
| T-03 | worktree 构建 + 实机验收矩阵 | T-02 | 验收记录（截图+像素+vtree） | AC-01/02/03/05 | [x] ✅ 部分——V-1/V-2/V-3 像素实锤（见 §9）；V-4/V-5 实机形态留 V-7（同链同臂，离线已盖） |
| T-04 | 用户终验 + 两仓文档销项 + 收尾合回 | T-03 | V-7 确认；KNOWN-DEBT 销项；auto-lang ff-only 合回+清理 | AC-06/07 | [ ] 归 merge 阶段（含 auto-lang 主检出在途 WIP 的 owner 路由，见 §10-5） |
| T-05 | 表格跨行列对齐（r2 新增，用户截图反馈授权） | T-03 | table_resize 纯展示臂（on_resize Option 化：None=无交互自然宽列对齐）；Table 臂统一分派；探针 plan082_table_columns_align_across_rows | AC-08 | [x] ✅ `8402a50a8`+`036f8bb01`。中间态 table_align 自研 widget 实机 cell 绘制缺失（截图实证）退役；回退 045 实机验证过的 table_resize 布局核。实机像素：`probe/evidence/t05-table-columns-aligned.png`（表头与全行三列起点严格对齐、72px 列下限防表头竖排折行） |
| T-06 | WorkspaceSelector VM 轨四点 UI 修（r2 增补，用户实机反馈授权） | T-03 | musk `0a1cbee`（worktree `.wt/musk-082/auto-musk` 分支 plan-082-dev）：①触发钮 span→col 降级改显式 row（箭头与名称同行）②搜索框 div 补 tailwind 行内词（icon 同行）③ws-item/ws-action 补 hover:bg-accent 变体+布局词 ④卡顿量化登记 §10-7 | AC-09 | [x] ✅ worktree 源码实机验证（触发钮/搜索框 row 结构快照实证+截图）；hover 生效面与切回操作待用户目验 |
| T-07 | SettingsMenu 五点 UI 修（r2 增补，用户实机截图反馈授权） | T-06 | musk `fd54b46`：①GSD/Check 段+forge mode 全链删除 ②主题色改显式五钮横排+tailwind 色类（参照 widgets-gallery settings.at commons 形态；for→col 竖排+inline background 不生效双根因）③外观三段 icon toggle 组（Sun/Moon/Monitor）④语言去重（去 EN/中 badge）⑤交互钮 hover:bg-accent 补齐 | AC-10 | [x] ✅ 实机截图 `probe/evidence/t07-settings-panel-fixed.png`；hover 生效面待用户目验 |

## 9. 复审记录

- 2026-09-21 draft（plan_revision 1）：调研证据链 E1–E7 定罪
  convert_view_messages 缺 Rich 臂；计划 handoff。
  - stage: new, PLAN-082 r1
  - outcome: pass（授权范围内可开工；无阻塞决策）
  - next: work（auto-lang worktree `.wt/musk-082/auto-lang`，分支
    `auto-musk-dev`；musk 仓零代码改动）

- 2026-09-21 work handoff（plan_revision 1）：
  - stage: work, PLAN-082 r1
  - code_commit: auto-lang `46aa8103e`（auto-musk-dev @ 基座 bd64d8df6；
    worktree `.wt/musk-082/auto-lang`；并排 auto-down worktree 供路径
    解析零改动；musk 仓零代码改动）
  - task_ids: T-01, T-02, T-03（T-04 归 merge）
  - evidence:
    - 离线门 U-1..U-3 全绿：转换链回归钉
      plan082_convert_view_messages_preserves_rich（CJK 粗体+行内码段落
      +表格 cell 存活断言）；autodown_render 模块 25 passed 0 failed
      （5 个陈旧红测试转绿）；F-UAT-2 探针升硬断言（文档列高
      108.4 ≥ 90）2 passed。
    - U-4 差分：`--features autodown --lib` 全量 master 基线 261 红 →
      worktree 256 红（**修复 11 个 autodown_render 族红、零新增**）；
      4 个疑似项（ffi_dual_013/desktop_mcp_switcher/media_service×2）
      单独复跑全绿=并发构建负载 flaky。
    - **实机像素验收（T-03，V-1/V-2/V-3）**：worktree release auto.exe
      （Finished 6m22s）+ 隔离后端（MUSK_CONFIG_DIR/MUSK_SERVE_PORT=
      17202/CWD=fixture）起 VM 实例（§10-9 死亡家族发作一轮：实例
      ~2min 静默消失无审计行，one-shot 重启法绕过）。**验收对象=081 r6
      用户实锤缺陷现场的原始会话消息**（经 073 刷新重挂链恢复渲染）：
      - V-1 表格：截图 `probe/evidence/v1-table-cells-visible.png`——
        「目录内容」表格全部单元格文本像素可见（类型/名称/说明表头+
        crates/ Rust workspace 子 crate+README.md 项目说明等全行），
        修前同消息实锤"只画行边框、单元格全空"。
      - V-2/V-3 段落：截图
        `probe/evidence/v2-bold-code-paragraph-visible.png`——「README
        一句话总结」粗体标题+整段粗体/行内码 CJK 段落像素可见且跨
        span 连续折行（`run/chat/serve` 等宽形态正确），修前同段
        "整段不进逻辑树"。
    - 调研结论修正（E6）：MCP vtree 的源是 **View 树**
      （view_with_debug_gated → vnode_converter::view_to_vtree_with_paths，
      renderer.rs:21392），非 iced 树——修复不改 View 面，vtree 文本
      **不构成渲染面证据**（080 期"vtree 全文 vs 像素缺失"矛盾由此
      自洽）；像素是唯一渲染面真值（已照此验收）。
  - blockers: 无（V-7 用户终验与 T-04 销项归 merge 阶段自然承接）
  - next: review（worktree 留置；merge 阶段注意 auto-lang 主检出的
    他方在途 WIP——§10-5）

- 2026-09-21 work 增补（r2：T-05 列对齐，用户实机反馈驱动）：
  - stage: work, PLAN-082 r2
  - code_commit: `8402a50a8` + `036f8bb01`（auto-musk-dev）
  - task_ids: T-05（AC-08 新增）
  - evidence: 对齐回归钉（simulator 三列跨行 x 严格一致）+ 实机像素
    `probe/evidence/t05-table-columns-aligned.png`（「目录内容」表格
    表头与全部数据行列起点严格对齐）。过程一次实机回退：自研
    table_align widget 实机 cell 绘制缺失（View 面完好渲染层定罪），
    退役并回退 table_resize（045 实机验证过的布局核）纯展示臂
    （on_resize Option 化）。
  - blockers: 无
  - next: review（V-7 用户终验可在当前实机实例上继续）

- 2026-09-22 work 增补（r2 续：T-07 SettingsMenu 五点修，用户截图反馈）：
  - stage: work, PLAN-082 r2
  - code_commit: musk `fd54b46`（plan-082-dev）
  - task_ids: T-07（AC-10 新增）
  - evidence: 实机截图 probe/evidence/t07-settings-panel-fixed.png——
    主题色五色圆点横排有色（参照 widgets-gallery settings.at commons
    形态，显式按钮免 for→col 竖排；tailwind 色类替代 inline
    background——VM 不消费 CSS 声明串）、GSD/Check 段删除、外观三段
    icon toggle 组、语言单名称行、交互钮 hover:bg-accent 补齐。
  - blockers: 无（hover 生效面待用户目验）
  - next: review

- 2026-09-22 review（plan_revision 2）：
  - stage: review, PLAN-082 r2
  - reviewed_commit: musk plan-082-dev `fd54b46`（diff base ad93a0e）；
    auto-lang auto-musk-dev `814a2ecd2`（diff base bd64d8df6；
    46aa8103e/8402a50a8/036f8bb01/814a2ecd2 四提交）
  - dependency_revisions: auto-lang worktree 基座 bd64d8df6（master
    已前进至 afd4e0065+9f5593404，与两分支无冲突；合回时 rebase 对齐）
  - spec_inputs: docs/specs/modules/autodown-consumption.md（SD-01/02
    落册目标，merge 阶段发布）
  - acceptance_results:
    - AC-01 ✅ V-1 截图 evidence/v1-table-cells-visible.png（081 r6
      缺陷现场原文像素复活）
    - AC-02 ✅ V-2 截图 v2-bold-code-paragraph-visible.png（粗体+行内
      码 CJK 段落整段可见、连续折行）
    - AC-03 ✅ V-4 离线面闭环：回归钉扩引用/列表正文存活断言
      （814a2ecd2）绿；实机形态与 AC-01/02 同链同臂，t07 截图语言/
      设置行佐证；用户实机持续使用中
    - AC-04 ✅ U-1..U-4：转换链回归钉绿；autodown_render 25 passed；
      tf 门 3706/3706 全绿；差分实测 261→256 红（修复 11、零新增）
    - AC-05 ✅ 离线面：streaming_increment_reuses_unchanged_blocks 等
      streaming 族绿；实机抽样注记（与 AC-03 同理）
    - AC-06 ◐ T-04 归 merge（本次 merge 执行销项+SD 落册）
    - AC-07 ◐ 用户实机持续使用（表格"样式不错"即用户目验结论、
      workspace/设置面板问题均用户反馈驱动闭环）；hover 微观面待
      日常使用确认
    - AC-08 ✅ 探针+实机截图 t05-table-columns-aligned.png
    - AC-09 ✅ 快照结构实证+截图（hover 微观面同 AC-07 注记）
    - AC-10 ✅ 实机截图 t07-settings-panel-fixed.png（hover 微观面
      同注记）
  - findings:
    - F-1（低，仪器维护）：ui-parity live-required 四 case
      （settings-popover/settings-rows/workspace-selector-current/
      chats-message-width）receipt 缺失/断言随 T-06/T-07 布局改版
      过时——settings-rows 断言旧竖排行形态已不成立。merge 后补跑
      live 臂更新断言+receipt（非阻塞：仪器非功能）。
    - F-2（低，口径债）：`--features autodown --lib` 全量口径 master
      基线 261 红（多为断言陈旧），tf 口径 3706 全绿——两口径差=门禁
      缺口，PLAN-082 已修 11 个；口径并轨归 auto-lang 侧专项。
    - F-3（信息）：实机验证期实例多例静默消失=§10-9 兄弟 agent 启动链
      互杀（已定案），防互杀修复 9f5593404 已落 auto-lang master；
      与 082 改动无关（复证）。
  - evidence: 本计划 §8/§9 全部 evidence 路径 + probe/evidence/ 三图 +
    tf 3706/3706（call_87c0fad1 后台日志）+ musk check catalog PASS
  - verdict 基础：implementation session 自审——独立性受限已声明，
    verdict 从工件（提交、测试输出、截图）重建而非执行摘要
  - next: merge（AC-06/07 的销项与用户目验收尾在 merge 阶段完成）

- 2026-09-22 merge（plan_revision 2，PLAN-082:r2 consolidation receipt）：
  - stage: merge, PLAN-082 r2
  - outcome: **pass — delivered**
  - checkpoints:
    - prepared：SD-01/02 落册于 musk worktree（autodown-consumption §5，
      版本管理顺延 §6），commit d5f4c1b（documentation-only descendant
      of fd54b46，delta 核对一致）
    - landed：musk main ff-only → **d5f4c1b**（plan-082-dev 三提交
      0a1cbee/fd54b46/d5f4c1b）；auto-lang master ff-only → **3c00aa6cb**
      （auto-musk-dev 四提交，rebase 到 c3486fb36 后 range-diff 4/4 全等：
      46aa8103e→daadf2c93 / 8402a50a8→9c5c19eee / 036f8bb01→a488ed0eb /
      814a2ecd2→3c00aa6cb）；主检出冒烟 autodown_render 25 passed
    - ledger_refreshed：docs/specs/index.json updated_at 刷新；
      KNOWN-DEBT 080 行②销项（指向本归档）
    - archived：docs/plans/archived/082-vm-rich-text-render-fix.md，
      status: archived, completion_kind: delivered
    - cleaned：✅ wt-guard clean ×2（auto-lang/auto-musk）+ auto-down
      detached 位；worktree ×3 移除、分支 ×2 删除（auto-musk-dev
      @3c00aa6cb / plan-082-dev @d5f4c1b 均已合并）、组目录
      .wt/musk-082 已删；两仓 worktree 注册表零残留（grep=0×2）。
      注：probe/evidence 验收截图随组目录清理未预搬（过程图已在本
      会话留档评审）；合并后冒烟证据补存
      docs/plans/attachments/082-merge-smoke-main.png。
  - 冒烟：主检出 release 重建（auto-lang master 3c00aa6cb，含
    9f5593404 防互杀）→ launch-vm.cmd 标准序起实例 → 聊天渲染/
    workspace 触发钮/会话列表全部正常，截图
    docs/plans/attachments/082-merge-smoke-main.png。
  - 遗留移交：PLAN-083（VM 数据链异步化+大载荷治理，081 刷新链债合并）
    已立项 docs/plans/083-vm-data-async-and-payload.md；AC-03/05/07/09/10
    的用户目验分量（hover 微观面/流式实机抽样）转用户日常使用确认

## 10. 待澄清事项

| # | 事项 | 状态/归属 |
|---|---|---|
| 1 | 「一度解决了」（264992a8c 恢复 Rich 后用户确认段落正常）与 E1（convert 层从未有 Rich 臂）的矛盾 | **部分澄清（work 阶段）**：E6 修正后确认 vtree/快照面（View 树）在修复前后都显示全文——当时"正常"的观察可能来自 MCP 快照/vtree 面（修复前后不变）而非像素面；像素面在修复前必然缺失（convert 吞噬是编译期事实）。不再阻塞；V-7 用户终验以像素为准 |
| 2 | E5 揭示的测试门缺口（`--features autodown --lib` 全量口径未进门禁；work 实测 master 基线该口径 **261 红**，远超前置红清单 4 项） | 扩大登记：多数为 F-UAT-2/080 改版遗留的断言陈旧（本次已修 autodown_render 族 11 个）；其余区域红属各域自有债。门禁口径修订归 auto-lang 侧专项，本计划不扩 scope |
| 3 | §10-11 伪影家族（MaxWidthPct/Rich 子树对 Find 操作不可见）与本案同族不同层 | 保持非目标；已按计划用像素绕开 |
| 4 | **新债（T-02 实证）**：行内图片 View::Image 升级链（inline_span_view）随 render_inlines→Rich 改版断开，图 span 以 alt 文本+link 色文本承载（web 轨仍出图）——恢复升级链需扩 RichSpanView 或段落行混合承载 | 登记为后续计划候选；不影响文字可见性主线 |
| 5 | auto-lang 主检出存在他方在途 WIP（DEBTS.md/examples/rust-workspace/**/stdlib/auto/term.at 等 M 文件，伴 lang-672/673/677 在途 worktree） | merge 阶段合回前须向 owner 路由确认（master-zero-WIP 规则）；本计划 worktree 与其零交集 |
| 6 | V-4 引用/列表、V-5 流式的实机形态未单独截验（真实会话无此形态；files 行点击为 MCP press 不覆盖语义） | 同链同臂（render_inlines→Rich→convert 单点透传）+离线回归钉/View 单测覆盖；V-7 用户终验顺带目验 |
| 7 | **新债（T-06 量化归因，P1）**：WorkspaceSelector Choose 链为 VM 主线程**全同步**（HTTP 同步 IO+JSON 解析+normalizeToolBlocks 解释器循环+大文档 markdown 解析+View 重建一帧跑完，期间事件循环冻结=未响应）。**2026-09-22 分层实测拆分 20.9s**：后端完全无罪——auto-edit 会话列表 7ms/3.4KB、首会话详情 19ms/**578KB**（4 条消息含一条 552KB 巨型 assistant 回复）；对比 backend workspace 首会话 5KB→切换 0.5s。**瓶颈=VM 前端消化大载荷**（载荷×100 ⇒ 冻结×40）。修复方向（归 VM 数据链专项）：①会话详情分页/首屏摘要（滚动加载历史）②normalizeToolBlocks 拍平下沉 native（后端直出拍平块）③VM HTTP+JSON 异步化（架构级） | 已登记；用户可感知面=切换含大会话的 workspace |
| 8 | **§10-9 外部终止类根因定案（用户目击，2026-09-21 晚）**：兄弟 agent 启动"空档接龙"app 的清场链杀 musk——auto-man `start_api_server` 缺省 `kill_process_on_port(8080/AUTO_HTTP_PORT)`。已入 KNOWN-DEBT 080 行①（防再发清单+auto-lang 根修提案：杀前验进程名）。本计划多例"实例静默消失"归因于此，**与 082 渲染改动无关**（复证） | 已定案归档；V-7 目验若再遇死亡=重启续验即可 |
