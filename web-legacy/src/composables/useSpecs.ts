import { ref } from 'vue'
import { apiFetch } from './useAuth'

// Types mirroring backend crates/musk/src/specs.rs
export type SectionType =
  | 'Goals' | 'Architecture' | 'Designs' | 'Plans'
  | 'Tests' | 'Reviews' | 'Reports'

export type SpecStatus =
  | 'Empty' | 'Proposed' | 'Draft' | 'UnderReview' | 'Approved'
  | 'InProgress' | 'InImplementation' | 'Implemented' | 'Verified' | 'Done'
  | 'Archived' | 'Rejected' | 'Backlog' | 'Ready' | 'InReview' | 'Blocked'
  | 'Superseded' | 'Outdated' | 'Stable' | 'Deprecated' | 'Published'
  | 'Analysed' | 'Obsolete'

export interface SpecItem {
  id: string
  title: string
  content: string
  status: SpecStatus
  depends_on: string[]
  related: string[]
  priority: string | null
  assignee: string | null
  test_file: string | null
  file: string | null
  milestone: string | null
  module: string | null
  tags: string[]
  created_at: number
  modified_at: number
  completed_at: number | null
}

export interface SpecsSection {
  id: string
  section_type: SectionType
  title: string
  items: SpecItem[]
  status: SpecStatus
  content: string
  depends_on: string[]
  last_modified: number
  last_verified: number | null
}

export interface SpecsDocument {
  project: string
  version: number
  sections: SpecsSection[]
}

export interface SectionOverview {
  id: string
  section_type: SectionType
  title: string
  status: SpecStatus
  item_count: number
  status_counts: [string, number][]
}
export interface SpecsOverview {
  project: string
  version: number
  total_items: number
  sections: SectionOverview[]
}

// module-level singleton state
const doc = ref<SpecsDocument | null>(null)
const overview = ref<SpecsOverview | null>(null)
const loading = ref(false)
const error = ref('')

async function loadDoc() {
  loading.value = true
  error.value = ''
  const resp = await apiFetch('/api/specs')
  if (resp.ok) {
    doc.value = await resp.json()
  } else {
    error.value = `load specs: ${resp.status}`
  }
  loading.value = false
}

async function loadOverview() {
  const resp = await apiFetch('/api/specs/overview')
  if (resp.ok) overview.value = await resp.json()
}

async function upsertItem(sectionId: string, item: Partial<SpecItem> & { id: string }) {
  const full: SpecItem = {
    title: '',
    content: '',
    status: 'Empty',
    depends_on: [],
    related: [],
    priority: null,
    assignee: null,
    test_file: null,
    file: null,
    milestone: null,
    module: null,
    tags: [],
    created_at: 0,
    modified_at: 0,
    completed_at: null,
    ...item,
  }
  const resp = await apiFetch('/api/specs/item', {
    method: 'POST',
    body: JSON.stringify({ section_id: sectionId, item: full }),
  })
  if (resp.ok) {
    await loadDoc()
    await loadOverview()
  }
}

async function transitionItem(sectionId: string, itemId: string, newStatus: SpecStatus) {
  const resp = await apiFetch('/api/specs/transition', {
    method: 'POST',
    body: JSON.stringify({ section_id: sectionId, item_id: itemId, new_status: newStatus }),
  })
  if (resp.ok) {
    await loadDoc()
    await loadOverview()
  }
}

async function deleteItem(sectionId: string, itemId: string) {
  const resp = await apiFetch(`/api/specs/item/${sectionId}/${itemId}`, {
    method: 'DELETE',
  })
  if (resp.ok) {
    await loadDoc()
    await loadOverview()
  }
}

async function driftCheck() {
  const resp = await apiFetch('/api/specs/drift-check', { method: 'POST' })
  if (resp.ok) return await resp.json()
  return null
}

export function useSpecs() {
  return {
    doc, overview, loading, error,
    loadDoc, loadOverview, upsertItem, transitionItem, deleteItem, driftCheck,
  }
}

// Status → tailwind color class for badges (per designs/001 StatusBadge mapping)
export function statusColor(s: SpecStatus): string {
  switch (s) {
    case 'Empty': return 'gray'
    case 'Proposed': case 'Draft': case 'Backlog': return 'gray'
    case 'UnderReview': case 'InReview': case 'Analysed': return 'yellow'
    case 'Approved': case 'Ready': return 'blue'
    case 'InProgress': case 'InImplementation': return 'blue'
    case 'Implemented': case 'Published': return 'green'
    case 'Verified': case 'Done': case 'Stable': return 'green'
    case 'Blocked': case 'Rejected': return 'red'
    case 'Archived': case 'Superseded': case 'Outdated':
    case 'Deprecated': case 'Obsolete': return 'gray'
    default: return 'gray'
  }
}
