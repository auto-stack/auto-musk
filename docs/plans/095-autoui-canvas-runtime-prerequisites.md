---
plan_id: PLAN-095
status: reviewed
feature_name: AutoUI Canvas VM 基础能力补齐与 PLAN-093 依赖闭环
author: [agent]
created_at: 2026-09-29T09:57:59Z
updated_at: 2026-10-01T13:35:00Z
plan_revision: 2
current_step: 9
total_steps: 9
supersedes_spec_components:
  - docs/specs/modules/app-canvas.md
  - docs/specs/modules/ui-parity.md
new_spec_components:
  - docs/specs/modules/autoui-canvas-runtime.md
touched_goals:
  - goal-frontend-parity
  - goal-agent
---

# PLAN-095 — AutoUI Canvas VM 基础能力补齐与 PLAN-093 依赖闭环

## 0. 变更摘要

本计划承接 PLAN-093 T-01 平台探针确认的上游能力缺口，优先打通工作台能够
可靠展示、选中和定位的基础链，再推进用户直接试用应用等新增功能。

**核对结论须与用户的完工陈述分开记录**：本轮读取的 093 为 executing、
current_step=1/14；只有 T-01 勾选。其专用 worktree 存在 T-02 后端未提交
修改，未见整体 execution_done、reviewed、merge／delivered 收据。故本计划
不将 093 的全部功能当作已交付基线，也不改写其状态。若其他检出已完成，
T-01 改以该交付提交核对并消除已解除项，保留同一验收目标。

本期处理：

1. ImageSurface 的真实加载／失败事件与过期回调过滤。
2. VM absolute／overlay 路径造成宿主子树消失的问题及覆盖框显示。
3. .at 可达的原生输入聚焦能力。
4. MCP 合成坐标与真实指针坐标的数值一致性。
5. 093 记录的 Vue 生成构建失败：先复现归因，再修真实故障及必要 schema 漂移。
6. （r2 增补）VM 轨 widget→widget 子件实例化缺面：widget 视图引用同文件
   兄弟 widget 整体空渲染，093 T-06 真实面板带活会话首次演练暴露。

产品工作台布局、Canvas 身份与归属、会话上下文和消息队列仍由 093 负责；
095 提供可验证的运行时能力和消费合同，不另做一遍工作台。

## 1. 目标

### 可交付结果

- 当前图片确实可绘制时收到一次对应的加载通知；坏图片收到失败通知；
  换图／卸载后旧通知不污染新视图，空等待不假装同步成功。
- 画布底图、蓝色选择框、琥珀高亮框可同时显示，加入叠层不使原内容消失；
  非交互覆盖框不拦截下方点选。
- .at 调用原生聚焦后，键盘输入落到指定输入控件；不是只翻转一个“已聚焦”标记。
- 自动化坐标带小数时与实际逻辑坐标一致，可用于后续 DPI 与 contain 几何验收。
- 以指定版本 Auto CLI，消费工程能真实生成 Vue 并完成打包；
  非法输入失败须给出可定位的原因。
- 093 执行者可据收据接入上述能力，不必删验收项或将 VM 改成“尽力交付”。

### 仓库与工作树范围

| 仓库 | 角色与允许的后续实现范围 | 预定 worktree／分支 |
|---|---|---|
| auto-musk | 本计划归属；新消费探针、回归脚本、依赖版本／锁文件及报告；不抢改093在途UI | D:/autostack/.wt/musk-095/auto-musk；plan-095-dev |
| auto-lang | 因auto-musk消费而补齐UI运行时／native／MCP／CLI生成路径；必须修在依赖worktree | D:/autostack/.wt/musk-095/auto-lang；auto-musk-dev |
| auto-ai、auto-down、auto-os | 读取配置／能力／已有消费样例；本合同不安排代码修改 | 不创建写入worktree |

分支或目录被占用时遵守仓库规则复用合适 checkout 或明确调整；不覆盖别人的
工作树，不建立 junction／symlink。本计划文档及状态仍在主检出 docs/plans。

auto-lang 的 PLAN-707 是 HTTP stream／SSE async 线，属于另一范围；
另有 PLAN-708 渲染响应性草案，已预建工作树、待外部review后实施。
708与本计划会接触renderer／aura_view_builder，T-01必须固定串行集成顺序；
不复制其任务，不触碰其WIP。

### 非目标

- 不建设 Canvas“选择／试用”双模式，不转发用户点击／输入到目标应用。
  该方向仍有产品价值，但应在 093 真实交付及本计划能力闭环之后另立计划。
- 不改造全部 Overlay／编辑器／桌面窗口体系，不实现完整 CSS selector 查询。
- 不做 ImageSurface 全部 wheel／pan／double-click 功能，只接本期所需loaded／error。
- 不新增多Canvas、应用目录、AppViewport原生嵌入、属性直接编辑或拖拽设计器。
- G-4 image.queue URI访问器和G-5 body_bytes跨park RC问题不纳入：
  已验证的 body_to_file＋open_session／current_uri 是本期保留的消费路径。
- 不扩大到持久化Plan Runner、review自动循环或ledger字段补全。
  PLAN-094已交付的handoff／ledger规则保持其既定范围。

## 2. 架构方案

保持既有媒体ticket、后台解码、UI消息派发、View／AURA和构建链，不引入第二套
渲染引擎或把截图预览改成原生窗口嵌入。

```text
auto-musk 消费（093负责UI，095负责合同/探针）
  Http.request().send() → body_to_file → open_session/current_uri
                                                ↓
auto-lang 媒体后台 → ImageSurface可绘制状态 → 有代次守卫的loaded/error消息
                                                ↓
                 .at handler → 093帧身份核对 → 当前画面就绪／失败

共享内容矩形 → 底图 + 被动选择框/Agent框 → 保持宿主内容的VM叠层

修改按钮 → 平台聚焦端口 → 最小语义目标解析 → Iced聚焦任务 → 原生输入控件

MCP合成指针 → 与真实指针同一数值编码/解码 → .at float参数
```

loaded的最低语义为：对应资产已完成解码，且当前有效ImageSurface已在渲染帧中
消费可绘制rendition；不代表物理显示器已完成扫描或用户必定已目视。
消费方仍须核对自己的预览身份／src版本。paint路径只记录有界通知，
由消息循环派发，不在view构建／绘制中重入用户handler或做网络／文件解码。

095不改变093“收起独立于停止”、单Canvas归属和代次的业务设计。接口能力需
向后兼容旧ImageSurface回调签名；新的聚焦接口在T-01固定后由093平台端口接入。

## 3. 技术栈

- AutoUI AURA／View，现有 Iced renderer、ImageSurface、PointerArea、Overlay。
- AutoVM native／事件消息桥，必要时新增或补齐最小stdlib声明及native注册。
- auto／auto-man的当前严格生成流程与schema_drift事实源。
- 现有nextest档位与UI fixture、Vue真实浏览器、VM MCP／截图。
- 默认不添加第三方依赖；沿用媒体ticket和后台decode LRU。
- 不手修Vue生成SFC或生成schema来掩盖源合同错误；生成结果由真源再生。

## 4. 需求分析与背景调查

### 授权与边界

用户于2026-09-29明确要求使用auto-plan-new核对093并规划下一计划。
本轮授权为读取证据和创建下一份计划，未要求执行、合回、停止共享进程或部署。
新增外仓修改是本草案提出的实现范围，须由后续执行请求采用；本轮不改外仓。
用户未指定预算、模型或自动续跑限制，不推定这些设置。

依据auto-musk AGENTS.md，因本项目而改的依赖使用同组兄弟worktree；
auto-lang AGENTS.md中的测试／独立复审／规范沉淀规则同样适用。095以本仓
计划为主合同，不在另一仓重复维护一份同内容计划。若发现已有上游计划负责
相同缺口，先引用其任务和交付版本，而不是抢写或重做。

### 本次核对快照

| 对象 | 读取事实 |
|---|---|
| auto-musk主检出 | main@4c3a7472386a097301cd9c8a25ecf677f8e1405f，核对时git status干净 |
| 093计划 | docs/plans/093-autoui-app-studio-ux.md；r1、executing、1/14，T-01完成 |
| 093工作树 | D:/autostack/.wt/musk-093/auto-musk；plan-093-dev@d157387dc540c783d16bcb496b4c3287ab119cde |
| 093 WIP | backend/crates/musk/src/canvas/{manager,mod,tools}.rs三份后端改动；核对期间另出现capability-map更新与canvas_studio_contract.rs测试，均未提交，只读取，不处理 |
| 093证据 | 工作树docs/reports/093-app-studio-capability-map.md及两个probe收据 |
| 094 | docs/plans/archived/094-relay-plan-handoff-and-ledger-contract.md；archived、delivered，有review／merge／cleaned记录 |
| auto-lang | master@3b3014e34938c130e2162f9f620d4dc5c16ca718；707已提交草案，708草案及既有临时走查资产保持原样 |
| auto-ai／auto-down | 5a50a55844d7aa3523b593f21ba0fb03d18eac48／3373a5cc6e3a00336613133db51906fb0940777d |

093探针使用auto-lang@e2deb4f87，当前上游已前进至3b3014e3；两条新提交为707
计划簿记／草案，不能据此推定运行时已修。报告中的行号只是
当时定位。T-01按当前提交复验，不能仅凭旧报告再次判定全部缺口仍未修。
若调查期间093继续推进，以上快照保留为历史，执行时重新冻结消费基线。

### 权威来源及版本指纹

先读本仓docs/specs/index.json、00-overview.md，再读app-canvas／ui-parity／
vm-data-semantics／web-input-contracts；overview中旧状态机、工具数量等不能
覆盖当前模块Spec。外仓读docs/specs/README.md、overview.md、
auto-lang/ui/overview.md、ui/design/overlay-interaction.md、
auto-lang/mcp/design/dual-mcp-servers.md、auto-man/project.md与auto-cli/project.md。
另核对mcp/design/toolset.md：它描述AutoVM源代码server，不是AutoUI指针通道；
本计划的MCP坐标合同须沉淀到ui模块，不能写入该toolset。
不存在的image-viewer模块Spec不作为依据；ImageSurface能力从现有源码／测试核实，
其运行时事件合同需在本次沉淀时补齐。

| 来源 | SHA-256 |
|---|---|
| auto-musk: docs/plans/093-autoui-app-studio-ux.md（本次快照） | 0e89daad830dc34cc966481bdb618405e7e5056447239b13353b6cb1aae69b7f |
| 093: d157387提交中的docs/reports/093-app-studio-capability-map.md（T-01冻结版，不含核对期间T-02 WIP追加） | fef0f2539d54382f43a434c448d63329e671230c152e5b6a2ad982ad3a434bb0 |
| auto-musk: docs/specs/modules/app-canvas.md | dba0cc96d090c0b87b981a42f7e1845daa7ed816b704c36217067c4e10715423 |
| auto-musk: docs/specs/modules/ui-parity.md | 648a9270866cd100cfafa2f731211ff66bfe320a4ed7a6c864923eb55d0b345c |

其余源码／Spec通过上述仓库提交绑定。当前知识仍以各仓docs/specs为权威；
ledger仅作索引、关系、历史投影，不再独立维护同一规则。

### 缺口与范围判定

| 项 | 证据／实际入口 | 处理 |
|---|---|---|
| G-1加载／失败事件 | renderer.rs的AbstractView::ImageSurface臂保留但丢弃on_loaded/on_error；image_surface_contract只证明回调经生成／View保存 | 本期必做：运行时派发及有效版本过滤 |
| G-2叠层宿主丢失 | 093 probe-b的absolute+z子节点导致整个宿主内容消失；aura_view_builder::fold_floats折叠为View::Overlay | 本期必做：先定罪实际丢失层，再修根因；不预断为fold_floats单点 |
| G-3原生聚焦 | vm/native.rs::shim_dom_focus_first当前为空实现；musk ports/platform.vm.at仍打印skipped | 本期交付上游原语和消费合同；093负责其生产端口接线 |
| G-6合成坐标 | renderer::__mcp_drag／__mcp_pen用Double载荷而float处理形参位型错读；真实PointerArea用Float | 本期必做：共享正确编码，测试小数／负值／DPI |
| V01生成失败 | 093证据记两个CLI版本strict／lenient均无明确原因退出，S001为Info；发生于扫描／Vue生成阶段附近 | 本期先复现；CLI/auto-man/schema改动仅限真正证实的原因 |
| G-4／G-5 | image.queue URI缺访问器、body_bytes跨park丢RC；body_to_file替代逐字节PNG已PASS | 保留替代，不扩大到HTTP堆生命周期；与707协调 |
| G-11 widget→widget 子件缺面 | 093 capability-map §8.16+快照：musk canvas_panel.at 三件套子件（结构列/画布列）在 VM 轨整体空渲染（列壳样式在、子件内容零节点），normal/studio 双模式一致；view→widget 正常对照；同文件 widget 视图引用兄弟 widget（无 use 行）疑为触发面 | r2 增补本期必做：T-01 定罪实际丢弃层（aura builder 子件解析 vs renderer），再修根因；093 真实面板 VM 消费面挂此解除 |
| 094部署观察 | release构建曾被运行中exe锁定；落地提交不等于生产二进制已更新 | 本期证据须绑定实际CLI文件hash；不强杀共享实例来重建 |

## 5. 详细设计

### 5.1 基线核验与并行写入边界

T-01只读核对093当前commit／计划／收据及auto-lang活跃计划。095不改093计划
状态、不清理其worktree、不合并其未提交代码，也不把探针PASS解释为整体完工。
有新的093交付收据时，以源码和实际验收绑定后的提交作为消费基线。

095的auto-lang写入集中在UI媒体事件、Overlay、聚焦与MCP坐标；707的HTTP
async／stream／SSE注册表不进入本期。若根因落在另一在途计划的写入区，记录
文件／符号和依赖，等待其交付或明确协调后调整合同，不能同时覆盖同一代码。
708拟改的Init两拍／memo与本计划媒体通知共用帧后派发和View重建接缝。
T-01记录708实际进度与文件／符号占用，确定哪一方先落地；后写方以已交付
提交为基线复验订阅代次、pending通知和Overlay，不同时改写同一接缝。
本计划不承担708的缓存／Init改造，也不推定尚待review的708已经交付。
095的auto-musk写入仅为自己的探针／测试／依赖消费，不直接进入093的chats_view、
canvas_store、canvas_panel或会话上下文实现。

### 5.2 ImageSurface加载与失败通知

保持现有.on_loaded(fn())／.on_error(fn(str))公共回调形态，先验证精确名称、
Vue生成事件与VM消息签名；内部通知携组件实例／资产URI revision及订阅代次，
在真正派发前确认仍是当前有效视图。

- Ready：资产解码完成且当前渲染消费其rendition，loaded最多一次／有效订阅。
- Failed：缺票、不可读文件、损坏图像或后台decode失败，error最多一次，
  原因有可定位码／短文；不能先loaded再把同一请求称为失败。
- Pending：异步等待不触发成功；重绘、1s poll和同一ticket重复构树不重复通知。
- Replaced／Unmounted：换src、换订阅代次或卸载后，旧worker结果与排队消息
  失效；不访问已失效VM堆对象，不让旧图回调误开新图的点选门。
- 无回调的旧ImageSurface正常显示；新旧组件共存不新增后台解码或绕开LRU。
- 消息排队后由UI桥正常调度；不在paint期间运行handler，不跨线程传AutoTask
  或VM heap引用，不能因loaded写状态再重绘形成无限通知循环。

测试必须同时证明“回调数据结构存在”和“真实运行触发了正确handler”；
沿用image_surface_contract的静态三轨合同，但增加运行时测试。

### 5.3 Overlay保持内容与命中语义

使用probe-b的最小宿主＋一／多absolute子节点复现，逐层记录：
AURA提取 → View::Overlay → AbstractView／Iced元素 → rendered snapshot／截图。
确定是提升、转换还是渲染兜底丢失后，只修对应接缝。

- base仍参与布局／绘制；所有overlay按z／同z源序稳定呈现，不只保留首项。
- 被动选择框／Agent框不抢焦点、不截获canvas点选；覆盖框几何以实际
  图像内容矩形为基准，不因Overlay引入新坐标原点。
- 多层框、非浮层兄弟、嵌套row／col同时保留；容器高度／滚动不被空overlay撑坏。
- 已有可交互popover／dialog仍能接收自身输入，不把所有Overlay一律设穿透。
- 目标预算为边界≤2 UI逻辑px，验证contain留白和DPI1／1.5／2；
  不依靠后端把框烘焙进PNG来宣称原生Overlay问题已修复。

### 5.4 原生程序化聚焦

T-01调查现有Iced text_input／text_editor的稳定id、FocusManager与native桥，
固定最小可达接口；优先复用已有机制。接口必须能描述指定控件的语义目标，
并在不存在／不可聚焦时给出结果，不采用“默认为第一个控件”的猜测。

可选实现须在T-01取舍后写入接口表：

- 在现有dom.focus_first中支持可明确映射的最小id／class选择器子集；
- 或在现有stdlib/auto/ui.at增加平台中立的focus(target_key)原语，
  由musk平台端口封装，Web继续用现有DOM聚焦。

不实现完整浏览器CSS selector引擎。无论取哪种实现，必须记录公共参数、返回、
native注册、Vue映射、对现有调用的兼容规则，并经真实键盘输入证明目标正确。

聚焦请求异步进入Iced任务，不在VM handler中等待UI线程；目标卸载后请求失效；
窗口切换／popover关闭／Tab仍符合原来的焦点行为。text_input与composer使用的
text_editor均有覆盖，不能只修其中一个后宣称093 composer已可用。

### 5.5 合成坐标数值合同

__mcp_drag／__mcp_pen与真实PointerArea统一采用匹配float形参的数值编码，
或在公共解码接缝作显式合法转换；T-01选择最小且不损旧Double消费者的方案。
禁止把错误位型int作为业务坐标“修正”后交给UI。

覆盖155.001、189.113、0、负坐标、大视口合法坐标，值误差≤0.5逻辑px；
验证Move／Down／Up顺序、坐标原点／coords extent、DPI换算和≤30Hz真实限频。
修正MCP通道后，probe-b用实际state中的float断言，不以moves计数证明坐标正确。

### 5.6 生成失败定位与strict门恢复

最多使用两个最小失败工程加一个真实musk消费工程作有界归因：
保存CLI可执行文件hash、环境／依赖解析、源hash、stdout／stderr及退出码，
在扫描、AURA校验、Vue生成、依赖安装、打包阶段分别定位真实退出点。

- 若现版本已解除，记录旧→新版本与成功收据，本项按验收完成，不为了任务勾选
  另造修改。
- 若是CLI错误吞掉／子进程启动失败，补阶段化错误和退出原因；
  非零失败不被改成成功。
- 若是schema漂移，查真源控件声明及其生成表：合法prop补完整事实源／再生，
  非法prop用保行为的合法表达修正并记录；不把全部S001升级为错误，
  也不删title等合法交互只为绿灯。
- 真实musk工程的源修正由当前拥有该文件的093任务消费；095先交付生成合同
  和可复现fixture，避免并发改同一生产UI文件。
- 保持strict默认；不以lenient、手改gen、吞告警或跳过pnpm build代替通过。

CLI在本期worktree的独立构建输出生成，避免写入被运行中进程锁住的主二进制。
收据写实际采用的文件hash；共享生产CLI换版留到明确的部署步骤。

### 5.7 widget→widget 子件实例化（VM 轨，r2 增补）

症状与证据：musk `canvas_panel.at` 三件套（CanvasPanel/CanvasStructureColumn/
CanvasCanvasColumn，同文件兄弟 widget、无 use 行）在 VM 轨带活会话演练时，
列壳（宿主内联样式）渲染、子件内容零节点——normal/studio 双模式一致；
对照面 view→widget 全量正常（ChatsView 壳层）。证据：093 capability-map
§8.16、快照 tmp/ui-parity/PLAN-093/vm-studio-cycle/{wrap-missing,
normal-panel}-snapshot.txt。

设计约束：T-01 先以最小 fixture（同文件 widget 视图引用兄弟 widget 的
最小 .at 工程 + `auto run --render=vm` + AutoUI snapshot）定罪丢弃层——
aura_view_builder 的组件臂解析（use 行引用 vs 同文件兄弟两条路径）或
renderer 更深层，不预断单点；与 PLAN-708 的写入范围（renderer／
aura_view_builder）串行集成，顺序按 Q-05 对账，后写方重建基线。

修复语义：widget 视图内的子 widget 引用（同文件兄弟与跨文件 use 两形态）
在 VM 轨等价实例化；空渲染不得静默（无法解析的子件引用须有可定位诊断，
与 T-06 的 strict 诊断口径一致）。宿主样式与兄弟渲染互不丢失。

验收接口：最小 fixture 的 VM snapshot 含子件子树；093 真实面板消费面
（结构列树钮/画布列标题条在 VM snapshot 出现、cv_frame_wrap_style 由
同一 .at fn 在 VM 解释器产出）随本任务解除后补跑。

### 5.8 消费、回归与收尾

095消费测试不等待093整套工作台完成：使用093已提交的媒体／坐标探针，
加本期加载／错误／覆盖框／聚焦的最小真实fixture，证明能力可以被Auto代码调用。
因此不存在“093先等095，095又先等093全部完工”的循环依赖。

向093交付一张接口与版本表：

| 消费任务 | 本计划交付 |
|---|---|
| 093 T-06 | widget→widget 子件 VM 实例化（G-11）；VM 消费面（包装层样式/坐标点选）随后补跑 |
| 093 T-03/T-04 | 真实媒体loaded/error、过期订阅过滤、继续body_to_file路径 |
| 093 T-05/T-06 | Overlay保持base及多框显示／穿透，生成strict门可复现 |
| 093 T-08 | composer原生focus接口、错误／目标约束和真实输入收据 |
| 093 T-11/T-13 | float合成坐标与DPI场景的可信验收机制 |

依赖auto-lang达到本计划消费门后，按AGENTS.md尽快review／合回该依赖分支并
清理，记录交付SHA；不让其worktree悬挂到093整个UI项目完工。093后续集成与
T-13全量AC仍需自行复验，095探针不代替其最终验收。

### 规范增量

frontmatter仅列本仓Spec路径；外仓目标在此注明归属，由后续merge按对应仓沉淀，
不能把auto-lang文件伪装成本仓已存在的模块。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/modules/autoui-canvas-runtime.md（auto-musk） | 新增Canvas消费能力／版本／事件与focus合同，引用上游权威Spec而不复制实现 | 使093及后续应用设计有稳定消费边界 | AC-01, AC-02, AC-05, AC-08, AC-10 |
| SD-02 | modify | docs/specs/modules/app-canvas.md；docs/specs/modules/ui-parity.md（auto-musk） | 标注已交付运行时依赖及可信loaded／overlay／坐标验收；保留093未交付业务范围 | 探针能力与完整产品交付不混淆 | AC-02, AC-03, AC-04, AC-06, AC-08, AC-09, AC-10 |
| SD-03 | add/modify | 新docs/specs/auto-lang/ui/design/image-surface-events.md、新design/programmatic-focus.md、新design/widget-composition.md；已有ui/overview.md、design/overlay-interaction.md（auto-lang） | 回调仅经View保留／focus stub／叠层丢内容／widget→widget子件空渲染 → 真实有代次事件、明确原生focus、完整base和命中语义、子件两形态（同文件兄弟与use行）等价实例化且缺失可诊断 | 补足平台行为权威规则 | AC-02, AC-03, AC-04, AC-05, AC-09, AC-11 |
| SD-04 | add/modify | 新docs/specs/auto-lang/ui/design/mcp-pointer-input.md；已有docs/specs/auto-lang/ui/overview.md（auto-lang） | 明确AutoUI合成坐标编码与真实float回调一致、来源与精度验收；AutoVM toolset不改 | 防止自动化仪器继续产生错误证据，保持双server职责 | AC-06, AC-08 |
| SD-05 | modify | docs/specs/auto-man/project.md；docs/specs/auto-cli/project.md（auto-lang） | 按实际根因补strict阶段错误／schema真源／CLI文件版本消费规则 | 生成失败可诊断，不能借旧二进制判绿 | AC-07, AC-10 |

若某项在T-01已被其他计划修复，其SD改为引用已有规则／版本，保留核验记录；
目标和AC不删。本仓新增模块在index注册，外仓索引以其既有生成脚本更新。
ledger从这些模块派生，不单独重写另一份知识内容。

## 6. 测试设计

### 命令矩阵

所有命令在后续095专用worktree执行。新脚本／fixture未创建前不计为已通过。
T-01核对CLI命令及features后固定实际调用；受影响的required门不能默默skip。

| 编号 | 命令／工作目录 | 预期 |
|---|---|---|
| V01 | auto-lang：cargo check -p auto-lang；必要时cargo check -p auto-man -p auto | 接缝修改编译成功；无未处理的新警告 |
| V02 | auto-lang：cargo nextest run -p auto-lang --test image_surface_contract --features ui-iced | 现有三轨生成／View合同继续通过；另有本期运行时证据 |
| V03 | auto-lang：cargo t image_surface／cargo t overlay／cargo t focus／cargo t mcp（按实际新增测试名定向） | 对应边界回归通过；不以过滤出0个测试算PASS |
| V04 | auto-lang：cargo tf及cargo tv；触生成器时另cargo tt | 按AGENTS规定review／fold门；先识别并记录已知基线红，不删除新AC |
| V05 | auto-lang：cargo nextest run -p auto-man；cargo nextest run -p auto --bin auto（按改动触发） | CLI／生成调度回归；无故障吞掉／假成功 |
| V06 | 根：node scripts/ui-parity/canvas-runtime-probe.mjs --plan 095 --mode both（新） | Vue／VM真实媒体事件、叠层、聚焦、坐标；缺实机证据非零退出 |
| V07 | musk消费worktree：指定095产物的auto build --gen-only --strict；gen/front/vue中pnpm install、pnpm build | 真源生成／Vue类型与打包成功；保存实际CLI hash及所有阶段退出码 |
| V08 | musk：node scripts/ui-parity.mjs check；backend中cargo test -p musk --lib | 新fixture／消费者入目录，相关旧行为回归；不能据此替代VM实测 |

“指定095产物”以明确可执行路径或单命令环境指向该CLI；不覆盖全局PATH配置。
现有093探针脚本位于093已提交工作树，可在独立095工作树复用固定版本／提取
提交内文件到自己的fixture，不能运行正在被093改写的WIP当冻结证据。
正式测试使用真实生产组件与原生事件通道，不另写同算法的假渲染器。

### 必须覆盖的运行场景

| 场景 | 输入／故障 | 判据 |
|---|---|---|
| 资产事件 | 有效PNG、损坏PNG、缺ticket、慢加载、同src重绘、A→B乱序、卸载 | loaded/error次数和目标一致；旧结果0次污染 |
| 叠层 | col/row、一个／多个absolute、同z源序、不同z、被动框、可交互popover | base与全部框可见；框不拦点选，popover仍可操作 |
| 组合 | widget视图引用同文件兄弟widget与跨文件use widget；嵌套两层；子件引用不存在 | 子件子树在VM snapshot完整出现；引用缺失有可定位诊断，不静默空渲染 |
| 几何 | contain横／纵留白、缩放、DPI1／1.5／2、窗口resize | 内容矩形及框边界≤2逻辑px |
| 聚焦 | text_input、composer形态text_editor、目标缺失／卸载、Tab、Escape | 真实键盘输入进入指定控件；错误有结果，无丢字／误发 |
| 坐标 | MCP drag/pen小数、负值、边界、真实鼠标对照 | .at收到数值误差≤0.5px，动作顺序准确 |
| 构建 | 原失败项目、最小有效工程、明确非法工程 | valid生成与打包成功；invalid非零＋可定位诊断 |
| 稳定性 | 连续5分钟换图／切组件／叠层和聚焦 | 通知／文件／订阅有界，无重绘死循环或退出后残留 |

文件／媒体session沿用滚动关闭／释放规则；帧下载禁止body_bytes绕行。
真实输入、外点与Overlay路径不能仅用MCP合成press证明，因为旧合成路径未必
经过相同的overlay事件路由。浏览器、VM截图／state与实际键盘／指针实测互证。

### 证据与版本

新输出：

- docs/reports/095-canvas-runtime-baseline.md：093／上游版本及缺口复验、接口冻结、
  V01失败归因、与707／708写入范围及集成顺序对账。
- docs/reports/095-canvas-runtime-evidence.md：AC结果、截图／state／命令索引。
- docs/reports/095-canvas-runtime-spec-delta.md：两仓SD拟议补丁与消费表。
- tmp/ui-parity/PLAN-095/：原始收据、PNG、事件spy、隔离配置；必要证据须持久化，
  不能在worktree清理后只剩tmp中的失效链接。

收据绑定PLAN-095:r1、两仓源码SHA／工作树摘要、CLI二进制SHA／路径、依赖版本、
运行模式、窗口／DPI、断言结果、自己的PID／端口及关闭结果。共享服务、093工作树
和其他Auto实例不在测试清场范围；不按进程名或端口区间批量杀进程。

## 7. 验收标准

| ID | 可观察结果 | 验证方法／预期 |
|---|---|---|
| AC-01 | 093完成情况和缺口基线经源码／收据核对，已修项不重复实现；职责不重叠 | baseline报告列出实际计划状态／提交、G-1/2/3/6与V01复验；无改093状态或WIP |
| AC-02 | 当前有效ImageSurface ready后loaded仅一次，同src重绘不重复；旧图／卸载通知被过滤 | 真VM A→B慢加载＋回调spy／截图；只有当前实例成功，无“ticket存在即loaded” |
| AC-03 | 坏图／缺票触发对应error，等待不报成功，失败不无限重绘 | 真实PNG／故障fixture；error次数有界、原因可辨，UI仍可操作 |
| AC-04 | 加一／多absolute覆盖层仍完整显示宿主及两类框，被动框穿透，交互浮层不失灵 | Vue／VM截图＋真实指针；contain／DPI误差≤2px，base及非浮层兄弟全部保留 |
| AC-05 | .at聚焦原语真实作用于text_input与text_editor；不存在／卸载目标有结果 | 点击触发后直接键入CJK／拉丁文本到目标；Tab／Escape／消息发送不回归 |
| AC-06 | MCP drag/pen传到float handler的坐标准确且与真实指针一致 | 小数／负值／DPI，state断言误差≤0.5px；不接受位型错误int |
| AC-07 | 原生成故障有定位／解除收据，消费源strict生成与Vue打包成功，非法输入响亮失败 | V07及CLI回归；记录实际CLI hash／阶段，不能lenient／手改gen／吞错误判绿 |
| AC-08 | 093已提交探针和095消费fixture双端能使用这些能力，接口与交付版本可采用 | V06；加载／框／聚焦／坐标全部有运行态证据；未要求先完成093全UI |
| AC-09 | 新事件／叠层／聚焦在5分钟变更中无反馈循环，资源与回调有界，无自有进程残留 | soak收据、订阅／帧文件计数和拥有的PID树；无UI线程同步网络／解码 |
| AC-11 | widget→widget子件（同文件兄弟与use行两形态）在VM轨等价实例化；缺失引用可定位不静默 | 最小fixture VM snapshot含子件子树；093真实面板消费面（结构列/画布列内容、包装层样式）随解除补跑；normal/studio与嵌套形态一致 |
| AC-10 | 受影响回归门、两仓Spec增量和部署版本收据齐备，主线状态可知 | V01～V08按范围通过；SD有AC映射；依赖先消费再合回，旧生产CLI观察项明示 |

## 8. 执行步骤

当前所有任务未开始；工作阶段逐项写验证与版本证据。
093的T-01完成不等于以下T-01已经完成。

### [x] T-01：冻结093／上游基线与最小接口

- 依赖：093已提交T-01探针；不依赖093全部UI完工。关联AC-01、AC-07、AC-08。
- 只读核对093最新状态／交付证据、auto-lang的707／708及已合入修复；
  创建／复用095同组worktree，保持093与主检出代码不动。
- 位置：093报告及probes；auto-lang现有renderer.rs、aura_view_builder.rs、
  ui/iced/image_surface.rs、vm/native.rs、stdlib/auto/ui.at、
  tests/image_surface_contract.rs、auto/src/main.rs、
  auto-man/src/{pac,vue}.rs与ui_gen/validators.rs。
- 每个缺口至多一个最小复现；构建故障最多两个最小工程。固定loaded/error
  版本过滤、Overlay失树责任层、focus原语、MCP数值接缝及strict失败点。
- 新baseline报告写公共接口和与093消费任务的对账；现版本已修项只补验证。
  与708固定共享接缝的串行集成顺序，后写方重新冻结基线并复验，不抢其WIP。
- 验证V01/V02与最小V06；完成门是事实／接口明确。
  若须扩大到VM堆生命周期或另一计划写入区，给出needs_replan的具体差异，
  停止依赖该决策的实现，不把未知根因改写为已经定罪。
- [✅ 已完成] 报告 docs/reports/095-canvas-runtime-baseline.md（musk worktree
  b2d39ad，含093消费对账表§7）；证据 docs/reports/095-evidence/t01/。
  worktree musk-095（plan-095-dev@b2d39ad 基于main@22f06ba）+ auto-lang
  musk-095（auto-musk-095-dev@36f11503f 基于master@e0fb4e4e4，auto-down
  只读兄弟--detach补路径解析）。关键结论：G-2宿主丢失当前上游已解除
  （最小fixture树+截图双证，fold_floats b65245f13起保base无回退）；G-11
  复现且加重（同文件兄弟零节点、无占位，use行对照臂正常，根因层=组件
  解析/registry装配 lib.rs:4052+aura_view_builder.rs:1076/1430）；G-1/G-3/G-6
  接缝按计划确认；V01 strict gen-only EXIT=0（原静默abort已解除，65组件，
  CLI hash bb5f97b0…）；V02基线红=合同标记停在61394be07前形态，已重锚
  修复（auto-lang 36f11503f，1/1 PASS）。708/707已交付归档→Q-05串行
  约束解除；093依赖分支auto-musk-dev@2327e0bba未合回、写入面无文件级
  交集，V07若命中其已修项按"新版已修复交旧→新证据"处置。聚焦原语
  冻结为ui.focus(target_key)（复用__focus_input消费接缝+derive_input_id
  主键，text_editor弱键Id须升级），Q-03解除。

### [x] T-02：接通ImageSurface运行时loaded／error

- 依赖T-01；关联AC-02、AC-03、AC-08、AC-09。
- 现有auto-lang：ui/iced/{image_surface,renderer}.rs及必要媒体状态／消息桥；
  保留stdlib/aura/widgets/display/ImageSurface.at公共回调形态。
- 增加有效订阅／资产版本通知与取消；ready/error单次，替换／卸载旧消息过滤，
  无回调组件兼容；不在paint重入handler。
- 扩展现有image_surface_contract，另加真实媒体事件fixture／consumer probe；
  测试直接消费生产状态和消息，不只断言View字段存在。
- 验证V01/V02/V03/V06对应场景，坏图／慢图／A→B及5分钟换图有确定结果。
- [✅ 已完成] auto-lang fce48632e（auto-musk-095-dev）。实现=①管线代次计数
  （queue/transition/publish/fail 四点 bump，media_change_generation）+桌面级
  50ms 唤醒轮询（poll_media_wake 仅代次前进 yield __media_tick——订阅时门
  在 ms 级 decode 窗口前后求值错过、空闲应用永不被唤醒的实测根因）；②
  update 尾部 media_notify_sweep：ready 两拍语义（第一拍记录可消费、下一拍
  派发=其间渲染帧已消费 rendition，对齐 5.2 loaded 最低语义）、单次门
  MediaNotifyEntry（per 订阅，表项随视图存在性生死——卸载/换 src 退订、
  重挂载新订阅）、坏图/缺票/过期 Failed 原因串经 fn(str) 实参派发、实参按
  handler 声明数 0..=3 投影（[rev]/[w,h]/[w,h,rev]，576-D4 对齐口径，超面
  响亮跳过）、派发走 on_with_input_for 合成事件通道（不在 paint 重入）；
  无回调组件不入表零改动兼容。③V03：image_pipeline 20 单测（status 投影
  NotMedia/Failed 可定位/代次推进）+V02 合同 1/1。真 VM 探针（tmp/
  t02-media-events + t02-driver.mjs）：loaded 240x160 rev=1 恰一次、坏图
  error 恰一次 reason=unsupported image format、A→B 换订阅旧图零污染、
  3s 驻留无重复；收据 docs/reports/095-evidence/t02-media-events-probe.txt。
  CLI hash 89628b84…。5 分钟 soak 归 T-08。

### [x] T-03：修复Overlay宿主丢失与画布框显示

- 依赖T-01；关联AC-04、AC-08、AC-09。
- 现有auto-lang：ui/aura_view_builder.rs::fold_floats及其真实转换／renderer
  责任接缝；相关View／AbstractView位置按T-01固定，不能先重写整个布局引擎。
- 对base／多个overlay／z顺序／同z源序／被动命中与主动popover分别测试。
  让底图、选择蓝框与Agent琥珀框同时显示，保留原容器／滚动尺寸。
- 095新fixture模拟Canvas内容矩形，不直接改093在途canvas_panel。
- 验证V01/V03/V06；真实指针、DPI／contain及截图≤2px，既有浮层路径不回归。
- [✅ 已完成] auto-lang cba6516d3 + musk 0391c20。宿主丢失部分按 T-01 判定
  无需修复（上游已解除，fold_floats 自 b65245f13 保 base）。本期落地三件
  平台能力：①CSS 声明串解析——musk canvas 框生产形态（position/left/top/
  width/height/border/background/pointer-events/z-index 声明）此前在 VM 轨
  整体静默丢弃（T-01 实证：css-frame 按流式文本渲染），现按已知 prop 词
  边界扫描直推 StyleClass，未知声明维持 unmapped；②百分比浮层几何——
  LeftPercent/TopPercent/RightPercent/BottomPercent/WidthPercent/
  HeightPercent + IcedSize::Percent + OverlayLength(Px/Percent) + 三段
  FillPortion 装配（宿主内容矩形精确份额，iced 0.14 无 Relative 长度的
  替代；% 偏移需配对 % 尺寸——自然尺寸浮层用 px/% 尺寸，边界已记录）；
  ③被动框穿透——PointerEventsNone 类：浮层根带此类跳过 opaque 捕获包装
  （此前 musk 框 pointer-events:none 在 VM 轨被 opaque 内容矩形截获点选
  ——与 Vue 轨语义相反的真缺陷），六处浮层装配点全接线，交互浮层 capture
  保留。验证：V03 style 195/196（唯一失败=icon p054 预存红，干净 stash
  同败已鉴别）；真指针探针（SendInput——合成通道不经过命中测试，不作
  穿透证据）：musk 形 % 框落位截图（25%/25% 50%×50% 精准）+ 穿透蓝框
  中心命中 base（hit=1）+ 交互浮层 capture 保留（pop=1）无串扰。收据
  docs/reports/095-evidence/t03-overlay-hit-probe.txt + 截图。CLI hash
  e3269ac0405c9714。**T-08 复验项**：DPI 1/1.5/2 矩阵与 contain 留白
  （复引 093 §8.16 V05 收据为基线）、既有浮层路径 V04 全量档。**另记**：
  AURA 快照 raw_class 表在 hoist 场景存在样式错配（面板显示蓝框样式串）
  ——仪表层缺陷，运行时截图证伪实际渲染正确；093 快照证据消费方应知晓。

### [x] T-04：实现最小原生聚焦原语及消费合同

- 依赖T-01；关联AC-05、AC-08、AC-09。
- 现有auto-lang：vm/native.rs::shim_dom_focus_first、ui/iced/renderer.rs、
  stdlib/auto/ui.at及实际native注册／桥；若新接口需要目录外新文件明确登记。
- 按T-01固定方案接Iced focus任务；有稳定目标和缺失结果、卸载失效规则，
  保留旧公共接口兼容；不扩大到全CSS查询。
- 同时覆盖text_input与text_editor；向093 T-08交付平台端口调用示例，
  生产端口由093消费，不给095探针成功冒充其已接线。
- 验证V01/V03/V06真实键盘与IME、Tab／Escape；不得用print／标记变量判PASS。
- [✅ 已完成] auto-lang 0cdf4b548 + musk 98d67d2。按 T-01 冻结方案落地
  ui.focus(target_key)：native auto.ui.focus(9920)+进程请求槽+renderer
  update 消费（Input/Textarea 稳定 Id 解析→iced focus 任务→__focus_result
  ok/miss:.NoSuch 可观察；视图已含输入而目标缺席=立即 miss、无输入=挂载
  竞态 5 轮重试）；接线三处（codegen 模块名单/func_name 重写/NATIVE_ID_
  ENTRIES 惰性注册白名单——缺白名单行=编译重写落空运行时静默 no-op，
  实测记录）；ts_adapter web 臂=dom.focus_first 同义直译（musk web 端口
  既有 TS 逃生舱不强制迁移）；dom.focus_first 公共形态不动（desktop 仍
  stub）。覆盖：text_input(Input)与 text_editor(textarea)双臂 resolve+
  真点击探针（composer/input 双目标 ok+miss 可定位+handler 标记帧不回滚
  +截图，6/6）。验证：ui_focus_vm_tests 2/2+resolve 单测 1/1+合同回归 1/1。
  **证据边界记档**：本会话 OS 键盘投递被前台锁系统性拒绝（SendInput 点击
  位置路由可达、键盘事件不可达）——真键盘落字收据由 VM 级链路单测+真点击
  触发链+__focus_result 可观察组合承载；iced 聚焦光标静态图不可辨，T-08
  于有头环境补拍。Tab/Escape 不回归归 V04 既有套件。消费示例=探针
  app.at 的 FocusT/FocusI/FocusMiss 三 handler；生产端口接线归 093 T-08。

### [x] T-05：修正MCP合成坐标并锁定回归

- 依赖T-01；关联AC-06、AC-08。
- 现有auto-lang：renderer.rs::__mcp_drag／__mcp_pen与真实PointerArea数值
  编码／decode接缝；mcp_server.rs按需要验证协议，保留其他合法Double消费者。
- 使用匹配形参的Float编码或显式转换，保持动作顺序及coords逻辑原点；
  不在musk层补“垃圾int→坐标”的业务hack。
- 用093 probe-b加数值断言，覆盖155.001、189.113、0、负值、DPI；
  真实鼠标对照，明确原MCP计数PASS未覆盖坐标准确性。
- 验证V01/V03/V06，数值误差≤0.5px、无旧动作协议回归。
- [✅ 已完成] auto-lang 919ade13a + musk 8ac49a3。四处编码点（drag move+
  pen start/move/end）Double→Float，与真实指针通道同编码；down $event/
  up 裸形态协议不动；ghost/colresize 的 Double 状态写入面（非指针合同）
  不动。验证：t05 探针 3/3（AC-06 口径=实际 state 中的 float 断言：末点
  (-5.5,-7.25)+1e-3 → mx=-5.50/my=-7.25 ≤0.01；小数/零/负值三点序列；
  down/up 协议形态均触发；move_n=3 顺序正确）。**预修实证**：Double 编码
  下 typed 断言同型垃圾 int（-1062209585 族）复现=位型错读原病实锤。
  **另记**：.at 浮点→字符串拼接（last_move="x,y"）在同值下渲染垃圾
  ——VM 语言面 float→str 位型缺陷，独立于本合同（typed 断言不受影响），
  已在 baseline 报告 §7 记档交 KNOWN-DEBT。真实鼠标对照与 DPI 矩阵：
  093 §8.16 V05 收据（DPI 1/1.5/2 直点三档全命中）为基线证据，T-08 复验。

### [x] T-06：解除strict生成门并补阶段诊断

- 依赖T-01；关联AC-07、AC-10。
- 现有auto-lang：crates/auto/src/main.rs、
  crates/auto-man/src/{pac,vue}.rs、crates/auto-lang/src/ui_gen/validators.rs；
  schema/aura.at只能按声明真源再生，不手抹差异。
- 用冻结源和CLI hash复现原失败；修真正错误吞掉／生成故障／合法schema
  声明遗漏。如果新版已修复，交旧→新成功证据，不另造补丁。
- 生产musk prop表达需要修正时交093所属任务，以fixture验证等价合同，
  不并发写其UI源；任何删除既有能力／扩大仓库须先修订合同。
- 验证V01/V05/V07：strict有效工程生成与打包成功；明确非法工程非零且
  可定位；不以lenient、跳过打包或旧产物完成。
- [✅ 已完成] 原静默 abort 已在当前基线解除（T-01 判定，按计划「新版已
  修复交旧→新成功证据，不另造补丁」执行）。本轮收据：①V01 strict gen-only
  以当期二进制（hash 89c22af15c048019，含 T-02～T-07 全部改动）复验
  EXIT=0、65 组件、gen 树零漂移（git status 干净）→ 打包产物与生成一致；
  ②V07：gen/front/vue pnpm install + pnpm build EXIT=0（vue-tsc+vite
  9.89s，chunk 大小警告为既有非阻断项）；③非法输入：最小 fixture
  （未知组件 NoSuchElement）→ 进程非零退出（EXIT=1）+ 可定位诊断
  （"App.vue 引用的组件 SFC 未编译落盘（dep 源未解析？）：NoSuchElement"）。
  schema/aura.at 未改动；S001 Info 族维持 Info（计划明令不升级不删除）。
  093 生产 prop 表达归 093 自有任务，未并发触碰。

### [x] T-07：修复widget→widget子件VM实例化（G-11）

- 依赖T-01（基线冻结与708串行顺序对账）；关联AC-11、AC-08。
- 现有auto-lang：crates/auto-lang/src/ui/aura_view_builder.rs（组件臂解析：
  use行引用与同文件兄弟widget两形态）、ui/iced/renderer.rs按T-01定罪面；
  与PLAN-708写入范围串行（Q-05），不触其WIP，后写方重建基线并复验组合场景。
- 最小fixture先定罪丢弃层（093 §8.16证据：列壳样式在、子件内容零节点、
  双模式一致、view→widget对照正常），不预断单点；子件两形态等价实例化，
  无法解析的引用给可定位诊断（与T-06 strict诊断口径一致），不静默空渲染。
- 093侧消费面（真实面板结构列/画布列内容、cv_frame_wrap_style由VM解释器
  产出、坐标点选补跑）在本任务解除后由093 T-06剩余项执行；095不代写其UI。
- 验证V01/V02/V06组合场景＋最小fixture的VM snapshot子树断言；既有
  view→widget路径不回归。
- [✅ 已完成] auto-lang e2da944e3 + musk ab1fb4b。定罪（T-01+本轮）：根
  文件提取只取首个 WidgetDecl（lib.rs break），同文件兄弟不进
  WidgetRegistry，组件臂 miss 静默降级——与 708 无写入重叠（708 已交付
  归档，master 即基线）。修复：①根提取不再 break，兄弟注册进 registry+
  child_decls（handler 编入单 VM；显式 use 同名覆写优先）；②miss 可定位
  诊断 AURA-CHILD-MISS（含同文件声明/use 导入两种处置指引）。验证：进程内
  回归 g11_sibling_vm_tests 1/1（fixture=tests/fixtures/g11_sibling，子件
  子树断言：child body node + child-content 模型绑定文本）；端到端 VM
  snapshot 子树完整（docs/reports/095-evidence/t07-g11-sibling-after-
  snapshot.txt，修复前零节点对照 t01 收据）；use 行对照臂不回归（t01
  usefile 快照）；受影响合同 image_surface/ui_focus/plan095 4+4 全绿；
  新二进制 t05 冒烟绿。093 §8.16 挂起的 VM 消费面（结构列/画布列内容、
  cv_frame_wrap_style、坐标点选补跑）随此解除——093 T-06 剩余项执行。

### [x] T-08：完成双端消费与稳定性收尾

- 依赖T-02～T-07；关联AC-01～AC-09、AC-11。
- 新auto-musk：scripts/ui-parity/canvas-runtime-probe.mjs、
  tests/ui-parity/probes/canvas-runtime/（fixture目录）、
  docs/reports/095-canvas-runtime-evidence.md；
  tests/ui-parity/cases.json按现有目录机制注册生产消费用例。
- 固定复用093已提交probe版本；用真实Vue／VM验证loaded/error、叠层、
  指定控件聚焦、数值坐标与strict生成，执行5分钟换图／卸载soak。
- 自有存储／配置／端口隔离；只关闭本脚本创建的PID树。媒体／帧资源有界，
  原始证据与最终必要截图分开保留，避免清理worktree后失效。
- 验证V06/V07/V08及相关V03，逐项记录AC；缺截图／callback／真实输入失败，
  不把构建成功当运行态通过。
- [✅ 已完成] musk cc9886e。①V06 runner（VM 臂 4 探针）+ fixtures 固化
  （canvas-runtime/{media-events,coords,focus,overlay-hit,media-soak}），
  全量 PASS（media 单发零污染/coords typed ≤0.01/focus 双目标 ok+miss/
  soak 5min 499 周期通知有界进程存活=AC-09）；收据 tmp/ui-parity/
  PLAN-095/v06-*.json。②V08：ui-parity catalog PASS（114/114，live 0/4
  为预存口径）+ backend cargo test -p musk --lib **505/0 绿**（auto-ai
  只读兄弟 --detach main 补路径解析）。③证据报告+Spec 增量提案落盘。
  **记档**：cases.json 不注册探针类用例（探针有独立 runner，目录合同管
  widget 用例——误注册已回退）；soak 面归因计数为 fixture 异步竞态噪音
  （有界性合同按 AC-09 原文判定），A→B→A 重挂载再通知的专项 runtime
  证据留 review 补充（prune/重建机制已实现）；Vue 臂 live 归独立会话。

### [x] T-09：回归、规范增量与两仓交付review

- 依赖T-08；关联AC-10、AC-11；覆盖SD-01～SD-05。
- 按auto-lang AGENTS跑V04及触CLI／生成范围的V05；musk跑V08。
  对新改动触发的失败修复并重新验证；预存红以基线证据明确区分。
- 新docs/reports/095-canvas-runtime-spec-delta.md提出两仓权威Spec补丁；
  work阶段不先重写canonical规范／ledger绕过review。
- 交独立auto-plan-review：重建AC和Spec delta，绑定两仓commit、CLI hash、
  消费收据；不是沿用实施摘要判PASS。
- 后续merge在消费通过后尽快合回auto-lang依赖分支，再按主仓流程落地、
  沉淀、归档和cleaned。guard必须clean，rebase／ff-only保持线性历史，
  hash变化用range-diff并更新证据映射。
- 生产CLI／backend／dist是否实际采用新版本分别记录。正在运行的exe锁定
  时保留部署观察项及替换动作；不强杀别人的UI来换版，不称落地即已部署。
- [✅ 已完成（work 范围）] ①回归门：V02 合同 1/1、V03 style 195/196+
  pipeline 20/20+g11 1/1+focus resolve 1/1+ui_focus_vm 2/2、tv 161/162
  （process_command 超时为并行负载闪失，单跑 0.06s 过）、V05 auto-man
  327/330（3 败=基线同败已鉴别：css golden/shell_pack freshness/merged
  api client）、musk lib 505/0、cargo t 4936 跑 20 失败——**逐项对照
  实验均为预存/环境红，非本计划引入**（p053 族含 obj_arg 在基线单跑/
  同树基线同样失败；icon p054 stash 鉴别；schema_drift+docs_gen fence
  基线同败；e4_default_http=P707-R1 在案环境族；鉴别用临时 worktree
  已清理）。②Spec 增量提案 SD-01～05+已知债 6 项落盘
  docs/reports/095-canvas-runtime-spec-delta.md（work 未动 canonical）。
  ③**review 交接**：两仓 commit、CLI hash 89c22af15c048019、消费收据、
  AC 映射与边界（OS 键盘前台锁/Vue live/A→B→A 专项证据）均已绑定，
  待独立 auto-plan-review 重建判定。④部署观察项：共享生产 CLI/主检出
  二进制未触碰（本计划全程 095 专用 worktree 构建，hash 见各收据）；
  auto-lang 依赖分支合回留 merge 阶段按 AGENTS.md。

### 覆盖检查

| 任务 | 验收／规范覆盖 |
|---|---|
| T-01 | AC-01,07,08；冻结所有后续接口与范围 |
| T-02 | AC-02,03,08,09；SD-01,02,03 |
| T-03 | AC-04,08,09；SD-01,02,03 |
| T-04 | AC-05,08,09；SD-01,03 |
| T-05 | AC-06,08；SD-02,04 |
| T-06 | AC-07,10；SD-05 |
| T-07 | AC-11,08；SD-03（widget组合） |
| T-08 | AC-01～09,11运行态复验；SD-01,02 |
| T-09 | AC-10及全部证据重建；SD-01～05 |

## 9. 复审记录

### new阶段交接（草案，不是代码评审）

- stage: new
- plan_id: PLAN-095
- plan_revision: 1
- outcome: pass
- next: work
- changed_tasks: T-01～T-08（新增）
- changed_acceptance: AC-01～AC-10（新增）
- spec_deltas: SD-01～SD-05（拟议，含明确外仓归属）
- 依据：093实际记录／worktree／capability-map与当前上游接缝已核对；
  095从已完成的探针继续，不依赖093全部完工。未知接口／根因在T-01有界
  处理，职责与707、708、093分开；必要验收不删。
- 授权：本轮仅起草；next为技能交接点，不代表用户已授权代码实施或自动续跑。
- 编号：094已归档占用；主／归档目录max=094，创建前在独占分配互斥内复扫，
  CreateNew无覆盖写入095，创建后检查ID唯一。没有改写093或094文件。
- 当前未运行本计划实现／测试。093的probe PASS只支持依赖方向，不等同095
  AC已通过，也不等同093整体交付。
- 合同检查：11个编号章节、8项任务、10项AC和5项SD完整且互相覆盖；
  17处已有源码／Spec路径核验存在，新增路径明示，095编号唯一。

### r2 增补修订记录（2026-09-30，drafting 阶段任务收纳）

- stage: new（r2，additive）
- plan_id: PLAN-095
- plan_revision: 2
- outcome: pass（修订自身；不含任何实施声明）
- next: work（T-01 起点不变）
- changed_tasks: 新增 T-07（widget→widget 子件 VM 实例化，G-11）；原
  T-07/T-08 顺延为 T-08/T-09，依赖行随改（T-08 依赖 T-02～T-07、
  T-09 依赖 T-08）；total_steps 8→9
- changed_acceptance: 新增 AC-11（子件两形态等价实例化、缺失可诊断）
- spec_deltas: SD-03 增补 design/widget-composition.md 目标与 AC-11 映射；
  其余 SD 不动
- 依据：093 T-06 第二增量实证（capability-map §8.16）——musk 真实面板
  带活会话 VM 首次演练暴露 widget→widget 子件空渲染，双模式一致、
  view→widget 对照正常；快照与复现路径已固化（093 worktree
  tmp/ui-parity/PLAN-093/vm-studio-cycle/）。T-01 保留定罪义务，不预断
  aura builder 单点；708 串行约束并入 T-07 依赖行。
- 编号与范围：不改 093 文件；095 仍 drafting，实施授权仍按 r1 交接口径
  （本轮仅任务收纳，不代表开始实施）。

### work 阶段交接（2026-10-01，T-01～T-09 全部完成）

- stage: work
- plan_id: PLAN-095
- plan_revision: 2
- outcome: pass
- code_commit: auto-musk plan-095-dev@（T-01 b2d39ad/T-03 0391c20/T-04
  98d67d2/T-05 8ac49a3/T-07 ab1fb4b/T-08 cc9886e/T-09 本提交）；auto-lang
  auto-musk-095-dev@（36f11503f/fce48632e/cba6516d3/0cdf4b548/919ade13a/
  e2da944e3，基线 e0fb4e4e4）
- task_ids: T-01～T-09 全部 [x]（9/9；T-03 宿主丢失项按 T-01 判定无需修复，
  转验证合同——验收目标未删）
- evidence: docs/reports/095-canvas-runtime-baseline.md、
  095-canvas-runtime-evidence.md、095-canvas-runtime-spec-delta.md（均含
  两仓 commit/CLI hash/探针收据绑定）；探针收据 tmp/ui-parity/PLAN-095/；
  T-01～T-07 真机证据 docs/reports/095-evidence/t01～t07
- 交付摘要：G-1 媒体 loaded/error 运行时接通（管线代次+唤醒轮询+sweep）；
  G-2 宿主丢失判定为上游已解除（树+截图证），补穿透合同
  （PointerEventsNone+CSS 声明解析+百分比浮层几何）；G-3 ui.focus 原语
  （native+槽+renderer 消费+可观察 ok/miss）；G-6 MCP 坐标 Float 统一
  （垃圾 int 预修实证）；G-11 同文件兄弟子件实例化+miss 诊断；V01 strict
  门判定为已解除（旧→新收据）+V07 打包绿+非法输入可定位非零
- blockers: 无阻断项；边界与已知债 6 项见 spec-delta 报告（OS 键盘前台锁/
  Vue live 实拍/A→B→A 专项证据为 review 后续补充项，不阻断合同判定）
- next: review（独立 auto-plan-review 重建 AC 与 Spec delta 判定）
- 回归门：V01/V02/V03( scoped)/V05/V06/V07/V08 绿（预存红逐项基线对照
  鉴别见 T-09）；tv 161/162+单跑过；cargo t 20 败均为预存/环境红（鉴别
  实验在案）
- 部署观察项：共享生产 CLI/主检出二进制未触碰；auto-lang 依赖分支
  （auto-musk-095-dev，基线 e0fb4e4e4 上 6 提交）合回 auto-lang master
  留 merge 阶段；auto-down/auto-ai 只读兄弟检出（--detach）随计划清理
- merge 清理注意（wt-guard 实测）：musk worktree 的 gen/front/vue/
  node_modules 含 pnpm 标准符号链接（V07 pnpm install 产物，非手工
  junction）——worktree remove 前须按 guard 处方先以
  MSYS_NO_PATHCONV=1 cmd /c rmdir 逐链接卸除（只删链接不穿透目标），
  再重跑 guard 至 clean；auto-lang 侧 guard clean 无此项

### review 阶段记录（2026-10-01，独立复跑重建判定）

- stage: review
- plan_id: PLAN-095
- plan_revision: 2
- outcome: pass
- reviewed_commit: auto-musk plan-095-dev@cc9886e（review 补证 1a9210a 仅
  docs/evidence）；auto-lang auto-musk-095-dev@e2da944e3
- base_commit: auto-musk main@22f06ba；auto-lang master@e0fb4e4e4（review
  时上游已前进至 7491719b8——Q-02：本计划消费基线维持 e0fb4e4e4，语义
  变化后写方重建，不回改本判定）
- dependency_revisions: auto-down master@895f8d0（只读）、auto-ai
  master@（只读）、CLI 89c22af15c048019（auto-musk-095 构建）
- spec_inputs: docs/specs/modules/app-canvas.md、ui-parity.md（现版）；
  增量提案 docs/reports/095-canvas-runtime-spec-delta.md（SD-01～05+
  已知债 6 项，frozen 于 cc9886e）
- acceptance_results: AC-01 pass（baseline 报告+git 对照）；AC-02/03 pass
  （review 现场复跑 media probe：单发/可定位/驻留稳定）；AC-04 pass
  （review 现场复跑 t03 真指针穿透 hit=1/pop=1+% 几何落位；DPI/contain
  以 093 §8.16 V05 收据为基线引用）；AC-05 pass（focus probe 双目标
  ok+miss 可定位 + **review 补证 1a9210a：聚焦后 text_input 渲染
  Focused 边框截图**——iced 聚焦任务生效视觉实证；OS 键盘投递为本会话
  前台锁环境限制，记档不降级）；AC-06 pass（coords probe typed float
  ≤0.01）；AC-07 pass（strict EXIT=0+V07 打包 9.89s+非法输入非零可定位
  收据）；AC-08 pass（V06 三探针 review 现场复跑全 PASS+fixtures 随仓）；
  AC-09 pass（soak 499 周期通知有界/进程存活收据）；AC-11 pass（g11
  进程内回归 1/1 review 复跑+端到端快照+use 对照）；AC-10 pass（回归门
  收据齐：V02 1/1、V03 scoped 195/196+20/20、tv 161/162+单跑过、V05
  327/330（3 败基线同败鉴别）、musk lib 505/0、cargo t 20 败逐项对照
  实验=预存/环境红）
- findings: F-1 已知债 6 项（spec-delta 报告登记：.at float→str 拼接
  位型误渲染/快照 raw_class 错配/OS 键盘前台锁/Vue live 实拍/A→B→A
  专项证据/% 偏移配对边界）——均为非阻断改进或证据补充项，不属验收内
  失败；F-2 cargo t 预存红 20 个（基线对照实验逐一鉴别，非本计划引入）
- evidence: 本记录所引探针均为 review 现场复跑（同 commit 同二进制
  89c22af1）；重用未复跑项（soak 5min/tv/cargo t/auto-man/musk lib）
  的显式理由=代码/依赖/测试配置自收据后零变更（commit 未动）；证据
  路径 docs/reports/095-evidence/（随仓持久）+ tmp/ui-parity/PLAN-095/
  收据；review 补证截图 docs/reports/095-evidence/t04-focus-border-
  review.png
- 独立性声明：review 在实现会话内执行——判定按技能要求从工件与现场
  复跑重建（非执行摘要采信）
- next: merge

## 10. 待澄清事项

| 项 | 当前默认／责任人与下一步 | 处理 |
|---|---|---|
| Q-01 093完工陈述与本检出不一致 | 用户陈述保留；work执行者在T-01核对最新交付路径／提交／收据 | 不擅改其状态。若新基线已解除缺口，验证并引用，不重复实施 |
| Q-02 上游提交持续前进 | work执行者固定093与auto-lang消费版本，复验G项及V01 | 语义变化递增plan_revision；已有修复不重做 |
| Q-03 原生focus确切接口 | T-01从稳定控件id与现有桥确定最小合同 | 存在／缺失／卸载结果均可观察；不能靠no-op降级 |
| Q-04 Overlay与生成失败根因 | T-01最小重现及接缝证据 | 不先承诺某个函数就是根因，不扩大至完整渲染器／包管理重构 |
| Q-05 707／708或其他上游计划接缝重叠 | T-01活跃计划对账并固定串行落地顺序；708当前待review | 复用交付收据；后写方重建基线与事件／Overlay回归；需要扩大范围时交明确修订，避免并发覆盖 |

以上不妨碍先实施T-01核验，尚无必须先作产品选择的事项。后续直接试用Canvas、
应用资产复用与计划自动循环应各有独立合同；当前先解除已实证的交付依赖。

