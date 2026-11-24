import { describe, expect, it } from 'vitest'

import type { SheetCells, VbaComponent } from '../types/models'
import { searchAll } from './searchAll'

const sheets: SheetCells[] = [
  {
    name: 'Aba',
    cells: {
      A1: 'Total',
      B1: { formula: '=SOMA(A2:A9)', valor: 10 },
      C1: null,
    },
    tables: [],
    sampled: null,
  },
]
const vba: VbaComponent[] = [
  {
    name: 'M',
    kind: 'Module',
    code: 'Sub A()\n  total = 1\nEnd Sub',
    line_count: 3,
    procedures: [],
  },
]

describe('searchAll', () => {
  it('acha em células (valor e fórmula) e em VBA, sem diferenciar maiúsculas', () => {
    const r = searchAll(sheets, vba, 'TOTAL')
    expect(r.hits.map((h) => h.type)).toEqual(['cell', 'vba'])
    expect(searchAll(sheets, vba, 'soma').hits).toHaveLength(1)
  })

  it('consulta vazia não acha nada e o limite corta mas conta o total', () => {
    expect(searchAll(sheets, vba, '  ').total).toBe(0)
    const r = searchAll(sheets, vba, 't', 1)
    expect(r.hits).toHaveLength(1)
    expect(r.total).toBeGreaterThan(1)
  })
})
