import { useState } from 'react'
import { FolderOpen } from 'lucide-react'

import { clearCache, pickFolder, toAppError } from '../api/commands'
import { updateSettings, updateVbaOptions } from '../store/settingsActions'
import { useSettingsStore } from '../store/settingsStore'
import { loadRecent } from '../store/workbookActions'
import { ErrorAlert } from './ErrorAlert'
import { OutputSettings } from './OutputSettings'
import { TemplateSettings } from './TemplateSettings'
import { Button } from './ui/Button'
import { Checkbox } from './ui/Checkbox'
import { Dialog } from './ui/Dialog'
import { Tabs, type TabItem } from './ui/Tabs'

type Tab = 'geral' | 'saida' | 'modelos'
const TABS: TabItem<Tab>[] = [
  { id: 'geral', label: 'Geral' },
  { id: 'saida', label: 'Saída' },
  { id: 'modelos', label: 'Modelos' },
]

export function SettingsDialog({
  open,
  onClose,
}: {
  open: boolean
  onClose: () => void
}) {
  const settings = useSettingsStore((s) => s.settings)
  const error = useSettingsStore((s) => s.error)
  const setError = useSettingsStore((s) => s.setError)
  const [tab, setTab] = useState<Tab>('geral')
  const [cacheCleared, setCacheCleared] = useState(false)

  async function emptyCache() {
    try {
      await clearCache()
      setCacheCleared(true)
      loadRecent()
    } catch (e) {
      setError(toAppError(e))
    }
  }

  async function chooseFolder() {
    try {
      const dir = await pickFolder(settings.default_out_dir ?? undefined)
      if (dir) await updateSettings({ default_out_dir: dir })
    } catch (e) {
      setError(toAppError(e))
    }
  }

  return (
    <Dialog
      open={open}
      onClose={onClose}
      title="Configurações"
      wide
    >
      {error && <ErrorAlert error={error} />}
      <Tabs
        tabs={TABS}
        value={tab}
        onChange={setTab}
      />

      {tab === 'saida' && <OutputSettings />}
      {tab === 'modelos' && <TemplateSettings />}
      {tab === 'geral' && (
        <>
          <section className="flex flex-col gap-2">
            <h3 className="text-sm font-semibold">Pasta de exportação</h3>
            <p className="text-sm text-muted">
              O diálogo de exportação abre nesta pasta.
            </p>
            <div className="flex items-center gap-2">
              <span
                className="min-w-0 flex-1 truncate rounded-sm border border-border bg-surface-muted px-2 py-1.5 font-mono text-sm"
                title={settings.default_out_dir ?? undefined}
              >
                {settings.default_out_dir ??
                  'Nenhuma (usa a última pasta do sistema)'}
              </span>
              <Button
                size="sm"
                icon={FolderOpen}
                onClick={chooseFolder}
              >
                Escolher…
              </Button>
              <Button
                size="sm"
                variant="ghost"
                disabled={!settings.default_out_dir}
                onClick={() => updateSettings({ default_out_dir: null })}
              >
                Limpar
              </Button>
            </div>
          </section>

          <section className="flex flex-col gap-2">
            <h3 className="text-sm font-semibold">Cache</h3>
            <p className="text-sm text-muted">
              Guarda no disco, só neste computador, a última extração das 5
              planilhas mais recentes. Reabrir uma planilha que não mudou não
              relê o arquivo.
            </p>
            <Checkbox
              label="Guardar resultados em cache"
              checked={settings.use_cache}
              onChange={(v) => updateSettings({ use_cache: v })}
            />
            <div className="flex items-center gap-2">
              <Button
                size="sm"
                onClick={emptyCache}
              >
                Limpar cache
              </Button>
              {cacheCleared && (
                <span className="text-sm text-muted">Cache limpo.</span>
              )}
            </div>
          </section>

          <section className="flex flex-col gap-2">
            <h3 className="text-sm font-semibold">Código VBA</h3>
            <Checkbox
              label="Remover linhas em branco"
              checked={settings.vba_options.remove_blank_lines}
              onChange={(v) => updateVbaOptions({ remove_blank_lines: v })}
            />
            <Checkbox
              label="Ocultar linhas Attribute VB_"
              checked={settings.vba_options.hide_attributes}
              onChange={(v) => updateVbaOptions({ hide_attributes: v })}
            />
          </section>
        </>
      )}
    </Dialog>
  )
}
