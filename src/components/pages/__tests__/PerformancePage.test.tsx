import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { ToastProvider } from "../../../components/ui/Toast";
import { ConfirmProvider } from "../../../components/ui/ConfirmDialog";
import { ScoresProvider } from "../../../contexts/ScoresContext";
import { AdminProvider } from "../../../contexts/AdminContext";
import { PerformancePage } from "../PerformancePage";
import { invalidateCache } from "../../../hooks/use-invoke";
import type { PerformanceItem, BatchResult } from "../../../lib/types";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

const FAKE_ITEMS: PerformanceItem[] = [
  {
    id: "disable_prefetch",
    name: "Desactiver Prefetch",
    description: "Desactive le service Prefetch (utile sur SSD)",
    category: "SSD",
    isOptimized: false,
  },
  {
    id: "enable_game_mode",
    name: "Activer Game Mode",
    description: "Active le mode jeu de Windows",
    category: "Gaming",
    isOptimized: true,
  },
  {
    id: "disable_visual_effects",
    name: "Reduire les effets visuels",
    description: "Desactive les animations Windows",
    category: "UI",
    isOptimized: false,
  },
];

function renderPage() {
  return render(
    <AdminProvider>
      <ToastProvider>
        <ConfirmProvider>
          <ScoresProvider>
            <PerformancePage />
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
    if (cmd === "get_performance_items") return Promise.resolve(FAKE_ITEMS);
    if (cmd === "optimize_item") return Promise.resolve(null);
    if (cmd === "optimize_all") return Promise.resolve(EMPTY_BATCH_RESULT);
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

describe("PerformancePage", () => {
  it("renders performance items", async () => {
    renderPage();

    await waitFor(() => {
      expect(screen.getByText("Desactiver Prefetch")).toBeInTheDocument();
    });
    expect(screen.getByText("Activer Game Mode")).toBeInTheDocument();
    expect(screen.getByText("Reduire les effets visuels")).toBeInTheDocument();
  });

  it("passes { id } to optimize_item after confirmation", async () => {
    renderPage();

    await waitFor(() => {
      expect(screen.getByText("Desactiver Prefetch")).toBeInTheDocument();
    });

    // 2 items not optimized -> 2 Optimiser buttons
    const optimizeButtons = screen.getAllByRole("button", { name: "Optimiser" });
    expect(optimizeButtons).toHaveLength(2);
    fireEvent.click(optimizeButtons[0]);

    await waitFor(() => {
      expect(screen.getByRole("dialog")).toBeInTheDocument();
    });
    const confirmButton = screen.getByRole("button", { name: "Confirmer" });
    fireEvent.click(confirmButton);

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith("optimize_item", {
        id: "disable_prefetch",
      });
    });
  });

  it("hides Optimiser button for already optimized items", async () => {
    renderPage();

    await waitFor(() => {
      expect(screen.getByText("Activer Game Mode")).toBeInTheDocument();
    });

    // 1 item optimized -> only 2 Optimiser buttons (the unoptimized ones)
    const optimizeButtons = screen.getAllByRole("button", { name: "Optimiser" });
    expect(optimizeButtons).toHaveLength(2);
    // The optimized item has its "Optimisé" badge
    expect(screen.getByText("Optimisé")).toBeInTheDocument();
  });

  it("renders 'Tout optimiser' batch button in header", async () => {
    renderPage();

    await waitFor(() => {
      expect(screen.getByText("Desactiver Prefetch")).toBeInTheDocument();
    });

    const batchButton = screen.getByRole("button", { name: "Tout optimiser" });
    expect(batchButton).toBeInTheDocument();
  });
});

describe("PerformancePage refresh", () => {
  it("reloads the list after a single action", async () => {
    renderPage();
    await screen.findByText("Desactiver Prefetch");
    const count = () => mockedInvoke.mock.calls.filter(([c]) => c === "get_performance_items").length;
    expect(count()).toBe(1);

    fireEvent.click(screen.getAllByRole("button", { name: "Optimiser" })[0]);
    fireEvent.click(await screen.findByRole("button", { name: "Confirmer" }));
    await waitFor(() => expect(count()).toBe(2));
  });
});
