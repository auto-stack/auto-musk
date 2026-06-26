// useProject — auto-musk is single-project (no open/close model). auto-forge's
// version called /api/forge/project/* (status/open/close/recent/browse); musk
// has no project concept, so this is a stub that reports a default open project.
// ChatsView reads projectPath from here. Replace with real integration if musk
// later gains a project model.
import { ref, computed } from 'vue'

export interface ProjectInfo {
  path: string
  name: string
  specs_dir: string
  has_specs: boolean
  is_open: boolean
  is_empty?: boolean
}

export interface RecentProject {
  path: string
  name: string
  last_opened: number
}

export interface BrowseEntry {
  name: string
  path: string
  is_dir: boolean
}

const DEFAULT_INFO: ProjectInfo = {
  path: '.',
  name: 'musk',
  specs_dir: 'specs',
  has_specs: true,
  is_open: true,
  is_empty: false,
}

const _projectInfo = ref<ProjectInfo | null>(DEFAULT_INFO)
const _isLoading = ref(false)
const _error = ref<string | null>(null)
const _recentProjects = ref<RecentProject[]>([])

export function useProject() {
  const projectInfo = _projectInfo
  const isLoading = _isLoading
  const error = _error
  const recentProjects = _recentProjects

  const isOpen = computed(() => true)
  const projectName = computed(() => _projectInfo.value?.name ?? 'musk')
  const projectPath = computed(() => _projectInfo.value?.path ?? '.')

  async function fetchStatus() {
    // no-op: single-project, always open.
    _projectInfo.value = DEFAULT_INFO
  }

  async function openProject(_path: string) {
    _projectInfo.value = { ...DEFAULT_INFO, path: _path }
  }

  async function closeProject() {
    // no-op in single-project mode
  }

  async function loadRecent() {
    // no-op
  }

  async function browse(_path: string): Promise<BrowseEntry[]> {
    return []
  }

  return {
    projectInfo,
    isLoading,
    error,
    recentProjects,
    isOpen,
    projectName,
    projectPath,
    fetchStatus,
    openProject,
    closeProject,
    loadRecent,
    browse,
  }
}
