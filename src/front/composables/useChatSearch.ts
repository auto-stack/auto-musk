// useChatSearch.ts — chat 消息搜索过滤 composable（Plan 022 技术债 #2）
//
// 内部读 useForgeStoreStore 的 messages，提供 search（双向绑定搜索框）
// + filteredMessages（按关键字过滤消息列表）。
// 返回 reactive 对象（非 ref/computed），让 .at 模板直接 .chatSearch.search
// 和 v-for in .chatSearch.filteredMessages 可用（无需 .value）。
// codegen: const chatSearch = useChatSearch()

import { ref, computed, reactive } from 'vue'
import { useForgeStoreStore } from '@/stores/useForgeStoreStore'

export function useChatSearch() {
  const store = reactive(useForgeStoreStore())
  const search = ref('')

  const filteredMessages = computed(() => {
    const msgs = store.messages || []
    if (!search.value || !search.value.trim()) return msgs
    const q = search.value.toLowerCase()
    return msgs.filter((msg: any) => {
      const content = (msg?.content || '').toLowerCase()
      const role = (msg?.role || '').toLowerCase()
      return content.includes(q) || role.includes(q)
    })
  })

  // 包装成 reactive 对象——模板访问 .search / .filteredMessages 时
  // 自动解包 ref/computed，无需 .value
  return reactive({
    search,
    filteredMessages,
  })
}
