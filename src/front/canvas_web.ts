// canvas_web.ts — PLAN-087 T-07: 画布会话 web 侧调用（启动/停止）。
//
// 不走 api.at 生成绑定（手写路由，无绑定源）；结果对象而非 throw——.at 的
// catch 值 unknown（TS2571 实证），错误文案走结果对象（whitelist_web.ts
// 同口径）。状态/帧轮询不经此处：store 侧 Http.get_msg 直连（forge_store
// PollBackfill 同款），启动后随 1s 轮询自然反映状态。
// workspace 显式携带（start 需要它做沙箱判定；拦截器对 /api/canvas/* 注入
// 的 workspace 查询与显式 query 同名等价，显式传参以防拦截器缺位）。

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
    const response = await fetch(`/api/files/raw/${encodeURIComponent(path)}`, {
        method: 'GET',
        headers: { 'Content-Type': 'text/plain' },
    });
    if (!response.ok) return { ok: false, error: `HTTP ${response.status}` };
    return { ok: true, error: '', text: await response.text() };
}

/**
 * 帧点选监听（面板 setup 调一次，幂等）：document 级委托——点击目标为画布
 * 帧 img（src 前缀判定）时，按 显示→自然 尺寸比换算帧像素坐标并 POST
 * /api/canvas/pick。结果不在此回接 store：后端置 picked 后经既有 1s 状态
 * 轮询带出（≤1s 滞后，M2 接受；零新通道）。
 */
export function installCanvasFrameClicks(): void {
    if (typeof window === 'undefined') return;
    const w = window as unknown as { __muskCanvasClicks?: boolean };
    if (w.__muskCanvasClicks) return;
    w.__muskCanvasClicks = true;
    document.addEventListener('click', (ev) => {
        const target = ev.target as HTMLElement | null;
        const img = target?.closest?.('img') as HTMLImageElement | null;
        if (!img) return;
        const src = img.getAttribute('src') || '';
        if (!src.startsWith('/api/canvas/frame')) return;
        const rect = img.getBoundingClientRect();
        if (rect.width <= 0 || rect.height <= 0 || !img.naturalWidth) return;
        // object-contain 字幕区（img 盒内留白）点击换算出界坐标 → 后端
        // hit-test 未命中 → 204，语义即"点空白取消"，无需前端特判。
        const x = ((ev.clientX - rect.left) / rect.width) * img.naturalWidth;
        const y = ((ev.clientY - rect.top) / rect.height) * img.naturalHeight;
        void fetch('/api/canvas/pick', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ x, y }),
        });
    });
}
