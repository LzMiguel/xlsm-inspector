import type { CellValue } from "../types/models";

export function fmt(v: CellValue): string {
  if (v === null) return "";
  if (typeof v === "boolean") return v ? "VERDADEIRO" : "FALSO";
  return String(v).replace(/\r?\n/g, " ↵ ");
}
