#!/usr/bin/env bun
// Usage : bun run scripts/perf-clear.ts [--force]
// Supprime les fichiers perf-{backend.csv,frontend.jsonl} dans %LOCALAPPDATA%/Wintyx/Logs/.

import { existsSync, unlinkSync, readdirSync } from "node:fs";
import { join } from "node:path";

const logsDir = join(process.env.LOCALAPPDATA ?? ".", "Wintyx", "Logs");
const targets = [
  join(logsDir, "perf-backend.csv"),
  join(logsDir, "perf-frontend.jsonl"),
];

// Include any .bak rotations as well
if (existsSync(logsDir)) {
  for (const f of readdirSync(logsDir)) {
    if ((f.startsWith("perf-backend.") || f.startsWith("perf-frontend.")) && f.endsWith(".bak")) {
      targets.push(join(logsDir, f));
    }
  }
}

const existing = targets.filter((p) => existsSync(p));

if (existing.length === 0) {
  console.log(`Aucun fichier perf a supprimer dans ${logsDir}.`);
  process.exit(0);
}

const force = process.argv.includes("--force");
if (!force) {
  console.log(`Vont etre supprimes (${existing.length} fichiers) :`);
  for (const p of existing) console.log(`  ${p}`);
  console.log(`\nRelance avec --force pour confirmer.`);
  process.exit(0);
}

for (const p of existing) {
  try {
    unlinkSync(p);
    console.log(`Supprime : ${p}`);
  } catch (e) {
    console.error(`Echec ${p} : ${e}`);
  }
}
