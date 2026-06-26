// useSpecs — adapted for auto-musk's flat /api/specs/* endpoints (Plan 010 P2).
// auto-forge's version used /api/forge/specs/{project}/* (per-project namespace);
// auto-musk is single-project flat. Method signatures are kept compatible so
// SpecsView doesn't need changes (the `project` param is accepted but ignored).
import { ref } from 'vue'
import type { SpecsDocument, SpecsSection, SpecItem } from '@/types/specs'
import { authFetch } from './useAuth'

const API = '/api/specs'

// ─── Singleton state ────────────────────────────────────────────────────────
const _document = ref<SpecsDocument | null>(null)
const _isLoading = ref(false)
const _error = ref<string | null>(null)

export function useSpecs() {
  const document = _document
  const isLoading = _isLoading
  const error = _error

  async function loadDocument(_project = '') {
    isLoading.value = true
    error.value = null
    try {
      const resp = await authFetch(API)
      if (!resp.ok) throw new Error(`Failed to load specs: ${resp.status}`)
      document.value = await resp.json()
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      isLoading.value = false
    }
  }

  async function loadOverview(_project = ''): Promise<{ content: string; exists: boolean }> {
    try {
      const resp = await authFetch(`${API}/overview`)
      if (!resp.ok) throw new Error(`overview: ${resp.status}`)
      const data = await resp.json()
      // auto-musk overview is structured (sections/counts); synthesize a markdown summary.
      const lines: string[] = [`# ${data.project || 'specs'} (v${data.version}, ${data.total_items} items)\n`]
      for (const s of data.sections || []) {
        lines.push(`## ${s.title} — ${s.item_count} items (${s.status})`)
      }
      return { content: lines.join('\n'), exists: true }
    } catch {
      return { content: '', exists: false }
    }
  }

  async function loadModuleOutline(_project: string, _module: string): Promise<{ content: string; exists: boolean }> {
    // auto-musk has no module-outline endpoint; return empty.
    return { content: '', exists: false }
  }

  /** Save a section: auto-musk is item-level, so we diff against the loaded doc
   *  and upsert changed/added items, delete removed ones. */
  async function saveSection(_project: string, section: SpecsSection) {
    try {
      // Load current to diff
      const cur = await authFetch(API)
      const curDoc: SpecsDocument = cur.ok ? await cur.json() : { project: '', version: 0, sections: [] }
      const curSection = curDoc.sections.find((s) => s.id === section.id)
      const curIds = new Set((curSection?.items || []).map((i) => i.id))
      const newIds = new Set(section.items.map((i) => i.id))

      // Upsert all items in the section (simplest correct approach).
      for (const item of section.items) {
        await authFetch(`${API}/item`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ section_id: section.id, item }),
        })
      }
      // Delete removed items.
      for (const id of curIds) {
        if (!newIds.has(id)) {
          await authFetch(`${API}/item/${section.id}/${id}`, { method: 'DELETE' })
        }
      }
      await loadDocument()
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  async function saveDocument(_project: string, doc: SpecsDocument) {
    // auto-musk has no whole-document PUT; save section by section.
    for (const section of doc.sections) {
      await saveSection('', section)
    }
  }

  async function rebuildRelations(_project = '') {
    try {
      const resp = await authFetch(`${API}/rebuild-relations`, { method: 'POST' })
      if (!resp.ok) throw new Error(`rebuild: ${resp.status}`)
      document.value = await resp.json()
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  // ─── Helpers ──────────────────────────────────────────────────────────────

  function findItemById(id: string): { item: SpecItem; section: SpecsSection } | null {
    const doc = document.value
    if (!doc) return null
    for (const section of doc.sections) {
      const item = section.items.find((i) => i.id === id)
      if (item) return { item, section }
    }
    return null
  }

  function findSectionByItemId(id: string): SpecsSection | null {
    const doc = document.value
    if (!doc) return null
    return doc.sections.find((s) => s.items.some((i) => i.id === id)) ?? null
  }

  return {
    document,
    isLoading,
    error,
    loadDocument,
    loadOverview,
    loadModuleOutline,
    saveSection,
    saveDocument,
    findItemById,
    findSectionByItemId,
    rebuildRelations,
  }
}
