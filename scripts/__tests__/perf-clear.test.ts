import { describe, it, expect } from "vitest";
import { existsSync, writeFileSync, mkdtempSync, rmSync, mkdirSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { spawnSync } from "node:child_process";

const SUBPROCESS_TIMEOUT_MS = 30_000;

describe("perf-clear", () => {
  it("dry-run lists files without deleting (no --force)", () => {
    const tmpRoot = mkdtempSync(join(tmpdir(), "wintyx-perf-clear-test-"));
    try {
      const logsDir = join(tmpRoot, "Wintyx", "Logs");
      mkdirSync(logsDir, { recursive: true });
      writeFileSync(join(logsDir, "perf-backend.csv"), "fake");
      writeFileSync(join(logsDir, "perf-frontend.jsonl"), "fake");

      const r = spawnSync("bun", ["run", "scripts/perf-clear.ts"], {
        encoding: "utf-8",
        env: { ...process.env, LOCALAPPDATA: tmpRoot },
      });

      expect(r.status).toBe(0);
      expect(r.stdout).toContain("Vont etre supprimes");
      expect(existsSync(join(logsDir, "perf-backend.csv"))).toBe(true);
      expect(existsSync(join(logsDir, "perf-frontend.jsonl"))).toBe(true);
    } finally {
      rmSync(tmpRoot, { recursive: true, force: true });
    }
  }, SUBPROCESS_TIMEOUT_MS);
});
