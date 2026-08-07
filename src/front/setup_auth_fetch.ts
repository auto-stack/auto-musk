// setup_auth_fetch.ts — 全局 fetch 拦截器（对应 web/src/main.ts 的拦截器）
//
// Plan 022 Phase 5: 生成的 api.ts 用普通 fetch，但 auto-musk 后端需要
// Authorization: Bearer <musk_jwt> + workspace query 参数。这个函数在
// App 挂载时（.Init handler）调用一次，monkey-patch window.fetch 注入它们。
// 对应 web/src/main.ts 的拦截器 + useAuth.ts 的 authFetch 逻辑的合并。
//
// 逃生舱说明：AutoUI .at 无法表达任意 fetch 重写，故用 use { fn } 透传。

export function setupAuthFetch(): void {
  const originalFetch = window.fetch
  window.fetch = function (input: RequestInfo | URL, init?: RequestInit): Promise<Response> {
    const token = localStorage.getItem('musk_jwt')

    // 只对 /api/* 请求注入认证；其它（静态资源等）原样放行。
    const url = typeof input === 'string' ? input : input instanceof URL ? input.toString() : input.url
    if (token && url.startsWith('/api/') && !url.startsWith('/api/auth/login') && !url.startsWith('/api/auth/register')) {
      const headers = new Headers(init?.headers || {})
      headers.set('Authorization', `Bearer ${token}`)
      init = { ...init, headers }
    }

    // workspace query 参数（从 localStorage 或当前 URL 取）。
    if (url.startsWith('/api/') && !url.startsWith('/api/workspace/') && !url.startsWith('/api/auth/')) {
      const wid = localStorage.getItem('musk_workspace') || ''
      if (wid) {
        const sep = url.includes('?') ? '&' : '?'
        const newUrl = url + sep + 'workspace=' + encodeURIComponent(wid)
        if (typeof input === 'string') {
          input = newUrl
        } else if (input instanceof URL) {
          input = new URL(newUrl)
        } else {
          input = new Request(newUrl, init)
        }
      }
    }

    return originalFetch(input, init)
  }
}
