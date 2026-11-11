import { describe, it, expect } from "vitest";
import { compareVersions, isPreviousVersionEntry, estimateLabel, MIN_COMPARABLE_VERSION } from "../benchmark";
import type { BenchmarkEntry } from "../types";

function entry(over: Partial<BenchmarkEntry> & { durationMs?: number }): BenchmarkEntry {
  const { durationMs, ...rest } = over;
  return {
    timestamp: "2026-10-03 08:00:00",
    plausible: true,
    version: "2.18.0",
    result: {
      cpuScore: 500, memoryScore: 20000, diskReadScore: 2000, diskWriteScore: 1000,
      networkLatency: 10, overallScore: 90, durationMs: durationMs ?? 4500,
    },
    ...rest,
  } as BenchmarkEntry;
}

describe("compareVersions", () => {
  it("compare numériquement, pas lexicalement", () => {
    expect(compareVersions("2.9.1", "2.18.0")).toBeLessThan(0);
    expect(compareVersions("2.18.0", "2.18.0")).toBe(0);
    expect(compareVersions("2.18.1", "2.18.0")).toBeGreaterThan(0);
    expect(compareVersions("3.0.0", "2.18.0")).toBeGreaterThan(0);
  });
});

describe("isPreviousVersionEntry", () => {
  it("signale une entrée sans version ou antérieure à 2.18.0", () => {
    expect(MIN_COMPARABLE_VERSION).toBe("2.18.0");
    expect(isPreviousVersionEntry(entry({ version: null }))).toBe(true);
    expect(isPreviousVersionEntry(entry({ version: "2.17.1" }))).toBe(true);
  });

  it("garde comparables les entrées de 2.18.0 et après", () => {
    expect(isPreviousVersionEntry(entry({ version: "2.18.0" }))).toBe(false);
    expect(isPreviousVersionEntry(entry({ version: "2.19.0" }))).toBe(false);
  });
});

describe("estimateLabel", () => {
  it("annonce la durée de la dernière mesure, en secondes entières", () => {
    expect(estimateLabel([entry({ durationMs: 4500 })])).toBe("Estimation : environ 5 secondes");
    expect(estimateLabel([entry({ durationMs: 1200 })])).toBe("Estimation : environ 1 seconde");
  });

  it("ignore les entrées sans durée exploitable et retombe sur une valeur par défaut", () => {
    expect(estimateLabel([])).toBe("Estimation : quelques secondes");
    expect(estimateLabel([entry({ durationMs: 0 })])).toBe("Estimation : quelques secondes");
  });

  it("ne se fie qu'aux mesures d'une version comparable", () => {
    const old = entry({ version: null, durationMs: 30000 });
    expect(estimateLabel([old])).toBe("Estimation : quelques secondes");
  });
});
