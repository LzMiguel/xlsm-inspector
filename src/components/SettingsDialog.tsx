import { useEffect, useState } from 'react'
import { FolderOpen } from 'lucide-react'

import { clearCache, pickFolder, toAppError } from '../api/commands'
import { updateSettings, updateVbaOptions } from '../store/settingsActions'
import { useSettingsStore } from '../store/settingsStore'
import { loadRecent, refreshVba } from '../store/workbookActions'
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

  useEffect(() => {
    if (open) setCacheCleared(false)
  }, [open])

  const changeVbaOption = (o: Partial<typeof settings.vba_options>) => {
    updateVbaOptions(o)
    void refreshVba()
  }

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
            <h3 className="text-sm font-semibold">Resultados salvos</h3>
            <p className="text-sm text-muted">
              Guarda a última leitura dos 5 arquivos mais recentes.
            </p>
            <Checkbox
              label="Guardar resultados"
              checked={settings.use_cache}
              onChange={(v) => updateSettings({ use_cache: v })}
            />
            <div className="flex items-center gap-2">
              <Button
                size="sm"
                onClick={emptyCache}
              >
                Apagar resultados salvos
              </Button>
              {cacheCleared && (
                <span className="text-sm text-muted">Resultados apagados.</span>
              )}
            </div>
          </section>

          <section className="flex flex-col gap-2">
            <h3 className="text-sm font-semibold">Código VBA</h3>
            <Checkbox
              label="Remover linhas em branco"
              checked={settings.vba_options.remove_blank_lines}
              onChange={(v) => changeVbaOption({ remove_blank_lines: v })}
            />
            <Checkbox
              label="Ocultar linhas Attribute VB_"
              checked={settings.vba_options.hide_attributes}
              onChange={(v) => changeVbaOption({ hide_attributes: v })}
            />
          </section>
        </>
      )}
    </Dialog>
  )
}
