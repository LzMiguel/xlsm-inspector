import { useEffect, useState, type ReactNode } from 'react'

import { loadDiagnostics } from '../store/workbookActions'
import { useWorkbookStore } from '../store/workbookStore'
import type { AppError, CellIssue } from '../types/models'
import { ErrorAlert } from './ErrorAlert'
import { Badge } from './ui/Badge'
import { Button } from './ui/Button'

const fmtN = (n: number) => n.toLocaleString('pt-BR')

function Card({
  label,
  count,
  bad,
}: {
  label: string
  count: number
  bad?: boolean
}) {
  return (
    <div className="rounded-md border border-border bg-surface-muted px-3 py-2">
      <div
        className={`text-lg font-semibold tabular-nums ${bad && count > 0 ? 'text-danger' : ''}`}
      >
        {fmtN(count)}
      </div>
      <div className="text-xs text-muted">{label}</div>
    </div>
  )
}

function Section({
  title,
  empty,
  children,
}: {
  title: string
  empty?: boolean
  children: ReactNode
}) {
  if (empty) return null
  return (
    <section className="flex flex-col gap-1">
      <h3 className="text-sm font-semibold">{title}</h3>
      <ul className="flex flex-col divide-y divide-border rounded-md border border-border text-sm">
        {children}
      </ul>
    </section>
  )
}

function Item({ children }: { children: ReactNode }) {
  return <li className="flex items-center gap-3 px-3 py-1.5">{children}</li>
}

function Issues({
  title,
  items,
  total,
}: {
  title: string
  items: CellIssue[]
  total: number
}) {
  const goToCell = useWorkbookStore((s) => s.goToCell)
  return (
    <Section
      title={title}
      empty={items.length === 0}
    >
      {items.map((i) => (
        <Item key={`${i.sheet}!${i.cell}`}>
          <button
            type="button"
            title="Ver na guia Células"
            onClick={() => goToCell({ sheet: i.sheet, cell: i.cell })}
            className="font-mono text-accent underline focus-visible:outline-2 focus-visible:outline-accent"
          >
            {i.sheet}!{i.cell}
          </button>
          <span
            className="min-w-0 flex-1 truncate font-mono text-muted"
            title={i.detail}
          >
            {i.detail}
          </span>
        </Item>
      ))}
      {total > items.length && (
        <Item>
          <span className="text-muted">
            … e mais {fmtN(total - items.length)} (mostrando{' '}
            {fmtN(items.length)})
          </span>
        </Item>
      )}
    </Section>
  )
}

export function DiagnosticsView() {
  const d = useWorkbookStore((s) => s.diagnostics)
  const [error, setError] = useState<AppError | null>(null)

  useEffect(() => {
    if (!useWorkbookStore.getState().diagnostics)
      void loadDiagnostics().then(setError)
  }, [])

  if (!d && error)
    return (
      <div className="flex flex-col items-start gap-3 p-4">
        <ErrorAlert error={error} />
        <Button
          size="sm"
          onClick={() => {
            setError(null)
            void loadDiagnostics().then(setError)
          }}
        >
          Tentar de novo
        </Button>
      </div>
    )

  if (!d)
    return (
      <p className="p-6 text-center text-sm text-muted">
        Analisando a planilha…
      </p>
    )

  const clean =
    d.error_count +
      d.external_count +
      d.names.length +
      d.hidden_sheets.length +
      d.tables.length +
      d.pivots.length ===
    0

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-auto p-4">
      <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-6">
        <Card
          label="Erros de fórmula"
          count={d.error_count}
          bad
        />
        <Card
          label="Vínculos externos"
          count={d.external_count}
          bad
        />
        <Card
          label="Planilhas ocultas"
          count={d.hidden_sheets.length}
        />
        <Card
          label="Nomes definidos"
          count={d.names.length}
        />
        <Card
          label="Tabelas"
          count={d.tables.length}
        />
        <Card
          label="Tabelas dinâmicas"
          count={d.pivots.length}
        />
      </div>
      {clean && (
        <p className="text-sm text-muted">Nada a destacar nesta planilha.</p>
      )}

      <Issues
        title="Células com erro"
        items={d.errors}
        total={d.error_count}
      />
      <Issues
        title="Fórmulas com referência a outro arquivo"
        items={d.external_refs}
        total={d.external_count}
      />
      <Section
        title="Planilhas ocultas"
        empty={d.hidden_sheets.length === 0}
      >
        {d.hidden_sheets.map((h) => (
          <Item key={h.name}>
            <span>{h.name}</span>
            {h.very_hidden && <Badge variant="warning">muito oculta</Badge>}
          </Item>
        ))}
      </Section>
      <Section
        title="Nomes definidos"
        empty={d.names.length === 0}
      >
        {d.names.map((n) => (
          <Item key={n.name}>
            <span className="font-mono">{n.name}</span>
            <span
              className="min-w-0 flex-1 truncate font-mono text-muted"
              title={n.refers_to}
            >
              {n.refers_to}
            </span>
          </Item>
        ))}
      </Section>
      <Section
        title="Tabelas"
        empty={d.tables.length === 0}
      >
        {d.tables.map((t) => (
          <Item key={`${t.sheet}!${t.name}`}>
            <span className="font-mono">{t.name}</span>
            <span className="text-muted">
              {t.sheet}
              {t.range && ` · ${t.range}`}
            </span>
          </Item>
        ))}
      </Section>
      <Section
        title="Tabelas dinâmicas"
        empty={d.pivots.length === 0}
      >
        {d.pivots.map((p) => (
          <Item key={`${p.sheet}!${p.name}`}>
            <span className="font-mono">{p.name}</span>
            <span className="text-muted">{p.sheet}</span>
          </Item>
        ))}
      </Section>
    </div>
  )
}
