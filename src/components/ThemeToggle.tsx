import { Monitor, Moon, Sun } from "lucide-react";

import { useTheme, type Theme } from "../hooks/useTheme";
import { Button } from "./ui/Button";

const INFO: Record<Theme, { label: string; icon: typeof Sun }> = {
  system: { label: "Tema: automático (segue o sistema)", icon: Monitor },
  light: { label: "Tema: claro", icon: Sun },
  dark: { label: "Tema: escuro", icon: Moon },
};

export function ThemeToggle() {
  const { theme, cycle } = useTheme();
  const { label, icon } = INFO[theme];
  return <Button variant="ghost" icon={icon} aria-label={label} title={label} onClick={cycle} />;
}
