// relay_command_runner.ts — relay 命令执行桥（纯接线，无状态无解析）
//
// Plan 029 T14：relay_commands.ts 的解析/文案已原生化为 relay_commands.at；
// 本桥只剩执行（RelayStore.StartRun/AdvanceRun + Forge 消息推送，跨两个
// store，受 v1「单 store per file」限制）。待多 store 放开后并入 .at 并
// 删除本文件（D 组登记）。

import { useRelayStoreStore } from '@/stores/useRelayStoreStore'
import { useForgeStoreStore } from '@/stores/useForgeStoreStore'

export function runRelayCommand(cmd: {
  flow: string
  task: string
  emoji: string
  detail: string
  steps: any[] | null
}): void {
  const { StartRun, AdvanceRun } = useRelayStoreStore()
  const store = useForgeStoreStore()
  const req: any = { flow_id: cmd.flow, task: cmd.task }
  if (cmd.steps) req.steps = cmd.steps
  void (async () => {
    try {
      const runId = await StartRun(req)
      if (runId) {
        await AdvanceRun(runId)
        store.messages.push({
          id: `${cmd.flow}-${runId}`,
          role: 'assistant',
          content: `${cmd.emoji} **Relay 工作流已启动**\n\n**目标**: ${cmd.task}\n**Run ID**: \`${runId}\`\n**Flow**: ${cmd.flow}\n\n${cmd.detail}`,
          timestamp: Date.now(),
          profession_id: 'assistant',
        })
      }
    } catch (e) {
      store.error = `命令执行失败: ${e}`
    }
  })()
}
