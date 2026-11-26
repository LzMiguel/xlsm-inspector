import type { PreviewTab } from '../store/workbookStore'
import type { ItemFilter, WorkbookSummary } from '../types/models'

/** Itens desmarcados na sidebar, por nome. Vazio = tudo marcado. */
export interface Hidden {
  vba: string[]
  queries: string[]
  connections: string[]
}

export type HiddenKind = keyof Hidden

export const NOTHING_HIDDEN: Hidden = {
  vba: [],
  queries: [],
  connections: [],
}

export const visible = <T extends { name: string }>(
  list: T[],
  hidden: string[],
) => list.filter((x) => !hidden.includes(x.name))

const allHidden = (names: string[], hidden: string[]) =>
  names.length > 0 && names.every((n) => hidden.includes(n))

/** Filtro para a exportação; só inclui o tipo que tem algo desmarcado. */
export function itemsFilter(
  summary: WorkbookSummary,
  hidden: Hidden,
): ItemFilter {
  const kept = (names: string[], off: string[]) =>
    off.length ? names.filter((n) => !off.includes(n)) : undefined
  return {
    modules: kept(
      summary.vba.map((m) => m.name),
      hidden.vba,
    ),
    queries: kept(summary.queries, hidden.queries),
    connections: kept(summary.connections, hidden.connections),
  }
}

/** Guias que somem porque a seção inteira foi desmarcada. */
export function hiddenTabs(
  summary: WorkbookSummary | null,
  selectedSheets: string[],
  hidden: Hidden,
): PreviewTab[] {
  if (!summary) return []
  const off: PreviewTab[] = []
  if (summary.sheets.length > 0 && selectedSheets.length === 0)
    off.push('cells')
  if (
    allHidden(
      summary.vba.map((m) => m.name),
      hidden.vba,
    )
  )
    off.push('vba')
  const pq = summary.queries.length + summary.connections.length
  if (
    pq > 0 &&
    summary.queries.every((n) => hidden.queries.includes(n)) &&
    summary.connections.every((n) => hidden.connections.includes(n))
  )
    off.push('powerquery')
  return off
}
