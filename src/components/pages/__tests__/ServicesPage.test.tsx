import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { ToastProvider } from "../../../components/ui/Toast";
import { ConfirmProvider } from "../../../components/ui/ConfirmDialog";
import { ScoresProvider } from "../../../contexts/ScoresContext";
import { AdminProvider } from "../../../contexts/AdminContext";
import { ServicesPage } from "../ServicesPage";
import { invalidateCache } from "../../../hooks/use-invoke";
import type { ServiceItem } from "../../../lib/types";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

const FAKE_ITEMS: ServiceItem[] = [
  {
    name: "DiagTrack",
    displayName: "Connected User Experiences and Telemetry",
    description: "Service de telemetrie Microsoft",
    status: "Running",
    startType: "Automatic",
    risk: "high",
  },
  {
    name: "Spooler",
    displayName: "Print Spooler",
    description: "Gestionnaire d'impression",
    status: "Stopped",
    startType: "Manual",
    risk: "medium",
  },
  {
    name: "Themes",
    displayName: "Themes Service",
    description: "Gere les themes Windows",
    status: "Running",
    startType: "Automatic",
    risk: "low",
  },
];

function renderPage() {
  return render(
    <AdminProvider>
      <ToastProvider>
        <ConfirmProvider>
          <ScoresProvider>
            <ServicesPage />
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
    if (cmd === "get_services") return Promise.resolve(FAKE_ITEMS);
    if (cmd === "stop_service") return Promise.resolve(null);
    if (cmd === "start_service") return Promise.resolve(null);
    if (cmd === "get_admin_status") return Promise.resolve({ isAdmin: true, canElevate: true });
    if (cmd.endsWith("_score")) return Promise.resolve(75);
    return Promise.resolve(null);
  });
});

describe("ServicesPage", () => {
  it("renders services grouped by risk level", async () => {
    renderPage();

    await waitFor(() => {
      expect(screen.getByText("Connected User Experiences and Telemetry")).toBeInTheDocument();
    });
    expect(screen.getByText("Print Spooler")).toBeInTheDocument();
    expect(screen.getByText("Themes Service")).toBeInTheDocument();

    const headings = screen.getAllByRole("heading", { level: 2 });
    const headingTexts = headings.map((h) => h.textContent);
    expect(headingTexts.some((t) => t?.includes("Risque élevé"))).toBe(true);
    expect(headingTexts.some((t) => t?.includes("Risque moyen"))).toBe(true);
    expect(headingTexts.some((t) => t?.includes("Risque faible"))).toBe(true);
  });

  it("filters services via search input", async () => {
    renderPage();

    await waitFor(() => {
      expect(screen.getByText("Print Spooler")).toBeInTheDocument();
    });

    const searchInput = screen.getByPlaceholderText("Rechercher un service...");
    fireEvent.change(searchInput, { target: { value: "spooler" } });

    await waitFor(() => {
      expect(screen.queryByText("Connected User Experiences and Telemetry")).not.toBeInTheDocument();
    });
    expect(screen.getByText("Print Spooler")).toBeInTheDocument();
    expect(screen.queryByText("Themes Service")).not.toBeInTheDocument();
  });

  it("passes { name } (not { id }) to stop_service after confirmation", async () => {
    renderPage();

    await waitFor(() => {
      expect(screen.getByText("Connected User Experiences and Telemetry")).toBeInTheDocument();
    });

    // 2 services Running -> 2 boutons Arreter
    const stopButtons = screen.getAllByRole("button", { name: "Arrêter" });
    expect(stopButtons).toHaveLength(2);
    fireEvent.click(stopButtons[0]);

    await waitFor(() => {
      expect(screen.getByRole("dialog")).toBeInTheDocument();
    });
    const confirmButton = screen.getByRole("button", { name: "Confirmer" });
    fireEvent.click(confirmButton);

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith("stop_service", { name: "DiagTrack" });
    });
    // Verify it was not called with { id: ... }
    expect(mockedInvoke).not.toHaveBeenCalledWith("stop_service", { id: "DiagTrack" });
  });

  it("shows Demarrer button for stopped services", async () => {
    renderPage();

    await waitFor(() => {
      expect(screen.getByText("Print Spooler")).toBeInTheDocument();
    });

    const startButtons = screen.getAllByRole("button", { name: "Démarrer" });
    expect(startButtons).toHaveLength(1);
  });

  it("shows a non-installed badge and no action button for an absent service", async () => {
    const absent: ServiceItem = {
      name: "Fax",
      displayName: "Fax",
      description: "Legacy fax service",
      status: "Absent",
      startType: "--",
      risk: "Low",
    };
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_services") return Promise.resolve([absent]);
      if (cmd === "get_admin_status") return Promise.resolve({ isAdmin: true, canElevate: true });
      if (cmd.endsWith("_score")) return Promise.resolve(75);
      return Promise.resolve(null);
    });
    renderPage();

    await waitFor(() => {
      expect(screen.getByText("Non installé")).toBeInTheDocument();
    });
    expect(screen.queryByRole("button", { name: "Démarrer" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Arrêter" })).not.toBeInTheDocument();
  });
});

describe("ServicesPage refresh and unknown state", () => {
  it("reloads the list after an action, and again after Annuler", async () => {
    renderPage();
    await screen.findByText("Print Spooler");
    const count = () => mockedInvoke.mock.calls.filter(([c]) => c === "get_services").length;
    expect(count()).toBe(1);

    fireEvent.click(screen.getAllByRole("button", { name: "Arrêter" })[0]);
    fireEvent.click(await screen.findByRole("button", { name: "Confirmer" }));
    await waitFor(() => expect(count()).toBe(2));

    fireEvent.click(await screen.findByRole("button", { name: "Annuler" }));
    await waitFor(() => expect(count()).toBe(3));
  });

  it("shows Inconnu with a neutral badge and no action button when sc.exe failed", async () => {
    const unknown: ServiceItem = {
      name: "Wecsvc",
      displayName: "Windows Event Collector",
      description: "Collecte",
      status: "Unknown",
      startType: "--",
      risk: "Low",
    };
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_services") return Promise.resolve([unknown]);
      if (cmd === "get_admin_status") return Promise.resolve({ isAdmin: true, canElevate: true });
      if (cmd.endsWith("_score")) return Promise.resolve(75);
      return Promise.resolve(null);
    });
    renderPage();

    await screen.findByText("Windows Event Collector");
    expect(screen.getByText("Inconnu")).toBeInTheDocument();
    expect(screen.queryByText("Arrêté")).not.toBeInTheDocument();
    expect(screen.queryByText("--")).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Démarrer" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Arrêter" })).not.toBeInTheDocument();
  });
});
