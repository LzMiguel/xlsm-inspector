// Orquestra api + store. Componentes chamam estas funções; nada de invoke aqui.
import { extractCells, extractVba, inspectWorkbook, pickWorkbook, toAppError } from "../api/commands";
import { onProgress } from "../api/events";
import type { VbaComponent } from "../types/models";
import { useSettingsStore } from "./settingsStore";
import { useWorkbookStore } from "./workbookStore";

const store = () => useWorkbookStore.getState();

export async function openWorkbook(path: string) {
  store().startLoading(path);
  try {
    const summary = await inspectWorkbook(path);
    store().setSummary(summary);
    store().addRecent(path, summary.file_name);
  } catch (e) {
    // mantém a tela inicial; o erro aparece em um Alert
    const err = toAppError(e);
    if (err.code === "FileNotFound") store().removeRecent(path);
    store().setError(err);
  }
}

/** Diálogo nativo + abrir. Cancelar não faz nada. */
export async function chooseAndOpenWorkbook() {
  try {
    const path = await pickWorkbook();
    if (path) await openWorkbook(path);
  } catch (e) {
    store().setError(toAppError(e));
  }
}

export async function runExtraction() {
  const { filePath, selectedSheets, summary } = store();
  const vbaOptions = useSettingsStore.getState().settings.vba_options;
  if (!filePath) return;

  store().startExtracting();
  const unlisten = await onProgress((p) => store().setProgress(p)).catch(() => undefined);
  try {
    const cells = await extractCells(filePath, selectedSheets);

    // VBA: sem projeto não é erro, só lista vazia.
    let vba: VbaComponent[] = [];
    if (summary?.has_vba) {
      store().setProgress({ stage: "vba", current: 0, total: 0, label: "VBA" });
      vba = await extractVba(filePath, vbaOptions);
    }
    store().setResult({ sheets: cells.sheets, vba, warnings: cells.warnings });
  } catch (e) {
    store().setError(toAppError(e));
  } finally {
    unlisten?.();
  }
}

/** Reextrai só o VBA (mudou uma opção de limpeza). */
export async function refreshVba() {
  const { filePath, result } = store();
  const vbaOptions = useSettingsStore.getState().settings.vba_options;
  if (!filePath || !result) return;
  try {
    store().setVba(await extractVba(filePath, vbaOptions));
  } catch (e) {
    store().setError(toAppError(e));
  }
}
