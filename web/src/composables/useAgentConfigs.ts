// useAgentConfigs stub — auto-musk doesn't port the Harness agent-config layer
// (it lives in auto-os-config/daemon). AgentAvatar + ChatsView import this for
// avatar/agent lookups; we provide a no-op fallback so they compile + render
// defaults. Replace with real integration when Harness is wired.
import { ref } from 'vue'

// Permissive shape: ChatsView/AgentAvatar read avatar_url, profession_id,
// is_default, soul_id, etc. We type it loosely so the stub doesn't block
// compilation; the real AgentConfig will come from auto-os-config later.
export interface AgentConfig {
  id: string
  name: string
  profession: string
  // PLAN-030 复审修复：消费者（ChatsView/AgentAvatar）实际读取的字段
  // 显式声明，避免 index signature 的 unknown 传播成类型错误。
  profession_id: string
  is_default: boolean
  avatar_url?: string
  soul_id?: string
  [key: string]: unknown
}

const _configs = ref<AgentConfig[]>([])

export function useAgentConfigs() {
  return {
    configs: _configs,
    async loadConfigs() {
      // no-op: Harness not wired. Real impl fetches from /api/.../agents.
    },
    list: () => _configs.value,
    getById: (_id: string): AgentConfig | undefined => undefined,
    getByProfession: (_profession: string): AgentConfig | undefined => undefined,
  }
}
