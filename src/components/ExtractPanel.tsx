import { useMemo } from 'react'
import { RefreshCw, Table2 } from 'lucide-react'

import { openWorkbook, runExtraction } from '../store/workbookActions'
import { useWorkbookStore } from '../store/workbookStore'
import type { Progress } from '../types/models'
import { Alert } from './ui/Alert'
import { Button } from './ui/Button'
import { ScopeSelector } from './ScopeSelector'
import { ProgressBar } from './ui/ProgressBar'

function progressLabel(p: Progress | null): string {
  if (!p) return 'Iniciando…'
  if (p.stage === 'vba') return 'Lendo código VBA…'
  if (!p.label) return 'Finalizando…'
  return `Lendo planilha ${p.current + 1} de ${p.total}: ${p.label}`
}

export function ExtractPanel() {
  const status = useWorkbookStore((s) => s.status)
  const progress = useWorkbookStore((s) => s.progress)
  const result = useWorkbookStore((s) => s.result)
  const warnings = useWorkbookStore((s) => s.warnings)
  const noSheets = useWorkbookStore((s) => s.selectedSheets.length === 0)
  const filePath = useWorkbookStore((s) => s.filePath)
  const fromCache = useWorkbookStore((s) => s.fromCache)
  const scope = useWorkbookStore((s) => s.scope)
  const setScope = useWorkbookStore((s) => s.setScope)
  const extracting = status === 'extracting'

  const cellCount = useMemo(
    () =>
      result?.sheets.reduce((n, s) => n + Object.keys(s.cells).length, 0) ?? 0,
    [result],
  )

  return (
    <section className="flex shrink-0 flex-col gap-4 rounded-md border border-border bg-surface p-4">
      <div className="flex flex-wrap items-center gap-4">
        <Button
          variant={result ? 'secondary' : 'primary'}
          icon={Table2}
          loading={extracting}
          disabled={status === 'loading' || noSheets}
          onClick={runExtraction}
        >
          {result ? 'Ler de novo' : 'Ler arquivo'}
        </Button>
        <ScopeSelector
          value={scope}
          onChange={setScope}
          disabled={extracting}
        />
        {result && !extracting && (
          <p className="text-sm text-muted">
            {cellCount.toLocaleString('pt-BR')} células em{' '}
            {result.sheets.length} {result.sheets.length === 1 ? 'planilha' : 'planilhas'}{' '}
            · {result.vba.length} componente(s) VBA
          </p>
        )}
        {noSheets && (
          <p className="text-sm text-warning">Marque ao menos uma planilha.</p>
        )}
      </div>

      {fromCache && !extracting && (
        <Alert
          variant="info"
          title="Resultado salvo"
          action={
            <Button
              size="sm"
              variant="secondary"
              icon={RefreshCw}
              onClick={() => filePath && openWorkbook(filePath, true)}
            >
              Recarregar
            </Button>
          }
        >
          O arquivo não foi relido: este é o resultado da última leitura, e o
          arquivo não mudou desde então. Se você o editou e o resultado parece
          antigo, clique em Recarregar.
        </Alert>
      )}

      {extracting && (
        <ProgressBar
          current={progress?.current}
          total={progress?.stage === 'cells' ? progress.total : undefined}
          label={progressLabel(progress)}
        />
      )}

      {/* Muitos avisos rolam aqui dentro e não empurram a pré-visualização. */}
      {!extracting && warnings.length > 0 && (
        <div className="flex max-h-40 flex-col gap-2 overflow-y-auto">
          {warnings.map((w, i) => (
            <Alert
              key={`${w.kind}:${w.sheet}:${i}`}
              variant={w.kind === 'Sampled' ? 'info' : 'warning'}
              title={w.sheet ? `Planilha "${w.sheet}"` : 'Aviso'}
            >
              {w.message}
            </Alert>
          ))}
        </div>
      )}
    </section>
  )
}
