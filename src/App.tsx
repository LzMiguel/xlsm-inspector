import { useEffect, useState } from 'react'
import { FileSpreadsheet, Settings } from 'lucide-react'

import { ComponentList } from './components/ComponentList'
import { DropZone } from './components/DropZone'
import { ErrorAlert } from './components/ErrorAlert'
import { ExportPanel } from './components/ExportPanel'
import { ExtractPanel } from './components/ExtractPanel'
import { FileCard } from './components/FileCard'
import { Preview } from './components/Preview'
import { RecentFiles } from './components/RecentFiles'
import { SettingsDialog } from './components/SettingsDialog'
import { SheetList } from './components/SheetList'
import { ThemeToggle } from './components/ThemeToggle'
import { Button } from './components/ui/Button'
import { useFileDrop } from './hooks/useFileDrop'
import { useHotkey } from './hooks/useHotkey'
import { loadSettings } from './store/settingsActions'
import { chooseAndOpenWorkbook, openWorkbook } from './store/workbookActions'
import { useWorkbookStore } from './store/workbookStore'

function App() {
  const summary = useWorkbookStore((s) => s.summary)
  const result = useWorkbookStore((s) => s.result)
  const tab = useWorkbookStore((s) => s.tab)
  const status = useWorkbookStore((s) => s.status)
  const error = useWorkbookStore((s) => s.error)
  const dragging = useFileDrop((path) => {
    const st = useWorkbookStore.getState().status
    if (st !== 'loading' && st !== 'extracting') void openWorkbook(path)
  })

  const [settingsOpen, setSettingsOpen] = useState(false)
  useEffect(() => {
    loadSettings()
  }, [])
  useHotkey(',', () => setSettingsOpen((o) => !o))

  const busy = status === 'loading' || status === 'extracting'
  useHotkey('f', () => {
    if (summary && tab !== 'cells') useWorkbookStore.getState().setTab('search')
  })
  useHotkey('o', () => {
    if (!busy) chooseAndOpenWorkbook()
  })

  return (
    <div className="flex h-full flex-col">
      <header className="flex h-12 shrink-0 items-center gap-2 border-b border-border bg-surface px-4">
        <FileSpreadsheet
          size={16}
          strokeWidth={1.75}
          className="text-primary"
          aria-hidden
        />
        <h1 className="truncate text-lg font-semibold">
          {summary?.file_name ?? 'XLSM Inspector'}
        </h1>
        <div className="ml-auto flex items-center gap-1">
          <Button
            variant="ghost"
            icon={Settings}
            aria-label="Configurações (Ctrl+,)"
            title="Configurações (Ctrl+,)"
            onClick={() => setSettingsOpen(true)}
          />
          <ThemeToggle />
        </div>
      </header>

      <div className="flex min-h-0 flex-1">
        <aside
          aria-label="Planilhas e componentes"
          className="flex w-60 shrink-0 flex-col border-r border-border bg-surface"
        >
          <div className="min-h-0 flex-1 overflow-y-auto p-4">
            {!summary ? (
              <p className="text-sm text-subtle">Nenhum arquivo aberto.</p>
            ) : tab === 'vba' && result ? (
              <ComponentList />
            ) : (
              <SheetList />
            )}
          </div>
          {summary && <ExportPanel />}
        </aside>

        {/* Sem rolagem aqui: só a área de exibição (cada aba) rola, o cabeçalho fica fixo. */}
        <main className="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden p-6">
          <div className="mx-auto flex min-h-0 w-full max-w-5xl flex-1 flex-col gap-4">
            {error && (
              <div className="shrink-0">
                <ErrorAlert error={error} />
              </div>
            )}
            {summary ? (
              <>
                <FileCard
                  summary={summary}
                  disabled={busy}
                />
                <ExtractPanel />
                <Preview />
              </>
            ) : (
              <div className="mx-auto flex min-h-0 w-full max-w-2xl flex-1 flex-col gap-6 overflow-y-auto pt-8">
                <DropZone
                  dragging={dragging}
                  loading={status === 'loading'}
                  invalid={error?.code === 'UnsupportedFormat'}
                />
                <RecentFiles disabled={busy} />
              </div>
            )}
          </div>
        </main>
      </div>
      <SettingsDialog
        open={settingsOpen}
        onClose={() => setSettingsOpen(false)}
      />
    </div>
  )
}

export default App
