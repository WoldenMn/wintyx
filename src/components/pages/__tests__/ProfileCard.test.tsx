import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { ProfileCard } from "../ProfileCard";
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

const profileV270 = {
  id: "custom_1",
  name: "Test",
  description: "Desc",
  dataProtectionIds: ["dp_0", "dp_1"],
  hardeningIds: ["RDP"],
  performanceIds: [],
  servicesIds: ["DiagTrack", "WSearch"],
  bloatwareIds: ["Microsoft.SkypeApp"],
  startupIds: [],
  isCustom: true,
};

const profileV260Compat = {
  id: "custom_2",
  name: "Old",
  description: "",
  dataProtectionIds: ["dp_0"],
  hardeningIds: [],
  performanceIds: [],
  isCustom: true,
} as unknown as typeof profileV270;

function renderCard(profile: typeof profileV270) {
  return render(
    <AdminProvider>
      <ConfirmProvider>
        <ToastProvider>
          <ScoresProvider>
            <ProfileCard profile={profile} onApplied={() => {}} />
          </ScoresProvider>
        </ToastProvider>
      </ConfirmProvider>
    </AdminProvider>,
  );
}

describe("ProfileCard", () => {
  beforeEach(() => {
    invalidateCache();
    mockInvoke.mockReset();
    mockInvoke.mockImplementation((command: string) => {
      if (command === "get_admin_status") {
        return Promise.resolve({ isAdmin: false, canElevate: true });
      }
      if (command.startsWith("get_") && command.endsWith("_score")) {
        return Promise.resolve(80);
      }
      return Promise.resolve(null);
    });
  });

  it("displays 6 badges with correct counts (v2.7.0 profile)", () => {
    renderCard(profileV270);
    expect(screen.getByText(/2 protections/i)).toBeInTheDocument();
    expect(screen.getByText(/1 durcissements/i)).toBeInTheDocument();
    expect(screen.getByText(/0 optimisations/i)).toBeInTheDocument();
    expect(screen.getByText(/2 services/i)).toBeInTheDocument();
    expect(screen.getByText(/1 bloatwares/i)).toBeInTheDocument();
    expect(screen.getByText(/0 démarrage/i)).toBeInTheDocument();
  });

  it("handles undefined services/bloatware/startup gracefully (v2.6.0 compat)", () => {
    renderCard(profileV260Compat);
    expect(screen.getByText(/0 services/i)).toBeInTheDocument();
    expect(screen.getByText(/0 bloatwares/i)).toBeInTheDocument();
    expect(screen.getByText(/0 démarrage/i)).toBeInTheDocument();
  });
});
