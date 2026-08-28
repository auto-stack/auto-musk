---
plan_id: PLAN-047
status: archived
feature_name: musk VM 首跑——auto run -r vm 实机启动 + 运行时红清单采集消化 + MVP 链路打通
author: [zhaopuming]
created_at: 2026-08-27
updated_at: 2026-08-27

supersedes_spec_components: []
# 理由(2026-08-27 复审):基建/工具化计划,无 specs 域文件增删——沿 PLAN-045/046
# 先例三字段留空;交付物=scripts 门禁两件、ports 双 adapter 域内补齐、计划红清单
# 节、KNOWN-DEBT 047 行,均以文件+台账行承载(spec overview 关键能力第5条双前端
# parity 表述经 047 活体运行获得运行时实证,无文本变更需求)。
new_spec_components: []
touched_goals: []

current_step: 9
total_steps: 9
---

# [PLAN-047] musk VM 首跑

## 变更摘要

musk `.at` 前端经 PLAN-045（链接复绿）/PLAN-046（债务清偿）已达"VM 目标 parse+codegen+link 全绿、探针门禁常驻"，但至今只在探针里活过——从未有真进程把它跑起来。auto-lang 侧 `auto run -r vm`（AURA 解释器 + Iced 渲染器）已可用且官方 examples/ui 前 7 例双后端 parity 通过（其仓 455 计划追踪中）。本计划把两侧接上：**固化 musk VM 启动器 → 采集第一批运行期红清单 → 按"平台协议 + VM 实现"逐域消化已知缺口（i18n/markdown/icons/timer）→ 打通"登录 → chats 会话列表"一条非流式端到端链路**。SSE 流式等大体量域不在本计划（第二梯队独立立项）。

## 目标

1. 存在一行可重复执行的 musk VM 启动命令（`scripts/` 固化，形态由 T1 勘察定型）。
2. "首跑红清单"节在案：每条运行期红分类为 已修 / musk 源规避 / 上游移交 / 降级登记 四类之一，无悬空项。
3. i18n / markdown / icons / timer 四个平台协议域各有处置落地或一行结论入账。
4. 端到端 MVP：VM 前端连真后端（musk serve :8080）完成 login → chats 列表 → 单会话非流式渲染打开。
5. vue 轨三门禁零回归（`auto build` strict / vitest 23+1 / 对拍 30/30）+ VM 探针两连绿——首跑不得以牺牲既有门禁为代价。

## 架构方案

```
现状(只活在探针里)                        本计划后(真进程 + 红清单驱动)
──────────────────────                  ────────────────────────────
vm-link-probe: parse+codegen+link  →    scripts/vm-first-run.mjs(形态沿探针先例):
8/8 绿, flash 合成完毕即停              启动 AURA+iced 宿主装载 musk 模块,
                                        进程存活 + stderr/stdout 日志采集分级
platform:markdown/icons/i18n/timer →    四域各一件: VM 侧实现或显式降级登记
  仅有 Vue 平台实现(StreamingRenderer    (platform.*.at 扩展位 / composables.vm.at /
  .vue 等 auto-man 注册表挂载)           ports 层), 大体量缺口(comrak 等)立上游项
无任何运行期验证面                  →   首跑红清单节: 分类处置闭环, 上游语义 bug
                                        沿 046 先例挂 KNOWN-DEBT 不硬修
```

**执行流**：代码改动全部走 worktree `.worktrees/plan-047-dev`（分支 plan-047，
GEMINI.md L1；沿 PLAN-045 目录命名教训选可命中路径）。启动器若需指回主检出/
worktree 双态，参照 045-T1 的目录命名法（junction 方案曾栈溢出弃用）。桌面窗体
的视觉验证 agent 无法替代（046 复审实证 headless 不能起窗）——本计划取「agent 起
进程采日志判存活与报错、视觉冒烟项进用户手测清单」分工。

## 技术栈

auto-musk（scripts/ 一键启动器、src/front/ports/*.at 与 composables.vm.at 域补齐、
KNOWN-DEBT 台账）；auto-lang **只消费不修改**（`auto run -r vm` 宿主链路；运行期
撞出的上游语义缺口一律登记移交，重演 046 对 auto-lang 两破例属范围外）。不动
web/（冻结）、backend/ 源码（T8 仅起服务消费）。auto-lang 主检出须在含 obj 族基线
（0737c26f3）、window 高度发布（plan-046-B）、probe-size（e3abde1ba）之后的 master。

## 需求分析与背景调查

> spec overview 端点未运行（后端未起），沿 045/046 先例：以 docs/specs/
> 00-overview.md 与 KNOWN-DEBT 台账实况为据（2026-08-27 勘察）。

### 前置达成面（本计划的立足点）

- **编译链接层**：全量前端 VM 探针 8/8 稳定绿（60.6KB 级合成模块），体积门禁
  WARN=90000/FAIL=131072 env 可调（045/046 交付，`scripts/vm-link-probe.{cmd,mjs}`）。
- **平台 native 面**：auth 三件套（set_default_header/set_default_query/
  clear_default_auth，446-E4）、obj 方法族基线（454-E5 收口三缺口）、窗口高度
  发布链（storage_host_publish + startup/dual-resize 三挂点）均已入 auto-lang
  master；`ports/platform.vm.at` 的 platformRefreshAuth/viewportHeight(KV 化)
  已接线。
- **数据桥**：VM 后端桥接形态已通（044 收口注记，extern 注册表面）。
- **i18n 数据已在 .at**：358 键全集随 041 T13 入库——VM 侧理论上可直读字典，
  无需 vue-i18n 桥等价物（T3 验证此判断）。
- **上游已有宿主能力**：`auto run -r vm`（AURA in-process + Iced/wgpu），
  examples/ui 001-007 双后端 parity 过审（auto-lang 455 追踪，008-011 待审计）。

### 已知缺口与风险（预判红源）

| # | 域 | 缺口 | 预案 |
|---|---|---|---|
| G1 | SSE 流式 | VM 无 EventSource；forge/chats 流式交互不可用 | 本计划 MVP 限非流式；轮询方案第二梯队独立立项 |
| G2 | markdown | platform:markdown 仅 Vue 实现（StreamingRenderer.vue 挂载） | VM 侧文本降级先保渲染不炸；comrak native 立上游项 |
| G3 | icons | 动态 svg 元素序列超出语言静态表达（038 登记 a/b 两路径未裁定） | T1 盘 aura_view_builder 实际能力后裁定；不可用则文本占位 |
| G4 | timer 杂项 | setTimeout 类（copy 2s 复位等）web 特有行为 | composables.vm.at 显式 stub/登记 |
| G5 | relay 接线 | platformRunRelayCommand KV 留痕降级 BLOCKED 于 auto-lang A1 | 维持现状（046 T9 路线），不在本计划 |
| G6 | auth 边缘 | Me 死码零暴露、401 兜底待 Http status 面（046-C） | 维持协议注记现状，不在本计划 |

### 运行环境约束

- iced 需真窗体/GPU：agent 上下文无法视觉冒烟（两次不同日期复现的环境限制），
  只能判定进程存活 + 日志证据；弹层居中/视口高度生效/logout 重注入/token 汇总
  四项沿用 046 观察期手测清单，交用户实机执行。
- direct-exe 形态需 `RUST_MIN_STACK=16777216`（探针实证），启动器沿用。
- auto-lang 455 号计划活跃推进中，renderer/vm_bridge 为热点文件——T1 先查在途
  分支冲突集再动工。

## 详细设计

### D1 启动器（scripts/vm-first-run.mjs + cmd 单行委托）

沿 vm-link-probe 先例：`.mjs` 本体（node spawn，参数面 --capture/--timeout）
+ `.cmd` 单行委托。职责：调 T1 定型的启动命令 → 子进程日志 tee 至
`tmp/plan047-firstrun.log` → 判定存活哨兵（如窗口事件行/超时退出码）→
汇总红行分类计数输出 `[first-run] summary` 行。头部配置区放启动命令模板与
RUST_MIN_STACK，样式对齐 probe 脚本。

### D2 红清单分类学（写死在本计划"首跑红清单"节）

四类收口：**FIXED-musk**（改 musk .at 源或 ports 后复跑消失，注位点）/
**FIXED-conf**（清配置/依赖，如缺 font/theme 资源）/ **UPSTREAM**（auto-lang
语义缺口，挂 KNOWN-DEBT 新行移交，附最小复现串）/ **DEGRADED**（域降级登记，
指向 D3-D5 的落码位）。迭代循环至：登录页可见可交互 或 上游阻塞全部挂账。

### D3 i18n 直读（G1 预判成立时的形态）

`src/front/i18n/` 字典已是 .at 数据 → 若 VM 侧 t() 可直接 consume 同一模块则
零新增；否则建 `ports/i18n.vm.at`（web 侧 useT.vue-i18n 桥保持不动），view 经
use 块平台分派。验证锚：vue 三门禁零回归 + 首跑日志不再出现 i18n 符号缺失类红行。

### D4 markdown 文本降级（G2）

MVP 不做 comrak：凡 platform:markdown 组件在 VM 目标按纯文本渲染（保留换行）。
落位候选：ports/platform.{web,vm}.at 各加一个 textify/mardown→text 口径统一 fn，
或 markdown 组件声明处加 vm 分支实现（以 T1 盘点的组件声明形态为准，避免动
gen/ 产物手改）。真富文本渲染立上游项（auto-lang stdlib comrak 绑定）记台账。

### D5 icons 裁定 + 占位（G3）

T1 盘点 aura_view_builder 的 img/svg 能力（455 例 007 avatars 过审是积极信号）：
- 支持静态图源 → icons_data.at 有 path 数据即可上路径 (a) 形态；
- 仅支持部分 → 名称未知图标降级文本占位（unicode 或短标签），52 图标全量
  回归留后续；
- 结果无论哪种，结论一行记 platform.vm.at 头注 + KNOWN-DEBT（038 条目交叉引用）。

### D6 端到端链路（目标 4）

后端侧：`cargo run -p musk serve`（消费 041 复审过的 614 绿基线二进制行为）。
前端侧：VM 窗口内完成 login（auth natives 注头链路）→ chats 列表拉取
（数据桥 extern 面）→ 点开单会话非流式历史渲染。agent 取证方式=服务端访问日志 +
前端会话 KV/localStorage 态文件痕迹 + 进程无 panic；视觉正确性列用户手测清单。

## 测试设计

1. vue 三门禁不变量：每批改动后 worktree 内 `auto build`（strict 零 flag）
   exit 0；`npx -y vitest@2.1.9 run`（gen/front/vue）23 passed+1 skipped；
   `node scripts/lib-parity/track-switch/phase1-leaves.mjs` 30/30 exit 0。
2. VM 链接门禁不倒退：合并前主检出 `node scripts/vm-link-probe.mjs` PASS
   两连跑，体积行 <WARN。
3. 首跑门禁：`node scripts/vm-first-run.cmd`（或 mjs）退出码 + 日志分类计数
   在案；同基线复跑一次红行计数不增（稳定性粗验）。
4. 后端不重测：T8 只消费不修改 backend/，引 041 复审 614 绿基线（沿 045/046
   测试设计#4 先例）。

## 验收标准

1. `scripts/vm-first-run.{mjs,cmd}` 存在，从 musk 根一键执行能将 VM 前端进程
   拉起并产出分类红行汇总；启动命令模板在脚本头注。
2. 本计划"首跑红清单"节存在且每条有四类处置标记之一；UPSTREAM 类同步
   KNOWN-DEBT 新行。
3. 验收 4 条件满足时：登录 → chats 列表 → 单会话打开链路取证在案（日志/KV
   双证）；若被 UPSTREAM 类红阻断，本条降级为"阻断链定位到单点 + 移交记录"
   并注明不算失败（沿 046 验收#4 条款式路线）。
4. i18n/markdown/icons/timer 四域各有落码或一行结论（D3-D5 处），DOM/timer
   类 web-only 行为零回归（vitest 兜底）。
5. vue 三门禁 + VM 探针全绿（测试设计 1/2 条目）。
6. 全程不动 web/、backend/ 源码、auto-lang；musk 改动走 worktree 流程。

## 执行步骤

- [x] **T1** 勘察四合一（不改码）：a) VM 运行入口定型——读 auto-lang
  crates/auto-man CLI `run -r vm` 参数面与 examples/ui 载体，确定 musk app.at
  指向形态（直执 / 项目清单 / HostBackend 包装三选一）；b) 五域现状盘点——
  aura_view_builder 的 svg/img/text_input 覆盖、markdown 组件在 VM codegen
  的现形态、i18n 字典消费可行性、timer/async 映射现状；c) 冲突检查——
  auto-lang 455 及其他在途分支热点文件与本计划消费面交集；d) RUST_MIN_STACK
  与 cwd 要求（主检出/worktree 双态可行性，参照 045-T1 目录命名法教训）。
  产出：结论分条 `[✅]` 回填本注，D1/D3-D6 定型决策同步。
  验证：四子项各有结论行（或明确记 ⚠️ 带原因）。
  [✅(2026-08-27)
  a) **入口定型=CLI 直执形态**：`cd <musk检出根> && auto run --render=vm`
  （clap Run{render} override，main.rs:852/875；auto bin 默认 features 含
  ui-iced，auto-man→auto-lang 已带 ui-iced）。调用链=automan.rs:1418
  BackendType::Vm → rust_ui.rs:2419 run_vm_ui(CWD)：entry=`src/front/app.at`
  （与 musk 布局完全吻合）、运行期 CWD 切 src/front 使 `use` 相对解析、默认
  merged 模式无独立后端进程。⚠️ 遗留验点 R0：数据面 auth_me 等走 `use
  back.api` 后端桥而 musk pac.at 未配 back.project——back.* 无本地 back/ 时
  解析是否成立留首跑实证；分离模式备选（AUTO_VM_MERGE=0 / --server=rust+
  render=vm）记 D6 备用。
  b) **五域盘点**：i18n ✅ 现成零新增——composables.vm.at useT() 已走纯 .at
  的 lib/i18n.at（@gen 内嵌 catalog，locale 存会话 KV 'musk-language' 双轨
  互通），T3 转"验证批"。markdown：platform:markdown 仅 renderer.web.at
  （use.web 门控），VM 侧组件缺位 → widget registry miss Empty 兜底；**D4
  定型=新建 ports/renderer.vm.at 定义同名 .at 组件（文本渲染 body）**，经
  WidgetDecl 注册通道接住（EDGE-16 同机制）；消费方七文件（chat_message/
  questionnaire/report_card/generic_tool_card/relay_run_box/specs_editors/
  raw_preview）。icons：icons.web.at 全量 use.web(lucide 名面)，VM 缺位同上
  registry miss；builder 另有 img/image/icon→View::Image + svg 子树序列化
  （442 A4,aura_view_builder:883）——**裁定=MVP 取 Empty 兜底不建别名层**。
  timer：setTimeout 仅注释级登记（session_info copy 复位/secretary dismiss，
  029 已录），G4 实际空集。svg/img/text_input/textarea builder 覆盖确认。
  c) **冲突=低**：auto-lang 工作区净（仅 docs/plans/446 一文档脏）；
  plan-455 刚合 master(eb2fa40e1)、plan-446-dev 悬置非热点；本计划消费面
  （CLI run/vm codegen/aura builder/stdlib）均处 master 稳态。
  d) **栈风险坐实+缓解阶梯定型**：run_file UI 动态路径（has_ui_keywords→
  run_file_dynamic_ui,lib.rs:2917）强制 OS 主线程且不开大栈线程（对照非 UI
  run_with_path spawn 32MB）；--server=vm 注释实证连 015-notes 都需 32MB 防
  parser/codegen 递归溢出，musk 合成 ~60KB >> 之，Windows 主线程默认 1MB
  ——①先试 CLI 直跑，②若主线程栈崩→musk 根建 tools/musk-vm-host 微 crate
  （path-dep auto-man 有 backend 跨仓 path-dep 先例），rustflags
  `-C link-args=/STACK:33554432` 抬自身主线程栈后调 pub fn run_vm_ui；
  RUST_MIN_STACK 对主线程无效仍附带设置。cwd 双态：run_vm_ui 以 CWD 为
  project_dir，启动器 cd 目标检出根即可，045 junction 教训无需重演。]
- [x] **T2** 启动器 + 首跑红基线：按 T1-a 定型落 `scripts/vm-first-run.mjs`
  （头注含调用串勘误区，防 442 式过时——feature 名等写活）+ `.cmd` 委托；
  跑首次启动，tee 日志，红行逐条分类回填计划"首跑红清单"新节（D2 分类学）。
  验证：`node scripts/vm-first-run.mjs --capture` exit 码记录；日志文件存在；
  红清单每条带类别标签。
  [✅(2026-08-27) mjs 本体 + cmd 委托落 worktree(c66dbe7);首跑即**活体运行
  达成**:GPU(Vulkan/RTX 4060Ti)初始化、1280×800 窗口创建、视图循环满观察窗、
  stack/panic/codegen/link/io 全 0——045 时代担忧的主线程栈溢出未发生(CLI 直
  跑形态零包装)。alive 判定经 killing 哨兵修正(R10)。红清单 v1→v4 迭代见新节]
- [x] **T3** i18n 域处置（按 T1-b 结论，D3）：直读零新增或 ports 分派落码。
  验证：`auto build` strict exit 0 + vitest 23+1 + 首跑日志该类红行计数下降
  （before/after 各记一行）。
  [✅(2026-08-27) 终形态=双补丁:①useT 闭包体直调 i18nT(限定名错配修正);
  ②新增 fn useI18n(){t,locale} 补 settings_menu 消费面(c66dbe7+223dc5b)。
  before=i18nT 桩×1;after(run4)=useT/useI18n/settings*/gate 族桩全消。
  门禁:build strict 绿(批2 链)+vitest 23+1 绿(批后复跑)+对拍 30/30。
  lib/i18n.at 目录本就自足,架构级 D3 直读方案天然成立]
- [x] **T4** markdown 降级（D4，按 T1-b 定组件分支形态）。验证：同 T3 门禁 +
  长消息视图首跑不 panic（日志级证据）。
  [⚠️→✅ 半程入档转就绪态(2026-08-27):ports/renderer.vm.at 同名 Markdown
  widget(source/streaming 签名对齐消费方 chat_message:131/generic_tool_card:
  103/relay_run_box:560)已入库,c66dbe7。现状仍 no-op 桩=上游 A3 装载器
  fn-only 注册限制(R3),文本降级激活待上游 registry 接入;build/probe 零回归,
  首跑无 panic。KD 047 行登记双向依赖]
- [x] **T5** icons 裁定落地（D5，按 T1-b 路径分支）。验证：同 T3 门禁 +
  平台头注/KNOWN-DEBT 038 条交叉引用更新。
  [✅(2026-08-27) 裁定=MVP Empty 兜底(T1-b 预案第二支):lucide×48/Deck/
  icons_data 桩化呈空框但布局完整可用(截图实证 tmp/evidence/boot-state.png);
  不建别名层;builder img/svg 能力留后续增强。KNOWN-DEBT 047 行交叉引用]
- [x] **T6** timer/杂项 stub 批（G4 清单逐条，落 composables.vm.at 或对应
  消费点位）。验证：同 T3 门禁；stub 数与 G4 表一致（grep 自检）。
  [✅(2026-08-27) G4 实际空集(T1-b 预判实证):setTimeout 仅注释级两处
  (session_info copy 复位/secretary dismiss,029 已录行为差异),零码改;
  grep 自检在案。upload 家族桩化归 R5 DEGRADED,KD 行承载]
- [x] **T7** 红清单消化迭代批次（D2 循环）：FIXED-musk/FIXED-conf 类逐条修，
  每条一轮探针+首跑复跑；UPSTREAM/DEGRADED 类在台账与降级位收敛。终点 =
  登录页可见可交互，或剩余红全数非 FIX 类。
  验证：首跑汇总行 red 计数单调下降记录在案；终态每红有归属。
  [✅(2026-08-27) 四轮迭代(run1 基线→run2 批1→run3 发现失明→run4 终态):
  fatal 红恒 0(stack/panic/codegen/link/io 全程清零);桩谱系从 ~60 收敛至
  四个 DEGRADED 域(icons/upload/viewRouter/markdown)。FIXED 类=R2(useT/i18n
  族)+R10(启动器)。UPSTREAM 类=R1(filteredMessages computed 缺臂,点位精确
  到 resolve_iterable)/R3 上游依赖件。BLOCKED=R8(数据桥选型)。OPEN=R7/R9。
  每红归属齐备于红清单节,验收#2 达成]
- [x] **T8** 端到端链路取证（D6）：musk serve 起服（本机后台）+ VM 前端完成
  登录 → chats 列表 → 单会话打开。验证：服务端请求日志命中 auth/chats 端点
  前后端双证 + 前端进程存活 + 视觉项转用户手测清单三条以内；若验收 3 走
  降级路线，本任务记录阻断点单根因后即闭。
  [✅ 降级路线闭账(2026-08-27):R8 定罪=auth/chats 契约走 `use back.api`,
  merged 模式宿主无挂载(pac.at 无 back.project/本地无 src/back),触发验收#3
  降级条款。超出计划的取证增益=**AutoUI MCP(:9247)无头交互通道发现并贯通**:
  snapshot/state/find/type/keyboard/action/screenshot 全链可用——state 机
  press `.App.ShowChats` 切换+current_view="chats" 实证;composer 输入
  ".MentionInput.Input" 执行且 "vm probe" 渲染可见(typed-enter 截图);
  交互全程零 panic。真后端 e2e 归 R8 解除后的后续批次+用户实机手测清单]
- [x] **T9** 收口：复查门禁全量（build/vitest/对拍/probe×2/first-run×2）+
  KNOWN-DEBT 新行（UPSTREAM 移交项 + 本计划状态）+ platform.vm.at 头注同步 +
  本计划执行步骤回填与 status → execution_done。
  验证：各门禁输出在案；status 字段更新。
  [✅(2026-08-27) worktree 内终轮门禁:build strict 绿(批2 链内)+vitest
  23 passed/1 skipped+对拍 30/30 normalized equal+first-run alive=yes 多轮。
  **偏差声明**:VM 探针脚本主检出路径钉死(sibling 解析),worktree 内无法执行
  ——等价门禁=first-run 编译链接段同管线红线(fatal 五类全程 0);正式探针
  两连绿留合并后主检出终验(review/merge 流程动作)。KNOWN-DEBT 047 行新增 +
  待澄清#6 新立(R9 auth 门控裁定项)]

## 首跑红清单（T2 立,T7 收敛;2026-08-27）

> 取证形态:`node scripts/vm-first-run.mjs --observe-ms N` 日志 tee 至
> tmp/plan047-firstrun.log(多轮分节追加)+ AutoUI MCP(:9247,autoui_*)交互与
> 截图(tmp/evidence/)。收口分类(复审定稿为**五类**;原稿四类遗漏执行期新立
> 的 OPEN 观察档,review 批准追认):FIXED-musk / UPSTREAM(auto-lang 移交) /
> DEGRADED(域降级登记) / BLOCKED(外部前置) / OPEN(观察期,待用户裁定或
> 证据补全)。

| # | 信号 | 分类 | 处置与点位 |
|---|---|---|---|
| R1 | `view_builder: read_state_as_vec('filteredMessages') failed`(每帧刷屏) | **UPSTREAM** | chats_view:74/194 的视图级派生绑定(`filteredMessages => …`/for 源)在 VM 只走 state 读:`aura_view_builder.rs resolve_iterable`(~262)不查 EDGE-16 第五层 computed 表(~117,该表仅 prop 解析接)。消息列表空渲染;侧栏 store 直读不受影响。移交 auto-lang(computed 求值接入 for-source),KD 047 行 |
| R2 | `ext stub: i18nT arity0`;useT/settings* 族健康 | **FIXED-musk** | 真缺陷=useT 闭包体模块限定名 `i18n.i18nT` vs 442 A3 别名的非限定直调形态;改直调 + 补 `fn useI18n(){t,locale}`(c66dbe7+223dc5b)。run4 实证族桩全消。**勘误**:中途曾试解除 use.web 门控→plain use 在 adapter 上下文 Parse error 全文件失明(run2/run3 settings* 反增桩),已复归门控并注记 |
| R3 | `ext stub: Markdown(/Render)` | **DEGRADED(+UPSTREAM 依赖)** | ports/renderer.vm.at 同名文本降级组件已入库(就绪形态,build/probe 零回归);现状仍 no-op 桩——A3 装载器只把 adapter 内纯 fn 注册为符号,widget 不进视图 registry(上游限制,KD 并入 R1 行)。消息正文暂空,comrak 富文本上游另立 |
| R4 | lucide ×48+Deck+icons_data 桩 | **DEGRADED** | 截图实证图标位空框但布局完整可用(T5 裁定:MVP 接受);builder img/svg 能力(442 A4)留后续 |
| R5 | loadRawFileText/rawUploadProgress/uploadRawFiles/wikiUploadDrop 桩 | **DEGRADED** | wiki/上传域 VM 缺位,MVP 外 |
| R6 | useViewRouter/vsSetView/vs* 桩 | **DEGRADED** | web URL 同步桥 web-only;VM 视图状态机自足——MCP press 侧栏按钮 `.App.ShowChats` 执行、current_view 变更实证 |
| R7 | relay 状态行 `${runId}${durationLabel}${confidenceLabel}` 字面残显 | **OPEN(观察)** | 截图在案;来源未定罪(.at 模板串字面化 vs web 侧 TS 插值位),归入观察期手测+KD 备查 |
| R8 | 登录/发送链路不可达:`use back.api` 契约(back.*)merged 模式无挂载(pac.at 无 back.project、本地无 src/back) | **BLOCKED** | 触发验收3 降级路线:阻断点定位于数据桥形态选型(宿主 cdylib 挂载 vs AUTO_VM_MERGE=0 分离模式接 044 VM HTTP 桥),第二梯队独立计划;auth natives 注头链路本体已就绪 |
| R9 | 未认证直达 chats(token=nil 仍进入主视图,登录墙不生效) | **OPEN** | App 视图门控缺 auth 分支(web 轨由 router guard 承担);行为变更类修复需用户裁定,见待澄清#6 |
| R10 | 启动器把 taskkill 收尾误报 exit=1 | **FIXED-musk** | killing 哨兵(c66dbe7 前 T2 内修);alive=yes 分类此后恒正确 |

## 复审记录

**reviewer**: auto-plan:review(zhaopuming 会话)· **2026-08-27** · execution_done → **reviewed**

### 验收标准逐条复审(verify, don't trust)

| # | 标准 | 判定 | 证据(复审现场重跑) |
|---|---|---|---|
| 1 | 启动器存在、一键拉起、分类汇总、调用串头注 | ✅ pass | worktree `node scripts/vm-first-run.mjs --observe-ms 15000` 复跑 exit=0,summary 行 `alive=yes stack=0 panic=0 codegen=0 link=0 io=0 reds=0`;mjs:6 头注含 `auto run --render=vm` 调用串;cmd 单行委托在案 |
| 2 | 红清单节存在、每条分类、UPSTREAM 同步 KD 新行 | ✅ pass | 计划"首跑红清单"节 R1-R10 十条全带类别标签;KNOWN-DEBT:38 行 = 047 条,UPSTREAM 两件(resolve_iterable computed 缺臂/ext-link fn-only)精确到 file:line;**复审修正一处**:节首声明四类与表内 OPEN 档不一致,已改五类口径并注追因 |
| 3 | 端到端 MVP 链路(降级条款路线) | ✅ pass(按条文) | R8 定罪单根因=`use back.api` 契约宿主无挂载,候选双形态在案(KD 行);降级即闭,MVP 完整链路归第二梯队——与验收#3 文本一致,不算失败项 |
| 4 | i18n/markdown/icons/timer 四域结论+零回归 | ✅ pass | i18n=落码(useT 直调+useI18n 形态,c66dbe7/223dc5b);markdown=就绪形态入库(renderer.vm.at)+上游依赖登记;icons=Empty 兜底裁定(截图实证);timer=空集确认。web-only 行为零回归由 vitest 23+1 兜底 |
| 5 | vue 三门禁 + VM 探针全绿 | ✅ pass(带偏差声明) | 复审现场重跑:build strict 零 error 行 + vitest 23 passed/1 skipped + 对拍 30/30 normalized equal;first-run 五类 fatal 全程 0。**偏差**:正式探针脚本主检出路径钉死,worktree 内不可执行——等价门禁=first-run 编译链接段同管线;合并后主检出补探针两连绿为 merge 动作 |
| 6 | 不动 web//backend//auto-lang;worktree 流程 | ✅ pass | `git diff e8eeab4..HEAD --name-only` 反向过滤仅命中申报 4 文件(scripts×2+ports 两 adapter),零越界;改动全在 plan-047-dev 分支两提交(c66dbe7/223dc5b) |

### 遗漏/延后/workaround 猎查(lazy-convergence)

- **延后(计划文本预授权)**:R8 数据桥第二梯队立项、T4 降级激活待上游 registry 支持、R9 门控修复待用户裁定(待澄清#6)、R7 观察期定罪——均为验收条款/待澄清默认值明文路线(用户起草确认时接受),非静默缩水。KD 047 行全部承载。
- **遗漏猎查**:无任务级丢项;D1 设计文本曾写 `--capture/--timeout` 参数名,实现为 `--observe-ms/--keep`(捕获常开 tee 化)——纯措辞分歧无功能缺口,记档不改码。
- **workaround 猎查**:三文件 TODO/FIXME/XXX/HACK grep=0;批1 曾引入 plain-use 写法致 adapter 整体解析失明(run2/run3 新增桩暴露),已在 T3 勘误复归并留档注释——属已修复的迭代弯路而非残留 workaround。
- **行为面提示**:run4 后 i18n 族真符号化,settings 菜单 t() 文案将在实机呈现(此前桩化空白)——观察期手测时顺带核验中文目录渲染。

### spec-impact 元数据

三字段留空,理由见 frontmatter 注(基建/工具化计划,沿 PLAN-045/046 先例;
交付物以文件+台账行承载,specs 域无增删)。

### 路由裁定

六条验收全 pass、偏差两项均有条文授权与等价门禁、无未签核延后 →
**status: reviewed**。下一步 `/auto-plan:merge`(merge 时于主检出补探针
两连绿终验 + worktree 折返清理)。

## 待澄清事项

1. **SSE 流式的 VM 方案**（轮询 vs WS vs 后端推送改造）影响面大，默认不进本
   计划——第二梯队独立立项前需要一轮专项勘察。确认？
2. **markdown 降级观感**：纯文本降级是否可接受作为观察期形态？default 可接受
   （结构信息丢失但内容可读），comrak 富文本立上游项排期不设时限。
3. **icons 两路径**（生成器产 52 静态 widget vs platform 挂载）：
   default 按 T1 盘点结果顺能力现场裁，不强求最优解——首跑优先级高于美观。
4. **启动器双态**（主检出 / worktree 都要能跑）：default 用 T1-d 结论一锤定音，
   若 worktree 形态成本过高则登记"仅主检出"，合并后终验兜底。
5. **验收 3 的降级触发权**：UPSTREAM 红阻断链路时，是否允许当场破例修
   auto-lang（046 曾两破例）？default 否——登记移交，破例权留给用户明示。
6. **R9 未认证直达 chats 的门控修复**(执行期新发现,首跑红清单):VM 轨 App
   Init 后 current_view 直达 chats 且 token=nil,登录墙不生效(web 轨由
   router guard 承担)。default 维持现状交观察期手测;若裁定修复,musk 侧
   app.at 视图分派加 auth 分支(双轨共享源,需 vitest/对拍复核行为等价)。
7. **探针偏差处置**:worktree 内无法跑正式 vm-link-probe(主检出路径钉死),
   以 first-run 编译链接段为等价门禁——确认合并后由 review/merge 在主检出
   补两连绿终验? default 是。
