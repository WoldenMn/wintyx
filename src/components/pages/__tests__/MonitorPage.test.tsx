import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { MonitorPage } from "../MonitorPage";
import { useMonitor } from "../../../contexts/MonitorContext";

vi.mock("../../../contexts/MonitorContext", () => ({
  useMonitor: vi.fn(),
}));

const mockedUseMonitor = vi.mocked(useMonitor);

type MonitorValue = ReturnType<typeof useMonitor>;

function monitor(overrides: Partial<MonitorValue> = {}): MonitorValue {
  return {
    metrics: null,
    cpuHistory: [],
    ramHistory: [],
    isRunning: false,
    error: null,
    intervalMs: 1500,
    measuredIntervalMs: null,
    intervalError: null,
    setInterval: vi.fn(async () => {}),
    pause: vi.fn(async () => {}),
    resume: vi.fn(async () => {}),
    subscribe: vi.fn(),
    unsubscribe: vi.fn(),
    ...overrides,
  };
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe("MonitorPage erreur du monitoring", () => {
  it("affiche l'erreur du monitoring", () => {
    mockedUseMonitor.mockReturnValue(monitor({ error: "start_monitoring a echoue" }));
    render(<MonitorPage />);

    expect(screen.getByRole("alert")).toHaveTextContent("start_monitoring a echoue");
  });

  it("dit que l'intervalle n'a pas change quand le backend l'a refuse", () => {
    mockedUseMonitor.mockReturnValue(monitor({ intervalError: "refus du backend" }));
    render(<MonitorPage />);

    expect(screen.getByRole("alert")).toHaveTextContent("Intervalle non modifié : refus du backend");
  });

  it("relance le monitoring depuis le bouton Reessayer", () => {
    const resume = vi.fn(async () => {});
    mockedUseMonitor.mockReturnValue(monitor({ error: "echec", resume }));
    render(<MonitorPage />);

    fireEvent.click(screen.getByRole("button", { name: /réessayer/i }));
    expect(resume).toHaveBeenCalledTimes(1);
  });

  it("n'affiche aucune alerte sans erreur", () => {
    mockedUseMonitor.mockReturnValue(monitor());
    render(<MonitorPage />);

    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });
});

describe("MonitorPage intervalle", () => {
  it("ne propose pas d'intervalle sous 1 s", () => {
    mockedUseMonitor.mockReturnValue(monitor());
    render(<MonitorPage />);

    expect(screen.getByLabelText("Intervalle")).toHaveAttribute("min", "1000");
  });

  it("affiche l'intervalle réel mesuré", () => {
    mockedUseMonitor.mockReturnValue(monitor({ measuredIntervalMs: 1020 }));
    render(<MonitorPage />);

    expect(screen.getByTestId("measured-interval")).toHaveTextContent("Réel : 1.02 s");
  });
});
