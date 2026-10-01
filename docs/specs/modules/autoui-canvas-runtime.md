# autoui-canvas-runtime — Canvas 运行时消费合同

> 来源：PLAN-095 r2（review pass @auto-musk cc9886e / auto-lang e2da944e3，
> 2026-10-01）。本文定义应用层消费 AutoUI Canvas 运行时能力的事件、聚焦、
> 坐标与子件组合合同；平台行为的权威规则在 auto-lang 仓
> `docs/specs/auto-lang/ui/design/`（image-surface-events /
> programmatic-focus / widget-composition / mcp-pointer-input /
> overlay-interaction），此处只定消费边界，不复制实现。

## 1. ImageSurface 事件消费（AC-02/03）

- 事件声明：`onload: .Msg` / `onerror: .Msg(str)`。VM 轨由媒体管线终态
  驱动：资产解码 Ready 且渲染帧已消费 rendition 后派发一次 loaded
  （实参按 handler 声明数投影：0 参 / [rev] / [w,h] / [w,h,rev]）；坏图、
  缺票、过期等终态失败派发一次 error，`fn(str)` 声明按实参收可定位原因串。
- 消费方义务：核对自己预览的 src 版本（URI 内嵌票据 revision）；
  同一 src 重绘/轮询不重复通知；换 src / 卸载后旧订阅失效，重挂载视为
  新订阅会再次通知。
- 等待期间不假装成功：Pending 不派发 loaded。

## 2. 叠层与被动框穿透（AC-04）

- absolute+z 子节点 hoist 为 Overlay 叠层，宿主内容恒保留。
- 样式含 `pointer-events:none`（CSS 声明串或 Tailwind `pointer-events-none`
  均可）的浮层为**被动框**：不截获下方点选（不包 opaque 捕获）。
  选择框（蓝）/Agent 框（琥珀）一律声明此类。
- 未声明的浮层保持 capture（内容矩形内拦截）——交互浮层/popover 依赖
  此语义，不得一刀切穿透。
- 百分比定位/尺寸相对宿主内容矩形；**% 偏移须与 % 尺寸配对使用**
  （自然尺寸浮层用 px 或 % 尺寸，见已知债）。

## 3. 程序化聚焦（AC-05）

- 原语：`ui.focus(target_key)`；target_key = 目标输入控件的 .at 输入
  handler 事件键（`.Input` / `.Input($event)` / `Widget.event` 限定形）。
- 结果可观察：renderer 写 `__focus_result` 状态（`ok` / `miss:<key>`）。
  视图中已存在其他可聚焦输入而目标缺席 = 立即 miss；视图尚无输入
  （挂载竞态）时 5 轮重试后 miss。不做 no-op 降级。
- web 轨 `ui.focus(sel)` = `document.querySelector(sel)?.focus()`
  （与 dom.focus_first 同义）；musk web 端口既有 TS 逃生舱可保留。
- 覆盖面：text_input 与 textarea（text_editor）双形态，Id 主键
  `auto_input_{widget}_{event}` / `textarea_{widget}_{event}`。

## 4. 合成指针坐标（AC-06）

- `autoui_action drag/pen` 的坐标与真实指针通道**同一编码**
  （Float(x+1e-3)）；验收用类型化 float 状态断言（容差 ≤0.5px），
  不接受字符串拼接读数（.at float→str 拼接有独立位型缺陷，见已知债），
  不接受位型错读的 int。
- 动作顺序 Down→Move×n→Up 与真实路径一致；down 实参为 `$event` 标记
  （handler 声明 1 参），up 无实参（0 参）。

## 5. widget→widget 子件组合（AC-11）

- 同文件兄弟 widget 与跨文件 `use module: Widget` 两形态在 VM 轨等价
  实例化（子件子树完整、宿主壳不丢）。
- 未解析的子件引用产生 `AURA-CHILD-MISS` 定位诊断（stderr），不静默
  空渲染；处置 = 同文件声明或补 use 行。

## 6. 生成门（AC-07）

- `auto build --gen-only --strict` 为消费工程的生成门；非法输入非零退出
  且带可定位诊断（如未知组件 → "SFC 未编译落盘"）。S001 schema drift
  Info 维持 Info；升级或删除需逐条判断合法性。
- 收据必须绑定实际 CLI 文件 hash；不以 lenient / 旧产物 / 跳过打包判绿。

## 已知债（2026-10-01 登记）

1. `.at` float→字符串拼接位型误渲染（typed 值正确、str 拼接垃圾）——
   VM 语言面独立缺陷。
2. AURA 快照 raw_class 表在 hoist 场景样式错配（运行时正确、仪表错配）。
3. OS 键盘投递在受限会话被前台锁拒绝（点击位置路由不受影响）。
4. Vue 臂浏览器 live 实拍基建复用 093 live.mjs，未在 095 内执行。
5. % 偏移需配对 % 尺寸（浮层几何 v1 边界）。
6. 多 surface 共享同一 handler 键的通知合并（v1 单通知）。
