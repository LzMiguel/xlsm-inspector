import { useState } from "react";
import { Download } from "lucide-react";

import { exportOutputs, pickFolder, revealInFolder, toAppError } from "../api/commands";
import { useHotkey } from "../hooks/useHotkey";
import { useWorkbookStore } from "../store/workbookStore";
import type { AppError, ExportTargets } from "../types/models";
import { ErrorAlert } from "./ErrorAlert";
import { Button } from "./ui/Button";
import { Checkbox } from "./ui/Checkbox";
import { Toast } from "./ui/Toast";

/** Rodapé da barra lateral. Atalho: Ctrl+E. */
export function ExportPanel() {
  const filePath = useWorkbookStore((s) => s.filePath);
  const hasVba = useWorkbookStore((s) => s.summary?.has_vba ?? false);
  const selectedSheets = useWorkbookStore((s) => s.selectedSheets);
  const vbaOptions = useWorkbookStore((s) => s.vbaOptions);
  const result = useWorkbookStore((s) => s.result);
  const busyElsewhere = useWorkbookStore((s) => s.status === "loading" || s.status === "extracting");

  const [targets, setTargets] = useState<ExportTargets>({ json: true, cells_md: true, vba_md: true });
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<AppError | null>(null);
  const [done, setDone] = useState<string[] | null>(null);

  const vbaOn = targets.vba_md && hasVba;
  const needsCells = targets.json || targets.cells_md;
  const canExport =
    !busy && !busyElsewhere && (needsCells || vbaOn) && !(needsCells && selectedSheets.length === 0);
  const set = (k: keyof ExportTargets) => (v: boolean) => setTargets((t) => ({ ...t, [k]: v }));

  async function exportNow() {
    if (!filePath || !canExport) return;
    setError(null);
    try {
      const outDir = await pickFolder();
      if (!outDir) return;
      setBusy(true);
      const files = await exportOutputs({
        path: filePath,
        outDir,
        targets: { ...targets, vba_md: vbaOn },
        sheets: selectedSheets,
        vbaOptions,
      });
      setDone(files);
    } catch (e) {
      setError(toAppError(e));
    } finally {
      setBusy(false);
    }
  }

  useHotkey("e", exportNow);

  return (
    <section aria-label="Exportação" className="flex flex-col gap-3 border-t border-border p-4">
      <h2 className="text-sm font-semibold">Exportar</h2>

      <div className="flex flex-col gap-2">
        <Checkbox label="mapeamento_planilha.json" checked={targets.json} onChange={set("json")} />
        <Checkbox label="mapeamento_planilha.md" checked={targets.cells_md} onChange={set("cells_md")} />
        <Checkbox
          label={hasVba ? "vba_code.md" : "vba_code.md (sem VBA)"}
          checked={vbaOn}
          disabled={!hasVba}
          onChange={set("vba_md")}
        />
      </div>

      {error && <ErrorAlert error={error} />}

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
