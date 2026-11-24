import { useEffect, useMemo, useRef, useState } from 'react'
import { Check, Code2, Copy } from 'lucide-react'
import { useVirtualizer } from '@tanstack/react-virtual'

import { useWorkbookStore } from '../store/workbookStore'
import type { VbaComponent } from '../types/models'
import { Badge } from './ui/Badge'
import { Button } from './ui/Button'
import { EmptyState } from './ui/EmptyState'
import type { CodeToken } from './vbaHighlighter'
import { KIND_LABEL } from './vbaKinds'

const LINE_HEIGHT = 20
const EMPTY_VBA: VbaComponent[] = []

function CodeViewer({ component }: { component: VbaComponent }) {
  const [highlighted, setHighlighted] = useState<{
    code: string
    lines: CodeToken[][]
  } | null>(null)
  const [copied, setCopied] = useState(false)
  const copyTimerRef = useRef<ReturnType<typeof setTimeout> | undefined>(
    undefined,
  )
  const scrollRef = useRef<HTMLDivElement>(null)

  const plain = useMemo(() => component.code.split('\n'), [component.code])
  const tokens = highlighted?.code === component.code ? highlighted.lines : null
  const count = tokens?.length ?? plain.length

  useEffect(() => {
    let cancelled = false
    void import('./vbaHighlighter')
      .then(({ highlightVba }) => highlightVba(component.code))
      .then((lines) => {
        if (!cancelled) setHighlighted({ code: component.code, lines })
      })
      .catch(() => {})
    return () => {
      cancelled = true
    }
  }, [component.code])

  useEffect(() => {
    return () => {
      if (copyTimerRef.current !== undefined) clearTimeout(copyTimerRef.current)
    }
  }, [])

  const virtualizer = useVirtualizer({
    count,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => LINE_HEIGHT,
    overscan: 20,
  })

  const vbaLine = useWorkbookStore((s) => s.vbaLine)
  const vbaJumpSeq = useWorkbookStore((s) => s.vbaJumpSeq)
  useEffect(() => {
    if (vbaLine && count > 0) {
      const target = Math.min(Math.max(vbaLine - 1, 0), count - 1)
      virtualizer.scrollToIndex(target, { align: 'start' })
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [vbaLine, vbaJumpSeq, component.name, count])

  async function copy() {
    try {
      await navigator.clipboard.writeText(component.code)
      setCopied(true)
      if (copyTimerRef.current !== undefined) clearTimeout(copyTimerRef.current)
      copyTimerRef.current = setTimeout(() => setCopied(false), 1500)
    } catch {
      // ignore
    }
  }

  const gutter = String(count).length + 1

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex items-center gap-3 border-b border-border px-4 py-2">
        <h2 className="truncate text-lg font-semibold">{component.name}</h2>
        <Badge variant="primary">{KIND_LABEL[component.kind]}</Badge>
        <span className="text-sm text-muted">
          {component.line_count} linhas
        </span>
        <Button
          size="sm"
          className="ml-auto"
          icon={copied ? Check : Copy}
          onClick={copy}
          aria-label="Copiar código"
        >
          {copied ? 'Copiado' : 'Copiar'}
        </Button>
      </div>

      <div
        ref={scrollRef}
        tabIndex={0}
        aria-label={`Código de ${component.name}`}
        className="min-h-0 flex-1 overflow-auto bg-code-bg font-mono text-sm"
      >
        <div
          style={{ height: virtualizer.getTotalSize() }}
          className="relative w-max min-w-full"
        >
          {virtualizer.getVirtualItems().map((item) => (
            <div
              key={item.key}
              style={{
                transform: `translateY(${item.start}px)`,
                height: LINE_HEIGHT,
              }}
              className="absolute top-0 left-0 flex min-w-full whitespace-pre"
            >
              <span
                aria-hidden
                style={{ width: `${gutter + 1}ch` }}
                className="sticky left-0 shrink-0 bg-code-bg pr-3 text-right text-subtle select-none"
              >
                {item.index + 1}
              </span>
              <span>
                {tokens
                  ? tokens[item.index].map((t, i) => (
                      <span
                        key={i}
                        style={{ color: t.color }}
                      >
                        {t.content}
                      </span>
                    ))
                  : plain[item.index]}
              </span>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}

export function VbaViewer() {
  const vba = useWorkbookStore((s) => s.result?.vba) ?? EMPTY_VBA
  const selected = useWorkbookStore((s) => s.selectedComponent)
  const component = vba.find((c) => c.name === selected) ?? vba[0]

  if (!component) {
    return (
      <EmptyState
        icon={Code2}
        title="Este arquivo não tem código VBA"
        description="Só arquivos .xlsm com macros têm componentes para exibir."
      />
    )
  }
  return (
    <CodeViewer
      key={component.name}
      component={component}
    />
  )
}
