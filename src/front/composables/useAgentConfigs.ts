// useAgentConfigs.ts — agent 职业配置（逃生舱，替代 web/ 的空 stub）
//
// Plan 022 B 类：原生 useAgentConfigs 是空 stub（注释"Harness not wired"），
// 但后端 /api/forge/relay/professions 已实现（返回完整 Profession 列表）。
// 本 composable 调该端点，映射成 AgentConfig 形态供 AgentAvatar/ChatsView 用。
// 生成工程用全局 fetch 拦截器（setup_auth_fetch.ts 自动注入 jwt+workspace），
// 故直接 fetch，不用 authFetch。

import { ref } from 'vue'

export interface AgentConfig {
  id: string
  name: string
  profession: string
  profession_id: string
  avatar_url?: string
  is_default?: boolean
  [key: string]: unknown
}

const _configs = ref<AgentConfig[]>([])

export function useAgentConfigs() {
  const configs = _configs

  async function loadConfigs(): Promise<void> {
    try {
      const resp = await fetch('/api/forge/relay/professions')
      if (!resp.ok) return
      const data = await resp.json()
      // 后端返回 { professions: [{id, name, phase, ...}] }
      const profs = (data?.professions || []) as any[]
      _configs.value = profs
        .filter((p) => p && p.id && p.name)
        .map((p) => ({
          id: p.id,
          name: p.name,
          profession: p.id,
          profession_id: p.id,
          is_default: true,
        }))
    } catch {
      // 拉取失败保留空（AgentAvatar 走 HSL fallback，MentionInput 走 DEFAULT_PROFESSIONS）
    }
  }

  function list(): AgentConfig[] {
    return _configs.value
  }

  function getById(id: string): AgentConfig | undefined {
    return _configs.value.find((c) => c.id === id)
  }

  function getByProfession(profession: string): AgentConfig | undefined {
    return _configs.value.find(
      (c) => c.profession_id === profession || c.profession === profession,
    )
  }

  return { configs, loadConfigs, list, getById, getByProfession }
}
