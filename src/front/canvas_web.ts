// canvas_web.ts — PLAN-087 T-07: 画布会话 web 侧调用（启动/停止）。
//
// 不走 api.at 生成绑定（手写路由，无绑定源）；结果对象而非 throw——.at 的
// catch 值 unknown（TS2571 实证），错误文案走结果对象（whitelist_web.ts
// 同口径）。状态/帧轮询不经此处：store 侧 Http.get_msg 直连（forge_store
// PollBackfill 同款），启动后随 1s 轮询自然反映状态。
// workspace 显式携带（start 需要它做沙箱判定；拦截器对 /api/canvas/* 注入
// 的 workspace 查询与显式 query 同名等价，显式传参以防拦截器缺位）。

// PLAN-093 T-06:点选换算单源——canvas_helpers.at 经 use.web.fn 生成 ext
// TS 模块（forge_helpers 同链），点击委托与 VM 消费同一映射规则。
import { canvasMapPhysical } from './canvas_helpers';

export interface CanvasResult {
    ok: boolean;
    error: string;
    state?: string;
    app_path?: string;
}

async function canvasFetch(method: string, path: string, body?: object): Promise<CanvasResult> {
    const response = await fetch(path, {
        method,
        headers: { 'Content-Type': 'application/json' },
        body: body === undefined ? undefined : JSON.stringify(body),
    });
    const data: { state?: unknown; app_path?: unknown; error?: unknown } | null = await response
        .json()
        .catch(() => null);
    if (!response.ok) {
        const msg =
            data && typeof data.error === 'string' ? data.error : `HTTP ${response.status}`;
        return { ok: false, error: msg };
    }
    return {
        ok: true,
        error: '',
        state: data && typeof data.state === 'string' ? data.state : undefined,
        app_path: data && typeof data.app_path === 'string' ? data.app_path : undefined,
    };
}

export async function canvasStart(workspace: string, appPath: string): Promise<CanvasResult> {
    return canvasFetch(
        'POST',
        `/api/canvas/start?workspace=${encodeURIComponent(workspace)}`,
        { app_path: appPath },
    );
}

export async function canvasStop(): Promise<CanvasResult> {
    return canvasFetch('POST', '/api/canvas/stop');
}

// ── PLAN-088 T-07：点选/锚定 web 侧 ──────────────────────────────────────────

export interface CanvasPickResult {
    ok: boolean;
    error: string;
    /** pick 锚点对象 JSON 文本（同 status.picked 形态；供 PickBackfill）。 */
    pick?: string;
}

/** vnode 直选（层树联动）。204 = 未命中。返回 pick JSON 文本。 */
export async function canvasPickNode(vnodeId: string): Promise<CanvasPickResult> {
    const response = await fetch('/api/canvas/pick', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ vnode_id: vnodeId }),
    });
    if (!response.ok) {
        if (response.status === 204) return { ok: false, error: 'element not in current frame' };
        return { ok: false, error: `HTTP ${response.status}` };
    }
    const data = (await response.json().catch(() => null)) as unknown;
    return { ok: true, error: '', pick: JSON.stringify(data) };
}

/**
 * 源码抽屉文本加载（files 域同款形态）：/api/files/raw/{path} 不带 workspace
 * 参数——fetch 拦截器自动注入 jwt+workspace（显式带上反而重复字段 400，
 * T-08 实证）。行定位 AC-06 尽力项：M2 只做打开文件。
 */
/** PLAN-093 T-02/T-04: 显式清选（{clear:true}，与 x/y/vnode_id 互斥；
 * 后端同步清 picked——204/200 均视为清选成功）。 */
export async function canvasClearPick(): Promise<CanvasResult> {
    const response = await fetch('/api/canvas/pick', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ clear: true }),
    });
    if (response.status === 200) return { ok: true, error: '', state: 'stopped', app_path: '' };
    return { ok: false, error: `HTTP ${response.status}`, state: '', app_path: '' };
}

export async function canvasLoadSource(path: string): Promise<{ ok: boolean; error: string; text?: string }> {
    // PLAN-093 T-07 G-13: 源码读取走 canvas 域 query 通道
    // /api/canvas/source?workspace=&path=（/api/files/* 在 VM serve 落
    // ag 参数路由 + VM 转译 handler registry 状态桥缺失 → 200 "null"
    // 实测；Query Value 编组为 canvas 域已证通道）。ws 走 musk_workspace
    // localStorage（SetWorkspace 单源写入；本层无 store 访问）。
    const wid = typeof localStorage !== 'undefined' ? (localStorage.getItem('musk_workspace') || '') : '';
    const response = await fetch(`/api/canvas/source?workspace=${encodeURIComponent(wid)}&path=${encodeURIComponent(path)}`, {
        method: 'GET',
        headers: { 'Content-Type': 'text/plain' },
    });
    if (!response.ok) return { ok: false, error: `HTTP ${response.status}` };
    return { ok: true, error: '', text: await response.text() };
}

/**
 * 帧点选监听（面板 setup 调一次，幂等；PLAN-093 T-06 收敛+守卫化）：
 * document 级委托收敛为单实例，监听内身份/版本/几何三重守卫——
 * ①身份：目标为画布帧 img（src 前缀判定）；
 * ②版本：img 已加载且 dataset.cvLoadedSrc（capture load 监听打标）与
 *   当前 src 一致——旧帧 img 加载完成不响应新版本点击（T-06 门控）；
 * ③几何：点击落在内容包装层 .cv-frame-wrap（bbox_pct/点选的统一坐标
 *   系）内 → canvasMapPhysical（canvas_helpers 单源，Vue/VM 同式）换算
 *   帧物理像素 POST /api/canvas/pick；落在容器留白（盒内、包装层外）→
 *   显式清选 {clear:true}（取代旧 object-contain 出界 204 巧合语义）。
 * 结果不在此回接 store：后端置 picked 后经既有 1s 状态轮询带出。
 */
export function installCanvasFrameClicks(): void {
    if (typeof window === 'undefined') return;
    const w = window as unknown as { __muskCanvasClicks?: boolean };
    if (w.__muskCanvasClicks) return;
    w.__muskCanvasClicks = true;
    // 加载打标（capture——load 不冒泡）：帧 img 加载完成记录其 src 版本。
    document.addEventListener('load', (ev) => {
        const t = ev.target as HTMLImageElement | null;
        if (!t || t.tagName !== 'IMG') return;
        const src = t.getAttribute('src') || '';
        if (src.startsWith('/api/canvas/frame')) t.dataset.cvLoadedSrc = src;
    }, true);
    document.addEventListener('click', (ev) => {
        const target = ev.target as HTMLElement | null;
        if (!target) return;
        // 容器命中即处理（T-06 实测修正：留白点击目标是容器而非 img，
        // 旧 closest('img') 早退使清选分支不可达）。
        const wrap = target.closest('.cv-frame-wrap') as HTMLElement | null;
        const box = (wrap ?? target.closest('.cv-frame-box')) as HTMLElement | null;
        if (!box) return;
        const img = box.querySelector('img[src^="/api/canvas/frame"]') as HTMLImageElement | null;
        if (!img) return;
        const src = img.getAttribute('src') || '';
        // 版本守卫：未加载完成 / 打标版本 ≠ 当前版本 → 忽略（旧帧迟到
        // 加载完成不切回旧代次响应）。
        if (!img.complete || !img.naturalWidth) return;
        if ((img.dataset.cvLoadedSrc || '') !== src) return;
        if (!wrap) {
            // 容器留白（内容盒外）→ 显式清选。
            void fetch('/api/canvas/pick', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ clear: true }),
            });
            return;
        }
        // 内容盒内 → 单源换算（canvas_helpers.at 生成模块——与 VM 同式）。
        const rect = wrap.getBoundingClientRect();
        if (rect.width <= 0 || rect.height <= 0) return;
        const ox = Math.round(ev.clientX - rect.left);
        const oy = Math.round(ev.clientY - rect.top);
        const m = canvasMapPhysical(ox, oy, Math.round(rect.width), Math.round(rect.height), img.naturalWidth, img.naturalHeight);
        if (!m.inside) {
            void fetch('/api/canvas/pick', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ clear: true }),
            });
            return;
        }
        void fetch('/api/canvas/pick', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ x: m.x, y: m.y }),
        });
    });
}
