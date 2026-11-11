import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { ToastProvider } from "../../../components/ui/Toast";
import { ConfirmProvider } from "../../../components/ui/ConfirmDialog";
import { ScoresProvider } from "../../../contexts/ScoresContext";
import { AdminProvider } from "../../../contexts/AdminContext";
import { BloatwarePage } from "../BloatwarePage";
import { invalidateCache } from "../../../hooks/use-invoke";
import type { BloatwareItem, BatchResult } from "../../../lib/types";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

const FAKE_ITEMS: BloatwareItem[] = [
  {
    packageName: "Microsoft.XboxApp",
    displayName: "Xbox",
    publisher: "Microsoft Corporation",
    category: "Games",
    isInstalled: true,
    version: "48.0.0",
  },
  {
    packageName: "Microsoft.SkypeApp",
    displayName: "Skype",
    publisher: "Microsoft Corporation",
    category: "Communication",
    isInstalled: true,
    version: "8.0.0",
  },
  {
    packageName: "Microsoft.GetHelp",
    displayName: "Get Help",
    publisher: "Microsoft Corporation",
    category: "System",
    isInstalled: false,
    version: "10.0.0",
  },
];

function renderPage() {
  return render(
    <AdminProvider>
      <ToastProvider>
        <ConfirmProvider>
          <ScoresProvider>
            <BloatwarePage />
          </ScoresProvider>
        </ConfirmProvider>
      </ToastProvider>
    </AdminProvider>,
  );
}

const EMPTY_BATCH_RESULT: BatchResult = {
  total: 0,
  success: 0,
  failed: 0,
  errors: [],
  dryRun: false,
  previewItems: [],
};

beforeEach(() => {
  vi.clearAllMocks();
  invalidateCache();
  mockedInvoke.mockImplementation((cmd: string) => {
    if (cmd === "get_bloatware_items") return Promise.resolve(FAKE_ITEMS);
    if (cmd === "remove_bloatware") return Promise.resolve(null);
    if (cmd === "remove_all_bloatware") return Promise.resolve(EMPTY_BATCH_RESULT);
    if (cmd === "create_pre_batch_snapshot") return Promise.resolve({
      timestamp: "2026-05-22T10:00:00Z",
      restorePointCreated: false,
      restorePointLimited: false,
      backupPath: null,
    });
    if (cmd === "get_admin_status") return Promise.resolve({ isAdmin: true, canElevate: true });
    if (cmd.endsWith("_score")) return Promise.resolve(75);
    return Promise.resolve(null);
  });
});

describe("BloatwarePage", () => {
  it("renders bloatware items", async () => {
    renderPage();

    await waitFor(() => {
      expect(screen.getByText("Xbox")).toBeInTheDocument();
    });
    expect(screen.getByText("Skype")).toBeInTheDocument();
    expect(screen.getByText("Get Help")).toBeInTheDocument();
  });

  it("passes { packageName } to remove_bloatware after confirmation", async () => {
    renderPage();

    await waitFor(() => {
      expect(screen.getByText("Xbox")).toBeInTheDocument();
    });

    // 2 items installed -> 2 Supprimer buttons
    const removeButtons = screen.getAllByRole("button", { name: "Supprimer" });
    expect(removeButtons).toHaveLength(2);
    fireEvent.click(removeButtons[0]);

    await waitFor(() => {
      expect(screen.getByRole("dialog")).toBeInTheDocument();
    });
    const confirmButton = screen.getByRole("button", { name: "Confirmer" });
    fireEvent.click(confirmButton);

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith("remove_bloatware", {
        packageName: "Microsoft.XboxApp",
      });
    });
    // Verify it was not called with { id: ... }
    expect(mockedInvoke).not.toHaveBeenCalledWith("remove_bloatware", { id: "Microsoft.XboxApp" });
  });

  it("shows Supprimé badge for uninstalled items (no remove button)", async () => {
    renderPage();

    await waitFor(() => {
      expect(screen.getByText("Get Help")).toBeInTheDocument();
    });

    // Get Help is not installed -> shows "Supprimé" badge instead of button
    expect(screen.getByText("Supprimé")).toBeInTheDocument();
    const removeButtons = screen.getAllByRole("button", { name: "Supprimer" });
    expect(removeButtons).toHaveLength(2);
  });

  it("renders 'Tout supprimer' batch button in header", async () => {
    renderPage();

    await waitFor(() => {
      expect(screen.getByText("Xbox")).toBeInTheDocument();
    });

    const batchButton = screen.getByRole("button", { name: "Tout supprimer" });
    expect(batchButton).toBeInTheDocument();
    // The 2 installed items mean the batch button should NOT be disabled
    expect(batchButton).not.toBeDisabled();
  });
});

describe("BloatwarePage refresh", () => {
  it("reloads the list after a single action", async () => {
    renderPage();
    await screen.findByText("Xbox");
    const count = () => mockedInvoke.mock.calls.filter(([c]) => c === "get_bloatware_items").length;
    expect(count()).toBe(1);

    fireEvent.click(screen.getAllByRole("button", { name: "Supprimer" })[0]);
    fireEvent.click(await screen.findByRole("button", { name: "Confirmer" }));
    await waitFor(() => expect(count()).toBe(2));
  });
});
