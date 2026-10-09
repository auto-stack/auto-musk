# canvas-embed — canvas 应用画面的宿主内嵌呈现契约（PLAN-099 新组件）

> 来源：PLAN-097 架构定案（e5fe941：原生窗=帧渲染源，面板镜像=内嵌呈现
> 面）+ PLAN-099 T-05 落地（auto-lang 6afac0405 渲染器相对 URL 解析）。
> 证据：preview-live-01/02 双场景实机 + e2e phase-A（.demo/musk-099/）。

## 背景：镜像架构（不可真内嵌）

iced 无 re-parent，OS 级窗内嵌不可行。既定架构（PLAN-097）：

- **画布原生窗**：app 的帧渲染源（VM 壳 spawn 的原生 iced 窗，确定性停靠
  `MUSK_CANVAS_WINDOW_POS=<app_name>=x,y`）。
- **宿主面板（CanvasPanel）**：内嵌呈现面——结构树镜像（status 轮询）+
  帧区（`/api/canvas/frame` 快照消费）。

## 契约

1. **帧区接线**：CanvasPanel 帧区 `img src = /api/canvas/frame?t=<seq>&gen=<gen>`
   （`canvasFrameUrl`，身份+seq 双键）。seq 变化（app 画面演进）→ 新 URL →
   下一拍轮询换入新帧；无帧/停止 → 占位状态语。
2. **VM 轨相对 URL 解析（PLAN-099）**：VM 壳无页面源，渲染器
   `load_image_bytes` 对 `/` 开头相对 URL 按 `AUTO_HTTP_BASE`/`AUTO_HTTP_PORT`
   展开为绝对地址（auto-lang `resolve_http_base_url` 单源，与 VM 轨
   `Http.get` 同词汇）。web 轨浏览器 origin 原生解析，行为不变。此前
   相对路径落本地文件臂恒失败 = 面板帧区恒 `?`（preview-live-02 实测）。
3. **帧快照端点**：`GET /api/canvas/frame?workspace=&t=&gen=` PNG 直出；
   代次/seq 不符 409（携带当前值，客户端重对账）；无帧 503。
4. **会话重绑（AC-07）**：canvas 会话单例（start=替换）。代次变化
   （`generation_id` != 本地）→ StatusBackfill 统一清场（picked/overlay/
   tree/帧 URL/帧尺寸）→ 下一拍重拉新代次帧；app 切换（stop→start 或
   start 新 app）→ 新 gen/新 seq → 面板帧与原生窗同切，无上一会话残留。
5. **渲染线程纪律**：VM 帧加载走渲染器负缓存+本地文件 1s 重试语义
   （PLAN-099 前既有）；HTTP 帧源（本契约展开后）每 URL 至多一次同步抓取
   （3s 超时，os-007 既有契约），帧 URL 含 seq 天然去重。

## 关联实现

- auto-musk：`src/front/canvas_panel.at`（帧区 img + mouse-area 坐标换算）、
  `src/front/canvas_store.at`（StatusBackfill 代次对账清场 + seq→URL 换入）、
  `src/front/canvas_helpers.at`（canvasFrameUrl）、
  `backend/crates/musk/src/canvas/mod.rs`（四路由 + source 只读）。
- auto-lang：`crates/auto-lang/src/ui/iced/renderer.rs`（load_image_bytes
  相对 URL 解析，6afac0405）、`crates/auto-lang/src/vm/ffi/stdlib.rs`
  （resolve_http_base_url，pub(crate) 单源）。
- 原生窗停靠：e5fe941（MUSK_CANVAS_WINDOW_POS per-app_name 映射，VM 壳
  守护进程按 Hello app_name 匹配）。

## 已知边界

- 帧区刷新粒度 = seq 版本；app 内高频动画的中间帧不逐帧上镜（快照语义，
  非视频流）。演示形态（表单/看板类 app）seq 粒度足够。
- 原生窗"演示态最小化"未做（停靠位已不遮宿主；收起交互归后续）。
- SSE 帧流（替代轮询快照）未立项——500ms status 轮询 + seq 去重已满足
  演示刷新观感。
