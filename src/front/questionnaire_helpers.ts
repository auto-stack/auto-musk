// questionnaire_helpers.ts — QuestionnaireCard 答案记录逻辑逃生舱（Plan 023 队列 B1）
//
// 对标 src/front/components/QuestionnaireCard.vue（逃生舱，已删除）。
// .at 无法表达：动态键记录 answers 的数组 includes/toggle、canSubmit 遍历校验、
// 可选字段 || 兜底。放逃生舱 fn，component fn 经 use { fn } 引入。

/** 单选是否选中（answers[qid] === opt）。 */
export function questionnaireSingleChecked(answers: any, qid: string, opt: string): boolean {
  return answers[qid] === opt
}

/** 多选是否选中（answers[qid] 数组 includes opt）。 */
export function questionnaireMultiChecked(answers: any, qid: string, opt: string): boolean {
  return (answers[qid] || []).includes(opt)
}

/** 多选 toggle：opt 已在则移除，否则追加（返回新数组）。 */
export function questionnaireToggleAnswer(answers: any, qid: string, opt: string): string[] {
  const cur: string[] = answers[qid] || []
  return cur.includes(opt) ? cur.filter((x) => x !== opt) : [...cur, opt]
}

/** other 输入框的答案键（qid + '__other'）。 */
export function questionnaireOtherKey(qid: string): string {
  return `${qid}__other`
}

/** 是否可提交：所有非 optional 问题都有答案（对齐原 canSubmit）。 */
export function questionnaireCanSubmit(questions: any[], answers: any): boolean {
  for (const q of questions) {
    if (q.optional) continue
    const ans = answers[q.id]
    if (q.type === 'multiple') {
      if (!(ans || []).length) return false
    } else {
      if (!ans || String(ans).trim() === '') return false
    }
  }
  return true
}

/** placeholder 兜底（q.placeholder || fallback）。 */
export function questionnairePlaceholder(q: any, fallback: string): string {
  return q.placeholder || fallback
}

/** 从 input 事件提取当前值（受控组件：动态键 v-model 是 codegen 缺口，用 value:+oninput+$event 兜底）。 */
export function eventInputValue(e: any): string {
  return e?.target?.value ?? ''
}
