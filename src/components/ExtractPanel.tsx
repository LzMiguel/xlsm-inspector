import { RefreshCw, Table2 } from 'lucide-react'

import {
  openWorkbook,
  refreshVba,
  runExtraction,
} from '../store/workbookActions'
import { updateVbaOptions } from '../store/settingsActions'
import { useSettingsStore } from '../store/settingsStore'
import { useWorkbookStore } from '../store/workbookStore'
import type { Progress, VbaOptions } from '../types/models'
import { Alert } from './ui/Alert'
import { Button } from './ui/Button'
import { Checkbox } from './ui/Checkbox'
import { ScopeSelector } from './ScopeSelector'
import { ProgressBar } from './ui/ProgressBar'

function progressLabel(p: Progress | null): string {
  if (!p) return 'Iniciando…'
  if (p.stage === 'vba') return 'Extraindo código VBA…'
  if (!p.label) return 'Finalizando…'
  return `Processando aba ${p.current + 1} de ${p.total}: ${p.label}`
}

export function ExtractPanel() {
  const status = useWorkbookStore((s) => s.status)
  const progress = useWorkbookStore((s) => s.progress)
  const result = useWorkbookStore((s) => s.result)
  const warnings = useWorkbookStore((s) => s.warnings)
  const hasVba = useWorkbookStore((s) => s.summary?.has_vba ?? false)
  const noSheets = useWorkbookStore((s) => s.selectedSheets.length === 0)
  const vbaOptions = useSettingsStore((s) => s.settings.vba_options)
  const filePath = useWorkbookStore((s) => s.filePath)
  const fromCache = useWorkbookStore((s) => s.fromCache)
  const scope = useWorkbookStore((s) => s.scope)
  const setScope = useWorkbookStore((s) => s.setScope)
  const extracting = status === 'extracting'

  const changeOption = (o: Partial<VbaOptions>) => {
    updateVbaOptions(o)
    if (result) refreshVba()
  }

  const cellCount =
    result?.sheets.reduce((n, s) => n + Object.keys(s.cells).length, 0) ?? 0

  return (
    <section className="flex flex-col gap-4 rounded-md border border-border bg-surface p-4">
      <div className="flex flex-wrap items-center gap-4">
        <Button
          variant={result ? 'secondary' : 'primary'}
          icon={Table2}
          loading={extracting}
          disabled={status === 'loading' || noSheets}
          onClick={runExtraction}
        >
          {result ? 'Extrair novamente' : 'Extrair'}
        </Button>
        <ScopeSelector
          value={scope}
          onChange={setScope}
          disabled={extracting}
        />
        {result && !extracting && (
          <p className="text-sm text-muted">
            {cellCount.toLocaleString('pt-BR')} células em{' '}
            {result.sheets.length} {result.sheets.length === 1 ? 'aba' : 'abas'}{' '}
            · {result.vba.length} componente(s) VBA
          </p>
        )}
        {noSheets && (
          <p className="text-sm text-warning">Marque ao menos uma aba.</p>
        )}
      </div>

      {fromCache && !extracting && (
        <Alert
          variant="info"
          title="Resultado do cache"
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
          O arquivo não foi relido: é a última extração, com o arquivo igual ao
          de então.
        </Alert>
      )}

      {hasVba && (
        <div className="flex flex-wrap gap-x-6 gap-y-2">
          <Checkbox
            label="Remover linhas em branco do VBA"
            checked={vbaOptions.remove_blank_lines}
            disabled={extracting}
            onChange={(v) => changeOption({ remove_blank_lines: v })}
          />
          <Checkbox
            label="Ocultar linhas Attribute VB_"
            checked={vbaOptions.hide_attributes}
            disabled={extracting}
            onChange={(v) => changeOption({ hide_attributes: v })}
          />
        </div>
      )}

      {extracting && (
        <ProgressBar
          current={progress?.current}
          total={progress?.stage === 'cells' ? progress.total : undefined}
          label={progressLabel(progress)}
        />
      )}

      {!extracting &&
        warnings.map((w) => (
          <Alert
            key={`${w.kind}:${w.sheet}`}
            variant={w.kind === 'Sampled' ? 'info' : 'warning'}
            title={`Aba "${w.sheet}"`}
          >
            {w.message}
          </Alert>
        ))}
    </section>
  )
}
