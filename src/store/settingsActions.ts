// Orquestra api + store de configurações. Nada de invoke aqui.
import { getSettings, saveSettings, toAppError } from "../api/commands";
import type { Settings, VbaOptions } from "../types/models";
import { useSettingsStore } from "./settingsStore";

const store = () => useSettingsStore.getState();

export async function loadSettings() {
  try {
    store().setSettings(await getSettings());
  } catch (e) {
    store().setError(toAppError(e));
  }
}

/** Aplica na hora e grava em seguida. */
export async function updateSettings(patch: Partial<Settings>) {
  const next = { ...store().settings, ...patch };
  store().setSettings(next);
  try {
    await saveSettings(next);
  } catch (e) {
    store().setError(toAppError(e));
  }
}

export const updateVbaOptions = (o: Partial<VbaOptions>) =>
  updateSettings({ vba_options: { ...store().settings.vba_options, ...o } });
