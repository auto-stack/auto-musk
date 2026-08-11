// workspace_helpers.ts — WorkspaceSelector 数据逻辑逃生舱（Plan 023 队列 A2）
//
// 对标 src/front/components/WorkspaceSelector.vue（逃生舱，已删除）的 loadRecent/
// loadStatus/choose 数据部分。.at 无法表达 async find/filter + localStorage 组合
// （宿主 API 链式调用 + 数组 find），放逃生舱 fn，component fn 经 use { fn } 引入。
//
// workspace_list/workspace_status 来自生成的 @/lib/api（src/back/api.at codegen）。

import { workspace_list, workspace_status } from '@/lib/api'

/** WorkspaceMeta（与 api.ts WorkspaceMeta 对齐） */
export interface WorkspaceMeta {
  id: string
  path: string
  name: string
  is_empty?: boolean
}

/** 拉取最近打开的工作区列表（对齐原 loadRecent：workspace_list + 静默失败）。 */
export async function loadRecentWorkspaces(): Promise<WorkspaceMeta[]> {
  try {
    const data = await workspace_list()
    return data.workspaces || []
  } catch {
    return []
  }
}

/**
 * 拉取当前工作区元信息（对齐原 loadStatus：localStorage wid + workspace_status）。
 * 返回 null 表示无当前工作区或读取失败（静默）。
 */
export async function loadCurrentWorkspace(): Promise<WorkspaceMeta | null> {
  const wid = localStorage.getItem('musk_workspace') || ''
  if (!wid) return null
  try {
    const data = await workspace_status(wid)
    return (data.workspace || data) as WorkspaceMeta
  } catch {
    return null
  }
}

/** 从最近列表按 id 找工作区（.at 无数组 find）。 */
export function findWorkspace(recent: WorkspaceMeta[], id: string): WorkspaceMeta | null {
  return recent.find((w) => w.id === id) || null
}
