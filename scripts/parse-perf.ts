#!/usr/bin/env bun
// Usage : bun run scripts/parse-perf.ts [date]
// Lit %LOCALAPPDATA%/Wintyx/Logs/perf-{backend.csv,frontend.jsonl}
// Ecrit audits/perf-baseline-YYYY-MM-DD.md

import { readFileSync, writeFileSync, existsSync, mkdirSync } from "node:fs";
import { join } from "node:path";

export type BackendSample = { ts: number; cmd: string; durationUs: number };
export type FrontendSample = {
  ts: number;
  cmd: string;
  argsKey: string;
  durationMs: number;
  cacheHit: boolean;
};

export type AggregateStats = {
  count: number;
  min: number;
  p50: number;
  p95: number;
  p99: number;
  max: number;
  mean: number;
  total: number;
};

function percentile(sorted: number[], p: number): number {
  if (sorted.length === 0) return 0;
  const idx = Math.min(sorted.length - 1, Math.floor((p / 100) * sorted.length));
  return sorted[idx];
}

export function aggregate(durations: number[]): AggregateStats {
  if (durations.length === 0) {
    return { count: 0, min: 0, p50: 0, p95: 0, p99: 0, max: 0, mean: 0, total: 0 };
  }
  const sorted = [...durations].sort((a, b) => a - b);
  const total = sorted.reduce((a, b) => a + b, 0);
  return {
    count: sorted.length,
    min: sorted[0],
    max: sorted[sorted.length - 1],
    p50: percentile(sorted, 50),
    p95: percentile(sorted, 95),
    p99: percentile(sorted, 99),
    mean: total / sorted.length,
    total,
  };
}

export function parseCsv(content: string): BackendSample[] {
  return content
    .split("\n")
    .filter((l) => l.trim().length > 0)
    .map((line): BackendSample | null => {
      const parts = line.split(",");
      if (parts.length !== 3) return null;
      const [ts, cmd, durationUs] = parts;
      const tsNum = Number(ts);
      const durationNum = Number(durationUs);
      if (!Number.isFinite(tsNum) || !Number.isFinite(durationNum) || cmd.length === 0) {
        return null;
      }
      return { ts: tsNum, cmd, durationUs: durationNum };
    })
    .filter((s): s is BackendSample => s !== null);
}

export function parseJsonl(content: string): FrontendSample[] {
  return content
    .split("\n")
    .filter((l) => l.trim().length > 0)
    .map((line): FrontendSample | null => {
      try {
        const parsed = JSON.parse(line) as FrontendSample;
        if (typeof parsed.cmd !== "string" || typeof parsed.durationMs !== "number") {
          return null;
        }
        return parsed;
      } catch {
        return null;
      }
    })
    .filter((s): s is FrontendSample => s !== null);
}

function groupByCmd<T extends { cmd: string }>(samples: T[]): Map<string, T[]> {
  const map = new Map<string, T[]>();
  for (const s of samples) {
    const list = map.get(s.cmd) ?? [];
    list.push(s);
    map.set(s.cmd, list);
  }
  return map;
}

export function renderMarkdown(input: {
  backendSamples: BackendSample[];
  frontendSamples: FrontendSample[];
  date: string;
}): string {
  const { backendSamples, frontendSamples, date } = input;

  const backendByCmd = groupByCmd(backendSamples);
  const frontendByCmd = groupByCmd(frontendSamples);

  const backendAggregates = Array.from(backendByCmd.entries()).map(([cmd, samples]) => ({
    cmd,
    ...aggregate(samples.map((s) => s.durationUs / 1000)), // us -> ms
  }));

  const frontendAggregates = Array.from(frontendByCmd.entries()).map(([cmd, samples]) => ({
    cmd,
    ...aggregate(samples.filter((s) => !s.cacheHit).map((s) => s.durationMs)),
  }));

  const top10Backend = [...backendAggregates].sort((a, b) => b.mean - a.mean).slice(0, 10);
  const top10Frontend = [...frontendAggregates].sort((a, b) => b.mean - a.mean).slice(0, 10);

  const lines: string[] = [];
  lines.push(`# Wintyx - Baseline performance ${date}`);
  lines.push("");
  lines.push(`**Build** : feature cargo perf et VITE_PERF=1`);
  lines.push(`**Session** : parcours manuel des pages a mesurer`);
  lines.push(`**Backend samples** : ${backendSamples.length}`);
  lines.push(`**Frontend samples** : ${frontendSamples.length} (dont ${frontendSamples.filter((s) => s.cacheHit).length} cache hits)`);
  lines.push("");
  lines.push("## Note methodologique");
  lines.push("");
  lines.push("- **Percentile naive** : `p50/p95/p99` = nearest-rank inferieur (`floor(N * p / 100)`). Biais notable sur petit N : pour N=1, `p50 = p95 = p99 = max`. Pour N=10, `p95` correspond au 9e plus grand. Interpreter avec prudence sous N<20.");
  lines.push("- **Cache hits frontend** : les samples `cacheHit=true` ont `durationMs=0` et sont **exclus** des agregats frontend (ils gonfleraient le compteur sans refleter de travail reel). Le compteur global `Frontend samples` ci-dessus les inclut pour visibilite, mais les tableaux de hotspots filtrent.");
  lines.push("- **Delta frontend - backend** : positif = overhead Tauri IPC + serde + scheduler React. Delta > 50ms sur une commande au backend < 100ms suggere un payload volumineux (serialization couteuse) ou un re-render React lourd a investiguer.");
  lines.push("- **Premier call** : peut etre plus lent que les suivants (init COM/WMI lazy, cold cache). Le rapport ne discrimine pas first vs subsequent, c'est attendu pour une baseline N=1 session typique.");
  lines.push("- **Early returns** : les commandes qui font `require_admin()?` ou `?` sur erreur sont mesurees (`PerfGuard` RAII capture le drop sur tous les paths). Une commande qui echoue tot apparait avec une duree faible, normal.");
  lines.push("");

  const renderTable = (rows: Array<{ cmd: string } & AggregateStats>) => {
    const out: string[] = [];
    out.push("| Rang | Commande | Samples | p50 | p95 | max | mean | Total cumule |");
    out.push("|---|---|---:|---:|---:|---:|---:|---:|");
    rows.forEach((r, i) => {
      out.push(
        `| ${i + 1} | ${r.cmd} | ${r.count} | ${r.p50.toFixed(1)} | ${r.p95.toFixed(1)} | ${r.max.toFixed(1)} | ${r.mean.toFixed(1)} | ${r.total.toFixed(0)} |`,
      );
    });
    return out.join("\n");
  };

  lines.push("## 1. Top 10 hotspots backend (ms, temps Rust pur)");
  lines.push("");
  lines.push(renderTable(top10Backend));
  lines.push("");

  lines.push("## 2. Top 10 hotspots frontend (ms, invoke roundtrip)");
  lines.push("");
  lines.push(renderTable(top10Frontend));
  lines.push("");

  lines.push("## 3. Delta frontend - backend (overhead Tauri/React)");
  lines.push("");
  lines.push("| Commande | Backend p50 (ms) | Frontend p50 (ms) | Delta | Delta % |");
  lines.push("|---|---:|---:|---:|---:|");
  for (const fa of frontendAggregates) {
    const ba = backendAggregates.find((b) => b.cmd === fa.cmd);
    if (!ba || ba.p50 === 0) continue;
    const delta = fa.p50 - ba.p50;
    const pct = (delta / ba.p50) * 100;
    lines.push(`| ${fa.cmd} | ${ba.p50.toFixed(1)} | ${fa.p50.toFixed(1)} | ${delta > 0 ? "+" : ""}${delta.toFixed(1)} | ${pct > 0 ? "+" : ""}${pct.toFixed(0)}% |`);
  }
  lines.push("");

  lines.push("## 4. Time-to-interactive par page");
  lines.push("");
  lines.push("_A completer manuellement depuis les markers `[perf] T3` console._");
  lines.push("");

  lines.push("## 5. Flux complets (mesures manuellement clic -> fin)");
  lines.push("");
  lines.push("_A completer manuellement pendant la session._");
  lines.push("");

  lines.push("## 6. Recommandations priorisees");
  lines.push("");
  lines.push("### P1 - Quick wins (ROI estime eleve)");
  for (const r of top10Backend.slice(0, 5)) {
    lines.push(`- [ ] \`${r.cmd}\` : ${r.mean.toFixed(0)}ms mean sur ${r.count} samples (total cumule ${r.total.toFixed(0)}ms) -> a investiguer.`);
  }
  lines.push("");
  lines.push("### P2 - Optimisations ciblees");
  lines.push("_A remplir manuellement apres lecture des sections 1-3._");
  lines.push("");
  lines.push("### P3 - A investiguer");
  lines.push("_A remplir manuellement._");
  lines.push("");

  lines.push("## Annexes");
  lines.push("");
  lines.push("### A. Toutes les commandes triees par mean desc");
  lines.push("");
  lines.push(renderTable([...backendAggregates].sort((a, b) => b.mean - a.mean)));

  return lines.join("\n");
}

// Entry point
if (import.meta.main) {
  const date = process.argv[2] ?? new Date().toISOString().slice(0, 10);
  const logsDir = join(process.env.LOCALAPPDATA ?? ".", "Wintyx", "Logs");
  const backendPath = join(logsDir, "perf-backend.csv");
  const frontendPath = join(logsDir, "perf-frontend.jsonl");

  const backendSamples = existsSync(backendPath) ? parseCsv(readFileSync(backendPath, "utf-8")) : [];
  const frontendSamples = existsSync(frontendPath) ? parseJsonl(readFileSync(frontendPath, "utf-8")) : [];

  if (backendSamples.length === 0 && frontendSamples.length === 0) {
    console.error(`Aucun sample trouve dans ${logsDir}. Lance d'abord la session avec build --features perf + VITE_PERF=1.`);
    process.exit(1);
  }

  const md = renderMarkdown({ backendSamples, frontendSamples, date });
  mkdirSync("audits", { recursive: true });
  const outPath = join("audits", `perf-baseline-${date}.md`);
  writeFileSync(outPath, md, "utf-8");
  console.log(`Rapport ecrit : ${outPath}`);
  console.log(`  Backend samples : ${backendSamples.length}`);
  console.log(`  Frontend samples : ${frontendSamples.length}`);
}
