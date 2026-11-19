import { create } from 'zustand'

import type { AppError, Settings } from '../types/models'

export const DEFAULT_SETTINGS: Settings = {
  version: 1,
  default_out_dir: null,
  vba_options: { remove_blank_lines: true, hide_attributes: false },
  file_names: {
    json: 'mapeamento_planilha',
    cells_md: 'mapeamento_planilha',
    vba_md: 'vba_code',
    bundle: 'contexto_ia',
  },
  template: {
    title: '# Mapeamento de Estrutura: {arquivo}',
    intro: '',
    sheet_heading: '## Aba: {aba}',
    cell_header: 'Célula',
    content_header: 'Conteúdo Extraído',
    formula_label: 'Fórmula',
    value_label: 'Valor',
    vba_heading: '## COMPONENTE: {componente}',
    footer: '',
  },
  presets: [],
  use_cache: true,
}

interface SettingsState {
  settings: Settings
  error: AppError | null
  setSettings: (s: Settings) => void
  setError: (e: AppError | null) => void
}

export const useSettingsStore = create<SettingsState>((set) => ({
  settings: DEFAULT_SETTINGS,
  error: null,
  setSettings: (settings) => set({ settings, error: null }),
  setError: (error) => set({ error }),
}))
