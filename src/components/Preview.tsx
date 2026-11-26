import {
  Code2,
  Database,
  LayoutList,
  Search,
  Stethoscope,
  Table2,
} from 'lucide-react'
import { useEffect } from 'react'

import { useWorkbookStore, type PreviewTab } from '../store/workbookStore'
import type { SheetCells } from '../types/models'
import { CellTable } from './CellTable'
import { DiagnosticsView } from './DiagnosticsView'
import { hiddenTabs } from './marks'
import { PowerQueryView } from './PowerQueryView'
import { SearchView } from './SearchView'
import { EmptyState } from './ui/EmptyState'
import { Tabs, type TabItem } from './ui/Tabs'
import { VbaViewer } from './VbaViewer'
import { WorkbookOverview } from './WorkbookOverview'

const EMPTY_SHEETS: SheetCells[] = []

const TABS: TabItem<PreviewTab>[] = [
  { id: 'summary', label: 'Resumo', icon: LayoutList },
  { id: 'cells', label: 'Células', icon: Table2 },
  { id: 'vba', label: 'VBA', icon: Code2 },
  { id: 'diagnostics', label: 'Diagnóstico', icon: Stethoscope },
  { id: 'powerquery', label: 'Power Query', icon: Database },
  { id: 'search', label: 'Busca', icon: Search },
]

export function Preview() {
  const stored = useWorkbookStore((s) => s.tab)
  const summary = useWorkbookStore((s) => s.summary)
  const selected = useWorkbookStore((s) => s.selectedSheets)
  const hidden = useWorkbookStore((s) => s.hidden)
  const setTab = useWorkbookStore((s) => s.setTab)
  const filePath = useWorkbookStore((s) => s.filePath)
  const result = useWorkbookStore((s) => s.result)
  const sheets = (result?.sheets ?? EMPTY_SHEETS).filter((s) =>
    selected.includes(s.name),
  )
  const off = hiddenTabs(summary, selected, hidden)
  const gone = off.includes(stored)
  const tab = gone ? 'summary' : stored
  useEffect(() => {
    if (gone) setTab('summary')
  }, [gone, setTab])
  const needsRead =
    !result && (tab === 'cells' || tab === 'vba' || tab === 'search')

  return (
    <section
      aria-label="Pré-visualização"
      className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-md border border-border bg-surface"
    >
      <Tabs
        tabs={TABS.filter((t) => !off.includes(t.id))}
        value={tab}
        onChange={setTab}
        className="px-4"
      />
      {tab === 'summary' && <WorkbookOverview />}
      {needsRead && (
        <EmptyState
          icon={Table2}
          title="Nada lido ainda"
          description="Clique em “Ler arquivo” para ver aqui as células, o VBA e a busca. Diagnóstico e Power Query já funcionam."
        />
      )}
      {!needsRead && tab === 'cells' && (
        <CellTable
          key={filePath ?? ''}
          sheets={sheets}
        />
      )}
      {!needsRead && tab === 'vba' && <VbaViewer />}
      {tab === 'diagnostics' && <DiagnosticsView />}
      {tab === 'powerquery' && <PowerQueryView />}
      {!needsRead && tab === 'search' && <SearchView />}
    </section>
  )
}
