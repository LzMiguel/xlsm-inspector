import { create } from "zustand";

import type {
  AppError,
  ExtractionResult,
  Progress,
  Warning,
  WorkbookSummary,
} from "../types/models";

export type Status = "idle" | "loading" | "extracting" | "ready" | "error";

interface WorkbookState {
  filePath: string | null;
  summary: WorkbookSummary | null;
  selectedSheets: string[];
  result: ExtractionResult | null;
  progress: Progress | null;
  warnings: Warning[];
  status: Status;
  error: AppError | null;

  startLoading: (path: string) => void;
  setSummary: (summary: WorkbookSummary) => void;
  toggleSheet: (name: string) => void;
  setAllSheets: (selected: boolean) => void;
  startExtracting: () => void;
  setProgress: (p: Progress | null) => void;
  setResult: (r: ExtractionResult) => void;
  setError: (e: AppError) => void;
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
};

export const useWorkbookStore = create<WorkbookState>((set, get) => ({
  ...initial,

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
    set({ result, warnings: result.warnings, progress: null, status: "ready" }),

  setError: (error) => set({ error, progress: null, status: "error" }),

  reset: () => set({ ...initial }),
}));
