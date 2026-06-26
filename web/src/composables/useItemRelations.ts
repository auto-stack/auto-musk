import { ref, computed } from 'vue'
import type { SpecItem } from '@/types/specs'
import { authFetch } from './useAuth'

const API = '/api/specs'

export interface RelatedItem {
  id: string
  title: string
  section_type: string
  status: string
}

export function useItemRelations(_project: string) {
  const loading = ref(false)
  const parents = ref<RelatedItem[]>([])
  const children = ref<RelatedItem[]>([])

  async function loadRelations(itemId: string) {
    loading.value = true
    try {
      const resp = await authFetch(`${API}/related/${encodeURIComponent(itemId)}`)
      if (!resp.ok) throw new Error(`Failed to load relations: ${resp.status}`)
      const data = await resp.json()
      // auto-musk returns { depends_on: [...], related: [...] } (id lists only).
      // Map to RelatedItem (title/status resolved client-side from the doc).
      const depIds: string[] = data.depends_on || []
      const relIds: string[] = data.related || []
      parents.value = depIds.map((id) => ({ id, title: id, section_type: '', status: '' }))
      children.value = relIds.map((id) => ({ id, title: id, section_type: '', status: '' }))
    } catch (e) {
      console.error('Failed to load relations:', e)
      parents.value = []
      children.value = []
    } finally {
      loading.value = false
    }
  }

  return {
    loading,
    parents,
    children,
    loadRelations,
  }
}
