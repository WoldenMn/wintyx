import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { ToastProvider } from "../../../components/ui/Toast";
import { ConfirmProvider } from "../../../components/ui/ConfirmDialog";
import { AdminProvider } from "../../../contexts/AdminContext";
import { BackupPage } from "../BackupPage";
import { invalidateCache } from "../../../hooks/use-invoke";
import type { RestorePointOutcome } from "../../../lib/types";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

function mockBackend(outcome: RestorePointOutcome) {
  mockedInvoke.mockImplementation((cmd: string) => {
    if (cmd === "get_registry_backups") return Promise.resolve([]);
    if (cmd === "is_system_protection_enabled") return Promise.resolve(true);
    if (cmd === "get_backup_folder") return Promise.resolve("C:\\Backups");
    if (cmd === "get_admin_status") return Promise.resolve({ isAdmin: true, canElevate: true });
    if (cmd === "create_restore_point") return Promise.resolve(outcome);
    return Promise.resolve(null);
  });
}

function renderPage() {
  return render(
    <AdminProvider>
      <ToastProvider>
        <ConfirmProvider>
          <BackupPage />
        </ConfirmProvider>
      </ToastProvider>
    </AdminProvider>,
  );
}

async function clickCreate() {
  const button = await screen.findByRole("button", { name: /créer un point de restauration/i });
  await waitFor(() => expect(button).toBeEnabled());
  fireEvent.click(button);
}

beforeEach(() => {
  vi.clearAllMocks();
  invalidateCache();
});

describe("BackupPage etat de la protection systeme", () => {
  it("affiche Indéterminée avec la cause quand l'etat n'a pas pu etre lu", async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_registry_backups") return Promise.resolve([]);
      if (cmd === "is_system_protection_enabled") return Promise.reject({ code: "Registry", details: "acces refuse" });
      if (cmd === "get_backup_folder") return Promise.resolve("C:\\Backups");
      if (cmd === "get_admin_status") return Promise.resolve({ isAdmin: true, canElevate: true });
      return Promise.resolve(null);
    });
    renderPage();

    expect(await screen.findByText("Indéterminée")).toBeInTheDocument();
    expect(screen.getByText(/acces refuse/)).toBeInTheDocument();
    expect(screen.queryByText("Désactivée")).not.toBeInTheDocument();
    const create = screen.getByRole("button", { name: /créer un point de restauration/i });
    await waitFor(() => expect(create).toBeEnabled());
  });
});

describe("BackupPage points de restauration", () => {
  function backend(isAdmin: boolean) {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_registry_backups") return Promise.resolve([]);
      if (cmd === "is_system_protection_enabled") return Promise.resolve(true);
      if (cmd === "get_backup_folder") return Promise.resolve("/Backups");
      if (cmd === "get_admin_status") return Promise.resolve({ isAdmin, canElevate: true });
      if (cmd === "get_restore_points") {
        return Promise.resolve([{ description: "Point de test", creationTime: "2026-10-01 09:00:00", sequenceNumber: 3, restoreType: "MODIFY_SETTINGS" }]);
      }
      return Promise.resolve(null);
    });
  }

  it("liste les points de restauration en administrateur", async () => {
    backend(true);
    renderPage();

    expect(await screen.findByText("Point de test")).toBeInTheDocument();
  });

  it("indique les droits requis en non-administrateur", async () => {
    backend(false);
    renderPage();

    expect(await screen.findByText(/Droits administrateur requis pour lister/)).toBeInTheDocument();
  });
});

describe("BackupPage restore point", () => {
  it("shows a success toast when a restore point is really created", async () => {
    mockBackend("created");
    renderPage();
    await clickCreate();

    await waitFor(() => {
      const texts = screen.queryAllByRole("status").map((t) => t.textContent ?? "");
      expect(texts.some((t) => /point de restauration créé/i.test(t))).toBe(true);
    });
  });

  it("shows a warning, not a success, when Windows skipped the restore point", async () => {
    mockBackend("skippedWindowsLimit");
    renderPage();
    await clickCreate();

    await waitFor(() => {
      const texts = screen.queryAllByRole("status").map((t) => t.textContent ?? "");
      expect(texts.some((t) => /non créé.*24 h/i.test(t))).toBe(true);
    });
    const texts = screen.queryAllByRole("status").map((t) => t.textContent ?? "");
    expect(texts.some((t) => /point de restauration créé/i.test(t))).toBe(false);
  });
});
