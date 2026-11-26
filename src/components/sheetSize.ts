import type { SheetInfo } from '../types/models'

/** Acima disso a aba é destacada como pesada. */
const LARGE_CELLS = 500_000

const compact = new Intl.NumberFormat('pt-BR', {
  notation: 'compact',
  maximumFractionDigits: 1,
})

/** Tamanho aproximado pela dimensão declarada; 1 × 1 também é o valor de "não informado". */
export function sheetSize({ rows, cols }: SheetInfo) {
  const cells = rows * cols
  const unknown = cells <= 1
  return {
    label: unknown ? '—' : compact.format(cells),
    title: unknown
      ? 'Tamanho não informado ou só uma célula'
      : `${rows.toLocaleString('pt-BR')} linhas × ${cols.toLocaleString('pt-BR')} colunas (aproximado)`,
    large: cells > LARGE_CELLS,
  }
}
