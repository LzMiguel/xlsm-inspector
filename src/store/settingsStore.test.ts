import { beforeEach, describe, expect, it, vi } from 'vitest'

vi.mock('../api/commands', () => ({
  getSettings: vi.fn(),
  saveSettings: vi.fn(),
  toAppError: (e: unknown) => e,
}))

import * as api from '../api/commands'
import {
  loadSettings,
  updateSettings,
  updateVbaOptions,
} from './settingsActions'
import { DEFAULT_SETTINGS, useSettingsStore } from './settingsStore'

const s = () => useSettingsStore.getState()

describe('configurações', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    useSettingsStore.setState({ settings: DEFAULT_SETTINGS, error: null })
  })

  it('carrega do backend', async () => {
    vi.mocked(api.getSettings).mockResolvedValue({
      ...DEFAULT_SETTINGS,
      default_out_dir: 'C:\\x',
    })
    await loadSettings()
    expect(s().settings.default_out_dir).toBe('C:\\x')
  })

  it('aplica na hora e grava; opções de VBA mesclam parcial', async () => {
    vi.mocked(api.saveSettings).mockResolvedValue(undefined)
    await updateVbaOptions({ hide_attributes: true })
    expect(s().settings.vba_options).toEqual({
      remove_blank_lines: true,
      hide_attributes: true,
    })
    expect(api.saveSettings).toHaveBeenCalledWith(s().settings)
  })

  it('gravações seguidas entram em fila e a última vence', async () => {
    const order: (string | null)[] = []
    vi.mocked(api.saveSettings).mockImplementation(async (s) => {
      await Promise.resolve()
      order.push(s.default_out_dir)
    })
    await Promise.all([
      updateSettings({ default_out_dir: 'a' }),
      updateSettings({ default_out_dir: 'b' }),
    ])
    expect(order).toEqual(['a', 'b'])
  })

  it('falha ao gravar mantém o valor e expõe o erro', async () => {
    vi.mocked(api.saveSettings).mockRejectedValue({
      code: 'WriteFailed',
      message: 'x',
    })
    await updateSettings({ default_out_dir: 'D:\\y' })
    expect(s().settings.default_out_dir).toBe('D:\\y')
    expect(s().error?.code).toBe('WriteFailed')
  })
})
