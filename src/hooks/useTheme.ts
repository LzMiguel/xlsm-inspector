import { useEffect, useState } from "react";

export type Theme = "system" | "light" | "dark";

const KEY = "xlsm-inspector:theme";

function read(): Theme {
  try {
    const v = localStorage.getItem(KEY);
    if (v === "light" || v === "dark") return v;
  } catch {
    // storage indisponível: segue o sistema
  }
  return "system";
}

/** "system" = sem `data-theme` → o CSS segue `prefers-color-scheme`. */
export function useTheme() {
  const [theme, setTheme] = useState<Theme>(read);

  useEffect(() => {
    const el = document.documentElement;
    if (theme === "system") el.removeAttribute("data-theme");
    else el.dataset.theme = theme;
    try {
      if (theme === "system") localStorage.removeItem(KEY);
      else localStorage.setItem(KEY, theme);
    } catch {
      // ignora
    }
  }, [theme]);

  const cycle = () => setTheme((t) => (t === "system" ? "light" : t === "light" ? "dark" : "system"));
  return { theme, cycle };
}
