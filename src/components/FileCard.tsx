import { FileSpreadsheet, House } from 'lucide-react'

import { chooseAndOpenWorkbook } from '../store/workbookActions'
import { useWorkbookStore } from '../store/workbookStore'
import type { WorkbookSummary } from '../types/models'
import { Badge } from './ui/Badge'
import { Button } from './ui/Button'

function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`
  return `${(n / (1024 * 1024)).toFixed(1)} MB`
}

export function FileCard({
  summary,
  disabled,
}: {
  summary: WorkbookSummary
  disabled: boolean
}) {
  const n = summary.sheets.length
  const reset = useWorkbookStore((s) => s.reset)
  return (
    <section className="flex shrink-0 items-center gap-4 rounded-md border border-border bg-surface p-4">
      <FileSpreadsheet
        size={20}
        strokeWidth={1.75}
        className="shrink-0 text-primary"
        aria-hidden
      />
      <div className="min-w-0 flex-1">
        <p
          className="truncate text-base font-medium"
          title={summary.path}
        >
          {summary.file_name}
        </p>
        <p className="text-sm text-muted">
          {formatBytes(summary.size_bytes)} · {n}{' '}
          {n === 1 ? 'planilha' : 'planilhas'}
        </p>
      </div>
      {summary.has_vba && <Badge variant="primary">VBA</Badge>}
      <Button
        variant="ghost"
        size="sm"
        icon={House}
        disabled={disabled}
        onClick={reset}
      >
        Início
      </Button>
      <Button
        variant="secondary"
        size="sm"
        disabled={disabled}
        onClick={chooseAndOpenWorkbook}
      >
        Trocar arquivo
      </Button>
    </section>
  )
}
