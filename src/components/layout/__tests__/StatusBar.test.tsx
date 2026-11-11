import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, act, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AdminProvider } from "../../../contexts/AdminContext";
import { ToastProvider } from "../../ui/Toast";
import StatusBar from "../StatusBar";
import type { AdminStatus } from "../../../lib/types";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

function mockStatus(status: AdminStatus) {
  mockedInvoke.mockImplementation(async (cmd: string) => {
    if (cmd === "get_admin_status") return status;
    return null;
  });
}

function Wrapper({ children }: { children: ReactNode }) {
  return (
    <ToastProvider>
      <AdminProvider>{children}</AdminProvider>
    </ToastProvider>
  );
}

beforeEach(() => {
  vi.clearAllMocks();
  localStorage.clear();
  vi.useFakeTimers({ shouldAdvanceTime: true });
});

afterEach(() => {
  vi.useRealTimers();
});

describe("StatusBar", () => {
  it("renders health score and status message", () => {
    mockStatus({ isAdmin: true, canElevate: true });
    render(
      <Wrapper>
        <StatusBar healthScore={72} statusMessage="Test" />
      </Wrapper>,
    );
    expect(screen.getByText("72/100")).toBeInTheDocument();
    expect(screen.getByText("Test")).toBeInTheDocument();
  });

  it("affiche un etat neutre, jamais 0/100 en rouge, tant que les scores ne sont pas charges", () => {
    mockStatus({ isAdmin: true, canElevate: true });
    render(
      <Wrapper>
        <StatusBar healthScore={null} statusMessage="Scan initial..." />
      </Wrapper>,
    );
    const score = screen.getByTestId("status-score");
    expect(score).toHaveTextContent("Analyse…");
    expect(score.className).not.toContain("text-danger");
    expect(screen.queryByText(/0\/100/)).not.toBeInTheDocument();
  });

  it("affiche inconnu en cas d'erreur de scores, meme avec un score a 0", () => {
    mockStatus({ isAdmin: true, canElevate: true });
    render(
      <Wrapper>
        <StatusBar healthScore={0} scoreError statusMessage="Test" />
      </Wrapper>,
    );
    const score = screen.getByTestId("status-score");
    expect(score).toHaveTextContent("inconnu");
    expect(score.className).not.toContain("text-danger");
    expect(screen.queryByText(/0\/100/)).not.toBeInTheDocument();
  });

  it("garde le rouge pour un vrai score bas mesure", () => {
    mockStatus({ isAdmin: true, canElevate: true });
    render(
      <Wrapper>
        <StatusBar healthScore={0} statusMessage="Test" />
      </Wrapper>,
    );
    const score = screen.getByTestId("status-score");
    expect(score).toHaveTextContent("0/100");
    expect(score.className).toContain("text-danger");
  });

  it("shows Ctrl+K hint on first mount when localStorage is empty", () => {
    mockStatus({ isAdmin: true, canElevate: true });
    render(
      <Wrapper>
        <StatusBar />
      </Wrapper>,
    );
    expect(screen.getByText(/palette de commandes/i)).toBeInTheDocument();
    expect(screen.getByText("Ctrl+K")).toBeInTheDocument();
  });

  it("does NOT show Ctrl+K hint when flag is already set", () => {
    localStorage.setItem("wintyx-hint-cmdk-shown", "1");
    mockStatus({ isAdmin: true, canElevate: true });
    render(
      <Wrapper>
        <StatusBar />
      </Wrapper>,
    );
    expect(screen.queryByText(/palette de commandes/i)).toBeNull();
  });

  it("dismisses hint after 8s and persists flag", () => {
    mockStatus({ isAdmin: true, canElevate: true });
    render(
      <Wrapper>
        <StatusBar />
      </Wrapper>,
    );
    expect(screen.getByText(/palette de commandes/i)).toBeInTheDocument();
    act(() => {
      vi.advanceTimersByTime(8100);
    });
    expect(screen.queryByText(/palette de commandes/i)).toBeNull();
    expect(localStorage.getItem("wintyx-hint-cmdk-shown")).toBe("1");
  });

  it("dismisses hint on X click and persists flag", () => {
    mockStatus({ isAdmin: true, canElevate: true });
    render(
      <Wrapper>
        <StatusBar />
      </Wrapper>,
    );
    const dismiss = screen.getByRole("button", { name: /fermer l'astuce/i });
    act(() => {
      dismiss.click();
    });
    expect(screen.queryByText(/palette de commandes/i)).toBeNull();
    expect(localStorage.getItem("wintyx-hint-cmdk-shown")).toBe("1");
  });

  it("shows admin banner when not admin (after load)", async () => {
    mockStatus({ isAdmin: false, canElevate: true });
    render(
      <Wrapper>
        <StatusBar />
      </Wrapper>,
    );
    await waitFor(() => {
      expect(screen.getByText(/Mode non-admin/i)).toBeInTheDocument();
    });
    expect(
      screen.getByRole("button", { name: /Relancer Wintyx/i }),
    ).toBeInTheDocument();
  });

  it("hides admin banner when user is admin", async () => {
    mockStatus({ isAdmin: true, canElevate: true });
    render(
      <Wrapper>
        <StatusBar />
      </Wrapper>,
    );
    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith("get_admin_status");
    });
    expect(screen.queryByText(/Mode non-admin/i)).toBeNull();
  });
});
