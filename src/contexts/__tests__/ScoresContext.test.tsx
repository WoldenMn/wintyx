import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, act } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { ScoresProvider, useScores } from "../ScoresContext";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

function Consumer() {
  const { scores, globalScore, loaded, scoresError } = useScores();
  return (
    <div>
      <span data-testid="loaded">{String(loaded)}</span>
      <span data-testid="dp">{scores.dataProtection}</span>
      <span data-testid="global">{globalScore}</span>
      <span data-testid="error">{scoresError ?? ""}</span>
    </div>
  );
}

function MemoizationProbe({ onRender }: { onRender: (ref: object) => void }) {
  const ctx = useScores();
  onRender(ctx);
  return null;
}

const ALL_SCORES = {
  dataProtection: 80,
  services: 70,
  hardening: 60,
  performance: 90,
  bloatware: 100,
};

beforeEach(() => {
  vi.clearAllMocks();
  mockedInvoke.mockImplementation(async (cmd: string) => {
    if (cmd === "get_all_scores") return { ...ALL_SCORES };
    return null;
  });
});

describe("ScoresContext", () => {
  it("loads all 5 scores via a single get_all_scores invoke on mount (PERF-01)", async () => {
    render(
      <ScoresProvider>
        <Consumer />
      </ScoresProvider>,
    );

    await waitFor(() => {
      expect(screen.getByTestId("loaded")).toHaveTextContent("true");
    });

    expect(mockedInvoke).toHaveBeenCalledWith("get_all_scores");
    // +1 appel fire-and-forget record_score_snapshot apres un refresh reussi.
    expect(mockedInvoke).toHaveBeenCalledWith("record_score_snapshot", expect.objectContaining({ scores: expect.any(Object) }));
    expect(mockedInvoke).toHaveBeenCalledTimes(2);
  });

  it("keys the scores by domain id from the batch payload", async () => {
    render(
      <ScoresProvider>
        <Consumer />
      </ScoresProvider>,
    );

    await waitFor(() => {
      expect(screen.getByTestId("dp")).toHaveTextContent("80");
    });
  });

  it("computes globalScore as rounded average of 5 domains", async () => {
    render(
      <ScoresProvider>
        <Consumer />
      </ScoresProvider>,
    );

    await waitFor(() => {
      expect(screen.getByTestId("global")).toHaveTextContent("80");
    });
  });

  it("calcule le score global sur les seuls domaines lus et n'enregistre pas une journee incomplete", async () => {
    mockedInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_all_scores") return { ...ALL_SCORES, bloatware: null };
      return null;
    });
    render(
      <ScoresProvider>
        <Consumer />
      </ScoresProvider>,
    );
    await waitFor(() => expect(screen.getByTestId("loaded").textContent).toBe("true"));
    expect(screen.getByTestId("global").textContent).toBe("75");
    expect(mockedInvoke.mock.calls.some(([cmd]) => cmd === "record_score_snapshot")).toBe(false);
  });

  it("sets loaded=true and exposes scoresError without crashing when a score command rejects", async () => {
    mockedInvoke.mockRejectedValue("WMI unavailable");

    render(
      <ScoresProvider>
        <Consumer />
      </ScoresProvider>,
    );

    await waitFor(() => {
      expect(screen.getByTestId("loaded")).toHaveTextContent("true");
    });

    // Scores stay at their initial empty value (0), no crash, error exposed
    // au contexte (politique mode degrade v2.12.0 : echec visible, pas avale).
    expect(screen.getByTestId("dp")).toHaveTextContent("0");
    expect(screen.getByTestId("global")).toHaveTextContent("0");
    expect(screen.getByTestId("error")).toHaveTextContent("WMI unavailable");
  });

  it("provides memoized value ref stable across re-renders when scores unchanged", async () => {
    const refs: object[] = [];
    const { rerender } = render(
      <ScoresProvider>
        <MemoizationProbe onRender={(ref) => refs.push(ref)} />
      </ScoresProvider>,
    );

    await waitFor(() => {
      expect(refs.length).toBeGreaterThan(0);
    });

    const firstRef = refs[refs.length - 1];
    await act(async () => {
      rerender(
        <ScoresProvider>
          <MemoizationProbe onRender={(ref) => refs.push(ref)} />
        </ScoresProvider>,
      );
    });

    expect(firstRef).toBe(refs[refs.length - 1]);
  });

  it("concurrent refreshScores awaits the in-flight batch instead of resolving early (bug-frontend-02)", async () => {
    const resolvers: Array<(v: typeof ALL_SCORES) => void> = [];
    mockedInvoke.mockImplementation(
      () => new Promise<typeof ALL_SCORES>((r) => { resolvers.push(r); }),
    );

    let refreshFn: () => Promise<void> = async () => {};
    function Probe({ expose }: { expose: (fn: () => Promise<void>) => void }) {
      expose(useScores().refreshScores);
      return null;
    }
    render(
      <ScoresProvider>
        <Probe expose={(fn) => { refreshFn = fn; }} />
      </ScoresProvider>,
    );

    // L'invoke batch du mount (get_all_scores) est en vol, non resolu.
    await waitFor(() => expect(mockedInvoke).toHaveBeenCalledTimes(1));

    let settled = false;
    await act(async () => {
      const p = refreshFn();
      void p.then(() => { settled = true; });
      await Promise.resolve();
      await Promise.resolve();
    });
    // Sans le fix, refreshScores() resolvait immediatement (scores stale percus).
    expect(settled).toBe(false);
    expect(mockedInvoke).toHaveBeenCalledTimes(1); // pas de second batch

    await act(async () => {
      resolvers.forEach((r) => r({ ...ALL_SCORES }));
      await Promise.resolve();
    });
    expect(settled).toBe(true);
  });
});
