import { Code2, Stethoscope, Table2 } from "lucide-react";

import { useWorkbookStore, type PreviewTab } from "../store/workbookStore";
import { CellTable } from "./CellTable";
import { DiagnosticsView } from "./DiagnosticsView";
import { Tabs, type TabItem } from "./ui/Tabs";
import { VbaViewer } from "./VbaViewer";

const TABS: TabItem<PreviewTab>[] = [
  { id: "cells", label: "Células", icon: Table2 },
  { id: "vba", label: "VBA", icon: Code2 },
  { id: "diagnostics", label: "Diagnóstico", icon: Stethoscope },
];

export function Preview() {
  const tab = useWorkbookStore((s) => s.tab);
  const setTab = useWorkbookStore((s) => s.setTab);
  const sheets = useWorkbookStore((s) => s.result?.sheets ?? []);

  return (
    <section
      aria-label="Pré-visualização"
      className="flex h-[60vh] min-h-80 flex-col overflow-hidden rounded-md border border-border bg-surface"
    >
      <Tabs tabs={TABS} value={tab} onChange={setTab} className="px-4" />
      {tab === "cells" && <CellTable sheets={sheets} />}
      {tab === "vba" && <VbaViewer />}
      {tab === "diagnostics" && <DiagnosticsView />}
    </section>
  );
}
