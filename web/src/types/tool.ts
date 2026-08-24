export interface ToolCallInfo {
  id: string
  name: string
  arguments: Record<string, unknown>
  result?: string
  /** PLAN-040: streaming partial (tool_update SSE), appended while running */
  partial?: string
  /** PLAN-042: structured payload (edit diff / truncation / full output path) */
  details?: {
    diff?: string
    patch?: string
    first_changed_line?: number
    truncation?: { total_lines: number; output_lines: number; truncated_by: string; last_line_partial?: boolean }
    full_output_path?: string
  } | null
  status: 'pending' | 'running' | 'success' | 'error'
  _expanded?: boolean
}
