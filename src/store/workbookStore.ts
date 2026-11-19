import { create } from 'zustand'

import type {
  AppError,
  ExtractionResult,
  ExtractScope,
  CachedExtraction,
  Diagnostics,
  PathStatus,
  PowerQuery,
  Progress,
  RecentFile,
  VbaComponent,
  Warning,
  WorkbookSummary,
} from '../types/models'

export type Status = 'idle' | 'loading' | 'extracting' | 'ready' | 'error'
export type PreviewTab = 'cells' | 'vba' | 'diagnostics' | 'powerquery' | 'search'

export interface CellFocus {
  sheet: string
  cell: string
}

interface WorkbookState {
  filePath: string | null
  summary: WorkbookSummary | null
  selectedSheets: string[]
  result: ExtractionResult | null
  progress: Progress | null
  warnings: Warning[]
  status: Status
  error: AppError | null

  tab: PreviewTab
  selectedComponent: string | null
  diagnostics: Diagnostics | null
  powerQuery: PowerQuery | null
  focus: CellFocus | null
  vbaLine: number | null
  scope: ExtractScope
  fromCache: boolean
  recent: RecentFile[]
  recentStatus: Record<string, PathStatus>

  setRecent: (list: RecentFile[]) => void
  setRecentStatus: (status: Record<string, PathStatus>) => void
  loadCached: (c: CachedExtraction) => void
  startLoading: (path: string) => void
  setSummary: (summary: WorkbookSummary) => void
  toggleSheet: (name: string) => void
  setAllSheets: (selected: boolean) => void
  startExtracting: () => void
  setProgress: (p: Progress | null) => void
  setResult: (r: ExtractionResult) => void
  setVba: (vba: VbaComponent[]) => void
  setError: (e: AppError) => void
  setTab: (tab: PreviewTab) => void
  setDiagnostics: (d: Diagnostics) => void
  setPowerQuery: (p: PowerQuery) => void
  goToCell: (focus: CellFocus) => void
  setScope: (scope: ExtractScope) => void
  selectComponent: (name: string) => void
  goToVba: (name: string, line: number) => void
  reset: () => void
}

const initial = {
  filePath: null,
  summary: null,
  selectedSheets: [],
  result: null,
  progress: null,
  warnings: [],
  status: 'idle' as Status,
  error: null,
  tab: 'cells' as PreviewTab,
  selectedComponent: null,
  diagnostics: null,
  powerQuery: null,
  focus: null,
  vbaLine: null as number | null,
  fromCache: false,
}

const pickComponent = (vba: VbaComponent[], current: string | null) =>
  vba.some((c) => c.name === current) ? current : (vba[0]?.name ?? null)

export const useWorkbookStore = create<WorkbookState>((set, get) => ({
  ...initial,
  recent: [],
  recentStatus: {},
  scope: { kind: 'full' },

  setRecent: (recent) => set({ recent }),

  setRecentStatus: (recentStatus) => set({ recentStatus }),

  loadCached: (c) => {
    const names = get().summary?.sheets.map((s) => s.name) ?? []
    set({
      selectedSheets: names.filter((n) => c.sheets.includes(n)),
      scope: c.scope,
      result: c.result,
      warnings: c.result.warnings,
      selectedComponent: pickComponent(c.result.vba, null),
      progress: null,
      status: 'ready',
      fromCache: true,
    })
  },

  startLoading: (path) =>
    set({ ...initial, filePath: path, status: 'loading' }),

  setSummary: (summary) =>
    set({
      summary,
      selectedSheets: summary.sheets.map((s) => s.name),
      status: 'ready',
      error: null,
    }),

  toggleSheet: (name) => {
    const cur = get().selectedSheets
    set({
      selectedSheets: cur.includes(name)
        ? cur.filter((n) => n !== name)
        : [...cur, name],
    })
  },

  setAllSheets: (selected) =>
    set({
      selectedSheets: selected
        ? (get().summary?.sheets.map((s) => s.name) ?? [])
        : [],
    }),

  startExtracting: () =>
    set({
      status: 'extracting',
      progress: null,
      result: null,
      error: null,
      fromCache: false,
    }),

  setProgress: (progress) => set({ progress }),

  setResult: (result) =>
    set({
      result,
      warnings: result.warnings,
      progress: null,
      status: 'ready',
      selectedComponent: pickComponent(result.vba, get().selectedComponent),
    }),

  setVba: (vba) => {
    const { result, selectedComponent } = get()
    if (!result) return
    set({
      result: { ...result, vba },
      selectedComponent: pickComponent(vba, selectedComponent),
    })
  },

  setError: (error) => set({ error, progress: null, status: 'error' }),

  setTab: (tab) => set({ tab, focus: null }),

  setDiagnostics: (diagnostics) => set({ diagnostics }),

  setPowerQuery: (powerQuery) => set({ powerQuery }),

  goToCell: (focus) => set({ tab: 'cells', focus }),

  setScope: (scope) => set({ scope }),

  selectComponent: (selectedComponent) =>
    set({ selectedComponent, vbaLine: null }),

  goToVba: (selectedComponent, vbaLine) =>
    set({ tab: 'vba', selectedComponent, vbaLine }),

  reset: () => set({ ...initial }),
}))
