// PLAN-061 T19 (D25 ⭐): chatActivePath 叶子包含回归门禁。
// 症状:重建会话 10 条只渲染 9 条——主路径头段 [0..anchor] + chain
// 回走最远只到叶子父节点,叶子本身从未 append;仅 chain.length==0 的
// 早退分支含叶。修复(forge_helpers.at chain 走完后 push 叶子)后全绿。
import { describe, it, expect } from 'vitest'
import { chatActivePath } from '../ext/src/front/forge_helpers'

// 线性链 m0→m1→…→m9(leaf=m9)
function linearChain(n: number) {
  const msgs = []
  for (let i = 0; i < n; i++) {
    msgs.push({ id: `m${i}`, parent_id: i === 0 ? null : `m${i - 1}`, role: 'user', content: `c${i}` })
  }
  return msgs
}

describe('chatActivePath (D25 leaf inclusion)', () => {
  it('linear chain keeps the leaf (10 → 10, last id matches)', () => {
    const msgs = linearChain(10)
    const out = chatActivePath(msgs, 'm9') as Array<{ id: string }>
    expect(out).toHaveLength(10)
    expect(out[out.length - 1].id).toBe('m9')
    expect(out.map((m) => m.id)).toEqual(['m0', 'm1', 'm2', 'm3', 'm4', 'm5', 'm6', 'm7', 'm8', 'm9'])
  })

  it('branched chain keeps the leaf and excludes the sibling', () => {
    // m0→m1→m2;m3(未选兄弟)与 m4(叶)同为 m2 子节点
    const msgs = [
      { id: 'm0', parent_id: null },
      { id: 'm1', parent_id: 'm0' },
      { id: 'm2', parent_id: 'm1' },
      { id: 'm3', parent_id: 'm2' },
      { id: 'm4', parent_id: 'm2' },
    ]
    const out = chatActivePath(msgs, 'm4') as Array<{ id: string }>
    expect(out.map((m) => m.id)).toEqual(['m0', 'm1', 'm2', 'm4'])
  })

  it('leaf-is-root (no parent) degenerates to inclusive prefix', () => {
    const msgs = [
      { id: 'm0', parent_id: null },
      { id: 'm1', parent_id: null },
    ]
    const out = chatActivePath(msgs, 'm0') as Array<{ id: string }>
    expect(out.map((m) => m.id)).toEqual(['m0'])
  })
})
