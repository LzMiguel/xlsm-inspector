import { useState } from 'react'
import { FileDown } from 'lucide-react'

import {
  exportVbaFiles,
  pickFolder,
  revealInFolder,
  toAppError,
} from '../api/commands'
import { updateVbaOptions } from '../store/settingsActions'
import { useSettingsStore } from '../store/settingsStore'
import { refreshVba } from '../store/workbookActions'
import { useWorkbookStore } from '../store/workbookStore'
import type { AppError, VbaComponent, VbaModuleInfo } from '../types/models'
import { ErrorAlert } from './ErrorAlert'
import { visible } from './marks'
import { Badge } from './ui/Badge'
import { Button } from './ui/Button'
import { Checkbox } from './ui/Checkbox'
import { cn } from './ui/cn'
import { Toast } from './ui/Toast'
import { KIND_GROUP_LABEL, KIND_ORDER } from './vbaKinds'

const EMPTY_VBA: VbaComponent[] = []
const EMPTY_MODULES: VbaModuleInfo[] = []

/** Seção VBA da sidebar: marca o que é exibido e exportado; o nome abre o módulo. */
export function ComponentList() {
  const modules = useWorkbookStore((s) => s.summary?.vba) ?? EMPTY_MODULES
  const read = useWorkbookStore((s) => s.result?.vba) ?? EMPTY_VBA
  const hasResult = useWorkbookStore((s) => s.result !== null)
  const hidden = useWorkbookStore((s) => s.hidden.vba)
  const toggle = useWorkbookStore((s) => s.toggleHidden)
  const setHidden = useWorkbookStore((s) => s.setHidden)
  const tab = useWorkbookStore((s) => s.tab)
  const setTab = useWorkbookStore((s) => s.setTab)
  const selected = useWorkbookStore((s) => s.selectedComponent)
  const select = useWorkbookStore((s) => s.selectComponent)
  const goToVba = useWorkbookStore((s) => s.goToVba)
  const busy = useWorkbookStore(
    (s) => s.status === 'extracting' || s.status === 'loading',
  )
  const filePath = useWorkbookStore((s) => s.filePath)
  const defaultDir = useSettingsStore((s) => s.settings.default_out_dir)
  const vbaOptions = useSettingsStore((s) => s.settings.vba_options)
  const [exporting, setExporting] = useState(false)
  const [done, setDone] = useState<string[] | null>(null)
  const [error, setError] = useState<AppError | null>(null)

  if (modules.length === 0) return null

  const marked = visible(modules, hidden).map((m) => m.name)
  const shown = visible(read, hidden)
  const current =
    tab === 'vba'
      ? (shown.find((c) => c.name === selected) ?? shown[0])?.name
      : undefined

  async function exportFiles() {
    if (!filePath || exporting) return
    setExporting(true)
    setError(null)
    try {
      const dir = await pickFolder(defaultDir ?? undefined)
      if (dir) setDone(await exportVbaFiles(filePath, dir, marked))
    } catch (e) {
      setError(toAppError(e))
    } finally {
      setExporting(false)
    }
  }

  const changeOption = (o: Partial<typeof vbaOptions>) => {
    updateVbaOptions(o)
    void refreshVba()
  }

  return (
    <nav
      aria-label="Componentes VBA"
      className="flex flex-col gap-2"
    >
      <Checkbox
        label={
          <span className="font-semibold">
            VBA{' '}
            <span className="font-normal text-muted">
              ({marked.length}/{modules.length})
            </span>
          </span>
        }
        checked={marked.length === modules.length}
        indeterminate={marked.length > 0 && marked.length < modules.length}
        disabled={busy}
        onChange={(all) =>
          setHidden('vba', all ? [] : modules.map((m) => m.name))
        }
      />

      {tab === 'vba' && hasResult && (
        <div className="flex flex-col gap-1">
          <Checkbox
            label="Remover linhas em branco"
            checked={vbaOptions.remove_blank_lines}
            onChange={(v) => changeOption({ remove_blank_lines: v })}
          />
          <Checkbox
            label="Ocultar linhas Attribute VB_"
            checked={vbaOptions.hide_attributes}
            onChange={(v) => changeOption({ hide_attributes: v })}
          />
        </div>
      )}

      {KIND_ORDER.map((kind) => {
        const items = modules.filter((m) => m.kind === kind)
        if (items.length === 0) return null
        return (
          <section
            key={kind}
            aria-label={KIND_GROUP_LABEL[kind]}
          >
            <h3 className="mb-1 text-xs font-medium tracking-wide text-muted uppercase">
              {KIND_GROUP_LABEL[kind]}
            </h3>
            <ul className="flex flex-col gap-1">
              {items.map((m) => {
                const off = hidden.includes(m.name)
                const component = read.find((c) => c.name === m.name)
                const active = current === m.name
                return (
                  <li key={m.name}>
                    <div className="flex items-center gap-1 pl-1">
                      <Checkbox
                        label={null}
                        aria-label={`Incluir ${m.name}`}
                        checked={!off}
                        disabled={busy}
                        onChange={() => toggle('vba', m.name)}
                      />
                      <button
                        type="button"
                        aria-current={active || undefined}
                        disabled={off}
                        onClick={() => {
                          select(m.name)
                          setTab('vba')
                        }}
                        className={cn(
                          'flex min-w-0 flex-1 items-center gap-2 rounded-sm px-2 py-1 text-left text-sm',
                          'transition-colors duration-150 ease-out disabled:opacity-50',
                          'focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent',
                          active
                            ? 'bg-primary-soft text-text'
                            : 'text-text enabled:hover:bg-surface-muted',
                        )}
                      >
                        <span
                          className="flex-1 truncate"
                          title={m.name}
                        >
                          {m.name}
                        </span>
                        <Badge>{component?.line_count ?? m.line_count}</Badge>
                      </button>
                    </div>
                    {active && component && component.procedures.length > 0 && (
                      <ul
                        aria-label={`Procedimentos de ${m.name}`}
                        className="mt-1 mb-1 ml-8"
                      >
                        {component.procedures.map((p) => (
                          <li key={p.start_line}>
                            <button
                              type="button"
                              onClick={() => goToVba(m.name, p.start_line)}
                              title={`${p.visibility} ${p.kind} · linhas ${p.start_line}-${p.end_line}`}
                              className="w-full truncate rounded-sm px-2 py-0.5 text-left text-xs text-muted hover:bg-surface-muted hover:text-text focus-visible:outline-2 focus-visible:outline-accent"
                            >
                              {p.name}
                              <span className="text-subtle"> · {p.kind}</span>
                            </button>
                          </li>
                        ))}
                      </ul>
                    )}
                  </li>
                )
              })}
            </ul>
          </section>
        )
      })}

      {tab === 'vba' && hasResult && (
        <Button
          size="sm"
          icon={FileDown}
          loading={exporting}
          disabled={marked.length === 0}
          onClick={exportFiles}
        >
          Exportar como arquivos
        </Button>
      )}
      {error && <ErrorAlert error={error} />}
      {done && (
        <Toast
          message={`${done.length} arquivo(s) .bas/.cls/.frm gerado(s).`}
          actionLabel="Abrir pasta"
          onAction={() => {
            if (done[0])
              revealInFolder(done[0]).catch((e) => setError(toAppError(e)))
          }}
          onClose={() => setDone(null)}
        />
      )}
    </nav>
  )
}
