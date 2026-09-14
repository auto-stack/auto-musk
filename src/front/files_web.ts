// files_web.ts — PLAN-068 文件浏览器 web 侧正文加载。
//
// 走 .text()——api.at 生成的绑定固定 response.json()，对 text/markdown
// 等文本响应必炸（E2E 实证 loading 永挂）；media 预览不经绑定
// （<img>/<video> 直接吃 /api/files/raw URL）。

export async function loadFilesFileText(path: string): Promise<string> {
    const response = await fetch(`/api/files/raw/${encodeURIComponent(path)}`, {
        method: 'GET',
        headers: { 'Content-Type': 'application/json' },
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.text();
}
