# PLAN-088 T-02《锚点契约记录》

日期：2026-09-23。基线：auto-lang@8fecfcf69（musk-088 组兄弟位 release 构建）+
musk plan-088-dev（841fdbf）。方法：自足 MCP 探针 `tmp/t02/anchor-probe.mjs`
（spawn `auto run --render=vm` + 直连 /mcp；孤儿进程零遗留），fixtures =
counter（087 模板同源）+ loop-list（自写最小 for 列表，单文件）。

## 实测结论（按计划 T-02 三问）

### ① 帧像素坐标 ↔ vtree bbox

- **1:1 假设按字面不成立**：vtree bbox = 窗口**逻辑**像素；帧 PNG = 逻辑 ×
  **DPI scale**（本机 200%：固定窗 480×680 → PNG 960×1360，精确 2.0）。
- **修正契约**：scale = `frame_png_width / AUTO_VM_WINDOW 逻辑宽(480)`；
  pick 入参=帧 PNG 像素（前端只做 显示→自然 换算），musk 后端做 PNG→逻辑
  （除 scale）。musk session 恒注入 AUTO_VM_WINDOW=480x680（M1 契约），scale
  单源在 musk。
- **固定窗前提**：pac `window:"fit"` 的 app 在 fit 重排后 bounds 陈旧（实测
  counter fit 窗 bbox 停留在 (0,0,161.6,108.8) 内容空间 vs PNG 400×400 窗）。
  **M2 锚定契约 = 固定窗 app**（模板池已用固定窗口径交付）；fit app 的 pick
  降级尽力，登记 §10。

### ② vtree source 锚点（缺口实锤 → 升级兜底路径）

- **静态读 + 运行时双证**：`autoui_vtree` Atom 输出**无 source prop**——
  `ComputedNodeLite.source` 在 VM live 链无写入者（全仓唯一写点=
  headless_adapter 测试件）；`VNode.source_span`（字节偏移）确已随帧填充
  （select_rect 信封 span 可证：counter [77,312]、loop [200,355]，与 app.at
  实际字节区间吻合），但 **Atom 序列化器不输出它**。
- **for_iter 同缺口**：probe 有 `record_for`，但 renderer 两处合并站
  （view 同步块 21718 / wrap_debug 合并 22190）都只合 raw_class+events →
  Atom/AURA 均无 for_iter。loop fixture 实测 27 vnode 循环展开✓、实例 action
  payload✓（"Delta task"），唯实例标注不可得。
- **按钮/文本叶件无实测 bbox**：MCP-only 轨（无 F12）bounds 探针容器不包裹
  （Plan 402 §13.10 定制，防吃点击）——实测 button 仅 0×0 box、无顶层 bbox；
  PLAN-080 live 链同款已知（vmRectOf 取最近 @rect 祖先代理）。容器族
  （col/row）bounds 正常。
- **静态首拍无 bounds**：needs_bounds 仅 dirty×capture_debug 帧请求
  （PLAN-650 E-2），MCP 激活前的引导帧不采 → 静止 app 永无 bounds（实测
  counter 静置两拍 false；loop 因 fit 重排偶发 true——时序不可依赖）。
- **最小填充集（§4 预授权兜底，auto-lang 改动三件）**：
  1. `ui/vtree_atom.rs`：include_source 时输出 `span: {offset, len}` prop
     （vnode.source_span 现成数据，~5 行）——行号换算 musk 侧做（读 .at 文件
     自扫行首表；多文件按 span 落点+kind 关键词启发定文件，M2 主路径单文件）。
  2. `ui/iced/renderer.rs`：view 同步块 + wrap_debug 合并站把 probe ForIter
     并入 computed（~8 行；两站都需——否则 bounds 回路 from_live 覆盖时
     for_context 得而复失）。
  3. `ui/session.rs` VM DevToolsState::new：`debug_mode` 增 env 门控
     `AUTO_DEBUG_CAPTURE=1`（~1 行）——canvas 会话注入后：引导帧即采 bounds
     （G4）+ 叶件 bounds 容器包裹生效（G3）。风险面评估：M2 画布无真鼠点击
     （用户点击仅选区不驱动）、无 hover、devtools_open 不置位（面板不出镜）、
     MCP action 走 handler 注入通道不经 widget 点击路径——Plan 402 的回归面
     （真鼠点击×200ms 心跳重建）不适用；登记 KNOWN-DEBT 观察项。
- `autoui_select_rect`：span 信封健全，保留为 pick 正确性交叉验证通道
  （T-08 用），不作运行时主路径（中心命中+顶层修剪 ≠ 点选语义，实测全窗
  rect 只返 1 顶层节点）。

### ③ json 信封可得性

- `autoui_vtree` **无 json 格式**（Atom 文本单臂）→ anchor.rs 走 Atom 解析
  （原计划预设臂）。实测格式两处与假设偏差，解析器已按实修正：
  节点体 prop 分隔=`; `，**对象值内分隔=`,`**（auto-val print_object）；
  vnode id = **路径哈希大整数**（非顺序号）；字符串裸引号无转义。
- AURA snapshot（@rect+[for] 标注）与 autoui_inspect 的 source 描述与实现
  不符（均无 span）——不在消费面。

## 证据文件（本目录）

- `fixture-counter-{frame.png, vtree.atom, vtree-after-action.atom,
  select.json, state.txt, report.json}`
- `fixture-loop-{...同上...}`
- 探针：worktree `tmp/t02/anchor-probe.mjs`（入库前评估；fixtures 同目录）。

## 对执行步骤的影响

- T-03 anchor.rs：按修正契约实现（scale 换算在 musk manager/frame 侧；
  Atom 解析器双分隔符；哈希 vnode id 全程透传不假设顺序）。
- T-04/T-06 工具面不变。T-05 overlay 失效判定沿用"索引重建保留交集"。
- 新增 musk 侧任务：canvas 会话 spawn 注入 `AUTO_DEBUG_CAPTURE=1`；
  模板/fixture 固定窗口径校验。
- 087 模板池缺陷顺手发现：COUNTER_PAC/HELLO_PAC 内嵌 `#` 注释行 pac 解析
  报错（`Unknown section`）——087 生成指导里的模板文本带 `#` 注释，用户照抄
  即炸。M2 内顺手修（`//` 注释）。
