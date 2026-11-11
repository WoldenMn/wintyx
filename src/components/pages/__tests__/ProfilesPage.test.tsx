import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { ProfilesPage } from "../ProfilesPage";
import { ToastProvider } from "../../ui/Toast";
import { ConfirmProvider } from "../../ui/ConfirmDialog";
import { AdminProvider } from "../../../contexts/AdminContext";
import { ScoresProvider } from "../../../contexts/ScoresContext";
import { invalidateCache } from "../../../hooks/use-invoke";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;

const presetFixture = {
  id: "bureau",
  name: "Bureau",
  description: "Standard preset",
  dataProtectionIds: ["dp_0"],
  hardeningIds: [],
  performanceIds: [],
  isCustom: false,
};

const customFixture = {
  id: "custom_123",
  name: "Mon profil",
  description: "Cloned from Bureau",
  dataProtectionIds: ["dp_0"],
  hardeningIds: [],
  performanceIds: [],
  isCustom: true,
};

function renderPage() {
  return render(
    <AdminProvider>
      <ConfirmProvider>
        <ToastProvider>
          <ScoresProvider>
            <ProfilesPage />
          </ScoresProvider>
        </ToastProvider>
      </ConfirmProvider>
    </AdminProvider>,
  );
}

describe("ProfilesPage", () => {
  beforeEach(() => {
    invalidateCache();
    mockInvoke.mockReset();
    mockInvoke.mockImplementation((command: string) => {
      if (command === "get_profiles") {
        return Promise.resolve([presetFixture, customFixture]);
      }
      if (command === "get_admin_status") {
        return Promise.resolve({ isAdmin: false, canElevate: true });
      }
      if (command.startsWith("get_") && command.endsWith("_score")) {
        return Promise.resolve(80);
      }
      return Promise.resolve(null);
    });
  });

  it("renders presets and customs in separate sections", async () => {
    renderPage();
    await waitFor(() => {
      expect(screen.getByText(/Profils prédéfinis/i)).toBeInTheDocument();
    });
    expect(screen.getByText(/Mes profils/i)).toBeInTheDocument();
  });

  it("displays Custom badge only on custom profiles", async () => {
    renderPage();
    await waitFor(() => {
      expect(screen.getByText(/Mon profil/i)).toBeInTheDocument();
    });
    const customBadges = screen.queryAllByText(/^Personnalisé$/i);
    expect(customBadges.length).toBe(1);
  });

  it("shows clone button above customs section", async () => {
    renderPage();
    await waitFor(() => {
      expect(screen.getByRole("button", { name: /Cloner un preset/i })).toBeInTheDocument();
    });
  });

  it("renders edit and delete buttons only on custom cards", async () => {
    renderPage();
    await waitFor(() => {
      expect(screen.getByText(/Mon profil/i)).toBeInTheDocument();
    });
    const editButtons = screen.queryAllByRole("button", { name: /Éditer/i });
    expect(editButtons.length).toBe(1);
    const deleteButtons = screen.queryAllByRole("button", { name: /Supprimer/i });
    expect(deleteButtons.length).toBe(1);
  });

  it("opens clone dialog when clone button clicked", async () => {
    renderPage();
    await waitFor(() => {
      expect(screen.getByRole("button", { name: /Cloner un preset/i })).toBeInTheDocument();
    });
    fireEvent.click(screen.getByRole("button", { name: /Cloner un preset/i }));
    expect(screen.getByRole("dialog")).toBeInTheDocument();
  });

  it("shows empty state when no customs available", async () => {
    mockInvoke.mockImplementation((command: string) => {
      if (command === "get_profiles") return Promise.resolve([presetFixture]);
      if (command === "get_admin_status") {
        return Promise.resolve({ isAdmin: false, canElevate: true });
      }
      if (command.startsWith("get_") && command.endsWith("_score")) {
        return Promise.resolve(80);
      }
      return Promise.resolve(null);
    });
    renderPage();
    await waitFor(() => {
      expect(screen.getByText(/Aucun profil personnalisé/i)).toBeInTheDocument();
    });
  });
});
