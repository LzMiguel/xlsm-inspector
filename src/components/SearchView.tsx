import { useDeferredValue, useMemo, useRef, useState } from 'react'
import { Search } from 'lucide-react'

import { useHotkey } from '../hooks/useHotkey'
import { useWorkbookStore } from '../store/workbookStore'
import type { SheetCells, VbaComponent } from '../types/models'
import { visible } from './marks'
import { searchAll } from './searchAll'
import { Badge } from './ui/Badge'
import { EmptyState } from './ui/EmptyState'
import { Input } from './ui/Fields'

const EMPTY_SHEETS: SheetCells[] = []
const EMPTY_VBA: VbaComponent[] = []

export function SearchView() {
  const result = useWorkbookStore((s) => s.result)
  const selected = useWorkbookStore((s) => s.selectedSheets)
  const hidden = useWorkbookStore((s) => s.hidden.vba)
  const goToCell = useWorkbookStore((s) => s.goToCell)
  const goToVba = useWorkbookStore((s) => s.goToVba)
  const [query, setQuery] = useState('')
  const deferred = useDeferredValue(query)
  const searchRef = useRef<HTMLInputElement>(null)

  useHotkey('f', () => searchRef.current?.focus())

  const { hits, total } = useMemo(
    () =>
      searchAll(
        (result?.sheets ?? EMPTY_SHEETS).filter((s) =>
          selected.includes(s.name),
        ),
        visible(result?.vba ?? EMPTY_VBA, hidden),
        deferred,
      ),
    [result, selected, hidden, deferred],
  )

  if (!result) {
    return (
      <EmptyState
        icon={Search}
        title="Leia o arquivo para buscar"
        description="A busca vale para as células e o código VBA já lidos."
      />
    )
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex items-center gap-3 p-3">
        <Input
          ref={searchRef}
          type="search"
          autoFocus
          aria-label="Buscar em células e VBA"
          placeholder="Buscar em células e VBA…"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          className="h-8 flex-1 py-0"
        />
        <span className="text-sm text-muted tabular-nums">
          {total.toLocaleString('pt-BR')} resultado(s)
        </span>
      </div>
      <ul className="min-h-0 flex-1 divide-y divide-border overflow-auto border-t border-border text-sm">
        {hits.map((h, i) => (
          <li key={i}>
            <button
              type="button"
              onClick={() =>
                h.type === 'cell'
                  ? goToCell({ sheet: h.sheet, cell: h.cell })
                  : goToVba(h.component, h.line)
              }
              className="flex w-full items-center gap-3 px-3 py-1.5 text-left hover:bg-surface-muted focus-visible:outline-2 focus-visible:outline-accent"
            >
              <Badge>{h.type === 'cell' ? 'Célula' : 'VBA'}</Badge>
              <span className="shrink-0 font-mono text-accent">
                {h.type === 'cell'
                  ? `${h.sheet}!${h.cell}`
                  : `${h.component}:${h.line}`}
              </span>
              <span className="min-w-0 flex-1 truncate font-mono text-muted">
                {h.text}
              </span>
            </button>
          </li>
        ))}
        {total > hits.length && (
          <li className="px-3 py-1.5 text-muted">
            … mostrando {hits.length} de {total.toLocaleString('pt-BR')}. Refine
            a busca.
          </li>
        )}
      </ul>
    </div>
  )
}
