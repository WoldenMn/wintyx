import { Moon, Sun } from "lucide-react";
import { useTheme } from "./ThemeProvider";

export function ThemeSwitcher() {
  const { theme, setTheme } = useTheme();
  const isDark = theme === "obsidian";

  return (
    <button
      onClick={() => setTheme(isDark ? "daylight" : "obsidian")}
      aria-label={isDark ? "Passer au thème clair" : "Passer au thème sombre"}
      title={isDark ? "Thème clair" : "Thème sombre"}
      className="w-full flex items-center gap-2.5 px-3 py-2 rounded-md hover:bg-bg-hover text-fg-dim hover:text-fg-primary transition-colors text-sm focus:outline-none focus-visible:ring-2 focus-visible:ring-accent"
    >
      {isDark ? <Sun size={16} aria-hidden="true" /> : <Moon size={16} aria-hidden="true" />}
      <span>{isDark ? "Thème clair" : "Thème sombre"}</span>
    </button>
  );
}
