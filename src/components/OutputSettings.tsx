import { updateSettings } from "../store/settingsActions";
import { DEFAULT_SETTINGS, useSettingsStore } from "../store/settingsStore";
import type { FileNames } from "../types/models";
import { Button } from "./ui/Button";
import { Field, Input } from "./ui/Fields";

const NAME_FIELDS: { key: keyof FileNames; label: string; ext: string }[] = [
  { key: "json", label: "Células (JSON)", ext: ".json" },
  { key: "cells_md", label: "Células (Markdown)", ext: ".md" },
  { key: "vba_md", label: "Código VBA (Markdown)", ext: ".md" },
  { key: "bundle", label: "Arquivo único para IA (Markdown)", ext: ".md" },
];

/** Aba "Saída" das configurações: nomes dos arquivos. */
export function OutputSettings() {
  const names = useSettingsStore((s) => s.settings.file_names);

  const setName = (k: keyof FileNames) => (v: string) =>
    updateSettings({ file_names: { ...names, [k]: v } });

  return (
    <section className="flex flex-col gap-3">
      <div className="flex items-center gap-2">
        <h3 className="text-sm font-semibold">Nome dos arquivos</h3>
        <Button
          className="ml-auto"
          size="sm"
          variant="ghost"
          onClick={() => updateSettings({ file_names: DEFAULT_SETTINGS.file_names })}
        >
          Restaurar nomes
        </Button>
      </div>
      <p className="text-sm text-muted">
        Use {"{arquivo}"} (nome da planilha, sem extensão) e {"{data}"} (AAAA-MM-DD). A extensão é
        fixa e o JSON nunca muda de formato.
      </p>
      {NAME_FIELDS.map(({ key, label, ext }) => (
        <Field key={key} label={label}>
          <div className="flex items-center gap-1">
            <Input value={names[key]} onChange={(e) => setName(key)(e.currentTarget.value)} />
            <span className="font-mono text-sm text-muted">{ext}</span>
          </div>
        </Field>
      ))}
    </section>
  );
}
