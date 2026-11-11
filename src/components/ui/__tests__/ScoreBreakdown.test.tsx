import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { ScoreBreakdown } from "../ScoreBreakdown";
import type {
  DataProtectionItem,
  HardeningItem,
  ServiceItem,
} from "../../../lib/types";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

const DP_ITEMS: DataProtectionItem[] = [
  {
    id: "dp_1",
    name: "Telemetrie DiagTrack",
    description: "Service de telemetrie",
    category: "Telemetrie",
    isProtected: false,
    recommended: true,
  },
  {
    id: "dp_2",
    name: "Smartscreen",
    description: "Filtrage URL",
    category: "Securite",
    isProtected: true,
    recommended: true,
  },
];

const HARDENING_ITEMS: HardeningItem[] = [
  {
    id: "h_1",
    category: "Network",
    name: "SMBv1",
    description: "Protocole SMB v1",
    riskLevel: "Essential",
    isSecure: false,
    isUnknown: false,
    isFixable: true,
  },
];

const SERVICE_ITEMS: ServiceItem[] = [
  {
    displayName: "DiagTrack",
    name: "diagtrack",
    risk: "High",
    status: "Running",
    description: "Telemetry service",
    startType: "Automatique",
  },
];

beforeEach(() => {
  vi.clearAllMocks();
});

describe("ScoreBreakdown", () => {
  it("renders nothing when domain is null", () => {
    const { container } = render(
      <ScoreBreakdown domain={null} onClose={() => {}} />,
    );
    expect(container.querySelector('[role="dialog"]')).toBeNull();
    expect(mockedInvoke).not.toHaveBeenCalled();
  });

  it("fetches items when a domain is provided", async () => {
    mockedInvoke.mockResolvedValueOnce(DP_ITEMS);
    render(<ScoreBreakdown domain="dataProtection" onClose={() => {}} />);
    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith("get_data_protection_items");
    });
  });

  it("displays items grouped by category with ok ratio", async () => {
    mockedInvoke.mockResolvedValueOnce(DP_ITEMS);
    render(<ScoreBreakdown domain="dataProtection" onClose={() => {}} />);
    await waitFor(() => {
      expect(screen.getByText("Telemetrie DiagTrack")).toBeInTheDocument();
    });
    expect(screen.getByText("Smartscreen")).toBeInTheDocument();
    expect(screen.getByText(/1\/2 items au bon état/i)).toBeInTheDocument();
    expect(screen.getByText("Non protégé")).toBeInTheDocument();
    expect(screen.getByText("Protégé")).toBeInTheDocument();
  });

  it("calls onClose on Escape", async () => {
    mockedInvoke.mockResolvedValueOnce(DP_ITEMS);
    const onClose = vi.fn();
    render(<ScoreBreakdown domain="dataProtection" onClose={onClose} />);
    await waitFor(() => {
      expect(screen.getByText("Telemetrie DiagTrack")).toBeInTheDocument();
    });
    fireEvent.keyDown(window, { key: "Escape" });
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("calls onClose when close button is clicked", async () => {
    mockedInvoke.mockResolvedValueOnce(DP_ITEMS);
    const onClose = vi.fn();
    render(<ScoreBreakdown domain="dataProtection" onClose={onClose} />);
    await waitFor(() => {
      expect(screen.getByText("Telemetrie DiagTrack")).toBeInTheDocument();
    });
    fireEvent.click(screen.getByRole("button", { name: /fermer/i }));
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("dialog has proper aria attributes", async () => {
    mockedInvoke.mockResolvedValueOnce(HARDENING_ITEMS);
    render(<ScoreBreakdown domain="hardening" onClose={() => {}} />);
    await waitFor(() => {
      expect(screen.getByRole("dialog")).toBeInTheDocument();
    });
    const dialog = screen.getByRole("dialog");
    expect(dialog.getAttribute("aria-modal")).toBe("true");
    expect(dialog.getAttribute("aria-labelledby")).toBe("breakdown-title");
  });

  it("marks running High-risk services as non-ok", async () => {
    mockedInvoke.mockResolvedValueOnce(SERVICE_ITEMS);
    render(<ScoreBreakdown domain="services" onClose={() => {}} />);
    await waitFor(() => {
      expect(screen.getByText("DiagTrack")).toBeInTheDocument();
    });
    expect(screen.getByText("Actif")).toBeInTheDocument();
    expect(screen.getByText(/0\/1 items au bon état/i)).toBeInTheDocument();
  });

  it("shows error when invoke rejects", async () => {
    mockedInvoke.mockRejectedValueOnce(new Error("WMI down"));
    render(<ScoreBreakdown domain="performance" onClose={() => {}} />);
    await waitFor(() => {
      expect(screen.getByText(/Erreur de chargement/i)).toBeInTheDocument();
    });
    expect(screen.getByRole("alert").textContent).toContain("WMI down");
  });

  it("decrit une erreur de commande au lieu d'afficher [object Object]", async () => {
    mockedInvoke.mockRejectedValueOnce({ code: "WmiUnavailable", details: "Winmgmt arrete" });
    render(<ScoreBreakdown domain="performance" onClose={() => {}} />);
    const alert = await screen.findByRole("alert");
    expect(alert.textContent).not.toContain("[object Object]");
    expect(alert.textContent).toContain("Winmgmt arrete");
  });

  it("propose Reessayer apres un echec et recharge le detail", async () => {
    let loads = 0;
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd !== "get_data_protection_items") return Promise.resolve(undefined);
      loads += 1;
      return loads === 1
        ? Promise.reject({ code: "Generic", details: "echec reseau" })
        : Promise.resolve(DP_ITEMS);
    });
    render(<ScoreBreakdown domain="dataProtection" onClose={() => {}} />);
    await screen.findByRole("alert");

    fireEvent.click(screen.getByRole("button", { name: /réessayer/i }));

    expect(await screen.findByText("Telemetrie DiagTrack")).toBeInTheDocument();
    expect(screen.queryByRole("alert")).toBeNull();
    expect(mockedInvoke.mock.calls.filter(([c]) => c === "get_data_protection_items")).toHaveLength(2);
  });

  it("garde le focus dans le dialogue quand le parent re-rend avec un nouvel onClose", async () => {
    mockedInvoke.mockResolvedValue(DP_ITEMS);
    const outside = document.createElement("button");
    document.body.appendChild(outside);
    outside.focus();
    const { rerender } = render(<ScoreBreakdown domain="dataProtection" onClose={() => {}} />);
    await screen.findByText("Telemetrie DiagTrack");
    const closeButton = screen.getByRole("button", { name: /fermer/i });
    closeButton.focus();
    expect(document.activeElement).toBe(closeButton);

    rerender(<ScoreBreakdown domain="dataProtection" onClose={() => {}} />);

    expect(document.activeElement).toBe(closeButton);
    outside.remove();
  });

  it("rend le focus a l'element qui a ouvert le dialogue quand il se ferme", async () => {
    mockedInvoke.mockResolvedValue(DP_ITEMS);
    const opener = document.createElement("button");
    document.body.appendChild(opener);
    opener.focus();
    const { rerender } = render(<ScoreBreakdown domain="dataProtection" onClose={() => {}} />);
    await screen.findByText("Telemetrie DiagTrack");
    expect(document.activeElement).not.toBe(opener);

    rerender(<ScoreBreakdown domain={null} onClose={() => {}} />);

    expect(document.activeElement).toBe(opener);
    opener.remove();
  });

  it("appelle le dernier onClose fourni, pas celui du premier rendu", async () => {
    mockedInvoke.mockResolvedValue(DP_ITEMS);
    const first = vi.fn();
    const second = vi.fn();
    const { rerender } = render(<ScoreBreakdown domain="dataProtection" onClose={first} />);
    await screen.findByText("Telemetrie DiagTrack");
    rerender(<ScoreBreakdown domain="dataProtection" onClose={second} />);
    fireEvent.keyDown(window, { key: "Escape" });
    expect(first).not.toHaveBeenCalled();
    expect(second).toHaveBeenCalledTimes(1);
  });

  it("cancels in-flight fetch when domain changes rapidly", async () => {
    // Scenario : user clique sur domain1 puis switch rapidement sur domain2
    // avant que le fetch du premier ne resolve. Le premier setItems doit etre
    // annule pour ne pas ecraser les items du second.
    let resolveFirst: ((value: DataProtectionItem[]) => void) | null = null;
    mockedInvoke.mockImplementationOnce(
      () => new Promise<DataProtectionItem[]>((r) => { resolveFirst = r; }),
    );
    mockedInvoke.mockResolvedValueOnce([SERVICE_ITEMS[0]]);

    const { rerender } = render(
      <ScoreBreakdown domain="dataProtection" onClose={() => {}} />,
    );
    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith("get_data_protection_items");
    });

    // Switch avant que le fetch dataProtection ne resolve
    rerender(<ScoreBreakdown domain="services" onClose={() => {}} />);
    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith("get_services");
    });

    // Resoudre le premier fetch APRES le switch
    resolveFirst!(DP_ITEMS);
    await new Promise((r) => setTimeout(r, 10));

    // Le DOM ne doit PAS afficher les items du 1er domaine
    expect(screen.queryByText("Telemetrie DiagTrack")).toBeNull();
    // Doit afficher ceux du 2e domaine
    expect(screen.getByText("DiagTrack")).toBeInTheDocument();
  });
});
