// Único ponto de contato com o backend (invoke + plugins). Componentes nunca
// importam @tauri-apps diretamente.
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { revealItemInDir } from "@tauri-apps/plugin-opener";

import type {
  AppError,
  AppErrorCode,
  BundleEstimate,
  Diagnostics,
  ExportTargets,
  ExtractionResult,
  ExtractScope,
  FileNames,
  OnExists,
  OpenResult,
  OutputTemplate,
  RecentFile,
  RecentStatus,
  Settings,
  TemplatePreset,
  VbaComponent,
  VbaOptions,
} from "../types/models";

const CODES: readonly AppErrorCode[] = [
  "FileNotFound",
  "FileLocked",
  "UnsupportedFormat",
  "CorruptedFile",
  "NoVbaProject",
  "WriteFailed",
  "InvalidName",
  "FileExists",
  "Internal",
];

/** Normaliza qualquer rejeição do invoke para `AppError`. */
export function toAppError(e: unknown): AppError {
  if (typeof e === "object" && e !== null && "code" in e && "message" in e) {
    const { code, message } = e as { code: unknown; message: unknown };
    if (CODES.includes(code as AppErrorCode) && typeof message === "string") {
      return { code: code as AppErrorCode, message };
    }
  }
  return { code: "Internal", message: e instanceof Error ? e.message : String(e) };
}

async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (e) {
    throw toAppError(e);
  }
}

/** `reload`: ignora o cache e relê o arquivo. */
export const openWorkbookFile = (path: string, reload?: boolean, vbaOptions?: VbaOptions) =>
  call<OpenResult>("open_workbook", { path, reload, vbaOptions });

/** Células + VBA. `sheets` omitido = todas as abas. */
export const extractWorkbook = (
  path: string,
  sheets?: string[],
  scope?: ExtractScope,
  vbaOptions?: VbaOptions,
) => call<ExtractionResult>("extract_workbook", { path, sheets, scope, vbaOptions });

export const diagnoseWorkbook = (path: string) => call<Diagnostics>("diagnose_workbook", { path });

export const getRecent = () => call<RecentFile[]>("get_recent");
export const removeRecentFile = (path: string) => call<RecentFile[]>("remove_recent", { path });
export const clearRecentFiles = () => call<RecentFile[]>("clear_recent");
export const checkRecent = () => call<RecentStatus[]>("check_recent");
export const clearCache = () => call<void>("clear_cache");

export const extractVba = (path: string, options?: VbaOptions) =>
  call<VbaComponent[]>("extract_vba", { path, options });

export interface ExportParams {
  path: string;
  outDir: string;
  targets?: ExportTargets;
  sheets?: string[];
  vbaOptions?: VbaOptions;
  names?: FileNames;
  template?: OutputTemplate;
  onExists?: OnExists;
  scope?: ExtractScope;
  /** Divide o arquivo único em partes de até N tokens (estimados); omitido = uma só parte. */
  splitTokens?: number;
}

/** Devolve os caminhos dos arquivos gerados. */
export const exportOutputs = (p: ExportParams) =>
  call<string[]>("export_outputs", { ...p });

export type EstimateParams = Pick<
  ExportParams,
  "path" | "sheets" | "scope" | "vbaOptions" | "template" | "splitTokens"
>;

/** Tokens (estimativa aproximada) e nº de partes do arquivo único. */
export const estimateBundle = (p: EstimateParams) =>
  call<BundleEstimate>("estimate_bundle", { ...p });

/** Diálogo nativo; `null` se cancelado. */
export async function pickWorkbook(): Promise<string | null> {
  const r = await open({
    multiple: false,
    directory: false,
    filters: [{ name: "Planilhas do Excel", extensions: ["xlsx", "xlsm"] }],
  });
  return typeof r === "string" ? r : null;
}

export const getSettings = () => call<Settings>("get_settings");

export const saveSettings = (settings: Settings) => call<void>("save_settings", { settings });

export const getBuiltinTemplates = () => call<TemplatePreset[]>("builtin_templates");

/** Markdown de exemplo, renderizado no Rust com dados fictícios. */
export const previewTemplate = (template: OutputTemplate) =>
  call<string>("preview_template", { template });

/** `defaultPath`: pasta onde o diálogo abre. */
export async function pickFolder(defaultPath?: string): Promise<string | null> {
  const r = await open({ multiple: false, directory: true, defaultPath });
  return typeof r === "string" ? r : null;
}

/** Abre o Explorer com o item selecionado (ex.: um arquivo exportado). */
export const revealInFolder = (path: string) => revealItemInDir(path);
