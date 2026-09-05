// PLAN-061 T1 (D1): specSectionItems 大小写归一回归门禁。
// 后端 GET /api/specs 返回 section_type 首字母大写("Goals"),
// 前端以小写("goals")查询——全等比较永不匹配,结构化视图恒空。
// 修复(src/front/specs_helpers.at 归一化 + 重生成)后本 spec 须全绿;
// 人为回退大小写敏感比较应转红。
import { describe, it, expect } from 'vitest'
import { specSectionItems } from '../ext/src/front/specs_helpers'

describe('specSectionItems (D1 case normalization)', () => {
  it('matches backend "Goals" against query "goals"', () => {
    const doc = { sections: [{ section_type: 'Goals', items: [{ id: 'g1' }, { id: 'g2' }] }] }
    expect(specSectionItems(doc, 'goals')).toHaveLength(2)
  })

  it('matches lowercase data against capitalized query (symmetric)', () => {
    const doc = { sections: [{ section_type: 'architecture', items: [{ id: 'a1' }] }] }
    expect(specSectionItems(doc, 'Architecture')).toHaveLength(1)
  })

  it('still excludes non-matching sections and tolerates missing fields', () => {
    const doc = { sections: [
      { section_type: 'Goals', items: [{ id: 'g1' }] },
      { section_type: 'Designs', items: [{ id: 'd1' }] },
    ] }
    expect(specSectionItems(doc, 'goals')).toHaveLength(1)
    expect(specSectionItems(null, 'goals')).toHaveLength(0)
    expect(specSectionItems({ sections: [{ section_type: 'Goals' }] }, 'goals')).toHaveLength(0)
  })
})
