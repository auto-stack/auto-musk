import { describe, it, expect } from 'vitest'
import { splitFrontmatter } from '../frontmatter'

describe('splitFrontmatter', () => {
  it('parses a real plan frontmatter with flat keys, inline arrays and ISO dates', () => {
    const content = [
      '---',
      'plan_id: PLAN-033',
      'status: executing',
      'feature_name: 计划模块 UI/UX 改进',
      'author: [zhaopuming]',
      'created_at: 2026-08-22T10:40:25+08:00',
      'supersedes_spec_components: []',
      'current_step: 3',
      'total_steps: 12',
      '---',
      '',
      '# [PLAN-033] 计划模块 UI/UX 改进',
      '',
      '## 变更摘要',
      '',
      '正文内容。',
    ].join('\n')
    const r = splitFrontmatter(content)
    expect(r).not.toBeNull()
    expect(r!.meta['plan_id']).toBe('PLAN-033')
    expect(r!.meta['feature_name']).toBe('计划模块 UI/UX 改进')
    expect(r!.meta['created_at']).toBe('2026-08-22T10:40:25+08:00')
    expect(r!.meta['author']).toEqual(['zhaopuming'])
    expect(r!.meta['supersedes_spec_components']).toEqual([])
    expect(r!.meta['current_step']).toBe('3')
    // body 不含 frontmatter，保留正文；开头的空行被剥掉（首个标题紧跟渲染顶部）
    expect(r!.body).not.toContain('plan_id')
    expect(r!.body.startsWith('# [PLAN-033]')).toBe(true)
    expect(r!.body).toContain('正文内容。')
  })

  it('parses block-style list values (- item)', () => {
    const content = '---\nstatus: drafting\ntouched_goals:\n  - G1\n  - G2\n---\n\nbody'
    const r = splitFrontmatter(content)!
    expect(r.meta['touched_goals']).toEqual(['G1', 'G2'])
    expect(r.meta['status']).toBe('drafting')
  })

  it('strips surrounding quotes from values', () => {
    const content = '---\nfeature_name: "Some Name"\nnote: \'single\'\n---\nbody'
    const r = splitFrontmatter(content)!
    expect(r.meta['feature_name']).toBe('Some Name')
    expect(r.meta['note']).toBe('single')
  })

  it('handles CRLF line endings', () => {
    const content = '---\r\nplan_id: PLAN-001\r\nstatus: drafting\r\n---\r\n\r\n# Title\r\n'
    const r = splitFrontmatter(content)!
    expect(r.meta['plan_id']).toBe('PLAN-001')
    expect(r.body).toContain('# Title')
  })

  it('returns null when there is no frontmatter', () => {
    expect(splitFrontmatter('# Just a title\n\nbody')).toBeNull()
    expect(splitFrontmatter('')).toBeNull()
  })

  it('does not mistake a mid-body --- separator for frontmatter', () => {
    const content = '# Title\n\nabove\n\n---\n\nbelow'
    expect(splitFrontmatter(content)).toBeNull()
  })

  it('keeps key order as written', () => {
    const content = '---\nzzz: 1\naaa: 2\n---\nbody'
    const r = splitFrontmatter(content)!
    expect(Object.keys(r.meta)).toEqual(['zzz', 'aaa'])
  })
})
