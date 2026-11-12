import { useEffect, useRef } from "react";
import { CheckCircle2, X } from "lucide-react";

import { Button } from "./Button";

interface ToastProps {
  message: string;
  actionLabel?: string;
  onAction?: () => void;
  onClose: () => void;
  /** ms até sumir. Padrão 4000. */
  duration?: number;
}

/** Renderize condicionalmente: monta = aparece; some sozinho após `duration`. */
export function Toast({ message, actionLabel, onAction, onClose, duration = 4000 }: ToastProps) {
  const closeRef = useRef(onClose);
  closeRef.current = onClose;

  useEffect(() => {
    const t = setTimeout(() => closeRef.current(), duration);
    return () => clearTimeout(t);
  }, [message, duration]);

  return (
    <div
      role="status"
      className="fixed right-4 bottom-4 z-50 flex items-center gap-3 rounded-lg border border-border bg-surface p-3 shadow-md"
    >
      <CheckCircle2 size={16} strokeWidth={1.75} className="text-success" aria-hidden />
      <span className="text-sm">{message}</span>
      {actionLabel && onAction && (
        <Button size="sm" variant="ghost" onClick={onAction}>
          {actionLabel}
        </Button>
      )}
      <Button size="sm" variant="ghost" aria-label="Fechar" onClick={onClose} icon={X} />
    </div>
  );
}
