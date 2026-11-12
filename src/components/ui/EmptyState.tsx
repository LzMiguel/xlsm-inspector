import type { ReactNode } from "react";
import type { LucideIcon } from "lucide-react";

export function EmptyState({
  icon: Icon,
  title,
  description,
  children,
}: {
  icon: LucideIcon;
  title: string;
  description?: string;
  children?: ReactNode;
}) {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-2 p-12 text-center">
      <Icon size={48} strokeWidth={1.5} className="text-subtle" aria-hidden />
      <h2 className="text-xl font-semibold">{title}</h2>
      {description && <p className="max-w-md text-base text-muted">{description}</p>}
      {children && <div className="mt-4">{children}</div>}
    </div>
  );
}
