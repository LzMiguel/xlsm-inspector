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

export async function openWorkbook(path: string, reload = false) {
  store().startLoading(path)
  try {
    const { summary, cached } = await openWorkbookFile(
      path,
      reload,
      vbaOptions(),
    )
    store().setSummary(summary)
    if (cached) store().loadCached(cached)
  } catch (e) {
    store().setError(toAppError(e))
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

  store().startExtracting()
  const unlisten = await onProgress((p) => store().setProgress(p)).catch(
    () => undefined,
  )
  try {
    store().setResult(
      await extractWorkbook(filePath, selectedSheets, scope, vbaOptions()),
    )
  } catch (e) {
    store().setError(toAppError(e))
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
  try {
    store().setVba(await extractVba(filePath, vbaOptions()))
  } catch (e) {
    store().setError(toAppError(e))
  }
}

export async function loadRecent() {
  try {
    store().setRecent(await getRecent())
    const status: Record<string, PathStatus> = {}
    for (const r of await checkRecent()) status[r.path] = r.status
    store().setRecentStatus(status)
  } catch {
    // Sem histórico
  }
}

export async function forgetRecent(path: string) {
  try {
    store().setRecent(await removeRecentFile(path))
  } catch (e) {
    store().setError(toAppError(e))
  }
}

export async function forgetAllRecent() {
  try {
    store().setRecent(await clearRecentFiles())
  } catch (e) {
    store().setError(toAppError(e))
  }
}
