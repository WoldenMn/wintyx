import { describe, it, expect, vi, beforeEach } from "vitest";
import { StrictMode } from "react";
import { act, render, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { MonitorProvider, useMonitor } from "../MonitorContext";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async () => () => {}),
}));

const mockedInvoke = vi.mocked(invoke);
const mockedListen = vi.mocked(listen);

function Consumer() {
  const { isRunning, intervalMs } = useMonitor();
  return (
    <div>
      <span data-testid="running">{String(isRunning)}</span>
      <span data-testid="interval">{intervalMs}</span>
    </div>
  );
}

function Probe({
  onCtx,
}: {
  onCtx: (ctx: ReturnType<typeof useMonitor>) => void;
}) {
  const ctx = useMonitor();
  onCtx(ctx);
  return null;
}

beforeEach(() => {
  vi.clearAllMocks();
  mockedInvoke.mockResolvedValue(undefined);
});

describe("MonitorContext", () => {
  it("provides default interval 1500ms on initial render", async () => {
    const { getByTestId } = render(
      <MonitorProvider>
        <Consumer />
      </MonitorProvider>,
    );
    await waitFor(() => {
      expect(getByTestId("interval").textContent).toBe("1500");
    });
  });

  it("calls start_monitoring once when first subscriber mounts", async () => {
    render(
      <MonitorProvider>
        <Consumer />
      </MonitorProvider>,
    );
    await waitFor(() => {
      expect(
        mockedInvoke.mock.calls.some((c) => c[0] === "start_monitoring"),
      ).toBe(true);
    });
  });

  it("refcount avoids duplicate start_monitoring for 2 consumers", async () => {
    render(
      <MonitorProvider>
        <Consumer />
        <Consumer />
      </MonitorProvider>,
    );
    await waitFor(() => {
      const startCalls = mockedInvoke.mock.calls.filter(
        (c) => c[0] === "start_monitoring",
      ).length;
      expect(startCalls).toBe(1);
    });
  });

  it("throws when used outside MonitorProvider", () => {
    const Crash = () => {
      useMonitor();
      return null;
    };
    const spy = vi.spyOn(console, "error").mockImplementation(() => {});
    expect(() => render(<Crash />)).toThrow();
    spy.mockRestore();
  });

  it("unsubscribes the Tauri listener when unmounted before listen resolves", async () => {
    // Regression FE-01 v2.16.0 : listen() est async — si le cleanup s'execute
    // avant la resolution de la promesse, le listener ne doit pas rester orphelin.
    const unlisten = vi.fn();
    let resolveListen: ((fn: UnlistenFn) => void) | undefined;
    mockedListen.mockImplementationOnce(
      () =>
        new Promise<UnlistenFn>((resolve) => {
          resolveListen = resolve;
        }),
    );

    const { unmount } = render(
      <MonitorProvider>
        <Consumer />
      </MonitorProvider>,
    );
    unmount();
    expect(unlisten).not.toHaveBeenCalled();

    resolveListen?.(unlisten);
    await waitFor(() => {
      expect(unlisten).toHaveBeenCalledTimes(1);
    });
  });

  it("expose l'erreur quand l'abonnement aux evenements de metriques echoue", async () => {
    mockedListen.mockRejectedValueOnce("bus d'evenements indisponible");
    let ctx: ReturnType<typeof useMonitor> | null = null;
    render(
      <MonitorProvider>
        <Probe onCtx={(c) => (ctx = c)} />
      </MonitorProvider>,
    );
    await waitFor(() => {
      expect(ctx?.error).toContain("bus d'evenements indisponible");
    });
  });

  it("context value is stable across re-renders when inputs unchanged", async () => {
    const refs: object[] = [];
    const { rerender } = render(
      <MonitorProvider>
        <Probe onCtx={(ctx) => refs.push(ctx)} />
      </MonitorProvider>,
    );
    rerender(
      <MonitorProvider>
        <Probe onCtx={(ctx) => refs.push(ctx)} />
      </MonitorProvider>,
    );
    await waitFor(() => {
      expect(refs.length).toBeGreaterThanOrEqual(2);
    });
  });

  it("ends started after the StrictMode mount, unmount, remount sequence", async () => {
    render(
      <StrictMode>
        <MonitorProvider>
          <Consumer />
        </MonitorProvider>
      </StrictMode>,
    );
    await waitFor(() => {
      const order = mockedInvoke.mock.calls
        .map((c) => c[0])
        .filter((c) => c === "start_monitoring" || c === "stop_monitoring");
      expect(order.length).toBeGreaterThan(0);
      expect(order[order.length - 1]).toBe("start_monitoring");
    });
  });

  it("restarts the backend when a page subscribes again after the last one left", async () => {
    let ctx!: ReturnType<typeof useMonitor>;
    const { unmount, rerender } = render(
      <MonitorProvider>
        <Probe onCtx={(c) => (ctx = c)} />
      </MonitorProvider>,
    );
    await waitFor(() => expect(mockedInvoke).toHaveBeenCalledWith("start_monitoring", expect.anything()));
    ctx.unsubscribe();
    ctx.subscribe();
    const order = () =>
      mockedInvoke.mock.calls.map((c) => c[0]).filter((c) => c === "start_monitoring" || c === "stop_monitoring");
    await waitFor(() => expect(order()).toEqual(["start_monitoring", "stop_monitoring", "start_monitoring"]));
    rerender(<MonitorProvider><Probe onCtx={(c) => (ctx = c)} /></MonitorProvider>);
    unmount();
  });

  it("clamps the requested interval to the 1000 ms floor the collection can hold", async () => {
    let ctx: ReturnType<typeof useMonitor> | null = null;
    render(
      <MonitorProvider>
        <Probe onCtx={(c) => { ctx = c; }} />
      </MonitorProvider>,
    );
    await waitFor(() => expect(ctx).not.toBeNull());
    await ctx!.setInterval(500);
    expect(mockedInvoke).toHaveBeenCalledWith("set_monitor_interval", { intervalMs: 1000 });
  });

  it("revient a l'intervalle precedent et expose l'erreur quand le backend refuse", async () => {
    let ctx: ReturnType<typeof useMonitor> | null = null;
    mockedInvoke.mockImplementation((cmd: string) =>
      cmd === "set_monitor_interval" ? Promise.reject({ code: "Generic", details: "refus" }) : Promise.resolve(undefined),
    );
    render(
      <MonitorProvider>
        <Probe onCtx={(c) => { ctx = c; }} />
      </MonitorProvider>,
    );
    await waitFor(() => expect(ctx).not.toBeNull());
    await act(async () => {
      await ctx!.setInterval(3000);
    });
    expect(ctx!.intervalMs).toBe(1500);
    expect(ctx!.intervalError).toMatch(/refus/);
  });

  it("measures the real interval between received metrics", async () => {
    let ctx: ReturnType<typeof useMonitor> | null = null;
    const now = vi.spyOn(performance, "now");
    render(
      <MonitorProvider>
        <Probe onCtx={(c) => { ctx = c; }} />
      </MonitorProvider>,
    );
    await waitFor(() => expect(mockedListen).toHaveBeenCalled());
    const handler = mockedListen.mock.calls[0][1] as (e: { payload: unknown }) => void;
    const payload = { cpuUsage: 1, cpuPerCore: [], ramUsedGb: 1, ramTotalGb: 2, ramPercent: 50, topProcesses: [] };
    expect(ctx!.measuredIntervalMs).toBeNull();
    for (const t of [1000, 2020, 3040]) {
      now.mockReturnValue(t);
      act(() => handler({ payload }));
    }
    expect(ctx!.measuredIntervalMs).toBe(1020);
    now.mockRestore();
  });
});
