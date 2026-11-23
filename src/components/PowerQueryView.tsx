import { Check, Copy, Database } from 'lucide-react'
import { useEffect, useState } from 'react'

import { loadPowerQuery } from '../store/workbookActions'
import { useWorkbookStore } from '../store/workbookStore'
import type { AppError, PowerQueryItem } from '../types/models'
import { ErrorAlert } from './ErrorAlert'
import { Button } from './ui/Button'
import { EmptyState } from './ui/EmptyState'
import { highlightM, type CodeToken } from './vbaHighlighter'

function Code({ code }: { code: string }) {
  const [tokens, setTokens] = useState<CodeToken[][] | null>(null)

  useEffect(() => {
    let cancelled = false
    highlightM(code)
      .then((t) => !cancelled && setTokens(t))
      .catch(() => {})
    return () => {
      cancelled = true
    }
  }, [code])

  return (
    <pre className="overflow-auto bg-code-bg p-3 font-mono text-xs">
      {tokens
        ? tokens.map((line, i) => (
            <div key={i} className="whitespace-pre">
              {line.length === 0
                ? '\u200b'
                : line.map((t, j) => (
                    <span key={j} style={{ color: t.color }}>
                      {t.content}
                    </span>
                  ))}
            </div>
          ))
        : code}
    </pre>
  )
}

function Query({ q }: { q: PowerQueryItem }) {
  const [open, setOpen] = useState(false)
  const [copied, setCopied] = useState(false)

  async function copy() {
    try {
      await navigator.clipboard.writeText(q.code)
      setCopied(true)
      setTimeout(() => setCopied(false), 1500)
    } catch {
      // ignore
    }
  }

  return (
    <details
      className="rounded-md border border-border bg-surface-muted"
      onToggle={(e) => setOpen(e.currentTarget.open)}
    >
      <summary className="cursor-pointer px-3 py-2 text-sm font-medium">
        {q.name}
      </summary>
      {open && (
        <div className="border-t border-border">
          <div className="flex justify-end px-3 py-2">
            <Button
              size="sm"
              icon={copied ? Check : Copy}
              aria-label={`Copiar código de ${q.name}`}
              onClick={copy}
            >
              {copied ? 'Copiado' : 'Copiar'}
            </Button>
          </div>
          <Code code={q.code} />
        </div>
      )}
    </details>
  )
}

export function PowerQueryView() {
  const pq = useWorkbookStore((s) => s.powerQuery)
  const [error, setError] = useState<AppError | null>(null)

  useEffect(() => {
    if (!useWorkbookStore.getState().powerQuery)
      void loadPowerQuery().then(setError)
  }, [])

  if (!pq && error)
    return (
      <div className="flex flex-col items-start gap-3 p-4">
        <ErrorAlert error={error} />
        <Button
          size="sm"
          onClick={() => {
            setError(null)
            void loadPowerQuery().then(setError)
          }}
        >
          Tentar de novo
        </Button>
      </div>
    )

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
            <Query key={q.name} q={q} />
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
