import { ref, computed, watch } from 'vue'

export interface PendingGate {
  gateId: string
  runId: string
  profession: string
  title: string
  sectionId?: string
  since: number
  status: 'pending' | 'reviewing' | 'approved' | 'rejected' | 'snoozed'
}

// ─── Singleton state ────────────────────────────────────────────────────────
const _gates = ref<PendingGate[]>([])
const _currentSecretary = ref<PendingGate | null>(null)

function promoteNext() {
  const next = _gates.value.find((g) => g.status === 'pending')
  _currentSecretary.value = next ?? null
}

// Plan 028 T14：gate_reached→registerGate 的路由原在 forge_stream.ts（逃生舱
// 直调）。forge_stream 原生化后，这里 watch store.current_gate 驱动
// SecretaryMessage 审批条（GateCard 仍由 current_gate 直接驱动）。
function _resolveGate(gateId: string, decision: 'approved' | 'rejected') {
  const gate = _gates.value.find((g) => g.gateId === gateId)
  if (gate) {
    gate.status = decision
  }
  if (_currentSecretary.value?.gateId === gateId) {
    promoteNext()
  }
}

function _registerGate(gate: Omit<PendingGate, 'status'>) {
  const existing = _gates.value.find((g) => g.gateId === gate.gateId)
  if (existing) return
  const newGate: PendingGate = { ...gate, status: 'pending' }
  _gates.value.push(newGate)
  if (!_currentSecretary.value) {
    _currentSecretary.value = newGate
  }
}

import { useForgeStoreStore } from '@/stores/useForgeStoreStore'
import { useRelayStoreStore } from '@/stores/useRelayStoreStore'
const _forge = useForgeStoreStore()
watch(_forge.current_gate, (gate: any) => {
  if (!gate || !gate.gate_id) return
  _registerGate({
    gateId: gate.gate_id,
    runId: gate.run_id ?? '',
    profession: gate.profession || 'unknown',
    title: gate.title || (gate.profession || 'agent') + ' needs approval',
    sectionId: gate.section_id ?? '',
    since: gate.since ?? Date.now(),
  })
})

// Plan 028 T16：relay SSE 的 gate_reached/gate_resolved 经 relay_store.gate_signal
// 中转（原 useEventRouter.handleEvent 的有效路由，setEventCallbacks 分支无调用方不迁）。
const _relay = useRelayStoreStore()
watch(_relay.gate_signal, (sig: any) => {
  if (!sig || !sig.gate_id) return
  if (sig.kind === 'reached') {
    _registerGate({
      gateId: sig.gate_id,
      runId: sig.run_id ?? '',
      profession: sig.profession || 'unknown',
      title: sig.title || (sig.profession || 'agent') + ' needs approval',
      sectionId: sig.section_id ?? '',
      since: Date.now(),
    })
  } else if (sig.kind === 'resolved') {
    _resolveGate(sig.gate_id, sig.decision === 'rejected' ? 'rejected' : 'approved')
  }
})

export function useGateInbox() {
  const gates = _gates
  const currentSecretary = _currentSecretary
  const badgeCount = computed(() => _gates.value.filter((g) => g.status === 'pending').length)
  const hasPending = computed(() => badgeCount.value > 0)

  function registerGate(gate: Omit<PendingGate, 'status'>) {
    _registerGate(gate)
  }

  function resolveGate(gateId: string, decision: 'approved' | 'rejected') {
    _resolveGate(gateId, decision)
  }

  function dismissSecretary() {
    // Dismiss current secretary view without resolving — next pending gate shows
    promoteNext()
  }

  function snoozeGate(gateId: string) {
    const gate = _gates.value.find((g) => g.gateId === gateId)
    if (gate) {
      gate.status = 'snoozed'
      if (_currentSecretary.value?.gateId === gateId) {
        promoteNext()
      }
    }
  }

  function wakeSnoozed() {
    for (const gate of _gates.value) {
      if (gate.status === 'snoozed') {
        gate.status = 'pending'
      }
    }
    if (!_currentSecretary.value) {
      promoteNext()
    }
  }

  function clearResolved() {
    _gates.value = _gates.value.filter((g) => g.status !== 'approved' && g.status !== 'rejected')
    if (_currentSecretary.value && !['approved', 'rejected'].includes(_currentSecretary.value.status)) {
      return
    }
    promoteNext()
  }

  return {
    gates,
    currentSecretary,
    badgeCount,
    hasPending,
    registerGate,
    resolveGate,
    dismissSecretary,
    snoozeGate,
    wakeSnoozed,
    clearResolved,
  }
}
