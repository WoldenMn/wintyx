import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { ToastProvider } from "../../../components/ui/Toast";
import { ConfirmProvider } from "../../../components/ui/ConfirmDialog";
import { ScoresProvider } from "../../../contexts/ScoresContext";
import { AdminProvider } from "../../../contexts/AdminContext";
import { CleanupPage } from "../CleanupPage";
import { invalidateCache } from "../../../hooks/use-invoke";
import type { CleanupTarget, CleanResult } from "../../../lib/types";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

const FAKE_ITEMS: CleanupTarget[] = [
  {
    id: "temp",
    name: "Dossier Temp",
    description: "Fichiers temporaires utilisateur",
    path: "C:\\Users\\Test\\AppData\\Local\\Temp",
    sizeBytes: 1_500_000_000,
    sizeDisplay: "1.5 GB",
    isSystem: false,
    accessible: true,
  },
  {
    id: "prefetch",
    name: "Prefetch",
    description: "Cache de prefetch Windows",
    path: "C:\\Windows\\Prefetch",
    sizeBytes: 250_000_000,
    sizeDisplay: "250 MB",
    isSystem: true,
    accessible: true,
  },
];

const EMPTY_CLEAN_RESULT: CleanResult = {
  freedBytes: 1_750_000_000,
  errors: [],
  dryRun: false,
  previewItems: [],
};

function renderPage() {
  return render(
    <AdminProvider>
      <ToastProvider>
        <ConfirmProvider>
          <ScoresProvider>
            <CleanupPage />
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
    if (cmd === "get_cleanup_targets") return Promise.resolve(FAKE_ITEMS);
    if (cmd === "clean_target") return Promise.resolve(null);
    if (cmd === "clean_all") return Promise.resolve(EMPTY_CLEAN_RESULT);
    if (cmd === "get_admin_status") return Promise.resolve({ isAdmin: true, canElevate: true });
    if (cmd.endsWith("_score")) return Promise.resolve(75);
    return Promise.resolve(null);
  });
});

describe("CleanupPage", () => {
  it("renders cleanup targets with computed subtitle", async () => {
    renderPage();

    await waitFor(() => {
      expect(screen.getByText("Dossier Temp")).toBeInTheDocument();
    });
    expect(screen.getByText("Prefetch")).toBeInTheDocument();
    // subtitleCompute: "N cibles détectées — X récupérables"
    expect(screen.getByText(/2 cibles détectées/i)).toBeInTheDocument();
  });

  it("passes { id } to clean_target after confirmation", async () => {
    renderPage();

    await waitFor(() => {
      expect(screen.getByText("Dossier Temp")).toBeInTheDocument();
    });

    const cleanButtons = screen.getAllByRole("button", { name: "Nettoyer" });
    expect(cleanButtons).toHaveLength(2);
    fireEvent.click(cleanButtons[0]);

    await waitFor(() => {
      expect(screen.getByRole("dialog")).toBeInTheDocument();
    });
    const confirmButton = screen.getByRole("button", { name: "Confirmer" });
    fireEvent.click(confirmButton);

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith("clean_target", { id: "temp" });
    });
  });

  it("renders 'Tout nettoyer' batch button and invokes clean_all on confirm", async () => {
    renderPage();

    await waitFor(() => {
      expect(screen.getByText("Dossier Temp")).toBeInTheDocument();
    });

    const batchButton = screen.getByRole("button", { name: "Tout nettoyer" });
    expect(batchButton).toBeInTheDocument();
    expect(batchButton).not.toBeDisabled();

    fireEvent.click(batchButton);

    await waitFor(() => {
      expect(screen.getByRole("dialog")).toBeInTheDocument();
    });
    const confirmButton = screen.getByRole("button", { name: "Confirmer" });
    fireEvent.click(confirmButton);

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith("clean_all");
    });
  });

  it("displays size badge for each target", async () => {
    renderPage();

    await waitFor(() => {
      expect(screen.getByText("Dossier Temp")).toBeInTheDocument();
    });

    expect(screen.getByText("1.5 GB")).toBeInTheDocument();
    expect(screen.getByText("250 MB")).toBeInTheDocument();
  });

  it("shows the admin label and disables cleaning for an inaccessible target", async () => {
    const blocked: CleanupTarget = {
      id: "wintemp",
      name: "Temp Windows",
      description: "Fichiers temporaires systeme",
      path: "C:/Windows/Temp",
      sizeBytes: 0,
      sizeDisplay: "Inaccessible sans droits administrateur",
      isSystem: true,
      accessible: false,
    };
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_cleanup_targets") return Promise.resolve([...FAKE_ITEMS, blocked]);
      if (cmd === "get_admin_status") return Promise.resolve({ isAdmin: true, canElevate: true });
      if (cmd.endsWith("_score")) return Promise.resolve(75);
      return Promise.resolve(null);
    });
    renderPage();

    await waitFor(() => {
      expect(screen.getByText("Temp Windows")).toBeInTheDocument();
    });
    expect(screen.getByText("Inaccessible sans droits administrateur")).toBeInTheDocument();
    expect(screen.queryByText("0 B")).not.toBeInTheDocument();
    expect(screen.getByText(/1 inaccessible/i)).toBeInTheDocument();
    const cleanButtons = screen.getAllByRole("button", { name: "Nettoyer" });
    expect(cleanButtons[2]).toBeDisabled();
    expect(cleanButtons[0]).not.toBeDisabled();
  });
});
