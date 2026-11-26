import { useWorkbookStore } from '../store/workbookStore'
import type { HiddenKind } from './marks'
import { Checkbox } from './ui/Checkbox'

const EMPTY: string[] = []

function Items({ kind, title }: { kind: HiddenKind; title: string }) {
  const names =
    useWorkbookStore((s) =>
      kind === 'queries' ? s.summary?.queries : s.summary?.connections,
    ) ?? EMPTY
  const hidden = useWorkbookStore((s) => s.hidden[kind])
  const toggle = useWorkbookStore((s) => s.toggleHidden)
  const busy = useWorkbookStore(
    (s) => s.status === 'extracting' || s.status === 'loading',
  )

  if (names.length === 0) return null

  return (
    <section aria-label={title}>
      <h3 className="mb-1 text-xs font-medium tracking-wide text-muted uppercase">
        {title}
      </h3>
      <ul className="flex flex-col gap-1">
        {names.map((name, i) => (
          <li key={`${name}-${i}`}>
            <Checkbox
              className="w-full rounded-sm px-1 py-1 hover:bg-surface-muted"
              checked={!hidden.includes(name)}
              disabled={busy}
              onChange={() => toggle(kind, name)}
              label={
                <span
                  className="min-w-0 flex-1 truncate"
                  title={name}
                >
                  {name}
                </span>
              }
            />
          </li>
        ))}
      </ul>
    </section>
  )
}

/** Seção Power Query da sidebar: consultas e conexões. */
export function PowerQueryList() {
  const summary = useWorkbookStore((s) => s.summary)
  const hidden = useWorkbookStore((s) => s.hidden)
  const setHidden = useWorkbookStore((s) => s.setHidden)
  const busy = useWorkbookStore(
    (s) => s.status === 'extracting' || s.status === 'loading',
  )

  if (!summary) return null
  const { queries, connections } = summary
  const total = queries.length + connections.length
  if (total === 0) return null

  const marked =
    queries.filter((n) => !hidden.queries.includes(n)).length +
    connections.filter((n) => !hidden.connections.includes(n)).length

  return (
    <nav
      aria-label="Power Query"
      className="flex flex-col gap-2"
    >
      <Checkbox
        label={
          <span className="font-semibold">
            Power Query{' '}
            <span className="font-normal text-muted">
              ({marked}/{total})
            </span>
          </span>
        }
        checked={marked === total}
        indeterminate={marked > 0 && marked < total}
        disabled={busy}
        onChange={(all) => {
          setHidden('queries', all ? [] : queries)
          setHidden('connections', all ? [] : connections)
        }}
      />
      <Items
        kind="queries"
        title="Consultas"
      />
      <Items
        kind="connections"
        title="Conexões"
      />
    </nav>
  )
}
