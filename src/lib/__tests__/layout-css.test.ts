/// <reference types="node" />
import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const css = readFileSync(resolve(process.cwd(), "src/index.css"), "utf-8");
const read = (p: string) => readFileSync(resolve(process.cwd(), p), "utf-8");

describe("mise en page pleine largeur", () => {
  it("réserve la gouttière de défilement sur la racine de page", () => {
    expect(css).toMatch(/#main-content \.animate-page-in > \.overflow-y-auto\s*\{\s*scrollbar-gutter:\s*stable/);
  });

  it("rend la barre de la sidebar visible", () => {
    expect(css).toContain(".sidebar-scroll::-webkit-scrollbar-thumb");
    expect(read("src/components/layout/Sidebar.tsx")).toContain("sidebar-scroll");
  });

  it("ne plafonne plus Diagnostic et Aide à une colonne étroite", () => {
    for (const p of ["src/components/pages/DiagnosticPage.tsx", "src/components/pages/HelpPage.tsx"]) {
      expect(read(p)).not.toContain("max-w-3xl");
    }
    expect(read("src/components/pages/HelpPage.tsx")).toContain("max-w-[75ch]");
  });

  it("aligne Diagnostic sur le conteneur standard des pages", () => {
    expect(read("src/components/pages/DiagnosticPage.tsx")).toContain('className="p-7 overflow-y-auto h-full"');
  });
});
