import { useDeferredValue, useMemo, useRef, useState } from "react";
import { Search, Table2 } from "lucide-react";
import { useVirtualizer } from "@tanstack/react-virtual";

import { useHotkey } from "../hooks/useHotkey";
import { useWorkbookStore } from "../store/workbookStore";
import type { CellContent, SheetCells, TableInfo } from "../types/models";
import { fmt } from "./cellFormat";
import { TableView } from "./TableView";
import { EmptyState } from "./ui/EmptyState";

interface Row {
  coord: string;
  formula: string;
  value: string;
  table?: TableInfo;
}

const COLUMNS = "grid grid-cols-[110px_minmax(0,1fr)_minmax(0,1fr)] gap-3 px-3";
const ROW_HEIGHT = 32;

function toRow(coord: string, c: CellContent): Row {
  if (typeof c === "object" && c !== null) {
    return { coord, formula: c.formula.replace(/\r?\n/g, " ↵ "), value: fmt(c.valor) };
  }
  return { coord, formula: "", value: fmt(c) };
}

export function CellTable({ sheets }: { sheets: SheetCells[] }) {
  // vindo do Diagnóstico: abre na aba e na célula (ou na tabela que a contém)
  const f = useWorkbookStore((s) => s.focus);
  const focus = f && sheets.some((s) => s.name === f.sheet) ? f : null;
  const [sheetName, setSheetName] = useState(focus?.sheet ?? sheets[0]?.name ?? "");
  const [query, setQuery] = useState(focus?.cell ?? "");
  const deferred = useDeferredValue(query);
  const searchRef = useRef<HTMLInputElement>(null);
  const scrollRef = useRef<HTMLDivElement>(null);

  useHotkey("f", () => searchRef.current?.focus());

  const [tableName, setTableName] = useState<string | null>(
    () =>
      sheets
        .find((s) => s.name === focus?.sheet)
        ?.tables.find((t) => t.cells.includes(focus?.cell ?? ""))?.name ?? null,
  );

  const sheet = sheets.find((s) => s.name === sheetName) ?? sheets[0];
  const table = sheet?.tables.find((t) => t.name === tableName) ?? null;

  // Cada tabela vira uma linha (intervalo | nome) na posição da sua primeira célula.
  const rows = useMemo<Row[]>(() => {
    if (!sheet) return [];
    const owner = new Map<string, TableInfo>();
    for (const t of sheet.tables) for (const k of t.cells) owner.set(k, t);
    const seen = new Set<TableInfo>();
    const out: Row[] = [];
    for (const [k, c] of Object.entries(sheet.cells)) {
      const t = owner.get(k);
      if (!t) out.push(toRow(k, c));
      else if (!seen.has(t)) {
        seen.add(t);
        out.push({ coord: t.range, formula: "", value: t.name, table: t });
      }
    }
    return out;
  }, [sheet]);

  const filtered = useMemo(() => {
    const q = deferred.trim().toLowerCase();
    if (!q) return rows;
    return rows.filter(
      (r) =>
        r.coord.toLowerCase().includes(q) ||
        r.formula.toLowerCase().includes(q) ||
        r.value.toLowerCase().includes(q),
    );
  }, [rows, deferred]);

  const virtualizer = useVirtualizer({
    count: filtered.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => ROW_HEIGHT,
    overscan: 12,
  });

  if (table) return <TableView table={table} onBack={() => setTableName(null)} />;

  if (sheets.length === 0) {
    return (
      <EmptyState
        icon={Table2}
        title="Nenhuma aba extraída"
        description="Marque ao menos uma aba na barra lateral e extraia de novo."
      />
    );
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex items-center gap-3 p-3">
        <select
          aria-label="Aba"
          value={sheet.name}
          onChange={(e) => {
            setSheetName(e.target.value);
            setTableName(null);
          }}
          className="h-8 max-w-56 rounded-sm border border-border bg-surface px-2 text-sm focus-visible:outline-2 focus-visible:outline-accent"
        >
          {sheets.map((s) => (
            <option key={s.name} value={s.name}>
              {s.name}
            </option>
          ))}
        </select>

        <label className="relative flex-1">
          <Search
            size={16}
            strokeWidth={1.75}
            className="pointer-events-none absolute top-2 left-2 text-subtle"
            aria-hidden
          />
          <input
            ref={searchRef}
            type="search"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Buscar por célula, fórmula ou valor (Ctrl+F)"
            aria-label="Buscar células"
            className="h-8 w-full rounded-sm border border-border bg-surface pr-2 pl-8 text-sm placeholder:text-subtle focus-visible:outline-2 focus-visible:outline-accent"
          />
        </label>

        {sheet.sampled && (
          <span
            title="A extração usou só parte das linhas desta aba"
            className="shrink-0 rounded-sm bg-warning-soft px-2 py-0.5 text-xs font-medium"
          >
            amostra: {sheet.sampled.kept_rows.toLocaleString("pt-BR")} de{" "}
            {sheet.sampled.total_rows.toLocaleString("pt-BR")} linhas
          </span>
        )}

        <span className="shrink-0 text-sm text-muted tabular-nums">
          {filtered.length.toLocaleString("pt-BR")} de {rows.length.toLocaleString("pt-BR")}
        </span>
      </div>

      <div
        role="table"
        aria-label={`Células da aba ${sheet.name}`}
        aria-rowcount={filtered.length + 1}
        className="flex min-h-0 flex-1 flex-col border-t border-border"
      >
        <div
          role="row"
          className={`${COLUMNS} h-8 shrink-0 items-center border-b border-border bg-surface-muted text-xs font-medium text-muted`}
        >
          <span role="columnheader">Célula</span>
          <span role="columnheader">Fórmula</span>
          <span role="columnheader">Valor</span>
        </div>

        <div ref={scrollRef} tabIndex={0} className="min-h-0 flex-1 overflow-auto">
          {filtered.length === 0 ? (
            <p className="p-6 text-center text-sm text-muted">
              {rows.length === 0 ? "Nenhuma célula com conteúdo aqui." : "Nada encontrado."}
            </p>
          ) : (
            <div style={{ height: virtualizer.getTotalSize() }} className="relative w-full">
              {virtualizer.getVirtualItems().map((item) => {
                const r = filtered[item.index];
                return (
                  <div
                    key={item.key}
                    role="row"
                    aria-rowindex={item.index + 2}
                    style={{ transform: `translateY(${item.start}px)`, height: ROW_HEIGHT }}
                    className={`${COLUMNS} absolute top-0 left-0 w-full items-center border-b border-border text-sm`}
                  >
                    <span role="cell" className="truncate font-mono">
                      {r.coord}
                    </span>
                    <span
                      role="cell"
                      title={r.formula}
                      className="truncate font-mono text-code-keyword"
                    >
                      {r.formula}
                    </span>
                    {r.table ? (
                      <button
                        type="button"
                        role="cell"
                        title={`Abrir tabela ${r.value}`}
                        onClick={() => setTableName(r.value)}
                        className="truncate text-left font-medium text-accent underline focus-visible:outline-2 focus-visible:outline-accent"
                      >
                        {r.value}
                      </button>
                    ) : (
                      <span role="cell" title={r.value} className="truncate tabular-nums">
                        {r.value}
                      </span>
                    )}
                  </div>
                );
              })}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
