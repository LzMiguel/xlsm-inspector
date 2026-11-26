import type { ReactNode } from 'react'

import { useWorkbookStore } from '../store/workbookStore'
import { sheetSize } from './sheetSize'
import { KIND_LABEL } from './vbaKinds'

const fmtN = (n: number) => n.toLocaleString('pt-BR')

function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="flex flex-col gap-1">
      <h3 className="text-sm font-semibold">{title}</h3>
      <ul className="flex flex-col divide-y divide-border rounded-md border border-border text-sm">
        {children}
      </ul>
    </section>
  )
}

function Row({ name, detail }: { name: string; detail?: string }) {
  return (
    <li className="flex items-center gap-3 px-3 py-1.5">
      <span
        className="min-w-0 flex-1 truncate"
        title={name}
      >
        {name}
      </span>
      {detail && (
        <span className="shrink-0 text-muted tabular-nums">{detail}</span>
      )}
    </li>
  )
}

export function WorkbookOverview() {
  const s = useWorkbookStore((st) => st.summary)
  if (!s) return null

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-auto p-4">
      <Section title={`Planilhas (${s.sheets.length})`}>
        {s.sheets.map((sheet) => {
          const unknown = sheet.rows * sheet.cols <= 1
          const size = unknown
            ? 'tamanho não informado'
            : `${fmtN(sheet.rows)} linhas × ${fmtN(sheet.cols)} colunas (≈ ${sheetSize(sheet).label} células)`
          const tables = sheet.tables.length
            ? ` · tabelas: ${sheet.tables.join(', ')}`
            : ''
          return (
            <Row
              key={sheet.name}
              name={sheet.name}
              detail={size + tables}
            />
          )
        })}
      </Section>

      {s.has_vba && (
        <Section title={`VBA (${s.vba.length})`}>
          {s.vba.length === 0 && (
            <Row name="Não foi possível listar os módulos." />
          )}
          {s.vba.map((m) => (
            <Row
              key={m.name}
              name={m.name}
              detail={`${KIND_LABEL[m.kind]} · ${fmtN(m.line_count)} linha(s)`}
            />
          ))}
        </Section>
      )}

      {s.pq_unreadable && (
        <p className="text-sm text-warning">
          Power Query e conexões não puderam ser lidos.
        </p>
      )}
      {s.queries.length > 0 && (
        <Section title={`Consultas Power Query (${s.queries.length})`}>
          {s.queries.map((q) => (
            <Row
              key={q}
              name={q}
            />
          ))}
        </Section>
      )}
      {s.connections.length > 0 && (
        <Section title={`Conexões (${s.connections.length})`}>
          {s.connections.map((c, i) => (
            <Row
              key={`${c}-${i}`}
              name={c}
            />
          ))}
        </Section>
      )}
    </div>
  )
}
