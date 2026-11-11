import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ThemeProvider, migrateThemeId, useTheme, type ThemeId } from "../ThemeProvider";
import { ThemeSwitcher } from "../ThemeSwitcher";

describe("migrateThemeId", () => {
  it("garde obsidian", () => expect(migrateThemeId("obsidian")).toBe("obsidian"));
  it("garde daylight", () => expect(migrateThemeId("daylight")).toBe("daylight"));
  it("migre l'ancien default vers obsidian", () => expect(migrateThemeId("default")).toBe("obsidian"));
  it("migre cyberpunk vers obsidian", () => expect(migrateThemeId("cyberpunk")).toBe("obsidian"));
  it("migre glass vers obsidian", () => expect(migrateThemeId("glass")).toBe("obsidian"));
  it("migre nordic vers daylight", () => expect(migrateThemeId("nordic")).toBe("daylight"));
  it("retombe sur obsidian pour une valeur inconnue", () => expect(migrateThemeId("xxx")).toBe("obsidian"));
  it("retombe sur obsidian pour null", () => expect(migrateThemeId(null)).toBe("obsidian"));
});

function ThemeConsumer({ target }: { target: string }) {
  const { theme, setTheme } = useTheme();
  return (
    <div>
      <span data-testid="current-theme">{theme}</span>
      <button onClick={() => setTheme(target as ThemeId)}>changer</button>
    </div>
  );
}

describe("ThemeProvider", () => {
  beforeEach(() => {
    localStorage.clear();
    document.documentElement.removeAttribute("data-theme");
  });

  it("applique data-theme obsidian sur documentElement au montage par defaut", () => {
    render(
      <ThemeProvider>
        <div />
      </ThemeProvider>,
    );
    expect(document.documentElement.getAttribute("data-theme")).toBe("obsidian");
  });

  it("applique le theme persiste au montage", () => {
    localStorage.setItem("wintyx-theme", "daylight");
    render(
      <ThemeProvider>
        <div />
      </ThemeProvider>,
    );
    expect(document.documentElement.getAttribute("data-theme")).toBe("daylight");
  });

  it("setTheme applique et persiste le nouveau theme dans wintyx-theme", () => {
    render(
      <ThemeProvider>
        <ThemeConsumer target="daylight" />
      </ThemeProvider>,
    );
    fireEvent.click(screen.getByRole("button", { name: "changer" }));
    expect(screen.getByTestId("current-theme").textContent).toBe("daylight");
    expect(localStorage.getItem("wintyx-theme")).toBe("daylight");
    expect(document.documentElement.getAttribute("data-theme")).toBe("daylight");
  });

  it("setTheme ignore un id invalide (garde VALID_THEME_IDS)", () => {
    render(
      <ThemeProvider>
        <ThemeConsumer target="cyberpunk" />
      </ThemeProvider>,
    );
    fireEvent.click(screen.getByRole("button", { name: "changer" }));
    expect(screen.getByTestId("current-theme").textContent).toBe("obsidian");
    expect(localStorage.getItem("wintyx-theme")).toBe("obsidian");
    expect(document.documentElement.getAttribute("data-theme")).toBe("obsidian");
  });
});

describe("ThemeSwitcher", () => {
  beforeEach(() => {
    localStorage.clear();
    document.documentElement.removeAttribute("data-theme");
  });

  it("bascule obsidian vers daylight au clic et met a jour l'aria-label", () => {
    render(
      <ThemeProvider>
        <ThemeSwitcher />
      </ThemeProvider>,
    );
    const btn = screen.getByRole("button", { name: "Passer au thème clair" });
    fireEvent.click(btn);
    expect(document.documentElement.getAttribute("data-theme")).toBe("daylight");
    expect(btn.getAttribute("aria-label")).toBe("Passer au thème sombre");
  });

  it("bascule daylight vers obsidian au clic", () => {
    localStorage.setItem("wintyx-theme", "daylight");
    render(
      <ThemeProvider>
        <ThemeSwitcher />
      </ThemeProvider>,
    );
    fireEvent.click(screen.getByRole("button", { name: "Passer au thème sombre" }));
    expect(document.documentElement.getAttribute("data-theme")).toBe("obsidian");
    expect(localStorage.getItem("wintyx-theme")).toBe("obsidian");
  });
});
