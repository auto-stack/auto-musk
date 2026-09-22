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
