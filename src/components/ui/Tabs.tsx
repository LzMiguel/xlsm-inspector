import type { KeyboardEvent } from "react";
import type { LucideIcon } from "lucide-react";

import { cn } from "./cn";

export interface TabItem<T extends string> {
  id: T;
  label: string;
  icon?: LucideIcon;
}

interface TabsProps<T extends string> {
  tabs: TabItem<T>[];
  value: T;
  onChange: (id: T) => void;
  className?: string;
}

export function Tabs<T extends string>({ tabs, value, onChange, className }: TabsProps<T>) {
  const onKeyDown = (e: KeyboardEvent<HTMLDivElement>) => {
    let nextIdx = -1;
    if (e.key === "ArrowRight" || e.key === "ArrowLeft") {
      const step = e.key === "ArrowRight" ? 1 : -1;
      const i = tabs.findIndex((t) => t.id === value);
      nextIdx = (i + step + tabs.length) % tabs.length;
    } else if (e.key === "Home") {
      nextIdx = 0;
    } else if (e.key === "End") {
      nextIdx = tabs.length - 1;
    }
    if (nextIdx < 0) return;
    e.preventDefault();
    const next = tabs[nextIdx];
    onChange(next.id);
    e.currentTarget.querySelector<HTMLElement>(`[data-tab="${next.id}"]`)?.focus();
  };

  return (
    <div
      role="tablist"
      onKeyDown={onKeyDown}
      className={cn("flex gap-4 border-b border-border", className)}
    >
      {tabs.map(({ id, label, icon: Icon }) => {
        const active = id === value;
        return (
          <button
            key={id}
            type="button"
            role="tab"
            data-tab={id}
            aria-selected={active}
            tabIndex={active ? 0 : -1}
            onClick={() => onChange(id)}
            className={cn(
              "-mb-px inline-flex h-10 items-center gap-2 border-b-2 text-sm font-medium",
              "transition-colors duration-150 ease-out",
              "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent",
              active
                ? "border-primary text-text"
                : "border-transparent text-muted hover:text-text",
            )}
          >
            {Icon && <Icon size={16} strokeWidth={1.75} aria-hidden />}
            {label}
          </button>
        );
      })}
    </div>
  );
}
