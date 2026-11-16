import { useState } from "react";
import { Download } from "lucide-react";

import { exportOutputs, pickFolder, revealInFolder, toAppError } from "../api/commands";
import { useHotkey } from "../hooks/useHotkey";
import { useSettingsStore } from "../store/settingsStore";
import { useWorkbookStore } from "../store/workbookStore";
import type { AppError, ExportTargets, ExtractScope, FileNames, OnExists } from "../types/models";
import { ErrorAlert } from "./ErrorAlert";
import { ScopeSelector } from "./ScopeSelector";
import { Alert } from "./ui/Alert";
import { Button } from "./ui/Button";
import { Checkbox } from "./ui/Checkbox";
import { Input } from "./ui/Fields";
import { Toast } from "./ui/Toast";

const ROWS: { key: keyof ExportTargets; name: keyof FileNames; ext: string; label: string }[] = [
  { key: "json", name: "json", ext: ".json", label: "Células (JSON)" },
  { key: "cells_md", name: "cells_md", ext: ".md", label: "Células (Markdown)" },
  { key: "vba_md", name: "vba_md", ext: ".md", label: "Código VBA (Markdown)" },
];

/** Rodapé da barra lateral. Atalho: Ctrl+E. */
export function ExportPanel() {
  const filePath = useWorkbookStore((s) => s.filePath);
  const hasVba = useWorkbookStore((s) => s.summary?.has_vba ?? false);
  const selectedSheets = useWorkbookStore((s) => s.selectedSheets);
  const settings = useSettingsStore((s) => s.settings);
  const result = useWorkbookStore((s) => s.result);
  const busyElsewhere = useWorkbookStore((s) => s.status === "loading" || s.status === "extracting");

  const [targets, setTargets] = useState<ExportTargets>({ json: true, cells_md: true, vba_md: true });
  // ajustes só desta exportação; o padrão vem das configurações
  const extractScope = useWorkbookStore((s) => s.scope);
  const [exportScope, setExportScope] = useState<ExtractScope | null>(null);
  const [nameEdits, setNameEdits] = useState<Partial<FileNames>>({});
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<AppError | null>(null);
  const [conflict, setConflict] = useState<{ outDir: string; message: string } | null>(null);
  const [done, setDone] = useState<string[] | null>(null);

  // Exportação independente da extração; começa igual a ela.
  const scope = exportScope ?? extractScope;
  // Sugestão de nome para saída parcial (o VBA não é cortado).
  const suffix = scope.kind === "full" ? "" : "_amostra";
  const { json, cells_md, vba_md } = settings.file_names;
  const names: FileNames = {
    json: json + suffix,
    cells_md: cells_md + suffix,
    vba_md,
    ...nameEdits,
  };
  const vbaOn = targets.vba_md && hasVba;
  const needsCells = targets.json || targets.cells_md;
  const canExport =
    !busy && !busyElsewhere && (needsCells || vbaOn) && !(needsCells && selectedSheets.length === 0);
  const set = (k: keyof ExportTargets) => (v: boolean) => setTargets((t) => ({ ...t, [k]: v }));

  async function run(outDir: string, onExists: OnExists) {
    if (!filePath) return;
    setBusy(true);
    setError(null);
    setConflict(null);
    try {
      const files = await exportOutputs({
        path: filePath,
        outDir,
        targets: { ...targets, vba_md: vbaOn },
        sheets: selectedSheets,
        vbaOptions: settings.vba_options,
        names,
        template: settings.template,
        onExists,
        scope,
      });
      setDone(files);
    } catch (e) {
      const err = toAppError(e);
      if (err.code === "FileExists") setConflict({ outDir, message: err.message });
      else setError(err);
    } finally {
      setBusy(false);
    }
  }

  async function exportNow() {
    if (!filePath || !canExport) return;
    try {
      const outDir = await pickFolder(settings.default_out_dir ?? undefined);
      if (outDir) await run(outDir, "fail");
    } catch (e) {
      setError(toAppError(e));
    }
  }

  useHotkey("e", exportNow);

  return (
    <section aria-label="Exportação" className="flex flex-col gap-3 border-t border-border p-4">
      <h2 className="text-sm font-semibold">Exportar</h2>

      <ScopeSelector value={scope} onChange={setExportScope} disabled={busy} />

      <div className="flex flex-col gap-2">
        {ROWS.map(({ key, name, ext, label }) => {
          const disabled = key === "vba_md" && !hasVba;
          const checked = key === "vba_md" ? vbaOn : targets[key];
          return (
            <div key={key} className="flex flex-col gap-1">
              <Checkbox
                label={disabled ? `${label} (sem VBA)` : label}
                checked={checked}
                disabled={disabled}
                onChange={set(key)}
              />
              {checked && (
                <div className="flex items-center gap-1 pl-6">
                  <Input
                    aria-label={`Nome do arquivo ${ext}`}
                    className="py-1 font-mono text-xs"
                    value={names[name]}
                    onChange={(e) => {
                      const v = e.currentTarget.value;
                      setNameEdits((n) => ({ ...n, [name]: v }));
                    }}
                  />
                  <span className="font-mono text-xs text-muted">{ext}</span>
                </div>
              )}
            </div>
          );
        })}
      </div>

      {error && <ErrorAlert error={error} />}

      {conflict && (
        <Alert variant="warning" title="Já existem arquivos com esse nome">
          <p>{conflict.message}</p>
          <div className="mt-2 flex flex-wrap gap-2">
            <Button size="sm" onClick={() => run(conflict.outDir, "overwrite")}>
              Sobrescrever
            </Button>
            <Button size="sm" onClick={() => run(conflict.outDir, "suffix")}>
              Adicionar sufixo _2
            </Button>
            <Button size="sm" variant="ghost" onClick={() => setConflict(null)}>
              Cancelar
            </Button>
          </div>
        </Alert>
      )}

      <Button
        className="w-full"
        variant={result ? "primary" : "secondary"}
        icon={Download}
        loading={busy}
        disabled={!canExport}
        title="Ctrl+E"
        onClick={exportNow}
      >
        Exportar…
      </Button>

      {done && (
        <Toast
          message={`Concluído: ${done.length} arquivo(s) gerado(s).`}
          actionLabel="Abrir pasta"
          onAction={() => {
            if (done[0]) revealInFolder(done[0]).catch((e) => setError(toAppError(e)));
          }}
          onClose={() => setDone(null)}
        />
      )}
    </section>
  );
}
