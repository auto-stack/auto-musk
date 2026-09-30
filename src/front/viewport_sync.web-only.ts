// viewport_sync.web-only.ts — PLAN-093 T-05: 视口宽同步（web 侧宽度通道）。
//
// store 的宽度分层布局（§5.2）需要视口宽进入 .at 状态：web 轨经本模块把
// window.innerWidth 写入 localStorage('app.viewport_w')（首装 + resize 各
// 一次；resize 有 rAF 节流）。VM 轨由 iced renderer 发布
// vm.window_inner_width（plan-046-B 桥），store 两个键都读、web 键优先。

function writeViewport(): void {
  try {
    const w = window.innerWidth;
    if (w > 0) localStorage.setItem('app.viewport_w', String(w));
  } catch {
    // 隐私模式等 localStorage 不可用面：宽度分层退默认档，不致命。
  }
}

export function syncViewport(): void {
  writeViewport();
  let scheduled = false;
  window.addEventListener('resize', () => {
    if (scheduled) return;
    scheduled = true;
    requestAnimationFrame(() => {
      scheduled = false;
      writeViewport();
    });
  });
}
