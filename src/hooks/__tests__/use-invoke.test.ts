import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { useInvoke, invalidateCache, getCacheSize, prefetchCommand } from "../use-invoke";
import type { CommandName } from "../../lib/command-names";
import { reportFrontendError } from "../../lib/frontend-log";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("../../lib/frontend-log", () => ({
  reportFrontendError: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

// Les tests exercent la mecanique du hook avec un invoke mocke : les noms de
// commande sont des identifiants arbitraires, cast en CommandName (DX-01).
const cmd = (name: string) => name as CommandName;

beforeEach(() => {
  vi.clearAllMocks();
  invalidateCache();
});

describe("useInvoke", () => {
  it("starts with loading state", () => {
    mockedInvoke.mockReturnValue(new Promise(() => {}));
    const { result } = renderHook(() => useInvoke(cmd("test_command")));
    expect(result.current.isLoading).toBe(true);
    expect(result.current.data).toBeNull();
    expect(result.current.error).toBeNull();
  });

  it("sets data on successful invoke", async () => {
    mockedInvoke.mockResolvedValue({ name: "test" });
    const { result } = renderHook(() =>
      useInvoke<{ name: string }>(cmd("get_data"))
    );

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.data).toEqual({ name: "test" });
    expect(result.current.error).toBeNull();
    expect(mockedInvoke).toHaveBeenCalledWith("get_data", {});
  });

  it("sets error on failed invoke", async () => {
    mockedInvoke.mockRejectedValue("Command failed");
    const { result } = renderHook(() => useInvoke(cmd("bad_command")));

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.data).toBeNull();
    expect(result.current.error).toBe("Command failed");
  });

  it("journalise l'echec d'une commande avec son nom", async () => {
    mockedInvoke.mockRejectedValue({ code: "NotFound", details: "x" });
    const { result } = renderHook(() => useInvoke(cmd("bad_command")));

    await waitFor(() => {
      expect(result.current.error).not.toBeNull();
    });

    expect(reportFrontendError).toHaveBeenCalledWith("commande bad_command", {
      code: "NotFound",
      details: "x",
    });
  });

  it("ne journalise rien quand la commande reussit", async () => {
    mockedInvoke.mockResolvedValue("ok");
    const { result } = renderHook(() => useInvoke(cmd("good_command")));

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(reportFrontendError).not.toHaveBeenCalled();
  });

  it("passes args to invoke", async () => {
    mockedInvoke.mockResolvedValue("ok");
    const { result } = renderHook(() =>
      useInvoke(cmd("cmd"), { id: 42 })
    );

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(mockedInvoke).toHaveBeenCalledWith("cmd", { id: 42 });
  });

  it("refresh triggers re-fetch", async () => {
    mockedInvoke.mockResolvedValueOnce("first").mockResolvedValueOnce("second");
    const { result } = renderHook(() => useInvoke<string>(cmd("cmd")));

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });
    expect(result.current.data).toBe("first");

    await act(async () => {
      result.current.refresh();
    });

    await waitFor(() => {
      expect(result.current.data).toBe("second");
    });
    expect(mockedInvoke).toHaveBeenCalledTimes(2);
  });

  it("refetches when args change", async () => {
    mockedInvoke.mockResolvedValueOnce("v1").mockResolvedValueOnce("v2");
    const { result, rerender } = renderHook(
      ({ id }: { id: number }) => useInvoke<string>(cmd("cmd"), { id }),
      { initialProps: { id: 1 } },
    );

    await waitFor(() => {
      expect(result.current.data).toBe("v1");
    });
    expect(mockedInvoke).toHaveBeenNthCalledWith(1, "cmd", { id: 1 });

    rerender({ id: 2 });

    await waitFor(() => {
      expect(result.current.data).toBe("v2");
    });
    expect(mockedInvoke).toHaveBeenNthCalledWith(2, "cmd", { id: 2 });
    expect(mockedInvoke).toHaveBeenCalledTimes(2);
  });

  it("does not refetch when args object has same content", async () => {
    mockedInvoke.mockResolvedValue("data");
    const { result, rerender } = renderHook(
      ({ id }: { id: number }) => useInvoke<string>(cmd("cmd"), { id }),
      { initialProps: { id: 1 } },
    );

    await waitFor(() => {
      expect(result.current.data).toBe("data");
    });
    const callsAfterMount = mockedInvoke.mock.calls.length;

    rerender({ id: 1 });

    await waitFor(() => {
      expect(mockedInvoke.mock.calls.length).toBe(callsAfterMount);
    });
  });

  it("does not setState after unmount (mountedRef guard)", async () => {
    let resolveInvoke: (value: string) => void = () => {};
    mockedInvoke.mockImplementation(
      () => new Promise<string>((res) => { resolveInvoke = res; }),
    );
    const errorSpy = vi.spyOn(console, "error").mockImplementation(() => {});
    const { result, unmount } = renderHook(() => useInvoke<string>(cmd("slow_cmd")));

    expect(result.current.isLoading).toBe(true);

    unmount();
    resolveInvoke("late_result");
    await new Promise((r) => setTimeout(r, 10));

    const stateUpdateWarning = errorSpy.mock.calls.find((call) =>
      String(call[0]).includes("state update on an unmounted"),
    );
    expect(stateUpdateWarning).toBeUndefined();
    errorSpy.mockRestore();
  });

  it("serves cached value synchronously on remount (no flicker)", async () => {
    mockedInvoke.mockResolvedValue("fresh");
    const first = renderHook(() => useInvoke<string>(cmd("cached_cmd")));
    await waitFor(() => {
      expect(first.result.current.data).toBe("fresh");
    });
    first.unmount();

    let resolveSecond: (v: string) => void = () => {};
    mockedInvoke.mockImplementation(
      () => new Promise<string>((r) => { resolveSecond = r; }),
    );
    const second = renderHook(() => useInvoke<string>(cmd("cached_cmd")));
    expect(second.result.current.isLoading).toBe(false);
    expect(second.result.current.data).toBe("fresh");

    resolveSecond("fresher");
    await waitFor(() => {
      expect(second.result.current.data).toBe("fresher");
    });
  });

  it("invalidateCache() clears all cached entries", async () => {
    mockedInvoke.mockResolvedValue("x");
    const { unmount } = renderHook(() => useInvoke<string>(cmd("a")));
    await waitFor(() => expect(getCacheSize()).toBe(1));
    unmount();
    invalidateCache();
    expect(getCacheSize()).toBe(0);
  });

  it("invalidateCache(command) clears only matching entries", async () => {
    mockedInvoke.mockResolvedValue("v");
    const h1 = renderHook(() => useInvoke<string>(cmd("alpha")));
    await waitFor(() => expect(h1.result.current.data).toBe("v"));
    h1.unmount();
    const h2 = renderHook(() => useInvoke<string>(cmd("beta")));
    await waitFor(() => expect(h2.result.current.data).toBe("v"));
    h2.unmount();
    expect(getCacheSize()).toBe(2);

    invalidateCache("alpha");
    expect(getCacheSize()).toBe(1);
  });

  it("remount after invalidateCache shows loading again", async () => {
    mockedInvoke.mockResolvedValue("v1");
    const first = renderHook(() => useInvoke<string>(cmd("cmd")));
    await waitFor(() => expect(first.result.current.data).toBe("v1"));
    first.unmount();

    invalidateCache("cmd");

    let resolveNew: (v: string) => void = () => {};
    mockedInvoke.mockImplementation(
      () => new Promise<string>((r) => { resolveNew = r; }),
    );
    const second = renderHook(() => useInvoke<string>(cmd("cmd")));
    expect(second.result.current.isLoading).toBe(true);
    expect(second.result.current.data).toBeNull();
    resolveNew("v2");
    await waitFor(() => expect(second.result.current.data).toBe("v2"));
  });

  it("evicts oldest entry beyond MAX_CACHE_SIZE", async () => {
    mockedInvoke.mockResolvedValue("v");
    for (let i = 0; i < 55; i++) {
      prefetchCommand(cmd(`cmd_${i}`));
    }
    await waitFor(() => expect(mockedInvoke).toHaveBeenCalledTimes(55));
    await waitFor(() => expect(getCacheSize()).toBe(50));
  });

  it("keeps stale data when refresh fails but exposes error", async () => {
    mockedInvoke.mockResolvedValueOnce("v1");
    const { result } = renderHook(() => useInvoke<string>(cmd("unstable_cmd")));
    await waitFor(() => expect(result.current.data).toBe("v1"));

    mockedInvoke.mockRejectedValueOnce("network down");
    await act(async () => {
      result.current.refresh();
    });
    await waitFor(() => expect(result.current.error).toBe("network down"));
    expect(result.current.data).toBe("v1");
  });

  it("repasse en chargement quand on relance apres un echec sans donnee", async () => {
    mockedInvoke.mockRejectedValueOnce("wmi down");
    const { result } = renderHook(() => useInvoke<string[]>(cmd("first_load_fails")));
    await waitFor(() => expect(result.current.error).toBe("wmi down"));
    expect(result.current.isLoading).toBe(false);

    mockedInvoke.mockReturnValueOnce(new Promise(() => {}));
    act(() => {
      result.current.refresh();
    });
    expect(result.current.error).toBeNull();
    expect(result.current.data).toBeNull();
    expect(result.current.isLoading).toBe(true);
  });

  it("prefetchCommand populates cache silently", async () => {
    mockedInvoke.mockResolvedValueOnce("prefetched");
    prefetchCommand(cmd("preview_cmd"));
    await waitFor(() => expect(getCacheSize()).toBe(1));
  });

  it("prefetchCommand is a no-op when key already cached", async () => {
    mockedInvoke.mockResolvedValueOnce("first");
    prefetchCommand(cmd("already_there"));
    await waitFor(() => expect(getCacheSize()).toBe(1));
    const initialCalls = mockedInvoke.mock.calls.length;
    prefetchCommand(cmd("already_there"));
    expect(mockedInvoke.mock.calls.length).toBe(initialCalls);
  });

  it("prefetchCommand swallows errors silently", async () => {
    mockedInvoke.mockRejectedValueOnce("boom");
    prefetchCommand(cmd("failing_cmd"));
    await new Promise((r) => setTimeout(r, 10));
    expect(getCacheSize()).toBe(0);
  });

  it("rendered hook sees prefetched cache immediately", async () => {
    mockedInvoke.mockResolvedValueOnce("fresh_from_prefetch");
    prefetchCommand(cmd("warm_cmd"));
    await waitFor(() => expect(getCacheSize()).toBe(1));

    let resolveRefetch: (v: string) => void = () => {};
    mockedInvoke.mockImplementation(
      () => new Promise<string>((r) => { resolveRefetch = r; }),
    );
    const { result } = renderHook(() => useInvoke<string>(cmd("warm_cmd")));
    expect(result.current.isLoading).toBe(false);
    expect(result.current.data).toBe("fresh_from_prefetch");
    resolveRefetch("fresher");
  });

  it("ignores out-of-order responses: last emitted request wins (bug-frontend-01)", async () => {
    mockedInvoke.mockResolvedValueOnce("initial");
    const { result } = renderHook(() => useInvoke<string>(cmd("seq_cmd")));
    await waitFor(() => expect(result.current.data).toBe("initial"));

    let resolveFirst: (v: string) => void = () => {};
    let resolveSecond: (v: string) => void = () => {};
    mockedInvoke
      .mockImplementationOnce(() => new Promise<string>((r) => { resolveFirst = r; }))
      .mockImplementationOnce(() => new Promise<string>((r) => { resolveSecond = r; }));

    await act(async () => {
      result.current.refresh();
      result.current.refresh();
    });

    // Le dernier emis resout en premier -> il gagne.
    await act(async () => {
      resolveSecond("second");
    });
    await waitFor(() => expect(result.current.data).toBe("second"));

    // La reponse obsolete arrive en retard et ne doit PAS ecraser la fraiche.
    await act(async () => {
      resolveFirst("first-late");
      await Promise.resolve();
    });
    expect(result.current.data).toBe("second");
  });

  it("applique le delai propre a la commande : defaut 30 s, long 180 s", async () => {
    vi.useFakeTimers();
    try {
      mockedInvoke.mockReturnValue(new Promise(() => {}));
      const quick = renderHook(() => useInvoke(cmd("get_quick_thing")));
      const long = renderHook(() => useInvoke(cmd("run_threat_scan")));

      await act(async () => {
        await vi.advanceTimersByTimeAsync(31_000);
      });
      expect(quick.result.current.error).toContain("30000");
      expect(long.result.current.isLoading).toBe(true);
      expect(long.result.current.error).toBeNull();

      await act(async () => {
        await vi.advanceTimersByTimeAsync(150_000);
      });
      expect(long.result.current.error).toContain("180000");
    } finally {
      vi.useRealTimers();
    }
  });
});
