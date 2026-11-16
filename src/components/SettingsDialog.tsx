import { FolderOpen } from "lucide-react";

import { pickFolder, toAppError } from "../api/commands";
import { updateSettings, updateVbaOptions } from "../store/settingsActions";
import { useSettingsStore } from "../store/settingsStore";
import { ErrorAlert } from "./ErrorAlert";
import { Button } from "./ui/Button";
import { Checkbox } from "./ui/Checkbox";
import { Dialog } from "./ui/Dialog";

export function SettingsDialog({ open, onClose }: { open: boolean; onClose: () => void }) {
  const settings = useSettingsStore((s) => s.settings);
  const error = useSettingsStore((s) => s.error);
  const setError = useSettingsStore((s) => s.setError);

  async function chooseFolder() {
    try {
      const dir = await pickFolder(settings.default_out_dir ?? undefined);
      if (dir) await updateSettings({ default_out_dir: dir });
    } catch (e) {
      setError(toAppError(e));
    }
  }

  return (
    <Dialog open={open} onClose={onClose} title="Configurações">
      {error && <ErrorAlert error={error} />}

      <section className="flex flex-col gap-2">
        <h3 className="text-sm font-semibold">Pasta de exportação</h3>
        <p className="text-sm text-muted">O diálogo de exportação abre nesta pasta.</p>
        <div className="flex items-center gap-2">
          <span
            className="min-w-0 flex-1 truncate rounded-sm border border-border bg-surface-muted px-2 py-1.5 font-mono text-sm"
            title={settings.default_out_dir ?? undefined}
          >
            {settings.default_out_dir ?? "Nenhuma (usa a última pasta do sistema)"}
          </span>
          <Button size="sm" icon={FolderOpen} onClick={chooseFolder}>
            Escolher…
          </Button>
          <Button
            size="sm"
            variant="ghost"
            disabled={!settings.default_out_dir}
            onClick={() => updateSettings({ default_out_dir: null })}
          >
            Limpar
          </Button>
        </div>
      </section>

      <section className="flex flex-col gap-2">
        <h3 className="text-sm font-semibold">Código VBA</h3>
        <Checkbox
          label="Remover linhas em branco"
          checked={settings.vba_options.remove_blank_lines}
          onChange={(v) => updateVbaOptions({ remove_blank_lines: v })}
        />
        <Checkbox
          label="Ocultar linhas Attribute VB_"
          checked={settings.vba_options.hide_attributes}
          onChange={(v) => updateVbaOptions({ hide_attributes: v })}
        />
      </section>
    </Dialog>
  );
}
