import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { ToastProvider } from "../../ui/Toast";
import { ConfirmProvider } from "../../ui/ConfirmDialog";
import { ScoresProvider } from "../../../contexts/ScoresContext";
import { AdminProvider } from "../../../contexts/AdminContext";
import { DriversPage } from "../DriversPage";
import { invalidateCache } from "../../../hooks/use-invoke";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

const MOCK_SUMMARY = {
  total: 100,
  unsigned: 2,
  duplicates: 3,
  phantoms: 5,
  obsolete: 8,
  thirdParty: 12,
  problems: 1,
};

const MOCK_ISSUES = [
  {
    id: "dup-1",
    kind: { kind: "duplicate" },
    severity: "low",
    deviceName: "Realtek Audio",
    manufacturer: "Realtek",
    driverVersion: "12.0.1.0",
    driverDate: "2023-01-01",
    provider: "Realtek",
    infName: "oem1.inf",
    instanceId: null,
    reason: "Version dupliquee",
  },
  {
    id: "phantom-1",
    kind: { kind: "phantom" },
    severity: "medium",
    deviceName: "Old USB",
    manufacturer: "Generic",
    driverVersion: "",
    driverDate: "",
    provider: "Generic",
    infName: "",
    instanceId: "USB\\X\\1",
    reason: "Peripherique fantome",
  },
];

function renderPage() {
  return render(
    <AdminProvider>
      <ToastProvider>
        <ConfirmProvider>
          <ScoresProvider>
            <DriversPage />
          </ScoresProvider>
        </ConfirmProvider>
      </ToastProvider>
    </AdminProvider>,
  );
}

beforeEach(() => {
  vi.clearAllMocks();
  invalidateCache();
  mockedInvoke.mockImplementation((cmd: string) => {
    // get_driver_audit(_summary) renvoient Result<T> cote Rust -> invoke()
    // resout T directement (data deballee), pas un wrapper { status, data }.
    if (cmd === "get_driver_audit_summary")
      return Promise.resolve(MOCK_SUMMARY);
    if (cmd === "get_driver_audit")
      return Promise.resolve(MOCK_ISSUES);
    if (cmd === "get_admin_status")
      return Promise.resolve({ isAdmin: true, canElevate: true });
    if (cmd.endsWith("_score")) return Promise.resolve(75);
    return Promise.resolve(null);
  });
});

describe("DriversPage", () => {
  it("affiche les 6 StatCards avec valeurs du summary", async () => {
    renderPage();

    await waitFor(() => {
      expect(screen.getByText("100")).toBeInTheDocument();
    });

    expect(screen.getByText("Total")).toBeInTheDocument();
    expect(screen.getByText("2")).toBeInTheDocument();
    expect(screen.getByText("3")).toBeInTheDocument();
    expect(screen.getByText("5")).toBeInTheDocument();
    expect(screen.getByText("8")).toBeInTheDocument();
    expect(screen.getByText("12")).toBeInTheDocument();
  });

  it("filtre par doublons quand on clique sur le filtre Doublons", async () => {
    renderPage();

    await waitFor(() => {
      expect(screen.getByText("Realtek Audio")).toBeInTheDocument();
    });

    const doublesButton = screen.getByRole("button", { name: "Doublons" });
    fireEvent.click(doublesButton);

    expect(screen.getByText("Realtek Audio")).toBeInTheDocument();
    expect(screen.queryByText("Old USB")).not.toBeInTheDocument();
  });

  it("clic sur la StatCard Fantomes change le filtre", async () => {
    renderPage();

    await waitFor(() => {
      expect(screen.getByText("Old USB")).toBeInTheDocument();
    });

    const fantomesCard = screen.getByRole("button", {
      name: /Fantômes : 5/i,
    });
    fireEvent.click(fantomesCard);

    expect(screen.getByText("Old USB")).toBeInTheDocument();
    expect(screen.queryByText("Realtek Audio")).not.toBeInTheDocument();
  });

  it("affiche le bouton 'Nettoyer fantomes' avec le compteur quand phantoms > 0", async () => {
    renderPage();

    await waitFor(() => {
      expect(
        screen.getByRole("button", { name: /Nettoyer fantômes \(5\)/i }),
      ).toBeInTheDocument();
    });
  });

  it("affiche un empty state quand aucun probleme detecte", async () => {
    invalidateCache();
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_driver_audit_summary")
        return Promise.resolve({
          total: 0,
          unsigned: 0,
          duplicates: 0,
          phantoms: 0,
          obsolete: 0,
          thirdParty: 0,
          problems: 0,
        });
      if (cmd === "get_driver_audit") return Promise.resolve([]);
      if (cmd === "get_admin_status")
        return Promise.resolve({ isAdmin: true, canElevate: true });
      if (cmd.endsWith("_score")) return Promise.resolve(100);
      return Promise.resolve(null);
    });

    renderPage();

    await waitFor(() => {
      expect(
        screen.getByText("Aucun problème détecté"),
      ).toBeInTheDocument();
    });
  });
});

describe("DriversPage actions", () => {
  async function uninstallFirst() {
    renderPage();
    await screen.findByText("Realtek Audio");
    fireEvent.click(screen.getByRole("button", { name: /Désinstaller/i }));
    fireEvent.click(await screen.findByRole("button", { name: "Confirmer" }));
  }

  it("announces a failed uninstall as an error, not a success", async () => {
    const base = mockedInvoke.getMockImplementation()!;
    mockedInvoke.mockImplementation((cmd: string, args?: unknown) => {
      if (cmd === "uninstall_driver")
        return Promise.resolve({ total: 1, success: 0, failed: 1, errors: ["oem1.inf : pilote en cours d'utilisation"], dryRun: false, previewItems: [] });
      return base(cmd, args as never);
    });
    await uninstallFirst();

    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent("pilote en cours d'utilisation");
    expect(screen.queryByText("Pilote oem1.inf désinstallé")).not.toBeInTheDocument();
  });

  it("reloads the summary counters as well as the list after an action", async () => {
    const base = mockedInvoke.getMockImplementation()!;
    mockedInvoke.mockImplementation((cmd: string, args?: unknown) => {
      if (cmd === "uninstall_driver")
        return Promise.resolve({ total: 1, success: 1, failed: 0, errors: [], dryRun: false, previewItems: [] });
      return base(cmd, args as never);
    });
    await uninstallFirst();

    const count = (c: string) => mockedInvoke.mock.calls.filter(([x]) => x === c).length;
    await waitFor(() => expect(count("get_driver_audit")).toBe(2));
    expect(count("get_driver_audit_summary")).toBe(2);
  });
});
