import { useEffect, useState } from 'react'
import { Download } from 'lucide-react'

import {
  estimateBundle,
  exportOutputs,
  getBuiltinTemplates,
  pickFolder,
  revealInFolder,
  toAppError,
} from '../api/commands'
import { useHotkey } from '../hooks/useHotkey'
import { useSettingsStore } from '../store/settingsStore'
import { useWorkbookStore } from '../store/workbookStore'
import type {
  AppError,
  BundleEstimate,
  ExportTargets,
  ExtractScope,
  FileNames,
  OnExists,
  TemplatePreset,
} from '../types/models'
import { ErrorAlert } from './ErrorAlert'
import { ScopeSelector } from './ScopeSelector'
import { Alert } from './ui/Alert'
import { Button } from './ui/Button'
import { Checkbox } from './ui/Checkbox'
import { Dialog } from './ui/Dialog'
import { Field, Input, Select } from './ui/Fields'
import { Toast } from './ui/Toast'

const ROWS: {
  key: keyof ExportTargets
  name: keyof FileNames
  ext: string
  label: string
}[] = [
  { key: 'json', name: 'json', ext: '.json', label: 'Células (JSON)' },
  {
    key: 'cells_md',
    name: 'cells_md',
    ext: '.md',
    label: 'Células (Markdown)',
  },
  { key: 'vba_md', name: 'vba_md', ext: '.md', label: 'Código VBA (Markdown)' },
  {
    key: 'power_query',
    name: 'power_query',
    ext: '.md',
    label: 'Power Query (Markdown)',
  },
  {
    key: 'bundle',
    name: 'bundle',
    ext: '.md',
    label: 'Arquivo único para IA (Markdown)',
  },
]

const DEFAULT_SPLIT = '30000'

export function ExportPanel() {
  const filePath = useWorkbookStore((s) => s.filePath)
  const hasVba = useWorkbookStore((s) => s.summary?.has_vba ?? false)
  const selectedSheets = useWorkbookStore((s) => s.selectedSheets)
  const settings = useSettingsStore((s) => s.settings)
  const result = useWorkbookStore((s) => s.result)
  const busyElsewhere = useWorkbookStore(
    (s) => s.status === 'loading' || s.status === 'extracting',
  )

  const [targets, setTargets] = useState<ExportTargets>({
    json: true,
    cells_md: true,
    vba_md: true,
    power_query: true,
    bundle: false,
  })
  const [open, setOpen] = useState(false)
  const [splitOn, setSplitOn] = useState(false)
  const [splitText, setSplitText] = useState(DEFAULT_SPLIT)
  const [estimate, setEstimate] = useState<BundleEstimate | null>(null)
  const extractScope = useWorkbookStore((s) => s.scope)
  const [exportScope, setExportScope] = useState<ExtractScope | null>(null)
  const [nameEdits, setNameEdits] = useState<Partial<FileNames>>({})
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<AppError | null>(null)
  const [conflict, setConflict] = useState<{
    outDir: string
    message: string
  } | null>(null)
  const [done, setDone] = useState<string[] | null>(null)

  const scope = exportScope ?? extractScope
  const suffix = scope.kind === 'full' ? '' : '_amostra'
  const { json, cells_md, vba_md, power_query, bundle } = settings.file_names
  const names: FileNames = {
    json: json + suffix,
    cells_md: cells_md + suffix,
    vba_md,
    power_query,
    bundle: bundle + suffix,
    ...nameEdits,
  }
  const vbaOn = targets.vba_md && hasVba
  const needsCells = targets.json || targets.cells_md || targets.bundle
  const splitValue = Number.parseInt(splitText, 10)
  const splitTokens = splitOn && splitValue >= 1 ? splitValue : undefined
  const estimateOn =
    open && targets.bundle && !!filePath && selectedSheets.length > 0
  const { vba_options: vbaOptions, presets } = settings
  const [templateName, setTemplateName] = useState('')
  const [builtin, setBuiltin] = useState<TemplatePreset[]>([])
  const template =
    [...builtin, ...presets].find((p) => p.name === templateName)?.template ??
    settings.template

  useEffect(() => {
    if (!open) return
    getBuiltinTemplates()
      .then(setBuiltin)
      .catch((e) => setError(toAppError(e)))
  }, [open])

  useEffect(() => {
    if (!estimateOn || !filePath) return
    let cancelled = false
    const timer = setTimeout(() => {
      estimateBundle({
        path: filePath,
        sheets: selectedSheets,
        scope,
        vbaOptions,
        template,
        splitTokens,
      })
        .then((e) => !cancelled && setEstimate(e))
        .catch(() => !cancelled && setEstimate(null))
    }, 300)
    return () => {
      cancelled = true
      clearTimeout(timer)
    }
  }, [
    estimateOn,
    filePath,
    selectedSheets,
    scope,
    vbaOptions,
    template,
    splitTokens,
  ])
  const canExport =
    !busy &&
    !busyElsewhere &&
    (needsCells || vbaOn || targets.power_query) &&
    !(needsCells && selectedSheets.length === 0)
  // nomes finais mostrados antes de gravar (o Rust resolve {arquivo} e {data} do mesmo jeito)
  const stem =
    filePath
      ?.split(/[\\/]/)
      .pop()
      ?.replace(/\.[^.]+$/, '') ?? ''
  const today = new Date().toLocaleDateString('sv-SE')
  const chosen = ROWS.filter(({ key }) =>
    key === 'vba_md' ? vbaOn : targets[key],
  ).map(
    ({ name, ext }) =>
      names[name].replaceAll('{arquivo}', stem).replaceAll('{data}', today) +
      ext,
  )
  const set = (k: keyof ExportTargets) => (v: boolean) =>
    setTargets((t) => ({ ...t, [k]: v }))

  async function run(outDir: string, onExists: OnExists) {
    if (!filePath) return
    setBusy(true)
    setError(null)
    setConflict(null)
    try {
      const files = await exportOutputs({
        path: filePath,
        outDir,
        targets: { ...targets, vba_md: vbaOn },
        sheets: selectedSheets,
        vbaOptions: settings.vba_options,
        names,
        template,
        onExists,
        scope,
        splitTokens: targets.bundle ? splitTokens : undefined,
      })
      setDone(files)
      setOpen(false)
    } catch (e) {
      const err = toAppError(e)
      if (err.code === 'FileExists')
        setConflict({ outDir, message: err.message })
      else setError(err)
    } finally {
      setBusy(false)
    }
  }

  async function exportNow() {
    if (!filePath || !canExport) return
    try {
      const outDir = await pickFolder(settings.default_out_dir ?? undefined)
      if (outDir) await run(outDir, 'fail')
    } catch (e) {
      setError(toAppError(e))
    }
  }

  const canOpen = !!filePath && !busyElsewhere
  const openDialog = () => {
    if (!canOpen) return
    setError(null)
    setConflict(null)
    setOpen(true)
  }

  useHotkey('e', openDialog)

  const options = (
    <>
      <ScopeSelector
        value={scope}
        onChange={setExportScope}
        disabled={busy}
      />

      <Field
        label="Modelo do texto (Markdown)"
        hint="Vale só para esta exportação (não altera as configurações)."
      >
        <Select
          value={templateName}
          disabled={busy}
          onChange={(e) => setTemplateName(e.currentTarget.value)}
        >
          <option value="">Modelo ativo nas configurações</option>
          {builtin.map((p) => (
            <option key={p.name}>{p.name}</option>
          ))}
          {presets.length > 0 && (
            <optgroup label="Meus modelos">
              {presets.map((p) => (
                <option key={p.name}>{p.name}</option>
              ))}
            </optgroup>
          )}
        </Select>
      </Field>

      <div className="flex flex-col gap-2">
        {ROWS.map(({ key, name, ext, label }) => {
          const disabled = key === 'vba_md' && !hasVba
          const checked = key === 'vba_md' ? vbaOn : targets[key]
          return (
            <div
              key={key}
              className="flex flex-col gap-1"
            >
              <Checkbox
                label={disabled ? `${label} (sem VBA)` : label}
                checked={checked}
                disabled={disabled}
                onChange={set(key)}
              />
              {checked && (
                <div className="flex items-center gap-1 pl-6">
                  <Input
                    aria-label={`Nome do arquivo ${ext}`}
                    className="py-1 font-mono text-xs"
                    value={names[name]}
                    onChange={(e) => {
                      const v = e.currentTarget.value
                      setNameEdits((n) => ({ ...n, [name]: v }))
                    }}
                  />
                  <span className="font-mono text-xs text-muted">{ext}</span>
                </div>
              )}
              {key === 'bundle' && checked && (
                <div className="flex flex-col gap-2 pl-6">
                  <Checkbox
                    label="Dividir em partes"
                    checked={splitOn}
                    onChange={setSplitOn}
                  />
                  {splitOn && (
                    <Field label="Tokens por parte (aproximado)">
                      <Input
                        type="number"
                        min={1}
                        step={1000}
                        className="py-1 font-mono text-xs"
                        value={splitText}
                        onChange={(e) => setSplitText(e.currentTarget.value)}
                      />
                    </Field>
                  )}
                  <p
                    className="text-xs text-muted"
                    aria-live="polite"
                  >
                    {estimateOn && estimate
                      ? `≈ ${estimate.tokens.toLocaleString('pt-BR')} tokens (estimativa)` +
                        (estimate.parts > 1
                          ? ` · ${estimate.parts} partes`
                          : '')
                      : 'Estimativa de tokens: ≈ caracteres ÷ 4.'}
                  </p>
                </div>
              )}
            </div>
          )
        })}
      </div>

      <p className="text-xs text-muted">
        {chosen.length === 0
          ? 'Nenhum arquivo selecionado.'
          : `Serão gravados: ${chosen.join(', ')}. VBA e Power Query só saem se o arquivo tiver.`}
      </p>

      {error && <ErrorAlert error={error} />}

      {conflict && (
        <Alert
          variant="warning"
          title="Já existem arquivos com esse nome"
        >
          <p>{conflict.message}</p>
          <div className="mt-2 flex flex-wrap gap-2">
            <Button
              size="sm"
              onClick={() => run(conflict.outDir, 'overwrite')}
            >
              Sobrescrever
            </Button>
            <Button
              size="sm"
              onClick={() => run(conflict.outDir, 'suffix')}
            >
              Adicionar sufixo _2
            </Button>
            <Button
              size="sm"
              variant="ghost"
              onClick={() => setConflict(null)}
            >
              Cancelar
            </Button>
          </div>
        </Alert>
      )}

      <div className="flex justify-end gap-2">
        <Button
          variant="ghost"
          disabled={busy}
          onClick={() => setOpen(false)}
        >
          Cancelar
        </Button>
        <Button
          icon={Download}
          loading={busy}
          disabled={!canExport}
          onClick={exportNow}
        >
          Exportar
        </Button>
      </div>
    </>
  )

  return (
    <section
      aria-label="Exportação"
      className="flex flex-col gap-3 border-t border-border p-4"
    >
      <Button
        className="w-full"
        variant={result ? 'primary' : 'secondary'}
        icon={Download}
        disabled={!canOpen}
        title="Ctrl+E"
        onClick={openDialog}
      >
        Exportar…
      </Button>
      {!result && canOpen && (
        <p className="text-xs text-muted">
          Sem leitura prévia, o arquivo será lido ao exportar.
        </p>
      )}

      <Dialog
        open={open}
        onClose={() => setOpen(false)}
        title="Exportar"
      >
        {options}
      </Dialog>

      {done && (
        <Toast
          message={`Concluído: ${done.length} arquivo(s) gerado(s).`}
          actionLabel="Abrir pasta"
          onAction={() => {
            if (done[0])
              revealInFolder(done[0]).catch((e) => setError(toAppError(e)))
          }}
          onClose={() => setDone(null)}
        />
      )}
    </section>
  )
}
