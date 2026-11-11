import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { AdminProvider } from "../../../contexts/AdminContext";
import { RestorePointsSection } from "../RestorePointsSection";
import { invalidateCache } from "../../../hooks/use-invoke";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

function mockBackend(isAdmin: boolean, points: unknown) {
  mockedInvoke.mockImplementation((cmd: string) => {
    if (cmd === "get_admin_status") return Promise.resolve({ isAdmin, canElevate: true });
    if (cmd === "get_restore_points") return points instanceof Error ? Promise.reject({ code: "ProcessFailed", details: { command: "x", detail: "Accès refusé" } }) : Promise.resolve(points);
    return Promise.resolve(null);
  });
}

function renderSection(refreshKey = 0) {
  return render(
    <AdminProvider>
      <RestorePointsSection refreshKey={refreshKey} />
    </AdminProvider>,
  );
}

beforeEach(() => {
  vi.clearAllMocks();
  invalidateCache();
});

describe("RestorePointsSection", () => {
  it("demande les droits administrateur au lieu de lister sans y avoir accès", async () => {
    mockBackend(false, []);
    renderSection();

    expect(await screen.findByText(/Droits administrateur requis/)).toBeInTheDocument();
    expect(mockedInvoke.mock.calls.some(([cmd]) => cmd === "get_restore_points")).toBe(false);
  });

  it("liste les points de restauration en administrateur", async () => {
    mockBackend(true, [
      { description: "Wintyx - Point manuel", creationTime: "2026-10-01 09:00:00", sequenceNumber: 12, restoreType: "MODIFY_SETTINGS" },
      { description: "Installation d'un pilote", creationTime: "2026-09-20 18:30:00", sequenceNumber: 11, restoreType: "DEVICE_DRIVER_INSTALL" },
    ]);
    renderSection();

    expect(await screen.findByText("Wintyx - Point manuel")).toBeInTheDocument();
    expect(screen.getByText("Installation d'un pilote")).toBeInTheDocument();
    expect(screen.getByText("2026-10-01 09:00:00")).toBeInTheDocument();
  });

  it("annonce l'absence de point de restauration", async () => {
    mockBackend(true, []);
    renderSection();

    expect(await screen.findByText("Aucun point de restauration")).toBeInTheDocument();
  });

  it("affiche l'erreur avec un bouton Réessayer", async () => {
    mockBackend(true, new Error("refus"));
    renderSection();

    expect(await screen.findByRole("button", { name: /réessayer/i })).toBeInTheDocument();
  });
});
