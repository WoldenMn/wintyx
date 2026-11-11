import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { ToastProvider } from "../../../components/ui/Toast";
import { ConfirmProvider } from "../../../components/ui/ConfirmDialog";
import { ScoresProvider } from "../../../contexts/ScoresContext";
import { AdminProvider } from "../../../contexts/AdminContext";
import { DataProtectionPage } from "../DataProtectionPage";
import { invalidateCache } from "../../../hooks/use-invoke";
import type { DataProtectionItem } from "../../../lib/types";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

const FAKE_ITEMS: DataProtectionItem[] = [
  {
    id: "telemetry.diagtrack",
    name: "Telemetry DiagTrack",
    description: "Service de telemetrie",
    category: "Telemetrie",
    isProtected: false,
    recommended: true,
  },
  {
    id: "defender.cloud",
    name: "Defender Cloud",
    description: "Protection cloud",
    category: "Defender",
    isProtected: true,
    recommended: true,
  },
];

function renderPage() {
  return render(
    <AdminProvider>
      <ToastProvider>
        <ConfirmProvider>
          <ScoresProvider>
            <DataProtectionPage />
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
    if (cmd === "get_data_protection_items") return Promise.resolve(FAKE_ITEMS);
    if (cmd === "protect_item") return Promise.resolve(null);
    if (cmd === "get_admin_status") return Promise.resolve({ isAdmin: true, canElevate: true });
    if (cmd.endsWith("_score")) return Promise.resolve(75);
    return Promise.resolve(null);
  });
});

describe("DataProtectionPage", () => {
  it("passes item.id (not item.name) to protect_item after confirmation", async () => {
    renderPage();

    await waitFor(() => {
      expect(screen.getByText("Telemetry DiagTrack")).toBeInTheDocument();
    });

    const protectButton = screen.getByRole("button", { name: "Protéger" });
    fireEvent.click(protectButton);

    // useToggleAction ouvre une confirmation : il faut cliquer sur "Confirmer"
    await waitFor(() => {
      expect(screen.getByRole("dialog")).toBeInTheDocument();
    });
    const confirmButton = screen.getByRole("button", { name: "Confirmer" });
    fireEvent.click(confirmButton);

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith("protect_item", {
        id: "telemetry.diagtrack",
      });
    });
    expect(mockedInvoke).not.toHaveBeenCalledWith("protect_item", {
      id: "Telemetry DiagTrack",
    });
  });

  it("presente un reglage facultatif sans l'afficher comme non protege", async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_data_protection_items")
        return Promise.resolve([
          { id: "dp_19", name: "SmartScreen", description: "Facultatif", category: "Telemetrie", isProtected: false, recommended: false },
        ]);
      if (cmd === "get_admin_status") return Promise.resolve({ isAdmin: true, canElevate: true });
      return Promise.resolve(null);
    });
    renderPage();

    expect(await screen.findByText("Facultatif", { selector: "span" })).toBeInTheDocument();
    expect(screen.queryByText("Non protégé")).not.toBeInTheDocument();
  });

  it("hides Proteger button for already protected items", async () => {
    renderPage();

    await waitFor(() => {
      expect(screen.getByText("Defender Cloud")).toBeInTheDocument();
    });

    const protectButtons = screen.getAllByRole("button", { name: "Protéger" });
    expect(protectButtons).toHaveLength(1);
  });

  it("groups items by category with section headings", async () => {
    renderPage();

    await waitFor(() => {
      expect(screen.getByText("Telemetry DiagTrack")).toBeInTheDocument();
    });

    const headings = screen.getAllByRole("heading", { level: 2 });
    const headingTexts = headings.map((h) => h.textContent);
    // groupLabel mappe les categories backend vers le francais accentue
    // (Telemetrie -> Télémétrie) ; "Defender" reste en fallback (hors mapping).
    expect(headingTexts).toContain("Télémétrie");
    expect(headingTexts).toContain("Defender");
  });
});

describe("DataProtectionPage refresh", () => {
  it("reloads the list after a single action", async () => {
    renderPage();
    await screen.findByText("Telemetry DiagTrack");
    const count = () => mockedInvoke.mock.calls.filter(([c]) => c === "get_data_protection_items").length;
    expect(count()).toBe(1);

    fireEvent.click(screen.getAllByRole("button", { name: "Protéger" })[0]);
    fireEvent.click(await screen.findByRole("button", { name: "Confirmer" }));
    await waitFor(() => expect(count()).toBe(2));
  });
});
