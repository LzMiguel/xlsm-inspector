import { create } from "zustand";

import type {
  AppError,
  ExtractionResult,
  Progress,
  VbaComponent,
  VbaOptions,
  Warning,
  WorkbookSummary,
} from "../types/models";

export type Status = "idle" | "loading" | "extracting" | "ready" | "error";
export type PreviewTab = "cells" | "vba";

interface WorkbookState {
  filePath: string | null;
  summary: WorkbookSummary | null;
  selectedSheets: string[];
  result: ExtractionResult | null;
  progress: Progress | null;
  warnings: Warning[];
  status: Status;
  error: AppError | null;

  // Pré-visualização / opções (não resetam ao trocar de arquivo, exceto a seleção)
  tab: PreviewTab;
  selectedComponent: string | null;
  vbaOptions: VbaOptions;

  startLoading: (path: string) => void;
  setSummary: (summary: WorkbookSummary) => void;
  toggleSheet: (name: string) => void;
  setAllSheets: (selected: boolean) => void;
  startExtracting: () => void;
  setProgress: (p: Progress | null) => void;
  setResult: (r: ExtractionResult) => void;
  setVba: (vba: VbaComponent[]) => void;
  setError: (e: AppError) => void;
  setTab: (tab: PreviewTab) => void;
  selectComponent: (name: string) => void;
  setVbaOptions: (o: Partial<VbaOptions>) => void;
  reset: () => void;
}

const initial = {
  filePath: null,
  summary: null,
  selectedSheets: [],
  result: null,
  progress: null,
  warnings: [],
  status: "idle" as Status,
  error: null,
  tab: "cells" as PreviewTab,
  selectedComponent: null,
};

/** Mantém a seleção atual se o componente ainda existir; senão, o primeiro. */
const pickComponent = (vba: VbaComponent[], current: string | null) =>
  vba.some((c) => c.name === current) ? current : (vba[0]?.name ?? null);

export const useWorkbookStore = create<WorkbookState>((set, get) => ({
  ...initial,
  vbaOptions: { remove_blank_lines: true, hide_attributes: false },

  // Novo arquivo: descarta qualquer resultado anterior.
  startLoading: (path) => set({ ...initial, filePath: path, status: "loading" }),

  // Todas as abas marcadas por padrão (RF02).
  setSummary: (summary) =>
    set({
      summary,
      selectedSheets: summary.sheets.map((s) => s.name),
      status: "ready",
      error: null,
    }),

  toggleSheet: (name) => {
    const cur = get().selectedSheets;
    set({
      selectedSheets: cur.includes(name) ? cur.filter((n) => n !== name) : [...cur, name],
    });
  },

  setAllSheets: (selected) =>
    set({ selectedSheets: selected ? (get().summary?.sheets.map((s) => s.name) ?? []) : [] }),

  startExtracting: () => set({ status: "extracting", progress: null, result: null, error: null }),

  setProgress: (progress) => set({ progress }),

  setResult: (result) =>
    set({
      result,
      warnings: result.warnings,
      progress: null,
      status: "ready",
      selectedComponent: pickComponent(result.vba, get().selectedComponent),
    }),

  setVba: (vba) => {
    const { result, selectedComponent } = get();
    if (!result) return;
    set({ result: { ...result, vba }, selectedComponent: pickComponent(vba, selectedComponent) });
  },

  setError: (error) => set({ error, progress: null, status: "error" }),

  setTab: (tab) => set({ tab }),

  selectComponent: (selectedComponent) => set({ selectedComponent }),

  setVbaOptions: (o) => set({ vbaOptions: { ...get().vbaOptions, ...o } }),

  reset: () => set({ ...initial }),
}));
