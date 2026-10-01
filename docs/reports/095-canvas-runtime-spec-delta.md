# PLAN-095 Canvas 运行时规范增量提案（T-09）

- 计划：PLAN-095 r2；阶段 work → review 交接
- 日期：2026-10-01
- 绑定：auto-lang auto-musk-095-dev@（T-02 fce48632e / T-03 cba6516d3 /
  T-04 0cdf4b548 / T-05 919ade13a / T-07 e2da944e3）；auto-musk plan-095-dev
  @（T-01 b2d39ad / T-03 0391c20 / T-04 98d67d2 / T-05 8ac49a3 / T-07
  ab1fb4b / T-08 本报告同 commit）；CLI 89c22af15c048019。
- 性质：**提案**——canonical Spec / ledger 落地由 merge 按对应仓流程执行，
  work 阶段不改两仓权威规范（计划 §5.8 纪律）。

## SD-01 add — auto-musk docs/specs/modules/autoui-canvas-runtime.md（新模块）

Canvas 消费能力/事件/聚焦合同（消费边界，引用上游权威不复制实现）：

1. ImageSurface 事件合同：`onload: .Msg` / `onerror: .Msg(str)` 在 VM 轨
   经媒体管线终态驱动（解码 Ready 下一帧派发 loaded、终态失败派发 error
   带原因串），单次/订阅代次过滤；消费方核对自身 src 版本（AC-02/03）。
2. 叠层合同：absolute+z hoist 保宿主；`pointer-events:none`（CSS 声明串或
   Tailwind 形）= 被动浮层穿透；交互浮层默认 capture（AC-04）。
3. 聚焦原语：`ui.focus(target_key)`——target_key=输入控件 .at 输入
   handler 事件键；结果经 `__focus_result`（ok / miss:<key>）可观察；
   web 轨 = dom.focus_first 同义（AC-05）。
4. 合成坐标：`autoui_action drag/pen` 坐标与真实指针同为 Float(x+1e-3)
   编码；验收用 typed float 状态断言（AC-06）。
5. 子件组合：同文件兄弟与跨文件 use 两形态在 VM 轨等价实例化；未解析
   引用有 AURA-CHILD-MISS 定位诊断（AC-11）。

## SD-02 modify — auto-musk docs/specs/modules/app-canvas.md、ui-parity.md

标注已交付的运行时依赖与可信验收机制：媒体事件/叠层穿透/聚焦/坐标/
子件实例化的验收锚（V06 探针 + 收据路径）；093 未交付业务范围（工作台
UX、Canvas 身份/归属）保留其状态不混淆。

## SD-03 add/modify — auto-lang docs/specs/auto-lang/ui/design/

- **image-surface-events.md（新）**：管线代次计数 + 桌面级 50ms 唤醒轮询
  （订阅时门看不到 ms 级窗口的根因）+ update 尾部 sweep（ready 两拍、
  单次门、卸载退订、实参按声明数 0..=3 投影、576-D4 对齐、不在 paint
  重入）。
- **programmatic-focus.md（新）**：auto.ui.focus(9920) 请求槽 → renderer
  消费 → iced focus 任务；结果状态可观察；三处接线缺一即静默 no-op
  （NATIVE_ID_ENTRIES 白名单行实测记录）。
- **widget-composition.md（新）**：同文件兄弟注册（根提取不 break）+ use
  两形态等价 + miss 诊断口径。
- **overlay-interaction.md（改）**：pointer-events-none 语义、百分比浮层
  几何（三段 FillPortion 装配；iced 0.14 无 Relative 的替代）、% 偏移需
  配对 % 尺寸边界。
- **ui/overview.md（改）**：样式系统 CSS 声明串支持（musk canvas 框生产
  形态）、StyleClass 新类（PointerEventsNone/Percent 族）、Color::from_css。

## SD-04 add/modify — auto-lang docs/specs/auto-lang/ui/design/mcp-pointer-input.md（新）、ui/overview.md（改）

合成指针坐标编码 = Float(x+1e-3)（与真实通道同一编码/解码口径）；
Double 编码位型错读实证（垃圾 int）；来源/精度验收 = typed float 状态
断言口径；AutoVM toolset（源码 server）不变。

## SD-05 modify — auto-lang docs/specs/auto-man/project.md、auto-cli/project.md

strict 生成门现状：原静默 abort 已在 3b3014e3→e0fb4e4e 区间解除（旧→新
收据）；非法输入诊断口径（未解析组件 → 非零退出 + "SFC 未编译落盘" 可
定位信息）；CLI 版本消费规则（收据绑实际文件 hash——本计划 CLI
89c22af15c048019，主二进制锁定/部署观察项维持既有规则）。

## 已知债与边界（随增量一并登记）

1. `.at` 浮点→字符串拼接位型误渲染（T-05 实证：typed 值正确、str 拼接
   垃圾）——VM 语言面独立缺陷，建议独立计划。
2. AURA 快照 raw_class 表在 hoist 场景样式错配（T-03 实证：运行时正确、
   仪表错配）——快照仪表层缺陷。
3. OS 键盘投递前台锁（本会话）——真键盘落字收据的有头环境补拍。
4. Vue 臂浏览器实拍（live）——基建复用 093 live.mjs，独立会话执行。
5. % 偏移需配对 % 尺寸（浮层几何 v1 边界）。
6. 多 surface 共享 handler 键的通知合并（v1 登记边界）。
