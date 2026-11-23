import { useState } from "react";
import { FileDown } from "lucide-react";

import { exportVbaFiles, pickFolder, revealInFolder, toAppError } from "../api/commands";
import { updateVbaOptions } from "../store/settingsActions";
import { useSettingsStore } from "../store/settingsStore";
import { refreshVba } from "../store/workbookActions";
import { useWorkbookStore } from "../store/workbookStore";
import type { AppError } from "../types/models";
import { ErrorAlert } from "./ErrorAlert";
import { Badge } from "./ui/Badge";
import { Button } from "./ui/Button";
import { Checkbox } from "./ui/Checkbox";
import { cn } from "./ui/cn";
import { Toast } from "./ui/Toast";
import { KIND_GROUP_LABEL, KIND_ORDER } from "./vbaKinds";

export function ComponentList() {
  const vba = useWorkbookStore((s) => s.result?.vba ?? []);
  const selected = useWorkbookStore((s) => s.selectedComponent);
  const select = useWorkbookStore((s) => s.selectComponent);
  const goToVba = useWorkbookStore((s) => s.goToVba);
  const filePath = useWorkbookStore((s) => s.filePath);
  const defaultDir = useSettingsStore((s) => s.settings.default_out_dir);
  const vbaOptions = useSettingsStore((s) => s.settings.vba_options);
  const [done, setDone] = useState<string[] | null>(null);
  const [error, setError] = useState<AppError | null>(null);

  async function exportFiles() {
    if (!filePath) return;
    try {
      const dir = await pickFolder(defaultDir ?? undefined);
      if (dir) setDone(await exportVbaFiles(filePath, dir));
    } catch (e) {
      setError(toAppError(e));
    }
  }

  const changeOption = (o: Partial<typeof vbaOptions>) => {
    updateVbaOptions(o);
    void refreshVba();
  };

  if (vba.length === 0) return <p className="text-sm text-subtle">Nenhum componente VBA.</p>;

  return (
    <nav aria-label="Componentes VBA" className="flex flex-col gap-4">
      <div className="flex flex-col gap-1">
        <Checkbox
          label="Remover linhas em branco"
          checked={vbaOptions.remove_blank_lines}
          onChange={(v) => changeOption({ remove_blank_lines: v })}
        />
        <Checkbox
          label="Ocultar linhas Attribute VB_"
          checked={vbaOptions.hide_attributes}
          onChange={(v) => changeOption({ hide_attributes: v })}
        />
      </div>
      {KIND_ORDER.map((kind) => {
        const items = vba.filter((c) => c.kind === kind);
        if (items.length === 0) return null;
        return (
          <section key={kind} aria-label={KIND_GROUP_LABEL[kind]}>
            <h2 className="mb-1 text-xs font-medium tracking-wide text-muted uppercase">
              {KIND_GROUP_LABEL[kind]}
            </h2>
            <ul className="flex flex-col gap-1">
              {items.map((c) => {
                const active = c.name === selected;
                return (
                  <li key={c.name}>
                    <button
                      type="button"
                      aria-current={active || undefined}
                      onClick={() => select(c.name)}
                      className={cn(
                        "flex w-full items-center gap-2 rounded-sm px-2 py-1 text-left text-sm",
                        "transition-colors duration-150 ease-out",
                        "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent",
                        active ? "bg-primary-soft text-text" : "text-text hover:bg-surface-muted",
                      )}
                    >
                      <span className="flex-1 truncate" title={c.name}>
                        {c.name}
                      </span>
                      <Badge>{c.line_count}</Badge>
                    </button>
                    {active && c.procedures.length > 0 && (
                      <ul aria-label={`Procedimentos de ${c.name}`} className="mt-1 mb-1 ml-3">
                        {c.procedures.map((p) => (
                          <li key={`${p.start_line}`}>
                            <button
                              type="button"
                              onClick={() => goToVba(c.name, p.start_line)}
                              title={`${p.visibility} ${p.kind} · linhas ${p.start_line}-${p.end_line}`}
                              className="w-full truncate rounded-sm px-2 py-0.5 text-left text-xs text-muted hover:bg-surface-muted hover:text-text focus-visible:outline-2 focus-visible:outline-accent"
                            >
                              {p.name}
                              <span className="text-subtle"> · {p.kind}</span>
                            </button>
                          </li>
                        ))}
                      </ul>
                    )}
                  </li>
                );
              })}
            </ul>
          </section>
        );
      })}
      <Button size="sm" icon={FileDown} onClick={exportFiles}>
        Exportar como arquivos
      </Button>
      {error && <ErrorAlert error={error} />}
      {done && (
        <Toast
          message={`${done.length} arquivo(s) .bas/.cls/.frm gerado(s).`}
          actionLabel="Abrir pasta"
          onAction={() => {
            if (done[0]) revealInFolder(done[0]).catch((e) => setError(toAppError(e)));
          }}
          onClose={() => setDone(null)}
        />
      )}
    </nav>
  );
}
