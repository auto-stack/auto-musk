---
plan_id: PLAN-080
status: executing
feature_name: 桌面实机验收暴露的 VM 轨缺陷批次修复（gallery 真机一致性臂 + AutoUI 级 button 默认重置 + settings 类弹窗 popover 化）
author: [agent]
created_at: 2026-09-20T15:59:36+08:00
updated_at: 2026-09-21T01:20:00+08:00
plan_revision: 1
current_step: 4
total_steps: 6
supersedes_spec_components:
  - "docs/specs/modules/ui-default-styles.md"
  - "docs/specs/modules/ui-parity.md"
  - "docs/specs/modules/ui-compositions.md"
new_spec_components: []
touched_goals: [goal-frontend-parity]
depends_on: ["PLAN-074", "PLAN-075", "PLAN-078", "PLAN-079"]
---

# PLAN-080 — 桌面实机验收暴露的 VM 轨缺陷批次修复

## 0. 变更摘要

074-079 对齐系列归档后的**首次桌面实机验收**（2026-09-20，真 musk.exe 后端 +
`AUTO_REUSE_BACKEND` 复用门链路，本会话已打通）暴露一批 VM 轨缺陷：消息区宽度
与 Vue 观感不一致、WorkspaceSelector 不显示当前工作区、收缩 rail 的裸 button
漏出 VM 默认预设（紫底）、settings 弹层是居中 modal 而非就近 popover、settings
内部组件布局散架。用户裁定五项修复方向（详见 §4 授权记录），核心策略：
**以 gallery 一致性检验为仪器**（补真机臂治"物化绿而实机红"盲区）、
**button 默认样式做 AutoUI 级统一重置**（对齐 vue inject_styles 语义，参照
design 010 迁移矩阵，不做逐按钮覆盖）、**settings 类弹窗 popover 化为通用
修改**（AutoUI 级 BP 方向）。

前置事实（本会话实测定罪，作为背景基线）：
- VM `__json_object` 语义：GET 缺键返 null 安全 / SET 缺键硬崩（PLAN-044
  engine.rs）——073 归一化补写崩点已修（auto-musk d313fa1，构造替换式）。
- `auto run` stub 后端与真后端的打架已由 auto-lang `34ee15a47` 复用门解决。
- 上述两笔为 hotfix 先行落地；本计划覆盖其余验收发现 + 工装补强。

## 1. 目标

- gallery 补**真机一致性臂**：对选定组件采集实机 VM 几何/行为与 Vue 对拍，
  治"materialize 物化快照绿而实机红"的盲区（079 门挂接）。
- 消息区宽度、WorkspaceSelector、settings 弹层与内部布局四组用户可见缺陷
  经双轨对拍定罪并修复。
- VM 裸 button 默认预设（紫底 + h-10 px-4）做 **AutoUI 级统一重置**对齐
  vue 默认形态；musk 内逐按钮后置覆盖类随之清查。
- settings 类弹窗统一为就近 popover（锚定触发件、非 modal、外点自动关闭），
  优先 AutoUI 级 BP。
- 结清本会话余项：web 轨 vue-tsc/build 门补跑、ui-parity 截图基线刷新
  （侧栏 bg-muted/40 变更后）、VM `__json_object` SET 语义入册。

非目标：恢复旧 web/ 轨；重做 gallery 业务模板；修改后端 API 契约；处理
075 已登记的远期像素级差异（次像素抗锯齿等三项）。

影响仓库：auto-musk（gallery/组件/样式）；auto-lang（T-03 button 预设、
T-04 popover 原语，同组 worktree `auto-musk-dev` 分支）。
依赖：PLAN-074 gallery 基建、075 默认样式对账、078 组合用例、079 发布门。
参照设计：[design 010](../designs/010-web-only-css-migration-matrix.md)
（inject_styles → .at 单一真源迁移矩阵，§44 样式族进 gallery 要求）、
[design 012](../designs/012-vue-vm-parity-gallery-roadmap.md)（S1-S6 蓝图）。

## 2. 架构方案

分层修复，仪器先行：

1. **仪器层（T-01）**：ui-parity runner 扩真机臂——实机 `auto run --render=vm`
   （真后端 + 复用门链路）+ 几何/计算样式采集通道（AutoUI MCP 9247 系或截图
   几何探针，T-01 调查定案），与 Vue 臂同 case 对拍。物化臂保留（结构/回归）。
2. **数据单源层（T-02）**：WorkspaceSelector.current 与 ForgeStore.workspace
   的初始化同源化（ws_load_current 单源），触发器显示当前 workspace 路径。
3. **AutoUI 原语层（T-03/T-04，auto-lang）**：button 默认预设重置（variants.rs
   `button_size_preset`/variant 底色缺省对齐 vue 产物默认）；锚定 popover 原语
   （承接 PLAN-059 T9 的外点/ESC 关闭语义，新增锚定 + 非 modal 形态）。
4. **组件消费层（T-04/T-05，auto-musk）**：settings_menu.at 迁 popover；内部
   行布局按 gallery 对拍修复；musk 内逐按钮后置覆盖类清查（验证性清查，
   依赖预设重置生效）。
5. **收尾层（T-06）**：web 轨门补跑、基线刷新、语义入册。

## 3. 技术栈

- gallery：`scripts/ui-parity.mjs` + `scripts/ui-parity/materialize.mjs` +
  `tests/ui-parity/cases.json` + fixtures（074-079 既有体系扩展）。
- VM 渲染：auto-lang `crates/auto-lang/src/ui/style/variants.rs`
  （button_size_preset 缺省 `h-10 px-4`、variant 底色）及 dialog 家族。
- musk 组件：`src/front/settings_menu.at`、`src/front/workspace_selector.at`、
  `src/front/app.at`（rail）、`src/front/chat_message.at`（rowClass）。
- 真机链：`MUSK_SERVE_PORT=17201 musk.exe serve` +
  `AUTO_REUSE_BACKEND=1 RUST_MIN_STACK=16777216 auto run --render=vm`。

## 4. 需求分析与背景调查

**用户授权记录（2026-09-20 实机验收反馈，五项裁定原文意译）**：
1. 消息区宽度问题——"添加 gallery"（用例进 gallery 一致性检验）+ 实测。
2. WorkspaceSelector——"把 workspace 的选择组件也加进 gallery 进行一致性
   测试，然后让它把当前的 workspace 路径列出来"。
3. VM button 默认预设——"是 VM 的 iced 默认样式问题，应该统一（对任何
   AutoUI 的 VM 版 iced app）进行默认的重设（参照 vue 版里的 inject css）。
   这套在 design 里有文档跟踪。需要把所有 button 的默认样式都覆盖掉，改成
   和 vue 版一致，而不是每次针对单个 button 重置。"
4. settings 弹层——"需要单独修改；且是针对所有 settings 类型的弹窗
   （popover 向）的通用修改。建议加入 gallery，或者做成 AutoUI 级的 BP"。
5. settings 内部布局——"通过 gallery 一致性检验来修"。
6. （2026-09-21 UAT 追加）本地应用免登录——"把开启时的登录页面去掉（这个是
   local app，不需要登录）"→ AuthStore.Init 合成 local 身份，musk 099eed7。
另：本会话前面提到的修改需求（含 hotfix 余项：web 门补跑、基线刷新）一并
入本计划。

**代码实锚（本会话调查）**：
- 消息行宽：`chat_message.at:122` rowClass `max-w-[70%]`（两轨同源类，差异
  疑在 VM 百分比 max-w / stretch 解析，待 T-01 双拍定罪）。
- WorkspaceSelector：`current` 经 `ws_load_current()` 自初始化，与
  `ForgeStore.Init` 的 `musk-demo` 兜底（`forge_store.at:133`）不同源；触发器
  无 current 时显示"选择工作目录"（`workspace_selector.at:30`）。
- button 预设：auto-lang `variants.rs` `button_size_preset` 缺省 `h-10 px-4`
  + variant 底色缺省紫（primary）；musk 侧既有"后置类逐项覆盖"注释为证
  （chats_view.at 071 r4 段）。
- settings 弹层：`settings_menu.at:179` `dialog (open:)` 受控（PLAN-059 T9
  从手搓 absolute `.settings-panel` 迁来，历史锚定实现可参考）。
- gallery 现状：`shell-nav-sidebar`/`shell-settings-menu`/
  `shell-workspace-selector` fixtures 已注册（078），但为物化结构快照，无
  实机几何/行为断言——盲区实锚。
- VM `__json_object` SET 缺键硬崩语义（engine.rs PLAN-044 分支）本会话实证，
  尚未入 spec/design 册。
- **实机事件（2026-09-20 16:32）**：真 musk.exe 后端在用户删除 auto-edit
  工作区一个会话（DELETE /api/chats/session/6e44d5b7…?workspace=auto-edit
  返回 200）之后不久以 exit 1 退出，日志无 panic/error 尾巴——成因未定罪，
  归 T-02 会话操作域排查（重现条件：删会话后观察后端存活）。

约束：hotfix 已落地的行为（d313fa1 构造替换式归一化、34ee15a47 复用门、
auto-lang api_gen/E0433 三笔）不回退；079 门（`node scripts/ui-parity.mjs
check`）全绿为准入。

## 5. 详细设计

### 5.1 gallery 真机一致性臂（T-01）

- runner 增 `--live` 臂：起真后端 + `auto run --render=vm`（复用门），对
  注册了 live 断言的 case 采集实机几何（元素盒模型/计算样式），与 Vue 臂
  （同状态）对拍，差异超预算即红。采集通道 T-01 调查二选一并记录决策：
  AutoUI MCP（9247 系已有 listener）优先，截图几何探针兜底。
- 新 case 注册：`chats-message-width`（消息行宽度几何）、
  `workspace-selector-current`（含 current 路径文本断言）、
  `settings-popover`（弹层形态/锚定断言）、`settings-rows`（内部行布局）。
- 079 required 门挂接：live 臂进 `check` 的 required 面（可标记
  `live-required`，离线环境降级为 skip + 显式留痕，不静默绿）。

### 5.2 工作区单源化（T-02）

- `ForgeStore.Init` 与 `WorkspaceSelector` 的 current 初始化收敛到
  `ws_load_current()` 单源（store 持真值，selector 读 store）。
- 触发器文本行显示当前 workspace **路径**（current.name + title=path 既有，
  补可见路径文本），无 current 时保持引导文案。
- 会话点击无反应排查：SwitchSession 对 stale 会话（get 404）的行为定罪
  （静默/提示/清理）并修；命中区问题（若有）一并修。

### 5.3 AutoUI 级 button 默认重置（T-03，auto-lang）

- `variants.rs` 的 button variant/size 缺省对齐 vue 产物默认形态（参照
  design 010 inject_styles 语义与 shadcn 默认面）：裸 button 无底色预设
  （或与 vue 完全同形的缺省），尺寸预设不与显式类打架。
- 类合并优先级核查：确保显式后置类可完整覆盖预设（既有 musk 注释称可
  覆盖，实锚 rail icon 反例——定罪优先级语义）。
- musk 侧逐按钮后置覆盖清查：预设重置生效后，`bg-transparent` 等纯为压
  预设而写的后置类可清（验证性清查，逐处跑对拍）。
- 跨 app 校验：auto-lang examples（015-notes 等）裸 button 形态不回归。

### 5.4 settings 类弹窗 popover 化（T-04）

- auto-lang 增锚定 popover 原语（`popover (anchor:)` 形态）：锚定触发件
  就近浮层、非 modal（不遮罩不居中）、外点/ESC 自动关闭（语义承接
  PLAN-059 T9 dialog 家族）。若调查判 auto-lang 侧代价过高，降级方案为
  musk 级手搓锚定面板（复用 T9 前的 `.settings-panel` 历史实现 + 外点
  语义），并登记 auto-lang BP 待办——决策记录进复审。
- `settings_menu.at` 迁移 popover；"所有 settings 类弹窗"面扫（musk 内
  dialog 用法盘点，settings 向的一并迁）。
- gallery `settings-popover` case 双臂断言（形态/锚定/外点关闭行为）。

### 5.5 settings 内部布局（T-05）

- 主题/强调色/语言行按 gallery 双拍逐项修（VM 侧布局散架：行内元素换行/
  间距/对齐），以 Vue 臂为基准，几何差入预算。

### 5.6 收尾（T-06）

- web 轨 `auto build`（vue-tsc 面）补跑结清 d313fa1 hotfix 欠账。
- ui-parity 截图基线刷新（侧栏 bg-muted/40 等本会话样式变化），走基线
  更新 review 流程。
- VM `__json_object` GET-null/SET-crash 语义写入 spec（ui-default-styles
  或 autodown-consumption 邻位，T-06 定案落点）。

### 规范增量

| delta_id | add/modify/retire | target | before/after | rationale | AC |
|:--|:--|:--|:--|:--|:--|
| SD-01 | modify | `docs/specs/modules/ui-default-styles.md` | 无 button 默认形态契约 → 增"VM 裸 button 默认与 vue 产物一致（无预设底色），显式类完整覆盖优先"契约及 AutoUI 级重置事实源指向 | 用户裁定统一重置非逐按钮；design 010 §44 要求 | AC-04 |
| SD-02 | modify | `docs/specs/modules/ui-parity.md` | 门=物化结构对拍 → 增 live-required 真机几何/行为臂契约（离线降级显式 skip） | 物化绿而实机红盲区实锚 | AC-01/02/06 |
| SD-03 | modify | `docs/specs/modules/ui-compositions.md` | settings 类弹层无形态契约 → 增"settings 向弹层一律就近 popover（锚定/非 modal/外点关）"契约 | 用户裁定通用修改 | AC-05 |

## 6. 测试设计

- gallery：新 case 四件（§5.1）双臂；故意回归注入（几何漂移/弹层变 modal）
  必红（沿 079 故意红方法论）。
- auto-lang：`variants.rs` 预设单测更新 + `cargo t`（ui-iced 日常档）+ 受
  影响 examples 编译；popover 原语配单测（锚定/外点语义）。
- musk：真机链端到端（复用门 + 真后端）手验清单：rail 收缩形态、settings
  开合/外点、workspace 路径显示、消息宽度、会话切换（含 stale 用例）。
- web 轨：`auto build` + 既有 style-parity 门。

## 7. 验收标准

- **AC-01** gallery 真机臂生效：live-required case 对当前已知缺陷（rail
  icon 底色、消息宽度）修前红、修后绿；离线跑 `check` 显式 skip 留痕。
  验证：`node scripts/ui-parity.mjs check`（live 臂）前后对比 + 故意注入红。
- **AC-02** 消息区宽度双轨几何一致：同状态消息行宽度差 ≤ 既定预算（沿 079
  ≤2px 口径或计划内裁定值）。验证：`chats-message-width` case 双臂报告。
- **AC-03** WorkspaceSelector 显示当前 workspace 路径，current 与 store 单源；
  会话点击行为定罪修复（stale 会话有明确反馈）。验证：live case 断言 +
  真机手验（含点击每类会话）。
- **AC-04** VM 裸 button 默认形态与 vue 一致：收缩 rail 全部 icon 无预设
  底色、仅选中项差异；auto-lang 级实现（非 musk 逐按钮覆盖），examples
  不回归。验证：gallery 对拍 + `cargo t` 绿 + 015-notes 等抽样目检。
- **AC-05** settings 类弹层为就近 popover：锚定触发件、非 modal、外点自动
  关闭；musk 内 settings 向弹层全量迁移。验证：`settings-popover` case +
  真机手验（开合/外点/多弹层互斥如有）。
- **AC-06** settings 内部行布局双拍一致（几何差入预算）。验证：
  `settings-rows` case 双臂报告 + 截图基线刷新走 review。
- **AC-07** 本会话余项结清：web 轨 build 门绿、ui-parity 基线刷新完成、
  VM `__json_object` SET 语义入册。验证：命令输出 + spec/design 文本落点。

## 8. 执行步骤

- [ ] **T-01 gallery 真机一致性臂 + 新 case 注册 + 消息宽度定罪修复** [🔁 UAT 重开（2026-09-21）：
  余 F-UAT-1——cee16d694 消息行宽修复引入 **CJK 用户气泡空泡回归**：max-w-[70%] pct 臂对
  CJK 文本测量 ≈0 宽（泡塌缩为 padding 盒、渲染按宽裁剪后不可见），latin 同泡正常。
  证据链（UAT 会话实测）：后端存储 content="你好" 完好（输入/传输链排除）→ 侧栏会话项/
  助手 markdown 表格/输入框占位符中文全部正常渲染（VM 字体栈 CJK 能力排除）→ 同泡 latin
  （"heello"）正常（泡路径排除）→ 唯一剩群=用户泡独有的 max-w-pct 文本测量/裁剪臂。
  修前（080 原始缺陷）该泡为满宽可见（文本在、宽度错），修后宽度对而 CJK 文本灭=修复引入。
  live case 盲区根因：chats-message-width 夹具为拉丁 marker（live 臂拉丁 marker 政策），
  CJK 变体未覆盖。修法：pct 臂 CJK 宽度测量（char 宽表/字形成测）+ live case 增 CJK 变体；
  F-R1 面此前已收口不变]
  （auto-musk 1488482 + auto-lang cee16d694；F-1 余项前轮收口：musk f82bdf9）。
  仪器：live.mjs（隔离真后端 MUSK_CONFIG_DIR + 复用门 + MCP include_bounds + Vue dist 静态服务对拍，
  采集语义沉淀见 ui-parity.md SD-02）；4 case 注册 + 修前红定罪收据 4 份
  （docs/reports/ui-parity/080-*.pre-fix.json）。消息宽度定罪：VM 行 835/867
  ≈96% 满宽贴左——三渲染缺陷（pct 强制 Fill 丢 CSS 语义 / pct 臂丢
  align_self / MaxWidthPct 缺 operate 委托），修于 auto-lang cee16d694。
  **F-1 收口（f82bdf9）**：右缘对拍改**像素真值锚**——纯 Node PNG 解码
  （node:zlib）+ 色相域主色判定（hue 225-265°，覆盖 89,99,207 与
  147,148,245 两档 primary，固定 RGB 容差漏检后者）+ 列密度聚类（稠密列
  ≥12，抗稀疏字形噪声）；两坑（canvas 跨源/扫描挂点）随纯 Node 实现
  退役。裁决：**VM 用户气泡视觉右对齐正确**（像素右缘 1263.5 ≈ vue
  1264，hits 32k 实心簇），快照 @rect capped 盒（x429 左置）是伪影——
  probe5 截图 vs 快照矛盾结案；case PASS（ratio 护栏 0.674≤0.72 同绿）。
- [x] **T-02 工作区单源化 + 路径显示 + 会话点击定罪** [✅ 已完成]
  （auto-musk 1488482；VM 面解阻复验：auto-lang 26b8cca44 + musk f82bdf9）。
  ws_resolve_current() 单源解析 + ForgeStore workspace_name/path 真值字段 +
  SetWorkspace 消息 + selector 读 store 显示路径（pathOrName 两级单层 if）；
  musk-demo 硬编码退役。**VM 面解阻（F-2 收口）**：债三连同根=UI（aura/
  front .at）路径裸 Http.get 经 canonical 规则落 auto.http.get（Response
  句柄 int，probe080 实证 to_str="2"）——auto-lang 26b8cca44 于 vm/
  codegen.rs Call 臂补 get_json+to_value 改写（ts_adapter Plan 028 F8 同
  配方；脚本 bigvm 路径 PLAN-442 本有，UI 路径缺口），musk relay_store/
  agent_configs/settings_forge_helpers/tool_gate_card 整族同修；债②（JSON
  数组 for-in/索引返索引）当前构建不复现（probe 实证 w.id=ws 正常，疑债③
  下游表象）；债①（back.api workspace_list 返空）同根收口。**vue 臂占位
  根因（f82bdf9）**：ws_resolve_current 裸调 ws_load_current()——vue 侧
  async fn 裸调用返 Promise，`Promise != null` 恒真永远短路返 null（20×
  500ms 轮询实证）；修法=内联 status 逻辑（plain fn 内 .await 链与登录
  Submit 静默 exit 1 死亡现场时间线强相关，bare Http.get 是已验证形态）。
  终局复验（live r2-r7）：触发器双臂显示完整路径、sessionClickOk 双臂
  true（vm 位置锚——会话行按钮快照恒无样式/空标签）、ws-trigger 几何
  15px 代理预算（vm 祖先代理盒 vs vue 容器盒非等价，文本/点击仍硬门），
  workspace-selector-current case PASS。
- [x] **T-03 AutoUI 级 button 默认重置 + musk 覆盖类清查** [✅ 已完成]
  （auto-lang 0cbf5a031 + musk 1488482）。缺省 chromeless（preflight 等价）
  三表同步（variants.rs / rust codegen / vue cva defaultVariants 移除），
  显式 variant="default" 保留 PLAN-571 基线；目标测 18/18 绿、button 面
  52 测 51 绿 1 前置红（icon_component_child…，干净 master 同红）。musk
  侧：rail 图标钮 rail-icon-btn → tailwind 词表 + 触碰面 bg-transparent/
  border-none 后置类清查。examples 裸 button 双轨同变（parity 保持）。
- [ ] **T-04 popover 原语 + settings 弹层迁移**（复审重开收窄：余 F-3——间歇静默 exit 1 崩溃根修（auto-lang 债，取证完备案）+ 真机外点/ESC 手验清单（用户验收项）；迁移本体/re-press/锚定全绿）（musk 1488482 + f82bdf9）。
  调查定案：auto-lang 锚定 popover 原语已在（Plan 422+ 家族，PLAN-059 T9
  语义），零 auto-lang 改动；settings_menu.at dialog 受控 modal → popover
  嵌套（top-start + w-80 + 注入 toggle + 家族关闭语义 + 关闭行退役）。
  dialog 面扫：仅 chats 删除确认 alert 正确保留 modal。**F-3 收口
  （f82bdf9）**：re-press toggle 关闭**验证通过**——旧判据 includes('GSD')
  假红（popover content 预构建=文本恒在树中，popover_probe 六态实证），
  改判据"GSD 行 @rect 面板宽（<800px）=开 / 视口宽占位=关"；live
  settings-popover case PASS（锚定 + re-press 双臂绿，web ESC ✓）。
  **余项**：①VM app 间歇静默 exit 1（~2/8 复现，登录 Submit 后整进程
  exit=1 无 panic 无错误尾巴；exit-audit 无 code=1 行=非 panic/
  Process.exit/main_return-Ok；另有一型 MCP 活而 UI vtree 永不发布——
  取证落 plan §10 余项 9，auto-lang 侧独立收口；时间线与 plain fn 内
  .await 链强相关但最小隔离不复现，musk 侧已规避=内联形态）；②真机
  外点/ESC 手验清单（MCP 合成事件不经 overlay 路由，唯真机可验——用户
  验收项）。
- [ ] **T-05 settings 内部布局修复**（复审重开收窄：宽度面已收口（66.7→2.2px）；余字体度量族——text 行高映射 21 vs 15.2、popover 锚 x 偏移 ~11px、按钮高 ~12px（auto-lang 样式映射，独立验证周期））。
  settings-rows 双臂测量打通 + F-4 宽度收口（f82bdf9）：popover 标签摘
  class:"w-80"（两轨语义错位——web 落 wrapper div 无效果 / VM 落 content
  列且顶替 PLAN-528 W9 缺省 chrome 成 w-80=320 无 bg/border/p-4，内宽
  308 vs web 241）→ 双轨同归 w-72 缺省 chrome，modeRow/标题宽差
  66.7/205.3→**2.2px**；五处 section 标题加 w-full（VM hug 文本 w=36 vs
  web 块级满宽 241）。mt-auto 推底修复（auto-lang 71f8027ae）已生效
  （ws-trigger.y 314→5px、rows y 261→29-74px）。余 h 5.5-11.8px + x
  11.4px + y 累积（面板高差派生）=auto-lang 字体度量族（text-xs 行高
  VM 缺 line-height 映射、popover 锚偏移、按钮高映射），登记 §10 余项
  10 随独立验证周期收口。
- [x] **T-06 收尾：web 门补跑 + 基线刷新 + SET 语义入册** [✅ 已完成（F-R2 收口采 P660-D1
  收口路线：auto-lang 7d6888076——scaffold 三处发射 src/vite-env.d.ts（create-vue 惯例三斜线，
  不设 tsconfig types 免窄化 @types）+ prepare_vue_sources 补发射 auto-sources.ts + run 缺文件
  自愈；musk auto build 全 pipeline 绿（vue-tsc && vite，14.7s）+ 014-weather 原始复现点全绿
  （1.59s）+ tf 3649/3652 零新增；收据 docs/reports/ui-parity/080-web-gate.p660d1.md
  （musk f5d5558）。基线刷新+SET 入册已验收绿]
  （基线刷新轮：musk f01dff6）。~~web 门：auto build 全 pipeline 绿（本轮 ×2：
  vue-tsc + vite，新 auto 二进制 26b8cca44，含 T-02 内联化/T-04 popover/
  T-05 面板宽全部改动）。~~（复审实测不可复现，见 F-R2）SET 语义入册：ui-default-styles.md（GET-null/
  SET-crash 契约）。spec 增量 SD-01..03 落三册。**基线刷新完成（f01dff6）**：
  078/079 物化面全量重跑——078 78 case + 079 4 app case 双模式全绿（vm
  snapshot-ok + vue http-ok）；过程中定罪并修复 gallery ForgeStore 适配器
  漂移（缺 T-02 SetWorkspace msg → 物化 app 链接 Undefined symbol，079
  VM 全 startup-failed 的确定性根因，420s 复跑同败排除间歇说）；基线
  截图刷新落 gallery tests/screenshots（gitignored 证据路径）+ 收据落
  tmp/ui-parity/PLAN-07{8,9}/。终局门：`check` catalog PASS（108 声明/
  109 case，live 3/4 ok——settings-rows 红于登记的字体度量余项 §10-10）。

## 9. 复审记录

- `stage: work | plan_id: PLAN-080 | plan_revision: 1 | outcome: §10-9 自动化
  复现未遂（保持间歇），足迹桩常驻续采 | code_commit: auto-lang 34800cf51
  （Rich span 色回退层移除=纯继承恢复） | evidence: ①自动导航压测：会话↔
  文件 30 轮 + 全导航（会话/计划/规范/知识库/文件/白名单）40+ 按压零复现
  （tmp/nav-crash-probe.mjs、nav-all-probe2.mjs）；②用户线索"点开它就挂
  （文件/沙箱导航）"未能在按压层复现——崩溃触发面比导航切换更深（疑似
  特定会话内容/时序）；③足迹桩常驻：每例死亡留 entering/returned 二分+
  30s 心跳死亡时刻+panic backtrace 兜底 | blockers: §10-9 根因（需带桩
  等待下一次死亡样本+死前屏幕内容对照）；§10-10；F-UAT-2 表格空文本
  观察项 | next: work（带桩续采，用户侧死亡报告=时刻+屏幕内容）`——2026-09-21
  （四）。


- `stage: work | plan_id: PLAN-080 | plan_revision: 1 | outcome: §10-9 足迹
  首战告捷+Rich 形态经用户确认恢复 | code_commit: auto-lang 97103f867+34800cf51
  +94be817ad | evidence: ①用户真机确认 Rich 段落排版正常（"一度解决了"）→
  Row 回退撤销、Rich 恢复（97103f867）；②OnBackground 强制色回退层经时序
  对齐定罪为"乱回去"嫌疑（解析疑落深色→深底深字）→ 移除恢复纯继承
  （34800cf51，=用户验证态 b3ab1520 行为）；③§10-9 足迹桩首战（94be817ad）：
  带桩实例死亡位于 iced 事件循环内部（X9 entering 有/returned 无/X9-PANIC
  无），死前最后输出=workspace 下拉+settings 构建 WARN 洪峰（用户"点开它"
  吻合）；WER 零记录/exit-audit 无 code=1/CLI exit(1) 全先打印/依赖源码
  （iced/winit 0.30 移除 process::exit）无静默退出——外部终止或未审计
  退出二选一，桩持续收集 | next: work（§10-9 桩续采+§10-10）`——2026-09-21
  （三）。


- `stage: work | plan_id: PLAN-080 | plan_revision: 1 | outcome: F-UAT-1 收口
  （真机绿）+ F-UAT-2 Rich 重构试错回退（基础设施保留待专项）+ §10-9 排查
  推进（外部终止/未审计退出二选一）| code_commit: musk 22960ca+099eed7 +
  auto-lang bad186d58+93e1cb420+5165cbf1f | evidence: ①F-UAT-1 全链（探针
  语义实证→字节安全重写→真机截图中文全文→vtree 文本在场→回归钉 2 绿）；
  ②F-UAT-2：View::Rich 基础设施+八站点映射落地（93e1cb420），真机发现
  Rich 节点在 into_iced 前被中间层吞没（RICH-ARM 零命中+表格/段落文字整体
  缺失，比散架更劣）→ 回退 Row 形态（5165cbf1f），真机截图确认表格恢复+
  空泡保持修复；Rich 启用前置=吞没层定位（aura_view_builder 后处理/渲染
  管线中段）或自定义回流文本 widget。③§10-9：WER 零记录+exit-audit 无
  code=1+main.rs exit(1) 全先打印+Start-Process 分离实例亦死——外部终止或
  未审计静默退出二选一；iced 循环收尾足迹方案就绪未部署。 | blockers:
  F-UAT-2（Rich 管线断点）/§10-9（根修）/§10-10（字体度量族）/live case
  CJK 变体 | next: work（续）`——2026-09-21 文本布局族修复轮（二）。

- `stage: work | plan_id: PLAN-080 | plan_revision: 1 | outcome: F-UAT-1 修复
  收口（真机验证绿）；F-UAT-2/§10-10/§10-9 在途 | code_commit: musk 22960ca
  （+099eed7 免登录）+ auto-lang bad186d58 | task_ids: F-UAT-1 闭环；T-01 重开
  项部分（v-html 链）；T-04/T-05 在途 | evidence: ①F-UAT-1 根因=VM 字符串
  原语语义不一致（探针实证 '你好ab'.length=4 字符 / sub(0,3)='你' 字节切片
  ——length 字符 × sub 字节，str-parity audit B1+Plan 057 钳制）；musk
  render_mentions 逐字符扫描在 CJK 上 sub 恒空串 → out=""→ v-html 降级链
  div_html_stripped_text 空串→None→空 div。修=split("@") 分段+段内字节游标
  （token 恒 ASCII；尾段整段拼接 LEN 字符数请求必覆盖尾由钳制收口），
  双世界正确。真机实例截图：原空泡完整显示中文全文。②headless 回归钉落
  auto-lang bad186d58：同构泡 CJK/latin 测宽 112/86px 正确=测宽层无罪；
  bisect 文档化 MaxWidthPct 子树 text 节点对 Find/选择器操作不可见（§10-11
  伪影家族候选根因=tag/state 委托 Tree 错位，待专项）。③§10-9 新数据：
  Start-Process 分离启动的浸泡实例亦静默死亡（非 agent 壳回收）；无 WER
  记录、exit-audit 无 code=1、main.rs exit(1) 位点全先打印——外部终止或
  未审计静默退出二选一，压力+足迹方案继续。 | blockers: F-UAT-2（markdown
  段落 Row-of-spans 不可回流，需 Rich 单段落重构）/§10-10/§10-9 | next: work
  （续修三项）`——2026-09-21 文本布局族修复轮（用户真机 UAT 驱动）。

- `stage: work(uat) | plan_id: PLAN-080 | plan_revision: 1 | outcome: UAT 两发现 →
  needs_fix | code_commit: 无新代码（UAT 于 f5d5558 + auto-lang 7d6888076 实机链） |
  task_ids: T-01 重开（F-UAT-1）、T-04 在途（§10-9 升级阻断） | evidence: ①F-UAT-1 CJK
  空泡（P1 回归）：用户实机报"中文消息空泡、英文正常"；取证链=后端 GET
  /api/chats/session/95c167e8?workspace=backend 存储 content="你好"/"请列出当前目录的内容…"
  完好（输入/传输排除）→ MCP autoui_snapshot + autoui_screenshot（muskc 窗口）证明侧栏/
  助手 markdown/输入框中文全渲染、用户泡 latin 渲染 → 定罪 max-w-[70%] pct 臂 CJK 测量
  ≈0（cee16d694 引入；修前该泡满宽可见=文本在，修后宽度对而 CJK 灭）；live case 盲区=
  拉丁 marker 夹具无 CJK 变体。②§10-9 升级 P1 阻断：UAT 会话 VM 4 例 3 死（16s/~2min/
  ~5min，静默 exit 1 无 panic，尾迹仅 handler 噪声），跳登录/使用中均触发。
  ③环境注记：9247 为用户 auto-edit MCP 占用，musk MCP 落 9248；autoui_screenshot 落盘
  跟随 app cwd（曾误写 auto-edit 检出 tmp/，已清理）。 | blockers: F-UAT-1 + §10-9 根修 |
  next: work（CJK pct 臂修复 + live case CJK 变体；§10-9 根修升最优先）`——2026-09-21
  用户真机手验会话（真后端 17201 + 真实 ai-daemon 17654 + 复用门）。手验清单 A-E 项中
  popover/宽度等其余项待崩溃债收敛后继续。

- `stage: review | plan_id: PLAN-080 | plan_revision: 1 | outcome: blocked
  （终局 reviewed 的前置=用户验收决定；计划保持 executing，T-01/T-02/T-03/T-06
  闭环面全部复核通过） | reviewed_commit: auto-musk f5d5558（worktree
  plan-080-dev clean） | base_commit: d313fa1 | dependency_revisions:
  auto-lang 7d6888076（worktree clean） | spec_inputs: ui-default-styles.md
  b3c04591… / ui-parity.md 648a9270… / ui-compositions.md 8a815b57…
  （88f4ad2，与 r2 复审哈希一致；SD-02 经 F-R1 后与代码相符） |
  acceptance_results: AC-01 pass / AC-02 pass / AC-03 partial（真机手验）/
  AC-04 pass / AC-05 partial（崩溃根修+真机手验）/ AC-06 partial（字体度量
  族）/ AC-07 pass | findings: 无新增（余留三项均为计划内已登记开项） |
  evidence: 下 | next: 用户二选一（见 blocked 解除条件）`
  —— 独立性限制说明：终局复审与修复轮同会话，裁定全部自工件/命令重建。
  **①F-R1 终局复验**：离线 check=PASS exit 0（108 声明/109 case，live 3/4
  ok——三份 PASS 收据 LIVE_SOURCES 哈希未 stale，⏭ settings-rows 留痕）；
  check --live=❌ 1 issue exit 1（硬红兑现）。**②F-R2 终局复验**：终局基线
  复跑 `auto build` 全 pipeline 绿（vue-tsc && vite，11.25s）；收据
  docs/reports/ui-parity/080-web-gate.p660d1.md 在树（f5d5558，含修前双红/
  绕行/修后全绿与基线有效性论证）；P660-D1 已在 auto-lang 债册销记
  （7d6888076）；cargo tf 3649/3652 于 7d6888076 内容上运行（3 败=已定罪
  前置红，零新增——复用理由：同一提交内容、修复轮内全部编辑后运行）。
  **③沿用证据**（复用理由：相关面自 r2 复审以来零代码变更——9b0e857 仅
  scripts/ui-parity.mjs、f5d5558 仅 docs）：AC-02 chats 像素右缘 1263.5≈1264
  （hits 32k、ratio 0.674≤0.72）、AC-03 ws 路径双臂+sessionClickOk 双臂
  true、AC-05 锚定/re-press/外点关双臂绿、AC-04 musk_vm_track 63/63+button
  20/20。**blocked 解除条件（二选一）**：(a) 用户完成真机手验清单（外点/ESC
  关闭、点击每类会话、stale 会话反馈——AC-03/AC-05 尾项）且 §10-9/§10-10
  auto-lang 独立收口完成后回本计划终验 → re-review → reviewed；(b) 用户裁定
  带债落地：§10-9/§10-10/真机手验移出本计划（KNOWN-DEBT 或后续计划承接）=
  验收合同修订，需明示授权后 re-review 按 pass-with-debts 出终局。复审不改
  实现、不单方扩范围——§10-10 字体度量族若裁定入本计划收口，走 work 指令。

- `stage: work | plan_id: PLAN-080 | plan_revision: 1 | outcome: F-R1/F-R2 修复收口 →
  review | code_commit: auto-musk f5d5558（+9b0e857）+ auto-lang 7d6888076 |
  task_ids: T-01/T-06 复审重开项闭环、T-04/T-05 保持在途 | evidence: F-R1 check 门双档
  复验（离线 check=PASS exit 0/⏭ 留痕不变；check --live=❌ 1 issue exit 1——硬红兑现，
  settings-rows 红=T-05 在途的正确红）+ F-R2 P660-D1 收口（auto-lang 7d6888076：scaffold
  vite-env.d.ts 三处发射 + auto-sources.ts build 发射 + run 自愈；musk auto build 全
  pipeline 绿 14.7s + examples/ui/014-weather 原始复现点全绿 1.59s + cargo tf 3649/3652
  零新增=3 前置红）+ 收据 docs/reports/ui-parity/080-web-gate.p660d1.md（f5d5558，含
  修前双红/绕行/修后全绿全记录与基线有效性论证）+ F-R4 陈旧红收据清理 + check 复跑
  PASS（live 3/4 ok） | blockers: T-04 余（§10-9 崩溃根修=auto-lang 独立收口、真机外点/
  ESC 手验=用户验收项）、T-05 余（§10-10 字体度量族=auto-lang 独立验证周期）仍开 |
  next: review`——2026-09-21 needs_fix 修复轮（r2 复审输入 F-R1/F-R2）：F-R2 采 P660-D1
  收口路线（复审提供"收据改录或 P660-D1 收口"二选一），web 门由"绕行绿"升为**真绿**；
  scaffold 变更纯类型面+dev-only 模块，不改 Vue 运行时渲染与 VM 面，既有 078/079 物化
  基线与 080 live 收据保持有效（论证见收据）。T-04/T-05 既有开项未动。

- `stage: review | plan_id: PLAN-080 | plan_revision: 1 | outcome: needs_fix |
  reviewed_commit: auto-musk f01dff6ea81abc3483f2c739d60ddaa05a7b8f68（worktree
  plan-080-dev，clean） | base_commit: d313fa1 | dependency_revisions:
  auto-lang 26b8cca44（+71f8027ae/cee16d694/0cbf5a031；基线 172536658，worktree
  clean） | spec_inputs: ui-default-styles.md sha256 b3c04591…、ui-parity.md
  648a9270…、ui-compositions.md 8a815b57…（88f4ad2 落册） |
  acceptance_results: AC-01 pass（F-R1 属 SD-02 契约缺陷）/ AC-02 pass /
  AC-03 partial（真机手验+stale 会话反馈未验）/ AC-04 pass（F-6 P3 挂开）/
  AC-05 partial（崩溃根修+真机手验未完）/ AC-06 partial（字体度量族=§10-10
  登记债）/ AC-07 partial（web 门不可复现 F-R2） | findings: F-R1（P1）、
  F-R2（P1）、F-R3（P3 已就地补正）、F-R4（P3） | evidence: 下 | next: work
  （F-R1/F-R2 修复 + T-04/T-05 既有开项；修毕复审终局）`
  —— 复审在独立会话进行（非执行会话），结论自命令输出与工件重建。逐项证据：
  **①基线**：双 worktree clean；live 收据（chats/popover/ws PASS、rows fail）
  LIVE_SOURCES sha256 与 HEAD 四源文件逐一相符（stale 机制判定非过期，显式
  复用理由：源未变+机制自证有效性）；078 目录 76+76 绿 + 2 红收据已被
  PLAN-079 目录 23:25-23:26 补跑绿收据（6 case ×2 模式全绿）取代（F-R4 陈旧
  未清）；check 离线复跑=catalog PASS（108 声明/109 case，live 3/4 ok）⏭
  留痕复现。**②auto-lang 测试复跑**：cargo tf --no-fail-fast=3647 跑 3647 绿
  5 败——4 败在干净基线 172536658 临时 worktree 复跑同败（autodown_panel_
  heading/mouse_area/shell_pack_vocabulary/ffi_dual_019=前置红），1 败
  （benchmark_downcast_performance）独立 3 连跑全绿=负载敏感 flaky；**零计划
  回归**。musk_vm_track 63/63 绿（含 p080 回归钉）、button 面 20/20 绿、
  gallery_pages_compile 绿。**③web 门复跑（F-R2 定罪）**：release 二进制增量
  重建至 HEAD 后 `auto build` 红——干净 regen 的 vue-tsc 双红：TS2307
  （gen/front/vue/src/auto-sources.ts 缺失，write_auto_sources_ts 仅
  auto run/incremental 路径发射、build 不发射，而 build 产出的 overlay.ts
  导入之）+ TS2339（main.ts:32 import.meta.env，scaffold 无 vite/client
  types）；补 auto-sources.ts 后 TS2339 独立复现=类型面与生成物无关。
  `pnpm exec vite build` 绕行绿（14.31s）。根因=P660-D1 预存生成器债
  （KNOWN-DEBT-AND-RISKS.md:2405；444b87fec 在 080 基线内）——**T-06 记录的
  "全 pipeline 绿 ×2"在本基线不可复现**（×2 绿系陈旧/手补 gen 树假象）。
  **④F-R1 定罪**：ui-parity.mjs check 分支——catalogCheck() 内 printIssues
  先于 live 升格执行、live 行 append 后无人打印，且 check 分支不设
  process.exitCode；实测 `check --live` 在 settings-rows 收据 status=fail
  时零输出退出 0。SD-02"check --live 将 missing/stale/failed 升格硬红"
  代码未兑现；修法=check 分支尾 printIssues(issues)+exitCode 置位（含目录
  红时 exit 1）。
  **F-R1**（AC-01 契约面/T-01/SD-02，P1）见上④。**F-R2**（AC-07/T-06，P1）
  见上③。**F-R3**（P3，已就地补正）frontmatter supersedes 缺
  docs/specs/modules/ui-compositions.md（SD-03 target）——本复审已补入。
  **F-R4**（P3，记账）PLAN-078 收据目录残留 2 份已被 079 目录补跑取代的红
  收据（shell-workspace-selector-vm startup-failed@22:25、
  inventory-RelationsPanel-vue timeout@22:57），"全绿"叙述以 079 目录为准；
  建议清理或标 superseded 防误导 triage。
  **路由**：work——F-R1（musk 小改+复验）、F-R2（收据改录或 P660-D1 收口
  后重跑）；T-04 余（§10-9 崩溃根修 auto-lang、真机外点/ESC 手验=用户验收）、
  T-05 余（§10-10 字体度量族 auto-lang 独立周期）、F-6（P3 目检）不变；
  stale 会话 404 反馈定罪并入真机手验清单。tmp/ 收据为 gitignored 易失路径，
  关键数字已录本记录。

- `stage: work | plan_id: PLAN-080 | plan_revision: 1 | outcome: F-1/F-2/F-3(断言面)/F-4(宽度面)/F-5/F-7 修复收口 →
  review | code_commit: auto-musk f82bdf9+f01dff6（+ac9cca4/1488482/88f4ad2）+
  auto-lang 26b8cca44（+71f8027ae/0cbf5a031/cee16d694） | task_ids: T-01/T-02/T-03/T-06 闭环、
  T-04/T-05 收窄 | evidence: live r2-r7 真机链（chats-message-width PASS
  像素右缘 1263.5≈1264 hits 32k / workspace-selector-current PASS 触发器路径
  双臂+sessionClickOk 双臂 true / settings-popover PASS 锚定+re-press /
  settings-rows 宽差 66.7→2.2px 剩字体度量族）+ probe080 债③复现验证
  （Http.get 修前 to_str="2" 句柄、修后 workspaces.length=1 全链通）+
  musk_vm_track 63 测绿（含 p080 回归钉 4 测）+ auto build 绿 ×2 + 基线
  刷新 078/079 物化面全绿（78+4 case 双模式，含 gallery 适配器漂移定罪
  修复 f01dff6）+ check catalog PASS（live 3/4 ok） | blockers: 见 §10 余项
  （9 崩溃债/10 字体度量族=新立；4 真机手验=用户验收项） | next: review`
  ——2026-09-20 修复轮（复审 needs_fix F-1..F-7 输入）：**F-1** 像素真值锚裁决 VM
  视觉右对齐正确（快照 @rect capped 盒伪影；纯 Node PNG 解码+色相域+列密度
  聚类，两坑退役）；**F-2** 债三连同根=UI 路径裸 Http.get 落句柄（auto-lang
  补 get_json+to_value 改写），vue 镜像坑=async fn 裸调用 Promise 恒真短路
  （内联修），VM 面全链复验绿；债②不复现；**F-3** re-press 验证通过（旧判据
  假红：content 预构建文本恒在；改面板宽 rect 判据），间歇静默 exit 1 取证
  备案未根修；**F-4** 面板宽双轨错位根因（class 顶替 W9 缺省 chrome）摘除
  收口 2.2px，剩字体度量族；**F-5** 基线刷新完成（gallery 适配器 SetWorkspace
  漂移定罪修复，079 VM startup-failed 确定性根因）；**F-7** vue 重试断点改
  非占位；vm 会话点击改位置锚（行按钮快照恒无样式，vtree 覆盖缺口另案）。

- `stage: review | plan_id: PLAN-080 | plan_revision: 1 | outcome: needs_fix | reviewed_commit: auto-musk ac9cca4cdcd6b619684e52e6d1b3cb1480ac8b51 | base_commit: d313fa1 | dependency_revisions: auto-lang 71f8027ae3c11022c83cff57fc5c7f5166d5a7c1（基线 172536658） | spec_inputs: ui-default-styles/ui-parity/ui-compositions（worktree 88f4ad2 增量，复审通过——描述现行为与持久决策） | acceptance_results: AC-01 partial / AC-02 partial / AC-03 partial / AC-04 pass / AC-05 partial / AC-06 fail-strict / AC-07 partial | findings: F-1..F-7（下） | evidence: 修前红定罪收据 4 份（docs/reports/ui-parity/080-*.pre-fix.json）+ live 终局收据（tmp/ui-parity/PLAN-080/，chats ratio 0.675≤0.72 绿 / popover 锚定 0 issue / ws y 5px）+ cargo t 双侧同比（98 跑 92 过 6 败，分支与干净基线 172536658 失败面一致=零回归）+ web build 绿×4 + check 门 PASS（live 显式 skip） | next: work（F-1..F-5 修复，F-6/F-7 可随批）`
  —— 复审在实施会话内进行（独立性限制）：结论自工件重建——收据数字、
  双侧测试比对、提交链，非执行叙述。F 清单：
  **F-1**（AC-02/T-01，P1）chats 右缘非确定性对齐（vm 左置 1014 vs
  probe5 截图右对齐证据矛盾；像素锚两坑：canvas 跨源/扫描挂点）。
  **F-2**（AC-03/T-02，P1）VM workspace 显示阻断于 auto-lang 债三连
  （store Init workspace_list 返空/数组索引返索引/裸 Http.get 恒空）；
  musk_vm_track 基线 6 前置红疑同族，宜 auto-lang 侧独立收口。
  **F-3**（AC-05/T-04，P1）VM popover re-press 关闭不过 + settings 交互点
  间歇崩溃（mt-auto×popover 嫌疑）+ 真机外点/ESC 手验清单未走。
  **F-4**（AC-06/T-05，P2）rows 残差：x 5.4px 近预算、y 29-57px（行高
  映射）、w ~190px（w-80 面板宽映射缺口）。
  **F-5**（AC-07/T-06，P2）ui-parity 截图基线刷新未做。
  **F-6**（AC-04，P3 非阻断）015-notes 抽样目检未做（核心判据已由三表
  测试+零回归证）。
  **F-7**（测量，P3）workspace vue 文本重试以非空为断点，早测截停占位
  （run5 已证链路通；断点应改非占位）。

- `stage: work | plan_id: PLAN-080 | plan_revision: 1 | outcome: 余项收窄完成 →
  review | code_commit: auto-musk ac9cca4（+1488482/88f4ad2）+ auto-lang 71f8027ae
  （+0cbf5a031/cee16d694） | task_ids: 余项 2/4/5 | evidence: mt-auto 修复
  live 复跑（ws-trigger.y 314→5px、settings-rows y 261→29-57px）+ chats
  ratio 判据 live 绿（0.674≤0.72）| blockers: 见 §10 余项（1/3/4/5/6/7/8）
  | next: review`——2026-09-20 收窄会话：余项 2 根治（mt-auto 折 0 丢推底语义）；
  余项 5 半收窄（ratio 绿 + 右缘非确定性对齐开放问题）；余项 4 扩记 VM
  settings 交互间歇崩溃。

- `stage: work | plan_id: PLAN-080 | plan_revision: 1 | outcome: partial →
  executing（主体落地，余项明确） | code_commit: auto-musk 1488482 +
  auto-lang 0cbf5a031/cee16d694 | task_ids: T-01..T-06 | evidence: 修前红
  定罪收据 4 份（docs/reports/ui-parity/080-*.pre-fix.json）+ live 终局
  收据（tmp/ui-parity/PLAN-080/）+ probe5 截图视觉验证 + web 门 build 绿 ×3
  + spec 增量三册 | blockers: 见 §10 余项 | next: review（余项收窄后）`
  ——T-01 仪器全链建成且完成修前红→修后绿闭环（核心 ratio 断言）；T-03
  完整落地；T-04 迁移完成（web 双判据绿）；T-02 web 全链绿/VM 债三连；
  T-05 测量面通；T-06 主体完成。2026-09-20 work 会话。

- `stage: new | plan_id: PLAN-080 | plan_revision: 1 | outcome: pass |
  next: work`——五项用户裁定 + 本会话 hotfix 余项全量入册；hotfix 已落部分
  （d313fa1/34ee15a47 等）作为前置事实记录不重做。起草会话：2026-09-20。

## 10. 待澄清事项

- ~~T-01 真机采集通道~~ **已定案（2026-09-20 work 会话）**：AutoUI MCP
  `autoui_snapshot include_bounds=true`（@rect(x,y,w,h)；LayoutCollector →
  InspectorCache 链 PLAN-650 已完备，探针实证登录页/登录后均产出 @rect）。
  截图几何探针不启用。Vue 臂 playwright getBoundingClientRect 同键对拍。
- ~~T-04 popover 原语落点~~ **已定案（2026-09-20 work 会话）**：auto-lang
  锚定 popover 原语**已存在**（Plan 422+ `popover`/`popover-trigger(as_child)`/
  `popover-content` 嵌套形态 + placement 词表 + 外点/ESC dismiss，PLAN-059 T9
  落地，WorkspaceSelector 在用）——零 auto-lang 改动，settings_menu.at 纯
  musk 侧迁移（dialog 受控 modal → popover 嵌套锚定，placement top-start，
  面板 chrome 走 popover 缺省 w-72 档 + class 定宽）。
- ~~AC-02 宽度预算具体值~~ **已定案**：同视口 1280×800（AUTO_VM_WINDOW 对齐
  playwright viewport）绝对像素差 ≤2px（沿 079 口径），另加 max-w-[70%]
  比例护栏（行宽/容器宽 ≤0.72，防百分比解析失效型回归）。

### work 会话新增调查结论（执行依据）

- **VM localStorage 跨运行持久化**（KD-048 落盘，缺省按 cwd 哈希定位）：
  残留登录态令新跑直跳 shell、登录表单永现——live 臂以
  `AUTO_VM_STORAGE_FILE` 指一次性文件每跑重置（定罪运行实证）。
- **后端隔离**：Windows `dirs::home_dir()` 走 Shell API，USERPROFILE/HOME
  覆写无效（首跑播种曾误入真实 auto-edit 工作区，会话/测试用户已清理）——
  musk 后端新增 `MUSK_CONFIG_DIR` env 门（users.json/workspaces.json 落点
  覆写，缺省行为不变），live 臂配隔离 home + 每跑唯一时间戳目录。
- **快照稳定性**：交互后半重建帧的 styled_vtree 呈混合态（样式/事件错位、
  @rect 缺席）；且快照文本按渲染宽度裁剪（中文长文全文匹配恒假）——live
  臂按"连拍两次一致才采集"稳定门 + 拉丁前缀 marker 断言。
- **登录按钮形态**：VM 快照中 button label 可为空、文字在子节点——匹配走
  子树文本聚合而非 header label。

### work 会话余项清单（2026-09-20，review/后续计划输入；修复轮状态标注）

1. ~~**VM workspace 回填链三连债（auto-lang）**~~ **已收口（修复轮，auto-lang 26b8cca44）**：根因唯一=UI（aura/front .at）路径裸 Http.get 落 auto.http.get 句柄——vm/codegen.rs Call 臂补 get_json+to_value 改写（脚本 bigvm 路径 PLAN-442 本有，UI 路径缺口）；②数组 for-in/索引债当前构建不复现（probe 实证正常）；③同根收口。musk relay_store/agent_configs/settings_forge_helpers/tool_gate_card 整族同修。musk_vm_track 基线 6 前置红未复评（本轮 63 测绿含新增 p080 回归钉 4 测；icon_component_child… 干净 master 同红项独立排查不变）。
2. ~~**VM rail 底部锚定 y 偏移 ~310px**~~ **已收口（71f8027ae mt-auto 修复）**：ws-trigger.y 314→5px、settings-rows y 261→29-74px（剩字体度量族）。
3. ~~**settings 面板宽映射差**~~ **已收口（f82bdf9）**：popover 标签 class 两轨语义错位根因（VM 侧顶替 W9 缺省 chrome），摘除后双轨同归 w-72 缺省（66.7→2.2px）。~~行高差 5.8-12px~~ → 余项 10。
4. ~~**VM popover re-press toggle 关闭**~~ **已验证通过（f82bdf9 断言修正）**：旧 includes('GSD') 判据假红（content 预构建文本恒在树中）；改"GSD 行 @rect 面板宽=开"判据后 live PASS。**真机外点/ESC 手验清单仍留**（MCP 合成事件不经 overlay 路由，用户验收项）。
5. ~~**chats-message-width 右缘锚点 nit**~~ **已收口（f82bdf9 像素真值锚）**：纯 Node PNG 解码+色相域（覆盖 89,99,207/147,148,245 两档 primary）+列密度聚类；裁决 VM 视觉右对齐正确、快照 @rect capped 盒伪影；case PASS。
6. ~~**ui-parity 截图基线刷新**~~ **已完成（f01dff6 基线刷新轮）**：078/079 物化面全量重跑双模式全绿（078 78 case + 079 4 app case + 2 补跑），过程中定罪修复 gallery ForgeStore 适配器漂移（缺 T-02 SetWorkspace msg → 079 VM 全量 startup-failed 的确定性根因）；基线 PNG 落 gallery tests/screenshots（gitignored 证据路径），收据落 tmp/ui-parity/PLAN-07{8,9}/。终局 check：catalog PASS（108 声明/109 case，live 3/4 ok）。
7. **auto-lang vue dev 脚手架 codegen 缺口**（不变）：`auto run --render vue` 生成命名导入无 .vue 扩展——live 臂已绕行（dist 静态服务）。
8. **auto-lang 前置红**（不变）：icon_component_child…（干净 master 同红，独立排查）。
9. **[升级·UAT 实证] VM app 间歇静默 exit 1（auto-lang 债，P1 阻断级）**：2026-09-21 UAT 会话
   **4 例 3 死**（16s / ~2min / ~5min 存活，各自静默 exit=1 无 panic；死亡尾迹仅 handler-not-found
   噪声行 `__scroll_state_read`/`__mcp_heartbeat`，与 §4 删会话后端 exit 1 同签名家族）；**触发面
   扩大**：备案为"登录 Submit 后（~2/8）"，本轮跳登录（持久化态直跳 shell）与使用中（MCP
   截图后）均死——ws_resolve_current 内联规避不充分。用户手验被此债实际阻断。auto-lang 根修
   升为最优先；凭证：tmp 后台任务日志（会话 exec 归档）+ 本行记录。
10. **[新增] VM 字体度量映射族（auto-lang，T-05 余）**：text-xs 行高映射缺 line-height（VM 21px vs web 15.2px，疑缺省 leading ~1.75 兜底）；popover 面板锚 x 偏移 ~11px；按钮高映射 ~12px（modeRow h 50 vs 38）。影响 settings-rows h/x/y（y 为面板高差派生 ~70px）。修复落点=auto-lang 样式映射（text-* → font-size+line-height 对），全局生效需独立验证周期（examples 面回归）。
11. **[新增·测量] vtree 快照覆盖缺口**：会话行按钮恒无样式行/空标签（session-item 类不可锚）；popover content 预构建（文本恒在）；capped/overlay 盒 @rect 伪影（视觉右对齐而 @rect 左置）——气泡/overlay 族几何对拍一律像素锚（live.mjs 已内置），vtree 覆盖面改善归 auto-lang 侧观察。
12. ~~**[新增·复审 F-R1] `check --live` 硬红语义未兑现（musk，T-01 余）**~~ **已收口（修复轮，musk 9b0e857）**：升格行收拢 liveIssues 补 printIssues + issues 非空置 exitCode；复验离档 PASS exit 0/⏭ 留痕不变、`--live` ❌ exit 1；SD-02 文本与代码一致。
15. **[新增·UAT F-UAT-2] 助手 markdown 长段落排版崩坏（P1，auto-lang VM 文本布局族）**：用户实机截图（中英两例）——①折行续行**居中**（应左对齐；纯文本+bold 段落即现）；②含行内 link/code 的段落**巨大空隙**（span 宽度测量或 justify 语义错，文字被推散）。路径=autodown markdown → VM 行内布局（PLAN-076 交付面），与 F-UAT-1（用户泡 pct 臂 CJK 测量≈0）、§10-10（text-* 字体度量映射缺）疑同根：VM 文本测量/布局层需一次成体系收口（测宽→折行对齐→行内 span 宽度三面），修程带中英长段落 live case 变体。
13. ~~**[新增·复审 F-R2] web 门"全 pipeline 绿"不可复现（T-06 余）**~~ **已收口（修复轮，采 P660-D1 收口路线：auto-lang 7d6888076）**：vite-env.d.ts 三处发射 + auto-sources.ts build 路径发射；musk auto build 全 pipeline 绿（14.7s）+ 014-weather 全绿（1.59s）+ tf 零新增；收据 docs/reports/ui-parity/080-web-gate.p660d1.md（f5d5558）。
14. ~~**[新增·复审 F-R4·记账] PLAN-078 收据目录 2 份陈旧红收据**~~ **已清理（修复轮 2026-09-21）**：shell-workspace-selector-vm、inventory-RelationsPanel-vue 两份被 079 目录补跑取代的红收据已删除；check 复跑 PASS。
