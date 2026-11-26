import { describe, expect, it } from 'vitest'

import { sheetSize } from './sheetSize'

const size = (rows: number, cols: number) =>
  sheetSize({ name: 'A', rows, cols, tables: [] })

describe('sheetSize', () => {
  it('abrevia, destaca as grandes e trata 1 × 1 como não informado', () => {
    expect(size(1, 1).label).toBe('—')
    expect(size(10, 4)).toMatchObject({ label: '40', large: false })
    expect(size(100_000, 12).label).toMatch(/^1,2\smi$/)
    expect(size(100_000, 12).large).toBe(true)
    expect(size(500_000, 1).large).toBe(false)
  })
})
