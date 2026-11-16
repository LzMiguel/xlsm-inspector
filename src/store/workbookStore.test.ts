import { beforeEach, describe, expect, it } from "vitest";

import type { ExtractionResult, VbaComponent, WorkbookSummary } from "../types/models";
import { useWorkbookStore } from "./workbookStore";

const summary: WorkbookSummary = {
  path: "C:\\x\\a.xlsm",
  file_name: "a.xlsm",
  size_bytes: 10,
  has_vba: true,
  sheets: [
    { name: "A", cell_count: 2, formula_count: 0 },
    { name: "B", cell_count: 1, formula_count: 1 },
  ],
};

const comp = (name: string): VbaComponent => ({ name, kind: "Module", code: "", line_count: 0 });
const result = (names: string[]): ExtractionResult => ({
  sheets: [],
  vba: names.map(comp),
  warnings: [],
});

const s = () => useWorkbookStore.getState();

describe("workbookStore", () => {
  beforeEach(() => s().reset());

  it("marca todas as abas ao receber o resumo", () => {
    s().startLoading("C:\\x\\a.xlsm");
    expect(s().status).toBe("loading");
    s().setSummary(summary);
    expect(s().selectedSheets).toEqual(["A", "B"]);
    expect(s().status).toBe("ready");
  });

  it("alterna e marca/desmarca todas", () => {
    s().setSummary(summary);
    s().toggleSheet("A");
    expect(s().selectedSheets).toEqual(["B"]);
    s().toggleSheet("A");
    expect(s().selectedSheets.sort()).toEqual(["A", "B"]);
    s().setAllSheets(false);
    expect(s().selectedSheets).toEqual([]);
    s().setAllSheets(true);
    expect(s().selectedSheets).toEqual(["A", "B"]);
  });

  it("selecionando o primeiro componente e preservando a escolha ao reextrair", () => {
    s().setSummary(summary);
    s().startExtracting();
    s().setResult(result(["M1", "M2"]));
    expect(s().selectedComponent).toBe("M1");
    s().selectComponent("M2");
    s().setVba([comp("M2"), comp("M3")]);
    expect(s().selectedComponent).toBe("M2");
    s().setVba([comp("M3")]);
    expect(s().selectedComponent).toBe("M3");
  });

  it("novo arquivo descarta o resultado", () => {
    s().setSummary(summary);
    s().setResult(result(["M1"]));
    s().startLoading("outro.xlsx");
    expect(s().result).toBeNull();
    expect(s().summary).toBeNull();
  });

  it("erro zera o progresso e marca o status", () => {
    s().setProgress({ stage: "cells", current: 1, total: 2, label: "A" });
    s().setError({ code: "CorruptedFile", message: "x" });
    expect(s().status).toBe("error");
    expect(s().progress).toBeNull();
    expect(s().error?.code).toBe("CorruptedFile");
  });

  it("guarda os avisos do resultado", () => {
    s().setResult({
      ...result([]),
      warnings: [{ kind: "MissingCache", sheet: "A", count: 3, message: "m" }],
    });
    expect(s().warnings).toHaveLength(1);
  });
});
