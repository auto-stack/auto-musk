// usePlans — PLAN-024: plan CRUD + state machine + merge via /api/plans/*.
// Mirrors useSpecs' singleton-state + authFetch pattern.
import { ref } from 'vue'
import type { PlanFile, PlanStatus, MergeResult } from '@/types/plans'
import { authFetch } from './useAuth'

const API = '/api/plans'

// ─── Singleton state ────────────────────────────────────────────────────────
const _plans = ref<PlanFile[]>([])
const _current = ref<PlanFile | null>(null)
const _isLoading = ref(false)
const _error = ref<string | null>(null)

export function usePlans() {
  const plans = _plans
  const current = _current
  const isLoading = _isLoading
  const error = _error

  async function loadPlans(includeArchived = false): Promise<void> {
    isLoading.value = true
    error.value = null
    try {
      const resp = await authFetch(`${API}?include_archived=${includeArchived}`)
      if (!resp.ok) throw new Error(`Failed to load plans: ${resp.status}`)
      const data = await resp.json()
      _plans.value = (data.plans ?? []) as PlanFile[]
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      isLoading.value = false
    }
  }

  async function loadPlan(seq: number): Promise<PlanFile | null> {
    try {
      const resp = await authFetch(`${API}/${seq}`)
      if (!resp.ok) throw new Error(`Failed to load plan ${seq}: ${resp.status}`)
      const plan = (await resp.json()) as PlanFile
      _current.value = plan
      return plan
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      return null
    }
  }

  async function createPlan(feature_name: string, content = ''): Promise<PlanFile | null> {
    try {
      const resp = await authFetch(API, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ feature_name, content }),
      })
      if (!resp.ok) {
        const txt = await resp.text()
        throw new Error(`create failed: ${resp.status} ${txt}`)
      }
      return (await resp.json()) as PlanFile
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      return null
    }
  }

  async function updatePlan(seq: number, content: string): Promise<PlanFile | null> {
    try {
      const resp = await authFetch(`${API}/${seq}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ content }),
      })
      if (!resp.ok) throw new Error(`update failed: ${resp.status}`)
      const plan = (await resp.json()) as PlanFile
      _current.value = plan
      return plan
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      return null
    }
  }

  async function transitionPlan(seq: number, status: PlanStatus): Promise<PlanFile | null> {
    try {
      const resp = await authFetch(`${API}/${seq}/transition`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ status }),
      })
      if (!resp.ok) {
        const txt = await resp.text()
        throw new Error(`transition failed: ${resp.status} ${txt}`)
      }
      const plan = (await resp.json()) as PlanFile
      _current.value = plan
      return plan
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      return null
    }
  }

  async function archivePlan(seq: number): Promise<PlanFile | null> {
    try {
      const resp = await authFetch(`${API}/${seq}/archive`, { method: 'POST' })
      if (!resp.ok) throw new Error(`archive failed: ${resp.status}`)
      return (await resp.json()) as PlanFile
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      return null
    }
  }

  async function mergePlan(seq: number): Promise<MergeResult | null> {
    try {
      const resp = await authFetch(`${API}/${seq}/merge`, { method: 'POST' })
      if (!resp.ok) {
        const txt = await resp.text()
        throw new Error(`merge failed: ${resp.status} ${txt}`)
      }
      return (await resp.json()) as MergeResult
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      return null
    }
  }

  return {
    plans,
    current,
    isLoading,
    error,
    loadPlans,
    loadPlan,
    createPlan,
    updatePlan,
    transitionPlan,
    archivePlan,
    mergePlan,
  }
}
