import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent, act } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { ToastProvider } from "../../../components/ui/Toast";
import { ConfirmProvider } from "../../../components/ui/ConfirmDialog";
import { ScoresProvider } from "../../../contexts/ScoresContext";
import { AdminProvider } from "../../../contexts/AdminContext";
import { HardeningPage } from "../HardeningPage";
import { invalidateCache } from "../../../hooks/use-invoke";
import type { BatchResult, HardeningItem, SnapshotInfo } from "../../../lib/types";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

const ITEMS: HardeningItem[] = [
  {
    id: "SMBv1",
    name: "Désactiver SMBv1",
    description: "Protocole obsolète",
    category: "network",
    riskLevel: "critical",
    isSecure: false,
    isUnknown: false,
    isFixable: true,
  },
  {
    id: "RDP",
    name: "Désactiver le bureau à distance",
    description: "Réduit la surface d'attaque",
    category: "network",
    riskLevel: "high",
    isSecure: false,
    isUnknown: false,
    isFixable: true,
  },
];

const BATCH: BatchResult = { total: 2, success: 2, failed: 0, errors: [], dryRun: false, previewItems: [] };

const SNAPSHOT: SnapshotInfo = {
  restorePointCreated: false,
  restorePointLimited: false,
  registryBackups: { total: 7, success: 7, failed: 0, errors: [], dryRun: false, previewItems: [] },
  timestamp: "2026-09-13 10:00:00",
  actionName: "test",
};

function renderPage() {
  return render(
    <AdminProvider>
      <ToastProvider>
        <ConfirmProvider>
          <ScoresProvider>
            <HardeningPage />
          </ScoresProvider>
        </ConfirmProvider>
      </ToastProvider>
    </AdminProvider>,
  );
}

function batchCommandsCalled(): string[] {
  const relevant = new Set(["create_pre_batch_snapshot", "apply_hardening_all", "apply_hardening_essential"]);
  return mockedInvoke.mock.calls
    .filter(([cmd]) => relevant.has(cmd))
    .map(([cmd, args]) => ((args as { options?: { dryRun?: boolean } } | undefined)?.options?.dryRun ? `${cmd} (aperçu)` : cmd));
}

beforeEach(() => {
  vi.clearAllMocks();
  invalidateCache();
  mockedInvoke.mockImplementation((cmd: string) => {
    if (cmd === "get_hardening_checks") return Promise.resolve(ITEMS);
    if (cmd === "apply_hardening_all" || cmd === "apply_hardening_essential") return Promise.resolve(BATCH);
    if (cmd === "create_pre_batch_snapshot") return Promise.resolve(SNAPSHOT);
    if (cmd === "get_admin_status") return Promise.resolve({ isAdmin: true, canElevate: true });
    if (cmd === "get_all_scores") {
      return Promise.resolve({ dataProtection: 50, services: 50, hardening: 50, performance: 50, bloatware: 50 });
    }
    return Promise.resolve(null);
  });
});

async function clickAndConfirm(buttonName: string) {
  await waitFor(() => {
    expect(screen.getByText("Désactiver SMBv1")).toBeInTheDocument();
  });
  await act(async () => {
    fireEvent.click(screen.getByRole("button", { name: buttonName }));
  });
  const confirmButton = await screen.findByRole("button", { name: "Confirmer" });
  await act(async () => {
    fireEvent.click(confirmButton);
  });
  await screen.findByText("2 corrections appliquées");
}

describe("HardeningPage unknown state", () => {
  it("shows Inconnu without a green badge or fix button when a check cannot be read", async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_hardening_checks") {
        return Promise.resolve([
          { id: "AccountLockout", name: "Verrouillage de compte", description: "Seuil", category: "Accounts", riskLevel: "Recommended", isSecure: false, isUnknown: true, isFixable: true },
        ]);
      }
      if (cmd === "get_admin_status") return Promise.resolve({ isAdmin: false, canElevate: true });
      return Promise.resolve(null);
    });
    renderPage();

    await screen.findByText("Verrouillage de compte");
    expect(screen.getByText("Inconnu")).toBeInTheDocument();
    expect(screen.queryByText("Appliqué")).not.toBeInTheDocument();
    expect(screen.queryByText("Non appliqué")).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Corriger" })).not.toBeInTheDocument();
  });
});

describe("HardeningPage batch actions", () => {
  it("takes a registry snapshot before applying every correction", async () => {
    renderPage();

    await clickAndConfirm("Appliquer tout");

    await waitFor(() => {
      expect(batchCommandsCalled()).toEqual(["create_pre_batch_snapshot", "apply_hardening_all"]);
    });
  });

  it("takes a registry snapshot before applying the essential corrections", async () => {
    renderPage();

    await clickAndConfirm("Appliquer essentiels");

    await waitFor(() => {
      expect(batchCommandsCalled()).toEqual([
        "apply_hardening_essential (aperçu)",
        "create_pre_batch_snapshot",
        "apply_hardening_essential",
      ]);
    });
  });
});

describe("HardeningPage manual and single fixes", () => {
  const CREDENTIAL_GUARD: HardeningItem = {
    id: "CredentialGuard",
    name: "Credential Guard",
    description: "Isole les identifiants. Configuration manuelle : stratégie de groupe",
    category: "Credentials",
    riskLevel: "Essential",
    isSecure: false,
    isUnknown: false,
    isFixable: false,
  };

  it("offers no Corriger button for a check that cannot be fixed automatically and leaves it out of Appliquer tout", async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_hardening_checks") return Promise.resolve([...ITEMS, CREDENTIAL_GUARD]);
      if (cmd === "get_admin_status") return Promise.resolve({ isAdmin: true, canElevate: true });
      return Promise.resolve(null);
    });
    renderPage();

    await screen.findByText("Credential Guard");
    expect(screen.getAllByRole("button", { name: "Corriger" })).toHaveLength(2);
    expect(screen.getByText("Configuration manuelle")).toBeInTheDocument();

    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: "Appliquer tout" }));
    });
    const dialog = await screen.findByRole("dialog");
    expect(dialog).toHaveTextContent("2 corrections");
    expect(dialog).not.toHaveTextContent("Credential Guard");
  });

  it("lists Essential before Recommended and does not paint them green", async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_hardening_checks") {
        return Promise.resolve([
          { ...ITEMS[0], id: "R", name: "Item recommandé", riskLevel: "Recommended" },
          { ...ITEMS[1], id: "E", name: "Item essentiel", riskLevel: "Essential" },
        ]);
      }
      if (cmd === "get_admin_status") return Promise.resolve({ isAdmin: true, canElevate: true });
      return Promise.resolve(null);
    });
    renderPage();

    await screen.findByText("Item essentiel");
    const headings = screen.getAllByRole("heading", { level: 2 }).map((h) => h.textContent);
    expect(headings).toEqual(["Essentiel", "Recommandé"]);
    expect(screen.getAllByText("Essentiel")[1].className).not.toMatch(/success/);
    expect(screen.getAllByText("Recommandé")[1].className).not.toMatch(/success/);
  });

  it("opens a single confirmation and applies the fix once when Corriger is double-clicked", async () => {
    renderPage();
    await screen.findByText("Désactiver SMBv1");

    const [fix] = screen.getAllByRole("button", { name: "Corriger" });
    await act(async () => {
      fireEvent.click(fix);
      fireEvent.click(fix);
    });
    await act(async () => {
      fireEvent.click(await screen.findByRole("button", { name: "Confirmer" }));
    });

    await waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
    expect(mockedInvoke.mock.calls.filter(([c]) => c === "apply_hardening_fix")).toHaveLength(1);
    expect(mockedInvoke.mock.calls.filter(([c]) => c === "get_hardening_checks")).toHaveLength(2);
  });
});
