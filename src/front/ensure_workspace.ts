// ensure_workspace.ts — 确保有默认 workspace（Plan 022 C 类 parity）
//
// 原生 web/ 有 WorkspaceSelector 组件让用户选 workspace。Auto 版没有该组件，
// 这里在 App Init 时自动选第一个可用 workspace（存入 localStorage musk_workspace）。
// forge_store/wiki_store 的 Init 从 localStorage 读 workspace。
//
// 逃生舱说明：AutoUI .at 无法表达 workspace 选择 UI，用 use { fn } 在 App.Init 注入。

export async function ensureWorkspace(): Promise<void> {
  const existing = localStorage.getItem('musk_workspace')
  if (existing) return

  const token = localStorage.getItem('musk_jwt')
  if (!token) return

  try {
    const resp = await fetch('/api/workspace/list', {
      headers: { 'Authorization': `Bearer ${token}` }
    })
    if (!resp.ok) return
    const data = await resp.json()
    const workspaces = data.workspaces || []
    // 优先选非空的 workspace
    const nonEmpty = workspaces.find((w: any) => !w.is_empty)
    const chosen = nonEmpty || workspaces[0]
    if (chosen) {
      localStorage.setItem('musk_workspace', chosen.id)
    }
  } catch {
    // 静默失败——workspace 不是阻塞项
  }
}
