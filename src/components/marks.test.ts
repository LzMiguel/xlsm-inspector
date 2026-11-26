import { describe, expect, it } from 'vitest'

import type { WorkbookSummary } from '../types/models'
import { hiddenTabs, itemsFilter, NOTHING_HIDDEN } from './marks'

const summary: WorkbookSummary = {
  path: 'a.xlsm',
  file_name: 'a.xlsm',
  size_bytes: 1,
  has_vba: true,
  sheets: [{ name: 'A', rows: 1, cols: 1, tables: [] }],
  vba: [
    { name: 'M1', kind: 'Module', line_count: 1 },
    { name: 'M2', kind: 'Module', line_count: 1 },
  ],
  queries: ['Q'],
  connections: ['C'],
  pq_unreadable: false,
}

describe('marcações da sidebar', () => {
  it('sem nada desmarcado, não filtra nem esconde guias', () => {
    expect(itemsFilter(summary, NOTHING_HIDDEN)).toEqual({})
    expect(hiddenTabs(summary, ['A'], NOTHING_HIDDEN)).toEqual([])
  })

  it('filtra só o tipo com algo desmarcado', () => {
    const f = itemsFilter(summary, { ...NOTHING_HIDDEN, vba: ['M1'] })
    expect(f.modules).toEqual(['M2'])
    expect(f.queries).toBeUndefined()
  })

  it('a guia some quando a seção inteira é desmarcada', () => {
    const some = { vba: ['M1'], queries: ['Q'], connections: [] }
    expect(hiddenTabs(summary, ['A'], some)).toEqual([])
    const all = { vba: ['M1', 'M2'], queries: ['Q'], connections: ['C'] }
    expect(hiddenTabs(summary, [], all)).toEqual(['cells', 'vba', 'powerquery'])
  })

  it('arquivo sem VBA nem Power Query mantém as guias', () => {
    const empty = { ...summary, vba: [], queries: [], connections: [] }
    expect(hiddenTabs(empty, ['A'], NOTHING_HIDDEN)).toEqual([])
  })
})
