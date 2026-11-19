import { Database } from 'lucide-react'
import { useEffect } from 'react'

import { loadPowerQuery } from '../store/workbookActions'
import { useWorkbookStore } from '../store/workbookStore'
import { EmptyState } from './ui/EmptyState'

export function PowerQueryView() {
  const pq = useWorkbookStore((s) => s.powerQuery)

  useEffect(() => {
    if (!useWorkbookStore.getState().powerQuery) void loadPowerQuery()
  }, [])

  if (!pq)
    return (
      <p className="p-6 text-center text-sm text-muted">Lendo Power Query…</p>
    )

  if (pq.queries.length === 0 && pq.connections.length === 0)
    return (
      <EmptyState
        icon={Database}
        title="Sem Power Query"
        description="Este arquivo não tem consultas nem conexões do Power Query."
      />
    )

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-auto p-4">
      {pq.queries.length > 0 && (
        <section className="flex flex-col gap-3">
          <h3 className="text-sm font-semibold">
            Consultas ({pq.queries.length})
          </h3>
          {pq.queries.map((q) => (
            <details
              key={q.name}
              className="rounded-md border border-border bg-surface-muted"
            >
              <summary className="cursor-pointer px-3 py-2 text-sm font-medium">
                {q.name}
              </summary>
              <pre className="overflow-auto border-t border-border p-3 font-mono text-xs">
                {q.code}
              </pre>
            </details>
          ))}
        </section>
      )}
      {pq.connections.length > 0 && (
        <section className="flex flex-col gap-3">
          <h3 className="text-sm font-semibold">
            Conexões ({pq.connections.length})
          </h3>
          {pq.connections.map((c, i) => (
            <div
              key={`${c.name}-${i}`}
              className="rounded-md border border-border bg-surface-muted p-3"
            >
              <div className="text-sm font-medium">{c.name}</div>
              {c.description && (
                <p className="text-xs text-muted">{c.description}</p>
              )}
              {c.connection && (
                <code className="mt-2 block break-all font-mono text-xs">
                  {c.connection}
                </code>
              )}
            </div>
          ))}
        </section>
      )}
    </div>
  )
}
