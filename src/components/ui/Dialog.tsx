import { useEffect, useId, useRef, type ReactNode } from "react";
import { X } from "lucide-react";

import { Button } from "./Button";

interface DialogProps {
  open: boolean;
  onClose: () => void;
  title: string;
  children: ReactNode;
}

/** `<dialog>` nativo: foco preso, Esc fecha e o fundo fica inerte. */
export function Dialog({ open, onClose, title, children }: DialogProps) {
  const ref = useRef<HTMLDialogElement>(null);
  const titleId = useId();

  useEffect(() => {
    const d = ref.current;
    if (!d) return;
    if (open && !d.open) d.showModal();
    if (!open && d.open) d.close();
  }, [open]);

  return (
    <dialog
      ref={ref}
      aria-labelledby={titleId}
      onClose={onClose}
      onClick={(e) => e.target === e.currentTarget && onClose()}
      className="m-auto w-full max-w-lg rounded-lg border border-border bg-surface p-0 text-text shadow-md backdrop:bg-text/40"
    >
      <div className="flex items-center gap-2 border-b border-border px-4 py-3">
        <h2 id={titleId} className="text-lg font-semibold">
          {title}
        </h2>
        <Button className="ml-auto" size="sm" variant="ghost" icon={X} aria-label="Fechar" onClick={onClose} />
      </div>
      <div className="flex flex-col gap-6 p-4">{children}</div>
    </dialog>
  );
}
