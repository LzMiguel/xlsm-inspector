import { useWorkbookStore } from "../store/workbookStore";
import { Badge } from "./ui/Badge";
import { cn } from "./ui/cn";
import { KIND_GROUP_LABEL, KIND_ORDER } from "./vbaKinds";

export function ComponentList() {
  const vba = useWorkbookStore((s) => s.result?.vba ?? []);
  const selected = useWorkbookStore((s) => s.selectedComponent);
  const select = useWorkbookStore((s) => s.selectComponent);

  if (vba.length === 0) return <p className="text-sm text-subtle">Nenhum componente VBA.</p>;

  return (
    <nav aria-label="Componentes VBA" className="flex flex-col gap-4">
      {KIND_ORDER.map((kind) => {
        const items = vba.filter((c) => c.kind === kind);
        if (items.length === 0) return null;
        return (
          <section key={kind} aria-label={KIND_GROUP_LABEL[kind]}>
            <h2 className="mb-1 text-xs font-medium tracking-wide text-muted uppercase">
              {KIND_GROUP_LABEL[kind]}
            </h2>
            <ul className="flex flex-col gap-1">
              {items.map((c) => {
                const active = c.name === selected;
                return (
                  <li key={c.name}>
                    <button
                      type="button"
                      aria-current={active || undefined}
                      onClick={() => select(c.name)}
                      className={cn(
                        "flex w-full items-center gap-2 rounded-sm px-2 py-1 text-left text-sm",
                        "transition-colors duration-150 ease-out",
                        "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent",
                        active ? "bg-primary-soft text-text" : "text-text hover:bg-surface-muted",
                      )}
                    >
                      <span className="flex-1 truncate" title={c.name}>
                        {c.name}
                      </span>
                      <Badge>{c.line_count}</Badge>
                    </button>
                  </li>
                );
              })}
            </ul>
          </section>
        );
      })}
    </nav>
  );
}
