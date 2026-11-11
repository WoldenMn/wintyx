import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { ToastProvider } from "../../../components/ui/Toast";
import { ConfirmProvider } from "../../../components/ui/ConfirmDialog";
import { ScoresProvider } from "../../../contexts/ScoresContext";
import { AdminProvider } from "../../../contexts/AdminContext";
import { StartupPage } from "../StartupPage";
import { invalidateCache } from "../../../hooks/use-invoke";
import type { StartupItem } from "../../../lib/types";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

const FAKE_ITEMS: StartupItem[] = [
  {
    name: "OneDrive",
    command: "C:\\Users\\Test\\AppData\\Local\\Microsoft\\OneDrive\\OneDrive.exe",
    location: "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
    locationDisplay: "Registry (HKCU)",
    isEnabled: true,
  },
  {
    name: "Teams",
    command: "C:\\Users\\Test\\AppData\\Local\\Microsoft\\Teams\\Teams.exe",
    location: "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
    locationDisplay: "Registry (HKCU)",
    isEnabled: false,
  },
];

function renderPage() {
  return render(
    <AdminProvider>
      <ToastProvider>
        <ConfirmProvider>
          <ScoresProvider>
            <StartupPage />
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
    if (cmd === "get_startup_items") return Promise.resolve(FAKE_ITEMS);
    if (cmd === "disable_startup_item") return Promise.resolve(null);
    if (cmd === "enable_startup_item") return Promise.resolve(null);
    if (cmd === "get_admin_status") return Promise.resolve({ isAdmin: true, canElevate: true });
    if (cmd.endsWith("_score")) return Promise.resolve(75);
    return Promise.resolve(null);
  });
});

describe("StartupPage", () => {
  it("renders startup items", async () => {
    renderPage();

    await waitFor(() => {
      expect(screen.getByText("OneDrive")).toBeInTheDocument();
    });
    expect(screen.getByText("Teams")).toBeInTheDocument();
  });

  it("passes { name, location } to disable_startup_item after confirmation", async () => {
    renderPage();

    await waitFor(() => {
      expect(screen.getByText("OneDrive")).toBeInTheDocument();
    });

    // OneDrive is enabled -> shows "Désactiver"
    const disableButton = screen.getByRole("button", { name: "Désactiver" });
    fireEvent.click(disableButton);

    await waitFor(() => {
      expect(screen.getByRole("dialog")).toBeInTheDocument();
    });
    const confirmButton = screen.getByRole("button", { name: "Confirmer" });
    fireEvent.click(confirmButton);

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith("disable_startup_item", {
        name: "OneDrive",
        location: "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
      });
    });
    // Verify it was not called with only { name } or { id }
    expect(mockedInvoke).not.toHaveBeenCalledWith("disable_startup_item", { name: "OneDrive" });
    expect(mockedInvoke).not.toHaveBeenCalledWith("disable_startup_item", { id: "OneDrive" });
  });

  it("shows Reactiver button for disabled items", async () => {
    renderPage();

    await waitFor(() => {
      expect(screen.getByText("Teams")).toBeInTheDocument();
    });

    const reactivateButtons = screen.getAllByRole("button", { name: "Réactiver" });
    expect(reactivateButtons).toHaveLength(1);
  });

  it("passes { name, location } to enable_startup_item after confirmation", async () => {
    renderPage();

    await waitFor(() => {
      expect(screen.getByText("Teams")).toBeInTheDocument();
    });

    const enableButton = screen.getByRole("button", { name: "Réactiver" });
    fireEvent.click(enableButton);

    await waitFor(() => {
      expect(screen.getByRole("dialog")).toBeInTheDocument();
    });
    const confirmButton = screen.getByRole("button", { name: "Confirmer" });
    fireEvent.click(confirmButton);

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith("enable_startup_item", {
        name: "Teams",
        location: "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
      });
    });
  });
});

describe("StartupPage refresh", () => {
  it("reloads the list after a single action", async () => {
    renderPage();
    await screen.findByText("OneDrive");
    const count = () => mockedInvoke.mock.calls.filter(([c]) => c === "get_startup_items").length;
    expect(count()).toBe(1);

    fireEvent.click(screen.getAllByRole("button", { name: "Désactiver" })[0]);
    fireEvent.click(await screen.findByRole("button", { name: "Confirmer" }));
    await waitFor(() => expect(count()).toBe(2));
  });
});
