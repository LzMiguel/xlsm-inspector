// Único ponto de contato com o backend (invoke + plugins). Componentes nunca
// importam @tauri-apps diretamente.
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { revealItemInDir } from "@tauri-apps/plugin-opener";

import type {
  AppError,
  AppErrorCode,
  CellsExtraction,
  ExportTargets,
  FileNames,
  OnExists,
  OutputTemplate,
  Settings,
  TemplatePreset,
  VbaComponent,
  VbaOptions,
  WorkbookSummary,
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

export const inspectWorkbook = (path: string) =>
  call<WorkbookSummary>("inspect_workbook", { path });

/** `sheets` omitido = todas as abas. */
export const extractCells = (path: string, sheets?: string[]) =>
  call<CellsExtraction>("extract_cells", { path, sheets });

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
}

/** Devolve os caminhos dos arquivos gerados. */
export const exportOutputs = (p: ExportParams) =>
  call<string[]>("export_outputs", { ...p });

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
