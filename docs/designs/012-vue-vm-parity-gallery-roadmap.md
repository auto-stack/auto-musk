# 012 — Musk Vue / VM 一致性：默认规约、统一引擎、组件 Gallery 与 App 验收

> 日期：2026-09-19；状态：提案，待后续按计划执行；本次仅设计与计划，不代表实现完成。
> 用户方向：不引入完整 CSS 引擎；Vue CSS 与 VM 通用初始化以同一规约对齐；
> AutoDown 统一消费 autodown-engine；先组件一致，再组合与 App 一致。
> 本仓设计目录沿用 `docs/designs/`，不另建平行 `docs/design/`。

## 1. 结论与边界

应建立独立的 `musk-widgets-gallery`。消息 Block 状态有限、可构造、不应依赖真实 LLM；
隔离后能直接比较同一输入和同一交互。它是产品组件的验收入口，不是另一份 UI 产品。
先锁定 L1 组件，再锁定 L2 消息/输入/页面组合，最后验证 L3 App 流程。
组件分别合格并不自动证明组合合格：继承、可用宽度、滚动和焦点必须在 L2/L3 再验。

范围：`auto run -r vue` 的生成 Vue 与 `auto run -r vm` 的原生 Iced。
`web/` 历史手写前端不作为新基准；`gen/` 不作为手工修复真源。
近期目标：结构、功能、状态和主要几何/排版大体一致；最终目标：受控环境内像素级一致。
不替换 Iced 为 WebView，不实现完整 CSS，也不为每个平台维护两套业务组件。

## 2. 依据与当前差异（源码调查，不是本次运行截图结论）

- 权威项目知识：`docs/specs/00-overview.md`、`01-architecture.md`、
  `03-front-component-groups.md`、`modules/chat-streaming.md`、`modules/web-input-contracts.md`、
  `modules/vm-data-semantics.md`、`modules/vm-process-stability.md`、`modules/files-browser.md`。
  overview/组件清单仍描述旧 web 双轨、ThinkBlock 文件及旧 markdown 库，不能据此认定当前消费路径；
  计划明确提出 Spec 修订，设计阶段不改 canonical Specs。
- 默认规约：`../auto-lang/docs/design/autoui/base-styles-and-visual-parity.md`（Design 22）。
  §2–5 应用默认属性；§4.5/4.6/7 引擎文档面。相关机制：Design 29/30。
- Jade 先例：`../auto-down/jade-garden/front/component-gallery/`；
  `../auto-down/docs/plans/attachments/072-inventory.md` 及 archived/071、072、073。
  复用其 inventory、fixtures、双端证据思路，不复制绝对路径 link 依赖；本仓禁止 junction/symlink。
- Musk PLAN-072 为 AutoDown 三修，PLAN-073 为聊天运行修复，均仍在 execution_done；
  本路线不覆盖或重开它们，基于交付后的实际 HEAD，执行前再次核对状态与语义。
- ThinkBlock 当前不是独立 widget：`chat_message.at` 内联 thinking 分支，块级 state 区分
  streaming/done；ToolBlock/GenericToolCard 旧文件仍存在，但产品主链使用内联分派。
  历史原因注记包括 VM 多实例状态共享和孙件 store 调用不可靠；vm-data-semantics又记录部分读侧已修，
  不能据旧注释认定当前仍坏，需双实例实测。不得为 gallery 重引这些旧路径。
- `ports/renderer.vm.at` 的 Markdown 仍是纯文本；Vue 端已消费 `@autodown/engine`。
  当前可见 DSL 标签包括 `autodown` 与 `autodown_editor`（上游 showcase），并不能据此
  假定它们已满足统一引擎契约；P076 要核验注册、属性、模式与编辑事件的完整映射。
- 默认样式已部分建立：button variants 单源及部分互锁存在；ghost 仍排除，
  VM h1/h2 缺 tracking-tight，p/text 默认注入不足；文档 12px 块节奏与原生布局 8px 不同。
  §7.3 标题分叉记录又落后于当前代码（当前已用 25.3/21.3/18.9px）。
  此类差异需“规约/实现/实际最终属性”三方定责，不能盲改一侧或把文档改成现状。
- Musk Vue 注入含主题、字体、导航、内部文档覆盖；CSS 本身合法。
  问题是缺少 VM 等价初始化、共同规约条目和最终效果证明。

## 3. 三仓职责与唯一源

| 层 | 责任仓 | 应交付 | 不应放在此层 |
|---|---|---|---|
| AutoUI 基础控件/样式 | auto-lang | 默认属性与覆盖优先级、主题、字体、布局、焦点、原生渲染、诊断 | Musk 业务分派、某条消息硬编码补丁 |
| AutoDown 引擎 | auto-down（必要时联动 auto-lang 原生注册/后端） | autodown-engine view/stream/edit 双端契约、块排版/高亮/编辑/流式 | Musk 通过深层 CSS 或另造 Markdown 渲染器修引擎 |
| Musk 组件与 App | auto-musk | 同一组件源、状态/事件、gallery、主题声明、宿主组合、引擎适配 | 私有文档解析器、复制一份组件供 gallery 使用 |

物理实现可能跨仓，逻辑契约仍归对应层。先定位缺陷所有者，再在同组兄弟 worktree 修复。
跨仓接口调整必须有消费验证与版本/产物收据；依赖被成功消费后及时合回清理，不悬挂。
本次用户授权调查、总设计和多个 draft；没有授权把本设计视为已经完成的实施。

## 4. 组件盘点与 Gallery 范围

完整词法 widget 清单见附录 A；它是初始盘点，不等于运行可达性证明。
P074 必须追踪从 App 的 import、端口变体、条件分派到各组件的真实消费路径，归三桶：
①双端有实现（差异待验）；②仅 Vue 有；③仅 VM 有。另标记 retired/unreachable，不能算通过。
每项保存 stable ID、源路径/符号、父消费者、状态场景、依赖、修复归属和双端证据。

| Gallery 单元组 | 必须覆盖的当前生产形态 | 核心状态 |
|---|---|---|
| 消息 L1/L2 | UserMessage、ChatMessage；内联 thinking/text/tool 分支 | 用户/助手身份、空/短/长、中英、历史回放、流式、错误、分叉 |
| ThinkingBlock 逻辑单元 | ChatMessage 内联分支（不是已删除 think_block.at） | streaming 尾部预览、done 折叠/展开、多个块独立状态、跨轮收束 |
| ToolBlock 逻辑单元 | 生产内联通用工具卡、ToolGateCard、ErrandCard、TaskPlanCard、RelayRunBox、ReportCard | running/completed/failed/gate_waiting、空/长参数、长结果、审批/拒绝、嵌套运行 |
| 其他业务卡 | QuestionnaireCard、GateCard、SecretaryMessage/Wrapper、StreamingTable | 可达性核实后纳入有效场景；只遗留则记录替代/退役，不复活 |
| 输入与会话 | MentionInput、MentionDropdown、AgentAvatar、SessionInfo | 中英输入、IME、mention 键盘、发送/停止、禁用、聚焦、多行 |
| 导航与壳 | NavSidebar、NavListItem、ContentHeader、WorkspaceSelector、SettingsMenu、LoginPage、DeleteConfirmDialog | 展开/收缩、主题/语言、选中/悬停、菜单/弹窗、错误 |
| 页面内容 | FileTree/TreeIcon、WikiNav/RawPreview、Specs 叶/类别/详情/编辑器 | 长树、搜索、空/错/加载、编辑/保存、窄宽度 |
| 引擎消费探针 | renderer 端口、specs_editors、文件/计划/Wiki/报告正文 | autodown-engine view/stream/edit；只测宿主传参和尺寸，深层缺陷回上游 |

消息块不必第一天全部重新拆成 widget。先用真实 ChatMessage + 单块 fixture 隔离测试。
需要抽离时采用 props + 显式事件、状态由父层按稳定 block ID 持有；先验两实例独立、
排序/增量更新不串状态，再切换产品消费。不允许仅为 gallery 维护第二份内联模板。

## 5. Gallery 架构与确定性

新增独立入口 `examples/musk-widgets-gallery/pac.at`、`src/front/app.at`，
两端均从该目录用 `auto run -r vue` / `auto run -r vm` 启动。
组件复用优先使用编译器支持的跨目录模块导入。若模块根解析不支持，P074 首项探针
裁定：上游补导入，或使用只读自动物化并校验源 hash 的构建产物；禁止人工复制和链接目录。
fixture 适配器只替换数据/时钟/网络/副作用，不替换组件渲染与状态逻辑。
Store 型组件用记录事件的假 API/SSE/存储隔离；不得连真实工作区审批、发消息或执行工具。
每次 reset 恢复相同 fixture、固定时间/ID/随机种子；stream 使用同一离散 chunk 序列与检查点。

建议新路径（均由 P074 实现，不是假称已有工具）：
- `tests/ui-parity/cases.json`：单元/状态覆盖与三桶清单。
- `tests/ui-parity/fixtures/`：共享输入和事件序列。
- `scripts/ui-parity.mjs`：list/check/run/report，具体接口在 P074 冻结并写 README。
- `tests/ui-parity/baselines/manifest.json`：基线元数据/hash，图片存忽略目录或外部制品。
- `docs/reports/ui-parity/`：差异、证据链接、版本、预算、所有者与验收记录。

Vue 用浏览器 DOM/交互/截图，VM 用 AutoUI MCP snapshot/真实输入/截图；
分配私有AUTOUI_MCP_PORT并读取实际端口，只清理自身PID，不能争抢默认端口或扫杀其他会话。
不能只用直接状态赋值证明焦点、点击或 IME。重用 auto-lang autoui-verifier 工具，
Jade gallery 的 fixtures 模型；`vm-link-probe` 和 `vm-first-run` 只证明链接/存活，不证明视觉。

## 6. 默认样式合同

继续以 AutoLang Design 22 为共同约定，应用补充条目放 Musk 默认样式映射表（P075 新增）。
不强制把合理 CSS 全迁走。旧docs/designs/010迁移矩阵按历史语境保留，P075补充本次裁定注记。每条至少含：
`rule_id / scope / element+state / expected / vue_base+injected / vm_preset+fallback /
user_override_precedence / owner / test_case / revision / evidence / status`。
区分基础控件、引擎内部与宿主品牌主题，不能拿应用 h1 字号替代文档 h1。
覆盖 normal/hover/focus/disabled/selected、深浅主题、显式部分覆盖及父级继承。
优先共享 token/recipe 数据源，双方静态投影；短期不能同源的行使用最终属性互锁。
CSS cascade 与类合并结果不同也须检测，不能只断言某个字符串出现过。
规范不清/互相冲突的条目先给决策记录，保留前后值；不可用降标让 gate 变绿。

## 7. AutoDown 收敛

Musk 消费统一 autodown-engine 的 view/stream/edit 契约；旧 Markdown/MarkdownRender
可暂留外观兼容适配，但内部只能转接统一引擎，最终清退纯文本降级。
P076 枚举 renderer.web/vm、components/MarkdownRender.vue、specs_editors、raw_preview、
files_view、plans_view、wiki_view、report_card 等全部真实调用，包括注册资产与 vendor stamp。
当前 DSL 标签并不是两个独立自制渲染器的授权，须证明指向同一组件体系的双端后端。
统一内容、final/streaming、theme/accent、readonly、change/save、scroll/selection 的消费合同；
未闭合 fence/table/callout、分片中文、完成切换/取消、edit→序列化→view 必须有用例。
引擎内样式差异由 auto-down 修（必要时 auto-lang 同组修原生后端），Musk 只保留宿主尺寸/主题传递。
引擎旧 CSS 覆盖逐条归属；上游等价实现验收之后才能移除，不能先删除导致回归。
view/stream/edit 均在本路线范围内，不能以 view 通过冒充引擎整体通过。

## 8. 测量预算与完成定义

第一阶段冻结环境：字体文件/hash、主题/token、locale、OS、GPU/渲染器、窗口内容区域、
浏览器 DPR/缩放、VM scale；参考 1280×800，另测 960×720/1600×1000 和 100%/150% DPI。
Windows 缩放换算必须用内容区域逻辑像素，不能把标题栏计入；加载完字体后截图。
同数据/同状态/同动画时点取帧；时间固定而非整片遮罩。动画另验时序/状态。

近期“大体一致”建议预算（新提案，P074 工具校准不能自行放宽）：
- 功能：关键动作、顺序、内容、焦点/输入/审批状态零缺失；无占位冒充完整实现。
- 几何：关键容器边界/对齐点偏差 ≤2 logical px，长流累计高度偏差 ≤4px；
  无裁切、遮挡、不可达按钮；指定换行 fixture 的行数相同。
- 颜色：token 值一致；平坦区域各 RGB 通道差 ≤2（以同 alpha 合成背景采样）。
- 字体：字体源、字号、字重、行高一致；抗锯齿像素差单独报告，不能掩盖换行/高度差。
- 总像素差/SSIM 为诊断指标，字体边缘白名单必须局部、有 owner/原因/期限，禁止整控件 mask。
- 第一轮基线仅是现状，不能自动成为期望；每项需要规约+双端证据+人工局部放大审查。
最终像素级：锁定参考平台后逐项缩减预算至非文字区域零差、文字边缘有明确可复验预算；
不承诺跨浏览器/GPU 的逐字节 PNG 恒等，也不得把近期完成声称为最终像素目标完成。

DoD：有限单元表全部归类；适用场景全部跑过，缺件/待验/豁免不算 PASS；
两端截图、属性/布局快照、交互断言、版本与命令完整；更新基线需解释刻意变化；
聚合报告非零退出码拦截缺失与回归，任何延期需明确裁定，不能静默删场景。

## 9. 六阶段计划与依赖

| 阶段 | Plan | 结果 | 前置 |
|---|---|---|---|
| S1 | 074 | 全量可达性清单、真实组件 gallery、确定性双端 gate | 无 |
| S2 | 075 | Design 22/宿主默认样式对账并修复、主题/字体 | 074 |
| S3 | 076 | autodown-engine view/stream/edit 接入与上游差异关闭 | 074；主题验收依赖075 |
| S4 | 077 | thinking/tool/审批等消息 L1 与 ChatMessage L2 全绿 | 075、076 |
| S5 | 078 | 输入/壳/文件/Wiki/Specs/Plans 组件与组合全绿 | 075、076；组合回归077 |
| S6 | 079 | App 流程、后端双模式、回归门与最终收据 | 077、078 |

075/076 可以分工作域推进，但同一上游模块冲突必须串行；编号不隐含自动执行授权。
后续任务若发现整套渲染机制重构，先登记受影响 AC、修订计划，不在组件修复中无限扩范围。
每阶段独立 review；未完成的依赖不能标绿。无必要不创建新 Blueprint 包；复用≥2处再评估。

## 10. App 与后端矩阵

视觉比较先使用同一固定 fixture HTTP 服务，隔离业务后端差异；App gate 再覆盖：
Vue+VM HTTP、VM+VM HTTP、VM merged。`pac.at` 当前 api:rust，不能仅凭 -r vm
就声称已测 merged。用 CLI 显式 server/merge 配置并记录真实启动日志、进程与请求路径。
当前产品默认 Rust HTTP 作为兼容回归面保留，不擅自修改 pac 后端默认。
若 merged 路由/VM SSE 有缺口，要记录为明确功能问题并修复或提交范围决策，不换模式掩盖。
App 场景：登录/恢复→工作区→新会话→用户输入→thinking/text/tools→审批/拒绝→完成/停止/
错误→刷新历史/分叉；另覆盖导航、主题、文件树与文档 view/edit/save/reopen。
使用临时工作区与可控事件服务，不依赖真实 LLM 完成确定性验收；真实服务仅补充 smoke。

## 11. 执行纪律与风险

计划草稿/进度在主检出 docs/plans；代码和设计修改在专用 worktree。
组 `.wt/musk-NNN/{auto-musk,auto-lang,auto-down}`；分支按本仓 AGENTS，禁止任何
junction/symlink（含包管理器自动生成链接）；依赖使用 env→兄弟目录→主检出。
先验证无链接的依赖安装方案；不直接复用 Jade 的 link: 绝对路径配置。
合回前提交并确保 clean，wt-guard clean 后清理。不得覆盖他人脏改动。
纯文档阶段不跑编译测试；实现阶段按实际改动范围验证，禁止反复无差别全量测试。
风险重点：旧状态共享回归、gallery 与产品双源、样式覆盖顺序、旧生成资产、引擎 stamp 不一致、
VM 数据/SSE能力与视觉混淆、字体网络加载不稳定、大量截图无版本基线。

## 12. 调查版本

- auto-musk: `04eb90531643223076a5d9d2a9572ccaaf6d1c14`
- auto-lang: `278efbea7e95fe05f45f86f7871a5c72043d4f3c`
- auto-down: `7c0b774e17f079eaa2462b0d9028781d75458f50`

| 调查源 | SHA-256 |
|---|---|
| `docs/specs/00-overview.md` | `4a72537724de30488766cdb49e3f590a952887848c8967670c981d05f080fc31` |
| `docs/specs/03-front-component-groups.md` | `58f6390b13ae94123b9c8c566f21518d7f4dc63c26f5114eee6654ca374ee26d` |
| `docs/specs/modules/chat-streaming.md` | `51c1648fd9f468c8de126b506ece1a979d7f865a5faaefb5c921f90ee50d0319` |
| `src/front/chat_message.at` | `43775f35cce022209e78abaa24050696f86d4cedad0cb7b46c3c74c032dee8b7` |
| `src/front/ports/renderer.vm.at` | `9a9ee8dca3047c80b8e919099f37ef407531302421e0cb2fc4c55d51b6e57f41` |
| `src/front/inject_styles.web-only.ts` | `7bb93bfd0113b8ad1d0440925e99848980815cb172c0239ca4b95221083510aa` |

## 附录 A：当前 .at widget 声明初始清单

本表由当前 src/front 词法扫描生成，包含遗留/端口/组合，未宣称全部可达或双端可用。
ThinkBlock 逻辑单元另按 §4 内联分支纳入。P074 需补调用可达性和三桶分类。

| 源文件 | 声明 |
|---|---|
| `src/front/agent_avatar.at` | AgentAvatar |
| `src/front/app.at` | App |
| `src/front/chat_message.at` | ChatMessage |
| `src/front/chats_view.at` | ChatsView |
| `src/front/content_header.at` | ContentHeader |
| `src/front/errand_card.at` | ErrandCard |
| `src/front/files_view.at` | FilesView |
| `src/front/filetree.at` | FileTree |
| `src/front/gate_card.at` | GateCard |
| `src/front/generic_tool_card.at` | GenericToolCard |
| `src/front/lib/icon.at` | Icon |
| `src/front/login.at` | LoginPage |
| `src/front/mention_dropdown.at` | MentionDropdown |
| `src/front/mention_input.at` | MentionInput |
| `src/front/nav_item.at` | NavListItem |
| `src/front/nav_sidebar.at` | NavSidebar |
| `src/front/plans_view.at` | PlansView |
| `src/front/ports/delete_confirm.vm.at` | DeleteConfirmDialog |
| `src/front/ports/renderer.vm.at` | Markdown |
| `src/front/questionnaire_card.at` | QuestionnaireCard |
| `src/front/raw_preview.at` | RawPreview |
| `src/front/relay_run_box.at` | RelayRunBox |
| `src/front/report_card.at` | ReportCard |
| `src/front/secretary_message.at` | SecretaryMessage |
| `src/front/secretary_message_wrapper.at` | SecretaryMessageWrapper |
| `src/front/session_info.at` | SessionInfo |
| `src/front/settings_menu.at` | SettingsMenu |
| `src/front/specs_category.at` | ArchitectureCards, DesignCards, ReportCards, ReviewCards, TestsCards, GoalsTable |
| `src/front/specs_detail.at` | StatusTransition, RelationsPanel, GoalDetail, ReviewDetail, TestDetail, ReportDetail, SpecItemDetail, GoalDetailModal |
| `src/front/specs_editors.at` | TagInput, AutoDownEditor, TestEditor, GoalEditor, MarkdownEditor |
| `src/front/specs_leaf.at` | StatusBadge, SpecLink, SpecItemRow, CategoryList |
| `src/front/specs_tree.at` | TreeView |
| `src/front/specs_view.at` | SpecsView |
| `src/front/streaming_table.at` | StreamingTable |
| `src/front/task_plan_card.at` | TaskPlanCard |
| `src/front/tool_block.at` | ToolBlock |
| `src/front/tool_gate_card.at` | ToolGateCard |
| `src/front/tree_icon.at` | TreeIcon |
| `src/front/user_message.at` | UserMessage |
| `src/front/whitelist_view.at` | WhitelistView |
| `src/front/wiki_nav.at` | WikiNav |
| `src/front/wiki_view.at` | WikiView |
| `src/front/workspace_selector.at` | WorkspaceSelector |

扫描合计：62 个 widget 声明；不是 62 个已验收单元。

## 计划入口

- [PLAN-074](../plans/074-musk-widgets-gallery.md)
- [PLAN-075](../plans/075-default-style-contract.md)
- [PLAN-076](../plans/076-autodown-engine-parity.md)
- [PLAN-077](../plans/077-message-block-parity.md)
- [PLAN-078](../plans/078-shell-composition-parity.md)
- [PLAN-079](../plans/079-app-parity-release-gates.md)
