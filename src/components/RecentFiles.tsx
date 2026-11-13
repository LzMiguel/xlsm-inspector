import { Clock, FileSpreadsheet, X } from "lucide-react";

import { openWorkbook } from "../store/workbookActions";
import { useWorkbookStore } from "../store/workbookStore";
import { Button } from "./ui/Button";

const folderOf = (path: string) => path.slice(0, Math.max(path.lastIndexOf("\\"), path.lastIndexOf("/")));

const when = (ms: number) =>
  new Date(ms).toLocaleString("pt-BR", { dateStyle: "short", timeStyle: "short" });

export function RecentFiles({ disabled }: { disabled: boolean }) {
  const recent = useWorkbookStore((s) => s.recent);
  const remove = useWorkbookStore((s) => s.removeRecent);
  const clear = useWorkbookStore((s) => s.clearRecent);

  if (recent.length === 0) return null;

  return (
    <section aria-label="Arquivos recentes">
      <div className="mb-2 flex items-center gap-2">
        <Clock size={16} strokeWidth={1.75} className="text-subtle" aria-hidden />
        <h2 className="text-lg font-semibold">Recentes</h2>
        <Button variant="ghost" size="sm" className="ml-auto" onClick={clear}>
          Limpar histórico
        </Button>
      </div>
      <ul className="divide-y divide-border rounded-md border border-border bg-surface">
        {recent.map((r) => (
          <li key={r.path} className="flex items-center gap-1 pr-2">
            <button
              type="button"
              disabled={disabled}
              onClick={() => openWorkbook(r.path)}
              title={r.path}
              className="flex min-w-0 flex-1 items-center gap-3 p-3 text-left hover:bg-surface-muted focus-visible:outline-2 focus-visible:outline-accent disabled:opacity-50"
            >
              <FileSpreadsheet size={16} strokeWidth={1.75} className="shrink-0 text-primary" aria-hidden />
              <span className="min-w-0 flex-1">
                <span className="block truncate text-base font-medium">{r.name}</span>
                <span className="block truncate font-mono text-xs text-muted">{folderOf(r.path)}</span>
              </span>
              <span className="shrink-0 text-xs text-muted tabular-nums">{when(r.openedAt)}</span>
            </button>
            <button
              type="button"
              aria-label={`Remover ${r.name} do histórico`}
              onClick={() => remove(r.path)}
              className="rounded-sm p-1 text-subtle hover:bg-surface-muted hover:text-text focus-visible:outline-2 focus-visible:outline-accent"
            >
              <X size={16} strokeWidth={1.75} aria-hidden />
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
}
