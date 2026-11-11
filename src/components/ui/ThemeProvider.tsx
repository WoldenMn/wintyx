import { createContext, useCallback, useContext, useState, useEffect, useMemo, type ReactNode } from "react";

export type ThemeId = "obsidian" | "daylight";

export interface ThemeInfo {
  id: ThemeId;
  name: string;
  description: string;
  preview: string[];
}

export const themes: ThemeInfo[] = [
  {
    id: "obsidian",
    name: "Obsidian",
    description: "Sombre, sobre, accent ion",
    preview: ["#0E0F13", "#16181F", "#2EC4D6", "#E7E8EC"],
  },
  {
    id: "daylight",
    name: "Daylight",
    description: "Clair, net, accent ion",
    preview: ["#FAFAFB", "#FFFFFF", "#0B6E7C", "#14161C"],
  },
];

const STORAGE_KEY = "wintyx-theme";
const VALID_THEME_IDS = new Set<ThemeId>(themes.map((t) => t.id));

/** Migre une valeur stockee (incluant les anciens themes v2.x) vers un ThemeId valide. */
export function migrateThemeId(raw: string | null): ThemeId {
  if (raw === "obsidian" || raw === "daylight") return raw;
  if (raw === "nordic") return "daylight";
  // default / cyberpunk / glass / inconnu / null -> sombre par defaut
  return "obsidian";
}

interface ThemeContextType {
  theme: ThemeId;
  setTheme: (t: ThemeId) => void;
  themeInfo: ThemeInfo;
}

const ThemeContext = createContext<ThemeContextType | null>(null);

function readStoredTheme(): string | null {
  try {
    return localStorage.getItem(STORAGE_KEY);
  } catch (e) {
    console.warn("Thème mémorisé illisible :", e);
    return null;
  }
}

function writeStoredTheme(theme: ThemeId) {
  try {
    localStorage.setItem(STORAGE_KEY, theme);
  } catch (e) {
    console.warn("Thème non mémorisé :", e);
  }
}

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setThemeState] = useState<ThemeId>(() => {
    const migrated = migrateThemeId(readStoredTheme());
    // Reecrit la valeur migree pour stabiliser le stockage des anciens utilisateurs.
    writeStoredTheme(migrated);
    return migrated;
  });

  const setTheme = useCallback((t: ThemeId) => {
    if (!VALID_THEME_IDS.has(t)) return;
    setThemeState(t);
    writeStoredTheme(t);
  }, []);

  useEffect(() => {
    document.documentElement.setAttribute("data-theme", theme);
  }, [theme]);

  const themeInfo = useMemo(
    () => themes.find((t) => t.id === theme) ?? themes[0],
    [theme],
  );

  const value = useMemo(() => ({ theme, setTheme, themeInfo }), [theme, setTheme, themeInfo]);

  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
}

export function useTheme() {
  const ctx = useContext(ThemeContext);
  if (!ctx) throw new Error("useTheme must be used within ThemeProvider");
  return ctx;
}
