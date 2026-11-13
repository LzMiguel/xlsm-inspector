export interface RecentFile {
  path: string;
  name: string;
  /** Epoch em ms da última abertura. */
  openedAt: number;
}

const KEY = "xlsm-inspector:recent";
const MAX = 10;

export function loadRecent(): RecentFile[] {
  try {
    const raw: unknown = JSON.parse(localStorage.getItem(KEY) ?? "[]");
    if (!Array.isArray(raw)) return [];
    return raw
      .filter(
        (r): r is RecentFile =>
          typeof r?.path === "string" && typeof r?.name === "string" && typeof r?.openedAt === "number",
      )
      .slice(0, MAX);
  } catch {
    return [];
  }
}

export function saveRecent(list: RecentFile[]) {
  try {
    localStorage.setItem(KEY, JSON.stringify(list));
  } catch {
    // storage indisponível: o histórico vale só nesta sessão
  }
}

/** Move/insere o arquivo no topo, sem duplicar, respeitando o limite. */
export function withRecent(list: RecentFile[], file: RecentFile): RecentFile[] {
  return [file, ...list.filter((r) => r.path !== file.path)].slice(0, MAX);
}
