// types/plans.ts — PLAN-024: Plan 数据模型（对齐后端 plans.rs PlanFile/MergeResult）。

export type PlanStatus =
  | 'drafting'
  | 'executing'
  | 'execution_done'
  | 'review_done'
  | 'merged'

export interface PlanFile {
  /** `PLAN-024`（= 文件名前缀 024）。 */
  id: string
  /** 3 位序号的数值（24）。 */
  seq: number
  /** 文件名（`024-xxx.md`）。 */
  filename: string
  status: PlanStatus
  feature_name: string
  /** 正文首行标题（`# [PLAN-024] xxx`），无则空。 */
  title: string
  /** 是否位于 `archived/` 子目录。 */
  archived: boolean
  /** 完整 markdown（含 frontmatter）。 */
  content: string
  /** frontmatter `created_at`（ISO 字符串）。 */
  created_at: string
  /** frontmatter `updated_at`（ISO 字符串）。 */
  updated_at: string
  /** 相对 `plans_dir` 的路径。 */
  path: string
}

export interface PlansListResponse {
  plans: PlanFile[]
}

export interface CreatePlanRequest {
  feature_name: string
  content?: string
}

export interface UpdatePlanRequest {
  content: string
}

export interface TransitionPlanRequest {
  status: PlanStatus
}

export interface MergeResult {
  plan_id: string
  sections_touched: string[]
  items_created: number
}

/**
 * 状态机合法迁移（对齐后端 `PlanStatus::can_transition`，008 §7.2）。
 * 幂等（from === to）总是合法。
 */
export function canTransition(from: PlanStatus, to: PlanStatus): boolean {
  if (from === to) return true
  const allowed: Record<PlanStatus, PlanStatus[]> = {
    drafting: ['executing', 'review_done'],
    executing: ['execution_done', 'drafting'],
    execution_done: ['review_done', 'executing'],
    review_done: ['merged', 'executing'],
    merged: [],
  }
  return (allowed[from] || []).includes(to)
}

/** 状态徽标的 CSS class 后缀（用于 PlanStatusBadge 配色）。 */
export const STATUS_TONE: Record<PlanStatus, string> = {
  drafting: 'muted',
  executing: 'info',
  execution_done: 'info',
  review_done: 'warn',
  merged: 'success',
}
