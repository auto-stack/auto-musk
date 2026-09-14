// whitelist_web.ts — PLAN-070 工作区目录白名单 web 侧调用。
//
// 不走 api.at 生成绑定：绑定在非 2xx 时 throw Error('HTTP <status>')，丢弃
// 后端 400 文案（目录不存在/黑名单拒绝/驱动器根等）——白名单视图必须把
// 拒绝原因原样展示给用户。返回结果对象而非 throw——.at 的 catch 值为
// unknown（TS2571 实证），取 error 文案走结果对象。
// workspace 显式携带（setup_auth_fetch 拦截器对 /api/workspace/* 豁免注入）。

export interface RootsResult {
    ok: boolean;
    roots: string[];
    error: string;
}

async function rootsFetch(method: string, path: string, body?: object): Promise<RootsResult> {
    const response = await fetch(path, {
        method,
        headers: { 'Content-Type': 'application/json' },
        body: body === undefined ? undefined : JSON.stringify(body),
    });
    const data: { roots?: unknown; error?: unknown } | null = await response
        .json()
        .catch(() => null);
    if (!response.ok) {
        const msg =
            data && typeof data.error === 'string' ? data.error : `HTTP ${response.status}`;
        return { ok: false, roots: [], error: msg };
    }
    return {
        ok: true,
        roots: data && Array.isArray(data.roots) ? (data.roots as string[]) : [],
        error: '',
    };
}

export async function loadWorkspaceRoots(workspace: string): Promise<RootsResult> {
    return rootsFetch(
        'GET',
        `/api/workspace/roots?workspace=${encodeURIComponent(workspace)}`,
    );
}

export async function addWorkspaceRoot(workspace: string, root: string): Promise<RootsResult> {
    return rootsFetch('POST', '/api/workspace/roots/add', { workspace, root });
}

export async function removeWorkspaceRoot(workspace: string, root: string): Promise<RootsResult> {
    return rootsFetch('POST', '/api/workspace/roots/remove', { workspace, root });
}
