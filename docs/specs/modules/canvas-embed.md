# canvas-embed — canvas 应用画面的宿主内嵌呈现契约

> 来源：PLAN-097 架构定案 + PLAN-099 T-05 + **PLAN-102 全面重定**（默认
> 内嵌 = remote-frame 呈现链；修订 v2：window 显式 501，双向支持归 v0.6
> VM 原生内嵌）。证据：PLAN-102 四轨矩阵 ALL PASS + vm-cycle 双臂 17/17
> + 实机视觉复验（`tmp/ui-parity/PLAN-102/` 收据族）。

## 背景：呈现架构（PLAN-102 后）

iced 无 re-parent，OS 级窗内嵌不可行（约束保留）。呈现链（替换 097/099
的"原生窗帧源 + 面板 img 镜像"）：

- **会话 spawn**：musk 后端 spawn rq bridge（`auto rqhost --pipe
  <name> --bridge-ws <port> --bridge-token <t>`，D-1）+ 目标 app
  remote 渲染（`AUTO_VM_RENDER=remote`，headless——无可见窗，census=0）。
- **数据面**：前端直连 bridge WS（status 携 `bridge_ws_url` 含 token）；
  订阅名 = URL `&app=<app_name>` 织参（session 尾随 bridge stderr 解析
  `adopted \`NAME\``——Hello app_name 精确匹配，缺参永不命中帧区恒占位）。
- **消费面双轨同源**：`.at` `remote-frame (url:, class:)` 元件
  （PLAN-760 J2）——VM 轨 = 订阅-渲染-输入一体（remote_frame_registry），
  web 轨 = `data-remote-frame` div 占位 + `installCanvasEmbed`
  （drawlist-renderer canvas 注入；安装即写 `embed_ready` 键破鸡生蛋）。
- **重放保真**：DisplayList v2 宿主侧重放（文本 run 宿主重整形）；画师
  字体钉 app 字体栈 Inter family（F-RF-3：generic family + 非常规字重
  在 cosmic 系统匹配下乱选字面 → 碎片化乱码；w=400 恰整值
  `Font::DEFAULT` 被 default_font 替换兜住，500 不会）。
- **输入换算**：contain-fit 逆映射（F-RF-1：scale=min(lw/fw,lh/fh) +
  居中偏移；拉伸映射在宽高比不等时输入错位）。

## 契约

1. **默认 embedded**：app 首帧直接在 Musk 内容区出现，无额外可见窗。
2. **window 模式诚实 501**（修订 v2）：`POST /api/canvas/present
   {"mode":"window"}` → 501（上游 rq 客户端 exit-on-EOF 重连 seam 未
   就绪——不冒装、不动会话）；embedded 幂等；stale generation 409；
   非法 mode 400。axum + vmhost 双面同律。
3. **会话身份**：呈现切换/重连不重启 app 进程、不换 generation；
   `generation_id` 变化 → StatusBackfill 统一清场（picked/overlay/tree/
   帧面/嵌入消费面重订阅）。
4. **帧快照端点保留**：`GET /api/canvas/frame`（rq 会话 watchdog 截图
   面，seq/代次守恒 409/503 语义不变）——非 embedded 降级路径与探针
   断言面。
5. **mcp_base 出 status**：spawn 登记目标 app MCP 基址（typed 状态
   断言直连；无会话空串）。

## 关联实现

- auto-musk：`src/front/canvas_panel.at`（remote-frame 嵌入）、
  `src/front/canvas_store.at`（presentation 回填+变化落键）、
  `src/front/canvas_embed_web.ts`（web 消费面）、
  `backend/crates/musk/src/canvas/{session,manager}.rs`（bridge 双子
  进程 + adopted 名解析 + `&app=` 织参 + mcp_base）。
- auto-lang：`remote-frame` 元件（PLAN-760）、
  `crates/auto-lang/src/ui/iced/{remote_frame_registry,broker_surface}.rs`
  （F-RF-1 contain 逆映射 `92067662c`、F-RF-3 Inter family `6da3e6201`
  + AUTO_RF_TEXTDUMP 双端载荷转储仪器）。

## 已知边界（v0.6 方向，用户 2026-10-11 裁决）

- 本版（v0.5）仅内嵌；内嵌/独立窗口双向支持归 v0.6，架构定为 **VM
  原生内嵌**（auto-os ui-gallery `AppViewport` 同进程组件形态）——本
  remote-frame 链为迁移后的退役候选。前置上游能力：任意 app 路径组件
  运行时装载（ui-gallery 为构建期静态导入固定集）+ DevTools
  vtree/pick/state 改从宿主渲染树取（现依赖目标 app 独立 MCP）。
- VM 轨 light 主题 = 上游 iced 渲染器 dark-only（`theme_fn` 恒
  `Theme::Dark`；`dom.set_dark` VM 臂仅写 storage）——双主题证据面在
  web 轨。
- 帧区输入 invalidation 由宿主面板轮询/消息泵驱动（无独立 redraw
  通道；F-04-3 证伪：count 变化静置 ≤2.5s 即呈现）。
