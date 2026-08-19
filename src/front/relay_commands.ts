// relay_commands.ts — /relay//superpower//spec1 命令路由（逃生舱）
//
// Plan 022 B 类批3：命令解析需调 useRelay.startRun + advanceRun + 推 assistant
// 消息（复杂控制流，.at handler 撞 parser 边界），故封逃生舱 fn。
// 返回 true = 命令已处理（父组件不再走普通发送）；false = 非命令，普通发送。

import { useRelayStoreStore } from '@/stores/useRelayStoreStore'
import { useForgeStoreStore } from '@/stores/useForgeStoreStore'

/**
 * 同步检测是否为 relay 命令（/relay//superpower//spec1）。
 * 返回 true = 是命令（异步执行已 fire-and-forget），父组件不再走普通发送；
 * false = 非命令，普通发送。
 * 同步返回是因为 .at handler 不支持 await（Promise 对 if 判断恒 truthy）。
 * 命令的异步执行（startRun/advanceRun/推消息）在内部 fire-and-forget。
 */
export function handleRelayCommand(text: string): boolean {
  const trimmed = text.trim()

  if (trimmed.startsWith('/relay ')) {
    const goal = trimmed.slice('/relay '.length).trim()
    if (goal) {
      void runRelay({ flow_id: 'default', task: goal }, goal, '🚀', 'default',
        `Advisor → Architect → Planner → Tester → Coder → Reviewer → Documenter 正在自动接力执行。`)
    }
    return true
  }

  if (trimmed.startsWith('/superpower ')) {
    const task = trimmed.slice('/superpower '.length).trim()
    if (task) {
      void runRelay({ flow_id: 'superpower', task }, task, '⚡', 'superpower',
        `流程：Brainstorm → Plan → Execute → Review。点击下方的 Run 卡片查看实时进度。`)
    }
    return true
  }

  if (trimmed.startsWith('/spec1 ')) {
    const goal = trimmed.slice('/spec1 '.length).trim()
    if (goal) {
      void runRelay(
        { flow_id: 'simple', task: goal, steps: [{ id: 'discover', profession_id: 'advisor' } as any] },
        goal, '🎯', 'simple',
        `Advisor 正在分析并尝试写出 Goals。此 Run 只执行 Advisor 一步。`,
      )
    }
    return true
  }

  return false
}

async function runRelay(
  req: { flow_id: string; task: string; steps?: any[] },
  goal: string,
  emoji: string,
  flow: string,
  detail: string,
): Promise<boolean> {
  const { StartRun, AdvanceRun } = useRelayStoreStore()
  const store = useForgeStoreStore()
  try {
    const runId = await StartRun(req)
    if (runId) {
      await AdvanceRun(runId)
      // 推一条 assistant 提示消息（参照原生 sendMessage 的消息格式）
      store.messages.value.push({
        id: `${flow}-${runId}`,
        role: 'assistant',
        content: `${emoji} **Relay 工作流已启动**\n\n**目标**: ${goal}\n**Run ID**: \`${runId}\`\n**Flow**: ${flow}\n\n${detail}`,
        timestamp: Date.now(),
        profession_id: 'assistant',
      })
    }
  } catch (e) {
    store.error.value = `命令执行失败: ${e}`
  }
  return true
}
