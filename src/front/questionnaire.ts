// questionnaire.ts — 消息内问卷解析（逃生舱，从 web/src/views/ChatsView.vue:829-1034 移植）
//
// Plan 022 Phase 7.3: 纯前端函数，解析 assistant 消息内容里的问卷（JSON 代码块 /
// free-text 问题 / markdown 表格），供 QuestionnaireCard 渲染。零 store/SSE 依赖。

export interface Question {
  id: string
  text: string
  type: 'single' | 'multiple' | 'text'
  options?: string[]
  placeholder?: string
  optional?: boolean
  otherLabel?: string
  otherPlaceholder?: string
}

export interface QuestionnaireResult {
  questions: Question[]
  strippedContent: string
}

interface MsgLike {
  role: string
  content: string
}

/**
 * Parse questionnaire from a message, returning questions + stripped content.
 * Mirrors web/src/views/ChatsView.vue:829 questionnaireFor.
 */
export function questionnaireFor(msg: MsgLike): QuestionnaireResult | undefined {
  if (msg.role !== 'assistant') return undefined

  // 1. Try structured JSON block first
  const blockRegex = /```json\s*\n([\s\S]*?)\n\s*```/g
  let match: RegExpExecArray | null
  while ((match = blockRegex.exec(msg.content)) !== null) {
    try {
      const json = JSON.parse(match[1].trim())
      if (json.type === 'questionnaire' && Array.isArray(json.questions) && json.questions.length > 0) {
        const stripped = msg.content.replace(match[0], '').trim()
        return { questions: json.questions, strippedContent: stripped }
      }
    } catch { /* ignore invalid JSON */ }
  }

  // 2. Fallback: detect free-text questions with optional sub-bullet options
  const lines = msg.content.split('\n')
  const questions: Question[] = []
  let consumedLines: number[] = []

  function isBullet(line: string): boolean {
    return /^\s*(?:[-*•])\s+/.test(line)
  }
  function isNumbered(line: string): boolean {
    return /^\s*\d+\.\s+/.test(line)
  }
  function stripNumbering(text: string): string {
    return text.replace(/^\s*\d+\.\s+/, '').trim()
  }
  function stripMarkdown(text: string): string {
    return text.replace(/\*\*/g, '').trim()
  }

  // Pass 1: scan for parent questions followed by child bullet options
  let i = 0
  while (i < lines.length) {
    const line = lines[i].trim()
    if (!line || line.startsWith('```') || consumedLines.includes(i)) { i++; continue }

    const isNumberedQuestion = isNumbered(line) && line.includes('?') && line.length > 10
    const isStandaloneQuestion = !isNumbered(line) && !isBullet(line) && line.endsWith('?') && line.length > 15
    const isParentCandidate = isNumberedQuestion || isStandaloneQuestion

    if (isParentCandidate) {
      const childOptions: string[] = []
      let j = i + 1
      while (j < lines.length) {
        const nextLine = lines[j]
        if (isBullet(nextLine)) {
          const optText = stripMarkdown(nextLine.replace(/^\s*(?:[-*•])\s+/, '').trim())
          if (optText) childOptions.push(optText)
          j++
        } else if (nextLine.trim() === '') { j++ } else { break }
      }
      const parentText = stripMarkdown(stripNumbering(line))
      if (childOptions.length >= 2) {
        questions.push({ id: `q${questions.length + 1}`, text: parentText, type: 'multiple', options: childOptions, otherLabel: 'Other:', otherPlaceholder: 'Type additional details...' })
        consumedLines.push(i); for (let k = i + 1; k < j; k++) consumedLines.push(k); i = j; continue
      } else if (childOptions.length === 1) {
        questions.push({ id: `q${questions.length + 1}`, text: parentText, type: 'single', options: [childOptions[0]], otherLabel: 'Other:', otherPlaceholder: 'Type additional details...' })
        consumedLines.push(i); for (let k = i + 1; k < j; k++) consumedLines.push(k); i = j; continue
      } else {
        questions.push({ id: `q${questions.length + 1}`, text: parentText, type: 'text', placeholder: 'Type your answer...' })
        consumedLines.push(i); i++; continue
      }
    }
    i++
  }

  // Pass 2: detect inline options after colon (comma or "or" separated)
  for (let i2 = 0; i2 < lines.length; i2++) {
    if (consumedLines.includes(i2)) continue
    const line = lines[i2].trim()
    if (!line.endsWith('?') || line.startsWith('```') || line.length <= 15) continue
    const colonMatch = line.match(/^(.+?)[:：\u2014\u2013\u2015-]\s*(.+)\?\s*$/)
    if (colonMatch) {
      const label = stripMarkdown(colonMatch[1]).trim()
      const optionsText = colonMatch[2]
      const opts = optionsText.split(/[,，、/\/]|\s+or\s+/).map(s => stripMarkdown(s).trim()).filter(s => s.length > 0 && s.toLowerCase() !== 'etc')
      if (opts.length >= 2) {
        questions.push({ id: `q${questions.length + 1}`, text: label, type: 'single', options: opts, otherLabel: 'Other:', otherPlaceholder: 'Type additional details...' })
        consumedLines.push(i2)
      }
    }
  }

  // Pass 3: detect markdown tables that act as questionnaires (rows with ? placeholders)
  if (questions.length === 0) {
    let tableStart = -1
    let tableEnd = -1
    for (let i3 = 0; i3 < lines.length; i3++) {
      if (lines[i3].trim().startsWith('|') && lines[i3].includes('|', lines[i3].indexOf('|') + 1)) {
        if (tableStart === -1) tableStart = i3
        tableEnd = i3
      } else if (tableStart !== -1 && lines[i3].trim() === '') { break }
    }
    if (tableStart !== -1 && tableEnd > tableStart + 1) {
      const tableLines = lines.slice(tableStart, tableEnd + 1)
      const dataRows = tableLines.filter(l => !l.includes('---') && !l.includes(':--'))
      if (dataRows.length >= 2) {
        const headerCells = dataRows[0].split('|').map(c => c.trim()).filter(Boolean)
        const hasPriorityHeader = headerCells.some(h => /priority/i.test(h))
        const rows = dataRows.slice(1)
        const hasPlaceholders = rows.some(r => r.includes('?') || r.includes('???'))
        if (hasPriorityHeader || hasPlaceholders) {
          const precedingText = lines.slice(0, tableStart).join('\n')
          const p0Match = precedingText.match(/P0\s*\(([^)]+)\)/)
          const p1Match = precedingText.match(/P1\s*\(([^)]+)\)/)
          const p2Match = precedingText.match(/P2\s*\(([^)]+)\)/)
          let priorityOptions = ['P0 (Critical)', 'P1 (Important)', 'P2 (Nice-to-have)']
          if (p0Match && p1Match && p2Match) {
            priorityOptions = [`P0 (${p0Match[1]})`, `P1 (${p1Match[1]})`, `P2 (${p2Match[1]})`]
          }
          for (const row of rows) {
            const cells = row.split('|').map(c => c.trim()).filter(Boolean)
            if (cells.length >= 2) {
              const featureName = cells[0].replace(/\*\*/g, '').trim()
              if (featureName && !featureName.toLowerCase().includes('other')) {
                questions.push({ id: `q${questions.length + 1}`, text: `Priority for "${featureName}"`, type: 'single', options: priorityOptions })
              }
            }
          }
          consumedLines = Array.from({ length: tableEnd - tableStart + 1 }, (_, idx) => tableStart + idx)
        }
      }
    }
  }

  if (questions.length >= 2) {
    const remaining = lines.filter((_, idx) => !consumedLines.includes(idx)).join('\n').trim()
    return { questions, strippedContent: remaining }
  }
  return undefined
}

/** 便捷谓词：消息是否含问卷。 */
export function hasQuestionnaire(msg: MsgLike): boolean {
  return questionnaireFor(msg) !== undefined
}

/** 便捷取值：消息的 questions 数组（无则空数组）。供 .at 绑定用。 */
export function getQuestions(msg: MsgLike): Question[] {
  return questionnaireFor(msg)?.questions ?? []
}
