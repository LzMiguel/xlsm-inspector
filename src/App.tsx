import { FileSpreadsheet } from "lucide-react";

import { ComponentList } from "./components/ComponentList";
import { DropZone } from "./components/DropZone";
import { ErrorAlert } from "./components/ErrorAlert";
import { ExportPanel } from "./components/ExportPanel";
import { ExtractPanel } from "./components/ExtractPanel";
import { FileCard } from "./components/FileCard";
import { Preview } from "./components/Preview";
import { SheetList } from "./components/SheetList";
import { ThemeToggle } from "./components/ThemeToggle";
import { useFileDrop } from "./hooks/useFileDrop";
import { useHotkey } from "./hooks/useHotkey";
import { chooseAndOpenWorkbook, openWorkbook } from "./store/workbookActions";
import { useWorkbookStore } from "./store/workbookStore";

function App() {
  const summary = useWorkbookStore((s) => s.summary);
  const result = useWorkbookStore((s) => s.result);
  const tab = useWorkbookStore((s) => s.tab);
  const status = useWorkbookStore((s) => s.status);
  const error = useWorkbookStore((s) => s.error);
  const dragging = useFileDrop(openWorkbook);

  const busy = status === "loading" || status === "extracting";
  useHotkey("o", () => {
    if (!busy) chooseAndOpenWorkbook();
  });

  return (
    <div className="flex h-full flex-col">
      {/* Barra superior: 48px */}
      <header className="flex h-12 shrink-0 items-center gap-2 border-b border-border bg-surface px-4">
        <FileSpreadsheet size={16} strokeWidth={1.75} className="text-primary" aria-hidden />
        <h1 className="truncate text-lg font-semibold">
          {summary?.file_name ?? "XLSM Inspector"}
        </h1>
        <div className="ml-auto">
          <ThemeToggle />
        </div>
      </header>

      <div className="flex min-h-0 flex-1">
        {/* Barra lateral: 240px. Abas (seleção) ou componentes VBA; exportação no rodapé. */}
        <aside
          aria-label="Abas e componentes"
          className="flex w-60 shrink-0 flex-col border-r border-border bg-surface"
        >
          <div className="min-h-0 flex-1 overflow-y-auto p-4">
            {!summary ? (
              <p className="text-sm text-subtle">Nenhum arquivo aberto.</p>
            ) : tab === "vba" && result ? (
              <ComponentList />
            ) : (
              <SheetList />
            )}
          </div>
          {summary && <ExportPanel />}
        </aside>

        {/* Conteúdo: scroll próprio */}
        <main className="min-w-0 flex-1 overflow-auto p-6">
          <div className="mx-auto flex max-w-5xl flex-col gap-4">
            {error && <ErrorAlert error={error} />}
            {summary ? (
              <>
                <FileCard summary={summary} disabled={busy} />
                <ExtractPanel />
                {result && <Preview />}
              </>
            ) : (
              <div className="mx-auto w-full max-w-2xl pt-8">
                <DropZone
                  dragging={dragging}
                  loading={status === "loading"}
                  invalid={error?.code === "UnsupportedFormat"}
                />
              </div>
            )}
          </div>
        </main>
      </div>
    </div>
  );
}

export default App;
