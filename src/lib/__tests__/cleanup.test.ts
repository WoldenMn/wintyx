import { describe, it, expect } from "vitest";
import { isCleanable, describeCleanAll } from "../cleanup";
import type { CleanupTarget } from "../types";

function target(over: Partial<CleanupTarget>): CleanupTarget {
  return {
    id: "t",
    name: "Cible",
    description: "",
    path: "",
    sizeBytes: 1_048_576,
    sizeDisplay: "1 Mo",
    isSystem: false,
    accessible: true,
    ...over,
  };
}

describe("isCleanable", () => {
  it("garde les cibles accessibles qui contiennent quelque chose", () => {
    expect(isCleanable(target({}))).toBe(true);
    expect(isCleanable(target({ sizeBytes: 0 }))).toBe(false);
    expect(isCleanable(target({ accessible: false, sizeBytes: 0 }))).toBe(false);
  });
});

describe("describeCleanAll", () => {
  const items = [
    target({ id: "a", name: "Temp utilisateur", sizeBytes: 2_097_152, sizeDisplay: "2 Mo" }),
    target({ id: "b", name: "Corbeille", sizeBytes: 1_048_576, sizeDisplay: "1 Mo" }),
    target({ id: "c", name: "Temp Windows", sizeBytes: 0, sizeDisplay: "Inaccessible", accessible: false }),
    target({ id: "d", name: "Cache vide", sizeBytes: 0, sizeDisplay: "0 o" }),
  ];

  it("compte les cibles nettoyables et cite les inaccessibles à part", () => {
    const text = describeCleanAll(items, (n) => `${n} o`);
    expect(text).toContain("2 cibles vont être nettoyées (3145728 o)");
    expect(text).toContain("- Temp utilisateur (2 Mo)");
    expect(text).toContain("- Corbeille (1 Mo)");
    expect(text).toContain("Non nettoyée (droits administrateur requis) :\n- Temp Windows");
    expect(text).not.toContain("Cache vide");
  });

  it("accorde au singulier et au pluriel", () => {
    expect(describeCleanAll([items[0]], (n) => `${n} o`)).toContain("1 cible va être nettoyée");
    const two = [items[0], items[2], target({ id: "e", name: "Prefetch", accessible: false, sizeBytes: 0 })];
    expect(describeCleanAll(two, (n) => `${n} o`)).toContain("Non nettoyées (droits administrateur requis)");
  });

  it("ne mentionne aucun inaccessible quand il n'y en a pas", () => {
    expect(describeCleanAll([items[0]], (n) => `${n} o`)).not.toContain("Non nettoyée");
  });
});
