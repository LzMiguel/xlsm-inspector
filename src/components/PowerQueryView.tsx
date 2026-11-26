import { Check, Copy, Database } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'

import { loadPowerQuery } from '../store/workbookActions'
import { useWorkbookStore } from '../store/workbookStore'
import type { AppError, PowerQueryItem } from '../types/models'
import { ErrorAlert } from './ErrorAlert'
import { visible } from './marks'
import { Button } from './ui/Button'
import { EmptyState } from './ui/EmptyState'
import type { CodeToken } from './vbaHighlighter'

function Code({ code }: { code: string }) {
  const [highlighted, setHighlighted] = useState<{
    code: string
    lines: CodeToken[][]
  } | null>(null)
  const tokens = highlighted?.code === code ? highlighted.lines : null

  useEffect(() => {
    let cancelled = false
    void import('./vbaHighlighter')
      .then(({ highlightM }) => highlightM(code))
      .then((lines) => {
        if (!cancelled) setHighlighted({ code, lines })
      })
      .catch(() => {})
    return () => {
      cancelled = true
    }
  }, [code])

  return (
    <pre className="overflow-auto bg-code-bg p-3 font-mono text-xs">
      {tokens
        ? tokens.map((line, i) => (
            <div
              key={i}
              className="whitespace-pre"
            >
              {line.length === 0
                ? '\u200b'
                : line.map((t, j) => (
                    <span
                      key={j}
                      style={{ color: t.color }}
                    >
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
  const timerRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined)

  useEffect(() => {
    return () => {
      if (timerRef.current !== undefined) clearTimeout(timerRef.current)
    }
  }, [])

  async function copy() {
    try {
      await navigator.clipboard.writeText(q.code)
      setCopied(true)
      if (timerRef.current !== undefined) clearTimeout(timerRef.current)
      timerRef.current = setTimeout(() => setCopied(false), 1500)
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
  const hidden = useWorkbookStore((s) => s.hidden)
  const [error, setError] = useState<AppError | null>(null)
  const [retrying, setRetrying] = useState(false)

  useEffect(() => {
    let cancelled = false
    if (!useWorkbookStore.getState().powerQuery) {
      void loadPowerQuery().then((err) => {
        if (!cancelled) setError(err)
      })
    }
    return () => {
      cancelled = true
    }
  }, [])

  if (!pq && error)
    return (
      <div className="flex flex-col items-start gap-3 p-4">
        <ErrorAlert error={error} />
        <Button
          size="sm"
          loading={retrying}
          onClick={() => {
            setError(null)
            setRetrying(true)
            void loadPowerQuery().then((err) => {
              setRetrying(false)
              setError(err)
            })
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

  const queries = visible(pq.queries, hidden.queries)
  const connections = visible(pq.connections, hidden.connections)

  if (queries.length === 0 && connections.length === 0)
    return (
      <EmptyState
        icon={Database}
        title="Sem Power Query"
        description="Este arquivo não tem consultas nem conexões do Power Query."
      />
    )

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-auto p-4">
      {queries.length > 0 && (
        <section className="flex flex-col gap-3">
          <h3 className="text-sm font-semibold">
            Consultas ({queries.length})
          </h3>
          {queries.map((q) => (
            <Query
              key={q.name}
              q={q}
            />
          ))}
        </section>
      )}
      {connections.length > 0 && (
        <section className="flex flex-col gap-3">
          <h3 className="text-sm font-semibold">
            Conexões ({connections.length})
          </h3>
          {connections.map((c, i) => (
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
