import { useState } from 'react'
import { FunctionSquare } from 'lucide-react'

import { useWorkbookStore } from '../store/workbookStore'
import type { SheetInfo } from '../types/models'
import { Button } from './ui/Button'
import { Checkbox } from './ui/Checkbox'
import { Input } from './ui/Fields'

const EMPTY_SHEETS: SheetInfo[] = []

export function SheetList() {
  const sheets = useWorkbookStore((s) => s.summary?.sheets) ?? EMPTY_SHEETS
  const selected = useWorkbookStore((s) => s.selectedSheets)
  const busy = useWorkbookStore(
    (s) => s.status === 'extracting' || s.status === 'loading',
  )
  const toggle = useWorkbookStore((s) => s.toggleSheet)
  const setAll = useWorkbookStore((s) => s.setAllSheets)

  const [query, setQuery] = useState('')
  const q = query.trim().toLowerCase()
  const visible = q
    ? sheets.filter((s) => s.name.toLowerCase().includes(q))
    : sheets

  const all = selected.length === sheets.length

  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-center justify-between">
        <h2 className="text-sm font-semibold">
          Planilhas{' '}
          <span className="font-normal text-muted">
            ({selected.length}/{sheets.length})
          </span>
        </h2>
        <Button
          size="sm"
          variant="ghost"
          disabled={busy}
          onClick={() => setAll(!all)}
        >
          {all ? 'Desmarcar todas' : 'Marcar todas'}
        </Button>
      </div>

      <p className="text-xs text-muted">
        Marque as planilhas que serão exportadas.
      </p>
      {sheets.length > 8 && (
        <Input
          type="search"
          aria-label="Filtrar planilhas"
          placeholder="Filtrar planilhas…"
          className="py-1"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
      )}

      <ul className="flex flex-col gap-1">
        {visible.length === 0 && (
          <li className="text-xs text-muted">Nenhuma planilha encontrada.</li>
        )}
        {visible.map((s) => (
          <li key={s.name}>
            <Checkbox
              className="w-full rounded-sm px-1 py-1 hover:bg-surface-muted"
              checked={selected.includes(s.name)}
              disabled={busy}
              onChange={() => toggle(s.name)}
              label={
                <span className="flex min-w-0 flex-1 items-center gap-2">
                  <span
                    className="flex-1 truncate"
                    title={s.name}
                  >
                    {s.name}
                  </span>
                  {s.formula_count > 0 && (
                    <FunctionSquare
                      size={14}
                      strokeWidth={1.75}
                      className="shrink-0 text-accent"
                      aria-label={`${s.formula_count} fórmulas`}
                    />
                  )}
                  <span className="shrink-0 font-mono text-xs text-muted tabular-nums">
                    {s.cell_count.toLocaleString('pt-BR')}
                  </span>
                </span>
              }
            />
          </li>
        ))}
      </ul>
    </div>
  )
}
