// types/plans.ts — PLAN-024: Plan 数据模型（对齐后端 plans.rs PlanFile/MergeResult）。
// PLAN-033: review_done→reviewed、merged→archived（单一终态：状态==archived
// ⇔ 文件位于 archived/；后端对旧值做兼容读取）。

export type PlanStatus =
  | 'drafting'
  | 'executing'
  | 'execution_done'
  | 'reviewed'
  | 'archived'

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
 * 状态机合法迁移表（对齐后端 `PlanStatus::can_transition`，008 §7.2 +
 * PLAN-033 修订）。`archived` 为终态，只能经 archive（搁置）或 merge
 * （沉淀，reviewed 专属）进入——不走 transition 端点。
 */
export const ALLOWED_TRANSITIONS: Record<PlanStatus, PlanStatus[]> = {
  drafting: ['executing', 'reviewed'],
  executing: ['execution_done', 'drafting'],
  execution_done: ['reviewed', 'executing'],
  reviewed: ['executing'],
  archived: [],
}

/**
 * 状态机合法迁移校验。幂等（from === to）总是合法。
 */
export function canTransition(from: PlanStatus, to: PlanStatus): boolean {
  if (from === to) return true
  return (ALLOWED_TRANSITIONS[from] || []).includes(to)
}

/**
 * 状态 → i18n key（`execution_done` → `plans.statusExecutionDone`）。
 * 徽标与转移按钮共用，保证状态文案全栈唯一来源。
 */
export function planStatusKey(s: PlanStatus): string {
  const camel = s.replace(/_([a-z])/g, (_, c) => c.toUpperCase()).replace(/^./, (c) => c.toUpperCase())
  return `plans.status${camel}`
}

/** 状态徽标的 CSS class 后缀（用于 PlanStatusBadge 配色）。 */
export const STATUS_TONE: Record<PlanStatus, string> = {
  drafting: 'muted',
  executing: 'info',
  execution_done: 'info',
  reviewed: 'warn',
  archived: 'success',
}
