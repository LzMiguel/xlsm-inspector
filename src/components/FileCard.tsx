import { FileSpreadsheet } from "lucide-react";

import { chooseAndOpenWorkbook } from "../store/workbookActions";
import type { WorkbookSummary } from "../types/models";
import { Badge } from "./ui/Badge";
import { Button } from "./ui/Button";

function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  return `${(n / (1024 * 1024)).toFixed(1)} MB`;
}

export function FileCard({ summary, disabled }: { summary: WorkbookSummary; disabled: boolean }) {
  const n = summary.sheets.length;
  return (
    <section className="flex items-center gap-4 rounded-md border border-border bg-surface p-4">
      <FileSpreadsheet size={20} strokeWidth={1.75} className="shrink-0 text-primary" aria-hidden />
      <div className="min-w-0 flex-1">
        <p className="truncate text-base font-medium" title={summary.path}>
          {summary.file_name}
        </p>
        <p className="text-sm text-muted">
          {formatBytes(summary.size_bytes)} · {n} {n === 1 ? "aba" : "abas"}
        </p>
      </div>
      {summary.has_vba && <Badge variant="primary">VBA</Badge>}
      <Button size="sm" disabled={disabled} onClick={chooseAndOpenWorkbook}>
        Trocar arquivo
      </Button>
    </section>
  );
}
