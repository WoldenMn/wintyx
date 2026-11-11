import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, act } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { ToastProvider } from "../../../components/ui/Toast";
import { AdminProvider } from "../../../contexts/AdminContext";
import { ThreatScanPage } from "../ThreatScanPage";
import type { ThreatScanResult } from "../../../lib/types";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async () => () => {}),
}));

const mockedInvoke = vi.mocked(invoke);

const RESULT: ThreatScanResult = {
  riskScore: 0,
  totalItemsScanned: 12,
  threats: [],
  categories: [
    { name: "Ports ouverts", scanned: 12, threatsFound: 0, riskLevel: "low", failed: false },
    { name: "Services non signés", scanned: 0, threatsFound: 0, riskLevel: "unknown", failed: true },
  ],
};

async function runScan() {
  render(
    <AdminProvider>
      <ToastProvider>
        <ThreatScanPage />
      </ToastProvider>
    </AdminProvider>,
  );
  await act(async () => {
    fireEvent.click(screen.getByRole("button", { name: "Lancer le scan" }));
  });
}

beforeEach(() => {
  vi.clearAllMocks();
  mockedInvoke.mockImplementation((cmd: string) => {
    if (cmd === "run_threat_scan") return Promise.resolve(RESULT);
    if (cmd === "get_admin_status") return Promise.resolve({ isAdmin: false, canElevate: true });
    return Promise.resolve(null);
  });
});

describe("ThreatScanPage categorie en echec", () => {
  it("affiche Non analysé au lieu de 0 menace pour une catégorie dont le scan a échoué", async () => {
    await runScan();

    expect(await screen.findByText("Non analysé")).toBeInTheDocument();
    expect(screen.getAllByText("0 menaces")).toHaveLength(1);
    expect(screen.getByRole("alert")).toHaveTextContent("Services non signés");
  });

  it("n'affiche pas d'indice de risque quand aucune catégorie n'a pu être analysée", async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === "run_threat_scan")
        return Promise.resolve({
          ...RESULT,
          totalItemsScanned: 0,
          categories: RESULT.categories.map((c) => ({ ...c, scanned: 0, failed: true })),
        });
      if (cmd === "get_admin_status") return Promise.resolve({ isAdmin: false, canElevate: true });
      return Promise.resolve(null);
    });
    await runScan();

    expect(await screen.findByText("Aucune catégorie n'a pu être analysée")).toBeInTheDocument();
    expect(screen.queryByText("/100")).not.toBeInTheDocument();
    expect(screen.queryByText(/0 menaces détectées/)).not.toBeInTheDocument();
  });

  it("n'affiche pas un indice partiel dans la couleur du succès", async () => {
    await runScan();

    const score = (await screen.findByText("/100")).parentElement;
    expect(score).not.toHaveClass("text-success");
  });

  it("n'affiche aucune alerte quand toutes les catégories ont été analysées", async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === "run_threat_scan") {
        return Promise.resolve({ ...RESULT, categories: [RESULT.categories[0]] });
      }
      if (cmd === "get_admin_status") return Promise.resolve({ isAdmin: false, canElevate: true });
      return Promise.resolve(null);
    });
    await runScan();

    await screen.findByText("Ports ouverts");
    expect(screen.queryByText("Non analysé")).not.toBeInTheDocument();
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });
});

describe("ThreatScanPage echec de scan", () => {
  it("garde le dernier resultat et affiche l'erreur quand un nouveau scan echoue", async () => {
    await runScan();
    await screen.findByText("Ports ouverts");

    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === "run_threat_scan") return Promise.reject({ code: "Generic", details: "PowerShell bloque" });
      if (cmd === "get_admin_status") return Promise.resolve({ isAdmin: false, canElevate: true });
      return Promise.resolve(null);
    });
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: "Relancer le scan" }));
    });

    expect(await screen.findByText("Ports ouverts")).toBeInTheDocument();
    expect(screen.getByTestId("scan-error")).toHaveTextContent("PowerShell bloque");
    expect(screen.getByRole("button", { name: "Relancer le scan" })).toBeInTheDocument();
  });

  it("revient a l'accueil avec l'erreur visible quand le premier scan echoue", async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === "run_threat_scan") return Promise.reject({ code: "Generic", details: "echec initial" });
      if (cmd === "get_admin_status") return Promise.resolve({ isAdmin: false, canElevate: true });
      return Promise.resolve(null);
    });
    await runScan();

    expect(screen.getByRole("button", { name: "Lancer le scan" })).toBeInTheDocument();
    expect(screen.getByTestId("scan-error")).toHaveTextContent("echec initial");
  });

  it("efface l'erreur quand un nouveau scan reussit", async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === "run_threat_scan") return Promise.reject({ code: "Generic", details: "echec initial" });
      if (cmd === "get_admin_status") return Promise.resolve({ isAdmin: false, canElevate: true });
      return Promise.resolve(null);
    });
    await runScan();
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === "run_threat_scan") return Promise.resolve(RESULT);
      if (cmd === "get_admin_status") return Promise.resolve({ isAdmin: false, canElevate: true });
      return Promise.resolve(null);
    });
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: "Lancer le scan" }));
    });

    await screen.findByText("Ports ouverts");
    expect(screen.queryByTestId("scan-error")).not.toBeInTheDocument();
  });
});
