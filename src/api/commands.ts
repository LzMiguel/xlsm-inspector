import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { revealItemInDir } from '@tauri-apps/plugin-opener'

import type {
  AppError,
  AppErrorCode,
  BundleEstimate,
  Diagnostics,
  ExportTargets,
  ExtractionResult,
  ExtractScope,
  FileNames,
  ItemFilter,
  OnExists,
  OpenResult,
  OutputTemplate,
  PowerQuery,
  RecentFile,
  RecentStatus,
  Settings,
  TemplatePreset,
  VbaComponent,
  VbaOptions,
} from '../types/models'

const CODES: readonly AppErrorCode[] = [
  'FileNotFound',
  'FileLocked',
  'UnsupportedFormat',
  'CorruptedFile',
  'NoVbaProject',
  'WriteFailed',
  'InvalidName',
  'FileExists',
  'Internal',
]

export function toAppError(e: unknown): AppError {
  if (typeof e === 'object' && e !== null && 'code' in e && 'message' in e) {
    const { code, message } = e as { code: unknown; message: unknown }
    if (CODES.includes(code as AppErrorCode) && typeof message === 'string') {
      return { code: code as AppErrorCode, message }
    }
  }
  return {
    code: 'Internal',
    message: e instanceof Error ? e.message : String(e),
  }
}

async function call<T>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await invoke<T>(cmd, args)
  } catch (e) {
    throw toAppError(e)
  }
}

export const openWorkbookFile = (
  path: string,
  reload?: boolean,
  vbaOptions?: VbaOptions,
) => call<OpenResult>('open_workbook', { path, reload, vbaOptions })

export const extractWorkbook = (
  path: string,
  sheets?: string[],
  scope?: ExtractScope,
  vbaOptions?: VbaOptions,
) =>
  call<ExtractionResult>('extract_workbook', {
    path,
    sheets,
    scope,
    vbaOptions,
  })

export const diagnoseWorkbook = (path: string) =>
  call<Diagnostics>('diagnose_workbook', { path })

export const extractPowerQuery = (path: string) =>
  call<PowerQuery>('extract_power_query', { path })

export const getRecent = () => call<RecentFile[]>('get_recent')
export const removeRecentFile = (path: string) =>
  call<RecentFile[]>('remove_recent', { path })
export const clearRecentFiles = () => call<RecentFile[]>('clear_recent')
export const checkRecent = () => call<RecentStatus[]>('check_recent')
export const clearCache = () => call<void>('clear_cache')

export const extractVba = (path: string, options?: VbaOptions) =>
  call<VbaComponent[]>('extract_vba', { path, options })

export const exportVbaFiles = (
  path: string,
  outDir: string,
  names?: string[],
) => call<string[]>('export_vba_files', { path, outDir, names })

export interface ExportParams {
  path: string
  outDir: string
  targets?: ExportTargets
  sheets?: string[]
  vbaOptions?: VbaOptions
  names?: FileNames
  template?: OutputTemplate
  onExists?: OnExists
  scope?: ExtractScope
  splitTokens?: number
  items?: ItemFilter
  zip?: boolean
}

export const exportOutputs = (p: ExportParams) =>
  call<string[]>('export_outputs', { ...p })

export type EstimateParams = Pick<
  ExportParams,
  | 'path'
  | 'sheets'
  | 'scope'
  | 'vbaOptions'
  | 'template'
  | 'splitTokens'
  | 'items'
> & { diagnostics?: boolean }

export const estimateBundle = (p: EstimateParams) =>
  call<BundleEstimate>('estimate_bundle', { ...p })

export async function pickWorkbook(): Promise<string | null> {
  const r = await open({
    multiple: false,
    directory: false,
    filters: [{ name: 'Planilhas do Excel', extensions: ['xlsx', 'xlsm'] }],
  })
  return typeof r === 'string' ? r : null
}

export const getSettings = () => call<Settings>('get_settings')

export const saveSettings = (settings: Settings) =>
  call<void>('save_settings', { settings })

export const getBuiltinTemplates = () =>
  call<TemplatePreset[]>('builtin_templates')

export const previewTemplate = (template: OutputTemplate) =>
  call<string>('preview_template', { template })

export async function pickFolder(defaultPath?: string): Promise<string | null> {
  const r = await open({ multiple: false, directory: true, defaultPath })
  return typeof r === 'string' ? r : null
}

export const revealInFolder = (path: string) => revealItemInDir(path)
