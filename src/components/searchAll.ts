import type { SheetCells, VbaComponent } from '../types/models'
import { fmt } from './cellFormat'

export type Hit =
  | { type: 'cell'; sheet: string; cell: string; text: string }
  | { type: 'vba'; component: string; line: number; text: string }

export interface SearchResult {
  hits: Hit[]
  total: number
}

export function searchAll(
  sheets: SheetCells[],
  vba: VbaComponent[],
  query: string,
  limit = 200,
): SearchResult {
  const q = query.trim().toLowerCase()
  const hits: Hit[] = []
  let total = 0
  if (!q) return { hits, total }
  const add = (h: Hit) => {
    total++
    if (hits.length < limit) hits.push(h)
  }

  for (const s of sheets) {
    for (const [cell, c] of Object.entries(s.cells)) {
      const text =
        typeof c === 'object' && c !== null
          ? `${c.formula} ${fmt(c.valor)}`
          : fmt(c)
      if (text.toLowerCase().includes(q))
        add({ type: 'cell', sheet: s.name, cell, text })
    }
  }
  for (const comp of vba) {
    comp.code.split('\n').forEach((line, i) => {
      if (line.toLowerCase().includes(q)) {
        add({
          type: 'vba',
          component: comp.name,
          line: i + 1,
          text: line.trim(),
        })
      }
    })
  }
  return { hits, total }
}
