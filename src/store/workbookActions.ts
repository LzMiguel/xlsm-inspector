import {
  checkRecent,
  clearRecentFiles,
  diagnoseWorkbook,
  extractPowerQuery,
  extractVba,
  extractWorkbook,
  getRecent,
  openWorkbookFile,
  pickWorkbook,
  removeRecentFile,
  toAppError,
} from '../api/commands'
import { onProgress } from '../api/events'
import type { AppError, PathStatus } from '../types/models'
import { useSettingsStore } from './settingsStore'
import { useWorkbookStore } from './workbookStore'

const store = () => useWorkbookStore.getState()
const vbaOptions = () => useSettingsStore.getState().settings.vba_options

let openSeq = 0
let extractSeq = 0
let vbaSeq = 0
let recentSeq = 0

export async function openWorkbook(path: string, reload = false) {
  const seq = ++openSeq
  ++extractSeq
  ++vbaSeq
  store().startLoading(path)
  try {
    const { summary, cached } = await openWorkbookFile(
      path,
      reload,
      vbaOptions(),
    )
    if (seq !== openSeq || store().filePath !== path) return
    store().setSummary(summary)
    if (cached) store().loadCached(cached)
  } catch (e) {
    if (seq !== openSeq) return
    const err = toAppError(e)
    store().setError(err)
    if (err.code === 'FileNotFound') void loadRecent()
  }
}

export async function chooseAndOpenWorkbook() {
  try {
    const path = await pickWorkbook()
    if (path) await openWorkbook(path)
  } catch (e) {
    store().setError(toAppError(e))
  }
}

export async function runExtraction() {
  const { filePath, selectedSheets, scope } = store()
  if (!filePath) return

  const seq = ++extractSeq
  ++vbaSeq
  store().startExtracting()
  const unlisten = await onProgress((p) => {
    if (seq === extractSeq && store().filePath === filePath) {
      store().setProgress(p)
    }
  }).catch(() => undefined)
  try {
    const res = await extractWorkbook(
      filePath,
      selectedSheets,
      scope,
      vbaOptions(),
    )
    if (seq === extractSeq && store().filePath === filePath) {
      store().setResult(res)
    }
  } catch (e) {
    if (seq === extractSeq && store().filePath === filePath) {
      store().setError(toAppError(e))
    }
  } finally {
    unlisten?.()
  }
}

// O erro volta para a própria aba (não vira o alerta global do topo).
export async function loadDiagnostics(): Promise<AppError | null> {
  const { filePath } = store()
  if (!filePath) return null
  try {
    const d = await diagnoseWorkbook(filePath)
    if (store().filePath === filePath) store().setDiagnostics(d)
    return null
  } catch (e) {
    return toAppError(e)
  }
}

export async function loadPowerQuery(): Promise<AppError | null> {
  const { filePath } = store()
  if (!filePath) return null
  try {
    const p = await extractPowerQuery(filePath)
    if (store().filePath === filePath) store().setPowerQuery(p)
    return null
  } catch (e) {
    return toAppError(e)
  }
}

export async function refreshVba() {
  const { filePath, result } = store()
  if (!filePath || !result) return
  const seq = ++vbaSeq
  try {
    const vba = await extractVba(filePath, vbaOptions())
    if (seq === vbaSeq && store().filePath === filePath) {
      store().setVba(vba)
    }
  } catch (e) {
    if (seq === vbaSeq && store().filePath === filePath) {
      store().setError(toAppError(e))
    }
  }
}

export async function loadRecent() {
  const seq = ++recentSeq
  try {
    const list = await getRecent()
    if (seq !== recentSeq) return
    store().setRecent(list)
    const status: Record<string, PathStatus> = {}
    for (const r of await checkRecent()) status[r.path] = r.status
    if (seq === recentSeq) store().setRecentStatus(status)
  } catch {
    // Sem histórico
  }
}

export async function forgetRecent(path: string) {
  const seq = ++recentSeq
  try {
    const list = await removeRecentFile(path)
    if (seq === recentSeq) store().setRecent(list)
  } catch (e) {
    store().setError(toAppError(e))
  }
}

export async function forgetAllRecent() {
  const seq = ++recentSeq
  try {
    const list = await clearRecentFiles()
    if (seq === recentSeq) store().setRecent(list)
  } catch (e) {
    store().setError(toAppError(e))
  }
}
