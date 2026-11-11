import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, act } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { ToastProvider } from "../../../components/ui/Toast";
import { ConfirmProvider } from "../../../components/ui/ConfirmDialog";
import { ScoresProvider } from "../../../contexts/ScoresContext";
import { AdminProvider } from "../../../contexts/AdminContext";
import { DashboardPage } from "../DashboardPage";
import { invalidateCache } from "../../../hooks/use-invoke";
import type { BatchResult, SnapshotInfo, SystemInfoData } from "../../../lib/types";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

const SYSTEM: SystemInfoData = {
  osName: "Windows 11 Pro",
  cpuName: "Processeur de test",
  totalRamGb: 32,
  usedRamGb: 12,
  diskUsedPercent: 40,
  uptimeSeconds: 3600,
  computerName: "PC-TEST",
  antivirusName: "Microsoft Defender",
  cpuUsage: 10,
  gpuName: "Carte graphique de test",
  cpuCores: 12,
  cpuClockMhz: 4400,
};

const SNAPSHOT: SnapshotInfo = {
  restorePointCreated: false,
  restorePointLimited: false,
  registryBackups: { total: 7, success: 7, failed: 0, errors: [], dryRun: false, previewItems: [] },
  timestamp: "2026-09-13 10:00:00",
  actionName: "test",
};

const BATCH: BatchResult = { total: 2, success: 2, failed: 0, errors: [], dryRun: false, previewItems: [] };

beforeEach(() => {
  vi.clearAllMocks();
  invalidateCache();
  mockedInvoke.mockImplementation((cmd: string) => {
    if (cmd === "get_system_info") return Promise.resolve(SYSTEM);
    if (cmd === "get_score_history") return Promise.resolve([]);
    if (cmd === "get_defender_status") return Promise.resolve(null);
    if (cmd === "get_admin_status") return Promise.resolve({ isAdmin: true, canElevate: true });
    if (cmd === "get_all_scores") {
      return Promise.resolve({ dataProtection: 50, services: 50, hardening: 50, performance: 50, bloatware: 50 });
    }
    if (cmd === "create_pre_batch_snapshot") return Promise.resolve(SNAPSHOT);
    if (cmd === "protect_all" || cmd === "apply_hardening_essential") return Promise.resolve(BATCH);
    return Promise.resolve(null);
  });
});

function renderDashboard() {
  return render(
    <AdminProvider>
      <ToastProvider>
        <ConfirmProvider>
          <ScoresProvider>
            <DashboardPage />
          </ScoresProvider>
        </ConfirmProvider>
      </ToastProvider>
    </AdminProvider>,
  );
}

describe("DashboardPage analyse de démarrage", () => {
  it("remplace Passer par un message d'attente quand seule la collecte reste à finir", async () => {
    const base = mockedInvoke.getMockImplementation()!;
    mockedInvoke.mockImplementation((cmd: string, args?: unknown) => {
      if (cmd === "get_system_info") return new Promise(() => {});
      return base(cmd, args as never);
    });
    renderDashboard();

    await act(async () => {
      fireEvent.click(await screen.findByRole("button", { name: "Passer" }));
    });

    expect(screen.queryByRole("button", { name: "Passer" })).not.toBeInTheDocument();
    expect(screen.getByText("En attente des données système...")).toBeInTheDocument();
  });
});

describe("DashboardPage scores jamais lus", () => {
  it("annonce des scores indisponibles au lieu de 0 sur 100 quand la première lecture échoue", async () => {
    const base = mockedInvoke.getMockImplementation()!;
    mockedInvoke.mockImplementation((cmd: string, args?: unknown) => {
      if (cmd === "get_all_scores") return Promise.reject({ code: "Generic", details: "WMI injoignable" });
      return base(cmd, args as never);
    });
    renderDashboard();

    expect(await screen.findByText(/Scores indisponibles/, {}, { timeout: 4000 })).toBeInTheDocument();
    expect(screen.getByText(/WMI injoignable/)).toBeInTheDocument();
    expect(screen.queryByLabelText(/sur 100/)).not.toBeInTheDocument();
    expect(screen.queryByText(/peut-être obsolètes/)).not.toBeInTheDocument();
  });
});

describe("DashboardPage domaine illisible", () => {
  it("affiche -- pour un domaine non lu et le dit sous le score global", async () => {
    const base = mockedInvoke.getMockImplementation()!;
    mockedInvoke.mockImplementation((cmd: string, args?: unknown) => {
      if (cmd === "get_all_scores") {
        return Promise.resolve({ dataProtection: 50, services: 50, hardening: 50, performance: 50, bloatware: null });
      }
      return base(cmd, args as never);
    });
    renderDashboard();

    expect(await screen.findByText(/Calculé sans : Bloatware/, {}, { timeout: 4000 })).toBeInTheDocument();
    expect(screen.getByText("Lecture impossible")).toBeInTheDocument();
  });
});

describe("DashboardPage historique des scores", () => {
  it("affiche l'erreur de lecture de l'historique et non Collecte en cours", async () => {
    const base = mockedInvoke.getMockImplementation()!;
    mockedInvoke.mockImplementation((cmd: string, args?: unknown) => {
      if (cmd === "get_score_history") return Promise.reject({ code: "Generic", details: "historique corrompu" });
      return base(cmd, args as never);
    });
    renderDashboard();

    expect(await screen.findByText(/historique corrompu/, {}, { timeout: 4000 })).toBeInTheDocument();
    expect(screen.queryByText(/Collecte en cours/i)).not.toBeInTheDocument();
  });
});

describe("DashboardPage protect all", () => {
  it("takes a registry snapshot before applying protections and essential hardening", async () => {
    render(
      <AdminProvider>
        <ToastProvider>
          <ConfirmProvider>
            <ScoresProvider>
              <DashboardPage />
            </ScoresProvider>
          </ConfirmProvider>
        </ToastProvider>
      </AdminProvider>,
    );

    const protect = await screen.findByRole("button", { name: /protéger tout/i }, { timeout: 4000 });
    await act(async () => {
      fireEvent.click(protect);
    });
    const confirm = await screen.findByRole("button", { name: "Confirmer" });
    await act(async () => {
      fireEvent.click(confirm);
    });
    await screen.findByText("4 protections appliquées");

    const relevant = new Set(["create_pre_batch_snapshot", "protect_all", "apply_hardening_essential"]);
    const called = mockedInvoke.mock.calls
      .filter(([cmd]) => relevant.has(cmd))
      .map(([cmd, args]) => ((args as { options?: { dryRun?: boolean } } | undefined)?.options?.dryRun ? `${cmd} (aperçu)` : cmd));
    expect(called).toEqual([
      "protect_all (aperçu)",
      "apply_hardening_essential (aperçu)",
      "create_pre_batch_snapshot",
      "protect_all",
      "apply_hardening_essential",
    ]);
  });
});
