import { FileSpreadsheet } from "lucide-react";

import { EmptyState } from "./components/ui/EmptyState";
import { useWorkbookStore } from "./store/workbookStore";

function App() {
  const summary = useWorkbookStore((s) => s.summary);

  return (
    <div className="flex h-full flex-col">
      {/* Barra superior: 48px */}
      <header className="flex h-12 shrink-0 items-center gap-2 border-b border-border bg-surface px-4">
        <FileSpreadsheet size={16} strokeWidth={1.75} className="text-primary" aria-hidden />
        <h1 className="truncate text-lg font-semibold">
          {summary?.file_name ?? "XLSM Inspector"}
        </h1>
      </header>

      <div className="flex min-h-0 flex-1">
        {/* Barra lateral: 240px */}
        <aside
          aria-label="Abas e componentes"
          className="w-60 shrink-0 overflow-y-auto border-r border-border bg-surface p-4"
        >
          <p className="text-sm text-subtle">Nenhum arquivo aberto.</p>
        </aside>

        {/* Conteúdo: scroll próprio */}
        <main className="min-w-0 flex-1 overflow-auto">
          <EmptyState
            icon={FileSpreadsheet}
            title="Nenhum arquivo aberto"
            description="Abra um arquivo .xlsx ou .xlsm para ver as células e o código VBA."
          />
        </main>
      </div>
    </div>
  );
}

export default App;
