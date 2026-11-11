import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { ProfileEditView } from "../ProfileEditView";
import { ToastProvider } from "../../ui/Toast";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
import { invalidateCache } from "../../../hooks/use-invoke";
const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;

const customProfile = {
  id: "custom_123",
  name: "Mon profil",
  description: "",
  dataProtectionIds: ["dp_0"],
  hardeningIds: [],
  performanceIds: [],
  servicesIds: [],
  bloatwareIds: [],
  startupIds: [],
  isCustom: true,
};

const itemsFixture = {
  dataProtection: [{ id: "dp_0", label: "Telemetrie", category: "Privacy" }],
  hardening: [{ id: "RDP", label: "RDP", category: "Network" }],
  performance: [{ id: "visual_effects", label: "Visual Effects", category: "Display" }],
  services: [{ id: "DiagTrack", label: "Connected User Telemetry", category: "Low" }],
  bloatware: [{ id: "Microsoft.SkypeApp", label: "Skype", category: "Communication" }],
  startup: [{ id: "HKCU\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run|OneDrive", label: "OneDrive", category: "HKCU\\Run" }],
};

function renderEditView() {
  return render(
    <ToastProvider>
      <ProfileEditView
        profileId="custom_123"
        onSave={vi.fn()}
        onCancel={vi.fn()}
      />
    </ToastProvider>,
  );
}

describe("ProfileEditView", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    invalidateCache();
    mockInvoke.mockImplementation((command: string) => {
      if (command === "get_profiles") return Promise.resolve([customProfile]);
      if (command === "get_available_items") return Promise.resolve(itemsFixture);
      return Promise.resolve(null);
    });
  });

  it("renders the profile name in input", async () => {
    renderEditView();
    await waitFor(() => {
      const nameInput = screen.getByLabelText(/Nom/i) as HTMLInputElement;
      expect(nameInput.value).toBe("Mon profil");
    });
  });

  it("save button is disabled when name is empty", async () => {
    renderEditView();
    await waitFor(() => {
      expect(screen.getByLabelText(/Nom/i)).toBeInTheDocument();
    });
    const nameInput = screen.getByLabelText(/Nom/i);
    fireEvent.change(nameInput, { target: { value: "" } });
    expect(screen.getByRole("button", { name: /Enregistrer/i })).toBeDisabled();
  });

  it("save button is disabled when 0 items selected", async () => {
    mockInvoke.mockImplementation((command: string) => {
      if (command === "get_profiles") {
        return Promise.resolve([{ ...customProfile, dataProtectionIds: [] }]);
      }
      if (command === "get_available_items") return Promise.resolve(itemsFixture);
      return Promise.resolve(null);
    });
    renderEditView();
    await waitFor(() => {
      expect(screen.getByRole("button", { name: /Enregistrer/i })).toBeDisabled();
    });
  });

  it("calls onCancel when cancel button clicked", async () => {
    const onCancel = vi.fn();
    render(
      <ToastProvider>
        <ProfileEditView
          profileId="custom_123"
          onSave={vi.fn()}
          onCancel={onCancel}
        />
      </ToastProvider>,
    );
    await waitFor(() => {
      expect(screen.getByRole("button", { name: /Annuler/i })).toBeInTheDocument();
    });
    fireEvent.click(screen.getByRole("button", { name: /Annuler/i }));
    expect(onCancel).toHaveBeenCalled();
  });

  it("renders 6 ProfileItemSelector sections", async () => {
    renderEditView();
    await waitFor(() => {
      expect(screen.getByText(/Protection des données/i)).toBeInTheDocument();
    });
    expect(screen.getByText(/Durcissement/i)).toBeInTheDocument();
    expect(screen.getByText(/Performance/i)).toBeInTheDocument();
    expect(screen.getByText(/Services Windows/i)).toBeInTheDocument();
    expect(screen.getByText(/Bloatware/i)).toBeInTheDocument();
    expect(screen.getByText(/Démarrage/i)).toBeInTheDocument();
  });
});
