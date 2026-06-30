import { ref, computed } from 'vue'
import { authFetch } from './useAuth'
import { currentWorkspaceId } from './useWorkspaceId'

export interface WorkspaceMeta {
  id: string
  path: string
  name: string
  last_opened: number
  /** True if the project dir has no user files yet (triggers onboarding). */
  is_empty?: boolean
}

// Singleton state
const _current = ref<WorkspaceMeta | null>(null)
const _recent = ref<WorkspaceMeta[]>([])
const _isLoading = ref(false)
const _error = ref<string | null>(null)

const WS_STORAGE_KEY = 'musk-workspace-id'

export function useProject() {
  const isOpen = computed(() => _current.value !== null)
  const projectName = computed(() => _current.value?.name ?? null)
  const projectPath = computed(() => _current.value?.path ?? null)
  const workspaceId = computed(() => _current.value?.id ?? null)
  const currentWorkspace = _current
  const recentWorkspaces = _recent
  const isLoading = _isLoading
  const error = _error

  // Backwards-compat aliases for older views that may reference these names.
  const projectInfo = computed(() =>
    _current.value
      ? {
          path: _current.value.path,
          name: _current.value.name,
          specs_dir: 'specs',
          has_specs: true,
          is_open: true,
          is_empty: false,
        }
      : null,
  )
  const recentProjects = _recent

  function syncUrl(id: string | null) {
    // Persist to localStorage so a refresh restores the workspace even if the
    // URL query param was wiped by a view-switch.
    if (id) localStorage.setItem(WS_STORAGE_KEY, id)
    else localStorage.removeItem(WS_STORAGE_KEY)
    const url = new URL(window.location.href)
    if (id) url.searchParams.set('workspace', id)
    else url.searchParams.delete('workspace')
    window.history.replaceState({}, '', url.toString())
  }

  async function fetchStatus() {
    // Prefer URL param, fall back to localStorage.
    let id = new URL(window.location.href).searchParams.get('workspace')
    if (!id) id = localStorage.getItem(WS_STORAGE_KEY)
    const query = id ? `?workspace=${encodeURIComponent(id)}` : ''
    try {
      const resp = await authFetch(`/api/workspace/status${query}`)
      if (resp.ok) {
        const data = await resp.json()
        _current.value = data.workspace
        currentWorkspaceId.value = data.workspace.id
        syncUrl(data.workspace.id)
      }
    } catch {
      // ignore — leave current null
    }
  }

  async function openWorkspace(path: string): Promise<WorkspaceMeta> {
    const resp = await authFetch('/api/workspace/open', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ path }),
    })
    if (!resp.ok) throw new Error(`open workspace failed: ${resp.status}`)
    const data = await resp.json()
    _current.value = data.workspace
    currentWorkspaceId.value = data.workspace.id
    syncUrl(data.workspace.id)
    await loadRecent()
    return data.workspace as WorkspaceMeta
  }

  async function loadRecent() {
    try {
      const resp = await authFetch('/api/workspace/list')
      if (resp.ok) _recent.value = (await resp.json()).workspaces ?? []
    } catch {
      // ignore
    }
  }

  async function browse(path: string) {
    const q = path ? `?path=${encodeURIComponent(path)}` : ''
    const resp = await authFetch(`/api/workspace/browse${q}`)
    if (!resp.ok) return []
    return ((await resp.json()).entries ?? []) as { name: string; path: string }[]
  }

  return {
    isOpen,
    projectName,
    projectPath,
    workspaceId,
    currentWorkspace,
    recentWorkspaces,
    isLoading,
    error,
    // compat aliases
    projectInfo,
    recentProjects,
    fetchStatus,
    openWorkspace,
    loadRecent,
    browse,
    syncUrl,
  }
}
