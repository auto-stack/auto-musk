// workspace_helpers.ts — WorkspaceSelector 数据逻辑逃生舱（Plan 023 队列 A2）
//
// 对标 src/front/components/WorkspaceSelector.vue（逃生舱，已删除）的 loadRecent/
// loadStatus/choose 数据部分。.at 无法表达 async find/filter + localStorage 组合
// （宿主 API 链式调用 + 数组 find），放逃生舱 fn，component fn 经 use { fn } 引入。
//
// workspace_list/workspace_status 来自生成的 @/lib/api（src/back/api.at codegen）。

import { workspace_list, workspace_status, workspace_browse, workspace_open } from '@/lib/api'

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

/** 浏览目录条目（对齐原 browse：/api/workspace/browse?path）。仅当 path 含路径分隔符
 *  时才查询（避免短输入返回根目录长列表）；静默失败返回 []。 */
export async function browseWorkspace(path: string): Promise<{ name: string; path: string }[]> {
  if (!path || (!path.includes('/') && !path.includes('\\'))) return []
  try {
    const data = await workspace_browse(path)
    return data.entries || []
  } catch {
    return []
  }
}

/**
 * 打开/切换到指定路径的 workspace（对齐原 openWorkspace：/api/workspace/open）。
 * 成功返回新 workspace 的 meta；失败返回 null（静默）。
 */
export async function openWorkspace(path: string): Promise<WorkspaceMeta | null> {
  try {
    return await workspace_open({ path })
  } catch {
    return null
  }
}
