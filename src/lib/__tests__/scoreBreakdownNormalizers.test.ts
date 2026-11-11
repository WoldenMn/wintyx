import { describe, it, expect } from "vitest";
import { getDomainConfig } from "../scoreBreakdownNormalizers";

describe("libellés du détail des scores", () => {
  it("francise les catégories et niveaux du durcissement", () => {
    const [item] = getDomainConfig("hardening").normalize([
      { id: "RDP", name: "RDP", description: "", category: "Remote Access", riskLevel: "Essential", isSecure: false, isUnknown: false },
    ]);
    expect(item.category).toBe("Accès distant");
    expect(item.weightLabel).toBe("Essentiel");
  });

  it("laisse les réglages facultatifs hors du détail du score de protection des données", () => {
    const items = getDomainConfig("dataProtection").normalize([
      { id: "a", name: "A", description: "", category: "Telemetrie", isProtected: false, recommended: true },
      { id: "s", name: "SmartScreen", description: "", category: "Telemetrie", isProtected: false, recommended: false },
    ]);
    expect(items.map((i) => i.id)).toEqual(["a"]);
  });

  it("francise les catégories de la protection des données, de la performance et du bloatware", () => {
    const [dp] = getDomainConfig("dataProtection").normalize([
      { id: "a", name: "A", description: "", category: "Telemetrie", isProtected: true, recommended: true },
    ]);
    const [perf] = getDomainConfig("performance").normalize([
      { id: "b", name: "B", description: "", category: "Reseau", isOptimized: false },
    ]);
    const [bloat] = getDomainConfig("bloatware").normalize([
      { packageName: "p", displayName: "P", category: "Entertainment", publisher: "", isInstalled: true },
    ]);
    expect(dp.category).toBe("Télémétrie");
    expect(perf.category).toBe("Réseau");
    expect(bloat.category).toBe("Divertissement");
  });
});
