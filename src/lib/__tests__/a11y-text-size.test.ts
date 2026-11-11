import { describe, it, expect } from "vitest";

const sources = import.meta.glob(["../../**/*.{ts,tsx}", "!../../**/__tests__/**", "!../../**/*.test.*"], {
  query: "?raw",
  import: "default",
  eager: true,
}) as Record<string, string>;

describe("A11Y-08 taille de texte minimale", () => {
  it("parcourt bien les sources de l'application", () => {
    expect(Object.keys(sources).length).toBeGreaterThan(50);
  });

  it("n'utilise aucune taille de texte arbitraire sous 12 px", () => {
    const offenders = Object.entries(sources)
      .filter(([, content]) => /text-\[(?:[0-9]|1[01])px\]/.test(content))
      .map(([path]) => path);
    expect(offenders).toEqual([]);
  });
});
