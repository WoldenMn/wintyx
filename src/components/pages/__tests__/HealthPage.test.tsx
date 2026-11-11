import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { HealthPage } from "../HealthPage";
import { useMonitor } from "../../../contexts/MonitorContext";
import { useInvoke } from "../../../hooks/use-invoke";
import type { SystemInfoData } from "../../../lib/types";

vi.mock("../../../contexts/MonitorContext", () => ({
  useMonitor: vi.fn(),
}));
vi.mock("../../../hooks/use-invoke", () => ({
  useInvoke: vi.fn(),
}));

const mockedUseMonitor = vi.mocked(useMonitor);
const mockedUseInvoke = vi.mocked(useInvoke);

const SYSTEM: SystemInfoData = {
  osName: "Windows 11 Pro",
  cpuName: "Processeur de test",
  totalRamGb: 32,
  usedRamGb: 12,
  diskUsedPercent: 40,
  uptimeSeconds: 3600,
  computerName: "PC-TEST",
  antivirusName: "Microsoft Defender",
  cpuUsage: 10,
  gpuName: "Carte graphique de test",
  cpuCores: 12,
  cpuClockMhz: 4400,
};

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
  mockedUseInvoke.mockReturnValue({ data: SYSTEM, isLoading: false, error: null, refresh: vi.fn() } as never);
});

describe("HealthPage mesures en direct", () => {
  it("affiche l'erreur du monitoring au lieu d'un faux 0 %", () => {
    mockedUseMonitor.mockReturnValue(monitor({ error: "start_monitoring a echoue" }));
    render(<HealthPage />);

    expect(screen.getByRole("alert")).toHaveTextContent("start_monitoring a echoue");
    expect(screen.queryByRole("img", { name: /Utilisation CPU/ })).not.toBeInTheDocument();
    expect(screen.queryByRole("img", { name: /Utilisation RAM/ })).not.toBeInTheDocument();
  });

  it("relance le monitoring depuis le bouton Reessayer", () => {
    const resume = vi.fn(async () => {});
    mockedUseMonitor.mockReturnValue(monitor({ error: "echec", resume }));
    render(<HealthPage />);

    fireEvent.click(screen.getByRole("button", { name: /réessayer/i }));
    expect(resume).toHaveBeenCalledTimes(1);
  });

  it("indique la mesure en cours tant qu'aucune mesure n'est arrivee, sans afficher 0 %", () => {
    mockedUseMonitor.mockReturnValue(monitor());
    render(<HealthPage />);

    expect(screen.getAllByText(/Mesure en cours/).length).toBeGreaterThan(0);
    expect(screen.queryByRole("img", { name: /Utilisation CPU/ })).not.toBeInTheDocument();
  });

  it("affiche les anneaux quand des mesures sont disponibles", () => {
    mockedUseMonitor.mockReturnValue(
      monitor({
        metrics: { cpuUsage: 42, ramPercent: 61, ramUsedGb: 12.3, ramTotalGb: 32 } as MonitorValue["metrics"],
        cpuHistory: [40, 42],
        ramHistory: [60, 61],
      }),
    );
    render(<HealthPage />);

    expect(screen.getByRole("img", { name: "Utilisation CPU : 42%" })).toBeInTheDocument();
    expect(screen.getByRole("img", { name: "Utilisation RAM : 61%" })).toBeInTheDocument();
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });
});
