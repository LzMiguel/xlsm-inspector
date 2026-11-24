// Orquestra api + store de configurações. Nada de invoke aqui.
import { getSettings, saveSettings, toAppError } from '../api/commands'
import type { Settings, VbaOptions } from '../types/models'
import { useSettingsStore } from './settingsStore'

const store = () => useSettingsStore.getState()

export async function loadSettings() {
  try {
    store().setSettings(await getSettings())
  } catch (e) {
    store().setError(toAppError(e))
  }
}

let saving: Promise<void> = Promise.resolve()
let debounceTimer: ReturnType<typeof setTimeout> | undefined

export function updateSettings(patch: Partial<Settings>) {
  if (debounceTimer !== undefined) {
    clearTimeout(debounceTimer)
    debounceTimer = undefined
  }
  const next = { ...store().settings, ...patch }
  store().setSettings(next)
  saving = saving
    .then(() => saveSettings(next))
    .catch((e) => store().setError(toAppError(e)))
  return saving
}

export function updateSettingsDebounced(
  patch: Partial<Settings>,
  delayMs = 300,
) {
  const next = { ...store().settings, ...patch }
  store().setSettings(next)
  if (debounceTimer !== undefined) clearTimeout(debounceTimer)
  debounceTimer = setTimeout(() => {
    debounceTimer = undefined
    const latest = store().settings
    saving = saving
      .then(() => saveSettings(latest))
      .catch((e) => store().setError(toAppError(e)))
  }, delayMs)
}

export const updateVbaOptions = (o: Partial<VbaOptions>) =>
  updateSettings({ vba_options: { ...store().settings.vba_options, ...o } })
