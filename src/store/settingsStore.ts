import { create } from "zustand";

import type { AppError, Settings } from "../types/models";

// Mesmos padrões de store/settings.rs: valem até o `get_settings` responder.
export const DEFAULT_SETTINGS: Settings = {
  version: 1,
  default_out_dir: null,
  vba_options: { remove_blank_lines: true, hide_attributes: false },
};

interface SettingsState {
  settings: Settings;
  /** Falha ao gravar; as alterações continuam valendo até fechar o app. */
  error: AppError | null;
  setSettings: (s: Settings) => void;
  setError: (e: AppError | null) => void;
}

export const useSettingsStore = create<SettingsState>((set) => ({
  settings: DEFAULT_SETTINGS,
  error: null,
  setSettings: (settings) => set({ settings, error: null }),
  setError: (error) => set({ error }),
}));
