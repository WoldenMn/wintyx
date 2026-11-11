import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent, act } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { ToastProvider } from "../../../components/ui/Toast";
import { ConfirmProvider } from "../../../components/ui/ConfirmDialog";
import { BenchmarkPage } from "../BenchmarkPage";
import type { BatchResult, BenchmarkResult, SnapshotInfo } from "../../../lib/types";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

const RESULT: BenchmarkResult = {
  cpuScore: 120,
  memoryScore: 9000,
  diskReadScore: 1500,
  diskWriteScore: 1200,
  networkLatency: 12,
  overallScore: 71,
  durationMs: 30000,
};

const SNAPSHOT: SnapshotInfo = {
  restorePointCreated: false,
  restorePointLimited: false,
  registryBackups: { total: 7, success: 7, failed: 0, errors: [], dryRun: false, previewItems: [] },
  timestamp: "2026-09-13 10:00:00",
  actionName: "test",
};

function mockBackend(optimizeResult: BatchResult) {
  mockedInvoke.mockImplementation((cmd: string) => {
    if (cmd === "get_benchmark_history") return Promise.resolve([]);
    if (cmd === "run_benchmark") return Promise.resolve(RESULT);
    if (cmd === "optimize_all") return Promise.resolve(optimizeResult);
    if (cmd === "create_pre_batch_snapshot") return Promise.resolve(SNAPSHOT);
    return Promise.resolve(null);
  });
}

function renderPage() {
  return render(
    <ToastProvider>
      <ConfirmProvider>
        <BenchmarkPage />
      </ConfirmProvider>
    </ToastProvider>,
  );
}

async function runThenOptimize() {
  await act(async () => {
    fireEvent.click(screen.getByRole("button", { name: "Lancer le benchmark" }));
  });
  const optimize = await screen.findByRole("button", { name: /optimiser & re-tester/i });
  await act(async () => {
    fireEvent.click(optimize);
  });
  const confirm = await screen.findByRole("button", { name: "Confirmer" });
  await act(async () => {
    fireEvent.click(confirm);
  });
  await screen.findByText("Comparaison Avant / Après");
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe("BenchmarkPage history", () => {
  it("shows an out-of-bounds legacy entry as invalid instead of the raw figure", async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_benchmark_history") {
        return Promise.resolve([
          { timestamp: "2026-05-08 17:53:26", result: { ...RESULT, memoryScore: 100000000 }, plausible: false },
          { timestamp: "2026-06-19 12:44:30", result: RESULT, plausible: true, version: "2.18.0" },
        ]);
      }
      return Promise.resolve(null);
    });
    renderPage();

    expect(await screen.findByText("Mesure invalide (ancienne version)")).toBeTruthy();
    expect(screen.queryByText(/100.?000.?000/)).toBeNull();
    expect(screen.getByText(/^9.000$/)).toBeTruthy();
  });

  it("treats an entry without the plausible flag as valid, like the Rust default", async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_benchmark_history") {
        return Promise.resolve([{ timestamp: "2026-06-19 12:44:30", result: RESULT, version: "2.18.0" }]);
      }
      return Promise.resolve(null);
    });
    renderPage();

    expect(await screen.findByText(/^9.000$/)).toBeTruthy();
    expect(screen.queryByText("Mesure invalide (ancienne version)")).toBeNull();
  });
});

describe("BenchmarkPage version et estimation", () => {
  it("signale une entrée sans version ou antérieure à 2.18.0, pas une entrée récente", async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_benchmark_history") {
        return Promise.resolve([
          { timestamp: "2026-10-03 09:00:00", result: RESULT, plausible: true, version: "2.18.0" },
          { timestamp: "2026-09-01 09:00:00", result: RESULT, plausible: true, version: "2.17.1" },
          { timestamp: "2026-06-19 12:44:30", result: RESULT, plausible: true },
        ]);
      }
      return Promise.resolve(null);
    });
    renderPage();

    expect(await screen.findAllByText("Mesure d'une version précédente, non comparable")).toHaveLength(2);
  });

  it("annonce la durée de la dernière mesure et garde l'historique sous le résultat", async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_benchmark_history") {
        return Promise.resolve([
          { timestamp: "2026-10-03 09:00:00", result: { ...RESULT, durationMs: 4500 }, plausible: true, version: "2.18.0" },
        ]);
      }
      if (cmd === "run_benchmark") return new Promise(() => {});
      return Promise.resolve(null);
    });
    renderPage();

    await screen.findByText("Historique récent");
    fireEvent.click(screen.getByRole("button", { name: "Lancer le benchmark" }));
    expect(await screen.findByText("Estimation : environ 5 secondes")).toBeTruthy();
  });

  it("garde l'historique visible quand le résultat est à l'écran", async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_benchmark_history") {
        return Promise.resolve([{ timestamp: "2026-10-03 09:00:00", result: RESULT, plausible: true, version: "2.18.0" }]);
      }
      if (cmd === "run_benchmark") return Promise.resolve(RESULT);
      return Promise.resolve(null);
    });
    renderPage();

    await screen.findByText("Historique récent");
    fireEvent.click(screen.getByRole("button", { name: "Lancer le benchmark" }));
    await screen.findByText("Score global");
    expect(screen.getByText("Historique récent")).toBeTruthy();
  });
});

describe("BenchmarkPage optimize and retest", () => {
  it("applies only performance optimizations, after a snapshot, then re-runs the benchmark", async () => {
    mockBackend({ total: 3, success: 3, failed: 0, errors: [], dryRun: false, previewItems: [] });
    renderPage();

    await runThenOptimize();

    const calls = mockedInvoke.mock.calls.map(([cmd]) => cmd).filter((cmd) => cmd !== "get_benchmark_history");
    expect(calls).toEqual(["run_benchmark", "create_pre_batch_snapshot", "optimize_all", "run_benchmark"]);
  });

  it("ne relance pas le benchmark quand aucune optimisation n'a été appliquée", async () => {
    mockBackend({ total: 3, success: 0, failed: 3, errors: ["a", "b", "c"], dryRun: false, previewItems: [] });
    renderPage();

    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: "Lancer le benchmark" }));
    });
    await act(async () => {
      fireEvent.click(await screen.findByRole("button", { name: /optimiser & re-tester/i }));
    });
    await act(async () => {
      fireEvent.click(await screen.findByRole("button", { name: "Confirmer" }));
    });

    await waitFor(() => expect(mockedInvoke.mock.calls.some(([cmd]) => cmd === "optimize_all")).toBe(true));
    expect(mockedInvoke.mock.calls.filter(([cmd]) => cmd === "run_benchmark")).toHaveLength(1);
    expect(screen.queryByText("Comparaison Avant / Après")).not.toBeInTheDocument();
  });

  it("signale dans la comparaison que des optimisations ont échoué", async () => {
    mockBackend({ total: 3, success: 1, failed: 2, errors: ["a", "b"], dryRun: false, previewItems: [] });
    renderPage();

    await runThenOptimize();

    expect(screen.getByText(/Optimisations appliquées : 1 sur 3/)).toBeInTheDocument();
  });

  it("n'affiche pas un écart nul comme une régression", async () => {
    mockBackend({ total: 3, success: 3, failed: 0, errors: [], dryRun: false, previewItems: [] });
    renderPage();

    await runThenOptimize();

    for (const badge of screen.getAllByText("0.0%")) {
      expect(badge.className).not.toMatch(/danger/);
    }
  });

  it("reports failed optimizations instead of announcing success", async () => {
    mockBackend({ total: 3, success: 1, failed: 2, errors: ["a", "b"], dryRun: false, previewItems: [] });
    renderPage();

    await runThenOptimize();

    await waitFor(() => {
      const toasts = screen.queryAllByRole("status");
      expect(toasts.some((t) => /2 en échec/.test(t.textContent ?? ""))).toBe(true);
    });
  });
});
