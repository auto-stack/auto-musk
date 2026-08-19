// gate_router.ts — 跨 store 审批门路由桥（纯接线，无状态无逻辑）
//
// Plan 029 T12：useGateInbox.ts 的状态/逻辑已原生化为 gate_inbox.at store；
// 本桥只剩两个 watch（受 auto-lang v1「单 store per file」限制，store 间
// 无法互相引用）。待多 store 放开后并入 .at 并删除本文件（D 组登记）。
//
// 路由（与原 useGateInbox 完全一致）：
//   - forge.current_gate（gate_reached）→ Register
//   - relay.gate_signal（reached/resolved）→ Register / Resolve

import { watch } from 'vue'
import { useForgeStoreStore } from '@/stores/useForgeStoreStore'
import { useRelayStoreStore } from '@/stores/useRelayStoreStore'
import { useGateInboxStore } from '@/stores/useGateInboxStore'

export function useGateRouter() {
  const inbox = useGateInboxStore()
  const forge = useForgeStoreStore()
  const relay = useRelayStoreStore()

  watch(() => forge.current_gate, (gate: any) => {
    if (!gate || !gate.gate_id) return
    inbox.Register({
      gateId: gate.gate_id,
      runId: gate.run_id ?? '',
      profession: gate.profession || 'unknown',
      title: gate.title || (gate.profession || 'agent') + ' needs approval',
      sectionId: gate.section_id ?? '',
      since: gate.since ?? Date.now(),
    })
  })

  watch(() => relay.gate_signal, (sig: any) => {
    if (!sig || !sig.gate_id) return
    if (sig.kind === 'reached') {
      inbox.Register({
        gateId: sig.gate_id,
        runId: sig.run_id ?? '',
        profession: sig.profession || 'unknown',
        title: sig.title || (sig.profession || 'agent') + ' needs approval',
        sectionId: sig.section_id ?? '',
        since: Date.now(),
      })
    } else if (sig.kind === 'resolved') {
      inbox.Resolve(sig.gate_id, sig.decision === 'rejected' ? 'rejected' : 'approved')
    }
  })
}
