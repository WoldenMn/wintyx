import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, act } from "@testing-library/react";
import Sidebar from "../Sidebar";
import { invoke } from "@tauri-apps/api/core";
import { useScores } from "../../../contexts/ScoresContext";
import { notifySuspiciousTasksChanged } from "../../../lib/suspicious-tasks";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(0),
}));
vi.mock("../../../contexts/ScoresContext", () => ({
  useScores: vi.fn(),
}));
vi.mock("../../ui/ThemeSwitcher", () => ({
  ThemeSwitcher: () => null,
}));

const mockedUseScores = vi.mocked(useScores);
const mockedInvoke = vi.mocked(invoke);

type ScoresValue = ReturnType<typeof useScores>;

function scoresValue(overrides: Partial<ScoresValue> = {}): ScoresValue {
  return {
    scores: {
      dataProtection: 100,
      services: 100,
      hardening: 100,
      performance: 100,
      bloatware: 100,
    },
    globalScore: 100,
    loaded: true,
    measured: true,
    refreshScores: vi.fn(async () => {}),
    scoresError: null,
    ...overrides,
  };
}

function renderSidebar() {
  return render(<Sidebar currentPage="dashboard" onNavigate={() => {}} />);
}

beforeEach(() => {
  vi.clearAllMocks();
  mockedUseScores.mockReturnValue(scoresValue());
});

describe("Sidebar badges d'etat (UX-04)", () => {
  it("affiche un badge sur Durcissement quand le score passe sous le seuil", () => {
    mockedUseScores.mockReturnValue(
      scoresValue({ scores: { ...scoresValue().scores, hardening: 40 } }),
    );
    renderSidebar();
    expect(screen.getByText("Score durcissement faible")).toBeInTheDocument();
    expect(screen.getByText("!")).toBeInTheDocument();
  });

  it("affiche un badge sur Services quand le score passe sous le seuil", () => {
    mockedUseScores.mockReturnValue(
      scoresValue({ scores: { ...scoresValue().scores, services: 20 } }),
    );
    renderSidebar();
    expect(screen.getByText("Score services faible")).toBeInTheDocument();
  });

  it("n'affiche aucun badge quand les scores sont au-dessus des seuils", () => {
    renderSidebar();
    expect(screen.queryByText("Score durcissement faible")).not.toBeInTheDocument();
    expect(screen.queryByText("Score services faible")).not.toBeInTheDocument();
  });

  it("n'affiche aucun badge exactement au seuil (strictement inferieur)", () => {
    mockedUseScores.mockReturnValue(
      scoresValue({ scores: { ...scoresValue().scores, hardening: 50, services: 30 } }),
    );
    renderSidebar();
    expect(screen.queryByText("!")).not.toBeInTheDocument();
  });

  it("n'affiche aucun badge tant que les scores ne sont pas charges", () => {
    mockedUseScores.mockReturnValue(
      scoresValue({
        loaded: false,
        scores: { ...scoresValue().scores, hardening: 0, services: 0 },
      }),
    );
    renderSidebar();
    expect(screen.queryByText("!")).not.toBeInTheDocument();
  });

  it("n'affiche aucun badge quand le dernier refresh des scores a echoue", () => {
    mockedUseScores.mockReturnValue(
      scoresValue({
        scoresError: "WMI indisponible",
        scores: { ...scoresValue().scores, hardening: 0, services: 0 },
      }),
    );
    renderSidebar();
    expect(screen.queryByText("!")).not.toBeInTheDocument();
  });
});

function countCalls() {
  return mockedInvoke.mock.calls.filter(([cmd]) => cmd === "get_suspicious_task_count").length;
}

describe("Sidebar rafraichissement des taches suspectes", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
    mockedInvoke.mockResolvedValue(0);
  });

  it("n'emet pas un nouvel appel tant que le precedent n'a pas repondu", async () => {
    let release: (n: number) => void = () => {};
    mockedInvoke.mockImplementation(
      () => new Promise<number>((resolve) => { release = resolve; }),
    );
    renderSidebar();
    expect(countCalls()).toBe(1);

    await act(async () => {
      await vi.advanceTimersByTimeAsync(180_000);
    });
    expect(countCalls()).toBe(1);

    await act(async () => {
      release(2);
    });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(60_000);
    });
    expect(countCalls()).toBe(2);
  });

  it("ne sonde pas les taches fenetre masquee et reprend a son retour", async () => {
    mockedInvoke.mockResolvedValue(0);
    const hidden = vi.spyOn(document, "hidden", "get").mockReturnValue(true);
    renderSidebar();
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(countCalls()).toBe(1);

    await act(async () => {
      await vi.advanceTimersByTimeAsync(180_000);
    });
    expect(countCalls()).toBe(1);

    hidden.mockReturnValue(false);
    await act(async () => {
      document.dispatchEvent(new Event("visibilitychange"));
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(countCalls()).toBe(2);
    hidden.mockRestore();
  });

  it("se rafraichit des qu'une action sur les taches le demande", async () => {
    mockedInvoke.mockResolvedValue(3);
    renderSidebar();
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(countCalls()).toBe(1);
    expect(screen.getByText("3")).toBeInTheDocument();

    mockedInvoke.mockResolvedValue(1);
    await act(async () => {
      notifySuspiciousTasksChanged();
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(countCalls()).toBe(2);
    expect(screen.getByText("1")).toBeInTheDocument();
  });

  it("rejoue la demande de rafraichissement recue pendant un appel en cours", async () => {
    let release: (n: number) => void = () => {};
    mockedInvoke.mockImplementation(
      () => new Promise<number>((resolve) => { release = resolve; }),
    );
    renderSidebar();
    expect(countCalls()).toBe(1);

    await act(async () => {
      notifySuspiciousTasksChanged();
    });
    expect(countCalls()).toBe(1);

    mockedInvoke.mockResolvedValue(0);
    await act(async () => {
      release(5);
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(countCalls()).toBe(2);
  });
});

describe("Sidebar badge des taches suspectes", () => {
  it("signale une verification impossible au lieu d'un compteur a zero quand la commande echoue", async () => {
    mockedInvoke.mockRejectedValue("Get-ScheduledTask a echoue");
    renderSidebar();
    expect(await screen.findByText("Tâches suspectes : vérification impossible")).toBeInTheDocument();
    mockedInvoke.mockResolvedValue(0);
  });

  it("affiche le nombre de taches suspectes quand la commande reussit", async () => {
    mockedInvoke.mockResolvedValue(3);
    renderSidebar();
    expect(await screen.findByText("3")).toBeInTheDocument();
    mockedInvoke.mockResolvedValue(0);
  });
});
