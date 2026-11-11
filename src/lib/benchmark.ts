import type { BenchmarkEntry } from "./types";

/** Premiere version dont la metrique CPU du banc est comparable a l'actuelle. */
export const MIN_COMPARABLE_VERSION = "2.18.0";

export function compareVersions(a: string, b: string): number {
  const pa = a.split(".").map((n) => Number.parseInt(n, 10) || 0);
  const pb = b.split(".").map((n) => Number.parseInt(n, 10) || 0);
  for (let i = 0; i < Math.max(pa.length, pb.length); i++) {
    const diff = (pa[i] ?? 0) - (pb[i] ?? 0);
    if (diff !== 0) return diff;
  }
  return 0;
}

export function isPreviousVersionEntry(entry: BenchmarkEntry): boolean {
  return !entry.version || compareVersions(entry.version, MIN_COMPARABLE_VERSION) < 0;
}

/** `history` est trie du plus recent au plus ancien. */
export function estimateLabel(history: BenchmarkEntry[]): string {
  const last = history.find((e) => !isPreviousVersionEntry(e) && e.plausible !== false && e.result.durationMs > 0);
  if (!last) return "Estimation : quelques secondes";
  const seconds = Math.max(1, Math.round(last.result.durationMs / 1000));
  return `Estimation : environ ${seconds} ${seconds > 1 ? "secondes" : "seconde"}`;
}
