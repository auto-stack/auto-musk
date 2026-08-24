export interface ToolCallInfo {
  id: string
  name: string
  arguments: Record<string, unknown>
  result?: string
  /** PLAN-040: streaming partial (tool_update SSE), appended while running */
  partial?: string
  status: 'pending' | 'running' | 'success' | 'error'
  _expanded?: boolean
}
