import { useEffect, useRef } from "react";

/** Ctrl/Cmd + `key` (minúscula). Bloqueia o atalho padrão da WebView (ex.: Ctrl+F). */
export function useHotkey(key: string, handler: () => void) {
  const ref = useRef(handler);
  ref.current = handler;

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && !e.altKey && !e.shiftKey && e.key.toLowerCase() === key) {
        e.preventDefault();
        ref.current();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [key]);
}
