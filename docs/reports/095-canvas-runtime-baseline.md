# PLAN-095 Canvas 运行时基线核对报告（T-01）

- 计划：PLAN-095 r2（docs/plans/095-autoui-canvas-runtime-prerequisites.md）
- 阶段：work / T-01 冻结基线与最小接口
- 日期：2026-10-01
- 执行 worktree：`D:/autostack/.wt/musk-095/auto-musk`（分支 `plan-095-dev`）
- 依赖 worktree：`D:/autostack/.wt/musk-095/auto-lang`（分支 `auto-musk-095-dev`）
- 本报告所在仓提交基线：auto-musk main@22f06ba（worktree HEAD 同）
- 依赖仓基线：auto-lang master@e0fb4e4e4（worktree HEAD 同）

## 1. 版本冻结

| 对象 | 提交/状态 | 说明 |
|---|---|---|
| auto-musk main | 22f06ba | 093 已阶段合入（fc727dc 落 main），工作树干净 |
| 093 计划 | executing 8/14 | T-01～T-04 完成，T-06/T-07/T-10 进行中（快照时 1/14，已大幅前进，按 Q-01/Q-02 以当前态为准） |
| 093 worktree | `D:/autostack/.wt/musk-093/auto-musk` @fc727dc | **干净**（计划快照中的 manager/mod/tools WIP 已随 fc727dc 提交，无未提交代码） |
| 093 依赖分支 | auto-lang `auto-musk-dev`@2327e0bba | 领先 auto-lang master 7 提交（093 的 G-7/G-9/G-12 等 codegen 修复），**未合回**，仍由 093 持有；095 不复用、不改写 |
| auto-lang master | e0fb4e4e4 | 自计划快照 3b3014e3 前进了 708/711/712/713/716/717/718 等交付 |
| 707/708 | **均已交付归档**（docs/plans/archive/） | Q-05 的 708 串行约束解除：无在途 WIP，master 即含 708 渲染响应性交付；后写方基线=本 master |
| 本期 CLI | `D:/autostack/.wt/musk-095/auto-lang/target/debug/auto.exe` | sha256 `bb5f97b009cbfbd9c34de7601c9ae940d8e53c08546cb35ac74e8eb335042064`（dev 档 CARGO_PROFILE_DEV_DEBUG=0，版本串 `auto 0.1.0+v0.4.2-2423-ge0fb4e4e`） |

## 2. 缺口复验结论（当前 HEAD=e0fb4e4e4）

复现驱动：`auto-lang/tmp/t01-repro.mjs`（`auto run --render vm` + AutoUI MCP
autoui_snapshot/autoui_screenshot），fixture 于 `auto-lang/tmp/t01-g11-sibling/`
与 `auto-lang/tmp/t01-overlay-host/`。

| 缺口 | 计划快照判定 | 当前实测 | 处置 |
|---|---|---|---|
| G-1 ImageSurface loaded/error | 回调被丢弃、无运行时事件 | **仍缺**：renderer.rs:6399-6405 `let _ = (on_error, on_loaded, ...)` 显式丢弃；image_surface.rs（522 行）零回调引用；解码完成 `publish_ready`（image_pipeline.rs:891）只写缓存，无 UI 唤醒/无回调；seam 的 Loaded/Error 归一化（image_surface.rs:298-319）是死代码。可照抄先例：plan712 Video 上行 c3aa6d7be（video_uplink.rs 385 行五 handler 全链接线） | T-02 照原范围实施 |
| G-2 叠层宿主丢失 | absolute+z 致宿主内容消失 | **未复现（已解除）**：最小 fixture（base 两行+蓝/琥珀两 absolute 框）VM 树快照与真实截图均完整——base/框/框外尾部全部渲染，树中 Overlay 嵌套折叠正确（fold_floats b65245f13 起即保 base，3b3014e3..HEAD 无回退）。093 时代观察疑与 G-11 子件空渲染同源或已被上游修复 | T-03 转为验证+消费合同（穿透/DPI/contain/popover 不回归），无丢失修复项 |
| G-3 原生聚焦 | shim_dom_focus_first 空实现 | **仍缺**：native.rs:9054 pops+drops；musk `platform.vm.at:66` 打印 "focus composer skipped on vm target"；但上游已有可复用机制：`__focus_input` 状态约定→update_inner 消费→`iced::widget::operation::focus`+5 轮重试（renderer.rs:19615+）；text_input 稳定 Id=`auto_input_{widget}_{event}`（Plan 483 derive_input_id）；text_editor Id=placeholder 长度派生弱键（renderer.rs:4695+，需升级为主键） | T-04 按 §3 冻结接口实施 |
| G-6 MCP 合成坐标 | Double/Float 位型分叉 | **仍在**：合成通道 `Value::Double(x+0.001)`（renderer.rs:17383/17417/17424/17431）vs 真实通道 `Value::Float(x+0.001)`（aura_view_builder.rs:13253-13254）；payload encode/decode（"d"/"f"）保型直达 push_f64/push_f32（vm_bridge.rs:3252-3253）；float 形参位型错读风险与计划记录一致 | T-05 照原范围实施（Double→Float 编码统一） |
| G-11 widget→widget 子件 | 同文件兄弟疑触发 | **复现实锤且加重**：最小 fixture（G11Host 视图引同文件 G11Child、无 use 行）VM snapshot 中子件**零节点**（连 `<G11Child />` 占位文本也无，093 §8.16 的"列壳在、子件空"完全同型）。根因层=组件解析/registry 装配：lib.rs:4052-4061 根文件只取首个 WidgetDecl，同文件兄弟不注册；miss 路径静默（aura_view_builder.rs:1076-1080/1430-1434 占位降级，VM 渲染快照零输出） | T-07 照原范围实施（两形态等价+可定位诊断） |
| V01 strict 生成 | 两版 CLI 静默 abort | 见 §4 | 见 §4 |

## 3. 冻结的接口（Q-03 聚焦原语等）

### 3.1 原生聚焦（T-04，方案=计划 5.4 选项 b：平台中立原语）

- **原语**：`ui.focus(target_key)`（stdlib/auto/ui.at 新增；native 目录新增注册）。
  `target_key` = 目标输入控件的 .at 输入 handler 事件键（如 musk composer 的
  `.Input($event)`，mention_input.at:270）。
- **VM/iced 实现**：native 写定向聚焦请求状态（复用 `__focus_input` 消费接缝，
  renderer.rs:19615+）；消费时按 (widget,event) 主键经 derive_input_id 同款派生
  Id，在当前视图 collect_input_ids 集合中匹配；命中→`iced focus` 尾任务+有限重试
  （沿用 5 轮口径）；未命中/已卸载→可观察 miss 结果（结果状态+无操作），不猜测
  第一个控件。
- **text_editor 升级**：composer 形态（VM 轨 textarea→text_editor）的稳定 Id 从
  placeholder 长度弱键升级为 oninput handler 主键派生（PLAN-051 P2 的前提补齐），
  text_input 既有 Plan 483 主键不动。
- **web/Vue 映射**：`ui.focus(key)` 转 `document.querySelector('[data-auto-focus="<key>"]')?.focus()`；
  生成器在 textarea/text_input 输出面补 stamp `data-auto-focus`。musk web 端口
  （focus_composer.ts 逃生舱）保留不变。
- **兼容**：`dom.focus_first` 公共形态不动（web 真、desktop no-op）；musk 平台
  端口 `platformFocusComposer` 的 VM 臂改为调用新原语——该生产接线归 093 T-08，
  095 只交原语+消费示例+真实键盘输入收据。
- **验收口径**：text_input 与 text_editor 双形态真实键盘落字；目标缺失/卸载有
  结果；Tab/Escape 不回归。

### 3.2 ImageSurface 事件（T-02）

- 公共回调形态保持 `on_loaded(fn())`/`on_error(fn(str))`（stdlib/aura widgets
  display ImageSurface.at 现名）；通知内部携订阅代次+资产 revision；ready 后
  loaded 一次、失败 error 一次、替换/卸载旧消息过滤——语义按计划 5.2 原文执行。
- 接线参考面：plan712 Video 上行（c3aa6d7be）同构实现；解码完成唤醒接入
  plan711 帧泵订阅门（renderer.rs:22119/22129，现不含媒体条件）。

### 3.3 MCP 坐标（T-05）

- 统一为 Float 编码：renderer.rs 4 处 Double→Float（17383/17417/17424/17431），
  +0.001 分数化保留（nanbox 位型口径，dynamic.rs:2910 注释）；spec 字符串解析
  （mcp_server.rs:1723/1767）与 autoui_action 协议面不动。

### 3.4 G-11 子件实例化（T-07）

- 两形态等价：同文件兄弟 WidgetDecl 与跨文件 use 行引用均须在 VM 轨实例化；
  不可解析引用给可定位诊断（与 T-06 strict 诊断口径一致），不静默空渲染。
- 根因层冻结：lib.rs 根文件 WidgetDecl 装配选择 + aura_view_builder 组件臂
  miss 路径；renderer 不预断为责任层（最小 fixture 树快照即零节点，说明丢失
  在 AbstractView 构建期，早于 iced 渲染）。

## 4. V01 strict 生成复现（T-06 输入）

- 命令：`auto build --gen-only --strict`（worktree `D:/autostack/.wt/musk-095/auto-musk`@22f06ba，
  CLI hash 见 §1）。**结果：EXIT=0，"Generation complete (65 component(s))"**；
  S001 schema drift 仅 Info 非致命（title/size/source 条目与 093 冻结报告记录同型）。
- **归因：原 V01 静默 abort 已在当前基线解除**（auto-lang master e0fb4e4e4 期间
  的交付，具体解除提交未单独定位——两版旧 CLI（gc8f86ef/e2deb4f 构建）时代失败
  不再复现，旧→新证据即本节收据）。093 依赖分支上的 codegen 修复（G-7/G-12）
  属另一增量，非本门通过的必要条件（本 CLI 不含它们仍 EXIT=0）。
- T-06 剩余：V07 的 `pnpm install + pnpm build`（vue-tsc/vite 打包）与非法输入
  可定位诊断验证；本节 gen-only 通过不等于打包通过。

## 4.1 V02 image_surface_contract（T-01 要求）

- 命令：`cargo nextest run -p auto-lang --test image_surface_contract`（默认
  features 含 ui-iced）。**基线红已修**：断言标记停留在 61394be07 之前的生成
  形态（`objectFit: 'width'` 对象形态、裸 `@pan` 别名）——该提交有意把 :style
  改为 CSS 声明串（vue-tsc TS2345 修复，自带 031 工程 vue-tsc+vite 绿收据）、
  onpan 改 pointer 三相位包装；合同测试标记未跟上（vue.rs 内嵌测试已同步、
  本测试漏更）。修复=标记重锚到当前合法形态（`;object-fit:' + ('width')` +
  pointerdown/move/up 三绑定），语义合同不变，修复后 1/1 PASS。
- 该修复落在 auto-lang worktree（`crates/auto-lang/tests/image_surface_contract.rs`），
  作为 T-02 的 V02 门前置。

## 4.2 最小 V06（T-01 口径）

- 完整 V06 探针脚本归 T-08；T-01 以 `auto-lang/tmp/t01-repro.mjs` 驱动的两组
  真实 VM 运行（spawn `auto run --render vm` + AutoUI MCP）为最小运行态证据：
  快照/截图见 `docs/reports/095-evidence/t01/`。

## 5. 与在途工作的串行对账

- **708（renderer 响应性）**：已交付合入 master——本计划改动若触 renderer 订阅
  门/memo，基线即 master，无并发写入风险。
- **707（HTTP stream async）**：已交付；G-4/G-5 维持计划的保留口径（body_to_file
  路径），不进 HTTP 堆生命周期。
- **093 依赖分支（auto-musk-dev@2327e0bba，未合回）**：与本计划写入面重叠检查
  ——093 修复集中在 codegen/store composable（G-7/G-12）与 vm resume（G-9）；
  095 写入面为 image_surface 事件/Overlay 验证/聚焦原语/MCP 编码/组件装配。
  **文件级交集：无**（093 的 aura_view_builder 触碰为渲染 memo/登记面，095 不改
  fold_floats）。095 的 V07 生成门若命中 093 已修的生成 bug，按计划 T-06"新版
  已修复则交旧→新证据"处置，不重复实现；合回顺序留 merge 阶段按 AGENTS.md。
- **093 musk worktree**：干净、无并发写入风险；095 不触 093 在途 UI 文件。

## 6. 环境事实

- D: 盘满（100%）曾致链接失败（LLVM no space）；本 worktree 改用
  `CARGO_PROFILE_DEV_DEBUG=0` 重建通过；auto-down 只读兄弟检出
  （--detach）补齐 `../../../auto-down` 路径依赖解析（照 musk-093 组先例）。
- pac.at 裸 `front` 行在当前解析器下报 "Undefined variable: front"（093 探针
  资产的历史写法）；`auto run` 以 pac.at back_port 覆写 AUTO_HTTP_PORT——095
  探针/fixture 一律不含端口行，端口全走 env。

## 7. 与 093 消费任务的对账（交付面版本）

| 093 消费任务 | 本计划交付（接口冻结见 §3） | 当前基线状态 |
|---|---|---|
| 093 T-03/T-04（媒体帧） | 真实 loaded/error + 代次过滤（T-02） | G-1 未修，T-02 实施；body_to_file 路径保持 |
| 093 T-05/T-06（叠层+strict 门） | 叠层验证合同（T-03）；strict 打包验证（T-06） | 宿主丢失已解除（§2 G-2）；gen-only EXIT=0（§4） |
| 093 T-06（G-11 消费面） | widget→widget 两形态等价+诊断（T-07） | 已复现（§2 G-11），093 §8.16 挂起的 VM 消费面待 T-07 解除后补跑 |
| 093 T-08（composer 聚焦） | ui.focus 原语+真实键盘收据（T-04） | 接口冻结（§3.1）；生产端口接线归 093 |
| 093 T-11/T-13（坐标验收） | float 合成坐标统一（T-05） | 分叉确认（§2 G-6） |
