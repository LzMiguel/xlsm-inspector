import { useDeferredValue, useEffect, useMemo, useRef, useState } from 'react'
import { ArrowLeft, Search } from 'lucide-react'
import { useVirtualizer } from '@tanstack/react-virtual'

import type { CellContent, TableInfo } from '../types/models'
import { Button } from './ui/Button'
import { Checkbox } from './ui/Checkbox'
import { Input } from './ui/Fields'
import { fmt } from './cellFormat'

const ROW_HEIGHT = 32
const ROW_HEIGHT_FORMULA = 52
const CHAR_PX = 7.5
const MIN_COL = 72
const MAX_COL_TRUNCATED = 280
const MAX_COL_FULL = 520
const GAP = 12
const PAD = 12

interface Cell {
  formula: string
  value: string
}

const toCell = (c: CellContent): Cell =>
  typeof c === 'object' && c !== null
    ? { formula: c.formula.replace(/\r?\n/g, ' ↵ '), value: fmt(c.valor) }
    : { formula: '', value: fmt(c) }

export function TableView({
  table,
  onBack,
}: {
  table: TableInfo
  onBack: () => void
}) {
  const [query, setQuery] = useState('')
  const deferred = useDeferredValue(query)
  const scrollRef = useRef<HTMLDivElement>(null)

  const rows = useMemo(() => table.rows.map((r) => r.map(toCell)), [table])
  const hasFormula = useMemo(
    () => rows.some((r) => r.some((c) => c.formula)),
    [rows],
  )
  const [truncate, setTruncate] = useState(true)
  const rowHeight = hasFormula ? ROW_HEIGHT_FORMULA : ROW_HEIGHT

  const maxLens = useMemo(
    () =>
      table.columns.map((name, i) => {
        let len = name.length
        for (const r of rows) {
          const c = r[i]
          if (!c) continue
          len = Math.max(len, c.value.length, c.formula.length)
        }
        return len
      }),
    [table, rows],
  )
  const widths = useMemo(() => {
    const max = truncate ? MAX_COL_TRUNCATED : MAX_COL_FULL
    return maxLens.map((len) =>
      Math.min(max, Math.max(MIN_COL, Math.ceil(len * CHAR_PX))),
    )
  }, [maxLens, truncate])
  const filtered = useMemo(() => {
    const q = deferred.trim().toLowerCase()
    return q
      ? rows.filter((r) =>
          r.some(
            (c) =>
              c.value.toLowerCase().includes(q) ||
              c.formula.toLowerCase().includes(q),
          ),
        )
      : rows
  }, [rows, deferred])

  useEffect(() => {
    scrollRef.current?.scrollTo({ top: 0 })
  }, [deferred])

  const virtualizer = useVirtualizer({
    count: filtered.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => (truncate ? rowHeight : ROW_HEIGHT),
    getItemKey: (i) => `${truncate}:${i}`,
    overscan: 12,
  })

  const grid = {
    gridTemplateColumns: widths.map((w) => `${w}px`).join(' '),
  }
  const minWidth =
    widths.reduce((n, w) => n + w, 0) + (widths.length - 1) * GAP + PAD * 2

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex items-center gap-3 p-3">
        <Button
          variant="secondary"
          size="sm"
          icon={ArrowLeft}
          onClick={onBack}
        >
          Voltar
        </Button>
        <span className="font-mono text-sm">
          {table.range} | {table.name}
        </span>
        <Checkbox
          label="Truncar células"
          checked={truncate}
          onChange={setTruncate}
          className="ml-auto"
        />
        <label className="relative w-72">
          <Search
            size={16}
            strokeWidth={1.75}
            className="pointer-events-none absolute top-2 left-2 text-subtle"
            aria-hidden
          />
          <Input
            type="search"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Buscar na tabela"
            aria-label="Buscar na tabela"
            className="h-8 py-0 pr-2 pl-8 placeholder:text-subtle"
          />
        </label>
        <span className="shrink-0 text-sm text-muted tabular-nums">
          {filtered.length.toLocaleString('pt-BR')} de{' '}
          {rows.length.toLocaleString('pt-BR')}
        </span>
      </div>

      <div
        ref={scrollRef}
        tabIndex={0}
        className="min-h-0 flex-1 overflow-auto border-t border-border"
      >
        <div
          style={{ width: minWidth }}
          className="min-w-full"
          role="table"
          aria-label={`Tabela ${table.name}`}
        >
          <div
            role="row"
            style={grid}
            className="sticky top-0 z-10 grid h-8 min-w-full items-center gap-3 border-b border-border bg-surface-muted px-3 text-xs font-medium text-muted"
          >
            {table.columns.map((c, i) => (
              <span
                key={i}
                role="columnheader"
                className={truncate ? 'truncate' : 'wrap-break-word'}
                title={c}
              >
                {c}
              </span>
            ))}
          </div>
          <div
            style={{ height: virtualizer.getTotalSize() }}
            className="relative w-full"
          >
            {virtualizer.getVirtualItems().map((item) => (
              <div
                key={item.key}
                ref={truncate ? undefined : virtualizer.measureElement}
                data-index={item.index}
                role="row"
                style={{
                  ...grid,
                  transform: `translateY(${item.start}px)`,
                  ...(truncate
                    ? { height: rowHeight }
                    : { minHeight: ROW_HEIGHT }),
                }}
                className={`absolute top-0 left-0 grid w-full gap-3 border-b border-border px-3 text-sm ${
                  truncate ? 'items-center' : 'items-start py-1.5'
                }`}
              >
                {filtered[item.index].map((c, i) => (
                  <span
                    key={i}
                    role="cell"
                    title={truncate ? c.formula || c.value : undefined}
                    className="min-w-0"
                  >
                    {c.formula && (
                      <span
                        className={`block font-mono text-xs text-code-keyword ${
                          truncate ? 'truncate' : 'break-all'
                        }`}
                      >
                        {c.formula}
                      </span>
                    )}
                    <span
                      className={`block tabular-nums ${
                        truncate ? 'truncate' : 'wrap-break-word'
                      }`}
                    >
                      {c.value}
                    </span>
                  </span>
                ))}
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  )
}
