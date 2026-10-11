// canvas_embed_web.ts — PLAN-102 T-04：embedded 呈现的 web 消费面。
//
// 职责（web 平台门面——install 家族口径；VM 轨空桩走 img 降级）：
//   ① 轮询 .cv-embed-frame 宿主 div（面板挂卸鲁棒）+ canvas.bridge_url 键
//     （store 回填变化落键）——两者就绪即建连。
//   ② 连接 = @auto/drawlist-renderer connect()（Hello 订阅/FrameReady→
//     renderFrame Canvas2D/断线 30s 预算重连）。
//   ③ operate 输入回传：DOM 指针事件（坐标 = 帧逻辑域×缩放比换算）+
//      键盘字符；canvas.input_mode 键 == "inspect" 时只观察不发（检查态
//      走既有 pick 通道——AC-03 分工）。
//   ④ 就绪标记 canvas.embed_ready = "1"（store 据此切帧区分支）。
// 会话身份：桥 url 变化（代次切换/停止）→ 断旧连建新连；旧连接迟到帧
// 不入新画布（连接对象整体替换）。

import {
  connect,
  renderFrame,
  type RemoteConnection,
} from '@auto/drawlist-renderer';

interface EmbedState {
  host: HTMLDivElement | null;
  canvas: HTMLCanvasElement | null;
  ctx: CanvasRenderingContext2D | null;
  conn: RemoteConnection | null;
  url: string;
  logical: { w: number; h: number };
}

function readKey(k: string): string {
  try { return localStorage.getItem(k) ?? ''; } catch { return ''; }
}

function inputMode(): string {
  return readKey('canvas.input_mode') || 'operate';
}

/** 帧逻辑尺寸（Welcome 载荷；缺省 480x680 = canvas 目标窗基线）。 */
function frameScale(st: EmbedState): { sx: number; sy: number } {
  const cw = st.canvas?.clientWidth ?? 0;
  const ch = st.canvas?.clientHeight ?? 0;
  if (cw <= 0 || ch <= 0 || st.logical.w <= 0 || st.logical.h <= 0) return { sx: 1, sy: 1 };
  return { sx: st.logical.w / cw, sy: st.logical.h / ch };
}

function toLogical(st: EmbedState, ev: { clientX: number; clientY: number }): { x: number; y: number } {
  const r = st.canvas?.getBoundingClientRect();
  if (!r) return { x: 0, y: 0 };
  const { sx, sy } = frameScale(st);
  return {
    x: Math.round((ev.clientX - r.left) * sx),
    y: Math.round((ev.clientY - r.top) * sy),
  };
}

function attachInput(st: EmbedState): void {
  const c = st.canvas;
  if (!c || (c as unknown as { __cvEmbedInput?: boolean }).__cvEmbedInput) return;
  (c as unknown as { __cvEmbedInput?: boolean }).__cvEmbedInput = true;
  c.tabIndex = 0;
  c.style.outline = 'none';
  c.addEventListener('pointerdown', (ev) => {
    c.focus();
    if (inputMode() !== 'operate' || !st.conn) return;
    const p = toLogical(st, ev);
    st.conn.sendPointerDown(p.x, p.y);
  });
  c.addEventListener('keydown', (ev) => {
    if (inputMode() !== 'operate' || !st.conn) return;
    if (ev.key.length === 1 && !ev.ctrlKey && !ev.metaKey && !ev.altKey) {
      st.conn.sendChar(ev.key);
      ev.preventDefault();
    }
  });
}

function dial(st: EmbedState, url: string): void {
  if (st.conn) { try { st.conn.close(); } catch { /* best effort */ } st.conn = null; }
  st.url = url;
  st.conn = connect(
    { url, appName: 'musk-canvas-embed' },
    {
      onWelcome: (w) => {
        st.logical = { w: w.rect?.w ?? 480, h: w.rect?.h ?? 680 };
      },
      onFrame: (f) => {
        if (!st.ctx) return;
        renderFrame(st.ctx, f.payload);
      },
      onDead: () => { st.conn = null; },
    },
  );
}

export function installCanvasEmbed(): void {
  if (typeof window === 'undefined') return;
  const w = window as unknown as { __muskCanvasEmbed?: boolean };
  if (w.__muskCanvasEmbed) return;
  w.__muskCanvasEmbed = true;
  const st: EmbedState = { host: null, canvas: null, ctx: null, conn: null, url: '', logical: { w: 0, h: 0 } };
  let readyMarked = false;

  const measure = () => {
    const host = document.querySelector('.cv-embed-frame') as HTMLDivElement | null;
    if (!host) return;
    if (st.host !== host) {
      st.host = host;
      st.canvas = document.createElement('canvas');
      st.canvas.style.width = '100%';
      st.canvas.style.height = '100%';
      st.canvas.style.display = 'block';
      host.replaceChildren(st.canvas);
      st.ctx = st.canvas.getContext('2d');
      attachInput(st);
    }
    // 尺寸随宿主（DPR 感知——逻辑坐标换算在事件侧）。
    if (st.canvas && st.host) {
      const dpr = window.devicePixelRatio || 1;
      const cw = Math.max(1, Math.round(st.host.clientWidth * dpr));
      const chh = Math.max(1, Math.round(st.host.clientHeight * dpr));
      if (st.canvas.width !== cw || st.canvas.height !== chh) {
        st.canvas.width = cw;
        st.canvas.height = chh;
      }
    }
    if (!readyMarked && st.ctx) {
      readyMarked = true;
      try { localStorage.setItem('canvas.embed_ready', '1'); } catch { /* 不致命 */ }
    }
    // 桥端点变化 → 重连（旧连接迟到帧不污染新代次）。
    const url = readKey('canvas.bridge_url');
    if (url && url !== st.url) dial(st, url);
    if (!url && st.conn) { try { st.conn.close(); } catch { /* best effort */ } st.conn = null; st.url = ''; }
  };
  setInterval(measure, 500);
  setTimeout(measure, 0);
}
